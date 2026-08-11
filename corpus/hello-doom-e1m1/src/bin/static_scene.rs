//! Native first-frame proof for the Slice 5B static E1M1 presentation policy.
//!
//! The WAD is read only at this corpus edge. `tokimu-render` receives ordinary
//! meshes, texture bytes, materials, and one explicit opaque 3D pipeline.

use std::{collections::BTreeSet, env, fs, io, sync::Arc, time::Instant};

use archive_provider::{ArchiveFormat, ArchiveReadLimits, ZipArchiveProvider};
use doom_geometry_provider::{
    doom_point_to_tokimu, locate_doom_point_subsector, resolve_doom_linedef_subsector_membership,
    resolve_doom_subsector_bsp_paths, resolve_doom_subsector_regions,
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
    build_static_draw_plan, build_static_texture_uploads, classify_static_draw_frustum_rejection,
    classify_static_draw_sphere_frustum_rejection, doom_heading_forward, observer_direction,
    observer_right, observer_yaw_from_forward, prepare_e1m1_flat_textures, prepare_e1m1_flats,
    prepare_e1m1_masked_middle_cutouts, prepare_e1m1_wall_textures, prepare_e1m1_walls,
    prepared_e1m1_masked_middle_texture_names, StaticDrawAabb, StaticDrawFrustumRejection,
    StaticDrawPlanEntry, StaticDrawSource, StaticDrawSphere, StaticTextureUpload,
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
use tokimu_input::{KeyCode, MouseButton};
use winit::window::CursorGrabMode;

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
    opaque_bounds: Vec<Option<StaticDrawAabb>>,
    cutout_bounds: Vec<Option<StaticDrawAabb>>,
    opaque_grid: Option<UniformGridAabbIndex>,
    cutout_grid: Option<UniformGridAabbIndex>,
    membership_selection: DoomMembershipSelectionInput,
    candidate_selection: CandidateSelection,
    frame_index: u64,
    exit_after_two_frames: bool,
    opaque_selected: Vec<bool>,
    cutout_selected: Vec<bool>,
    commands: Vec<RenderCommand>,
    window: Option<Arc<NativeWindow>>,
    mouse_captured: bool,
    pressed_keys: BTreeSet<KeyCode>,
}

struct SceneInput {
    opaque_draws: Vec<StaticDrawPlanEntry>,
    opaque_uploads: Vec<StaticTextureUpload>,
    cutout_draws: Vec<StaticDrawPlanEntry>,
    cutout_uploads: Vec<StaticTextureUpload>,
    spawn_observer: SpawnObserver,
    reject_report: DoomRejectReport,
    topology_report: DoomTopologyReport,
    membership_selection: DoomMembershipSelectionInput,
}

/// Source-only observation of Doom's `REJECT` monster-sight prefilter for the
/// source player-one sector. It is intentionally not candidate-selection or
/// render-visibility data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DoomRejectReport {
    sector_count: usize,
    byte_len: usize,
    player_sector: usize,
    forbidden_monster_sectors: usize,
}

/// Bounded source-topology observation for AR-0025. It does not identify
/// render candidates or choose a visibility traversal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DoomTopologyReport {
    linedefs: usize,
    no_subsector_membership: usize,
    one_subsector_membership: usize,
    multiple_subsector_membership: usize,
    maximum_subsector_membership: usize,
}

#[derive(Clone, Debug)]
struct DoomMembershipSelectionInput {
    subsector_bounds: Vec<Option<StaticDrawAabb>>,
    linedef_subsectors: Vec<Vec<u32>>,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CandidateSelection {
    FullSubmission,
    FrustumAabb,
    /// Fixed corpus evidence configuration, not a renderer or application
    /// selection contract. AR-0025 compares this grid with the AABB baseline.
    UniformGrid8x4x8,
    DoomMembershipUnion,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct CandidateSelectionSummary {
    candidates: usize,
    rejected: usize,
    submitted: usize,
    uncertain_bounds: usize,
    rejected_by_plane: [usize; 6],
}

/// Corpus-only aggregate of a contiguous caller-owned draw range. The range is
/// a comparative selection unit, not a renderer batch, material group, or
/// source identity.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct GroupCandidateSelectionSummary {
    groups: usize,
    rejected_groups: usize,
    submitted_groups: usize,
    submitted_draws: usize,
    uncertain_groups: usize,
}

/// Corpus-local static uniform grid for AR-0025 Stage 2. It owns neither scene
/// membership nor rendering: callers retain the ordered draw list and the grid
/// only proposes which declared bounds need exact AABB/frustum testing.
#[derive(Debug)]
struct UniformGridAabbIndex {
    bounds: StaticDrawAabb,
    dimensions: [usize; 3],
    cells: Vec<Vec<usize>>,
    uncertain_draws: Vec<usize>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct UniformGridSelectionSummary {
    cells_tested: usize,
    cells_rejected: usize,
    grid_candidates: usize,
    exact_tests: usize,
    submitted: usize,
    rejected: usize,
    uncertain_bounds: usize,
    rejected_by_plane: [usize; 6],
}

impl CandidateSelectionSummary {
    fn merge(&mut self, other: Self) {
        self.candidates += other.candidates;
        self.rejected += other.rejected;
        self.submitted += other.submitted;
        self.uncertain_bounds += other.uncertain_bounds;
        for (total, value) in self
            .rejected_by_plane
            .iter_mut()
            .zip(other.rejected_by_plane)
        {
            *total += value;
        }
    }
}

fn main() -> PlatformResult<()> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let include_cutouts = args.iter().any(|argument| argument == "--masked-cutouts");
    let spawn_observer = args.iter().any(|argument| argument == "--spawn-observer");
    let spawn_yaw_plus_90 = args
        .iter()
        .any(|argument| argument == "--spawn-yaw-plus-90");
    let frustum_aabb = args.iter().any(|argument| argument == "--frustum-aabb");
    let frustum_grid = args
        .iter()
        .any(|argument| argument == "--frustum-grid-8x4x8");
    let candidate_report = args.iter().any(|argument| argument == "--candidate-report");
    let candidate_turn_trace = args
        .iter()
        .any(|argument| argument == "--candidate-turn-trace");
    let candidate_position_trace = args
        .iter()
        .any(|argument| argument == "--candidate-position-trace");
    let candidate_pathological = args
        .iter()
        .any(|argument| argument == "--candidate-pathological-report");
    let candidate_grid_report = args
        .iter()
        .any(|argument| argument == "--candidate-grid-report");
    let candidate_temporal_report = args
        .iter()
        .any(|argument| argument == "--candidate-temporal-report");
    let doom_reject_report = args
        .iter()
        .any(|argument| argument == "--doom-reject-report");
    let doom_topology_report = args
        .iter()
        .any(|argument| argument == "--doom-topology-report");
    let doom_membership_report = args
        .iter()
        .any(|argument| argument == "--doom-membership-report");
    let doom_membership_union = args
        .iter()
        .any(|argument| argument == "--doom-membership-union");
    let flat_normal_report = args
        .iter()
        .any(|argument| argument == "--flat-normal-report");
    let measure_two_frames = args
        .iter()
        .any(|argument| argument == "--measure-two-frames");
    args.retain(|argument| argument != "--masked-cutouts");
    args.retain(|argument| argument != "--spawn-observer");
    args.retain(|argument| argument != "--spawn-yaw-plus-90");
    args.retain(|argument| argument != "--frustum-aabb");
    args.retain(|argument| argument != "--frustum-grid-8x4x8");
    args.retain(|argument| argument != "--candidate-report");
    args.retain(|argument| argument != "--candidate-turn-trace");
    args.retain(|argument| argument != "--candidate-position-trace");
    args.retain(|argument| argument != "--candidate-pathological-report");
    args.retain(|argument| argument != "--candidate-grid-report");
    args.retain(|argument| argument != "--candidate-temporal-report");
    args.retain(|argument| argument != "--doom-reject-report");
    args.retain(|argument| argument != "--doom-topology-report");
    args.retain(|argument| argument != "--doom-membership-report");
    args.retain(|argument| argument != "--doom-membership-union");
    args.retain(|argument| argument != "--flat-normal-report");
    args.retain(|argument| argument != "--measure-two-frames");
    let [package, member] = args.as_slice() else {
        return Err(
            "usage: static_scene <canonical-doom-zip> <WAD-member-name> [--masked-cutouts] [--spawn-observer] [--spawn-yaw-plus-90] [--frustum-aabb] [--frustum-grid-8x4x8] [--doom-membership-union] [--candidate-report] [--candidate-turn-trace] [--candidate-position-trace] [--candidate-pathological-report] [--candidate-grid-report] [--candidate-temporal-report] [--doom-reject-report] [--doom-topology-report] [--doom-membership-report] [--flat-normal-report] [--measure-two-frames]".into(),
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
    if candidate_report {
        report_candidate_selection(&scene, include_cutouts, center, radius);
        return Ok(());
    }
    if candidate_turn_trace {
        report_candidate_turn_trace(&scene, include_cutouts, center, radius);
        return Ok(());
    }
    if candidate_position_trace {
        report_candidate_position_trace(&scene, include_cutouts, center, radius);
        return Ok(());
    }
    if candidate_pathological {
        report_pathological_candidate_fixture();
        return Ok(());
    }
    if candidate_grid_report {
        report_uniform_grid_selection(&scene, include_cutouts, center, radius);
        return Ok(());
    }
    if candidate_temporal_report {
        report_temporal_candidate_carry(&scene, include_cutouts, center, radius);
        return Ok(());
    }
    if doom_reject_report {
        report_doom_reject(&scene.reject_report);
        return Ok(());
    }
    if doom_topology_report {
        report_doom_topology(&scene.topology_report);
        return Ok(());
    }
    if doom_membership_report {
        report_doom_membership_union(&scene, center, radius, include_cutouts);
        return Ok(());
    }
    if flat_normal_report {
        report_flat_normals(&scene.opaque_draws);
        return Ok(());
    }
    let opaque_bounds = draw_bounds(&scene.opaque_draws);
    let cutout_bounds = draw_bounds(&scene.cutout_draws);
    let opaque_grid = frustum_grid
        .then(|| UniformGridAabbIndex::build(&opaque_bounds, [8, 4, 8]))
        .flatten();
    let cutout_grid = frustum_grid
        .then(|| UniformGridAabbIndex::build(&cutout_bounds, [8, 4, 8]))
        .flatten();
    let draw_count = scene.opaque_draws.len()
        + if include_cutouts {
            scene.cutout_draws.len()
        } else {
            0
        };
    let opaque_selected = vec![true; scene.opaque_draws.len()];
    let cutout_selected = vec![true; scene.cutout_draws.len()];
    let commands = Vec::with_capacity(draw_count + 1);
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
                yaw: observer_yaw_from_forward(scene.spawn_observer.forward)
                    + if spawn_yaw_plus_90 {
                        std::f32::consts::FRAC_PI_2
                    } else {
                        0.0
                    },
                pitch: 0.0,
                last_cursor: None,
            }),
            opaque_bounds,
            cutout_bounds,
            opaque_grid,
            cutout_grid,
            membership_selection: scene.membership_selection,
            candidate_selection: if frustum_grid {
                CandidateSelection::UniformGrid8x4x8
            } else if doom_membership_union {
                CandidateSelection::DoomMembershipUnion
            } else if frustum_aabb {
                CandidateSelection::FrustumAabb
            } else {
                CandidateSelection::FullSubmission
            },
            frame_index: 0,
            exit_after_two_frames: measure_two_frames,
            opaque_selected,
            cutout_selected,
            commands,
            window: None,
            mouse_captured: false,
            pressed_keys: BTreeSet::new(),
        },
    )
}

impl App {
    fn set_mouse_captured(&mut self, captured: bool) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        if captured {
            let grabbed = window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
            if grabbed.is_ok() {
                window.set_cursor_visible(false);
                self.mouse_captured = true;
                if let Some(look) = self.observer_look.as_mut() {
                    look.last_cursor = None;
                }
            }
        } else {
            let _ = window.set_cursor_grab(CursorGrabMode::None);
            window.set_cursor_visible(true);
            self.mouse_captured = false;
            if let Some(look) = self.observer_look.as_mut() {
                look.last_cursor = None;
            }
        }
    }

    fn apply_inspection_movement(&mut self, delta_seconds: f64) {
        let Some(observer) = self.spawn_observer.as_mut() else {
            return;
        };
        let Some(look) = self.observer_look else {
            return;
        };
        let mut direction = Vec3::ZERO;
        let forward = observer_direction(look.yaw, 0.0);
        let right = observer_right(forward);
        if self.pressed_keys.contains(&KeyCode::KeyW) {
            direction += forward;
        }
        if self.pressed_keys.contains(&KeyCode::KeyS) {
            direction -= forward;
        }
        if self.pressed_keys.contains(&KeyCode::KeyD) {
            direction += right;
        }
        if self.pressed_keys.contains(&KeyCode::KeyA) {
            direction -= right;
        }
        if self.pressed_keys.contains(&KeyCode::KeyE) {
            direction += Vec3::Y;
        }
        if self.pressed_keys.contains(&KeyCode::KeyQ) {
            direction -= Vec3::Y;
        }
        if direction.length_squared() > 0.0 {
            observer.position += direction.normalize() * (480.0 * delta_seconds as f32);
        }
    }
}

impl PlatformEventHandler for App {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window.clone(), size.width, size.height)?;
        self.window = Some(window);
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
            "E1M1 native first-frame metadata: opaque_draws={}; cutout_draws={}; cutouts_enabled={}; camera={}; candidate_selection={}; backend={}; device={}; adapter={}",
            self.draws.len(),
            self.cutout_draws.len(),
            self.include_cutouts,
            if self.spawn_observer.is_some() { "source-spawn-observer" } else { "overview" },
            match self.candidate_selection {
                CandidateSelection::FullSubmission => "full-submission",
                CandidateSelection::FrustumAabb => "frustum-aabb",
                CandidateSelection::UniformGrid8x4x8 => "uniform-grid-8x4x8",
                CandidateSelection::DoomMembershipUnion => "doom-membership-union",
            },
            renderer.backend_api(),
            renderer.device_kind(),
            renderer.adapter_name(),
        );
        self.renderer = Some(renderer);
        Ok(())
    }
    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::MouseMotion { delta_x, delta_y } = event {
            if self.mouse_captured {
                if let Some(look) = self.observer_look.as_mut() {
                    apply_observer_look_delta(look, delta_x, delta_y);
                }
            }
            return Ok(());
        }
        if let PlatformInputEvent::MouseInput {
            button: MouseButton::Left,
            pressed: true,
        } = event
        {
            self.set_mouse_captured(true);
            return Ok(());
        }
        if let PlatformInputEvent::KeyboardInput { key, pressed } = event {
            if key == KeyCode::Escape && pressed {
                self.set_mouse_captured(false);
                self.pressed_keys.clear();
            } else if pressed {
                self.pressed_keys.insert(key);
            } else {
                self.pressed_keys.remove(&key);
            }
            return Ok(());
        }
        if let PlatformInputEvent::CursorMoved { x, y } = event {
            if let Some(look) = self.observer_look.as_mut() {
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

    fn on_frame(&mut self, delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        self.apply_inspection_movement(delta_seconds);
        let frame_started = Instant::now();
        let camera = scene_camera(
            self.size,
            self.center,
            self.radius,
            self.spawn_observer,
            self.observer_look,
        );
        let selection_started = Instant::now();
        let view_projection = camera.projection * camera.view;
        let mut selection = CandidateSelectionSummary::default();
        let mut rejection_samples = Vec::new();
        if self.candidate_selection == CandidateSelection::DoomMembershipUnion {
            select_membership_candidates(
                &self.draws,
                view_projection,
                &self.membership_selection,
                &mut self.opaque_selected,
                &mut selection,
                &mut rejection_samples,
                self.frame_index == 0,
            );
        } else {
            select_current_candidates(
                self.candidate_selection,
                self.opaque_grid.as_ref(),
                &self.opaque_bounds,
                &self.draws,
                view_projection,
                &mut self.opaque_selected,
                &mut selection,
                &mut rejection_samples,
                self.frame_index == 0,
            );
        }
        let opaque_submitted = selection.submitted;
        if self.include_cutouts {
            if self.candidate_selection == CandidateSelection::DoomMembershipUnion {
                select_membership_candidates(
                    &self.cutout_draws,
                    view_projection,
                    &self.membership_selection,
                    &mut self.cutout_selected,
                    &mut selection,
                    &mut rejection_samples,
                    self.frame_index == 0,
                );
            } else {
                select_current_candidates(
                    self.candidate_selection,
                    self.cutout_grid.as_ref(),
                    &self.cutout_bounds,
                    &self.cutout_draws,
                    view_projection,
                    &mut self.cutout_selected,
                    &mut selection,
                    &mut rejection_samples,
                    self.frame_index == 0,
                );
            }
        }
        let cutout_submitted = selection.submitted - opaque_submitted;
        let selection_time = selection_started.elapsed();
        let command_started = Instant::now();
        self.commands.clear();
        self.commands.push(RenderCommand::Clear(ClearCommand {
            color: Color::rgb(0.015, 0.02, 0.025),
        }));
        for (index, draw) in self.draws.iter().enumerate() {
            if !self.opaque_selected[index] {
                continue;
            }
            let mesh = MeshHandle(index as u64 + 1);
            self.commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
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
                if !self.cutout_selected[offset] {
                    continue;
                }
                let mesh = MeshHandle(self.draws.len() as u64 + offset as u64 + 1);
                self.commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh,
                    material: draw.material,
                    pipeline: cutout_pipeline,
                    instance: Instance2d::identity(),
                    camera: Some(CAMERA),
                    viewport: None,
                }));
            }
        }
        let command_time = command_started.elapsed();
        let renderer = self
            .renderer
            .as_mut()
            .ok_or_else(|| io::Error::other("renderer missing"))?;
        renderer.upload_camera(CAMERA, camera);
        renderer.begin_frame();
        renderer.submit(&self.commands);
        renderer.present()?;
        let stats = renderer.end_frame();
        if self.frame_index < 2 {
            eprintln!(
                "E1M1 AR-0025 {} frame: selection={:?}; candidates={}; rejected={}; submitted={}; opaque_submitted={}; cutout_submitted={}; uncertain_bounds={}; rejected_by_plane=[left:{},right:{},bottom:{},top:{},near:{},far:{}]; selection_cpu_us={}; command_build_cpu_us={}; frame_cpu_us={}; draws={}; material_resolutions={}; pipeline_switches={}; mesh_uploads={}; mesh_replacements={}; lifetime_mesh_uploads={}; lifetime_mesh_replacements={}",
                if self.frame_index == 0 { "first" } else { "warm" },
                self.candidate_selection,
                selection.candidates,
                selection.rejected,
                selection.submitted,
                opaque_submitted,
                cutout_submitted,
                selection.uncertain_bounds,
                selection.rejected_by_plane[0],
                selection.rejected_by_plane[1],
                selection.rejected_by_plane[2],
                selection.rejected_by_plane[3],
                selection.rejected_by_plane[4],
                selection.rejected_by_plane[5],
                selection_time.as_micros(),
                command_time.as_micros(),
                frame_started.elapsed().as_micros(),
                stats.frame.draw_calls,
                stats.frame.material_resolutions,
                stats.frame.pipeline_switches,
                stats.frame.mesh_uploads,
                stats.frame.mesh_replacements,
                stats.lifetime.mesh_uploads,
                stats.lifetime.mesh_replacements,
            );
            if self.frame_index == 0 && !rejection_samples.is_empty() {
                eprintln!(
                    "E1M1 AR-0025 bounded rejection samples ({} of {}): {}",
                    rejection_samples.len(),
                    selection.rejected,
                    rejection_samples.join(" | "),
                );
            }
        }
        self.frame_index = self.frame_index.saturating_add(1);
        Ok(if self.exit_after_two_frames && self.frame_index >= 2 {
            FrameOutcome::Exit
        } else {
            FrameOutcome::Continue
        })
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
    let regions = resolve_doom_subsector_regions(&map, &paths)?;
    let sector = ownership
        .iter()
        .find(|entry| entry.source_subsector == location.source_subsector)
        .ok_or_else(|| io::Error::other("player-one start subsector has no sector ownership"))?;
    let vertical = &map.sectors[usize::from(sector.sector_index)];
    let player_sector = usize::from(sector.sector_index);
    let forbidden_monster_sectors = (0..map.reject.sector_count())
        .filter(|monster_sector| {
            map.reject
                .forbids_monster_sight(*monster_sector, player_sector)
                .expect("bounded source sector indices remain valid")
        })
        .count();
    let reject_report = DoomRejectReport {
        sector_count: map.reject.sector_count(),
        byte_len: map.reject.observation.byte_len,
        player_sector,
        forbidden_monster_sectors,
    };
    let memberships = resolve_doom_linedef_subsector_membership(&map);
    let mut topology_report = DoomTopologyReport {
        linedefs: memberships.len(),
        no_subsector_membership: 0,
        one_subsector_membership: 0,
        multiple_subsector_membership: 0,
        maximum_subsector_membership: 0,
    };
    for membership in &memberships {
        let count = membership.source_subsectors.len();
        topology_report.maximum_subsector_membership =
            topology_report.maximum_subsector_membership.max(count);
        match count {
            0 => topology_report.no_subsector_membership += 1,
            1 => topology_report.one_subsector_membership += 1,
            _ => topology_report.multiple_subsector_membership += 1,
        }
    }
    let subsector_bounds = regions
        .iter()
        .zip(&ownership)
        .map(|(region, ownership)| {
            let sector = &map.sectors[usize::from(ownership.sector_index)];
            let mut min_x = f64::INFINITY;
            let mut min_z = f64::INFINITY;
            let mut max_x = f64::NEG_INFINITY;
            let mut max_z = f64::NEG_INFINITY;
            for [x, z] in &region.vertices {
                min_x = min_x.min(*x);
                min_z = min_z.min(*z);
                max_x = max_x.max(*x);
                max_z = max_z.max(*z);
            }
            StaticDrawAabb::from_minimum_maximum(
                Vec3::new(min_x as f32, f32::from(sector.floor_height), min_z as f32),
                Vec3::new(max_x as f32, f32::from(sector.ceiling_height), max_z as f32),
            )
        })
        .collect();
    let membership_selection = DoomMembershipSelectionInput {
        subsector_bounds,
        linedef_subsectors: memberships
            .iter()
            .map(|membership| {
                membership
                    .source_subsectors
                    .iter()
                    .map(|source| source.record_index)
                    .collect()
            })
            .collect(),
    };
    let source_eye_height =
        (f64::from(vertical.floor_height) + f64::from(vertical.ceiling_height)) * 0.5;
    let world_eye = doom_point_to_tokimu(start.position.map(f64::from), source_eye_height);
    let spawn_observer = SpawnObserver {
        position: Vec3::new(
            world_eye[0] as f32,
            world_eye[1] as f32,
            world_eye[2] as f32,
        ),
        forward: doom_heading_forward(start.angle),
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
        reject_report,
        topology_report,
        membership_selection,
    })
}

fn report_doom_reject(report: &DoomRejectReport) {
    println!(
        "E1M1 AR-0025 Doom REJECT source observation: sectors={}; bytes={}; player_sector={}; monster_sectors_forbidden_to_sight_player={}; monster_sectors_not_forbidden={}; meaning=classic-monster-sight-prefilter-not-render-visibility",
        report.sector_count,
        report.byte_len,
        report.player_sector,
        report.forbidden_monster_sectors,
        report.sector_count - report.forbidden_monster_sectors,
    );
}

fn report_doom_topology(report: &DoomTopologyReport) {
    println!(
        "E1M1 AR-0025 Doom SEGS-to-SSECTORS source observation: linedefs={}; no_subsector_membership={}; one_subsector_membership={}; multiple_subsector_membership={}; maximum_subsector_membership={}; meaning=source-topology-not-render-membership",
        report.linedefs,
        report.no_subsector_membership,
        report.one_subsector_membership,
        report.multiple_subsector_membership,
        report.maximum_subsector_membership,
    );
}

fn report_flat_normals(draws: &[StaticDrawPlanEntry]) {
    let mut floors_up = 0;
    let mut floors_down = 0;
    let mut ceilings_up = 0;
    let mut ceilings_down = 0;
    for draw in draws {
        let StaticDrawSource::Flat { plane, .. } = draw.source else {
            continue;
        };
        let normal_y = draw.mesh.normals.first().map_or(0.0, |normal| normal[1]);
        let is_floor = plane == doom_geometry_provider::DoomSurfacePlane::Floor;
        match (is_floor, normal_y.is_sign_positive()) {
            (true, true) => floors_up += 1,
            (true, false) => floors_down += 1,
            (false, true) => ceilings_up += 1,
            (false, false) => ceilings_down += 1,
        }
    }
    println!(
        "E1M1 flat-normal observation: floor_up={floors_up}; floor_down={floors_down}; ceiling_up={ceilings_up}; ceiling_down={ceilings_down}"
    );
}

fn report_doom_membership_union(
    scene: &SceneInput,
    center: Vec3,
    radius: f32,
    include_cutouts: bool,
) {
    let size = [1280.0, 800.0];
    let poses = [
        ("overview", scene_camera(size, center, radius, None, None)),
        (
            "spawn-yaw-plus-90",
            scene_camera(
                size,
                center,
                radius,
                Some(scene.spawn_observer),
                Some(ObserverLook {
                    yaw: observer_yaw_from_forward(scene.spawn_observer.forward)
                        + std::f32::consts::FRAC_PI_2,
                    pitch: 0.0,
                    last_cursor: None,
                }),
            ),
        ),
    ];
    for (name, camera) in poses {
        let view_projection = camera.projection * camera.view;
        let selection_started = Instant::now();
        let selected_subsectors = scene
            .membership_selection
            .subsector_bounds
            .iter()
            .map(|bounds| {
                bounds.is_none_or(|bounds| {
                    classify_static_draw_frustum_rejection(bounds, view_projection).is_none()
                })
            })
            .collect::<Vec<_>>();
        let draws = scene.opaque_draws.iter().chain(
            include_cutouts
                .then_some(&scene.cutout_draws)
                .into_iter()
                .flatten(),
        );
        let submitted = draws
            .filter(|draw| {
                doom_membership_draw_selected(
                    draw,
                    &selected_subsectors,
                    &scene.membership_selection.linedef_subsectors,
                )
            })
            .count();
        let selection_cpu_us = selection_started.elapsed().as_micros();
        println!(
            "E1M1 AR-0025 membership-union control: pose={name}; source_subsectors={}/{}; submitted_draws={submitted}; candidates={}; selection_cpu_us={selection_cpu_us}; meaning=conservative-source-membership-not-bsp-visibility",
            selected_subsectors.iter().filter(|selected| **selected).count(),
            selected_subsectors.len(),
            scene.opaque_draws.len() + if include_cutouts { scene.cutout_draws.len() } else { 0 },
        );
    }
}

fn doom_membership_draw_selected(
    draw: &StaticDrawPlanEntry,
    selected_subsectors: &[bool],
    linedef_subsectors: &[Vec<u32>],
) -> bool {
    match draw.source {
        StaticDrawSource::Flat {
            source_subsector, ..
        } => selected_subsectors
            .get(source_subsector.record_index as usize)
            .copied()
            .unwrap_or(true),
        StaticDrawSource::Wall { source_linedef } => linedef_subsectors
            .get(source_linedef.record_index as usize)
            .map(|subsectors| {
                subsectors.iter().any(|subsector| {
                    selected_subsectors
                        .get(*subsector as usize)
                        .copied()
                        .unwrap_or(true)
                })
            })
            .unwrap_or(true),
    }
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

fn scene_camera(
    size: [f32; 2],
    center: Vec3,
    radius: f32,
    spawn_observer: Option<SpawnObserver>,
    observer_look: Option<ObserverLook>,
) -> Camera {
    let mut camera = Camera::perspective_3d(size[0], size[1]);
    // `Camera::perspective_3d` deliberately serves small corpus fixtures
    // with a 100-unit far plane. E1M1's ordinary source coordinates span
    // thousands of units, so this consumer owns an explicit overview
    // projection rather than treating that convenience default as a
    // renderer-wide Doom policy.
    let aspect = size[0] / size[1].max(1.0);
    camera.projection = Mat4::perspective_rh_gl(
        60.0_f32.to_radians(),
        aspect,
        (radius * 0.000_1).max(0.1),
        radius * 4.0,
    );
    camera.view = if let (Some(observer), Some(look)) = (spawn_observer, observer_look) {
        Mat4::look_at_rh(
            observer.position,
            observer.position + observer_direction(look.yaw, look.pitch) * 128.0,
            Vec3::Y,
        )
    } else {
        Mat4::look_at_rh(
            center + Vec3::new(radius, radius * 0.72, radius),
            center,
            Vec3::Y,
        )
    };
    camera
}

fn draw_bounds(draws: &[StaticDrawPlanEntry]) -> Vec<Option<StaticDrawAabb>> {
    draws
        .iter()
        .map(|draw| StaticDrawAabb::from_positions(&draw.mesh.positions))
        .collect()
}

fn draw_spheres(draws: &[StaticDrawPlanEntry]) -> Vec<Option<StaticDrawSphere>> {
    draws
        .iter()
        .map(|draw| StaticDrawSphere::from_positions(&draw.mesh.positions))
        .collect()
}

fn candidate_is_selected(
    policy: CandidateSelection,
    bounds: Option<StaticDrawAabb>,
    view_projection: Mat4,
    summary: &mut CandidateSelectionSummary,
    source_label: &str,
    rejection_samples: &mut Vec<String>,
    capture_sample: bool,
) -> bool {
    summary.candidates += 1;
    let rejection = match (policy, bounds) {
        (CandidateSelection::FullSubmission, _) => None,
        (CandidateSelection::UniformGrid8x4x8, _) => {
            unreachable!("uniform-grid selection must use the grid broad-phase path")
        }
        (CandidateSelection::DoomMembershipUnion, _) => {
            unreachable!("membership selection must use source-topology evidence")
        }
        (CandidateSelection::FrustumAabb, Some(bounds)) => {
            classify_static_draw_frustum_rejection(bounds, view_projection)
        }
        (CandidateSelection::FrustumAabb, None) => {
            // Uncertain bounds fail open. This preserves correctness while
            // retaining pressure to repair the invalid candidate evidence.
            summary.uncertain_bounds += 1;
            None
        }
    };
    if let Some(rejection) = rejection {
        summary.rejected += 1;
        summary.rejected_by_plane[frustum_rejection_index(rejection)] += 1;
        if capture_sample && rejection_samples.len() < 12 {
            rejection_samples.push(format!("{source_label}:{rejection:?}"));
        }
        false
    } else {
        summary.submitted += 1;
        true
    }
}

#[allow(clippy::too_many_arguments)]
fn select_candidates(
    policy: CandidateSelection,
    bounds: &[Option<StaticDrawAabb>],
    draws: &[StaticDrawPlanEntry],
    view_projection: Mat4,
    selected: &mut [bool],
    summary: &mut CandidateSelectionSummary,
    rejection_samples: &mut Vec<String>,
    capture_samples: bool,
) {
    debug_assert_eq!(bounds.len(), draws.len());
    debug_assert_eq!(selected.len(), draws.len());
    for ((selected, bounds), draw) in selected.iter_mut().zip(bounds.iter().copied()).zip(draws) {
        *selected = candidate_is_selected(
            policy,
            bounds,
            view_projection,
            summary,
            &draw.source_label,
            rejection_samples,
            capture_samples,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn select_current_candidates(
    policy: CandidateSelection,
    grid: Option<&UniformGridAabbIndex>,
    bounds: &[Option<StaticDrawAabb>],
    draws: &[StaticDrawPlanEntry],
    view_projection: Mat4,
    selected: &mut [bool],
    summary: &mut CandidateSelectionSummary,
    rejection_samples: &mut Vec<String>,
    capture_samples: bool,
) {
    match policy {
        CandidateSelection::FullSubmission | CandidateSelection::FrustumAabb => select_candidates(
            policy,
            bounds,
            draws,
            view_projection,
            selected,
            summary,
            rejection_samples,
            capture_samples,
        ),
        CandidateSelection::UniformGrid8x4x8 => {
            let Some(grid) = grid else {
                // A grid cannot be derived when every bound is uncertain. Full
                // submission remains the explicit conservative fallback.
                selected.fill(true);
                summary.candidates += bounds.len();
                summary.submitted += bounds.len();
                summary.uncertain_bounds += bounds.len();
                return;
            };
            let (grid_selected, grid_summary) = grid.select(bounds, view_projection);
            debug_assert_eq!(grid_selected.len(), draws.len());
            selected.copy_from_slice(&grid_selected);
            summary.candidates += bounds.len();
            summary.rejected += grid_summary.rejected;
            summary.submitted += grid_summary.submitted;
            summary.uncertain_bounds += grid_summary.uncertain_bounds;
            for (total, rejected) in summary
                .rejected_by_plane
                .iter_mut()
                .zip(grid_summary.rejected_by_plane)
            {
                *total += rejected;
            }
            if capture_samples {
                for (selected, draw) in grid_selected.iter().zip(draws) {
                    if !selected && rejection_samples.len() < 12 {
                        rejection_samples
                            .push(format!("{}:uniform-grid-filtered", draw.source_label));
                    }
                }
            }
        }
        CandidateSelection::DoomMembershipUnion => {
            unreachable!("membership selection must use source-topology evidence")
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn select_membership_candidates(
    draws: &[StaticDrawPlanEntry],
    view_projection: Mat4,
    input: &DoomMembershipSelectionInput,
    selected: &mut [bool],
    summary: &mut CandidateSelectionSummary,
    rejection_samples: &mut Vec<String>,
    capture_samples: bool,
) {
    let subsectors = input
        .subsector_bounds
        .iter()
        .map(|bounds| {
            bounds.is_none_or(|bounds| {
                classify_static_draw_frustum_rejection(bounds, view_projection).is_none()
            })
        })
        .collect::<Vec<_>>();
    for (selected, draw) in selected.iter_mut().zip(draws) {
        summary.candidates += 1;
        *selected = doom_membership_draw_selected(draw, &subsectors, &input.linedef_subsectors);
        if *selected {
            summary.submitted += 1;
        } else {
            summary.rejected += 1;
            if capture_samples && rejection_samples.len() < 12 {
                rejection_samples.push(format!("{}:doom-membership-filtered", draw.source_label));
            }
        }
    }
}

const fn frustum_rejection_index(rejection: StaticDrawFrustumRejection) -> usize {
    match rejection {
        StaticDrawFrustumRejection::Left => 0,
        StaticDrawFrustumRejection::Right => 1,
        StaticDrawFrustumRejection::Bottom => 2,
        StaticDrawFrustumRejection::Top => 3,
        StaticDrawFrustumRejection::Near => 4,
        StaticDrawFrustumRejection::Far => 5,
    }
}

fn summarize_candidate_selection<'a, Bounds: Copy>(
    draws: impl Iterator<Item = (&'a StaticDrawPlanEntry, Option<Bounds>)>,
    camera: Camera,
    classify: impl Fn(Bounds, Mat4) -> Option<StaticDrawFrustumRejection>,
) -> (CandidateSelectionSummary, Vec<String>) {
    let mut summary = CandidateSelectionSummary::default();
    let mut samples = Vec::new();
    let view_projection = camera.projection * camera.view;
    for (draw, bounds) in draws {
        summary.candidates += 1;
        let rejection = match bounds {
            Some(bounds) => classify(bounds, view_projection),
            None => {
                summary.uncertain_bounds += 1;
                None
            }
        };
        if let Some(rejection) = rejection {
            summary.rejected += 1;
            summary.rejected_by_plane[frustum_rejection_index(rejection)] += 1;
            if samples.len() < 12 {
                samples.push(format!("{}:{rejection:?}", draw.source_label));
            }
        } else {
            summary.submitted += 1;
        }
    }
    (summary, samples)
}

fn summarize_grouped_aabb_selection(
    bounds: &[Option<StaticDrawAabb>],
    view_projection: Mat4,
    group_size: usize,
) -> GroupCandidateSelectionSummary {
    assert!(
        group_size > 0,
        "grouped selection requires a non-zero group size"
    );
    let mut summary = GroupCandidateSelectionSummary::default();
    for group in bounds.chunks(group_size) {
        summary.groups += 1;
        let group_bounds = if group.iter().all(Option::is_some) {
            StaticDrawAabb::enclosing_iter(group.iter().flatten().copied())
        } else {
            None
        };
        let rejected = group_bounds
            .and_then(|bounds| classify_static_draw_frustum_rejection(bounds, view_projection))
            .is_some();
        if rejected {
            summary.rejected_groups += 1;
        } else {
            summary.submitted_groups += 1;
            summary.submitted_draws += group.len();
            if group_bounds.is_none() {
                summary.uncertain_groups += 1;
            }
        }
    }
    summary
}

impl UniformGridAabbIndex {
    fn build(bounds: &[Option<StaticDrawAabb>], dimensions: [usize; 3]) -> Option<Self> {
        if dimensions.contains(&0) {
            return None;
        }
        let scene_bounds = StaticDrawAabb::enclosing_iter(bounds.iter().flatten().copied())?;
        let cell_count = dimensions
            .into_iter()
            .try_fold(1_usize, usize::checked_mul)?;
        let mut index = Self {
            bounds: scene_bounds,
            dimensions,
            cells: (0..cell_count).map(|_| Vec::new()).collect(),
            uncertain_draws: Vec::new(),
        };
        for (draw_index, bounds) in bounds.iter().copied().enumerate() {
            let Some(bounds) = bounds else {
                index.uncertain_draws.push(draw_index);
                continue;
            };
            let minimum = index.cell_coordinates(bounds.minimum());
            let maximum = index.cell_coordinates(bounds.maximum());
            for z in minimum[2]..=maximum[2] {
                for y in minimum[1]..=maximum[1] {
                    for x in minimum[0]..=maximum[0] {
                        let cell_index = index.cell_index([x, y, z]);
                        index.cells[cell_index].push(draw_index);
                    }
                }
            }
        }
        Some(index)
    }

    fn cell_coordinates(&self, point: Vec3) -> [usize; 3] {
        let minimum = self.bounds.minimum();
        let maximum = self.bounds.maximum();
        let extent = maximum - minimum;
        [
            grid_coordinate(point.x, minimum.x, extent.x, self.dimensions[0]),
            grid_coordinate(point.y, minimum.y, extent.y, self.dimensions[1]),
            grid_coordinate(point.z, minimum.z, extent.z, self.dimensions[2]),
        ]
    }

    fn cell_index(&self, coordinates: [usize; 3]) -> usize {
        (coordinates[2] * self.dimensions[1] + coordinates[1]) * self.dimensions[0] + coordinates[0]
    }

    fn cell_bounds(&self, cell_index: usize) -> StaticDrawAabb {
        let x = cell_index % self.dimensions[0];
        let y = (cell_index / self.dimensions[0]) % self.dimensions[1];
        let z = cell_index / (self.dimensions[0] * self.dimensions[1]);
        let minimum = self.bounds.minimum();
        let extent = self.bounds.maximum() - minimum;
        let cell_minimum = minimum
            + Vec3::new(
                extent.x * x as f32 / self.dimensions[0] as f32,
                extent.y * y as f32 / self.dimensions[1] as f32,
                extent.z * z as f32 / self.dimensions[2] as f32,
            );
        let cell_maximum = minimum
            + Vec3::new(
                extent.x * (x + 1) as f32 / self.dimensions[0] as f32,
                extent.y * (y + 1) as f32 / self.dimensions[1] as f32,
                extent.z * (z + 1) as f32 / self.dimensions[2] as f32,
            );
        StaticDrawAabb::from_minimum_maximum(cell_minimum, cell_maximum)
            .expect("uniform grid construction must produce finite ordered bounds")
    }

    fn select(
        &self,
        bounds: &[Option<StaticDrawAabb>],
        view_projection: Mat4,
    ) -> (Vec<bool>, UniformGridSelectionSummary) {
        let mut candidates = vec![false; bounds.len()];
        let mut summary = UniformGridSelectionSummary::default();
        for &draw_index in &self.uncertain_draws {
            candidates[draw_index] = true;
            summary.uncertain_bounds += 1;
        }
        for (cell_index, draw_indices) in self.cells.iter().enumerate() {
            // Empty cells cannot contribute candidates. Skipping them keeps this
            // corpus broad phase honest: the reported cell tests represent
            // occupied spatial buckets, rather than a resolution-dependent
            // sweep over bookkeeping with no draw membership.
            if draw_indices.is_empty() {
                continue;
            }
            summary.cells_tested += 1;
            if classify_static_draw_frustum_rejection(self.cell_bounds(cell_index), view_projection)
                .is_some()
            {
                summary.cells_rejected += 1;
                continue;
            }
            for &draw_index in draw_indices {
                candidates[draw_index] = true;
            }
        }
        summary.grid_candidates = candidates.iter().filter(|candidate| **candidate).count();
        for (candidate, bounds) in candidates.iter_mut().zip(bounds.iter().copied()) {
            if !*candidate {
                summary.rejected += 1;
                continue;
            }
            match bounds {
                Some(bounds) => {
                    summary.exact_tests += 1;
                    if let Some(rejection) =
                        classify_static_draw_frustum_rejection(bounds, view_projection)
                    {
                        *candidate = false;
                        summary.rejected += 1;
                        summary.rejected_by_plane[frustum_rejection_index(rejection)] += 1;
                    } else {
                        summary.submitted += 1;
                    }
                }
                None => summary.submitted += 1,
            }
        }
        (candidates, summary)
    }
}

fn grid_coordinate(value: f32, minimum: f32, extent: f32, dimension: usize) -> usize {
    if extent <= f32::EPSILON {
        return 0;
    }
    (((value - minimum) / extent * dimension as f32).floor() as isize)
        .clamp(0, dimension.saturating_sub(1) as isize) as usize
}

fn report_candidate_selection(
    scene: &SceneInput,
    include_cutouts: bool,
    center: Vec3,
    radius: f32,
) {
    let opaque_bounds = draw_bounds(&scene.opaque_draws);
    let cutout_bounds = draw_bounds(&scene.cutout_draws);
    let opaque_spheres = draw_spheres(&scene.opaque_draws);
    let cutout_spheres = draw_spheres(&scene.cutout_draws);
    let mut ordered_bounds = opaque_bounds.clone();
    if include_cutouts {
        ordered_bounds.extend(cutout_bounds.iter().copied());
    }
    let size = [1280.0, 800.0];
    let spawn_look = ObserverLook {
        yaw: observer_yaw_from_forward(scene.spawn_observer.forward),
        pitch: 0.0,
        last_cursor: None,
    };
    let spawn_yaw = spawn_look.yaw;
    for (pose, camera) in [
        ("overview", scene_camera(size, center, radius, None, None)),
        (
            "source-spawn-forward",
            scene_camera(
                size,
                center,
                radius,
                Some(scene.spawn_observer),
                Some(spawn_look),
            ),
        ),
        (
            "source-spawn-yaw-plus-90",
            scene_camera(
                size,
                center,
                radius,
                Some(scene.spawn_observer),
                Some(ObserverLook {
                    yaw: spawn_yaw + std::f32::consts::FRAC_PI_2,
                    ..spawn_look
                }),
            ),
        ),
        (
            "source-spawn-yaw-plus-180",
            scene_camera(
                size,
                center,
                radius,
                Some(scene.spawn_observer),
                Some(ObserverLook {
                    yaw: spawn_yaw + std::f32::consts::PI,
                    ..spawn_look
                }),
            ),
        ),
        (
            "source-spawn-yaw-minus-90",
            scene_camera(
                size,
                center,
                radius,
                Some(scene.spawn_observer),
                Some(ObserverLook {
                    yaw: spawn_yaw - std::f32::consts::FRAC_PI_2,
                    ..spawn_look
                }),
            ),
        ),
    ] {
        let opaque = scene.opaque_draws.iter().zip(opaque_bounds.iter().copied());
        let cutouts = scene.cutout_draws.iter().zip(cutout_bounds.iter().copied());
        let selection_started = Instant::now();
        let (mut summary, mut samples) =
            summarize_candidate_selection(opaque, camera, classify_static_draw_frustum_rejection);
        let opaque_submitted = summary.submitted;
        let cutout_submitted = if include_cutouts {
            let (cutout_summary, cutout_samples) = summarize_candidate_selection(
                cutouts,
                camera,
                classify_static_draw_frustum_rejection,
            );
            let submitted = cutout_summary.submitted;
            summary.merge(cutout_summary);
            let remaining = 12usize.saturating_sub(samples.len());
            samples.extend(cutout_samples.into_iter().take(remaining));
            submitted
        } else {
            0
        };
        let selection_cpu_us = selection_started.elapsed().as_micros();
        println!(
            "E1M1 AR-0025 fixed-pose report: pose={pose}; policy=frustum-aabb; candidates={}; rejected={}; submitted={}; opaque_submitted={}; cutout_submitted={}; uncertain_bounds={}; selection_cpu_us={selection_cpu_us}; rejected_by_plane=[left:{},right:{},bottom:{},top:{},near:{},far:{}]; cutouts_enabled={}",
            summary.candidates,
            summary.rejected,
            summary.submitted,
            opaque_submitted,
            cutout_submitted,
            summary.uncertain_bounds,
            summary.rejected_by_plane[0],
            summary.rejected_by_plane[1],
            summary.rejected_by_plane[2],
            summary.rejected_by_plane[3],
            summary.rejected_by_plane[4],
            summary.rejected_by_plane[5],
            include_cutouts,
        );
        println!(
            "E1M1 AR-0025 bounded rejection samples: pose={pose}; shown={}; total_rejected={}; {}",
            samples.len(),
            summary.rejected,
            samples.join(" | "),
        );

        let opaque = scene
            .opaque_draws
            .iter()
            .zip(opaque_spheres.iter().copied());
        let cutouts = scene
            .cutout_draws
            .iter()
            .zip(cutout_spheres.iter().copied());
        let selection_started = Instant::now();
        let (mut summary, mut samples) = summarize_candidate_selection(
            opaque,
            camera,
            classify_static_draw_sphere_frustum_rejection,
        );
        let opaque_submitted = summary.submitted;
        let cutout_submitted = if include_cutouts {
            let (cutout_summary, cutout_samples) = summarize_candidate_selection(
                cutouts,
                camera,
                classify_static_draw_sphere_frustum_rejection,
            );
            let submitted = cutout_summary.submitted;
            summary.merge(cutout_summary);
            let remaining = 12usize.saturating_sub(samples.len());
            samples.extend(cutout_samples.into_iter().take(remaining));
            submitted
        } else {
            0
        };
        let selection_cpu_us = selection_started.elapsed().as_micros();
        println!(
            "E1M1 AR-0025 fixed-pose report: pose={pose}; policy=frustum-sphere; candidates={}; rejected={}; submitted={}; opaque_submitted={}; cutout_submitted={}; uncertain_bounds={}; selection_cpu_us={selection_cpu_us}; rejected_by_plane=[left:{},right:{},bottom:{},top:{},near:{},far:{}]; cutouts_enabled={}",
            summary.candidates,
            summary.rejected,
            summary.submitted,
            opaque_submitted,
            cutout_submitted,
            summary.uncertain_bounds,
            summary.rejected_by_plane[0],
            summary.rejected_by_plane[1],
            summary.rejected_by_plane[2],
            summary.rejected_by_plane[3],
            summary.rejected_by_plane[4],
            summary.rejected_by_plane[5],
            include_cutouts,
        );
        println!(
            "E1M1 AR-0025 bounded rejection samples: pose={pose}; policy=frustum-sphere; shown={}; total_rejected={}; {}",
            samples.len(),
            summary.rejected,
            samples.join(" | "),
        );

        let view_projection = camera.projection * camera.view;
        for group_size in [8, 32] {
            let selection_started = Instant::now();
            let summary =
                summarize_grouped_aabb_selection(&ordered_bounds, view_projection, group_size);
            let selection_cpu_us = selection_started.elapsed().as_micros();
            println!(
                "E1M1 AR-0025 fixed-pose report: pose={pose}; policy=frustum-aabb-contiguous-group-{group_size}; candidate_draws={}; groups={}; rejected_groups={}; submitted_groups={}; submitted_draws={}; uncertain_groups={}; selection_cpu_us={selection_cpu_us}; cutouts_enabled={}",
                ordered_bounds.len(),
                summary.groups,
                summary.rejected_groups,
                summary.submitted_groups,
                summary.submitted_draws,
                summary.uncertain_groups,
                include_cutouts,
            );
        }
    }
}

fn summarize_scene_aabb_selection(
    scene: &SceneInput,
    include_cutouts: bool,
    opaque_bounds: &[Option<StaticDrawAabb>],
    cutout_bounds: &[Option<StaticDrawAabb>],
    camera: Camera,
) -> CandidateSelectionSummary {
    let (mut summary, _) = summarize_candidate_selection(
        scene.opaque_draws.iter().zip(opaque_bounds.iter().copied()),
        camera,
        classify_static_draw_frustum_rejection,
    );
    if include_cutouts {
        let (cutout_summary, _) = summarize_candidate_selection(
            scene.cutout_draws.iter().zip(cutout_bounds.iter().copied()),
            camera,
            classify_static_draw_frustum_rejection,
        );
        summary.merge(cutout_summary);
    }
    summary
}

/// Deterministic in-place 360-degree source-spawn trace. It deliberately
/// changes only camera yaw: no player movement, topology, or runtime state is
/// inferred from the report.
fn report_candidate_turn_trace(
    scene: &SceneInput,
    include_cutouts: bool,
    center: Vec3,
    radius: f32,
) {
    let size = [1280.0, 800.0];
    let opaque_bounds = draw_bounds(&scene.opaque_draws);
    let cutout_bounds = draw_bounds(&scene.cutout_draws);
    let source_yaw = observer_yaw_from_forward(scene.spawn_observer.forward);
    let mut minimum_submitted = usize::MAX;
    let mut maximum_submitted = 0_usize;
    let mut total_submitted = 0_usize;
    let mut total_selection_cpu_us = 0_u128;
    for (frame, yaw_offset_degrees) in (0..=360).step_by(45).enumerate() {
        let yaw = source_yaw + (yaw_offset_degrees as f32).to_radians();
        let camera = scene_camera(
            size,
            center,
            radius,
            Some(scene.spawn_observer),
            Some(ObserverLook {
                yaw,
                pitch: 0.0,
                last_cursor: None,
            }),
        );
        let started = Instant::now();
        let summary = summarize_scene_aabb_selection(
            scene,
            include_cutouts,
            &opaque_bounds,
            &cutout_bounds,
            camera,
        );
        let selection_cpu_us = started.elapsed().as_micros();
        minimum_submitted = minimum_submitted.min(summary.submitted);
        maximum_submitted = maximum_submitted.max(summary.submitted);
        total_submitted += summary.submitted;
        total_selection_cpu_us += selection_cpu_us;
        println!(
            "E1M1 AR-0025 turn trace: frame={frame}; yaw_offset_degrees={yaw_offset_degrees}; candidates={}; rejected={}; submitted={}; uncertain_bounds={}; selection_cpu_us={selection_cpu_us}",
            summary.candidates,
            summary.rejected,
            summary.submitted,
            summary.uncertain_bounds,
        );
    }
    println!(
        "E1M1 AR-0025 turn trace summary: frames=9; candidates_per_frame={}; submitted_min={minimum_submitted}; submitted_max={maximum_submitted}; submitted_total={total_submitted}; selection_cpu_us_total={total_selection_cpu_us}; cutouts_enabled={include_cutouts}",
        scene.opaque_draws.len() + if include_cutouts { scene.cutout_draws.len() } else { 0 },
    );
}

/// Deterministic local-coordinate offsets from the reviewed source spawn. The
/// offsets are camera-test inputs only: they do not claim collision-safe Doom
/// movement, player state advancement, or a traversable source path.
fn report_candidate_position_trace(
    scene: &SceneInput,
    include_cutouts: bool,
    center: Vec3,
    radius: f32,
) {
    let size = [1280.0, 800.0];
    let opaque_bounds = draw_bounds(&scene.opaque_draws);
    let cutout_bounds = draw_bounds(&scene.cutout_draws);
    let opaque_spheres = draw_spheres(&scene.opaque_draws);
    let cutout_spheres = draw_spheres(&scene.cutout_draws);
    let mut ordered_bounds = opaque_bounds.clone();
    if include_cutouts {
        ordered_bounds.extend(cutout_bounds.iter().copied());
    }
    let forward = scene.spawn_observer.forward;
    let mut minimum_submitted = usize::MAX;
    let mut maximum_submitted = 0_usize;
    let mut total_submitted = 0_usize;
    for (frame, forward_offset) in [-256.0_f32, -128.0, 0.0, 128.0, 256.0]
        .into_iter()
        .enumerate()
    {
        let position = scene.spawn_observer.position + forward * forward_offset;
        let mut camera = scene_camera(size, center, radius, None, None);
        camera.view = Mat4::look_at_rh(position, position + forward * 128.0, Vec3::Y);
        let started = Instant::now();
        let summary = summarize_scene_aabb_selection(
            scene,
            include_cutouts,
            &opaque_bounds,
            &cutout_bounds,
            camera,
        );
        let aabb_selection_cpu_us = started.elapsed().as_micros();
        let started = Instant::now();
        let (mut sphere_summary, _) = summarize_candidate_selection(
            scene
                .opaque_draws
                .iter()
                .zip(opaque_spheres.iter().copied()),
            camera,
            classify_static_draw_sphere_frustum_rejection,
        );
        if include_cutouts {
            let (cutout_summary, _) = summarize_candidate_selection(
                scene
                    .cutout_draws
                    .iter()
                    .zip(cutout_spheres.iter().copied()),
                camera,
                classify_static_draw_sphere_frustum_rejection,
            );
            sphere_summary.merge(cutout_summary);
        }
        let sphere_selection_cpu_us = started.elapsed().as_micros();
        let view_projection = camera.projection * camera.view;
        let started = Instant::now();
        let group_8 = summarize_grouped_aabb_selection(&ordered_bounds, view_projection, 8);
        let group_8_selection_cpu_us = started.elapsed().as_micros();
        let started = Instant::now();
        let group_32 = summarize_grouped_aabb_selection(&ordered_bounds, view_projection, 32);
        let group_32_selection_cpu_us = started.elapsed().as_micros();
        minimum_submitted = minimum_submitted.min(summary.submitted);
        maximum_submitted = maximum_submitted.max(summary.submitted);
        total_submitted += summary.submitted;
        println!(
            "E1M1 AR-0025 position trace: frame={frame}; source=player-one-local-forward-offset; forward_offset={forward_offset}; camera=({:.1},{:.1},{:.1}); candidates={}; aabb_submitted={}; aabb_rejected={}; aabb_selection_cpu_us={aabb_selection_cpu_us}; sphere_submitted={}; sphere_rejected={}; sphere_selection_cpu_us={sphere_selection_cpu_us}; group_8_submitted={}; group_8_selection_cpu_us={group_8_selection_cpu_us}; group_32_submitted={}; group_32_selection_cpu_us={group_32_selection_cpu_us}; uncertain_bounds={}",
            position.x,
            position.y,
            position.z,
            summary.candidates,
            summary.submitted,
            summary.rejected,
            sphere_summary.submitted,
            sphere_summary.rejected,
            group_8.submitted_draws,
            group_32.submitted_draws,
            summary.uncertain_bounds,
        );
    }
    println!(
        "E1M1 AR-0025 position trace summary: frames=5; candidates_per_frame={}; submitted_min={minimum_submitted}; submitted_max={maximum_submitted}; submitted_total={total_submitted}; cutouts_enabled={include_cutouts}; movement_claim=none",
        scene.opaque_draws.len() + if include_cutouts { scene.cutout_draws.len() } else { 0 },
    );
}

fn report_uniform_grid_selection(
    scene: &SceneInput,
    include_cutouts: bool,
    center: Vec3,
    radius: f32,
) {
    let mut bounds = draw_bounds(&scene.opaque_draws);
    if include_cutouts {
        bounds.extend(draw_bounds(&scene.cutout_draws));
    }
    let size = [1280.0, 800.0];
    let spawn_yaw = observer_yaw_from_forward(scene.spawn_observer.forward);
    let overview_camera = scene_camera(size, center, radius, None, None);
    let mut poses = vec![(
        "overview".to_owned(),
        overview_camera.projection * overview_camera.view,
    )];
    for yaw_offset_degrees in (0..=360).step_by(45) {
        let camera = scene_camera(
            size,
            center,
            radius,
            Some(scene.spawn_observer),
            Some(ObserverLook {
                yaw: spawn_yaw + (yaw_offset_degrees as f32).to_radians(),
                pitch: 0.0,
                last_cursor: None,
            }),
        );
        poses.push((
            format!("source-spawn-yaw-offset-{yaw_offset_degrees}"),
            camera.projection * camera.view,
        ));
    }
    for forward_offset in [-256.0_f32, -128.0, 128.0, 256.0] {
        let position =
            scene.spawn_observer.position + scene.spawn_observer.forward * forward_offset;
        let mut camera = scene_camera(size, center, radius, None, None);
        camera.view = Mat4::look_at_rh(
            position,
            position + scene.spawn_observer.forward * 128.0,
            Vec3::Y,
        );
        poses.push((
            format!("source-spawn-forward-offset-{forward_offset:+.0}"),
            camera.projection * camera.view,
        ));
    }
    for dimensions in [[4, 2, 4], [8, 4, 8], [16, 4, 16]] {
        let build_started = Instant::now();
        let Some(grid) = UniformGridAabbIndex::build(&bounds, dimensions) else {
            println!("E1M1 AR-0025 uniform grid: unavailable; reason=no-finite-bounds");
            return;
        };
        let build_cpu_us = build_started.elapsed().as_micros();
        let cell_memberships = grid.cells.iter().map(Vec::len).sum::<usize>();
        let cell_capacity = grid.cells.iter().map(Vec::capacity).sum::<usize>();
        let occupied_cells = grid.cells.iter().filter(|cell| !cell.is_empty()).count();
        let estimated_index_bytes = grid.cells.len() * std::mem::size_of::<Vec<usize>>()
            + cell_capacity * std::mem::size_of::<usize>();
        println!(
            "E1M1 AR-0025 uniform grid build: dimensions={}x{}x{}; cells={}; occupied_cells={occupied_cells}; draw_bounds={}; uncertain_draws={}; cell_memberships={cell_memberships}; cell_capacity={cell_capacity}; estimated_index_bytes={estimated_index_bytes}; build_cpu_us={build_cpu_us}",
            dimensions[0],
            dimensions[1],
            dimensions[2],
            grid.cells.len(),
            bounds.len(),
            grid.uncertain_draws.len(),
        );
        for (pose, view_projection) in &poses {
            let started = Instant::now();
            let (_survivors, summary) = grid.select(&bounds, *view_projection);
            let selection_cpu_us = started.elapsed().as_micros();
            println!(
                "E1M1 AR-0025 uniform grid: dimensions={}x{}x{}; pose={pose}; candidates={}; cells_tested={}; cells_rejected={}; grid_candidates={}; exact_tests={}; submitted={}; rejected={}; uncertain_bounds={}; selection_cpu_us={selection_cpu_us}",
                dimensions[0],
                dimensions[1],
                dimensions[2],
                bounds.len(),
                summary.cells_tested,
                summary.cells_rejected,
                summary.grid_candidates,
                summary.exact_tests,
                summary.submitted,
                summary.rejected,
                summary.uncertain_bounds,
            );
        }
    }
}

/// AR-0025 theory trial: retain temporal overlap facts without granting a
/// prior frame authority over the current one. Every row first performs the
/// fresh conservative AABB classification; a one-frame carried set is then
/// reported only to show the cost of avoiding boundary churn. It is never used
/// to skip the fresh test, so abrupt turns and declared teleports fail safely.
fn report_temporal_candidate_carry(
    scene: &SceneInput,
    include_cutouts: bool,
    center: Vec3,
    radius: f32,
) {
    let mut bounds = draw_bounds(&scene.opaque_draws);
    if include_cutouts {
        bounds.extend(draw_bounds(&scene.cutout_draws));
    }
    let size = [1280.0, 800.0];
    let source_yaw = observer_yaw_from_forward(scene.spawn_observer.forward);
    let base_camera = scene_camera(size, center, radius, None, None);
    let expanded_projection = Mat4::perspective_rh_gl(
        72.0_f32.to_radians(),
        size[0] / size[1],
        (radius * 0.000_1).max(0.1),
        radius * 4.0,
    );
    let mut poses = Vec::new();
    for (label, yaw_offset_degrees) in [
        ("smooth-yaw-0", 0.0_f32),
        ("smooth-yaw-5", 5.0),
        ("smooth-yaw-10", 10.0),
        ("abrupt-turn-190", 190.0),
    ] {
        let camera = scene_camera(
            size,
            center,
            radius,
            Some(scene.spawn_observer),
            Some(ObserverLook {
                yaw: source_yaw + yaw_offset_degrees.to_radians(),
                pitch: 0.0,
                last_cursor: None,
            }),
        );
        poses.push((label, camera.view));
    }
    let teleport_position = scene.spawn_observer.position + scene.spawn_observer.forward * 1024.0;
    let mut teleport_camera = scene_camera(size, center, radius, None, None);
    teleport_camera.view = Mat4::look_at_rh(
        teleport_position,
        teleport_position + scene.spawn_observer.forward * 128.0,
        Vec3::Y,
    );
    poses.push(("declared-teleport-forward-1024", teleport_camera.view));

    let mut prior = None::<Vec<bool>>;
    let mut prior_expanded = None::<Vec<bool>>;
    for (frame, (label, view)) in poses.into_iter().enumerate() {
        let view_projection = base_camera.projection * view;
        let fresh_started = Instant::now();
        let fresh = bounds
            .iter()
            .copied()
            .map(|bounds| {
                bounds.is_none_or(|bounds| {
                    classify_static_draw_frustum_rejection(bounds, view_projection).is_none()
                })
            })
            .collect::<Vec<_>>();
        let fresh_cpu_us = fresh_started.elapsed().as_micros();
        let fresh_submitted = fresh.iter().filter(|selected| **selected).count();
        let expanded = bounds
            .iter()
            .copied()
            .map(|bounds| {
                bounds.is_none_or(|bounds| {
                    classify_static_draw_frustum_rejection(bounds, expanded_projection * view)
                        .is_none()
                })
            })
            .collect::<Vec<_>>();
        let expanded_submitted = expanded.iter().filter(|selected| **selected).count();
        let expanded_contains_fresh = fresh
            .iter()
            .zip(&expanded)
            .all(|(fresh, expanded)| !fresh || *expanded);
        assert!(
            expanded_contains_fresh,
            "expanded-frustum corpus trial must retain every fresh candidate"
        );
        let (prior_submitted, overlap, newly_visible, no_longer_visible, carried_submitted) =
            if let Some(prior) = &prior {
                let prior_submitted = prior.iter().filter(|selected| **selected).count();
                let overlap = prior
                    .iter()
                    .zip(&fresh)
                    .filter(|(prior, fresh)| **prior && **fresh)
                    .count();
                let newly_visible = fresh
                    .iter()
                    .zip(prior)
                    .filter(|(fresh, prior)| **fresh && !**prior)
                    .count();
                let no_longer_visible = prior
                    .iter()
                    .zip(&fresh)
                    .filter(|(prior, fresh)| **prior && !**fresh)
                    .count();
                let carried_submitted = prior
                    .iter()
                    .zip(&fresh)
                    .filter(|(prior, fresh)| **prior || **fresh)
                    .count();
                (
                    prior_submitted,
                    overlap,
                    newly_visible,
                    no_longer_visible,
                    carried_submitted,
                )
            } else {
                (0, 0, fresh_submitted, 0, fresh_submitted)
            };
        let expanded_prior_overlap = prior_expanded.as_ref().map_or(0, |prior_expanded| {
            prior_expanded
                .iter()
                .zip(&expanded)
                .filter(|(prior, expanded)| **prior && **expanded)
                .count()
        });
        println!(
            "E1M1 AR-0025 temporal carry: frame={frame}; pose={label}; candidates={}; fresh_submitted={fresh_submitted}; expanded_submitted={expanded_submitted}; expanded_contains_fresh={expanded_contains_fresh}; expanded_prior_overlap={expanded_prior_overlap}; prior_submitted={prior_submitted}; overlap={overlap}; newly_visible={newly_visible}; no_longer_visible={no_longer_visible}; carried_submitted={carried_submitted}; fresh_aabb_cpu_us={fresh_cpu_us}; authoritative_fresh_classification=true; abrupt_or_teleport_fallback=true; cutouts_enabled={include_cutouts}",
            bounds.len(),
        );
        prior = Some(fresh);
        prior_expanded = Some(expanded);
    }
}

/// A small source-neutral fixture of interleaved off-frustum, crossing, and
/// overlapping bounds. Interleaving intentionally stresses aggregate bounds:
/// coarse contiguous groups must fail open even though many individual draws
/// are safely rejectable.
fn report_pathological_candidate_fixture() {
    let mut bounds = Vec::with_capacity(128);
    for index in 0..32 {
        let offset = index as f32 * 0.001;
        bounds.extend([
            fixture_bounds([-4.0, -0.5 + offset, -0.5], [-3.0, 0.5 + offset, 0.5]),
            fixture_bounds([-0.5, -0.5 + offset, 3.0], [0.5, 0.5 + offset, 4.0]),
            fixture_bounds([-2.0, -0.25 + offset, -0.25], [0.25, 0.25 + offset, 0.25]),
            fixture_bounds([-0.25, -0.25 + offset, -0.25], [0.25, 0.25 + offset, 0.25]),
        ]);
    }
    let per_draw = bounds
        .iter()
        .copied()
        .filter(|bounds| classify_static_draw_frustum_rejection(*bounds, Mat4::IDENTITY).is_none())
        .count();
    println!(
        "AR-0025 pathological fixture: policy=per-draw-aabb; candidates={}; submitted={per_draw}; rejected={}",
        bounds.len(),
        bounds.len() - per_draw,
    );
    let optional_bounds = bounds.iter().copied().map(Some).collect::<Vec<_>>();
    for group_size in [8, 32] {
        let started = Instant::now();
        let summary =
            summarize_grouped_aabb_selection(&optional_bounds, Mat4::IDENTITY, group_size);
        println!(
            "AR-0025 pathological fixture: policy=contiguous-group-{group_size}; candidate_draws={}; groups={}; rejected_groups={}; submitted_draws={}; selection_cpu_us={}",
            bounds.len(),
            summary.groups,
            summary.rejected_groups,
            summary.submitted_draws,
            started.elapsed().as_micros(),
        );
    }
}

fn fixture_bounds(minimum: [f32; 3], maximum: [f32; 3]) -> StaticDrawAabb {
    StaticDrawAabb::from_positions(&[minimum, maximum])
        .expect("pathological fixture bounds must be finite")
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
        apply_observer_look_delta, candidate_is_selected, summarize_grouped_aabb_selection,
        CandidateSelection, CandidateSelectionSummary, ObserverLook, UniformGridAabbIndex,
    };
    use doom_geometry_provider::doom_point_to_tokimu;
    use hello_doom_e1m1::{
        classify_static_draw_frustum_rejection, classify_static_draw_sphere_frustum_rejection,
        doom_heading_degrees_to_observer_yaw, doom_heading_forward, observer_direction,
        observer_right, observer_yaw_from_forward, observer_yaw_to_doom_heading_degrees,
        StaticDrawAabb, StaticDrawFrustumRejection, StaticDrawSphere,
    };
    use tokimu_core::math::{Mat4, Vec3};

    #[test]
    fn source_spawn_heading_maps_doom_cardinal_angles_to_world_xz() {
        let east = doom_heading_forward(0);
        let north = doom_heading_forward(90);

        assert!((east.x - 1.0).abs() < 0.000_1);
        assert!(east.z.abs() < 0.000_1);
        assert!(north.x.abs() < 0.000_1);
        assert!((north.z - 1.0).abs() < 0.000_1);
    }

    #[test]
    fn source_orientation_round_trips_through_observer_yaw() {
        for source_degrees in [0.0, 45.0, 90.0, 180.0, 270.0, 359.0] {
            let yaw = doom_heading_degrees_to_observer_yaw(source_degrees);
            let round_trip = observer_yaw_to_doom_heading_degrees(yaw);
            assert!(
                (round_trip - source_degrees).abs() < 0.000_1,
                "source={source_degrees} yaw={yaw} round_trip={round_trip}"
            );
        }
    }

    #[test]
    fn source_heading_and_observer_look_share_the_declared_right_handed_axes() {
        let source_north = doom_heading_forward(90);
        let yaw = observer_yaw_from_forward(source_north);
        let initial = observer_direction(yaw, 0.0);
        let positive_yaw = observer_direction(yaw + std::f32::consts::FRAC_PI_2, 0.0);
        let screen_right = observer_right(initial);
        let screen_right_turn = observer_direction(yaw - std::f32::consts::FRAC_PI_2, 0.0);
        let upward_look = observer_direction(yaw, 0.5);

        assert!(initial.x.abs() < 0.000_1);
        assert!((initial.z - 1.0).abs() < 0.000_1);
        assert!((positive_yaw.x - 1.0).abs() < 0.000_1);
        assert!(positive_yaw.z.abs() < 0.000_1);
        assert!((screen_right.x + 1.0).abs() < 0.000_1);
        assert!(screen_right.z.abs() < 0.000_1);
        assert!(screen_right_turn.dot(screen_right) > 0.999_9);
        assert!(upward_look.y > 0.0);
    }

    #[test]
    fn source_spawn_command_replay_preserves_converted_forward_strafe_and_yaw() {
        let source_spawn = doom_point_to_tokimu([1056.0, -3616.0], 36.0);
        let position = Vec3::new(
            source_spawn[0] as f32,
            source_spawn[1] as f32,
            source_spawn[2] as f32,
        );
        let source_yaw = doom_heading_degrees_to_observer_yaw(90.0);
        let forward = observer_direction(source_yaw, 0.0);
        let right = observer_right(forward);

        assert_eq!(position, Vec3::new(1056.0, 36.0, -3616.0));
        assert!((forward - Vec3::Z).length() < 0.000_1);
        assert!((right + Vec3::X).length() < 0.000_1);
        assert_eq!(position + forward * 16.0, Vec3::new(1056.0, 36.0, -3600.0));
        assert_eq!(position + right * 16.0, Vec3::new(1040.0, 36.0, -3616.0));

        let screen_right_yaw = source_yaw - std::f32::consts::FRAC_PI_2;
        assert!(observer_direction(screen_right_yaw, 0.0).dot(right) > 0.999_9);
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
        assert!(observer_direction(look.yaw, 0.0).dot(observer_right(Vec3::Z)) > 0.0);

        apply_observer_look_delta(&mut look, 0.0, -10_000.0);
        assert_eq!(look.pitch, 0.7);
    }

    #[test]
    fn frustum_aabb_rejects_only_bounds_wholly_outside_one_clip_plane() {
        let inside = bounds([-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]);
        let outside_left = bounds([-3.0, -0.5, -0.5], [-2.0, 0.5, 0.5]);
        let crossing_left = bounds([-2.0, -0.5, -0.5], [0.0, 0.5, 0.5]);

        assert_eq!(
            classify_static_draw_frustum_rejection(inside, Mat4::IDENTITY),
            None
        );
        assert_eq!(
            classify_static_draw_frustum_rejection(outside_left, Mat4::IDENTITY),
            Some(StaticDrawFrustumRejection::Left)
        );
        assert_eq!(
            classify_static_draw_frustum_rejection(crossing_left, Mat4::IDENTITY),
            None
        );
    }

    #[test]
    fn frustum_selection_fails_open_for_uncertain_bounds() {
        let mut summary = CandidateSelectionSummary::default();
        let mut samples = Vec::new();

        assert!(candidate_is_selected(
            CandidateSelection::FrustumAabb,
            None,
            Mat4::IDENTITY,
            &mut summary,
            "uncertain",
            &mut samples,
            true,
        ));
        assert_eq!(summary.candidates, 1);
        assert_eq!(summary.submitted, 1);
        assert_eq!(summary.uncertain_bounds, 1);
        assert!(samples.is_empty());
    }

    #[test]
    fn frustum_sphere_rejects_only_a_sphere_wholly_outside_one_clip_plane() {
        let inside = sphere([-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]);
        let outside_right = sphere([2.0, -0.5, -0.5], [3.0, 0.5, 0.5]);
        let crossing_right = sphere([0.5, -0.5, -0.5], [1.5, 0.5, 0.5]);

        assert_eq!(
            classify_static_draw_sphere_frustum_rejection(inside, Mat4::IDENTITY),
            None
        );
        assert_eq!(
            classify_static_draw_sphere_frustum_rejection(outside_right, Mat4::IDENTITY),
            Some(StaticDrawFrustumRejection::Right)
        );
        assert_eq!(
            classify_static_draw_sphere_frustum_rejection(crossing_right, Mat4::IDENTITY),
            None
        );
    }

    #[test]
    fn frustum_selection_preserves_survivor_order() {
        let bounds = [
            bounds([-0.5, -0.5, -0.5], [0.5, 0.5, 0.5]),
            bounds([2.0, -0.5, -0.5], [3.0, 0.5, 0.5]),
            bounds([-0.25, -0.25, -0.25], [0.25, 0.25, 0.25]),
        ];
        let labels = ["A", "B", "C"];
        let mut summary = CandidateSelectionSummary::default();
        let mut samples = Vec::new();
        let survivors = bounds
            .iter()
            .copied()
            .zip(labels)
            .filter_map(|(bounds, label)| {
                candidate_is_selected(
                    CandidateSelection::FrustumAabb,
                    Some(bounds),
                    Mat4::IDENTITY,
                    &mut summary,
                    label,
                    &mut samples,
                    true,
                )
                .then_some(label)
            })
            .collect::<Vec<_>>();

        assert_eq!(survivors, ["A", "C"]);
        assert_eq!(summary.candidates, 3);
        assert_eq!(summary.rejected, 1);
        assert_eq!(summary.submitted, 2);
    }

    #[test]
    fn grouped_selection_fails_open_for_crossing_or_uncertain_members() {
        let bounds = [
            Some(bounds([-0.5, -0.5, -0.5], [0.5, 0.5, 0.5])),
            Some(bounds([2.0, -0.5, -0.5], [3.0, 0.5, 0.5])),
            None,
        ];

        let groups = summarize_grouped_aabb_selection(&bounds, Mat4::IDENTITY, 2);
        assert_eq!(groups.groups, 2);
        assert_eq!(groups.rejected_groups, 0);
        assert_eq!(groups.submitted_draws, 3);
        assert_eq!(groups.uncertain_groups, 1);
    }

    #[test]
    fn uniform_grid_preserves_the_per_draw_conservative_survivors() {
        let bounds = [
            Some(bounds([-0.5, -0.5, -0.5], [0.5, 0.5, 0.5])),
            Some(bounds([-3.0, -0.5, -0.5], [-2.0, 0.5, 0.5])),
            Some(bounds([-2.0, -0.5, -0.5], [0.25, 0.5, 0.5])),
            None,
        ];
        let index =
            UniformGridAabbIndex::build(&bounds, [2, 1, 1]).expect("fixture has finite bounds");
        let (survivors, summary) = index.select(&bounds, Mat4::IDENTITY);

        assert_eq!(survivors, [true, false, true, true]);
        assert_eq!(summary.submitted, 3);
        assert_eq!(summary.rejected, 1);
        assert_eq!(summary.uncertain_bounds, 1);
    }

    fn bounds(minimum: [f32; 3], maximum: [f32; 3]) -> StaticDrawAabb {
        StaticDrawAabb::from_positions(&[minimum, maximum]).expect("finite test bounds")
    }

    fn sphere(minimum: [f32; 3], maximum: [f32; 3]) -> StaticDrawSphere {
        StaticDrawSphere::from_positions(&[minimum, maximum]).expect("finite test sphere")
    }
}
