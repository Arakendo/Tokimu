//! Source-record and non-mutating runtime evidence reports.
//!
//! These reports inspect retained Doom meaning without owning application
//! lifecycle, renderer state, or source mutation policy.

use super::super::*;

/// Narrow source-to-lowered inspection for a single wall selected from a
/// native `LOOK` observation. It keeps Doom source meaning at the corpus edge
/// and is diagnostic only: no geometry classification changes here.
pub(crate) fn report_wall_source(scene: &SceneInput, record_index: u32) {
    let map = &scene.door_geometry_source.map;
    let Some(linedef) = map
        .linedefs
        .iter()
        .find(|linedef| linedef.source.record_index == record_index)
    else {
        println!("E1M1 wall source report: linedef={record_index} missing");
        return;
    };
    let side = |index: Option<u16>| -> String {
        let Some(index) = index else {
            return "none".to_owned();
        };
        let Some(sidedef) = map.sidedefs.get(usize::from(index)) else {
            return format!("sidedef={index}:missing");
        };
        let Some(sector) = map.sectors.get(usize::from(sidedef.sector)) else {
            return format!("sidedef={index}:sector={}:missing", sidedef.sector);
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
        "E1M1 wall source report: linedef={} flags=0x{:04x} special={} tag={} right/front=[{}] left/back=[{}]",
        linedef.source.record_index,
        linedef.flags,
        linedef.special,
        linedef.tag,
        side(linedef.right_sidedef),
        side(linedef.left_sidedef),
    );
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
        if source_linedef.record_index != record_index {
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
    println!("E1M1 wall source report summary: generated_wall_draws={generated}");
}

pub(crate) fn report_doom_reject(report: &DoomRejectReport) {
    println!(
        "E1M1 AR-0025 Doom REJECT source observation: sectors={}; bytes={}; player_sector={}; monster_sectors_forbidden_to_sight_player={}; monster_sectors_not_forbidden={}; meaning=classic-monster-sight-prefilter-not-render-visibility",
        report.sector_count,
        report.byte_len,
        report.player_sector,
        report.forbidden_monster_sectors,
        report.sector_count - report.forbidden_monster_sectors,
    );
}

pub(crate) fn report_doom_topology(report: &DoomTopologyReport) {
    println!(
        "E1M1 AR-0025 Doom SEGS-to-SSECTORS source observation: linedefs={}; no_subsector_membership={}; one_subsector_membership={}; multiple_subsector_membership={}; maximum_subsector_membership={}; meaning=source-topology-not-render-membership",
        report.linedefs,
        report.no_subsector_membership,
        report.one_subsector_membership,
        report.multiple_subsector_membership,
        report.maximum_subsector_membership,
    );
}

/// Reports the deterministic `Use` resolution of every nonzero E1M1 line.
/// This is source/request evidence only: accepted door intent is deliberately
/// not treated as a moved sector, and crossing-only specials remain visible.
pub(crate) fn report_doom_use_activation(source: &DoomLineActivationSource) {
    let mut accepted = 0;
    let mut no_special = 0;
    let mut wrong_activation = 0;
    let mut unsupported = 0;
    let mut invalid_target = 0;
    let mut details = Vec::new();
    for linedef in source
        .linedefs
        .iter()
        .filter(|linedef| linedef.special != 0)
    {
        let resolution = resolve_doom_line_activation(
            source,
            DoomLineActivationRequest {
                source_linedef: linedef.source,
                activation: DoomLineActivation::Use,
            },
        );
        match resolution {
            DoomLineActivationResolution::Accepted { intent, .. } => {
                accepted += 1;
                details.push(format!(
                    "{}:special{}:tag{}:accepted:{}:target:{}",
                    linedef.source.record_index,
                    linedef.special,
                    linedef.tag,
                    compact_activation_intent(intent),
                    compact_activation_target(intent),
                ));
            }
            DoomLineActivationResolution::NoSpecial { .. } => no_special += 1,
            DoomLineActivationResolution::WrongActivation { required, .. } => {
                wrong_activation += 1;
                details.push(format!(
                    "{}:special{}:tag{}:requires:{required:?}",
                    linedef.source.record_index, linedef.special, linedef.tag
                ));
            }
            DoomLineActivationResolution::UnsupportedSpecial { .. } => {
                unsupported += 1;
                details.push(format!(
                    "{}:special{}:tag{}:unsupported",
                    linedef.source.record_index, linedef.special, linedef.tag
                ));
            }
            DoomLineActivationResolution::UnknownLinedef { .. } => unreachable!(
                "a request derived from the retained E1M1 lines must resolve to one of them"
            ),
            DoomLineActivationResolution::MissingManualDoorTarget {
                missing_left_sidedef,
                ..
            } => {
                invalid_target += 1;
                details.push(format!(
                    "{}:special{}:missing-opposite-sidedef:{missing_left_sidedef:?}",
                    linedef.source.record_index, linedef.special
                ));
            }
            DoomLineActivationResolution::InvalidManualDoorTarget {
                sidedef_index,
                sector_index,
                ..
            } => {
                invalid_target += 1;
                details.push(format!(
                    "{}:special{}:invalid-target:sidedef{}:sector{}",
                    linedef.source.record_index, linedef.special, sidedef_index, sector_index
                ));
            }
        }
    }
    println!(
        "E1M1 Slice 8 use-request observation: nonzero_linedefs={}; accepted={accepted}; no_special={no_special}; wrong_activation={wrong_activation}; unsupported={unsupported}; invalid_target={invalid_target}; accepted_effects_are_not_executed=true; details={}",
        details.len(),
        details.join(" | "),
    );
}

/// Runs each E1M1 manual-door intent through the corpus-local, deterministic
/// moving-sector state machine. It reports height transitions only: no mesh,
/// collision, input reach, or renderer state is changed by this evidence path.
pub(crate) fn report_doom_manual_door_runtime(source: &DoomLineActivationSource) {
    let mut started = 0;
    let mut rejected = 0;
    let mut details = Vec::new();
    for linedef in source.linedefs.iter().filter(|line| line.special == 1) {
        let DoomLineActivationResolution::Accepted {
            intent: DoomLineActivationIntent::RaiseDoor { target_sector },
            ..
        } = resolve_doom_line_activation(
            source,
            DoomLineActivationRequest {
                source_linedef: linedef.source,
                activation: DoomLineActivation::Use,
            },
        )
        else {
            unreachable!("classified E1M1 code-1 lines must resolve to manual-door intent");
        };
        let mut door = match DoomManualDoorRuntime::start(
            source,
            target_sector,
            DoomManualDoorPolicy::CLASSIC_NORMAL,
        ) {
            Ok(door) => door,
            Err(error) => {
                rejected += 1;
                details.push(format!(
                    "line{}:target-sector{}:start-rejected:{error:?}",
                    linedef.source.record_index, target_sector.record_index
                ));
                continue;
            }
        };
        started += 1;
        let mut ticks = 0_u32;
        let mut reached_waiting = false;
        while door.phase != DoomManualDoorPhase::Closed && ticks < 4_096 {
            let transition = door.advance_tick();
            reached_waiting |=
                matches!(transition.after_phase, DoomManualDoorPhase::Waiting { .. });
            ticks += 1;
        }
        details.push(format!(
            "line{}:target-sector{}:closed-height{}:open-height{}:ticks{}:waited{}:final={:?}",
            linedef.source.record_index,
            target_sector.record_index,
            door.closed_ceiling_height,
            door.open_ceiling_height,
            ticks,
            reached_waiting,
            door.phase,
        ));
    }
    println!(
        "E1M1 Slice 8 manual-door runtime observation: code1_lines={}; started={started}; start_rejected={rejected}; source_map_mutated=false; presentation_mutated=false; details={}",
        details.len(),
        details.join(" | "),
    );
}

/// Runs the two tagged E1M1 moving-floor effects through their distinct
/// released-source state machines. This path changes no imported sector,
/// collision world, mesh, or renderer resource.
pub(crate) fn report_doom_moving_floor_runtime(source: &DoomLineActivationSource) {
    let mut details = Vec::new();
    let mut started = 0_usize;
    let mut rejected = 0_usize;
    for linedef in source
        .linedefs
        .iter()
        .filter(|line| matches!(line.special, 36 | 88))
    {
        let resolution = resolve_doom_line_activation(
            source,
            DoomLineActivationRequest {
                source_linedef: linedef.source,
                activation: DoomLineActivation::Cross,
            },
        );
        match resolution {
            DoomLineActivationResolution::Accepted {
                intent: DoomLineActivationIntent::LowerFloorTurbo { tag },
                ..
            } => match DoomTurboLowerFloorRuntime::start_tagged(
                source,
                tag,
                DoomTurboLowerFloorPolicy::CLASSIC,
            ) {
                Ok(mut floors) => {
                    started += floors.len();
                    for floor in &mut floors {
                        let mut ticks = 0_u32;
                        while floor.phase != DoomTurboLowerFloorPhase::Complete && ticks < 4_096 {
                            floor.advance_tick();
                            ticks += 1;
                        }
                        details.push(format!(
                            "line{}:code36:tag{}:sector{}:{}->{}:ticks{}:final={:?}:one-shot=true",
                            linedef.source.record_index,
                            tag,
                            floor.target_sector.record_index,
                            floor.start_floor_height,
                            floor.destination_floor_height,
                            ticks,
                            floor.phase,
                        ));
                    }
                }
                Err(error) => {
                    rejected += 1;
                    details.push(format!(
                        "line{}:code36:tag{}:start-rejected:{error:?}",
                        linedef.source.record_index, tag
                    ));
                }
            },
            DoomLineActivationResolution::Accepted {
                intent: DoomLineActivationIntent::PlatformDownWaitUpStay { tag },
                ..
            } => match DoomDownWaitUpStayRuntime::start_tagged(
                source,
                tag,
                DoomDownWaitUpStayPolicy::CLASSIC,
            ) {
                Ok(mut platforms) => {
                    started += platforms.len();
                    for platform in &mut platforms {
                        let mut ticks = 0_u32;
                        while platform.phase != DoomDownWaitUpStayPhase::Complete && ticks < 4_096 {
                            platform.advance_tick();
                            ticks += 1;
                        }
                        details.push(format!(
                            "line{}:code88:tag{}:sector{}:high{}:low{}:ticks{}:final={:?}:retriggerable-after-complete=true",
                            linedef.source.record_index,
                            tag,
                            platform.target_sector.record_index,
                            platform.high_floor_height,
                            platform.low_floor_height,
                            ticks,
                            platform.phase,
                        ));
                    }
                }
                Err(error) => {
                    rejected += 1;
                    details.push(format!(
                        "line{}:code88:tag{}:start-rejected:{error:?}",
                        linedef.source.record_index, tag
                    ));
                }
            },
            other => {
                rejected += 1;
                details.push(format!(
                    "line{}:special{}:unexpected-resolution:{other:?}",
                    linedef.source.record_index, linedef.special
                ));
            }
        }
    }
    println!(
        "E1M1 Slice 8 moving-floor runtime observation: source_lines={}; started_sectors={started}; start_rejected={rejected}; source_map_mutated=false; presentation_mutated=false; details={}",
        details.len(),
        details.join(" | "),
    );
}

pub(crate) fn report_flat_normals(draws: &[StaticDrawPlanEntry]) {
    let mut floors_up = 0;
    let mut floors_down = 0;
    let mut ceilings_up = 0;
    let mut ceilings_down = 0;
    for draw in draws {
        let StaticDrawSource::Flat { plane, .. } = draw.source else {
            continue;
        };
        let normal_y = draw.mesh.normals.first().map_or(0.0, |normal| normal[1]);
        let is_floor = plane == doom_geometry_provider::DoomSurfacePlane::Floor;
        match (is_floor, normal_y.is_sign_positive()) {
            (true, true) => floors_up += 1,
            (true, false) => floors_down += 1,
            (false, true) => ceilings_up += 1,
            (false, false) => ceilings_down += 1,
        }
    }
    println!(
        "E1M1 flat-normal observation: floor_up={floors_up}; floor_down={floors_down}; ceiling_up={ceilings_up}; ceiling_down={ceilings_down}"
    );
}

pub(crate) fn report_doom_membership_union(
    scene: &SceneInput,
    center: Vec3,
    radius: f32,
    include_cutouts: bool,
) {
    let size = [1280.0, 800.0];
    let poses = [
        ("overview", scene_camera(size, center, radius, None, None)),
        (
            "spawn-yaw-plus-90",
            scene_camera(
                size,
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
    for (name, camera) in poses {
        let view_projection = camera.projection * camera.view;
        let selection_started = Instant::now();
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
        let draws = scene.opaque_draws.iter().chain(
            include_cutouts
                .then_some(&scene.cutout_draws)
                .into_iter()
                .flatten(),
        );
        let submitted = draws
            .filter(|draw| {
                membership_draw_selected(
                    draw,
                    &selected_subsectors,
                    &scene.membership_selection.linedef_subsectors,
                )
            })
            .count();
        let selection_cpu_us = selection_started.elapsed().as_micros();
        println!(
            "E1M1 AR-0025 membership-union control: pose={name}; source_subsectors={}/{}; submitted_draws={submitted}; candidates={}; selection_cpu_us={selection_cpu_us}; meaning=conservative-source-membership-not-bsp-visibility",
            selected_subsectors.iter().filter(|selected| **selected).count(),
            selected_subsectors.len(),
            scene.opaque_draws.len() + if include_cutouts { scene.cutout_draws.len() } else { 0 },
        );
    }
}
