use std::sync::Arc;
use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawRenderableCommand,
    Instance2d, KeyCode, Material, MaterialHandle, Mesh, MeshHandle, MouseButton, NativeWindow,
    Pipeline, PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent,
    PlatformResult, RenderCommand, Renderable, RenderableHandle, Renderer, WgpuBackend,
    WindowConfig,
};

const TRIANGLE_HANDLE: MeshHandle = MeshHandle(1);
const QUAD_HANDLE: MeshHandle = MeshHandle(2);
const TRIANGLE_MATERIAL: MaterialHandle = MaterialHandle(1);
const TRIANGLE_SECOND_MATERIAL: MaterialHandle = MaterialHandle(2);
const TRIANGLE_THIRD_MATERIAL: MaterialHandle = MaterialHandle(3);
const TRIANGLE_FOURTH_MATERIAL: MaterialHandle = MaterialHandle(4);
const QUAD_MATERIAL: MaterialHandle = MaterialHandle(5);
const DIAMOND_MATERIAL: MaterialHandle = MaterialHandle(6);
const TRIANGLE_PIPELINE: PipelineHandle = PipelineHandle(1);
const TRIANGLE_RENDERABLE: RenderableHandle = RenderableHandle(1);
const TRIANGLE_SECOND_RENDERABLE: RenderableHandle = RenderableHandle(2);
const TRIANGLE_THIRD_RENDERABLE: RenderableHandle = RenderableHandle(3);
const TRIANGLE_FOURTH_RENDERABLE: RenderableHandle = RenderableHandle(4);
const QUAD_RENDERABLE: RenderableHandle = RenderableHandle(5);
const DIAMOND_RENDERABLE: RenderableHandle = RenderableHandle(6);
const TRIANGLE_CAMERA: CameraHandle = CameraHandle(1);

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello Triangle".into(),
            width: 1280,
            height: 720,
        },
        HelloTriangleApp::default(),
    )
}

struct HelloTriangleApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    camera: Camera,
    camera_world_height: f32,
    window_size: [f32; 2],
    motion_phase: f32,
    input_offset: [f32; 2],
    cursor_offset: [f32; 2],
    square_offset: [f32; 2],
    diamond_offset: [f32; 2],
    paused: bool,
    palette_mode: bool,
    reverse_motion: bool,
    mouse_hold_active: bool,
    accent: Color,
    logged_backend: bool,
}

impl Default for HelloTriangleApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            camera: Camera::default(),
            camera_world_height: 2.0,
            window_size: [1.0, 1.0],
            motion_phase: 0.0,
            input_offset: [0.0, 0.0],
            cursor_offset: [0.0, 0.0],
            square_offset: [0.0, 0.0],
            diamond_offset: [0.0, 0.0],
            paused: false,
            palette_mode: false,
            reverse_motion: false,
            mouse_hold_active: false,
            accent: Color::rgb(0.08, 0.18, 0.34),
            logged_backend: false,
        }
    }
}

impl PlatformEventHandler for HelloTriangleApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(TRIANGLE_HANDLE, &Mesh::triangle());
        renderer.upload_mesh(QUAD_HANDLE, &Mesh::quad());
        renderer.upload_mesh(MeshHandle(3), &Mesh::diamond());
        renderer.upload_pipeline(
            TRIANGLE_PIPELINE,
            &Pipeline::new("triangle-pipeline", PipelineKind::SolidColor2d),
        )?;
        renderer.upload_material(
            TRIANGLE_MATERIAL,
            &Material::new("triangle-material", Color::rgb(0.96, 0.72, 0.28)),
        )?;
        renderer.upload_material(
            TRIANGLE_SECOND_MATERIAL,
            &Material::new("triangle-material-secondary", Color::rgb(0.30, 0.84, 0.88)),
        )?;
        renderer.upload_material(
            TRIANGLE_THIRD_MATERIAL,
            &Material::new("triangle-material-tertiary", Color::rgb(0.95, 0.40, 0.55)),
        )?;
        renderer.upload_material(
            TRIANGLE_FOURTH_MATERIAL,
            &Material::new("triangle-material-quaternary", Color::rgb(0.96, 0.92, 0.44)),
        )?;
        renderer.upload_material(
            QUAD_MATERIAL,
            &Material::new("square-material", Color::rgb(0.62, 0.52, 0.96)),
        )?;
        renderer.upload_material(
            DIAMOND_MATERIAL,
            &Material::new("diamond-material", Color::rgb(0.36, 0.95, 0.58)),
        )?;
        renderer.upload_renderable(
            TRIANGLE_RENDERABLE,
            Renderable::new(TRIANGLE_HANDLE, TRIANGLE_MATERIAL, TRIANGLE_PIPELINE),
        );
        renderer.upload_renderable(
            TRIANGLE_SECOND_RENDERABLE,
            Renderable::new(TRIANGLE_HANDLE, TRIANGLE_SECOND_MATERIAL, TRIANGLE_PIPELINE),
        );
        renderer.upload_renderable(
            TRIANGLE_THIRD_RENDERABLE,
            Renderable::new(TRIANGLE_HANDLE, TRIANGLE_THIRD_MATERIAL, TRIANGLE_PIPELINE),
        );
        renderer.upload_renderable(
            TRIANGLE_FOURTH_RENDERABLE,
            Renderable::new(TRIANGLE_HANDLE, TRIANGLE_FOURTH_MATERIAL, TRIANGLE_PIPELINE),
        );
        renderer.upload_renderable(
            QUAD_RENDERABLE,
            Renderable::new(QUAD_HANDLE, QUAD_MATERIAL, TRIANGLE_PIPELINE),
        );
        renderer.upload_renderable(
            DIAMOND_RENDERABLE,
            Renderable::new(MeshHandle(3), DIAMOND_MATERIAL, TRIANGLE_PIPELINE),
        );
        self.camera
            .set_orthographic_2d_with_height(
                size.width as f32,
                size.height as f32,
                self.camera_world_height,
            );
        renderer.upload_camera(TRIANGLE_CAMERA, self.camera);
        renderer.set_active_camera(TRIANGLE_CAMERA);
        self.renderer = Some(renderer);
        self.update_window_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::KeyboardInput { key, pressed } = event {
            if pressed {
                self.handle_key_press(key);
                self.update_window_title();
            }
        }

        if let PlatformInputEvent::CursorMoved { x, y } = event {
            self.handle_cursor_move(x, y);
            self.update_window_title();
        }

        if let PlatformInputEvent::MouseInput { button, pressed } = event {
            self.mouse_hold_active = pressed;
            self.handle_mouse_press(button, pressed);
            self.update_window_title();
        }

        if let PlatformInputEvent::Resized { width, height } = event {
            self.window_size = [width.max(1) as f32, height.max(1) as f32];
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize_surface(width, height);
                self.camera.set_orthographic_2d_with_height(
                    width as f32,
                    height as f32,
                    self.camera_world_height,
                );
                renderer.upload_camera(TRIANGLE_CAMERA, self.camera);
            }
        }

        Ok(())
    }

    fn on_frame(&mut self, delta_seconds: f64) -> PlatformResult<()> {
        let (stats, renderer_name, backend_api, adapter_name, device_kind) = {
            let Some(renderer) = self.renderer.as_mut() else {
                return Ok(());
            };

            if !self.paused {
                let motion_delta = delta_seconds as f32 * std::f32::consts::TAU * 0.25;
                if self.reverse_motion {
                    self.motion_phase = (self.motion_phase - motion_delta)
                        .rem_euclid(std::f32::consts::TAU);
                } else {
                    self.motion_phase = (self.motion_phase + motion_delta)
                        .rem_euclid(std::f32::consts::TAU);
                }
            }

            self.accent = if self.palette_mode {
                Color::rgb(
                    0.20 + self.motion_phase.cos() * 0.04,
                    0.10 + self.motion_phase.sin() * 0.03,
                    0.28 + self.motion_phase.cos() * 0.05,
                )
            } else if self.paused {
                Color::rgb(0.06, 0.10, 0.20)
            } else {
                Color::rgb(
                    0.08 + self.motion_phase.sin() * 0.03,
                    0.18 + self.motion_phase.cos() * 0.02,
                    0.34 + self.motion_phase.sin() * 0.04,
                )
            };

            if self.mouse_hold_active {
                self.accent = Color::rgb(
                    (self.accent.r + 0.10).min(1.0),
                    (self.accent.g + 0.08).min(1.0),
                    (self.accent.b + 0.12).min(1.0),
                );
            }

            renderer.begin_frame();
            let hold_boost = if self.mouse_hold_active { 1.0 } else { 0.0 };
            let drag_pull_x = self.cursor_offset[0] * hold_boost * 0.35;
            let drag_pull_y = self.cursor_offset[1] * hold_boost * 0.35;
            let left_wobble = self.motion_phase.sin() * (0.04 + hold_boost * 0.02);
            let right_orbit_x = 0.35 + self.motion_phase.cos() * (0.12 + hold_boost * 0.05) + drag_pull_x;
            let right_orbit_y = self.motion_phase.sin() * (0.08 + hold_boost * 0.04) + drag_pull_y;
            let right_pulse_scale = 0.30 + self.motion_phase.sin() * (0.05 + hold_boost * 0.02);
            let third_orbit_x = self.motion_phase.sin() * (0.18 + hold_boost * 0.04) + drag_pull_x * 0.5;
            let third_orbit_y = -0.33 + self.motion_phase.cos() * (0.05 + hold_boost * 0.03) + drag_pull_y * 0.5;
            let third_scale = 0.22 + self.motion_phase.cos() * (0.03 + hold_boost * 0.01);
            let fourth_orbit_x = -self.motion_phase.cos() * (0.16 + hold_boost * 0.04) - drag_pull_x * 0.25;
            let fourth_orbit_y = 0.30 + self.motion_phase.sin() * (0.06 + hold_boost * 0.03) + drag_pull_y * 0.25;
            let fourth_scale = 0.18 + self.motion_phase.sin() * (0.02 + hold_boost * 0.01);
            let quad_orbit_x = self.input_offset[0] + self.motion_phase.cos() * (0.22 + hold_boost * 0.05) + drag_pull_x * 0.75;
            let quad_orbit_y = self.input_offset[1] + self.square_offset[1] + self.motion_phase.sin() * (0.03 + hold_boost * 0.02) + drag_pull_y * 0.75;
            let quad_scale = 0.24 + self.motion_phase.cos() * (0.02 + hold_boost * 0.01);
            let diamond_orbit_x = self.input_offset[0] + self.diamond_offset[0] + self.motion_phase.sin() * (0.16 + hold_boost * 0.04) + drag_pull_x;
            let diamond_orbit_y = self.input_offset[1] + self.diamond_offset[1] + 0.18 + self.motion_phase.cos() * (0.07 + hold_boost * 0.03) + drag_pull_y;
            let diamond_scale = 0.20 + self.motion_phase.sin() * (0.025 + hold_boost * 0.01);
            let left_instance = Instance2d::translated(-0.35 + left_wobble, 0.0)
                .with_rotation(-self.motion_phase * 0.5);
            let right_instance = Instance2d::identity()
                .with_translation([right_orbit_x, right_orbit_y])
                .with_scale([right_pulse_scale, right_pulse_scale])
                .with_rotation(self.motion_phase);
            let third_instance = Instance2d::identity()
                .with_translation([third_orbit_x, third_orbit_y])
                .with_scale([third_scale, third_scale])
                .with_rotation(self.motion_phase * 1.5);
            let fourth_instance = Instance2d::identity()
                .with_translation([fourth_orbit_x, fourth_orbit_y])
                .with_scale([fourth_scale, fourth_scale])
                .with_rotation(-self.motion_phase * 0.75);
            let quad_instance = Instance2d::identity()
                .with_translation([quad_orbit_x, quad_orbit_y])
                .with_scale([quad_scale, quad_scale])
                .with_rotation(self.motion_phase * 0.35);
            let diamond_instance = Instance2d::identity()
                .with_translation([diamond_orbit_x, diamond_orbit_y])
                .with_scale([diamond_scale, diamond_scale])
                .with_rotation(-self.motion_phase * 0.9);
            renderer.submit(&[
                RenderCommand::Clear(ClearCommand { color: self.accent }),
                RenderCommand::DrawRenderable(DrawRenderableCommand {
                    renderable: TRIANGLE_RENDERABLE,
                    instance: left_instance,
                }),
                RenderCommand::DrawRenderable(DrawRenderableCommand {
                    renderable: TRIANGLE_SECOND_RENDERABLE,
                    instance: right_instance,
                }),
                RenderCommand::DrawRenderable(DrawRenderableCommand {
                    renderable: TRIANGLE_THIRD_RENDERABLE,
                    instance: third_instance,
                }),
                RenderCommand::DrawRenderable(DrawRenderableCommand {
                    renderable: TRIANGLE_FOURTH_RENDERABLE,
                    instance: fourth_instance,
                }),
                RenderCommand::DrawRenderable(DrawRenderableCommand {
                    renderable: QUAD_RENDERABLE,
                    instance: quad_instance,
                }),
                RenderCommand::DrawRenderable(DrawRenderableCommand {
                    renderable: DIAMOND_RENDERABLE,
                    instance: diamond_instance,
                }),
            ]);
            let stats = renderer.present()?;
            (
                stats,
                renderer.name().to_string(),
                renderer.backend_api().to_string(),
                renderer.adapter_name().to_string(),
                renderer.device_kind().to_string(),
            )
        };
        self.update_window_title();

        if !self.logged_backend {
            println!(
                "Tokimu hello-triangle proof: renderer={} backend={} adapter={} device_type={} clear={:?} draw_calls={}",
                renderer_name,
                backend_api,
                adapter_name,
                device_kind,
                self.accent,
                stats.draw_calls
            );
            self.logged_backend = true;
        }

        Ok(())
    }
}

impl HelloTriangleApp {
    fn handle_key_press(&mut self, key: KeyCode) {
        let step = 0.04;

        match key {
            KeyCode::KeyA => self.input_offset[0] -= step,
            KeyCode::KeyD => self.input_offset[0] += step,
            KeyCode::KeyW => self.input_offset[1] += step,
            KeyCode::KeyS => self.input_offset[1] -= step,
            KeyCode::ArrowLeft => self.input_offset[0] -= step * 1.5,
            KeyCode::ArrowRight => self.input_offset[0] += step * 1.5,
            KeyCode::ArrowUp => self.input_offset[1] += step * 1.5,
            KeyCode::ArrowDown => self.input_offset[1] -= step * 1.5,
            KeyCode::Space => {
                self.input_offset = [0.0, 0.0];
                self.cursor_offset = [0.0, 0.0];
                self.square_offset = [0.0, 0.0];
                self.diamond_offset = [0.0, 0.0];
            }
            KeyCode::Escape => {}
        }

        self.input_offset[0] = self.input_offset[0].clamp(-0.5, 0.5);
        self.input_offset[1] = self.input_offset[1].clamp(-0.4, 0.4);
    }

    fn handle_cursor_move(&mut self, x: f32, y: f32) {
        let width = self.window_size[0].max(1.0);
        let height = self.window_size[1].max(1.0);
        let hold_boost = if self.mouse_hold_active { 1.0 } else { 0.0 };
        self.cursor_offset = [
            ((x / width) - 0.5) * (0.6 + hold_boost * 0.25),
            -((y / height) - 0.5) * (0.4 + hold_boost * 0.18),
        ];
        self.square_offset = [self.cursor_offset[0] * (1.0 + hold_boost * 0.25), self.cursor_offset[1] * 0.5];
        self.diamond_offset = [self.cursor_offset[0] * 0.5, self.cursor_offset[1] * (1.0 + hold_boost * 0.3)];
    }

    fn handle_mouse_press(&mut self, button: MouseButton, pressed: bool) {
        if pressed {
            match button {
                MouseButton::Left => self.paused = !self.paused,
                MouseButton::Right => self.palette_mode = !self.palette_mode,
                MouseButton::Middle => self.reverse_motion = !self.reverse_motion,
            }
        }
    }

    fn update_window_title(&self) {
        if let Some(window) = self.window.as_ref() {
            let activity_tag = if self.input_offset == [0.0, 0.0]
                && self.cursor_offset == [0.0, 0.0]
                && self.square_offset == [0.0, 0.0]
                && self.diamond_offset == [0.0, 0.0]
            {
                "neutral"
            } else {
                "active"
            };
            let motion_tag = if self.paused { "paused" } else { "moving" };
            let palette_tag = if self.palette_mode { "alt" } else { "default" };
            let direction_tag = if self.reverse_motion { "reverse" } else { "forward" };
            let hold_tag = if self.mouse_hold_active { "drag" } else { "idle" };
            let title = format!(
                "Tokimu Hello Triangle | mode={} motion={} palette={} direction={} hold={} key=({:.2}, {:.2}) mouse=({:.2}, {:.2})",
                activity_tag,
                motion_tag,
                palette_tag,
                direction_tag,
                hold_tag,
                self.input_offset[0],
                self.input_offset[1],
                self.cursor_offset[0],
                self.cursor_offset[1]
            );
            window.set_title(&title);
        }
    }
}
