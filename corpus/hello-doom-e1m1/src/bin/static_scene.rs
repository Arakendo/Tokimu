//! Native first-frame proof for the Slice 5B static E1M1 presentation policy.
//!
//! The WAD is read only at this corpus edge. `tokimu-render` receives ordinary
//! meshes, texture bytes, materials, and one explicit opaque 3D pipeline.

use std::{env, fs, io, sync::Arc};

use archive_provider::{ArchiveFormat, ArchiveReadLimits, ZipArchiveProvider};
use doom_geometry_provider::{
    locate_doom_point_subsector, resolve_doom_subsector_bsp_paths,
    resolve_doom_subsector_sector_ownership,
};
use doom_map_provider::{decode_doom_map_core, resolve_doom_player_one_start};
use doom_raster_provider::{
    DoomFlatDecodeLimits, DoomPatchDecodeLimits, DoomRasterDecodeLimits, DoomTextureComposeLimits,
    DoomTextureDecodeLimits,
};
use doom_wad_package::{
    read_wad_package_member, select_doom_episode_map, InspectWadPackageRequest,
};
use doom_wad_provider::WadReadLimits;
use hello_doom_e1m1::{
    build_experimental_cutout_draw_plan, build_experimental_cutout_texture_uploads,
    build_static_draw_plan, build_static_texture_uploads, prepare_e1m1_flat_textures,
    prepare_e1m1_flats, prepare_e1m1_masked_middle_cutouts, prepare_e1m1_wall_textures,
    prepare_e1m1_walls, prepared_e1m1_masked_middle_texture_names, StaticDrawPlanEntry,
    StaticTextureUpload,
};
use resource_space::{
    AddressCasePolicy, FolderId, InMemoryResourceSpace, ResourceMetadata, ResourceName,
    ResourceRootDescriptor, ResourceRootId, StoreId,
};
use resource_space_archive::InspectArchiveResourceRequest;
use tokimu::{
    run_window_with_app, BlendMode, Camera, CameraHandle, CategoricalCutout, ClearCommand, Color,
    ColorWriteMask, CullMode, CutoutComparison, CutoutThreshold, DepthTest, DrawMeshCommand,
    FrameOutcome, Instance2d, MeshHandle, NativeWindow, Pipeline, PipelineHandle, PipelineKind,
    PipelineRenderState, PlatformEventHandler, PlatformInputEvent, PlatformResult, RenderCommand,
    Renderer, WgpuBackend, WindowConfig,
};
use tokimu_core::math::{Mat4, Vec3};

const WAD_LIMITS: WadReadLimits =
    WadReadLimits::new(64 * 1024 * 1024, 8_192, 16 * 1024 * 1024, 64 * 1024 * 1024);
const MAP_LIMITS: doom_map_provider::DoomMapDecodeLimits = doom_map_provider::DoomMapDecodeLimits {
    max_things: 100_000,
    max_vertices: 100_000,
    max_linedefs: 100_000,
    max_sidedefs: 100_000,
    max_sectors: 100_000,
    max_segs: 100_000,
    max_subsectors: 100_000,
    max_nodes: 100_000,
    max_reject_bytes: 64 * 1024 * 1024,
    max_blockmap_bytes: 64 * 1024 * 1024,
    max_blockmap_cells: 1_000_000,
    max_blockmap_linedef_refs: 10_000_000,
    max_total_record_bytes: 64 * 1024 * 1024,
};
const RASTER_LIMITS: DoomRasterDecodeLimits = DoomRasterDecodeLimits {
    max_playpal_bytes: 64 * 1024 * 1024,
    max_palettes: 4096,
    max_colormap_bytes: 64 * 1024 * 1024,
    max_colormaps: 4096,
    max_total_decoded_bytes: 128 * 1024 * 1024,
};
const FLAT_LIMITS: DoomFlatDecodeLimits = DoomFlatDecodeLimits {
    max_flat_bytes: 4096,
};
const TEXTURE_LIMITS: DoomTextureDecodeLimits = DoomTextureDecodeLimits {
    max_pnames_bytes: 64 * 1024 * 1024,
    max_texture_bytes: 64 * 1024 * 1024,
    max_patch_names: 1_000_000,
    max_textures: 1_000_000,
    max_patches_per_texture: 16_384,
    max_total_patch_references: 10_000_000,
};
const PATCH_LIMITS: DoomPatchDecodeLimits = DoomPatchDecodeLimits {
    max_patch_bytes: 64 * 1024 * 1024,
    max_width: 4096,
    max_height: 4096,
    max_pixels: 16 * 1024 * 1024,
    max_posts: 16 * 1024 * 1024,
};
const COMPOSE_LIMITS: DoomTextureComposeLimits = DoomTextureComposeLimits {
    max_width: 4096,
    max_height: 4096,
    max_pixels: 16 * 1024 * 1024,
};
const CAMERA: CameraHandle = CameraHandle(1);

struct App {
    renderer: Option<WgpuBackend>,
    draws: Vec<StaticDrawPlanEntry>,
    uploads: Vec<StaticTextureUpload>,
    cutout_draws: Vec<StaticDrawPlanEntry>,
    cutout_uploads: Vec<StaticTextureUpload>,
    include_cutouts: bool,
    pipeline: PipelineHandle,
    cutout_pipeline: Option<PipelineHandle>,
    size: [f32; 2],
    center: Vec3,
    radius: f32,
    spawn_observer: Option<SpawnObserver>,
    observer_look: Option<ObserverLook>,
}

struct SceneInput {
    opaque_draws: Vec<StaticDrawPlanEntry>,
    opaque_uploads: Vec<StaticTextureUpload>,
    cutout_draws: Vec<StaticDrawPlanEntry>,
    cutout_uploads: Vec<StaticTextureUpload>,
    spawn_observer: SpawnObserver,
}

/// Corpus-local source-spawn observer. This is a fixed visual evidence camera,
/// not runtime player state, movement, collision, or an original-Doom claim.
#[derive(Clone, Copy, Debug)]
struct SpawnObserver {
    position: Vec3,
    forward: Vec3,
    source_record: u32,
    source_position: [i16; 2],
    source_angle: u16,
    sector: u32,
    floor: i16,
    ceiling: i16,
}

/// Presentation-only look state for the opt-in source-spawn observer. It is
/// deliberately not imported player orientation, runtime state, or input
/// policy beyond this native evidence application.
#[derive(Clone, Copy, Debug)]
struct ObserverLook {
    yaw: f32,
    pitch: f32,
    last_cursor: Option<[f32; 2]>,
}

fn main() -> PlatformResult<()> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let include_cutouts = args.iter().any(|argument| argument == "--masked-cutouts");
    let spawn_observer = args.iter().any(|argument| argument == "--spawn-observer");
    args.retain(|argument| argument != "--masked-cutouts");
    args.retain(|argument| argument != "--spawn-observer");
    let [package, member] = args.as_slice() else {
        return Err(
            "usage: static_scene <canonical-doom-zip> <WAD-member-name> [--masked-cutouts] [--spawn-observer]".into(),
        );
    };
    let scene = prepare_scene(package, member)?;
    let bounds_draws = scene
        .opaque_draws
        .iter()
        .chain(scene.cutout_draws.iter())
        .cloned()
        .collect::<Vec<_>>();
    let (center, radius) = scene_bounds(&bounds_draws);
    let draw_count = scene.opaque_draws.len()
        + if include_cutouts {
            scene.cutout_draws.len()
        } else {
            0
        };
    run_window_with_app(
        WindowConfig {
            title: format!("Tokimu DOOM E1M1 | {draw_count} draws"),
            width: 1280,
            height: 800,
        },
        App {
            renderer: None,
            draws: scene.opaque_draws,
            uploads: scene.opaque_uploads,
            cutout_draws: scene.cutout_draws,
            cutout_uploads: scene.cutout_uploads,
            include_cutouts,
            pipeline: PipelineHandle(0),
            cutout_pipeline: None,
            size: [1280.0, 800.0],
            center,
            radius,
            spawn_observer: spawn_observer.then_some(scene.spawn_observer),
            observer_look: spawn_observer.then_some(ObserverLook {
                yaw: spawn_heading_yaw(scene.spawn_observer.forward),
                pitch: 0.0,
                last_cursor: None,
            }),
        },
    )
}

impl PlatformEventHandler for App {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window, size.width, size.height)?;
        for upload in &self.uploads {
            renderer.create_texture_rgba8(upload.texture, upload.descriptor, &upload.rgba8)?;
            renderer.upload_material(upload.material, &upload.material_value)?;
        }
        if self.include_cutouts {
            for upload in &self.cutout_uploads {
                renderer.create_texture_rgba8(upload.texture, upload.descriptor, &upload.rgba8)?;
                renderer.upload_material(upload.material, &upload.material_value)?;
            }
        }
        if let Some(observer) = self.spawn_observer {
            eprintln!(
                "E1M1 source-spawn observer: THINGS #{} at=({}, {}) angle={} sector={} floor={} ceiling={} eye=({:.1}, {:.1}, {:.1}) forward=({:.3}, {:.3}, {:.3})",
                observer.source_record,
                observer.source_position[0],
                observer.source_position[1],
                observer.source_angle,
                observer.sector,
                observer.floor,
                observer.ceiling,
                observer.position.x,
                observer.position.y,
                observer.position.z,
                observer.forward.x,
                observer.forward.y,
                observer.forward.z,
            );
        }
        self.pipeline = renderer.register_pipeline(
            &Pipeline::new("doom-e1m1-static-opaque", PipelineKind::Textured3d).with_render_state(
                PipelineRenderState {
                    blend: BlendMode::Opaque,
                    depth_test: DepthTest::LessEqual,
                    depth_write: true,
                    cull_mode: CullMode::Back,
                    color_write: ColorWriteMask::ALL,
                },
            )?,
        )?;
        if self.include_cutouts {
            self.cutout_pipeline =
                Some(renderer.register_pipeline(&Pipeline::textured_3d_cutout(
                    "doom-e1m1-masked-cutout",
                    CategoricalCutout::new(
                        CutoutThreshold::new(0.0)?,
                        CutoutComparison::DiscardAtOrBelow,
                    ),
                ))?);
        }
        self.upload_static_meshes(&mut renderer);
        eprintln!(
            "E1M1 native first-frame metadata: opaque_draws={}; cutout_draws={}; cutouts_enabled={}; camera={}; backend={}; device={}; adapter={}",
            self.draws.len(),
            self.cutout_draws.len(),
            self.include_cutouts,
            if self.spawn_observer.is_some() { "source-spawn-observer" } else { "overview" },
            renderer.backend_api(),
            renderer.device_kind(),
            renderer.adapter_name(),
        );
        self.renderer = Some(renderer);
        Ok(())
    }
    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::MouseMotion { delta_x, delta_y } = event {
            if let Some(look) = self.observer_look.as_mut() {
                apply_observer_look_delta(look, delta_x, delta_y);
            }
            return Ok(());
        }
        if let PlatformInputEvent::CursorMoved { x, y } = event {
            if let Some(look) = self.observer_look.as_mut() {
                if let Some([last_x, last_y]) = look.last_cursor {
                    apply_observer_look_delta(look, x - last_x, y - last_y);
                }
                look.last_cursor = Some([x, y]);
            }
            return Ok(());
        }
        if let PlatformInputEvent::Resized { width, height } = event {
            self.size = [width.max(1) as f32, height.max(1) as f32];
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize_surface(width, height);
            }
        }
        Ok(())
    }
    fn on_frame(&mut self, _: f64) -> PlatformResult<FrameOutcome> {
        let mut camera = Camera::perspective_3d(self.size[0], self.size[1]);
        // `Camera::perspective_3d` deliberately serves small corpus fixtures
        // with a 100-unit far plane. E1M1's ordinary source coordinates span
        // thousands of units, so this consumer owns an explicit overview
        // projection rather than treating that convenience default as a
        // renderer-wide Doom policy.
        let aspect = self.size[0] / self.size[1].max(1.0);
        camera.projection = Mat4::perspective_rh_gl(
            60.0_f32.to_radians(),
            aspect,
            (self.radius * 0.000_1).max(0.1),
            self.radius * 4.0,
        );
        camera.view =
            if let (Some(observer), Some(look)) = (self.spawn_observer, self.observer_look) {
                Mat4::look_at_rh(
                    observer.position,
                    observer.position + observer_forward(look.yaw, look.pitch) * 128.0,
                    Vec3::Y,
                )
            } else {
                Mat4::look_at_rh(
                    self.center + Vec3::new(self.radius, self.radius * 0.72, self.radius),
                    self.center,
                    Vec3::Y,
                )
            };
        let renderer = self
            .renderer
            .as_mut()
            .ok_or_else(|| io::Error::other("renderer missing"))?;
        renderer.upload_camera(CAMERA, camera);
        renderer.begin_frame();
        let mut commands = vec![RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.015, 0.02, 0.025),
        })];
        for (index, draw) in self.draws.iter().enumerate() {
            let mesh = MeshHandle(index as u64 + 1);
            commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh,
                material: draw.material,
                pipeline: self.pipeline,
                instance: Instance2d::identity(),
                camera: Some(CAMERA),
                viewport: None,
            }));
        }
        if self.include_cutouts {
            let cutout_pipeline = self
                .cutout_pipeline
                .ok_or_else(|| io::Error::other("masked-cutout pipeline missing"))?;
            for (offset, draw) in self.cutout_draws.iter().enumerate() {
                let mesh = MeshHandle(self.draws.len() as u64 + offset as u64 + 1);
                commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh,
                    material: draw.material,
                    pipeline: cutout_pipeline,
                    instance: Instance2d::identity(),
                    camera: Some(CAMERA),
                    viewport: None,
                }));
            }
        }
        renderer.submit(&commands);
        renderer.present()?;
        Ok(FrameOutcome::Continue)
    }
}

impl App {
    /// Static corpus geometry crosses the provider boundary once at startup.
    /// Camera motion changes only the uploaded camera and submitted draws;
    /// re-uploading 1,861 immutable meshes per frame would be avoidable
    /// steady-state allocation and buffer replacement.
    fn upload_static_meshes(&self, renderer: &mut WgpuBackend) {
        for (index, draw) in self.draws.iter().enumerate() {
            renderer.upload_mesh(MeshHandle(index as u64 + 1), &draw.mesh);
        }
        if self.include_cutouts {
            for (offset, draw) in self.cutout_draws.iter().enumerate() {
                renderer.upload_mesh(
                    MeshHandle(self.draws.len() as u64 + offset as u64 + 1),
                    &draw.mesh,
                );
            }
        }
    }
}

fn prepare_scene(package: &str, member: &str) -> PlatformResult<SceneInput> {
    let bytes = fs::read(package)?;
    let mut space =
        InMemoryResourceSpace::new(StoreId::from_u128(5_101), AddressCasePolicy::Sensitive);
    let folder = FolderId::from_u128(5_102);
    space.create_root(
        ResourceRootDescriptor::new(ResourceRootId::from_u128(5_103), "E1M1 native package"),
        folder,
        ResourceMetadata::default(),
    )?;
    let name = ResourceName::parse("canonical-doom-package.zip", AddressCasePolicy::Sensitive)?;
    space.insert_resource(folder, name.clone(), bytes, ResourceMetadata::default())?;
    let read = read_wad_package_member(
        &space,
        InspectWadPackageRequest {
            archive: InspectArchiveResourceRequest {
                source_folder: folder,
                source_name: name,
                format: ArchiveFormat::Zip,
                limits: ArchiveReadLimits::new(
                    64 * 1024 * 1024,
                    2048,
                    16 * 1024 * 1024,
                    64 * 1024 * 1024,
                    4096,
                ),
            },
            member_name: member.to_owned(),
            wad_source_label: format!("{package}:{member}"),
            wad_limits: WAD_LIMITS,
        },
        &ZipArchiveProvider,
    )?;
    let selection = select_doom_episode_map(&read.observation.wad, "E1M1")?;
    let map = decode_doom_map_core(&read.bytes, &selection, MAP_LIMITS)?;
    let start = resolve_doom_player_one_start(&map.things)?;
    let paths = resolve_doom_subsector_bsp_paths(&map)?;
    let location = locate_doom_point_subsector(start.position, &paths)?;
    let ownership = resolve_doom_subsector_sector_ownership(&map)?;
    let sector = ownership
        .iter()
        .find(|entry| entry.source_subsector == location.source_subsector)
        .ok_or_else(|| io::Error::other("player-one start subsector has no sector ownership"))?;
    let vertical = &map.sectors[usize::from(sector.sector_index)];
    let spawn_observer = SpawnObserver {
        position: Vec3::new(
            f32::from(start.position[0]),
            (f32::from(vertical.floor_height) + f32::from(vertical.ceiling_height)) * 0.5,
            f32::from(start.position[1]),
        ),
        forward: spawn_forward(start.angle),
        source_record: start.source.record_index,
        source_position: start.position,
        source_angle: start.angle,
        sector: sector.source_sector.record_index,
        floor: vertical.floor_height,
        ceiling: vertical.ceiling_height,
    };
    let flats = prepare_e1m1_flats(&read.bytes, &read.observation.wad, MAP_LIMITS)?;
    let walls = prepare_e1m1_walls(
        &read.bytes,
        &read.observation.wad,
        MAP_LIMITS,
        TEXTURE_LIMITS,
    )?;
    let cutouts = prepare_e1m1_masked_middle_cutouts(
        &read.bytes,
        &read.observation.wad,
        MAP_LIMITS,
        TEXTURE_LIMITS,
    )?;
    let flat_textures = prepare_e1m1_flat_textures(
        &read.bytes,
        &read.observation.wad,
        &flats,
        RASTER_LIMITS,
        FLAT_LIMITS,
    )?;
    let names = hello_doom_e1m1::prepared_e1m1_wall_texture_names(&walls);
    let wall_textures = prepare_e1m1_wall_textures(
        &read.bytes,
        &read.observation.wad,
        &names,
        RASTER_LIMITS,
        TEXTURE_LIMITS,
        PATCH_LIMITS,
        COMPOSE_LIMITS,
    )?;
    let uploads = build_static_texture_uploads(&flat_textures, &wall_textures);
    let draws = build_static_draw_plan(&flats, &walls, &uploads)?;
    let masked_names = prepared_e1m1_masked_middle_texture_names(&walls);
    let masked_textures = prepare_e1m1_wall_textures(
        &read.bytes,
        &read.observation.wad,
        &masked_names,
        RASTER_LIMITS,
        TEXTURE_LIMITS,
        PATCH_LIMITS,
        COMPOSE_LIMITS,
    )?;
    let cutout_uploads =
        build_experimental_cutout_texture_uploads(&masked_textures, uploads.len() as u64 + 1);
    let cutout_draws = build_experimental_cutout_draw_plan(&cutouts, &cutout_uploads)?;
    Ok(SceneInput {
        opaque_draws: draws,
        opaque_uploads: uploads,
        cutout_draws,
        cutout_uploads,
        spawn_observer,
    })
}

/// Maps classic Doom heading degrees onto the corpus X/Z world convention:
/// angle 0 points +X and 90 points +Z. This has no pitch or player policy.
fn spawn_forward(angle: u16) -> Vec3 {
    let radians = f32::from(angle).to_radians();
    Vec3::new(radians.cos(), 0.0, radians.sin())
}

/// Converts the corpus X/Z source heading into the observer-camera yaw
/// convention where yaw zero points +Z.
fn spawn_heading_yaw(forward: Vec3) -> f32 {
    forward.x.atan2(forward.z)
}

/// Produces a right-handed first-person camera direction. Positive yaw turns
/// from +Z toward +X; positive pitch looks up. This stays local to the static
/// observer until a runtime player policy is separately admitted.
fn observer_forward(yaw: f32, pitch: f32) -> Vec3 {
    Vec3::new(
        yaw.sin() * pitch.cos(),
        pitch.sin(),
        yaw.cos() * pitch.cos(),
    )
    .normalize_or_zero()
}

fn apply_observer_look_delta(look: &mut ObserverLook, delta_x: f32, delta_y: f32) {
    // `look_at_rh` receives the source-world forward vector, whose horizontal
    // view sign is opposite the screen-space cursor delta on the native path:
    // moving right therefore subtracts yaw to turn the displayed view right.
    // Moving down looks down. This is a first-person observer convention, not
    // the AR-0021 model-orbit convention.
    look.yaw -= delta_x * 0.0032;
    look.pitch = (look.pitch - delta_y * 0.0024).clamp(-0.7, 0.7);
}

fn scene_bounds(draws: &[StaticDrawPlanEntry]) -> (Vec3, f32) {
    let mut minimum = [f32::INFINITY; 3];
    let mut maximum = [f32::NEG_INFINITY; 3];
    for draw in draws {
        for position in &draw.mesh.positions {
            for axis in 0..3 {
                minimum[axis] = minimum[axis].min(position[axis]);
                maximum[axis] = maximum[axis].max(position[axis]);
            }
        }
    }
    let center = Vec3::new(
        (minimum[0] + maximum[0]) * 0.5,
        (minimum[1] + maximum[1]) * 0.5,
        (minimum[2] + maximum[2]) * 0.5,
    );
    let radius = (maximum[0] - minimum[0])
        .max(maximum[1] - minimum[1])
        .max(maximum[2] - minimum[2])
        .max(1.0);
    (center, radius)
}

#[cfg(test)]
mod tests {
    use super::{
        apply_observer_look_delta, observer_forward, spawn_forward, spawn_heading_yaw, ObserverLook,
    };

    #[test]
    fn source_spawn_heading_maps_doom_cardinal_angles_to_world_xz() {
        let east = spawn_forward(0);
        let north = spawn_forward(90);

        assert!((east.x - 1.0).abs() < 0.000_1);
        assert!(east.z.abs() < 0.000_1);
        assert!(north.x.abs() < 0.000_1);
        assert!((north.z - 1.0).abs() < 0.000_1);
    }

    #[test]
    fn source_heading_and_observer_look_share_the_declared_right_handed_axes() {
        let source_north = spawn_forward(90);
        let yaw = spawn_heading_yaw(source_north);
        let initial = observer_forward(yaw, 0.0);
        let right_turn = observer_forward(yaw + std::f32::consts::FRAC_PI_2, 0.0);
        let upward_look = observer_forward(yaw, 0.5);

        assert!(initial.x.abs() < 0.000_1);
        assert!((initial.z - 1.0).abs() < 0.000_1);
        assert!((right_turn.x - 1.0).abs() < 0.000_1);
        assert!(right_turn.z.abs() < 0.000_1);
        assert!(upward_look.y > 0.0);
    }

    #[test]
    fn observer_look_uses_first_person_pointer_signs_and_bounded_pitch() {
        let mut look = ObserverLook {
            yaw: 0.0,
            pitch: 0.0,
            last_cursor: None,
        };

        apply_observer_look_delta(&mut look, 100.0, -100.0);
        assert!(look.yaw < 0.0);
        assert!(look.pitch > 0.0);

        apply_observer_look_delta(&mut look, 0.0, -10_000.0);
        assert_eq!(look.pitch, 0.7);
    }
}
