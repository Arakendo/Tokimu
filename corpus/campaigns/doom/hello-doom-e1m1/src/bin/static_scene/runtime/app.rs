//! Native E1M1 application lifecycle and interaction composition.
//!
//! This remains corpus-local: it coordinates platform events, observer motion,
//! Doom interaction, and prepared presentation without promoting those policies
//! into engine crates.

use super::super::*;
use crate::render_strategies::source_covered_global_shell;
use crate::render_strategies::source_occurrence_supported;
use hello_doom_e1m1::ordered_occurrence::prepare_ordered_occurrence_declarations;

impl App {
    fn refresh_ordered_coverage_for_observer(&mut self) -> PlatformResult<()> {
        let (Some(source), Some(observer), Some(look)) = (
            self.ordered_coverage_source.as_ref(),
            self.spawn_observer,
            self.observer_look,
        ) else {
            return Ok(());
        };
        let (source_position, source_heading_radians) =
            observer_doom_source_pose(observer, look, self.comparative_embedding);
        let view = DoomOrderedCoverageView {
            source_position,
            source_heading_radians,
            eye_height: f64::from(observer.position.y),
        };
        let identity = OrderedPreparationIdentity {
            source_position: view.source_position,
            source_heading_bits: view.source_heading_radians.to_bits(),
            eye_height: view.eye_height as i16,
            door_ceilings: self
                .active_manual_doors
                .iter()
                .map(|door| (door.target_sector.record_index, door.current_ceiling_height))
                .collect(),
            turbo_floors: self
                .active_turbo_floors
                .iter()
                .map(|floor| (floor.target_sector.record_index, floor.current_floor_height))
                .collect(),
            platform_floors: self
                .active_down_wait_up_platforms
                .iter()
                .map(|platform| {
                    (
                        platform.target_sector.record_index,
                        platform.current_floor_height,
                    )
                })
                .collect(),
        };
        if self.ordered_preparation_identity.as_ref() == Some(&identity) {
            return Ok(());
        }
        let cutout_materials = source
            .cutout_uploads
            .iter()
            .map(|upload| (upload.source_name.clone(), upload.material))
            .collect::<BTreeMap<_, _>>();
        // Activation and time progression remain in this application. The
        // preparer receives only their already-current sector-height facts.
        let runtime_map = self.current_doom_visibility_map()?;
        let (mut opaque_draws, mut cutout_draws, conservation_report, preparation_mode) =
            if self.source_occurrence_support_filter {
                let prepared = source_occurrence_supported::prepare(
                    source,
                    &runtime_map,
                    view.source_position,
                    view.source_heading_radians,
                    view.eye_height as i16,
                )?;
                (
                    prepared.opaque_draws,
                    prepared.cutout_draws,
                    prepared.report,
                    "source-occurrence-supported",
                )
            } else if self.source_covered_domain_filter {
                let prepared = source_covered_global_shell::prepare(
                    source,
                    &runtime_map,
                    view.source_position,
                    view.source_heading_radians,
                )?;
                (
                    prepared.opaque_draws,
                    prepared.cutout_draws,
                    prepared.observation.report(),
                    "source-covered-global-shell",
                )
            } else {
                let prepared = prepare_ordered_occurrence_declarations(
                    &runtime_map,
                    view.source_position,
                    view.source_heading_radians,
                    view.eye_height as i16,
                    &source.door_geometry_source.wall_extents,
                    &source.door_geometry_source.wall_materials,
                    &cutout_materials,
                    &source.opaque_uploads,
                )
                .map_err(io::Error::other)?;
                (
                    prepared.opaque_draws,
                    prepared.cutout_draws,
                    prepared.conservation_report,
                    "ordered-occurrence",
                )
            };
        reembed_draws_for_comparison(&mut opaque_draws, self.comparative_embedding);
        reembed_draws_for_comparison(&mut cutout_draws, self.comparative_embedding);

        self.draws = opaque_draws;
        self.cutout_draws = cutout_draws;
        self.opaque_bounds = draw_bounds(&self.draws);
        self.cutout_bounds = draw_bounds(&self.cutout_draws);
        self.opaque_grid = self
            .opaque_grid
            .as_ref()
            .and_then(|_| UniformGridAabbIndex::build(&self.opaque_bounds, [8, 4, 8]));
        self.cutout_grid = self
            .cutout_grid
            .as_ref()
            .and_then(|_| UniformGridAabbIndex::build(&self.cutout_bounds, [8, 4, 8]));
        self.opaque_selected = vec![true; self.draws.len()];
        self.cutout_selected = vec![true; self.cutout_draws.len()];
        self.opaque_draw_enabled = vec![true; self.draws.len()];
        self.dynamic_door_draws.clear();
        self.dynamic_door_mesh_handles.clear();
        self.dirty_opaque_meshes.clear();
        self.commands.clear();
        if let Some(mut renderer) = self.renderer.take() {
            self.upload_static_meshes(&mut renderer);
            self.renderer = Some(renderer);
        }
        self.ordered_preparation_identity = Some(identity);
        eprintln!(
            "E1M1 live Doom preparation refreshed: mode={preparation_mode}; source=({},{}); heading_degrees={:.3}; eye_height={:.3}; opaque_draws={}; cutout_draws={}; submission={}; {}",
            view.source_position[0],
            view.source_position[1],
            view.source_heading_radians.to_degrees(),
            view.eye_height,
            self.draws.len(),
            self.cutout_draws.len(),
            candidate_selection_label(self.candidate_selection, true),
            conservation_report,
        );
        Ok(())
    }

    fn set_mouse_captured(&mut self, captured: bool) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        if captured {
            let grabbed = window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
            if grabbed.is_ok() {
                window.set_cursor_visible(false);
                self.mouse_captured = true;
                if let Some(look) = self.observer_look.as_mut() {
                    look.last_cursor = None;
                }
            }
        } else {
            let _ = window.set_cursor_grab(CursorGrabMode::None);
            window.set_cursor_visible(true);
            self.mouse_captured = false;
            if let Some(look) = self.observer_look.as_mut() {
                look.last_cursor = None;
            }
        }
    }

    fn apply_inspection_movement(&mut self, delta_seconds: f64) {
        if self.debug_console.is_open() {
            return;
        }
        let Some(observer) = self.spawn_observer else {
            return;
        };
        let Some(look) = self.observer_look else {
            return;
        };
        if let Some(delta) = inspection_movement_delta(
            &self.input,
            look.yaw,
            self.noclip,
            delta_seconds,
            WALK_SPEED,
            RUN_SPEED_MULTIPLIER,
        ) {
            if let Some(collision) = self.walk_collision.as_ref().filter(|_| !self.noclip) {
                let source_before =
                    observer_doom_source_pose(observer, look, self.comparative_embedding).0;
                let observation = collision.move_disc_in_embedding(
                    self.comparative_embedding,
                    [observer.position.x, observer.position.z],
                    [delta.x, delta.z],
                    WALK_RADIUS,
                );
                if observation.contacted_linedefs != self.last_collision_contacts {
                    if !observation.contacted_linedefs.is_empty() {
                        eprintln!(
                            "E1M1 walk collision: contacts={:?}; broad_phase_candidates={}; fallback_to_all_blocking_walls={}",
                            observation.contacted_linedefs,
                            observation.broad_phase_candidates,
                            observation.used_full_wall_fallback,
                        );
                    }
                    self.last_collision_contacts = observation.contacted_linedefs;
                }
                self.apply_walk_floor_transition(observation.resolved_position);
                if let (Some(observer), Some(look)) = (self.spawn_observer, self.observer_look) {
                    let source_after =
                        observer_doom_source_pose(observer, look, self.comparative_embedding).0;
                    self.try_cross_special_lines(source_before, source_after);
                }
            } else if let Some(observer) = self.spawn_observer.as_mut() {
                observer.position += delta;
            }
        }
    }

    /// Applies a source-sector floor result after horizontal collision. This
    /// keeps vertical state at the corpus application edge: `tokimu-render`
    /// still receives only the resulting camera, and imported WAD records are
    /// not mutated.
    fn apply_walk_floor_transition(&mut self, candidate_position: [f32; 2]) {
        let Some(floors) = self.walk_floors.as_ref() else {
            return;
        };
        let active_ceiling_overrides = self
            .active_manual_doors
            .iter()
            // Retain closed entries too: the final closing tick must restore
            // the original source-height wall spans, not leave the last open
            // geometry resident after the door has finished moving.
            .map(|door| (door.target_sector, door.current_ceiling_height))
            .collect::<Vec<_>>();
        let active_floor_overrides = self
            .active_turbo_floors
            .iter()
            .map(|floor| (floor.target_sector, floor.current_floor_height))
            .chain(
                self.active_down_wait_up_platforms
                    .iter()
                    .map(|platform| (platform.target_sector, platform.current_floor_height)),
            )
            .collect::<Vec<_>>();
        let Some(observer) = self.spawn_observer.as_mut() else {
            return;
        };
        let resolution = floors.resolve_transition_in_embedding(
            self.comparative_embedding,
            candidate_position,
            observer.floor,
            &active_floor_overrides,
            &active_ceiling_overrides,
        );
        match resolution {
            DoomWalkFloorResolution::Accepted {
                source_sector,
                floor_height,
                ceiling_height,
            } => {
                let floor_delta = f32::from(floor_height - observer.floor);
                observer.position.x = candidate_position[0];
                observer.position.z = candidate_position[1];
                observer.position.y += floor_delta;
                observer.sector = source_sector.record_index;
                observer.floor = floor_height;
                observer.ceiling = ceiling_height;
                let message = format!(
                    "accepted:sector={}:floor={floor_height}:ceiling={ceiling_height}",
                    source_sector.record_index
                );
                if self.last_floor_transition.as_deref() != Some(&message) {
                    eprintln!("E1M1 walk floor transition: {message}");
                    self.last_floor_transition = Some(message);
                }
            }
            DoomWalkFloorResolution::StepTooHigh {
                source_sector,
                current_floor_height,
                candidate_floor_height,
                maximum_step_up,
            } => {
                let message = format!(

                    "blocked-step:sector={}:from={current_floor_height}:to={candidate_floor_height}:max-up={maximum_step_up}",
                    source_sector.record_index
                );
                if self.last_floor_transition.as_deref() != Some(&message) {
                    eprintln!("E1M1 walk floor transition: {message}");
                    self.last_floor_transition = Some(message);
                }
            }
            DoomWalkFloorResolution::InsufficientClearance {
                source_sector,
                floor_height,
                ceiling_height,
                required_clearance,
            } => {
                let message = format!(
                    "blocked-clearance:sector={}:floor={floor_height}:ceiling={ceiling_height}:required={required_clearance}",
                    source_sector.record_index
                );
                if self.last_floor_transition.as_deref() != Some(&message) {
                    eprintln!("E1M1 walk floor transition: {message}");
                    self.last_floor_transition = Some(message);
                }
            }
            DoomWalkFloorResolution::PointOutsideUniqueSubsector { point } => {
                let message = format!("retained-ambiguous-point=({}, {})", point[0], point[1]);
                if self.last_floor_transition.as_deref() != Some(&message) {
                    eprintln!("E1M1 walk floor transition: {message}");
                    self.last_floor_transition = Some(message);
                }
            }
        }
    }

    fn release_walk_keys(&mut self) {
        release_navigation_keys(&mut self.input);
    }

    fn reset_spawn_observer(&mut self) {
        self.spawn_observer = self.initial_spawn_observer;
        self.observer_look = self.initial_observer_look;

        self.last_collision_contacts.clear();
        self.last_floor_transition = None;
        eprintln!("E1M1 source-spawn observer reset");
    }

    fn toggle_debug_console(&mut self) {
        let opening = !self.debug_console.is_open();
        if opening {
            self.set_mouse_captured(false);
            self.release_walk_keys();
        }

        self.debug_console.set_open(opening);
    }

    fn set_bsp_diagnostic_focus(&mut self, focus: BspDiagnosticFocus) {
        if !self.bsp_diagnostic_enabled || self.bsp_diagnostic_focus == focus {
            return;
        }
        self.bsp_diagnostic_focus = focus;
        let message = format!("BSP diagnostic focus={}", focus.label());
        eprintln!("E1M1 {message}; membership=unchanged-global-full");
        self.debug_console.append(message);
    }

    fn submit_debug_console(&mut self) {
        let Some(command) = self.debug_console.take_submission() else {
            return;
        };
        eprintln!("[doom-console] > {command}");
        let response = match parse_debug_command(&command) {
            DebugCommand::Help => "commands: HELP | CLEAR | STATUS | CAMERA | COLLISION | LOOK [PIXEL x y|NDC x y] | SCAN [columns rows] | USE <linedef> | NOCLIP [ON|OFF|TOGGLE]".to_owned(),
            DebugCommand::Clear => {
                self.debug_console.clear();
                eprintln!("[doom-console] [doom] transcript cleared");
                return;
            }
            DebugCommand::Camera => self.spawn_observer.map_or_else(
                || "camera: source-spawn observer unavailable".to_owned(),
                |observer| {
                    let look = self.observer_look.unwrap_or(ObserverLook {
                        yaw: 0.0,
                        pitch: 0.0,
                        last_cursor: None,
                    });
                    let (source_position, source_angle) =
                        observer_doom_source_pose(observer, look, self.comparative_embedding);
                    let (source_origin, source_height) = self
                        .comparative_embedding
                        .lower_direction(observer.position);
                    let (source_direction, source_direction_height) = self
                        .comparative_embedding
                        .lower_direction(observer_direction(look.yaw, look.pitch));
                    format!(
                        "camera: position=({:.2},{:.2},{:.2}) yaw={:.4} pitch={:.4} source_thing={} source_pose=({},{};{:.1}deg) sector={} floor={} ceiling={} headless_scan_replay=--bsp-diagnostic-scan-report={:.9},{:.9},{:.9},{:.9},{:.9},{:.9},{:.0},{:.0},{DEFAULT_SCAN_COLUMNS},{DEFAULT_SCAN_ROWS}",
                        observer.position.x,
                        observer.position.y,
                        observer.position.z,
                        look.yaw,
                        look.pitch,
                        observer.source_record,
                        source_position[0],
                        source_position[1],
                        source_angle.to_degrees(),
                        observer.sector,
                        observer.floor,
                        observer.ceiling,
                        source_origin[0],
                        source_origin[1],
                        source_height,
                        source_direction[0],
                        source_direction[1],
                        source_direction_height,
                        self.size[0],
                        self.size[1],
                    )
                },
            ),
            DebugCommand::Status => format!(
                "status: frame={} draws={} cutouts={} selection={:?} mouse_capture={} noclip={} active_manual_doors={} details={}",
                self.frame_index,
                self.draws.len(),
                self.cutout_draws.len(),
                self.candidate_selection,
                self.mouse_captured,
                self.noclip,
                self.active_manual_doors
                    .iter()
                    .filter(|door| door.phase != DoomManualDoorPhase::Closed)
                    .count(),
                self.active_manual_doors
                    .iter()
                    .filter(|door| door.phase != DoomManualDoorPhase::Closed)
                    .map(|door| format!(
                        "sector{}:{}/{}/{}:{:?}",
                        door.target_sector.record_index,
                        door.current_ceiling_height,
                        door.closed_ceiling_height,
                        door.open_ceiling_height,
                        door.phase,
                    ))
                    .collect::<Vec<_>>()
                    .join("|"),
            ),
            DebugCommand::Collision => self.walk_collision.as_ref().map_or_else(
                || "collision: unavailable; run with --walk-collision".to_owned(),
                |world| {
                    format!(
                        "collision: radius={WALK_RADIUS} blocking_linedefs={} noclip={} current_sector={} floor={} ceiling={} last_contacts={:?}",
                        world.blocking_wall_count(),
                        self.noclip,
                        self.spawn_observer.map_or(0, |observer| observer.sector),
                        self.spawn_observer.map_or(0, |observer| observer.floor),
                        self.spawn_observer.map_or(0, |observer| observer.ceiling),
                        self.last_collision_contacts,
                    )
                },
            ),
            DebugCommand::Look(command) => self.resolve_look_command(&command),
            DebugCommand::Scan(command) => self.resolve_scan_command(&command),
            DebugCommand::Use(command) => self.resolve_use_command(&command),
            DebugCommand::Noclip(NoclipAction::Toggle) => {
                self.noclip = !self.noclip;
                format!("noclip: {}", self.noclip)
            }
            DebugCommand::Noclip(NoclipAction::On) => {
                self.noclip = true;
                "noclip: true".to_owned()
            }
            DebugCommand::Noclip(NoclipAction::Off) => {
                self.noclip = false;
                "noclip: false".to_owned()
            }
            DebugCommand::Unsupported(command) => format!("unsupported command: {command}"),
        };
        eprintln!("[doom-console] {response}");
        self.debug_console.append(response);
    }

    fn resolve_use_command(&mut self, command: &str) -> String {
        let argument = command.strip_prefix("use").unwrap_or_default().trim();
        let Ok(record_index) = argument.parse::<u32>() else {
            return "use: expected USE <source-linedef-index>; LOOK retains a wall source index"
                .to_owned();
        };
        self.resolve_use_linedef(record_index)
    }

    fn resolve_use_linedef(&mut self, record_index: u32) -> String {
        let Some(linedef) = self
            .activation_source
            .linedefs
            .iter()
            .find(|linedef| linedef.source.record_index == record_index)
        else {
            return format!("use: source linedef {record_index} is not present in E1M1");
        };
        match resolve_doom_line_activation(
            &self.activation_source,
            DoomLineActivationRequest {
                source_linedef: linedef.source,
                activation: DoomLineActivation::Use,
            },
        ) {
            DoomLineActivationResolution::Accepted {
                source_linedef,
                special,
                intent:
                    DoomLineActivationIntent::RaiseDoor {
                        target_sector,
                    },
            } => self.start_manual_door(source_linedef.record_index, special, target_sector),
            DoomLineActivationResolution::Accepted {
                source_linedef,
                special,
                intent,
            } => format!(
                "use: accepted linedef={} lump={} special={} intent={} target={} execution=deferred-to-future-runtime-owner",
                source_linedef.record_index,
                source_linedef.lump_index,
                special,
                compact_activation_intent(intent),
                compact_activation_target(intent),
            ),
            DoomLineActivationResolution::NoSpecial { source_linedef } => format!(
                "use: linedef={} lump={} has no source special",
                source_linedef.record_index, source_linedef.lump_index,
            ),
            DoomLineActivationResolution::WrongActivation {
                source_linedef,
                special,
                required,
                ..
            } => format!(
                "use: linedef={} special={} requires {:?}; requested Use",
                source_linedef.record_index, special, required,
            ),
            DoomLineActivationResolution::UnsupportedSpecial {
                source_linedef,
                special,
            } => format!(
                "use: linedef={} special={} is retained but not admitted for a use request",
                source_linedef.record_index, special,
            ),
            DoomLineActivationResolution::UnknownLinedef { source_linedef } => format!(
                "use: source linedef={} lump={} is unavailable",
                source_linedef.record_index, source_linedef.lump_index,
            ),
            DoomLineActivationResolution::MissingManualDoorTarget {
                source_linedef,
                missing_left_sidedef,
            } => format!(
                "use: manual-door linedef={} cannot resolve opposite sidedef={missing_left_sidedef:?}",
                source_linedef.record_index,
            ),
            DoomLineActivationResolution::InvalidManualDoorTarget {
                source_linedef,
                sidedef_index,
                sector_index,

            } => format!(
                "use: manual-door linedef={} has invalid target sidedef={} sector={}",
                source_linedef.record_index, sidedef_index, sector_index,
            ),
        }
    }

    fn try_use_center_wall(&mut self) -> String {
        let (Some(observer), Some(look)) = (self.spawn_observer, self.observer_look) else {
            return "use: source-spawn observer unavailable".to_owned();
        };
        let (source_position, source_angle) =
            observer_doom_source_pose(observer, look, self.comparative_embedding);
        let ray = [source_angle.cos(), source_angle.sin()];
        let active_ceiling_overrides = self
            .active_manual_doors
            .iter()
            .map(|door| (door.target_sector.record_index, door.current_ceiling_height))
            .collect::<BTreeMap<_, _>>();
        match trace_doom_use_lines(
            &self.door_geometry_source.map,
            source_position,
            ray,
            &active_ceiling_overrides,
        ) {
            DoomUseTraceResult::Special { distance, linedef } => {
                let outcome = self.resolve_use_linedef(linedef);
                format!("use: source-trace-distance={distance:.3}; {outcome}")
            }
            DoomUseTraceResult::BackSide { distance, linedef } => format!(
                "use: source-trace-distance={distance:.3}; linedef={linedef}; rejected=back-side"
            ),
            DoomUseTraceResult::Blocked { distance, linedef } => format!(
                "use: source-trace-distance={distance:.3}; linedef={linedef}; blocked=closed-nonspecial-line"
            ),
            DoomUseTraceResult::NoIntercept => format!(
                "use: no source linedef intersects the classic {CLASSIC_USE_RANGE:.0}-unit trace"
            ),
        }
    }

    fn start_manual_door(
        &mut self,
        source_linedef: u32,
        special: u16,
        target_sector: doom_map_provider::DoomSourceRecord,
    ) -> String {
        let replacement = match DoomManualDoorRuntime::start(
            &self.activation_source,
            target_sector,
            DoomManualDoorPolicy::CLASSIC_NORMAL,
        ) {
            Ok(door) => door,
            Err(error) => {
                return format!(
                    "use: manual-door linedef={source_linedef} special={special} target-sector={} start-rejected={error:?}",
                    target_sector.record_index,
                );
            }
        };
        if let Some(active) = self
            .active_manual_doors
            .iter_mut()
            .find(|door| door.target_sector == target_sector)
        {
            if active.phase != DoomManualDoorPhase::Closed {
                let (before, after) = active
                    .reuse_by_player()
                    .expect("non-closed door must accept player reuse");
                return format!(
                    "use: manual-door linedef={source_linedef} target-sector={} reused phase={before:?}->{after:?}",
                    target_sector.record_index,
                );
            }
            *active = replacement;
        } else {
            self.active_manual_doors.push(replacement);
        }
        let boundary_linedefs =
            manual_door_boundary_linedefs(&self.activation_source, target_sector);
        let prepared_meshes_at_closed_height = self
            .draws
            .iter()
            .filter(|draw| {
                is_door_mesh_for_target(draw, target_sector, &boundary_linedefs)
                    && draw.mesh.positions.iter().any(|position| {
                        (position[1] - f32::from(replacement.closed_ceiling_height)).abs()
                            <= f32::EPSILON
                    })
            })
            .count();
        format!(
            "use: manual-door started linedef={source_linedef} special={special} target-sector={} closed-height={} open-height={} prepared-meshes-at-closed-height={prepared_meshes_at_closed_height} policy=2-units-per-tick/150-tick-wait",
            target_sector.record_index,
            replacement.closed_ceiling_height,
            replacement.open_ceiling_height,
        )
    }

    fn try_cross_special_lines(&mut self, source_before: [i16; 2], source_after: [i16; 2]) {
        let crossings = source_motion_special_crossings(
            &self.door_geometry_source.map.vertices,
            &self.door_geometry_source.map.linedefs,
            source_before,
            source_after,
        );
        for source_linedef in crossings {
            let resolution = resolve_doom_line_activation(
                &self.activation_source,
                DoomLineActivationRequest {
                    source_linedef,
                    activation: DoomLineActivation::Cross,
                },
            );

            let message = match resolution {
                DoomLineActivationResolution::Accepted {
                    special: 36,
                    intent: DoomLineActivationIntent::LowerFloorTurbo { tag },
                    ..
                } => {
                    if self
                        .consumed_one_shot_cross_lines
                        .contains(&source_linedef.record_index)
                    {
                        continue;
                    }
                    match DoomTurboLowerFloorRuntime::start_tagged(
                        &self.activation_source,
                        tag,
                        DoomTurboLowerFloorPolicy::CLASSIC,
                    ) {
                        Ok(floors) => {
                            let targets = floors
                                .iter()
                                .map(|floor| floor.target_sector.record_index.to_string())
                                .collect::<Vec<_>>()
                                .join(",");
                            self.active_turbo_floors.extend(floors);
                            self.consumed_one_shot_cross_lines
                                .insert(source_linedef.record_index);
                            format!(
                                "cross: turbo-lower started linedef={} tag={tag} sectors=[{targets}]",
                                source_linedef.record_index
                            )
                        }
                        Err(error) => format!(
                            "cross: turbo-lower linedef={} tag={tag} start-rejected={error:?}",
                            source_linedef.record_index
                        ),
                    }
                }
                DoomLineActivationResolution::Accepted {
                    special: 88,
                    intent: DoomLineActivationIntent::PlatformDownWaitUpStay { tag },
                    ..
                } => match DoomDownWaitUpStayRuntime::start_tagged(
                    &self.activation_source,
                    tag,
                    DoomDownWaitUpStayPolicy::CLASSIC,
                ) {
                    Ok(platforms) => {
                        let mut started = Vec::new();
                        for platform in platforms {
                            if let Some(active) = self
                                .active_down_wait_up_platforms
                                .iter_mut()
                                .find(|active| active.target_sector == platform.target_sector)
                            {
                                if active.phase != DoomDownWaitUpStayPhase::Complete {
                                    continue;
                                }
                                *active = platform;
                            } else {
                                self.active_down_wait_up_platforms.push(platform);
                            }
                            started.push(platform.target_sector.record_index.to_string());
                        }
                        if started.is_empty() {
                            format!(
                                "cross: platform linedef={} tag={tag} already-active",
                                source_linedef.record_index
                            )
                        } else {
                            format!(
                                "cross: platform started linedef={} tag={tag} sectors=[{}]",
                                source_linedef.record_index,
                                started.join(",")
                            )
                        }
                    }
                    Err(error) => format!(
                        "cross: platform linedef={} tag={tag} start-rejected={error:?}",
                        source_linedef.record_index
                    ),
                },
                DoomLineActivationResolution::Accepted {
                    special: 11,
                    intent: DoomLineActivationIntent::ExitLevel { .. },
                    ..
                } => format!(
                    "cross: exit linedef={} retained; map transition not implemented",
                    source_linedef.record_index
                ),
                other => format!(
                    "cross: linedef={} unexpected-resolution={other:?}",
                    source_linedef.record_index
                ),
            };
            eprintln!("E1M1 {message}");

            self.debug_console.append(message);
        }
    }

    pub(super) fn advance_active_moving_floors(&mut self, delta_seconds: f64) {
        self.moving_floor_tick_accumulator += delta_seconds.clamp(0.0, 0.25);
        let mut changed = false;
        while self.moving_floor_tick_accumulator >= DOOM_TIC_SECONDS {
            self.moving_floor_tick_accumulator -= DOOM_TIC_SECONDS;
            let floor_transitions = self
                .active_turbo_floors
                .iter_mut()
                .filter(|floor| floor.phase != DoomTurboLowerFloorPhase::Complete)
                .filter_map(|floor| {
                    let before = floor.current_floor_height;
                    floor.advance_tick();
                    (before != floor.current_floor_height).then_some((
                        floor.target_sector,
                        before,
                        floor.current_floor_height,
                    ))
                })
                .collect::<Vec<_>>();
            let platform_transitions = self
                .active_down_wait_up_platforms
                .iter_mut()
                .filter(|platform| platform.phase != DoomDownWaitUpStayPhase::Complete)
                .filter_map(|platform| {
                    let before = platform.current_floor_height;
                    platform.advance_tick();
                    (before != platform.current_floor_height).then_some((
                        platform.target_sector,
                        before,
                        platform.current_floor_height,
                    ))
                })
                .collect::<Vec<_>>();
            for (target_sector, before, after) in
                floor_transitions.into_iter().chain(platform_transitions)
            {
                self.dirty_opaque_meshes.extend(apply_sector_flat_height(
                    &mut self.draws,
                    target_sector,
                    doom_geometry_provider::DoomSurfacePlane::Floor,
                    before,
                    after,
                ));
                carry_observer_with_floor(
                    self.spawn_observer.as_mut(),
                    target_sector,
                    before,
                    after,
                );
                changed = true;
            }
        }
        if changed {
            match self.refresh_active_dynamic_wall_meshes() {
                Ok(()) => self.door_visual_diagnostic = None,
                Err(error) => {
                    let diagnostic = format!("moving-sector visual refresh failed: {error}");
                    if self.door_visual_diagnostic.as_deref() != Some(&diagnostic) {
                        eprintln!("E1M1 {diagnostic}");
                        self.debug_console.append(diagnostic.clone());
                    }
                    self.door_visual_diagnostic = Some(diagnostic);
                }
            }
        }
    }

    fn advance_active_manual_doors(&mut self, delta_seconds: f64) {
        self.door_tick_accumulator += delta_seconds.clamp(0.0, 0.25);
        let mut changed = false;
        while self.door_tick_accumulator >= DOOM_TIC_SECONDS {
            self.door_tick_accumulator -= DOOM_TIC_SECONDS;
            let transitions = self
                .active_manual_doors
                .iter_mut()
                .filter(|door| door.phase != DoomManualDoorPhase::Closed)
                .map(DoomManualDoorRuntime::advance_tick)
                .filter(|tick| tick.before_height != tick.after_height)
                .collect::<Vec<_>>();

            for tick in transitions {
                self.dirty_opaque_meshes
                    .extend(apply_door_ceiling_flat_height(
                        &mut self.draws,
                        tick.target_sector,
                        tick.before_height,
                        tick.after_height,
                    ));
                changed = true;
            }
        }
        if changed {
            match self.refresh_active_dynamic_wall_meshes() {
                Ok(()) => self.door_visual_diagnostic = None,
                Err(error) => {
                    let diagnostic = format!("door visual refresh failed: {error}");
                    if self.door_visual_diagnostic.as_deref() != Some(&diagnostic) {
                        eprintln!("E1M1 {diagnostic}");
                        self.debug_console.append(diagnostic.clone());
                    }
                    self.door_visual_diagnostic = Some(diagnostic);
                }
            }
        }
    }

    /// Produces the Doom-local topology snapshot used by the live Stage 3B
    /// visibility control. Decoded WAD records remain immutable; only the
    /// already-authoritative corpus runtime heights are projected into this
    /// short-lived source view before BSP traversal.
    fn current_doom_visibility_map(&self) -> PlatformResult<DoomMapCore> {
        let snapshots =
            self.active_manual_doors
                .iter()
                .map(|door| DoomSectorRuntimeHeightSnapshot {
                    source_sector: door.target_sector,
                    floor_height: None,
                    ceiling_height: Some(door.current_ceiling_height),
                })
                .chain(self.active_turbo_floors.iter().map(|floor| {
                    DoomSectorRuntimeHeightSnapshot {
                        source_sector: floor.target_sector,
                        floor_height: Some(floor.current_floor_height),
                        ceiling_height: None,
                    }
                }))
                .chain(self.active_down_wait_up_platforms.iter().map(|platform| {
                    DoomSectorRuntimeHeightSnapshot {
                        source_sector: platform.target_sector,
                        floor_height: Some(platform.current_floor_height),
                        ceiling_height: None,
                    }
                }))
                .collect::<Vec<_>>();
        Ok(project_doom_sector_runtime_heights(
            &self.door_geometry_source.map,
            &snapshots,
        )?)
    }

    /// Prepares Candidate 1 as one all-or-nothing Doom-owned G2 batch.
    ///
    /// The ordered Doom protocol owns which F_SKY1 regions have authority and
    /// which source SEG supplies their depth. The renderer sees only bounded,
    /// submission-local clip geometry. Any unresolved source fact fails open
    /// for the entire batch, leaving the unchanged global scene submission as
    /// the frame's sole geometry authority.
    fn prepare_candidate1_sky_depth_batch(
        &self,
    ) -> PlatformResult<Option<Candidate1SkyDepthBatch>> {
        if !self.candidate1_sky_depth_enabled {
            return Ok(None);
        }
        let (Some(observer), Some(look)) = (self.spawn_observer, self.observer_look) else {
            if self.frame_index == 0 {
                eprintln!(
                    "E1M1 Candidate 1 fail-open: reason=missing-source-observer; local-batch=omitted; global-full-submission=unchanged"
                );
            }
            return Ok(None);
        };

        let map = self.current_doom_visibility_map()?;
        let (source_position, heading_radians) =
            observer_doom_source_pose(observer, look, self.comparative_embedding);
        let source_eye_height = observer.position.y.round() as i16;
        let fixture = DoomVisibilityFixture {
            name: "e1m1-candidate1-authoritative-sky".to_owned(),
            map: map.clone(),
            viewer: DoomFixtureViewer {
                position: source_position,
                heading_radians,
            },
            watched_subsectors: BTreeSet::new(),
        };
        let traversal = fixture.observe_classic_bsp().map_err(io::Error::other)?;
        let vertical = fixture
            .observe_classic_vertical_clips(
                source_eye_height,
                &self.door_geometry_source.wall_extents,
            )
            .map_err(io::Error::other)?;
        let runtime_snapshot = format!(
            "doors={:?};turbo-floors={:?};platforms={:?}",
            self.active_manual_doors
                .iter()
                .map(|door| (door.target_sector, door.current_ceiling_height))
                .collect::<Vec<_>>(),
            self.active_turbo_floors
                .iter()
                .map(|floor| (floor.target_sector, floor.current_floor_height))
                .collect::<Vec<_>>(),
            self.active_down_wait_up_platforms
                .iter()
                .map(|platform| (platform.target_sector, platform.current_floor_height))
                .collect::<Vec<_>>(),
        );
        let regions = model_authoritative_sky_regions(
            &vertical,
            &traversal.admitted_seg_order,
            AuthoritativeSkyViewIdentity {
                fixture: fixture.name.clone(),
                source_position,
                heading_radians,
                source_eye_height,
            },
            &runtime_snapshot,
        );
        let region_conservation = !regions.fail_open
            && regions.omitted_sky_intervals == 0
            && regions.omitted_sky_cells == 0
            && regions.removed_non_sky_contributions == 0
            && regions.modeled_sky_intervals == regions.input_sky_intervals
            && regions.modeled_sky_cells == regions.input_sky_cells;
        if !region_conservation {
            if self.frame_index < 2 {
                eprintln!(
                    "E1M1 Candidate 1 fail-open: reason=authoritative-region-conservation; regions={}; input-intervals={}; modeled-intervals={}; omitted-intervals={}; input-cells={}; modeled-cells={}; omitted-cells={}; removed-non-sky={}; local-batch=omitted; global-full-submission=unchanged",
                    regions.regions.len(),
                    regions.input_sky_intervals,
                    regions.modeled_sky_intervals,
                    regions.omitted_sky_intervals,
                    regions.input_sky_cells,
                    regions.modeled_sky_cells,
                    regions.omitted_sky_cells,
                    regions.removed_non_sky_contributions,
                );
            }
            return Ok(None);
        }
        if regions.regions.is_empty() {
            if self.frame_index < 2 {
                eprintln!(
                    "E1M1 Candidate 1 inactive: reason=no-authoritative-sky-in-current-view; input-intervals=0; modeled-intervals=0; local-batch=not-required; global-full-submission=unchanged"
                );
            }
            return Ok(None);
        }

        let near = f64::from((self.radius * 0.000_1).max(0.1));
        let far = f64::from(self.radius * 4.0);
        let depth = prepare_authoritative_sky_source_depth_declarations(
            &regions,
            &map,
            near,
            far,
            "e1m1-candidate1-authoritative-sky-depth",
        );
        let approximation =
            observe_authoritative_sky_source_depth_approximation(&regions, &depth, &map, near, far);
        let depth_conservation = depth.persistent_mesh_identities == 0
            && depth.declarations.len() == regions.regions.len()
            && depth.outcomes.len() == regions.regions.len()
            && depth
                .outcomes
                .iter()
                .all(|outcome| outcome.declaration.is_some() && outcome.rejection.is_none());
        if !depth_conservation {
            if self.frame_index < 2 {
                eprintln!(
                    "E1M1 Candidate 1 fail-open: reason=source-depth-realization; regions={}; declarations={}; outcomes={:?}; persistent-mesh-identities={}; local-batch=omitted; global-full-submission=unchanged",
                    regions.regions.len(),
                    depth.declarations.len(),
                    depth.outcomes,
                    depth.persistent_mesh_identities,
                );
            }
            return Ok(None);
        }
        if self.frame_index < 2 {
            eprintln!(
                "E1M1 Candidate 1 oracle/triangle comparison: oracle-samples={}; coverage-mismatches={}; coverage-extra-cells={}; coverage-missing-cells={}; depth-samples={}; unresolved-depth-samples={}; max-clip-depth-error={:.9}; mean-clip-depth-error={:.9}; meaning=doom-ledger-column-centers-versus-continuous-triangle-realization-not-pixel-parity",
                approximation.oracle_samples,
                approximation.coverage_mismatches,
                approximation.coverage_extra_cells,
                approximation.coverage_missing_cells,
                approximation.depth_samples,
                approximation.unresolved_depth_samples,
                approximation.maximum_absolute_clip_depth_error,
                approximation.mean_absolute_clip_depth_error,
            );
        }

        let submission_identity = self.frame_index.saturating_add(1);
        let snapshot = prepare_authoritative_sky_submission_local_geometry(
            &depth,
            SubmissionIdentity(submission_identity),
            SubmissionLocalGeometryLimits::default(),
        )
        .map_err(io::Error::other)?;
        if snapshot.persistent_mesh_identities != 0
            || snapshot.payloads.len() != depth.declarations.len()
            || snapshot.draws.len() != depth.declarations.len()
        {
            return Err(io::Error::other(
                "Candidate 1 G2 lowering violated submission-local conservation",
            )
            .into());
        }

        let pipeline = self
            .candidate1_sky_depth_pipeline
            .ok_or_else(|| io::Error::other("Candidate 1 sky-depth pipeline missing"))?;
        let mut builder = ExperimentalSubmissionLocalGeometryBuilder::new(
            ExperimentalSubmissionIdentity(submission_identity),
        );
        let mut local_ids = Vec::with_capacity(snapshot.payloads.len());
        for payload in &snapshot.payloads {
            local_ids.push(
                builder
                    .add_geometry(Mesh::uniform_normal(
                        payload.positions.clone(),
                        [0.0, 0.0, -1.0],
                    ))
                    .map_err(io::Error::other)?,
            );
        }
        for draw in &snapshot.draws {
            let geometry = local_ids
                .get(draw.geometry.slot as usize)
                .copied()
                .ok_or_else(|| io::Error::other("Candidate 1 local slot missing"))?;
            builder
                .add_draw(ExperimentalLocalGeometryDraw {
                    geometry,
                    material: CANDIDATE1_SKY_DEPTH_MATERIAL,
                    pipeline,
                    instance: Instance2d::identity(),
                    camera: Some(CANDIDATE1_CLIP_CAMERA),
                    viewport: None,
                    material_override: None,
                })
                .map_err(io::Error::other)?;
        }
        let batch = builder.finish().map_err(io::Error::other)?;
        Ok(Some(Candidate1SkyDepthBatch {
            batch,
            source_regions: regions.regions.len(),
            declarations: depth.declarations.len(),
            vertices: snapshot.total_vertices,
            triangles: snapshot.total_triangles,
            structural_fingerprint: snapshot.structural_fingerprint,
        }))
    }

    /// Re-lowers only wall spans attributable to the active manual-door
    /// sectors from a clone of the already decoded map. Runtime ceiling state
    /// replaces the clone's source height; WAD bytes and source records remain
    /// unchanged. This prevents vertex-only deformation from silently becoming
    /// Doom wall-span or UV policy.
    fn refresh_active_dynamic_wall_meshes(&mut self) -> PlatformResult<()> {
        let mut map = self.door_geometry_source.map.clone();
        let active_ceilings = self
            .active_manual_doors
            .iter()
            // A completed closing tick must also restore the source-height
            // spans; keeping closed entries here makes that final refresh
            // explicit.
            .map(|door| {
                (
                    door.target_sector,
                    door.current_ceiling_height,
                    manual_door_boundary_linedefs(&self.activation_source, door.target_sector),
                )
            })
            .collect::<Vec<_>>();
        let active_floors = self
            .active_turbo_floors
            .iter()
            .map(|floor| {
                (
                    floor.target_sector,
                    floor.current_floor_height,
                    manual_door_boundary_linedefs(&self.activation_source, floor.target_sector),
                )
            })
            .chain(self.active_down_wait_up_platforms.iter().map(|platform| {
                (
                    platform.target_sector,
                    platform.current_floor_height,
                    manual_door_boundary_linedefs(&self.activation_source, platform.target_sector),
                )
            }))
            .collect::<Vec<_>>();
        for (target_sector, height, _) in &active_ceilings {
            if let Some(sector) = map
                .sectors
                .iter_mut()
                .find(|sector| sector.source == *target_sector)
            {
                sector.ceiling_height = *height;
            }
        }
        for (target_sector, height, _) in &active_floors {
            if let Some(sector) = map
                .sectors
                .iter_mut()
                .find(|sector| sector.source == *target_sector)
            {
                sector.floor_height = *height;
            }
        }

        let mut dynamic_meshes = BTreeMap::<String, Vec<DynamicDoorWallMesh>>::new();
        for triangle in
            lower_doom_textured_wall_triangles(&map, &self.door_geometry_source.wall_extents)?
        {
            let affected_by_ceiling =
                active_ceilings
                    .iter()
                    .any(|(target_sector, _, boundaries)| {
                        triangle.source_sector == *target_sector
                            || (triangle.role == doom_geometry_provider::DoomWallTextureRole::Upper
                                && boundaries.contains(&triangle.source_linedef))
                    });
            let affected_by_floor = active_floors.iter().any(|(target_sector, _, boundaries)| {
                triangle.source_sector == *target_sector
                    || (triangle.role == doom_geometry_provider::DoomWallTextureRole::Lower
                        && boundaries.contains(&triangle.source_linedef))
            });
            if !affected_by_ceiling && !affected_by_floor {
                continue;
            }
            let Some(extent) = self
                .door_geometry_source
                .wall_extents
                .iter()
                .find(|extent| extent.name == triangle.texture_name)
            else {
                return Err(io::Error::other(format!(
                    "active door wall {} has no retained texture extent",
                    triangle.texture_name
                ))
                .into());
            };
            let mut mesh = match lower_static_wall_triangle(&triangle, extent.clone()) {
                Ok(lowered) => lowered.mesh,
                // These were already retained as zero-area source omissions by
                // the static preparation. A runtime height substitution can
                // encounter the same authored empty band; it is not a reason
                // to terminate the presentation loop.
                Err(StaticFlatLoweringError::DegenerateTriangle) => continue,
                Err(error) => return Err(error.into()),
            };
            // Dynamic wall spans are lowered from unchanged Doom source facts
            // after the initial scene adapter has run. Apply the same explicit
            // embedding and wall-U migration before they join that scene.
            reembed_comparative_mesh(&mut mesh, self.comparative_embedding, true);
            dynamic_meshes
                .entry(dynamic_wall_triangle_key(
                    triangle.source_linedef,
                    triangle.source_sidedef,
                    triangle.source_sector,
                    triangle.role,
                    &triangle.texture_name,
                ))
                .or_default()
                .push(DynamicDoorWallMesh {
                    mesh,
                    source_linedef: triangle.source_linedef,
                    source_sidedef: triangle.source_sidedef,
                    source_sector: triangle.source_sector,
                    role: triangle.role,

                    texture_name: triangle.texture_name,
                });
        }

        let mut existing = std::collections::BTreeMap::<String, Vec<usize>>::new();
        for (index, draw) in self.draws.iter().enumerate() {
            let affected = active_ceilings
                .iter()
                .any(|(target_sector, _, boundaries)| {
                    is_dynamic_mesh_for_target(
                        draw,
                        *target_sector,
                        doom_geometry_provider::DoomSurfacePlane::Ceiling,
                        boundaries,
                    )
                })
                || active_floors.iter().any(|(target_sector, _, boundaries)| {
                    is_dynamic_mesh_for_target(
                        draw,
                        *target_sector,
                        doom_geometry_provider::DoomSurfacePlane::Floor,
                        boundaries,
                    )
                });
            if affected {
                if let Some(key) = static_wall_triangle_key(draw) {
                    existing.entry(key).or_default().push(index);
                }
            }
        }

        for (key, indices) in existing {
            let Some(meshes) = dynamic_meshes.remove(&key) else {
                // A zero-height source band is absent from the fresh lowering.
                // Dynamic-only spans are explicitly suppressed while absent;
                // ordinary static spans retain their source-height geometry.
                for index in indices {
                    if self.dynamic_door_draws.contains(&index) {
                        self.opaque_draw_enabled[index] = false;
                    }
                }
                continue;
            };
            if meshes.len() != indices.len() {
                continue;
            }
            for (index, mesh) in indices.into_iter().zip(meshes) {
                self.draws[index].mesh = mesh.mesh;
                self.opaque_bounds[index] =
                    StaticDrawAabb::from_positions(&self.draws[index].mesh.positions);
                self.opaque_draw_enabled[index] = true;
                self.dirty_opaque_meshes.insert(index);
            }
        }
        let mut missing_materials = Vec::new();
        for meshes in dynamic_meshes.into_values() {
            for mesh in meshes {
                let Some(material) = self
                    .door_geometry_source
                    .wall_materials
                    .get(&mesh.texture_name)
                    .copied()
                else {
                    missing_materials.push(mesh.texture_name);
                    continue;
                };
                let index = self.draws.len();

                self.draws.push(StaticDrawPlanEntry {
                    mesh: mesh.mesh,
                    material,
                    source_label: format!(
                        "wall:{}:{}",
                        mesh.source_linedef.record_index, mesh.texture_name
                    ),
                    source: StaticDrawSource::Wall {
                        source_linedef: mesh.source_linedef,
                        source_sidedef: mesh.source_sidedef,
                        source_sector: mesh.source_sector,
                        role: mesh.role,
                    },
                });
                self.opaque_bounds.push(StaticDrawAabb::from_positions(
                    &self.draws[index].mesh.positions,
                ));
                self.opaque_selected.push(true);
                self.opaque_draw_enabled.push(true);
                self.dynamic_door_draws.insert(index);
                let handle = MeshHandle(self.next_dynamic_mesh_handle);
                self.next_dynamic_mesh_handle = self.next_dynamic_mesh_handle.saturating_add(1);
                self.dynamic_door_mesh_handles.insert(index, handle);
                self.dirty_opaque_meshes.insert(index);
                // The fixed grid was built for the static scene. Fall back to
                // its existing conservative non-grid selection until a later
                // corpus result earns a dynamic-index policy.
                self.opaque_grid = None;
            }
        }
        missing_materials.sort();
        missing_materials.dedup();
        if missing_materials.is_empty() {
            self.door_geometry_diagnostic = None;
        } else {
            let diagnostic = format!(
                "door geometry has no prepared material for: {}",
                missing_materials.join(", ")
            );
            if self.door_geometry_diagnostic.as_deref() != Some(&diagnostic) {
                eprintln!("E1M1 {diagnostic}");
                self.debug_console.append(diagnostic.clone());
            }
            self.door_geometry_diagnostic = Some(diagnostic);
        }
        Ok(())
    }

    pub(super) fn refresh_active_manual_door_wall_meshes(&mut self) -> PlatformResult<()> {
        self.refresh_active_dynamic_wall_meshes()
    }

    fn resolve_look_command(&self, command: &str) -> String {
        let arguments = command.split_whitespace().skip(1).collect::<Vec<_>>();
        let (ndc, sample) = match arguments.as_slice() {
            [] => ([0.0, 0.0], None),
            [space, x, y] if *space == "pixel" => {
                let (Ok(x), Ok(y)) = (x.parse::<f32>(), y.parse::<f32>()) else {
                    return "look: expected LOOK PIXEL <x> <y> with finite client-area coordinates"
                        .to_owned();
                };
                if !x.is_finite()
                    || !y.is_finite()
                    || x < 0.0
                    || y < 0.0
                    || x > self.size[0]
                    || y > self.size[1]
                {
                    return format!(
                        "look: pixel must be inside client area 0..{:.0},0..{:.0}",
                        self.size[0], self.size[1]
                    );
                }
                (
                    [2.0 * x / self.size[0] - 1.0, 1.0 - 2.0 * y / self.size[1]],
                    Some(format!("pixel=({x:.3},{y:.3})")),
                )
            }
            [space, x, y] if *space == "ndc" => {
                let (Ok(x), Ok(y)) = (x.parse::<f32>(), y.parse::<f32>()) else {
                    return "look: expected LOOK NDC <x> <y> with finite values from -1 through 1"
                        .to_owned();
                };
                if !x.is_finite()
                    || !y.is_finite()
                    || !(-1.0..=1.0).contains(&x)
                    || !(-1.0..=1.0).contains(&y)
                {
                    return "look: NDC x and y must be finite values from -1 through 1".to_owned();
                }
                ([x, y], Some(format!("ndc=({x:.6},{y:.6})")))
            }
            _ => {
                return "look: expected LOOK, LOOK PIXEL <x> <y>, or LOOK NDC <x> <y>".to_owned();
            }
        };
        let Some(look) = self.observer_look else {
            return "look: source-spawn observer unavailable".to_owned();
        };
        let direction = self.inspection_direction(look, ndc);
        let observation = self.inspect_ray(direction);
        match sample {
            None => observation,
            Some(sample) => {
                let center_direction = observer_direction(look.yaw, look.pitch);
                let (source_center, _) =
                    self.comparative_embedding.lower_direction(center_direction);
                let (source_sample, _) = self.comparative_embedding.lower_direction(direction);
                let center_heading = source_center[1].atan2(source_center[0]).to_degrees();
                let sample_heading = source_sample[1].atan2(source_sample[0]).to_degrees();
                let mut heading_offset = sample_heading - center_heading;
                while heading_offset > 180.0 {
                    heading_offset -= 360.0;
                }
                while heading_offset < -180.0 {
                    heading_offset += 360.0;
                }
                format!(
                    "look_sample={sample},ndc=({:.6},{:.6}),client=({:.0},{:.0}),bsp-view-heading-degrees={center_heading:.3},sample-ray-heading-degrees={sample_heading:.3},sample-minus-view-heading-degrees={heading_offset:.3} meaning=classification-uses-frozen-camera-view-while-classic-source-trace-follows-sample-ray\n{observation}",
                    ndc[0], ndc[1], self.size[0], self.size[1]
                )
            }
        }
    }

    fn inspection_direction(&self, look: ObserverLook, ndc: [f32; 2]) -> Vec3 {
        viewport_inspection_direction(observer_direction(look.yaw, look.pitch), self.size, ndc)
    }

    fn resolve_scan_command(&self, command: &str) -> String {
        if !self.bsp_diagnostic_enabled {
            return "scan: requires --bsp-diagnostic-full".to_owned();
        }
        let arguments = command.split_whitespace().skip(1).collect::<Vec<_>>();
        let (columns, rows) = match arguments.as_slice() {
            [] => (DEFAULT_SCAN_COLUMNS, DEFAULT_SCAN_ROWS),
            [columns, rows] => {
                let (Ok(columns), Ok(rows)) = (columns.parse::<usize>(), rows.parse::<usize>())
                else {
                    return "scan: expected SCAN or SCAN <columns> <rows>".to_owned();
                };
                (columns, rows)
            }
            _ => return "scan: expected SCAN or SCAN <columns> <rows>".to_owned(),
        };
        if columns < 4
            || rows < 4
            || columns > 128
            || rows > 128
            || columns.saturating_mul(rows) > MAX_SCAN_SAMPLES
        {
            return format!(
                "scan: grid axes must be 4..128 and contain at most {MAX_SCAN_SAMPLES} samples"
            );
        }
        let (Some(observer), Some(look)) = (self.spawn_observer, self.observer_look) else {
            return "scan: source-spawn observer unavailable".to_owned();
        };
        let map = match self.current_doom_visibility_map() {
            Ok(map) => map,
            Err(error) => {
                return format!("scan: current runtime-height snapshot unavailable:{error}")
            }
        };
        let manifest = match observe_bsp_diagnostic_manifest(
            &map,
            &self.draws,
            &self.cutout_draws,
            observer,
            look,
            self.comparative_embedding,
            {
                let camera = scene_camera(
                    self.size,
                    self.center,
                    self.radius,
                    Some(observer),
                    Some(look),
                );
                camera.projection * camera.view
            },
        ) {
            Ok(manifest) => manifest,
            Err(error) => return format!("scan: BSP manifest unavailable:{error}"),
        };
        scan_bsp_viewport(
            observer.position,
            observer_direction(look.yaw, look.pitch),
            self.size,
            columns,
            rows,
            &self.draws,
            &self.cutout_draws,
            self.include_cutouts,
            &manifest,
        )
        .report()
    }

    fn inspect_ray(&self, direction: Vec3) -> String {
        let (Some(observer), Some(look)) = (self.spawn_observer, self.observer_look) else {
            return "look: source-spawn observer unavailable".to_owned();
        };
        let hit = nearest_prepared_ray_hit(
            observer.position,
            direction,
            &self.draws,
            self.include_cutouts.then_some(self.cutout_draws.as_slice()),
        );
        let ordinary = format_look_ray_observation(
            observer.position,
            direction,
            self.comparative_embedding,
            hit,
            nearest_sky_boundary_ray_hit(
                observer.position,
                direction,
                &self.doom_sky_boundary_draws,
            ),
            nearest_source_sky_plane_ray_hit(
                observer.position,
                direction,
                &self.diagnostic_sky_draws,
            ),
        );
        let (source_xy, source_eye_height) = self
            .comparative_embedding
            .lower_direction(observer.position);
        let (source_direction, _) = self.comparative_embedding.lower_direction(direction);
        let classic = format_source_classic_ray_trace(
            &self.door_geometry_source.map,
            source_xy,
            source_direction,
            hit,
        );
        let center_direction = observer_direction(look.yaw, look.pitch);
        let (source_center_direction, _) =
            self.comparative_embedding.lower_direction(center_direction);
        let plane_occurrence = match self.current_doom_visibility_map() {
            Ok(map) => format_source_classic_plane_span_support(
                &map,
                &self.door_geometry_source.wall_extents,
                source_xy,
                source_center_direction,
                source_eye_height,
                hit,
            ),
            Err(error) => format!("classic_plane_occurrence=unavailable:map:{error}"),
        };
        let bsp = if !self.bsp_diagnostic_enabled {
            "bsp_shadow_classification=disabled".to_owned()
        } else {
            match self.current_doom_visibility_map().and_then(|map| {
                observe_bsp_diagnostic_manifest(
                    &map,
                    &self.draws,
                    &self.cutout_draws,
                    observer,
                    look,
                    self.comparative_embedding,
                    {
                        let camera = scene_camera(
                            self.size,
                            self.center,
                            self.radius,
                            Some(observer),
                            Some(look),
                        );
                        camera.projection * camera.view
                    },
                )
            }) {
                Ok(manifest) => hit.map_or_else(
                    || "bsp_shadow_classification=no-ordinary-hit".to_owned(),
                    |hit| {
                        let classification = describe_bsp_diagnostic_hit(
                            &manifest,
                            hit.draw,
                            hit.family == "cutout",
                            &self.draws,
                            &self.cutout_draws,
                        );
                        format!(
                            "{classification},source:{}",
                            compact_draw_source(&hit.draw.source)
                        )
                    },
                ),
                Err(error) => format!("bsp_shadow_classification=unavailable:{error}"),
            }
        };
        format!("{ordinary}\n{classic}\n{plane_occurrence}\n{bsp}")
    }

    fn rebuild_debug_console(&mut self, renderer: &mut WgpuBackend) -> PlatformResult<()> {
        let font = self
            .debug_font
            .as_ref()
            .ok_or_else(|| io::Error::other("debug console font missing"))?;
        let raster = self
            .debug_console
            .rasterize(font, self.size[0].max(320.0) as u32);
        renderer.try_upload_texture(
            DEBUG_TEXTURE,
            &Texture::rgba8(raster.width, raster.height, raster.rgba8),
        )?;
        renderer.upload_material(
            DEBUG_MATERIAL,
            &Material::new("doom-debug-console", Color::rgb(1.0, 1.0, 1.0))
                .with_texture(DEBUG_TEXTURE),
        )?;
        Ok(())
    }
}

fn trace_doom_use_lines(
    map: &DoomMapCore,
    viewer: [i16; 2],
    ray: [f64; 2],
    active_ceiling_overrides: &BTreeMap<u32, i16>,
) -> DoomUseTraceResult {
    let mut intercepts = map
        .linedefs
        .iter()
        .filter_map(|linedef| {
            let start = map.vertices.get(usize::from(linedef.start_vertex))?;
            let end = map.vertices.get(usize::from(linedef.end_vertex))?;
            let distance =
                source_ray_segment_depth(viewer, ray, [start.x, start.y], [end.x, end.y])?;
            within_classic_use_range(distance).then_some((distance, linedef, start, end))
        })
        .collect::<Vec<_>>();
    intercepts.sort_by(|left, right| left.0.total_cmp(&right.0));

    for (distance, linedef, start, end) in intercepts {
        if linedef.special != 0 {
            return if source_seg_facing(viewer, [start.x, start.y], [end.x, end.y])
                == SourceSegFacing::Front
            {
                DoomUseTraceResult::Special {
                    distance,
                    linedef: linedef.source.record_index,
                }
            } else {
                DoomUseTraceResult::BackSide {
                    distance,
                    linedef: linedef.source.record_index,
                }
            };
        }
        if doom_line_open_range(map, linedef, active_ceiling_overrides) <= 0 {
            return DoomUseTraceResult::Blocked {
                distance,
                linedef: linedef.source.record_index,
            };
        }
    }
    DoomUseTraceResult::NoIntercept
}

/// Returns source specials crossed by one accepted horizontal movement in
/// movement order. This is Doom-corpus trigger evidence, not a generic
/// collision event or spatial-query API.
pub(crate) fn source_motion_special_crossings(
    vertices: &[doom_map_provider::DoomVertex],
    linedefs: &[doom_map_provider::DoomLinedef],
    from: [i16; 2],
    to: [i16; 2],
) -> Vec<doom_map_provider::DoomSourceRecord> {
    let motion = [
        f64::from(to[0]) - f64::from(from[0]),
        f64::from(to[1]) - f64::from(from[1]),
    ];
    if motion[0].abs() <= f64::EPSILON && motion[1].abs() <= f64::EPSILON {
        return Vec::new();
    }
    let cross = |left: [f64; 2], right: [f64; 2]| left[0] * right[1] - left[1] * right[0];
    let mut crossings = linedefs
        .iter()
        .filter(|linedef| matches!(linedef.special, 11 | 36 | 88))
        .filter_map(|linedef| {
            let start = vertices.get(usize::from(linedef.start_vertex))?;
            let end = vertices.get(usize::from(linedef.end_vertex))?;
            let wall = [
                f64::from(end.x) - f64::from(start.x),
                f64::from(end.y) - f64::from(start.y),
            ];
            let offset = [
                f64::from(start.x) - f64::from(from[0]),
                f64::from(start.y) - f64::from(from[1]),
            ];
            let denominator = cross(motion, wall);
            if denominator.abs() <= f64::EPSILON {
                return None;
            }
            let movement_progress = cross(offset, wall) / denominator;
            let wall_progress = cross(offset, motion) / denominator;
            (movement_progress > f64::EPSILON
                && movement_progress <= 1.0
                && (0.0..=1.0).contains(&wall_progress))
            .then_some((movement_progress, linedef.source))
        })
        .collect::<Vec<_>>();
    crossings.sort_by(|left, right| left.0.total_cmp(&right.0));
    crossings.into_iter().map(|(_, source)| source).collect()
}

pub(crate) fn within_classic_use_range(distance: f64) -> bool {
    distance.is_finite() && (0.0..=f64::from(CLASSIC_USE_RANGE)).contains(&distance)
}

fn doom_line_open_range(
    map: &DoomMapCore,
    linedef: &doom_map_provider::DoomLinedef,
    active_ceiling_overrides: &BTreeMap<u32, i16>,
) -> i16 {
    let (Some(right_index), Some(left_index)) = (linedef.right_sidedef, linedef.left_sidedef)
    else {
        return 0;
    };
    let Some(right_side) = map.sidedefs.get(usize::from(right_index)) else {
        return 0;
    };
    let Some(left_side) = map.sidedefs.get(usize::from(left_index)) else {
        return 0;
    };
    let Some(right_sector) = map.sectors.get(usize::from(right_side.sector)) else {
        return 0;
    };
    let Some(left_sector) = map.sectors.get(usize::from(left_side.sector)) else {
        return 0;
    };
    let right_ceiling = active_ceiling_overrides
        .get(&right_sector.source.record_index)
        .copied()
        .unwrap_or(right_sector.ceiling_height);
    let left_ceiling = active_ceiling_overrides
        .get(&left_sector.source.record_index)
        .copied()
        .unwrap_or(left_sector.ceiling_height);
    right_ceiling.min(left_ceiling) - right_sector.floor_height.max(left_sector.floor_height)
}

pub(crate) fn compact_activation_intent(intent: DoomLineActivationIntent) -> &'static str {
    match intent {
        DoomLineActivationIntent::RaiseDoor { .. } => "raise-door-from-interacting-side",
        DoomLineActivationIntent::ExitLevel { .. } => "exit-level",
        DoomLineActivationIntent::LowerFloorTurbo { .. } => "lower-floor-turbo",
        DoomLineActivationIntent::PlatformDownWaitUpStay { .. } => "platform-down-wait-up-stay",
    }
}

pub(crate) fn compact_activation_target(intent: DoomLineActivationIntent) -> String {
    match intent {
        DoomLineActivationIntent::RaiseDoor { target_sector } => format!(
            "sector={} lump={}",
            target_sector.record_index, target_sector.lump_index
        ),
        DoomLineActivationIntent::ExitLevel { tag }
        | DoomLineActivationIntent::LowerFloorTurbo { tag }
        | DoomLineActivationIntent::PlatformDownWaitUpStay { tag } => format!("tag={tag}"),
    }
}

pub(crate) fn compact_draw_source(source: &StaticDrawSource) -> String {
    match source {
        StaticDrawSource::Wall {
            source_linedef,
            source_sidedef,
            source_sector,
            ..
        } => format!(
            "wall linedef={} sidedef={} sector={} lumps={}/{}/{}",
            source_linedef.record_index,
            source_sidedef.record_index,
            source_sector.record_index,
            source_linedef.lump_index,
            source_sidedef.lump_index,
            source_sector.lump_index,
        ),
        StaticDrawSource::Flat {
            source_subsector,
            source_sector,
            plane,
        } => format!(
            "flat subsector={} sector={} plane={plane:?} lumps={}/{}",
            source_subsector.record_index,
            source_sector.record_index,
            source_subsector.lump_index,
            source_sector.lump_index,
        ),
    }
}

impl PlatformEventHandler for App {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window.clone(), size.width, size.height)?;
        window.set_ime_allowed(true);
        self.window = Some(window);
        for upload in &self.uploads {
            renderer.create_texture_rgba8(upload.texture, upload.descriptor, &upload.rgba8)?;
            renderer.upload_material(upload.material, &upload.material_value)?;
        }
        if self.include_cutouts {
            for upload in &self.cutout_uploads {
                renderer.create_texture_rgba8(upload.texture, upload.descriptor, &upload.rgba8)?;
                renderer.upload_material(upload.material, &upload.material_value)?;
            }
        }
        if self.bsp_diagnostic_enabled {
            upload_bsp_diagnostic_materials(&mut renderer)?;
            eprintln!(
                "E1M1 BSP shadow diagnostic enabled: focus={}; {}; membership=unchanged-global-full; classification-authority=appearance-only; focus-controls=Z-all,X-accepted,M-rejected,Q-unresolved",
                self.bsp_diagnostic_focus.label(),
                bsp_diagnostic_legend(),
            );
        }
        if self.diagnostic_sky_enabled {
            // AR-0027 Alternative A: this corpus chooses a checked-in Purple
            // PNG for retained sky omissions. It is not a Doom asset lookup,
            // a renderer fallback, or successful source resolution.
            let decoded = decode_png(
                include_bytes!("../../../../../../../assets/PNG/Purple/texture_01.png"),
                DecodeLimits::default(),
            )?;
            let prepared = prepare_renderer_texture(&decoded, TextureUse::ColorSrgb)
                .map_err(io::Error::other)?;
            renderer.create_texture_rgba8(
                DIAGNOSTIC_SKY_TEXTURE,
                tokimu::Rgba8TextureDescriptor::new(
                    prepared.texture.width,
                    prepared.texture.height,
                    tokimu::Rgba8TextureColorSpace::Srgb,
                ),
                &prepared.texture.rgba8,
            )?;
            renderer.upload_material(
                DIAGNOSTIC_SKY_MATERIAL,
                &Material::new("e1m1-diagnostic-sky-omission", Color::rgb(1.0, 1.0, 1.0))
                    .with_texture(DIAGNOSTIC_SKY_TEXTURE)
                    .with_texture_sampler(TextureSampler {
                        filter: TextureFilter::Point,
                        address_u: TextureAddressMode::Repeat,
                        address_v: TextureAddressMode::Repeat,
                    }),
            )?;
            eprintln!(
                "E1M1 AR-0027 diagnostic sky stand-in enabled: draws={}; asset=corpus/assets/PNG/Purple/texture_01.png; records={}",
                self.diagnostic_sky_draws.len(),
                self.diagnostic_sky_records.len(),
            );
            for record in self.diagnostic_sky_records.iter().take(8) {
                eprintln!("E1M1 AR-0027 diagnostic record: {record}");
            }
        }
        if self.doom_sky_enabled {
            let sky = match &self.doom_sky_texture.eligibility {
                StaticTextureEligibility::Opaque(sky) => sky,
                StaticTextureEligibility::DeferredAlpha {
                    uncovered_pixels, ..
                } => {
                    return Err(io::Error::other(format!(
                        "E1M1 SKY1 retained {uncovered_pixels} uncovered pixels; sky coverage policy remains unresolved"
                    ))
                    .into());
                }
            };
            renderer.create_texture_rgba8(
                DOOM_SKY_TEXTURE,
                sky.descriptor,
                &self.doom_sky_texture.rgba8,
            )?;
            renderer.upload_material(
                DOOM_SKY_MATERIAL,
                &Material::new("doom-e1m1-sky1-panorama", Color::rgb(1.0, 1.0, 1.0))
                    .with_texture(DOOM_SKY_TEXTURE)
                    .with_texture_sampler(TextureSampler {
                        filter: TextureFilter::Point,
                        address_u: TextureAddressMode::Repeat,
                        address_v: TextureAddressMode::Clamp,
                    }),
            )?;
            renderer.upload_mesh(DOOM_SKY_MESH, &self.doom_sky_mesh);
            renderer.upload_material(
                DOOM_SKY_BOUNDARY_MATERIAL,
                &Material::new(
                    "doom-e1m1-paired-sky-depth-boundary",
                    Color::rgb(0.0, 0.0, 0.0),
                ),
            )?;
            if self.source_sky_plane_depth_global_control {
                for (index, draw) in self.diagnostic_sky_draws.iter().enumerate() {
                    renderer.upload_mesh(
                        MeshHandle(DOOM_SOURCE_SKY_PLANE_MESH_BASE + index as u64),
                        &draw.mesh,
                    );
                }
                eprintln!(
                    "E1M1 experimental source-sky-plane depth coverage: triangles={}; policy=global-exact-retained-F_SKY1-source-flat-meshes; scope=corpus-local-falsification-control",
                    self.diagnostic_sky_draws.len(),
                );
            }
            if self.source_sky_plane_depth_enabled {
                eprintln!(
                    "E1M1 experimental viewer-relative sky depth: policy=classic-visible-F_SKY1-screen-cells-on-source-ceiling-planes; scope=corpus-local-falsification-control",
                );
            }
            let boundary_sources = self
                .doom_sky_boundary_draws
                .iter()
                .map(|draw| draw.source_linedef.record_index)
                .collect::<BTreeSet<_>>();
            eprintln!(
                "E1M1 corpus sky enabled: source=SKY1; raster={}x{}; presentation=static-panorama-cylinder; scope=corpus-local-non-equivalent-to-original-view-dependent-sky",
                sky.descriptor.width,
                sky.descriptor.height,
            );
            eprintln!(
                "E1M1 paired-sky boundary evidence retained: triangles={}; linedefs={}; presentation=disabled-after-valid-hut-geometry-was-clipped",
                self.doom_sky_boundary_draws.len(),
                boundary_sources.len(),
            );
            if let Some(sample) = self.doom_sky_boundary_draws.first() {
                eprintln!(
                    "E1M1 paired-sky boundary sample: linedef={}; sidedef={}; sector={}",
                    sample.source_linedef.record_index,
                    sample.source_sidedef.record_index,
                    sample.source_sector.record_index,
                );
            }
        }
        if self.diagnostic_sky_enabled {
            for (index, draw) in self.diagnostic_sky_draws.iter().enumerate() {
                renderer.upload_mesh(
                    MeshHandle(DIAGNOSTIC_SKY_MESH_BASE + index as u64),
                    &draw.mesh,
                );
            }
        }
        self.debug_font = Some(
            UiFontRasterizer::from_bytes(UiFontSource::from_native_default()?.bytes)
                .map_err(io::Error::other)?,
        );
        renderer.upload_mesh(DEBUG_QUAD, &Mesh::quad());
        renderer.upload_material(
            DEBUG_CURSOR_MATERIAL,
            &Material::new("doom-debug-center-cursor", Color::rgb(0.35, 0.95, 0.82)),
        )?;
        self.debug_pipeline = Some(renderer.register_pipeline(&Pipeline::new(
            "doom-debug-console",
            PipelineKind::Texture2d,
        ))?);
        if let Some(observer) = self.spawn_observer {
            eprintln!(
                "E1M1 source-spawn observer: THINGS #{} at=({}, {}) angle={} sector={} floor={} ceiling={} eye=({:.1}, {:.1}, {:.1}) forward=({:.3}, {:.3}, {:.3})",
                observer.source_record,
                observer.source_position[0],
                observer.source_position[1],
                observer.source_angle,
                observer.sector,
                observer.floor,
                observer.ceiling,
                observer.position.x,
                observer.position.y,
                observer.position.z,
                observer.forward.x,
                observer.forward.y,
                observer.forward.z,
            );
        }
        if let Some(collision) = &self.walk_collision {
            eprintln!(
                "E1M1 Slice 6 walk proof: radius={WALK_RADIUS}; walk-speed={WALK_SPEED}; run-speed={}; blocking_linedefs={}; broad_phase=source-blockmap-with-full-wall-fallback; noclip={}; controls=WASD-move-shift-run-E-use-click-capture-escape-release-R-reset-noclip-space-up-left-control-down",
                WALK_SPEED * RUN_SPEED_MULTIPLIER,
                collision.blocking_wall_count(),

                self.noclip,
            );
        }
        self.pipeline = renderer.register_pipeline(
            &Pipeline::new("doom-e1m1-static-opaque", PipelineKind::Textured3d).with_render_state(
                PipelineRenderState {
                    blend: BlendMode::Opaque,
                    depth_test: DepthTest::LessEqual,
                    depth_write: true,
                    cull_mode: CullMode::Back,
                    color_write: ColorWriteMask::ALL,
                },
            )?,
        )?;
        if self.doom_sky_enabled {
            self.doom_sky_pipeline = Some(
                renderer.register_pipeline(
                    &Pipeline::new("doom-e1m1-sky-panorama", PipelineKind::Textured3d)
                        .with_render_state(PipelineRenderState {
                            blend: BlendMode::Opaque,
                            depth_test: DepthTest::LessEqual,
                            depth_write: false,
                            cull_mode: CullMode::None,
                            color_write: ColorWriteMask::ALL,
                        })?,
                )?,
            );
            self.doom_sky_boundary_pipeline = Some(
                renderer.register_pipeline(
                    &Pipeline::new(
                        "doom-e1m1-paired-sky-boundary-depth",
                        PipelineKind::LitColor3d,
                    )
                    .with_render_state(PipelineRenderState {
                        blend: BlendMode::Opaque,
                        depth_test: DepthTest::LessEqual,
                        depth_write: true,
                        // The retained triangle winding faces the higher
                        // source sector's owning sidedef. Sky-boundary depth
                        // therefore has authority only from that source side;
                        // making it double-sided hides legitimate geometry
                        // (including the hut) when viewed through the same
                        // boundary from the opposite sector.
                        cull_mode: CullMode::Back,
                        color_write: ColorWriteMask::NONE,
                    })?,
                )?,
            );
        }
        if self.candidate1_sky_depth_enabled {
            renderer.upload_material(
                CANDIDATE1_SKY_DEPTH_MATERIAL,
                &Material::new(
                    "e1m1-candidate1-authoritative-sky-depth",
                    Color::rgb(0.0, 0.0, 0.0),
                ),
            )?;
            renderer.upload_camera(CANDIDATE1_CLIP_CAMERA, Camera::default());
            self.candidate1_sky_depth_pipeline = Some(
                renderer.register_pipeline(
                    &Pipeline::new(
                        "e1m1-candidate1-authoritative-sky-depth",
                        PipelineKind::SolidColor2d,
                    )
                    .with_render_state(PipelineRenderState {
                        blend: BlendMode::Opaque,
                        depth_test: DepthTest::LessEqual,
                        depth_write: true,
                        cull_mode: CullMode::None,
                        color_write: ColorWriteMask::NONE,
                    })?,
                )?,
            );
        }
        if self.include_cutouts {
            self.cutout_pipeline =
                Some(renderer.register_pipeline(&Pipeline::textured_3d_cutout(
                    "doom-e1m1-masked-cutout",
                    CategoricalCutout::new(
                        CutoutThreshold::new(0.0)?,
                        CutoutComparison::DiscardAtOrBelow,
                    ),
                ))?);
        }
        self.upload_static_meshes(&mut renderer);
        eprintln!(
            "E1M1 native first-frame metadata: strategy={}; stages={}; topology_inventory_records={}; topology_inventory_hash={:016x}; opaque_draws={}; cutout_draws={}; cutouts_enabled={}; bsp_diagnostic={}; bsp_focus={}; camera={}; candidate_selection={}; walk_collision={}; noclip={}; backend={}; device={}; adapter={}",
            self.render_strategy_name,
            self.render_strategy_stages,
            self.topology_inventory.records.len(),
            self.topology_inventory.aggregate_hash,
            self.draws.len(),
            self.cutout_draws.len(),
            self.include_cutouts,
            self.bsp_diagnostic_enabled,
            self.bsp_diagnostic_focus.label(),
            if self.spawn_observer.is_some() { "source-spawn-observer" } else { "overview" },
            candidate_selection_label(
                self.candidate_selection,
                self.ordered_coverage_prepared,
            ),
            self.walk_collision.is_some(),
            self.noclip,
            renderer.backend_api(),
            renderer.device_kind(),
            renderer.adapter_name(),
        );
        self.renderer = Some(renderer);
        Ok(())
    }
    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::KeyboardInput {
            key: KeyCode::Backquote,
            pressed: true,
        } = event
        {
            self.toggle_debug_console();
            return Ok(());
        }
        if self.debug_console.is_open() {
            match event {
                PlatformInputEvent::TextInput(text) => self.debug_console.insert_text(&text),
                PlatformInputEvent::KeyboardInput {
                    key: KeyCode::Enter,
                    pressed: true,
                } => self.submit_debug_console(),
                PlatformInputEvent::KeyboardInput {
                    key: KeyCode::Backspace,
                    pressed: true,
                } => self.debug_console.backspace(),
                PlatformInputEvent::KeyboardInput {
                    key: KeyCode::Escape,
                    pressed: true,
                } => self.toggle_debug_console(),
                PlatformInputEvent::Resized { width, height } => {
                    self.size = [width.max(1) as f32, height.max(1) as f32];
                    self.debug_console.invalidate();
                    if let Some(renderer) = self.renderer.as_mut() {
                        renderer.resize_surface(width, height);
                    }
                }
                _ => {}
            }
            return Ok(());
        }
        if let Some(input_event) = event.as_input_event() {
            if !self.fixed_reconstruction_camera {
                self.input.apply_event(input_event);
            }
        }
        if let PlatformInputEvent::MouseMotion { delta_x, delta_y } = event {
            if self.mouse_captured && !self.fixed_reconstruction_camera {
                if let Some(look) = self.observer_look.as_mut() {
                    apply_look_delta(look, delta_x, delta_y);
                }
            }
            return Ok(());
        }
        if let PlatformInputEvent::MouseInput {
            button: MouseButton::Left,
            pressed: true,
        } = event
        {
            if !self.fixed_reconstruction_camera {
                self.set_mouse_captured(true);
            }
            return Ok(());
        }
        if let PlatformInputEvent::KeyboardInput { key, pressed } = event {
            if key == KeyCode::Escape && pressed {
                self.set_mouse_captured(false);
                self.release_walk_keys();
            } else if pressed
                && self.bsp_diagnostic_enabled
                && matches!(
                    key,
                    KeyCode::KeyZ | KeyCode::KeyX | KeyCode::KeyM | KeyCode::KeyQ
                )
            {
                match key {
                    KeyCode::KeyZ => self.set_bsp_diagnostic_focus(BspDiagnosticFocus::All),
                    KeyCode::KeyX => self.set_bsp_diagnostic_focus(BspDiagnosticFocus::Accepted),
                    KeyCode::KeyM => self.set_bsp_diagnostic_focus(BspDiagnosticFocus::Rejected),
                    KeyCode::KeyQ => self.set_bsp_diagnostic_focus(BspDiagnosticFocus::Unresolved),
                    _ => {}
                }
            } else if key == KeyCode::KeyR && pressed {
                self.reset_spawn_observer();
            } else if key == KeyCode::KeyE && pressed {
                let outcome = self.try_use_center_wall();
                eprintln!("E1M1 {outcome}");
                self.debug_console.append(outcome);
            }
            return Ok(());
        }
        if let PlatformInputEvent::CursorMoved { x, y } = event {
            if let Some(look) = self.observer_look.as_mut() {
                look.last_cursor = Some([x, y]);
            }
            return Ok(());
        }
        if let PlatformInputEvent::Resized { width, height } = event {
            self.size = [width.max(1) as f32, height.max(1) as f32];
            self.debug_console.invalidate();
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize_surface(width, height);
            }
        }
        Ok(())
    }

    fn on_frame(&mut self, delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        if !self.fixed_reconstruction_camera {
            self.apply_inspection_movement(delta_seconds);
        }
        self.advance_active_manual_doors(delta_seconds);
        self.advance_active_moving_floors(delta_seconds);
        self.refresh_ordered_coverage_for_observer()?;
        let frame_started = Instant::now();
        let mut camera = scene_camera(
            self.size,
            self.center,
            self.radius,
            self.spawn_observer,
            self.observer_look,
        );
        if self.fixed_reconstruction_camera {
            let aspect = self.size[0] / self.size[1].max(1.0);
            camera.projection = tokimu_core::math::try_projection_perspective_rh_gl(
                (2.0 * classic_presentation_half_vertical_fov()) as f32,
                aspect,
                (self.radius * 0.000_1).max(0.1),
                self.radius * 4.0,
            )
            .expect("classic presentation projection must remain finite and ordered");
        }
        let selection_started = Instant::now();
        let view_projection = camera.projection * camera.view;
        let candidate1_sky_depth_batch = self.prepare_candidate1_sky_depth_batch()?;
        let source_sky_span_depth_mesh = if self.source_sky_plane_depth_enabled {
            let (mesh, triangles) = prepare_viewer_relative_source_sky_span_mesh(
                &self.door_geometry_source,
                self.spawn_observer
                    .expect("source sky plane selection requires an observer"),
                self.observer_look
                    .expect("source sky plane selection requires observer look"),
                self.comparative_embedding,
            )?;
            if self.frame_index == 0 {
                eprintln!(
                    "E1M1 viewer-relative source-sky depth: reconstructed_triangles={triangles}; policy=classic-visible-F_SKY1-screen-cells-on-source-ceiling-planes; scope=corpus-falsification-control-not-pixel-parity",
                );
            }
            mesh
        } else if self.source_sky_plane_depth_global_control {
            self.source_sky_plane_selected.fill(true);
            None
        } else {
            None
        };
        let mut selection = CandidateSelectionSummary::default();
        let mut rejection_samples = Vec::new();
        if self.candidate_selection == CandidateSelection::DoomMembershipUnion {
            select_membership_candidates(
                &self.draws,
                view_projection,
                &self.membership_selection,
                &mut self.opaque_selected,
                &mut selection,
                &mut rejection_samples,
                self.frame_index == 0,
            );
        } else if self.candidate_selection == CandidateSelection::DoomSegPerColumn {
            select_seg_per_column_candidates(
                &self.draws,
                self.doom_seg_dynamic_selection
                    .as_ref()
                    .expect("dynamic SEG selection has retained source input"),
                &self.door_geometry_source.map,
                self.spawn_observer
                    .expect("dynamic SEG selection requires an observer"),
                self.observer_look
                    .expect("dynamic SEG selection requires observer look"),
                self.comparative_embedding,
                &mut self.opaque_selected,
                &mut selection,
                &mut rejection_samples,
                self.frame_index == 0,
            )?;
        } else if self.candidate_selection == CandidateSelection::DoomClassicBsp {
            let visibility_map = self.current_doom_visibility_map()?;
            select_seg_classic_bsp_candidates(
                &self.draws,
                self.doom_seg_dynamic_selection
                    .as_ref()
                    .expect("classic BSP selection has retained source input"),
                &visibility_map,
                self.spawn_observer
                    .expect("classic BSP selection requires an observer"),
                self.observer_look
                    .expect("classic BSP selection requires observer look"),
                self.comparative_embedding,
                &mut self.opaque_selected,
                &mut selection,
                &mut rejection_samples,
                self.frame_index == 0,
            )?;
        } else {
            select_current_candidates(
                self.candidate_selection,
                self.opaque_grid.as_ref(),
                &self.opaque_bounds,
                &self.draws,
                view_projection,
                &mut self.opaque_selected,
                &mut selection,
                &mut rejection_samples,
                self.frame_index == 0,
            );
        }
        let opaque_submitted = selection.submitted;
        if self.include_cutouts {
            if self.candidate_selection == CandidateSelection::DoomMembershipUnion {
                select_membership_candidates(
                    &self.cutout_draws,
                    view_projection,
                    &self.membership_selection,
                    &mut self.cutout_selected,
                    &mut selection,
                    &mut rejection_samples,
                    self.frame_index == 0,
                );
            } else if matches!(
                self.candidate_selection,
                CandidateSelection::DoomSegPerColumn | CandidateSelection::DoomClassicBsp
            ) {
                self.cutout_selected.fill(true);
                selection.candidates += self.cutout_selected.len();
                selection.submitted += self.cutout_selected.len();
            } else {
                select_current_candidates(
                    self.candidate_selection,
                    self.cutout_grid.as_ref(),
                    &self.cutout_bounds,
                    &self.cutout_draws,
                    view_projection,
                    &mut self.cutout_selected,
                    &mut selection,
                    &mut rejection_samples,
                    self.frame_index == 0,
                );
            }
            if let Some(observer) = self.spawn_observer.filter(|_| !self.bsp_diagnostic_enabled) {
                select_masked_middle_owning_sides(
                    &self.cutout_draws,
                    observer.position,
                    &mut self.cutout_selected,
                    &mut selection,
                    &mut rejection_samples,
                    self.frame_index == 0,
                );
            }
        }
        let cutout_submitted = selection.submitted - opaque_submitted;
        let bsp_diagnostic = if self.bsp_diagnostic_enabled {
            let visibility_map = self.current_doom_visibility_map()?;
            let manifest = observe_bsp_diagnostic_manifest(
                &visibility_map,
                &self.draws,
                &self.cutout_draws,
                self.spawn_observer
                    .expect("BSP diagnostic requires a source observer"),
                self.observer_look
                    .expect("BSP diagnostic requires observer look"),
                self.comparative_embedding,
                view_projection,
            )?;
            if self.frame_index == 0 {
                eprintln!(
                    "E1M1 BSP shadow diagnostic manifest: focus={}; {}; original-submitted={}; renderer-removals=0; reasons=retained-per-draw",
                    self.bsp_diagnostic_focus.label(),
                    manifest.report(),
                    self.draws.len() + self.cutout_draws.len(),
                );
            }
            Some(manifest)
        } else {
            None
        };
        let selection_time = selection_started.elapsed();
        let command_started = Instant::now();
        self.commands.clear();
        self.commands.push(RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.015, 0.02, 0.025),
        }));
        if self.doom_sky_enabled {
            let pipeline = self
                .doom_sky_pipeline
                .ok_or_else(|| io::Error::other("Doom sky pipeline missing"))?;
            let sky_draw = DrawMeshCommand {
                mesh: DOOM_SKY_MESH,
                material: DOOM_SKY_MATERIAL,
                pipeline,
                instance: Instance2d::identity(),
                camera: Some(CAMERA),
                viewport: None,
            };
            if self.bsp_diagnostic_enabled {
                self.commands.push(bsp_diagnostic_command(
                    sky_draw,
                    BspDiagnosticDraw {
                        family: BspDiagnosticFamily::Skybox,
                        disposition: BspDiagnosticDisposition::UnresolvedFailOpen,
                        reason: BspDiagnosticReason::PresentationGlobal,
                    },
                    self.bsp_diagnostic_focus,
                )?);
            } else {
                self.commands.push(RenderCommand::DrawMesh(sky_draw));
            }
            let boundary_pipeline = self
                .doom_sky_boundary_pipeline
                .ok_or_else(|| io::Error::other("Doom sky boundary pipeline missing"))?;
            if source_sky_span_depth_mesh.is_some() {
                self.commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: DOOM_VIEWER_SKY_SPAN_MESH,
                    material: DOOM_SKY_BOUNDARY_MATERIAL,
                    pipeline: boundary_pipeline,
                    instance: Instance2d::identity(),
                    camera: Some(CAMERA),
                    viewport: None,
                }));
            }
            if self.source_sky_plane_depth_global_control {
                for (index, selected) in self.source_sky_plane_selected.iter().enumerate() {
                    if !selected {
                        continue;
                    }
                    self.commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                        mesh: MeshHandle(DOOM_SOURCE_SKY_PLANE_MESH_BASE + index as u64),
                        material: DOOM_SKY_BOUNDARY_MATERIAL,

                        pipeline: boundary_pipeline,
                        instance: Instance2d::identity(),
                        camera: Some(CAMERA),
                        viewport: None,
                    }));
                }
            }
        }
        // Candidate 1 is deliberately inserted after the sky panorama and
        // before every ordinary world declaration. The Doom consumer owns
        // the authoritative coverage; tokimu-render sees only G2 clip-space
        // geometry with depth-only render state.
        let candidate1_insertion_index = self.commands.len();
        for (index, draw) in self.draws.iter().enumerate() {
            if !self.opaque_selected[index] || !self.opaque_draw_enabled[index] {
                continue;
            }
            let mesh = self
                .dynamic_door_mesh_handles
                .get(&index)
                .copied()
                .unwrap_or(MeshHandle(index as u64 + 1));
            let draw_command = DrawMeshCommand {
                mesh,
                material: draw.material,
                pipeline: self.pipeline,
                instance: Instance2d::identity(),
                camera: Some(CAMERA),
                viewport: None,
            };
            if let Some(diagnostic) = bsp_diagnostic
                .as_ref()
                .map(|manifest| manifest.opaque[index])
            {
                self.commands.push(bsp_diagnostic_command(
                    draw_command,
                    diagnostic,
                    self.bsp_diagnostic_focus,
                )?);
            } else {
                self.commands.push(RenderCommand::DrawMesh(draw_command));
            }
        }
        if self.include_cutouts {
            let cutout_pipeline = self
                .cutout_pipeline
                .ok_or_else(|| io::Error::other("masked-cutout pipeline missing"))?;
            for (offset, draw) in self.cutout_draws.iter().enumerate() {
                if !self.cutout_selected[offset] {
                    continue;
                }
                let mesh = MeshHandle(self.cutout_mesh_base + offset as u64);
                let draw_command = DrawMeshCommand {
                    mesh,
                    material: draw.material,
                    pipeline: cutout_pipeline,
                    instance: Instance2d::identity(),
                    camera: Some(CAMERA),
                    viewport: None,
                };
                if let Some(diagnostic) = bsp_diagnostic
                    .as_ref()
                    .map(|manifest| manifest.cutouts[offset])
                {
                    self.commands.push(bsp_diagnostic_command(
                        draw_command,
                        diagnostic,
                        self.bsp_diagnostic_focus,
                    )?);
                } else {
                    self.commands.push(RenderCommand::DrawMesh(draw_command));
                }
            }
        }
        if self.diagnostic_sky_enabled {
            for (index, _) in self.diagnostic_sky_draws.iter().enumerate() {
                self.commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: MeshHandle(DIAGNOSTIC_SKY_MESH_BASE + index as u64),
                    material: DIAGNOSTIC_SKY_MATERIAL,
                    pipeline: self.pipeline,
                    instance: Instance2d::identity(),
                    camera: Some(CAMERA),
                    viewport: None,
                }));
            }
        }
        if self.debug_console.is_open() {
            let debug_pipeline = self
                .debug_pipeline
                .ok_or_else(|| io::Error::other("debug console pipeline missing"))?;
            self.commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: DEBUG_QUAD,
                material: DEBUG_MATERIAL,
                pipeline: debug_pipeline,
                instance: Instance2d::new(
                    [0.0, 0.72],
                    [(self.size[0] / self.size[1]).max(1.0) * 2.0, 0.56],
                    0.0,
                ),
                camera: Some(DEBUG_CAMERA),
                viewport: None,
            }));
        } else if let Some(debug_pipeline) = self.debug_pipeline {
            for size in [[0.032, 0.003], [0.003, 0.048]] {
                self.commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: DEBUG_QUAD,
                    material: DEBUG_CURSOR_MATERIAL,
                    pipeline: debug_pipeline,
                    instance: Instance2d::new([0.0, 0.0], size, 0.0),
                    camera: Some(DEBUG_CAMERA),
                    viewport: None,
                }));
            }
        }
        let command_time = command_started.elapsed();
        if self.debug_console.is_open() && self.debug_console.take_dirty() {
            let mut renderer = self
                .renderer
                .take()
                .ok_or_else(|| io::Error::other("renderer missing"))?;
            let rebuilt = self.rebuild_debug_console(&mut renderer);
            self.renderer = Some(renderer);
            rebuilt?;
        }
        let dynamic_mesh_uploads = std::mem::take(&mut self.dirty_opaque_meshes)
            .into_iter()
            .map(|index| {
                (
                    self.dynamic_door_mesh_handles
                        .get(&index)
                        .copied()
                        .unwrap_or(MeshHandle(index as u64 + 1)),
                    self.draws[index].mesh.clone(),
                )
            })
            .collect::<Vec<_>>();
        let renderer = self
            .renderer
            .as_mut()
            .ok_or_else(|| io::Error::other("renderer missing"))?;
        for (handle, mesh) in dynamic_mesh_uploads {
            renderer.upload_mesh(handle, &mesh);
        }
        if let Some(mesh) = source_sky_span_depth_mesh {
            renderer.upload_mesh(DOOM_VIEWER_SKY_SPAN_MESH, &mesh);
        }
        renderer.upload_camera(CAMERA, camera);
        if self.debug_pipeline.is_some() {
            renderer.upload_camera(
                DEBUG_CAMERA,
                Camera::orthographic_2d(self.size[0], self.size[1]),
            );
        }
        renderer.begin_frame();
        renderer.submit(&self.commands[..candidate1_insertion_index]);
        let candidate1_observation = candidate1_sky_depth_batch
            .as_ref()
            .map(|candidate| {
                renderer
                    .submit_experimental_submission_local_geometry(&candidate.batch)
                    .map_err(io::Error::other)
            })
            .transpose()?;
        renderer.submit(&self.commands[candidate1_insertion_index..]);
        renderer.present()?;
        let stats = renderer.end_frame();
        if self.candidate1_sky_depth_enabled && self.frame_index < 2 {
            if let Some(candidate) = candidate1_sky_depth_batch.as_ref() {
                eprintln!(
                    "E1M1 AR-0030 Candidate 1 {} frame: source-regions={}; declarations={}; vertices={}; triangles={}; submission-scoped-fingerprint={}; renderer-observation={candidate1_observation:?}; persistent-mesh-identity=none; fallback=not-taken",
                    if self.frame_index == 0 { "first" } else { "warm" },
                    candidate.source_regions,
                    candidate.declarations,
                    candidate.vertices,
                    candidate.triangles,
                    candidate.structural_fingerprint,
                );
            } else {
                eprintln!(
                    "E1M1 AR-0030 Candidate 1 {} frame: local-batch=omitted; fallback=global-full-submission; partial-authority=forbidden",
                    if self.frame_index == 0 { "first" } else { "warm" },
                );
            }
        }
        if self.frame_index < 2 {
            eprintln!(
                "E1M1 AR-0025 {} frame: strategy={}; stages={}; selection={:?}; candidates={}; rejected={}; submitted={}; opaque_submitted={}; cutout_submitted={}; uncertain_bounds={}; rejected_by_plane=[left:{},right:{},bottom:{},top:{},near:{},far:{}]; selection_cpu_us={}; command_build_cpu_us={}; frame_cpu_us={}; draws={}; material_resolutions={}; pipeline_switches={}; mesh_uploads={}; mesh_replacements={}; lifetime_mesh_uploads={}; lifetime_mesh_replacements={}",
                if self.frame_index == 0 { "first" } else { "warm" },
                self.render_strategy_name,
                self.render_strategy_stages,
                self.candidate_selection,
                selection.candidates,
                selection.rejected,

                selection.submitted,
                opaque_submitted,
                cutout_submitted,
                selection.uncertain_bounds,
                selection.rejected_by_plane[0],
                selection.rejected_by_plane[1],
                selection.rejected_by_plane[2],
                selection.rejected_by_plane[3],
                selection.rejected_by_plane[4],
                selection.rejected_by_plane[5],
                selection_time.as_micros(),
                command_time.as_micros(),
                frame_started.elapsed().as_micros(),
                stats.frame.draw_calls,
                stats.frame.material_resolutions,
                stats.frame.pipeline_switches,
                stats.frame.mesh_uploads,
                stats.frame.mesh_replacements,

                stats.lifetime.mesh_uploads,
                stats.lifetime.mesh_replacements,
            );
            if self.frame_index == 0 && !rejection_samples.is_empty() {
                eprintln!(
                    "E1M1 AR-0025 bounded rejection samples ({} of {}): {}",
                    rejection_samples.len(),
                    selection.rejected,
                    rejection_samples.join(" | "),
                );
            }
        }
        self.frame_index = self.frame_index.saturating_add(1);
        Ok(if self.exit_after_two_frames && self.frame_index >= 2 {
            FrameOutcome::Exit
        } else {
            FrameOutcome::Continue
        })
    }
}

/// Applies Doom's sidedef ownership after ordinary camera candidate selection.
///
/// A two-sided middle texture belongs to the sidedef that names it. Classic
/// Doom therefore presents it only while viewing that owning face; the reverse
/// SEG cannot borrow the other side's middle texture. The renderer's generic
/// cutout pipeline is deliberately two-sided, so this source rule remains in
/// the Doom consumer rather than becoming alpha or renderer policy.
impl App {
    /// Static corpus geometry crosses the provider boundary once at startup.
    /// Camera motion changes only the uploaded camera and submitted draws;
    /// re-uploading 1,861 immutable meshes per frame would be avoidable
    /// steady-state allocation and buffer replacement.
    fn upload_static_meshes(&self, renderer: &mut WgpuBackend) {
        for (index, draw) in self.draws.iter().enumerate() {
            renderer.upload_mesh(MeshHandle(index as u64 + 1), &draw.mesh);
        }
        if self.include_cutouts {
            for (offset, draw) in self.cutout_draws.iter().enumerate() {
                renderer.upload_mesh(
                    MeshHandle(self.cutout_mesh_base + offset as u64),
                    &draw.mesh,
                );
            }
        }
    }
}
