use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, KeyCode, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use ui_tools::{UiButton, UiButtonId, UiButtonSpec, UiCardRole, UiControlRole, UiDrawer, UiRect, UiSurfaceCommand, UiSurfaceRole, UiTheme, UiWorkspaceLayout};

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
        WindowConfig { title: "Tokimu Hello UI Dashboard".into(), width: 1360, height: 840 },
        HelloUiDashboardApp::new(),
    )
}

struct HelloUiDashboardApp {
    renderer: Option<WgpuBackend>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    theme: UiTheme,
    active_tab: usize,
    active_card: usize,
    hovered_button: Option<usize>,
}

impl Default for HelloUiDashboardApp {
    fn default() -> Self {
        Self { renderer: None, window_size: [1.0, 1.0], pipeline: PipelineHandle(0), theme: UiTheme::default(), active_tab: 0, active_card: 0, hovered_button: None }
    }
}

impl HelloUiDashboardApp {
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
            let shadow_rect = UiRect::new([rect.center[0] + 0.012, rect.center[1] - 0.012], [rect.size[0], rect.size[1]]);
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
}

impl PlatformEventHandler for HelloUiDashboardApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(REGION_MESH, &Mesh::quad());
        renderer.upload_material(BACKDROP_MATERIAL, &Material::new("ui-dashboard-backdrop", Color::rgb(0.05, 0.06, 0.08)))?;
        renderer.upload_material(SURFACE_MATERIAL, &Material::new("ui-dashboard-surface", Color::rgb(0.18, 0.20, 0.25)))?;
        renderer.upload_material(PANEL_MATERIAL, &Material::new("ui-dashboard-panel", Color::rgb(0.14, 0.16, 0.20)))?;
        renderer.upload_material(CARD_MATERIAL, &Material::new("ui-dashboard-card", Color::rgb(0.22, 0.24, 0.30)))?;
        renderer.upload_material(ACTIVE_MATERIAL, &Material::new("ui-dashboard-active", Color::rgb(0.34, 0.56, 0.86)))?;
        renderer.upload_material(MUTED_MATERIAL, &Material::new("ui-dashboard-muted", Color::rgb(0.10, 0.12, 0.14)))?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new("hello-ui-dashboard-pipeline", PipelineKind::SolidColor2d))?;
        self.renderer = Some(renderer);
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        match event {
            PlatformInputEvent::KeyboardInput { key, pressed: true } => match key {
                KeyCode::ArrowLeft => self.active_tab = self.active_tab.saturating_sub(1),
                KeyCode::ArrowRight => self.active_tab = (self.active_tab + 1) % 3,
                KeyCode::ArrowUp => self.active_card = self.active_card.saturating_sub(1),
                KeyCode::ArrowDown => self.active_card = (self.active_card + 1) % 3,
                _ => {}
            },
            PlatformInputEvent::MouseInput { button: tokimu::MouseButton::Left, pressed: true } => {
                self.active_card = (self.active_card + 1) % 3;
            }
            PlatformInputEvent::CursorMoved { x, y } => {
                let width = self.window_size[0].max(1.0);
                let height = self.window_size[1].max(1.0);
                let px = x / width;
                let py = y / height;
                self.hovered_button = if py > 0.82 && py < 0.90 {
                    if px > 0.40 && px < 0.54 { Some(0) } else if px > 0.54 && px < 0.68 { Some(1) } else { None }
                } else { None };
            }
            PlatformInputEvent::Resized { width, height } => {
                self.window_size = [width.max(1) as f32, height.max(1) as f32];
                if let Some(renderer) = self.renderer.as_mut() { renderer.resize_surface(width, height); }
            }
            _ => {}
        }
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let Some(renderer) = self.renderer.as_mut() else { return Ok(FrameOutcome::Continue); };

        renderer.upload_camera(CAMERA_HANDLE, Camera::orthographic_2d(self.window_size[0], self.window_size[1]));
        renderer.begin_frame();
        renderer.submit(&[RenderCommand::Clear(ClearCommand { color: Color::rgb(0.05, 0.06, 0.08) })]);

        let layout = UiWorkspaceLayout::new(
            self.window_size,
            [
                UiButtonSpec::new(UiButtonId(0), "OVERVIEW"),
                UiButtonSpec::new(UiButtonId(1), "LAYOUT"),
                UiButtonSpec::new(UiButtonId(2), "STATUS"),
            ],
            [
                ui_tools::UiCardSpec::new(UiCardRole::Browser, "", ""),
                ui_tools::UiCardSpec::new(UiCardRole::Editor, "", ""),
                ui_tools::UiCardSpec::new(UiCardRole::Inspector, "", ""),
            ],
        );

        let mut surfaces = Vec::new();
        let mut text = Vec::new();
        {
            let mut drawer = UiDrawer::new(&mut surfaces, &mut text, &self.theme);
            drawer.workspace(&layout.workspace);
            drawer.surface(&layout.header);
            drawer.surface(&layout.toolbar);
            drawer.surface(&layout.sidebar);
            drawer.surface(&layout.canvas);
            drawer.surface(&layout.inspector);
            drawer.surface(&layout.status_bar);
            drawer.surface(&layout.card_grid);
        }

        for command in surfaces { Self::draw_surface(renderer, self.pipeline, &command); }

        let buttons = layout.buttons;
        for (index, button) in buttons.iter().enumerate() {
            let state = if Some(index) == self.hovered_button { ui_tools::UiInteractionState::Hovered } else if index == self.active_tab { ui_tools::UiInteractionState::Selected } else { ui_tools::UiInteractionState::Idle };
            let control = UiButton::new(button.id, button.label, button.rect);
            let mut button_surfaces = Vec::new();
            let mut button_text = Vec::new();
            {
                let mut drawer = UiDrawer::new(&mut button_surfaces, &mut button_text, &self.theme);
                drawer.button(&control, state, UiControlRole::Primary);
            }
            for command in button_surfaces { Self::draw_surface(renderer, self.pipeline, &command); }
        }

        for (index, card) in layout.cards.iter().enumerate() {
            let mut card = *card;
            card.role = if index == self.active_card { UiCardRole::Inspector } else if index == self.active_tab { UiCardRole::Editor } else { UiCardRole::Browser };
            let mut card_surfaces = Vec::new();
            let mut card_text = Vec::new();
            {
                let mut drawer = UiDrawer::new(&mut card_surfaces, &mut card_text, &self.theme);
                drawer.card(&card);
            }
            for command in card_surfaces { Self::draw_surface(renderer, self.pipeline, &command); }
        }

        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}