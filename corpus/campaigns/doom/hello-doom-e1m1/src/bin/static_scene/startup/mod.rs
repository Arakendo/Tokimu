//! Native E1M1 composition startup and command-line dispatch.
//!
//! Strategy implementations remain in `render_strategies`; this subject
//! parses the corpus controls and wires the selected subjects together.

use super::*;

pub(crate) fn run() -> PlatformResult<()> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let preserve_east = args.iter().any(|argument| argument == "--embedding-east");
    let preserve_north = args.iter().any(|argument| argument == "--embedding-north");
    let current_reflected = args
        .iter()
        .any(|argument| argument == "--embedding-current-reflected");
    let comparative_embedding = match (preserve_east, preserve_north, current_reflected) {
        (false, false, false) => DoomComparativeEmbedding::PreserveNorth,
        (false, false, true) => DoomComparativeEmbedding::CurrentReflected,
        (true, false, false) => DoomComparativeEmbedding::PreserveEast,
        (false, true, false) => DoomComparativeEmbedding::PreserveNorth,
        _ => return Err("choose only one comparative embedding".into()),
    };
    let include_cutouts = !args
        .iter()
        .any(|argument| argument == "--no-masked-cutouts");

    let diagnostic_sky = args
        .iter()
        .any(|argument| argument == "--diagnostic-sky-omissions");
    let doom_sky = !diagnostic_sky && !args.iter().any(|argument| argument == "--no-doom-sky");
    let source_sky_plane_depth = doom_sky
        && args
            .iter()
            .any(|argument| argument == "--source-sky-plane-depth");
    let source_sky_plane_depth_global_control = doom_sky
        && args
            .iter()
            .any(|argument| argument == "--source-sky-plane-depth-global-control");
    let spawn_observer = !args.iter().any(|argument| argument == "--overview-camera");
    let spawn_yaw_plus_90 = args
        .iter()
        .any(|argument| argument == "--spawn-yaw-plus-90");
    let walk_collision = !args
        .iter()
        .any(|argument| argument == "--no-walk-collision");
    let walk_collision_report = args
        .iter()
        .any(|argument| argument == "--walk-collision-report");
    let noclip = args.iter().any(|argument| argument == "--noclip");
    let frustum_aabb = args.iter().any(|argument| argument == "--frustum-aabb");
    let frustum_grid = args
        .iter()
        .any(|argument| argument == "--frustum-grid-8x4x8");
    let candidate_report = args.iter().any(|argument| argument == "--candidate-report");
    let candidate_turn_trace = args
        .iter()
        .any(|argument| argument == "--candidate-turn-trace");
    let candidate_position_trace = args
        .iter()
        .any(|argument| argument == "--candidate-position-trace");
    let candidate_pathological = args
        .iter()
        .any(|argument| argument == "--candidate-pathological-report");
    let candidate_grid_report = args
        .iter()
        .any(|argument| argument == "--candidate-grid-report");
    let candidate_temporal_report = args
        .iter()
        .any(|argument| argument == "--candidate-temporal-report");
    let doom_reject_report = args
        .iter()
        .any(|argument| argument == "--doom-reject-report");
    let doom_topology_report = args
        .iter()
        .any(|argument| argument == "--doom-topology-report");
    let doom_membership_report = args
        .iter()
        .any(|argument| argument == "--doom-membership-report");
    let doom_membership_union = args
        .iter()
        .any(|argument| argument == "--doom-membership-union");
    let flat_normal_report = args
        .iter()
        .any(|argument| argument == "--flat-normal-report");

    let special_activation_report = args
        .iter()
        .any(|argument| argument == "--special-activation-report");
    let door_runtime_report = args
        .iter()
        .any(|argument| argument == "--door-runtime-report");
    let moving_floor_runtime_report = args
        .iter()
        .any(|argument| argument == "--moving-floor-runtime-report");
    let moving_floor_resource_replay_report = args
        .iter()
        .any(|argument| argument == "--moving-floor-resource-replay-report");
    let door_resource_replay_report = args
        .iter()
        .any(|argument| argument == "--door-resource-replay-report");
    let measure_two_frames = args
        .iter()
        .any(|argument| argument == "--measure-two-frames");
    let spatial_orientation_report = args
        .iter()
        .any(|argument| argument == "--spatial-orientation-report");
    let spatial_landmark_candidates_report = args
        .iter()
        .any(|argument| argument == "--spatial-landmark-candidates-report");
    let spatial_flat_uv_report = args
        .iter()
        .any(|argument| argument == "--spatial-flat-uv-report");
    let hut_wall_candidates_report = args
        .iter()
        .any(|argument| argument == "--hut-wall-candidates-report");
    let doom_seg_report = args.iter().any(|argument| argument == "--doom-seg-report");
    let doom_seg_clip_report = args
        .iter()
        .any(|argument| argument == "--doom-seg-clip-report");
    let doom_hut_clip_report = args
        .iter()
        .any(|argument| argument == "--doom-hut-clip-report");
    let doom_seg_clip_grid_report = args
        .iter()
        .any(|argument| argument == "--doom-seg-clip-2d-report");
    let doom_seg_clip_per_column_report = args
        .iter()
        .any(|argument| argument == "--doom-seg-clip-per-column-report");
    let doom_seg_per_column_turn_trace = args
        .iter()
        .any(|argument| argument == "--doom-seg-per-column-turn-trace");
    let doom_seg_per_column_position_trace = args
        .iter()
        .any(|argument| argument == "--doom-seg-per-column-position-trace");
    let doom_seg_per_column_failure_trace = args
        .iter()
        .any(|argument| argument == "--doom-seg-per-column-failure-trace");
    let doom_seg_per_column_order_trace = args
        .iter()
        .any(|argument| argument == "--doom-seg-per-column-order-trace");
    let doom_seg_classic_admission_trace = args
        .iter()
        .any(|argument| argument == "--doom-seg-classic-admission-trace");
    let doom_seg_classic_bsp_trace = args
        .iter()
        .any(|argument| argument == "--doom-seg-classic-bsp-trace");
    let doom_seg_classic_vertical_clip_trace = args
        .iter()
        .any(|argument| argument == "--doom-seg-classic-vertical-clip-trace");
    let doom_seg_classic_plane_identity_trace = args
        .iter()
        .any(|argument| argument == "--doom-seg-classic-plane-identity-trace");
    let doom_seg_classic_plane_span_trace = args
        .iter()
        .any(|argument| argument == "--doom-seg-classic-plane-span-trace");
    let doom_seg_classic_plane_presentation = args
        .iter()
        .any(|argument| argument == "--doom-seg-classic-plane-presentation");
    let doom_seg_classic_context_presentation = args
        .iter()
        .any(|argument| argument == "--doom-seg-classic-context-presentation");
    let doom_seg_ordered_coverage_report = args
        .iter()
        .any(|argument| argument == "--doom-seg-ordered-coverage-report");
    let doom_seg_ordered_coverage_pose_matrix = args
        .iter()
        .any(|argument| argument == "--doom-seg-ordered-coverage-pose-matrix");
    let doom_seg_ordered_coverage_presentation = args
        .iter()
        .any(|argument| argument == "--doom-seg-ordered-coverage-presentation");
    let trial_render_strategy = TrialRenderStrategy::from_args(
        &args,
        doom_seg_ordered_coverage_presentation,
        frustum_aabb,
    )?;
    let doom_seg_clip_presentation = args
        .iter()
        .any(|argument| argument == "--doom-seg-clip-presentation");
    let doom_seg_per_column_presentation = args
        .iter()
        .any(|argument| argument == "--doom-seg-per-column-presentation");
    let doom_seg_per_column_dynamic = args
        .iter()
        .any(|argument| argument == "--doom-seg-per-column-dynamic");
    let doom_seg_classic_dynamic = args
        .iter()
        .any(|argument| argument == "--doom-seg-classic-dynamic");
    if [
        doom_seg_clip_presentation,
        doom_seg_per_column_presentation,
        doom_seg_per_column_dynamic,
        doom_seg_classic_dynamic,
        doom_seg_classic_plane_presentation,
        doom_seg_classic_context_presentation,
        doom_seg_ordered_coverage_presentation,
    ]
    .iter()
    .filter(|enabled| **enabled)
    .count()
        > 1
    {
        return Err("choose only one Stage 3B SEG presentation control".into());
    }
    if trial_render_strategy.is_some()
        && [
            doom_seg_clip_presentation,
            doom_seg_per_column_presentation,
            doom_seg_per_column_dynamic,
            doom_seg_classic_dynamic,
            doom_seg_classic_plane_presentation,
            doom_seg_classic_context_presentation,
        ]
        .iter()
        .any(|enabled| *enabled)
    {
        return Err(
            "A/B/C render strategies cannot be combined with legacy Stage 3B presentation controls"
                .into(),
        );
    }
    let wall_source_report = args.iter().find_map(|argument| {
        argument
            .strip_prefix("--wall-source-report=")
            .and_then(|record| record.parse::<u32>().ok())
    });
    let look_ray_report = args
        .iter()
        .find_map(|argument| argument.strip_prefix("--look-ray-report="))
        .map(parse_source_look_ray)
        .transpose()?;
    args.retain(|argument| argument != "--masked-cutouts");
    args.retain(|argument| argument != "--no-masked-cutouts");
    args.retain(|argument| argument != "--diagnostic-sky-omissions");
    args.retain(|argument| argument != "--doom-sky");
    args.retain(|argument| argument != "--no-doom-sky");
    args.retain(|argument| argument != "--source-sky-plane-depth");
    args.retain(|argument| argument != "--source-sky-plane-depth-global-control");
    args.retain(|argument| argument != "--spawn-observer");
    args.retain(|argument| argument != "--overview-camera");
    args.retain(|argument| argument != "--spawn-yaw-plus-90");
    args.retain(|argument| argument != "--walk-collision");
    args.retain(|argument| argument != "--no-walk-collision");
    args.retain(|argument| argument != "--walk-collision-report");
    args.retain(|argument| argument != "--noclip");
    args.retain(|argument| argument != "--frustum-aabb");
    args.retain(|argument| argument != "--frustum-grid-8x4x8");
    args.retain(|argument| argument != "--candidate-report");
    args.retain(|argument| argument != "--candidate-turn-trace");
    args.retain(|argument| argument != "--candidate-position-trace");
    args.retain(|argument| argument != "--candidate-pathological-report");
    args.retain(|argument| argument != "--candidate-grid-report");
    args.retain(|argument| argument != "--candidate-temporal-report");
    args.retain(|argument| argument != "--doom-reject-report");
    args.retain(|argument| argument != "--doom-topology-report");
    args.retain(|argument| argument != "--doom-membership-report");
    args.retain(|argument| argument != "--doom-membership-union");
    args.retain(|argument| argument != "--flat-normal-report");
    args.retain(|argument| argument != "--special-activation-report");
    args.retain(|argument| argument != "--door-runtime-report");
    args.retain(|argument| argument != "--moving-floor-runtime-report");
    args.retain(|argument| argument != "--moving-floor-resource-replay-report");
    args.retain(|argument| argument != "--door-resource-replay-report");
    args.retain(|argument| argument != "--measure-two-frames");
    args.retain(|argument| argument != "--spatial-orientation-report");
    args.retain(|argument| argument != "--spatial-landmark-candidates-report");
    args.retain(|argument| argument != "--spatial-flat-uv-report");
    args.retain(|argument| argument != "--hut-wall-candidates-report");
    args.retain(|argument| argument != "--doom-seg-report");
    args.retain(|argument| argument != "--doom-seg-clip-report");
    args.retain(|argument| argument != "--doom-hut-clip-report");
    args.retain(|argument| argument != "--doom-seg-clip-2d-report");
    args.retain(|argument| argument != "--doom-seg-clip-per-column-report");
    args.retain(|argument| argument != "--doom-seg-per-column-turn-trace");
    args.retain(|argument| argument != "--doom-seg-per-column-position-trace");
    args.retain(|argument| argument != "--doom-seg-per-column-failure-trace");
    args.retain(|argument| argument != "--doom-seg-per-column-order-trace");
    args.retain(|argument| argument != "--doom-seg-classic-admission-trace");
    args.retain(|argument| argument != "--doom-seg-classic-bsp-trace");
    args.retain(|argument| argument != "--doom-seg-classic-vertical-clip-trace");
    args.retain(|argument| argument != "--doom-seg-classic-plane-identity-trace");
    args.retain(|argument| argument != "--doom-seg-classic-plane-span-trace");
    args.retain(|argument| argument != "--doom-seg-classic-plane-presentation");
    args.retain(|argument| argument != "--doom-seg-classic-context-presentation");
    args.retain(|argument| argument != "--doom-seg-ordered-coverage-report");
    args.retain(|argument| argument != "--doom-seg-ordered-coverage-pose-matrix");
    args.retain(|argument| argument != "--doom-seg-ordered-coverage-presentation");

    args.retain(|argument| argument != "--doom-seg-clip-presentation");
    args.retain(|argument| argument != "--doom-seg-per-column-presentation");
    args.retain(|argument| argument != "--doom-seg-per-column-dynamic");
    args.retain(|argument| argument != "--doom-seg-classic-dynamic");
    render_strategies::remove_cli_args(&mut args);
    args.retain(|argument| !argument.starts_with("--wall-source-report="));
    args.retain(|argument| !argument.starts_with("--look-ray-report="));
    args.retain(|argument| argument != "--embedding-east");
    args.retain(|argument| argument != "--embedding-north");
    args.retain(|argument| argument != "--embedding-current-reflected");
    let [package, member] = args.as_slice() else {
        return Err(
            "usage: static_scene <canonical-doom-zip> <WAD-member-name> [--render-strategy=a|b|c|global-full-submission|prepared-full-submission|prepared-frustum-filtered] [--no-masked-cutouts] [--no-doom-sky|--diagnostic-sky-omissions] [--source-sky-plane-depth|--source-sky-plane-depth-global-control] [--overview-camera] [--spawn-yaw-plus-90] [--embedding-current-reflected|--embedding-east|--embedding-north] [--no-walk-collision] [--walk-collision-report] [--noclip] [--frustum-aabb] [--frustum-grid-8x4x8] [--doom-membership-union] [--doom-seg-per-column-dynamic|--doom-seg-classic-dynamic] [--candidate-report] [--candidate-turn-trace] [--candidate-position-trace] [--candidate-pathological-report] [--candidate-grid-report] [--candidate-temporal-report] [--doom-reject-report] [--doom-topology-report] [--doom-membership-report] [--doom-seg-report] [--doom-seg-classic-admission-trace|--doom-seg-classic-bsp-trace|--doom-seg-classic-vertical-clip-trace|--doom-seg-classic-plane-identity-trace|--doom-seg-classic-plane-span-trace|--doom-seg-classic-plane-presentation|--doom-seg-classic-context-presentation|--doom-seg-ordered-coverage-report|--doom-seg-ordered-coverage-pose-matrix|--doom-seg-ordered-coverage-presentation] [--flat-normal-report] [--special-activation-report] [--door-runtime-report] [--moving-floor-runtime-report|--moving-floor-resource-replay-report] [--door-resource-replay-report] [--spatial-orientation-report] [--spatial-landmark-candidates-report] [--spatial-flat-uv-report] [--hut-wall-candidates-report] [--wall-source-report=<linedef>] [--look-ray-report=<source-x,source-y,source-z,direction-x,direction-y,direction-z>] [--measure-two-frames]".into(),
        );
    };
    if (walk_collision || walk_collision_report) && !spawn_observer {
        return Err(
            "--walk-collision requires the source-spawn camera; omit --overview-camera".into(),
        );
    }
    if doom_seg_per_column_dynamic && !spawn_observer {
        return Err(
            "--doom-seg-per-column-dynamic requires the source-spawn observer; omit --overview-camera"
                .into(),
        );
    }
    if doom_seg_classic_dynamic && !spawn_observer {
        return Err(
            "--doom-seg-classic-dynamic requires the source-spawn observer; omit --overview-camera"
                .into(),
        );
    }
    if doom_seg_classic_plane_presentation && !spawn_observer {
        return Err(
            "--doom-seg-classic-plane-presentation requires the source-spawn observer; omit --overview-camera"
                .into(),
        );
    }
    if doom_seg_classic_context_presentation && !spawn_observer {
        return Err(
            "--doom-seg-classic-context-presentation requires the source-spawn observer; omit --overview-camera"
                .into(),
        );
    }
    if (doom_seg_ordered_coverage_report
        || matches!(
            trial_render_strategy,
            Some(
                TrialRenderStrategy::PreparedFullSubmission
                    | TrialRenderStrategy::PreparedFrustumFiltered
            )
        ))
        && !spawn_observer
    {
        return Err(
            "ordered coverage comparison requires the source-spawn observer; omit --overview-camera"
                .into(),
        );
    }
    if comparative_embedding != DoomComparativeEmbedding::CurrentReflected && walk_collision_report
    {
        return Err(
            "comparative embeddings currently exclude the source-space collision report replay; use interactive --walk-collision for converted correspondence evidence"
                .into(),
        );
    }
    let mut scene = prepare_scene(package, member)?;
    if spatial_orientation_report {
        report_spatial_orientation(&scene);
        return Ok(());
    }
    if spatial_landmark_candidates_report {
        report_spatial_landmark_candidates(&scene);
        return Ok(());
    }
    if hut_wall_candidates_report {
        report_hut_wall_candidates(&scene);
        return Ok(());
    }
    if doom_seg_report {
        report_doom_seg_lowering(&scene)?;
        return Ok(());
    }
    if doom_seg_clip_report {
        report_doom_seg_screen_clip(&scene, false)?;
        return Ok(());
    }
    if doom_hut_clip_report {
        report_doom_seg_screen_clip(&scene, true)?;
        return Ok(());
    }
    if doom_seg_clip_grid_report {
        report_doom_seg_screen_grid(&scene, false)?;
        return Ok(());
    }
    if doom_seg_clip_per_column_report {
        report_doom_seg_screen_grid(&scene, true)?;
        return Ok(());
    }
    if doom_seg_per_column_turn_trace {
        report_doom_seg_per_column_turn_trace(&scene)?;
        return Ok(());
    }
    if doom_seg_per_column_position_trace {
        report_doom_seg_per_column_position_trace(&scene)?;
        return Ok(());
    }
    if doom_seg_per_column_failure_trace {
        report_doom_seg_per_column_failure_trace(&scene)?;
        return Ok(());
    }
    if doom_seg_per_column_order_trace {
        report_doom_seg_per_column_order_trace(&scene)?;

        return Ok(());
    }
    if doom_seg_classic_admission_trace {
        report_doom_seg_classic_admission_trace(&scene)?;
        return Ok(());
    }
    if doom_seg_classic_bsp_trace {
        report_doom_seg_classic_bsp_trace(&scene)?;
        return Ok(());
    }
    if doom_seg_classic_vertical_clip_trace {
        report_doom_seg_classic_vertical_clip_trace(&scene)?;
        return Ok(());
    }

    if doom_seg_classic_plane_identity_trace {
        report_doom_seg_classic_plane_identity_trace(&scene)?;
        return Ok(());
    }
    if doom_seg_classic_plane_span_trace {
        report_doom_seg_classic_plane_span_trace(&scene)?;
        return Ok(());
    }
    if doom_seg_ordered_coverage_report {
        let presentation = prepare_doom_seg_ordered_coverage_presentation(&scene)?;
        eprintln!(
            "E1M1 AR-0025 Slice 7 ordered-coverage report: wall-conservation=[retained-cells:{} reconstructed-triangles:{} lowered-triangles:{} source-degenerate-cells:{} source-unresolved-cells:{} lowering-degenerate-triangles:{} lowering-unresolved-triangles:{}]; grouped-wall-meshes={}; opaque-draws={}; cutout-draws={}; plane-conservation=[ordinary:{} reconstructed:{} rejected:{} lowered:{}]; sky-background-intervals={}; cutout-key-conservation={}/{}; coverage=[transitions:{} fail-open:{} reasons:{:?}]; bsp=[leaves:{} far-pruned:{} admitted-segs:{} solid-range-pruning:{}]; degenerate-omissions={}; unresolved-contributions={}; samples={:?}; meaning=one-fixed-source-observation-lowered-to-complete-prepared-declarations",
            presentation.retained_cells,
            presentation.reconstructed_triangles,
            presentation.lowered_wall_triangles,
            presentation.source_degenerate_cells,
            presentation.source_unresolved_cells,
            presentation.lowering_degenerate_triangles,
            presentation.lowering_unresolved_triangles,
            presentation.grouped_wall_meshes,
            presentation.opaque_draws.len(),
            presentation.cutout_draws.len(),
            presentation.ordinary_plane_intervals,
            presentation.reconstructed_plane_quads,
            presentation.rejected_plane_intervals,
            presentation.lowered_plane_quads,
            presentation.sky_plane_intervals,
            presentation.lowered_cutout_keys,
            presentation.source_cutout_keys,
            presentation.coverage_transitions,
            presentation.coverage_fail_open,
            presentation.coverage_fail_open_reasons,
            presentation.bsp_leaves_visited,
            presentation.bsp_far_children_pruned,
            presentation.bsp_admitted_segs,
            presentation.bsp_solid_range_pruning,
            presentation.degenerate_omissions,
            presentation.unresolved_cells,
            presentation.samples,
        );
        return Ok(());
    }
    if doom_seg_ordered_coverage_pose_matrix {
        report_doom_seg_ordered_coverage_pose_matrix(&scene)?;
        return Ok(());
    }
    if let Some(linedef) = wall_source_report {
        report_wall_source(&scene, linedef);
        return Ok(());
    }
    if doom_seg_clip_presentation {
        render_strategies::legacy_comparisons::visible_seg::apply(&mut scene)?;
    }
    if doom_seg_per_column_presentation {
        render_strategies::legacy_comparisons::per_column::apply(&mut scene)?;
    }
    if doom_seg_classic_plane_presentation {
        render_strategies::legacy_comparisons::classic_planes::apply(&mut scene)?;
    }
    if doom_seg_classic_context_presentation {
        render_strategies::legacy_comparisons::classic_context::apply(&mut scene)?;
    }
    let ordered_coverage_source = trial_render_strategy
        .filter(|strategy| {
            matches!(
                strategy,
                TrialRenderStrategy::PreparedFullSubmission
                    | TrialRenderStrategy::PreparedFrustumFiltered
            )
        })
        .map(|_| Box::new(scene.clone()));
    let ordered_coverage_camera_bounds = ordered_coverage_source.as_ref().map(|source| {
        let draws = source
            .opaque_draws
            .iter()
            .chain(source.cutout_draws.iter())
            .cloned()
            .collect::<Vec<_>>();
        scene_bounds(&draws)
    });
    let applied_trial_strategy = trial_render_strategy
        .map(|strategy| strategy.apply(&mut scene))
        .transpose()?;
    let doom_seg_dynamic_selection = if doom_seg_per_column_dynamic || doom_seg_classic_dynamic {
        let selection = prepare_doom_seg_per_column_dynamic_scene(&mut scene)?;
        eprintln!(
            "E1M1 AR-0025 Stage 3B dynamic SEG control: mode={}; retained_seg_records={}; retained_flat_subsectors={}; unsupported_textures={:?}; meaning=source-local-draw-enable-experiment-not-renderer-visibility",
            if doom_seg_classic_dynamic { "classic-bsp" } else { "per-column-grid" },
            selection.draw_indices_by_seg.len(),
            selection.flat_indices_by_subsector.len(),
            selection.unsupported_textures,
        );
        Some(selection)
    } else {
        None
    };
    reembed_scene_for_comparison(&mut scene, comparative_embedding);
    if let Some(ray) = look_ray_report {
        report_source_look_ray(&scene, comparative_embedding, ray, include_cutouts);
        return Ok(());
    }
    if spatial_flat_uv_report {
        report_spatial_flat_uv(&scene, comparative_embedding);
        return Ok(());
    }
    let bounds_draws = scene
        .opaque_draws
        .iter()
        .chain(scene.cutout_draws.iter())
        .cloned()
        .collect::<Vec<_>>();
    let (center, radius) =
        ordered_coverage_camera_bounds.unwrap_or_else(|| scene_bounds(&bounds_draws));
    if candidate_report {
        report_candidate_selection(&scene, include_cutouts, center, radius);
        return Ok(());
    }
    if candidate_turn_trace {
        report_candidate_turn_trace(&scene, include_cutouts, center, radius);
        return Ok(());
    }
    if candidate_position_trace {
        report_candidate_position_trace(&scene, include_cutouts, center, radius);
        return Ok(());
    }
    if candidate_pathological {
        report_pathological_candidate_fixture();
        return Ok(());
    }
    if candidate_grid_report {
        report_uniform_grid_selection(&scene, include_cutouts, center, radius);
        return Ok(());
    }
    if candidate_temporal_report {
        report_temporal_candidate_carry(&scene, include_cutouts, center, radius);
        return Ok(());
    }
    if doom_reject_report {
        report_doom_reject(&scene.reject_report);
        return Ok(());
    }

    if doom_topology_report {
        report_doom_topology(&scene.topology_report);
        return Ok(());
    }
    if doom_membership_report {
        report_doom_membership_union(&scene, center, radius, include_cutouts);
        return Ok(());
    }
    if flat_normal_report {
        report_flat_normals(&scene.opaque_draws);
        return Ok(());
    }
    if special_activation_report {
        report_doom_use_activation(&scene.activation_source);
        return Ok(());
    }
    if door_runtime_report {
        report_doom_manual_door_runtime(&scene.activation_source);
        return Ok(());
    }
    if moving_floor_runtime_report {
        report_doom_moving_floor_runtime(&scene.activation_source);
        return Ok(());
    }
    if walk_collision_report {
        report_walk_collision(&scene);
        return Ok(());
    }
    let include_cutouts = include_cutouts && !doom_seg_clip_presentation;
    let opaque_bounds = draw_bounds(&scene.opaque_draws);
    let cutout_bounds = draw_bounds(&scene.cutout_draws);
    let opaque_grid = frustum_grid
        .then(|| UniformGridAabbIndex::build(&opaque_bounds, [8, 4, 8]))
        .flatten();
    let cutout_grid = frustum_grid
        .then(|| UniformGridAabbIndex::build(&cutout_bounds, [8, 4, 8]))
        .flatten();
    let draw_count = scene.opaque_draws.len()
        + if include_cutouts {
            scene.cutout_draws.len()
        } else {
            0
        }
        + if diagnostic_sky {
            scene.diagnostic_sky_draws.len()
        } else {
            0
        }
        // Paired-sky boundary meshes remain retained source evidence, but
        // E1M1 proved that presenting them unconditionally can hide valid
        // foreground geometry (the hut from the spawn-room window). The
        // synthetic paired-sky fixture remains the bounded mechanism control.
        + if source_sky_plane_depth_global_control {
            scene.diagnostic_sky_draws.len()
        } else if source_sky_plane_depth {
            1
        } else {
            0
        }
        + usize::from(doom_sky);

    let opaque_selected = vec![true; scene.opaque_draws.len()];
    let cutout_selected = vec![true; scene.cutout_draws.len()];
    let source_sky_plane_selected = vec![false; scene.diagnostic_sky_draws.len()];
    let cutout_mesh_base = if ordered_coverage_source.is_some() {
        ORDERED_COVERAGE_CUTOUT_MESH_BASE
    } else {
        scene.opaque_draws.len() as u64 + 1
    };
    let commands = Vec::with_capacity(draw_count + 1);
    let mut app = App {
        renderer: None,
        draws: scene.opaque_draws,
        uploads: scene.opaque_uploads,
        cutout_draws: scene.cutout_draws,
        cutout_uploads: scene.cutout_uploads,
        diagnostic_sky_draws: scene.diagnostic_sky_draws,
        diagnostic_sky_enabled: diagnostic_sky,
        diagnostic_sky_records: scene.diagnostic_sky_records,
        doom_sky_texture: scene.doom_sky_texture,
        doom_sky_mesh: build_doom_sky_cylinder(center, radius).map_err(io::Error::other)?,
        doom_sky_boundary_draws: scene.doom_sky_boundary_draws,
        doom_sky_enabled: doom_sky,
        source_sky_plane_depth_enabled: source_sky_plane_depth,
        source_sky_plane_depth_global_control,
        source_sky_plane_selected,
        cutout_mesh_base,
        include_cutouts,
        pipeline: PipelineHandle(0),
        cutout_pipeline: None,
        doom_sky_pipeline: None,
        doom_sky_boundary_pipeline: None,
        debug_pipeline: None,
        debug_font: None,
        debug_console: DoomDebugConsole::default(),
        size: [1280.0, 800.0],
        center,
        radius,
        spawn_observer: spawn_observer.then_some(scene.spawn_observer),
        initial_spawn_observer: spawn_observer.then_some(scene.spawn_observer),
        observer_look: spawn_observer.then_some(ObserverLook {
            yaw: observer_yaw_from_forward(scene.spawn_observer.forward)
                + if spawn_yaw_plus_90 {
                    std::f32::consts::FRAC_PI_2
                } else {
                    0.0
                },
            pitch: 0.0,
            last_cursor: None,
        }),
        initial_observer_look: spawn_observer.then_some(ObserverLook {
            yaw: observer_yaw_from_forward(scene.spawn_observer.forward)
                + if spawn_yaw_plus_90 {
                    std::f32::consts::FRAC_PI_2
                } else {
                    0.0
                },
            pitch: 0.0,
            last_cursor: None,
        }),
        walk_collision: walk_collision.then_some(scene.walk_collision),
        walk_floors: walk_collision.then_some(scene.walk_floors),
        noclip,
        last_collision_contacts: Vec::new(),
        last_floor_transition: None,
        opaque_bounds,
        cutout_bounds,
        opaque_grid,
        cutout_grid,
        membership_selection: scene.membership_selection,
        activation_source: scene.activation_source,
        door_geometry_source: scene.door_geometry_source,
        active_manual_doors: Vec::new(),
        door_tick_accumulator: 0.0,
        active_turbo_floors: Vec::new(),
        active_down_wait_up_platforms: Vec::new(),
        consumed_one_shot_cross_lines: BTreeSet::new(),
        moving_floor_tick_accumulator: 0.0,
        dirty_opaque_meshes: HashSet::new(),
        door_visual_diagnostic: None,
        door_geometry_diagnostic: None,
        dynamic_door_draws: BTreeSet::new(),
        dynamic_door_mesh_handles: BTreeMap::new(),
        next_dynamic_mesh_handle: if ordered_coverage_source.is_some() {
            ORDERED_COVERAGE_DYNAMIC_MESH_BASE
        } else {
            cutout_mesh_base + cutout_selected.len() as u64
        },
        opaque_draw_enabled: opaque_selected.clone(),
        candidate_selection: if let Some(applied) = applied_trial_strategy {
            applied.candidate_selection
        } else if doom_seg_classic_dynamic {
            CandidateSelection::DoomClassicBsp
        } else if frustum_grid {
            CandidateSelection::UniformGrid8x4x8
        } else if doom_membership_union {
            CandidateSelection::DoomMembershipUnion
        } else if doom_seg_per_column_dynamic {
            CandidateSelection::DoomSegPerColumn
        } else if frustum_aabb {
            CandidateSelection::FrustumAabb
        } else {
            CandidateSelection::FullSubmission
        },
        doom_seg_dynamic_selection,
        frame_index: 0,
        exit_after_two_frames: measure_two_frames,
        opaque_selected,
        cutout_selected,
        commands,
        window: None,
        mouse_captured: false,
        input: InputState::default(),
        comparative_embedding,
        ordered_coverage_prepared: applied_trial_strategy
            .is_some_and(|applied| applied.ordered_coverage_prepared),
        ordered_coverage_source,
        ordered_coverage_view: None,
        fixed_reconstruction_camera: applied_trial_strategy
            .is_some_and(|applied| applied.fixed_reconstruction_camera)
            || doom_seg_classic_plane_presentation
            || doom_seg_classic_context_presentation
            || doom_seg_ordered_coverage_presentation,
    };
    if door_resource_replay_report {
        report_door_resource_replay(&mut app)?;
        return Ok(());
    }
    if moving_floor_resource_replay_report {
        report_moving_floor_resource_replay(&mut app)?;
        return Ok(());
    }
    run_window_with_app(
        WindowConfig {
            title: format!(
                "Tokimu DOOM E1M1 | {draw_count} draws | {comparative_embedding:?}{}",
                if app.fixed_reconstruction_camera {
                    " | fixed-source-spawn"
                } else {
                    ""
                }
            ),
            width: 1280,
            height: 800,
        },
        app,
    )
}
