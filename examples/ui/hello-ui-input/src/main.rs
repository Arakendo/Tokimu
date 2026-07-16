use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, MouseButton, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use tokimu_input::{InputState, KeyCode};
use ui_tools::{UiRect, UiSurfaceRole};

const REGION_MESH: MeshHandle = MeshHandle(1);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const BACKDROP_MATERIAL: MaterialHandle = MaterialHandle(1);
const SURFACE_MATERIAL: MaterialHandle = MaterialHandle(2);
const PANEL_MATERIAL: MaterialHandle = MaterialHandle(3);
const CARD_MATERIAL: MaterialHandle = MaterialHandle(4);
const ACTIVE_MATERIAL: MaterialHandle = MaterialHandle(5);
const MUTED_MATERIAL: MaterialHandle = MaterialHandle(6);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FocusTarget {
    Mouse,
    Keyboard,
    Capture,
}

impl FocusTarget {
    fn next(self) -> Self {
        match self {
            Self::Mouse => Self::Keyboard,
            Self::Keyboard => Self::Capture,
            Self::Capture => Self::Mouse,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Mouse => Self::Capture,
            Self::Keyboard => Self::Mouse,
            Self::Capture => Self::Keyboard,
        }
    }
}

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello UI Input".into(),
            width: 1200,
            height: 760,
        },
        HelloUiInputApp::new(),
    )
}

struct HelloUiInputApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    input: InputState,
    focus: FocusTarget,
    hovered: Option<FocusTarget>,
    captured: bool,
}

impl Default for HelloUiInputApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            input: InputState::default(),
            focus: FocusTarget::Mouse,
            hovered: None,
            captured: false,
        }
    }
}

impl HelloUiInputApp {
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

    fn draw_region(
        renderer: &mut WgpuBackend,
        pipeline: PipelineHandle,
        rect: UiRect,
        role: UiSurfaceRole,
        active: bool,
    ) {
        let style_role = if active { UiSurfaceRole::Selected } else { role };
        renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: REGION_MESH,
            material: Self::material_for_role(style_role),
            pipeline,
            instance: Instance2d::new(rect.center, rect.size, 0.0),
            camera: Some(CAMERA_HANDLE),
            viewport: None,
        })]);
    }

    fn layout(&self) -> [UiRect; 3] {
        let width = self.window_size[0].max(1.0);
        let height = self.window_size[1].max(1.0);
        let half_height = 1.0;
        let half_width = half_height * (width / height);
        let column_width = (half_width * 1.68 / 3.0).max(0.42);
        let card_y = 0.06;
        [
            UiRect::new([-column_width - 0.18, card_y], [column_width, 0.72]),
            UiRect::new([0.0, card_y], [column_width, 0.72]),
            UiRect::new([column_width + 0.18, card_y], [column_width, 0.72]),
        ]
    }

    fn focus_rects(&self) -> [UiRect; 3] {
        let width = self.window_size[0].max(1.0);
        let height = self.window_size[1].max(1.0);
        let half_height = 1.0;
        let half_width = half_height * (width / height);
        let base_y = -0.56;
        [
            UiRect::new([-half_width + 0.42, base_y], [0.42, 0.12]),
            UiRect::new([0.0, base_y], [0.42, 0.12]),
            UiRect::new([half_width - 0.42, base_y], [0.42, 0.12]),
        ]
    }

    fn focus_at_point(&self, point: [f32; 2]) -> Option<FocusTarget> {
        let rects = self.focus_rects();
        if rects[0].contains(point) {
            Some(FocusTarget::Mouse)
        } else if rects[1].contains(point) {
            Some(FocusTarget::Keyboard)
        } else if rects[2].contains(point) {
            Some(FocusTarget::Capture)
        } else {
            None
        }
    }

    fn cursor_world(&self) -> [f32; 2] {
        let width = self.window_size[0].max(1.0);
        let height = self.window_size[1].max(1.0);
        let half_height = 1.0;
        let half_width = half_height * (width / height);
        let x = (self.input.mouse.x / width) * (half_width * 2.0) - half_width;
        let y = half_height - (self.input.mouse.y / height) * (half_height * 2.0);
        [x, y]
    }

    fn update_hover(&mut self) {
        self.hovered = self.focus_at_point(self.cursor_world());
    }

    fn update_title(&self) {
        if let Some(window) = self.window.as_ref() {
            window.set_title(&format!(
                "Tokimu Hello UI Input | focus={:?} | hovered={:?} | mouse={} | left={} | right={} | capture={}",
                self.focus,
                self.hovered,
                if self.input.mouse.is_pressed(MouseButton::Left) { "down" } else { "up" },
                if self.input.keyboard.is_pressed(KeyCode::ArrowLeft) { "down" } else { "up" },
                if self.input.keyboard.is_pressed(KeyCode::ArrowRight) { "down" } else { "up" },
                if self.captured { "on" } else { "off" },
            ));
        }
    }

    fn draw_scene(&mut self) -> PlatformResult<FrameOutcome> {
        let columns = self.layout();
        let focus_rects = self.focus_rects();

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

        Self::draw_region(renderer, self.pipeline, columns[0], UiSurfaceRole::Panel, self.focus == FocusTarget::Mouse);
        Self::draw_region(renderer, self.pipeline, columns[1], UiSurfaceRole::Card, self.focus == FocusTarget::Keyboard);
        Self::draw_region(renderer, self.pipeline, columns[2], UiSurfaceRole::Toolbar, self.focus == FocusTarget::Capture);

        Self::draw_region(renderer, self.pipeline, focus_rects[0], UiSurfaceRole::Region, self.focus == FocusTarget::Mouse || self.hovered == Some(FocusTarget::Mouse));
        Self::draw_region(renderer, self.pipeline, focus_rects[1], UiSurfaceRole::Region, self.focus == FocusTarget::Keyboard || self.hovered == Some(FocusTarget::Keyboard));
        Self::draw_region(renderer, self.pipeline, focus_rects[2], UiSurfaceRole::Region, self.focus == FocusTarget::Capture || self.hovered == Some(FocusTarget::Capture));

        let _ = renderer.present()?;
        self.update_title();
        Ok(FrameOutcome::Continue)
    }
}

impl PlatformEventHandler for HelloUiInputApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(REGION_MESH, &Mesh::quad());
        renderer.upload_material(
            BACKDROP_MATERIAL,
            &Material::new("ui-input-backdrop", Color::rgb(0.05, 0.06, 0.08)),
        )?;
        renderer.upload_material(
            SURFACE_MATERIAL,
            &Material::new("ui-input-surface", Color::rgb(0.18, 0.20, 0.25)),
        )?;
        renderer.upload_material(
            PANEL_MATERIAL,
            &Material::new("ui-input-panel", Color::rgb(0.14, 0.16, 0.20)),
        )?;
        renderer.upload_material(
            CARD_MATERIAL,
            &Material::new("ui-input-card", Color::rgb(0.22, 0.24, 0.30)),
        )?;
        renderer.upload_material(
            ACTIVE_MATERIAL,
            &Material::new("ui-input-active", Color::rgb(0.34, 0.56, 0.86)),
        )?;
        renderer.upload_material(
            MUTED_MATERIAL,
            &Material::new("ui-input-muted", Color::rgb(0.10, 0.12, 0.14)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "hello-ui-input-pipeline",
            PipelineKind::SolidColor2d,
        ))?;
        self.renderer = Some(renderer);
        self.update_hover();
        self.update_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let Some(input_event) = event.as_input_event() {
            self.input.apply_event(input_event);
        }

        match event {
            PlatformInputEvent::CursorMoved { .. } => {
                self.update_hover();
            }
            PlatformInputEvent::MouseInput {
                button: MouseButton::Left,
                pressed: true,
            } => {
                self.captured = !self.captured;
                if let Some(focus) = self.hovered {
                    self.focus = focus;
                }
            }
            PlatformInputEvent::KeyboardInput { key, pressed: true } => match key {
                KeyCode::ArrowLeft => self.focus = self.focus.prev(),
                KeyCode::ArrowRight => self.focus = self.focus.next(),
                KeyCode::Space => self.captured = !self.captured,
                _ => {}
            },
            PlatformInputEvent::Resized { width, height } => {
                self.window_size = [width.max(1) as f32, height.max(1) as f32];
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize_surface(width, height);
                }
            }
            _ => {}
        }

        self.update_title();
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        self.draw_scene()
    }
}