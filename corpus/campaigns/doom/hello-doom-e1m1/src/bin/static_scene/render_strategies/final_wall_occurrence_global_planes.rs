//! Corpus-private final-wall-occurrence A/B over untouched global planes.

use std::collections::BTreeMap;

use doom_map_provider::DoomMapCore;
use tokimu::PlatformResult;

use super::{AppliedRenderStrategy, CandidateSelection, SceneInput};
use crate::presentation::prepare_ordered_occurrence_submission;
use crate::StaticDrawSource;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FinalWallOccurrenceGlobalPlanesPreparation {
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
) -> PlatformResult<FinalWallOccurrenceGlobalPlanesPreparation> {
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
        return Ok(FinalWallOccurrenceGlobalPlanesPreparation {
            opaque_draws: source.opaque_draws.clone(),
            cutout_draws: source.cutout_draws.clone(),
            report: format!(
                "wall-preparation=[{}]; disposition=unresolved-fail-open-global-full; output=[opaque:{},cutout:{}]; planes=global-full-unchanged; conservation=balanced; renderer-vocabulary=ordinary-declarations-only",
                ordered.walls.report(),
                source.opaque_draws.len(),
                source.cutout_draws.len(),
            ),
        });
    }

    let global_planes = source
        .opaque_draws
        .iter()
        .filter(|draw| matches!(draw.source, StaticDrawSource::Flat { .. }))
        .cloned()
        .collect::<Vec<_>>();
    let opaque_walls = ordered
        .walls
        .prepared_declarations
        .iter()
        .filter(|declaration| !declaration.cutout)
        .map(|declaration| declaration.draw.clone())
        .collect::<Vec<_>>();
    let cutout_draws = ordered
        .walls
        .prepared_declarations
        .iter()
        .filter(|declaration| declaration.cutout)
        .map(|declaration| declaration.draw.clone())
        .collect::<Vec<_>>();
    let mut opaque_draws = Vec::with_capacity(global_planes.len() + opaque_walls.len());
    opaque_draws.extend(global_planes.iter().cloned());
    opaque_draws.extend(opaque_walls.iter().cloned());

    if global_planes.len()
        != source
            .opaque_draws
            .iter()
            .filter(|draw| matches!(draw.source, StaticDrawSource::Flat { .. }))
            .count()
        || opaque_walls.len() != ordered.walls.lowered_opaque_meshes
        || cutout_draws.len() != ordered.walls.lowered_cutout_meshes
        || opaque_draws.len() != global_planes.len() + opaque_walls.len()
    {
        return Err("final-wall-occurrence global-plane conservation failed".into());
    }

    let report = format!(
        "wall-preparation=[{}]; global-planes={}; ordered-opaque-walls={}; ordered-cutout-walls={}; output=[opaque:{},cutout:{}]; planes=global-full-unchanged; conservation=balanced; renderer-vocabulary=ordinary-declarations-only",
        ordered.walls.report(),
        global_planes.len(),
        opaque_walls.len(),
        cutout_draws.len(),
        opaque_draws.len(),
        cutout_draws.len(),
    );
    Ok(FinalWallOccurrenceGlobalPlanesPreparation {
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
        "E1M1 final-wall-occurrence-global-planes initial preparation: {}",
        prepared.report,
    );
    scene.opaque_draws = prepared.opaque_draws;
    scene.cutout_draws = prepared.cutout_draws;
    Ok(AppliedRenderStrategy {
        candidate_selection: CandidateSelection::FullSubmission,
        ordered_coverage_prepared: false,
        source_covered_domain_filter: false,
        source_occurrence_support_filter: false,
        final_wall_occurrence_filter: true,
        fixed_reconstruction_camera: false,
    })
}
