//! Native Slice 2 visual comparison for AR-0023.
//!
//! The cutout shaders are deliberately corpus-local candidate mechanisms. They
//! do not change `Textured3d`, introduce renderer vocabulary, or infer policy
//! from the fixture bytes.

use std::{io, sync::Arc};

use hello_alpha_policy::{fixtures, FixtureId, INTERIOR_THRESHOLD, VIEWPORT};
use tokimu::{
    run_window_with_app, BlendMode, Camera, CameraHandle, ClearCommand, Color, CullMode, DepthTest,
    DrawMeshCommand, FrameOutcome, Instance2d, Material, MaterialHandle, Mesh, MeshHandle,
    NativeWindow, Pipeline, PipelineHandle, PipelineKind, PipelineRenderState,
    PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand, Renderer,
    Rgba8TextureColorSpace, Rgba8TextureDescriptor, TextureHandle, WgpuBackend, WindowConfig,
};

const CAMERA: CameraHandle = CameraHandle(1);
const MIXED_TEXTURE: TextureHandle = TextureHandle(1);
const BINARY_TEXTURE: TextureHandle = TextureHandle(2);
const MIXED_MATERIAL: MaterialHandle = MaterialHandle(1);
const BINARY_MATERIAL: MaterialHandle = MaterialHandle(2);
const BACKGROUND_MATERIAL: MaterialHandle = MaterialHandle(3);
const OPAQUE_MESH: MeshHandle = MeshHandle(1);
const CUTOUT_BELOW_MESH: MeshHandle = MeshHandle(2);
const CUTOUT_AT_OR_BELOW_MESH: MeshHandle = MeshHandle(3);
const DEPTH_BACKGROUND_MESH: MeshHandle = MeshHandle(4);
const DEPTH_CUTOUT_MESH: MeshHandle = MeshHandle(5);

struct App {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    size: [f32; 2],
    opaque_pipeline: PipelineHandle,
    cutout_below_pipeline: PipelineHandle,
    cutout_at_or_below_pipeline: PipelineHandle,
    meshes: [(MeshHandle, Mesh); 5],
}

impl Default for App {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            size: [VIEWPORT[0] as f32, VIEWPORT[1] as f32],
            opaque_pipeline: PipelineHandle(0),
            cutout_below_pipeline: PipelineHandle(0),
            cutout_at_or_below_pipeline: PipelineHandle(0),
            meshes: [
                (OPAQUE_MESH, quad_at_depth(0.0)),
                (CUTOUT_BELOW_MESH, quad_at_depth(0.0)),
                (CUTOUT_AT_OR_BELOW_MESH, quad_at_depth(0.0)),
                (DEPTH_BACKGROUND_MESH, quad_at_depth(0.5)),
                (DEPTH_CUTOUT_MESH, quad_at_depth(0.0)),
            ],
        }
    }
}

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Alpha Policy | loading comparative cutout scene".into(),
            width: VIEWPORT[0],
            height: VIEWPORT[1],
        },
        App::default(),
    )
}

impl PlatformEventHandler for App {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window.clone(), size.width, size.height)?;
        upload_fixture_texture(&mut renderer, MIXED_TEXTURE, FixtureId::MixedAlpha)?;
        upload_fixture_texture(&mut renderer, BINARY_TEXTURE, FixtureId::BinaryMask)?;
        renderer.upload_material(
            MIXED_MATERIAL,
            &Material::new("alpha-study-mixed", Color::rgba(1.0, 1.0, 1.0, 1.0))
                .with_texture(MIXED_TEXTURE),
        )?;
        renderer.upload_material(
            BINARY_MATERIAL,
            &Material::new("alpha-study-binary", Color::rgba(1.0, 1.0, 1.0, 1.0))
                .with_texture(BINARY_TEXTURE),
        )?;
        renderer.upload_material(
            BACKGROUND_MATERIAL,
            &Material::new(
                "alpha-study-opaque-background",
                Color::rgb(0.15, 0.55, 0.95),
            ),
        )?;

        let opaque_state = PipelineRenderState {
            blend: BlendMode::Opaque,
            depth_test: DepthTest::LessEqual,
            depth_write: true,
            cull_mode: CullMode::None,
            color_write: Default::default(),
        };
        self.opaque_pipeline = renderer.register_pipeline(
            &Pipeline::new("alpha-study-opaque", PipelineKind::Textured3d)
                .with_render_state(opaque_state)?,
        )?;
        self.cutout_below_pipeline = renderer.register_pipeline(
            &Pipeline::custom_wgsl("alpha-study-cutout-below", cutout_shader("<"))
                .with_render_state(opaque_state)?,
        )?;
        self.cutout_at_or_below_pipeline = renderer.register_pipeline(
            &Pipeline::custom_wgsl("alpha-study-cutout-at-or-below", cutout_shader("<="))
                .with_render_state(opaque_state)?,
        )?;

        window.set_title(&format!(
            "Tokimu Alpha Policy | opaque | cutout < / <= {:.7} | depth write | backend={} device={} adapter={}",
            INTERIOR_THRESHOLD,
            renderer.backend_api(),
            renderer.device_kind(),
            renderer.adapter_name(),
        ));
        self.window = Some(window);
        self.renderer = Some(renderer);
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::Resized { width, height } = event {
            self.size = [width.max(1) as f32, height.max(1) as f32];
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize_surface(width, height);
            }
        }
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let renderer = self
            .renderer
            .as_mut()
            .ok_or_else(|| io::Error::other("renderer missing"))?;
        for (handle, mesh) in &self.meshes {
            renderer.upload_mesh(*handle, mesh);
        }
        renderer.upload_camera(
            CAMERA,
            Camera::orthographic_2d_with_height(self.size[0], self.size[1], 2.0),
        );
        renderer.begin_frame();
        renderer.submit(&[
            RenderCommand::Clear(ClearCommand {
                color: Color::rgb(0.015, 0.02, 0.025),
            }),
            draw(
                OPAQUE_MESH,
                MIXED_MATERIAL,
                self.opaque_pipeline,
                [-1.0, 0.35],
                [0.52, 0.52],
            ),
            draw(
                CUTOUT_BELOW_MESH,
                MIXED_MATERIAL,
                self.cutout_below_pipeline,
                [0.0, 0.35],
                [0.52, 0.52],
            ),
            draw(
                CUTOUT_AT_OR_BELOW_MESH,
                MIXED_MATERIAL,
                self.cutout_at_or_below_pipeline,
                [1.0, 0.35],
                [0.52, 0.52],
            ),
            draw(
                DEPTH_BACKGROUND_MESH,
                BACKGROUND_MATERIAL,
                self.opaque_pipeline,
                [0.0, -0.55],
                [0.95, 0.36],
            ),
            draw(
                DEPTH_CUTOUT_MESH,
                BINARY_MATERIAL,
                self.cutout_below_pipeline,
                [0.0, -0.55],
                [0.95, 0.36],
            ),
        ]);
        renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}

fn draw(
    mesh: MeshHandle,
    material: MaterialHandle,
    pipeline: PipelineHandle,
    translation: [f32; 2],
    scale: [f32; 2],
) -> RenderCommand {
    RenderCommand::DrawMesh(DrawMeshCommand {
        mesh,
        material,
        pipeline,
        instance: Instance2d::new(translation, scale, 0.0),
        camera: Some(CAMERA),
        viewport: None,
    })
}

fn upload_fixture_texture(
    renderer: &mut WgpuBackend,
    handle: TextureHandle,
    fixture_id: FixtureId,
) -> PlatformResult<()> {
    let fixture = fixtures()
        .into_iter()
        .find(|fixture| fixture.id() == fixture_id)
        .ok_or_else(|| io::Error::other("alpha-study fixture missing"))?;
    renderer.create_texture_rgba8(
        handle,
        Rgba8TextureDescriptor::new(
            fixture.width(),
            fixture.height(),
            Rgba8TextureColorSpace::Srgb,
        ),
        fixture.rgba8(),
    )?;
    Ok(())
}

fn quad_at_depth(depth: f32) -> Mesh {
    Mesh::quad()
        .with_texture_coordinates(vec![
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 1.0],
            [0.0, 0.0],
            [1.0, 1.0],
            [1.0, 0.0],
        ])
        .unwrap()
        .with_positions_at_depth(depth)
}

trait MeshDepthExt {
    fn with_positions_at_depth(self, depth: f32) -> Self;
}

impl MeshDepthExt for Mesh {
    fn with_positions_at_depth(mut self, depth: f32) -> Self {
        for position in &mut self.positions {
            position[2] = depth;
        }
        self
    }
}

fn cutout_shader(comparison: &str) -> String {
    format!(
        r#"
@group(0) @binding(0) var<uniform> material_color: vec4<f32>;
@group(0) @binding(1) var material_texture: texture_2d<f32>;
@group(0) @binding(2) var material_sampler: sampler;
struct InstanceParams {{ translation: vec2<f32>, scale: vec2<f32>, rotation: vec2<f32>, padding: vec2<f32>, }};
@group(1) @binding(0) var<uniform> instance_params: InstanceParams;
@group(2) @binding(0) var<uniform> camera_params: mat4x4<f32>;
struct VertexOutput {{ @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32>, }};
@vertex fn vs_main(@location(0) position: vec3<f32>, @location(1) _normal: vec3<f32>, @location(2) uv: vec2<f32>) -> VertexOutput {{
    let scaled = position.xy * instance_params.scale;
    let rotated = vec2<f32>((scaled.x * instance_params.rotation.y) - (scaled.y * instance_params.rotation.x), (scaled.x * instance_params.rotation.x) + (scaled.y * instance_params.rotation.y));
    var output: VertexOutput;
    output.position = camera_params * vec4<f32>(rotated.x + instance_params.translation.x, rotated.y + instance_params.translation.y, position.z, 1.0);
    output.uv = uv;
    return output;
}}
@fragment fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {{
    let sampled = textureSample(material_texture, material_sampler, uv) * material_color;
    if (sampled.a {comparison} {INTERIOR_THRESHOLD:.7}) {{ discard; }}
    return sampled;
}}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_scene_uses_the_shared_threshold_as_explicit_shader_input() {
        assert!(cutout_shader("<").contains("sampled.a < 0.5019608"));
        assert!(cutout_shader("<=").contains("sampled.a <= 0.5019608"));
    }

    #[test]
    fn native_scene_quads_supply_renderer_neutral_uvs() {
        let mesh = quad_at_depth(0.25);
        assert!(mesh.has_texture_coordinates());
        assert!(mesh.positions.iter().all(|position| position[2] == 0.25));
    }
}
