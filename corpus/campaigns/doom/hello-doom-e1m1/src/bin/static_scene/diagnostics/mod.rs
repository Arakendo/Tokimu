//! Corpus-local diagnostics for inspecting prepared Doom presentation evidence.
//!
//! These helpers retain source identity and replayable observations. They do
//! not establish a renderer picking API or stable console-command contract.

mod campaign_reports;
mod candidate_policy_reports;
mod candidate_reports;
mod candidate_trace_reports;
mod command;
mod look;
mod screen_clip_report;
mod seg_reports;
mod source_protocol_traces;
mod source_reports;

pub(super) use campaign_reports::{
    report_hut_wall_candidates, report_spatial_flat_uv, report_spatial_landmark_candidates,
    report_spatial_orientation, report_walk_collision,
};
pub(super) use candidate_policy_reports::{
    report_pathological_candidate_fixture, report_temporal_candidate_carry,
    report_uniform_grid_selection,
};
pub(super) use candidate_reports::report_candidate_selection;
pub(super) use candidate_trace_reports::{
    report_candidate_position_trace, report_candidate_turn_trace,
};
pub(super) use command::{parse_debug_command, DebugCommand, NoclipAction};
pub(super) use look::{
    format_look_ray_observation, format_source_classic_ray_trace, nearest_prepared_ray_hit,
    nearest_sky_boundary_ray_hit, nearest_source_sky_plane_ray_hit, parse_source_look_ray,
    report_source_look_ray,
};
pub(super) use screen_clip_report::report_doom_seg_screen_clip;
pub(super) use seg_reports::report_doom_seg_lowering;
pub(super) use source_protocol_traces::{
    report_doom_seg_classic_admission_trace, report_doom_seg_classic_bsp_trace,
    report_doom_seg_classic_plane_identity_trace, report_doom_seg_classic_plane_span_trace,
    report_doom_seg_classic_vertical_clip_trace, report_doom_seg_ordered_coverage_pose_matrix,
    report_doom_seg_per_column_failure_trace, report_doom_seg_per_column_order_trace,
    report_doom_seg_per_column_position_trace, report_doom_seg_per_column_turn_trace,
    report_doom_seg_screen_grid,
};
pub(super) use source_reports::{
    report_doom_manual_door_runtime, report_doom_membership_union,
    report_doom_moving_floor_runtime, report_doom_reject, report_doom_topology,
    report_doom_use_activation, report_flat_normals, report_wall_source,
};
