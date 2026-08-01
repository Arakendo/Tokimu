use std::sync::Arc;

use hello_runtime_observation::{
    CommandAuthority, CommandResult, ObservationLimits, RuntimeCommand, RuntimeInspectionAdapter,
};
use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, KeyCode, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use ui_tools::{
    layout_bitmap_text, UiFrameLayout, UiHorizontalSplitLayout, UiInsets, UiRect, UiTextAlign,
    UiTextOverflow, UiTextRole, UiTextSpec,
};

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

    fn draw_rect(&mut self, renderer: &mut WgpuBackend, rect: UiRect, material: MaterialHandle) {
        renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: QUAD,
            material,
            pipeline: self.pipeline,
            instance: Instance2d::new(rect.center, rect.size, 0.0),
            camera: Some(CAMERA),
            viewport: None,
        })]);
    }

    fn draw_text(
        &mut self,
        renderer: &mut WgpuBackend,
        text: &str,
        rect: UiRect,
        role: UiTextRole,
        height: f32,
    ) {
        let spec = UiTextSpec::new(text, rect, role)
            .with_alignment(UiTextAlign::Start, UiTextAlign::Center)
            .with_overflow(UiTextOverflow::Ellipsis);
        let commands = layout_bitmap_text(&spec, height)
            .into_iter()
            .map(|quad| {
                RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: QUAD,
                    material: if matches!(role, UiTextRole::Title | UiTextRole::Heading) {
                        TEXT
                    } else {
                        MUTED
                    },
                    pipeline: self.pipeline,
                    instance: Instance2d::new(quad.center, quad.size, 0.0),
                    camera: Some(CAMERA),
                    viewport: None,
                })
            })
            .collect::<Vec<_>>();
        renderer.submit(&commands);
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
                        ) => {
                            format!(
                                "POSITION: {:.2}, {:.2}, {:.2}",
                                position.x, position.y, position.z
                            )
                        }
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

        // The inspector is intentionally arranged by observation concern, not
        // by the source runtime's internal objects or storage layout.
        let frame =
            UiFrameLayout::for_window(self.window_size, UiInsets::uniform(0.08), 0.15, 0.24, 0.035);
        let panes = UiHorizontalSplitLayout::new(frame.body, 0.5, 0.05, 0.85, 0.85);
        let footer = UiHorizontalSplitLayout::new(frame.footer.inset(0.05), 0.55, 0.05, 0.75, 0.60);

        self.draw_rect(&mut renderer, frame.content, PANEL);
        self.draw_rect(&mut renderer, frame.header, ACCENT);
        self.draw_rect(&mut renderer, panes.leading, BACKDROP);
        self.draw_rect(&mut renderer, panes.trailing, BACKDROP);
        self.draw_rect(&mut renderer, frame.footer, BACKDROP);
        self.draw_rect(
            &mut renderer,
            UiRect::new(
                [
                    frame.footer.center[0],
                    frame.footer.center[1] + frame.footer.size[1] * 0.5 - 0.018,
                ],
                [frame.footer.size[0] - 0.10, 0.008],
            ),
            ACCENT,
        );

        self.draw_text(
            &mut renderer,
            "RUNTIME OBSERVATION INSPECTOR",
            frame.header.inset(0.05),
            UiTextRole::Title,
            0.042,
        );
        self.draw_text(
            &mut renderer,
            if panes.fits_minimums {
                "LAYOUT: FIT"
            } else {
                "LAYOUT: NARROW"
            },
            UiRect::new(
                [
                    frame.header.center[0] + frame.header.size[0] * 0.5 - 0.38,
                    frame.header.center[1],
                ],
                [0.28, frame.header.size[1] - 0.06],
            ),
            UiTextRole::Caption,
            0.020,
        );
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
        let world_content = panes.leading.inset(0.05);
        let world_left = world_content.center[0] - world_content.size[0] * 0.5;
        let world_top = world_content.center[1] + world_content.size[1] * 0.5;
        for (index, line) in world_lines.iter().enumerate() {
            self.draw_text(
                &mut renderer,
                line,
                UiRect::new(
                    [
                        world_left + world_content.size[0] * 0.5,
                        world_top - 0.045 - index as f32 * 0.105,
                    ],
                    [world_content.size[0], 0.065],
                ),
                if index == 0 {
                    UiTextRole::Heading
                } else {
                    UiTextRole::Body
                },
                if index == 0 { 0.030 } else { 0.023 },
            );
        }

        let presentation_lines = [
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
        let presentation_content = panes.trailing.inset(0.05);
        let presentation_top = presentation_content.center[1] + presentation_content.size[1] * 0.5;
        for (index, line) in presentation_lines.iter().enumerate() {
            self.draw_text(
                &mut renderer,
                line,
                UiRect::new(
                    [
                        presentation_content.center[0],
                        presentation_top - 0.045 - index as f32 * 0.115,
                    ],
                    [presentation_content.size[0], 0.070],
                ),
                if index == 0 {
                    UiTextRole::Heading
                } else {
                    UiTextRole::Body
                },
                if index == 0 { 0.030 } else { 0.024 },
            );
        }

        let diagnostics = if observation.payload.diagnostics.is_empty() {
            "DIAGNOSTICS: NONE".to_owned()
        } else {
            format!("DIAGNOSTICS: {}", observation.payload.diagnostics[0].code)
        };
        self.draw_text(
            &mut renderer,
            "COMMANDS",
            UiRect::new(
                [
                    footer.leading.center[0],
                    footer.leading.center[1] + footer.leading.size[1] * 0.5 - 0.025,
                ],
                [footer.leading.size[0], 0.035],
            ),
            UiTextRole::Heading,
            0.026,
        );
        self.draw_text(
            &mut renderer,
            "LEFT/RIGHT SELECT   D MOVE   E DISABLE   X REJECT",
            UiRect::new(
                [footer.leading.center[0], footer.leading.center[1] - 0.005],
                [footer.leading.size[0], 0.035],
            ),
            UiTextRole::Caption,
            0.017,
        );
        self.draw_text(
            &mut renderer,
            "SPACE APPLY   R TARGET   A CLIP   S PLAY",
            UiRect::new(
                [footer.leading.center[0], footer.leading.center[1] - 0.055],
                [footer.leading.size[0], 0.035],
            ),
            UiTextRole::Caption,
            0.017,
        );
        self.draw_text(
            &mut renderer,
            &diagnostics,
            UiRect::new(
                [
                    footer.trailing.center[0],
                    footer.trailing.center[1] + footer.trailing.size[1] * 0.5 - 0.045,
                ],
                [footer.trailing.size[0], 0.045],
            ),
            UiTextRole::Caption,
            0.021,
        );
        self.draw_text(
            &mut renderer,
            &format!("LAST COMMAND: {result}"),
            UiRect::new(
                [footer.trailing.center[0], footer.trailing.center[1] - 0.035],
                [footer.trailing.size[0], 0.045],
            ),
            UiTextRole::Caption,
            0.021,
        );

        let _ = renderer.present()?;
        self.renderer = Some(renderer);
        self.update_title();
        Ok(FrameOutcome::Continue)
    }
}
