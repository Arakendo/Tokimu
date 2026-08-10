//! Native Slice 3 blend ordering and depth-write comparison for AR-0023.
//!
//! The four panels hold the exact fixture, camera, UVs, and blend equation
//! steady. Only the caller submission order or explicit depth-write state
//! changes. This is corpus evidence, not a renderer admission.

use std::{io, sync::Arc};

use hello_alpha_policy::{
    blend_shader_source, fixtures, FixtureId, BLEND_BACKGROUND_DEPTH, BLEND_FAR_DEPTH,
    BLEND_FAR_OFFSET, BLEND_NEAR_DEPTH, BLEND_NEAR_OFFSET, BLEND_PANELS, BLEND_PANEL_SCALE,
    BLEND_REFERENCE_DEPTH, BLEND_REFERENCE_TRANSLATION, VIEWPORT,
};
use tokimu::{
    run_window_with_app, BlendMode, Camera, CameraHandle, ClearCommand, Color, CullMode, DepthTest,
    DrawMeshCommand, FrameOutcome, Instance2d, Material, MaterialHandle, Mesh, MeshHandle,
    NativeWindow, Pipeline, PipelineHandle, PipelineRenderState, PlatformEventHandler,
    PlatformInputEvent, PlatformResult, RenderCommand, Renderer, Rgba8TextureColorSpace,
    Rgba8TextureDescriptor, TextureHandle, WgpuBackend, WindowConfig,
};

const _: () = assert!(BLEND_NEAR_DEPTH > BLEND_FAR_DEPTH);
const _: () = assert!(BLEND_FAR_DEPTH > BLEND_BACKGROUND_DEPTH);

const CAMERA: CameraHandle = CameraHandle(1);
const MIXED_TEXTURE: TextureHandle = TextureHandle(1);
const GRADIENT_TEXTURE: TextureHandle = TextureHandle(2);
const RED_MATERIAL: MaterialHandle = MaterialHandle(1);
const GREEN_MATERIAL: MaterialHandle = MaterialHandle(2);
const BLUE_MATERIAL: MaterialHandle = MaterialHandle(3);
const GRADIENT_MATERIAL: MaterialHandle = MaterialHandle(4);
const FAR_BLEND_MESH: MeshHandle = MeshHandle(1);
const NEAR_BLEND_MESH: MeshHandle = MeshHandle(2);
const BACKGROUND_MESH: MeshHandle = MeshHandle(3);
const REFERENCE_MESH: MeshHandle = MeshHandle(4);
const GRADIENT_MESH: MeshHandle = MeshHandle(5);
// Positive Tokimu/GL clip depths deliberately pressure the WGPU adapter's
// explicit conversion into WebGPU's [0, 1] clip-depth interval (AR-0024).

struct App {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    size: [f32; 2],
    alpha_no_depth: PipelineHandle,
    alpha_depth: PipelineHandle,
    opaque: PipelineHandle,
    solid_reference: PipelineHandle,
    frame_index: u64,
}

impl Default for App {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            size: [VIEWPORT[0] as f32, VIEWPORT[1] as f32],
            alpha_no_depth: PipelineHandle(0),
            alpha_depth: PipelineHandle(0),
            opaque: PipelineHandle(0),
            solid_reference: PipelineHandle(0),
            frame_index: 0,
        }
    }
}

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Alpha Policy | loading blend comparison".into(),
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
        self.window = Some(window.clone());
        let mut renderer = WgpuBackend::for_window(window.clone(), size.width, size.height)?;
        require_invalid_pipeline_state_rejection()?;
        upload_fixture_texture(&mut renderer, MIXED_TEXTURE, FixtureId::MixedAlpha)?;
        upload_fixture_texture(
            &mut renderer,
            GRADIENT_TEXTURE,
            FixtureId::ContinuousGradient,
        )?;
        for (handle, label, color) in [
            (
                RED_MATERIAL,
                "alpha-study-red",
                Color::rgba(1.0, 0.2, 0.2, 1.0),
            ),
            (
                GREEN_MATERIAL,
                "alpha-study-green",
                Color::rgba(0.2, 1.0, 0.3, 1.0),
            ),
        ] {
            renderer.upload_material(
                handle,
                &Material::new(label, color).with_texture(MIXED_TEXTURE),
            )?;
        }
        renderer.upload_material(
            BLUE_MATERIAL,
            &Material::new("alpha-study-blue-backing", Color::rgb(0.1, 0.3, 0.95)),
        )?;
        renderer.upload_material(
            GRADIENT_MATERIAL,
            &Material::new("alpha-study-continuous-gradient", Color::rgb(1.0, 1.0, 1.0))
                .with_texture(GRADIENT_TEXTURE),
        )?;

        let alpha_no_depth = PipelineRenderState {
            blend: BlendMode::AlphaBlend,
            depth_test: DepthTest::LessEqual,
            depth_write: false,
            cull_mode: CullMode::None,
            color_write: Default::default(),
        };
        let alpha_depth = PipelineRenderState {
            depth_write: true,
            ..alpha_no_depth
        };
        let opaque = PipelineRenderState {
            blend: BlendMode::Opaque,
            depth_test: DepthTest::LessEqual,
            depth_write: true,
            cull_mode: CullMode::None,
            color_write: Default::default(),
        };
        self.alpha_no_depth = renderer.register_pipeline(
            &Pipeline::custom_wgsl("alpha-study-blend-no-depth-write", blend_shader_source())
                .with_render_state(alpha_no_depth)?,
        )?;
        self.alpha_depth = renderer.register_pipeline(
            &Pipeline::custom_wgsl("alpha-study-blend-depth-write", blend_shader_source())
                .with_render_state(alpha_depth)?,
        )?;
        self.opaque = renderer.register_pipeline(
            &Pipeline::new(
                "alpha-study-opaque-backing",
                tokimu::PipelineKind::Textured3d,
            )
            .with_render_state(opaque)?,
        )?;
        self.solid_reference = renderer.register_pipeline(&Pipeline::new(
            "alpha-study-solid-reference",
            tokimu::PipelineKind::SolidColor2d,
        ))?;
        // These meshes and the initial camera are fixed corpus inputs. Upload
        // them once so a later warm frame measures renderer reuse rather than
        // fixture-authored resource churn.
        renderer.upload_mesh(FAR_BLEND_MESH, &quad_at_depth(BLEND_FAR_DEPTH));
        renderer.upload_mesh(NEAR_BLEND_MESH, &quad_at_depth(BLEND_NEAR_DEPTH));
        renderer.upload_mesh(BACKGROUND_MESH, &quad_at_depth(BLEND_BACKGROUND_DEPTH));
        renderer.upload_mesh(REFERENCE_MESH, &quad_at_depth(BLEND_REFERENCE_DEPTH));
        renderer.upload_mesh(GRADIENT_MESH, &quad_at_depth(BLEND_REFERENCE_DEPTH));
        renderer.upload_camera(
            CAMERA,
            Camera::orthographic_2d_with_height(self.size[0], self.size[1], 2.0),
        );
        window.set_title(&format!(
            "Tokimu Alpha Policy | blend order + depth write | invalid-state=rejected | backend={} device={} adapter={}",
            renderer.backend_api(),
            renderer.device_kind(),
            renderer.adapter_name()
        ));
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
            // The opaque backing plus continuous-gradient overlay makes both
            // provider visibility and every alpha byte 0..=255 observable.
            draw(
                REFERENCE_MESH,
                BLUE_MATERIAL,
                self.solid_reference,
                BLEND_REFERENCE_TRANSLATION,
            ),
            draw(
                GRADIENT_MESH,
                GRADIENT_MATERIAL,
                self.alpha_no_depth,
                BLEND_REFERENCE_TRANSLATION,
            ),
            // Top left: caller submits far then near. Top right reverses only that sequence.
            draw(
                FAR_BLEND_MESH,
                RED_MATERIAL,
                self.alpha_no_depth,
                at(BLEND_PANELS[0], BLEND_FAR_OFFSET),
            ),
            draw(
                NEAR_BLEND_MESH,
                GREEN_MATERIAL,
                self.alpha_no_depth,
                at(BLEND_PANELS[0], BLEND_NEAR_OFFSET),
            ),
            draw(
                NEAR_BLEND_MESH,
                GREEN_MATERIAL,
                self.alpha_no_depth,
                at(BLEND_PANELS[1], BLEND_NEAR_OFFSET),
            ),
            draw(
                FAR_BLEND_MESH,
                RED_MATERIAL,
                self.alpha_no_depth,
                at(BLEND_PANELS[1], BLEND_FAR_OFFSET),
            ),
            // Lower panels keep the same near-to-far caller order and differ
            // only in the explicit depth-write state. With writes enabled the
            // later far quad fails depth; without writes it still contributes.
            draw(BACKGROUND_MESH, BLUE_MATERIAL, self.opaque, BLEND_PANELS[2]),
            draw(
                NEAR_BLEND_MESH,
                GREEN_MATERIAL,
                self.alpha_no_depth,
                at(BLEND_PANELS[2], BLEND_NEAR_OFFSET),
            ),
            draw(
                FAR_BLEND_MESH,
                RED_MATERIAL,
                self.alpha_no_depth,
                at(BLEND_PANELS[2], BLEND_FAR_OFFSET),
            ),
            draw(BACKGROUND_MESH, BLUE_MATERIAL, self.opaque, BLEND_PANELS[3]),
            draw(
                NEAR_BLEND_MESH,
                GREEN_MATERIAL,
                self.alpha_depth,
                at(BLEND_PANELS[3], BLEND_NEAR_OFFSET),
            ),
            draw(
                FAR_BLEND_MESH,
                RED_MATERIAL,
                self.alpha_depth,
                at(BLEND_PANELS[3], BLEND_FAR_OFFSET),
            ),
        ]);
        let stats = renderer.present()?;
        renderer.poll_diagnostics();
        let diagnostics = renderer.drain_diagnostics();
        let diagnostic_summary = diagnostics
            .first()
            .map(|record| record.message.as_str())
            .unwrap_or("none");
        self.frame_index = self.frame_index.saturating_add(1);
        if self.frame_index == 1 {
            if let Some(window) = self.window.as_ref() {
                let observation = format!(
                    "Tokimu Alpha Policy | blend order + depth write | invalid-state=rejected | first frame: {} draws, {} material resolutions, {} pipeline switches, {} binding allocations, {} uniform writes, {} mesh uploads, diagnostic={diagnostic_summary}",
                    stats.frame.draw_calls,
                    stats.frame.material_resolutions,
                    stats.frame.pipeline_switches,
                    stats.frame.binding_allocations,
                    stats.frame.uniform_buffer_writes,
                    stats.frame.mesh_uploads,
                );
                println!("{observation}");
                window.set_title(&observation);
            }
        } else if self.frame_index == 2 {
            if let Some(window) = self.window.as_ref() {
                let observation = format!(
                    "Tokimu Alpha Policy | blend order + depth write | warm frame: {} draws, {} material resolutions, {} pipeline switches, {} binding allocations, {} uniform writes, {} mesh uploads, diagnostic={diagnostic_summary}",
                    stats.frame.draw_calls,
                    stats.frame.material_resolutions,
                    stats.frame.pipeline_switches,
                    stats.frame.binding_allocations,
                    stats.frame.uniform_buffer_writes,
                    stats.frame.mesh_uploads,
                );
                println!("{observation}");
                window.set_title(&observation);
            }
        }
        Ok(FrameOutcome::Continue)
    }
}

fn at(panel: [f32; 2], offset: [f32; 2]) -> [f32; 2] {
    [panel[0] + offset[0], panel[1] + offset[1]]
}

fn draw(
    mesh: MeshHandle,
    material: MaterialHandle,
    pipeline: PipelineHandle,
    translation: [f32; 2],
) -> RenderCommand {
    RenderCommand::DrawMesh(DrawMeshCommand {
        mesh,
        material,
        pipeline,
        instance: Instance2d::new(translation, BLEND_PANEL_SCALE, 0.0),
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
        .ok_or_else(|| io::Error::other(format!("alpha-study fixture missing: {fixture_id:?}")))?;
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

fn require_invalid_pipeline_state_rejection() -> PlatformResult<()> {
    let invalid_state = PipelineRenderState {
        depth_test: DepthTest::Disabled,
        depth_write: true,
        ..PipelineRenderState::painter_ordered_2d()
    };
    if Pipeline::new(
        "alpha-study-invalid-depth-state",
        tokimu::PipelineKind::Textured3d,
    )
    .with_render_state(invalid_state)
    .is_ok()
    {
        return Err(Box::new(io::Error::other(
            "invalid alpha-study depth state unexpectedly passed pipeline validation",
        )));
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blend_scene_keeps_caller_order_and_depth_state_separate() {
        assert!(blend_shader_source().contains("return textureSample"));
        assert!(!blend_shader_source().contains("discard"));
        assert_ne!(BLEND_PANELS[0], BLEND_PANELS[1]);
        assert!(fixtures().into_iter().any(|fixture| {
            fixture.id() == FixtureId::ContinuousGradient
                && fixture.width() == 256
                && fixture.alpha_distribution().len() == 256
        }));
    }

    #[test]
    fn invalid_depth_state_is_rejected_before_the_valid_scene_is_constructed() {
        assert!(require_invalid_pipeline_state_rejection().is_ok());
    }

    #[test]
    fn fixture_restores_positive_depth_pressure_after_provider_conversion() {
        let camera = Camera::orthographic_2d_with_height(960.0, 600.0, 2.0);
        for depth in [
            BLEND_NEAR_DEPTH,
            BLEND_FAR_DEPTH,
            BLEND_BACKGROUND_DEPTH,
            BLEND_REFERENCE_DEPTH,
        ] {
            let clip_depth = camera
                .projection
                .project_point3(tokimu::math::Vec3::new(0.0, 0.0, depth))
                .z;
            assert!(
                (-1.0..=1.0).contains(&clip_depth),
                "world z {depth} escaped Tokimu's GL clip interval at {clip_depth}"
            );
        }
    }
}
