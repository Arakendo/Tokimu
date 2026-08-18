//! Corpus-private live realization of final Doom source-occurrence support.
//!
//! Final ordered wall declarations are combined with reconstructed plane
//! triangles clipped by exact plane key, source sector, and retained source
//! cells. Every result is ordinary geometry before it reaches the renderer.

use std::collections::BTreeMap;

use doom_map_provider::DoomMapCore;
use tokimu::PlatformResult;

use super::{AppliedRenderStrategy, CandidateSelection, SceneInput};
use crate::presentation::{
    prepare_ordered_occurrence_submission, prepare_plane_cell_geometry_support_shadow,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SourceOccurrenceSupportedPreparation {
    pub(crate) opaque_draws: Vec<crate::StaticDrawPlanEntry>,
    pub(crate) cutout_draws: Vec<crate::StaticDrawPlanEntry>,
    pub(crate) report: String,
}

pub(crate) fn prepare(
    source: &SceneInput,
    runtime_map: &DoomMapCore,
    viewer: [i16; 2],
    heading: f64,
    eye_height: i16,
) -> PlatformResult<SourceOccurrenceSupportedPreparation> {
    let cutout_materials = source
        .cutout_uploads
        .iter()
        .map(|upload| (upload.source_name.clone(), upload.material))
        .collect::<BTreeMap<_, _>>();
    let ordered = prepare_ordered_occurrence_submission(
        runtime_map,
        viewer,
        heading,
        eye_height,
        &source.door_geometry_source.wall_extents,
        &source.door_geometry_source.wall_materials,
        &cutout_materials,
        &source.opaque_uploads,
    )
    .map_err(std::io::Error::other)?;
    ordered
        .verify_conservation()
        .map_err(std::io::Error::other)?;
    if ordered.walls.unresolved_fail_open != 0 {
        return Err(format!(
            "source-occurrence-supported wall preparation is unresolved: {}",
            ordered.walls.unresolved_fail_open,
        )
        .into());
    }

    let planes = prepare_plane_cell_geometry_support_shadow(
        runtime_map,
        viewer,
        heading,
        eye_height,
        &source.door_geometry_source.wall_extents,
        &source.opaque_uploads,
    )?;
    planes
        .verify_conservation()
        .map_err(std::io::Error::other)?;
    if planes.unresolved_source_surfaces != 0 || planes.unresolved_fragments != 0 {
        return Err(format!(
            "source-occurrence-supported plane preparation is unresolved: surfaces={}, fragments={}",
            planes.unresolved_source_surfaces, planes.unresolved_fragments,
        )
        .into());
    }

    let opaque_walls = ordered
        .walls
        .prepared_declarations
        .iter()
        .filter(|declaration| !declaration.cutout)
        .map(|declaration| declaration.draw.clone());
    let cutout_draws = ordered
        .walls
        .prepared_declarations
        .iter()
        .filter(|declaration| declaration.cutout)
        .map(|declaration| declaration.draw.clone())
        .collect::<Vec<_>>();
    let opaque_draws = opaque_walls
        .chain(planes.draws.iter().cloned())
        .collect::<Vec<_>>();

    let expected_opaque = ordered.walls.lowered_opaque_meshes + planes.draws.len();
    if opaque_draws.len() != expected_opaque
        || cutout_draws.len() != ordered.walls.lowered_cutout_meshes
    {
        return Err(format!(
            "source-occurrence-supported declaration conservation failed: opaque={}/{expected_opaque}, cutout={}/{}",
            opaque_draws.len(),
            cutout_draws.len(),
            ordered.walls.lowered_cutout_meshes,
        )
        .into());
    }
    let report = format!(
        "walls=[{}]; planes=[{}]; output=[opaque:{},cutout:{}]; conservation=balanced; renderer-vocabulary=ordinary-declarations-only",
        ordered.walls.report(),
        planes.report(),
        opaque_draws.len(),
        cutout_draws.len(),
    );
    Ok(SourceOccurrenceSupportedPreparation {
        opaque_draws,
        cutout_draws,
        report,
    })
}

pub(super) fn apply(scene: &mut SceneInput) -> PlatformResult<AppliedRenderStrategy> {
    let prepared = prepare(
        scene,
        &scene.door_geometry_source.map,
        scene.spawn_observer.source_position,
        f64::from(scene.spawn_observer.source_angle).to_radians(),
        scene.spawn_observer.position.y as i16,
    )?;
    eprintln!(
        "E1M1 source-occurrence-supported initial preparation: {}",
        prepared.report,
    );
    scene.opaque_draws = prepared.opaque_draws;
    scene.cutout_draws = prepared.cutout_draws;
    Ok(AppliedRenderStrategy {
        candidate_selection: CandidateSelection::FullSubmission,
        ordered_coverage_prepared: false,
        source_covered_domain_filter: false,
        source_occurrence_support_filter: true,
        fixed_reconstruction_camera: false,
    })
}
