//! Native first-frame proof for the Slice 5B static E1M1 presentation policy.
//!
//! The WAD is read only at this corpus edge. `tokimu-render` receives ordinary
//! meshes, texture bytes, materials, and one explicit opaque 3D pipeline.

use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    env, fs, io,
    sync::Arc,
    time::Instant,
};

use archive_provider::{ArchiveFormat, ArchiveReadLimits, ZipArchiveProvider};
use doom_geometry_provider::{
    doom_point_to_tokimu, locate_doom_point_subsector, lower_doom_textured_wall_triangles,
    resolve_doom_linedef_subsector_membership, resolve_doom_subsector_bsp_paths,
    resolve_doom_subsector_regions, resolve_doom_subsector_sector_ownership, DoomTextureExtent,
};
use doom_map_provider::{decode_doom_map_core, resolve_doom_player_one_start, DoomMapCore};
use doom_raster_provider::{
    DoomFlatDecodeLimits, DoomPatchDecodeLimits, DoomRasterDecodeLimits, DoomTextureComposeLimits,
    DoomTextureDecodeLimits,
};
use doom_wad_package::{
    read_wad_package_member, select_doom_episode_map, InspectWadPackageRequest,
};
use doom_wad_provider::WadReadLimits;
use hello_doom_e1m1::collision::{
    DoomWalkCollisionWorld, DoomWalkFloorResolution, DoomWalkFloorWorld,
};
use hello_doom_e1m1::debug_console::DoomDebugConsole;
use hello_doom_e1m1::specials::{
    resolve_doom_line_activation, DoomLineActivation, DoomLineActivationIntent,
    DoomLineActivationRequest, DoomLineActivationResolution, DoomLineActivationSource,
    DoomManualDoorPhase, DoomManualDoorPolicy, DoomManualDoorRuntime,
};
use hello_doom_e1m1::{
    build_experimental_cutout_draw_plan, build_experimental_cutout_texture_uploads,
    build_static_draw_plan, build_static_texture_uploads, classify_static_draw_frustum_rejection,
    classify_static_draw_sphere_frustum_rejection, doom_heading_forward,
    lower_static_wall_triangle, observe_doom_ground_frame_with_embedding, observer_direction,
    observer_right, observer_yaw_from_forward, prepare_e1m1_flat_textures, prepare_e1m1_flats,
    prepare_e1m1_masked_middle_cutouts, prepare_e1m1_sky_diagnostic_flats,
    prepare_e1m1_wall_texture_extents, prepare_e1m1_wall_textures, prepare_e1m1_walls,
    prepared_e1m1_masked_middle_texture_names, reembed_comparative_mesh, DoomComparativeEmbedding,
    StaticDrawAabb, StaticDrawFrustumRejection, StaticDrawPlanEntry, StaticDrawSource,
    StaticDrawSphere, StaticFlatLoweringError, StaticTextureUpload,
};
use raster_image_corpus::{decode_png, prepare_renderer_texture, DecodeLimits, TextureUse};
use resource_space::{
    AddressCasePolicy, FolderId, InMemoryResourceSpace, ResourceMetadata, ResourceName,
    ResourceRootDescriptor, ResourceRootId, StoreId,
};
use resource_space_archive::InspectArchiveResourceRequest;
use tokimu::{
    run_window_with_app, BlendMode, Camera, CameraHandle, CategoricalCutout, ClearCommand, Color,
    ColorWriteMask, CullMode, CutoutComparison, CutoutThreshold, DepthTest, DrawMeshCommand,
    FrameOutcome, Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PipelineRenderState, PlatformEventHandler, PlatformInputEvent,
    PlatformResult, RenderCommand, Renderer, Texture, TextureAddressMode, TextureFilter,
    TextureHandle, TextureSampler, WgpuBackend, WindowConfig,
};
use tokimu_core::math::{Mat4, Vec3};
use tokimu_input::{InputState, KeyCode, MouseButton};
use ui_tools::provider::{UiFontRasterizer, UiFontSource};
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
const DEBUG_CAMERA: CameraHandle = CameraHandle(2);
const DEBUG_QUAD: MeshHandle = MeshHandle(9_000_001);
const DEBUG_TEXTURE: TextureHandle = TextureHandle(9_000_001);
const DEBUG_MATERIAL: MaterialHandle = MaterialHandle(9_000_001);
const DEBUG_CURSOR_MATERIAL: MaterialHandle = MaterialHandle(9_000_002);
const DIAGNOSTIC_SKY_TEXTURE: TextureHandle = TextureHandle(9_000_010);
const DIAGNOSTIC_SKY_MATERIAL: MaterialHandle = MaterialHandle(9_000_010);
const DIAGNOSTIC_SKY_MESH_BASE: u64 = 9_000_100;
const WALK_SPEED: f32 = 240.0;
const WALK_RADIUS: f32 = 16.0;
const DOOM_TIC_SECONDS: f64 = 1.0 / 35.0;

struct App {
    renderer: Option<WgpuBackend>,
    draws: Vec<StaticDrawPlanEntry>,
    uploads: Vec<StaticTextureUpload>,
    cutout_draws: Vec<StaticDrawPlanEntry>,
    cutout_uploads: Vec<StaticTextureUpload>,
    diagnostic_sky_draws: Vec<StaticDrawPlanEntry>,
    diagnostic_sky_enabled: bool,
    diagnostic_sky_records: Vec<String>,
    cutout_mesh_base: u64,
    include_cutouts: bool,
    pipeline: PipelineHandle,
    cutout_pipeline: Option<PipelineHandle>,
    debug_pipeline: Option<PipelineHandle>,
    debug_font: Option<UiFontRasterizer>,
    debug_console: DoomDebugConsole,
    size: [f32; 2],
    center: Vec3,
    radius: f32,
    spawn_observer: Option<SpawnObserver>,
    initial_spawn_observer: Option<SpawnObserver>,
    observer_look: Option<ObserverLook>,
    initial_observer_look: Option<ObserverLook>,
    walk_collision: Option<DoomWalkCollisionWorld>,
    walk_floors: Option<DoomWalkFloorWorld>,
    noclip: bool,
    last_collision_contacts: Vec<u32>,
    last_floor_transition: Option<String>,
    opaque_bounds: Vec<Option<StaticDrawAabb>>,
    cutout_bounds: Vec<Option<StaticDrawAabb>>,
    opaque_grid: Option<UniformGridAabbIndex>,
    cutout_grid: Option<UniformGridAabbIndex>,
    membership_selection: DoomMembershipSelectionInput,
    activation_source: DoomLineActivationSource,
    door_geometry_source: DoomDynamicDoorGeometrySource,
    active_manual_doors: Vec<DoomManualDoorRuntime>,
    door_tick_accumulator: f64,
    dirty_opaque_meshes: HashSet<usize>,
    door_visual_diagnostic: Option<String>,
    door_geometry_diagnostic: Option<String>,
    dynamic_door_draws: BTreeSet<usize>,
    dynamic_door_mesh_handles: BTreeMap<usize, MeshHandle>,
    next_dynamic_mesh_handle: u64,
    opaque_draw_enabled: Vec<bool>,
    candidate_selection: CandidateSelection,
    frame_index: u64,
    exit_after_two_frames: bool,
    opaque_selected: Vec<bool>,
    cutout_selected: Vec<bool>,
    commands: Vec<RenderCommand>,
    window: Option<Arc<NativeWindow>>,
    mouse_captured: bool,
    input: InputState,
    comparative_embedding: DoomComparativeEmbedding,
}

struct SceneInput {
    opaque_draws: Vec<StaticDrawPlanEntry>,
    opaque_uploads: Vec<StaticTextureUpload>,
    cutout_draws: Vec<StaticDrawPlanEntry>,
    cutout_uploads: Vec<StaticTextureUpload>,
    diagnostic_sky_draws: Vec<StaticDrawPlanEntry>,
    diagnostic_sky_records: Vec<String>,
    spawn_observer: SpawnObserver,
    walk_collision: DoomWalkCollisionWorld,
    walk_floors: DoomWalkFloorWorld,
    reject_report: DoomRejectReport,
    topology_report: DoomTopologyReport,
    membership_selection: DoomMembershipSelectionInput,
    activation_source: DoomLineActivationSource,
    door_geometry_source: DoomDynamicDoorGeometrySource,
}

/// Immutable source geometry retained only so Slice 8 can re-lower the
/// affected Doom wall spans from runtime-owned sector heights. It does not
/// become renderer or generic world state.
#[derive(Clone, Debug)]
struct DoomDynamicDoorGeometrySource {
    map: DoomMapCore,
    wall_extents: Vec<DoomTextureExtent>,
    wall_materials: BTreeMap<String, MaterialHandle>,
}

#[derive(Clone, Debug)]
struct DynamicDoorWallMesh {
    mesh: Mesh,
    source_linedef: doom_map_provider::DoomSourceRecord,
    source_sidedef: doom_map_provider::DoomSourceRecord,
    source_sector: doom_map_provider::DoomSourceRecord,
    role: doom_geometry_provider::DoomWallTextureRole,
    texture_name: String,
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
    let preserve_east = args.iter().any(|argument| argument == "--embedding-east");
    let preserve_north = args.iter().any(|argument| argument == "--embedding-north");
    let comparative_embedding = match (preserve_east, preserve_north) {
        (false, false) => DoomComparativeEmbedding::CurrentReflected,
        (true, false) => DoomComparativeEmbedding::PreserveEast,
        (false, true) => DoomComparativeEmbedding::PreserveNorth,
        (true, true) => return Err("choose only one comparative embedding".into()),
    };
    let include_cutouts = args.iter().any(|argument| argument == "--masked-cutouts");
    let diagnostic_sky = args
        .iter()
        .any(|argument| argument == "--diagnostic-sky-omissions");
    let spawn_observer = args.iter().any(|argument| argument == "--spawn-observer");
    let spawn_yaw_plus_90 = args
        .iter()
        .any(|argument| argument == "--spawn-yaw-plus-90");
    let walk_collision = args.iter().any(|argument| argument == "--walk-collision");
    let walk_collision_report = args
        .iter()
        .any(|argument| argument == "--walk-collision-report");
    let noclip = args.iter().any(|argument| argument == "--noclip");
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
    let special_activation_report = args
        .iter()
        .any(|argument| argument == "--special-activation-report");
    let door_runtime_report = args
        .iter()
        .any(|argument| argument == "--door-runtime-report");
    let door_resource_replay_report = args
        .iter()
        .any(|argument| argument == "--door-resource-replay-report");
    let measure_two_frames = args
        .iter()
        .any(|argument| argument == "--measure-two-frames");
    let spatial_orientation_report = args
        .iter()
        .any(|argument| argument == "--spatial-orientation-report");
    let spatial_landmark_candidates_report = args
        .iter()
        .any(|argument| argument == "--spatial-landmark-candidates-report");
    let spatial_flat_uv_report = args
        .iter()
        .any(|argument| argument == "--spatial-flat-uv-report");
    args.retain(|argument| argument != "--masked-cutouts");
    args.retain(|argument| argument != "--diagnostic-sky-omissions");
    args.retain(|argument| argument != "--spawn-observer");
    args.retain(|argument| argument != "--spawn-yaw-plus-90");
    args.retain(|argument| argument != "--walk-collision");
    args.retain(|argument| argument != "--walk-collision-report");
    args.retain(|argument| argument != "--noclip");
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
    args.retain(|argument| argument != "--special-activation-report");
    args.retain(|argument| argument != "--door-runtime-report");
    args.retain(|argument| argument != "--door-resource-replay-report");
    args.retain(|argument| argument != "--measure-two-frames");
    args.retain(|argument| argument != "--spatial-orientation-report");
    args.retain(|argument| argument != "--spatial-landmark-candidates-report");
    args.retain(|argument| argument != "--spatial-flat-uv-report");
    args.retain(|argument| argument != "--embedding-east");
    args.retain(|argument| argument != "--embedding-north");
    let [package, member] = args.as_slice() else {
        return Err(
            "usage: static_scene <canonical-doom-zip> <WAD-member-name> [--masked-cutouts] [--diagnostic-sky-omissions] [--spawn-observer] [--spawn-yaw-plus-90] [--embedding-east|--embedding-north] [--walk-collision] [--walk-collision-report] [--noclip] [--frustum-aabb] [--frustum-grid-8x4x8] [--doom-membership-union] [--candidate-report] [--candidate-turn-trace] [--candidate-position-trace] [--candidate-pathological-report] [--candidate-grid-report] [--candidate-temporal-report] [--doom-reject-report] [--doom-topology-report] [--doom-membership-report] [--flat-normal-report] [--special-activation-report] [--door-runtime-report] [--door-resource-replay-report] [--spatial-orientation-report] [--spatial-landmark-candidates-report] [--spatial-flat-uv-report] [--measure-two-frames]".into(),
        );
    };
    if (walk_collision || walk_collision_report) && !spawn_observer {
        return Err("--walk-collision requires --spawn-observer".into());
    }
    if comparative_embedding != DoomComparativeEmbedding::CurrentReflected && walk_collision_report
    {
        return Err(
            "comparative embeddings currently exclude the source-space collision report replay; use interactive --walk-collision for converted correspondence evidence"
                .into(),
        );
    }
    let mut scene = prepare_scene(package, member)?;
    if spatial_orientation_report {
        report_spatial_orientation(&scene);
        return Ok(());
    }
    if spatial_landmark_candidates_report {
        report_spatial_landmark_candidates(&scene);
        return Ok(());
    }
    reembed_scene_for_comparison(&mut scene, comparative_embedding);
    if spatial_flat_uv_report {
        report_spatial_flat_uv(&scene, comparative_embedding);
        return Ok(());
    }
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
    if special_activation_report {
        report_doom_use_activation(&scene.activation_source);
        return Ok(());
    }
    if door_runtime_report {
        report_doom_manual_door_runtime(&scene.activation_source);
        return Ok(());
    }
    if walk_collision_report {
        report_walk_collision(&scene);
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
        }
        + if diagnostic_sky {
            scene.diagnostic_sky_draws.len()
        } else {
            0
        };
    let opaque_selected = vec![true; scene.opaque_draws.len()];
    let cutout_selected = vec![true; scene.cutout_draws.len()];
    let cutout_mesh_base = scene.opaque_draws.len() as u64 + 1;
    let commands = Vec::with_capacity(draw_count + 1);
    let mut app = App {
        renderer: None,
        draws: scene.opaque_draws,
        uploads: scene.opaque_uploads,
        cutout_draws: scene.cutout_draws,
        cutout_uploads: scene.cutout_uploads,
        diagnostic_sky_draws: scene.diagnostic_sky_draws,
        diagnostic_sky_enabled: diagnostic_sky,
        diagnostic_sky_records: scene.diagnostic_sky_records,
        cutout_mesh_base,
        include_cutouts,
        pipeline: PipelineHandle(0),
        cutout_pipeline: None,
        debug_pipeline: None,
        debug_font: None,
        debug_console: DoomDebugConsole::default(),
        size: [1280.0, 800.0],
        center,
        radius,
        spawn_observer: spawn_observer.then_some(scene.spawn_observer),
        initial_spawn_observer: spawn_observer.then_some(scene.spawn_observer),
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
        initial_observer_look: spawn_observer.then_some(ObserverLook {
            yaw: observer_yaw_from_forward(scene.spawn_observer.forward)
                + if spawn_yaw_plus_90 {
                    std::f32::consts::FRAC_PI_2
                } else {
                    0.0
                },
            pitch: 0.0,
            last_cursor: None,
        }),
        walk_collision: walk_collision.then_some(scene.walk_collision),
        walk_floors: walk_collision.then_some(scene.walk_floors),
        noclip,
        last_collision_contacts: Vec::new(),
        last_floor_transition: None,
        opaque_bounds,
        cutout_bounds,
        opaque_grid,
        cutout_grid,
        membership_selection: scene.membership_selection,
        activation_source: scene.activation_source,
        door_geometry_source: scene.door_geometry_source,
        active_manual_doors: Vec::new(),
        door_tick_accumulator: 0.0,
        dirty_opaque_meshes: HashSet::new(),
        door_visual_diagnostic: None,
        door_geometry_diagnostic: None,
        dynamic_door_draws: BTreeSet::new(),
        dynamic_door_mesh_handles: BTreeMap::new(),
        next_dynamic_mesh_handle: cutout_mesh_base + cutout_selected.len() as u64,
        opaque_draw_enabled: opaque_selected.clone(),
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
        input: InputState::default(),
        comparative_embedding,
    };
    if door_resource_replay_report {
        report_door_resource_replay(&mut app)?;
        return Ok(());
    }
    run_window_with_app(
        WindowConfig {
            title: format!("Tokimu DOOM E1M1 | {draw_count} draws | {comparative_embedding:?}"),
            width: 1280,
            height: 800,
        },
        app,
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
        if self.debug_console.is_open() {
            return;
        }
        let Some(observer) = self.spawn_observer else {
            return;
        };
        let Some(look) = self.observer_look else {
            return;
        };
        let mut direction = Vec3::ZERO;
        let forward = observer_direction(look.yaw, 0.0);
        let right = observer_right(forward);
        if self.input.keyboard.is_pressed(KeyCode::KeyW) {
            direction += forward;
        }
        if self.input.keyboard.is_pressed(KeyCode::KeyS) {
            direction -= forward;
        }
        if self.input.keyboard.is_pressed(KeyCode::KeyD) {
            direction += right;
        }
        if self.input.keyboard.is_pressed(KeyCode::KeyA) {
            direction -= right;
        }
        if direction.length_squared() > 0.0 {
            let delta = direction.normalize() * (WALK_SPEED * delta_seconds as f32);
            if let Some(collision) = self.walk_collision.as_ref().filter(|_| !self.noclip) {
                let observation = collision.move_disc_in_embedding(
                    self.comparative_embedding,
                    [observer.position.x, observer.position.z],
                    [delta.x, delta.z],
                    WALK_RADIUS,
                );
                if observation.contacted_linedefs != self.last_collision_contacts {
                    if !observation.contacted_linedefs.is_empty() {
                        eprintln!(
                            "E1M1 walk collision: contacts={:?}; broad_phase_candidates={}; fallback_to_all_blocking_walls={}",
                            observation.contacted_linedefs,
                            observation.broad_phase_candidates,
                            observation.used_full_wall_fallback,
                        );
                    }
                    self.last_collision_contacts = observation.contacted_linedefs;
                }
                self.apply_walk_floor_transition(observation.resolved_position);
            } else if let Some(observer) = self.spawn_observer.as_mut() {
                observer.position += delta;
            }
        }
    }

    /// Applies a source-sector floor result after horizontal collision. This
    /// keeps vertical state at the corpus application edge: `tokimu-render`
    /// still receives only the resulting camera, and imported WAD records are
    /// not mutated.
    fn apply_walk_floor_transition(&mut self, candidate_position: [f32; 2]) {
        let Some(floors) = self.walk_floors.as_ref() else {
            return;
        };
        let active_ceiling_overrides = self
            .active_manual_doors
            .iter()
            // Retain closed entries too: the final closing tick must restore
            // the original source-height wall spans, not leave the last open
            // geometry resident after the door has finished moving.
            .map(|door| (door.target_sector, door.current_ceiling_height))
            .collect::<Vec<_>>();
        let Some(observer) = self.spawn_observer.as_mut() else {
            return;
        };
        let resolution = floors.resolve_transition_in_embedding(
            self.comparative_embedding,
            candidate_position,
            observer.floor,
            &active_ceiling_overrides,
        );
        match resolution {
            DoomWalkFloorResolution::Accepted {
                source_sector,
                floor_height,
                ceiling_height,
            } => {
                let floor_delta = f32::from(floor_height - observer.floor);
                observer.position.x = candidate_position[0];
                observer.position.z = candidate_position[1];
                observer.position.y += floor_delta;
                observer.floor = floor_height;
                observer.ceiling = ceiling_height;
                let message = format!(
                    "accepted:sector={}:floor={floor_height}:ceiling={ceiling_height}",
                    source_sector.record_index
                );
                if self.last_floor_transition.as_deref() != Some(&message) {
                    eprintln!("E1M1 walk floor transition: {message}");
                    self.last_floor_transition = Some(message);
                }
            }
            DoomWalkFloorResolution::StepTooHigh {
                source_sector,
                current_floor_height,
                candidate_floor_height,
                maximum_step_up,
            } => {
                let message = format!(
                    "blocked-step:sector={}:from={current_floor_height}:to={candidate_floor_height}:max-up={maximum_step_up}",
                    source_sector.record_index
                );
                if self.last_floor_transition.as_deref() != Some(&message) {
                    eprintln!("E1M1 walk floor transition: {message}");
                    self.last_floor_transition = Some(message);
                }
            }
            DoomWalkFloorResolution::InsufficientClearance {
                source_sector,
                floor_height,
                ceiling_height,
                required_clearance,
            } => {
                let message = format!(
                    "blocked-clearance:sector={}:floor={floor_height}:ceiling={ceiling_height}:required={required_clearance}",
                    source_sector.record_index
                );
                if self.last_floor_transition.as_deref() != Some(&message) {
                    eprintln!("E1M1 walk floor transition: {message}");
                    self.last_floor_transition = Some(message);
                }
            }
            DoomWalkFloorResolution::PointOutsideUniqueSubsector { point } => {
                let message = format!("retained-ambiguous-point=({}, {})", point[0], point[1]);
                if self.last_floor_transition.as_deref() != Some(&message) {
                    eprintln!("E1M1 walk floor transition: {message}");
                    self.last_floor_transition = Some(message);
                }
            }
        }
    }

    fn release_walk_keys(&mut self) {
        for key in [KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD] {
            self.input.keyboard.release(key);
        }
    }

    fn reset_spawn_observer(&mut self) {
        self.spawn_observer = self.initial_spawn_observer;
        self.observer_look = self.initial_observer_look;
        self.last_collision_contacts.clear();
        self.last_floor_transition = None;
        eprintln!("E1M1 source-spawn observer reset");
    }

    fn toggle_debug_console(&mut self) {
        let opening = !self.debug_console.is_open();
        if opening {
            self.set_mouse_captured(false);
            self.release_walk_keys();
        }
        self.debug_console.set_open(opening);
    }

    fn submit_debug_console(&mut self) {
        let Some(command) = self.debug_console.take_submission() else {
            return;
        };
        let normalized = command.trim().to_ascii_lowercase();
        let response = match normalized.as_str() {
            "help" => "commands: HELP | CLEAR | STATUS | CAMERA | COLLISION | LOOK | USE <linedef> | NOCLIP [ON|OFF|TOGGLE]".to_owned(),
            "clear" => {
                self.debug_console.clear();
                return;
            }
            "camera" => self.spawn_observer.map_or_else(
                || "camera: source-spawn observer unavailable".to_owned(),
                |observer| {
                    let look = self.observer_look.unwrap_or(ObserverLook {
                        yaw: 0.0,
                        pitch: 0.0,
                        last_cursor: None,
                    });
                    format!(
                        "camera: position=({:.2},{:.2},{:.2}) yaw={:.4} pitch={:.4} source_thing={}",
                        observer.position.x,
                        observer.position.y,
                        observer.position.z,
                        look.yaw,
                        look.pitch,
                        observer.source_record,
                    )
                },
            ),
            "status" => format!(
                "status: frame={} draws={} cutouts={} selection={:?} mouse_capture={} noclip={} active_manual_doors={} details={}",
                self.frame_index,
                self.draws.len(),
                self.cutout_draws.len(),
                self.candidate_selection,
                self.mouse_captured,
                self.noclip,
                self.active_manual_doors
                    .iter()
                    .filter(|door| door.phase != DoomManualDoorPhase::Closed)
                    .count(),
                self.active_manual_doors
                    .iter()
                    .filter(|door| door.phase != DoomManualDoorPhase::Closed)
                    .map(|door| format!(
                        "sector{}:{}/{}/{}:{:?}",
                        door.target_sector.record_index,
                        door.current_ceiling_height,
                        door.closed_ceiling_height,
                        door.open_ceiling_height,
                        door.phase,
                    ))
                    .collect::<Vec<_>>()
                    .join("|"),
            ),
            "collision" => self.walk_collision.as_ref().map_or_else(
                || "collision: unavailable; run with --walk-collision".to_owned(),
                |world| {
                    format!(
                        "collision: radius={WALK_RADIUS} blocking_linedefs={} noclip={} last_contacts={:?}",
                        world.blocking_wall_count(),
                        self.noclip,
                        self.last_collision_contacts,
                    )
                },
            ),
            "look" | "inspect" => self.inspect_center_ray(),
            command if command.starts_with("use") => self.resolve_use_command(command),
            "noclip" | "noclip toggle" => {
                self.noclip = !self.noclip;
                format!("noclip: {}", self.noclip)
            }
            "noclip on" => {
                self.noclip = true;
                "noclip: true".to_owned()
            }
            "noclip off" => {
                self.noclip = false;
                "noclip: false".to_owned()
            }
            _ => format!("unsupported command: {command}"),
        };
        self.debug_console.append(response);
    }

    fn resolve_use_command(&mut self, command: &str) -> String {
        let argument = command.strip_prefix("use").unwrap_or_default().trim();
        let Ok(record_index) = argument.parse::<u32>() else {
            return "use: expected USE <source-linedef-index>; LOOK retains a wall source index"
                .to_owned();
        };
        self.resolve_use_linedef(record_index)
    }

    fn resolve_use_linedef(&mut self, record_index: u32) -> String {
        let Some(linedef) = self
            .activation_source
            .linedefs
            .iter()
            .find(|linedef| linedef.source.record_index == record_index)
        else {
            return format!("use: source linedef {record_index} is not present in E1M1");
        };
        match resolve_doom_line_activation(
            &self.activation_source,
            DoomLineActivationRequest {
                source_linedef: linedef.source,
                activation: DoomLineActivation::Use,
            },
        ) {
            DoomLineActivationResolution::Accepted {
                source_linedef,
                special,
                intent:
                    DoomLineActivationIntent::RaiseDoor {
                        target_sector,
                    },
            } => self.start_manual_door(source_linedef.record_index, special, target_sector),
            DoomLineActivationResolution::Accepted {
                source_linedef,
                special,
                intent,
            } => format!(
                "use: accepted linedef={} lump={} special={} intent={} target={} execution=deferred-to-future-runtime-owner",
                source_linedef.record_index,
                source_linedef.lump_index,
                special,
                compact_activation_intent(intent),
                compact_activation_target(intent),
            ),
            DoomLineActivationResolution::NoSpecial { source_linedef } => format!(
                "use: linedef={} lump={} has no source special",
                source_linedef.record_index, source_linedef.lump_index,
            ),
            DoomLineActivationResolution::WrongActivation {
                source_linedef,
                special,
                required,
                ..
            } => format!(
                "use: linedef={} special={} requires {:?}; requested Use",
                source_linedef.record_index, special, required,
            ),
            DoomLineActivationResolution::UnsupportedSpecial {
                source_linedef,
                special,
            } => format!(
                "use: linedef={} special={} is retained but not admitted for a use request",
                source_linedef.record_index, special,
            ),
            DoomLineActivationResolution::UnknownLinedef { source_linedef } => format!(
                "use: source linedef={} lump={} is unavailable",
                source_linedef.record_index, source_linedef.lump_index,
            ),
            DoomLineActivationResolution::MissingManualDoorTarget {
                source_linedef,
                missing_left_sidedef,
            } => format!(
                "use: manual-door linedef={} cannot resolve opposite sidedef={missing_left_sidedef:?}",
                source_linedef.record_index,
            ),
            DoomLineActivationResolution::InvalidManualDoorTarget {
                source_linedef,
                sidedef_index,
                sector_index,
            } => format!(
                "use: manual-door linedef={} has invalid target sidedef={} sector={}",
                source_linedef.record_index, sidedef_index, sector_index,
            ),
        }
    }

    fn try_use_center_wall(&mut self) -> String {
        if self.comparative_embedding != DoomComparativeEmbedding::CurrentReflected {
            return format!(
                "use: unavailable during AR-0028 {:?} visual comparison; dynamic-door source correspondence remains an explicit open control",
                self.comparative_embedding
            );
        }
        let (Some(observer), Some(look)) = (self.spawn_observer, self.observer_look) else {
            return "use: source-spawn observer unavailable".to_owned();
        };
        let direction = observer_direction(look.yaw, look.pitch).normalize_or_zero();
        let mut nearest: Option<(f32, u32)> = None;
        for draw in self.draws.iter().chain(
            self.include_cutouts
                .then_some(&self.cutout_draws)
                .into_iter()
                .flatten(),
        ) {
            let StaticDrawSource::Wall { source_linedef, .. } = draw.source else {
                continue;
            };
            let Some(distance) = nearest_mesh_ray_hit(observer.position, direction, &draw.mesh)
            else {
                continue;
            };
            if nearest.is_none_or(|(nearest_distance, _)| distance < nearest_distance) {
                nearest = Some((distance, source_linedef.record_index));
            }
        }
        match nearest {
            Some((distance, source_linedef)) => {
                let outcome = self.resolve_use_linedef(source_linedef);
                format!("use: center-wall-distance={distance:.3}; {outcome}")
            }
            None => "use: no exact prepared wall intersects the center ray".to_owned(),
        }
    }

    fn start_manual_door(
        &mut self,
        source_linedef: u32,
        special: u16,
        target_sector: doom_map_provider::DoomSourceRecord,
    ) -> String {
        let replacement = match DoomManualDoorRuntime::start(
            &self.activation_source,
            target_sector,
            DoomManualDoorPolicy::CLASSIC_NORMAL,
        ) {
            Ok(door) => door,
            Err(error) => {
                return format!(
                    "use: manual-door linedef={source_linedef} special={special} target-sector={} start-rejected={error:?}",
                    target_sector.record_index,
                );
            }
        };
        if let Some(active) = self
            .active_manual_doors
            .iter_mut()
            .find(|door| door.target_sector == target_sector)
        {
            if active.phase != DoomManualDoorPhase::Closed {
                return format!(
                    "use: manual-door linedef={source_linedef} target-sector={} already-active phase={:?}",
                    target_sector.record_index, active.phase
                );
            }
            *active = replacement;
        } else {
            self.active_manual_doors.push(replacement);
        }
        let boundary_linedefs =
            manual_door_boundary_linedefs(&self.activation_source, target_sector);
        let prepared_meshes_at_closed_height = self
            .draws
            .iter()
            .filter(|draw| {
                is_door_mesh_for_target(draw, target_sector, &boundary_linedefs)
                    && draw.mesh.positions.iter().any(|position| {
                        (position[1] - f32::from(replacement.closed_ceiling_height)).abs()
                            <= f32::EPSILON
                    })
            })
            .count();
        format!(
            "use: manual-door started linedef={source_linedef} special={special} target-sector={} closed-height={} open-height={} prepared-meshes-at-closed-height={prepared_meshes_at_closed_height} policy=2-units-per-tick/150-tick-wait",
            target_sector.record_index,
            replacement.closed_ceiling_height,
            replacement.open_ceiling_height,
        )
    }

    fn advance_active_manual_doors(&mut self, delta_seconds: f64) {
        self.door_tick_accumulator += delta_seconds.clamp(0.0, 0.25);
        let mut changed = false;
        while self.door_tick_accumulator >= DOOM_TIC_SECONDS {
            self.door_tick_accumulator -= DOOM_TIC_SECONDS;
            let transitions = self
                .active_manual_doors
                .iter_mut()
                .filter(|door| door.phase != DoomManualDoorPhase::Closed)
                .map(DoomManualDoorRuntime::advance_tick)
                .filter(|tick| tick.before_height != tick.after_height)
                .collect::<Vec<_>>();
            for tick in transitions {
                self.dirty_opaque_meshes
                    .extend(apply_door_ceiling_flat_height(
                        &mut self.draws,
                        tick.target_sector,
                        tick.before_height,
                        tick.after_height,
                    ));
                changed = true;
            }
        }
        if changed {
            match self.refresh_active_manual_door_wall_meshes() {
                Ok(()) => self.door_visual_diagnostic = None,
                Err(error) => {
                    let diagnostic = format!("door visual refresh failed: {error}");
                    if self.door_visual_diagnostic.as_deref() != Some(&diagnostic) {
                        eprintln!("E1M1 {diagnostic}");
                        self.debug_console.append(diagnostic.clone());
                    }
                    self.door_visual_diagnostic = Some(diagnostic);
                }
            }
        }
    }

    /// Re-lowers only wall spans attributable to the active manual-door
    /// sectors from a clone of the already decoded map. Runtime ceiling state
    /// replaces the clone's source height; WAD bytes and source records remain
    /// unchanged. This prevents vertex-only deformation from silently becoming
    /// Doom wall-span or UV policy.
    fn refresh_active_manual_door_wall_meshes(&mut self) -> PlatformResult<()> {
        let mut map = self.door_geometry_source.map.clone();
        let active = self
            .active_manual_doors
            .iter()
            // A completed closing tick must also restore the source-height
            // spans; keeping closed entries here makes that final refresh
            // explicit.
            .map(|door| {
                (
                    door.target_sector,
                    door.current_ceiling_height,
                    manual_door_boundary_linedefs(&self.activation_source, door.target_sector),
                )
            })
            .collect::<Vec<_>>();
        for (target_sector, height, _) in &active {
            if let Some(sector) = map
                .sectors
                .iter_mut()
                .find(|sector| sector.source == *target_sector)
            {
                sector.ceiling_height = *height;
            }
        }

        let mut dynamic_meshes = BTreeMap::<String, Vec<DynamicDoorWallMesh>>::new();
        for triangle in
            lower_doom_textured_wall_triangles(&map, &self.door_geometry_source.wall_extents)?
        {
            let affected = active.iter().any(|(target_sector, _, boundaries)| {
                triangle.source_sector == *target_sector
                    || (triangle.role == doom_geometry_provider::DoomWallTextureRole::Upper
                        && boundaries.contains(&triangle.source_linedef))
            });
            if !affected {
                continue;
            }
            let Some(extent) = self
                .door_geometry_source
                .wall_extents
                .iter()
                .find(|extent| extent.name == triangle.texture_name)
            else {
                return Err(io::Error::other(format!(
                    "active door wall {} has no retained texture extent",
                    triangle.texture_name
                ))
                .into());
            };
            let mesh = match lower_static_wall_triangle(&triangle, extent.clone()) {
                Ok(lowered) => lowered.mesh,
                // These were already retained as zero-area source omissions by
                // the static preparation. A runtime height substitution can
                // encounter the same authored empty band; it is not a reason
                // to terminate the presentation loop.
                Err(StaticFlatLoweringError::DegenerateTriangle) => continue,
                Err(error) => return Err(error.into()),
            };
            dynamic_meshes
                .entry(dynamic_wall_triangle_key(
                    triangle.source_linedef,
                    triangle.source_sidedef,
                    triangle.source_sector,
                    triangle.role,
                    &triangle.texture_name,
                ))
                .or_default()
                .push(DynamicDoorWallMesh {
                    mesh,
                    source_linedef: triangle.source_linedef,
                    source_sidedef: triangle.source_sidedef,
                    source_sector: triangle.source_sector,
                    role: triangle.role,
                    texture_name: triangle.texture_name,
                });
        }

        let mut existing = std::collections::BTreeMap::<String, Vec<usize>>::new();
        for (index, draw) in self.draws.iter().enumerate() {
            let affected = active.iter().any(|(target_sector, _, boundaries)| {
                is_door_mesh_for_target(draw, *target_sector, boundaries)
            });
            if affected {
                if let Some(key) = static_wall_triangle_key(draw) {
                    existing.entry(key).or_default().push(index);
                }
            }
        }

        for (key, indices) in existing {
            let Some(meshes) = dynamic_meshes.remove(&key) else {
                // A zero-height source band is absent from the fresh lowering.
                // Dynamic-only spans are explicitly suppressed while absent;
                // ordinary static spans retain their source-height geometry.
                for index in indices {
                    if self.dynamic_door_draws.contains(&index) {
                        self.opaque_draw_enabled[index] = false;
                    }
                }
                continue;
            };
            if meshes.len() != indices.len() {
                continue;
            }
            for (index, mesh) in indices.into_iter().zip(meshes) {
                self.draws[index].mesh = mesh.mesh;
                self.opaque_bounds[index] =
                    StaticDrawAabb::from_positions(&self.draws[index].mesh.positions);
                self.opaque_draw_enabled[index] = true;
                self.dirty_opaque_meshes.insert(index);
            }
        }
        let mut missing_materials = Vec::new();
        for meshes in dynamic_meshes.into_values() {
            for mesh in meshes {
                let Some(material) = self
                    .door_geometry_source
                    .wall_materials
                    .get(&mesh.texture_name)
                    .copied()
                else {
                    missing_materials.push(mesh.texture_name);
                    continue;
                };
                let index = self.draws.len();
                self.draws.push(StaticDrawPlanEntry {
                    mesh: mesh.mesh,
                    material,
                    source_label: format!(
                        "wall:{}:{}",
                        mesh.source_linedef.record_index, mesh.texture_name
                    ),
                    source: StaticDrawSource::Wall {
                        source_linedef: mesh.source_linedef,
                        source_sidedef: mesh.source_sidedef,
                        source_sector: mesh.source_sector,
                        role: mesh.role,
                    },
                });
                self.opaque_bounds.push(StaticDrawAabb::from_positions(
                    &self.draws[index].mesh.positions,
                ));
                self.opaque_selected.push(true);
                self.opaque_draw_enabled.push(true);
                self.dynamic_door_draws.insert(index);
                let handle = MeshHandle(self.next_dynamic_mesh_handle);
                self.next_dynamic_mesh_handle = self.next_dynamic_mesh_handle.saturating_add(1);
                self.dynamic_door_mesh_handles.insert(index, handle);
                self.dirty_opaque_meshes.insert(index);
                // The fixed grid was built for the static scene. Fall back to
                // its existing conservative non-grid selection until a later
                // corpus result earns a dynamic-index policy.
                self.opaque_grid = None;
            }
        }
        missing_materials.sort();
        missing_materials.dedup();
        if missing_materials.is_empty() {
            self.door_geometry_diagnostic = None;
        } else {
            let diagnostic = format!(
                "door geometry has no prepared material for: {}",
                missing_materials.join(", ")
            );
            if self.door_geometry_diagnostic.as_deref() != Some(&diagnostic) {
                eprintln!("E1M1 {diagnostic}");
                self.debug_console.append(diagnostic.clone());
            }
            self.door_geometry_diagnostic = Some(diagnostic);
        }
        Ok(())
    }

    fn inspect_center_ray(&self) -> String {
        let (Some(observer), Some(look)) = (self.spawn_observer, self.observer_look) else {
            return "look: source-spawn observer unavailable".to_owned();
        };
        let direction = observer_direction(look.yaw, look.pitch).normalize_or_zero();
        let mut nearest: Option<(f32, &StaticDrawPlanEntry, &'static str)> = None;
        for (draw, family) in self.draws.iter().map(|draw| (draw, "opaque")).chain(
            self.include_cutouts
                .then_some(())
                .into_iter()
                .flat_map(|_| self.cutout_draws.iter().map(|draw| (draw, "cutout"))),
        ) {
            for triangle in draw.mesh.positions.chunks_exact(3) {
                let Some(distance) = ray_triangle_distance(
                    observer.position,
                    direction,
                    Vec3::from_array(triangle[0]),
                    Vec3::from_array(triangle[1]),
                    Vec3::from_array(triangle[2]),
                ) else {
                    continue;
                };
                if nearest.is_none_or(|(current, _, _)| distance < current) {
                    nearest = Some((distance, draw, family));
                }
            }
        }
        nearest.map_or_else(
            || "look: no prepared triangle intersects the center ray".to_owned(),
            |(distance, draw, family)| {
                format!(
                    "look: exact prepared-triangle hit distance={distance:.3} family={family} material={} label={} source={}",
                    draw.material.0,
                    draw.source_label,
                    compact_draw_source(&draw.source),
                )
            },
        )
    }

    fn rebuild_debug_console(&mut self, renderer: &mut WgpuBackend) -> PlatformResult<()> {
        let font = self
            .debug_font
            .as_ref()
            .ok_or_else(|| io::Error::other("debug console font missing"))?;
        let raster = self
            .debug_console
            .rasterize(font, self.size[0].max(320.0) as u32);
        renderer.try_upload_texture(
            DEBUG_TEXTURE,
            &Texture::rgba8(raster.width, raster.height, raster.rgba8),
        )?;
        renderer.upload_material(
            DEBUG_MATERIAL,
            &Material::new("doom-debug-console", Color::rgb(1.0, 1.0, 1.0))
                .with_texture(DEBUG_TEXTURE),
        )?;
        Ok(())
    }
}

fn compact_activation_intent(intent: DoomLineActivationIntent) -> &'static str {
    match intent {
        DoomLineActivationIntent::RaiseDoor { .. } => "raise-door-from-interacting-side",
        DoomLineActivationIntent::ExitLevel { .. } => "exit-level",
        DoomLineActivationIntent::LowerFloorTurbo { .. } => "lower-floor-turbo",
        DoomLineActivationIntent::PlatformDownWaitUpStay { .. } => "platform-down-wait-up-stay",
    }
}

fn compact_activation_target(intent: DoomLineActivationIntent) -> String {
    match intent {
        DoomLineActivationIntent::RaiseDoor { target_sector } => format!(
            "sector={} lump={}",
            target_sector.record_index, target_sector.lump_index
        ),
        DoomLineActivationIntent::ExitLevel { tag }
        | DoomLineActivationIntent::LowerFloorTurbo { tag }
        | DoomLineActivationIntent::PlatformDownWaitUpStay { tag } => format!("tag={tag}"),
    }
}

fn compact_draw_source(source: &StaticDrawSource) -> String {
    match source {
        StaticDrawSource::Wall {
            source_linedef,
            source_sidedef,
            source_sector,
            ..
        } => format!(
            "wall linedef={} sidedef={} sector={} lumps={}/{}/{}",
            source_linedef.record_index,
            source_sidedef.record_index,
            source_sector.record_index,
            source_linedef.lump_index,
            source_sidedef.lump_index,
            source_sector.lump_index,
        ),
        StaticDrawSource::Flat {
            source_subsector,
            source_sector,
            plane,
        } => format!(
            "flat subsector={} sector={} plane={plane:?} lumps={}/{}",
            source_subsector.record_index,
            source_sector.record_index,
            source_subsector.lump_index,
            source_sector.lump_index,
        ),
    }
}

impl PlatformEventHandler for App {
    fn on_native_window_created(&mut self, window: Arc<NativeWindow>) -> PlatformResult<()> {
        let size = window.inner_size();
        self.size = [size.width.max(1) as f32, size.height.max(1) as f32];
        let mut renderer = WgpuBackend::for_window(window.clone(), size.width, size.height)?;
        window.set_ime_allowed(true);
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
        if self.diagnostic_sky_enabled {
            // AR-0027 Alternative A: this corpus chooses a checked-in Purple
            // PNG for retained sky omissions. It is not a Doom asset lookup,
            // a renderer fallback, or successful source resolution.
            let decoded = decode_png(
                include_bytes!("../../../assets/PNG/Purple/texture_01.png"),
                DecodeLimits::default(),
            )?;
            let prepared = prepare_renderer_texture(&decoded, TextureUse::ColorSrgb)
                .map_err(io::Error::other)?;
            renderer.create_texture_rgba8(
                DIAGNOSTIC_SKY_TEXTURE,
                tokimu::Rgba8TextureDescriptor::new(
                    prepared.texture.width,
                    prepared.texture.height,
                    tokimu::Rgba8TextureColorSpace::Srgb,
                ),
                &prepared.texture.rgba8,
            )?;
            renderer.upload_material(
                DIAGNOSTIC_SKY_MATERIAL,
                &Material::new("e1m1-diagnostic-sky-omission", Color::rgb(1.0, 1.0, 1.0))
                    .with_texture(DIAGNOSTIC_SKY_TEXTURE)
                    .with_texture_sampler(TextureSampler {
                        filter: TextureFilter::Point,
                        address_u: TextureAddressMode::Repeat,
                        address_v: TextureAddressMode::Repeat,
                    }),
            )?;
            for (index, draw) in self.diagnostic_sky_draws.iter().enumerate() {
                renderer.upload_mesh(
                    MeshHandle(DIAGNOSTIC_SKY_MESH_BASE + index as u64),
                    &draw.mesh,
                );
            }
            eprintln!(
                "E1M1 AR-0027 diagnostic sky stand-in enabled: draws={}; asset=corpus/assets/PNG/Purple/texture_01.png; records={}",
                self.diagnostic_sky_draws.len(),
                self.diagnostic_sky_records.len(),
            );
            for record in self.diagnostic_sky_records.iter().take(8) {
                eprintln!("E1M1 AR-0027 diagnostic record: {record}");
            }
        }
        self.debug_font = Some(
            UiFontRasterizer::from_bytes(UiFontSource::from_native_default()?.bytes)
                .map_err(io::Error::other)?,
        );
        renderer.upload_mesh(DEBUG_QUAD, &Mesh::quad());
        renderer.upload_material(
            DEBUG_CURSOR_MATERIAL,
            &Material::new("doom-debug-center-cursor", Color::rgb(0.35, 0.95, 0.82)),
        )?;
        self.debug_pipeline = Some(renderer.register_pipeline(&Pipeline::new(
            "doom-debug-console",
            PipelineKind::Texture2d,
        ))?);
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
        if let Some(collision) = &self.walk_collision {
            eprintln!(
                "E1M1 Slice 6 walk proof: radius={WALK_RADIUS}; speed={WALK_SPEED}; blocking_linedefs={}; broad_phase=source-blockmap-with-full-wall-fallback; noclip={}; controls=WASD-move-click-capture-escape-release-R-reset",
                collision.blocking_wall_count(),
                self.noclip,
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
            "E1M1 native first-frame metadata: opaque_draws={}; cutout_draws={}; cutouts_enabled={}; camera={}; candidate_selection={}; walk_collision={}; noclip={}; backend={}; device={}; adapter={}",
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
            self.walk_collision.is_some(),
            self.noclip,
            renderer.backend_api(),
            renderer.device_kind(),
            renderer.adapter_name(),
        );
        self.renderer = Some(renderer);
        Ok(())
    }
    fn on_platform_event(&mut self, event: PlatformInputEvent) -> PlatformResult<()> {
        if let PlatformInputEvent::KeyboardInput {
            key: KeyCode::Backquote,
            pressed: true,
        } = event
        {
            self.toggle_debug_console();
            return Ok(());
        }
        if self.debug_console.is_open() {
            match event {
                PlatformInputEvent::TextInput(text) => self.debug_console.insert_text(&text),
                PlatformInputEvent::KeyboardInput {
                    key: KeyCode::Enter,
                    pressed: true,
                } => self.submit_debug_console(),
                PlatformInputEvent::KeyboardInput {
                    key: KeyCode::Backspace,
                    pressed: true,
                } => self.debug_console.backspace(),
                PlatformInputEvent::KeyboardInput {
                    key: KeyCode::Escape,
                    pressed: true,
                } => self.toggle_debug_console(),
                PlatformInputEvent::Resized { width, height } => {
                    self.size = [width.max(1) as f32, height.max(1) as f32];
                    self.debug_console.invalidate();
                    if let Some(renderer) = self.renderer.as_mut() {
                        renderer.resize_surface(width, height);
                    }
                }
                _ => {}
            }
            return Ok(());
        }
        if let Some(input_event) = event.as_input_event() {
            self.input.apply_event(input_event);
        }
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
                self.release_walk_keys();
            } else if key == KeyCode::KeyR && pressed {
                self.reset_spawn_observer();
            } else if key == KeyCode::KeyE && pressed {
                let outcome = self.try_use_center_wall();
                eprintln!("E1M1 {outcome}");
                self.debug_console.append(outcome);
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
            self.debug_console.invalidate();
            if let Some(renderer) = self.renderer.as_mut() {
                renderer.resize_surface(width, height);
            }
        }
        Ok(())
    }

    fn on_frame(&mut self, delta_seconds: f64) -> PlatformResult<FrameOutcome> {
        self.apply_inspection_movement(delta_seconds);
        self.advance_active_manual_doors(delta_seconds);
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
            if !self.opaque_selected[index] || !self.opaque_draw_enabled[index] {
                continue;
            }
            let mesh = self
                .dynamic_door_mesh_handles
                .get(&index)
                .copied()
                .unwrap_or(MeshHandle(index as u64 + 1));
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
                let mesh = MeshHandle(self.cutout_mesh_base + offset as u64);
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
        if self.diagnostic_sky_enabled {
            for (index, _) in self.diagnostic_sky_draws.iter().enumerate() {
                self.commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: MeshHandle(DIAGNOSTIC_SKY_MESH_BASE + index as u64),
                    material: DIAGNOSTIC_SKY_MATERIAL,
                    pipeline: self.pipeline,
                    instance: Instance2d::identity(),
                    camera: Some(CAMERA),
                    viewport: None,
                }));
            }
        }
        if self.debug_console.is_open() {
            let debug_pipeline = self
                .debug_pipeline
                .ok_or_else(|| io::Error::other("debug console pipeline missing"))?;
            self.commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                mesh: DEBUG_QUAD,
                material: DEBUG_MATERIAL,
                pipeline: debug_pipeline,
                instance: Instance2d::new(
                    [0.0, 0.72],
                    [(self.size[0] / self.size[1]).max(1.0) * 2.0, 0.56],
                    0.0,
                ),
                camera: Some(DEBUG_CAMERA),
                viewport: None,
            }));
        } else if let Some(debug_pipeline) = self.debug_pipeline {
            for size in [[0.032, 0.003], [0.003, 0.048]] {
                self.commands.push(RenderCommand::DrawMesh(DrawMeshCommand {
                    mesh: DEBUG_QUAD,
                    material: DEBUG_CURSOR_MATERIAL,
                    pipeline: debug_pipeline,
                    instance: Instance2d::new([0.0, 0.0], size, 0.0),
                    camera: Some(DEBUG_CAMERA),
                    viewport: None,
                }));
            }
        }
        let command_time = command_started.elapsed();
        if self.debug_console.is_open() && self.debug_console.take_dirty() {
            let mut renderer = self
                .renderer
                .take()
                .ok_or_else(|| io::Error::other("renderer missing"))?;
            let rebuilt = self.rebuild_debug_console(&mut renderer);
            self.renderer = Some(renderer);
            rebuilt?;
        }
        let dynamic_mesh_uploads = std::mem::take(&mut self.dirty_opaque_meshes)
            .into_iter()
            .map(|index| {
                (
                    self.dynamic_door_mesh_handles
                        .get(&index)
                        .copied()
                        .unwrap_or(MeshHandle(index as u64 + 1)),
                    self.draws[index].mesh.clone(),
                )
            })
            .collect::<Vec<_>>();
        let renderer = self
            .renderer
            .as_mut()
            .ok_or_else(|| io::Error::other("renderer missing"))?;
        for (handle, mesh) in dynamic_mesh_uploads {
            renderer.upload_mesh(handle, &mesh);
        }
        renderer.upload_camera(CAMERA, camera);
        if self.debug_pipeline.is_some() {
            renderer.upload_camera(
                DEBUG_CAMERA,
                Camera::orthographic_2d(self.size[0], self.size[1]),
            );
        }
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
                    MeshHandle(self.cutout_mesh_base + offset as u64),
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
    let walk_collision = DoomWalkCollisionWorld::from_map(&map);
    let walk_floors = DoomWalkFloorWorld::from_map(&map)?;
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
    let activation_source = DoomLineActivationSource::from_map(&map);
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
    let wall_extents =
        prepare_e1m1_wall_texture_extents(&read.bytes, &read.observation.wad, TEXTURE_LIMITS)?;
    let mut names = hello_doom_e1m1::prepared_e1m1_wall_texture_names(&walls);
    names.extend(manual_door_dynamic_wall_texture_names(
        &map,
        &activation_source,
        &wall_extents,
    )?);
    names.sort();
    names.dedup();
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
    let wall_materials = uploads
        .iter()
        .filter(|upload| upload.source_kind == hello_doom_e1m1::StaticTextureSourceKind::Wall)
        .map(|upload| (upload.source_name.clone(), upload.material))
        .collect();
    let draws = build_static_draw_plan(&flats, &walls, &uploads)?;
    let diagnostic_sky_draws =
        prepare_e1m1_sky_diagnostic_flats(&read.bytes, &read.observation.wad, MAP_LIMITS)?
            .into_iter()
            .map(|flat| StaticDrawPlanEntry {
                source_label: format!(
                    "diagnostic-sky:{}:{}:{:?}",
                    flat.source.subsector.record_index,
                    flat.source.sector.record_index,
                    flat.source.plane
                ),
                source: StaticDrawSource::Flat {
                    source_subsector: flat.source.subsector,
                    source_sector: flat.source.sector,
                    plane: flat.source.plane,
                },
                mesh: flat.mesh,
                material: DIAGNOSTIC_SKY_MATERIAL,
            })
            .collect::<Vec<_>>();
    let diagnostic_sky_records = diagnostic_sky_draws
        .iter()
        .map(|draw| {
            format!(
            "reason=intentional-source-sky-omission; original={}; stand-in=Purple/texture_01.png",
            compact_draw_source(&draw.source),
        )
        })
        .collect::<Vec<_>>();
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
        diagnostic_sky_draws,
        diagnostic_sky_records,
        spawn_observer,
        walk_collision,
        walk_floors,
        reject_report,
        topology_report,
        membership_selection,
        activation_source,
        door_geometry_source: DoomDynamicDoorGeometrySource {
            map,
            wall_extents,
            wall_materials,
        },
    })
}

/// Applies one AR-0028 candidate after ordinary Doom preparation so decoded
/// source facts and the active provider remain unchanged. This is sufficient
/// for fixed-camera and noclip visual comparison; collision, Doom membership,
/// and dynamic doors are deliberately excluded until their source
/// correspondence controls are migrated too.
fn reembed_scene_for_comparison(scene: &mut SceneInput, embedding: DoomComparativeEmbedding) {
    if embedding == DoomComparativeEmbedding::CurrentReflected {
        return;
    }
    for draw in scene
        .opaque_draws
        .iter_mut()
        .chain(scene.cutout_draws.iter_mut())
        .chain(scene.diagnostic_sky_draws.iter_mut())
    {
        let reverse_wall_u = matches!(draw.source, StaticDrawSource::Wall { .. });
        reembed_comparative_mesh(&mut draw.mesh, embedding, reverse_wall_u);
        if matches!(draw.source, StaticDrawSource::Flat { .. }) {
            // Flat coordinates are a continuous source-spatial field, so the
            // orientation-preserving migration reverses U about the source
            // origin. Per-triangle reflection would change the phase at every
            // triangulation boundary and introduce seams.
            for uv in &mut draw.mesh.texture_coordinates {
                uv[0] = -uv[0];
            }
        }
    }

    scene.spawn_observer.position = embedding.lift_direction(
        scene.spawn_observer.source_position.map(f32::from),
        scene.spawn_observer.position.y,
    );
    scene.spawn_observer.forward =
        embedding.lift_heading_degrees(f32::from(scene.spawn_observer.source_angle));

    for bounds in &mut scene.membership_selection.subsector_bounds {
        *bounds = bounds.and_then(|bounds| reembed_aabb(bounds, embedding));
    }
}

fn reembed_aabb(
    bounds: StaticDrawAabb,
    embedding: DoomComparativeEmbedding,
) -> Option<StaticDrawAabb> {
    let minimum = bounds.minimum();
    let maximum = bounds.maximum();
    let mut transformed_minimum = Vec3::splat(f32::INFINITY);
    let mut transformed_maximum = Vec3::splat(f32::NEG_INFINITY);
    for x in [minimum.x, maximum.x] {
        for y in [minimum.y, maximum.y] {
            for z in [minimum.z, maximum.z] {
                let point = embedding.lift_direction([x, z], y);
                transformed_minimum = transformed_minimum.min(point);
                transformed_maximum = transformed_maximum.max(point);
            }
        }
    }
    StaticDrawAabb::from_minimum_maximum(transformed_minimum, transformed_maximum)
}

fn report_spatial_flat_uv(scene: &SceneInput, embedding: DoomComparativeEmbedding) {
    let camera_right = observer_right(scene.spawn_observer.forward);
    let mut aligned = 0usize;
    let mut opposed = 0usize;
    let mut neutral = 0usize;
    for draw in &scene.diagnostic_sky_draws {
        for left in 0..draw.mesh.positions.len() {
            for right in left + 1..draw.mesh.positions.len() {
                let world_delta = Vec3::from_array(draw.mesh.positions[right])
                    - Vec3::from_array(draw.mesh.positions[left]);
                let screen_delta = camera_right.dot(world_delta);
                let u_delta = draw.mesh.texture_coordinates[right][0]
                    - draw.mesh.texture_coordinates[left][0];
                let product = screen_delta * u_delta;
                if product > 0.000_1 {
                    aligned += 1;
                } else if product < -0.000_1 {
                    opposed += 1;
                } else {
                    neutral += 1;
                }
            }
        }
    }
    println!(
        "E1M1 AR-0028 flat-U observation: embedding={embedding:?}; diagnostic_sky_draws={}; camera_right=({:.1},{:.1},{:.1}); aligned_pairs={aligned}; opposed_pairs={opposed}; neutral_pairs={neutral}",
        scene.diagnostic_sky_draws.len(),
        camera_right.x,
        camera_right.y,
        camera_right.z,
    );
}

/// Bounded, renderer-free Slice 6 evidence. The replay does not claim an
/// original-Doom movement model; it only proves that this corpus-local disc
/// calculation produces the same source-line contacts and end position when
/// given the same fixed command sequence twice.
fn report_walk_collision(scene: &SceneInput) {
    let start = [
        scene.spawn_observer.position.x,
        scene.spawn_observer.position.z,
    ];
    let forward = scene.spawn_observer.forward.normalize_or_zero();
    let right = observer_right(forward);
    let commands = [
        [forward.x * 12.0, forward.z * 12.0],
        [forward.x * 12.0, forward.z * 12.0],
        [right.x * 12.0, right.z * 12.0],
        [right.x * 12.0, right.z * 12.0],
        [-forward.x * 6.0, -forward.z * 6.0],
    ];
    let replay = |world: &DoomWalkCollisionWorld| {
        let mut position = start;
        let mut contacts = BTreeSet::new();
        let mut fallback = false;
        for command in commands {
            let observation = world.move_disc(position, command, WALK_RADIUS);
            position = observation.resolved_position;
            contacts.extend(observation.contacted_linedefs);
            fallback |= observation.used_full_wall_fallback;
        }
        (position, contacts, fallback)
    };
    let first = replay(&scene.walk_collision);
    let second = replay(&scene.walk_collision);
    assert_eq!(
        first, second,
        "fixed collision replay must be deterministic"
    );
    let probe = scene
        .walk_collision
        .probe_nearest_blocking_wall(start, WALK_RADIUS)
        .expect("decoded E1M1 has blocking linedefs");
    assert!(
        probe
            .observation
            .contacted_linedefs
            .contains(&probe.source_linedef),
        "nearest-wall probe must retain its blocking linedef contact"
    );
    println!(
        "E1M1 Slice 6 walk replay: start=({:.1},{:.1}); commands={}; radius={}; blocking_linedefs={}; end=({:.3},{:.3}); contacts={:?}; blockmap_full_wall_fallback={}; deterministic_replay=true; nearest_wall_probe=[linedef:{}; initial_distance:{:.3}; contacts:{:?}; fallback:{}]; scope=corpus-local-disc-no-clearance-or-step-policy",
        start[0],
        start[1],
        commands.len(),
        WALK_RADIUS,
        scene.walk_collision.blocking_wall_count(),
        first.0[0],
        first.0[1],
        first.1,
        first.2,
        probe.source_linedef,
        probe.distance_before_move,
        probe.observation.contacted_linedefs,
        probe.observation.used_full_wall_fallback,
    );
}

/// Renderer-free AR-0028 evidence at the canonical Doom player-one start.
/// It reports the competing source and observer bases without selecting a
/// repair or changing the source conversion.
fn report_spatial_orientation(scene: &SceneInput) {
    let radians = f32::from(scene.spawn_observer.source_angle).to_radians();
    let source_forward = [radians.cos(), radians.sin()];
    let source_right = [radians.sin(), -radians.cos()];
    for embedding in DoomComparativeEmbedding::ALL {
        let observation =
            observe_doom_ground_frame_with_embedding(embedding, source_right, source_forward);
        println!(
            "E1M1 AR-0028 ground frame: embedding={:?}; thing={}; source-position=({}, {}); source-angle={}; source-right=({:.3},{:.3}); source-forward=({:.3},{:.3}); source-cross={:.3}; lifted-right=({:.3},{:.3},{:.3}); lifted-forward=({:.3},{:.3},{:.3}); world-up-cross={:.3}; camera-right=({:.3},{:.3},{:.3}); source-right/camera-right-alignment={:.3}",
            observation.embedding,
            scene.spawn_observer.source_record,
            scene.spawn_observer.source_position[0],
            scene.spawn_observer.source_position[1],
            scene.spawn_observer.source_angle,
            observation.source_right[0],
            observation.source_right[1],
            observation.source_forward[0],
            observation.source_forward[1],
            observation.source_signed_orientation,
            observation.lifted_right.x,
            observation.lifted_right.y,
            observation.lifted_right.z,
            observation.lifted_forward.x,
            observation.lifted_forward.y,
            observation.lifted_forward.z,
            observation.lifted_orientation_about_world_up,
            observation.camera_right.x,
            observation.camera_right.y,
            observation.camera_right.z,
            observation.source_right_camera_right_alignment,
        );
    }
}

/// Bounded source-record candidates for identifying the exterior hut in a
/// canonical comparison. Texture-name filtering only narrows the inspection;
/// it does not assert that any listed record is the landmark.
fn report_spatial_landmark_candidates(scene: &SceneInput) {
    let map = &scene.door_geometry_source.map;
    let spawn = scene.spawn_observer.source_position.map(f32::from);
    let radians = f32::from(scene.spawn_observer.source_angle).to_radians();
    let forward = [radians.cos(), radians.sin()];
    let right = [radians.sin(), -radians.cos()];

    println!(
        "E1M1 AR-0028 landmark candidates: spawn=({}, {}); angle={}; filter=BROWN*|*DOOR*",
        spawn[0], spawn[1], scene.spawn_observer.source_angle
    );
    for linedef in &map.linedefs {
        let texture_names = [linedef.right_sidedef, linedef.left_sidedef]
            .into_iter()
            .flatten()
            .filter_map(|index| map.sidedefs.get(usize::from(index)))
            .flat_map(|side| {
                [
                    side.upper_texture.as_str(),
                    side.lower_texture.as_str(),
                    side.middle_texture.as_str(),
                ]
            })
            .filter(|name| *name != "-")
            .collect::<BTreeSet<_>>();
        if !texture_names
            .iter()
            .any(|name| name.contains("BROWN") || name.contains("DOOR"))
        {
            continue;
        }
        let Some(start) = map.vertices.get(usize::from(linedef.start_vertex)) else {
            continue;
        };
        let Some(end) = map.vertices.get(usize::from(linedef.end_vertex)) else {
            continue;
        };
        let midpoint = [
            (f32::from(start.x) + f32::from(end.x)) * 0.5,
            (f32::from(start.y) + f32::from(end.y)) * 0.5,
        ];
        let relative = [midpoint[0] - spawn[0], midpoint[1] - spawn[1]];
        let forward_offset = relative[0] * forward[0] + relative[1] * forward[1];
        let source_right_offset = relative[0] * right[0] + relative[1] * right[1];
        println!(
            "linedef={}; vertices={}({},{}) -> {}({},{}) ; textures={}; source-forward-offset={:.1}; source-right-offset={:.1}",
            linedef.source.record_index,
            linedef.start_vertex,
            start.x,
            start.y,
            linedef.end_vertex,
            end.x,
            end.y,
            texture_names.into_iter().collect::<Vec<_>>().join("|"),
            forward_offset,
            source_right_offset,
        );
    }
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

/// Reports the deterministic `Use` resolution of every nonzero E1M1 line.
/// This is source/request evidence only: accepted door intent is deliberately
/// not treated as a moved sector, and crossing-only specials remain visible.
fn report_doom_use_activation(source: &DoomLineActivationSource) {
    let mut accepted = 0;
    let mut no_special = 0;
    let mut wrong_activation = 0;
    let mut unsupported = 0;
    let mut invalid_target = 0;
    let mut details = Vec::new();
    for linedef in source
        .linedefs
        .iter()
        .filter(|linedef| linedef.special != 0)
    {
        let resolution = resolve_doom_line_activation(
            source,
            DoomLineActivationRequest {
                source_linedef: linedef.source,
                activation: DoomLineActivation::Use,
            },
        );
        match resolution {
            DoomLineActivationResolution::Accepted { intent, .. } => {
                accepted += 1;
                details.push(format!(
                    "{}:special{}:tag{}:accepted:{}:target:{}",
                    linedef.source.record_index,
                    linedef.special,
                    linedef.tag,
                    compact_activation_intent(intent),
                    compact_activation_target(intent),
                ));
            }
            DoomLineActivationResolution::NoSpecial { .. } => no_special += 1,
            DoomLineActivationResolution::WrongActivation { required, .. } => {
                wrong_activation += 1;
                details.push(format!(
                    "{}:special{}:tag{}:requires:{required:?}",
                    linedef.source.record_index, linedef.special, linedef.tag
                ));
            }
            DoomLineActivationResolution::UnsupportedSpecial { .. } => {
                unsupported += 1;
                details.push(format!(
                    "{}:special{}:tag{}:unsupported",
                    linedef.source.record_index, linedef.special, linedef.tag
                ));
            }
            DoomLineActivationResolution::UnknownLinedef { .. } => unreachable!(
                "a request derived from the retained E1M1 lines must resolve to one of them"
            ),
            DoomLineActivationResolution::MissingManualDoorTarget {
                missing_left_sidedef,
                ..
            } => {
                invalid_target += 1;
                details.push(format!(
                    "{}:special{}:missing-opposite-sidedef:{missing_left_sidedef:?}",
                    linedef.source.record_index, linedef.special
                ));
            }
            DoomLineActivationResolution::InvalidManualDoorTarget {
                sidedef_index,
                sector_index,
                ..
            } => {
                invalid_target += 1;
                details.push(format!(
                    "{}:special{}:invalid-target:sidedef{}:sector{}",
                    linedef.source.record_index, linedef.special, sidedef_index, sector_index
                ));
            }
        }
    }
    println!(
        "E1M1 Slice 8 use-request observation: nonzero_linedefs={}; accepted={accepted}; no_special={no_special}; wrong_activation={wrong_activation}; unsupported={unsupported}; invalid_target={invalid_target}; accepted_effects_are_not_executed=true; details={}",
        details.len(),
        details.join(" | "),
    );
}

/// Runs each E1M1 manual-door intent through the corpus-local, deterministic
/// moving-sector state machine. It reports height transitions only: no mesh,
/// collision, input reach, or renderer state is changed by this evidence path.
fn report_doom_manual_door_runtime(source: &DoomLineActivationSource) {
    let mut started = 0;
    let mut rejected = 0;
    let mut details = Vec::new();
    for linedef in source.linedefs.iter().filter(|line| line.special == 1) {
        let DoomLineActivationResolution::Accepted {
            intent: DoomLineActivationIntent::RaiseDoor { target_sector },
            ..
        } = resolve_doom_line_activation(
            source,
            DoomLineActivationRequest {
                source_linedef: linedef.source,
                activation: DoomLineActivation::Use,
            },
        )
        else {
            unreachable!("classified E1M1 code-1 lines must resolve to manual-door intent");
        };
        let mut door = match DoomManualDoorRuntime::start(
            source,
            target_sector,
            DoomManualDoorPolicy::CLASSIC_NORMAL,
        ) {
            Ok(door) => door,
            Err(error) => {
                rejected += 1;
                details.push(format!(
                    "line{}:target-sector{}:start-rejected:{error:?}",
                    linedef.source.record_index, target_sector.record_index
                ));
                continue;
            }
        };
        started += 1;
        let mut ticks = 0_u32;
        let mut reached_waiting = false;
        while door.phase != DoomManualDoorPhase::Closed && ticks < 4_096 {
            let transition = door.advance_tick();
            reached_waiting |=
                matches!(transition.after_phase, DoomManualDoorPhase::Waiting { .. });
            ticks += 1;
        }
        details.push(format!(
            "line{}:target-sector{}:closed-height{}:open-height{}:ticks{}:waited{}:final={:?}",
            linedef.source.record_index,
            target_sector.record_index,
            door.closed_ceiling_height,
            door.open_ceiling_height,
            ticks,
            reached_waiting,
            door.phase,
        ));
    }
    println!(
        "E1M1 Slice 8 manual-door runtime observation: code1_lines={}; started={started}; start_rejected={rejected}; source_map_mutated=false; presentation_mutated=false; details={}",
        details.len(),
        details.join(" | "),
    );
}

/// Replays the exact dynamic-resource lifetime that exposed the E1M1 handle
/// collision: a closed `DOORTRAK` span is absent, opening materializes it,
/// closing suppresses it, and reopening must reuse the original dynamic
/// identities. This is a no-window, corpus-only observation rather than a
/// renderer lifecycle contract.
fn report_door_resource_replay(app: &mut App) -> PlatformResult<()> {
    let Some(source_linedef) = app
        .activation_source
        .linedefs
        .iter()
        .find(|linedef| linedef.special == 1)
        .map(|linedef| linedef.source)
    else {
        return Err("E1M1 contains no code-1 manual door for resource replay".into());
    };
    let DoomLineActivationResolution::Accepted {
        intent: DoomLineActivationIntent::RaiseDoor { target_sector },
        ..
    } = resolve_doom_line_activation(
        &app.activation_source,
        DoomLineActivationRequest {
            source_linedef,
            activation: DoomLineActivation::Use,
        },
    )
    else {
        return Err("E1M1 code-1 manual door did not resolve for resource replay".into());
    };

    let mut door = DoomManualDoorRuntime::start(
        &app.activation_source,
        target_sector,
        DoomManualDoorPolicy::CLASSIC_NORMAL,
    )
    .map_err(|error| io::Error::other(format!("manual-door replay start failed: {error:?}")))?;
    let closed_height = door.closed_ceiling_height;
    let open_height = door.open_ceiling_height;
    app.active_manual_doors.push(door);

    app.refresh_active_manual_door_wall_meshes()?;
    let closed_initial_draws = app.dynamic_door_draws.len();
    let closed_initial_handles = app.dynamic_door_mesh_handles.clone();

    door = app.active_manual_doors[0];
    door.current_ceiling_height = open_height;
    door.phase = DoomManualDoorPhase::Waiting { remaining_ticks: 1 };
    app.active_manual_doors[0] = door;
    app.refresh_active_manual_door_wall_meshes()?;
    let opened_handles = app.dynamic_door_mesh_handles.clone();
    let opened_sources = app
        .dynamic_door_draws
        .iter()
        .map(|index| format!("{index}:{}", app.draws[*index].source_label))
        .collect::<Vec<_>>();
    let opened_enabled = app
        .dynamic_door_draws
        .iter()
        .filter(|index| app.opaque_draw_enabled[**index])
        .count();

    door = app.active_manual_doors[0];
    door.current_ceiling_height = closed_height;
    door.phase = DoomManualDoorPhase::Closed;
    app.active_manual_doors[0] = door;
    app.refresh_active_manual_door_wall_meshes()?;
    let closed_suppressed = app
        .dynamic_door_draws
        .iter()
        .filter(|index| !app.opaque_draw_enabled[**index])
        .count();

    door = app.active_manual_doors[0];
    door.current_ceiling_height = open_height;
    door.phase = DoomManualDoorPhase::Waiting { remaining_ticks: 1 };
    app.active_manual_doors[0] = door;
    app.refresh_active_manual_door_wall_meshes()?;
    let reopened_handles = app.dynamic_door_mesh_handles.clone();
    let reopened_enabled = app
        .dynamic_door_draws
        .iter()
        .filter(|index| app.opaque_draw_enabled[**index])
        .count();
    let cutout_last_handle = app
        .include_cutouts
        .then_some(app.cutout_mesh_base + app.cutout_draws.len() as u64 - 1);
    let dynamic_handles_are_after_cutouts = opened_handles
        .values()
        .all(|handle| cutout_last_handle.is_none_or(|cutout| handle.0 > cutout));

    println!(
        "E1M1 Slice 1 dynamic-resource replay: linedef={}; target-sector={}; closed-initial-draws={closed_initial_draws}; closed-initial-handles={}; opened-handles={:?}; opened-sources={}; opened-enabled={opened_enabled}; closed-suppressed={closed_suppressed}; reopened-handles={:?}; reopened-enabled={reopened_enabled}; stable-reopen={}; dynamic-after-cutouts={dynamic_handles_are_after_cutouts}; cutout-last-handle={cutout_last_handle:?}; source-map-mutated=false; renderer-initialized=false",
        source_linedef.record_index,
        target_sector.record_index,
        closed_initial_handles.len(),
        opened_handles,
        opened_sources.join(" | "),
        reopened_handles,
        opened_handles == reopened_handles,
    );
    Ok(())
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
        StaticDrawSource::Wall { source_linedef, .. } => linedef_subsectors
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

/// Corpus-local presentation lowering for one active manual-door ceiling flat.
/// Wall spans are re-lowered from retained source data rather than deformed in
/// place, preserving the distinction between height changes and texture-span
/// policy.
fn apply_door_ceiling_flat_height(
    draws: &mut [StaticDrawPlanEntry],
    target_sector: doom_map_provider::DoomSourceRecord,
    previous_height: i16,
    next_height: i16,
) -> Vec<usize> {
    let previous = f32::from(previous_height);
    let next = f32::from(next_height);
    let mut changed = Vec::new();
    for (index, draw) in draws.iter_mut().enumerate() {
        let is_target_ceiling = matches!(
            draw.source,
            StaticDrawSource::Flat {
                source_sector,
                plane: doom_geometry_provider::DoomSurfacePlane::Ceiling,
                ..
            } if source_sector == target_sector
        );
        if !is_target_ceiling {
            continue;
        }
        let mut modified = false;
        for position in &mut draw.mesh.positions {
            if (position[1] - previous).abs() <= f32::EPSILON {
                position[1] = next;
                modified = true;
            }
        }
        if modified {
            changed.push(index);
        }
    }
    changed
}

fn dynamic_wall_triangle_key(
    source_linedef: doom_map_provider::DoomSourceRecord,
    source_sidedef: doom_map_provider::DoomSourceRecord,
    source_sector: doom_map_provider::DoomSourceRecord,
    role: doom_geometry_provider::DoomWallTextureRole,
    texture_name: &str,
) -> String {
    format!(
        "{}/{}/{}/{}/{}/{}/{:?}/{texture_name}",
        source_linedef.lump_index,
        source_linedef.record_index,
        source_sidedef.lump_index,
        source_sidedef.record_index,
        source_sector.lump_index,
        source_sector.record_index,
        role,
    )
}

fn static_wall_triangle_key(draw: &StaticDrawPlanEntry) -> Option<String> {
    let StaticDrawSource::Wall {
        source_linedef,
        source_sidedef,
        source_sector,
        role,
    } = draw.source
    else {
        return None;
    };
    let (_, texture_name) = draw.source_label.rsplit_once(':')?;
    Some(dynamic_wall_triangle_key(
        source_linedef,
        source_sidedef,
        source_sector,
        role,
        texture_name,
    ))
}

fn is_door_mesh_for_target(
    draw: &StaticDrawPlanEntry,
    target_sector: doom_map_provider::DoomSourceRecord,
    boundary_linedefs: &[doom_map_provider::DoomSourceRecord],
) -> bool {
    match draw.source {
        StaticDrawSource::Flat {
            source_sector,
            plane: doom_geometry_provider::DoomSurfacePlane::Ceiling,
            ..
        } => source_sector == target_sector,
        StaticDrawSource::Wall { source_sector, .. } if source_sector == target_sector => true,
        StaticDrawSource::Wall {
            source_linedef,
            role: doom_geometry_provider::DoomWallTextureRole::Upper,
            ..
        } => boundary_linedefs.contains(&source_linedef),
        StaticDrawSource::Wall { .. } => false,
        StaticDrawSource::Flat { .. } => false,
    }
}

/// Returns the source linedefs which bound an active manual-door sector. The
/// result remains Doom-corpus evidence: the visual lowerer receives only these
/// retained identities, not a generalized portal or moving-wall contract.
fn manual_door_boundary_linedefs(
    source: &DoomLineActivationSource,
    target_sector: doom_map_provider::DoomSourceRecord,
) -> Vec<doom_map_provider::DoomSourceRecord> {
    source
        .linedefs
        .iter()
        .filter(|line| {
            [line.right_sidedef, line.left_sidedef]
                .into_iter()
                .flatten()
                .filter_map(|sidedef| source.sidedefs.get(usize::from(sidedef)))
                .filter_map(|sidedef| source.sectors.get(usize::from(sidedef.sector)))
                .any(|sector| sector.source == target_sector)
        })
        .map(|line| line.source)
        .collect()
}

/// Determines the source textures which become geometrically relevant at the
/// fully-open height of presently classified manual doors. This admits no
/// extra renderer behavior: it only makes their ordinary texture/material
/// inputs available before a runtime door can create the corresponding spans.
fn manual_door_dynamic_wall_texture_names(
    map: &DoomMapCore,
    source: &DoomLineActivationSource,
    extents: &[DoomTextureExtent],
) -> Result<Vec<String>, io::Error> {
    let mut open_map = map.clone();
    let mut targets = Vec::new();
    for line in &source.linedefs {
        let DoomLineActivationResolution::Accepted {
            intent: DoomLineActivationIntent::RaiseDoor { target_sector },
            ..
        } = resolve_doom_line_activation(
            source,
            DoomLineActivationRequest {
                source_linedef: line.source,
                activation: DoomLineActivation::Use,
            },
        )
        else {
            continue;
        };
        if targets.contains(&target_sector) {
            continue;
        }
        let door = DoomManualDoorRuntime::start(
            source,
            target_sector,
            DoomManualDoorPolicy::CLASSIC_NORMAL,
        )
        .map_err(|error| io::Error::other(format!("manual-door preparation failed: {error:?}")))?;
        let sector = open_map
            .sectors
            .iter_mut()
            .find(|sector| sector.source == target_sector)
            .ok_or_else(|| io::Error::other("manual-door target disappeared from decoded map"))?;
        sector.ceiling_height = door.open_ceiling_height;
        targets.push(target_sector);
    }

    let triangles = lower_doom_textured_wall_triangles(&open_map, extents).map_err(|error| {
        io::Error::other(format!("manual-door span preparation failed: {error}"))
    })?;
    let mut names = triangles
        .into_iter()
        .filter(|triangle| {
            targets.iter().any(|target_sector| {
                triangle.source_sector == *target_sector
                    || (triangle.role == doom_geometry_provider::DoomWallTextureRole::Upper
                        && manual_door_boundary_linedefs(source, *target_sector)
                            .contains(&triangle.source_linedef))
            })
        })
        .map(|triangle| triangle.texture_name)
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    Ok(names)
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

fn ray_triangle_distance(origin: Vec3, direction: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Option<f32> {
    const EPSILON: f32 = 1.0e-6;
    let edge_ab = b - a;
    let edge_ac = c - a;
    let perpendicular = direction.cross(edge_ac);
    let determinant = edge_ab.dot(perpendicular);
    if determinant.abs() <= EPSILON {
        return None;
    }

    let inverse_determinant = determinant.recip();
    let from_a = origin - a;
    let u = from_a.dot(perpendicular) * inverse_determinant;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let cross = from_a.cross(edge_ab);
    let v = direction.dot(cross) * inverse_determinant;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let distance = edge_ac.dot(cross) * inverse_determinant;
    (distance > EPSILON && distance.is_finite()).then_some(distance)
}

/// Returns the closest exact triangle hit in one prepared mesh. This remains
/// corpus-local inspection machinery; callers retain the source identity from
/// the owning draw rather than treating the mesh as a generic picking object.
fn nearest_mesh_ray_hit(origin: Vec3, direction: Vec3, mesh: &Mesh) -> Option<f32> {
    mesh.positions
        .chunks_exact(3)
        .filter_map(|triangle| {
            ray_triangle_distance(
                origin,
                direction,
                Vec3::from_array(triangle[0]),
                Vec3::from_array(triangle[1]),
                Vec3::from_array(triangle[2]),
            )
        })
        .min_by(f32::total_cmp)
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
        apply_observer_look_delta, candidate_is_selected, nearest_mesh_ray_hit,
        ray_triangle_distance, summarize_grouped_aabb_selection, CandidateSelection,
        CandidateSelectionSummary, ObserverLook, UniformGridAabbIndex,
    };
    use doom_geometry_provider::doom_point_to_tokimu;
    use hello_doom_e1m1::{
        classify_static_draw_frustum_rejection, classify_static_draw_sphere_frustum_rejection,
        doom_heading_degrees_to_observer_yaw, doom_heading_forward, observer_direction,
        observer_right, observer_yaw_from_forward, observer_yaw_to_doom_heading_degrees,
        reembed_comparative_mesh, DoomComparativeEmbedding, StaticDrawAabb,
        StaticDrawFrustumRejection, StaticDrawSphere,
    };
    use tokimu::Mesh;
    use tokimu_core::math::{Mat4, Vec3};

    #[test]
    fn center_ray_reports_an_exact_triangle_hit_distance() {
        let distance = ray_triangle_distance(
            Vec3::ZERO,
            Vec3::Z,
            Vec3::new(-1.0, -1.0, 5.0),
            Vec3::new(1.0, -1.0, 5.0),
            Vec3::new(0.0, 1.0, 5.0),
        )
        .expect("center ray should hit the fixture triangle");
        assert!((distance - 5.0).abs() < 0.000_1);
    }

    #[test]
    fn center_ray_rejects_a_triangle_outside_the_ray() {
        assert!(ray_triangle_distance(
            Vec3::ZERO,
            Vec3::Z,
            Vec3::new(2.0, -1.0, 5.0),
            Vec3::new(4.0, -1.0, 5.0),
            Vec3::new(3.0, 1.0, 5.0),
        )
        .is_none());
    }

    #[test]
    fn candidate_embeddings_preserve_exact_picking_distance() {
        let source_mesh = Mesh::uniform_normal(
            vec![[-1.0, -1.0, 5.0], [1.0, -1.0, 5.0], [0.0, 1.0, 5.0]],
            [0.0, 0.0, -1.0],
        )
        .with_texture_coordinates(vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]])
        .unwrap();
        let source_distance = nearest_mesh_ray_hit(Vec3::ZERO, Vec3::Z, &source_mesh).unwrap();

        for embedding in [
            DoomComparativeEmbedding::PreserveEast,
            DoomComparativeEmbedding::PreserveNorth,
        ] {
            let mut candidate_mesh = source_mesh.clone();
            reembed_comparative_mesh(&mut candidate_mesh, embedding, false);
            let candidate_direction = embedding.lift_direction([0.0, 1.0], 0.0);
            let candidate_distance =
                nearest_mesh_ray_hit(Vec3::ZERO, candidate_direction, &candidate_mesh).unwrap();
            assert!((candidate_distance - source_distance).abs() < 0.000_1);
        }
    }

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
