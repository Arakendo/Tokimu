//! Doom-owned viewer-relative presentation preparation for the E1M1 corpus.
//!
//! This private subject module owns the intermediate observations produced
//! before ordinary Tokimu render declarations. It does not define renderer
//! visibility policy or public Doom-provider API.

mod legacy_source_protocol;
mod lowering;
mod model;
mod preparation;
mod sky_cylinder;
mod sky_span;
mod viewport;

pub(crate) use legacy_source_protocol::{
    count_classic_bsp_static_flat_draws, observe_doom_seg_classic_admission,
    observe_doom_seg_classic_bsp, observe_doom_seg_classic_plane_identities,
    observe_doom_seg_screen_grid, observe_doom_seg_screen_grid_with_order,
    prepare_doom_seg_classic_context_presentation, prepare_doom_seg_classic_plane_presentation,
    prepare_doom_seg_clip_presentation, prepare_doom_seg_ordered_coverage_presentation,
    prepare_doom_seg_ordered_coverage_presentation_for_view,
    prepare_doom_seg_per_column_dynamic_scene, prepare_doom_seg_per_column_presentation,
    resolve_doom_seg_classic_plane_flats, source_ray_segment_depth, source_seg_facing,
    source_seg_linedef_interval, summarize_classic_bsp_plane_marks,
    summarize_classic_bsp_wall_triangle_roles, visible_column_runs, SourceSegFacing,
};
#[cfg(test)]
pub(crate) use legacy_source_protocol::{
    finalize_doom_seg_classic_plane_spans, merge_solid_range, retain_doom_seg_classic_plane_range,
    source_bbox_fov_column_interval, source_fov_column_interval,
    source_point_segment_distance_squared, source_segment_outside_horizontal_fov,
    source_sky_sectors, SourceBBoxProjection,
};
pub(crate) use lowering::{doom_wall_role_key, lower_doom_seg_classic_plane_presentation};

pub(crate) use model::{
    DoomCoverageFailOpenSummary, DoomOrderedCoveragePreparation, DoomOrderedCoverageView,
    DoomSegClassicAdmissionObservation, DoomSegClassicContextPresentation,
    DoomSegClassicPlaneFlatResolution, DoomSegClassicPlaneIdentityObservation,
    DoomSegClassicPlanePresentation, DoomSegClipPresentation, DoomSegOrderedCoveragePresentation,
    DoomSegPerColumnPresentation, DoomSegScreenGridObservation, DoomSegScreenGridOrder,
};
pub(crate) use preparation::{
    prepare_doom_ordered_coverage, reconstruct_doom_seg_classic_plane_cells,
    reconstruct_doom_seg_classic_sky_cells,
};
pub(crate) use sky_cylinder::build_doom_sky_cylinder;
pub(crate) use sky_span::prepare_viewer_relative_source_sky_span_mesh;
pub(crate) use viewport::classic_presentation_half_vertical_fov;
#[cfg(test)]
pub(crate) use viewport::{
    CLASSIC_PRESENTATION_COLUMNS, CLASSIC_PRESENTATION_HALF_HORIZONTAL_FOV,
    CLASSIC_PRESENTATION_ROWS,
};
