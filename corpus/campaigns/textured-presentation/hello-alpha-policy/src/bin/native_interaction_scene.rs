//! Native Slice 4 alpha-policy interaction fixture for AR-0023.
//!
//! This corpus scene keeps generic textured inputs fixed while comparing
//! cutout-over-opaque, blend-over-opaque, and an actual depth intersection
//! between a fixed cutout plane and a sloped blended plane. It is not a public
//! material, ordering, shader-resource, or batching contract.

use std::{io, sync::Arc};

use hello_alpha_policy::{
    blend_shader_source, fixtures, interaction_manifest_fingerprint, FixtureId,
    INTERACTION_BACKGROUND_DEPTH, INTERACTION_BLEND_LEFT_DEPTH, INTERACTION_BLEND_RIGHT_DEPTH,
    INTERACTION_FOREGROUND_DEPTH, INTERACTION_PANELS, INTERACTION_PANEL_SCALE, INTERIOR_THRESHOLD,
    VIEWPORT,
};
use tokimu::{
    run_window_with_app, BlendMode, Camera, CameraHandle, CategoricalCutout, ClearCommand, Color,
    CullMode, CutoutComparison, CutoutThreshold, DepthTest, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle,
    PipelineKind, PipelineRenderState, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, Rgba8TextureColorSpace, Rgba8TextureDescriptor, TextureHandle,
    WgpuBackend, WindowConfig,
};

const _: () = assert!(INTERACTION_BACKGROUND_DEPTH < INTERACTION_FOREGROUND_DEPTH);

const CAMERA: CameraHandle = CameraHandle(1);
const MIXED_TEXTURE: TextureHandle = TextureHandle(1);
const BINARY_TEXTURE: TextureHandle = TextureHandle(2);
const MIXED_MATERIAL: MaterialHandle = MaterialHandle(1);
const BINARY_MATERIAL: MaterialHandle = MaterialHandle(2);
const BACKGROUND_MATERIAL: MaterialHandle = MaterialHandle(3);
const BACKGROUND_MESH: MeshHandle = MeshHandle(1);
const CUTOUT_MESH: MeshHandle = MeshHandle(2);
const BLEND_MESH: MeshHandle = MeshHandle(3);
const SLOPED_BLEND_MESH: MeshHandle = MeshHandle(4);

struct App {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    size: [f32; 2],
    opaque: PipelineHandle,
    cutout: PipelineHandle,
    blend_no_depth: PipelineHandle,
    blend_depth: PipelineHandle,
    frame_index: u64,
}

impl Default for App {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            size: [VIEWPORT[0] as f32, VIEWPORT[1] as f32],
            opaque: PipelineHandle(0),
            cutout: PipelineHandle(0),
            blend_no_depth: PipelineHandle(0),
            blend_depth: PipelineHandle(0),
            frame_index: 0,
        }
    }
}

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Alpha Policy | loading Slice 4 interaction fixture".into(),
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
            &Material::new("alpha-study-interaction-mixed", Color::rgb(1.0, 1.0, 1.0))
                .with_texture(MIXED_TEXTURE),
        )?;
        renderer.upload_material(
            BINARY_MATERIAL,
            &Material::new("alpha-study-interaction-binary", Color::rgb(1.0, 1.0, 1.0))
                .with_texture(BINARY_TEXTURE),
        )?;
        renderer.upload_material(
            BACKGROUND_MATERIAL,
            &Material::new(
                "alpha-study-interaction-backing",
                Color::rgb(0.1, 0.3, 0.95),
            ),
        )?;

        let opaque_state = PipelineRenderState {
            blend: BlendMode::Opaque,
            depth_test: DepthTest::LessEqual,
            depth_write: true,
            cull_mode: CullMode::None,
            color_write: Default::default(),
        };
        let blend_no_depth_state = PipelineRenderState {
            blend: BlendMode::AlphaBlend,
            depth_test: DepthTest::LessEqual,
            depth_write: false,
            cull_mode: CullMode::None,
            color_write: Default::default(),
        };
        let blend_depth_state = PipelineRenderState {
            depth_write: true,
            ..blend_no_depth_state
        };
        self.opaque = renderer.register_pipeline(
            &Pipeline::new("alpha-study-interaction-opaque", PipelineKind::Textured3d)
                .with_render_state(opaque_state)?,
        )?;
        self.cutout = renderer.register_pipeline(&Pipeline::textured_3d_cutout(
            "alpha-study-interaction-cutout",
            CategoricalCutout::new(
                CutoutThreshold::new(INTERIOR_THRESHOLD)?,
                CutoutComparison::DiscardBelow,
            ),
        ))?;
        self.blend_no_depth = renderer.register_pipeline(
            &Pipeline::custom_wgsl(
                "alpha-study-interaction-blend-no-depth",
                blend_shader_source(),
            )
            .with_render_state(blend_no_depth_state)?,
        )?;
        self.blend_depth = renderer.register_pipeline(
            &Pipeline::custom_wgsl("alpha-study-interaction-blend-depth", blend_shader_source())
                .with_render_state(blend_depth_state)?,
        )?;

        renderer.upload_mesh(
            BACKGROUND_MESH,
            &quad_at_depth(INTERACTION_BACKGROUND_DEPTH),
        );
        renderer.upload_mesh(CUTOUT_MESH, &quad_at_depth(INTERACTION_FOREGROUND_DEPTH));
        renderer.upload_mesh(BLEND_MESH, &quad_at_depth(INTERACTION_FOREGROUND_DEPTH));
        renderer.upload_mesh(
            SLOPED_BLEND_MESH,
            &sloped_quad(INTERACTION_BLEND_LEFT_DEPTH, INTERACTION_BLEND_RIGHT_DEPTH),
        );
        renderer.upload_camera(
            CAMERA,
            Camera::orthographic_2d_with_height(self.size[0], self.size[1], 2.0),
        );
        window.set_title(&format!(
            "Tokimu Alpha Policy | Slice 4 cutout/blend interaction | backend={} device={} adapter={}",
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
                renderer.upload_camera(
                    CAMERA,
                    Camera::orthographic_2d_with_height(self.size[0], self.size[1], 2.0),
                );
            }
        }
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let renderer = self
            .renderer
            .as_mut()
            .ok_or_else(|| io::Error::other("renderer missing"))?;
        renderer.begin_frame();
        renderer.submit(&[
            RenderCommand::Clear(ClearCommand {
                color: Color::rgb(0.015, 0.02, 0.025),
            }),
            // Cutout over opaque.
            draw(
                BACKGROUND_MESH,
                BACKGROUND_MATERIAL,
                self.opaque,
                INTERACTION_PANELS[0],
                0.0,
            ),
            draw(
                CUTOUT_MESH,
                BINARY_MATERIAL,
                self.cutout,
                INTERACTION_PANELS[0],
                0.0,
            ),
            // No-depth-write Blend over opaque.
            draw(
                BACKGROUND_MESH,
                BACKGROUND_MATERIAL,
                self.opaque,
                INTERACTION_PANELS[1],
                0.0,
            ),
            draw(
                BLEND_MESH,
                MIXED_MATERIAL,
                self.blend_no_depth,
                INTERACTION_PANELS[1],
                0.0,
            ),
            // The cutout plane intersects the sloped Blend plane in depth.
            draw(
                BACKGROUND_MESH,
                BACKGROUND_MATERIAL,
                self.opaque,
                INTERACTION_PANELS[2],
                0.0,
            ),
            draw(
                SLOPED_BLEND_MESH,
                MIXED_MATERIAL,
                self.blend_depth,
                INTERACTION_PANELS[2],
                0.0,
            ),
            draw(
                CUTOUT_MESH,
                BINARY_MATERIAL,
                self.cutout,
                INTERACTION_PANELS[2],
                0.0,
            ),
        ]);
        let stats = renderer.present()?;
        renderer.poll_diagnostics();
        let diagnostic = renderer
            .drain_diagnostics()
            .first()
            .map(|record| record.message.clone())
            .unwrap_or_else(|| "none".to_owned());
        self.frame_index = self.frame_index.saturating_add(1);
        if self.frame_index == 1 {
            let observation = format!(
                "Tokimu Alpha Policy | Slice 4 interaction | first frame: {} draws, {} material resolutions, {} pipeline switches, manifest={}, diagnostic={diagnostic}",
                stats.frame.draw_calls,
                stats.frame.material_resolutions,
                stats.frame.pipeline_switches,
                interaction_manifest_fingerprint().expect("fixed interaction manifest serializes"),
            );
            println!("{observation}");
            if let Some(window) = self.window.as_ref() {
                window.set_title(&observation);
            }
        }
        Ok(FrameOutcome::Continue)
    }
}

fn draw(
    mesh: MeshHandle,
    material: MaterialHandle,
    pipeline: PipelineHandle,
    translation: [f32; 2],
    rotation: f32,
) -> RenderCommand {
    RenderCommand::DrawMesh(DrawMeshCommand {
        mesh,
        material,
        pipeline,
        instance: Instance2d::new(translation, INTERACTION_PANEL_SCALE, rotation),
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
    let mut mesh = Mesh::quad()
        .with_texture_coordinates(vec![
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 1.0],
            [0.0, 0.0],
            [1.0, 1.0],
            [1.0, 0.0],
        ])
        .expect("fixed UV count matches quad");
    for position in &mut mesh.positions {
        position[2] = depth;
    }
    mesh
}

fn sloped_quad(left_depth: f32, right_depth: f32) -> Mesh {
    let mut mesh = quad_at_depth(left_depth);
    for position in &mut mesh.positions {
        position[2] = if position[0] < 0.0 {
            left_depth
        } else {
            right_depth
        };
    }
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interaction_mesh_crosses_the_cutout_depth() {
        let mesh = sloped_quad(INTERACTION_BLEND_LEFT_DEPTH, INTERACTION_BLEND_RIGHT_DEPTH);
        assert!(mesh
            .positions
            .iter()
            .any(|position| position[2] < INTERACTION_FOREGROUND_DEPTH));
        assert!(mesh
            .positions
            .iter()
            .any(|position| position[2] > INTERACTION_FOREGROUND_DEPTH));
        assert!(mesh.has_texture_coordinates());
    }
}
