use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand,
    FrameOutcome, Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow,
    Pipeline, PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent,
    PlatformResult, RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use tokimu_core::math::{Mat4, Vec3};

const MODEL_MESH: MeshHandle = MeshHandle(1);
const FLOOR_MESH: MeshHandle = MeshHandle(2);
const TOOL_MESH: MeshHandle = MeshHandle(3);
const MODEL_MATERIAL: MaterialHandle = MaterialHandle(1);
const FLOOR_MATERIAL: MaterialHandle = MaterialHandle(2);
const TOOL_MATERIAL: MaterialHandle = MaterialHandle(3);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

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
    elapsed_seconds: f64,
    pipeline: PipelineHandle,
}

impl Default for HelloCadApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            elapsed_seconds: 0.0,
            pipeline: PipelineHandle(0),
        }
    }
}

impl HelloCadApp {
    fn new() -> Self {
        Self::default()
    }

    fn update_window_title(&self) {
        if let Some(window) = self.window.as_ref() {
            window.set_title(&format!(
                "Tokimu Hello CAD | mesh mutation demo | elapsed={:.1}s",
                self.elapsed_seconds
            ));
        }
    }

    fn render_scene(&mut self) -> PlatformResult<FrameOutcome> {
        let elapsed = self.elapsed_seconds as f32;
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        renderer.upload_mesh(MODEL_MESH, &build_model_mesh(elapsed));
        renderer.upload_mesh(FLOOR_MESH, &build_floor_mesh(elapsed));
        renderer.upload_mesh(TOOL_MESH, &build_tool_mesh(elapsed));

        let mut camera = Camera::perspective_3d(self.window_size[0], self.window_size[1]);
        let orbit = elapsed * 0.32;
        let eye = Vec3::new(orbit.cos() * 5.25, 2.4 + orbit.sin() * 0.2, orbit.sin() * 5.25);
        camera.view = Mat4::look_at_rh(eye, Vec3::new(0.0, 0.25, 0.0), Vec3::Y);
        renderer.upload_camera(CAMERA_HANDLE, camera);

        renderer.begin_frame();
        renderer.submit(&[
            RenderCommand::Clear(ClearCommand {
                color: Color::rgb(0.05, 0.07, 0.11),
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: FLOOR_MESH,
                material: FLOOR_MATERIAL,
                pipeline: self.pipeline,
                instance: Instance2d::identity(),
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: MODEL_MESH,
                material: MODEL_MATERIAL,
                pipeline: self.pipeline,
                instance: Instance2d::identity(),
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: TOOL_MESH,
                material: TOOL_MATERIAL,
                pipeline: self.pipeline,
                instance: Instance2d::identity(),
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            }),
        ]);
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
            &Material::new("cad-model", Color::rgb(0.88, 0.82, 0.72)),
        )?;
        renderer.upload_material(
            FLOOR_MATERIAL,
            &Material::new("cad-floor", Color::rgb(0.08, 0.10, 0.13)),
        )?;
        renderer.upload_material(
            TOOL_MATERIAL,
            &Material::new("cad-tool", Color::rgb(0.96, 0.62, 0.26)),
        )?;
        self.pipeline = renderer
            .register_pipeline(&Pipeline::new("cad-pipeline", PipelineKind::LitColor3d))?;
        self.renderer = Some(renderer);
        self.update_window_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::CloseRequested = event {
            return Ok(());
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
        self.update_window_title();
        self.render_scene()
    }
}

fn build_model_mesh(seconds: f32) -> Mesh {
    let edit_cycle = edit_cycle(seconds);
    mutate_cube_mesh(
        seconds,
        Vec3::new(0.0, 0.2, 0.0),
        Vec3::new(1.05, 1.05, 1.05),
        Vec3::new(edit_cycle * 0.12, seconds * 0.18, edit_cycle * 0.08),
        edit_cycle * 0.16,
    )
}

fn build_floor_mesh(seconds: f32) -> Mesh {
    mutate_cube_mesh(
        seconds,
        Vec3::new(0.0, -1.05, 0.0),
        Vec3::new(10.0, 0.12, 10.0),
        Vec3::ZERO,
        0.0,
    )
}

fn build_tool_mesh(seconds: f32) -> Mesh {
    let orbit = seconds * 0.8;
    let radius = 2.35;
    let translation = Vec3::new(orbit.cos() * radius, 1.1 + orbit.sin() * 0.15, orbit.sin() * radius);
    mutate_cube_mesh(
        seconds,
        translation,
        Vec3::new(0.28, 0.28, 1.35),
        Vec3::new(seconds * 0.6, seconds * 0.25, seconds * 0.9),
        0.02,
    )
}

fn mutate_cube_mesh(seconds: f32, translation: Vec3, scale: Vec3, rotation: Vec3, warp: f32) -> Mesh {
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
                let chamfer_bias = chamfer_edge(point, warp * 0.9, Vec3::new(1.0, 1.0, 0.0));
                let bulge_bias = bulge_face(
                    point,
                    warp * 0.75,
                    Vec3::new(0.0, 1.0, 0.0),
                    seconds,
                );
                let edit_bias = chamfer_bias + bulge_bias;
                transform.transform_point3(point + edit_bias)
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

fn edit_cycle(seconds: f32) -> f32 {
    let phase = seconds.rem_euclid(8.0);
    match phase {
        p if p < 2.0 => p / 2.0,
        p if p < 4.0 => 1.0,
        p if p < 6.0 => 1.0 - (p - 4.0) / 2.0,
        p => 1.0 - (p - 6.0) / 2.0,
    }
}

fn chamfer_edge(point: Vec3, amount: f32, edge_mask: Vec3) -> Vec3 {
    let edge_weight = point.x.abs().min(point.y.abs()).min(0.5);
    let x_bias = point.x.signum() * edge_mask.x * edge_weight * amount * -0.8;
    let y_bias = point.y.signum() * edge_mask.y * edge_weight * -amount;
    let z_bias = edge_mask.z * edge_weight * amount * 0.18;
    Vec3::new(x_bias, y_bias, z_bias)
}

fn bulge_face(point: Vec3, amount: f32, normal: Vec3, seconds: f32) -> Vec3 {
    let face_weight = (point.y.max(0.0) * 2.0).clamp(0.0, 1.0);
    let wobble = (seconds * 1.8 + point.x * 2.0 + point.z * 1.5).sin() * 0.35;
    normal.normalize_or_zero() * amount * face_weight * (1.0 + wobble)
}
