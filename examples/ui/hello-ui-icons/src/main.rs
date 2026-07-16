use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, MouseButton, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use ui_tools::{UiCard, UiCardRole, UiDrawer, UiRect, UiRegion, UiSurfaceCommand, UiSurfaceRole, UiTheme};

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
        WindowConfig { title: "Tokimu Hello UI Icons".into(), width: 1260, height: 820 },
        HelloUiIconsApp::new(),
    )
}

struct HelloUiIconsApp {
    renderer: Option<WgpuBackend>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    theme: UiTheme,
    selected_tile: usize,
    hovered_tile: Option<usize>,
}

impl Default for HelloUiIconsApp {
    fn default() -> Self {
        Self { renderer: None, window_size: [1.0, 1.0], pipeline: PipelineHandle(0), theme: UiTheme::default(), selected_tile: 0, hovered_tile: None }
    }
}

impl HelloUiIconsApp {
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
        renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: REGION_MESH,
            material: Self::material_for_role(command.style.role),
            pipeline,
            instance: Instance2d::new(rect.center, rect.size, 0.0),
            camera: Some(CAMERA_HANDLE),
            viewport: None,
        })]);
    }

    fn tile_rects(&self) -> [UiRect; 4] {
        [
            UiRect::new([-0.36, 0.20], [0.40, 0.44]),
            UiRect::new([0.04, 0.20], [0.40, 0.44]),
            UiRect::new([-0.36, -0.32], [0.40, 0.44]),
            UiRect::new([0.04, -0.32], [0.40, 0.44]),
        ]
    }
}

impl PlatformEventHandler for HelloUiIconsApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(REGION_MESH, &Mesh::quad());
        renderer.upload_material(BACKDROP_MATERIAL, &Material::new("ui-icons-backdrop", Color::rgb(0.05, 0.06, 0.08)))?;
        renderer.upload_material(SURFACE_MATERIAL, &Material::new("ui-icons-surface", Color::rgb(0.18, 0.20, 0.25)))?;
        renderer.upload_material(PANEL_MATERIAL, &Material::new("ui-icons-panel", Color::rgb(0.14, 0.16, 0.20)))?;
        renderer.upload_material(CARD_MATERIAL, &Material::new("ui-icons-card", Color::rgb(0.22, 0.24, 0.30)))?;
        renderer.upload_material(ACTIVE_MATERIAL, &Material::new("ui-icons-active", Color::rgb(0.34, 0.56, 0.86)))?;
        renderer.upload_material(MUTED_MATERIAL, &Material::new("ui-icons-muted", Color::rgb(0.10, 0.12, 0.14)))?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new("hello-ui-icons-pipeline", PipelineKind::SolidColor2d))?;
        self.renderer = Some(renderer);
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        match event {
            PlatformInputEvent::MouseInput { button: MouseButton::Left, pressed: true } => {
                if let Some(index) = self.hovered_tile { self.selected_tile = index; }
            }
            PlatformInputEvent::CursorMoved { x, y } => {
                let width = self.window_size[0].max(1.0);
                let height = self.window_size[1].max(1.0);
                let px = x / width;
                let py = y / height;
                self.hovered_tile = self.tile_rects().iter().enumerate().find_map(|(index, rect)| {
                    let left = (rect.center[0] - rect.size[0] * 0.5 + 0.5) * 0.5;
                    let right = (rect.center[0] + rect.size[0] * 0.5 + 0.5) * 0.5;
                    let top = 0.5 - (rect.center[1] + rect.size[1] * 0.5) * 0.5;
                    let bottom = 0.5 - (rect.center[1] - rect.size[1] * 0.5) * 0.5;
                    (px >= left && px <= right && py >= top && py <= bottom).then_some(index)
                });
            }
            PlatformInputEvent::KeyboardInput { key, pressed: true } => match key {
                tokimu::KeyCode::ArrowLeft => self.selected_tile = self.selected_tile.saturating_sub(1),
                tokimu::KeyCode::ArrowRight => self.selected_tile = (self.selected_tile + 1) % 4,
                tokimu::KeyCode::Space => self.selected_tile = (self.selected_tile + 1) % 4,
                _ => {}
            },
            PlatformInputEvent::Resized { width, height } => {
                self.window_size = [width.max(1) as f32, height.max(1) as f32];
                if let Some(renderer) = self.renderer.as_mut() { renderer.resize_surface(width, height); }
            }
            _ => {}
        }
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let tiles = self.tile_rects();
        let hovered_tile = self.hovered_tile;
        let selected_tile = self.selected_tile;
        let Some(renderer) = self.renderer.as_mut() else { return Ok(FrameOutcome::Continue); };

        renderer.upload_camera(CAMERA_HANDLE, Camera::orthographic_2d(self.window_size[0], self.window_size[1]));
        renderer.begin_frame();
        renderer.submit(&[RenderCommand::Clear(ClearCommand { color: Color::rgb(0.05, 0.06, 0.08) })]);

        let mut surfaces = Vec::new();
        let mut text = Vec::new();
        {
            let mut drawer = ui_tools::UiDrawer::new(&mut surfaces, &mut text, &self.theme);
            drawer.surface(&UiRegion::panel(UiRect::new([0.0, 0.0], [self.window_size[0] / self.window_size[1], 1.40])));
        }

        for (index, rect) in tiles.iter().enumerate() {
            let tile_role = if index == selected_tile { UiSurfaceRole::Selected } else if Some(index) == hovered_tile { UiSurfaceRole::Accent } else { UiSurfaceRole::Card };
            let outer = UiCard::new(UiCardRole::Browser, "", "", UiRegion::card(*rect));
            let mut tile_surfaces = Vec::new();
            let mut tile_text = Vec::new();
            {
                let mut drawer = UiDrawer::new(&mut tile_surfaces, &mut tile_text, &self.theme);
                drawer.card(&outer);
            }
            for command in tile_surfaces {
                let mut command = command;
                if command.style.role == UiSurfaceRole::Card { command.style.role = tile_role; }
                Self::draw_surface(renderer, self.pipeline, &command);
            }

            let icon = UiSurfaceCommand { rect: UiRect::new([rect.center[0], rect.center[1] + 0.08], [0.10, 0.10]), style: self.theme.surface(tile_role) };
            let caption = UiSurfaceCommand { rect: UiRect::new([rect.center[0], rect.center[1] - 0.10], [rect.size[0] * 0.62, 0.06]), style: self.theme.surface(UiSurfaceRole::Overlay) };
            Self::draw_surface(renderer, self.pipeline, &icon);
            Self::draw_surface(renderer, self.pipeline, &caption);
        }

        for command in surfaces {
            Self::draw_surface(renderer, self.pipeline, &command);
        }

        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}