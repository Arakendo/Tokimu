use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, MouseButton, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use tokimu_core::math::{Mat4, Vec3, Vec4};
use ui_tools::{window_to_world, UiButtonId, UiButtonSpec, UiToolbarLayout};

const MODEL_MESH: MeshHandle = MeshHandle(1);
const UI_MESH: MeshHandle = MeshHandle(2);
const MODEL_MATERIAL: MaterialHandle = MaterialHandle(1);
const MODEL_SELECTED_MATERIAL: MaterialHandle = MaterialHandle(2);
const UI_BUTTON_MATERIAL: MaterialHandle = MaterialHandle(4);
const UI_BUTTON_HOVER_MATERIAL: MaterialHandle = MaterialHandle(5);
const UI_ACTIVE_MATERIAL: MaterialHandle = MaterialHandle(6);
const UI_HEADER_MATERIAL: MaterialHandle = MaterialHandle(7);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);
const UI_CAMERA_HANDLE: CameraHandle = CameraHandle(2);

const CUBE_MIN: Vec3 = Vec3::splat(-0.5);
const CUBE_MAX: Vec3 = Vec3::splat(0.5);

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello CAD".into(),
            width: 1280,
            height: 720,
        },
        HelloCadApp::new(),
    )
}

struct HelloCadApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    pipeline: PipelineHandle,
    ui_pipeline: PipelineHandle,
    cube_selected: bool,
    cursor_position: [f32; 2],
    hovered_button: Option<UiButtonId>,
    model_offset: Vec3,
    model_rotation: Vec3,
    model_scale: Vec3,
}

impl Default for HelloCadApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            pipeline: PipelineHandle(0),
            ui_pipeline: PipelineHandle(0),
            cube_selected: false,
            cursor_position: [0.0, 0.0],
            hovered_button: None,
            model_offset: Vec3::ZERO,
            model_rotation: Vec3::ZERO,
            model_scale: Vec3::ONE,
        }
    }
}

impl HelloCadApp {
    fn new() -> Self {
        Self::default()
    }

    fn toolbar_layout(&self) -> UiToolbarLayout {
        UiToolbarLayout::new(
            self.window_size,
            [
                UiButtonSpec::new(UiButtonId(0), "select"),
                UiButtonSpec::new(UiButtonId(1), "deselect"),
                UiButtonSpec::new(UiButtonId(2), "reset"),
            ],
        )
    }

    fn camera_for_scene(&self) -> Camera {
        let mut camera = Camera::perspective_3d(self.window_size[0], self.window_size[1]);
        camera.view =
            Mat4::look_at_rh(Vec3::new(4.0, 3.0, 4.0), Vec3::new(0.0, 0.25, 0.0), Vec3::Y);
        camera
    }

    fn camera_for_overlay(&self) -> Camera {
        Camera::orthographic_2d(self.window_size[0], self.window_size[1])
    }

    fn hovered_control(&self) -> Option<UiButtonId> {
        self.toolbar_layout()
            .button_at(window_to_world(self.window_size, self.cursor_position))
    }

    fn cube_transform(&self) -> Mat4 {
        Mat4::from_translation(Vec3::new(0.0, 0.2, 0.0) + self.model_offset)
            * Mat4::from_rotation_z(self.model_rotation.z)
            * Mat4::from_rotation_y(self.model_rotation.y)
            * Mat4::from_rotation_x(self.model_rotation.x)
            * Mat4::from_scale(Vec3::new(1.05, 1.05, 1.05) * self.model_scale)
    }

    fn camera_ray_from_cursor(&self) -> Option<(Vec3, Vec3)> {
        let width = self.window_size[0].max(1.0);
        let height = self.window_size[1].max(1.0);
        let ndc_x = (self.cursor_position[0] / width) * 2.0 - 1.0;
        let ndc_y = 1.0 - (self.cursor_position[1] / height) * 2.0;
        let camera = self.camera_for_scene();
        let inverse_view_projection = (camera.projection * camera.view).inverse();

        let near = inverse_view_projection * Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
        let far = inverse_view_projection * Vec4::new(ndc_x, ndc_y, 1.0, 1.0);
        if near.w.abs() <= f32::EPSILON || far.w.abs() <= f32::EPSILON {
            return None;
        }

        let world_near = near.truncate() / near.w;
        let world_far = far.truncate() / far.w;
        let direction = (world_far - world_near).normalize_or_zero();
        if direction.length_squared() <= f32::EPSILON {
            None
        } else {
            Some((world_near, direction))
        }
    }

    fn cube_hit_test(&self) -> bool {
        let Some((ray_origin, ray_direction)) = self.camera_ray_from_cursor() else {
            return false;
        };

        let inverse_transform = self.cube_transform().inverse();
        let local_origin = (inverse_transform * ray_origin.extend(1.0)).truncate();
        let local_direction = (inverse_transform * (ray_origin + ray_direction).extend(1.0))
            .truncate()
            - local_origin;

        ray_intersects_aabb(local_origin, local_direction, CUBE_MIN, CUBE_MAX)
    }

    fn apply_control(&mut self, control: UiButtonId) {
        match control.0 {
            0 => self.cube_selected = true,
            1 => self.cube_selected = false,
            2 => {
                self.cube_selected = false;
                self.model_offset = Vec3::ZERO;
                self.model_rotation = Vec3::ZERO;
                self.model_scale = Vec3::ONE;
            }
            _ => {}
        }
    }

    fn update_window_title(&self) {
        if let Some(window) = self.window.as_ref() {
            let layout = self.toolbar_layout();
            let hover_label = self
                .hovered_button
                .map(|button| layout.button_label(button))
                .unwrap_or("none");
            window.set_title(&format!(
                "Tokimu Hello CAD | cube={} | hover={} | select/deselect/reset | arrows move | Q/E rotate | Z/X scale | R reset",
                if self.cube_selected { "selected" } else { "idle" },
                hover_label,
            ));
        }
    }

    fn render_scene(&mut self) -> PlatformResult<FrameOutcome> {
        let model_mesh = build_model_mesh(self.model_offset, self.model_rotation, self.model_scale);
        let layout = self.toolbar_layout();
        let hovered = self.hovered_button;
        let scene_camera = self.camera_for_scene();
        let overlay_camera = self.camera_for_overlay();
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        renderer.upload_mesh(MODEL_MESH, &model_mesh);
        renderer.upload_mesh(UI_MESH, &Mesh::quad());
        renderer.upload_camera(CAMERA_HANDLE, scene_camera);
        renderer.upload_camera(UI_CAMERA_HANDLE, overlay_camera);

        renderer.begin_frame();
        let commands = vec![
            RenderCommand::Clear(ClearCommand {
                color: Color::rgb(0.07, 0.08, 0.10),
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: MODEL_MESH,
                material: if self.cube_selected {
                    MODEL_SELECTED_MATERIAL
                } else {
                    MODEL_MATERIAL
                },
                pipeline: self.pipeline,
                instance: Instance2d::identity(),
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: UI_MESH,
                material: UI_HEADER_MATERIAL,
                pipeline: self.ui_pipeline,
                instance: Instance2d::new(layout.header.center, layout.header.size, 0.0),
                camera: Some(UI_CAMERA_HANDLE),
                viewport: None,
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: UI_MESH,
                material: UI_BUTTON_MATERIAL,
                pipeline: self.ui_pipeline,
                instance: Instance2d::new(layout.toolbar.center, layout.toolbar.size, 0.0),
                camera: Some(UI_CAMERA_HANDLE),
                viewport: None,
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: UI_MESH,
                material: if self.cube_selected {
                    UI_ACTIVE_MATERIAL
                } else {
                    UI_BUTTON_MATERIAL
                },
                pipeline: self.ui_pipeline,
                instance: Instance2d::new(layout.status.center, layout.status.size, 0.0),
                camera: Some(UI_CAMERA_HANDLE),
                viewport: None,
            }),
        ];

        for button in layout.buttons {
            let button_material = if Some(button.id) == hovered {
                UI_BUTTON_HOVER_MATERIAL
            } else if self.cube_selected && button.id == UiButtonId(0) {
                UI_ACTIVE_MATERIAL
            } else {
                UI_BUTTON_MATERIAL
            };
            renderer.submit(&[RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: UI_MESH,
                material: button_material,
                pipeline: self.ui_pipeline,
                instance: Instance2d::new(button.rect.center, button.rect.size, 0.0),
                camera: Some(UI_CAMERA_HANDLE),
                viewport: None,
            })]);
        }

        renderer.submit(&commands);
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}

impl PlatformEventHandler for HelloCadApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_material(
            MODEL_MATERIAL,
            &Material::new("cad-model", Color::rgb(0.82, 0.78, 0.68)),
        )?;
        renderer.upload_material(
            MODEL_SELECTED_MATERIAL,
            &Material::new("cad-model-selected", Color::rgb(0.98, 0.86, 0.42)),
        )?;
        renderer.upload_material(
            UI_HEADER_MATERIAL,
            &Material::new("cad-ui-header", Color::rgb(0.15, 0.18, 0.27)),
        )?;
        renderer.upload_material(
            UI_BUTTON_MATERIAL,
            &Material::new("cad-ui-button", Color::rgb(0.19, 0.23, 0.31)),
        )?;
        renderer.upload_material(
            UI_BUTTON_HOVER_MATERIAL,
            &Material::new("cad-ui-button-hover", Color::rgb(0.29, 0.35, 0.46)),
        )?;
        renderer.upload_material(
            UI_ACTIVE_MATERIAL,
            &Material::new("cad-ui-active", Color::rgb(0.34, 0.76, 0.56)),
        )?;
        self.pipeline =
            renderer.register_pipeline(&Pipeline::new("cad-pipeline", PipelineKind::LitColor3d))?;
        self.ui_pipeline = renderer.register_pipeline(&Pipeline::new(
            "cad-ui-pipeline",
            PipelineKind::SolidColor2d,
        ))?;
        self.renderer = Some(renderer);
        self.update_window_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::CloseRequested = event {
            return Ok(());
        }

        if let PlatformInputEvent::CursorMoved { x, y } = event {
            self.cursor_position = [x, y];
            self.hovered_button = self.hovered_control();
            self.update_window_title();
        }

        if let PlatformInputEvent::MouseInput {
            button: MouseButton::Left,
            pressed: true,
        } = event
        {
            if let Some(button) = self.hovered_control() {
                self.apply_control(button);
            } else if self.cube_hit_test() {
                self.cube_selected = !self.cube_selected;
            } else {
                self.cube_selected = false;
            }

            self.update_window_title();
        }

        if let PlatformInputEvent::KeyboardInput { key, pressed: true } = event {
            let step = 0.12;
            match key {
                tokimu::KeyCode::ArrowLeft => self.model_offset.x -= step,
                tokimu::KeyCode::ArrowRight => self.model_offset.x += step,
                tokimu::KeyCode::ArrowUp => self.model_offset.y += step,
                tokimu::KeyCode::ArrowDown => self.model_offset.y -= step,
                tokimu::KeyCode::KeyQ => self.model_rotation.y += 0.18,
                tokimu::KeyCode::KeyE => self.model_rotation.y -= 0.18,
                tokimu::KeyCode::KeyZ => self.model_scale *= 0.92,
                tokimu::KeyCode::KeyX => self.model_scale *= 1.08,
                tokimu::KeyCode::KeyR => {
                    self.apply_control(UiButtonId(2));
                }
                _ => {}
            }

            self.model_scale.x = self.model_scale.x.clamp(0.25, 4.0);
            self.model_scale.y = self.model_scale.y.clamp(0.25, 4.0);
            self.model_scale.z = self.model_scale.z.clamp(0.25, 4.0);
            self.update_window_title();
        }

        if let PlatformInputEvent::Resized { width, height } = event {
            self.window_size = [width.max(1) as f32, height.max(1) as f32];
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize_surface(width, height);
            }
        }

        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        self.hovered_button = self.hovered_control();
        self.update_window_title();
        self.render_scene()
    }
}

fn build_model_mesh(offset: Vec3, rotation: Vec3, scale: Vec3) -> Mesh {
    mutate_cube_mesh(
        Vec3::new(0.0, 0.2, 0.0) + offset,
        Vec3::new(1.05, 1.05, 1.05) * scale,
        rotation,
    )
}

fn mutate_cube_mesh(translation: Vec3, scale: Vec3, rotation: Vec3) -> Mesh {
    let transform = Mat4::from_translation(translation)
        * Mat4::from_rotation_z(rotation.z)
        * Mat4::from_rotation_y(rotation.y)
        * Mat4::from_rotation_x(rotation.x)
        * Mat4::from_scale(scale);
    let normal_transform = transform.inverse().transpose();
    let base = Mesh::cube();

    Mesh::new(
        base.positions
            .into_iter()
            .map(|position| {
                let point = Vec3::from_array(position);
                transform.transform_point3(point).to_array()
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

fn ray_intersects_aabb(origin: Vec3, direction: Vec3, min: Vec3, max: Vec3) -> bool {
    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;

    for axis in 0..3 {
        let origin_component = origin[axis];
        let direction_component = direction[axis];

        if direction_component.abs() <= 1.0e-6 {
            if origin_component < min[axis] || origin_component > max[axis] {
                return false;
            }
            continue;
        }

        let inv_direction = 1.0 / direction_component;
        let mut t1 = (min[axis] - origin_component) * inv_direction;
        let mut t2 = (max[axis] - origin_component) * inv_direction;
        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }

        t_min = t_min.max(t1);
        t_max = t_max.min(t2);
        if t_max < t_min {
            return false;
        }
    }

    t_max >= 0.0
}
