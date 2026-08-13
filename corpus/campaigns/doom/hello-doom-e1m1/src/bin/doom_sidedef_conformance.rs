//! Bounded visual proof for Doom right/front and left/back texture direction.
//!
//! The fixture passes two synthetic one-sided linedefs through the real Doom
//! geometry provider. Their opposed source directions make each owning side
//! face the same camera while preserving distinct right/front and left/back
//! readable asymmetric source art. No Doom-specific branch exists in the
//! renderer.

use std::{env, sync::Arc};

use doom_geometry_provider::{
    lower_doom_textured_wall_triangles, DoomTextureExtent, DoomWallSideKind,
};
use doom_map_provider::{
    DoomBlockmapObservation, DoomLinedef, DoomMapCore, DoomRejectMatrix, DoomSector, DoomSidedef,
    DoomSourceRecord, DoomVertex,
};
use hello_doom_e1m1::{
    lower_static_wall_triangle, reembed_comparative_mesh, DoomComparativeEmbedding,
};
use render_orientation_conformance::{
    directional_atlas_rgba8, DIRECTIONAL_ATLAS_HEIGHT, DIRECTIONAL_ATLAS_WIDTH,
};
use tokimu::{
    run_window_with_app, BlendMode, Camera, CameraHandle, ClearCommand, Color, ColorWriteMask,
    CullMode, DepthTest, DrawMeshCommand, FrameOutcome, Instance2d, Material, MaterialHandle,
    MeshHandle, NativeWindow, Pipeline, PipelineHandle, PipelineKind, PipelineRenderState,
    PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand, Renderer,
    Rgba8TextureColorSpace, Rgba8TextureDescriptor, TextureHandle, WgpuBackend, WindowConfig,
};
use tokimu_core::math::Vec3;

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 600;
const CAMERA: CameraHandle = CameraHandle(1);
const FRONT_TEXTURE: TextureHandle = TextureHandle(1);
const BACK_TEXTURE: TextureHandle = TextureHandle(2);
const FRONT_MATERIAL: MaterialHandle = MaterialHandle(1);
const BACK_MATERIAL: MaterialHandle = MaterialHandle(2);

fn main() -> PlatformResult<()> {
    let embedding = match env::args().nth(1).as_deref() {
        None | Some("current") => DoomComparativeEmbedding::CurrentReflected,
        Some("east") => DoomComparativeEmbedding::PreserveEast,
        Some("north") => DoomComparativeEmbedding::PreserveNorth,
        Some(_) => return Err("usage: doom_sidedef_conformance [current|east|north]".into()),
    };
    run_window_with_app(
        WindowConfig {
            title: format!(
                "Tokimu Doom sidedef conformance | embedding={embedding:?} | left: left/back | right: right/front"
            ),
            width: WIDTH,
            height: HEIGHT,
        },
        SidedefApp {
            embedding,
            ..SidedefApp::default()
        },
    )
}

struct SidedefApp {
    renderer: Option<WgpuBackend>,
    pipeline: PipelineHandle,
    size: [u32; 2],
    embedding: DoomComparativeEmbedding,
}

impl Default for SidedefApp {
    fn default() -> Self {
        Self {
            renderer: None,
            pipeline: PipelineHandle(0),
            size: [0, 0],
            embedding: DoomComparativeEmbedding::CurrentReflected,
        }
    }
}

impl PlatformEventHandler for SidedefApp {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1), size.height.max(1)];
        let mut renderer = WgpuBackend::for_window(window, self.size[0], self.size[1])?;

        let triangles = fixture_triangles();
        for (index, triangle) in triangles.iter().enumerate() {
            let extent = DoomTextureExtent {
                name: triangle.texture_name.clone(),
                width: 320,
                height: 96,
            };
            let mut lowered = lower_static_wall_triangle(triangle, extent)
                .expect("bounded sidedef triangle must lower");
            reembed_comparative_mesh(&mut lowered.mesh, self.embedding, true);
            renderer.upload_mesh(MeshHandle(index as u64 + 1), &lowered.mesh);
        }

        let (front, back) = split_directional_atlas();
        let descriptor = Rgba8TextureDescriptor::new(320, 96, Rgba8TextureColorSpace::Srgb);
        renderer.create_texture_rgba8(FRONT_TEXTURE, descriptor, &front)?;
        renderer.create_texture_rgba8(BACK_TEXTURE, descriptor, &back)?;
        renderer.upload_material(
            FRONT_MATERIAL,
            &Material::new("doom-right-front-label", Color::rgb(1.0, 1.0, 1.0))
                .with_texture(FRONT_TEXTURE),
        )?;
        renderer.upload_material(
            BACK_MATERIAL,
            &Material::new("doom-left-back-label", Color::rgb(1.0, 1.0, 1.0))
                .with_texture(BACK_TEXTURE),
        )?;

        renderer.upload_camera(CAMERA, fixture_camera(self.size, self.embedding));
        self.pipeline = renderer.register_pipeline(
            &Pipeline::new("doom-sidedef-conformance", PipelineKind::Textured3d)
                .with_render_state(PipelineRenderState {
                    blend: BlendMode::Opaque,
                    depth_test: DepthTest::LessEqual,
                    depth_write: true,
                    cull_mode: CullMode::Back,
                    color_write: ColorWriteMask::ALL,
                })?,
        )?;
        self.renderer = Some(renderer);
        Ok(())
    }

    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::Resized { width, height } = event {
            self.size = [width.max(1), height.max(1)];
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize_surface(width, height);
                renderer.upload_camera(CAMERA, fixture_camera(self.size, self.embedding));
            }
        }
        Ok(())
    }

    fn on_frame(&mut self, _delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        let Some(renderer) = self.renderer.as_mut() else {
            return Ok(FrameOutcome::Continue);
        };
        let triangles = fixture_triangles();
        let mut commands = vec![RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.025, 0.035, 0.045),
        })];
        for (index, triangle) in triangles.iter().enumerate() {
            let material = match triangle.side {
                DoomWallSideKind::Right => FRONT_MATERIAL,
                DoomWallSideKind::Left => BACK_MATERIAL,
            };
            commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: MeshHandle(index as u64 + 1),
                material,
                pipeline: self.pipeline,
                instance: Instance2d::default(),
                camera: Some(CAMERA),
                viewport: None,
            }));
        }
        renderer.begin_frame();
        renderer.submit(&commands);
        let _ = renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}

fn fixture_camera(size: [u32; 2], embedding: DoomComparativeEmbedding) -> Camera {
    let source_eye = [0.0, -700.0];
    let source_forward = [0.0, 1.0];
    let eye = embedding.lift_direction(source_eye, 48.0);
    let forward = embedding.lift_direction(source_forward, 0.0);
    Camera {
        view: tokimu_core::math::try_view_look_at_rh(eye, eye + forward, Vec3::Y)
            .expect("camera basis must be finite and non-degenerate"),
        projection: tokimu_core::math::try_projection_perspective_rh_gl(
            45.0_f32.to_radians(),
            size[0].max(1) as f32 / size[1].max(1) as f32,
            0.1,
            1_000.0,
        )
        .expect("perspective parameters must be finite and ordered"),
    }
}

fn fixture_triangles() -> Vec<doom_geometry_provider::DoomTexturedWallTriangle> {
    lower_doom_textured_wall_triangles(
        &fixture_map(),
        &[
            DoomTextureExtent {
                name: "FRONT_LABEL".into(),
                width: 320,
                height: 96,
            },
            DoomTextureExtent {
                name: "BACK_LABEL".into(),
                width: 320,
                height: 96,
            },
        ],
    )
    .expect("bounded two-sided fixture must lower")
}

fn fixture_map() -> DoomMapCore {
    let source = |record_index| DoomSourceRecord {
        lump_index: 1,
        record_index,
    };
    DoomMapCore {
        map_name: "SIDEFIX".into(),
        things: Vec::new(),
        vertices: vec![
            DoomVertex {
                source: source(0),
                x: -340,
                y: 0,
            },
            DoomVertex {
                source: source(1),
                x: -20,
                y: 0,
            },
            DoomVertex {
                source: source(2),
                x: 340,
                y: 0,
            },
            DoomVertex {
                source: source(3),
                x: 20,
                y: 0,
            },
        ],
        linedefs: vec![
            DoomLinedef {
                source: source(0),
                start_vertex: 0,
                end_vertex: 1,
                flags: 0,
                special: 0,
                tag: 0,
                right_sidedef: Some(0),
                left_sidedef: None,
            },
            DoomLinedef {
                source: source(1),
                start_vertex: 2,
                end_vertex: 3,
                flags: 0,
                special: 0,
                tag: 0,
                right_sidedef: None,
                left_sidedef: Some(1),
            },
        ],
        sidedefs: vec![
            DoomSidedef {
                source: source(0),
                x_offset: 0,
                y_offset: 0,
                upper_texture: "-".into(),
                lower_texture: "-".into(),
                middle_texture: "FRONT_LABEL".into(),
                sector: 0,
            },
            DoomSidedef {
                source: source(1),
                x_offset: 0,
                y_offset: 0,
                upper_texture: "-".into(),
                lower_texture: "-".into(),
                middle_texture: "BACK_LABEL".into(),
                sector: 1,
            },
        ],
        sectors: vec![fixture_sector(source(0)), fixture_sector(source(1))],
        segs: Vec::new(),
        subsectors: Vec::new(),
        nodes: Vec::new(),
        reject: DoomRejectMatrix::default(),
        blockmap: DoomBlockmapObservation {
            lump_index: 0,
            origin_x: 0,
            origin_y: 0,
            columns: 0,
            rows: 0,
            cells: 0,
            unique_linedef_lists: 0,
            linedef_references: 0,
            cell_linedefs: Vec::new(),
        },
    }
}

fn fixture_sector(source: DoomSourceRecord) -> DoomSector {
    DoomSector {
        source,
        floor_height: 0,
        ceiling_height: 96,
        floor_texture: "FLOOR0_1".into(),
        ceiling_texture: "CEIL1_1".into(),
        light_level: 160,
        special: 0,
        tag: 0,
    }
}

fn split_directional_atlas() -> (Vec<u8>, Vec<u8>) {
    assert_eq!(DIRECTIONAL_ATLAS_WIDTH, 320);
    assert_eq!(DIRECTIONAL_ATLAS_HEIGHT, 192);
    let atlas = directional_atlas_rgba8();
    let panel_bytes = (DIRECTIONAL_ATLAS_WIDTH * (DIRECTIONAL_ATLAS_HEIGHT / 2) * 4) as usize;
    let front = atlas[..panel_bytes].to_vec();
    let back = atlas[panel_bytes..].to_vec();
    (front, back)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_lowers_two_triangles_for_each_opposed_owning_side() {
        let triangles = fixture_triangles();
        assert_eq!(triangles.len(), 4);
        assert_eq!(
            triangles
                .iter()
                .filter(|item| item.side == DoomWallSideKind::Right)
                .count(),
            2
        );
        assert_eq!(
            triangles
                .iter()
                .filter(|item| item.side == DoomWallSideKind::Left)
                .count(),
            2
        );
        assert!(triangles
            .iter()
            .filter(|item| item.side == DoomWallSideKind::Right)
            .all(|item| item.texture_name == "FRONT_LABEL"));
        assert!(triangles
            .iter()
            .filter(|item| item.side == DoomWallSideKind::Left)
            .all(|item| item.texture_name == "BACK_LABEL"));

        let right_center = triangles
            .iter()
            .filter(|item| item.side == DoomWallSideKind::Right)
            .flat_map(|item| item.positions)
            .map(|position| position[0])
            .sum::<f64>()
            / 6.0;
        let left_center = triangles
            .iter()
            .filter(|item| item.side == DoomWallSideKind::Left)
            .flat_map(|item| item.positions)
            .map(|position| position[0])
            .sum::<f64>()
            / 6.0;
        assert!(right_center < 0.0);
        assert!(left_center > 0.0);
    }

    #[test]
    fn atlas_split_retains_one_complete_panel_per_side() {
        let (front, back) = split_directional_atlas();
        assert_eq!(front.len(), 320 * 96 * 4);
        assert_eq!(back.len(), 320 * 96 * 4);
        assert_ne!(front, back);
    }

    #[test]
    fn orientation_preserving_candidates_rebuild_camera_facing_winding() {
        for embedding in [
            DoomComparativeEmbedding::PreserveEast,
            DoomComparativeEmbedding::PreserveNorth,
        ] {
            let eye = embedding.lift_direction([0.0, -700.0], 48.0);
            for triangle in fixture_triangles() {
                let extent = DoomTextureExtent {
                    name: triangle.texture_name.clone(),
                    width: 320,
                    height: 96,
                };
                let mut lowered = lower_static_wall_triangle(&triangle, extent).unwrap();
                reembed_comparative_mesh(&mut lowered.mesh, embedding, true);

                let center = lowered
                    .mesh
                    .positions
                    .iter()
                    .map(|position| Vec3::from_array(*position))
                    .sum::<Vec3>()
                    / lowered.mesh.positions.len() as f32;
                let normal = Vec3::from_array(lowered.mesh.normals[0]);
                assert!(
                    normal.dot(eye - center) > 0.0,
                    "{embedding:?} {:?} wall no longer faces its source-side observer",
                    triangle.side
                );
                assert!(lowered
                    .mesh
                    .normals
                    .iter()
                    .all(|candidate| Vec3::from_array(*candidate).dot(normal) > 0.999));
            }
        }
    }

    #[test]
    fn orientation_preserving_candidates_keep_readable_u_toward_camera_right() {
        for embedding in [
            DoomComparativeEmbedding::PreserveEast,
            DoomComparativeEmbedding::PreserveNorth,
        ] {
            let forward = embedding.lift_direction([0.0, 1.0], 0.0).normalize();
            let camera_right = forward.cross(Vec3::Y).normalize();
            for triangle in fixture_triangles() {
                let extent = DoomTextureExtent {
                    name: triangle.texture_name.clone(),
                    width: 320,
                    height: 96,
                };
                let mut lowered = lower_static_wall_triangle(&triangle, extent).unwrap();
                reembed_comparative_mesh(&mut lowered.mesh, embedding, true);

                let mut horizontal_pair_observed = false;
                for left in 0..lowered.mesh.positions.len() {
                    for right in left + 1..lowered.mesh.positions.len() {
                        let screen_delta = camera_right.dot(
                            Vec3::from_array(lowered.mesh.positions[right])
                                - Vec3::from_array(lowered.mesh.positions[left]),
                        );
                        let u_delta = lowered.mesh.texture_coordinates[right][0]
                            - lowered.mesh.texture_coordinates[left][0];
                        if screen_delta.abs() > 0.001 && u_delta.abs() > 0.001 {
                            horizontal_pair_observed = true;
                            assert!(
                                screen_delta * u_delta > 0.0,
                                "{embedding:?} {:?} wall reverses readable U across the camera",
                                triangle.side
                            );
                        }
                    }
                }
                assert!(horizontal_pair_observed);
            }
        }
    }
}
