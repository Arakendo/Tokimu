//! Candidate-selection measurements and pathological controls.
//!
//! These reports observe existing candidate policies without owning scene
//! preparation or renderer submission.

use super::super::*;

pub(crate) fn report_candidate_selection(
    scene: &SceneInput,
    include_cutouts: bool,
    center: Vec3,
    radius: f32,
) {
    let opaque_bounds = draw_bounds(&scene.opaque_draws);
    let cutout_bounds = draw_bounds(&scene.cutout_draws);
    let opaque_spheres = draw_spheres(&scene.opaque_draws);
    let cutout_spheres = draw_spheres(&scene.cutout_draws);
    let mut ordered_bounds = opaque_bounds.clone();
    if include_cutouts {
        ordered_bounds.extend(cutout_bounds.iter().copied());
    }
    let size = [1280.0, 800.0];
    let spawn_look = ObserverLook {
        yaw: observer_yaw_from_forward(scene.spawn_observer.forward),
        pitch: 0.0,
        last_cursor: None,
    };
    let spawn_yaw = spawn_look.yaw;
    for (pose, camera) in [
        ("overview", scene_camera(size, center, radius, None, None)),
        (
            "source-spawn-forward",
            scene_camera(
                size,
                center,
                radius,
                Some(scene.spawn_observer),
                Some(spawn_look),
            ),
        ),
        (
            "source-spawn-yaw-plus-90",
            scene_camera(
                size,
                center,
                radius,
                Some(scene.spawn_observer),
                Some(ObserverLook {
                    yaw: spawn_yaw + std::f32::consts::FRAC_PI_2,
                    ..spawn_look
                }),
            ),
        ),
        (
            "source-spawn-yaw-plus-180",
            scene_camera(
                size,
                center,
                radius,
                Some(scene.spawn_observer),
                Some(ObserverLook {
                    yaw: spawn_yaw + std::f32::consts::PI,
                    ..spawn_look
                }),
            ),
        ),
        (
            "source-spawn-yaw-minus-90",
            scene_camera(
                size,
                center,
                radius,
                Some(scene.spawn_observer),
                Some(ObserverLook {
                    yaw: spawn_yaw - std::f32::consts::FRAC_PI_2,
                    ..spawn_look
                }),
            ),
        ),
    ] {
        let opaque = scene.opaque_draws.iter().zip(opaque_bounds.iter().copied());
        let cutouts = scene.cutout_draws.iter().zip(cutout_bounds.iter().copied());
        let selection_started = Instant::now();
        let (mut summary, mut samples) =
            summarize_candidate_selection(opaque, camera, classify_static_draw_frustum_rejection);
        let opaque_submitted = summary.submitted;
        let cutout_submitted = if include_cutouts {
            let (cutout_summary, cutout_samples) = summarize_candidate_selection(
                cutouts,
                camera,
                classify_static_draw_frustum_rejection,
            );
            let submitted = cutout_summary.submitted;
            summary.merge(cutout_summary);
            let remaining = 12usize.saturating_sub(samples.len());
            samples.extend(cutout_samples.into_iter().take(remaining));
            submitted
        } else {
            0
        };
        let selection_cpu_us = selection_started.elapsed().as_micros();
        println!(
            "E1M1 AR-0025 fixed-pose report: pose={pose}; policy=frustum-aabb; candidates={}; rejected={}; submitted={}; opaque_submitted={}; cutout_submitted={}; uncertain_bounds={}; selection_cpu_us={selection_cpu_us}; rejected_by_plane=[left:{},right:{},bottom:{},top:{},near:{},far:{}]; cutouts_enabled={}",
            summary.candidates,
            summary.rejected,
            summary.submitted,
            opaque_submitted,
            cutout_submitted,
            summary.uncertain_bounds,
            summary.rejected_by_plane[0],
            summary.rejected_by_plane[1],
            summary.rejected_by_plane[2],
            summary.rejected_by_plane[3],
            summary.rejected_by_plane[4],
            summary.rejected_by_plane[5],
            include_cutouts,
        );
        println!(
            "E1M1 AR-0025 bounded rejection samples: pose={pose}; shown={}; total_rejected={}; {}",
            samples.len(),
            summary.rejected,
            samples.join(" | "),
        );

        let opaque = scene
            .opaque_draws
            .iter()
            .zip(opaque_spheres.iter().copied());
        let cutouts = scene
            .cutout_draws
            .iter()
            .zip(cutout_spheres.iter().copied());
        let selection_started = Instant::now();
        let (mut summary, mut samples) = summarize_candidate_selection(
            opaque,
            camera,
            classify_static_draw_sphere_frustum_rejection,
        );
        let opaque_submitted = summary.submitted;
        let cutout_submitted = if include_cutouts {
            let (cutout_summary, cutout_samples) = summarize_candidate_selection(
                cutouts,
                camera,
                classify_static_draw_sphere_frustum_rejection,
            );
            let submitted = cutout_summary.submitted;
            summary.merge(cutout_summary);
            let remaining = 12usize.saturating_sub(samples.len());
            samples.extend(cutout_samples.into_iter().take(remaining));
            submitted
        } else {
            0
        };
        let selection_cpu_us = selection_started.elapsed().as_micros();
        println!(
            "E1M1 AR-0025 fixed-pose report: pose={pose}; policy=frustum-sphere; candidates={}; rejected={}; submitted={}; opaque_submitted={}; cutout_submitted={}; uncertain_bounds={}; selection_cpu_us={selection_cpu_us}; rejected_by_plane=[left:{},right:{},bottom:{},top:{},near:{},far:{}]; cutouts_enabled={}",
            summary.candidates,
            summary.rejected,
            summary.submitted,
            opaque_submitted,
            cutout_submitted,
            summary.uncertain_bounds,
            summary.rejected_by_plane[0],
            summary.rejected_by_plane[1],
            summary.rejected_by_plane[2],
            summary.rejected_by_plane[3],
            summary.rejected_by_plane[4],
            summary.rejected_by_plane[5],
            include_cutouts,
        );
        println!(
            "E1M1 AR-0025 bounded rejection samples: pose={pose}; policy=frustum-sphere; shown={}; total_rejected={}; {}",
            samples.len(),
            summary.rejected,
            samples.join(" | "),
        );

        let view_projection = camera.projection * camera.view;
        for group_size in [8, 32] {
            let selection_started = Instant::now();
            let summary =
                summarize_grouped_aabb_selection(&ordered_bounds, view_projection, group_size);
            let selection_cpu_us = selection_started.elapsed().as_micros();
            println!(
                "E1M1 AR-0025 fixed-pose report: pose={pose}; policy=frustum-aabb-contiguous-group-{group_size}; candidate_draws={}; groups={}; rejected_groups={}; submitted_groups={}; submitted_draws={}; uncertain_groups={}; selection_cpu_us={selection_cpu_us}; cutouts_enabled={}",
                ordered_bounds.len(),
                summary.groups,
                summary.rejected_groups,
                summary.submitted_groups,
                summary.submitted_draws,
                summary.uncertain_groups,
                include_cutouts,
            );
        }
    }
}
