//! Native control for the Doom-local cutout/non-occluder distinction.
//!
//! A caller-declared categorical cutout is deliberately rendered in front of
//! a far opaque surface.  The far surface remains visible through transparent
//! texels.  This proves only the local raster result: alpha coverage is not
//! permission for Doom presentation preparation to treat a masked middle as an
//! opaque source-visibility blocker.

use std::{io, sync::Arc};

use tokimu::{
    run_window_with_app, BlendMode, Camera, CameraHandle, CategoricalCutout, ClearCommand, Color,
    CullMode, CutoutComparison, CutoutThreshold, DepthTest, DrawMeshCommand, FrameOutcome,
    Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline, PipelineHandle,
    PipelineKind, PipelineRenderState, PlatformEventHandler, PlatformInputEvent, PlatformResult,
    RenderCommand, Renderer, Rgba8TextureColorSpace, Rgba8TextureDescriptor, TextureHandle,
    WgpuBackend, WindowConfig,
};

const CAMERA: CameraHandle = CameraHandle(1);
const BACKGROUND_MESH: MeshHandle = MeshHandle(1);
const FAR_WALL_MESH: MeshHandle = MeshHandle(2);
const CUTOUT_MESH: MeshHandle = MeshHandle(3);
const BACKGROUND_MATERIAL: MaterialHandle = MaterialHandle(1);
const FAR_WALL_MATERIAL: MaterialHandle = MaterialHandle(2);
const CUTOUT_MATERIAL: MaterialHandle = MaterialHandle(3);
const CUTOUT_TEXTURE: TextureHandle = TextureHandle(1);

#[derive(Default)]
struct App {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    size: [f32; 2],
    background: PipelineHandle,
    opaque: PipelineHandle,
    cutout: PipelineHandle,
    frames: u8,
}

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Doom cutout non-occluder | loading".into(),
            width: 960,
            height: 600,
        },
        App::default(),
    )
}

impl PlatformEventHandler for App {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window.clone(), size.width, size.height)?;
        self.background = renderer.register_pipeline(
            &Pipeline::new("doom-cutout-control-background", PipelineKind::Textured3d)
                .with_render_state(PipelineRenderState {
                    blend: BlendMode::Opaque,
                    depth_test: DepthTest::LessEqual,
                    depth_write: false,
                    cull_mode: CullMode::None,
                    color_write: Default::default(),
                })?,
        )?;
        self.opaque = renderer.register_pipeline(
            &Pipeline::new("doom-cutout-control-opaque", PipelineKind::Textured3d)
                .with_render_state(PipelineRenderState {
                    blend: BlendMode::Opaque,
                    depth_test: DepthTest::LessEqual,
                    depth_write: true,
                    cull_mode: CullMode::None,
                    color_write: Default::default(),
                })?,
        )?;
        self.cutout = renderer.register_pipeline(&Pipeline::textured_3d_cutout(
            "doom-cutout-control-categorical",
            CategoricalCutout::new(CutoutThreshold::new(0.5)?, CutoutComparison::DiscardBelow),
        ))?;
        renderer.create_texture_rgba8(
            CUTOUT_TEXTURE,
            Rgba8TextureDescriptor::new(4, 4, Rgba8TextureColorSpace::Srgb),
            &cutout_checker_rgba8(),
        )?;
        renderer.upload_material(
            BACKGROUND_MATERIAL,
            &Material::new("doom-cutout-background", Color::rgb(0.08, 0.24, 0.48)),
        )?;
        renderer.upload_material(
            FAR_WALL_MATERIAL,
            &Material::new("doom-cutout-far-wall", Color::rgb(1.0, 0.35, 0.12)),
        )?;
        renderer.upload_material(
            CUTOUT_MATERIAL,
            &Material::new("doom-cutout-masked-middle", Color::rgb(1.0, 1.0, 1.0))
                .with_texture(CUTOUT_TEXTURE),
        )?;
        // The backing is color-only: it must not establish depth. The far
        // opaque wall establishes depth at the reference plane and the
        // categorical cutout is nearer. Transparent cutout texels therefore
        // preserve the far wall, rather than becoming source-visibility
        // authority themselves.
        renderer.upload_mesh(BACKGROUND_MESH, &quad_at_depth(0.0));
        renderer.upload_mesh(FAR_WALL_MESH, &quad_at_depth(0.0));
        renderer.upload_mesh(CUTOUT_MESH, &quad_at_depth(0.5));
        renderer.upload_camera(
            CAMERA,
            Camera::orthographic_2d_with_height(self.size[0], self.size[1], 2.0),
        );
        window.set_title(&format!(
            "Tokimu Doom cutout non-occluder | backend={} adapter={}",
            renderer.backend_api(),
            renderer.adapter_name(),
        ));
        self.renderer = Some(renderer);
        self.window = Some(window);
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
                color: Color::rgb(0.01, 0.015, 0.02),
            }),
            draw(
                BACKGROUND_MESH,
                BACKGROUND_MATERIAL,
                self.background,
                [0.0, 0.0],
                [1.8, 1.6],
            ),
            draw(
                FAR_WALL_MESH,
                FAR_WALL_MATERIAL,
                self.opaque,
                [0.0, -0.08],
                [0.82, 0.72],
            ),
            draw(
                CUTOUT_MESH,
                CUTOUT_MATERIAL,
                self.cutout,
                [0.0, 0.12],
                [1.15, 0.78],
            ),
        ]);
        let stats = renderer.present()?;
        let diagnostics = renderer.drain_diagnostics();
        if let Some(record) = diagnostics.first() {
            return Err(io::Error::other(format!(
                "cutout non-occluder backend diagnostic: category={:?}; source={}; message={}",
                record.kind, record.source, record.message
            ))
            .into());
        }
        if self.frames == 0 {
            let observation = format!(
                "cutout non-occluder first frame: cutout=declared-threshold-0.5; transparent-texels=far-wall-visible; draws={}; materials={}; pipelines={}; diagnostic=none",
                stats.frame.draw_calls, stats.frame.material_resolutions, stats.frame.pipeline_switches,
            );
            eprintln!("{observation}");
            if let Some(window) = self.window.as_ref() {
                window.set_title(&format!("Tokimu Doom cutout non-occluder | {observation}"));
            }
        } else if self.frames == 1 {
            if stats.frame.mesh_uploads != 0 || stats.frame.mesh_replacements != 0 {
                return Err(io::Error::other(format!(
                    "cutout non-occluder warm frame mutated static meshes: uploads={}; replacements={}",
                    stats.frame.mesh_uploads, stats.frame.mesh_replacements
                ))
                .into());
            }
            eprintln!(
                "cutout non-occluder warm frame: draws={}; materials={}; pipelines={}; mesh_uploads=0; mesh_replacements=0; diagnostic=none",
                stats.frame.draw_calls, stats.frame.material_resolutions, stats.frame.pipeline_switches,
            );
        }
        self.frames = self.frames.saturating_add(1);
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
        instance: Instance2d::identity()
            .with_translation(translation)
            .with_scale(scale),
        camera: Some(CAMERA),
        viewport: None,
    })
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

fn cutout_checker_rgba8() -> Vec<u8> {
    let mut rgba8 = Vec::with_capacity(4 * 4 * 4);
    for row in 0..4 {
        for column in 0..4 {
            let opaque = (row + column) % 2 == 0;
            rgba8.extend_from_slice(&[0x35, 0xd9, 0x78, if opaque { 0xff } else { 0x00 }]);
        }
    }
    rgba8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cutout_control_has_both_opaque_and_transparent_texels() {
        let alpha = cutout_checker_rgba8()
            .chunks_exact(4)
            .map(|pixel| pixel[3])
            .collect::<Vec<_>>();
        assert!(alpha.contains(&0));
        assert!(alpha.contains(&255));
        assert!(quad_at_depth(-0.2).has_texture_coordinates());
    }
}
