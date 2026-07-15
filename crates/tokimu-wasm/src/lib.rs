#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use tokimu_core::World;

#[cfg(target_arch = "wasm32")]
use tokimu_input::{InputState, KeyCode, MouseButton};

#[cfg(target_arch = "wasm32")]
use tokimu_platform::{wasm::install_browser_input_bridge, WindowConfig};

#[cfg(target_arch = "wasm32")]
use tokimu_runtime::{
    advance_field_sprint, App, FieldSprintState, Plugin, RunLoopSummary, FIELD_SPRINT_TARGET_POINTS,
};

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{closure::Closure, JsCast};

#[cfg(target_arch = "wasm32")]
use web_sys::{
    window, CanvasRenderingContext2d, Document, Element, HtmlCanvasElement, HtmlElement, Window,
};

pub fn boot_message() -> &'static str {
    "Tokimu WASM runtime app ready"
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    boot_browser_demo()
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn boot_browser_demo() -> Result<(), JsValue> {
    boot_browser_bridge()
}

#[cfg(target_arch = "wasm32")]
fn boot_browser_bridge() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = window().ok_or_else(|| JsValue::from_str("browser window is unavailable"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("browser document is unavailable"))?;
    let canvas = ensure_canvas(&document, &WindowConfig::default())?;
    let context = canvas
        .get_context("2d")
        .map_err(|error| JsValue::from(error))?
        .ok_or_else(|| JsValue::from_str("2D canvas context is unavailable"))?
        .dyn_into::<CanvasRenderingContext2d>()
        .map_err(|_| JsValue::from_str("failed to acquire 2D canvas context"))?;
    let status = document
        .get_element_by_id("status")
        .ok_or_else(|| JsValue::from_str("status element is unavailable"))?
        .dyn_into::<HtmlElement>()
        .map_err(|_| JsValue::from_str("status element is not an HtmlElement"))?;
    let hud = BrowserHud::from_document(&document)?;
    let app = Rc::new(RefCell::new(App::default()));
    {
        let mut app = app.borrow_mut();
        app.add_plugin(&BrowserDemoPlugin);
    }

    install_browser_input_bridge(WindowConfig::default(), {
        let app = app.clone();
        move |event| {
            let last_event = format!("{:?}", event);
            let mut app = app.borrow_mut();

            if let Some(input_event) = event.as_input_event() {
                app.apply_input_event(input_event);
            }

            if let Some(scene) = app.world.resource_mut::<BrowserScene>() {
                scene.last_event = last_event;
            }
        }
    })
    .map_err(|error| JsValue::from_str(&error.to_string()))?;

    start_browser_loop(window, canvas, context, status, hud, app)
}

#[cfg(target_arch = "wasm32")]
fn ensure_canvas(document: &Document, config: &WindowConfig) -> Result<HtmlCanvasElement, JsValue> {
    if let Some(element) = document.get_element_by_id("tokimu-canvas") {
        return element
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| JsValue::from_str("tokimu-canvas exists but is not a canvas element"));
    }

    let canvas = document
        .create_element("canvas")
        .map_err(JsValue::from)?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| JsValue::from_str("failed to create a canvas element"))?;
    canvas.set_id("tokimu-canvas");
    canvas.set_width(config.width.max(1));
    canvas.set_height(config.height.max(1));

    let body = document
        .body()
        .ok_or_else(|| JsValue::from_str("browser document has no body"))?;
    body.append_child(&canvas).map_err(JsValue::from)?;

    Ok(canvas)
}

#[cfg(target_arch = "wasm32")]
fn start_browser_loop(
    window: Window,
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    status: HtmlElement,
    hud: BrowserHud,
    app: Rc<RefCell<App>>,
) -> Result<(), JsValue> {
    let frame_count = Rc::new(RefCell::new(0u32));
    let started_at = Rc::new(RefCell::new(None::<f64>));
    let previous_frame_at = Rc::new(RefCell::new(None::<f64>));
    let animation_frame = Rc::new(RefCell::new(None::<Closure<dyn FnMut(f64)>>));
    let animation_frame_handle = animation_frame.clone();
    let window_handle = window.clone();
    let canvas_handle = canvas.clone();
    let context_handle = context.clone();
    let status_handle = status.clone();
    let hud_handle = hud.clone();
    let started_at_handle = started_at.clone();
    let previous_frame_handle = previous_frame_at.clone();
    let frame_count_handle = frame_count.clone();
    let app_handle = app.clone();

    *animation_frame.borrow_mut() = Some(Closure::wrap(Box::new(move |now: f64| {
        let mut started_at_ref = started_at_handle.borrow_mut();
        let started_at_value = started_at_ref.get_or_insert(now);
        let elapsed_seconds = (now - *started_at_value) / 1000.0;
        let frames_per_second = {
            let mut previous_frame_ref = previous_frame_handle.borrow_mut();
            let frames_per_second = previous_frame_ref
                .map(|previous_frame_at| {
                    let delta_milliseconds = (now - previous_frame_at).max(0.001);
                    1000.0 / delta_milliseconds
                })
                .unwrap_or(0.0);
            *previous_frame_ref = Some(now);
            frames_per_second
        };
        let pulse = (elapsed_seconds * 2.0).sin() * 0.5 + 0.5;
        let current_frame = {
            let mut frame_count_ref = frame_count_handle.borrow_mut();
            *frame_count_ref += 1;
            *frame_count_ref
        };

        let summary = {
            let mut app = app_handle.borrow_mut();
            let fixed_step_seconds = app.config.fixed_time_step_seconds;
            let input_snapshot = app.input.clone();
            app.world.insert_resource(input_snapshot);
            app.world.insert_resource(BrowserViewport {
                width: canvas_handle.width() as f64,
                height: canvas_handle.height() as f64,
            });

            let run_loop_summary = app.tick_with_fixed_updates(elapsed_seconds, |world| {
                advance_browser_scene(world, elapsed_seconds, fixed_step_seconds);
            });

            let summary = {
                let scene = app
                    .world
                    .resource::<BrowserScene>()
                    .expect("BrowserScene should be initialized by the plugin");
                summarize_browser_scene(scene, run_loop_summary)
            };

            summary
        };

        resize_canvas_to_display_size(&window_handle, &canvas_handle);
        draw_frame(
            &context_handle,
            &canvas_handle,
            current_frame,
            elapsed_seconds,
            pulse,
            &summary,
        );
        status_handle.set_text_content(Some(&format!(
            "{} | runtime app driving browser scene",
            boot_message(),
        )));
        hud_handle.set_frame(current_frame);
        hud_handle.set_elapsed(elapsed_seconds);
        hud_handle.set_fps(frames_per_second);
        hud_handle.set_player(summary.player_x, summary.player_y);
        hud_handle.set_cursor(summary.cursor_x, summary.cursor_y);
        hud_handle.set_last_event(&summary.last_event);
        hud_handle.set_status(&summary.status_line);

        if let Some(callback) = animation_frame_handle.borrow().as_ref() {
            let _ = window_handle.request_animation_frame(callback.as_ref().unchecked_ref());
        }
    }) as Box<dyn FnMut(f64)>));

    status.set_text_content(Some(&format!(
        "{} | starting runtime app...",
        boot_message()
    )));
    hud.set_frame(0);
    hud.set_elapsed(0.0);
    hud.set_fps(0.0);
    hud.set_player(320.0, 200.0);
    hud.set_cursor(0.0, 0.0);
    hud.set_last_event("booting runtime app");
    hud.set_status("booting");
    if let Some(callback) = animation_frame.borrow().as_ref() {
        window
            .request_animation_frame(callback.as_ref().unchecked_ref())
            .map_err(JsValue::from)?;
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
struct BrowserDemoPlugin;

#[cfg(target_arch = "wasm32")]
impl Plugin for BrowserDemoPlugin {
    fn build(&self, app: &mut App) {
        app.config.fixed_time_step_seconds = 1.0 / 60.0;
        app.config.max_fixed_steps_per_frame = 4;
        app.world.insert_resource(BrowserScene::default());
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug)]
struct BrowserScene {
    sprint: FieldSprintState,
    player_position: [f64; 2],
    cursor_position: [f64; 2],
    cursor_trail: [[f64; 2]; 4],
    last_event: String,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug)]
struct BrowserViewport {
    width: f64,
    height: f64,
}

#[cfg(target_arch = "wasm32")]
impl Default for BrowserScene {
    fn default() -> Self {
        let mut sprint = FieldSprintState::default();
        sprint.target_position = FIELD_SPRINT_TARGET_POINTS[0];
        Self {
            sprint,
            player_position: [320.0, 200.0],
            cursor_position: [0.0, 0.0],
            cursor_trail: [[0.0, 0.0]; 4],
            last_event: "booting".to_string(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug)]
struct BrowserFrameSummary {
    player_x: f64,
    player_y: f64,
    cursor_x: f64,
    cursor_y: f64,
    cursor_trail: [[f64; 2]; 4],
    last_event: String,
    status_line: String,
    palette_lift: f64,
    fixed_updates: u32,
    hit_fixed_step_cap: bool,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
struct BrowserHud {
    frame: Element,
    elapsed: Element,
    fps: Element,
    player: Element,
    cursor: Element,
    last_event: Element,
    status: Element,
}

#[cfg(target_arch = "wasm32")]
impl BrowserHud {
    fn from_document(document: &Document) -> Result<Self, JsValue> {
        Ok(Self {
            frame: required_element(document, "hud-frame")?,
            elapsed: required_element(document, "hud-elapsed")?,
            fps: required_element(document, "hud-fps")?,
            player: required_element(document, "hud-player")?,
            cursor: required_element(document, "hud-cursor")?,
            last_event: required_element(document, "hud-last-event")?,
            status: required_element(document, "status")?,
        })
    }

    fn set_frame(&self, frame: u32) {
        self.frame.set_text_content(Some(&frame.to_string()));
    }

    fn set_elapsed(&self, elapsed_seconds: f64) {
        self.elapsed
            .set_text_content(Some(&format!("{:.2}s", elapsed_seconds)));
    }

    fn set_fps(&self, frames_per_second: f64) {
        self.fps
            .set_text_content(Some(&format!("{:.1}", frames_per_second)));
    }

    fn set_player(&self, x: f64, y: f64) {
        self.player
            .set_text_content(Some(&format!("({:.0}, {:.0})", x, y)));
    }

    fn set_cursor(&self, x: f64, y: f64) {
        self.cursor
            .set_text_content(Some(&format!("({:.0}, {:.0})", x, y)));
    }

    fn set_last_event(&self, last_event: &str) {
        self.last_event.set_text_content(Some(last_event));
    }

    fn set_status(&self, status: &str) {
        self.status.set_text_content(Some(status));
    }
}

#[cfg(target_arch = "wasm32")]
fn required_element(document: &Document, element_id: &str) -> Result<Element, JsValue> {
    document
        .get_element_by_id(element_id)
        .ok_or_else(|| JsValue::from_str(&format!("{} element is unavailable", element_id)))
}

#[cfg(target_arch = "wasm32")]
fn advance_browser_scene(world: &mut World, elapsed_seconds: f64, fixed_step_seconds: f64) {
    let Some(input) = world.resource::<InputState>().cloned() else {
        return;
    };

    let Some(viewport) = world.resource::<BrowserViewport>().copied() else {
        return;
    };

    let Some(scene) = world.resource_mut::<BrowserScene>() else {
        return;
    };

    advance_field_sprint(
        &mut scene.sprint,
        &input,
        input.mouse.is_pressed(MouseButton::Left),
        fixed_step_seconds as f32,
    );
    scene.player_position[0] =
        ((scene.sprint.player_position[0] as f64 + 1.0) * 0.5 * viewport.width)
            .clamp(10.0, viewport.width - 10.0);
    scene.player_position[1] = ((1.0 - (scene.sprint.player_position[1] as f64 + 1.0) * 0.5)
        * viewport.height)
        .clamp(10.0, viewport.height - 10.0);
    scene.cursor_position = [input.mouse.x as f64, input.mouse.y as f64];
    scene.sprint.palette_mode = input.mouse.is_pressed(MouseButton::Right);
    scene.sprint.reverse_motion = input.mouse.is_pressed(MouseButton::Middle);
    scene.sprint.paused = false;

    let current_cursor = scene.cursor_position;
    scene.cursor_trail.rotate_right(1);
    scene.cursor_trail[0] = current_cursor;
}

#[cfg(target_arch = "wasm32")]
fn summarize_browser_scene(
    scene: &BrowserScene,
    run_loop_summary: RunLoopSummary,
) -> BrowserFrameSummary {
    let cap_suffix = if run_loop_summary.hit_fixed_step_cap {
        " | fixed step cap hit"
    } else {
        ""
    };

    BrowserFrameSummary {
        player_x: scene.player_position[0],
        player_y: scene.player_position[1],
        cursor_x: scene.cursor_position[0],
        cursor_y: scene.cursor_position[1],
        cursor_trail: scene.cursor_trail,
        last_event: scene.last_event.clone(),
        status_line: format!(
            "field sprint score={} target={} player=({:.0},{:.0}) cursor=({:.0},{:.0}) {} | fixed updates {}{}",
            scene.sprint.score,
            scene.sprint.target_index,
            scene.player_position[0],
            scene.player_position[1],
            scene.cursor_position[0],
            scene.cursor_position[1],
            scene.last_event,
            run_loop_summary.fixed_updates,
            cap_suffix,
        ),
        palette_lift: scene.sprint.accent_rgba[0] as f64,
        fixed_updates: run_loop_summary.fixed_updates,
        hit_fixed_step_cap: run_loop_summary.hit_fixed_step_cap,
    }
}

#[cfg(target_arch = "wasm32")]
fn resize_canvas_to_display_size(window: &Window, canvas: &HtmlCanvasElement) {
    let device_pixel_ratio = window.device_pixel_ratio().max(1.0);
    let display_width = (canvas.client_width() as f64 * device_pixel_ratio).max(1.0) as u32;
    let display_height = (canvas.client_height() as f64 * device_pixel_ratio).max(1.0) as u32;

    if canvas.width() != display_width || canvas.height() != display_height {
        canvas.set_width(display_width);
        canvas.set_height(display_height);
    }
}

#[cfg(target_arch = "wasm32")]
fn draw_frame(
    context: &CanvasRenderingContext2d,
    canvas: &HtmlCanvasElement,
    frame_count: u32,
    elapsed_seconds: f64,
    pulse: f64,
    summary: &BrowserFrameSummary,
) {
    let background = 0.15 + summary.palette_lift * 0.2;
    context.set_fill_style_str(&format!(
        "rgb({:.0}, {:.0}, {:.0})",
        background * 255.0,
        background * 255.0,
        (background + 0.08) * 255.0
    ));
    context.fill_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);

    context.set_fill_style_str("rgba(255, 255, 255, 0.04)");
    draw_grid(context, canvas, 80.0);

    let vignette_alpha = 0.10 + summary.palette_lift * 0.06;
    context.set_fill_style_str(&format!("rgba(9, 12, 20, {:.3})", vignette_alpha));
    context.fill_rect(0.0, 0.0, canvas.width() as f64, 22.0);
    context.fill_rect(
        0.0,
        canvas.height() as f64 - 22.0,
        canvas.width() as f64,
        22.0,
    );
    context.fill_rect(0.0, 0.0, 22.0, canvas.height() as f64);
    context.fill_rect(
        canvas.width() as f64 - 22.0,
        0.0,
        22.0,
        canvas.height() as f64,
    );

    context.set_stroke_style_str("rgba(125, 227, 255, 0.14)");
    context.set_line_width(2.0);
    context.begin_path();
    context.move_to(16.0, 16.0);
    context.line_to(40.0, 16.0);
    context.move_to(16.0, 16.0);
    context.line_to(16.0, 40.0);
    context.move_to(canvas.width() as f64 - 16.0, 16.0);
    context.line_to(canvas.width() as f64 - 40.0, 16.0);
    context.move_to(canvas.width() as f64 - 16.0, 16.0);
    context.line_to(canvas.width() as f64 - 16.0, 40.0);
    context.move_to(16.0, canvas.height() as f64 - 16.0);
    context.line_to(40.0, canvas.height() as f64 - 16.0);
    context.move_to(16.0, canvas.height() as f64 - 16.0);
    context.line_to(16.0, canvas.height() as f64 - 40.0);
    context.move_to(canvas.width() as f64 - 16.0, canvas.height() as f64 - 16.0);
    context.line_to(canvas.width() as f64 - 40.0, canvas.height() as f64 - 16.0);
    context.move_to(canvas.width() as f64 - 16.0, canvas.height() as f64 - 16.0);
    context.line_to(canvas.width() as f64 - 16.0, canvas.height() as f64 - 40.0);
    context.stroke();

    context.set_stroke_style_str("rgba(244, 247, 251, 0.05)");
    context.set_line_width(1.0);
    context.stroke_rect(
        12.0,
        12.0,
        canvas.width() as f64 - 24.0,
        canvas.height() as f64 - 24.0,
    );

    context.set_fill_style_str("rgba(9, 12, 20, 0.30)");
    context.fill_rect(18.0, 18.0, 244.0, 48.0);
    context.set_fill_style_str("rgba(125, 227, 255, 0.10)");
    context.fill_rect(18.0, 18.0, 244.0, 2.0);
    context.set_fill_style_str("rgba(255, 95, 95, 0.14)");
    context.fill_rect(222.0, 26.0, 40.0, 14.0);
    context.set_fill_style_str(&format!("rgba(255, 95, 95, {:.3})", 0.55 + pulse * 0.35));
    context.fill_rect(
        227.0 + pulse * 0.5,
        29.0 + pulse * 0.2,
        7.0 + pulse * 1.5,
        7.0 + pulse * 1.5,
    );
    context.set_fill_style_str("#ff5f5f");
    context.fill_rect(227.0, 29.0, 7.0, 7.0);

    let sweep_y = (elapsed_seconds * 36.0) % canvas.height() as f64;
    context.set_fill_style_str("rgba(125, 227, 255, 0.06)");
    context.fill_rect(0.0, sweep_y, canvas.width() as f64, 9.0);

    let diagonal_offset =
        (elapsed_seconds * 120.0) % (canvas.width() as f64 + canvas.height() as f64);
    context.set_stroke_style_str("rgba(125, 227, 255, 0.08)");
    context.set_line_width(2.0);
    context.begin_path();
    context.move_to(-(canvas.height() as f64) + diagonal_offset, 0.0);
    context.line_to(diagonal_offset, canvas.height() as f64);
    context.stroke();

    let scan_y = 20.0 + (elapsed_seconds * 24.0 % 34.0);
    context.set_fill_style_str("rgba(125, 227, 255, 0.10)");
    context.fill_rect(20.0, scan_y, 256.0, 1.5);

    context.set_fill_style_str("rgba(125, 227, 255, 0.24)");
    let center_x = canvas.width() as f64 * 0.5;
    let center_y = canvas.height() as f64 * 0.5;
    context.fill_rect(center_x - 1.0, center_y - 10.0, 2.0, 20.0);
    context.fill_rect(center_x - 10.0, center_y - 1.0, 20.0, 2.0);
    context.set_fill_style_str("rgba(125, 227, 255, 0.05)");
    draw_soft_glow(context, center_x, center_y, 14.0 + pulse * 6.0, 2);

    context.set_fill_style_str("#7de3ff");
    context.fill_rect(
        24.0,
        24.0,
        ((canvas.width().saturating_sub(48)) as f64) * pulse,
        18.0,
    );

    context.set_stroke_style_str("rgba(125, 227, 255, 0.18)");
    context.set_line_width(1.5);
    context.begin_path();
    context.move_to(summary.player_x, summary.player_y);
    context.line_to(summary.cursor_x, summary.cursor_y);
    context.stroke();

    let connector_mid_x = summary.player_x + (summary.cursor_x - summary.player_x) * 0.5;
    let connector_mid_y = summary.player_y + (summary.cursor_y - summary.player_y) * 0.5;
    context.set_fill_style_str(&format!("rgba(125, 227, 255, {:.3})", 0.14 + pulse * 0.18));
    context.begin_path();
    let _ = context.arc(
        connector_mid_x,
        connector_mid_y,
        2.5 + pulse * 1.5,
        0.0,
        std::f64::consts::TAU,
    );
    context.fill();

    let connector_distance = ((summary.cursor_x - summary.player_x).powi(2)
        + (summary.cursor_y - summary.player_y).powi(2))
    .sqrt();
    context.set_font("10px monospace");
    context.set_fill_style_str("rgba(9, 12, 20, 0.6)");
    context.fill_rect(connector_mid_x + 8.0, connector_mid_y + 4.0, 44.0, 16.0);
    context.set_fill_style_str(&format!("rgba(244, 247, 251, {:.3})", 0.55 + pulse * 0.2));
    let _ = context.fill_text(
        &format!("{:.0}px", connector_distance),
        connector_mid_x + 11.0,
        connector_mid_y + 16.0,
    );

    let connector_dx = summary.cursor_x - summary.player_x;
    let connector_dy = summary.cursor_y - summary.player_y;
    let connector_length = (connector_dx * connector_dx + connector_dy * connector_dy)
        .sqrt()
        .max(1.0);
    let arrow_scale = 10.0 / connector_length;
    let arrow_base_x = summary.cursor_x - connector_dx * arrow_scale;
    let arrow_base_y = summary.cursor_y - connector_dy * arrow_scale;
    let perpendicular_x = -connector_dy / connector_length * 4.0;
    let perpendicular_y = connector_dx / connector_length * 4.0;
    context.set_fill_style_str("rgba(125, 227, 255, 0.28)");
    context.begin_path();
    context.move_to(summary.cursor_x, summary.cursor_y);
    context.line_to(
        arrow_base_x + perpendicular_x,
        arrow_base_y + perpendicular_y,
    );
    context.line_to(
        arrow_base_x - perpendicular_x,
        arrow_base_y - perpendicular_y,
    );
    context.close_path();
    context.fill();

    context.set_line_width(1.0);
    for (index, segment) in summary.cursor_trail.iter().enumerate().skip(1) {
        let previous_segment = summary.cursor_trail[index - 1];
        let alpha = 0.22 / index as f64;
        context.set_stroke_style_str(&format!("rgba(255, 95, 95, {:.3})", alpha));
        context.begin_path();
        context.move_to(previous_segment[0], previous_segment[1]);
        context.line_to(segment[0], segment[1]);
        context.stroke();
    }

    let player_ring = 16.0 + pulse * 10.0;
    context.set_stroke_style_str("rgba(255, 159, 28, 0.28)");
    context.begin_path();
    let _ = context.arc(
        summary.player_x,
        summary.player_y,
        player_ring,
        0.0,
        std::f64::consts::TAU,
    );
    context.stroke();

    let orbit_angle = elapsed_seconds * 1.4;
    let orbit_x = summary.player_x + orbit_angle.cos() * (player_ring + 7.0);
    let orbit_y = summary.player_y + orbit_angle.sin() * (player_ring + 7.0);
    context.set_fill_style_str("rgba(255, 209, 140, 0.18)");
    draw_soft_glow(context, orbit_x, orbit_y, 6.0 + pulse * 2.0, 2);
    context.set_fill_style_str("rgba(255, 209, 140, 0.72)");
    context.fill_rect(orbit_x - 2.0, orbit_y - 2.0, 4.0, 4.0);

    context.set_fill_style_str("rgba(255, 159, 28, 0.08)");
    draw_soft_glow(
        context,
        summary.player_x,
        summary.player_y,
        player_ring + 14.0,
        3,
    );

    context.set_fill_style_str("#ff9f1c");
    context.fill_rect(summary.player_x - 10.0, summary.player_y - 10.0, 20.0, 20.0);
    context.set_font("bold 10px monospace");
    context.set_fill_style_str("rgba(9, 12, 20, 0.55)");
    context.fill_rect(summary.player_x + 10.0, summary.player_y - 18.0, 16.0, 12.0);
    context.set_fill_style_str("rgba(255, 209, 140, 0.85)");
    let _ = context.fill_text("P", summary.player_x + 13.0, summary.player_y - 12.0);

    let cursor_ring = 8.0 + (1.0 - pulse) * 6.0;
    context.set_stroke_style_str("rgba(255, 95, 95, 0.22)");
    context.begin_path();
    let _ = context.arc(
        summary.cursor_x,
        summary.cursor_y,
        cursor_ring,
        0.0,
        std::f64::consts::TAU,
    );
    context.stroke();

    context.set_fill_style_str("rgba(255, 95, 95, 0.06)");
    draw_soft_glow(
        context,
        summary.cursor_x,
        summary.cursor_y,
        cursor_ring + 10.0 + pulse * 2.0,
        2,
    );

    context.set_fill_style_str("#ff5f5f");
    context.fill_rect(summary.cursor_x - 3.0, summary.cursor_y - 3.0, 6.0, 6.0);
    context.set_fill_style_str("rgba(9, 12, 20, 0.55)");
    context.fill_rect(summary.cursor_x + 7.0, summary.cursor_y - 16.0, 16.0, 12.0);
    context.set_fill_style_str("rgba(255, 182, 182, 0.9)");
    let _ = context.fill_text("C", summary.cursor_x + 10.0, summary.cursor_y - 10.0);
    context.set_fill_style_str("#f5f7fb");
    let fixed_step_suffix = if summary.hit_fixed_step_cap { " !" } else { "" };
    context.set_font("16px monospace");
    let _ = context.fill_text("Tokimu live scene", 28.0, 40.0);
    context.set_fill_style_str("rgba(244, 247, 251, 0.68)");
    let _ = context.fill_text("grid / player / cursor", 28.0, 58.0);
    context.set_fill_style_str("#ffb5b5");
    context.set_font("bold 9px monospace");
    let _ = context.fill_text("LIVE", 239.0, 37.0);
    context.set_fill_style_str("#f5f7fb");
    let _ = context.fill_text(
        &format!(
            "Tokimu runtime app frame {} / fixed {}{}",
            frame_count, summary.fixed_updates, fixed_step_suffix
        ),
        24.0,
        92.0,
    );
    let _ = context.fill_text(&format!("elapsed {:.2}s", elapsed_seconds), 24.0, 116.0);
    let _ = context.fill_text(&summary.status_line, 24.0, 140.0);
}

#[cfg(target_arch = "wasm32")]
fn draw_grid(context: &CanvasRenderingContext2d, canvas: &HtmlCanvasElement, spacing: f64) {
    let width = canvas.width() as f64;
    let height = canvas.height() as f64;

    let mut x = spacing;
    while x < width {
        context.fill_rect(x, 0.0, 1.0, height);
        x += spacing;
    }

    let mut y = spacing;
    while y < height {
        context.fill_rect(0.0, y, width, 1.0);
        y += spacing;
    }
}

#[cfg(target_arch = "wasm32")]
fn draw_soft_glow(context: &CanvasRenderingContext2d, x: f64, y: f64, radius: f64, layers: u32) {
    for layer in 0..layers {
        let layer_ratio = 1.0 - layer as f64 / layers as f64;
        let layer_radius = radius * layer_ratio;
        context.begin_path();
        let _ = context.arc(x, y, layer_radius, 0.0, std::f64::consts::TAU);
        context.fill();
    }
}

#[cfg(target_arch = "wasm32")]
fn axis(negative: bool, positive: bool) -> f64 {
    match (negative, positive) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    }
}
