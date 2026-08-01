use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, KeyCode, Material, MaterialHandle, Mesh, MeshHandle, MouseButton, NativeWindow,
    Pipeline, PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent,
    PlatformResult, RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use tokimu_input::InputState;
use ui_tools::consumer::{
    UiActivationKey, UiFocusDirection, UiNodeId, UiPointerEvent, UiPointerPhase, UiPointerRouter,
    UiResolvedFocus, UiResolvedTree, UiTextInputEvent, UiTextInputOperation, UiTextInputRouter,
    UiTextRole,
};
use ui_tools::{
    layout_bitmap_text, lower_resolved_tree_to_draw_list, UiDrawCommand, UiDrawList, UiSurfaceRole,
    UiTheme,
};

mod model;
mod ui;

use model::SettingsModel;

const QUAD: MeshHandle = MeshHandle(1);
const CAMERA: CameraHandle = CameraHandle(1);
const BACKDROP: MaterialHandle = MaterialHandle(1);
const PANEL: MaterialHandle = MaterialHandle(2);
const ACCENT: MaterialHandle = MaterialHandle(3);
const TEXT: MaterialHandle = MaterialHandle(4);
const MUTED: MaterialHandle = MaterialHandle(5);

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu UI Settings Workbench".into(),
            width: 1100,
            height: 760,
        },
        SettingsWorkbenchApp::default(),
    )
}

struct SettingsWorkbenchApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    input: InputState,
    model: SettingsModel,
    focus: UiResolvedFocus,
    pointer: UiPointerRouter,
    revision: u64,
}

impl Default for SettingsWorkbenchApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            window_size: [1100.0, 760.0],
            pipeline: PipelineHandle(0),
            input: InputState::default(),
            model: SettingsModel::default(),
            focus: UiResolvedFocus::default(),
            pointer: UiPointerRouter::default(),
            revision: 0,
        }
    }
}

impl SettingsWorkbenchApp {
    fn resolved_tree(&self) -> Result<UiResolvedTree, String> {
        let scene = ui::build_settings_scene(
            self.window_size,
            &self.model,
            self.focus.focused(),
            self.pointer.hover(),
        );
        scene
            .tree
            .resolve(scene.viewport)
            .map_err(|error| format!("settings layout failed: {error:?}"))
    }

    fn cursor_world(&self, tree: &UiResolvedTree) -> [f32; 2] {
        let width = self.window_size[0].max(1.0);
        let height = self.window_size[1].max(1.0);
        let left = tree.viewport.center[0] - tree.viewport.size[0] * 0.5;
        let top = tree.viewport.center[1] + tree.viewport.size[1] * 0.5;
        [
            left + (self.input.mouse.x / width) * tree.viewport.size[0],
            top - (self.input.mouse.y / height) * tree.viewport.size[1],
        ]
    }

    fn route_pointer(&mut self, phase: UiPointerPhase) -> Result<Option<UiNodeId>, String> {
        let tree = self.resolved_tree()?;
        let resolution = self
            .pointer
            .route(&tree, UiPointerEvent::new(self.cursor_world(&tree), phase));
        if matches!(phase, UiPointerPhase::Press) {
            self.focus.set_focus(&tree, resolution.target);
        }
        Ok(resolution.activated)
    }

    fn route_text_operation(&mut self, operation: UiTextInputOperation) -> Result<bool, String> {
        let tree = self.resolved_tree()?;
        let resolution =
            UiTextInputRouter.route(&tree, &mut self.focus, UiTextInputEvent::new(operation));
        Ok(resolution
            .target
            .is_some_and(|target| self.model.apply_edit(target, resolution.operation)))
    }

    fn apply_text_operation(&mut self, operation: UiTextInputOperation) -> Result<(), String> {
        if self.route_text_operation(operation)? {
            self.revision = self.revision.saturating_add(1);
        }
        Ok(())
    }

    fn activate(&mut self, target: Option<UiNodeId>) {
        if target.is_some_and(|id| self.model.activate(id)) {
            self.revision = self.revision.saturating_add(1);
        }
    }

    fn update_title(&self) {
        if let Some(window) = self.window.as_ref() {
            window.set_title(&format!(
                "Tokimu UI Settings Workbench | dirty={} | focus={:?}",
                self.model.is_dirty(),
                self.focus.focused()
            ));
        }
    }

    fn submit_draw_list(
        renderer: &mut WgpuBackend,
        pipeline: PipelineHandle,
        draw_list: &UiDrawList,
    ) {
        let mut commands = Vec::new();
        for entry in draw_list.entries() {
            match &entry.command {
                UiDrawCommand::PushClip(_) | UiDrawCommand::PopClip => {}
                UiDrawCommand::Surface(surface) => {
                    commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                        mesh: QUAD,
                        material: material_for_surface(surface.style.role),
                        pipeline,
                        instance: Instance2d::new(surface.rect.center, surface.rect.size, 0.0),
                        camera: Some(CAMERA),
                        viewport: None,
                    }));
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
                                    pipeline,
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

impl PlatformEventHandler for SettingsWorkbenchApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        window.set_ime_allowed(true);
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(QUAD, &Mesh::quad());
        for (handle, name, color) in [
            (BACKDROP, "settings-backdrop", Color::rgb(0.04, 0.05, 0.07)),
            (PANEL, "settings-panel", Color::rgb(0.15, 0.19, 0.24)),
            (ACCENT, "settings-accent", Color::rgb(0.22, 0.55, 0.72)),
            (TEXT, "settings-text", Color::rgb(0.90, 0.94, 0.98)),
            (MUTED, "settings-muted", Color::rgb(0.65, 0.72, 0.80)),
        ] {
            renderer.upload_material(handle, &Material::new(name, color))?;
        }
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "ui-settings-workbench-pipeline",
            PipelineKind::SolidColor2d,
        ))?;
        self.renderer = Some(renderer);
        self.route_pointer(UiPointerPhase::Move)
            .map_err(std::io::Error::other)?;
        self.update_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let Some(input_event) = event.as_input_event() {
            self.input.apply_event(input_event);
        }

        match event {
            PlatformInputEvent::TextInput(text) => {
                for character in text.chars() {
                    if self
                        .route_text_operation(UiTextInputOperation::Insert(character))
                        .map_err(std::io::Error::other)?
                    {
                        self.revision = self.revision.saturating_add(1);
                    }
                }
            }
            PlatformInputEvent::CursorMoved { .. } => {
                self.route_pointer(UiPointerPhase::Move)
                    .map_err(std::io::Error::other)?;
            }
            PlatformInputEvent::MouseInput {
                button: MouseButton::Left,
                pressed: true,
            } => {
                self.route_pointer(UiPointerPhase::Press)
                    .map_err(std::io::Error::other)?;
            }
            PlatformInputEvent::MouseInput {
                button: MouseButton::Left,
                pressed: false,
            } => {
                let activated = self
                    .route_pointer(UiPointerPhase::Release)
                    .map_err(std::io::Error::other)?;
                self.activate(activated);
            }
            PlatformInputEvent::KeyboardInput { key, pressed: true } => {
                let tree = self.resolved_tree().map_err(std::io::Error::other)?;
                match key {
                    KeyCode::ArrowUp => {
                        self.focus.move_focus(&tree, UiFocusDirection::Backward);
                    }
                    KeyCode::ArrowDown => {
                        self.focus.move_focus(&tree, UiFocusDirection::Forward);
                    }
                    KeyCode::ArrowLeft => {
                        self.apply_text_operation(UiTextInputOperation::MoveLeft)
                            .map_err(std::io::Error::other)?;
                    }
                    KeyCode::ArrowRight => {
                        self.apply_text_operation(UiTextInputOperation::MoveRight)
                            .map_err(std::io::Error::other)?;
                    }
                    KeyCode::Backspace => {
                        self.apply_text_operation(UiTextInputOperation::DeleteBackward)
                            .map_err(std::io::Error::other)?;
                    }
                    KeyCode::Delete => {
                        self.apply_text_operation(UiTextInputOperation::DeleteForward)
                            .map_err(std::io::Error::other)?;
                    }
                    KeyCode::Enter | KeyCode::Space => {
                        let target = self.focus.activate(
                            &tree,
                            if key == KeyCode::Enter {
                                UiActivationKey::Enter
                            } else {
                                UiActivationKey::Space
                            },
                        );
                        self.activate(target);
                    }
                    _ => {}
                }
            }
            PlatformInputEvent::Resized { width, height } => {
                self.window_size = [width.max(1) as f32, height.max(1) as f32];
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize_surface(width, height);
                }
                let tree = self.resolved_tree().map_err(std::io::Error::other)?;
                self.focus.reconcile(&tree);
                self.route_pointer(UiPointerPhase::Move)
                    .map_err(std::io::Error::other)?;
            }
            _ => {}
        }
        self.update_title();
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let resolved = self.resolved_tree().map_err(std::io::Error::other)?;
        let draw_list =
            lower_resolved_tree_to_draw_list(&resolved, &UiTheme::default(), self.revision);
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
        Self::submit_draw_list(&mut renderer, self.pipeline, &draw_list);
        let _ = renderer.present()?;
        self.renderer = Some(renderer);
        Ok(FrameOutcome::Continue)
    }
}
