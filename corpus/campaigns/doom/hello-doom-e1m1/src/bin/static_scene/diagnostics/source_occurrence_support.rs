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

#[derive(Clone, Copy, Debug)]
struct NeutralPitchPose {
    name: &'static str,
    viewer: [i16; 2],
    eye_height: i16,
    heading_degrees: f64,
}

const NEUTRAL_PITCH_POSES: [NeutralPitchPose; 4] = [
    NeutralPitchPose {
        name: "source-spawn",
        viewer: [1056, -3616],
        eye_height: 36,
        heading_degrees: 90.0,
    },
    NeutralPitchPose {
        name: "near-wall-a",
        viewer: [1202, -3502],
        eye_height: 36,
        heading_degrees: -24.0,
    },
    NeutralPitchPose {
        name: "near-wall-b",
        viewer: [1296, -3427],
        eye_height: 36,
        heading_degrees: -0.4,
    },
    NeutralPitchPose {
        name: "courtyard",
        viewer: [1514, -2481],
        eye_height: 36,
        heading_degrees: -29.2,
    },
];

#[derive(Clone, Copy, Debug)]
struct NeutralPositivePlaneControl {
    name: &'static str,
    pose: NeutralPitchPose,
    cell: [usize; 2],
    direction: [f64; 3],
    plane: DoomSurfacePlane,
    sector: u32,
    subsector: u32,
    label: &'static str,
}

const NEUTRAL_POSITIVE_PLANE_CONTROLS: [NeutralPositivePlaneControl; 3] = [
    NeutralPositivePlaneControl {
        name: "spawn-ceiling-97",
        pose: NEUTRAL_PITCH_POSES[0],
        cell: [155, 65],
        direction: [0.027_482_742, 0.977_164_152, 0.210_701_020],
        plane: DoomSurfacePlane::Ceiling,
        sector: 38,
        subsector: 97,
        label: "flat:38:CEIL3_5",
    },
    NeutralPositivePlaneControl {
        name: "spawn-floor-105",
        pose: NEUTRAL_PITCH_POSES[0],
        cell: [155, 125],
        direction: [0.027_763_764, 0.987_156_054, -0.157_327_996],
        plane: DoomSurfacePlane::Floor,
        sector: 39,
        subsector: 105,
        label: "flat:39:FLAT14",
    },
    NeutralPositivePlaneControl {
        name: "near-wall-floor-102",
        pose: NEUTRAL_PITCH_POSES[1],
        cell: [165, 145],
        direction: [0.891_667_121, -0.360_822_431, -0.273_380_537],
        plane: DoomSurfacePlane::Floor,
        sector: 38,
        subsector: 102,
        label: "flat:38:FLOOR4_8",
    },
];

#[derive(Clone, Copy, Debug)]
struct ModeratePitchHoleRay {
    name: &'static str,
    origin: [f64; 3],
    direction: [f64; 3],
}

const MODERATE_PITCH_HOLE_RAYS: [ModeratePitchHoleRay; 3] = [
    ModeratePitchHoleRay {
        name: "sector-38-subsector-114-floor",
        origin: [1011.078_369_141, -3023.246_826_172, 36.0],
        direction: [0.830_659_449, 0.459_405_005, -0.314_566_344],
    },
    ModeratePitchHoleRay {
        name: "sector-2-subsector-116-floor",
        origin: [1286.139_648_438, -2552.103_515_625, 36.0],
        direction: [-0.152_104_899, -0.900_991_142, -0.406_299_144],
    },
    ModeratePitchHoleRay {
        name: "sector-12-subsector-29-floor",
        origin: [1741.810_791_016, -2522.975_341_797, 36.0],
        direction: [0.860_694_408, -0.438_084_006, -0.259_398_639],
    },
];

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

/// Searches a bounded neutral-pitch source projection for exact positive plane
/// controls. Discovery requires independent agreement from the complete
/// reconstructed shell, final source cell, existing ordered declaration and
/// the source-cell geometry shadow. It changes no submitted presentation.
pub(crate) fn report_neutral_pitch_positive_planes(scene: &SceneInput) -> PlatformResult<()> {
    const GRID_COLUMNS: usize = 32;
    const GRID_ROWS: usize = 20;

    let cutout_materials = scene
        .cutout_uploads
        .iter()
        .map(|upload| (upload.source_name.clone(), upload.material))
        .collect::<BTreeMap<_, _>>();
    let wall_triangles = lower_doom_seg_textured_wall_triangles(
        &scene.door_geometry_source.map,
        &scene.door_geometry_source.wall_extents,
    )?;
    let (control_rows, control_fingerprint) =
        replay_neutral_positive_plane_controls(scene, &wall_triangles, &cutout_materials)?;
    let mut candidates = Vec::new();
    let mut representatives = BTreeMap::<String, (usize, String)>::new();
    let mut fingerprint = fnv_offset();

    for pose in NEUTRAL_PITCH_POSES {
        let heading = pose.heading_degrees.to_radians();
        let traversal = observe_doom_classic_bsp(
            &scene.door_geometry_source.map,
            pose.viewer,
            heading,
            &BTreeSet::new(),
        )?;
        let marks = observe_doom_seg_plane_marks(&scene.door_geometry_source.map, pose.eye_height)?;
        let vertical = observe_shared_doom_classic_vertical_clip_state(
            &scene.door_geometry_source.map,
            &wall_triangles,
            &marks,
            &traversal,
            pose.viewer,
            heading,
            f64::from(pose.eye_height),
        );
        let ordered = prepare_ordered_occurrence_submission(
            &scene.door_geometry_source.map,
            pose.viewer,
            heading,
            pose.eye_height,
            &scene.door_geometry_source.wall_extents,
            &scene.door_geometry_source.wall_materials,
            &cutout_materials,
            &scene.opaque_uploads,
        )
        .map_err(io::Error::other)?;
        ordered.verify_conservation().map_err(io::Error::other)?;
        let shadow = prepare_plane_cell_geometry_support_shadow(
            &scene.door_geometry_source.map,
            pose.viewer,
            heading,
            pose.eye_height,
            &scene.door_geometry_source.wall_extents,
            &scene.opaque_uploads,
        )?;
        shadow.verify_conservation().map_err(io::Error::other)?;

        for grid_row in 0..GRID_ROWS {
            let row = grid_row * SOURCE_ROWS / GRID_ROWS + SOURCE_ROWS / GRID_ROWS / 2;
            for grid_column in 0..GRID_COLUMNS {
                let column =
                    grid_column * SOURCE_COLUMNS / GRID_COLUMNS + SOURCE_COLUMNS / GRID_COLUMNS / 2;
                let source_direction = source_cell_center_direction(column, row, heading);
                let (origin, direction) = source_ray_vectors(
                    [
                        f64::from(pose.viewer[0]),
                        f64::from(pose.viewer[1]),
                        f64::from(pose.eye_height),
                    ],
                    source_direction,
                );
                let Some(global_hit) = nearest_prepared_ray_hit(
                    origin,
                    direction,
                    &scene.opaque_draws,
                    Some(&scene.cutout_draws),
                ) else {
                    continue;
                };
                if !matches!(global_hit.draw.source, StaticDrawSource::Flat { .. })
                    || !matches!(
                        source_cell_support(
                            &scene.door_geometry_source.map,
                            &vertical,
                            global_hit.draw,
                            column,
                            row,
                        ),
                        CellSupport::Supported(_)
                    )
                {
                    continue;
                }
                let ordered_draws = ordered
                    .plane_lowering
                    .prepared_declarations
                    .iter()
                    .filter(|declaration| same_flat_source(&declaration.draw, global_hit.draw))
                    .map(|declaration| declaration.draw.clone())
                    .collect::<Vec<_>>();
                let Some(ordered_hit) =
                    nearest_prepared_ray_hit(origin, direction, &ordered_draws, None)
                else {
                    continue;
                };
                let shadow_draws = shadow
                    .draws
                    .iter()
                    .filter(|draw| same_flat_source(draw, global_hit.draw))
                    .cloned()
                    .collect::<Vec<_>>();
                let Some(shadow_hit) =
                    nearest_prepared_ray_hit(origin, direction, &shadow_draws, None)
                else {
                    continue;
                };
                let StaticDrawSource::Flat {
                    source_subsector,
                    source_sector,
                    plane,
                } = global_hit.draw.source
                else {
                    unreachable!("flat source checked above")
                };
                let report = format!(
                    "pose={}:viewer=({},{},{}):heading-degrees={:.3}:cell=({column},{row}):direction=({:.9},{:.9},{:.9}):plane={plane:?}:sector={}:subsector={}:label={}:global-distance={:.3}:ordered-distance={:.3}:shadow-distance={:.3}",
                    pose.name,
                    pose.viewer[0],
                    pose.viewer[1],
                    pose.eye_height,
                    pose.heading_degrees,
                    source_direction[0],
                    source_direction[1],
                    source_direction[2],
                    source_sector.record_index,
                    source_subsector.record_index,
                    global_hit.draw.source_label,
                    global_hit.distance,
                    ordered_hit.distance,
                    shadow_hit.distance,
                );
                hash_text(&mut fingerprint, &report);
                let identity = format!(
                    "{}:{plane:?}:{}:{}",
                    pose.name, source_sector.record_index, source_subsector.record_index,
                );
                let center_distance =
                    column.abs_diff(SOURCE_COLUMNS / 2) + row.abs_diff(SOURCE_ROWS / 2);
                representatives
                    .entry(identity)
                    .and_modify(|(stored_distance, stored_report)| {
                        if center_distance < *stored_distance {
                            *stored_distance = center_distance;
                            *stored_report = report.clone();
                        }
                    })
                    .or_insert_with(|| (center_distance, report.clone()));
                candidates.push(report);
            }
        }
    }

    if candidates.is_empty() {
        return Err(io::Error::other(
            "neutral-pitch exact positive-plane search found no four-way agreements",
        )
        .into());
    }
    println!(
        "E1M1 neutral-pitch exact positive-plane search: poses={}; grid={}x{}; candidates={}; representative-identities={}; discovery-fingerprint={fingerprint:016x}; frozen-controls={}; control-fingerprint={control_fingerprint:016x}; pitch-lift-degrees=[-15,0,15]; agreement=complete-shell-plus-final-source-cell-plus-ordered-declaration-plus-cell-geometry-shadow; renderer-mutation=false; controls=[{}]; representatives=[{}]",
        NEUTRAL_PITCH_POSES.len(),
        GRID_COLUMNS,
        GRID_ROWS,
        candidates.len(),
        representatives.len(),
        control_rows.len(),
        control_rows.join(" | "),
        representatives
            .values()
            .map(|(_, report)| report.as_str())
            .take(32)
            .collect::<Vec<_>>()
            .join(" | "),
    );
    Ok(())
}

/// Replays the complete exact-ray acceptance matrix against the opt-in live
/// candidate without opening a renderer window.
pub(crate) fn report_source_occurrence_live_candidate(scene: &SceneInput) -> PlatformResult<()> {
    let mut rows = Vec::new();
    let mut passed = 0usize;
    let mut fingerprint = fnv_offset();

    for capture in WALKABOUT_CAPTURES {
        let heading = capture.direction[1].atan2(capture.direction[0]);
        let prepared = crate::render_strategies::source_occurrence_supported::prepare(
            scene,
            &scene.door_geometry_source.map,
            [
                capture.origin[0].round() as i16,
                capture.origin[1].round() as i16,
            ],
            heading,
            capture.origin[2].round() as i16,
        )?;
        let (origin, direction) = source_ray_vectors(capture.origin, capture.direction);
        let draws = prepared
            .opaque_draws
            .iter()
            .chain(&prepared.cutout_draws)
            .filter(|draw| capture_target_matches(draw, capture.target))
            .cloned()
            .collect::<Vec<_>>();
        let hit = nearest_prepared_ray_hit(origin, direction, &draws, None);
        let expected_hit = capture.expected_cell_support == ExpectedCellSupport::Supported;
        let result = hit.is_some() == expected_hit;
        passed += usize::from(result);
        let row = format!(
            "case={}:expected={}:candidate={}:distance={}:result={}",
            capture.name,
            if expected_hit { "hit" } else { "none" },
            if hit.is_some() { "hit" } else { "none" },
            hit.map(|hit| format!("{:.3}", hit.distance))
                .unwrap_or_else(|| "none".to_owned()),
            if result { "pass" } else { "fail" },
        );
        hash_text(&mut fingerprint, &row);
        rows.push(row);
    }

    for case in ordered_six_ray_cases() {
        let heading = case.direction[1].atan2(case.direction[0]);
        let prepared = crate::render_strategies::source_occurrence_supported::prepare(
            scene,
            &scene.door_geometry_source.map,
            [case.origin[0].round() as i16, case.origin[1].round() as i16],
            heading,
            case.origin[2].round() as i16,
        )?;
        let (origin, direction) = source_ray_vectors(case.origin, case.direction);
        let draws = prepared
            .opaque_draws
            .iter()
            .chain(&prepared.cutout_draws)
            .filter(|draw| six_ray_target_matches(draw, case.expected))
            .cloned()
            .collect::<Vec<_>>();
        let hit = nearest_prepared_ray_hit(origin, direction, &draws, None);
        // The historical reached ceiling remains a positive object-occurrence
        // control, but the exact ray is now correctly expected to be absent.
        let result = hit.is_none();
        passed += usize::from(result);
        let row = format!(
            "case={}:expected=none:candidate={}:distance={}:result={}",
            case.name,
            if hit.is_some() { "hit" } else { "none" },
            hit.map(|hit| format!("{:.3}", hit.distance))
                .unwrap_or_else(|| "none".to_owned()),
            if result { "pass" } else { "fail" },
        );
        hash_text(&mut fingerprint, &row);
        rows.push(row);
    }

    let (wall_origin, wall_direction, wall_linedef, _) = positive_wall_support_control();
    let wall_heading = wall_direction[1].atan2(wall_direction[0]);
    let wall_prepared = crate::render_strategies::source_occurrence_supported::prepare(
        scene,
        &scene.door_geometry_source.map,
        [wall_origin[0].round() as i16, wall_origin[1].round() as i16],
        wall_heading,
        wall_origin[2].round() as i16,
    )?;
    let (origin, direction) = source_ray_vectors(wall_origin, wall_direction);
    let wall_draws = wall_prepared
        .opaque_draws
        .iter()
        .chain(&wall_prepared.cutout_draws)
        .filter(|draw| {
            matches!(
                draw.source,
                StaticDrawSource::Wall { source_linedef, .. }
                    if source_linedef.record_index == wall_linedef
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    let wall_hit = nearest_prepared_ray_hit(origin, direction, &wall_draws, None);
    let wall_result = wall_hit.is_some();
    passed += usize::from(wall_result);
    let row = format!(
        "case=positive-wall-135:expected=hit:candidate={}:distance={}:result={}",
        if wall_hit.is_some() { "hit" } else { "none" },
        wall_hit
            .map(|hit| format!("{:.3}", hit.distance))
            .unwrap_or_else(|| "none".to_owned()),
        if wall_result { "pass" } else { "fail" },
    );
    hash_text(&mut fingerprint, &row);
    rows.push(row);

    for control in NEUTRAL_POSITIVE_PLANE_CONTROLS {
        let heading = control.pose.heading_degrees.to_radians();
        let prepared = crate::render_strategies::source_occurrence_supported::prepare(
            scene,
            &scene.door_geometry_source.map,
            control.pose.viewer,
            heading,
            control.pose.eye_height,
        )?;
        let (origin, direction) = source_ray_vectors(
            [
                f64::from(control.pose.viewer[0]),
                f64::from(control.pose.viewer[1]),
                f64::from(control.pose.eye_height),
            ],
            control.direction,
        );
        let draws = prepared
            .opaque_draws
            .iter()
            .filter(|draw| flat_source_matches_control(draw, control))
            .cloned()
            .collect::<Vec<_>>();
        let hit = nearest_prepared_ray_hit(origin, direction, &draws, None);
        let result = hit.is_some();
        passed += usize::from(result);
        let row = format!(
            "case={}:expected=hit:candidate={}:distance={}:result={}",
            control.name,
            if hit.is_some() { "hit" } else { "none" },
            hit.map(|hit| format!("{:.3}", hit.distance))
                .unwrap_or_else(|| "none".to_owned()),
            if result { "pass" } else { "fail" },
        );
        hash_text(&mut fingerprint, &row);
        rows.push(row);
    }

    let total = WALKABOUT_CAPTURES.len()
        + ordered_six_ray_cases().len()
        + 1
        + NEUTRAL_POSITIVE_PLANE_CONTROLS.len();
    if passed != total {
        return Err(io::Error::other(format!(
            "source-occurrence live candidate acceptance failed: {passed}/{total}; rows=[{}]",
            rows.join(" | "),
        ))
        .into());
    }
    let (coverage_rows, coverage_fingerprint) = source_occurrence_candidate_grid_delta(scene)?;
    let moderate_hole_rows = diagnose_moderate_pitch_holes(scene)?;
    println!(
        "E1M1 source-occurrence live candidate acceptance: passed={passed}/{total}; fingerprint={fingerprint:016x}; historical-ceiling-104=object-positive-exact-ray-negative; broad-grid-authority=diagnostic-delta-not-correctness-proof; broad-grid-fingerprint={coverage_fingerprint:016x}; renderer-mutation=false; rows=[{}]; broad-grid=[{}]; moderate-pitch-holes=[{}]",
        rows.join(" | "),
        coverage_rows.join(" | "),
        moderate_hole_rows.join(" | "),
    );
    Ok(())
}

/// Performs the final bounded audit requested after the live strategy was
/// falsified: for each moderate-pitch hole, distinguish absence of its exact
/// final source cell from failure to realize support that actually exists.
fn diagnose_moderate_pitch_holes(scene: &SceneInput) -> PlatformResult<Vec<String>> {
    let wall_triangles = lower_doom_seg_textured_wall_triangles(
        &scene.door_geometry_source.map,
        &scene.door_geometry_source.wall_extents,
    )?;
    let cutout_materials = scene
        .cutout_uploads
        .iter()
        .map(|upload| (upload.source_name.clone(), upload.material))
        .collect::<BTreeMap<_, _>>();
    let mut rows = Vec::new();

    for case in MODERATE_PITCH_HOLE_RAYS {
        let heading = case.direction[1].atan2(case.direction[0]);
        let (column, row) = source_projection_sample(case.direction, heading)
            .map_err(|error| io::Error::other(format!("{}:{error}", case.name)))?;
        let (origin, direction) = source_ray_vectors(case.origin, case.direction);
        let global_hit = nearest_prepared_ray_hit(
            origin,
            direction,
            &scene.opaque_draws,
            Some(&scene.cutout_draws),
        )
        .ok_or_else(|| io::Error::other(format!("{} lost global-full hit", case.name)))?;
        if !matches!(global_hit.draw.source, StaticDrawSource::Flat { .. }) {
            return Err(io::Error::other(format!(
                "{} expected a global-full plane, got {}",
                case.name, global_hit.draw.source_label
            ))
            .into());
        }

        let viewer = [case.origin[0].round() as i16, case.origin[1].round() as i16];
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
            &wall_triangles,
            &marks,
            &traversal,
            viewer,
            heading,
            f64::from(eye_height),
        );
        let exact_cell = source_cell_support(
            &scene.door_geometry_source.map,
            &vertical,
            global_hit.draw,
            column,
            row,
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
        let matching_ordered_declarations = ordered
            .plane_lowering
            .prepared_declarations
            .iter()
            .filter(|declaration| same_flat_source(&declaration.draw, global_hit.draw))
            .map(|declaration| declaration.draw.clone())
            .collect::<Vec<_>>();
        let ordered_target_hit =
            nearest_prepared_ray_hit(origin, direction, &matching_ordered_declarations, None);
        let prepared = crate::render_strategies::source_occurrence_supported::prepare(
            scene,
            &scene.door_geometry_source.map,
            viewer,
            heading,
            eye_height,
        )?;
        let matching_candidate_draws = prepared
            .opaque_draws
            .iter()
            .filter(|draw| same_flat_source(draw, global_hit.draw))
            .cloned()
            .collect::<Vec<_>>();
        let candidate_target_hit =
            nearest_prepared_ray_hit(origin, direction, &matching_candidate_draws, None);
        let horizontal = case.direction[0].hypot(case.direction[1]);
        let elevation = case.direction[2].atan2(horizontal).to_degrees();

        rows.push(format!(
            "case={}:elevation-degrees={elevation:.3}:cell=({column},{row}):global={}:distance={:.3}:exact-cell={}:detail={}:matching-ordered-declarations={}:ordered-target-hit={}:matching-candidate-draws={}:candidate-target-hit={}",
            case.name,
            global_hit.draw.source_label,
            global_hit.distance,
            exact_cell.label(),
            exact_cell.detail(),
            matching_ordered_declarations.len(),
            if ordered_target_hit.is_some() { "hit" } else { "none" },
            matching_candidate_draws.len(),
            if candidate_target_hit.is_some() { "hit" } else { "none" },
        ));
    }
    Ok(rows)
}

/// Measures how much complete-shell nearest-hit coverage the candidate removes
/// over a broad neutral-view grid. A delta is not automatically a defect: some
/// removed complete-shell hits are exactly the unsupported geometry under
/// study. The report exists to make a visually hole-filled walkabout
/// quantitatively reproducible instead of expanding admission to make it pass.
fn source_occurrence_candidate_grid_delta(
    scene: &SceneInput,
) -> PlatformResult<(Vec<String>, u64)> {
    const GRID_COLUMNS: usize = 32;
    const GRID_ROWS: usize = 20;

    let mut rows = Vec::new();
    let mut fingerprint = fnv_offset();
    for pose in NEUTRAL_PITCH_POSES {
        let heading = pose.heading_degrees.to_radians();
        let prepared = crate::render_strategies::source_occurrence_supported::prepare(
            scene,
            &scene.door_geometry_source.map,
            pose.viewer,
            heading,
            pose.eye_height,
        )?;
        let mut complete_hits = 0usize;
        let mut candidate_hits = 0usize;
        let mut complete_hit_candidate_misses = 0usize;
        let mut nearest_hit_disagreements = 0usize;
        let mut missing_sources = BTreeMap::<String, (usize, usize, usize)>::new();
        let mut displaced_sources = BTreeMap::<String, (usize, usize, usize)>::new();

        for grid_row in 0..GRID_ROWS {
            let row = grid_row * SOURCE_ROWS / GRID_ROWS + SOURCE_ROWS / GRID_ROWS / 2;
            for grid_column in 0..GRID_COLUMNS {
                let column =
                    grid_column * SOURCE_COLUMNS / GRID_COLUMNS + SOURCE_COLUMNS / GRID_COLUMNS / 2;
                let source_direction = source_cell_center_direction(column, row, heading);
                let (origin, direction) = source_ray_vectors(
                    [
                        f64::from(pose.viewer[0]),
                        f64::from(pose.viewer[1]),
                        f64::from(pose.eye_height),
                    ],
                    source_direction,
                );
                let complete_hit = nearest_prepared_ray_hit(
                    origin,
                    direction,
                    &scene.opaque_draws,
                    Some(&scene.cutout_draws),
                );
                let candidate_hit = nearest_prepared_ray_hit(
                    origin,
                    direction,
                    &prepared.opaque_draws,
                    Some(&prepared.cutout_draws),
                );
                complete_hits += usize::from(complete_hit.is_some());
                candidate_hits += usize::from(candidate_hit.is_some());
                match (complete_hit, candidate_hit) {
                    (Some(complete_hit), None) => {
                        complete_hit_candidate_misses += 1;
                        missing_sources
                            .entry(complete_hit.draw.source_label.clone())
                            .and_modify(|entry| entry.0 += 1)
                            .or_insert((1, column, row));
                    }
                    (Some(complete_hit), Some(candidate_hit))
                        if complete_hit.draw.source != candidate_hit.draw.source
                            || (complete_hit.distance - candidate_hit.distance).abs() > 0.05 =>
                    {
                        nearest_hit_disagreements += 1;
                        let key = format!(
                            "{}->{}",
                            complete_hit.draw.source_label, candidate_hit.draw.source_label
                        );
                        displaced_sources
                            .entry(key)
                            .and_modify(|entry| entry.0 += 1)
                            .or_insert((1, column, row));
                    }
                    _ => {}
                }
            }
        }

        let missing = missing_sources
            .iter()
            .map(|(source, (samples, column, row))| format!("{source}:{samples}@({column},{row})"))
            .collect::<Vec<_>>()
            .join(",");
        let displaced = displaced_sources
            .iter()
            .map(|(sources, (samples, column, row))| {
                format!("{sources}:{samples}@({column},{row})")
            })
            .collect::<Vec<_>>()
            .join(",");
        let report = format!(
            "pose={}:samples={}:complete-hits={complete_hits}:candidate-hits={candidate_hits}:complete-hit-candidate-misses={complete_hit_candidate_misses}:nearest-hit-disagreements={nearest_hit_disagreements}:missing-sources=[{missing}]:displaced-sources=[{displaced}]",
            pose.name,
            GRID_COLUMNS * GRID_ROWS,
        );
        hash_text(&mut fingerprint, &report);
        rows.push(report);
    }
    Ok((rows, fingerprint))
}

fn replay_neutral_positive_plane_controls(
    scene: &SceneInput,
    wall_triangles: &[DoomSegTexturedWallTriangle],
    cutout_materials: &BTreeMap<String, MaterialHandle>,
) -> PlatformResult<(Vec<String>, u64)> {
    let mut rows = Vec::new();
    let mut fingerprint = fnv_offset();
    for control in NEUTRAL_POSITIVE_PLANE_CONTROLS {
        let heading = control.pose.heading_degrees.to_radians();
        let projected_cell = source_projection_sample(control.direction, heading)
            .map_err(|error| io::Error::other(format!("{}:{error}", control.name)))?;
        if projected_cell != (control.cell[0], control.cell[1]) {
            return Err(io::Error::other(format!(
                "neutral positive control {} moved from cell {:?} to {projected_cell:?}",
                control.name, control.cell,
            ))
            .into());
        }
        let (origin, direction) = source_ray_vectors(
            [
                f64::from(control.pose.viewer[0]),
                f64::from(control.pose.viewer[1]),
                f64::from(control.pose.eye_height),
            ],
            control.direction,
        );
        let global_hit = nearest_prepared_ray_hit(
            origin,
            direction,
            &scene.opaque_draws,
            Some(&scene.cutout_draws),
        )
        .ok_or_else(|| io::Error::other(format!("{} lost complete-shell hit", control.name)))?;
        if global_hit.draw.source_label != control.label
            || !flat_source_matches_control(global_hit.draw, control)
        {
            return Err(io::Error::other(format!(
                "neutral positive control {} expected {} sector {}/subsector {:?}, got {} {:?}",
                control.name,
                control.label,
                control.sector,
                control.subsector,
                global_hit.draw.source_label,
                global_hit.draw.source,
            ))
            .into());
        }

        let traversal = observe_doom_classic_bsp(
            &scene.door_geometry_source.map,
            control.pose.viewer,
            heading,
            &BTreeSet::new(),
        )?;
        let marks =
            observe_doom_seg_plane_marks(&scene.door_geometry_source.map, control.pose.eye_height)?;
        let vertical = observe_shared_doom_classic_vertical_clip_state(
            &scene.door_geometry_source.map,
            wall_triangles,
            &marks,
            &traversal,
            control.pose.viewer,
            heading,
            f64::from(control.pose.eye_height),
        );
        let support = source_cell_support(
            &scene.door_geometry_source.map,
            &vertical,
            global_hit.draw,
            control.cell[0],
            control.cell[1],
        );
        if !matches!(support, CellSupport::Supported(_)) {
            return Err(io::Error::other(format!(
                "neutral positive control {} lost final source cell: {}",
                control.name,
                support.detail(),
            ))
            .into());
        }

        let ordered = prepare_ordered_occurrence_submission(
            &scene.door_geometry_source.map,
            control.pose.viewer,
            heading,
            control.pose.eye_height,
            &scene.door_geometry_source.wall_extents,
            &scene.door_geometry_source.wall_materials,
            cutout_materials,
            &scene.opaque_uploads,
        )
        .map_err(io::Error::other)?;
        ordered.verify_conservation().map_err(io::Error::other)?;
        let ordered_draws = ordered
            .plane_lowering
            .prepared_declarations
            .iter()
            .filter(|declaration| flat_source_matches_control(&declaration.draw, control))
            .map(|declaration| declaration.draw.clone())
            .collect::<Vec<_>>();
        let ordered_hit = nearest_prepared_ray_hit(origin, direction, &ordered_draws, None)
            .ok_or_else(|| {
                io::Error::other(format!("{} lost ordered declaration ray", control.name))
            })?;

        let shadow = prepare_plane_cell_geometry_support_shadow(
            &scene.door_geometry_source.map,
            control.pose.viewer,
            heading,
            control.pose.eye_height,
            &scene.door_geometry_source.wall_extents,
            &scene.opaque_uploads,
        )?;
        shadow.verify_conservation().map_err(io::Error::other)?;
        let shadow_draws = shadow
            .draws
            .iter()
            .filter(|draw| flat_source_matches_control(draw, control))
            .cloned()
            .collect::<Vec<_>>();
        let shadow_hit = nearest_prepared_ray_hit(origin, direction, &shadow_draws, None)
            .ok_or_else(|| {
                io::Error::other(format!("{} lost geometry-shadow ray", control.name))
            })?;
        if (global_hit.distance - ordered_hit.distance).abs() > 0.01
            || (global_hit.distance - shadow_hit.distance).abs() > 0.01
        {
            return Err(io::Error::other(format!(
                "neutral positive control {} distance disagreement: global={:.6} ordered={:.6} shadow={:.6}",
                control.name, global_hit.distance, ordered_hit.distance, shadow_hit.distance,
            ))
            .into());
        }

        let mut pitch_rows = Vec::new();
        for pitch_degrees in [-15.0_f64, 0.0, 15.0] {
            let (ndc, lifted_direction) = project_and_reconstruct_pitched_source_ray(
                control.direction,
                heading,
                pitch_degrees.to_radians(),
            )
            .ok_or_else(|| {
                io::Error::other(format!(
                    "{} is outside the {pitch_degrees}-degree pitched view",
                    control.name,
                ))
            })?;
            let (_, lifted_world_direction) = source_ray_vectors(
                [
                    f64::from(control.pose.viewer[0]),
                    f64::from(control.pose.viewer[1]),
                    f64::from(control.pose.eye_height),
                ],
                lifted_direction,
            );
            let lifted_hit =
                nearest_prepared_ray_hit(origin, lifted_world_direction, &shadow_draws, None)
                    .ok_or_else(|| {
                        io::Error::other(format!(
                            "{} lost neutral-authorized world support at pitch {pitch_degrees}",
                            control.name,
                        ))
                    })?;
            if (lifted_hit.distance - shadow_hit.distance).abs() > 0.01 {
                return Err(io::Error::other(format!(
                    "{} pitch {pitch_degrees} changed world hit distance {:.6} to {:.6}",
                    control.name, shadow_hit.distance, lifted_hit.distance,
                ))
                .into());
            }
            pitch_rows.push(format!(
                "pitch={pitch_degrees:.0}:ndc=({:.3},{:.3}):distance={:.3}",
                ndc[0], ndc[1], lifted_hit.distance,
            ));
        }
        let row = format!(
            "control={}:pose={}:cell=({},{}):plane={:?}:sector={}:subsector={}:label={}:distance={:.3}:cell=supported:ordered=hit:shadow=hit:pitch-lift=[{}]",
            control.name,
            control.pose.name,
            control.cell[0],
            control.cell[1],
            control.plane,
            control.sector,
            control.subsector,
            control.label,
            global_hit.distance,
            pitch_rows.join(","),
        );
        hash_text(&mut fingerprint, &row);
        rows.push(row);
    }
    Ok((rows, fingerprint))
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

fn source_cell_center_direction(column: usize, row: usize, heading: f64) -> [f64; 3] {
    let horizontal_normalized = ((column as f64 + 0.5) / SOURCE_COLUMNS as f64) * 2.0 - 1.0;
    let vertical_normalized = 1.0 - ((row as f64 + 0.5) / SOURCE_ROWS as f64) * 2.0;
    let half_vertical_fov =
        (std::f64::consts::FRAC_PI_4.tan() / (SOURCE_COLUMNS as f64 / SOURCE_ROWS as f64)).atan();
    let forward = [heading.cos(), heading.sin()];
    let right = [-forward[1], forward[0]];
    let lateral = horizontal_normalized * std::f64::consts::FRAC_PI_4.tan();
    let vertical = vertical_normalized * half_vertical_fov.tan();
    let direction = [
        forward[0] + right[0] * lateral,
        forward[1] + right[1] * lateral,
        vertical,
    ];
    let length = direction[0].hypot(direction[1]).hypot(direction[2]);
    [
        direction[0] / length,
        direction[1] / length,
        direction[2] / length,
    ]
}

fn same_flat_source(candidate: &StaticDrawPlanEntry, target: &StaticDrawPlanEntry) -> bool {
    matches!(
        (candidate.source, target.source),
        (
            StaticDrawSource::Flat {
                source_subsector: candidate_subsector,
                source_sector: candidate_sector,
                plane: candidate_plane,
            },
            StaticDrawSource::Flat {
                source_subsector: target_subsector,
                source_sector: target_sector,
                plane: target_plane,
            },
        ) if candidate_subsector == target_subsector
            && candidate_sector == target_sector
            && candidate_plane == target_plane
    )
}

fn flat_source_matches_control(
    draw: &StaticDrawPlanEntry,
    control: NeutralPositivePlaneControl,
) -> bool {
    matches!(
        draw.source,
        StaticDrawSource::Flat {
            source_subsector,
            source_sector,
            plane,
        } if source_subsector.record_index == control.subsector
            && source_sector.record_index == control.sector
            && plane == control.plane
    )
}

fn project_and_reconstruct_pitched_source_ray(
    direction: [f64; 3],
    heading: f64,
    pitch: f64,
) -> Option<([f64; 2], [f64; 3])> {
    let neutral_forward = [heading.cos(), heading.sin(), 0.0];
    let right = [-heading.sin(), heading.cos(), 0.0];
    let pitched_forward = [
        neutral_forward[0] * pitch.cos(),
        neutral_forward[1] * pitch.cos(),
        pitch.sin(),
    ];
    let pitched_up = [
        -neutral_forward[0] * pitch.sin(),
        -neutral_forward[1] * pitch.sin(),
        pitch.cos(),
    ];
    let dot = |left: [f64; 3], right: [f64; 3]| {
        left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
    };
    let depth = dot(direction, pitched_forward);
    if depth <= f64::EPSILON {
        return None;
    }
    let half_vertical_fov =
        (std::f64::consts::FRAC_PI_4.tan() / (SOURCE_COLUMNS as f64 / SOURCE_ROWS as f64)).atan();
    let ndc = [
        dot(direction, right) / depth / std::f64::consts::FRAC_PI_4.tan(),
        dot(direction, pitched_up) / depth / half_vertical_fov.tan(),
    ];
    if !ndc
        .into_iter()
        .all(|value| value.is_finite() && (-1.0..=1.0).contains(&value))
    {
        return None;
    }
    let reconstructed = [
        pitched_forward[0]
            + right[0] * ndc[0] * std::f64::consts::FRAC_PI_4.tan()
            + pitched_up[0] * ndc[1] * half_vertical_fov.tan(),
        pitched_forward[1]
            + right[1] * ndc[0] * std::f64::consts::FRAC_PI_4.tan()
            + pitched_up[1] * ndc[1] * half_vertical_fov.tan(),
        pitched_forward[2]
            + right[2] * ndc[0] * std::f64::consts::FRAC_PI_4.tan()
            + pitched_up[2] * ndc[1] * half_vertical_fov.tan(),
    ];
    let length = reconstructed[0]
        .hypot(reconstructed[1])
        .hypot(reconstructed[2]);
    Some((
        ndc,
        [
            reconstructed[0] / length,
            reconstructed[1] / length,
            reconstructed[2] / length,
        ],
    ))
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

    #[test]
    fn neutral_positive_plane_controls_are_diverse_and_cell_stable() {
        let names = NEUTRAL_POSITIVE_PLANE_CONTROLS
            .iter()
            .map(|control| control.name)
            .collect::<BTreeSet<_>>();
        let poses = NEUTRAL_POSITIVE_PLANE_CONTROLS
            .iter()
            .map(|control| control.pose.name)
            .collect::<BTreeSet<_>>();
        assert_eq!(names.len(), NEUTRAL_POSITIVE_PLANE_CONTROLS.len());
        assert!(poses.len() >= 2);
        assert!(NEUTRAL_POSITIVE_PLANE_CONTROLS
            .iter()
            .any(|control| control.plane == DoomSurfacePlane::Floor));
        assert!(NEUTRAL_POSITIVE_PLANE_CONTROLS
            .iter()
            .any(|control| control.plane == DoomSurfacePlane::Ceiling));
        for control in NEUTRAL_POSITIVE_PLANE_CONTROLS {
            let heading = control.pose.heading_degrees.to_radians();
            assert_eq!(
                source_projection_sample(control.direction, heading).unwrap(),
                (control.cell[0], control.cell[1]),
            );
        }
    }

    #[test]
    fn pitched_projection_reconstructs_the_same_authorized_world_ray() {
        for control in NEUTRAL_POSITIVE_PLANE_CONTROLS {
            let heading = control.pose.heading_degrees.to_radians();
            for pitch_degrees in [-15.0_f64, 0.0, 15.0] {
                let (_, reconstructed) = project_and_reconstruct_pitched_source_ray(
                    control.direction,
                    heading,
                    pitch_degrees.to_radians(),
                )
                .unwrap();
                let dot = control.direction[0] * reconstructed[0]
                    + control.direction[1] * reconstructed[1]
                    + control.direction[2] * reconstructed[2];
                assert!(dot > 0.999_999_999);
            }
        }
    }
}
