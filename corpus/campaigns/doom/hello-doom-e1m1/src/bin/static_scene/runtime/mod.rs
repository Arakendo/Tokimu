//! Mutable E1M1 runtime composition subjects.
//!
//! These modules remain corpus-local and do not promote Doom activation or
//! dynamic-geometry policy into Tokimu's renderer.

mod app;
mod dynamic_geometry;
mod replay_reports;

#[cfg(test)]
pub(crate) use app::{
    advance_scrolling_wall_uvs, discover_secret_sector, source_motion_special_crossings,
    switch_material_for_draw, within_classic_use_range,
};
pub(crate) use app::{compact_activation_intent, compact_activation_target, compact_draw_source};

pub(crate) use dynamic_geometry::{
    apply_door_ceiling_flat_height, apply_sector_flat_height, carry_observer_with_floor,
    dynamic_wall_triangle_key, is_door_mesh_for_target, is_dynamic_mesh_for_target,
    manual_door_boundary_linedefs, manual_door_dynamic_wall_texture_names,
    static_wall_triangle_key,
};
pub(crate) use replay_reports::{report_door_resource_replay, report_moving_floor_resource_replay};
