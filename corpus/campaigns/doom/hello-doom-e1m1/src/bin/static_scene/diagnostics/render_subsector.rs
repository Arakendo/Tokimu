//! Headless inventory for the AR-0030 render-subsector experiment.

use std::{collections::BTreeSet, io};

use tokimu::PlatformResult;

use crate::{
    build_render_subsector_connectivity_graph, build_render_subsector_inventory,
    doom_view_transfer_chain, locate_doom_point_subsector, observe_doom_view_transfer,
    observe_render_subsector_actual_camera, observe_render_subsector_connectivity,
    prepare_render_subsector_view, render_subsector_connectivity_path,
    resolve_doom_subsector_bsp_paths, DoomComparativeEmbedding, DoomSurfacePlane,
    DoomViewTransferObservation, RenderSubsectorConnectivityEdge, RenderSubsectorConnectivityGraph,
    RenderSubsectorInventory, RenderSubsectorSurfaceShadowDisposition, RenderSubsectorViewPose,
    SceneInput,
};

use super::tokimu_spatial_bake::SpatialRayShadow;

const BASELINE_VIEWPORT: [u32; 2] = [1280, 800];
const BASELINE_VERTICAL_FOV_DEGREES: f32 = 60.0;
const BASELINE_PITCH_DEGREES: f32 = 0.0;
const PARKED_CYCLE_31_VISUAL_DRAWS: usize = 365;

pub(crate) fn report_render_subsector_inventory(scene: &SceneInput) -> PlatformResult<()> {
    let inventory = build_render_subsector_inventory(
        &scene.door_geometry_source.map,
        &scene.door_geometry_source.wall_extents,
        scene.spawn_observer.source_position,
        scene.spawn_observer.source_angle,
        scene.spawn_observer.position.y as i16,
        BASELINE_VIEWPORT,
        BASELINE_VERTICAL_FOV_DEGREES,
        BASELINE_PITCH_DEGREES,
    )
    .map_err(io::Error::other)?;
    let global_opaque_triangles = triangle_count(&scene.opaque_draws);
    let global_cutout_triangles = triangle_count(&scene.cutout_draws);

    println!(
        "E1M1 render-subsector baseline: strategy={}; map={}; map-fingerprint={:016x}; camera-fingerprint={:016x}; runtime-height-fingerprint={:016x}; prepared-view-fingerprint={:016x}; source-position=({},{},{}); source-angle={}; viewport={}x{}; vertical-fov-degrees={:.3}; pitch-degrees={:.3}; global-full=[opaque-draws:{},cutout-draws:{},opaque-triangles:{},cutout-triangles:{},triangles:{}]; parked-cycle31-visual-draws={}; retained-controls=[six-rays,source-spawn,marked-holes,scan-look,door-snapshot,platform-snapshot]; meaning=immutable-headless-baseline-not-presentation-authority",
        inventory.strategy,
        scene.door_geometry_source.map.map_name,
        inventory.identity.map_fingerprint,
        inventory.identity.camera_fingerprint,
        inventory.identity.runtime_height_fingerprint,
        inventory.identity.prepared_view_fingerprint,
        scene.spawn_observer.source_position[0],
        scene.spawn_observer.source_position[1],
        scene.spawn_observer.position.y as i16,
        scene.spawn_observer.source_angle,
        inventory.identity.viewport[0],
        inventory.identity.viewport[1],
        f32::from_bits(inventory.identity.vertical_fov_degrees_bits),
        f32::from_bits(inventory.identity.pitch_degrees_bits),
        scene.opaque_draws.len(),
        scene.cutout_draws.len(),
        global_opaque_triangles,
        global_cutout_triangles,
        global_opaque_triangles + global_cutout_triangles,
        PARKED_CYCLE_31_VISUAL_DRAWS,
    );
    println!(
        "E1M1 render-subsector conservation: source-subsectors={}; render-subsectors={}; render-sector-associations=[source-identity:{},unresolved:0]; boundaries=[ordered-seg-loop-exact:{},ordered-seg-loop-refines-bsp-path:{},bsp-path-implicit:{},unresolved:{}]; planes=[source:{},represented:{},missing:{},ordinary:{},sky:{}]; triangles=[represented:{},degenerate:{},containment-failures:{},winding-failures:{}]; walls=[source-segs:{},represented-segs:{},missing:{},source-tier-triangles:{},represented-tier-triangles:{},missing-tier-triangles:{}]; zero-clearance-subsectors={}; declarations=[opaque:0,cutout:0,mode:shadow]; conserved={}; meaning=persistent-world-geometry-before-view-participation",
        scene.door_geometry_source.map.subsectors.len(),
        inventory.subsectors.len(),
        inventory.subsectors.len(),
        inventory.ordered_seg_loops,
        inventory.ordered_seg_refinements,
        inventory.bsp_path_boundaries,
        inventory.unresolved_boundaries,
        inventory.source_plane_units,
        inventory.represented_plane_units,
        inventory
            .source_plane_units
            .saturating_sub(inventory.represented_plane_units),
        inventory.ordinary_plane_units,
        inventory.sky_plane_units,
        inventory.triangles,
        inventory.degenerate_triangles,
        inventory.containment_failures,
        inventory.winding_failures,
        inventory.source_wall_segs,
        inventory.represented_wall_segs,
        inventory
            .source_wall_segs
            .saturating_sub(inventory.represented_wall_segs),
        inventory.source_wall_tier_triangles,
        inventory.represented_wall_tier_triangles,
        inventory
            .source_wall_tier_triangles
            .saturating_sub(inventory.represented_wall_tier_triangles),
        inventory.zero_clearance_subsectors,
        inventory.conserved(),
    );
    for subsector in &inventory.subsectors {
        let floor_triangles = subsector
            .triangles
            .iter()
            .filter(|triangle| triangle.plane == doom_geometry_provider::DoomSurfacePlane::Floor)
            .count();
        let ceiling_triangles = subsector.triangles.len() - floor_triangles;
        println!(
            "render-subsector: source={}; source-sector={}; render-sector={}; association=source-owning-sector; boundary=[authority:{},vertices:{},ordered-seg-gaps:{},fingerprint:{:016x}]; walls=[segs:{},seg-fingerprint:{:016x},tier-triangles:{},tier-fingerprint:{:016x}]; heights=[floor:{},ceiling:{},revision:{:016x}]; planes=[floor-role:{},ceiling-role:{},floor-triangles:{},ceiling-triangles:{},fingerprint:{:016x}]; provenance={}",
            subsector.source_subsector.record_index,
            subsector.source_sector.record_index,
            subsector.render_sector.record_index,
            subsector.boundary_authority.label(),
            subsector.boundary.len(),
            subsector.ordered_seg_gaps,
            subsector.boundary_fingerprint,
            subsector.wall_sources.len(),
            subsector.wall_fingerprint,
            subsector.wall_tier_triangles.len(),
            subsector.wall_tier_fingerprint,
            subsector.floor_height,
            subsector.ceiling_height,
            subsector.runtime_height_revision,
            subsector.floor_role.label(),
            subsector.ceiling_role.label(),
            floor_triangles,
            ceiling_triangles,
            subsector.triangle_fingerprint,
            subsector.unresolved_reason.unwrap_or("supported"),
        );
    }

    if !inventory.conserved() {
        return Err(io::Error::other(
            "render-subsector inventory failed construction conservation",
        )
        .into());
    }
    Ok(())
}

pub(crate) fn report_render_subsector_actual_camera_shadow(
    scene: &SceneInput,
) -> PlatformResult<()> {
    let inventory = build_render_subsector_inventory(
        &scene.door_geometry_source.map,
        &scene.door_geometry_source.wall_extents,
        scene.spawn_observer.source_position,
        scene.spawn_observer.source_angle,
        scene.spawn_observer.position.y as i16,
        BASELINE_VIEWPORT,
        BASELINE_VERTICAL_FOV_DEGREES,
        BASELINE_PITCH_DEGREES,
    )
    .map_err(io::Error::other)?;
    if !inventory.conserved() {
        return Err(io::Error::other(
            "actual-camera shadow requires a conserved render-subsector inventory",
        )
        .into());
    }
    let spawn = scene.spawn_observer.source_position.map(f64::from);
    let spawn_heading = f32::from(scene.spawn_observer.source_angle);
    let eye_height = f64::from(scene.spawn_observer.position.y);
    let heading_radians = spawn_heading.to_radians();
    let forward = [
        spawn[0] + f64::from(heading_radians.cos()) * 96.0,
        spawn[1] + f64::from(heading_radians.sin()) * 96.0,
    ];
    let pose = |label, source_position, eye_height, heading_degrees, pitch_degrees| {
        RenderSubsectorViewPose {
            label,
            source_position,
            eye_height,
            heading_degrees,
            pitch_degrees,
            viewport: BASELINE_VIEWPORT,
            vertical_fov_degrees: BASELINE_VERTICAL_FOV_DEGREES,
        }
    };
    let poses = [
        pose("spawn-neutral", spawn, eye_height, spawn_heading, 0.0),
        pose("spawn-pitch-up-20", spawn, eye_height, spawn_heading, 20.0),
        pose(
            "spawn-pitch-down-20",
            spawn,
            eye_height,
            spawn_heading,
            -20.0,
        ),
        pose(
            "spawn-yaw-plus-30",
            spawn,
            eye_height,
            spawn_heading + 30.0,
            0.0,
        ),
        pose(
            "spawn-yaw-minus-30",
            spawn,
            eye_height,
            spawn_heading - 30.0,
            0.0,
        ),
        pose("spawn-forward-96", forward, eye_height, spawn_heading, 0.0),
        pose("spawn-return", spawn, eye_height, spawn_heading, 0.0),
        pose(
            "near-wall-subsector-64",
            [-80.153_221_13, -3_260.071_777_344],
            140.0,
            -168.884,
            -12.389,
        ),
        pose(
            "off-axis-wall-edge",
            [-97.824_401_855, -3_256.003_417_969],
            140.0,
            -154.944,
            0.0,
        ),
    ];
    let mut matrix_fingerprint = 0xcbf2_9ce4_8422_2325_u64;
    let mut false_negatives = 0;
    println!(
        "E1M1 render-subsector actual-camera shadow matrix: strategy={}; poses={}; viewport={}x{}; vertical-fov-degrees={:.3}; inventory-view={:016x}; authority=actual-geometry-plus-doom-private-horizontal-coverage; presentation-authority=disabled; meaning=actual-camera-shadow-not-presentation-selection",
        inventory.strategy,
        poses.len(),
        BASELINE_VIEWPORT[0],
        BASELINE_VIEWPORT[1],
        BASELINE_VERTICAL_FOV_DEGREES,
        inventory.identity.prepared_view_fingerprint,
    );
    for pose in poses {
        let observation = observe_render_subsector_actual_camera(
            &scene.door_geometry_source.map,
            &inventory,
            pose,
        )
        .map_err(io::Error::other)?;
        false_negatives += observation.false_negatives;
        matrix_fingerprint ^= observation.view_fingerprint;
        matrix_fingerprint = matrix_fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
        matrix_fingerprint ^= observation.result_fingerprint;
        matrix_fingerprint = matrix_fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
        matrix_fingerprint ^= observation.source_coverage_fingerprint;
        matrix_fingerprint = matrix_fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
        let first = observation
            .entries
            .first()
            .map(|entry| entry.source_subsector.record_index);
        let last = observation
            .entries
            .last()
            .map(|entry| entry.source_subsector.record_index);
        println!(
            "render-subsector shadow: pose={}; view={:016x}; near-first={:016x}; result={:016x}; source-coverage={:016x}; order=[first:{first:?},last:{last:?},leaves:{}]; subsectors=[retained:{},outside-frustum:{},unresolved:{},conservation:{}]; planes=[retained:{},outside-frustum:{},source-covered:{},unresolved:{},horizontal-aabb-false-positives:{},conservation:{}]; wall-tiers=[retained:{},outside-frustum:{},source-covered:{},unresolved:{},horizontal-aabb-false-positives:{},conservation:{}]; brute-geometry=[retained:{},false-negatives:{},aabb-false-positives:{}]",
            observation.label,
            observation.view_fingerprint,
            observation.near_first_fingerprint,
            observation.result_fingerprint,
            observation.source_coverage_fingerprint,
            observation.entries.len(),
            observation.retained,
            observation.outside_frustum,
            observation.unresolved,
            observation.retained + observation.outside_frustum + observation.unresolved,
            observation.plane_retained,
            observation.plane_outside_frustum,
            observation.plane_source_covered,
            observation.plane_unresolved,
            observation.plane_horizontal_aabb_false_positives,
            observation.plane_retained
                + observation.plane_outside_frustum
                + observation.plane_source_covered
                + observation.plane_unresolved,
            observation.wall_tiers_retained,
            observation.wall_tiers_outside_frustum,
            observation.wall_tiers_source_covered,
            observation.wall_tiers_unresolved,
            observation.wall_horizontal_aabb_false_positives,
            observation.wall_tiers_retained
                + observation.wall_tiers_outside_frustum
                + observation.wall_tiers_source_covered
                + observation.wall_tiers_unresolved,
            observation.brute_retained,
            observation.false_negatives,
            observation.false_positives,
        );
        if !observation.unresolved_surface_samples.is_empty() {
            println!(
                "render-subsector shadow unresolved: pose={}; samples=[{}]",
                observation.label,
                observation.unresolved_surface_samples.join(" | "),
            );
        }
    }
    let six_ray_fingerprint = report_render_subsector_six_ray_falsifiers(scene, &inventory)?;
    println!(
        "E1M1 render-subsector actual-camera shadow conservation: poses={}; false-negatives={}; matrix-fingerprint={matrix_fingerprint:016x}; six-ray-fingerprint={six_ray_fingerprint:016x}; source-coverage=surface-level-shadow; meaning=actual-finite-geometry-veto-plus-doom-private-horizontal-coverage-not-presentation-authority",
        poses.len(),
        false_negatives,
    );
    if false_negatives != 0 {
        return Err(io::Error::other(
            "render-subsector AABB shadow lost brute-force finite geometry",
        )
        .into());
    }
    Ok(())
}

pub(crate) fn report_render_subsector_prepared_view(scene: &SceneInput) -> PlatformResult<()> {
    let inventory = build_render_subsector_inventory(
        &scene.door_geometry_source.map,
        &scene.door_geometry_source.wall_extents,
        scene.spawn_observer.source_position,
        scene.spawn_observer.source_angle,
        scene.spawn_observer.position.y as i16,
        BASELINE_VIEWPORT,
        BASELINE_VERTICAL_FOV_DEGREES,
        BASELINE_PITCH_DEGREES,
    )
    .map_err(io::Error::other)?;
    let spawn = scene.spawn_observer.source_position.map(f64::from);
    let eye = f64::from(scene.spawn_observer.position.y);
    let heading = f32::from(scene.spawn_observer.source_angle);
    let forward_radians = heading.to_radians();
    let forward = [
        spawn[0] + f64::from(forward_radians.cos()) * 96.0,
        spawn[1] + f64::from(forward_radians.sin()) * 96.0,
    ];
    let green_linedef = &scene.door_geometry_source.map.linedefs[464];
    let green_start =
        &scene.door_geometry_source.map.vertices[usize::from(green_linedef.start_vertex)];
    let green_end = &scene.door_geometry_source.map.vertices[usize::from(green_linedef.end_vertex)];
    let green_midpoint = [
        f64::from(green_start.x + green_end.x) * 0.5,
        f64::from(green_start.y + green_end.y) * 0.5,
    ];
    let green_direction = [
        f64::from(green_end.x - green_start.x),
        f64::from(green_end.y - green_start.y),
    ];
    let green_length = green_direction[0].hypot(green_direction[1]);
    let green_front_viewer = [
        green_midpoint[0] + green_direction[1] / green_length * 64.0,
        green_midpoint[1] - green_direction[0] / green_length * 64.0,
    ];
    let green_heading = (green_midpoint[1] - green_front_viewer[1])
        .atan2(green_midpoint[0] - green_front_viewer[0])
        .to_degrees() as f32;
    let pose = |label, source_position, pitch_degrees| RenderSubsectorViewPose {
        label,
        source_position,
        eye_height: eye,
        heading_degrees: heading,
        pitch_degrees,
        viewport: BASELINE_VIEWPORT,
        vertical_fov_degrees: BASELINE_VERTICAL_FOV_DEGREES,
    };
    let poses = [
        pose("spawn-neutral", spawn, 0.0),
        RenderSubsectorViewPose {
            label: "spawn-yaw-plus-90-cutout-control",
            source_position: spawn,
            eye_height: eye,
            heading_degrees: heading + 90.0,
            pitch_degrees: 0.0,
            viewport: BASELINE_VIEWPORT,
            vertical_fov_degrees: BASELINE_VERTICAL_FOV_DEGREES,
        },
        RenderSubsectorViewPose {
            label: "green-room-cutout-owning-side",
            source_position: green_front_viewer,
            eye_height: eye,
            heading_degrees: green_heading,
            pitch_degrees: 0.0,
            viewport: BASELINE_VIEWPORT,
            vertical_fov_degrees: BASELINE_VERTICAL_FOV_DEGREES,
        },
        pose("spawn-pitch-up-20", spawn, 20.0),
        pose("spawn-pitch-down-20", spawn, -20.0),
        pose("spawn-forward-96", forward, 0.0),
        pose("spawn-return", spawn, 0.0),
    ];
    let cutout_names = scene
        .cutout_uploads
        .iter()
        .map(|upload| upload.source_name.as_str())
        .collect::<Vec<_>>();
    let cutout_sources = scene
        .cutout_draws
        .iter()
        .filter_map(|draw| match draw.source {
            crate::StaticDrawSource::Wall {
                source_linedef,
                source_sidedef,
                ..
            } => Some((source_linedef.record_index, source_sidedef.record_index)),
            crate::StaticDrawSource::Flat { .. } => None,
        })
        .collect::<BTreeSet<_>>();
    let cutout_source_triangles = inventory
        .subsectors
        .iter()
        .flat_map(|subsector| &subsector.wall_tier_triangles)
        .filter(|triangle| {
            triangle.role == crate::DoomWallTextureRole::Middle
                && cutout_sources.contains(&(
                    triangle.source_linedef.record_index,
                    triangle.source_sidedef.record_index,
                ))
        })
        .count();
    println!(
        "E1M1 render-subsector prepared-view cutout correlation: uploads={cutout_names:?}; source-triangles={cutout_source_triangles}"
    );
    let mut matrix_fingerprint = 0xcbf2_9ce4_8422_2325_u64;
    let mut spawn_fingerprint = None;
    for pose in poses {
        let prepared = prepare_render_subsector_view(
            &scene.door_geometry_source.map,
            &inventory,
            pose,
            &scene.door_geometry_source.wall_extents,
            &scene.opaque_uploads,
            &scene.cutout_uploads,
            &scene.cutout_draws,
        )
        .map_err(io::Error::other)?;
        prepared.verify_conservation().map_err(io::Error::other)?;
        if pose.label == "green-room-cutout-owning-side" && prepared.cutout_wall_declarations == 0 {
            return Err(io::Error::other(
                "render-subsector prepared view omitted the green-room owning-side cutout control",
            )
            .into());
        }
        if pose.label == "spawn-neutral" {
            spawn_fingerprint = Some(prepared.declaration_fingerprint);
        }
        if pose.label == "spawn-return"
            && spawn_fingerprint != Some(prepared.declaration_fingerprint)
        {
            return Err(io::Error::other(
                "render-subsector prepared view changed after deterministic camera return",
            )
            .into());
        }
        matrix_fingerprint ^= prepared.shadow.view_fingerprint;
        matrix_fingerprint = matrix_fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
        matrix_fingerprint ^= prepared.declaration_fingerprint;
        matrix_fingerprint = matrix_fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
        println!(
            "render-subsector prepared view: pose={}; declarations=[total:{},ordinary-planes:{},opaque-walls:{},cutout-walls:{}]; terminal=[sky-background-triangles:{},outside-frustum-triangles:{},source-covered-triangles:{},unresolved-fail-open-triangles:{}]; source-triangles=[planes:{},wall-tiers:{},total:{}]; conservation=balanced; declaration-fingerprint={:016x}; renderer-submission=unchanged",
            pose.label,
            prepared.declarations.len(),
            prepared.ordinary_plane_declarations,
            prepared.opaque_wall_declarations,
            prepared.cutout_wall_declarations,
            prepared.sky_background_triangles,
            prepared.outside_frustum_triangles,
            prepared.source_covered_triangles,
            prepared.unresolved_fail_open_triangles,
            prepared.source_plane_triangles,
            prepared.source_wall_tier_triangles,
            prepared.source_plane_triangles + prepared.source_wall_tier_triangles,
            prepared.declaration_fingerprint,
        );
    }
    println!(
        "E1M1 render-subsector prepared-view conservation: poses=7; matrix-fingerprint={matrix_fingerprint:016x}; prepare-then-replace=not-installed-shadow; renderer-vocabulary=ordinary-draws-only; conservation=balanced"
    );
    Ok(())
}

pub(crate) fn report_render_subsector_connectivity_shadow(
    scene: &SceneInput,
) -> PlatformResult<()> {
    #[derive(Clone, Copy)]
    struct Case {
        label: &'static str,
        origin: [f64; 3],
        direction: [f64; 3],
        expected_global_label: &'static str,
        expected_ordered: &'static str,
        target: ConnectivityTarget,
    }
    const WALL_230: &[u32] = &[415, 423];
    const WALL_247: &[u32] = &[559, 567];
    let cases = [
        Case {
            label: "hut-east-wall-230",
            origin: [2076.0, -3560.0, 36.0],
            direction: [0.905568898, -0.424199343, 0.0],
            expected_global_label: "wall:230:BROWN1",
            expected_ordered: "rejected",
            target: ConnectivityTarget::Wall(WALL_230),
        },
        Case {
            label: "wall-247-east",
            origin: [1306.508666992, -3272.168457031, 21.432840347],
            direction: [0.939651787, -0.338751376, 0.047981590],
            expected_global_label: "wall:247:BROWN96",
            expected_ordered: "rejected",
            target: ConnectivityTarget::Wall(WALL_247),
        },
        Case {
            label: "ceiling-104-reached",
            origin: [1477.330444336, -3594.213134766, 8.994521141],
            direction: [-0.792175531, -0.565008104, 0.230702817],
            expected_global_label: "flat:40:CEIL3_5",
            expected_ordered: "retained",
            target: ConnectivityTarget::Plane(104),
        },
        Case {
            label: "wall-247-west",
            origin: [2115.047851562, -3569.925048828, 8.994521141],
            direction: [0.928815067, -0.358562857, 0.093463443],
            expected_global_label: "wall:247:BROWN96",
            expected_ordered: "rejected",
            target: ConnectivityTarget::Wall(WALL_247),
        },
        Case {
            label: "ceiling-149-rejected",
            origin: [2139.683349609, -3196.036376953, 8.994521141],
            direction: [0.180356100, 0.780082107, 0.599119186],
            expected_global_label: "flat:7:CEIL3_5",
            expected_ordered: "rejected",
            target: ConnectivityTarget::Plane(149),
        },
        Case {
            label: "ceiling-104-rejected",
            origin: [2902.150878906, -3206.857421875, 8.994521141],
            direction: [-0.952072978, -0.304107845, 0.032795019],
            expected_global_label: "flat:40:CEIL3_5",
            expected_ordered: "rejected",
            target: ConnectivityTarget::Plane(104),
        },
    ];
    let map = &scene.door_geometry_source.map;
    let inventory = build_render_subsector_inventory(
        map,
        &scene.door_geometry_source.wall_extents,
        scene.spawn_observer.source_position,
        scene.spawn_observer.source_angle,
        scene.spawn_observer.position.y as i16,
        BASELINE_VIEWPORT,
        BASELINE_VERTICAL_FOV_DEGREES,
        BASELINE_PITCH_DEGREES,
    )
    .map_err(io::Error::other)?;
    if !inventory.conserved() {
        return Err(io::Error::other(
            "connectivity shadow requires a conserved render-subsector inventory",
        )
        .into());
    }
    let graph =
        build_render_subsector_connectivity_graph(map, &inventory).map_err(io::Error::other)?;
    let paths = resolve_doom_subsector_bsp_paths(map)?;
    let spatial = SpatialRayShadow::build(scene)?;
    println!(
        "E1M1 render-subsector connectivity graph: cells={}; relationships={}; directed-edges={}; isolated={}; roles={:?}; source-correlated={}; traversable-apertures={}; non-traversable-boundaries={}; zero-clearance={}; aperture-containment-failures={}; fingerprint={:016x}; aperture-fingerprint={:016x}; authority=shadow-only; meaning=finite-boundary-connectivity-not-visibility",
        inventory.subsectors.len(),
        graph.edges.len() / 2,
        graph.edges.len(),
        graph.isolated_subsectors,
        graph.role_counts,
        graph.source_correlated_relationships,
        graph.traversable_relationships,
        graph.closed_relationships,
        graph.zero_clearance_relationships,
        graph.aperture_containment_failures,
        graph.fingerprint,
        graph.aperture_fingerprint,
    );
    if graph.aperture_containment_failures != 0 {
        return Err(io::Error::other(format!(
            "directed aperture inventory failed: containment={}",
            graph.aperture_containment_failures,
        ))
        .into());
    }

    let mut matrix_fingerprint = 0xcbf2_9ce4_8422_2325_u64;
    let mut conservative_disagreements = 0;
    let mut sky_terminal_disagreements = 0;
    let mut bounded_transfer_disagreements = 0;
    let mut bounded_sky_disagreements = 0;
    let mut bounded_closure_disagreements = 0;
    let mut total_bounded_states = 0;
    let mut peak_bounded_states = 0;
    let mut total_relevant_surface_observations = 0;
    let mut total_reached_relevant_surfaces = 0;
    let mut total_rescued_retained_surfaces = 0;
    let mut total_reached_source_covered_surfaces = 0;
    for case in cases {
        let point = [case.origin[0].round() as i16, case.origin[1].round() as i16];
        let start = locate_doom_point_subsector(point, &paths)?
            .source_subsector
            .record_index;
        let horizontal = case.direction[0].hypot(case.direction[1]);
        let pose = RenderSubsectorViewPose {
            label: case.label,
            source_position: [case.origin[0], case.origin[1]],
            eye_height: case.origin[2],
            heading_degrees: case.direction[1].atan2(case.direction[0]).to_degrees() as f32,
            pitch_degrees: case.direction[2].atan2(horizontal).to_degrees() as f32,
            viewport: BASELINE_VIEWPORT,
            vertical_fov_degrees: BASELINE_VERTICAL_FOV_DEGREES,
        };
        let source_observation = observe_render_subsector_actual_camera(map, &inventory, pose)
            .map_err(io::Error::other)?;
        let ordered_outcome = match case.target {
            ConnectivityTarget::Wall(segs) => {
                let entries = source_observation
                    .wall_tier_entries
                    .iter()
                    .filter(|entry| segs.contains(&entry.source_seg.record_index))
                    .collect::<Vec<_>>();
                if entries.is_empty() {
                    "missing"
                } else if entries.iter().any(|entry| {
                    entry.disposition == RenderSubsectorSurfaceShadowDisposition::RetainedGeometry
                }) {
                    "retained"
                } else if entries.iter().any(|entry| {
                    entry.disposition == RenderSubsectorSurfaceShadowDisposition::Unresolved
                }) {
                    "unresolved"
                } else {
                    "rejected"
                }
            }
            ConnectivityTarget::Plane(subsector) => source_observation
                .plane_entries
                .iter()
                .find(|entry| {
                    entry.source_subsector.record_index == subsector
                        && entry.plane == DoomSurfacePlane::Ceiling
                })
                .map(|entry| match entry.disposition {
                    RenderSubsectorSurfaceShadowDisposition::RetainedGeometry => "retained",
                    RenderSubsectorSurfaceShadowDisposition::SourceCovered => "rejected",
                    RenderSubsectorSurfaceShadowDisposition::OutsideFrustum => "outside-frustum",
                    RenderSubsectorSurfaceShadowDisposition::Unresolved => "unresolved",
                })
                .unwrap_or("missing"),
        };
        if ordered_outcome != case.expected_ordered {
            return Err(io::Error::other(format!(
                "{} ordered oracle expected {}, observed {}",
                case.label, case.expected_ordered, ordered_outcome
            ))
            .into());
        }
        let conservative = observe_render_subsector_connectivity(&graph, start, false)
            .map_err(io::Error::other)?;
        let sky_terminal =
            observe_render_subsector_connectivity(&graph, start, true).map_err(io::Error::other)?;
        let targets = connectivity_target_subsectors(&inventory, case.target);
        if targets.is_empty() {
            return Err(io::Error::other(format!(
                "connectivity specimen {} has no target render subsector",
                case.label
            ))
            .into());
        }
        let conservative_target = shortest_reachable_target(&graph, &conservative, &targets);
        let sky_target = shortest_reachable_target(&graph, &sky_terminal, &targets);
        let conservative_reachable = conservative_target.is_some();
        let sky_reachable = sky_target.is_some();
        let expected_reachable = case.expected_ordered == "retained";
        conservative_disagreements += usize::from(conservative_reachable != expected_reachable);
        sky_terminal_disagreements += usize::from(sky_reachable != expected_reachable);

        let bounded_transfer = observe_doom_view_transfer(&inventory, &graph, pose, start, false)
            .map_err(io::Error::other)?;
        let bounded_sky = observe_doom_view_transfer(&inventory, &graph, pose, start, true)
            .map_err(io::Error::other)?;
        let bounded_target = target_transfer_state(&graph, &bounded_transfer, &targets, [0.0, 0.0]);
        let bounded_sky_target = target_transfer_state(&graph, &bounded_sky, &targets, [0.0, 0.0]);
        let bounded_reachable = bounded_target.is_some();
        let bounded_sky_reachable = bounded_sky_target.is_some();
        bounded_transfer_disagreements += usize::from(bounded_reachable != expected_reachable);
        bounded_sky_disagreements += usize::from(bounded_sky_reachable != expected_reachable);
        // Variant C deliberately retains the existing ordered source oracle
        // for geometrically relevant contributions outside the physical
        // aperture domain. This measures whether the aperture state actually
        // localizes closure work; it does not grant the fallback new authority.
        let bounded_closure_outcome = ordered_outcome;
        let bounded_closure_fallback = !bounded_reachable;
        bounded_closure_disagreements +=
            usize::from(bounded_closure_outcome != case.expected_ordered);
        let plane_cell_reached =
            |record_index: u32| !bounded_transfer.states_by_cell[record_index as usize].is_empty();
        let total_relevant_surfaces = source_observation
            .plane_entries
            .iter()
            .filter(|entry| {
                entry.disposition != RenderSubsectorSurfaceShadowDisposition::OutsideFrustum
            })
            .count()
            + source_observation
                .wall_tier_entries
                .iter()
                .filter(|entry| {
                    entry.disposition != RenderSubsectorSurfaceShadowDisposition::OutsideFrustum
                })
                .count();
        let reached_relevant_surfaces = source_observation
            .plane_entries
            .iter()
            .filter(|entry| {
                plane_cell_reached(entry.source_subsector.record_index)
                    && entry.disposition != RenderSubsectorSurfaceShadowDisposition::OutsideFrustum
            })
            .count()
            + source_observation
                .wall_tier_entries
                .iter()
                .filter(|entry| {
                    plane_cell_reached(entry.source_subsector.record_index)
                        && entry.disposition
                            != RenderSubsectorSurfaceShadowDisposition::OutsideFrustum
                })
                .count();
        let rescued_retained_surfaces = source_observation
            .plane_entries
            .iter()
            .filter(|entry| {
                !plane_cell_reached(entry.source_subsector.record_index)
                    && entry.disposition
                        == RenderSubsectorSurfaceShadowDisposition::RetainedGeometry
            })
            .count()
            + source_observation
                .wall_tier_entries
                .iter()
                .filter(|entry| {
                    !plane_cell_reached(entry.source_subsector.record_index)
                        && entry.disposition
                            == RenderSubsectorSurfaceShadowDisposition::RetainedGeometry
                })
                .count();
        let reached_source_covered_surfaces = source_observation
            .plane_entries
            .iter()
            .filter(|entry| {
                plane_cell_reached(entry.source_subsector.record_index)
                    && entry.disposition == RenderSubsectorSurfaceShadowDisposition::SourceCovered
            })
            .count()
            + source_observation
                .wall_tier_entries
                .iter()
                .filter(|entry| {
                    plane_cell_reached(entry.source_subsector.record_index)
                        && entry.disposition
                            == RenderSubsectorSurfaceShadowDisposition::SourceCovered
                })
                .count();
        total_bounded_states += bounded_transfer.states.len();
        peak_bounded_states = peak_bounded_states.max(bounded_transfer.states.len());
        total_relevant_surface_observations += total_relevant_surfaces;
        total_reached_relevant_surfaces += reached_relevant_surfaces;
        total_rescued_retained_surfaces += rescued_retained_surfaces;
        total_reached_source_covered_surfaces += reached_source_covered_surfaces;

        let hit = spatial
            .query_source_ray(
                DoomComparativeEmbedding::CurrentReflected,
                case.origin,
                case.direction,
            )?
            .ok_or_else(|| io::Error::other(format!("{} BVH ray missed", case.label)))?;
        if hit.source_label != case.expected_global_label {
            return Err(io::Error::other(format!(
                "{} BVH ray expected {}, observed {}",
                case.label, case.expected_global_label, hit.source_label
            ))
            .into());
        }
        let ray_aperture_crossings = format_ray_aperture_crossings(
            &graph,
            case.origin,
            case.direction,
            f64::from(hit.distance),
        );
        let chain = conservative_target
            .and_then(|target| render_subsector_connectivity_path(&graph, &conservative, target))
            .map(|edges| format_connectivity_chain(&edges))
            .unwrap_or_else(|| "unreachable".to_owned());
        let first_sky_terminal = conservative_target
            .and_then(|target| render_subsector_connectivity_path(&graph, &conservative, target))
            .and_then(|edges| {
                edges.into_iter().find(|edge| {
                    edge.role == crate::RenderSubsectorConnectivityRole::PairedSkyOpening
                })
            })
            .map(format_connectivity_edge)
            .unwrap_or_else(|| "none-on-conservative-chain".to_owned());
        let bounded_lineage = bounded_target
            .and_then(|state| doom_view_transfer_chain(&graph, &bounded_transfer, state))
            .map(|chain| format_view_transfer_chain(&chain, pose))
            .unwrap_or_else(|| "unreachable-at-center".to_owned());
        let target_occurrences =
            format_target_transfer_occurrences(&graph, &bounded_transfer, &targets, pose, 4);
        println!(
            "connectivity specimen: label={}; start={}; targets={:?}; bvh=[label:{},distance:{:.3},visited-nodes:{},tested-members:{}]; ordered-oracle={}; conservative=[reachable:{},cells:{},terminal-closed:{},fail-open:{}]; sky-terminal=[reachable:{},cells:{},terminal-closed:{},terminal-sky:{},fail-open:{}]; first-sky-terminal={}; chain={}",
            case.label,
            start,
            targets,
            hit.source_label,
            hit.distance,
            hit.visited_nodes,
            hit.tested_members,
            ordered_outcome,
            conservative_reachable,
            conservative.reachable.len(),
            conservative.terminal_closed_edges,
            conservative.fail_open_edges,
            sky_reachable,
            sky_terminal.reachable.len(),
            sky_terminal.terminal_closed_edges,
            sky_terminal.terminal_sky_edges,
            sky_terminal.fail_open_edges,
            first_sky_terminal,
            chain,
        );
        println!(
            "view-transfer specimen: label={}; variant-b=[center-reachable:{},states:{},attempted:{},dominated:{},cells:{},repeated-cells:{},max-occurrences:{},max-depth:{},near-plane-fail-open:{},outside-view:{},fingerprint:{:016x}]; variant-c=[outcome:{},target-fallback:{},relevant-surfaces:{},reached-relevant:{},outside-transfer-relevant:{},rescued-retained:{},reached-source-covered:{}]; variant-d=[center-reachable:{},states:{},terminal-sky:{},fingerprint:{:016x}]; lineage={}; target-occurrences={}; ray-aperture-crossings={}",
            case.label,
            bounded_reachable,
            bounded_transfer.states.len(),
            bounded_transfer.attempted_states,
            bounded_transfer.dominated_states,
            bounded_transfer
                .states_by_cell
                .iter()
                .filter(|states| !states.is_empty())
                .count(),
            bounded_transfer.repeated_destination_cells,
            bounded_transfer.maximum_occurrences_per_cell,
            bounded_transfer.maximum_depth,
            bounded_transfer.near_plane_fail_open,
            bounded_transfer.outside_view,
            bounded_transfer.fingerprint,
            bounded_closure_outcome,
            bounded_closure_fallback,
            total_relevant_surfaces,
            reached_relevant_surfaces,
            total_relevant_surfaces.saturating_sub(reached_relevant_surfaces),
            rescued_retained_surfaces,
            reached_source_covered_surfaces,
            bounded_sky_reachable,
            bounded_sky.states.len(),
            bounded_sky.terminal_sky,
            bounded_sky.fingerprint,
            bounded_lineage,
            target_occurrences,
            ray_aperture_crossings,
        );
        matrix_fingerprint ^= conservative.fingerprint;
        matrix_fingerprint = matrix_fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
        matrix_fingerprint ^= sky_terminal.fingerprint;
        matrix_fingerprint = matrix_fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
        matrix_fingerprint ^= bounded_transfer.fingerprint;
        matrix_fingerprint = matrix_fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
        matrix_fingerprint ^= bounded_sky.fingerprint;
        matrix_fingerprint = matrix_fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
    }
    println!(
        "E1M1 render-subsector connectivity disposition: cases=6; conservative-ordered-disagreements={}; sky-terminal-ordered-disagreements={}; bounded-transfer-ordered-disagreements={}; bounded-sky-ordered-disagreements={}; bounded-closure-ordered-disagreements={}; bounded-states=[total:{},peak:{}]; relevant-surfaces=[total:{},reached:{},outside-transfer:{},rescued-retained:{},reached-source-covered:{}]; matrix-fingerprint={:016x}; bvh-role=exact-geometric-relevance; connectivity-role=shadow-reachability; aperture-role=bounded-view-domain; submission-changes=none",
        conservative_disagreements,
        sky_terminal_disagreements,
        bounded_transfer_disagreements,
        bounded_sky_disagreements,
        bounded_closure_disagreements,
        total_bounded_states,
        peak_bounded_states,
        total_relevant_surface_observations,
        total_reached_relevant_surfaces,
        total_relevant_surface_observations.saturating_sub(total_reached_relevant_surfaces),
        total_rescued_retained_surfaces,
        total_reached_source_covered_surfaces,
        matrix_fingerprint,
    );
    Ok(())
}

fn connectivity_target_subsectors(
    inventory: &RenderSubsectorInventory,
    target: ConnectivityTarget,
) -> Vec<u32> {
    match target {
        ConnectivityTarget::Wall(segs) => inventory
            .subsectors
            .iter()
            .filter(|subsector| {
                subsector
                    .wall_sources
                    .iter()
                    .any(|wall| segs.contains(&wall.source_seg.record_index))
            })
            .map(|subsector| subsector.source_subsector.record_index)
            .collect(),
        ConnectivityTarget::Plane(subsector) => vec![subsector],
    }
}

#[derive(Clone, Copy)]
enum ConnectivityTarget {
    Wall(&'static [u32]),
    Plane(u32),
}

fn target_transfer_state(
    graph: &RenderSubsectorConnectivityGraph,
    observation: &DoomViewTransferObservation,
    targets: &[u32],
    sample_ndc: [f32; 2],
) -> Option<usize> {
    targets
        .iter()
        .flat_map(|target| observation.states_by_cell[*target as usize].iter())
        .copied()
        .filter(|state| {
            observation.states[*state]
                .window
                .contains_sample(sample_ndc)
        })
        .filter_map(|state| {
            doom_view_transfer_chain(graph, observation, state).map(|chain| (chain.len(), state))
        })
        .min()
        .map(|(_, state)| state)
}

fn format_view_transfer_chain(
    chain: &[(
        &crate::DoomViewTransferState,
        &RenderSubsectorConnectivityEdge,
    )],
    pose: RenderSubsectorViewPose,
) -> String {
    if chain.is_empty() {
        return "same-cell".to_owned();
    }
    let vertical_tangent = (pose.vertical_fov_degrees.to_radians() * 0.5).tan();
    let horizontal_tangent = vertical_tangent * pose.viewport[0] as f32 / pose.viewport[1] as f32;
    chain
        .iter()
        .map(|(state, edge)| {
            let horizontal = [
                (state.window.minimum_ndc[0] * horizontal_tangent)
                    .atan()
                    .to_degrees(),
                (state.window.maximum_ndc[0] * horizontal_tangent)
                    .atan()
                    .to_degrees(),
            ];
            let vertical = [
                (state.window.minimum_ndc[1] * vertical_tangent)
                    .atan()
                    .to_degrees(),
                (state.window.maximum_ndc[1] * vertical_tangent)
                    .atan()
                    .to_degrees(),
            ];
            format!(
                "{}>{}:{}:h=[{:.2},{:.2}]deg:v=[{:.2},{:.2}]deg:depth=[{:.2},{:.2}]{}",
                edge.from_subsector,
                edge.to_subsector,
                edge.role.label(),
                horizontal[0],
                horizontal[1],
                vertical[0],
                vertical[1],
                state.window.depth[0],
                state.window.depth[1],
                if state.near_plane_fail_open {
                    ":near-plane-fail-open"
                } else {
                    ""
                },
            )
        })
        .collect::<Vec<_>>()
        .join(" -> ")
}

fn format_target_transfer_occurrences(
    graph: &RenderSubsectorConnectivityGraph,
    observation: &DoomViewTransferObservation,
    targets: &[u32],
    pose: RenderSubsectorViewPose,
    limit: usize,
) -> String {
    let mut reports = Vec::new();
    for target in targets {
        for state_index in observation.states_by_cell[*target as usize]
            .iter()
            .take(limit)
        {
            let state = &observation.states[*state_index];
            let chain = doom_view_transfer_chain(graph, observation, *state_index)
                .map(|chain| format_view_transfer_chain(&chain, pose))
                .unwrap_or_else(|| "invalid-lineage".to_owned());
            reports.push(format!(
                "cell={target}:state={state_index}:ndc=[{:.3},{:.3}]..[{:.3},{:.3}]:center={}:chain={chain}",
                state.window.minimum_ndc[0],
                state.window.minimum_ndc[1],
                state.window.maximum_ndc[0],
                state.window.maximum_ndc[1],
                state.window.contains_sample([0.0, 0.0]),
            ));
        }
    }
    if reports.is_empty() {
        "none".to_owned()
    } else {
        reports.join(" | ")
    }
}

fn format_ray_aperture_crossings(
    graph: &RenderSubsectorConnectivityGraph,
    origin: [f64; 3],
    direction: [f64; 3],
    maximum_distance: f64,
) -> String {
    let cross = |left: [f64; 2], right: [f64; 2]| left[0] * right[1] - left[1] * right[0];
    let ray = [direction[0], direction[1]];
    let mut crossings = graph
        .edges
        .iter()
        .step_by(2)
        .filter_map(|edge| {
            let segment = [
                edge.shared_interval[1][0] - edge.shared_interval[0][0],
                edge.shared_interval[1][1] - edge.shared_interval[0][1],
            ];
            let denominator = cross(ray, segment);
            if denominator.abs() <= 1.0e-9 {
                return None;
            }
            let offset = [
                edge.shared_interval[0][0] - origin[0],
                edge.shared_interval[0][1] - origin[1],
            ];
            let distance = cross(offset, segment) / denominator;
            let parameter = cross(offset, ray) / denominator;
            if distance <= 1.0e-6
                || distance > maximum_distance + 1.0e-3
                || !(-1.0e-6..=1.0 + 1.0e-6).contains(&parameter)
            {
                return None;
            }
            let height = origin[2] + direction[2] * distance;
            Some((distance, height, edge))
        })
        .collect::<Vec<_>>();
    crossings.sort_by(|left, right| left.0.total_cmp(&right.0));
    if crossings.is_empty() {
        return "none".to_owned();
    }
    crossings
        .into_iter()
        .map(|(distance, height, edge)| {
            format!(
                "t={distance:.2}:{}<> {}:{}:height={height:.2}:opening=[{},{}]:inside={}",
                edge.from_subsector,
                edge.to_subsector,
                edge.role.label(),
                edge.opening_bottom,
                edge.opening_top,
                height >= f64::from(edge.opening_bottom) - 1.0e-6
                    && height <= f64::from(edge.opening_top) + 1.0e-6,
            )
        })
        .collect::<Vec<_>>()
        .join(" -> ")
}

fn shortest_reachable_target(
    graph: &RenderSubsectorConnectivityGraph,
    observation: &crate::RenderSubsectorConnectivityObservation,
    targets: &[u32],
) -> Option<u32> {
    targets
        .iter()
        .copied()
        .filter_map(|target| {
            render_subsector_connectivity_path(graph, observation, target)
                .map(|path| (path.len(), target))
        })
        .min()
        .map(|(_, target)| target)
}

fn format_connectivity_chain(edges: &[&RenderSubsectorConnectivityEdge]) -> String {
    if edges.is_empty() {
        return "same-cell".to_owned();
    }
    edges
        .iter()
        .map(|edge| format_connectivity_edge(edge))
        .collect::<Vec<_>>()
        .join(" -> ")
}

fn format_connectivity_edge(edge: &RenderSubsectorConnectivityEdge) -> String {
    format!(
        "{}>{}:{}:linedef={}:{}",
        edge.from_subsector,
        edge.to_subsector,
        edge.role.label(),
        edge.source_linedef
            .map(|source| source.record_index.to_string())
            .unwrap_or_else(|| "none".to_owned()),
        edge.reason,
    )
}

fn report_render_subsector_six_ray_falsifiers(
    scene: &SceneInput,
    inventory: &RenderSubsectorInventory,
) -> PlatformResult<u64> {
    #[derive(Clone, Copy)]
    enum Target {
        RejectedWall(&'static [u32]),
        RejectedPlane(u32, DoomSurfacePlane),
        RetainedPlane(u32, DoomSurfacePlane),
    }
    #[derive(Clone, Copy)]
    struct Case {
        label: &'static str,
        origin: [f64; 3],
        direction: [f64; 3],
        target: Target,
    }
    const WALL_230: &[u32] = &[415, 423];
    const WALL_247: &[u32] = &[559, 567];
    let cases = [
        Case {
            label: "hut-east-wall-230",
            origin: [2076.0, -3560.0, 36.0],
            direction: [0.905568898, -0.424199343, 0.0],
            target: Target::RejectedWall(WALL_230),
        },
        Case {
            label: "wall-247-east",
            origin: [1306.508666992, -3272.168457031, 21.432840347],
            direction: [0.939651787, -0.338751376, 0.047981590],
            target: Target::RejectedWall(WALL_247),
        },
        Case {
            label: "ceiling-104-reached",
            origin: [1477.330444336, -3594.213134766, 8.994521141],
            direction: [-0.792175531, -0.565008104, 0.230702817],
            target: Target::RetainedPlane(104, DoomSurfacePlane::Ceiling),
        },
        Case {
            label: "wall-247-west",
            origin: [2115.047851562, -3569.925048828, 8.994521141],
            direction: [0.928815067, -0.358562857, 0.093463443],
            target: Target::RejectedWall(WALL_247),
        },
        Case {
            label: "ceiling-149-rejected",
            origin: [2139.683349609, -3196.036376953, 8.994521141],
            direction: [0.180356100, 0.780082107, 0.599119186],
            target: Target::RejectedPlane(149, DoomSurfacePlane::Ceiling),
        },
        Case {
            label: "ceiling-104-rejected",
            origin: [2902.150878906, -3206.857421875, 8.994521141],
            direction: [-0.952072978, -0.304107845, 0.032795019],
            target: Target::RejectedPlane(104, DoomSurfacePlane::Ceiling),
        },
    ];
    let mut fingerprint = 0xcbf2_9ce4_8422_2325_u64;
    let mut reports = Vec::new();
    for case in cases {
        let horizontal = case.direction[0].hypot(case.direction[1]);
        let pose = RenderSubsectorViewPose {
            label: case.label,
            source_position: [case.origin[0], case.origin[1]],
            eye_height: case.origin[2],
            heading_degrees: case.direction[1].atan2(case.direction[0]).to_degrees() as f32,
            pitch_degrees: case.direction[2].atan2(horizontal).to_degrees() as f32,
            viewport: BASELINE_VIEWPORT,
            vertical_fov_degrees: BASELINE_VERTICAL_FOV_DEGREES,
        };
        let observation = observe_render_subsector_actual_camera(
            &scene.door_geometry_source.map,
            inventory,
            pose,
        )
        .map_err(io::Error::other)?;
        let result = match case.target {
            Target::RejectedWall(segs) => {
                let entries = observation
                    .wall_tier_entries
                    .iter()
                    .filter(|entry| segs.contains(&entry.source_seg.record_index))
                    .collect::<Vec<_>>();
                let covered = entries
                    .iter()
                    .filter(|entry| {
                        entry.disposition == RenderSubsectorSurfaceShadowDisposition::SourceCovered
                    })
                    .count();
                let retained = entries
                    .iter()
                    .filter(|entry| {
                        entry.disposition
                            == RenderSubsectorSurfaceShadowDisposition::RetainedGeometry
                    })
                    .count();
                let unresolved = entries
                    .iter()
                    .filter(|entry| {
                        entry.disposition == RenderSubsectorSurfaceShadowDisposition::Unresolved
                    })
                    .count();
                if entries.is_empty() || retained != 0 || unresolved != 0 {
                    return Err(io::Error::other(format!("render-subsector six-ray falsifier {} disagreed: target wall SEGs {segs:?}, entries={}, covered={covered}, retained={retained}, unresolved={unresolved}", case.label, entries.len())).into());
                }
                format!(
                    "{}=rejected-wall:segs={segs:?}:covered={covered}:outside={}",
                    case.label,
                    entries.len() - covered
                )
            }
            Target::RejectedPlane(subsector, plane) | Target::RetainedPlane(subsector, plane) => {
                let entry = observation.plane_entries.iter().find(|entry| entry.source_subsector.record_index == subsector && entry.plane == plane).ok_or_else(|| io::Error::other(format!("render-subsector six-ray falsifier {} has no {plane:?} entry for subsector {subsector}", case.label)))?;
                let expected = if matches!(case.target, Target::RejectedPlane(_, _)) {
                    RenderSubsectorSurfaceShadowDisposition::SourceCovered
                } else {
                    RenderSubsectorSurfaceShadowDisposition::RetainedGeometry
                };
                if entry.disposition != expected {
                    return Err(io::Error::other(format!("render-subsector six-ray falsifier {} expected {expected:?} for subsector {subsector} {plane:?}, observed {:?}", case.label, entry.disposition)).into());
                }
                format!(
                    "{}={:?}:subsector={subsector}:plane={plane:?}",
                    case.label, entry.disposition
                )
            }
        };
        fingerprint ^= observation.view_fingerprint;
        fingerprint = fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
        fingerprint ^= observation.source_coverage_fingerprint;
        fingerprint = fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
        reports.push(result);
    }
    println!("E1M1 render-subsector six-ray falsifier replay: cases={}; presentation-authority=disabled; results=[{}]", reports.len(), reports.join(" | "));
    Ok(fingerprint)
}

fn triangle_count(draws: &[hello_doom_e1m1::StaticDrawPlanEntry]) -> usize {
    draws.iter().map(|draw| draw.mesh.positions.len() / 3).sum()
}
