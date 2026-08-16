//! Deterministic camera-pose traces over existing candidate policies.

use super::super::*;

pub(crate) fn summarize_scene_aabb_selection(
    scene: &SceneInput,
    include_cutouts: bool,
    opaque_bounds: &[Option<StaticDrawAabb>],
    cutout_bounds: &[Option<StaticDrawAabb>],
    camera: Camera,
) -> CandidateSelectionSummary {
    let (mut summary, _) = summarize_candidate_selection(
        scene.opaque_draws.iter().zip(opaque_bounds.iter().copied()),
        camera,
        classify_static_draw_frustum_rejection,
    );
    if include_cutouts {
        let (cutout_summary, _) = summarize_candidate_selection(
            scene.cutout_draws.iter().zip(cutout_bounds.iter().copied()),
            camera,
            classify_static_draw_frustum_rejection,
        );
        summary.merge(cutout_summary);
    }
    summary
}

/// Deterministic in-place 360-degree source-spawn trace. It deliberately
/// changes only camera yaw: no player movement, topology, or runtime state is
/// inferred from the report.
pub(crate) fn report_candidate_turn_trace(
    scene: &SceneInput,
    include_cutouts: bool,
    center: Vec3,
    radius: f32,
) {
    let size = [1280.0, 800.0];
    let opaque_bounds = draw_bounds(&scene.opaque_draws);
    let cutout_bounds = draw_bounds(&scene.cutout_draws);
    let source_yaw = observer_yaw_from_forward(scene.spawn_observer.forward);
    let mut minimum_submitted = usize::MAX;
    let mut maximum_submitted = 0_usize;
    let mut total_submitted = 0_usize;
    let mut total_selection_cpu_us = 0_u128;
    for (frame, yaw_offset_degrees) in (0..=360).step_by(45).enumerate() {
        let yaw = source_yaw + (yaw_offset_degrees as f32).to_radians();
        let camera = scene_camera(
            size,
            center,
            radius,
            Some(scene.spawn_observer),
            Some(ObserverLook {
                yaw,
                pitch: 0.0,
                last_cursor: None,
            }),
        );
        let started = Instant::now();
        let summary = summarize_scene_aabb_selection(
            scene,
            include_cutouts,
            &opaque_bounds,
            &cutout_bounds,
            camera,
        );
        let selection_cpu_us = started.elapsed().as_micros();
        minimum_submitted = minimum_submitted.min(summary.submitted);
        maximum_submitted = maximum_submitted.max(summary.submitted);
        total_submitted += summary.submitted;
        total_selection_cpu_us += selection_cpu_us;
        println!(
            "E1M1 AR-0025 turn trace: frame={frame}; yaw_offset_degrees={yaw_offset_degrees}; candidates={}; rejected={}; submitted={}; uncertain_bounds={}; selection_cpu_us={selection_cpu_us}",
            summary.candidates,
            summary.rejected,
            summary.submitted,
            summary.uncertain_bounds,
        );
    }
    println!(
        "E1M1 AR-0025 turn trace summary: frames=9; candidates_per_frame={}; submitted_min={minimum_submitted}; submitted_max={maximum_submitted}; submitted_total={total_submitted}; selection_cpu_us_total={total_selection_cpu_us}; cutouts_enabled={include_cutouts}",
        scene.opaque_draws.len() + if include_cutouts { scene.cutout_draws.len() } else { 0 },
    );
}

/// Deterministic local-coordinate offsets from the reviewed source spawn. The
/// offsets are camera-test inputs only: they do not claim collision-safe Doom
/// movement, player state advancement, or a traversable source path.
pub(crate) fn report_candidate_position_trace(
    scene: &SceneInput,
    include_cutouts: bool,
    center: Vec3,
    radius: f32,
) {
    let size = [1280.0, 800.0];
    let opaque_bounds = draw_bounds(&scene.opaque_draws);
    let cutout_bounds = draw_bounds(&scene.cutout_draws);
    let opaque_spheres = draw_spheres(&scene.opaque_draws);
    let cutout_spheres = draw_spheres(&scene.cutout_draws);
    let mut ordered_bounds = opaque_bounds.clone();
    if include_cutouts {
        ordered_bounds.extend(cutout_bounds.iter().copied());
    }
    let forward = scene.spawn_observer.forward;
    let mut minimum_submitted = usize::MAX;
    let mut maximum_submitted = 0_usize;
    let mut total_submitted = 0_usize;
    for (frame, forward_offset) in [-256.0_f32, -128.0, 0.0, 128.0, 256.0]
        .into_iter()
        .enumerate()
    {
        let position = scene.spawn_observer.position + forward * forward_offset;
        let mut camera = scene_camera(size, center, radius, None, None);
        camera.view =
            tokimu_core::math::try_view_look_at_rh(position, position + forward * 128.0, Vec3::Y)
                .expect("camera basis must be finite and non-degenerate");
        let started = Instant::now();
        let summary = summarize_scene_aabb_selection(
            scene,
            include_cutouts,
            &opaque_bounds,
            &cutout_bounds,
            camera,
        );
        let aabb_selection_cpu_us = started.elapsed().as_micros();
        let started = Instant::now();
        let (mut sphere_summary, _) = summarize_candidate_selection(
            scene
                .opaque_draws
                .iter()
                .zip(opaque_spheres.iter().copied()),
            camera,
            classify_static_draw_sphere_frustum_rejection,
        );
        if include_cutouts {
            let (cutout_summary, _) = summarize_candidate_selection(
                scene
                    .cutout_draws
                    .iter()
                    .zip(cutout_spheres.iter().copied()),
                camera,
                classify_static_draw_sphere_frustum_rejection,
            );
            sphere_summary.merge(cutout_summary);
        }
        let sphere_selection_cpu_us = started.elapsed().as_micros();
        let view_projection = camera.projection * camera.view;
        let started = Instant::now();
        let group_8 = summarize_grouped_aabb_selection(&ordered_bounds, view_projection, 8);
        let group_8_selection_cpu_us = started.elapsed().as_micros();
        let started = Instant::now();
        let group_32 = summarize_grouped_aabb_selection(&ordered_bounds, view_projection, 32);
        let group_32_selection_cpu_us = started.elapsed().as_micros();
        minimum_submitted = minimum_submitted.min(summary.submitted);
        maximum_submitted = maximum_submitted.max(summary.submitted);
        total_submitted += summary.submitted;
        println!(
            "E1M1 AR-0025 position trace: frame={frame}; source=player-one-local-forward-offset; forward_offset={forward_offset}; camera=({:.1},{:.1},{:.1}); candidates={}; aabb_submitted={}; aabb_rejected={}; aabb_selection_cpu_us={aabb_selection_cpu_us}; sphere_submitted={}; sphere_rejected={}; sphere_selection_cpu_us={sphere_selection_cpu_us}; group_8_submitted={}; group_8_selection_cpu_us={group_8_selection_cpu_us}; group_32_submitted={}; group_32_selection_cpu_us={group_32_selection_cpu_us}; uncertain_bounds={}",
            position.x,
            position.y,
            position.z,
            summary.candidates,
            summary.submitted,
            summary.rejected,
            sphere_summary.submitted,
            sphere_summary.rejected,
            group_8.submitted_draws,
            group_32.submitted_draws,
            summary.uncertain_bounds,
        );
    }
    println!(
        "E1M1 AR-0025 position trace summary: frames=5; candidates_per_frame={}; submitted_min={minimum_submitted}; submitted_max={maximum_submitted}; submitted_total={total_submitted}; cutouts_enabled={include_cutouts}; movement_claim=none",
        scene.opaque_draws.len() + if include_cutouts { scene.cutout_draws.len() } else { 0 },
    );
}
