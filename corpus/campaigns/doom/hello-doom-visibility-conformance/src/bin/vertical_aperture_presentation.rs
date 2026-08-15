//! Slice-5 native presentation control for one two-sided vertical aperture.
//!
//! Green upper and yellow lower tiers come from the production SEG wall
//! lowering path. An orange far surface remains visible only through the real
//! source opening. This is Doom-local presentation evidence, not a renderer
//! visibility contract.

use std::sync::Arc;

use doom_geometry_provider::{
    lower_doom_seg_textured_wall_triangles, DoomTextureExtent, DoomWallTextureRole,
};
use hello_doom_visibility_conformance::vertical_aperture_control_fixture;
use tokimu::{
    run_window_with_app, BlendMode, Color, ColorWriteMask, CullMode, DepthTest, DrawMeshCommand,
    FrameOutcome, Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PipelineRenderState, PlatformEventHandler, PlatformInputEvent,
    PlatformResult, RenderCommand, Renderer, WgpuBackend, WindowConfig,
};

const BACKGROUND_MESH: MeshHandle = MeshHandle(1);
const FAR_MESH: MeshHandle = MeshHandle(2);
const UPPER_MESH: MeshHandle = MeshHandle(3);
const LOWER_MESH: MeshHandle = MeshHandle(4);
const BACKGROUND_MATERIAL: MaterialHandle = MaterialHandle(1);
const FAR_MATERIAL: MaterialHandle = MaterialHandle(2);
const UPPER_MATERIAL: MaterialHandle = MaterialHandle(3);
const LOWER_MATERIAL: MaterialHandle = MaterialHandle(4);

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Doom vertical aperture | loading".into(),
            width: 960,
            height: 600,
        },
        VerticalAperturePresentation::default(),
    )
}

#[derive(Default)]
struct VerticalAperturePresentation {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    size: [u32; 2],
    background_pipeline: PipelineHandle,
    wall_pipeline: PipelineHandle,
    presented_frames: u8,
}

impl VerticalAperturePresentation {
    fn upload_fixture(&self, renderer: &mut WgpuBackend) -> PlatformResult<()> {
        let fixture = vertical_aperture_control_fixture()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let near_seg = fixture.map.segs[0].source;
        let walls = lower_doom_seg_textured_wall_triangles(
            &fixture.map,
            &[DoomTextureExtent {
                name: "WALL".to_owned(),
                width: 64,
                height: 128,
            }],
        )
        .map_err(|error| std::io::Error::other(error.to_string()))?;
        let upper = mesh_from_triangles(
            walls
                .iter()
                .filter(|triangle| {
                    triangle.source_seg == near_seg && triangle.role == DoomWallTextureRole::Upper
                })
                .map(|triangle| triangle.positions),
            -0.60,
        );
        let lower = mesh_from_triangles(
            walls
                .iter()
                .filter(|triangle| {
                    triangle.source_seg == near_seg && triangle.role == DoomWallTextureRole::Lower
                })
                .map(|triangle| triangle.positions),
            -0.60,
        );
        if upper.vertex_count() == 0 || lower.vertex_count() == 0 {
            return Err(std::io::Error::other(format!(
                "vertical-aperture source tiers missing: upper_vertices={}; lower_vertices={}",
                upper.vertex_count(),
                lower.vertex_count()
            ))
            .into());
        }

        renderer.upload_mesh(BACKGROUND_MESH, &Mesh::quad());
        renderer.upload_mesh(FAR_MESH, &Mesh::quad());
        renderer.upload_mesh(UPPER_MESH, &upper);
        renderer.upload_mesh(LOWER_MESH, &lower);
        renderer.upload_material(
            BACKGROUND_MATERIAL,
            &Material::new("outside", Color::rgb(0.04, 0.10, 0.18)),
        )?;
        renderer.upload_material(
            FAR_MATERIAL,
            &Material::new("far-opening-control", Color::rgb(0.92, 0.22, 0.12)),
        )?;
        renderer.upload_material(
            UPPER_MATERIAL,
            &Material::new("source-upper-tier", Color::rgb(0.12, 0.72, 0.28)),
        )?;
        renderer.upload_material(
            LOWER_MATERIAL,
            &Material::new("source-lower-tier", Color::rgb(0.94, 0.72, 0.08)),
        )?;
        eprintln!(
            "vertical-aperture native control: upper-triangles={}; lower-triangles={}; opening=24..96; order=background>far-control>upper>lower",
            upper.vertex_count() / 3,
            lower.vertex_count() / 3,
        );
        Ok(())
    }
}

impl PlatformEventHandler for VerticalAperturePresentation {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1), size.height.max(1)];
        let mut renderer = WgpuBackend::for_window(window.clone(), self.size[0], self.size[1])?;
        self.background_pipeline = renderer.register_pipeline(
            &Pipeline::new("vertical-aperture-background", PipelineKind::SolidColor2d)
                .with_render_state(PipelineRenderState {
                    blend: BlendMode::Opaque,
                    depth_test: DepthTest::LessEqual,
                    depth_write: false,
                    cull_mode: CullMode::None,
                    color_write: ColorWriteMask::ALL,
                })?,
        )?;
        self.wall_pipeline = renderer.register_pipeline(
            &Pipeline::new("vertical-aperture-depth", PipelineKind::SolidColor2d)
                .with_render_state(PipelineRenderState {
                    blend: BlendMode::Opaque,
                    depth_test: DepthTest::LessEqual,
                    depth_write: true,
                    cull_mode: CullMode::None,
                    color_write: ColorWriteMask::ALL,
                })?,
        )?;
        self.upload_fixture(&mut renderer)?;
        window.set_title(&format!(
            "Tokimu Doom vertical aperture | adapter={} | upper / opening / lower",
            renderer.adapter_name()
        ));
        self.renderer = Some(renderer);
        self.window = Some(window);
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::Resized { width, height } = event {
            self.size = [width.max(1), height.max(1)];
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize_surface(self.size[0], self.size[1]);
            }
        }
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };
        renderer.begin_frame();
        renderer.submit(&[
            draw(
                BACKGROUND_MESH,
                BACKGROUND_MATERIAL,
                self.background_pipeline,
                Instance2d::identity().with_scale([1.8, 1.6]),
            ),
            draw(
                FAR_MESH,
                FAR_MATERIAL,
                self.wall_pipeline,
                Instance2d::identity().with_scale([0.76, 1.2]),
            ),
            draw(
                UPPER_MESH,
                UPPER_MATERIAL,
                self.wall_pipeline,
                Instance2d::identity(),
            ),
            draw(
                LOWER_MESH,
                LOWER_MATERIAL,
                self.wall_pipeline,
                Instance2d::identity(),
            ),
        ]);
        let stats = renderer.present()?;
        let diagnostics = renderer.drain_diagnostics();
        if let Some(record) = diagnostics.first() {
            return Err(std::io::Error::other(format!(
                "vertical-aperture backend diagnostic: category={:?}; source={}; message={}",
                record.kind, record.source, record.message
            ))
            .into());
        }
        if self.presented_frames == 0 {
            if let Some(window) = self.window.as_ref() {
                window.set_title(&format!(
                    "Tokimu Doom vertical aperture | draws={} materials={} pipelines={} diagnostic=none",
                    stats.frame.draw_calls,
                    stats.frame.material_resolutions,
                    stats.frame.pipeline_switches
                ));
            }
            eprintln!(
                "vertical-aperture first frame: draws={}; materials={}; pipelines={}; diagnostic=none",
                stats.frame.draw_calls, stats.frame.material_resolutions, stats.frame.pipeline_switches
            );
        } else if self.presented_frames == 1 {
            if stats.frame.mesh_uploads != 0 || stats.frame.mesh_replacements != 0 {
                return Err(std::io::Error::other(
                    "vertical-aperture warm frame mutated static meshes",
                )
                .into());
            }
            eprintln!(
                "vertical-aperture warm frame: draws={}; materials={}; pipelines={}; mesh_uploads={}; mesh_replacements={}; lifetime_mesh_uploads={}; lifetime_mesh_replacements={}; diagnostic=none",
                stats.frame.draw_calls,
                stats.frame.material_resolutions,
                stats.frame.pipeline_switches,
                stats.frame.mesh_uploads,
                stats.frame.mesh_replacements,
                stats.lifetime.mesh_uploads,
                stats.lifetime.mesh_replacements
            );
        }
        self.presented_frames = self.presented_frames.saturating_add(1);
        Ok(FrameOutcome::Continue)
    }
}

fn draw(
    mesh: MeshHandle,
    material: MaterialHandle,
    pipeline: PipelineHandle,
    instance: Instance2d,
) -> RenderCommand {
    RenderCommand::DrawMesh(DrawMeshCommand {
        mesh,
        material,
        pipeline,
        instance,
        camera: None,
        viewport: None,
    })
}

fn mesh_from_triangles(
    triangles: impl IntoIterator<Item = [[f64; 3]; 3]>,
    clip_depth: f32,
) -> Mesh {
    let positions = triangles
        .into_iter()
        .flatten()
        .map(|position| {
            [
                position[0] as f32 / 64.0,
                position[1] as f32 / 100.0 - 0.80,
                clip_depth,
            ]
        })
        .collect::<Vec<_>>();
    Mesh::uniform_normal(positions, [0.0, 0.0, -1.0])
}
