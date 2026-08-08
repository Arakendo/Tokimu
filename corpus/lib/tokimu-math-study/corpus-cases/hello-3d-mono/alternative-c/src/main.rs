//! Alternative C port of the `hello-3d-mono` application shell.
//!
//! The app's math-facing code uses only the owned candidate vocabulary. The
//! renderer camera's provider representation is isolated in a study adapter.

use std::sync::Arc;

use tokimu::{
    run_window_with_app, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle,
    PipelineKind, PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand,
    Renderer, WgpuBackend, WindowConfig,
};
use tokimu_math_study::{
    alternative_c::{Mat4, Vec3},
    hello_3d_mono_adapters::alternative_c_camera,
};

const CUBE_MESH: MeshHandle = MeshHandle(1);
const CUBE_MATERIAL: MaterialHandle = MaterialHandle(1);
const CAMERA: CameraHandle = CameraHandle(1);

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello 3D Mono (math study C)".into(),
            width: 1280,
            height: 720,
        },
        Hello3dMonoApp::new(),
    )
}

struct Hello3dMonoApp {
    renderer: Option<WgpuBackend>,
    window_size: [f32; 2],
    elapsed_seconds: f64,
    pipeline: PipelineHandle,
}

impl Default for Hello3dMonoApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window_size: [1.0, 1.0],
            elapsed_seconds: 0.0,
            pipeline: PipelineHandle(0),
        }
    }
}

impl Hello3dMonoApp {
    fn new() -> Self {
        Self::default()
    }
}

impl PlatformEventHandler for Hello3dMonoApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
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
        renderer.upload_mesh(CUBE_MESH, &spin_cube(self.elapsed_seconds as f32));
        let angle = self.elapsed_seconds as f32 * 0.8;
        let eye = Vec3::new(
            angle.cos() * 3.0,
            0.35 + (self.elapsed_seconds as f32 * 1.3).sin() * 0.15,
            angle.sin() * 3.0,
        );
        renderer.upload_camera(
            CAMERA,
            alternative_c_camera(
                self.window_size[0],
                self.window_size[1],
                Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Y),
            ),
        );
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
                camera: Some(CAMERA),
                viewport: None,
            }),
        ]);
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}

fn spin_cube(seconds: f32) -> Mesh {
    let transform = Mat4::from_rotation_y(seconds * 0.7)
        * Mat4::from_rotation_x(seconds * 0.45)
        * Mat4::from_rotation_z(seconds * 0.25);
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
                transform
                    .transform_vector3(Vec3::from_array(normal))
                    .normalize()
                    .to_array()
            })
            .collect(),
    )
}
