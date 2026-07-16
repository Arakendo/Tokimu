use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use ui_tools::{
    layout_bitmap_text, UiButton, UiButtonId, UiControlRole, UiDrawer, UiLabel, UiLabelAnchor,
    UiRect, UiSurfaceCommand, UiSurfaceRole, UiTextRole, UiTheme,
};

const PANEL_MESH: MeshHandle = MeshHandle(1);
const GLYPH_MESH: MeshHandle = MeshHandle(2);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const BACKDROP_MATERIAL: MaterialHandle = MaterialHandle(1);
const PANEL_MATERIAL: MaterialHandle = MaterialHandle(2);
const SHELL_MATERIAL: MaterialHandle = MaterialHandle(3);
const MUTED_MATERIAL: MaterialHandle = MaterialHandle(4);
const TEXT_MATERIAL: MaterialHandle = MaterialHandle(5);

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello UI Panel".into(),
            width: 1100,
            height: 700,
        },
        HelloUiPanelApp::new(),
    )
}

struct HelloUiPanelApp {
    renderer: Option<WgpuBackend>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    theme: UiTheme,
}

impl Default for HelloUiPanelApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            theme: UiTheme::default(),
        }
    }
}

impl HelloUiPanelApp {
    fn new() -> Self {
        Self::default()
    }

    fn material_for_role(role: UiSurfaceRole) -> MaterialHandle {
        match role {
            UiSurfaceRole::Background => BACKDROP_MATERIAL,
            UiSurfaceRole::Panel => PANEL_MATERIAL,
            UiSurfaceRole::Card => SHELL_MATERIAL,
            UiSurfaceRole::Raised => SHELL_MATERIAL,
            UiSurfaceRole::Selected => PANEL_MATERIAL,
            UiSurfaceRole::Accent => PANEL_MATERIAL,
            UiSurfaceRole::Overlay => MUTED_MATERIAL,
            UiSurfaceRole::Toolbar | UiSurfaceRole::Region => PANEL_MATERIAL,
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
                let edges = [
                    UiRect::new(
                        [rect.center[0], rect.center[1] + rect.size[1] * 0.5 - border * 0.5],
                        [rect.size[0], border],
                    ),
                    UiRect::new(
                        [rect.center[0], rect.center[1] - rect.size[1] * 0.5 + border * 0.5],
                        [rect.size[0], border],
                    ),
                    UiRect::new(
                        [rect.center[0] - rect.size[0] * 0.5 + border * 0.5, rect.center[1]],
                        [border, rect.size[1]],
                    ),
                    UiRect::new(
                        [rect.center[0] + rect.size[0] * 0.5 - border * 0.5, rect.center[1]],
                        [border, rect.size[1]],
                    ),
                ];
                for edge in edges {
                    renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
                        mesh: PANEL_MESH,
                        material: border_material,
                        pipeline,
                        instance: Instance2d::new(edge.center, edge.size, 0.0),
                        camera: Some(CAMERA_HANDLE),
                        viewport: None,
                    })]);
                }
            }
        }
    }

    fn draw_text_command(
        renderer: &mut WgpuBackend,
        pipeline: PipelineHandle,
        command: &ui_tools::UiTextCommand,
    ) {
        let commands = layout_bitmap_text(&command.spec, command.style.height)
            .into_iter()
            .map(|quad| {
                RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: GLYPH_MESH,
                    material: TEXT_MATERIAL,
                    pipeline,
                    instance: Instance2d::new(quad.center, quad.size, 0.0),
                    camera: Some(CAMERA_HANDLE),
                    viewport: None,
                })
            })
            .collect::<Vec<_>>();
        renderer.submit(&commands);
    }
}

impl PlatformEventHandler for HelloUiPanelApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(PANEL_MESH, &Mesh::quad());
        renderer.upload_mesh(GLYPH_MESH, &Mesh::quad());
        renderer.upload_material(
            BACKDROP_MATERIAL,
            &Material::new("ui-backdrop", Color::rgb(0.05, 0.06, 0.09)),
        )?;
        renderer.upload_material(
            PANEL_MATERIAL,
            &Material::new("ui-panel", Color::rgb(0.21, 0.23, 0.29)),
        )?;
        renderer.upload_material(
            SHELL_MATERIAL,
            &Material::new("ui-shell", Color::rgb(0.15, 0.17, 0.21)),
        )?;
        renderer.upload_material(
            MUTED_MATERIAL,
            &Material::new("ui-muted", Color::rgb(0.11, 0.13, 0.16)),
        )?;
        renderer.upload_material(
            TEXT_MATERIAL,
            &Material::new("ui-text", Color::rgb(0.90, 0.93, 0.98)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "hello-ui-panel-pipeline",
            PipelineKind::SolidColor2d,
        ))?;
        self.renderer = Some(renderer);
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
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
            color: Color::rgb(0.05, 0.06, 0.09),
        })]);

        let outer = UiRect::new([0.0, 0.0], [0.88, 0.50]);
        let inner = UiRect::new([0.0, -0.01], [0.72, 0.28]);
        let strip = UiRect::new([0.0, 0.16], [0.72, 0.08]);
        let button = UiButton::new(
            UiButtonId(0),
            "ACTION",
            UiRect::new([0.0, -0.01], [0.28, 0.08]),
        );
        let title = UiLabel::new("PANEL", UiLabelAnchor::Center, [0.0, 0.16]);

        let mut surfaces = vec![
            UiSurfaceCommand {
                rect: outer,
                style: self.theme.surface(UiSurfaceRole::Raised),
            },
            UiSurfaceCommand {
                rect: strip,
                style: self.theme.surface(UiSurfaceRole::Accent),
            },
            UiSurfaceCommand {
                rect: inner,
                style: self.theme.surface(UiSurfaceRole::Panel),
            },
        ];
        let mut text = Vec::new();
        {
            let mut drawer = UiDrawer::new(&mut surfaces, &mut text, &self.theme);
            drawer.label(&title, UiTextRole::Title);
            drawer.button(&button, ui_tools::UiInteractionState::Idle, UiControlRole::Primary);
        }

        for command in surfaces {
            Self::draw_surface_command(renderer, self.pipeline, &command);
        }
        for command in text {
            Self::draw_text_command(renderer, self.pipeline, &command);
        }

        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}
