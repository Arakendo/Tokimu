use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, KeyCode, Material, MaterialHandle, Mesh, MeshHandle, MouseButton, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, ViewportRect, WgpuBackend, WindowConfig,
};
use ui_tools::{
    UiCard, UiCardRole, UiDrawer, UiRect, UiRegion, UiSurfaceCommand, UiSurfaceRole, UiTheme,
    UiVerticalScroll,
};

const REGION_MESH: MeshHandle = MeshHandle(1);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const BACKDROP_MATERIAL: MaterialHandle = MaterialHandle(1);
const SURFACE_MATERIAL: MaterialHandle = MaterialHandle(2);
const PANEL_MATERIAL: MaterialHandle = MaterialHandle(3);
const CARD_MATERIAL: MaterialHandle = MaterialHandle(4);
const ACTIVE_MATERIAL: MaterialHandle = MaterialHandle(5);
const MUTED_MATERIAL: MaterialHandle = MaterialHandle(6);

const ITEM_COUNT: usize = 10;

#[derive(Clone, Copy)]
struct ScrollItem {
    role: UiCardRole,
    label: &'static str,
    body: &'static str,
}

const ITEMS: [ScrollItem; ITEM_COUNT] = [
    ScrollItem { role: UiCardRole::Browser, label: "Entry A", body: "VISIBLE" },
    ScrollItem { role: UiCardRole::Editor, label: "Entry B", body: "VISIBLE" },
    ScrollItem { role: UiCardRole::Preview, label: "Entry C", body: "VISIBLE" },
    ScrollItem { role: UiCardRole::Inspector, label: "Entry D", body: "VISIBLE" },
    ScrollItem { role: UiCardRole::Status, label: "Entry E", body: "VISIBLE" },
    ScrollItem { role: UiCardRole::Browser, label: "Entry F", body: "VISIBLE" },
    ScrollItem { role: UiCardRole::Editor, label: "Entry G", body: "VISIBLE" },
    ScrollItem { role: UiCardRole::Preview, label: "Entry H", body: "VISIBLE" },
    ScrollItem { role: UiCardRole::Inspector, label: "Entry I", body: "VISIBLE" },
    ScrollItem { role: UiCardRole::Status, label: "Entry J", body: "VISIBLE" },
];

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig { title: "Tokimu Hello UI Scroll".into(), width: 1260, height: 820 },
        HelloUiScrollApp::new(),
    )
}

struct HelloUiScrollApp {
    renderer: Option<WgpuBackend>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    theme: UiTheme,
    scroll: UiVerticalScroll,
    target_offset: f32,
    selected_index: usize,
    hovered_index: Option<usize>,
}

impl Default for HelloUiScrollApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            theme: UiTheme::default(),
            scroll: UiVerticalScroll::new(UiRect::new([0.0, 0.0], [1.0, 1.0]), 2.2),
            target_offset: 0.0,
            selected_index: 0,
            hovered_index: None,
        }
    }
}

impl HelloUiScrollApp {
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

    fn draw_surface(
        renderer: &mut WgpuBackend,
        pipeline: PipelineHandle,
        command: &UiSurfaceCommand,
        viewport: Option<ViewportRect>,
    ) {
        let rect = command.rect;
        if matches!(command.style.elevation, ui_tools::UiElevation::Raised | ui_tools::UiElevation::Floating) {
            let shadow_rect = UiRect::new([rect.center[0] + 0.01, rect.center[1] - 0.01], [rect.size[0], rect.size[1]]);
            renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: REGION_MESH,
                material: MUTED_MATERIAL,
                pipeline,
                instance: Instance2d::new(shadow_rect.center, shadow_rect.size, 0.0),
                camera: Some(CAMERA_HANDLE),
                viewport,
            })]);
        }

        renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: REGION_MESH,
            material: Self::material_for_role(command.style.role),
            pipeline,
            instance: Instance2d::new(rect.center, rect.size, 0.0),
            camera: Some(CAMERA_HANDLE),
            viewport,
        })]);

        if let Some(border_role) = command.style.border_role {
            let border = command.style.border_width.min(rect.size[0] * 0.22).min(rect.size[1] * 0.22);
            if border > 0.0 {
                let border_rect = UiRect::new([rect.center[0], rect.center[1] + rect.size[1] * 0.5 - border * 0.5], [rect.size[0], border]);
                renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: REGION_MESH,
                    material: Self::material_for_role(border_role),
                    pipeline,
                    instance: Instance2d::new(border_rect.center, border_rect.size, 0.0),
                    camera: Some(CAMERA_HANDLE),
                    viewport,
                })]);
            }
        }
    }

    fn screen_to_world(&self, x: f32, y: f32) -> [f32; 2] {
        let width = self.window_size[0].max(1.0);
        let height = self.window_size[1].max(1.0);
        let half_height = 1.0;
        let half_width = half_height * (width / height);
        [
            (x / width) * (half_width * 2.0) - half_width,
            half_height - (y / height) * (half_height * 2.0),
        ]
    }

    fn viewport_rect(&self) -> (UiRect, ViewportRect) {
        let px = ViewportRect { x: self.window_size[0] * 0.18, y: self.window_size[1] * 0.16, width: self.window_size[0] * 0.64, height: self.window_size[1] * 0.66 };
        let world = UiRect::new(
            self.screen_to_world(px.x + px.width * 0.5, px.y + px.height * 0.5),
            [
                (px.width / self.window_size[0]) * (self.window_size[0] / self.window_size[1]).max(1.0) * 2.0,
                (px.height / self.window_size[1]) * 2.0,
            ],
        );
        (world, px)
    }

    fn rebuild_scroll_target(&mut self) {
        self.target_offset = self.target_offset.clamp(0.0, self.scroll.max_offset());
    }

    fn sync_scroll_view(&mut self, viewport: UiRect) {
        self.scroll.set_viewport(viewport);
        self.scroll.set_content_extent(0.22 * ITEMS.len() as f32);
        self.rebuild_scroll_target();
    }

    fn surface_role_for_item(role: UiCardRole) -> UiSurfaceRole {
        match role {
            UiCardRole::Browser => UiSurfaceRole::Card,
            UiCardRole::Editor => UiSurfaceRole::Raised,
            UiCardRole::Preview => UiSurfaceRole::Accent,
            UiCardRole::Inspector => UiSurfaceRole::Panel,
            UiCardRole::Status => UiSurfaceRole::Overlay,
            UiCardRole::Selected => UiSurfaceRole::Selected,
        }
    }
}

impl PlatformEventHandler for HelloUiScrollApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(REGION_MESH, &Mesh::quad());
        renderer.upload_material(BACKDROP_MATERIAL, &Material::new("ui-scroll-backdrop", Color::rgb(0.05, 0.06, 0.08)))?;
        renderer.upload_material(SURFACE_MATERIAL, &Material::new("ui-scroll-surface", Color::rgb(0.18, 0.20, 0.25)))?;
        renderer.upload_material(PANEL_MATERIAL, &Material::new("ui-scroll-panel", Color::rgb(0.14, 0.16, 0.20)))?;
        renderer.upload_material(CARD_MATERIAL, &Material::new("ui-scroll-card", Color::rgb(0.22, 0.24, 0.30)))?;
        renderer.upload_material(ACTIVE_MATERIAL, &Material::new("ui-scroll-active", Color::rgb(0.34, 0.56, 0.86)))?;
        renderer.upload_material(MUTED_MATERIAL, &Material::new("ui-scroll-muted", Color::rgb(0.10, 0.12, 0.14)))?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new("hello-ui-scroll-pipeline", PipelineKind::SolidColor2d))?;
        self.renderer = Some(renderer);
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        match event {
            PlatformInputEvent::KeyboardInput { key, pressed: true } => match key {
                KeyCode::ArrowUp => self.target_offset -= 0.22,
                KeyCode::ArrowDown => self.target_offset += 0.22,
                KeyCode::ArrowLeft => self.selected_index = self.selected_index.saturating_sub(1),
                KeyCode::ArrowRight => self.selected_index = (self.selected_index + 1) % ITEMS.len(),
                KeyCode::Space => self.target_offset = 0.0,
                _ => {}
            },
            PlatformInputEvent::CursorMoved { x, y } => {
                let layout = self.viewport_rect();
                self.sync_scroll_view(layout.0);
                let point = self.screen_to_world(x, y);
                let content_top = layout.0.center[1] + layout.0.size[1] * 0.5 - 0.16;
                self.hovered_index = ITEMS.iter().enumerate().find_map(|(index, _)| {
                    let item_y = content_top - index as f32 * 0.22;
                    let content_rect = UiRect::new(
                        [layout.0.center[0], item_y],
                        [layout.0.size[0] * 0.90, 0.18],
                    );
                    self.scroll.hit_test(content_rect, point).then_some(index)
                });
            }
            PlatformInputEvent::MouseInput { button: MouseButton::Left, pressed: true } => {
                if let Some(index) = self.hovered_index { self.selected_index = index; }
            }
            PlatformInputEvent::Resized { width, height } => {
                self.window_size = [width.max(1) as f32, height.max(1) as f32];
                if let Some(renderer) = self.renderer.as_mut() { renderer.resize_surface(width, height); }
            }
            _ => {}
        }
        self.rebuild_scroll_target();
        Ok(())
    }

    fn on_frame(&mut self, delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let step = (delta_seconds as f32 * 8.0).min(1.0);
        let (viewport_world, viewport_px) = self.viewport_rect();
        self.sync_scroll_view(viewport_world);
        let offset = self.scroll.offset();
        self.scroll
            .set_offset(offset + (self.target_offset - offset) * step);
        let height = self.window_size[1].max(1.0);
        let thumb_height = viewport_px.height * 0.24;
        let scroll_ratio = if self.scroll.max_offset() > 0.0 {
            self.scroll.offset() / self.scroll.max_offset()
        } else {
            0.0
        };
        let thumb_y = viewport_px.y
            + (viewport_px.height - thumb_height) * scroll_ratio;
        let thumb_center = self.screen_to_world(viewport_px.x + viewport_px.width + 18.0, thumb_y + thumb_height * 0.5);
        let thumb_size = [0.08, (thumb_height / height) * 2.0];

        let Some(renderer) = self.renderer.as_mut() else { return Ok(FrameOutcome::Continue); };

        renderer.upload_camera(CAMERA_HANDLE, Camera::orthographic_2d(self.window_size[0], self.window_size[1]));
        renderer.begin_frame();
        renderer.submit(&[RenderCommand::Clear(ClearCommand { color: Color::rgb(0.05, 0.06, 0.08) })]);

        let mut global_surfaces = Vec::new();
        let mut global_text = Vec::new();
        let mut content_surfaces = Vec::new();
        let mut content_text = Vec::new();
        {
            let theme = UiTheme::default();
            let mut drawer = UiDrawer::new(&mut global_surfaces, &mut global_text, &theme);
            drawer.surface(&UiRegion::panel(UiRect::new([0.0, 0.62], [viewport_world.size[0] * 1.08, 0.14])));
            drawer.surface(&UiRegion::panel(UiRect::new([0.0, -0.72], [viewport_world.size[0] * 1.08, 0.10])));
            drawer.surface(&UiRegion::panel(viewport_world));
        }

        let mut content_drawer = UiDrawer::new(&mut content_surfaces, &mut content_text, &self.theme);
        let content_top = viewport_world.center[1] + viewport_world.size[1] * 0.5 - 0.16;
        let item_spacing = 0.22;
        for (index, item) in ITEMS.iter().enumerate() {
            let content_rect = UiRect::new(
                [viewport_world.center[0], content_top - index as f32 * item_spacing],
                [viewport_world.size[0] * 0.90, 0.18],
            );
            if self.scroll.visible_rect(content_rect).is_none() {
                continue;
            }
            let rect = self.scroll.content_rect(content_rect);
            let mut card = UiCard::new(item.role, item.label, item.body, UiRegion::card(rect));
            card.surface_role = if index == self.selected_index {
                UiSurfaceRole::Selected
            } else if Some(index) == self.hovered_index {
                UiSurfaceRole::Accent
            } else {
                Self::surface_role_for_item(item.role)
            };
            content_drawer.card(&card);
        }

        for command in global_surfaces {
            Self::draw_surface(renderer, self.pipeline, &command, None);
        }
        for command in content_surfaces {
            Self::draw_surface(renderer, self.pipeline, &command, Some(viewport_px));
        }

        let thumb = UiSurfaceCommand { rect: UiRect::new([viewport_world.center[0] + viewport_world.size[0] * 0.52, viewport_world.center[1]], [0.06, viewport_world.size[1] * 0.74]), style: self.theme.surface(UiSurfaceRole::Overlay) };
        Self::draw_surface(renderer, self.pipeline, &thumb, Some(viewport_px));
        let thumb_rect = DrawMeshCommand { mesh: REGION_MESH, material: ACTIVE_MATERIAL, pipeline: self.pipeline, instance: Instance2d::new(thumb_center, thumb_size, 0.0), camera: Some(CAMERA_HANDLE), viewport: None };
        renderer.submit(&[RenderCommand::DrawMesh(thumb_rect)]);

        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}