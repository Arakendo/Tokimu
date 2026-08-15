//! Native presentation control for declared moving-platform floor snapshots.
//!
//! The green and yellow walls are independently lowered from one immutable
//! source map plus caller-declared floor facts. They are displayed side by side
//! only to make the changed source span legible; this fixture owns no platform
//! activation, timing, or movement policy.

use std::sync::Arc;

use doom_geometry_provider::{
    lower_doom_seg_textured_wall_triangles, DoomSectorRuntimeHeightSnapshot, DoomTextureExtent,
};
use hello_doom_visibility_conformance::moving_platform_snapshot_fixture;
use tokimu::{
    run_window_with_app, BlendMode, Color, ColorWriteMask, CullMode, DepthTest, DrawMeshCommand,
    FrameOutcome, Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PipelineRenderState, PlatformEventHandler, PlatformInputEvent,
    PlatformResult, RenderCommand, Renderer, WgpuBackend, WindowConfig,
};

const BACKGROUND_MESH: MeshHandle = MeshHandle(1);
const LOW_MESH: MeshHandle = MeshHandle(2);
const RAISED_MESH: MeshHandle = MeshHandle(3);
const BACKGROUND_MATERIAL: MaterialHandle = MaterialHandle(1);
const LOW_MATERIAL: MaterialHandle = MaterialHandle(2);
const RAISED_MATERIAL: MaterialHandle = MaterialHandle(3);

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Doom platform snapshots | loading".into(),
            width: 960,
            height: 600,
        },
        PlatformSnapshotPresentation::default(),
    )
}

#[derive(Default)]
struct PlatformSnapshotPresentation {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    size: [u32; 2],
    background_pipeline: PipelineHandle,
    source_pipeline: PipelineHandle,
    presented_frames: u8,
}

impl PlatformSnapshotPresentation {
    fn upload_fixture(&self, renderer: &mut WgpuBackend) -> PlatformResult<()> {
        let fixture = moving_platform_snapshot_fixture()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let source_sector = fixture.map.sectors[0].source;
        let low = fixture
            .with_runtime_height_snapshots(&[DoomSectorRuntimeHeightSnapshot {
                source_sector,
                floor_height: Some(0),
                ceiling_height: None,
            }])
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let raised = fixture
            .with_runtime_height_snapshots(&[DoomSectorRuntimeHeightSnapshot {
                source_sector,
                floor_height: Some(48),
                ceiling_height: None,
            }])
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let extents = [DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 64,
            height: 64,
        }];
        let low_triangles = lower_doom_seg_textured_wall_triangles(&low.map, &extents)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let raised_triangles = lower_doom_seg_textured_wall_triangles(&raised.map, &extents)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let low_mesh = mesh_from_triangles(low_triangles.iter().map(|triangle| triangle.positions));
        let raised_mesh =
            mesh_from_triangles(raised_triangles.iter().map(|triangle| triangle.positions));
        if low_mesh.vertex_count() == 0 || raised_mesh.vertex_count() == 0 {
            return Err(
                std::io::Error::other("platform snapshot lowering produced no wall mesh").into(),
            );
        }
        let low_minimum = low_triangles
            .iter()
            .flat_map(|triangle| triangle.positions)
            .map(|position| position[1] as i16)
            .min()
            .expect("non-empty lowerer result");
        let raised_minimum = raised_triangles
            .iter()
            .flat_map(|triangle| triangle.positions)
            .map(|position| position[1] as i16)
            .min()
            .expect("non-empty lowerer result");
        if (low_minimum, raised_minimum) != (0, 48) {
            return Err(std::io::Error::other(format!(
                "platform snapshots lost declared floors: low={low_minimum}; raised={raised_minimum}"
            ))
            .into());
        }

        renderer.upload_mesh(BACKGROUND_MESH, &Mesh::quad());
        renderer.upload_mesh(LOW_MESH, &low_mesh);
        renderer.upload_mesh(RAISED_MESH, &raised_mesh);
        renderer.upload_material(
            BACKGROUND_MATERIAL,
            &Material::new("platform-background", Color::rgb(0.04, 0.10, 0.18)),
        )?;
        renderer.upload_material(
            LOW_MATERIAL,
            &Material::new("floor-snapshot-0", Color::rgb(0.20, 0.78, 0.38)),
        )?;
        renderer.upload_material(
            RAISED_MATERIAL,
            &Material::new("floor-snapshot-48", Color::rgb(0.96, 0.62, 0.14)),
        )?;
        eprintln!(
            "platform native control: source=moving-platform-snapshot; low_floor={low_minimum}; raised_floor={raised_minimum}; low_triangles={}; raised_triangles={}; state=declared-snapshots-only; order=background>floor-0>floor-48",
            low_triangles.len(), raised_triangles.len(),
        );
        Ok(())
    }
}

impl PlatformEventHandler for PlatformSnapshotPresentation {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1), size.height.max(1)];
        let mut renderer = WgpuBackend::for_window(window.clone(), self.size[0], self.size[1])?;
        self.background_pipeline = renderer.register_pipeline(
            &Pipeline::new("platform-background", PipelineKind::SolidColor2d).with_render_state(
                PipelineRenderState {
                    blend: BlendMode::Opaque,
                    depth_test: DepthTest::LessEqual,
                    depth_write: false,
                    cull_mode: CullMode::None,
                    color_write: ColorWriteMask::ALL,
                },
            )?,
        )?;
        self.source_pipeline = renderer.register_pipeline(
            &Pipeline::new("platform-source", PipelineKind::SolidColor2d).with_render_state(
                PipelineRenderState {
                    blend: BlendMode::Opaque,
                    depth_test: DepthTest::LessEqual,
                    depth_write: true,
                    cull_mode: CullMode::None,
                    color_write: ColorWriteMask::ALL,
                },
            )?,
        )?;
        self.upload_fixture(&mut renderer)?;
        window.set_title(&format!(
            "Tokimu Doom platform snapshots | adapter={} | floor 0 / floor 48",
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
                LOW_MESH,
                LOW_MATERIAL,
                self.source_pipeline,
                Instance2d::identity().with_translation([-0.45, -0.10]),
            ),
            draw(
                RAISED_MESH,
                RAISED_MATERIAL,
                self.source_pipeline,
                Instance2d::identity().with_translation([0.45, -0.10]),
            ),
        ]);
        let stats = renderer.present()?;
        if let Some(record) = renderer.drain_diagnostics().first() {
            return Err(std::io::Error::other(format!(
                "platform backend diagnostic: category={:?}; source={}; message={}",
                record.kind, record.source, record.message
            ))
            .into());
        }
        if self.presented_frames == 0 {
            if let Some(window) = self.window.as_ref() {
                window.set_title(&format!("Tokimu Doom platform snapshots | draws={} materials={} pipelines={} diagnostic=none", stats.frame.draw_calls, stats.frame.material_resolutions, stats.frame.pipeline_switches));
            }
            eprintln!(
                "platform first frame: draws={}; materials={}; pipelines={}; diagnostic=none",
                stats.frame.draw_calls,
                stats.frame.material_resolutions,
                stats.frame.pipeline_switches
            );
        } else if self.presented_frames == 1 {
            if stats.frame.mesh_uploads != 0 || stats.frame.mesh_replacements != 0 {
                return Err(std::io::Error::other(
                    "platform warm frame mutated static snapshot meshes",
                )
                .into());
            }
            eprintln!("platform warm frame: draws={}; materials={}; pipelines={}; mesh_uploads={}; mesh_replacements={}; lifetime_mesh_uploads={}; lifetime_mesh_replacements={}; diagnostic=none", stats.frame.draw_calls, stats.frame.material_resolutions, stats.frame.pipeline_switches, stats.frame.mesh_uploads, stats.frame.mesh_replacements, stats.lifetime.mesh_uploads, stats.lifetime.mesh_replacements);
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

fn mesh_from_triangles(triangles: impl IntoIterator<Item = [[f64; 3]; 3]>) -> Mesh {
    let positions = triangles
        .into_iter()
        .flatten()
        .map(|position| {
            [
                position[0] as f32 / 112.0,
                position[1] as f32 / 96.0 - 0.67,
                -0.60,
            ]
        })
        .collect::<Vec<_>>();
    Mesh::uniform_normal(positions, [0.0, 0.0, -1.0])
}
