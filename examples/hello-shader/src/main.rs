use std::sync::Arc;

use tokimu::{
    run_window_with_app, Camera, CameraHandle, ClearCommand, Color, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle,
    PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand, Renderer, WgpuBackend,
    WindowConfig,
};
use tokimu_assets::{AssetHandle, AssetStore};
use tokimu_core::math::{Mat4, Vec3};

const QUAD_MESH: MeshHandle = MeshHandle(1);
const TRIANGLE_MESH: MeshHandle = MeshHandle(2);
const DIAMOND_MESH: MeshHandle = MeshHandle(3);
const NEON_MATERIAL: MaterialHandle = MaterialHandle(1);
const INK_MATERIAL: MaterialHandle = MaterialHandle(2);
const HIGHLIGHT_MATERIAL: MaterialHandle = MaterialHandle(3);
const CAMERA_HANDLE: CameraHandle = CameraHandle(1);

const SHADER_VARIANTS: [ShaderVariantDefinition; 3] = [
    ShaderVariantDefinition {
        name: "neon",
        asset_label: "shaders/neon.wgsl",
        source: NEON_WGSL,
        pipeline_label: "shader-neon-pipeline",
        material: NEON_MATERIAL,
    },
    ShaderVariantDefinition {
        name: "ink",
        asset_label: "shaders/ink.wgsl",
        source: INK_WGSL,
        pipeline_label: "shader-ink-pipeline",
        material: INK_MATERIAL,
    },
    ShaderVariantDefinition {
        name: "ripple",
        asset_label: "shaders/ripple.wgsl",
        source: RIPPLE_WGSL,
        pipeline_label: "shader-ripple-pipeline",
        material: HIGHLIGHT_MATERIAL,
    },
];

#[derive(Clone, Copy, Debug)]
struct ShaderAsset;

#[derive(Clone, Copy)]
struct ShaderVariantDefinition {
    name: &'static str,
    asset_label: &'static str,
    source: &'static str,
    pipeline_label: &'static str,
    material: MaterialHandle,
}

struct ShaderVariantRuntime {
    definition: ShaderVariantDefinition,
    asset: AssetHandle<ShaderAsset>,
    pipeline: PipelineHandle,
}

const NEON_WGSL: &str = include_str!("../assets/neon.wgsl");
const INK_WGSL: &str = include_str!("../assets/ink.wgsl");
const RIPPLE_WGSL: &str = include_str!("../assets/ripple.wgsl");

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Hello Shader".into(),
            width: 1280,
            height: 720,
        },
        HelloShaderApp::new(),
    )
}

struct HelloShaderApp {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    window_size: [f32; 2],
    elapsed_seconds: f64,
    shader_variant: usize,
    shader_store: AssetStore,
    shader_variants: Vec<ShaderVariantRuntime>,
}

impl Default for HelloShaderApp {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            window_size: [1.0, 1.0],
            elapsed_seconds: 0.0,
            shader_variant: 0,
            shader_store: AssetStore::default(),
            shader_variants: Vec::new(),
        }
    }
}

impl HelloShaderApp {
    fn new() -> Self {
        Self::default()
    }

    fn update_window_title(&self) {
        if let Some(window) = self.window.as_ref() {
            let variant = &self.shader_variants[self.shader_variant];
            let inventory = self.shader_store.inventory();
            let source_label = inventory
                .entries
                .get(self.shader_variant)
                .and_then(|entry| entry.source.as_deref())
                .unwrap_or(SHADER_VARIANTS[self.shader_variant].asset_label);
            window.set_title(&format!(
                "Tokimu Hello Shader | variant={} ({}) | asset=#{} {} | elapsed={:.1}s",
                self.shader_variant,
                variant.definition.name,
                variant.asset.id().0,
                source_label,
                self.elapsed_seconds
            ));
        }
    }

    fn current_pipeline(&self) -> PipelineHandle {
        self.shader_variants[self.shader_variant % self.shader_variants.len()].pipeline
    }

    fn current_material(&self) -> MaterialHandle {
        self.shader_variants[self.shader_variant % self.shader_variants.len()]
            .definition
            .material
    }

    fn cycle_shader_variant(&mut self, step: isize) {
        let count = self.shader_variants.len() as isize;
        let next = (self.shader_variant as isize + step).rem_euclid(count);
        self.shader_variant = next as usize;
    }

    fn render_scene(&mut self) -> PlatformResult<FrameOutcome> {
        let seconds = self.elapsed_seconds as f32;
        let active_pipeline = self.current_pipeline();
        let active_material = self.current_material();
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };

        renderer.upload_mesh(QUAD_MESH, &Mesh::quad());
        renderer.upload_mesh(TRIANGLE_MESH, &Mesh::triangle());
        renderer.upload_mesh(DIAMOND_MESH, &Mesh::diamond());

        let mut camera =
            Camera::orthographic_2d_with_height(self.window_size[0], self.window_size[1], 4.0);
        camera.view = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
        renderer.upload_camera(CAMERA_HANDLE, camera);
        renderer.set_active_camera(CAMERA_HANDLE);

        let quad_instance = Instance2d::identity()
            .with_translation([-1.9, 0.0])
            .with_scale([1.4, 1.4])
            .with_rotation(seconds * 0.35);
        let triangle_instance = Instance2d::identity()
            .with_translation([0.15, 0.1])
            .with_scale([1.0, 1.0])
            .with_rotation(-seconds * 0.55);
        let diamond_instance = Instance2d::identity()
            .with_translation([2.1, -0.08])
            .with_scale([1.25, 1.25])
            .with_rotation(seconds * 0.8);
        let accent_instance = Instance2d::identity()
            .with_translation([0.0, -1.2 + seconds.sin() * 0.08])
            .with_scale([3.8, 0.22])
            .with_rotation(seconds * 0.12);

        renderer.begin_frame();
        renderer.submit(&[
            RenderCommand::Clear(ClearCommand {
                color: Color::rgb(0.05, 0.06, 0.09),
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: QUAD_MESH,
                material: active_material,
                pipeline: active_pipeline,
                instance: quad_instance,
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: TRIANGLE_MESH,
                material: active_material,
                pipeline: active_pipeline,
                instance: triangle_instance,
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: DIAMOND_MESH,
                material: active_material,
                pipeline: active_pipeline,
                instance: diamond_instance,
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: QUAD_MESH,
                material: HIGHLIGHT_MATERIAL,
                pipeline: active_pipeline,
                instance: accent_instance,
                camera: Some(CAMERA_HANDLE),
                viewport: None,
            }),
        ]);
        let _ = renderer.present()?;
        self.update_window_title();
        Ok(FrameOutcome::Continue)
    }
}

impl PlatformEventHandler for HelloShaderApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.window_size = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.window = Some(window.clone());

        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        renderer.upload_mesh(QUAD_MESH, &Mesh::quad());
        renderer.upload_mesh(TRIANGLE_MESH, &Mesh::triangle());
        renderer.upload_mesh(DIAMOND_MESH, &Mesh::diamond());
        renderer.upload_material(
            NEON_MATERIAL,
            &Material::new("shader-neon", Color::rgb(0.90, 0.36, 0.92)),
        )?;
        renderer.upload_material(
            INK_MATERIAL,
            &Material::new("shader-ink", Color::rgb(0.30, 0.82, 0.96)),
        )?;
        renderer.upload_material(
            HIGHLIGHT_MATERIAL,
            &Material::new("shader-highlight", Color::rgb(0.97, 0.86, 0.44)),
        )?;
        for variant in SHADER_VARIANTS {
            let asset = self
                .shader_store
                .allocate_with_source::<ShaderAsset, _>(variant.asset_label);
            let pipeline = renderer.register_pipeline(&Pipeline::custom_wgsl(
                variant.pipeline_label,
                variant.source,
            ))?;
            self.shader_variants.push(ShaderVariantRuntime {
                definition: variant,
                asset,
                pipeline,
            });
        }
        self.renderer = Some(renderer);
        self.update_window_title();
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::CloseRequested = event {
            return Ok(());
        }

        if let PlatformInputEvent::KeyboardInput { key, pressed } = event {
            if pressed {
                match key {
                    tokimu::KeyCode::Space | tokimu::KeyCode::ArrowRight => {
                        self.cycle_shader_variant(1)
                    }
                    tokimu::KeyCode::ArrowLeft => self.cycle_shader_variant(-1),
                    _ => {}
                }
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
        self.render_scene()
    }
}
