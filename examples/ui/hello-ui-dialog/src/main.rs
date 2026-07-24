use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, KeyCode, Material, MaterialHandle, Mesh, MeshHandle, MouseButton, NativeWindow,
    Pipeline, PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent,
    PlatformResult, RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use ui_tools::{UiCard, UiCardRole, UiRect, UiRegion, UiSurfaceCommand, UiSurfaceRole, UiTheme};

const REGION_MESH: MeshHandle = MeshHandle(1);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const BACKDROP_MATERIAL: MaterialHandle = MaterialHandle(1);
const SURFACE_MATERIAL: MaterialHandle = MaterialHandle(2);
const PANEL_MATERIAL: MaterialHandle = MaterialHandle(3);
const CARD_MATERIAL: MaterialHandle = MaterialHandle(4);
const ACTIVE_MATERIAL: MaterialHandle = MaterialHandle(5);
const MUTED_MATERIAL: MaterialHandle = MaterialHandle(6);

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello UI Dialog".into(),
            width: 1260,
            height: 820,
        },
        HelloUiDialogApp::new(),
    )
}

struct HelloUiDialogApp {
    renderer: Option<WgpuBackend>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    theme: UiTheme,
    open: bool,
    hovered_button: Option<usize>,
    active_button: usize,
}

impl Default for HelloUiDialogApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            theme: UiTheme::default(),
            open: true,
            hovered_button: None,
            active_button: 1,
        }
    }
}

impl HelloUiDialogApp {
    fn new() -> Self {
        Self::default()
    }

    fn material_for_role(role: UiSurfaceRole) -> MaterialHandle {
        match role {
            UiSurfaceRole::Background => BACKDROP_MATERIAL,
            UiSurfaceRole::Region => SURFACE_MATERIAL,
            UiSurfaceRole::Panel => PANEL_MATERIAL,
            UiSurfaceRole::Card => CARD_MATERIAL,
            UiSurfaceRole::Toolbar => PANEL_MATERIAL,
            UiSurfaceRole::Raised => PANEL_MATERIAL,
            UiSurfaceRole::Selected => ACTIVE_MATERIAL,
            UiSurfaceRole::Accent => ACTIVE_MATERIAL,
            UiSurfaceRole::Overlay => MUTED_MATERIAL,
        }
    }

    fn draw_surface(
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
                [rect.center[0] + 0.01, rect.center[1] - 0.01],
                [rect.size[0], rect.size[1]],
            );
            renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: REGION_MESH,
                material: MUTED_MATERIAL,
                pipeline,
                instance: Instance2d::new(shadow_rect.center, shadow_rect.size, 0.0),
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            })]);
        }

        renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: REGION_MESH,
            material: Self::material_for_role(command.style.role),
            pipeline,
            instance: Instance2d::new(rect.center, rect.size, 0.0),
            camera: Some(CAMERA_HANDLE),
            viewport: None,
        })]);
    }

    fn button_rects(&self) -> [UiRect; 2] {
        let width = self.window_size[0].max(1.0) / self.window_size[1].max(1.0);
        let dialog_w = (width * 0.92).min(0.92);
        let base_y = -0.30;
        [
            UiRect::new([-0.16, base_y], [dialog_w * 0.32, 0.12]),
            UiRect::new([0.16, base_y], [dialog_w * 0.32, 0.12]),
        ]
    }
}

impl PlatformEventHandler for HelloUiDialogApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(REGION_MESH, &Mesh::quad());
        renderer.upload_material(
            BACKDROP_MATERIAL,
            &Material::new("ui-dialog-backdrop", Color::rgb(0.05, 0.06, 0.08)),
        )?;
        renderer.upload_material(
            SURFACE_MATERIAL,
            &Material::new("ui-dialog-surface", Color::rgb(0.18, 0.20, 0.25)),
        )?;
        renderer.upload_material(
            PANEL_MATERIAL,
            &Material::new("ui-dialog-panel", Color::rgb(0.14, 0.16, 0.20)),
        )?;
        renderer.upload_material(
            CARD_MATERIAL,
            &Material::new("ui-dialog-card", Color::rgb(0.22, 0.24, 0.30)),
        )?;
        renderer.upload_material(
            ACTIVE_MATERIAL,
            &Material::new("ui-dialog-active", Color::rgb(0.34, 0.56, 0.86)),
        )?;
        renderer.upload_material(
            MUTED_MATERIAL,
            &Material::new("ui-dialog-muted", Color::rgb(0.10, 0.12, 0.14)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "hello-ui-dialog-pipeline",
            PipelineKind::SolidColor2d,
        ))?;
        self.renderer = Some(renderer);
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        match event {
            PlatformInputEvent::KeyboardInput { key, pressed: true } => match key {
                KeyCode::Escape => self.open = false,
                KeyCode::Space => self.open = !self.open,
                KeyCode::ArrowLeft => self.active_button = 0,
                KeyCode::ArrowRight => self.active_button = 1,
                _ => {}
            },
            PlatformInputEvent::CursorMoved { x, y } => {
                let width = self.window_size[0].max(1.0);
                let height = self.window_size[1].max(1.0);
                let point = [x / width, y / height];
                self.hovered_button = if point[0] > 0.42
                    && point[0] < 0.58
                    && point[1] > 0.62
                    && point[1] < 0.76
                {
                    Some(0)
                } else if point[0] > 0.58 && point[0] < 0.74 && point[1] > 0.62 && point[1] < 0.76 {
                    Some(1)
                } else {
                    None
                };
            }
            PlatformInputEvent::MouseInput {
                button: MouseButton::Left,
                pressed: true,
            } => {
                if self.open {
                    if let Some(button) = self.hovered_button {
                        match button {
                            0 => self.open = false,
                            1 => self.open = false,
                            _ => {}
                        }
                    }
                } else {
                    self.open = true;
                }
            }
            PlatformInputEvent::Resized { width, height } => {
                self.window_size = [width.max(1) as f32, height.max(1) as f32];
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize_surface(width, height);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let buttons = self.button_rects();
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

        let backdrop = UiSurfaceCommand {
            rect: UiRect::new(
                [0.0, 0.0],
                [self.window_size[0] / self.window_size[1], 1.45],
            ),
            style: self.theme.surface(UiSurfaceRole::Overlay),
            clip: None,
        };
        Self::draw_surface(renderer, self.pipeline, &backdrop);

        if self.open {
            let dialog = UiCard::new(
                UiCardRole::Inspector,
                "",
                "",
                UiRegion::card(UiRect::new([0.0, 0.08], [0.80, 0.64])),
            );
            let dialog_rect = UiSurfaceCommand {
                rect: dialog.region.rect,
                style: self.theme.card(UiCardRole::Inspector),
                clip: None,
            };
            Self::draw_surface(renderer, self.pipeline, &dialog_rect);

            let header = UiSurfaceCommand {
                rect: UiRect::new([0.0, 0.30], [0.68, 0.08]),
                style: self.theme.surface(UiSurfaceRole::Selected),
                clip: None,
            };
            Self::draw_surface(renderer, self.pipeline, &header);

            let body = UiSurfaceCommand {
                rect: UiRect::new([0.0, 0.12], [0.62, 0.18]),
                style: self.theme.surface(UiSurfaceRole::Panel),
                clip: None,
            };
            Self::draw_surface(renderer, self.pipeline, &body);

            for (index, rect) in buttons.iter().enumerate() {
                let active = self.active_button == index || self.hovered_button == Some(index);
                let command = UiSurfaceCommand {
                    rect: *rect,
                    style: self.theme.control(
                        if index == 0 {
                            ui_tools::UiControlRole::Secondary
                        } else {
                            ui_tools::UiControlRole::Primary
                        },
                        if active {
                            ui_tools::UiInteractionState::Hovered
                        } else {
                            ui_tools::UiInteractionState::Idle
                        },
                    ),
                    clip: None,
                };
                Self::draw_surface(renderer, self.pipeline, &command);
            }
        } else {
            let reopen = UiSurfaceCommand {
                rect: UiRect::new([0.0, 0.0], [0.42, 0.12]),
                style: self.theme.surface(UiSurfaceRole::Panel),
                clip: None,
            };
            Self::draw_surface(renderer, self.pipeline, &reopen);
        }

        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}
