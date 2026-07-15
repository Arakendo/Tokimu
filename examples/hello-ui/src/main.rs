use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, KeyCode, Material, MaterialHandle, Mesh, MeshHandle, MouseButton, NativeWindow,
    Pipeline, PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};

const QUAD_MESH: MeshHandle = MeshHandle(1);
const DIAMOND_MESH: MeshHandle = MeshHandle(2);
const TRIANGLE_MESH: MeshHandle = MeshHandle(3);
const UI_CAMERA: CameraHandle = CameraHandle(1);

const BACKDROP_MATERIAL: MaterialHandle = MaterialHandle(1);
const SHADOW_MATERIAL: MaterialHandle = MaterialHandle(2);
const SURFACE_MATERIAL: MaterialHandle = MaterialHandle(3);
const SURFACE_ALT_MATERIAL: MaterialHandle = MaterialHandle(4);
const CYAN_MATERIAL: MaterialHandle = MaterialHandle(5);
const AMBER_MATERIAL: MaterialHandle = MaterialHandle(6);
const CORAL_MATERIAL: MaterialHandle = MaterialHandle(7);
const LIME_MATERIAL: MaterialHandle = MaterialHandle(8);
const ROSE_MATERIAL: MaterialHandle = MaterialHandle(9);
const WHITE_MATERIAL: MaterialHandle = MaterialHandle(10);

const TAB_NAMES: [&str; 3] = ["overview", "layout", "inspect"];
const FILTER_NAMES: [&str; 4] = ["grid", "focus", "dense", "calm"];
const CARD_NAMES: [&str; 3] = ["signal", "state", "timing"];
const MODE_NAMES: [&str; 2] = ["balanced", "dense"];
const ACTIVE_ACCENTS: [MaterialHandle; 3] = [CYAN_MATERIAL, AMBER_MATERIAL, CORAL_MATERIAL];
const FILTER_ACCENTS: [MaterialHandle; 4] = [LIME_MATERIAL, CYAN_MATERIAL, AMBER_MATERIAL, ROSE_MATERIAL];

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello UI".into(),
            width: 1360,
            height: 820,
        },
        HelloUiApp::new(),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UiZone {
    Tab(usize),
    Filter(usize),
    Card(usize),
    Toggle,
    Chart,
    Shell,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UiDensityMode {
    Balanced,
    Dense,
}

#[derive(Clone, Copy)]
struct UiRect {
    center: [f32; 2],
    size: [f32; 2],
}

impl UiRect {
    fn new(center: [f32; 2], size: [f32; 2]) -> Self {
        Self { center, size }
    }

    fn contains(&self, point: [f32; 2]) -> bool {
        let half_width = self.size[0] * 0.5;
        let half_height = self.size[1] * 0.5;
        point[0] >= self.center[0] - half_width
            && point[0] <= self.center[0] + half_width
            && point[1] >= self.center[1] - half_height
            && point[1] <= self.center[1] + half_height
    }

    fn offset(self, offset: [f32; 2]) -> Self {
        Self {
            center: [self.center[0] + offset[0], self.center[1] + offset[1]],
            ..self
        }
    }
}

struct UiLayout {
    shell: UiRect,
    header: UiRect,
    left_rail: UiRect,
    main: UiRect,
    right_rail: UiRect,
    tabs: [UiRect; 3],
    filters: [UiRect; 4],
    cards: [UiRect; 3],
    chart: UiRect,
    toggle: UiRect,
}

impl UiLayout {
    fn new(window_size: [f32; 2], mode: UiDensityMode) -> Self {
        let width = window_size[0].max(1.0);
        let height = window_size[1].max(1.0);
        let half_height = 1.0;
        let half_width = half_height * (width / height);
        let (side_width, gutter, header_height, rail_height, chart_height, card_height, filter_step) = match mode {
            UiDensityMode::Balanced => (0.62, 0.10, 0.20, 1.42, 0.46, 0.28, 0.24),
            UiDensityMode::Dense => (0.54, 0.07, 0.16, 1.30, 0.38, 0.23, 0.20),
        };
        let main_width = (half_width * 2.0 - side_width * 2.0 - gutter * 4.0).max(1.10);

        let shell = UiRect::new([0.0, 0.0], [half_width * 1.96, half_height * 1.74]);
        let header = UiRect::new([0.0, half_height - if matches!(mode, UiDensityMode::Dense) { 0.12 } else { 0.15 }], [half_width * 1.76, header_height]);
        let left_rail = UiRect::new(
            [-half_width + side_width * 0.5 + gutter, -0.03],
            [side_width, rail_height],
        );
        let main = UiRect::new([0.0, -0.03], [main_width, 1.42]);
        let right_rail = UiRect::new(
            [half_width - side_width * 0.5 - gutter, -0.03],
            [side_width, rail_height],
        );

        let tab_width = ((main_width - 0.18) / 3.0).max(if matches!(mode, UiDensityMode::Dense) { 0.35 } else { 0.42 });
        let tab_y = half_height - if matches!(mode, UiDensityMode::Dense) { 0.24 } else { 0.30 };
        let tabs = [
            UiRect::new([-main_width * 0.33, tab_y], [tab_width, 0.11]),
            UiRect::new([0.0, tab_y], [tab_width, 0.11]),
            UiRect::new([main_width * 0.33, tab_y], [tab_width, 0.11]),
        ];

        let filter_width = side_width - if matches!(mode, UiDensityMode::Dense) { 0.08 } else { 0.12 };
        let filters = [
            UiRect::new([left_rail.center[0], 0.42], [filter_width, 0.16]),
            UiRect::new([left_rail.center[0], 0.42 - filter_step], [filter_width, 0.16]),
            UiRect::new([left_rail.center[0], 0.42 - filter_step * 2.0], [filter_width, 0.16]),
            UiRect::new([left_rail.center[0], 0.42 - filter_step * 3.0], [filter_width, 0.16]),
        ];

        let cards = match mode {
            UiDensityMode::Balanced => {
                let card_width = ((main_width - 0.18) / 3.0).max(0.35);
                let card_y = 0.34;
                [
                    UiRect::new([-main_width * 0.33, card_y], [card_width, card_height]),
                    UiRect::new([0.0, card_y], [card_width, card_height]),
                    UiRect::new([main_width * 0.33, card_y], [card_width, card_height]),
                ]
            }
            UiDensityMode::Dense => {
                let card_width = main_width - 0.24;
                [
                    UiRect::new([0.0, 0.44], [card_width, 0.18]),
                    UiRect::new([0.0, 0.10], [card_width, 0.18]),
                    UiRect::new([0.0, -0.24], [card_width, 0.18]),
                ]
            }
        };

        let chart = UiRect::new([0.0, if matches!(mode, UiDensityMode::Dense) { -0.44 } else { -0.36 }], [main_width - 0.18, chart_height]);
        let toggle = UiRect::new([right_rail.center[0], if matches!(mode, UiDensityMode::Dense) { 0.24 } else { 0.30 }], [side_width - if matches!(mode, UiDensityMode::Dense) { 0.08 } else { 0.12 }, 0.22]);

        Self {
            shell,
            header,
            left_rail,
            main,
            right_rail,
            tabs,
            filters,
            cards,
            chart,
            toggle,
        }
    }
}

struct HelloUiApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    elapsed_seconds: f64,
    active_tab: usize,
    selected_filter: usize,
    selected_card: usize,
    detail_mode: bool,
    density_mode: UiDensityMode,
    mouse_down: bool,
    cursor_position: [f32; 2],
    hovered_zone: UiZone,
    pipeline: PipelineHandle,
}

impl Default for HelloUiApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            elapsed_seconds: 0.0,
            active_tab: 0,
            selected_filter: 0,
            selected_card: 0,
            detail_mode: true,
            density_mode: UiDensityMode::Balanced,
            mouse_down: false,
            cursor_position: [0.0, 0.0],
            hovered_zone: UiZone::Shell,
            pipeline: PipelineHandle(0),
        }
    }
}

impl HelloUiApp {
    fn new() -> Self {
        Self::default()
    }

    fn layout(&self) -> UiLayout {
        UiLayout::new(self.window_size, self.density_mode)
    }

    fn active_accent(&self) -> MaterialHandle {
        ACTIVE_ACCENTS[self.active_tab % ACTIVE_ACCENTS.len()]
    }

    fn cursor_world(&self) -> [f32; 2] {
        let width = self.window_size[0].max(1.0);
        let height = self.window_size[1].max(1.0);
        let half_height = 1.0;
        let half_width = half_height * (width / height);
        let x = (self.cursor_position[0] / width) * (half_width * 2.0) - half_width;
        let y = half_height - (self.cursor_position[1] / height) * (half_height * 2.0);
        [x, y]
    }

    fn zone_at_cursor(&self) -> UiZone {
        self.zone_at_point(self.cursor_world())
    }

    fn zone_at_point(&self, point: [f32; 2]) -> UiZone {
        let layout = self.layout();

        for (index, rect) in layout.tabs.iter().enumerate() {
            if rect.contains(point) {
                return UiZone::Tab(index);
            }
        }

        for (index, rect) in layout.filters.iter().enumerate() {
            if rect.contains(point) {
                return UiZone::Filter(index);
            }
        }

        for (index, rect) in layout.cards.iter().enumerate() {
            if rect.contains(point) {
                return UiZone::Card(index);
            }
        }

        if layout.toggle.contains(point) {
            return UiZone::Toggle;
        }

        if layout.chart.contains(point) {
            return UiZone::Chart;
        }

        if layout.main.contains(point) || layout.shell.contains(point) {
            return UiZone::Shell;
        }

        UiZone::Shell
    }

    fn cycle_tab(&mut self, step: isize) {
        let count = TAB_NAMES.len() as isize;
        self.active_tab = ((self.active_tab as isize + step).rem_euclid(count)) as usize;
    }

    fn cycle_filter(&mut self, step: isize) {
        let count = FILTER_NAMES.len() as isize;
        self.selected_filter = ((self.selected_filter as isize + step).rem_euclid(count)) as usize;
    }

    fn activate_zone(&mut self, zone: UiZone) {
        match zone {
            UiZone::Tab(index) => self.active_tab = index,
            UiZone::Filter(index) => self.selected_filter = index,
            UiZone::Card(index) => self.selected_card = index,
            UiZone::Toggle => self.detail_mode = !self.detail_mode,
            UiZone::Chart => self.selected_card = (self.selected_card + 1) % CARD_NAMES.len(),
            UiZone::Shell => {}
        }
    }

    fn zone_label(&self, zone: UiZone) -> &'static str {
        match zone {
            UiZone::Tab(index) => TAB_NAMES[index],
            UiZone::Filter(index) => FILTER_NAMES[index],
            UiZone::Card(index) => CARD_NAMES[index],
            UiZone::Toggle => "toggle",
            UiZone::Chart => "chart",
            UiZone::Shell => "shell",
        }
    }

    fn update_window_title(&self) {
        if let Some(window) = self.window.as_ref() {
            window.set_title(&format!(
                "Tokimu Hello UI | mode={} | tab={} | filter={} | focus={} | hover={} | detail={} | mouse={} | cursor=({:.0}, {:.0})",
                MODE_NAMES[self.density_mode as usize],
                TAB_NAMES[self.active_tab],
                FILTER_NAMES[self.selected_filter],
                CARD_NAMES[self.selected_card],
                self.zone_label(self.hovered_zone),
                if self.detail_mode { "on" } else { "off" },
                if self.mouse_down { "down" } else { "up" },
                self.cursor_position[0],
                self.cursor_position[1],
            ));
        }
    }

    fn toggle_density_mode(&mut self) {
        self.density_mode = match self.density_mode {
            UiDensityMode::Balanced => UiDensityMode::Dense,
            UiDensityMode::Dense => UiDensityMode::Balanced,
        };
    }

    fn render_scene(&mut self) -> PlatformResult<FrameOutcome> {
        let elapsed = self.elapsed_seconds as f32;
        let layout = self.layout();
        let active_accent = self.active_accent();
        let hovered_zone = self.hovered_zone;
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        renderer.upload_mesh(QUAD_MESH, &Mesh::quad());
        renderer.upload_mesh(DIAMOND_MESH, &Mesh::diamond());
        renderer.upload_mesh(TRIANGLE_MESH, &Mesh::triangle());

        let camera = Camera::orthographic_2d_with_height(self.window_size[0], self.window_size[1], 2.0);
        renderer.upload_camera(UI_CAMERA, camera);
        renderer.set_active_camera(UI_CAMERA);

        let mut commands = vec![RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.04, 0.05, 0.08),
        })];

        draw_panel(
            &mut commands,
            self.pipeline,
            QUAD_MESH,
            layout.shell,
            BACKDROP_MATERIAL,
            SHADOW_MATERIAL,
            active_accent,
            0.020,
        );
        draw_panel(
            &mut commands,
            self.pipeline,
            QUAD_MESH,
            layout.header,
            SURFACE_MATERIAL,
            SHADOW_MATERIAL,
            active_accent,
            0.018,
        );
        draw_panel(
            &mut commands,
            self.pipeline,
            QUAD_MESH,
            layout.left_rail,
            SURFACE_MATERIAL,
            SHADOW_MATERIAL,
            FILTER_ACCENTS[self.selected_filter],
            0.016,
        );
        draw_panel(
            &mut commands,
            self.pipeline,
            QUAD_MESH,
            layout.main,
            SURFACE_ALT_MATERIAL,
            SHADOW_MATERIAL,
            active_accent,
            0.016,
        );
        draw_panel(
            &mut commands,
            self.pipeline,
            QUAD_MESH,
            layout.right_rail,
            SURFACE_MATERIAL,
            SHADOW_MATERIAL,
            if self.detail_mode { LIME_MATERIAL } else { ROSE_MATERIAL },
            0.016,
        );

        for (index, tab_rect) in layout.tabs.iter().enumerate() {
            let fill = if index == self.active_tab {
                active_accent
            } else if hovered_zone == UiZone::Tab(index) {
                SURFACE_ALT_MATERIAL
            } else {
                SURFACE_MATERIAL
            };
            draw_chip(&mut commands, self.pipeline, QUAD_MESH, *tab_rect, fill, SHADOW_MATERIAL);
            if index == self.active_tab {
                let marker = UiRect::new([tab_rect.center[0], tab_rect.center[1] + 0.08], [0.08, 0.08]);
                draw_marker(&mut commands, self.pipeline, TRIANGLE_MESH, marker, active_accent, 0.0);
            }
        }

        for (index, filter_rect) in layout.filters.iter().enumerate() {
            let fill = if index == self.selected_filter {
                FILTER_ACCENTS[index]
            } else if hovered_zone == UiZone::Filter(index) {
                SURFACE_ALT_MATERIAL
            } else {
                SURFACE_MATERIAL
            };
            draw_chip(&mut commands, self.pipeline, QUAD_MESH, *filter_rect, fill, SHADOW_MATERIAL);
            let indicator = UiRect::new([
                filter_rect.center[0] - filter_rect.size[0] * 0.32,
                filter_rect.center[1],
            ], [0.06, 0.06]);
            draw_marker(
                &mut commands,
                self.pipeline,
                DIAMOND_MESH,
                indicator,
                if index == self.selected_filter { WHITE_MATERIAL } else { FILTER_ACCENTS[index] },
                elapsed * 0.7 + index as f32 * 0.2,
            );
        }

        for (index, card_rect) in layout.cards.iter().enumerate() {
            let active = index == self.selected_card;
            let hovered = hovered_zone == UiZone::Card(index);
            let fill = if active {
                active_accent
            } else if hovered {
                SURFACE_ALT_MATERIAL
            } else {
                SURFACE_MATERIAL
            };
            draw_panel(
                &mut commands,
                self.pipeline,
                QUAD_MESH,
                *card_rect,
                fill,
                SHADOW_MATERIAL,
                active_accent,
                0.030,
            );

            let top_bar = UiRect::new(
                [card_rect.center[0], card_rect.center[1] + card_rect.size[1] * 0.13],
                [card_rect.size[0] - 0.06, 0.04],
            );
            draw_rect(&mut commands, self.pipeline, QUAD_MESH, top_bar, active_accent, 0.0);

            let knob = UiRect::new(
                [card_rect.center[0] - card_rect.size[0] * 0.32, card_rect.center[1] - card_rect.size[1] * 0.14],
                [0.08, 0.08],
            );
            draw_marker(
                &mut commands,
                self.pipeline,
                DIAMOND_MESH,
                knob,
                if active { WHITE_MATERIAL } else { active_accent },
                elapsed * 1.2 + index as f32 * 0.3,
            );

            let meter_base_y = card_rect.center[1] - card_rect.size[1] * 0.10;
            for meter in 0..3 {
                let phase = elapsed * 0.8 + index as f32 * 0.4 + meter as f32 * 0.8;
                let value = 0.35 + 0.35 * phase.sin().abs();
                let bar_height = 0.04 + value * 0.08;
                let bar = UiRect::new(
                    [card_rect.center[0] - 0.12 + meter as f32 * 0.12, meter_base_y],
                    [0.07, bar_height],
                );
                draw_rect(&mut commands, self.pipeline, QUAD_MESH, bar, active_accent, 0.0);
            }
        }

        draw_panel(
            &mut commands,
            self.pipeline,
            QUAD_MESH,
            layout.chart,
            SURFACE_MATERIAL,
            SHADOW_MATERIAL,
            active_accent,
            0.026,
        );

        let bar_count = if self.detail_mode { 12 } else { 8 };
        let bar_step = layout.chart.size[0] / bar_count as f32;
        let bar_left = layout.chart.center[0] - layout.chart.size[0] * 0.5 + bar_step * 0.5;
        for index in 0..bar_count {
            let phase = elapsed * 1.15
                + self.active_tab as f32 * 0.6
                + self.selected_filter as f32 * 0.35
                + index as f32 * 0.45;
            let value = 0.20 + 0.68 * phase.sin().abs();
            let height = layout.chart.size[1] * value.max(0.20);
            let bar = UiRect::new(
                [bar_left + index as f32 * bar_step, layout.chart.center[1] - layout.chart.size[1] * 0.20 + height * 0.5],
                [bar_step * 0.46, height],
            );
            let bar_material = if index % 3 == self.selected_filter {
                active_accent
            } else {
                SURFACE_ALT_MATERIAL
            };
            draw_rect(&mut commands, self.pipeline, QUAD_MESH, bar, bar_material, 0.0);
        }

        let chart_marker = UiRect::new(
            [layout.chart.center[0] + layout.chart.size[0] * 0.36, layout.chart.center[1] + layout.chart.size[1] * 0.26],
            [0.10, 0.10],
        );
        draw_marker(&mut commands, self.pipeline, DIAMOND_MESH, chart_marker, WHITE_MATERIAL, elapsed * 1.5);

        draw_panel(
            &mut commands,
            self.pipeline,
            QUAD_MESH,
            layout.toggle,
            SURFACE_MATERIAL,
            SHADOW_MATERIAL,
            if self.detail_mode { LIME_MATERIAL } else { ROSE_MATERIAL },
            0.024,
        );
        let track = UiRect::new(
            [layout.toggle.center[0], layout.toggle.center[1] - 0.03],
            [layout.toggle.size[0] - 0.14, 0.08],
        );
        draw_rect(&mut commands, self.pipeline, QUAD_MESH, track, SHADOW_MATERIAL, 0.0);
        let knob_x = if self.detail_mode {
            track.center[0] + track.size[0] * 0.25
        } else {
            track.center[0] - track.size[0] * 0.25
        };
        draw_marker(
            &mut commands,
            self.pipeline,
            DIAMOND_MESH,
            UiRect::new([knob_x, track.center[1]], [0.11, 0.11]),
            if self.detail_mode { LIME_MATERIAL } else { ROSE_MATERIAL },
            elapsed * 1.1,
        );

        renderer.begin_frame();
        renderer.submit(&commands);
        let _ = renderer.present()?;
        self.update_window_title();
        Ok(FrameOutcome::Continue)
    }
}

impl PlatformEventHandler for HelloUiApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_material(BACKDROP_MATERIAL, &Material::new("ui-backdrop", Color::rgb(0.06, 0.08, 0.11)))?;
        renderer.upload_material(SHADOW_MATERIAL, &Material::new("ui-shadow", Color::rgb(0.02, 0.03, 0.05)))?;
        renderer.upload_material(SURFACE_MATERIAL, &Material::new("ui-surface", Color::rgb(0.11, 0.14, 0.19)))?;
        renderer.upload_material(SURFACE_ALT_MATERIAL, &Material::new("ui-surface-alt", Color::rgb(0.15, 0.18, 0.25)))?;
        renderer.upload_material(CYAN_MATERIAL, &Material::new("ui-cyan", Color::rgb(0.30, 0.82, 0.96)))?;
        renderer.upload_material(AMBER_MATERIAL, &Material::new("ui-amber", Color::rgb(0.96, 0.78, 0.31)))?;
        renderer.upload_material(CORAL_MATERIAL, &Material::new("ui-coral", Color::rgb(0.95, 0.48, 0.55)))?;
        renderer.upload_material(LIME_MATERIAL, &Material::new("ui-lime", Color::rgb(0.44, 0.92, 0.62)))?;
        renderer.upload_material(ROSE_MATERIAL, &Material::new("ui-rose", Color::rgb(0.89, 0.42, 0.78)))?;
        renderer.upload_material(WHITE_MATERIAL, &Material::new("ui-white", Color::rgb(0.95, 0.96, 0.98)))?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new("hello-ui-pipeline", PipelineKind::SolidColor2d))?;
        self.renderer = Some(renderer);
        self.hovered_zone = self.zone_at_cursor();
        self.update_window_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::CloseRequested = event {
            return Ok(());
        }

        if let PlatformInputEvent::CursorMoved { x, y } = event {
            self.cursor_position = [x as f32, y as f32];
            self.hovered_zone = self.zone_at_cursor();
            self.update_window_title();
        }

        if let PlatformInputEvent::MouseInput { button, pressed } = event {
            if button == MouseButton::Left {
                self.mouse_down = pressed;
                if pressed {
                    let zone = self.zone_at_cursor();
                    self.activate_zone(zone);
                    self.hovered_zone = zone;
                }
                self.update_window_title();
            }
        }

        if let PlatformInputEvent::KeyboardInput { key, pressed } = event {
            if pressed {
                match key {
                    KeyCode::ArrowLeft => self.cycle_tab(-1),
                    KeyCode::ArrowRight => self.cycle_tab(1),
                    KeyCode::ArrowUp => self.cycle_filter(-1),
                    KeyCode::ArrowDown => self.cycle_filter(1),
                    KeyCode::Space => self.toggle_density_mode(),
                    _ => {}
                }
                self.hovered_zone = self.zone_at_cursor();
                self.update_window_title();
            }
        }

        if let PlatformInputEvent::Resized { width, height } = event {
            self.window_size = [width.max(1) as f32, height.max(1) as f32];
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize_surface(width, height);
            }
        }

        Ok(())
    }

    fn on_frame(&mut self, delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        self.elapsed_seconds += delta_seconds;
        self.hovered_zone = self.zone_at_cursor();
        self.render_scene()
    }
}

fn draw_panel(
    commands: &mut Vec<RenderCommand>,
    pipeline: PipelineHandle,
    mesh: MeshHandle,
    rect: UiRect,
    fill: MaterialHandle,
    shadow: MaterialHandle,
    accent: MaterialHandle,
    strip_height: f32,
) {
    draw_rect(commands, pipeline, mesh, rect.offset([0.018, -0.018]), shadow, 0.0);
    draw_rect(commands, pipeline, mesh, rect, fill, 0.0);
    let strip = UiRect::new(
        [rect.center[0], rect.center[1] + rect.size[1] * 0.5 - strip_height * 0.5],
        [rect.size[0], strip_height],
    );
    draw_rect(commands, pipeline, mesh, strip, accent, 0.0);
}

fn draw_chip(
    commands: &mut Vec<RenderCommand>,
    pipeline: PipelineHandle,
    mesh: MeshHandle,
    rect: UiRect,
    fill: MaterialHandle,
    shadow: MaterialHandle,
) {
    draw_rect(commands, pipeline, mesh, rect.offset([0.015, -0.015]), shadow, 0.0);
    draw_rect(commands, pipeline, mesh, rect, fill, 0.0);
}

fn draw_marker(
    commands: &mut Vec<RenderCommand>,
    pipeline: PipelineHandle,
    mesh: MeshHandle,
    rect: UiRect,
    fill: MaterialHandle,
    rotation: f32,
) {
    draw_rect(commands, pipeline, mesh, rect, fill, rotation);
}

fn draw_rect(
    commands: &mut Vec<RenderCommand>,
    pipeline: PipelineHandle,
    mesh: MeshHandle,
    rect: UiRect,
    material: MaterialHandle,
    rotation: f32,
) {
    commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
        mesh,
        material,
        pipeline,
        instance: Instance2d::identity()
            .with_translation(rect.center)
            .with_scale(rect.size)
            .with_rotation(rotation),
        camera: Some(UI_CAMERA),
        viewport: None,
    }));
}
