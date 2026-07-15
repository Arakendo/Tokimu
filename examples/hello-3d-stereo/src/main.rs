use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle,
    PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand,
    Renderer, ViewportRect, WgpuBackend, WindowConfig,
};
use tokimu_core::math::{Mat4, Vec3};

const CUBE_MESH: MeshHandle = MeshHandle(1);
const CUBE_MATERIAL: MaterialHandle = MaterialHandle(1);
const LEFT_EYE_CAMERA: CameraHandle = CameraHandle(1);
const RIGHT_EYE_CAMERA: CameraHandle = CameraHandle(2);

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello 3D Stereo".into(),
            width: 1280,
            height: 720,
        },
        Hello3dApp::new(),
    )
}

struct Hello3dApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    elapsed_seconds: f64,
    pipeline: PipelineHandle,
}

impl Default for Hello3dApp {
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

impl Hello3dApp {
    fn new() -> Self {
        Self::default()
    }
}

impl PlatformEventHandler for Hello3dApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(CUBE_MESH, &Mesh::cube());
        renderer.upload_material(
            CUBE_MATERIAL,
            &Material::new("cube-material", Color::rgb(0.92, 0.72, 0.26)),
        )?;
        self.pipeline = renderer
            .register_pipeline(&Pipeline::new("cube-pipeline", PipelineKind::LitColor3d))?;
        self.renderer = Some(renderer);
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
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        self.elapsed_seconds += delta_seconds;
        let spun_cube = spin_cube(self.elapsed_seconds as f32);
        renderer.upload_mesh(CUBE_MESH, &spun_cube);

        let orbit_radius = 3.0;
        let orbit_height = 0.35 + (self.elapsed_seconds as f32 * 1.3).sin() * 0.15;
        let orbit_angle = self.elapsed_seconds as f32 * 0.8;
        let center_eye = Vec3::new(
            orbit_angle.cos() * orbit_radius,
            orbit_height,
            orbit_angle.sin() * orbit_radius,
        );
        let forward = (Vec3::ZERO - center_eye).normalize();
        let right = forward.cross(Vec3::Y).normalize();
        let eye_separation = 0.14;
        let left_eye = center_eye - right * eye_separation;
        let right_eye = center_eye + right * eye_separation;

        let mut left_camera =
            Camera::perspective_3d(self.window_size[0] * 0.5, self.window_size[1]);
        left_camera.view = Mat4::look_at_rh(left_eye, Vec3::ZERO, Vec3::Y);
        let mut right_camera =
            Camera::perspective_3d(self.window_size[0] * 0.5, self.window_size[1]);
        right_camera.view = Mat4::look_at_rh(right_eye, Vec3::ZERO, Vec3::Y);
        renderer.upload_camera(LEFT_EYE_CAMERA, left_camera);
        renderer.upload_camera(RIGHT_EYE_CAMERA, right_camera);

        renderer.begin_frame();
        renderer.submit(&[
            RenderCommand::Clear(ClearCommand {
                color: Color::rgb(0.06, 0.08, 0.12),
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: CUBE_MESH,
                material: CUBE_MATERIAL,
                pipeline: self.pipeline,
                instance: Instance2d::identity(),
                camera: Some(LEFT_EYE_CAMERA),
                viewport: Some(ViewportRect {
                    x: 0.0,
                    y: 0.0,
                    width: self.window_size[0] * 0.5,
                    height: self.window_size[1],
                }),
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: CUBE_MESH,
                material: CUBE_MATERIAL,
                pipeline: self.pipeline,
                instance: Instance2d::identity(),
                camera: Some(RIGHT_EYE_CAMERA),
                viewport: Some(ViewportRect {
                    x: self.window_size[0] * 0.5,
                    y: 0.0,
                    width: self.window_size[0] * 0.5,
                    height: self.window_size[1],
                }),
            }),
        ]);
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}

fn spin_cube(seconds: f32) -> Mesh {
    let yaw = seconds * 0.7;
    let pitch = seconds * 0.45;
    let roll = seconds * 0.25;
    let transform =
        Mat4::from_rotation_y(yaw) * Mat4::from_rotation_x(pitch) * Mat4::from_rotation_z(roll);
    let normal_transform =
        Mat4::from_rotation_y(yaw) * Mat4::from_rotation_x(pitch) * Mat4::from_rotation_z(roll);
    let base_cube = Mesh::cube();

    Mesh::new(
        base_cube
            .positions
            .into_iter()
            .map(|position| {
                transform
                    .transform_point3(Vec3::from_array(position))
                    .to_array()
            })
            .collect(),
        base_cube
            .normals
            .into_iter()
            .map(|normal| {
                normal_transform
                    .transform_vector3(Vec3::from_array(normal))
                    .normalize()
                    .to_array()
            })
            .collect(),
    )
}
