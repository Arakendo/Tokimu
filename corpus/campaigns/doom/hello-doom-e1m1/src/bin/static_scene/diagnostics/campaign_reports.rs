//! Campaign-level source, orientation, and collision reports.
//!
//! These are retained corpus evidence surfaces rather than runtime policy or
//! renderer-facing contracts.

use super::super::*;

pub(crate) fn report_spatial_flat_uv(scene: &SceneInput, embedding: DoomComparativeEmbedding) {
    let camera_right = observer_right(scene.spawn_observer.forward);
    let mut aligned = 0usize;
    let mut opposed = 0usize;
    let mut neutral = 0usize;
    for draw in &scene.diagnostic_sky_draws {
        for left in 0..draw.mesh.positions.len() {
            for right in left + 1..draw.mesh.positions.len() {
                let world_delta = Vec3::from_array(draw.mesh.positions[right])
                    - Vec3::from_array(draw.mesh.positions[left]);
                let screen_delta = camera_right.dot(world_delta);
                let u_delta = draw.mesh.texture_coordinates[right][0]
                    - draw.mesh.texture_coordinates[left][0];
                let product = screen_delta * u_delta;
                if product > 0.000_1 {
                    aligned += 1;
                } else if product < -0.000_1 {
                    opposed += 1;
                } else {
                    neutral += 1;
                }
            }
        }
    }
    println!(
        "E1M1 AR-0028 flat-U observation: embedding={embedding:?}; diagnostic_sky_draws={}; camera_right=({:.1},{:.1},{:.1}); aligned_pairs={aligned}; opposed_pairs={opposed}; neutral_pairs={neutral}",
        scene.diagnostic_sky_draws.len(),
        camera_right.x,
        camera_right.y,
        camera_right.z,
    );
}

/// Bounded, renderer-free Slice 6 evidence. The replay does not claim an
/// original-Doom movement model; it only proves that this corpus-local disc
/// calculation produces the same source-line contacts and end position when
/// given the same fixed command sequence twice.
pub(crate) fn report_walk_collision(scene: &SceneInput) {
    let start = [
        scene.spawn_observer.position.x,
        scene.spawn_observer.position.z,
    ];
    let forward = scene.spawn_observer.forward.normalize_or_zero();
    let right = observer_right(forward);
    let commands = [
        [forward.x * 12.0, forward.z * 12.0],
        [forward.x * 12.0, forward.z * 12.0],
        [right.x * 12.0, right.z * 12.0],
        [right.x * 12.0, right.z * 12.0],
        [-forward.x * 6.0, -forward.z * 6.0],
    ];
    let replay = |world: &DoomWalkCollisionWorld| {
        let mut position = start;
        let mut contacts = BTreeSet::new();
        let mut fallback = false;
        for command in commands {
            let observation = world.move_disc(position, command, WALK_RADIUS);
            position = observation.resolved_position;
            contacts.extend(observation.contacted_linedefs);
            fallback |= observation.used_full_wall_fallback;
        }
        (position, contacts, fallback)
    };
    let first = replay(&scene.walk_collision);
    let second = replay(&scene.walk_collision);
    assert_eq!(
        first, second,
        "fixed collision replay must be deterministic"
    );
    let probe = scene
        .walk_collision
        .probe_nearest_blocking_wall(start, WALK_RADIUS)
        .expect("decoded E1M1 has blocking linedefs");
    assert!(
        probe
            .observation
            .contacted_linedefs
            .contains(&probe.source_linedef),
        "nearest-wall probe must retain its blocking linedef contact"
    );
    println!(
        "E1M1 Slice 6 walk replay: start=({:.1},{:.1}); commands={}; radius={}; blocking_linedefs={}; end=({:.3},{:.3}); contacts={:?}; blockmap_full_wall_fallback={}; deterministic_replay=true; nearest_wall_probe=[linedef:{}; initial_distance:{:.3}; contacts:{:?}; fallback:{}]; scope=corpus-local-disc-no-clearance-or-step-policy",
        start[0],
        start[1],
        commands.len(),
        WALK_RADIUS,
        scene.walk_collision.blocking_wall_count(),
        first.0[0],
        first.0[1],
        first.1,
        first.2,
        probe.source_linedef,
        probe.distance_before_move,
        probe.observation.contacted_linedefs,
        probe.observation.used_full_wall_fallback,
    );
}

/// Renderer-free AR-0028 evidence at the canonical Doom player-one start.
/// It reports the competing source and observer bases without selecting a
/// repair or changing the source conversion.
pub(crate) fn report_spatial_orientation(scene: &SceneInput) {
    let radians = f32::from(scene.spawn_observer.source_angle).to_radians();
    let source_forward = [radians.cos(), radians.sin()];
    let source_right = [radians.sin(), -radians.cos()];
    for embedding in DoomComparativeEmbedding::ALL {
        let observation =
            observe_doom_ground_frame_with_embedding(embedding, source_right, source_forward);
        println!(
            "E1M1 AR-0028 ground frame: embedding={:?}; thing={}; source-position=({}, {}); source-angle={}; source-right=({:.3},{:.3}); source-forward=({:.3},{:.3}); source-cross={:.3}; lifted-right=({:.3},{:.3},{:.3}); lifted-forward=({:.3},{:.3},{:.3}); world-up-cross={:.3}; camera-right=({:.3},{:.3},{:.3}); source-right/camera-right-alignment={:.3}",
            observation.embedding,
            scene.spawn_observer.source_record,
            scene.spawn_observer.source_position[0],
            scene.spawn_observer.source_position[1],
            scene.spawn_observer.source_angle,
            observation.source_right[0],
            observation.source_right[1],
            observation.source_forward[0],
            observation.source_forward[1],
            observation.source_signed_orientation,
            observation.lifted_right.x,
            observation.lifted_right.y,
            observation.lifted_right.z,
            observation.lifted_forward.x,
            observation.lifted_forward.y,
            observation.lifted_forward.z,
            observation.lifted_orientation_about_world_up,
            observation.camera_right.x,
            observation.camera_right.y,
            observation.camera_right.z,
            observation.source_right_camera_right_alignment,
        );
    }
}

/// Bounded source-record candidates for identifying the exterior hut in a
/// canonical comparison. Texture-name filtering only narrows the inspection;
/// it does not assert that any listed record is the landmark.
pub(crate) fn report_spatial_landmark_candidates(scene: &SceneInput) {
    let map = &scene.door_geometry_source.map;
    let spawn = scene.spawn_observer.source_position.map(f32::from);
    let radians = f32::from(scene.spawn_observer.source_angle).to_radians();
    let forward = [radians.cos(), radians.sin()];
    let right = [radians.sin(), -radians.cos()];

    println!(
        "E1M1 AR-0028 landmark candidates: spawn=({}, {}); angle={}; filter=BROWN*|*DOOR*",
        spawn[0], spawn[1], scene.spawn_observer.source_angle
    );
    for linedef in &map.linedefs {
        let texture_names = [linedef.right_sidedef, linedef.left_sidedef]
            .into_iter()
            .flatten()
            .filter_map(|index| map.sidedefs.get(usize::from(index)))
            .flat_map(|side| {
                [
                    side.upper_texture.as_str(),
                    side.lower_texture.as_str(),
                    side.middle_texture.as_str(),
                ]
            })
            .filter(|name| *name != "-")
            .collect::<BTreeSet<_>>();
        if !texture_names
            .iter()
            .any(|name| name.contains("BROWN") || name.contains("DOOR"))
        {
            continue;
        }
        let Some(start) = map.vertices.get(usize::from(linedef.start_vertex)) else {
            continue;
        };
        let Some(end) = map.vertices.get(usize::from(linedef.end_vertex)) else {
            continue;
        };
        let midpoint = [
            (f32::from(start.x) + f32::from(end.x)) * 0.5,
            (f32::from(start.y) + f32::from(end.y)) * 0.5,
        ];
        let relative = [midpoint[0] - spawn[0], midpoint[1] - spawn[1]];
        let forward_offset = relative[0] * forward[0] + relative[1] * forward[1];
        let source_right_offset = relative[0] * right[0] + relative[1] * right[1];
        println!(
            "linedef={}; vertices={}({},{}) -> {}({},{}) ; textures={}; source-forward-offset={:.1}; source-right-offset={:.1}",
            linedef.source.record_index,
            linedef.start_vertex,
            start.x,
            start.y,
            linedef.end_vertex,
            end.x,
            end.y,
            texture_names.into_iter().collect::<Vec<_>>().join("|"),
            forward_offset,
            source_right_offset,
        );
    }
}

/// Source-first inspection of the exterior hut neighborhood. The retained
/// landmark is LINEDEFS #208's midpoint `(2176, -3824)`; the radius is only a
/// bounded corpus filter and does not classify any span as erroneous.
pub(crate) fn report_hut_wall_candidates(scene: &SceneInput) {
    const HUT: [f32; 2] = [2176.0, -3824.0];
    const RADIUS: f32 = 640.0;

    let map = &scene.door_geometry_source.map;
    let mut selected = BTreeSet::new();
    println!(
        "E1M1 hut wall source neighborhood: anchor=linedef-208-midpoint({},{}) radius={RADIUS}",
        HUT[0], HUT[1]
    );

    for linedef in &map.linedefs {
        let (Some(start), Some(end)) = (
            map.vertices.get(usize::from(linedef.start_vertex)),
            map.vertices.get(usize::from(linedef.end_vertex)),
        ) else {
            continue;
        };
        let midpoint = [
            (f32::from(start.x) + f32::from(end.x)) * 0.5,
            (f32::from(start.y) + f32::from(end.y)) * 0.5,
        ];
        let offset = [midpoint[0] - HUT[0], midpoint[1] - HUT[1]];
        if offset[0] * offset[0] + offset[1] * offset[1] > RADIUS * RADIUS {
            continue;
        }
        selected.insert(linedef.source.record_index);

        let sector_for_side = |index: Option<u16>| {
            index
                .and_then(|index| map.sidedefs.get(usize::from(index)))
                .and_then(|sidedef| map.sectors.get(usize::from(sidedef.sector)))
        };
        let classic_sky_upper_omission = match (
            sector_for_side(linedef.right_sidedef),
            sector_for_side(linedef.left_sidedef),
        ) {
            (Some(right), Some(left)) => {
                right.ceiling_texture == "F_SKY1"
                    && left.ceiling_texture == "F_SKY1"
                    && right.ceiling_height != left.ceiling_height
            }
            _ => false,
        };

        let side = |index: Option<u16>| -> String {
            let Some(index) = index else {
                return "none".to_owned();
            };
            let Some(sidedef) = map.sidedefs.get(usize::from(index)) else {
                return format!("sidedef={index}:missing");
            };
            let Some(sector) = map.sectors.get(usize::from(sidedef.sector)) else {
                return format!(
                    "sidedef={index}:sector={}:missing textures={}/{}/{}",
                    sidedef.sector,
                    sidedef.upper_texture,
                    sidedef.lower_texture,
                    sidedef.middle_texture,
                );
            };
            format!(
                "sidedef={} sector={} heights={}/{} flats={}/{} textures={}/{}/{} offsets={}/{}",
                sidedef.source.record_index,
                sidedef.sector,
                sector.floor_height,
                sector.ceiling_height,
                sector.floor_texture,
                sector.ceiling_texture,
                sidedef.upper_texture,
                sidedef.lower_texture,
                sidedef.middle_texture,
                sidedef.x_offset,
                sidedef.y_offset,
            )
        };

        println!(
            "source linedef={} flags=0x{:04x} vertices={}({},{}) -> {}({},{}) midpoint=({:.1},{:.1}) classic_sky_upper_omission={} right/front=[{}] left/back=[{}]",
            linedef.source.record_index,
            linedef.flags,
            linedef.start_vertex,
            start.x,
            start.y,
            linedef.end_vertex,
            end.x,
            end.y,
            midpoint[0],
            midpoint[1],
            classic_sky_upper_omission,
            side(linedef.right_sidedef),
            side(linedef.left_sidedef),
        );
    }

    let mut generated = 0usize;
    for draw in scene.opaque_draws.iter().chain(scene.cutout_draws.iter()) {
        let StaticDrawSource::Wall {
            source_linedef,
            source_sidedef,
            source_sector,
            role,
        } = draw.source
        else {
            continue;
        };
        if !selected.contains(&source_linedef.record_index) {
            continue;
        }
        let bottom = draw
            .mesh
            .positions
            .iter()
            .map(|position| position[1])
            .fold(f32::INFINITY, f32::min);
        let top = draw
            .mesh
            .positions
            .iter()
            .map(|position| position[1])
            .fold(f32::NEG_INFINITY, f32::max);
        println!(
            "generated linedef={} sidedef={} sector={} role={role:?} bottom={bottom:.1} top={top:.1} label={}",
            source_linedef.record_index,
            source_sidedef.record_index,
            source_sector.record_index,
            draw.source_label,
        );
        generated += 1;
    }
    println!(
        "E1M1 hut wall source neighborhood summary: linedefs={} generated_wall_draws={generated}",
        selected.len()
    );
}
