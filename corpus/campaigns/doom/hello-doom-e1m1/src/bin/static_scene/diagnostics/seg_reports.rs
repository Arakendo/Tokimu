//! Retained reports for Doom SEG-granular presentation experiments.
//!
//! These reports inspect source representation, ordering, and conservative
//! membership controls. They do not prepare renderer submissions or own Doom
//! presentation policy.

use super::super::*;

/// Stage 3B's first retained control. It does not submit SEG-derived geometry:
/// it establishes whether the source representation can preserve identity and
/// source-texel UV continuity before viewer-relative selection is attempted.
pub(crate) fn report_doom_seg_lowering(scene: &SceneInput) -> PlatformResult<()> {
    let triangles = lower_doom_seg_textured_wall_triangles(
        &scene.door_geometry_source.map,
        &scene.door_geometry_source.wall_extents,
    )?;
    let source_segs = triangles
        .iter()
        .map(|triangle| triangle.source_seg.record_index)
        .collect::<BTreeSet<_>>();
    let source_linedefs = triangles
        .iter()
        .map(|triangle| triangle.source_linedef.record_index)
        .collect::<BTreeSet<_>>();
    let mut seam_uvs = BTreeMap::<(u32, i32, i32), BTreeSet<String>>::new();
    for triangle in &triangles {
        for (position, texture_coordinate) in
            triangle.positions.iter().zip(triangle.texture_coordinates)
        {
            seam_uvs
                .entry((
                    triangle.source_linedef.record_index,
                    position[0].round() as i32,
                    position[2].round() as i32,
                ))
                .or_default()
                .insert(format!("{:.6}", texture_coordinate[0]));
        }
    }
    // Whole-map coincident positions can legitimately belong to different
    // sidedefs, roles, or textures. This is a diagnostic count only; the
    // provider regression proves continuity for a shared source side/role.
    let multi_u_coordinate_samples = seam_uvs.values().filter(|uvs| uvs.len() > 1).count();
    println!(
        "E1M1 AR-0025 Stage 3B SEG representation: triangles={}; source_segs={}; source_linedefs={}; multi_u_coordinate_samples={}; whole_linedef_opaque_draws={}",
        triangles.len(),
        source_segs.len(),
        source_linedefs.len(),
        multi_u_coordinate_samples,
        scene.opaque_draws.len(),
    );
    println!(
        "E1M1 AR-0025 Stage 3B sample SEG identities: {}",
        triangles
            .iter()
            .take(12)
            .map(|triangle| format!(
                "seg:{} line:{} side:{:?} role:{:?} texture:{}",
                triangle.source_seg.record_index,
                triangle.source_linedef.record_index,
                triangle.side,
                triangle.role,
                triangle.texture_name
            ))
            .collect::<Vec<_>>()
            .join(" | ")
    );
    let viewer_order = resolve_doom_viewer_subsector_order(
        &scene.door_geometry_source.map,
        scene.spawn_observer.source_position,
    )?;
    println!(
        "E1M1 AR-0025 Stage 3B near-first BSP traversal: viewer=({},{}); subsectors={}; first={:?}; last={:?}; meaning=source-order-only-no-screen-coverage",
        scene.spawn_observer.source_position[0],
        scene.spawn_observer.source_position[1],
        viewer_order.len(),
        viewer_order
            .iter()
            .take(8)
            .map(|source| source.record_index)
            .collect::<Vec<_>>(),
        viewer_order
            .iter()
            .rev()
            .take(8)
            .map(|source| source.record_index)
            .collect::<Vec<_>>(),
    );
    let occluders = observe_doom_seg_occluders(&scene.door_geometry_source.map)?;
    let mut occluder_kinds = BTreeMap::<String, usize>::new();
    for observation in &occluders {
        *occluder_kinds
            .entry(format!("{:?}", observation.kind))
            .or_default() += 1;
    }
    println!(
        "E1M1 AR-0025 Stage 3B Doom occluder authority: {occluder_kinds:?}; meaning=source-height-classification-only-no-projection-or-screen-coverage"
    );
    let bounds_draws = scene
        .opaque_draws
        .iter()
        .chain(scene.cutout_draws.iter())
        .cloned()
        .collect::<Vec<_>>();
    let (center, radius) = scene_bounds(&bounds_draws);
    let poses = [
        (
            "overview",
            scene_camera([1280.0, 800.0], center, radius, None, None),
        ),
        (
            "spawn-yaw-plus-90",
            scene_camera(
                [1280.0, 800.0],
                center,
                radius,
                Some(scene.spawn_observer),
                Some(ObserverLook {
                    yaw: observer_yaw_from_forward(scene.spawn_observer.forward)
                        + std::f32::consts::FRAC_PI_2,
                    pitch: 0.0,
                    last_cursor: None,
                }),
            ),
        ),
    ];
    let mut seg_subsectors = BTreeMap::new();
    for (subsector_index, subsector) in scene.door_geometry_source.map.subsectors.iter().enumerate()
    {
        for seg_index in subsector.first_seg..subsector.first_seg + subsector.seg_count {
            let seg = &scene.door_geometry_source.map.segs[usize::from(seg_index)];
            seg_subsectors.insert(seg.source.record_index, subsector_index);
        }
    }
    for (name, camera) in poses {
        let view_projection = camera.projection * camera.view;
        let selected_subsectors = scene
            .membership_selection
            .subsector_bounds
            .iter()
            .map(|bounds| {
                bounds.is_none_or(|bounds| {
                    classify_static_draw_frustum_rejection(bounds, view_projection).is_none()
                })
            })
            .collect::<Vec<_>>();
        let submitted = triangles
            .iter()
            .filter(|triangle| {
                seg_subsectors
                    .get(&triangle.source_seg.record_index)
                    .is_some_and(|subsector| selected_subsectors[*subsector])
            })
            .count();
        println!(
            "E1M1 AR-0025 Stage 3B SEG membership control: pose={name}; source_subsectors={}/{}; submitted_seg_triangles={submitted}; candidates={}; meaning=viewer-frustum-filtered-source-subsector-not-classic-doom-screen-clipping",
            selected_subsectors.iter().filter(|selected| **selected).count(),
            selected_subsectors.len(),
            triangles.len(),
        );
    }
    Ok(())
}
