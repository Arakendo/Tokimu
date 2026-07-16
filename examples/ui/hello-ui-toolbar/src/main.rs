use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, MouseButton, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use ui_tools::{window_to_world, UiButton, UiButtonId, UiControlRole, UiInteractionState, UiRect, UiTheme};

const PANEL_MESH: MeshHandle = MeshHandle(1);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const BACKDROP_MATERIAL: MaterialHandle = MaterialHandle(1);
const SHELL_MATERIAL: MaterialHandle = MaterialHandle(2);
const PANEL_MATERIAL: MaterialHandle = MaterialHandle(3);
const BUTTON_MATERIAL: MaterialHandle = MaterialHandle(4);
const BUTTON_HOVER_MATERIAL: MaterialHandle = MaterialHandle(5);
const BUTTON_SELECTED_MATERIAL: MaterialHandle = MaterialHandle(6);
const MUTED_MATERIAL: MaterialHandle = MaterialHandle(7);

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello UI Toolbar".into(),
            width: 1240,
            height: 720,
        },
        HelloUiToolbarApp::new(),
    )
}

struct HelloUiToolbarApp {
    renderer: Option<WgpuBackend>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    theme: UiTheme,
    hovered_button: Option<UiButtonId>,
    active_button: Option<UiButtonId>,
    cursor_position: [f32; 2],
    buttons: [UiButton; 3],
}

impl Default for HelloUiToolbarApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            theme: UiTheme::default(),
            hovered_button: None,
            active_button: None,
            cursor_position: [0.0, 0.0],
            buttons: [
                UiButton::new(UiButtonId(0), "", UiRect::new([-0.3, 0.55], [0.28, 0.11])),
                UiButton::new(UiButtonId(1), "", UiRect::new([0.0, 0.55], [0.28, 0.11])),
                UiButton::new(UiButtonId(2), "", UiRect::new([0.3, 0.55], [0.28, 0.11])),
            ],
        }
    }
}

impl HelloUiToolbarApp {
    fn new() -> Self {
        Self::default()
    }

    fn cursor_world(&self) -> [f32; 2] {
        window_to_world(self.window_size, self.cursor_position)
    }

    fn material_for_role(role: ui_tools::UiSurfaceRole) -> MaterialHandle {
        match role {
            ui_tools::UiSurfaceRole::Background => BACKDROP_MATERIAL,
            ui_tools::UiSurfaceRole::Region => PANEL_MATERIAL,
            ui_tools::UiSurfaceRole::Panel => BUTTON_MATERIAL,
            ui_tools::UiSurfaceRole::Card => SHELL_MATERIAL,
            ui_tools::UiSurfaceRole::Toolbar => PANEL_MATERIAL,
            ui_tools::UiSurfaceRole::Raised => SHELL_MATERIAL,
            ui_tools::UiSurfaceRole::Selected => BUTTON_SELECTED_MATERIAL,
            ui_tools::UiSurfaceRole::Accent => BUTTON_HOVER_MATERIAL,
            ui_tools::UiSurfaceRole::Overlay => MUTED_MATERIAL,
        }
    }

    fn draw_button(
        renderer: &mut WgpuBackend,
        pipeline: PipelineHandle,
        theme: &UiTheme,
        button: &UiButton,
        state: UiInteractionState,
        role: UiControlRole,
    ) {
        let style = theme.control(role, state);
        let rect = button.rect;
        if matches!(
            style.elevation,
            ui_tools::UiElevation::Raised | ui_tools::UiElevation::Floating
        ) {
            let shadow_rect = UiRect::new(
                [rect.center[0] + 0.012, rect.center[1] - 0.012],
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
            material: Self::material_for_role(style.role),
            pipeline,
            instance: Instance2d::new(rect.center, rect.size, 0.0),
            camera: Some(CAMERA_HANDLE),
            viewport: None,
        })]);

        if let Some(border_role) = style.border_role {
            let border = style.border_width.min(rect.size[0] * 0.22).min(rect.size[1] * 0.22);
            if border > 0.0 {
                let border_rect = UiRect::new(
                    [rect.center[0], rect.center[1] + rect.size[1] * 0.5 - border * 0.5],
                    [rect.size[0], border],
                );
                renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: PANEL_MESH,
                    material: Self::material_for_role(border_role),
                    pipeline,
                    instance: Instance2d::new(border_rect.center, border_rect.size, 0.0),
                    camera: Some(CAMERA_HANDLE),
                    viewport: None,
                })]);
            }
        }
    }
}

impl PlatformEventHandler for HelloUiToolbarApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(PANEL_MESH, &Mesh::quad());
        renderer.upload_material(
            BACKDROP_MATERIAL,
            &Material::new("ui-backdrop", Color::rgb(0.05, 0.06, 0.08)),
        )?;
        renderer.upload_material(
            SHELL_MATERIAL,
            &Material::new("ui-shell", Color::rgb(0.14, 0.16, 0.20)),
        )?;
        renderer.upload_material(
            PANEL_MATERIAL,
            &Material::new("ui-panel", Color::rgb(0.20, 0.22, 0.28)),
        )?;
        renderer.upload_material(
            BUTTON_MATERIAL,
            &Material::new("ui-button", Color::rgb(0.24, 0.26, 0.32)),
        )?;
        renderer.upload_material(
            BUTTON_HOVER_MATERIAL,
            &Material::new("ui-button-hover", Color::rgb(0.30, 0.34, 0.42)),
        )?;
        renderer.upload_material(
            BUTTON_SELECTED_MATERIAL,
            &Material::new("ui-button-selected", Color::rgb(0.34, 0.56, 0.86)),
        )?;
        renderer.upload_material(
            MUTED_MATERIAL,
            &Material::new("ui-muted", Color::rgb(0.11, 0.13, 0.16)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "hello-ui-toolbar-pipeline",
            PipelineKind::SolidColor2d,
        ))?;
        self.renderer = Some(renderer);
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::CursorMoved { x, y } = event {
            self.cursor_position = [x, y];
            let cursor = self.cursor_world();
            self.hovered_button = self
                .buttons
                .iter()
                .find(|button| button.contains(cursor))
                .map(|button| button.id);
        }

        if let PlatformInputEvent::MouseInput {
            button: MouseButton::Left,
            pressed: true,
        } = event
        {
            let cursor = self.cursor_world();
            self.active_button = self.buttons.iter().find_map(|button| {
                if button.contains(cursor) {
                    Some(if self.active_button == Some(button.id) {
                        None
                    } else {
                        Some(button.id)
                    })
                } else {
                    None
                }
            }).flatten();
        }

        if let PlatformInputEvent::Resized { width, height } = event {
            self.window_size = [width.max(1) as f32, height.max(1) as f32];
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize_surface(width, height);
            }
        }

        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        renderer.upload_camera(
            CAMERA_HANDLE,
            Camera::orthographic_2d(self.window_size[0], self.window_size[1]),
        );
        renderer.begin_frame();
        renderer.submit(&[RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.05, 0.06, 0.08),
        })]);

        let rail = UiRect::new([0.0, 0.58], [0.98, 0.18]);
        Self::draw_button(
            renderer,
            self.pipeline,
            &self.theme,
            &UiButton::new(UiButtonId(99), "", rail),
            UiInteractionState::Idle,
            UiControlRole::Secondary,
        );

        for button in &self.buttons {
            let state = if Some(button.id) == self.active_button {
                UiInteractionState::Selected
            } else if Some(button.id) == self.hovered_button {
                UiInteractionState::Hovered
            } else {
                UiInteractionState::Idle
            };
            let role = match button.id.0 {
                0 => UiControlRole::Secondary,
                1 => UiControlRole::Primary,
                _ => UiControlRole::Quiet,
            };
            Self::draw_button(renderer, self.pipeline, &self.theme, button, state, role);
        }

        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}
