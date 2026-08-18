//! Exact-ray shadow for source-occurrence support over reconstructed geometry.
//!
//! The bounded Classic columns/rows remain Doom-private evidence. This module
//! maps replayable source rays into that source projection and compares final
//! wall/plane cell support with exact prepared geometry; it changes no draws.

use super::super::*;
use super::ordered_causality::{
    ordered_six_ray_cases, positive_wall_support_control, OrderedSixRayExpectedTarget,
};
use doom_geometry_provider::DoomSegClassicPlaneKey;

const SOURCE_COLUMNS: usize = 320;
const SOURCE_ROWS: usize = 200;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExpectedCellSupport {
    Supported,
    Unsupported,
}

impl ExpectedCellSupport {
    const fn label(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct WalkaboutCapture {
    name: &'static str,
    origin: [f64; 3],
    direction: [f64; 3],
    expected_global_label: &'static str,
    target: CaptureTarget,
    expected_cell_support: ExpectedCellSupport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CaptureTarget {
    Plane {
        subsector: u32,
        sector: u32,
        plane: DoomSurfacePlane,
    },
    Wall {
        linedef: u32,
    },
}

const WALKABOUT_CAPTURES: [WalkaboutCapture; 5] = [
    WalkaboutCapture {
        name: "spawn-window-ceiling-55",
        origin: [-227.041_458_130, -3_152.000_976_562, 140.0],
        direction: [0.511_952_519, 0.852_522_492, 0.105_403_915],
        expected_global_label: "flat:24:FLOOR7_2",
        target: CaptureTarget::Plane {
            subsector: 55,
            sector: 24,
            plane: DoomSurfacePlane::Ceiling,
        },
        expected_cell_support: ExpectedCellSupport::Unsupported,
    },
    WalkaboutCapture {
        name: "spawn-window-ceiling-54",
        origin: [-205.733_337_402, -3_311.999_023_438, 140.0],
        direction: [0.475_691_646, -0.874_108_732, 0.098_241_337],
        expected_global_label: "flat:24:FLOOR7_2",
        target: CaptureTarget::Plane {
            subsector: 54,
            sector: 24,
            plane: DoomSurfacePlane::Ceiling,
        },
        expected_cell_support: ExpectedCellSupport::Unsupported,
    },
    WalkaboutCapture {
        name: "hut-ceiling-117",
        origin: [2_248.567_138_672, -3_360.645_263_672, -20.0],
        direction: [-0.929_613_948, -0.359_226_376, 0.082_306_169],
        expected_global_label: "flat:41:CEIL3_5",
        target: CaptureTarget::Plane {
            subsector: 117,
            sector: 41,
            plane: DoomSurfacePlane::Ceiling,
        },
        expected_cell_support: ExpectedCellSupport::Unsupported,
    },
    WalkaboutCapture {
        name: "hut-wall-241",
        origin: [2_042.021_240_234, -2_975.617_919_922, -20.0],
        direction: [0.613_750_577, -0.787_513_614, 0.055_970_095],
        expected_global_label: "wall:241:BROWN1",
        target: CaptureTarget::Wall { linedef: 241 },
        expected_cell_support: ExpectedCellSupport::Unsupported,
    },
    WalkaboutCapture {
        name: "hut-local-floor-130",
        origin: [2_447.991_455_078, -3_216.869_628_906, -20.0],
        direction: [-0.941_134_751, -0.201_053_977, -0.271_740_079],
        expected_global_label: "flat:5:FLOOR7_1",
        target: CaptureTarget::Plane {
            subsector: 130,
            sector: 5,
            plane: DoomSurfacePlane::Floor,
        },
        expected_cell_support: ExpectedCellSupport::Supported,
    },
];

#[derive(Clone, Debug, Eq, PartialEq)]
enum CellSupport {
    Supported(String),
    Unsupported(String),
    Unresolved(String),
}

impl CellSupport {
    const fn label(&self) -> &'static str {
        match self {
            Self::Supported(_) => "supported",
            Self::Unsupported(_) => "unsupported",
            Self::Unresolved(_) => "unresolved-fail-open",
        }
    }

    fn detail(&self) -> &str {
        match self {
            Self::Supported(detail) | Self::Unsupported(detail) | Self::Unresolved(detail) => {
                detail
            }
        }
    }
}

pub(crate) fn report_source_occurrence_support(scene: &SceneInput) -> PlatformResult<()> {
    let spatial = super::tokimu_spatial_bake::SpatialRayShadow::build(scene)?;
    let cutout_materials = scene
        .cutout_uploads
        .iter()
        .map(|upload| (upload.source_name.clone(), upload.material))
        .collect::<BTreeMap<_, _>>();
    let wall_triangles = lower_doom_seg_textured_wall_triangles(
        &scene.door_geometry_source.map,
        &scene.door_geometry_source.wall_extents,
    )?;
    let mut expected_passed = 0usize;
    let mut expected_total = 0usize;
    let mut plane_shadow_passed = 0usize;
    let mut plane_shadow_total = 0usize;
    let mut rows = Vec::new();
    let mut fingerprint = fnv_offset();

    for capture in WALKABOUT_CAPTURES {
        let exact = spatial
            .query_source_ray(
                DoomComparativeEmbedding::CurrentReflected,
                capture.origin,
                capture.direction,
            )?
            .ok_or_else(|| {
                io::Error::other(format!(
                    "source-occurrence capture {} has no complete-shell hit",
                    capture.name,
                ))
            })?;
        let (world_origin, world_direction) = source_ray_in_prepared_frame(capture);
        let global_hit = nearest_prepared_ray_hit(
            world_origin,
            world_direction,
            &scene.opaque_draws,
            Some(&scene.cutout_draws),
        )
        .ok_or_else(|| {
            io::Error::other(format!(
                "source-occurrence capture {} lost its brute-force complete-shell hit",
                capture.name,
            ))
        })?;
        if global_hit.draw.source_label != exact.source_label {
            return Err(io::Error::other(format!(
                "source-occurrence capture {} BVH/brute disagreement: {} versus {}",
                capture.name, exact.source_label, global_hit.draw.source_label,
            ))
            .into());
        }
        let target_draws = scene
            .opaque_draws
            .iter()
            .chain(&scene.cutout_draws)
            .filter(|draw| capture_target_matches(draw, capture.target))
            .cloned()
            .collect::<Vec<_>>();
        let target_hit =
            nearest_prepared_ray_hit(world_origin, world_direction, &target_draws, None)
                .ok_or_else(|| {
                    io::Error::other(format!(
                        "source-occurrence capture {} has no exact target intersection",
                        capture.name,
                    ))
                })?;
        if target_hit.draw.source_label != capture.expected_global_label {
            return Err(io::Error::other(format!(
                "source-occurrence capture {} expected target label {} but correlated {}",
                capture.name, capture.expected_global_label, target_hit.draw.source_label,
            ))
            .into());
        }

        let viewer = [
            capture.origin[0].round() as i16,
            capture.origin[1].round() as i16,
        ];
        let heading = capture.direction[1].atan2(capture.direction[0]);
        let eye_height = capture.origin[2].round() as i16;
        let sample = source_projection_sample(capture.direction, heading);
        let traversal = observe_doom_classic_bsp(
            &scene.door_geometry_source.map,
            viewer,
            heading,
            &BTreeSet::new(),
        )?;
        let plane_marks =
            observe_doom_seg_plane_marks(&scene.door_geometry_source.map, eye_height)?;
        let vertical = observe_shared_doom_classic_vertical_clip_state(
            &scene.door_geometry_source.map,
            &wall_triangles,
            &plane_marks,
            &traversal,
            viewer,
            heading,
            f64::from(eye_height),
        );
        let support = match sample.as_ref() {
            Ok(&(column, row)) => source_cell_support(
                &scene.door_geometry_source.map,
                &vertical,
                target_hit.draw,
                column,
                row,
            ),
            Err(reason) => CellSupport::Unresolved(reason.clone()),
        };

        let source_covered = crate::render_strategies::source_covered_global_shell::prepare(
            scene,
            &scene.door_geometry_source.map,
            viewer,
            heading,
        )?;
        let source_covered_hit = nearest_prepared_ray_hit(
            world_origin,
            world_direction,
            &source_covered.opaque_draws,
            Some(&source_covered.cutout_draws),
        )
        .map(|hit| hit.draw.source_label.as_str())
        .unwrap_or("none");
        let source_covered_target_draws = source_covered
            .opaque_draws
            .iter()
            .chain(&source_covered.cutout_draws)
            .filter(|draw| capture_target_matches(draw, capture.target))
            .cloned()
            .collect::<Vec<_>>();
        let source_covered_target_hit = nearest_prepared_ray_hit(
            world_origin,
            world_direction,
            &source_covered_target_draws,
            None,
        )
        .map(|hit| format!("{:.3}", hit.distance))
        .unwrap_or_else(|| "none".to_owned());

        let ordered = prepare_ordered_occurrence_submission(
            &scene.door_geometry_source.map,
            viewer,
            heading,
            eye_height,
            &scene.door_geometry_source.wall_extents,
            &scene.door_geometry_source.wall_materials,
            &cutout_materials,
            &scene.opaque_uploads,
        )
        .map_err(io::Error::other)?;
        ordered.verify_conservation().map_err(io::Error::other)?;
        let ordered_detail =
            ordered_target_detail(&ordered, target_hit.draw, world_origin, world_direction);
        let plane_shadow_detail = match capture.target {
            CaptureTarget::Wall { .. } => "not-applicable:wall-target".to_owned(),
            CaptureTarget::Plane { .. } => {
                let shadow = prepare_plane_cell_geometry_support_shadow(
                    &scene.door_geometry_source.map,
                    viewer,
                    heading,
                    eye_height,
                    &scene.door_geometry_source.wall_extents,
                    &scene.opaque_uploads,
                )?;
                shadow.verify_conservation().map_err(io::Error::other)?;
                let target_draws = shadow
                    .draws
                    .iter()
                    .filter(|draw| capture_target_matches(draw, capture.target))
                    .cloned()
                    .collect::<Vec<_>>();
                let target_distance =
                    nearest_prepared_ray_hit(world_origin, world_direction, &target_draws, None)
                        .map(|hit| format!("{:.3}", hit.distance));
                plane_shadow_total += 1;
                let expected_hit = capture.expected_cell_support == ExpectedCellSupport::Supported;
                plane_shadow_passed += usize::from(target_distance.is_some() == expected_hit);
                format!(
                    "target-distance={}; {}",
                    target_distance.unwrap_or_else(|| "none".to_owned()),
                    shadow.report(),
                )
            }
        };

        expected_total += 1;
        let passed = support.label() == capture.expected_cell_support.label();
        expected_passed += usize::from(passed);
        let expected_result = if passed { "pass" } else { "fail" };
        let (column, row) = sample
            .map(|(column, row)| (column.to_string(), row.to_string()))
            .unwrap_or_else(|_| ("none".to_owned(), "none".to_owned()));
        let row_report = format!(
            "case={}:global-nearest={}:global-nearest-distance={:.3}:target={}:target-distance={:.3}:sample=({column},{row}):cell-support={}:cell-detail={}:expected={}:result={expected_result}:source-covered-nearest={source_covered_hit}:source-covered-target-distance={source_covered_target_hit}:ordered=[{ordered_detail}]:plane-cell-geometry-shadow=[{plane_shadow_detail}]",
            capture.name,
            global_hit.draw.source_label,
            global_hit.distance,
            target_hit.draw.source_label,
            target_hit.distance,
            support.label(),
            support.detail(),
            capture.expected_cell_support.label(),
        );
        hash_text(&mut fingerprint, &row_report);
        rows.push(row_report);
    }

    let (retained_passed, retained_total, retained_rows) =
        retained_control_matrix(scene, &wall_triangles, &mut fingerprint)?;

    if expected_passed != expected_total {
        return Err(io::Error::other(format!(
            "source-occurrence support expected controls failed: passed={expected_passed}/{expected_total}; rows=[{}]",
            rows.join(" | "),
        ))
        .into());
    }
    if plane_shadow_passed != plane_shadow_total {
        return Err(io::Error::other(format!(
            "plane-cell geometry shadow controls failed: passed={plane_shadow_passed}/{plane_shadow_total}; rows=[{}]",
            rows.join(" | "),
        ))
        .into());
    }
    if retained_passed != retained_total {
        return Err(io::Error::other(format!(
            "source-occurrence support hypothesis falsified: capture-cells={expected_passed}/{expected_total}:plane-geometry={plane_shadow_passed}/{plane_shadow_total}:retained-exact-cells={retained_passed}/{retained_total}:fingerprint={fingerprint:016x}; retained-rows=[{}]; capture-rows=[{}]",
            retained_rows.join(" | "),
            rows.join(" | "),
        ))
        .into());
    }
    println!(
        "E1M1 source-occurrence support Slice 0-2: captures={}; cell-controls={expected_passed}/{expected_total}; plane-geometry-controls={plane_shadow_passed}/{plane_shadow_total}; retained-controls={retained_passed}/{retained_total}; wall-241=unsupported-final-wall-cell; projection={}x{}; fingerprint={fingerprint:016x}; renderer-mutation=false; rows=[{}]; retained-rows=[{}]",
        WALKABOUT_CAPTURES.len(),
        SOURCE_COLUMNS,
        SOURCE_ROWS,
        rows.join(" | "),
        retained_rows.join(" | "),
    );
    Ok(())
}

fn retained_control_matrix(
    scene: &SceneInput,
    wall_triangles: &[DoomSegTexturedWallTriangle],
    fingerprint: &mut u64,
) -> PlatformResult<(usize, usize, Vec<String>)> {
    let cutout_materials = scene
        .cutout_uploads
        .iter()
        .map(|upload| (upload.source_name.clone(), upload.material))
        .collect::<BTreeMap<_, _>>();
    let mut passed = 0usize;
    let mut rows = Vec::new();
    for case in ordered_six_ray_cases() {
        let (origin, direction) = source_ray_vectors(case.origin, case.direction);
        let target_draws = scene
            .opaque_draws
            .iter()
            .chain(&scene.cutout_draws)
            .filter(|draw| six_ray_target_matches(draw, case.expected))
            .cloned()
            .collect::<Vec<_>>();
        let target_hit = nearest_prepared_ray_hit(origin, direction, &target_draws, None)
            .ok_or_else(|| {
                io::Error::other(format!(
                    "retained source-cell control {} lost its exact target hit",
                    case.name,
                ))
            })?;
        if target_hit.draw.source_label != case.expected_global_label {
            return Err(io::Error::other(format!(
                "retained source-cell control {} expected {} but correlated {}",
                case.name, case.expected_global_label, target_hit.draw.source_label,
            ))
            .into());
        }
        let viewer = [case.origin[0].round() as i16, case.origin[1].round() as i16];
        let heading = case.direction[1].atan2(case.direction[0]);
        let eye_height = case.origin[2].round() as i16;
        let traversal = observe_doom_classic_bsp(
            &scene.door_geometry_source.map,
            viewer,
            heading,
            &BTreeSet::new(),
        )?;
        let marks = observe_doom_seg_plane_marks(&scene.door_geometry_source.map, eye_height)?;
        let vertical = observe_shared_doom_classic_vertical_clip_state(
            &scene.door_geometry_source.map,
            wall_triangles,
            &marks,
            &traversal,
            viewer,
            heading,
            f64::from(eye_height),
        );
        let ordered = prepare_ordered_occurrence_submission(
            &scene.door_geometry_source.map,
            viewer,
            heading,
            eye_height,
            &scene.door_geometry_source.wall_extents,
            &scene.door_geometry_source.wall_materials,
            &cutout_materials,
            &scene.opaque_uploads,
        )
        .map_err(io::Error::other)?;
        ordered.verify_conservation().map_err(io::Error::other)?;
        let ordered_target = ordered_target_detail(&ordered, target_hit.draw, origin, direction);
        let support = match source_projection_sample(case.direction, heading) {
            Ok((column, row)) => source_cell_support(
                &scene.door_geometry_source.map,
                &vertical,
                target_hit.draw,
                column,
                row,
            ),
            Err(reason) => source_support_without_projection(
                &scene.door_geometry_source.map,
                &vertical,
                target_hit.draw,
                reason,
            ),
        };
        let expected_supported = matches!(
            case.expected,
            OrderedSixRayExpectedTarget::PartialPlane { .. }
        );
        let cell_passed = matches!(support, CellSupport::Supported(_)) == expected_supported;
        let plane_shadow = match case.expected {
            OrderedSixRayExpectedTarget::RejectedWallSegs { .. } => None,
            OrderedSixRayExpectedTarget::RejectedPlane { .. }
            | OrderedSixRayExpectedTarget::PartialPlane { .. } => {
                let shadow = prepare_plane_cell_geometry_support_shadow(
                    &scene.door_geometry_source.map,
                    viewer,
                    heading,
                    eye_height,
                    &scene.door_geometry_source.wall_extents,
                    &scene.opaque_uploads,
                )?;
                shadow.verify_conservation().map_err(io::Error::other)?;
                let draws = shadow
                    .draws
                    .iter()
                    .filter(|draw| six_ray_target_matches(draw, case.expected))
                    .cloned()
                    .collect::<Vec<_>>();
                Some(nearest_prepared_ray_hit(origin, direction, &draws, None).is_some())
            }
        };
        let shadow_passed = plane_shadow.is_none_or(|hit| hit == expected_supported);
        let case_passed = cell_passed && shadow_passed;
        passed += usize::from(case_passed);
        let report = format!(
            "case={}:expected={}:cell={}:cell-detail={}:ordered-target=[{ordered_target}]:plane-shadow={}:result={}",
            case.name,
            if expected_supported {
                "supported"
            } else {
                "unsupported"
            },
            support.label(),
            support.detail(),
            plane_shadow
                .map(|hit| if hit { "hit" } else { "none" })
                .unwrap_or("not-applicable"),
            if case_passed { "pass" } else { "fail" },
        );
        hash_text(fingerprint, &report);
        rows.push(report);
    }

    let (positive_origin, positive_direction, positive_linedef, positive_label) =
        positive_wall_support_control();
    let (origin, direction) = source_ray_vectors(positive_origin, positive_direction);
    let draws = scene
        .opaque_draws
        .iter()
        .chain(&scene.cutout_draws)
        .filter(|draw| {
            matches!(
                draw.source,
                StaticDrawSource::Wall { source_linedef, .. }
                    if source_linedef.record_index == positive_linedef
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    let hit = nearest_prepared_ray_hit(origin, direction, &draws, None).ok_or_else(|| {
        io::Error::other("positive wall 135 source-cell control lost its exact hit")
    })?;
    if hit.draw.source_label != positive_label {
        return Err(io::Error::other(format!(
            "positive wall source-cell control expected {positive_label} but correlated {}",
            hit.draw.source_label,
        ))
        .into());
    }
    let viewer = [
        positive_origin[0].round() as i16,
        positive_origin[1].round() as i16,
    ];
    let heading = positive_direction[1].atan2(positive_direction[0]);
    let eye_height = positive_origin[2].round() as i16;
    let (column, row) = source_projection_sample(positive_direction, heading)
        .map_err(|error| io::Error::other(format!("positive-wall-135:{error}")))?;
    let traversal = observe_doom_classic_bsp(
        &scene.door_geometry_source.map,
        viewer,
        heading,
        &BTreeSet::new(),
    )?;
    let marks = observe_doom_seg_plane_marks(&scene.door_geometry_source.map, eye_height)?;
    let vertical = observe_shared_doom_classic_vertical_clip_state(
        &scene.door_geometry_source.map,
        wall_triangles,
        &marks,
        &traversal,
        viewer,
        heading,
        f64::from(eye_height),
    );
    let support = source_cell_support(
        &scene.door_geometry_source.map,
        &vertical,
        hit.draw,
        column,
        row,
    );
    let positive_passed = matches!(support, CellSupport::Supported(_));
    passed += usize::from(positive_passed);
    let report = format!(
        "case=positive-wall-135:expected=supported:cell={}:plane-shadow=not-applicable:result={}",
        support.label(),
        if positive_passed { "pass" } else { "fail" },
    );
    hash_text(fingerprint, &report);
    rows.push(report);
    Ok((passed, ordered_six_ray_cases().len() + 1, rows))
}

fn six_ray_target_matches(
    draw: &StaticDrawPlanEntry,
    expected: OrderedSixRayExpectedTarget,
) -> bool {
    match (draw.source, expected) {
        (
            StaticDrawSource::Wall { source_linedef, .. },
            OrderedSixRayExpectedTarget::RejectedWallSegs {
                source_linedef: expected_linedef,
                ..
            },
        ) => source_linedef.record_index == expected_linedef,
        (
            StaticDrawSource::Flat {
                source_subsector,
                plane,
                ..
            },
            OrderedSixRayExpectedTarget::RejectedPlane { subsector, kind }
            | OrderedSixRayExpectedTarget::PartialPlane { subsector, kind },
        ) => {
            source_subsector.record_index == subsector
                && matches!(
                    (plane, kind),
                    (DoomSurfacePlane::Floor, OrderedPlaneKind::Floor)
                        | (DoomSurfacePlane::Ceiling, OrderedPlaneKind::Ceiling)
                )
        }
        _ => false,
    }
}

fn capture_target_matches(draw: &StaticDrawPlanEntry, target: CaptureTarget) -> bool {
    match (draw.source, target) {
        (
            StaticDrawSource::Flat {
                source_subsector,
                source_sector,
                plane,
            },
            CaptureTarget::Plane {
                subsector,
                sector,
                plane: expected_plane,
            },
        ) => {
            source_subsector.record_index == subsector
                && source_sector.record_index == sector
                && plane == expected_plane
        }
        (StaticDrawSource::Wall { source_linedef, .. }, CaptureTarget::Wall { linedef }) => {
            source_linedef.record_index == linedef
        }
        _ => false,
    }
}

fn source_ray_in_prepared_frame(capture: WalkaboutCapture) -> (Vec3, Vec3) {
    source_ray_vectors(capture.origin, capture.direction)
}

fn source_ray_vectors(origin: [f64; 3], direction: [f64; 3]) -> (Vec3, Vec3) {
    let embedding = DoomComparativeEmbedding::CurrentReflected;
    let origin = embedding.lift_direction([origin[0] as f32, origin[1] as f32], origin[2] as f32);
    let direction = embedding
        .lift_direction(
            [direction[0] as f32, direction[1] as f32],
            direction[2] as f32,
        )
        .normalize_or_zero();
    (origin, direction)
}

fn source_projection_sample(direction: [f64; 3], heading: f64) -> Result<(usize, usize), String> {
    let horizontal_length = direction[0].hypot(direction[1]);
    if !horizontal_length.is_finite() || horizontal_length <= f64::EPSILON {
        return Err("vertical-ray".to_owned());
    }
    let ray_heading = direction[1].atan2(direction[0]);
    let azimuth = normalize_angle(ray_heading - heading);
    let horizontal_normalized = azimuth.tan() / std::f64::consts::FRAC_PI_4.tan();
    let half_vertical_fov =
        (std::f64::consts::FRAC_PI_4.tan() / (SOURCE_COLUMNS as f64 / SOURCE_ROWS as f64)).atan();
    let elevation = direction[2].atan2(horizontal_length);
    let vertical_normalized = elevation.tan() / half_vertical_fov.tan();
    if !horizontal_normalized.is_finite()
        || !vertical_normalized.is_finite()
        || !(-1.0..=1.0).contains(&horizontal_normalized)
        || !(-1.0..=1.0).contains(&vertical_normalized)
    {
        return Err(format!(
            "outside-source-projection:azimuth-degrees={:.3}:elevation-degrees={:.3}",
            azimuth.to_degrees(),
            elevation.to_degrees(),
        ));
    }
    let column = (((horizontal_normalized + 1.0) * 0.5 * SOURCE_COLUMNS as f64).floor() as usize)
        .min(SOURCE_COLUMNS - 1);
    let row = (((1.0 - vertical_normalized) * 0.5 * SOURCE_ROWS as f64).floor() as usize)
        .min(SOURCE_ROWS - 1);
    Ok((column, row))
}

fn normalize_angle(mut angle: f64) -> f64 {
    while angle > std::f64::consts::PI {
        angle -= std::f64::consts::TAU;
    }
    while angle < -std::f64::consts::PI {
        angle += std::f64::consts::TAU;
    }
    angle
}

fn source_support_without_projection(
    map: &DoomMapCore,
    vertical: &DoomSegClassicVerticalClipObservation,
    draw: &StaticDrawPlanEntry,
    projection_reason: String,
) -> CellSupport {
    match draw.source {
        StaticDrawSource::Flat {
            source_sector,
            plane,
            ..
        } => {
            let Some(sector) = map
                .sectors
                .iter()
                .find(|sector| sector.source == source_sector)
            else {
                return CellSupport::Unresolved(format!(
                    "source-sector-{}-missing:{projection_reason}",
                    source_sector.record_index,
                ));
            };
            let (kind, height, texture) = match plane {
                DoomSurfacePlane::Floor => (
                    DoomSegClassicPlaneKind::Floor,
                    sector.floor_height,
                    sector.floor_texture.clone(),
                ),
                DoomSurfacePlane::Ceiling => (
                    DoomSegClassicPlaneKind::Ceiling,
                    sector.ceiling_height,
                    sector.ceiling_texture.clone(),
                ),
            };
            let key = DoomSegClassicPlaneKey {
                kind,
                height,
                texture,
                light: sector.light_level,
            };
            if vertical.plane_spans.keys.contains_key(&key) {
                CellSupport::Unresolved(format!(
                    "plane-key-present-but-{projection_reason}:sector={}:kind={kind:?}",
                    source_sector.record_index,
                ))
            } else {
                CellSupport::Unsupported(format!(
                    "plane-key-absent-before-projection:sector={}:kind={kind:?}:{projection_reason}",
                    source_sector.record_index,
                ))
            }
        }
        StaticDrawSource::Wall {
            source_linedef,
            role,
            ..
        } => {
            let retained = vertical.ordered_wall_intervals.iter().any(|interval| {
                interval.source_linedef == source_linedef.record_index
                    && interval.role == role
                    && interval.retained_interval.is_some()
            });
            if retained {
                CellSupport::Unresolved(format!(
                    "wall-tier-present-but-{projection_reason}:linedef={}:role={role:?}",
                    source_linedef.record_index,
                ))
            } else {
                CellSupport::Unsupported(format!(
                    "wall-tier-absent-before-projection:linedef={}:role={role:?}:{projection_reason}",
                    source_linedef.record_index,
                ))
            }
        }
    }
}

fn source_cell_support(
    map: &DoomMapCore,
    vertical: &DoomSegClassicVerticalClipObservation,
    draw: &StaticDrawPlanEntry,
    column: usize,
    row: usize,
) -> CellSupport {
    match draw.source {
        StaticDrawSource::Flat {
            source_sector,
            plane,
            ..
        } => {
            let Some(sector) = map
                .sectors
                .iter()
                .find(|sector| sector.source == source_sector)
            else {
                return CellSupport::Unresolved(format!(
                    "source-sector-{}-missing",
                    source_sector.record_index,
                ));
            };
            let (kind, height, texture) = match plane {
                DoomSurfacePlane::Floor => (
                    DoomSegClassicPlaneKind::Floor,
                    sector.floor_height,
                    sector.floor_texture.clone(),
                ),
                DoomSurfacePlane::Ceiling => (
                    DoomSegClassicPlaneKind::Ceiling,
                    sector.ceiling_height,
                    sector.ceiling_texture.clone(),
                ),
            };
            let key = DoomSegClassicPlaneKey {
                kind,
                height,
                texture,
                light: sector.light_level,
            };
            let Some(instances) = vertical.plane_spans.keys.get(&key) else {
                return CellSupport::Unsupported(format!(
                    "plane-key-absent:sector={}:kind={kind:?}",
                    source_sector.record_index,
                ));
            };
            let supported = instances.iter().filter_map(|instance| {
                instance
                    .columns
                    .get(column)
                    .copied()
                    .flatten()
                    .filter(|interval| (interval[0]..=interval[1]).contains(&row))
            });
            let supported_intervals = supported.collect::<Vec<_>>();
            if supported_intervals.is_empty() {
                let column_intervals = instances
                    .iter()
                    .filter_map(|instance| instance.columns.get(column).copied().flatten())
                    .collect::<Vec<_>>();
                CellSupport::Unsupported(format!(
                    "plane-cell-absent:sector-provenance={}:kind={kind:?}:key-instances={}:column={column}:row={row}:column-intervals={column_intervals:?}",
                    source_sector.record_index,
                    instances.len(),
                ))
            } else {
                CellSupport::Supported(format!(
                    "plane-cell-present:sector-provenance={}:kind={kind:?}:column={column}:row={row}:intervals={supported_intervals:?}:authority=doom-plane-key-plus-spatial-cell-not-source-sector",
                    source_sector.record_index,
                ))
            }
        }
        StaticDrawSource::Wall {
            source_linedef,
            role,
            ..
        } => {
            let matching = vertical
                .ordered_wall_intervals
                .iter()
                .filter(|interval| {
                    interval.source_linedef == source_linedef.record_index
                        && interval.column == column
                        && interval.role == role
                })
                .collect::<Vec<_>>();
            let supported = matching
                .iter()
                .filter_map(|interval| interval.retained_interval.map(|rows| (interval, rows)))
                .filter(|(_, rows)| (rows[0]..=rows[1]).contains(&row))
                .collect::<Vec<_>>();
            if supported.is_empty() {
                CellSupport::Unsupported(format!(
                    "wall-cell-absent:linedef={}:role={role:?}:column={column}:row={row}:matching-intervals={}",
                    source_linedef.record_index,
                    matching.len(),
                ))
            } else {
                CellSupport::Supported(format!(
                    "wall-cell-present:linedef={}:role={role:?}:column={column}:row={row}:source-segs={:?}:intervals={:?}",
                    source_linedef.record_index,
                    supported
                        .iter()
                        .map(|(interval, _)| interval.source_seg)
                        .collect::<BTreeSet<_>>(),
                    supported.iter().map(|(_, rows)| *rows).collect::<Vec<_>>(),
                ))
            }
        }
    }
}

fn ordered_target_detail(
    ordered: &OrderedPreparedSubmissionObservation,
    target: &StaticDrawPlanEntry,
    origin: Vec3,
    direction: Vec3,
) -> String {
    match target.source {
        StaticDrawSource::Wall { source_linedef, .. } => {
            let draws = ordered
                .walls
                .prepared_declarations
                .iter()
                .filter(|declaration| {
                    declaration.occurrence.source_linedef == source_linedef.record_index
                })
                .map(|declaration| declaration.draw.clone())
                .collect::<Vec<_>>();
            let ray_hit = nearest_prepared_ray_hit(origin, direction, &draws, None)
                .map(|hit| format!("{:.3}", hit.distance))
                .unwrap_or_else(|| "none".to_owned());
            format!(
                "wall-linedef={}:prepared-declarations={}:ray-hit={ray_hit}",
                source_linedef.record_index,
                draws.len(),
            )
        }
        StaticDrawSource::Flat {
            source_subsector,
            source_sector,
            plane,
        } => {
            let kind = match plane {
                DoomSurfacePlane::Floor => OrderedPlaneKind::Floor,
                DoomSurfacePlane::Ceiling => OrderedPlaneKind::Ceiling,
            };
            let instances = ordered
                .planes
                .plane_instances
                .iter()
                .filter(|instance| {
                    instance.source_sector == source_sector.record_index && instance.kind == kind
                })
                .count();
            let destinations = ordered
                .planes
                .plane_destinations
                .iter()
                .filter(|destination| {
                    destination.source_sector == source_sector.record_index
                        && destination.source_subsector == source_subsector.record_index
                        && destination.kind == kind
                })
                .count();
            let draws = ordered
                .plane_lowering
                .prepared_declarations
                .iter()
                .filter(|declaration| declaration.source_subsector == source_subsector.record_index)
                .map(|declaration| declaration.draw.clone())
                .collect::<Vec<_>>();
            let ray_hit = nearest_prepared_ray_hit(origin, direction, &draws, None)
                .map(|hit| format!("{:.3}", hit.distance))
                .unwrap_or_else(|| "none".to_owned());
            format!(
                "plane-sector={}:subsector={}:kind={kind:?}:instances={instances}:destinations={destinations}:prepared-declarations={}:ray-hit={ray_hit}",
                source_sector.record_index,
                source_subsector.record_index,
                draws.len(),
            )
        }
    }
}

const fn fnv_offset() -> u64 {
    0xcbf2_9ce4_8422_2325
}

fn hash_text(state: &mut u64, value: &str) {
    for byte in value.as_bytes() {
        *state ^= u64::from(*byte);
        *state = state.wrapping_mul(0x0000_0100_0000_01b3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walkabout_capture_names_and_rays_are_stable() {
        let names = WALKABOUT_CAPTURES
            .iter()
            .map(|capture| capture.name)
            .collect::<BTreeSet<_>>();
        assert_eq!(names.len(), WALKABOUT_CAPTURES.len());
        assert_eq!(
            WALKABOUT_CAPTURES
                .iter()
                .filter(|capture| {
                    capture.expected_cell_support == ExpectedCellSupport::Unsupported
                })
                .count(),
            4,
        );
        assert!(WALKABOUT_CAPTURES.iter().all(|capture| {
            capture.origin.into_iter().all(f64::is_finite)
                && capture.direction.into_iter().all(f64::is_finite)
                && capture
                    .direction
                    .iter()
                    .any(|component| component.abs() > 0.0)
        }));
    }

    #[test]
    fn center_look_rays_map_to_center_column() {
        for capture in WALKABOUT_CAPTURES {
            let heading = capture.direction[1].atan2(capture.direction[0]);
            let (column, _) = source_projection_sample(capture.direction, heading).unwrap();
            assert_eq!(column, SOURCE_COLUMNS / 2);
        }
    }
}
