//! Conservation tests for the native Doom composition and its private subjects.

use std::collections::{BTreeMap, BTreeSet};

use super::{
    advance_scrolling_wall_uvs, arguments_for_rotated_map, build_doom_sky_cylinder,
    carry_observer_with_floor, diagnostic_skywall_mesh, discover_secret_sector,
    finalize_doom_seg_classic_plane_spans, merge_solid_range, mesh_owning_side_visible,
    nearest_mesh_ray_hit, ray_triangle_distance, retain_doom_seg_classic_plane_range,
    source_bbox_fov_column_interval, source_fov_column_interval, source_motion_special_crossings,
    source_point_segment_distance_squared, source_ray_segment_depth, source_seg_facing,
    source_segment_outside_horizontal_fov, source_sky_sectors, switch_material_for_draw,
    visible_column_runs, within_classic_use_range, DoomSegClassicPlaneInstance,
    DoomSegClassicPlaneKey, DoomSegClassicPlaneKind, DoomSegClassicPlaneSpanObservation,
    SourceBBoxProjection, SourceSegFacing, SpawnObserver,
};

#[test]
fn diagnostic_skywall_mesh_supplies_repeating_planar_texture_coordinates() {
    let mesh = diagnostic_skywall_mesh(vec![
        [64.0, 0.0, 32.0],
        [128.0, 64.0, 32.0],
        [128.0, 0.0, 32.0],
    ])
    .expect("diagnostic skywall mesh");

    assert!(mesh.has_texture_coordinates());
    assert_eq!(
        mesh.texture_coordinates,
        vec![[1.0, -0.0], [2.0, -1.0], [2.0, -0.0]]
    );
}

#[test]
fn map_rotation_arguments_replace_or_append_exactly_one_map_selector() {
    assert_eq!(
        arguments_for_rotated_map(&["package.zip".into(), "DOOM1.WAD".into()], "E1M2"),
        vec!["package.zip", "DOOM1.WAD", "--map=E1M2"]
    );
    assert_eq!(
        arguments_for_rotated_map(
            &[
                "package.zip".into(),
                "DOOM1.WAD".into(),
                "--map=E1M1".into(),
                "--noclip".into(),
            ],
            "E1M2",
        ),
        vec!["package.zip", "DOOM1.WAD", "--map=E1M2", "--noclip"]
    );
}
use doom_geometry_provider::doom_point_to_tokimu;
use doom_map_provider::{DoomLinedef, DoomSector, DoomSourceRecord, DoomVertex};
use hello_doom_e1m1::specials::{DoomSwitchTextureChange, DoomSwitchTextureSlot};
use hello_doom_e1m1::{
    classify_static_draw_frustum_rejection, classify_static_draw_sphere_frustum_rejection,
    doom_heading_degrees_to_observer_yaw, doom_heading_forward, observer_direction, observer_right,
    observer_yaw_from_forward, observer_yaw_to_doom_heading_degrees, reembed_comparative_mesh,
    DoomComparativeEmbedding, StaticDrawAabb, StaticDrawFrustumRejection, StaticDrawPlanEntry,
    StaticDrawSource, StaticDrawSphere,
};
use tokimu::{MaterialHandle, Mesh};

#[test]
fn switch_material_override_matches_exact_source_wall_and_slot() {
    let source = |record_index| DoomSourceRecord {
        lump_index: 8,
        record_index,
    };
    let draw = StaticDrawPlanEntry {
        mesh: Mesh::triangle(),
        material: MaterialHandle(10),
        source_label: "wall:7:SW1EXIT".to_owned(),
        source: StaticDrawSource::Wall {
            source_linedef: source(7),
            source_sidedef: source(9),
            source_sector: source(11),
            role: doom_geometry_provider::DoomWallTextureRole::Middle,
        },
    };
    let change = DoomSwitchTextureChange {
        source_linedef: source(7),
        source_sidedef: source(9),
        slot: DoomSwitchTextureSlot::Middle,
        before_texture: "SW1EXIT".to_owned(),
        after_texture: "SW2EXIT".to_owned(),
    };
    let materials = BTreeMap::from([("SW2EXIT".to_owned(), MaterialHandle(20))]);

    assert_eq!(
        switch_material_for_draw(&draw, &[change.clone()], &materials),
        Some(MaterialHandle(20))
    );
    let mut wrong_slot = change;
    wrong_slot.slot = DoomSwitchTextureSlot::Upper;
    assert_eq!(
        switch_material_for_draw(&draw, &[wrong_slot], &materials),
        None
    );
}

#[test]
fn scrolling_wall_uvs_advance_one_source_texel_per_tick_on_selected_sidedefs() {
    let source = |record_index| DoomSourceRecord {
        lump_index: 9,
        record_index,
    };
    let wall = |sidedef, material| StaticDrawPlanEntry {
        mesh: Mesh::triangle()
            .with_texture_coordinates(vec![[0.0, 0.0], [0.5, 0.0], [1.0, 1.0]])
            .expect("aligned wall UVs"),
        material,
        source_label: "scrolling-wall".to_owned(),
        source: StaticDrawSource::Wall {
            source_linedef: source(4),
            source_sidedef: source(sidedef),
            source_sector: source(12),
            role: doom_geometry_provider::DoomWallTextureRole::Middle,
        },
    };
    let mut draws = vec![wall(7, MaterialHandle(20)), wall(8, MaterialHandle(20))];
    let changed = advance_scrolling_wall_uvs(
        &mut draws,
        &BTreeSet::from([7]),
        &BTreeMap::from([(20, 1.0 / 64.0)]),
        2,
    );

    assert_eq!(changed, vec![0]);
    assert_eq!(draws[0].mesh.texture_coordinates[0], [2.0 / 64.0, 0.0]);
    assert_eq!(
        draws[0].mesh.texture_coordinates[2],
        [1.0 + 2.0 / 64.0, 1.0]
    );
    assert_eq!(draws[1].mesh.texture_coordinates[0], [0.0, 0.0]);
}

#[test]
fn secret_sector_progress_is_first_entry_only_and_source_indexed() {
    let sector = |record_index, special| DoomSector {
        source: DoomSourceRecord {
            lump_index: 14,
            record_index,
        },
        floor_height: 0,
        ceiling_height: 128,
        floor_texture: "FLOOR0_1".to_owned(),
        ceiling_texture: "CEIL1_1".to_owned(),
        light_level: 160,
        special,
        tag: 0,
    };
    let sectors = vec![sector(0, 0), sector(1, 9)];
    let mut discovered = BTreeSet::new();

    assert!(!discover_secret_sector(&sectors, 0, &mut discovered));
    assert!(discover_secret_sector(&sectors, 1, &mut discovered));
    assert!(!discover_secret_sector(&sectors, 1, &mut discovered));
    assert!(!discover_secret_sector(&sectors, 99, &mut discovered));
    assert_eq!(discovered, BTreeSet::from([1]));
}

#[test]
fn masked_middle_ownership_is_visible_only_from_the_supplied_normal_side() {
    let mesh = Mesh::uniform_normal(
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        [0.0, 0.0, 1.0],
    );

    assert!(mesh_owning_side_visible(&mesh, Vec3::new(0.0, 0.0, 8.0)));
    assert!(!mesh_owning_side_visible(&mesh, Vec3::new(0.0, 0.0, -8.0)));
    assert!(mesh_owning_side_visible(&Mesh::default(), Vec3::ZERO));
}

#[test]
fn doom_sky_cylinder_is_a_closed_horizontal_panorama_seam() {
    let mesh = build_doom_sky_cylinder(Vec3::ZERO, 10.0).expect("sky mesh");

    assert_eq!(mesh.positions.len(), 64 * 6);
    assert_eq!(mesh.normals.len(), mesh.positions.len());
    assert_eq!(mesh.texture_coordinates.len(), mesh.positions.len());
    assert_eq!(mesh.texture_coordinates[0], [0.0, 1.0]);
    assert_eq!(mesh.texture_coordinates[64 * 6 - 2], [63.0 / 64.0, 0.0]);
    assert_eq!(mesh.texture_coordinates[64 * 6 - 1], [1.0, 0.0]);
    assert!(mesh
        .positions
        .iter()
        .all(|position| position.iter().all(|component| component.is_finite())));
}

#[test]
fn source_ray_segment_depth_retains_finite_forward_intersection() {
    assert_eq!(
        source_ray_segment_depth([0, 0], [1.0, 0.0], [10, -5], [10, 5]),
        Some(10.0)
    );
    assert_eq!(
        source_ray_segment_depth([0, 0], [1.0, 0.0], [-10, -5], [-10, 5]),
        None
    );
}

#[test]
fn source_motion_crossings_are_ordered_and_ignore_non_cross_specials() {
    let source = |record_index| DoomSourceRecord {
        lump_index: 17,
        record_index,
    };
    let vertices = vec![
        DoomVertex {
            source: source(0),
            x: 5,
            y: -5,
        },
        DoomVertex {
            source: source(1),
            x: 5,
            y: 5,
        },
        DoomVertex {
            source: source(2),
            x: 8,
            y: -5,
        },
        DoomVertex {
            source: source(3),
            x: 8,
            y: 5,
        },
        DoomVertex {
            source: source(4),
            x: 3,
            y: -5,
        },
        DoomVertex {
            source: source(5),
            x: 3,
            y: 5,
        },
    ];
    let line = |record_index, start_vertex, end_vertex, special| DoomLinedef {
        source: source(record_index),
        start_vertex,
        end_vertex,
        flags: 0,
        special,
        tag: 1,
        right_sidedef: None,
        left_sidedef: None,
    };
    let lines = vec![
        line(20, 0, 1, 88),
        line(21, 2, 3, 36),
        line(22, 4, 5, 1),
        line(23, 6, 7, 11),
    ];
    assert_eq!(
        source_motion_special_crossings(&vertices, &lines, [0, 0], [10, 0]),
        vec![source(20), source(21)]
    );
    assert!(source_motion_special_crossings(&vertices, &lines, [0, 0], [2, 0]).is_empty());
}

#[test]
fn moving_floor_carries_only_its_stationary_sector_observer() {
    let target = DoomSourceRecord {
        lump_index: 17,
        record_index: 4,
    };
    let mut observer = SpawnObserver {
        position: Vec3::new(10.0, 36.0, 20.0),
        forward: Vec3::Z,
        source_record: 0,
        source_position: [10, 20],
        source_angle: 0,
        sector: 4,
        floor: 0,
        ceiling: 72,
    };

    assert!(carry_observer_with_floor(
        Some(&mut observer),
        target,
        0,
        -4
    ));
    assert_eq!(observer.floor, -4);
    assert_eq!(observer.position.y, 32.0);
    assert!(!carry_observer_with_floor(
        Some(&mut observer),
        DoomSourceRecord {
            lump_index: 17,
            record_index: 5,
        },
        -4,
        -8,
    ));
    assert_eq!(observer.floor, -4);
    assert_eq!(observer.position.y, 32.0);
}

#[test]
fn source_point_segment_distance_retains_finite_nearest_point() {
    assert!(
        (source_point_segment_distance_squared([2, 3], [0, 0], [4, 0]) - 9.0).abs() < f64::EPSILON
    );
    assert!(
        (source_point_segment_distance_squared([8, 0], [0, 0], [4, 0]) - 16.0).abs() < f64::EPSILON
    );
}

#[test]
fn source_seg_facing_retains_directed_right_side_rule() {
    assert_eq!(
        source_seg_facing([0, 0], [10, -5], [10, 5]),
        SourceSegFacing::Back
    );
    assert_eq!(
        source_seg_facing([0, 0], [10, 5], [10, -5]),
        SourceSegFacing::Front
    );
    assert_eq!(
        source_seg_facing([0, 0], [10, 0], [20, 0]),
        SourceSegFacing::EdgeOn
    );
}

#[test]
fn horizontal_solid_ranges_union_without_treating_gaps_as_closed() {
    let mut ranges = vec![[4, 7]];
    assert!(!merge_solid_range(&mut ranges, [9, 11]));
    assert_eq!(ranges, vec![[4, 7], [9, 11]]);
    assert!(!merge_solid_range(&mut ranges, [8, 8]));
    assert_eq!(ranges, vec![[4, 11]]);
    assert!(merge_solid_range(&mut ranges, [6, 9]));
    assert_eq!(ranges, vec![[4, 11]]);
}

#[test]
fn source_fov_interval_clamps_to_declared_diagnostic_columns() {
    assert_eq!(source_fov_column_interval(-2.0, 2.0, 1.0, 10), [0, 9]);
    assert_eq!(source_fov_column_interval(-0.5, 0.5, 1.0, 10), [3, 6]);
}

#[test]
fn source_fov_rejects_only_segments_outside_on_the_same_side() {
    let half_fov = 1.0;
    assert!(source_segment_outside_horizontal_fov(1.1, 1.5, half_fov));
    assert!(source_segment_outside_horizontal_fov(-1.1, -1.5, half_fov));
    assert!(!source_segment_outside_horizontal_fov(-1.5, 1.5, half_fov));
    assert!(!source_segment_outside_horizontal_fov(0.0, 1.5, half_fov));
}

#[test]
fn source_bbox_interval_distinguishes_outside_from_fail_open_cases() {
    assert_eq!(
        source_bbox_fov_column_interval([0, 0], 0.0, [5, -5, 10, 20], 1.0, 10),
        SourceBBoxProjection::Interval([3, 6])
    );
    assert_eq!(
        source_bbox_fov_column_interval([0, 0], 0.0, [5, -5, -20, -10], 1.0, 10),
        SourceBBoxProjection::Uncertain
    );
    assert_eq!(
        source_bbox_fov_column_interval([0, 0], 0.0, [5, -5, -10, 10], 1.0, 10),
        SourceBBoxProjection::Uncertain
    );
    assert_eq!(
        source_bbox_fov_column_interval([0, 0], 0.0, [20, 10, 10, 20], 0.2, 10),
        SourceBBoxProjection::OutsideFov
    );
}
use tokimu_core::math::{Mat4, Vec3};

#[test]
fn center_ray_reports_an_exact_triangle_hit_distance() {
    let distance = ray_triangle_distance(
        Vec3::ZERO,
        Vec3::Z,
        Vec3::new(-1.0, -1.0, 5.0),
        Vec3::new(1.0, -1.0, 5.0),
        Vec3::new(0.0, 1.0, 5.0),
    )
    .expect("center ray should hit the fixture triangle");
    assert!((distance - 5.0).abs() < 0.000_1);
}

#[test]
fn center_ray_rejects_a_triangle_outside_the_ray() {
    assert!(ray_triangle_distance(
        Vec3::ZERO,
        Vec3::Z,
        Vec3::new(2.0, -1.0, 5.0),
        Vec3::new(4.0, -1.0, 5.0),
        Vec3::new(3.0, 1.0, 5.0),
    )
    .is_none());
}

#[test]
fn physical_use_range_is_bounded_and_inclusive() {
    assert!(within_classic_use_range(0.0));
    assert!(within_classic_use_range(63.999));
    assert!(within_classic_use_range(64.0));
    assert!(!within_classic_use_range(64.001));
    assert!(!within_classic_use_range(f64::NAN));
    assert!(!within_classic_use_range(f64::INFINITY));
    assert!(!within_classic_use_range(-0.001));
}

#[test]
fn candidate_embeddings_preserve_exact_picking_distance() {
    let source_mesh = Mesh::uniform_normal(
        vec![[-1.0, -1.0, 5.0], [1.0, -1.0, 5.0], [0.0, 1.0, 5.0]],
        [0.0, 0.0, -1.0],
    )
    .with_texture_coordinates(vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]])
    .unwrap();
    let source_distance = nearest_mesh_ray_hit(Vec3::ZERO, Vec3::Z, &source_mesh).unwrap();

    for embedding in [
        DoomComparativeEmbedding::PreserveEast,
        DoomComparativeEmbedding::PreserveNorth,
    ] {
        let mut candidate_mesh = source_mesh.clone();
        reembed_comparative_mesh(&mut candidate_mesh, embedding, false);
        let candidate_direction = embedding.lift_direction([0.0, 1.0], 0.0);
        let candidate_distance =
            nearest_mesh_ray_hit(Vec3::ZERO, candidate_direction, &candidate_mesh).unwrap();
        assert!((candidate_distance - source_distance).abs() < 0.000_1);
    }
}

#[test]
fn source_spawn_heading_maps_doom_cardinal_angles_to_world_xz() {
    let east = doom_heading_forward(0);
    let north = doom_heading_forward(90);

    assert!((east.x - 1.0).abs() < 0.000_1);
    assert!(east.z.abs() < 0.000_1);
    assert!(north.x.abs() < 0.000_1);
    assert!((north.z - 1.0).abs() < 0.000_1);
}

#[test]
fn source_orientation_round_trips_through_observer_yaw() {
    for source_degrees in [0.0, 45.0, 90.0, 180.0, 270.0, 359.0] {
        let yaw = doom_heading_degrees_to_observer_yaw(source_degrees);
        let round_trip = observer_yaw_to_doom_heading_degrees(yaw);
        assert!(
            (round_trip - source_degrees).abs() < 0.000_1,
            "source={source_degrees} yaw={yaw} round_trip={round_trip}"
        );
    }
}

#[test]
fn source_heading_and_observer_look_share_the_declared_right_handed_axes() {
    let source_north = doom_heading_forward(90);
    let yaw = observer_yaw_from_forward(source_north);
    let initial = observer_direction(yaw, 0.0);
    let positive_yaw = observer_direction(yaw + std::f32::consts::FRAC_PI_2, 0.0);
    let screen_right = observer_right(initial);
    let screen_right_turn = observer_direction(yaw - std::f32::consts::FRAC_PI_2, 0.0);
    let upward_look = observer_direction(yaw, 0.5);

    assert!(initial.x.abs() < 0.000_1);
    assert!((initial.z - 1.0).abs() < 0.000_1);
    assert!((positive_yaw.x - 1.0).abs() < 0.000_1);
    assert!(positive_yaw.z.abs() < 0.000_1);
    assert!((screen_right.x + 1.0).abs() < 0.000_1);
    assert!(screen_right.z.abs() < 0.000_1);
    assert!(screen_right_turn.dot(screen_right) > 0.999_9);
    assert!(upward_look.y > 0.0);
}

#[test]
fn source_spawn_command_replay_preserves_converted_forward_strafe_and_yaw() {
    let source_spawn = doom_point_to_tokimu([1056.0, -3616.0], 36.0);
    let position = Vec3::new(
        source_spawn[0] as f32,
        source_spawn[1] as f32,
        source_spawn[2] as f32,
    );
    let source_yaw = doom_heading_degrees_to_observer_yaw(90.0);
    let forward = observer_direction(source_yaw, 0.0);
    let right = observer_right(forward);

    assert_eq!(position, Vec3::new(1056.0, 36.0, -3616.0));
    assert!((forward - Vec3::Z).length() < 0.000_1);
    assert!((right + Vec3::X).length() < 0.000_1);
    assert_eq!(position + forward * 16.0, Vec3::new(1056.0, 36.0, -3600.0));
    assert_eq!(position + right * 16.0, Vec3::new(1040.0, 36.0, -3616.0));

    let screen_right_yaw = source_yaw - std::f32::consts::FRAC_PI_2;
    assert!(observer_direction(screen_right_yaw, 0.0).dot(right) > 0.999_9);
}

#[test]
fn frustum_aabb_rejects_only_bounds_wholly_outside_one_clip_plane() {
    let inside = bounds([-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]);
    let outside_left = bounds([-3.0, -0.5, -0.5], [-2.0, 0.5, 0.5]);
    let crossing_left = bounds([-2.0, -0.5, -0.5], [0.0, 0.5, 0.5]);

    assert_eq!(
        classify_static_draw_frustum_rejection(inside, Mat4::IDENTITY),
        None
    );
    assert_eq!(
        classify_static_draw_frustum_rejection(outside_left, Mat4::IDENTITY),
        Some(StaticDrawFrustumRejection::Left)
    );
    assert_eq!(
        classify_static_draw_frustum_rejection(crossing_left, Mat4::IDENTITY),
        None
    );
}

#[test]
fn frustum_sphere_rejects_only_a_sphere_wholly_outside_one_clip_plane() {
    let inside = sphere([-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]);
    let outside_right = sphere([2.0, -0.5, -0.5], [3.0, 0.5, 0.5]);
    let crossing_right = sphere([0.5, -0.5, -0.5], [1.5, 0.5, 0.5]);

    assert_eq!(
        classify_static_draw_sphere_frustum_rejection(inside, Mat4::IDENTITY),
        None
    );
    assert_eq!(
        classify_static_draw_sphere_frustum_rejection(outside_right, Mat4::IDENTITY),
        Some(StaticDrawFrustumRejection::Right)
    );
    assert_eq!(
        classify_static_draw_sphere_frustum_rejection(crossing_right, Mat4::IDENTITY),
        None
    );
}

#[test]
fn diagnostic_screen_runs_keep_only_currently_uncovered_columns() {
    assert_eq!(
        visible_column_runs(&[true, false, false, true, false]),
        vec![[1, 3], [4, 5]]
    );
    assert!(visible_column_runs(&[true, true]).is_empty());
}

#[test]
fn classic_plane_span_accumulator_keeps_keys_separate_and_splits_collisions() {
    let floor = DoomSegClassicPlaneKey {
        kind: DoomSegClassicPlaneKind::Floor,
        height: 0,
        texture: String::from("FLOOR4_8"),
        light: 160,
    };
    let ceiling = DoomSegClassicPlaneKey {
        kind: DoomSegClassicPlaneKind::Ceiling,
        height: 72,
        texture: String::from("CEIL3_5"),
        light: 160,
    };
    let mut observation = DoomSegClassicPlaneSpanObservation::default();
    retain_doom_seg_classic_plane_range(
        &mut observation,
        floor.clone(),
        10,
        20,
        &[(0, 4, 7), (1, 5, 8)],
        4,
    );
    retain_doom_seg_classic_plane_range(&mut observation, floor.clone(), 11, 21, &[(1, 3, 6)], 4);
    retain_doom_seg_classic_plane_range(&mut observation, floor, 10, 22, &[(3, 2, 2)], 4);
    retain_doom_seg_classic_plane_range(&mut observation, ceiling, 10, 23, &[(2, 0, 1)], 4);
    finalize_doom_seg_classic_plane_spans(&mut observation);

    assert_eq!(observation.keys.len(), 2);
    assert_eq!(observation.plane_instances, 3);
    assert_eq!(observation.collision_splits, 1);
    assert_eq!(observation.horizontal_spans, 4);
    assert_eq!(observation.populated_columns, 5);
    assert_eq!(observation.populated_cells, 15);
    assert_eq!(observation.overlapping_writes, 0);
    assert_eq!(observation.empty_after_clip, 0);
    let floor_instances = observation
        .keys
        .values()
        .find(|instances| instances.len() == 2)
        .expect("floor key split into two instances");
    assert_eq!(floor_instances[0].source_sectors, BTreeSet::from([10]));
    assert_eq!(floor_instances[0].source_segs, BTreeSet::from([20, 22]));
    assert_eq!(floor_instances[1].source_sectors, BTreeSet::from([11]));
    assert_eq!(floor_instances[1].source_segs, BTreeSet::from([21]));
}

#[test]
fn source_sky_sector_admission_ignores_ordinary_ceiling_instances() {
    let instance = |sector| DoomSegClassicPlaneInstance {
        columns: vec![Some([0, 1])],
        column_sources: vec![Some([sector, sector + 100])],
        minimum_column: 0,
        maximum_column: 0,
        source_sectors: BTreeSet::from([sector]),
        source_segs: BTreeSet::from([sector + 100]),
    };
    let spans = DoomSegClassicPlaneSpanObservation {
        keys: BTreeMap::from([
            (
                DoomSegClassicPlaneKey {
                    kind: DoomSegClassicPlaneKind::Ceiling,
                    height: 0,
                    texture: String::from("F_SKY1"),
                    light: 0,
                },
                vec![instance(7), instance(9)],
            ),
            (
                DoomSegClassicPlaneKey {
                    kind: DoomSegClassicPlaneKind::Ceiling,
                    height: 64,
                    texture: String::from("CEIL3_5"),
                    light: 160,
                },
                vec![instance(11)],
            ),
        ]),
        ..Default::default()
    };

    assert_eq!(source_sky_sectors(&spans), BTreeSet::from([7, 9]));
}

fn bounds(minimum: [f32; 3], maximum: [f32; 3]) -> StaticDrawAabb {
    StaticDrawAabb::from_positions(&[minimum, maximum]).expect("finite test bounds")
}

fn sphere(minimum: [f32; 3], maximum: [f32; 3]) -> StaticDrawSphere {
    StaticDrawSphere::from_positions(&[minimum, maximum]).expect("finite test sphere")
}
