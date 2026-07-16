use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use ui_tools::{UiLabel, UiLabelAnchor, UiRect, UiRegion, UiSurfaceCommand, UiSurfaceRole, UiTextRole, UiTheme};

const BOX_MESH: MeshHandle = MeshHandle(1);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const BACKDROP_MATERIAL: MaterialHandle = MaterialHandle(1);
const FRAME_MATERIAL: MaterialHandle = MaterialHandle(2);
const INNER_MATERIAL: MaterialHandle = MaterialHandle(3);
const ACCENT_MATERIAL: MaterialHandle = MaterialHandle(4);
const MUTED_MATERIAL: MaterialHandle = MaterialHandle(5);

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello UI Box".into(),
            width: 1000,
            height: 680,
        },
        HelloUiBoxApp::new(),
    )
}

struct HelloUiBoxApp {
    renderer: Option<WgpuBackend>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    theme: UiTheme,
}

impl Default for HelloUiBoxApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            theme: UiTheme::default(),
        }
    }
}

impl HelloUiBoxApp {
    fn new() -> Self {
        Self::default()
    }

    fn material_for_role(role: UiSurfaceRole) -> MaterialHandle {
        match role {
            UiSurfaceRole::Background => BACKDROP_MATERIAL,
            UiSurfaceRole::Region => FRAME_MATERIAL,
            UiSurfaceRole::Panel => INNER_MATERIAL,
            UiSurfaceRole::Card => INNER_MATERIAL,
            UiSurfaceRole::Toolbar => FRAME_MATERIAL,
            UiSurfaceRole::Raised => ACCENT_MATERIAL,
            UiSurfaceRole::Selected => ACCENT_MATERIAL,
            UiSurfaceRole::Accent => ACCENT_MATERIAL,
            UiSurfaceRole::Overlay => MUTED_MATERIAL,
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
                [rect.center[0] + 0.01, rect.center[1] - 0.01],
                [rect.size[0], rect.size[1]],
            );
            renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: BOX_MESH,
                material: MUTED_MATERIAL,
                pipeline,
                instance: Instance2d::new(shadow_rect.center, shadow_rect.size, 0.0),
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            })]);
        }

        renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: BOX_MESH,
            material: Self::material_for_role(command.style.role),
            pipeline,
            instance: Instance2d::new(rect.center, rect.size, 0.0),
            camera: Some(CAMERA_HANDLE),
            viewport: None,
        })]);
    }
}

impl PlatformEventHandler for HelloUiBoxApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(BOX_MESH, &Mesh::quad());
        renderer.upload_material(
            BACKDROP_MATERIAL,
            &Material::new("ui-box-backdrop", Color::rgb(0.05, 0.06, 0.09)),
        )?;
        renderer.upload_material(
            FRAME_MATERIAL,
            &Material::new("ui-box-frame", Color::rgb(0.20, 0.22, 0.28)),
        )?;
        renderer.upload_material(
            INNER_MATERIAL,
            &Material::new("ui-box-inner", Color::rgb(0.14, 0.16, 0.20)),
        )?;
        renderer.upload_material(
            ACCENT_MATERIAL,
            &Material::new("ui-box-accent", Color::rgb(0.32, 0.54, 0.82)),
        )?;
        renderer.upload_material(
            MUTED_MATERIAL,
            &Material::new("ui-box-muted", Color::rgb(0.10, 0.12, 0.15)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "hello-ui-box-pipeline",
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

        let outer = UiRegion::new(
            ui_tools::UiRegionKind::Panel,
            UiSurfaceRole::Region,
            UiRect::new([0.0, 0.02], [0.88, 0.60]),
        );
        let inner = UiRegion::new(
            ui_tools::UiRegionKind::Panel,
            UiSurfaceRole::Panel,
            UiRect::new([0.0, -0.02], [0.66, 0.36]),
        );
        let strip = UiRegion::new(
            ui_tools::UiRegionKind::Panel,
            UiSurfaceRole::Raised,
            UiRect::new([0.0, 0.26], [0.66, 0.08]),
        );
        let label = UiLabel::new("BOX", UiLabelAnchor::Center, [0.0, 0.28]);
        let subtitle = UiLabel::new("BOUNDS / NESTING / SCALE", UiLabelAnchor::Center, [0.0, -0.22]);

        let mut surfaces = Vec::new();
        let mut text = Vec::new();
        {
            let mut drawer = ui_tools::UiDrawer::new(&mut surfaces, &mut text, &self.theme);
            drawer.surface(&outer);
            drawer.surface(&inner);
            drawer.surface(&strip);
            drawer.label(&label, UiTextRole::Title);
            drawer.label(&subtitle, UiTextRole::Caption);
        }

        for command in surfaces {
            Self::draw_surface_command(renderer, self.pipeline, &command);
        }

        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}
