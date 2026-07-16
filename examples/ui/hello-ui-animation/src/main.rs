use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, KeyCode, Material, MaterialHandle, Mesh, MeshHandle, MouseButton, NativeWindow,
    Pipeline, PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use ui_tools::{UiCard, UiCardRole, UiDrawer, UiRegion, UiRect, UiSurfaceCommand, UiSurfaceRole, UiTheme};

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
        WindowConfig { title: "Tokimu Hello UI Animation".into(), width: 1260, height: 820 },
        HelloUiAnimationApp::new(),
    )
}

struct HelloUiAnimationApp {
    renderer: Option<WgpuBackend>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    theme: UiTheme,
    progress: f32,
    target_open: bool,
    hovered: bool,
}

impl Default for HelloUiAnimationApp {
    fn default() -> Self {
        Self { renderer: None, window_size: [1.0, 1.0], pipeline: PipelineHandle(0), theme: UiTheme::default(), progress: 0.0, target_open: false, hovered: false }
    }
}

impl HelloUiAnimationApp {
    fn new() -> Self { Self::default() }

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

    fn draw_surface(renderer: &mut WgpuBackend, pipeline: PipelineHandle, command: &UiSurfaceCommand) {
        let rect = command.rect;
        if matches!(command.style.elevation, ui_tools::UiElevation::Raised | ui_tools::UiElevation::Floating) {
            let shadow_rect = UiRect::new([rect.center[0] + 0.01, rect.center[1] - 0.01], [rect.size[0], rect.size[1]]);
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

    fn toggle(&mut self) { self.target_open = !self.target_open; }
}

impl PlatformEventHandler for HelloUiAnimationApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(REGION_MESH, &Mesh::quad());
        renderer.upload_material(BACKDROP_MATERIAL, &Material::new("ui-animation-backdrop", Color::rgb(0.05, 0.06, 0.08)))?;
        renderer.upload_material(SURFACE_MATERIAL, &Material::new("ui-animation-surface", Color::rgb(0.18, 0.20, 0.25)))?;
        renderer.upload_material(PANEL_MATERIAL, &Material::new("ui-animation-panel", Color::rgb(0.14, 0.16, 0.20)))?;
        renderer.upload_material(CARD_MATERIAL, &Material::new("ui-animation-card", Color::rgb(0.22, 0.24, 0.30)))?;
        renderer.upload_material(ACTIVE_MATERIAL, &Material::new("ui-animation-active", Color::rgb(0.34, 0.56, 0.86)))?;
        renderer.upload_material(MUTED_MATERIAL, &Material::new("ui-animation-muted", Color::rgb(0.10, 0.12, 0.14)))?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new("hello-ui-animation-pipeline", PipelineKind::SolidColor2d))?;
        self.renderer = Some(renderer);
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        match event {
            PlatformInputEvent::KeyboardInput { key, pressed: true } => match key {
                KeyCode::Space => self.toggle(),
                KeyCode::ArrowLeft => self.target_open = false,
                KeyCode::ArrowRight => self.target_open = true,
                _ => {}
            },
            PlatformInputEvent::MouseInput { button: MouseButton::Left, pressed: true } => self.toggle(),
            PlatformInputEvent::CursorMoved { x, y } => {
                let width = self.window_size[0].max(1.0);
                let height = self.window_size[1].max(1.0);
                let point = [x / width, y / height];
                self.hovered = point[0] > 0.33 && point[0] < 0.67 && point[1] > 0.28 && point[1] < 0.72;
            }
            PlatformInputEvent::Resized { width, height } => {
                self.window_size = [width.max(1) as f32, height.max(1) as f32];
                if let Some(renderer) = self.renderer.as_mut() { renderer.resize_surface(width, height); }
            }
            _ => {}
        }
        Ok(())
    }

    fn on_frame(&mut self, delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let Some(renderer) = self.renderer.as_mut() else { return Ok(FrameOutcome::Continue); };
        let step = (delta_seconds as f32 * 6.0).min(1.0);
        let target = if self.target_open { 1.0 } else { 0.0 };
        self.progress += (target - self.progress) * step;
        let theme = self.theme;

        renderer.upload_camera(CAMERA_HANDLE, Camera::orthographic_2d(self.window_size[0], self.window_size[1]));
        renderer.begin_frame();
        renderer.submit(&[RenderCommand::Clear(ClearCommand { color: Color::rgb(0.05, 0.06, 0.08) })]);

        let mut surfaces = Vec::new();
            let mut surface_text = Vec::new();
        {
                let mut drawer = UiDrawer::new(&mut surfaces, &mut surface_text, &theme);
            drawer.surface(&UiRegion::panel(UiRect::new([0.0, 0.0], [self.window_size[0] / self.window_size[1], 1.45])));
        }

        let card_width = 0.36 + self.progress * 0.42;
        let card_height = 0.22 + self.progress * 0.36;
        let card_x = -0.20 + self.progress * 0.32;
        let card_y = 0.06 + self.progress * 0.08;
        let card = UiCard::new(UiCardRole::Editor, "", "", UiRegion::card(UiRect::new([card_x, card_y], [card_width, card_height])));

        let button_rect = UiRect::new([0.0, -0.58], [0.52, 0.12]);
        let button_card = UiCard::new(
            if self.target_open { UiCardRole::Status } else { UiCardRole::Browser },
            "",
            "",
            UiRegion::card(button_rect),
        );

        let mut animated = Vec::new();
            let mut animated_text = Vec::new();
        {
                let mut drawer = UiDrawer::new(&mut animated, &mut animated_text, &theme);
            drawer.card(&card);
            drawer.card(&button_card);
        }

        for command in surfaces {
            Self::draw_surface(renderer, self.pipeline, &command);
        }
        for command in animated {
            Self::draw_surface(renderer, self.pipeline, &command);
        }

        if self.hovered {
            let hover = UiSurfaceCommand { rect: UiRect::new([0.0, 0.54], [0.32, 0.06]), style: self.theme.surface(UiSurfaceRole::Accent) };
            Self::draw_surface(renderer, self.pipeline, &hover);
        }

        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}