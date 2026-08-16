//! Spatial-index, temporal-carry, and pathological candidate controls.

use super::super::*;

pub(crate) fn report_uniform_grid_selection(
    scene: &SceneInput,
    include_cutouts: bool,
    center: Vec3,
    radius: f32,
) {
    let mut bounds = draw_bounds(&scene.opaque_draws);
    if include_cutouts {
        bounds.extend(draw_bounds(&scene.cutout_draws));
    }
    let size = [1280.0, 800.0];
    let spawn_yaw = observer_yaw_from_forward(scene.spawn_observer.forward);
    let overview_camera = scene_camera(size, center, radius, None, None);
    let mut poses = vec![(
        "overview".to_owned(),
        overview_camera.projection * overview_camera.view,
    )];
    for yaw_offset_degrees in (0..=360).step_by(45) {
        let camera = scene_camera(
            size,
            center,
            radius,
            Some(scene.spawn_observer),
            Some(ObserverLook {
                yaw: spawn_yaw + (yaw_offset_degrees as f32).to_radians(),
                pitch: 0.0,
                last_cursor: None,
            }),
        );
        poses.push((
            format!("source-spawn-yaw-offset-{yaw_offset_degrees}"),
            camera.projection * camera.view,
        ));
    }
    for forward_offset in [-256.0_f32, -128.0, 128.0, 256.0] {
        let position =
            scene.spawn_observer.position + scene.spawn_observer.forward * forward_offset;
        let mut camera = scene_camera(size, center, radius, None, None);
        camera.view = tokimu_core::math::try_view_look_at_rh(
            position,
            position + scene.spawn_observer.forward * 128.0,
            Vec3::Y,
        )
        .expect("camera basis must be finite and non-degenerate");
        poses.push((
            format!("source-spawn-forward-offset-{forward_offset:+.0}"),
            camera.projection * camera.view,
        ));
    }
    for dimensions in [[4, 2, 4], [8, 4, 8], [16, 4, 16]] {
        let build_started = Instant::now();
        let Some(grid) = UniformGridAabbIndex::build(&bounds, dimensions) else {
            println!("E1M1 AR-0025 uniform grid: unavailable; reason=no-finite-bounds");
            return;
        };
        let build_cpu_us = build_started.elapsed().as_micros();
        let cell_memberships = grid.cells.iter().map(Vec::len).sum::<usize>();
        let cell_capacity = grid.cells.iter().map(Vec::capacity).sum::<usize>();
        let occupied_cells = grid.cells.iter().filter(|cell| !cell.is_empty()).count();
        let estimated_index_bytes = grid.cells.len() * std::mem::size_of::<Vec<usize>>()
            + cell_capacity * std::mem::size_of::<usize>();
        println!(
            "E1M1 AR-0025 uniform grid build: dimensions={}x{}x{}; cells={}; occupied_cells={occupied_cells}; draw_bounds={}; uncertain_draws={}; cell_memberships={cell_memberships}; cell_capacity={cell_capacity}; estimated_index_bytes={estimated_index_bytes}; build_cpu_us={build_cpu_us}",
            dimensions[0],
            dimensions[1],
            dimensions[2],
            grid.cells.len(),
            bounds.len(),
            grid.uncertain_draws.len(),
        );
        for (pose, view_projection) in &poses {
            let started = Instant::now();
            let (_survivors, summary) = grid.select(&bounds, *view_projection);
            let selection_cpu_us = started.elapsed().as_micros();
            println!(
                "E1M1 AR-0025 uniform grid: dimensions={}x{}x{}; pose={pose}; candidates={}; cells_tested={}; cells_rejected={}; grid_candidates={}; exact_tests={}; submitted={}; rejected={}; uncertain_bounds={}; selection_cpu_us={selection_cpu_us}",
                dimensions[0],
                dimensions[1],
                dimensions[2],
                bounds.len(),
                summary.cells_tested,
                summary.cells_rejected,
                summary.grid_candidates,
                summary.exact_tests,
                summary.submitted,
                summary.rejected,
                summary.uncertain_bounds,
            );
        }
    }
}

/// AR-0025 theory trial: retain temporal overlap facts without granting a
/// prior frame authority over the current one. Every row first performs the
/// fresh conservative AABB classification; a one-frame carried set is then
/// reported only to show the cost of avoiding boundary churn. It is never used
/// to skip the fresh test, so abrupt turns and declared teleports fail safely.
pub(crate) fn report_temporal_candidate_carry(
    scene: &SceneInput,
    include_cutouts: bool,
    center: Vec3,
    radius: f32,
) {
    let mut bounds = draw_bounds(&scene.opaque_draws);
    if include_cutouts {
        bounds.extend(draw_bounds(&scene.cutout_draws));
    }
    let size = [1280.0, 800.0];
    let source_yaw = observer_yaw_from_forward(scene.spawn_observer.forward);
    let base_camera = scene_camera(size, center, radius, None, None);
    let expanded_projection = tokimu_core::math::try_projection_perspective_rh_gl(
        72.0_f32.to_radians(),
        size[0] / size[1],
        (radius * 0.000_1).max(0.1),
        radius * 4.0,
    )
    .expect("perspective parameters must be finite and ordered");
    let mut poses = Vec::new();
    for (label, yaw_offset_degrees) in [
        ("smooth-yaw-0", 0.0_f32),
        ("smooth-yaw-5", 5.0),
        ("smooth-yaw-10", 10.0),
        ("abrupt-turn-190", 190.0),
    ] {
        let camera = scene_camera(
            size,
            center,
            radius,
            Some(scene.spawn_observer),
            Some(ObserverLook {
                yaw: source_yaw + yaw_offset_degrees.to_radians(),
                pitch: 0.0,
                last_cursor: None,
            }),
        );
        poses.push((label, camera.view));
    }
    let teleport_position = scene.spawn_observer.position + scene.spawn_observer.forward * 1024.0;
    let mut teleport_camera = scene_camera(size, center, radius, None, None);
    teleport_camera.view = tokimu_core::math::try_view_look_at_rh(
        teleport_position,
        teleport_position + scene.spawn_observer.forward * 128.0,
        Vec3::Y,
    )
    .expect("camera basis must be finite and non-degenerate");
    poses.push(("declared-teleport-forward-1024", teleport_camera.view));

    let mut prior = None::<Vec<bool>>;
    let mut prior_expanded = None::<Vec<bool>>;
    for (frame, (label, view)) in poses.into_iter().enumerate() {
        let view_projection = base_camera.projection * view;
        let fresh_started = Instant::now();
        let fresh = bounds
            .iter()
            .copied()
            .map(|bounds| {
                bounds.is_none_or(|bounds| {
                    classify_static_draw_frustum_rejection(bounds, view_projection).is_none()
                })
            })
            .collect::<Vec<_>>();
        let fresh_cpu_us = fresh_started.elapsed().as_micros();
        let fresh_submitted = fresh.iter().filter(|selected| **selected).count();
        let expanded = bounds
            .iter()
            .copied()
            .map(|bounds| {
                bounds.is_none_or(|bounds| {
                    classify_static_draw_frustum_rejection(bounds, expanded_projection * view)
                        .is_none()
                })
            })
            .collect::<Vec<_>>();
        let expanded_submitted = expanded.iter().filter(|selected| **selected).count();
        let expanded_contains_fresh = fresh
            .iter()
            .zip(&expanded)
            .all(|(fresh, expanded)| !fresh || *expanded);
        assert!(
            expanded_contains_fresh,
            "expanded-frustum corpus trial must retain every fresh candidate"
        );
        let (prior_submitted, overlap, newly_visible, no_longer_visible, carried_submitted) =
            if let Some(prior) = &prior {
                let prior_submitted = prior.iter().filter(|selected| **selected).count();
                let overlap = prior
                    .iter()
                    .zip(&fresh)
                    .filter(|(prior, fresh)| **prior && **fresh)
                    .count();
                let newly_visible = fresh
                    .iter()
                    .zip(prior)
                    .filter(|(fresh, prior)| **fresh && !**prior)
                    .count();
                let no_longer_visible = prior
                    .iter()
                    .zip(&fresh)
                    .filter(|(prior, fresh)| **prior && !**fresh)
                    .count();
                let carried_submitted = prior
                    .iter()
                    .zip(&fresh)
                    .filter(|(prior, fresh)| **prior || **fresh)
                    .count();
                (
                    prior_submitted,
                    overlap,
                    newly_visible,
                    no_longer_visible,
                    carried_submitted,
                )
            } else {
                (0, 0, fresh_submitted, 0, fresh_submitted)
            };
        let expanded_prior_overlap = prior_expanded.as_ref().map_or(0, |prior_expanded| {
            prior_expanded
                .iter()
                .zip(&expanded)
                .filter(|(prior, expanded)| **prior && **expanded)
                .count()
        });
        println!(
            "E1M1 AR-0025 temporal carry: frame={frame}; pose={label}; candidates={}; fresh_submitted={fresh_submitted}; expanded_submitted={expanded_submitted}; expanded_contains_fresh={expanded_contains_fresh}; expanded_prior_overlap={expanded_prior_overlap}; prior_submitted={prior_submitted}; overlap={overlap}; newly_visible={newly_visible}; no_longer_visible={no_longer_visible}; carried_submitted={carried_submitted}; fresh_aabb_cpu_us={fresh_cpu_us}; authoritative_fresh_classification=true; abrupt_or_teleport_fallback=true; cutouts_enabled={include_cutouts}",
            bounds.len(),
        );
        prior = Some(fresh);
        prior_expanded = Some(expanded);
    }
}

/// A small source-neutral fixture of interleaved off-frustum, crossing, and
/// overlapping bounds. Interleaving intentionally stresses aggregate bounds:
/// coarse contiguous groups must fail open even though many individual draws
/// are safely rejectable.
pub(crate) fn report_pathological_candidate_fixture() {
    let mut bounds = Vec::with_capacity(128);
    for index in 0..32 {
        let offset = index as f32 * 0.001;
        bounds.extend([
            fixture_bounds([-4.0, -0.5 + offset, -0.5], [-3.0, 0.5 + offset, 0.5]),
            fixture_bounds([-0.5, -0.5 + offset, 3.0], [0.5, 0.5 + offset, 4.0]),
            fixture_bounds([-2.0, -0.25 + offset, -0.25], [0.25, 0.25 + offset, 0.25]),
            fixture_bounds([-0.25, -0.25 + offset, -0.25], [0.25, 0.25 + offset, 0.25]),
        ]);
    }
    let per_draw = bounds
        .iter()
        .copied()
        .filter(|bounds| classify_static_draw_frustum_rejection(*bounds, Mat4::IDENTITY).is_none())
        .count();
    println!(
        "AR-0025 pathological fixture: policy=per-draw-aabb; candidates={}; submitted={per_draw}; rejected={}",
        bounds.len(),
        bounds.len() - per_draw,
    );
    let optional_bounds = bounds.iter().copied().map(Some).collect::<Vec<_>>();
    for group_size in [8, 32] {
        let started = Instant::now();
        let summary =
            summarize_grouped_aabb_selection(&optional_bounds, Mat4::IDENTITY, group_size);
        println!(
            "AR-0025 pathological fixture: policy=contiguous-group-{group_size}; candidate_draws={}; groups={}; rejected_groups={}; submitted_draws={}; selection_cpu_us={}",
            bounds.len(),
            summary.groups,
            summary.rejected_groups,
            summary.submitted_draws,
            started.elapsed().as_micros(),
        );
    }
}

pub(crate) fn fixture_bounds(minimum: [f32; 3], maximum: [f32; 3]) -> StaticDrawAabb {
    StaticDrawAabb::from_positions(&[minimum, maximum])
        .expect("pathological fixture bounds must be finite")
}
