use std::sync::Arc;

use hello_runtime_observation::{
    compare_observation_snapshots, CommandAuthority, CommandResult, ObservationComparisonConfig,
    ObservationEnvelope, ObservationLimits, RuntimeCommand, RuntimeInspectionAdapter,
};
use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, KeyCode, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use ui_tools::consumer::UiTextRole;
use ui_tools::{
    layout_bitmap_text, lower_resolved_tree_to_draw_list, UiDrawCommand, UiDrawList, UiSurfaceRole,
    UiTheme,
};

mod ui;

const QUAD: MeshHandle = MeshHandle(1);
const CAMERA: CameraHandle = CameraHandle(1);
const BACKDROP: MaterialHandle = MaterialHandle(1);
const PANEL: MaterialHandle = MaterialHandle(2);
const ACCENT: MaterialHandle = MaterialHandle(3);
const TEXT: MaterialHandle = MaterialHandle(4);
const MUTED: MaterialHandle = MaterialHandle(5);

fn main() -> PlatformResult<()> {
    let app = RuntimeInspectorApp::new().map_err(std::io::Error::other)?;
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello Runtime Inspector".into(),
            width: 1200,
            height: 760,
        },
        app,
    )
}

struct RuntimeInspectorApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    runtime: RuntimeInspectionAdapter,
    playback_accumulator_seconds: f64,
    selected_entity: u64,
    sequence: u64,
    next_command_id: u64,
    last_result: Option<CommandResult>,
    previous_observation: Option<ObservationEnvelope>,
    presentation_status: &'static str,
}

impl RuntimeInspectorApp {
    fn new() -> Result<Self, String> {
        let runtime = RuntimeInspectionAdapter::new(8)?;
        let selected_entity = runtime.arm_id().0;
        Ok(Self {
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            runtime,
            playback_accumulator_seconds: 0.0,
            selected_entity,
            sequence: 0,
            next_command_id: 1,
            last_result: None,
            previous_observation: None,
            presentation_status: "UNSELECTED",
        })
    }

    fn observation(&mut self) -> hello_runtime_observation::ObservationEnvelope {
        let observation = self.runtime.observe(
            self.sequence,
            Some(tokimu::EntityId(self.selected_entity)),
            ObservationLimits::default(),
        );
        self.sequence = self.sequence.saturating_add(1);
        observation
    }

    fn queue_move(&mut self) {
        self.last_result = Some(
            self.runtime
                .enqueue(hello_runtime_observation::CommandRequest {
                    id: self.next_command_id,
                    target: self.selected_entity,
                    authority: CommandAuthority::Operator,
                    expected_revision: Some(self.runtime.revision()),
                    command: RuntimeCommand::MoveBy {
                        delta: hello_runtime_observation::Position {
                            x: 0.25,
                            y: 0.0,
                            z: 0.0,
                        },
                    },
                }),
        );
        self.next_command_id = self.next_command_id.saturating_add(1);
    }

    fn queue_disable(&mut self) {
        self.last_result = Some(
            self.runtime
                .enqueue(hello_runtime_observation::CommandRequest {
                    id: self.next_command_id,
                    target: self.selected_entity,
                    authority: CommandAuthority::Operator,
                    expected_revision: Some(self.runtime.revision()),
                    command: RuntimeCommand::SetEnabled { enabled: false },
                }),
        );
        self.next_command_id = self.next_command_id.saturating_add(1);
    }

    fn queue_unknown_target(&mut self) {
        self.last_result = Some(
            self.runtime
                .enqueue(hello_runtime_observation::CommandRequest {
                    id: self.next_command_id,
                    // Deliberately outside the scenario's bounded entity set.
                    target: 99,
                    authority: CommandAuthority::Operator,
                    expected_revision: Some(self.runtime.revision()),
                    command: RuntimeCommand::SetEnabled { enabled: true },
                }),
        );
        self.next_command_id = self.next_command_id.saturating_add(1);
    }

    fn apply_queue(&mut self) {
        let trace = self
            .runtime
            .apply_pending_at_tick(self.runtime.tick().saturating_add(1));
        self.last_result = trace.results.last().cloned();
    }

    fn select_presentation_target(&mut self) {
        let result = self.runtime.select_arm_presentation();
        self.presentation_status = match result.disposition {
            hello_runtime_observation::PresentationCommandDisposition::Accepted => "SELECTED",
            hello_runtime_observation::PresentationCommandDisposition::RejectedUnknownTarget => {
                "UNRESOLVED"
            }
        };
    }

    fn next_clip(&mut self) {
        let _ = self.runtime.next_animation_step();
    }

    fn play_selected_clip(&mut self) {
        let _ = self.runtime.play_selected_animation();
    }

    fn update_title(&self) {
        if let Some(window) = self.window.as_ref() {
            window.set_title(&format!(
                "Tokimu Runtime Inspector | entity={} | revision={} | presentation={}",
                self.selected_entity,
                self.runtime.revision(),
                self.presentation_status
            ));
        }
    }

    fn submit_draw_list(&mut self, renderer: &mut WgpuBackend, draw_list: &UiDrawList) {
        let mut commands = Vec::new();
        for entry in draw_list.entries() {
            match &entry.command {
                // The native inspector does not currently use clipping. Keep
                // clip commands explicit so a later renderer adapter can add
                // scissor support without changing the consumer's draw list.
                UiDrawCommand::PushClip(_) | UiDrawCommand::PopClip => {}
                UiDrawCommand::Surface(surface) => {
                    commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                        mesh: QUAD,
                        material: material_for_surface(surface.style.role),
                        pipeline: self.pipeline,
                        instance: Instance2d::new(surface.rect.center, surface.rect.size, 0.0),
                        camera: Some(CAMERA),
                        viewport: None,
                    }))
                }
                UiDrawCommand::Text(text) => {
                    commands.extend(
                        layout_bitmap_text(&text.spec, text.style.height)
                            .into_iter()
                            .map(|quad| {
                                RenderCommand::DrawMesh(DrawMeshCommand {
                                    mesh: QUAD,
                                    material: if matches!(
                                        text.style.role,
                                        UiTextRole::Title | UiTextRole::Heading
                                    ) {
                                        TEXT
                                    } else {
                                        MUTED
                                    },
                                    pipeline: self.pipeline,
                                    instance: Instance2d::new(quad.center, quad.size, 0.0),
                                    camera: Some(CAMERA),
                                    viewport: None,
                                })
                            }),
                    );
                }
            }
        }
        renderer.submit(&commands);
    }
}

fn material_for_surface(role: UiSurfaceRole) -> MaterialHandle {
    match role {
        UiSurfaceRole::Panel | UiSurfaceRole::Raised => PANEL,
        UiSurfaceRole::Accent | UiSurfaceRole::Selected => ACCENT,
        UiSurfaceRole::Background
        | UiSurfaceRole::Region
        | UiSurfaceRole::Card
        | UiSurfaceRole::Toolbar
        | UiSurfaceRole::Overlay => BACKDROP,
    }
}

impl PlatformEventHandler for RuntimeInspectorApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(QUAD, &Mesh::quad());
        for (handle, name, color) in [
            (
                BACKDROP,
                "runtime-inspector-backdrop",
                Color::rgb(0.04, 0.05, 0.07),
            ),
            (
                PANEL,
                "runtime-inspector-panel",
                Color::rgb(0.15, 0.19, 0.24),
            ),
            (
                ACCENT,
                "runtime-inspector-accent",
                Color::rgb(0.22, 0.55, 0.72),
            ),
            (TEXT, "runtime-inspector-text", Color::rgb(0.90, 0.94, 0.98)),
            (
                MUTED,
                "runtime-inspector-muted",
                Color::rgb(0.65, 0.72, 0.80),
            ),
        ] {
            renderer.upload_material(handle, &Material::new(name, color))?;
        }
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "hello-runtime-inspector-pipeline",
            PipelineKind::SolidColor2d,
        ))?;
        self.renderer = Some(renderer);
        self.update_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        match event {
            PlatformInputEvent::KeyboardInput { key, pressed: true } => match key {
                KeyCode::ArrowLeft => self.selected_entity = self.runtime.root_id().0,
                KeyCode::ArrowRight => self.selected_entity = self.runtime.arm_id().0,
                KeyCode::KeyD => self.queue_move(),
                KeyCode::KeyE => self.queue_disable(),
                KeyCode::KeyX => self.queue_unknown_target(),
                KeyCode::Space => self.apply_queue(),
                KeyCode::KeyR => self.select_presentation_target(),
                KeyCode::KeyA => self.next_clip(),
                KeyCode::KeyS => self.play_selected_clip(),
                _ => {}
            },
            PlatformInputEvent::Resized { width, height } => {
                self.window_size = [width.max(1) as f32, height.max(1) as f32];
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize_surface(width, height);
                }
            }
            _ => {}
        }
        self.update_title();
        Ok(())
    }

    fn on_frame(&mut self, delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        self.playback_accumulator_seconds += delta_seconds;
        while self.playback_accumulator_seconds >= 1.0 / 60.0 {
            self.runtime.advance_animation_fixed_step();
            self.playback_accumulator_seconds -= 1.0 / 60.0;
        }

        let observation = self.observation();
        let snapshot_diff = self
            .previous_observation
            .as_ref()
            .map(|previous| {
                compare_observation_snapshots(
                    previous,
                    &observation,
                    &ObservationComparisonConfig::default(),
                )
                .map(|report| {
                    if report.payload.equal {
                        format!("SNAPSHOT: UNCHANGED @ REVISION {}", report.after.revision)
                    } else {
                        format!(
                            "SNAPSHOT: {} CHANGES / R{} -> R{}",
                            report.payload.differences.len(),
                            report.before.revision,
                            report.after.revision
                        )
                    }
                })
                .unwrap_or_else(|error| format!("SNAPSHOT: INCOMPATIBLE ({error})"))
            })
            .unwrap_or_else(|| "SNAPSHOT: INITIAL".to_owned());
        self.previous_observation = Some(observation.clone());
        let selected = observation.payload.selected.as_ref();
        let component_count = selected.map(|detail| detail.components.len()).unwrap_or(0);
        let relationship_count = selected
            .map(|detail| detail.relationships.len())
            .unwrap_or(0);
        let relationship_edge_count = selected
            .map(|detail| {
                detail
                    .relationships
                    .iter()
                    .map(|relationship| relationship.edges.len())
                    .sum::<usize>()
            })
            .unwrap_or(0);
        let component_lines = selected
            .map(|detail| {
                detail
                    .components
                    .iter()
                    .map(|component| match component {
                        hello_runtime_observation::ComponentValueObservation::Position(
                            position,
                        ) => format!(
                            "POSITION: {:.2}, {:.2}, {:.2}",
                            position.x, position.y, position.z
                        ),
                        hello_runtime_observation::ComponentValueObservation::Enabled(enabled) => {
                            format!("ENABLED: {enabled}")
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec!["SELECTED DETAIL: UNAVAILABLE".to_owned()]);

        let presentation_target_count = self.runtime.presentation().targets.len();
        let playback = self.runtime.playback().clone();
        let active_clip_name = self
            .runtime
            .animation_catalog()
            .get(playback.selected_clip)
            .map(|clip| clip.name.clone())
            .unwrap_or_else(|| "UNAVAILABLE".to_owned());
        let result = self
            .last_result
            .as_ref()
            .map(|result| format!("{:?}", result.disposition))
            .unwrap_or_else(|| "NONE".to_owned());

        let mut world_lines = vec![
            "WORLD OBSERVATION".to_owned(),
            format!("SELECTED ENTITY: {}", self.selected_entity),
            format!(
                "REVISION: {}    TICK: {}",
                observation.revision, observation.tick
            ),
            format!("ENTITIES: {}", observation.payload.entity_count),
            format!(
                "RELATION TYPES: {}",
                observation.payload.relationship_types.len()
            ),
            format!("DETAIL: {component_count} COMPONENTS / {relationship_count} RELATIONS"),
            format!("OUTGOING EDGES: {relationship_edge_count}"),
        ];
        world_lines.extend(component_lines);

        let presentation_lines = vec![
            "PRESENTATION + PLAYBACK".to_owned(),
            format!("PRESENTATION: {}", self.presentation_status),
            format!("RESOLVED TARGETS: {presentation_target_count}"),
            format!(
                "CLIP: {} ({}/{})",
                active_clip_name,
                playback.selected_clip.saturating_add(1),
                self.runtime.animation_catalog().len(),
            ),
            format!("MODE: {:?}", playback.mode),
            format!("LOCAL TIME: {:.2} S", playback.local_time_seconds),
            format!("CATALOG: {} CLIPS", self.runtime.animation_catalog().len()),
        ];
        let diagnostics = if observation.payload.diagnostics.is_empty() {
            "DIAGNOSTICS: NONE".to_owned()
        } else {
            format!("DIAGNOSTICS: {}", observation.payload.diagnostics[0].code)
        };
        let view = ui::RuntimeInspectorView {
            world_lines,
            presentation_lines,
            command_lines: vec![
                "COMMANDS".to_owned(),
                "LEFT/RIGHT SELECT   D MOVE   E DISABLE   X REJECT".to_owned(),
                "SPACE APPLY   R TARGET   A CLIP   S PLAY".to_owned(),
            ],
            diagnostic_lines: vec![
                diagnostics,
                snapshot_diff,
                format!("LAST COMMAND: {result}"),
            ],
        };
        let scene = ui::build_runtime_inspector_scene(self.window_size, &view);
        let resolved = scene.tree.resolve(scene.viewport).map_err(|error| {
            std::io::Error::other(format!("inspector layout failed: {error:?}"))
        })?;
        let draw_list =
            lower_resolved_tree_to_draw_list(&resolved, &UiTheme::default(), observation.revision);

        let Some(mut renderer) = self.renderer.take() else {
            return Ok(FrameOutcome::Continue);
        };
        renderer.upload_camera(
            CAMERA,
            Camera::orthographic_2d(self.window_size[0], self.window_size[1]),
        );
        renderer.begin_frame();
        renderer.submit(&[RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.04, 0.05, 0.07),
        })]);
        self.submit_draw_list(&mut renderer, &draw_list);

        let _ = renderer.present()?;
        self.renderer = Some(renderer);
        self.update_title();
        Ok(FrameOutcome::Continue)
    }
}
