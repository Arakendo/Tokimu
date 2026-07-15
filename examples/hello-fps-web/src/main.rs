use std::sync::Arc;

use serde::Serialize;
use tokimu::{
    App, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome, Instance2d,
    KeyCode, Material, MaterialHandle, Mesh, MeshHandle, MouseButton, Pipeline, PipelineHandle,
    PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand,
    Renderer, WgpuBackend, WindowConfig,
};
use tokimu_core::math::{Mat4, Vec3};

#[cfg(not(target_arch = "wasm32"))]
use tokimu::run_window_with_app;
use tokimu::NativeWindow;

#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};
#[cfg(target_arch = "wasm32")]
use js_sys::{Function, Object, Reflect};
#[cfg(target_arch = "wasm32")]
use web_sys::{CustomEvent, CustomEventInit};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
#[cfg(target_arch = "wasm32")]
use web_sys::{window, Document, HtmlCanvasElement, HtmlElement, Window};
#[cfg(target_arch = "wasm32")]
use tokimu::wasm::install_browser_input_bridge;

const FLOOR_MESH: MeshHandle = MeshHandle(1);
const FLOOR_MATERIAL: MaterialHandle = MaterialHandle(1);
const TARGET_MATERIAL: MaterialHandle = MaterialHandle(2);
const PROJECTILE_MATERIAL: MaterialHandle = MaterialHandle(3);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const TARGET_SLOT_COUNT: usize = 8;
const PROJECTILE_SLOT_COUNT: usize = 12;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello FPS Web".into(),
            width: 1280,
            height: 720,
        },
        HelloFpsWebApp::new(),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    boot_browser_fps_web()
}

struct HelloFpsWebApp {
    app: App,
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    frame_index: u32,
    pipeline: PipelineHandle,
    camera_position: Vec3,
    yaw: f32,
    pitch: f32,
    look_yaw: f32,
    look_pitch: f32,
    score: u32,
    wave: u32,
    fire_requested: bool,
    targets: Vec<TargetSlot>,
    projectiles: Vec<ProjectileSlot>,
}

impl Default for HelloFpsWebApp {
    fn default() -> Self {
        Self {
            app: App::default(),
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            frame_index: 0,
            pipeline: PipelineHandle(0),
            camera_position: Vec3::new(0.0, 1.6, -4.0),
            yaw: 0.0,
            pitch: 0.0,
            look_yaw: 0.0,
            look_pitch: 0.0,
            score: 0,
            wave: 0,
            fire_requested: false,
            targets: spawn_targets(0),
            projectiles: spawn_projectiles(),
        }
    }
}

impl HelloFpsWebApp {
    fn new() -> Self {
        Self::default()
    }

    fn update_camera_title(&self) {
        if let Some(window) = self.window.as_ref() {
            window.set_title(&format!(
                "Tokimu Hello FPS Web | score={} | wave={} | targets={} | projectiles={} | WASD move | cursor look | click fire",
                self.score,
                self.wave + 1,
                self.targets.iter().filter(|target| target.active).count(),
                self.projectiles.iter().filter(|projectile| projectile.active).count(),
            ));
        }
    }

    fn initialize_renderer(&mut self, mut renderer: WgpuBackend) -> PlatformResult<()> {
        renderer.upload_material(
            FLOOR_MATERIAL,
            &Material::new("fps-floor", Color::rgb(0.07, 0.09, 0.12)),
        )?;
        renderer.upload_material(
            TARGET_MATERIAL,
            &Material::new("fps-target", Color::rgb(0.90, 0.56, 0.22)),
        )?;
        renderer.upload_material(
            PROJECTILE_MATERIAL,
            &Material::new("fps-projectile", Color::rgb(0.96, 0.84, 0.40)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "fps-pipeline",
            PipelineKind::LitColor3d,
        ))?;
        self.renderer = Some(renderer);
        self.update_camera_title();
        Ok(())
    }

    fn handle_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::CloseRequested = event {
            return Ok(());
        }

        if let PlatformInputEvent::MouseMotion { delta_x, delta_y } = event {
            self.look_yaw -= delta_x * 0.0032;
            self.look_pitch = (self.look_pitch - delta_y * 0.0024).clamp(-0.7, 0.7);
            self.yaw = self.look_yaw;
            self.pitch = self.look_pitch;
            return Ok(());
        }

        if let PlatformInputEvent::CursorMoved { x, y } = event {
            self.set_look_from_cursor_position(x, y);
        }

        if let Some(input_event) = event.as_input_event() {
            self.app.apply_input_event(input_event);
        }

        if let PlatformInputEvent::MouseInput {
            button: MouseButton::Left,
            pressed: true,
        } = event
        {
            self.fire_requested = true;
        }

        if let PlatformInputEvent::Resized { width, height } = event {
            self.window_size = [width.max(1) as f32, height.max(1) as f32];
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize_surface(width, height);
            }
        }

        Ok(())
    }

    fn step_frame(&mut self, delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        self.frame_index = self
            .app
            .run_loop_diagnostics()
            .frame_count()
            .saturating_add(1) as u32;
        self.update_camera_pose(delta_seconds);
        self.update_projectiles(delta_seconds);
        self.update_targets(delta_seconds);
        self.resolve_hits();
        self.upload_scene_meshes();
        self.update_camera_title();
        self.render_scene()
    }

    fn publish_frame_snapshot(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let snapshot = NativeFrameSnapshot {
                frame: self.frame_index,
                elapsed_seconds: self.app.elapsed_seconds(),
                player: NativePlayerSnapshot {
                    x: self.camera_position.x,
                    y: self.camera_position.y,
                    z: self.camera_position.z,
                    yaw: self.yaw,
                    pitch: self.pitch,
                },
                hud: NativeHudSnapshot {
                    score: self.score,
                    wave: self.wave + 1,
                    targets: self.targets.iter().filter(|target| target.active).count() as u32,
                    projectiles: self.projectiles.iter().filter(|projectile| projectile.active).count() as u32,
                    status: if self.fire_requested {
                        "fire requested".to_string()
                    } else {
                        "running".to_string()
                    },
                },
            };

            let snapshot_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("web")
                .join("live-frame.json");
            if let Ok(serialized) = serde_json::to_string(&snapshot) {
                let _ = std::fs::write(snapshot_path, serialized);
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            let Some(window) = window() else {
                return;
            };

            let snapshot = build_frame_snapshot(
                self.frame_index,
                self.app.elapsed_seconds(),
                self.camera_position,
                self.yaw,
                self.pitch,
                self.score,
                self.wave + 1,
                self.targets.iter().filter(|target| target.active).count() as u32,
                self.projectiles.iter().filter(|projectile| projectile.active).count() as u32,
                if self.fire_requested {
                    "fire requested"
                } else {
                    "running"
                },
            );

            if let Ok(value) = Reflect::get(&window, &JsValue::from_str("tokimuHelloFpsWebPushFrame")) {
                if let Some(function) = value.dyn_ref::<Function>() {
                    let _ = function.call1(&JsValue::from(window.clone()), &snapshot);
                }
            }

            let mut event_init = CustomEventInit::new();
            event_init.detail(&snapshot);
            if let Ok(event) = CustomEvent::new_with_event_init_dict("tokimu:fps-frame", &event_init) {
                let _ = window.dispatch_event(&event);
            }
        }
    }

    fn cursor_look(&self) -> (f32, f32) {
        (self.look_yaw, self.look_pitch)
    }

    fn set_look_from_cursor_position(&mut self, cursor_x: f32, cursor_y: f32) {
        let width = self.window_size[0].max(1.0);
        let height = self.window_size[1].max(1.0);
        self.look_yaw = (0.5 - cursor_x.clamp(0.0, width) / width) * std::f32::consts::TAU * 0.65;
        self.look_pitch = ((0.5 - cursor_y.clamp(0.0, height) / height) * std::f32::consts::PI * 0.55)
            .clamp(-0.7, 0.7);
        self.yaw = self.look_yaw;
        self.pitch = self.look_pitch;
    }

    fn camera_forward(&self) -> Vec3 {
        let (yaw, pitch) = self.cursor_look();
        Vec3::new(yaw.sin() * pitch.cos(), pitch.sin(), yaw.cos() * pitch.cos()).normalize_or_zero()
    }

    fn movement_vectors(&self) -> (Vec3, Vec3) {
        let forward = self.camera_forward();
        let flat_forward = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        let right = flat_forward.cross(Vec3::Y).normalize_or_zero();
        (flat_forward, right)
    }

    fn update_camera_pose(&mut self, delta_seconds: f64) {
        let (flat_forward, right) = self.movement_vectors();
        let speed = 5.5;
        let move_forward = axis(
            self.app.input.keyboard.is_pressed(KeyCode::KeyS),
            self.app.input.keyboard.is_pressed(KeyCode::KeyW),
        );
        let move_right = axis(
            self.app.input.keyboard.is_pressed(KeyCode::KeyA),
            self.app.input.keyboard.is_pressed(KeyCode::KeyD),
        );
        self.camera_position += flat_forward * move_forward * speed * delta_seconds as f32;
        self.camera_position += right * move_right * speed * delta_seconds as f32;
        self.camera_position.y = 1.6;
    }

    fn update_projectiles(&mut self, delta_seconds: f64) {
        let forward = self.camera_forward();
        if self.fire_requested {
            self.fire_requested = false;
            if let Some(projectile) = self
                .projectiles
                .iter_mut()
                .find(|projectile| !projectile.active)
            {
                projectile.active = true;
                projectile.position = self.camera_position + forward * 0.8;
                projectile.velocity = forward * 18.0;
                projectile.ttl = 1.5;
            }
        }

        for projectile in &mut self.projectiles {
            if !projectile.active {
                continue;
            }
            projectile.position += projectile.velocity * delta_seconds as f32;
            projectile.ttl -= delta_seconds as f32;
            if projectile.ttl <= 0.0 {
                projectile.active = false;
            }
        }
    }

    fn update_targets(&mut self, delta_seconds: f64) {
        let wobble = self.wave as f32 * 0.25 + self.score as f32 * 0.05;
        for (index, target) in self.targets.iter_mut().enumerate() {
            if !target.active {
                continue;
            }
            let bob = (self.wave as f32 * 0.4 + index as f32).sin() * 0.12;
            let strafe = (self.wave as f32 * 0.2 + index as f32 * 0.6).cos() * 0.04;
            target.position.x += strafe * delta_seconds as f32;
            target.position.y = 0.85 + bob;
            target.rotation += delta_seconds as f32 * (0.45 + wobble * 0.04);
        }
    }

    fn resolve_hits(&mut self) {
        for projectile in &mut self.projectiles {
            if !projectile.active {
                continue;
            }

            for target in &mut self.targets {
                if !target.active {
                    continue;
                }

                if projectile.position.distance(target.position) < 0.95 {
                    projectile.active = false;
                    target.active = false;
                    self.score += 1;
                    break;
                }
            }
        }

        if self.targets.iter().all(|target| !target.active) {
            self.wave += 1;
            self.targets = spawn_targets(self.wave);
        }
    }

    fn upload_scene_meshes(&mut self) {
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };

        renderer.upload_mesh(FLOOR_MESH, &build_floor_mesh());

        for target in &self.targets {
            if target.active {
                renderer.upload_mesh(target.mesh, &cube_mesh(target.position, Vec3::splat(0.85), target.rotation));
            }
        }

        for projectile in &self.projectiles {
            if projectile.active {
                renderer.upload_mesh(
                    projectile.mesh,
                    &cube_mesh(projectile.position, Vec3::splat(0.18), 0.0),
                );
            }
        }
    }

    fn render_scene(&mut self) -> PlatformResult<FrameOutcome> {
        let camera_forward = self.camera_forward();
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        let mut camera = Camera::perspective_3d(self.window_size[0], self.window_size[1]);
        camera.view = Mat4::look_at_rh(
            self.camera_position,
            self.camera_position + camera_forward,
            Vec3::Y,
        );
        renderer.upload_camera(CAMERA_HANDLE, camera);

        let mut commands = vec![RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.05, 0.07, 0.11),
        })];

        commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
            mesh: FLOOR_MESH,
            material: FLOOR_MATERIAL,
            pipeline: self.pipeline,
            instance: Instance2d::identity(),
            camera: Some(CAMERA_HANDLE),
            viewport: None,
        }));

        for target in &self.targets {
            if target.active {
                commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: target.mesh,
                    material: TARGET_MATERIAL,
                    pipeline: self.pipeline,
                    instance: Instance2d::identity(),
                    camera: Some(CAMERA_HANDLE),
                    viewport: None,
                }));
            }
        }

        for projectile in &self.projectiles {
            if projectile.active {
                commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: projectile.mesh,
                    material: PROJECTILE_MATERIAL,
                    pipeline: self.pipeline,
                    instance: Instance2d::identity(),
                    camera: Some(CAMERA_HANDLE),
                    viewport: None,
                }));
            }
        }

        renderer.begin_frame();
        renderer.submit(&commands);
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}

impl PlatformEventHandler for HelloFpsWebApp {
    #[cfg(not(target_arch = "wasm32"))]
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        self.initialize_renderer(renderer)
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        self.handle_platform_event(event)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn on_frame(&mut self, delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let outcome = self.step_frame(delta_seconds);
        self.publish_frame_snapshot();
        outcome
    }
}

#[cfg(target_arch = "wasm32")]
fn boot_browser_fps_web() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = window().ok_or_else(|| JsValue::from_str("browser window is unavailable"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("browser document is unavailable"))?;
    let canvas = document
        .get_element_by_id("tokimu-canvas")
        .ok_or_else(|| JsValue::from_str("tokimu-canvas is unavailable"))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| JsValue::from_str("tokimu-canvas is not a canvas element"))?;
    let hud = BrowserHud::from_document(&document)?;
    let app = Rc::new(RefCell::new(HelloFpsWebApp::new()));

    install_browser_input_bridge(
        WindowConfig {
            title: "Tokimu Hello FPS Web".into(),
            width: canvas.width().max(1),
            height: canvas.height().max(1),
        },
        {
            let app = app.clone();
            move |event| {
                let mut app = app.borrow_mut();
                if let Some(input_event) = event.as_input_event() {
                    app.app.apply_input_event(input_event);
                }

                let _ = app.handle_platform_event(event);
            }
        },
    )
    .map_err(|error| JsValue::from_str(&error.to_string()))?;

    {
        let mut app = app.borrow_mut();
        app.app.input.mouse.x = canvas.width() as f32 * 0.5;
        app.app.input.mouse.y = canvas.height() as f32 * 0.5;
    }

    let canvas_for_renderer = canvas.clone();
    let hud_for_loop = hud.clone();
    let hud_for_error = hud.clone();
    let window_for_loop = window.clone();
    let app_for_loop = app.clone();

    spawn_local(async move {
        match WgpuBackend::for_window(
            canvas_for_renderer,
            canvas.width(),
            canvas.height(),
        )
        .await
        {
            Ok(renderer) => {
                if let Err(error) = app_for_loop.borrow_mut().initialize_renderer(renderer) {
                    hud_for_loop.set_status(&format!("Tokimu FPS Web | renderer init failed: {error}"));
                    return;
                }

                if let Err(error) = start_browser_loop(window_for_loop, hud_for_loop, app_for_loop) {
                    hud_for_error.set_status(&format!("Tokimu FPS Web | browser loop failed: {error:?}"));
                }
            }
            Err(error) => {
                hud_for_error.set_status(&format!("Tokimu FPS Web | renderer init failed: {error}"));
            }
        }
    });

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn start_browser_loop(
    window: Window,
    hud: BrowserHud,
    app: Rc<RefCell<HelloFpsWebApp>>,
) -> Result<(), JsValue> {
    let started_at = Rc::new(RefCell::new(None::<f64>));
    let previous_frame_at = Rc::new(RefCell::new(None::<f64>));
    let animation_frame = Rc::new(RefCell::new(None::<Closure<dyn FnMut(f64)>>));

    let started_at_handle = started_at.clone();
    let previous_frame_handle = previous_frame_at.clone();
    let animation_frame_handle = animation_frame.clone();
    let window_handle = window.clone();
    let hud_handle = hud.clone();
    let app_handle = app.clone();

    *animation_frame.borrow_mut() = Some(Closure::wrap(Box::new(move |now: f64| {
        let mut started_at_ref = started_at_handle.borrow_mut();
        let started_at_value = started_at_ref.get_or_insert(now);
        let elapsed_seconds = (now - *started_at_value) / 1000.0;
        let delta_seconds = {
            let mut previous_frame_ref = previous_frame_handle.borrow_mut();
            let delta_seconds = previous_frame_ref
                .map(|previous_frame_at| ((now - previous_frame_at) / 1000.0).max(0.000_1))
                .unwrap_or(1.0 / 60.0);
            *previous_frame_ref = Some(now);
            delta_seconds
        };

        {
            let mut app = app_handle.borrow_mut();
            app.app.config.fixed_time_step_seconds = 1.0 / 60.0;
            app.app.config.max_fixed_steps_per_frame = 4;
            if let Err(error) = app.step_frame(delta_seconds) {
                hud_handle.set_status(&format!("Tokimu FPS Web | render failed: {error}"));
                return;
            }
        }

        {
            let app = app_handle.borrow();
            hud_handle.sync(&app, elapsed_seconds);
        }

        if let Some(callback) = animation_frame_handle.borrow().as_ref() {
            let _ = window_handle.request_animation_frame(callback.as_ref().unchecked_ref());
        }
    }) as Box<dyn FnMut(f64)>));

    hud.set_status("Tokimu Hello FPS Web | browser 3D running");
    hud.set_frame(0);
    hud.set_elapsed(0.0);
    hud.set_player(0.0, 1.6, 0.0, 0.0, 0.0);
    hud.set_score(0);
    hud.set_wave(1);
    hud.set_targets(8);
    hud.set_projectiles(0);

    hud.set_status("Tokimu Hello FPS Web | browser 3D running | click canvas to lock mouse | Esc releases");

    if let Some(callback) = animation_frame.borrow().as_ref() {
        window
            .request_animation_frame(callback.as_ref().unchecked_ref())
            .map_err(JsValue::from)?;
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
struct BrowserHud {
    status: HtmlElement,
    frame: HtmlElement,
    elapsed: HtmlElement,
    player: HtmlElement,
    score: HtmlElement,
    wave: HtmlElement,
    targets: HtmlElement,
    projectiles: HtmlElement,
}

#[cfg(target_arch = "wasm32")]
impl BrowserHud {
    fn from_document(document: &Document) -> Result<Self, JsValue> {
        Ok(Self {
            status: require_html_element(document, "status")?,
            frame: require_html_element(document, "hud-frame")?,
            elapsed: require_html_element(document, "hud-elapsed")?,
            player: require_html_element(document, "hud-player")?,
            score: require_html_element(document, "hud-score")?,
            wave: require_html_element(document, "hud-wave")?,
            targets: require_html_element(document, "hud-targets")?,
            projectiles: require_html_element(document, "hud-projectiles")?,
        })
    }

    fn set_status(&self, value: &str) {
        self.status.set_text_content(Some(value));
    }

    fn set_frame(&self, value: u32) {
        self.frame.set_text_content(Some(&value.to_string()));
    }

    fn set_elapsed(&self, value: f64) {
        self.elapsed.set_text_content(Some(&format!("{value:.1}s")));
    }

    fn set_player(&self, x: f32, y: f32, z: f32, yaw: f32, pitch: f32) {
        self.player.set_text_content(Some(&format!(
            "pos {x:.1}, {y:.1}, {z:.1} | look {yaw:.2}, {pitch:.2}"
        )));
    }

    fn set_score(&self, value: u32) {
        self.score.set_text_content(Some(&value.to_string()));
    }

    fn set_wave(&self, value: u32) {
        self.wave.set_text_content(Some(&value.to_string()));
    }

    fn set_targets(&self, value: u32) {
        self.targets.set_text_content(Some(&value.to_string()));
    }

    fn set_projectiles(&self, value: u32) {
        self.projectiles.set_text_content(Some(&value.to_string()));
    }

    fn sync(&self, app: &HelloFpsWebApp, elapsed_seconds: f64) {
        self.set_status("Tokimu Hello FPS Web | browser 3D running");
        self.set_frame(app.frame_index);
        self.set_elapsed(elapsed_seconds);
        self.set_player(
            app.camera_position.x,
            app.camera_position.y,
            app.camera_position.z,
            app.yaw,
            app.pitch,
        );
        self.set_score(app.score);
        self.set_wave(app.wave + 1);
        self.set_targets(app.targets.iter().filter(|target| target.active).count() as u32);
        self.set_projectiles(app.projectiles.iter().filter(|projectile| projectile.active).count() as u32);
    }
}

#[cfg(target_arch = "wasm32")]
fn require_html_element(document: &Document, id: &str) -> Result<HtmlElement, JsValue> {
    document
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str(&format!("missing required element: {id}")))?
        .dyn_into::<HtmlElement>()
        .map_err(|_| JsValue::from_str(&format!("element {id} is not an HtmlElement")))
}

#[cfg(target_arch = "wasm32")]
fn build_frame_snapshot(
    frame: u32,
    elapsed_seconds: f64,
    player_position: Vec3,
    yaw: f32,
    pitch: f32,
    score: u32,
    wave: u32,
    targets: u32,
    projectiles: u32,
    status: &str,
) -> JsValue {
    let frame_value = Object::new();
    let player_value = Object::new();
    let hud_value = Object::new();

    set_number(&frame_value, "frame", frame as f64);
    set_number(&frame_value, "elapsedSeconds", elapsed_seconds);
    set_number(&player_value, "x", player_position.x as f64);
    set_number(&player_value, "y", player_position.y as f64);
    set_number(&player_value, "z", player_position.z as f64);
    set_number(&player_value, "yaw", yaw as f64);
    set_number(&player_value, "pitch", pitch as f64);
    set_number(&hud_value, "score", score as f64);
    set_number(&hud_value, "wave", wave as f64);
    set_number(&hud_value, "targets", targets as f64);
    set_number(&hud_value, "projectiles", projectiles as f64);
    set_string(&hud_value, "status", status);

    set_value(&frame_value, "player", &player_value.into());
    set_value(&frame_value, "hud", &hud_value.into());

    frame_value.into()
}

#[cfg(target_arch = "wasm32")]
fn set_number(object: &Object, key: &str, value: f64) {
    let _ = Reflect::set(object, &JsValue::from_str(key), &JsValue::from_f64(value));
}

#[cfg(target_arch = "wasm32")]
fn set_string(object: &Object, key: &str, value: &str) {
    let _ = Reflect::set(object, &JsValue::from_str(key), &JsValue::from_str(value));
}

#[cfg(target_arch = "wasm32")]
fn set_value(object: &Object, key: &str, value: &JsValue) {
    let _ = Reflect::set(object, &JsValue::from_str(key), value);
}

fn axis(negative: bool, positive: bool) -> f32 {
    match (negative, positive) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    }
}

fn build_floor_mesh() -> Mesh {
    cube_mesh(Vec3::new(0.0, -0.55, 8.0), Vec3::new(18.0, 0.1, 24.0), 0.0)
}

fn cube_mesh(translation: Vec3, scale: Vec3, rotation_y: f32) -> Mesh {
    let transform = Mat4::from_translation(translation)
        * Mat4::from_rotation_y(rotation_y)
        * Mat4::from_scale(scale);
    let normal_transform = transform.inverse().transpose();
    let base = Mesh::cube();

    Mesh::new(
        base.positions
            .into_iter()
            .map(|position| {
                transform
                    .transform_point3(Vec3::from_array(position))
                    .to_array()
            })
            .collect(),
        base.normals
            .into_iter()
            .map(|normal| {
                normal_transform
                    .transform_vector3(Vec3::from_array(normal))
                    .normalize_or_zero()
                    .to_array()
            })
            .collect(),
    )
}

fn spawn_targets(wave: u32) -> Vec<TargetSlot> {
    let radius = 5.0 + wave as f32 * 0.6;
    let z_bias = 10.0 + wave as f32 * 0.4;
    (0..TARGET_SLOT_COUNT)
        .map(|index| {
            let angle = index as f32 / TARGET_SLOT_COUNT as f32 * std::f32::consts::TAU
                + wave as f32 * 0.25;
            let position = Vec3::new(
                angle.cos() * radius,
                0.85,
                z_bias + angle.sin() * radius,
            );
            TargetSlot {
                mesh: target_mesh_handle(index),
                position,
                rotation: index as f32 * 0.4,
                active: true,
            }
        })
        .collect()
}

fn spawn_projectiles() -> Vec<ProjectileSlot> {
    (0..PROJECTILE_SLOT_COUNT)
        .map(|index| ProjectileSlot {
            mesh: projectile_mesh_handle(index),
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            ttl: 0.0,
            active: false,
        })
        .collect()
}

fn target_mesh_handle(index: usize) -> MeshHandle {
    MeshHandle(10 + index as u64)
}

fn projectile_mesh_handle(index: usize) -> MeshHandle {
    MeshHandle(100 + index as u64)
}

struct TargetSlot {
    mesh: MeshHandle,
    position: Vec3,
    rotation: f32,
    active: bool,
}

struct ProjectileSlot {
    mesh: MeshHandle,
    position: Vec3,
    velocity: Vec3,
    ttl: f32,
    active: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NativeFrameSnapshot {
    frame: u32,
    elapsed_seconds: f64,
    player: NativePlayerSnapshot,
    hud: NativeHudSnapshot,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NativePlayerSnapshot {
    x: f32,
    y: f32,
    z: f32,
    yaw: f32,
    pitch: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NativeHudSnapshot {
    score: u32,
    wave: u32,
    targets: u32,
    projectiles: u32,
    status: String,
}
