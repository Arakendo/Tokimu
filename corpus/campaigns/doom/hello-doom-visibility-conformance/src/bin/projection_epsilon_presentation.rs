//! Slice-5 native diagnostic presentation for source-projection edge cases.
//!
//! This control deliberately magnifies each source wall horizontally so a
//! near-plane fail-open, a one-unit-wide valid SEG, and an extremely-close
//! valid SEG can be inspected side by side. The magnification is presentation
//! only: admission continues to come from the shared Doom source observation.

use std::sync::Arc;

use doom_geometry_provider::{lower_doom_seg_textured_wall_triangles, DoomTextureExtent};
use hello_doom_visibility_conformance::{
    projection_close_forward_seg_fixture, projection_near_plane_crossing_fixture,
    projection_thin_forward_seg_fixture, DoomVisibilityFixture,
};
use tokimu::{
    run_window_with_app, BlendMode, Color, ColorWriteMask, CullMode, DepthTest, DrawMeshCommand,
    FrameOutcome, Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PipelineRenderState, PlatformEventHandler, PlatformInputEvent,
    PlatformResult, RenderCommand, Renderer, WgpuBackend, WindowConfig,
};

const BACKGROUND_MESH: MeshHandle = MeshHandle(1);
const THIN_MESH: MeshHandle = MeshHandle(2);
const CLOSE_MESH: MeshHandle = MeshHandle(3);
const BACKGROUND_MATERIAL: MaterialHandle = MaterialHandle(1);
const THIN_MATERIAL: MaterialHandle = MaterialHandle(2);
const CLOSE_MATERIAL: MaterialHandle = MaterialHandle(3);

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Doom projection epsilon | loading".into(),
            width: 960,
            height: 600,
        },
        ProjectionEpsilonPresentation::default(),
    )
}

#[derive(Default)]
struct ProjectionEpsilonPresentation {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    size: [u32; 2],
    background_pipeline: PipelineHandle,
    source_pipeline: PipelineHandle,
    presented_frames: u8,
}

impl ProjectionEpsilonPresentation {
    fn upload_fixture(&self, renderer: &mut WgpuBackend) -> PlatformResult<()> {
        let near = projection_near_plane_crossing_fixture()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let thin = projection_thin_forward_seg_fixture()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let close = projection_close_forward_seg_fixture()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let near_observation = near
            .observe_classic_bsp()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let thin_observation = thin
            .observe_classic_bsp()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let close_observation = close
            .observe_classic_bsp()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        if near_observation.near_plane_fail_open == 0
            || near_observation.solid_admitted != 0
            || thin_observation.solid_admitted != 1
            || close_observation.solid_admitted != 1
        {
            return Err(std::io::Error::other(format!(
                "projection-edge source observations unexpected: near={near_observation:?}; thin={thin_observation:?}; close={close_observation:?}"
            ))
            .into());
        }

        let thin_mesh = source_wall_mesh(&thin, 0.26)?;
        let close_mesh = source_wall_mesh(&close, 0.014)?;
        if thin_mesh.vertex_count() == 0 || close_mesh.vertex_count() == 0 {
            return Err(std::io::Error::other(format!(
                "projection-edge lowerer produced empty valid source mesh: thin_vertices={}; close_vertices={}",
                thin_mesh.vertex_count(),
                close_mesh.vertex_count()
            ))
            .into());
        }

        renderer.upload_mesh(BACKGROUND_MESH, &Mesh::quad());
        renderer.upload_mesh(THIN_MESH, &thin_mesh);
        renderer.upload_mesh(CLOSE_MESH, &close_mesh);
        renderer.upload_material(
            BACKGROUND_MATERIAL,
            &Material::new(
                "projection-epsilon-background",
                Color::rgb(0.04, 0.10, 0.18),
            ),
        )?;
        renderer.upload_material(
            THIN_MATERIAL,
            &Material::new("thin-valid-source-seg", Color::rgb(0.25, 0.84, 0.45)),
        )?;
        renderer.upload_material(
            CLOSE_MATERIAL,
            &Material::new("close-valid-source-seg", Color::rgb(0.98, 0.56, 0.26)),
        )?;
        eprintln!(
            "projection-epsilon native control: near_plane=fail-open:solid_admitted={}; thin=valid:solid_admitted={}:covered_columns={}; close=valid:solid_admitted={}:covered_columns={}; presentation=source-x-magnified-per-control",
            near_observation.solid_admitted,
            thin_observation.solid_admitted,
            thin_observation.solid_range_covered_columns,
            close_observation.solid_admitted,
            close_observation.solid_range_covered_columns,
        );
        Ok(())
    }
}

impl PlatformEventHandler for ProjectionEpsilonPresentation {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1), size.height.max(1)];
        let mut renderer = WgpuBackend::for_window(window.clone(), self.size[0], self.size[1])?;
        self.background_pipeline = renderer.register_pipeline(
            &Pipeline::new("projection-epsilon-background", PipelineKind::SolidColor2d)
                .with_render_state(PipelineRenderState {
                    blend: BlendMode::Opaque,
                    depth_test: DepthTest::LessEqual,
                    depth_write: false,
                    cull_mode: CullMode::None,
                    color_write: ColorWriteMask::ALL,
                })?,
        )?;
        self.source_pipeline = renderer.register_pipeline(
            &Pipeline::new("projection-epsilon-source", PipelineKind::SolidColor2d)
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
            "Tokimu Doom projection epsilon | adapter={} | fail-open / thin valid / close valid",
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
                THIN_MESH,
                THIN_MATERIAL,
                self.source_pipeline,
                Instance2d::identity().with_translation([-0.45, -0.10]),
            ),
            draw(
                CLOSE_MESH,
                CLOSE_MATERIAL,
                self.source_pipeline,
                Instance2d::identity().with_translation([0.45, -0.10]),
            ),
        ]);
        let stats = renderer.present()?;
        if let Some(record) = renderer.drain_diagnostics().first() {
            return Err(std::io::Error::other(format!(
                "projection-epsilon backend diagnostic: category={:?}; source={}; message={}",
                record.kind, record.source, record.message
            ))
            .into());
        }
        if self.presented_frames == 0 {
            if let Some(window) = self.window.as_ref() {
                window.set_title(&format!(
                    "Tokimu Doom projection epsilon | draws={} materials={} pipelines={} diagnostic=none",
                    stats.frame.draw_calls,
                    stats.frame.material_resolutions,
                    stats.frame.pipeline_switches
                ));
            }
            eprintln!(
                "projection-epsilon first frame: draws={}; materials={}; pipelines={}; diagnostic=none",
                stats.frame.draw_calls,
                stats.frame.material_resolutions,
                stats.frame.pipeline_switches
            );
        } else if self.presented_frames == 1 {
            if stats.frame.mesh_uploads != 0 || stats.frame.mesh_replacements != 0 {
                return Err(std::io::Error::other(
                    "projection-epsilon warm frame mutated static source meshes",
                )
                .into());
            }
            eprintln!(
                "projection-epsilon warm frame: draws={}; materials={}; pipelines={}; mesh_uploads={}; mesh_replacements={}; lifetime_mesh_uploads={}; lifetime_mesh_replacements={}; diagnostic=none",
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

fn source_wall_mesh(
    fixture: &DoomVisibilityFixture,
    horizontal_scale: f32,
) -> PlatformResult<Mesh> {
    let walls = lower_doom_seg_textured_wall_triangles(
        &fixture.map,
        &[DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 64,
            height: 128,
        }],
    )
    .map_err(|error| std::io::Error::other(error.to_string()))?;
    let positions = walls
        .into_iter()
        .flat_map(|triangle| triangle.positions)
        .map(|position| {
            [
                position[0] as f32 * horizontal_scale,
                position[1] as f32 / 96.0 - 0.67,
                -0.60,
            ]
        })
        .collect::<Vec<_>>();
    Ok(Mesh::uniform_normal(positions, [0.0, 0.0, -1.0]))
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
