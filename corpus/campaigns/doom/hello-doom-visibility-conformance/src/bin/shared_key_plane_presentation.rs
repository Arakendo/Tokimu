//! Native presentation control for two disjoint Doom floor regions that share
//! the same ordinary source plane key.
//!
//! It deliberately uses the production bounded-subsector surface lowerer.
//! Green and orange are fixture-only identity colors: they make it obvious if
//! a shared key collapses two decoded sectors into one rendered instance.

use std::sync::Arc;

use doom_geometry_provider::{
    lower_doom_subsector_surfaces, resolve_doom_subsector_bsp_paths, DoomSurfacePlane,
};
use hello_doom_visibility_conformance::shared_key_disjoint_plane_fixture;
use tokimu::{
    run_window_with_app, BlendMode, Color, ColorWriteMask, CullMode, DepthTest, DrawMeshCommand,
    FrameOutcome, Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PipelineRenderState, PlatformEventHandler, PlatformInputEvent,
    PlatformResult, RenderCommand, Renderer, WgpuBackend, WindowConfig,
};

const BACKGROUND_MESH: MeshHandle = MeshHandle(1);
const NEAR_MESH: MeshHandle = MeshHandle(2);
const FAR_MESH: MeshHandle = MeshHandle(3);
const BACKGROUND_MATERIAL: MaterialHandle = MaterialHandle(1);
const NEAR_MATERIAL: MaterialHandle = MaterialHandle(2);
const FAR_MATERIAL: MaterialHandle = MaterialHandle(3);

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Doom shared plane key | loading".into(),
            width: 960,
            height: 600,
        },
        SharedKeyPlanePresentation::default(),
    )
}

#[derive(Default)]
struct SharedKeyPlanePresentation {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    size: [u32; 2],
    background_pipeline: PipelineHandle,
    plane_pipeline: PipelineHandle,
    presented_frames: u8,
}

impl SharedKeyPlanePresentation {
    fn upload_fixture(&self, renderer: &mut WgpuBackend) -> PlatformResult<()> {
        let fixture = shared_key_disjoint_plane_fixture()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let paths = resolve_doom_subsector_bsp_paths(&fixture.map)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let surfaces = lower_doom_subsector_surfaces(&fixture.map, &paths)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let near = mesh_from_floor_triangles(
            surfaces
                .iter()
                .filter(|surface| {
                    surface.plane == DoomSurfacePlane::Floor
                        && surface.source_sector.record_index == 0
                })
                .map(|surface| surface.positions),
            -0.40,
        );
        let far = mesh_from_floor_triangles(
            surfaces
                .iter()
                .filter(|surface| {
                    surface.plane == DoomSurfacePlane::Floor
                        && surface.source_sector.record_index == 1
                })
                .map(|surface| surface.positions),
            -0.20,
        );
        if near.vertex_count() == 0 || far.vertex_count() == 0 {
            return Err(std::io::Error::other(format!(
                "shared plane source regions missing: near_vertices={}; far_vertices={}",
                near.vertex_count(),
                far.vertex_count()
            ))
            .into());
        }
        renderer.upload_mesh(BACKGROUND_MESH, &Mesh::quad());
        renderer.upload_mesh(NEAR_MESH, &near);
        renderer.upload_mesh(FAR_MESH, &far);
        renderer.upload_material(
            BACKGROUND_MATERIAL,
            &Material::new("shared-key-background", Color::rgb(0.04, 0.10, 0.18)),
        )?;
        renderer.upload_material(
            NEAR_MATERIAL,
            &Material::new("source-sector-0", Color::rgb(0.12, 0.72, 0.28)),
        )?;
        renderer.upload_material(
            FAR_MATERIAL,
            &Material::new("source-sector-1", Color::rgb(0.94, 0.44, 0.12)),
        )?;
        eprintln!(
            "shared-plane-key native control: shared_key=floor/height-0; near_sector=0 triangles={}; far_sector=1 triangles={}; order=background>near-plane>far-plane",
            near.vertex_count() / 3,
            far.vertex_count() / 3,
        );
        Ok(())
    }
}

impl PlatformEventHandler for SharedKeyPlanePresentation {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1), size.height.max(1)];
        let mut renderer = WgpuBackend::for_window(window.clone(), self.size[0], self.size[1])?;
        self.background_pipeline = renderer.register_pipeline(
            &Pipeline::new("shared-plane-key-background", PipelineKind::SolidColor2d)
                .with_render_state(PipelineRenderState {
                    blend: BlendMode::Opaque,
                    depth_test: DepthTest::LessEqual,
                    depth_write: false,
                    cull_mode: CullMode::None,
                    color_write: ColorWriteMask::ALL,
                })?,
        )?;
        self.plane_pipeline = renderer.register_pipeline(
            &Pipeline::new("shared-plane-key-source-plane", PipelineKind::SolidColor2d)
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
            "Tokimu Doom shared plane key | adapter={} | separate source instances",
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
                NEAR_MESH,
                NEAR_MATERIAL,
                self.plane_pipeline,
                Instance2d::identity(),
            ),
            draw(
                FAR_MESH,
                FAR_MATERIAL,
                self.plane_pipeline,
                Instance2d::identity(),
            ),
        ]);
        let stats = renderer.present()?;
        if let Some(record) = renderer.drain_diagnostics().first() {
            return Err(std::io::Error::other(format!(
                "shared-plane-key backend diagnostic: category={:?}; source={}; message={}",
                record.kind, record.source, record.message
            ))
            .into());
        }
        if self.presented_frames == 0 {
            if let Some(window) = self.window.as_ref() {
                window.set_title(&format!("Tokimu Doom shared plane key | draws={} materials={} pipelines={} diagnostic=none", stats.frame.draw_calls, stats.frame.material_resolutions, stats.frame.pipeline_switches));
            }
            eprintln!("shared-plane-key first frame: draws={}; materials={}; pipelines={}; diagnostic=none", stats.frame.draw_calls, stats.frame.material_resolutions, stats.frame.pipeline_switches);
        } else if self.presented_frames == 1 {
            if stats.frame.mesh_uploads != 0 || stats.frame.mesh_replacements != 0 {
                return Err(std::io::Error::other(
                    "shared-plane-key warm frame mutated static meshes",
                )
                .into());
            }
            eprintln!("shared-plane-key warm frame: draws={}; materials={}; pipelines={}; mesh_uploads={}; mesh_replacements={}; lifetime_mesh_uploads={}; lifetime_mesh_replacements={}; diagnostic=none", stats.frame.draw_calls, stats.frame.material_resolutions, stats.frame.pipeline_switches, stats.frame.mesh_uploads, stats.frame.mesh_replacements, stats.lifetime.mesh_uploads, stats.lifetime.mesh_replacements);
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

fn mesh_from_floor_triangles(
    triangles: impl IntoIterator<Item = [[f64; 3]; 3]>,
    clip_depth: f32,
) -> Mesh {
    let positions = triangles
        .into_iter()
        .flatten()
        .map(|position| {
            [
                position[0] as f32 / 112.0,
                position[2] as f32 / 144.0 - 0.70,
                clip_depth,
            ]
        })
        .collect();
    Mesh::uniform_normal(positions, [0.0, 0.0, -1.0])
}
