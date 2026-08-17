use super::{AppliedRenderStrategy, CandidateSelection, SceneInput};
use crate::presentation::OrderedPreparedSubmissionObservation;
use tokimu::PlatformResult;

/// Replaces the global E1M1 declaration domain with the complete declaration
/// set produced by one fixed-view Doom ordered-source preparation.
///
/// This remains corpus-local. Doom owns source traversal and contribution
/// preparation; the renderer receives ordinary declarations and does not know
/// about SEGs, subsectors, or source occurrence intervals.
pub(super) fn replace_declarations(
    scene: &mut SceneInput,
    prepared: &OrderedPreparedSubmissionObservation,
) -> PlatformResult<()> {
    prepared.verify_conservation().map_err(|error| {
        format!(
            "ordered occurrence submission failed conservation before scene replacement: {error}"
        )
    })?;
    let walls = &prepared.walls;
    let planes = &prepared.plane_lowering;
    if walls.unresolved_fail_open != 0 || planes.unresolved_fail_open != 0 {
        return Err(format!(
            "ordered occurrence preparation is unresolved: walls={}, planes={}",
            walls.unresolved_fail_open, planes.unresolved_fail_open,
        )
        .into());
    }

    let missing_opaque_materials = walls
        .prepared_declarations
        .iter()
        .filter(|declaration| !declaration.cutout)
        .map(|declaration| declaration.draw.material)
        .chain(
            planes
                .prepared_declarations
                .iter()
                .map(|declaration| declaration.draw.material),
        )
        .filter(|material| {
            !scene
                .opaque_uploads
                .iter()
                .any(|upload| upload.material == *material)
        })
        .collect::<Vec<_>>();
    let missing_cutout_materials = walls
        .prepared_declarations
        .iter()
        .filter(|declaration| declaration.cutout)
        .map(|declaration| declaration.draw.material)
        .filter(|material| {
            !scene
                .cutout_uploads
                .iter()
                .any(|upload| upload.material == *material)
        })
        .collect::<Vec<_>>();
    if !missing_opaque_materials.is_empty() || !missing_cutout_materials.is_empty() {
        return Err(format!(
            "ordered declaration upload identity is unresolved: opaque={missing_opaque_materials:?}, cutout={missing_cutout_materials:?}",
        )
        .into());
    }

    let opaque_walls = walls
        .prepared_declarations
        .iter()
        .filter(|declaration| !declaration.cutout)
        .map(|declaration| declaration.draw.clone());
    let opaque_planes = planes
        .prepared_declarations
        .iter()
        .map(|declaration| declaration.draw.clone());
    let cutout_walls = walls
        .prepared_declarations
        .iter()
        .filter(|declaration| declaration.cutout)
        .map(|declaration| declaration.draw.clone());

    scene.opaque_draws = opaque_walls.chain(opaque_planes).collect();
    scene.cutout_draws = cutout_walls.collect();

    let expected_opaque = walls.lowered_opaque_meshes + planes.lowered_plane_meshes;
    if scene.opaque_draws.len() != expected_opaque
        || scene.cutout_draws.len() != walls.lowered_cutout_meshes
    {
        return Err(format!(
            "ordered declaration conservation failed: opaque={}/{expected_opaque}, cutout={}/{}",
            scene.opaque_draws.len(),
            scene.cutout_draws.len(),
            walls.lowered_cutout_meshes,
        )
        .into());
    }

    Ok(())
}

pub(super) fn apply(_scene: &mut SceneInput) -> PlatformResult<AppliedRenderStrategy> {
    Ok(AppliedRenderStrategy {
        candidate_selection: CandidateSelection::FullSubmission,
        ordered_coverage_prepared: true,
        fixed_reconstruction_camera: true,
    })
}
