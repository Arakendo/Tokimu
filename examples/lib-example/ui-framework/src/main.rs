use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, MouseButton, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use ui_tools::{
    layout_bitmap_text, window_to_world, UiButtonId, UiButtonSpec, UiControlRole, UiDrawer, UiRect,
    UiSurfaceCommand, UiSurfaceRole, UiTextRole, UiTheme, UiWorkspaceLayout,
};

const PANEL_MESH: MeshHandle = MeshHandle(1);
const BUTTON_MESH: MeshHandle = MeshHandle(2);
const STATUS_MESH: MeshHandle = MeshHandle(3);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const BACKDROP_MATERIAL: MaterialHandle = MaterialHandle(1);
const SHELL_MATERIAL: MaterialHandle = MaterialHandle(2);
const PANEL_MATERIAL: MaterialHandle = MaterialHandle(3);
const BUTTON_MATERIAL: MaterialHandle = MaterialHandle(4);
const BUTTON_HOVER_MATERIAL: MaterialHandle = MaterialHandle(5);
const ACTIVE_MATERIAL: MaterialHandle = MaterialHandle(6);
const MUTED_MATERIAL: MaterialHandle = MaterialHandle(7);
const TEXT_MATERIAL: MaterialHandle = MaterialHandle(9);
const TEXT_SOFT_MATERIAL: MaterialHandle = MaterialHandle(10);

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu UI Framework Example".into(),
            width: 1280,
            height: 780,
        },
        UiFrameworkApp::new(),
    )
}

struct UiFrameworkApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    theme: UiTheme,
    cached_static_surfaces: Vec<UiSurfaceCommand>,
    cached_button_surfaces: Vec<UiSurfaceCommand>,
    cached_button_text: Vec<RenderCommand>,
    cached_window_size: [f32; 2],
    cached_hovered_button: Option<UiButtonId>,
    cached_active_button: Option<UiButtonId>,
    cursor_position: [f32; 2],
    hovered_button: Option<UiButtonId>,
    active_button: Option<UiButtonId>,
}

impl Default for UiFrameworkApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            theme: UiTheme::default(),
            cached_static_surfaces: Vec::new(),
            cached_button_surfaces: Vec::new(),
            cached_button_text: Vec::new(),
            cached_window_size: [0.0, 0.0],
            cached_hovered_button: None,
            cached_active_button: None,
            cursor_position: [0.0, 0.0],
            hovered_button: None,
            active_button: None,
        }
    }
}

impl UiFrameworkApp {
    fn new() -> Self {
        Self::default()
    }

    fn layout(&self) -> UiWorkspaceLayout {
        UiWorkspaceLayout::new(
            self.window_size,
            [
                UiButtonSpec::new(UiButtonId(0), "BROWSE"),
                UiButtonSpec::new(UiButtonId(1), "Button"),
                UiButtonSpec::new(UiButtonId(2), "RESET"),
            ],
            [
                ui_tools::UiCardSpec::new(
                    ui_tools::UiCardRole::Browser,
                    "Button",
                    "SELECT / DESELECT",
                ),
                ui_tools::UiCardSpec::new(
                    ui_tools::UiCardRole::Browser,
                    "Button",
                    "SELECT / DESELECT",
                ),
                ui_tools::UiCardSpec::new(
                    ui_tools::UiCardRole::Browser,
                    "Button",
                    "SELECT / DESELECT",
                ),
            ],
        )
    }

    fn cursor_world(&self) -> [f32; 2] {
        window_to_world(self.window_size, self.cursor_position)
    }

    fn update_hover(&mut self) {
        let layout = self.layout();
        let button = &layout.buttons[1];
        self.hovered_button = button.contains(self.cursor_world()).then_some(button.id);
    }

    fn button_label(&self, button: UiButtonId) -> &'static str {
        self.layout().button_label(button)
    }

    fn rebuild_cache(&mut self) {
        if self.cached_window_size == self.window_size
            && self.cached_hovered_button == self.hovered_button
            && self.cached_active_button == self.active_button
        {
            return;
        }

        let layout = self.layout();
        self.cached_static_surfaces.clear();
        self.cached_button_surfaces.clear();
        self.cached_button_text.clear();

        let button = &layout.buttons[1];
        let control_role = UiControlRole::Primary;
        let mut surfaces = Vec::new();
        let mut text_commands = Vec::new();
        let state = if Some(button.id) == self.active_button {
            ui_tools::UiInteractionState::Selected
        } else if Some(button.id) == self.hovered_button {
            ui_tools::UiInteractionState::Hovered
        } else {
            ui_tools::UiInteractionState::Idle
        };
        {
            let mut drawer = UiDrawer::new(&mut surfaces, &mut text_commands, &self.theme);
            drawer.button_strip(button, state, control_role);
        }

        self.cached_button_surfaces.extend(surfaces.into_iter());
        self.cached_button_text
            .extend(text_commands.into_iter().flat_map(|command| {
                Self::build_text_commands(
                    &command.spec,
                    self.theme.text(command.style.role).height,
                    Self::material_for_text_role(command.style.role),
                    self.pipeline,
                )
            }));

        self.cached_window_size = self.window_size;
        self.cached_hovered_button = self.hovered_button;
        self.cached_active_button = self.active_button;
    }

    fn material_for_role(role: UiSurfaceRole) -> MaterialHandle {
        match role {
            UiSurfaceRole::Background => BACKDROP_MATERIAL,
            UiSurfaceRole::Region => PANEL_MATERIAL,
            UiSurfaceRole::Panel => BUTTON_MATERIAL,
            UiSurfaceRole::Card => SHELL_MATERIAL,
            UiSurfaceRole::Toolbar => BUTTON_MATERIAL,
            UiSurfaceRole::Raised => SHELL_MATERIAL,
            UiSurfaceRole::Selected => ACTIVE_MATERIAL,
            UiSurfaceRole::Accent => BUTTON_HOVER_MATERIAL,
            UiSurfaceRole::Overlay => MUTED_MATERIAL,
        }
    }

    fn material_for_text_role(role: UiTextRole) -> MaterialHandle {
        match role {
            UiTextRole::Title | UiTextRole::Heading | UiTextRole::Button => TEXT_MATERIAL,
            UiTextRole::Body | UiTextRole::Caption | UiTextRole::Chip | UiTextRole::Status => {
                TEXT_SOFT_MATERIAL
            }
        }
    }

    fn draw_surface_command(
        renderer: &mut WgpuBackend,
        pipeline: PipelineHandle,
        command: &UiSurfaceCommand,
    ) {
        let rect = command.rect;
        if matches!(
            command.style.elevation,
            ui_tools::UiElevation::Raised | ui_tools::UiElevation::Floating
        ) {
            let shadow_rect = UiRect::new(
                [rect.center[0] + 0.014, rect.center[1] - 0.014],
                [rect.size[0], rect.size[1]],
            );
            renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: PANEL_MESH,
                material: MUTED_MATERIAL,
                pipeline,
                instance: Instance2d::new(shadow_rect.center, shadow_rect.size, 0.0),
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            })]);
        }

        renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: PANEL_MESH,
            material: Self::material_for_role(command.style.role),
            pipeline,
            instance: Instance2d::new(rect.center, rect.size, 0.0),
            camera: Some(CAMERA_HANDLE),
            viewport: None,
        })]);

        if let Some(border_role) = command.style.border_role {
            let border = command
                .style
                .border_width
                .min(rect.size[0] * 0.22)
                .min(rect.size[1] * 0.22);
            if border > 0.0 {
                let border_material = Self::material_for_role(border_role);
                let top = UiRect::new(
                    [
                        rect.center[0],
                        rect.center[1] + rect.size[1] * 0.5 - border * 0.5,
                    ],
                    [rect.size[0], border],
                );

                renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: PANEL_MESH,
                    material: border_material,
                    pipeline,
                    instance: Instance2d::new(top.center, top.size, 0.0),
                    camera: Some(CAMERA_HANDLE),
                    viewport: None,
                })]);
            }
        }
    }

    fn build_text_commands(
        spec: &ui_tools::UiTextSpec,
        height: f32,
        material: MaterialHandle,
        pipeline: PipelineHandle,
    ) -> Vec<RenderCommand> {
        layout_bitmap_text(spec, height)
            .into_iter()
            .map(|quad| {
                RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: BUTTON_MESH,
                    material,
                    pipeline,
                    instance: Instance2d::new(quad.center, quad.size, 0.0),
                    camera: Some(CAMERA_HANDLE),
                    viewport: None,
                })
            })
            .collect()
    }

    fn update_window_title(&self) {
        if let Some(window) = self.window.as_ref() {
            let active = self
                .active_button
                .map(|button| self.button_label(button))
                .unwrap_or("none");
            let hover = self
                .hovered_button
                .map(|button| self.button_label(button))
                .unwrap_or("none");
            window.set_title(&format!(
                "Tokimu UI Framework Example | 1 button | active={} | hover={}",
                active, hover,
            ));
        }
    }

    fn render_scene(&mut self) -> PlatformResult<FrameOutcome> {
        self.rebuild_cache();
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        renderer.upload_camera(
            CAMERA_HANDLE,
            Camera::orthographic_2d(self.window_size[0], self.window_size[1]),
        );
        renderer.begin_frame();

        renderer.submit(&[RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.05, 0.06, 0.09),
        })]);

        for command in self.cached_button_surfaces.iter() {
            Self::draw_surface_command(renderer, self.pipeline, command);
        }

        for command in self.cached_button_text.iter() {
            renderer.submit(&[*command]);
        }

        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }

    fn select_button_at_cursor(&mut self) {
        let layout = self.layout();
        let button = &layout.buttons[1];
        if button.contains(self.cursor_world()) {
            self.active_button = if self.active_button == Some(button.id) {
                None
            } else {
                Some(button.id)
            };
        } else {
            self.active_button = None;
        }
    }
}

impl PlatformEventHandler for UiFrameworkApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(PANEL_MESH, &Mesh::quad());
        renderer.upload_mesh(BUTTON_MESH, &Mesh::quad());
        renderer.upload_mesh(STATUS_MESH, &Mesh::quad());
        renderer.upload_material(
            BACKDROP_MATERIAL,
            &Material::new("ui-backdrop", Color::rgb(0.08, 0.09, 0.12)),
        )?;
        renderer.upload_material(
            SHELL_MATERIAL,
            &Material::new("ui-shell", Color::rgb(0.13, 0.15, 0.19)),
        )?;
        renderer.upload_material(
            PANEL_MATERIAL,
            &Material::new("ui-panel", Color::rgb(0.17, 0.19, 0.25)),
        )?;
        renderer.upload_material(
            BUTTON_MATERIAL,
            &Material::new("ui-button", Color::rgb(0.23, 0.27, 0.33)),
        )?;
        renderer.upload_material(
            BUTTON_HOVER_MATERIAL,
            &Material::new("ui-button-hover", Color::rgb(0.31, 0.35, 0.43)),
        )?;
        renderer.upload_material(
            ACTIVE_MATERIAL,
            &Material::new("ui-button-active", Color::rgb(0.34, 0.56, 0.86)),
        )?;
        renderer.upload_material(
            MUTED_MATERIAL,
            &Material::new("ui-muted", Color::rgb(0.14, 0.16, 0.20)),
        )?;
        renderer.upload_material(
            TEXT_MATERIAL,
            &Material::new("ui-text", Color::rgb(0.93, 0.95, 0.98)),
        )?;
        renderer.upload_material(
            TEXT_SOFT_MATERIAL,
            &Material::new("ui-text-soft", Color::rgb(0.72, 0.77, 0.84)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "ui-framework-pipeline",
            PipelineKind::SolidColor2d,
        ))?;
        self.renderer = Some(renderer);
        self.update_hover();
        self.update_window_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::CloseRequested = event {
            return Ok(());
        }

        if let PlatformInputEvent::CursorMoved { x, y } = event {
            self.cursor_position = [x, y];
            self.update_hover();
            self.update_window_title();
        }

        if let PlatformInputEvent::MouseInput {
            button: MouseButton::Left,
            pressed: true,
        } = event
        {
            self.select_button_at_cursor();
            self.update_hover();
            self.update_window_title();
        }

        if let PlatformInputEvent::KeyboardInput { key, pressed: true } = event {
            if matches!(key, tokimu::KeyCode::Escape) {
                self.active_button = None;
                self.update_window_title();
            }
        }

        if let PlatformInputEvent::Resized { width, height } = event {
            self.window_size = [width.max(1) as f32, height.max(1) as f32];
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize_surface(width, height);
            }
            self.update_hover();
        }

        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        self.render_scene()
    }
}
