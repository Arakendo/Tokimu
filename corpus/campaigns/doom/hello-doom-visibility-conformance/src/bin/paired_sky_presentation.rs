//! Slice-5 native presentation control for one paired-sky source boundary.
//!
//! This is intentionally a small, fixed diagnostic-display experiment. It does not claim
//! visplane parity or generic occlusion: it demonstrates the explicit
//! Doom-local order `sky colour -> paired-sky depth boundary -> far wall` over
//! the same source fixture used by the Level-1 tests.

use std::sync::Arc;

use doom_geometry_provider::{
    lower_doom_paired_sky_boundary_triangles, lower_doom_seg_textured_wall_triangles,
    DoomTextureExtent,
};
use hello_doom_visibility_conformance::{
    one_sky_far_control_fixture, paired_sky_far_control_fixture,
    single_sky_plane_far_control_fixture, DoomVisibilityFixture,
};
use tokimu::{
    math::{Mat4, Vec3},
    run_window_with_app, BlendMode, Camera, CameraHandle, ClearCommand, Color, ColorWriteMask,
    CullMode, DepthTest, DrawMeshCommand, FrameOutcome, Instance2d, Material, MaterialHandle, Mesh,
    MeshHandle, NativeWindow, Pipeline, PipelineHandle, PipelineKind, PipelineRenderState,
    PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand, Renderer, WgpuBackend,
    WindowConfig,
};

const SKY_MESH: MeshHandle = MeshHandle(1);
const BOUNDARY_MESH: MeshHandle = MeshHandle(2);
const FAR_WALL_MESH: MeshHandle = MeshHandle(3);
const SKY_MATERIAL: MaterialHandle = MaterialHandle(1);
const BOUNDARY_MATERIAL: MaterialHandle = MaterialHandle(2);
const FAR_WALL_MATERIAL: MaterialHandle = MaterialHandle(3);
const CAMERA: CameraHandle = CameraHandle(1);

fn main() -> PlatformResult<()> {
    let mode = if std::env::args().any(|argument| argument == "--single-sky-plane") {
        ControlMode::SingleSkyPlane
    } else if std::env::args().any(|argument| argument == "--one-sky") {
        ControlMode::OneSky
    } else {
        ControlMode::PairedSky
    };
    run_window_with_app(
        WindowConfig {
            title: "Tokimu Doom paired-sky presentation | loading".into(),
            width: 960,
            height: 600,
        },
        PairedSkyPresentation::new(mode),
    )
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum ControlMode {
    #[default]
    PairedSky,
    OneSky,
    SingleSkyPlane,
}

impl ControlMode {
    fn label(self) -> &'static str {
        match self {
            Self::PairedSky => "paired-sky",
            Self::OneSky => "one-sky-negative",
            Self::SingleSkyPlane => "single-sky-plane-coverage",
        }
    }

    fn fixture(self) -> PlatformResult<DoomVisibilityFixture> {
        let fixture = match self {
            Self::PairedSky => paired_sky_far_control_fixture(),
            Self::OneSky => one_sky_far_control_fixture(),
            Self::SingleSkyPlane => single_sky_plane_far_control_fixture(),
        };
        fixture.map_err(|error| std::io::Error::other(error.to_string()).into())
    }
}

#[derive(Default)]
struct PairedSkyPresentation {
    mode: ControlMode,
    renderer: Option<WgpuBackend>,
    window: Option<Arc<NativeWindow>>,
    size: [u32; 2],
    sky_pipeline: PipelineHandle,
    boundary_pipeline: PipelineHandle,
    wall_pipeline: PipelineHandle,
    presented_frames: u8,
}

impl PairedSkyPresentation {
    fn new(mode: ControlMode) -> Self {
        Self {
            mode,
            ..Self::default()
        }
    }

    fn upload_fixture(&mut self, renderer: &mut WgpuBackend) -> PlatformResult<()> {
        let fixture = self.mode.fixture()?;
        let boundaries = lower_doom_paired_sky_boundary_triangles(&fixture.map)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let walls = lower_doom_seg_textured_wall_triangles(
            &fixture.map,
            &[DoomTextureExtent {
                name: "WALL".to_owned(),
                width: 64,
                height: 128,
            }],
        )
        .map_err(|error| std::io::Error::other(error.to_string()))?;
        let near_seg = fixture.map.segs.first().map(|seg| seg.source);
        let far_seg = fixture.map.segs.get(1).map(|seg| seg.source);

        // This fixed diagnostic transform preserves source X/Y relationships
        // while assigning explicit clip-depth layers. It is not Doom world or
        // camera policy: its only job is to make this one ordering legible.
        let control_mesh = match self.mode {
            ControlMode::PairedSky => {
                mesh_from_triangles(boundaries.iter().map(|triangle| triangle.positions), -0.60)
            }
            ControlMode::OneSky => mesh_from_triangles(
                walls
                    .iter()
                    .filter(|triangle| Some(triangle.source_seg) == near_seg)
                    .map(|triangle| triangle.positions),
                -0.60,
            ),
            // This rectangle is deliberately a diagnostic projection of the
            // named source ceiling plane, not an inferred world-space sky
            // enclosure. Its bounded upper interval is the only authority it
            // demonstrates; the one-sky control remains the counterexample.
            ControlMode::SingleSkyPlane => clip_rect_mesh(-0.72, 0.72, 0.04, 0.68, -0.60),
        };
        let wall_mesh = mesh_from_triangles(
            walls
                .iter()
                .filter(|triangle| Some(triangle.source_seg) == far_seg)
                .map(|triangle| triangle.positions),
            0.20,
        );
        let wall_mesh = if self.mode == ControlMode::SingleSkyPlane {
            clip_rect_mesh(-0.52, 0.52, -0.68, 0.48, 0.20)
        } else {
            wall_mesh
        };
        // Use the renderer's established 2D quad for the source-independent
        // background control. The Doom-derived meshes remain separate draws.
        renderer.upload_mesh(SKY_MESH, &Mesh::quad());
        renderer.upload_mesh(BOUNDARY_MESH, &control_mesh);
        renderer.upload_mesh(FAR_WALL_MESH, &wall_mesh);
        renderer.upload_material(
            SKY_MATERIAL,
            &Material::new("synthetic-sky", Color::rgb(0.08, 0.24, 0.48)),
        )?;
        let (control_label, control_color) = match self.mode {
            ControlMode::PairedSky => ("paired-sky-depth-only", Color::rgb(1.0, 0.0, 1.0)),
            ControlMode::OneSky => ("ordinary-upper-wall", Color::rgb(0.12, 0.72, 0.28)),
            ControlMode::SingleSkyPlane => (
                "single-source-sky-plane-depth-only",
                Color::rgb(0.0, 1.0, 1.0),
            ),
        };
        renderer.upload_material(
            BOUNDARY_MATERIAL,
            &Material::new(control_label, control_color),
        )?;
        renderer.upload_material(
            FAR_WALL_MATERIAL,
            &Material::new("far-wall", Color::rgb(0.92, 0.22, 0.12)),
        )?;

        eprintln!(
            "{} native control: source-boundary-triangles={}; control-mesh-vertices={}; far-wall-triangles={}; order=sky-colour>source-control>far-wall",
            self.mode.label(),
            boundaries.len(),
            control_mesh.vertex_count(),
            walls
                .iter()
                .filter(|triangle| Some(triangle.source_seg) == far_seg)
                .count(),
        );
        Ok(())
    }
}

impl PlatformEventHandler for PairedSkyPresentation {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1), size.height.max(1)];
        let mut renderer = WgpuBackend::for_window(window.clone(), self.size[0], self.size[1])?;
        self.sky_pipeline = renderer.register_pipeline(
            &Pipeline::new("paired-sky-background", PipelineKind::SolidColor2d).with_render_state(
                PipelineRenderState {
                    blend: BlendMode::Opaque,
                    // The surface render pass always owns a depth attachment.
                    // Keep the background depth-compatible without letting it
                    // establish visibility authority.
                    depth_test: DepthTest::LessEqual,
                    depth_write: false,
                    cull_mode: CullMode::None,
                    color_write: ColorWriteMask::ALL,
                },
            )?,
        )?;
        self.boundary_pipeline = renderer.register_pipeline(
            &Pipeline::new("paired-sky-depth-boundary", PipelineKind::SolidColor2d)
                .with_render_state(PipelineRenderState {
                    blend: BlendMode::Opaque,
                    depth_test: DepthTest::LessEqual,
                    depth_write: true,
                    cull_mode: CullMode::None,
                    color_write: match self.mode {
                        ControlMode::PairedSky | ControlMode::SingleSkyPlane => {
                            ColorWriteMask::NONE
                        }
                        ControlMode::OneSky => ColorWriteMask::ALL,
                    },
                })?,
        )?;
        self.wall_pipeline = renderer.register_pipeline(
            &Pipeline::new("paired-sky-far-wall", PipelineKind::SolidColor2d).with_render_state(
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
        renderer.upload_camera(CAMERA, Camera::default());
        window.set_title(&format!(
            "Tokimu Doom {} presentation | adapter={} | sky -> source control -> far wall",
            self.mode.label(),
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
            RenderCommand::Clear(ClearCommand {
                color: Color::rgb(0.015, 0.02, 0.03),
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: SKY_MESH,
                material: SKY_MATERIAL,
                pipeline: self.sky_pipeline,
                instance: Instance2d::identity().with_scale([1.8, 1.6]),
                // This fixture supplies clip-space coordinates through the
                // established unnamed-2D-camera path.
                camera: Some(CAMERA),
                viewport: None,
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: BOUNDARY_MESH,
                material: BOUNDARY_MATERIAL,
                pipeline: self.boundary_pipeline,
                instance: Instance2d::identity(),
                camera: Some(CAMERA),
                viewport: None,
            }),
            RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: FAR_WALL_MESH,
                material: FAR_WALL_MATERIAL,
                pipeline: self.wall_pipeline,
                instance: Instance2d::identity(),
                camera: Some(CAMERA),
                viewport: None,
            }),
        ]);
        let stats = renderer.present()?;
        let diagnostics = renderer.drain_diagnostics();
        if let Some(record) = diagnostics.first() {
            let message = format!(
                "paired-sky backend diagnostic: frame={}; category={:?}; source={}; message={}",
                self.presented_frames.saturating_add(1),
                record.kind,
                record.source,
                record.message
            );
            eprintln!("{message}");
            return Err(std::io::Error::other(message).into());
        }
        if self.presented_frames == 0 {
            let diagnostic = "none";
            if let Some(window) = self.window.as_ref() {
                window.set_title(&format!(
                    "Tokimu Doom {} | draws={} materials={} pipelines={} diagnostic={diagnostic}",
                    self.mode.label(),
                    stats.frame.draw_calls,
                    stats.frame.material_resolutions,
                    stats.frame.pipeline_switches,
                ));
            }
            eprintln!(
                "{} first frame: draws={}; materials={}; pipelines={}; diagnostic={diagnostic}",
                self.mode.label(),
                stats.frame.draw_calls,
                stats.frame.material_resolutions,
                stats.frame.pipeline_switches,
            );
        } else if self.presented_frames == 1 {
            if stats.frame.mesh_uploads != 0 || stats.frame.mesh_replacements != 0 {
                return Err(std::io::Error::other(format!(
                    "{} unchanged warm frame mutated static meshes: uploads={}; replacements={}",
                    self.mode.label(),
                    stats.frame.mesh_uploads,
                    stats.frame.mesh_replacements
                ))
                .into());
            }
            eprintln!(
                "{} warm frame: draws={}; materials={}; pipelines={}; mesh_uploads={}; mesh_replacements={}; lifetime_mesh_uploads={}; lifetime_mesh_replacements={}; diagnostic=none",
                self.mode.label(),
                stats.frame.draw_calls,
                stats.frame.material_resolutions,
                stats.frame.pipeline_switches,
                stats.frame.mesh_uploads,
                stats.frame.mesh_replacements,
                stats.lifetime.mesh_uploads,
                stats.lifetime.mesh_replacements,
            );
        } else if self.presented_frames == 2 {
            if stats.frame.mesh_uploads != 0 || stats.frame.mesh_replacements != 0 {
                return Err(std::io::Error::other(format!(
                    "{} bounded camera move mutated static meshes: uploads={}; replacements={}",
                    self.mode.label(),
                    stats.frame.mesh_uploads,
                    stats.frame.mesh_replacements
                ))
                .into());
            }
            eprintln!(
                "{} camera-move frame: offset_x=0.08; draws={}; materials={}; pipelines={}; mesh_uploads={}; mesh_replacements={}; lifetime_mesh_uploads={}; lifetime_mesh_replacements={}; diagnostic=none",
                self.mode.label(),
                stats.frame.draw_calls,
                stats.frame.material_resolutions,
                stats.frame.pipeline_switches,
                stats.frame.mesh_uploads,
                stats.frame.mesh_replacements,
                stats.lifetime.mesh_uploads,
                stats.lifetime.mesh_replacements,
            );
        }
        self.presented_frames = self.presented_frames.saturating_add(1);
        Ok(FrameOutcome::Continue)
    }
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

/// Fixed screen-space diagnostic geometry for the single-sky-plane control.
/// The source fixture supplies the named `F_SKY1` ceiling identity; this helper
/// only makes its declared bounded projection legible in a corpus window.
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
    fn paired_sky_control_uses_explicit_three_draw_order() {
        let fixture = paired_sky_far_control_fixture().unwrap();
        let boundaries = lower_doom_paired_sky_boundary_triangles(&fixture.map).unwrap();
        assert_eq!(boundaries.len(), 2);
        assert_eq!(
            mesh_from_triangles(boundaries.iter().map(|triangle| triangle.positions), -0.60)
                .vertex_count(),
            6
        );
    }

    #[test]
    fn single_sky_plane_control_retains_a_bounded_depth_only_interval() {
        let mesh = clip_rect_mesh(-0.72, 0.72, 0.04, 0.68, -0.60);
        assert_eq!(mesh.vertex_count(), 6);
        assert!(single_sky_plane_far_control_fixture().is_ok());
    }
}
