//! Native Slice 2 visual comparison for AR-0023.
//!
//! The cutout profiles exercise ADR-0013's admitted renderer capability. The
//! fixture still owns policy selection and does not infer it from source bytes.

use std::{io, sync::Arc};

use hello_alpha_policy::{
    fixtures, FixtureId, INTERIOR_THRESHOLD, VIEWPORT, VISUAL_BACKGROUND_DEPTH, VISUAL_DEPTH_SCALE,
    VISUAL_DEPTH_TRANSLATION, VISUAL_FOREGROUND_DEPTH, VISUAL_PROFILE_SCALE,
    VISUAL_PROFILE_TRANSLATIONS,
};
use tokimu::{
    run_window_with_app, BlendMode, Camera, CameraHandle, CategoricalCutout, ClearCommand, Color,
    CullMode, CutoutComparison, CutoutThreshold, DepthTest, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle,
    PipelineKind, PipelineRenderState, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, Rgba8TextureColorSpace, Rgba8TextureDescriptor, TextureHandle,
    WgpuBackend, WindowConfig,
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
    threshold: f32,
    opaque_pipeline: PipelineHandle,
    cutout_below_pipeline: PipelineHandle,
    cutout_at_or_below_pipeline: PipelineHandle,
    meshes: [(MeshHandle, Mesh); 5],
}

impl App {
    fn new(threshold: f32) -> Self {
        Self {
            renderer: None,
            window: None,
            size: [VIEWPORT[0] as f32, VIEWPORT[1] as f32],
            threshold,
            opaque_pipeline: PipelineHandle(0),
            cutout_below_pipeline: PipelineHandle(0),
            cutout_at_or_below_pipeline: PipelineHandle(0),
            meshes: [
                (OPAQUE_MESH, quad_at_depth(VISUAL_FOREGROUND_DEPTH)),
                (CUTOUT_BELOW_MESH, quad_at_depth(VISUAL_FOREGROUND_DEPTH)),
                (
                    CUTOUT_AT_OR_BELOW_MESH,
                    quad_at_depth(VISUAL_FOREGROUND_DEPTH),
                ),
                (
                    DEPTH_BACKGROUND_MESH,
                    quad_at_depth(VISUAL_BACKGROUND_DEPTH),
                ),
                (DEPTH_CUTOUT_MESH, quad_at_depth(VISUAL_FOREGROUND_DEPTH)),
            ],
        }
    }
}

fn main() -> PlatformResult<()> {
    let threshold = selected_threshold()?;
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Alpha Policy | loading comparative cutout scene".into(),
            width: VIEWPORT[0],
            height: VIEWPORT[1],
        },
        App::new(threshold),
    )
}

fn selected_threshold() -> PlatformResult<f32> {
    let mut threshold = INTERIOR_THRESHOLD;
    for argument in std::env::args().skip(1) {
        threshold = match argument.as_str() {
            "--threshold=0" => 0.0,
            "--threshold=interior" => INTERIOR_THRESHOLD,
            "--threshold=1" => 1.0,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "expected --threshold=0, --threshold=interior, or --threshold=1",
                )
                .into())
            }
        };
    }
    Ok(threshold)
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
        self.cutout_below_pipeline = renderer.register_pipeline(&Pipeline::textured_3d_cutout(
            "alpha-study-cutout-below",
            CategoricalCutout::new(
                CutoutThreshold::new(self.threshold)?,
                CutoutComparison::DiscardBelow,
            ),
        ))?;
        self.cutout_at_or_below_pipeline =
            renderer.register_pipeline(&Pipeline::textured_3d_cutout(
                "alpha-study-cutout-at-or-below",
                CategoricalCutout::new(
                    CutoutThreshold::new(self.threshold)?,
                    CutoutComparison::DiscardAtOrBelow,
                ),
            ))?;

        window.set_title(&format!(
            "Tokimu Alpha Policy | opaque | cutout < / <= {:.7} | depth write | backend={} device={} adapter={}",
            self.threshold,
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
                VISUAL_PROFILE_TRANSLATIONS[0],
                VISUAL_PROFILE_SCALE,
            ),
            draw(
                CUTOUT_BELOW_MESH,
                MIXED_MATERIAL,
                self.cutout_below_pipeline,
                VISUAL_PROFILE_TRANSLATIONS[1],
                VISUAL_PROFILE_SCALE,
            ),
            draw(
                CUTOUT_AT_OR_BELOW_MESH,
                MIXED_MATERIAL,
                self.cutout_at_or_below_pipeline,
                VISUAL_PROFILE_TRANSLATIONS[2],
                VISUAL_PROFILE_SCALE,
            ),
            draw(
                DEPTH_BACKGROUND_MESH,
                BACKGROUND_MATERIAL,
                self.opaque_pipeline,
                VISUAL_DEPTH_TRANSLATION,
                VISUAL_DEPTH_SCALE,
            ),
            draw(
                DEPTH_CUTOUT_MESH,
                BINARY_MATERIAL,
                self.cutout_below_pipeline,
                VISUAL_DEPTH_TRANSLATION,
                VISUAL_DEPTH_SCALE,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_scene_uses_the_shared_threshold_as_explicit_cutout_input() {
        let threshold = CutoutThreshold::new(INTERIOR_THRESHOLD).unwrap();
        let pipeline = Pipeline::textured_3d_cutout(
            "native-scene-cutout",
            CategoricalCutout::new(threshold, CutoutComparison::DiscardBelow),
        );
        assert_eq!(pipeline.categorical_cutout().unwrap().threshold, threshold);
        assert_eq!(pipeline.render_state.blend, BlendMode::Opaque);
        assert!(pipeline.render_state.depth_write);
    }

    #[test]
    fn native_scene_quads_supply_renderer_neutral_uvs() {
        let mesh = quad_at_depth(0.25);
        assert!(mesh.has_texture_coordinates());
        assert!(mesh.positions.iter().all(|position| position[2] == 0.25));
    }

    #[test]
    fn missing_candidate_shader_is_a_typed_pipeline_rejection() {
        let error = Pipeline::custom_wgsl("alpha-study-missing-source", "")
            .validate()
            .expect_err("candidate shaders must not silently fall back");
        assert_eq!(
            error.to_string(),
            "custom WGSL pipeline `alpha-study-missing-source` is missing shader source"
        );
    }
}
