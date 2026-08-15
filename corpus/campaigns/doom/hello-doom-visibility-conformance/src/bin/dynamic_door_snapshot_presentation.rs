//! Slice-5 native presentation control for declared dynamic doorway heights.
//!
//! The left green region is lowered from the immutable source map plus a
//! caller-declared closed ceiling snapshot. The right orange region is a far
//! presentation control behind the same source opening after a caller-declared
//! open snapshot removes the source wall band. This fixture intentionally owns
//! no activation, timing, waiting, or reversal policy.

use std::sync::Arc;

use doom_geometry_provider::{lower_doom_two_sided_wall_bands, DoomSectorRuntimeHeightSnapshot};
use hello_doom_visibility_conformance::dynamic_door_snapshot_fixture;
use tokimu::{
    run_window_with_app, BlendMode, Color, ColorWriteMask, CullMode, DepthTest, DrawMeshCommand,
    FrameOutcome, Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PipelineRenderState, PlatformEventHandler, PlatformInputEvent,
    PlatformResult, RenderCommand, Renderer, WgpuBackend, WindowConfig,
};

const BACKGROUND_MESH: MeshHandle = MeshHandle(1);
const CLOSED_BAND_MESH: MeshHandle = MeshHandle(2);
const OPENING_CONTROL_MESH: MeshHandle = MeshHandle(3);
const BACKGROUND_MATERIAL: MaterialHandle = MaterialHandle(1);
const CLOSED_BAND_MATERIAL: MaterialHandle = MaterialHandle(2);
const OPENING_CONTROL_MATERIAL: MaterialHandle = MaterialHandle(3);

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Doom dynamic door snapshot | loading".into(),
            width: 960,
            height: 600,
        },
        DynamicDoorSnapshotPresentation::default(),
    )
}

#[derive(Default)]
struct DynamicDoorSnapshotPresentation {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    size: [u32; 2],
    background_pipeline: PipelineHandle,
    source_pipeline: PipelineHandle,
    presented_frames: u8,
}

impl DynamicDoorSnapshotPresentation {
    fn upload_fixture(&self, renderer: &mut WgpuBackend) -> PlatformResult<()> {
        let fixture = dynamic_door_snapshot_fixture()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let dynamic_sector = fixture.map.sectors[1].source;
        let closed = fixture
            .with_runtime_height_snapshots(&[DoomSectorRuntimeHeightSnapshot {
                source_sector: dynamic_sector,
                floor_height: None,
                ceiling_height: Some(0),
            }])
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let open = fixture
            .with_runtime_height_snapshots(&[DoomSectorRuntimeHeightSnapshot {
                source_sector: dynamic_sector,
                floor_height: None,
                ceiling_height: Some(128),
            }])
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let closed_bands = lower_doom_two_sided_wall_bands(&closed.map)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let open_bands = lower_doom_two_sided_wall_bands(&open.map)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        if closed_bands.is_empty() || !open_bands.is_empty() {
            return Err(std::io::Error::other(format!(
                "dynamic-door snapshot lowering unexpected: closed_triangles={}; open_triangles={}",
                closed_bands.len(),
                open_bands.len()
            ))
            .into());
        }

        renderer.upload_mesh(BACKGROUND_MESH, &Mesh::quad());
        let closed_mesh =
            mesh_from_triangles(closed_bands.iter().map(|triangle| triangle.positions));
        if closed_mesh.vertex_count() == 0 {
            return Err(std::io::Error::other(
                "dynamic-door closed snapshot produced no mesh vertices",
            )
            .into());
        }
        renderer.upload_mesh(CLOSED_BAND_MESH, &closed_mesh);
        renderer.upload_mesh(OPENING_CONTROL_MESH, &Mesh::quad());
        renderer.upload_material(
            BACKGROUND_MATERIAL,
            &Material::new("dynamic-door-background", Color::rgb(0.04, 0.10, 0.18)),
        )?;
        renderer.upload_material(
            CLOSED_BAND_MATERIAL,
            &Material::new("closed-source-height-band", Color::rgb(0.20, 0.78, 0.38)),
        )?;
        renderer.upload_material(
            OPENING_CONTROL_MATERIAL,
            &Material::new("open-aperture-far-control", Color::rgb(0.96, 0.40, 0.18)),
        )?;
        eprintln!(
            "dynamic-door native control: source=dynamic-door-snapshot; closed_triangles={}; closed_vertices={}; open_triangles={}; source_horizontal_axis=z; state=declared-snapshots-only; order=background>closed-band>open-aperture-control",
            closed_bands.len(), closed_mesh.vertex_count(), open_bands.len()
        );
        Ok(())
    }
}

impl PlatformEventHandler for DynamicDoorSnapshotPresentation {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1), size.height.max(1)];
        let mut renderer = WgpuBackend::for_window(window.clone(), self.size[0], self.size[1])?;
        self.background_pipeline = renderer.register_pipeline(
            &Pipeline::new("dynamic-door-background", PipelineKind::SolidColor2d)
                .with_render_state(PipelineRenderState {
                    blend: BlendMode::Opaque,
                    depth_test: DepthTest::LessEqual,
                    depth_write: false,
                    cull_mode: CullMode::None,
                    color_write: ColorWriteMask::ALL,
                })?,
        )?;
        self.source_pipeline = renderer.register_pipeline(
            &Pipeline::new("dynamic-door-source", PipelineKind::SolidColor2d).with_render_state(
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
            "Tokimu Doom dynamic door snapshots | adapter={} | closed band / open aperture",
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
                CLOSED_BAND_MESH,
                CLOSED_BAND_MATERIAL,
                self.source_pipeline,
                Instance2d::identity().with_translation([-0.50, -0.10]),
            ),
            draw(
                OPENING_CONTROL_MESH,
                OPENING_CONTROL_MATERIAL,
                self.source_pipeline,
                Instance2d::identity()
                    .with_translation([0.50, -0.10])
                    .with_scale([0.35, 0.62]),
            ),
        ]);
        let stats = renderer.present()?;
        if let Some(record) = renderer.drain_diagnostics().first() {
            return Err(std::io::Error::other(format!(
                "dynamic-door backend diagnostic: category={:?}; source={}; message={}",
                record.kind, record.source, record.message
            ))
            .into());
        }
        if self.presented_frames == 0 {
            if let Some(window) = self.window.as_ref() {
                window.set_title(&format!(
                    "Tokimu Doom dynamic door snapshots | draws={} materials={} pipelines={} diagnostic=none",
                    stats.frame.draw_calls,
                    stats.frame.material_resolutions,
                    stats.frame.pipeline_switches
                ));
            }
            eprintln!(
                "dynamic-door first frame: draws={}; materials={}; pipelines={}; diagnostic=none",
                stats.frame.draw_calls,
                stats.frame.material_resolutions,
                stats.frame.pipeline_switches
            );
        } else if self.presented_frames == 1 {
            if stats.frame.mesh_uploads != 0 || stats.frame.mesh_replacements != 0 {
                return Err(std::io::Error::other(
                    "dynamic-door warm frame mutated static snapshot meshes",
                )
                .into());
            }
            eprintln!(
                "dynamic-door warm frame: draws={}; materials={}; pipelines={}; mesh_uploads={}; mesh_replacements={}; lifetime_mesh_uploads={}; lifetime_mesh_replacements={}; diagnostic=none",
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

fn mesh_from_triangles(triangles: impl IntoIterator<Item = [[f64; 3]; 3]>) -> Mesh {
    let positions = triangles
        .into_iter()
        .flatten()
        // This fixture's shared doorway runs along source Z. Present that
        // varying axis horizontally rather than drawing its constant source X
        // coordinate edge-on.
        .map(|position| {
            [
                position[2] as f32 / 96.0,
                position[1] as f32 / 96.0 - 0.67,
                -0.60,
            ]
        })
        .collect::<Vec<_>>();
    Mesh::uniform_normal(positions, [0.0, 0.0, -1.0])
}
