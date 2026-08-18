use super::super::prepare_doom_seg_ordered_coverage_presentation;
use super::{AppliedRenderStrategy, CandidateSelection, SceneInput};
use tokimu::PlatformResult;

/// Strategy B consumes one coherent Doom-owned ordered observation and submits
/// every retained declaration. Generic candidate filtering does not participate.
pub(super) fn apply(scene: &mut SceneInput) -> PlatformResult<AppliedRenderStrategy> {
    apply_prepared_declarations(scene)?;
    Ok(AppliedRenderStrategy {
        candidate_selection: CandidateSelection::FullSubmission,
        ordered_coverage_prepared: true,
        source_covered_domain_filter: false,
        fixed_reconstruction_camera: false,
    })
}

pub(super) fn apply_prepared_declarations(scene: &mut SceneInput) -> PlatformResult<()> {
    let presentation = prepare_doom_seg_ordered_coverage_presentation(scene)?;
    eprintln!(
        "E1M1 AR-0025 Slice 7 ordered-coverage presentation: renderer-input=prepared-full-submission; wall-conservation=[retained-cells:{} reconstructed-triangles:{} lowered-triangles:{} source-degenerate-cells:{} source-unresolved-cells:{} lowering-degenerate-triangles:{} lowering-unresolved-triangles:{}]; grouped-wall-meshes={}; opaque-draws={}; cutout-draws={}; plane-conservation=[ordinary:{} reconstructed:{} rejected:{} lowered:{}]; sky-background-intervals={}; cutout-key-conservation={}/{}; coverage=[transitions:{} fail-open:{} reasons:{:?}]; bsp=[leaves:{} far-pruned:{} admitted-segs:{} solid-range-pruning:{}]; degenerate-omissions={}; unresolved-contributions={}; samples={:?}; meaning=one-fixed-source-observation-lowered-to-complete-prepared-declarations",
        presentation.retained_cells,
        presentation.reconstructed_triangles,
        presentation.lowered_wall_triangles,
        presentation.source_degenerate_cells,
        presentation.source_unresolved_cells,
        presentation.lowering_degenerate_triangles,
        presentation.lowering_unresolved_triangles,
        presentation.grouped_wall_meshes,
        presentation.opaque_draws.len(),
        presentation.cutout_draws.len(),
        presentation.ordinary_plane_intervals,
        presentation.reconstructed_plane_quads,
        presentation.rejected_plane_intervals,
        presentation.lowered_plane_quads,
        presentation.sky_plane_intervals,
        presentation.lowered_cutout_keys,
        presentation.source_cutout_keys,
        presentation.coverage_transitions,
        presentation.coverage_fail_open,
        presentation.coverage_fail_open_reasons,
        presentation.bsp_leaves_visited,
        presentation.bsp_far_children_pruned,
        presentation.bsp_admitted_segs,
        presentation.bsp_solid_range_pruning,
        presentation.degenerate_omissions,
        presentation.unresolved_cells,
        presentation.samples,
    );
    scene.opaque_draws = presentation.opaque_draws;
    scene.cutout_draws = presentation.cutout_draws;
    Ok(())
}
