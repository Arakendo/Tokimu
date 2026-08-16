//! Retained AR-0025 Doom source-protocol and legacy presentation mechanics.
//!
//! These corpus-private subjects own source traversal, coverage reconstruction,
//! and superseded comparison preparation. They do not define Tokimu renderer
//! visibility or public Doom-provider vocabulary.

mod classic_bsp;
mod comparison_preparation;
mod screen_projection;

#[cfg(test)]
pub(crate) use classic_bsp::finalize_doom_seg_classic_plane_spans;
pub(crate) use classic_bsp::{
    count_classic_bsp_static_flat_draws, observe_doom_seg_classic_admission,
    observe_doom_seg_classic_bsp, summarize_classic_bsp_plane_marks,
    summarize_classic_bsp_wall_triangle_roles,
};
#[cfg(test)]
pub(crate) use comparison_preparation::retain_doom_seg_classic_plane_range;
pub(crate) use comparison_preparation::{
    observe_doom_seg_classic_plane_identities, prepare_doom_seg_classic_context_presentation,
    prepare_doom_seg_classic_plane_presentation, prepare_doom_seg_clip_presentation,
    prepare_doom_seg_ordered_coverage_presentation,
    prepare_doom_seg_ordered_coverage_presentation_for_view,
    prepare_doom_seg_per_column_dynamic_scene, prepare_doom_seg_per_column_presentation,
    resolve_doom_seg_classic_plane_flats,
};
#[cfg(test)]
pub(crate) use screen_projection::{
    merge_solid_range, source_bbox_fov_column_interval, source_fov_column_interval,
    source_point_segment_distance_squared, source_segment_outside_horizontal_fov,
    source_sky_sectors, SourceBBoxProjection,
};
pub(crate) use screen_projection::{
    observe_doom_seg_screen_grid, observe_doom_seg_screen_grid_with_order,
    source_ray_segment_depth, source_seg_facing, source_seg_linedef_interval, visible_column_runs,
    SourceSegFacing,
};
