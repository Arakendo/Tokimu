use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand,
    FrameOutcome, Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow,
    Pipeline, PipelineHandle, PipelineKind, PlatformEventHandler, PlatformInputEvent,
    PlatformResult, RenderCommand, Renderer, WgpuBackend, WindowConfig,
};
use tokimu_assets::AssetStore;
use tokimu_core::math::{Mat4, Vec3};

const MODEL_MESH: MeshHandle = MeshHandle(1);
const FLOOR_MESH: MeshHandle = MeshHandle(2);
const MODEL_MATERIAL: MaterialHandle = MaterialHandle(1);
const FLOOR_MATERIAL: MaterialHandle = MaterialHandle(2);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello GLB".into(),
            width: 1280,
            height: 720,
        },
        HelloGlbApp::new(),
    )
}

struct HelloGlbApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    elapsed_seconds: f64,
    pipeline: PipelineHandle,
    assets: AssetStore,
}

impl Default for HelloGlbApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            elapsed_seconds: 0.0,
            pipeline: PipelineHandle(0),
            assets: AssetStore::default(),
        }
    }
}

impl HelloGlbApp {
    fn new() -> Self {
        Self::default()
    }

    fn update_window_title(&self) {
        if let Some(window) = self.window.as_ref() {
            let inventory = self.assets.inventory();
            let source = inventory
                .entries
                .first()
                .and_then(|entry| entry.source.as_deref())
                .unwrap_or("models/cube.glb");
            window.set_title(&format!(
                "Tokimu Hello GLB | source={} | elapsed={:.1}s",
                source, self.elapsed_seconds
            ));
        }
    }

    fn render_scene(&mut self) -> PlatformResult<FrameOutcome> {
        let seconds = self.elapsed_seconds as f32;
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        renderer.upload_mesh(MODEL_MESH, &build_model_mesh(seconds));
        renderer.upload_mesh(FLOOR_MESH, &build_floor_mesh(seconds));

        let mut camera = Camera::perspective_3d(self.window_size[0], self.window_size[1]);
        let orbit = seconds * 0.28;
        let eye = Vec3::new(orbit.cos() * 4.75, 1.8 + orbit.sin() * 0.15, orbit.sin() * 4.75);
        camera.view = Mat4::look_at_rh(eye, Vec3::new(0.0, 0.35, 0.0), Vec3::Y);
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
        ]);
        let _ = renderer.present()?;
        self.update_window_title();
        Ok(FrameOutcome::Continue)
    }
}

impl PlatformEventHandler for HelloGlbApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        self.assets.allocate_with_source::<Mesh, _>("models/cube.glb");

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_material(
            MODEL_MATERIAL,
            &Material::new("glb-model", Color::rgb(0.86, 0.79, 0.72)),
        )?;
        renderer.upload_material(
            FLOOR_MATERIAL,
            &Material::new("glb-floor", Color::rgb(0.08, 0.10, 0.13)),
        )?;
        self.pipeline = renderer.register_pipeline(&Pipeline::new(
            "glb-pipeline",
            PipelineKind::LitColor3d,
        ))?;
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
        self.render_scene()
    }
}

fn build_model_mesh(seconds: f32) -> Mesh {
    let wobble = (seconds * 1.4).sin() * 0.06;
    let twist = seconds * 0.18;
    let transform = Mat4::from_rotation_y(twist)
        * Mat4::from_rotation_x((seconds * 0.7).sin() * 0.15)
        * Mat4::from_scale(Vec3::new(1.0 + wobble * 0.5, 1.0 + wobble, 1.0 + wobble * 0.25))
        * Mat4::from_translation(Vec3::new(0.0, 0.35, 0.0));
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

fn build_floor_mesh(seconds: f32) -> Mesh {
    let pulse = 0.02 + seconds.sin().abs() * 0.01;
    let transform = Mat4::from_translation(Vec3::new(0.0, -0.8, 0.0))
        * Mat4::from_scale(Vec3::new(8.0, pulse, 8.0));
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
