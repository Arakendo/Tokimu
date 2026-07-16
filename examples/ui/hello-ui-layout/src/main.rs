use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use ui_tools::{UiDrawer, UiRect, UiSurfaceCommand, UiTheme, UiWorkspaceLayout};

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
            title: "Tokimu Hello UI Layout".into(),
            width: 1360,
            height: 840,
        },
        HelloUiLayoutApp::new(),
    )
}

struct HelloUiLayoutApp {
    renderer: Option<WgpuBackend>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    theme: UiTheme,
}

impl Default for HelloUiLayoutApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            theme: UiTheme::default(),
        }
    }
}

impl HelloUiLayoutApp {
    fn new() -> Self {
        Self::default()
    }

    fn material_for_role(role: ui_tools::UiSurfaceRole) -> MaterialHandle {
        match role {
            ui_tools::UiSurfaceRole::Background => BACKDROP_MATERIAL,
            ui_tools::UiSurfaceRole::Region => SURFACE_MATERIAL,
            ui_tools::UiSurfaceRole::Panel => PANEL_MATERIAL,
            ui_tools::UiSurfaceRole::Card => CARD_MATERIAL,
            ui_tools::UiSurfaceRole::Toolbar => PANEL_MATERIAL,
            ui_tools::UiSurfaceRole::Raised => PANEL_MATERIAL,
            ui_tools::UiSurfaceRole::Selected => ACTIVE_MATERIAL,
            ui_tools::UiSurfaceRole::Accent => ACTIVE_MATERIAL,
            ui_tools::UiSurfaceRole::Overlay => MUTED_MATERIAL,
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

        if let Some(border_role) = command.style.border_role {
            let border = command
                .style
                .border_width
                .min(rect.size[0] * 0.22)
                .min(rect.size[1] * 0.22);
            if border > 0.0 {
                let border_rect = UiRect::new(
                    [rect.center[0], rect.center[1] + rect.size[1] * 0.5 - border * 0.5],
                    [rect.size[0], border],
                );
                renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: REGION_MESH,
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

impl PlatformEventHandler for HelloUiLayoutApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(REGION_MESH, &Mesh::quad());
        renderer.upload_material(
            BACKDROP_MATERIAL,
            &Material::new("ui-layout-backdrop", Color::rgb(0.05, 0.06, 0.08)),
        )?;
        renderer.upload_material(
            SURFACE_MATERIAL,
            &Material::new("ui-layout-surface", Color::rgb(0.18, 0.20, 0.25)),
        )?;
        renderer.upload_material(
            PANEL_MATERIAL,
            &Material::new("ui-layout-panel", Color::rgb(0.14, 0.16, 0.20)),
        )?;
        renderer.upload_material(
            CARD_MATERIAL,
            &Material::new("ui-layout-card", Color::rgb(0.22, 0.24, 0.30)),
        )?;
        renderer.upload_material(
            ACTIVE_MATERIAL,
            &Material::new("ui-layout-active", Color::rgb(0.34, 0.56, 0.86)),
        )?;
        renderer.upload_material(
            MUTED_MATERIAL,
            &Material::new("ui-layout-muted", Color::rgb(0.10, 0.12, 0.14)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "hello-ui-layout-pipeline",
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
            color: Color::rgb(0.05, 0.06, 0.08),
        })]);

        let layout = UiWorkspaceLayout::new(
            self.window_size,
            [
                ui_tools::UiButtonSpec::new(ui_tools::UiButtonId(0), "HEADER"),
                ui_tools::UiButtonSpec::new(ui_tools::UiButtonId(1), "WORKSPACE"),
                ui_tools::UiButtonSpec::new(ui_tools::UiButtonId(2), "STATUS"),
            ],
            [
                ui_tools::UiCardSpec::new(ui_tools::UiCardRole::Browser, "Sidebar", "FILTERS + NAVIGATION"),
                ui_tools::UiCardSpec::new(ui_tools::UiCardRole::Editor, "Canvas", "MAIN CONTENT AREA"),
                ui_tools::UiCardSpec::new(ui_tools::UiCardRole::Inspector, "Inspector", "PROPERTIES + STATE"),
            ],
        );

        let mut surfaces = Vec::new();
        let mut text = Vec::new();
        {
            let mut drawer = UiDrawer::new(&mut surfaces, &mut text, &self.theme);
            drawer.workspace(&layout.workspace);
            drawer.toolbar(&layout.toolbar);
            drawer.surface(&layout.header);
            drawer.surface(&layout.sidebar);
            drawer.surface(&layout.canvas);
            drawer.surface(&layout.inspector);
            drawer.surface(&layout.status_bar);
            drawer.surface(&layout.card_grid);
        }

        for command in surfaces {
            Self::draw_surface_command(renderer, self.pipeline, &command);
        }

        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}