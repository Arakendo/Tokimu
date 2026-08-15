//! Slice-4B native presentation of Doom-owned partial source realization.
//!
//! The two orange draws are clipped from one far source SEG by the Doom
//! fixture's ordered-coverage result. The green control makes the nearer
//! authority visible rather than reviving the falsified invisible sky-depth
//! wall. Diagnostic columns explain the decision but never enter the renderer.

use std::sync::Arc;

use hello_doom_visibility_conformance::realize_partial_coverage_fragments;
use tokimu::{
    math::{Mat4, Vec3},
    run_window_with_app, BlendMode, Camera, CameraHandle, Color, ColorWriteMask, CullMode,
    DepthTest, DrawMeshCommand, FrameOutcome, Instance2d, Material, MaterialHandle, Mesh,
    MeshHandle, NativeWindow, Pipeline, PipelineHandle, PipelineKind, PipelineRenderState,
    PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand, Renderer, WgpuBackend,
    WindowConfig,
};

const BACKGROUND_MESH: MeshHandle = MeshHandle(1);
const NEAR_CONTROL_MESH: MeshHandle = MeshHandle(2);
const LEFT_FRAGMENT_MESH: MeshHandle = MeshHandle(3);
const RIGHT_FRAGMENT_MESH: MeshHandle = MeshHandle(4);
const BACKGROUND_MATERIAL: MaterialHandle = MaterialHandle(1);
const NEAR_CONTROL_MATERIAL: MaterialHandle = MaterialHandle(2);
const FAR_FRAGMENT_MATERIAL: MaterialHandle = MaterialHandle(3);
const CAMERA: CameraHandle = CameraHandle(1);

fn main() -> PlatformResult<()> {
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Doom ordered coverage | loading".into(),
            width: 960,
            height: 600,
        },
        OrderedCoveragePresentation::default(),
    )
}

#[derive(Default)]
struct OrderedCoveragePresentation {
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    size: [u32; 2],
    background_pipeline: PipelineHandle,
    source_pipeline: PipelineHandle,
    presented_frames: u8,
}

impl OrderedCoveragePresentation {
    fn upload_fixture(&self, renderer: &mut WgpuBackend) -> PlatformResult<()> {
        let manifest = realize_partial_coverage_fragments()
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let [left, right] = manifest.fragments.as_slice() else {
            return Err(std::io::Error::other(format!(
                "ordered coverage expected two fragments, observed {}",
                manifest.fragments.len()
            ))
            .into());
        };

        renderer.upload_mesh(BACKGROUND_MESH, &Mesh::quad());
        renderer.upload_mesh(
            NEAR_CONTROL_MESH,
            &clip_rect_mesh(-0.82, 0.82, -0.48, 0.48, -0.55),
        );
        renderer.upload_mesh(LEFT_FRAGMENT_MESH, &fragment_mesh(left));
        renderer.upload_mesh(RIGHT_FRAGMENT_MESH, &fragment_mesh(right));
        renderer.upload_material(
            BACKGROUND_MATERIAL,
            &Material::new("ordered-coverage-background", Color::rgb(0.04, 0.10, 0.18)),
        )?;
        renderer.upload_material(
            NEAR_CONTROL_MATERIAL,
            &Material::new("visible-near-authority", Color::rgb(0.12, 0.72, 0.28)),
        )?;
        renderer.upload_material(
            FAR_FRAGMENT_MATERIAL,
            &Material::new("retained-far-source-fragment", Color::rgb(0.94, 0.35, 0.12)),
        )?;

        eprintln!(
            "ordered-coverage native control: source-seg={}; excluded=[{:.6},{:.6}]; left-columns={}..{}; left-source=[{:.6},{:.6}]; right-columns={}..{}; right-source=[{:.6},{:.6}]; fragment-triangles={}/{}; order=background>visible-near-authority>left-fragment>right-fragment; meaning=doom-owned-source-fragments-not-renderer-scissors",
            left.source_seg.record_index,
            manifest.excluded_linedef_interval[0],
            manifest.excluded_linedef_interval[1],
            left.diagnostic_columns.first,
            left.diagnostic_columns.last,
            left.linedef_interval[0],
            left.linedef_interval[1],
            right.diagnostic_columns.first,
            right.diagnostic_columns.last,
            right.linedef_interval[0],
            right.linedef_interval[1],
            left.triangles.len(),
            right.triangles.len(),
        );
        Ok(())
    }
}

impl PlatformEventHandler for OrderedCoveragePresentation {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1), size.height.max(1)];
        let mut renderer = WgpuBackend::for_window(window.clone(), self.size[0], self.size[1])?;
        self.background_pipeline = renderer.register_pipeline(
            &Pipeline::new("ordered-coverage-background", PipelineKind::SolidColor2d)
                .with_render_state(PipelineRenderState {
                    blend: BlendMode::Opaque,
                    depth_test: DepthTest::LessEqual,
                    depth_write: false,
                    cull_mode: CullMode::None,
                    color_write: ColorWriteMask::ALL,
                })?,
        )?;
        self.source_pipeline = renderer.register_pipeline(
            &Pipeline::new("ordered-coverage-source", PipelineKind::SolidColor2d)
                .with_render_state(PipelineRenderState {
                    blend: BlendMode::Opaque,
                    depth_test: DepthTest::LessEqual,
                    depth_write: true,
                    cull_mode: CullMode::None,
                    color_write: ColorWriteMask::ALL,
                })?,
        )?;
        self.upload_fixture(&mut renderer)?;
        renderer.upload_camera(CAMERA, Camera::default());
        window.set_title(&format!(
            "Tokimu Doom ordered coverage | adapter={} | near authority + two far fragments",
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
        if self.presented_frames == 2 {
            renderer.upload_camera(
                CAMERA,
                Camera::new(
                    Mat4::from_translation(Vec3::new(-0.08, 0.0, 0.0)),
                    Mat4::IDENTITY,
                ),
            );
        }
        renderer.begin_frame();
        renderer.submit(&[
            draw(
                BACKGROUND_MESH,
                BACKGROUND_MATERIAL,
                self.background_pipeline,
                Instance2d::identity().with_scale([1.8, 1.6]),
            ),
            draw(
                NEAR_CONTROL_MESH,
                NEAR_CONTROL_MATERIAL,
                self.source_pipeline,
                Instance2d::identity(),
            ),
            draw(
                LEFT_FRAGMENT_MESH,
                FAR_FRAGMENT_MATERIAL,
                self.source_pipeline,
                Instance2d::identity(),
            ),
            draw(
                RIGHT_FRAGMENT_MESH,
                FAR_FRAGMENT_MATERIAL,
                self.source_pipeline,
                Instance2d::identity(),
            ),
        ]);
        let stats = renderer.present()?;
        if let Some(record) = renderer.drain_diagnostics().first() {
            return Err(std::io::Error::other(format!(
                "ordered-coverage backend diagnostic: category={:?}; source={}; message={}",
                record.kind, record.source, record.message
            ))
            .into());
        }
        if self.presented_frames == 0 {
            if let Some(window) = self.window.as_ref() {
                window.set_title(&format!(
                    "Tokimu Doom ordered coverage | draws={} materials={} pipelines={} diagnostic=none",
                    stats.frame.draw_calls,
                    stats.frame.material_resolutions,
                    stats.frame.pipeline_switches
                ));
            }
            eprintln!(
                "ordered-coverage first frame: draws={}; materials={}; pipelines={}; diagnostic=none",
                stats.frame.draw_calls,
                stats.frame.material_resolutions,
                stats.frame.pipeline_switches,
            );
        } else if self.presented_frames == 1 || self.presented_frames == 2 {
            if stats.frame.mesh_uploads != 0 || stats.frame.mesh_replacements != 0 {
                return Err(std::io::Error::other(format!(
                    "ordered-coverage static frame mutated meshes: frame={}; uploads={}; replacements={}",
                    self.presented_frames + 1,
                    stats.frame.mesh_uploads,
                    stats.frame.mesh_replacements
                ))
                .into());
            }
            eprintln!(
                "ordered-coverage {} frame: draws={}; materials={}; pipelines={}; mesh_uploads={}; mesh_replacements={}; diagnostic=none",
                if self.presented_frames == 1 { "warm" } else { "camera-jitter" },
                stats.frame.draw_calls,
                stats.frame.material_resolutions,
                stats.frame.pipeline_switches,
                stats.frame.mesh_uploads,
                stats.frame.mesh_replacements,
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
        camera: Some(CAMERA),
        viewport: None,
    })
}

fn fragment_mesh(
    fragment: &hello_doom_visibility_conformance::PartialCoverageSourceFragment,
) -> Mesh {
    Mesh::uniform_normal(
        fragment
            .triangles
            .iter()
            .flat_map(|triangle| triangle.positions)
            .map(|position| {
                [
                    position[0] as f32 / 48.0,
                    position[1] as f32 / 80.0 - 0.80,
                    -0.25,
                ]
            })
            .collect(),
        [0.0, 0.0, -1.0],
    )
}

fn clip_rect_mesh(left: f32, right: f32, bottom: f32, top: f32, depth: f32) -> Mesh {
    Mesh::uniform_normal(
        vec![
            [left, bottom, depth],
            [right, bottom, depth],
            [right, top, depth],
            [left, bottom, depth],
            [right, top, depth],
            [left, top, depth],
        ],
        [0.0, 0.0, -1.0],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presentation_uses_two_distinct_source_fragment_meshes() {
        let manifest = realize_partial_coverage_fragments().unwrap();
        assert_eq!(manifest.fragments.len(), 2);
        for fragment in &manifest.fragments {
            assert_eq!(
                fragment_mesh(fragment).vertex_count(),
                u32::try_from(fragment.triangles.len() * 3).unwrap()
            );
            assert!(!fragment.triangles.is_empty());
        }
        assert_eq!(
            manifest.fragments[0].source_seg,
            manifest.fragments[1].source_seg
        );
    }
}
