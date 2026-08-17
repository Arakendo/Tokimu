//! Headless traces for evolving Doom source-protocol experiments.
//!
//! These retain comparative evidence and failure poses. They neither select
//! production renderer work nor define a stable Tokimu visibility contract.

use super::super::*;

/// Second Stage 3B control: preserves the same source-owned traversal and
/// occluder classification, but records coverage in a bounded two-dimensional
/// source-space grid. It is deliberately headless before any new presentation
/// mode is considered, because the horizontal-only control was falsified by
/// visible spawn-room omissions.
pub(crate) fn report_doom_seg_screen_grid(
    scene: &SceneInput,
    per_column: bool,
) -> PlatformResult<()> {
    let observation = observe_doom_seg_screen_grid(
        &scene.door_geometry_source.map,
        scene.spawn_observer.position.y,
        per_column,
        scene.spawn_observer.source_position,
        f64::from(scene.spawn_observer.source_angle).to_radians(),
    )?;
    println!(
        "E1M1 AR-0025 Stage 3B two-dimensional screen-grid control: mode={}; columns=320; rows=200; outside={}; fully_covered={}; partial={}; fully_visible={}; coverage_contributors={}; covered_cells={}; selected_segs={}; meaning=bounded-source-space-control-not-renderer-or-historic-doom-visibility",
        if per_column { "per-column" } else { "rectangle" },
        observation.outside,
        observation.fully_covered,
        observation.partial,
        observation.fully_visible,
        observation.contributors,
        observation.covered_cells,
        observation.selected_seg_records.len(),
    );
    println!(
        "E1M1 AR-0025 Stage 3B two-dimensional screen-grid samples: {}",
        observation.samples.join(" | ")
    );
    Ok(())
}

/// Bounded source-pose trace for the per-column Stage 3B control. It proves
/// only that the source-owned candidate set changes with declared camera input;
/// it neither uploads resources nor schedules dynamic renderer work.
pub(crate) fn report_doom_seg_per_column_turn_trace(scene: &SceneInput) -> PlatformResult<()> {
    let base = f64::from(scene.spawn_observer.source_angle);
    for offset in [0.0, 90.0, 180.0, 270.0] {
        let angle = (base + offset).rem_euclid(360.0).to_radians();
        let observation = observe_doom_seg_screen_grid(
            &scene.door_geometry_source.map,
            scene.spawn_observer.position.y,
            true,
            scene.spawn_observer.source_position,
            angle,
        )?;
        println!(
            "E1M1 AR-0025 Stage 3B per-column turn trace: source_heading_degrees={:.0}; selected_segs={}; outside={}; fully_covered={}; partial={}; fully_visible={}; covered_cells={}; depth_order_inversions={}; meaning=headless-source-pose-observation-not-dynamic-renderer-selection",
            (base + offset).rem_euclid(360.0),
            observation.selected_seg_records.len(),
            observation.outside,
            observation.fully_covered,
            observation.partial,
            observation.fully_visible,
            observation.covered_cells,
            observation.depth_order_inversions,
        );
    }
    Ok(())
}

/// Bounded position counterpart to the heading trace. The offsets are source
/// test inputs around player one, not collision-valid movement or Doom play.
pub(crate) fn report_doom_seg_per_column_position_trace(scene: &SceneInput) -> PlatformResult<()> {
    let origin = scene.spawn_observer.source_position;
    let angle = f64::from(scene.spawn_observer.source_angle).to_radians();
    for [offset_x, offset_y] in [[0, 0], [64, 0], [-64, 0], [0, 64], [0, -64], [128, 64]] {
        let viewer = [origin[0] + offset_x, origin[1] + offset_y];
        let observation = observe_doom_seg_screen_grid(
            &scene.door_geometry_source.map,
            scene.spawn_observer.position.y,
            true,
            viewer,
            angle,
        )?;
        println!(
            "E1M1 AR-0025 Stage 3B per-column position trace: source_viewer=({},{}); offset=({},{}); selected_segs={}; outside={}; fully_covered={}; partial={}; fully_visible={}; covered_cells={}; depth_order_inversions={}; meaning=headless-source-pose-observation-not-dynamic-renderer-selection",
            viewer[0],
            viewer[1],
            offset_x,
            offset_y,
            observation.selected_seg_records.len(),
            observation.outside,
            observation.fully_covered,
            observation.partial,
            observation.fully_visible,
            observation.covered_cells,
            observation.depth_order_inversions,
        );
    }
    Ok(())
}

/// Replays the Slice 7 ordered-coverage reconstruction over the established
/// heading, declared-offset, and retained-failure poses. This remains a
/// headless source-input matrix: it proves that partial wall and plane
/// reconstruction remains bounded and attributable as the viewer changes,
/// not that the resulting frames are visually correct or collision-valid.
pub(crate) fn report_doom_seg_ordered_coverage_pose_matrix(
    scene: &SceneInput,
) -> PlatformResult<()> {
    let map = &scene.door_geometry_source.map;
    let source_triangles =
        lower_doom_seg_textured_wall_triangles(map, &scene.door_geometry_source.wall_extents)?;
    let eye_height = scene.spawn_observer.position.y as f64;
    let plane_marks = observe_doom_seg_plane_marks(map, eye_height as i16)?;
    let origin = scene.spawn_observer.source_position;
    let source_heading = f64::from(scene.spawn_observer.source_angle);
    let mut poses = Vec::new();
    for heading_offset in [0.0, 90.0, 180.0, 270.0] {
        poses.push((
            format!("turn-{heading_offset:.0}"),
            origin,
            (source_heading + heading_offset).rem_euclid(360.0),
        ));
    }
    for [offset_x, offset_y] in [[64, 0], [-64, 0], [0, 64], [0, -64], [128, 64]] {
        poses.push((
            format!("offset-{offset_x}-{offset_y}"),
            [origin[0] + offset_x, origin[1] + offset_y],
            source_heading,
        ));
    }
    poses.extend([
        (String::from("retained-near-wall-a"), [1202, -3502], -24.0),
        (String::from("retained-near-wall-b"), [1296, -3427], -0.4),
        (
            String::from("retained-courtyard-loss"),
            [1514, -2481],
            -29.2,
        ),
    ]);

    let mut total_wall_cells = 0usize;
    let mut total_wall_triangles = 0usize;
    let mut total_plane_cells = 0usize;
    let mut total_plane_triangles = 0usize;
    let mut total_degenerate_wall_cells = 0usize;
    let mut total_unresolved_wall_cells = 0usize;
    for (label, viewer, heading_degrees) in &poses {
        let heading = heading_degrees.to_radians();
        let traversal = observe_doom_seg_classic_bsp(map, *viewer, heading, &BTreeSet::new())?;
        let vertical = observe_shared_doom_classic_vertical_clip_state(
            map,
            &source_triangles,
            &plane_marks,
            &traversal,
            *viewer,
            heading,
            eye_height,
        );
        let walls = reconstruct_doom_ordered_wall_fragments(
            map,
            &source_triangles,
            &vertical,
            *viewer,
            heading,
            eye_height,
        );
        let planes = reconstruct_doom_seg_classic_plane_cells(
            &vertical.plane_spans,
            *viewer,
            heading,
            eye_height,
        );
        total_wall_cells += walls.retained_cells;
        total_wall_triangles += walls.reconstructed_triangles.len();
        total_plane_cells += planes.source_cells;
        total_plane_triangles += planes.reconstructed_triangles;
        total_degenerate_wall_cells += walls.degenerate_cells;
        total_unresolved_wall_cells += walls.unresolved_cells;
        println!(
            "E1M1 AR-0025 Slice 7 ordered-coverage pose: label={label}; source-viewer=({},{}); source-heading-degrees={heading_degrees:.1}; admitted-segs={}; wall-cells={}; wall-triangles={}; degenerate-wall-cells={}; unresolved-wall-cells={}; plane-source-cells={}; plane-triangles={}; plane-rejections=[horizon:{} behind:{} degenerate:{}]; wall-samples={:?}; meaning=headless-source-pose-reconstruction-not-visual-correctness-or-navigation",
            viewer[0],
            viewer[1],
            vertical.admitted_segs,
            walls.retained_cells,
            walls.reconstructed_triangles.len(),
            walls.degenerate_cells,
            walls.unresolved_cells,
            planes.source_cells,
            planes.reconstructed_triangles,
            planes.horizon_rejections,
            planes.behind_viewer_rejections,
            planes.degenerate_rejections,
            walls.samples,
        );
    }
    println!(
        "E1M1 AR-0025 Slice 7 ordered-coverage pose matrix: poses={}; total-wall-cells={total_wall_cells}; total-wall-triangles={total_wall_triangles}; total-degenerate-wall-cells={total_degenerate_wall_cells}; total-unresolved-wall-cells={total_unresolved_wall_cells}; total-plane-source-cells={total_plane_cells}; total-plane-triangles={total_plane_triangles}; matrix=spawn-four-headings-plus-five-declared-offsets-plus-three-retained-failures; movement-claim=none",
        poses.len(),
    );
    Ok(())
}

/// Replays the retained interactive false-negative poses from AR-0025 Cycle
/// 35. These are observation inputs, not navigation waypoints or a fix.
pub(crate) fn report_doom_seg_per_column_failure_trace(scene: &SceneInput) -> PlatformResult<()> {
    for (label, viewer, heading_degrees) in [
        ("near-wall-a", [1202, -3502], -24.0_f64),
        ("near-wall-b", [1296, -3427], -0.4_f64),
        ("courtyard-loss", [1514, -2481], -29.2_f64),
    ] {
        let observation = observe_doom_seg_screen_grid(
            &scene.door_geometry_source.map,
            scene.spawn_observer.position.y,
            true,
            viewer,
            heading_degrees.to_radians(),
        )?;
        println!(
            "E1M1 AR-0025 Stage 3B retained false-negative trace: pose={label}; source_viewer=({},{}); source_heading_degrees={heading_degrees:.1}; selected_segs={}; outside={}; fully_covered={}; partial={}; fully_visible={}; covered_cells={}; depth_order_inversions={}; depth_order_samples={}; meaning=known-visually-unsound-source-grid-pose-not-presentation-selection",
            viewer[0],
            viewer[1],
            observation.selected_seg_records.len(),
            observation.outside,
            observation.fully_covered,
            observation.partial,
            observation.fully_visible,
            observation.covered_cells,
            observation.depth_order_inversions,
            observation.depth_order_samples.join(" | "),
        );
    }
    Ok(())
}

/// Tests whether a global nearest-SEG approximation removes the local-depth
/// inversions exposed by the retained false-negative poses. This intentionally
/// does not upload or select a presentation: a single closest-point order may
/// still disagree with different rays across one long SEG.
pub(crate) fn report_doom_seg_per_column_order_trace(scene: &SceneInput) -> PlatformResult<()> {
    for (label, viewer, heading_degrees) in [
        ("near-wall-a", [1202, -3502], -24.0_f64),
        ("near-wall-b", [1296, -3427], -0.4_f64),
        ("courtyard-loss", [1514, -2481], -29.2_f64),
    ] {
        let leaf_order = observe_doom_seg_screen_grid_with_order(
            &scene.door_geometry_source.map,
            scene.spawn_observer.position.y,
            true,
            viewer,
            heading_degrees.to_radians(),
            DoomSegScreenGridOrder::BspLeafThenSource,
        )?;
        let nearest_order = observe_doom_seg_screen_grid_with_order(
            &scene.door_geometry_source.map,
            scene.spawn_observer.position.y,
            true,
            viewer,
            heading_degrees.to_radians(),
            DoomSegScreenGridOrder::NearestSegmentToViewer,
        )?;
        println!(
            "E1M1 AR-0025 Stage 3B ordering trace: pose={label}; source_viewer=({},{}); source_heading_degrees={heading_degrees:.1}; leaf-order=[selected:{} fully-covered:{} depth-inversions:{}]; nearest-segment-order=[selected:{} fully-covered:{} depth-inversions:{}]; meaning=diagnostic-order-comparison-not-presentation-selection-or-doom-parity",
            viewer[0],
            viewer[1],
            leaf_order.selected_seg_records.len(),
            leaf_order.fully_covered,
            leaf_order.depth_order_inversions,
            nearest_order.selected_seg_records.len(),
            nearest_order.fully_covered,
            nearest_order.depth_order_inversions,
        );
    }
    Ok(())
}

/// Tests the first separable portion of classic Doom's `R_AddLine` protocol:
/// source SEG facing and horizontal FOV admission, followed by Doom-owned
/// solid-versus-pass authority. It intentionally does not yet union solid
/// ranges, clip spans, prune BSP bboxes, or select presentation draws.
pub(crate) fn report_doom_seg_classic_admission_trace(scene: &SceneInput) -> PlatformResult<()> {
    for (label, viewer, heading_degrees) in [
        ("near-wall-a", [1202, -3502], -24.0_f64),
        ("near-wall-b", [1296, -3427], -0.4_f64),
        ("courtyard-loss", [1514, -2481], -29.2_f64),
    ] {
        let observation = observe_doom_seg_classic_admission(
            &scene.door_geometry_source.map,
            viewer,
            heading_degrees.to_radians(),
        )?;
        println!(
            "E1M1 AR-0025 Stage 3B classic-admission trace: pose={label}; source_viewer=({},{}); source_heading_degrees={heading_degrees:.1}; source_segs={}; backface_rejected={}; edge_on={}; outside_fov_rejected={}; near-plane-fail-open={}; solid_admitted={}; pass_admitted={}; solid-range-contributors={}; solid-range-fully-covered={}; solid-range-covered-columns={}; samples={}; meaning=doom-source-protocol-preflight-with-horizontal-range-union-not-bsp-bbox-pruning-or-presentation-selection",
            viewer[0],
            viewer[1],
            observation.source_segs,
            observation.backface_rejected,
            observation.edge_on,
            observation.outside_fov_rejected,
            observation.near_plane_fail_open,
            observation.solid_admitted,
            observation.pass_admitted,
            observation.solid_range_contributors,
            observation.solid_range_fully_covered,
            observation.solid_range_covered_columns,
            observation.samples.join(" | "),
        );
    }
    Ok(())
}

/// Replays the existing counterexample poses through a bounded Doom-style BSP
/// traversal. A far child is skipped only when its source bbox projects to an
/// interval wholly closed by earlier admitted solid ranges; all uncertain bbox
/// cases remain fail-open.
pub(crate) fn report_doom_seg_classic_bsp_trace(scene: &SceneInput) -> PlatformResult<()> {
    let lowerable_triangles = lower_doom_seg_textured_wall_triangles(
        &scene.door_geometry_source.map,
        &scene.door_geometry_source.wall_extents,
    )?;
    let hut_subsectors = scene
        .door_geometry_source
        .map
        .subsectors
        .iter()
        .enumerate()
        .filter_map(|(subsector_index, subsector)| {
            let first = usize::from(subsector.first_seg);
            let end = first + usize::from(subsector.seg_count);
            scene.door_geometry_source.map.segs[first..end]
                .iter()
                .any(|seg| seg.linedef == 247)
                .then_some(subsector_index as u16)
        })
        .collect::<BTreeSet<_>>();
    let plane_marks = observe_doom_seg_plane_marks(
        &scene.door_geometry_source.map,
        scene.spawn_observer.position.y as i16,
    )?;
    for (label, viewer, heading_degrees) in [
        ("near-wall-a", [1202, -3502], -24.0_f64),
        ("near-wall-b", [1296, -3427], -0.4_f64),
        ("courtyard-loss", [1514, -2481], -29.2_f64),
        // The retained source-derived hut direction is a control for the
        // exterior-wall suspect at linedef 247. It is not a navigation pose.
        (
            "hut-control",
            scene.spawn_observer.source_position,
            (-208.0_f64).atan2(1120.0).to_degrees(),
        ),
    ] {
        let observation = observe_doom_seg_classic_bsp(
            &scene.door_geometry_source.map,
            viewer,
            heading_degrees.to_radians(),
            &hut_subsectors,
        )?;
        let admitted_triangles = lowerable_triangles
            .iter()
            .filter(|triangle| {
                observation
                    .admitted_seg_records
                    .contains(&triangle.source_seg.record_index)
            })
            .count();
        let admitted_triangle_roles = summarize_classic_bsp_wall_triangle_roles(
            &lowerable_triangles,
            &observation.admitted_seg_records,
        );
        let admitted_plane_marks =
            summarize_classic_bsp_plane_marks(&plane_marks, &observation.admitted_seg_records);
        let admitted_hut_triangles = lowerable_triangles
            .iter()
            .filter(|triangle| {
                triangle.source_linedef.record_index == 247
                    && observation
                        .admitted_seg_records
                        .contains(&triangle.source_seg.record_index)
            })
            .count();
        let (retained_floor_draws, retained_ceiling_draws) =
            count_classic_bsp_static_flat_draws(scene, &observation);
        println!(
            "E1M1 AR-0025 Stage 3B classic-BSP trace: pose={label}; source_viewer=({},{}); source_heading_degrees={heading_degrees:.1}; leaves-visited={}; source-segs-visited={}; far-children-solid-pruned={}; far-children-outside-fov={}; far-children-fail-open={}; backface-rejected={}; edge-on={}; outside-fov-rejected={}; near-plane-fail-open={}; solid-admitted={}; pass-admitted={}; admitted-source-segs={}; lowerable-seg-wall-triangles={}; lowerable-wall-role-triangles=[upper:{} lower:{} middle:{}]; source-plane-marks=[floor:{} ceiling:{} paired-sky:{}]; retained-static-flats=[floor:{} ceiling:{}]; solid-range-contributors={}; solid-range-fully-covered={}; solid-range-covered-columns={}; hut-line-247=[subsectors:{:?} reached:{:?} visited:{} admitted:{} lowerable-seg-wall-triangles:{} elisions:{}]; samples={}; meaning=doom-source-protocol-comparison-not-historic-doom-parity-or-presentation-selection",
            viewer[0],
            viewer[1],
            observation.leaves_visited,
            observation.source_segs_visited,
            observation.far_children_pruned,
            observation.far_children_outside_fov,
            observation.far_children_fail_open,
            observation.backface_rejected,
            observation.edge_on,
            observation.outside_fov_rejected,
            observation.near_plane_fail_open,
            observation.solid_admitted,
            observation.pass_admitted,
            observation.admitted_seg_records.len(),
            admitted_triangles,
            admitted_triangle_roles.0,
            admitted_triangle_roles.1,
            admitted_triangle_roles.2,
            admitted_plane_marks.0,
            admitted_plane_marks.1,
            admitted_plane_marks.2,
            retained_floor_draws,
            retained_ceiling_draws,
            observation.solid_range_contributors,
            observation.solid_range_fully_covered,
            observation.solid_range_covered_columns,
            hut_subsectors.iter().collect::<Vec<_>>(),
            hut_subsectors
                .iter()
                .filter(|subsector| observation.visited_subsectors.contains(subsector))
                .collect::<Vec<_>>(),
            observation.hut_linedef_segs_visited,
            observation.hut_linedef_segs_admitted,
            admitted_hut_triangles,
            observation.watched_subsector_elisions.join(","),
            observation.samples.join(" | "),
        );
    }
    Ok(())
}

/// Retains the next separable source-protocol checkpoint after recursive BSP
/// admission: wall tiers update independent ceiling/floor clip boundaries,
/// while source plane marks remain separate facts. It deliberately stops
/// before visplane construction, flat selection, or any presentation mode.
pub(crate) fn report_doom_seg_classic_vertical_clip_trace(
    scene: &SceneInput,
) -> PlatformResult<()> {
    let lowerable_triangles = lower_doom_seg_textured_wall_triangles(
        &scene.door_geometry_source.map,
        &scene.door_geometry_source.wall_extents,
    )?;
    let plane_marks = observe_doom_seg_plane_marks(
        &scene.door_geometry_source.map,
        scene.spawn_observer.position.y as i16,
    )?;
    for (label, viewer, heading_degrees) in [
        (
            "source-spawn",
            scene.spawn_observer.source_position,
            90.0_f64,
        ),
        ("near-wall-a", [1202, -3502], -24.0_f64),
        ("near-wall-b", [1296, -3427], -0.4_f64),
        ("courtyard-loss", [1514, -2481], -29.2_f64),
        // Retained LOOK-ray exterior observations from the hut/sky fault
        // investigation. These are diagnostic source poses, not collision-
        // valid player locations. They test actual F_SKY1 plane authority,
        // rather than treating paired-sky boundary metadata as authority.
        ("exterior-hut-west", [2354, -3861], -87.7_f64),
        ("exterior-hut-east", [2076, -3560], -25.1_f64),
        ("exterior-far-east", [2801, -3450], -2.5_f64),
        (
            "hut-control",
            scene.spawn_observer.source_position,
            (-208.0_f64).atan2(1120.0).to_degrees(),
        ),
    ] {
        let traversal = observe_doom_seg_classic_bsp(
            &scene.door_geometry_source.map,
            viewer,
            heading_degrees.to_radians(),
            &BTreeSet::new(),
        )?;
        let vertical = observe_shared_doom_classic_vertical_clip_state(
            &scene.door_geometry_source.map,
            &lowerable_triangles,
            &plane_marks,
            &traversal,
            viewer,
            heading_degrees.to_radians(),
            scene.spawn_observer.position.y as f64,
        );
        let authoritative_sky = model_authoritative_sky_regions(
            &vertical,
            &traversal.admitted_seg_order,
            AuthoritativeSkyViewIdentity {
                fixture: format!("e1m1-{label}"),
                source_position: viewer,
                heading_radians: heading_degrees.to_radians(),
                source_eye_height: scene.spawn_observer.position.y as i16,
            },
            "decoded-source-heights",
        );
        println!(
            "E1M1 AR-0025 Stage 3B classic-vertical-clip trace: pose={label}; source_viewer=({},{}); source_heading_degrees={heading_degrees:.1}; admitted-segs={}; tier-spans=[upper:{} lower:{} middle:{}]; source-plane-marks=[floor:{} ceiling:{} paired-sky:{}]; authoritative-sky=[regions:{} input-intervals:{} modeled-intervals:{} omitted-intervals:{} cells:{}/{} fail-open:{} fingerprint:{}]; clip-updates=[ceiling:{} floor:{}]; center-column-trace={}; meaning=doom-owned-wall-tier-and-plane-clip-state-with-candidate1-authority-audit-not-presentation-visibility",
            viewer[0], viewer[1], vertical.admitted_segs, vertical.upper_tier_spans,
            vertical.lower_tier_spans, vertical.middle_tier_spans, vertical.floor_plane_marks,
            vertical.ceiling_plane_marks, vertical.paired_sky_adjustments,
            authoritative_sky.regions.len(),
            authoritative_sky.input_sky_intervals,
            authoritative_sky.modeled_sky_intervals,
            authoritative_sky.omitted_sky_intervals,
            authoritative_sky.modeled_sky_cells,
            authoritative_sky.input_sky_cells,
            authoritative_sky.fail_open,
            authoritative_sky.structural_fingerprint,
            vertical.ceiling_clip_updates, vertical.floor_clip_updates, vertical.samples.join(" | "),
        );
    }
    Ok(())
}

/// Establishes the decoded source grouping facts that must exist before a
/// later provider-local visplane/span experiment can be meaningful. Classic
/// Doom groups planes by height, flat identity, and light; sky ceilings use a
/// common height/light identity. This records no allocated plane or draw.
pub(crate) fn report_doom_seg_classic_plane_identity_trace(
    scene: &SceneInput,
) -> PlatformResult<()> {
    let plane_marks = observe_doom_seg_plane_marks(
        &scene.door_geometry_source.map,
        scene.spawn_observer.position.y as i16,
    )?;
    for (label, viewer, heading_degrees) in [
        ("source-spawn", [1056, -3616], 90.0_f64),
        ("near-wall-b", [1296, -3427], -0.4_f64),
        ("courtyard-loss", [1514, -2481], -29.2_f64),
    ] {
        let traversal = observe_doom_seg_classic_bsp(
            &scene.door_geometry_source.map,
            viewer,
            heading_degrees.to_radians(),
            &BTreeSet::new(),
        )?;
        let identities = observe_doom_seg_classic_plane_identities(
            &scene.door_geometry_source.map,
            &plane_marks,
            &traversal,
        );
        println!(
            "E1M1 AR-0025 Stage 3B classic-plane-identity trace: pose={label}; admitted-segs={}; contributors=[floor:{} ceiling:{} sky-ceiling:{}]; unique-source-keys=[floor:{} ceiling:{}]; samples={}; meaning=source-plane-grouping-prerequisite-not-visplanes-spans-flat-selection-or-presentation-visibility",
            traversal.admitted_seg_order.len(),
            identities.floor_mark_contributors,
            identities.ceiling_mark_contributors,
            identities.sky_ceiling_contributors,
            identities.unique_floor_keys,
            identities.unique_ceiling_keys,
            identities.samples.join(" | "),
        );
    }
    Ok(())
}

/// Reconstructs bounded, source-keyed plane cells from the clip state observed
/// immediately before each admitted wall range mutates it. This is the first
/// span-shaped Stage 3B evidence, but it remains headless and deliberately
/// stops before flat lookup, triangulation, upload, or presentation selection.
pub(crate) fn report_doom_seg_classic_plane_span_trace(scene: &SceneInput) -> PlatformResult<()> {
    let lowerable_triangles = lower_doom_seg_textured_wall_triangles(
        &scene.door_geometry_source.map,
        &scene.door_geometry_source.wall_extents,
    )?;
    let plane_marks = observe_doom_seg_plane_marks(
        &scene.door_geometry_source.map,
        scene.spawn_observer.position.y as i16,
    )?;
    for (label, viewer, heading_degrees) in [
        ("source-spawn", [1056, -3616], 90.0_f64),
        ("near-wall-a", [1202, -3502], -24.0_f64),
        ("near-wall-b", [1296, -3427], -0.4_f64),
        ("courtyard-loss", [1514, -2481], -29.2_f64),
        (
            "hut-control",
            scene.spawn_observer.source_position,
            (-208.0_f64).atan2(1120.0).to_degrees(),
        ),
    ] {
        let traversal = observe_doom_seg_classic_bsp(
            &scene.door_geometry_source.map,
            viewer,
            heading_degrees.to_radians(),
            &BTreeSet::new(),
        )?;
        let vertical = observe_shared_doom_classic_vertical_clip_state(
            &scene.door_geometry_source.map,
            &lowerable_triangles,
            &plane_marks,
            &traversal,
            viewer,
            heading_degrees.to_radians(),
            scene.spawn_observer.position.y as f64,
        );
        let spans = vertical.plane_spans;
        let flat_resolution = resolve_doom_seg_classic_plane_flats(scene, &spans);
        let cell_reconstruction = reconstruct_doom_seg_classic_plane_cells(
            &spans,
            viewer,
            heading_degrees.to_radians(),
            scene.spawn_observer.position.y as f64,
        );
        let floor_keys = spans
            .keys
            .keys()
            .filter(|key| key.kind == DoomSegClassicPlaneKind::Floor)
            .count();
        let ceiling_keys = spans.keys.len() - floor_keys;
        println!(
            "E1M1 AR-0025 Stage 3B classic-plane-span trace: pose={label}; source_viewer=({},{}); source_heading_degrees={heading_degrees:.1}; admitted-segs={}; source-plane-keys=[floor:{} ceiling:{}]; plane-instances={}; collision-splits={}; horizontal-spans={}; populated-columns={}; populated-cells={}; overlapping-writes={}; empty-after-clip={}; flat-resolution=[resolved-instances:{} unresolved-instances:{} sky-instances:{} candidate-draws:{} candidate-triangles:{}]; cell-reconstruction=[source-cells:{} quads:{} triangles:{} horizon-rejected:{} behind-rejected:{} degenerate-rejected:{} maximum-source-distance:{:.3}]; samples={}; flat-samples={}; cell-samples={}; meaning=bounded-doom-source-plane-instances-with-headless-viewer-relative-cell-reconstruction-not-visplane-parity-upload-or-presentation-visibility",
            viewer[0],
            viewer[1],
            vertical.admitted_segs,
            floor_keys,
            ceiling_keys,
            spans.plane_instances,
            spans.collision_splits,
            spans.horizontal_spans,
            spans.populated_columns,
            spans.populated_cells,
            spans.overlapping_writes,
            spans.empty_after_clip,
            flat_resolution.resolved_instances,
            flat_resolution.unresolved_instances,
            flat_resolution.sky_instances,
            flat_resolution.candidate_draws,
            flat_resolution.candidate_triangles,
            cell_reconstruction.source_cells,
            cell_reconstruction.reconstructed_quads,
            cell_reconstruction.reconstructed_triangles,
            cell_reconstruction.horizon_rejections,
            cell_reconstruction.behind_viewer_rejections,
            cell_reconstruction.degenerate_rejections,
            cell_reconstruction.maximum_source_distance,
            spans.samples.join(" | "),
            flat_resolution.samples.join(" | "),
            cell_reconstruction.samples.join(" | "),
        );
    }
    Ok(())
}
