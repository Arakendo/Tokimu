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
    clip_doom_seg_textured_wall_triangle_to_linedef_interval, doom_point_to_tokimu,
    locate_doom_point_subsector, lower_doom_paired_sky_boundary_triangles,
    lower_doom_seg_textured_wall_triangles, lower_doom_textured_wall_triangles,
    observe_doom_classic_bsp,
    observe_doom_classic_vertical_clip_state as observe_shared_doom_classic_vertical_clip_state,
    observe_doom_seg_occluders, observe_doom_seg_plane_marks, project_doom_sector_runtime_heights,
    reconstruct_doom_ordered_wall_fragments, resolve_doom_linedef_subsector_membership,
    resolve_doom_subsector_bsp_paths, resolve_doom_subsector_regions,
    resolve_doom_subsector_sector_ownership, resolve_doom_viewer_subsector_order,
    resolve_doom_wall_candidates, DoomClassicBspObservation, DoomSectorRuntimeHeightSnapshot,
    DoomSegClassicPlaneKind, DoomSegClassicPlaneSpanObservation,
    DoomSegClassicVerticalClipObservation, DoomSegPlaneMarkObservation,
    DoomSegTexturedWallTriangle, DoomSurfacePlane, DoomTextureExtent, DoomWallTextureRole,
};
#[cfg(test)]
use doom_geometry_provider::{DoomSegClassicPlaneInstance, DoomSegClassicPlaneKey};
#[cfg(test)]
use doom_map_provider::DoomBspChild;
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
    resolve_doom_line_activation, DoomDownWaitUpStayPhase, DoomDownWaitUpStayPolicy,
    DoomDownWaitUpStayRuntime, DoomLineActivation, DoomLineActivationIntent,
    DoomLineActivationRequest, DoomLineActivationResolution, DoomLineActivationSource,
    DoomManualDoorPhase, DoomManualDoorPolicy, DoomManualDoorRuntime, DoomTurboLowerFloorPhase,
    DoomTurboLowerFloorPolicy, DoomTurboLowerFloorRuntime,
};
use hello_doom_e1m1::{
    build_experimental_cutout_draw_plan, build_experimental_cutout_texture_uploads,
    build_static_draw_plan, build_static_texture_uploads, classify_static_draw_frustum_rejection,
    classify_static_draw_sphere_frustum_rejection, doom_heading_forward,
    lower_static_seg_wall_triangle, lower_static_wall_triangle,
    observe_doom_ground_frame_with_embedding, observer_direction, observer_right,
    observer_yaw_from_forward, prepare_e1m1_flat_textures, prepare_e1m1_flats,
    prepare_e1m1_masked_middle_cutouts, prepare_e1m1_sky_diagnostic_flats,
    prepare_e1m1_static_sky_panorama_texture, prepare_e1m1_wall_texture_extents,
    prepare_e1m1_wall_textures, prepare_e1m1_walls, prepared_e1m1_masked_middle_texture_names,
    reembed_comparative_mesh, DoomComparativeEmbedding, PreparedStaticTexture, StaticDrawAabb,
    StaticDrawPlanEntry, StaticDrawSource, StaticFlatLoweringError, StaticTextureEligibility,
    StaticTextureUpload,
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

/// Compatibility name retained while E1M1 migrates from its former
/// executable-local observation to the shared Doom-provider result.
type DoomSegClassicBspObservation = DoomClassicBspObservation;
use ui_tools::provider::{UiFontRasterizer, UiFontSource};
use winit::window::CursorGrabMode;

#[path = "static_scene/observer.rs"]
mod observer;

#[path = "static_scene/candidate_selection/mod.rs"]
mod candidate_selection;
#[path = "static_scene/controls/mod.rs"]
mod controls;
#[path = "static_scene/diagnostics/mod.rs"]
mod diagnostics;
#[path = "static_scene/presentation/mod.rs"]
mod presentation;
#[path = "static_scene/runtime/mod.rs"]
mod runtime;

use candidate_selection::*;
use controls::{inspection_movement_delta, release_navigation_keys};
use diagnostics::*;
use presentation::*;
use runtime::*;

use observer::{
    apply_look_delta, doom_source_pose as observer_doom_source_pose, scene_camera, ObserverLook,
    SpawnObserver,
};

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
const DOOM_SKY_TEXTURE: TextureHandle = TextureHandle(9_000_020);
const DOOM_SKY_MATERIAL: MaterialHandle = MaterialHandle(9_000_020);
const DOOM_SKY_MESH: MeshHandle = MeshHandle(9_000_020);
const DOOM_SKY_BOUNDARY_MATERIAL: MaterialHandle = MaterialHandle(9_000_021);
const DOOM_SOURCE_SKY_PLANE_MESH_BASE: u64 = 9_002_000;
const DOOM_VIEWER_SKY_SPAN_MESH: MeshHandle = MeshHandle(9_003_000);
const WALK_SPEED: f32 = 240.0;
const RUN_SPEED_MULTIPLIER: f32 = 2.0;
const WALK_RADIUS: f32 = 16.0;
// id Software's released `p_local.h` declares USERANGE as 64 map units.

// This remains a Doom-corpus interaction policy, not a generic Tokimu reach.
const CLASSIC_USE_RANGE: f32 = 64.0;
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
    doom_sky_texture: PreparedStaticTexture,
    doom_sky_mesh: Mesh,
    doom_sky_boundary_draws: Vec<DoomSkyBoundaryDepthDraw>,
    doom_sky_enabled: bool,
    source_sky_plane_depth_enabled: bool,
    source_sky_plane_depth_global_control: bool,
    source_sky_plane_selected: Vec<bool>,
    cutout_mesh_base: u64,
    include_cutouts: bool,
    pipeline: PipelineHandle,
    cutout_pipeline: Option<PipelineHandle>,
    doom_sky_pipeline: Option<PipelineHandle>,
    doom_sky_boundary_pipeline: Option<PipelineHandle>,
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
    active_turbo_floors: Vec<DoomTurboLowerFloorRuntime>,
    active_down_wait_up_platforms: Vec<DoomDownWaitUpStayRuntime>,
    consumed_one_shot_cross_lines: BTreeSet<u32>,
    moving_floor_tick_accumulator: f64,
    dirty_opaque_meshes: HashSet<usize>,
    door_visual_diagnostic: Option<String>,
    door_geometry_diagnostic: Option<String>,
    dynamic_door_draws: BTreeSet<usize>,
    dynamic_door_mesh_handles: BTreeMap<usize, MeshHandle>,
    next_dynamic_mesh_handle: u64,
    opaque_draw_enabled: Vec<bool>,
    candidate_selection: CandidateSelection,
    /// Corpus-local Stage 3B source selector. It maps retained Doom SEG
    /// identities to already-uploaded draw indices; it is not renderer state.
    doom_seg_dynamic_selection: Option<DoomSegDynamicSelectionInput>,
    frame_index: u64,
    exit_after_two_frames: bool,
    opaque_selected: Vec<bool>,
    cutout_selected: Vec<bool>,
    commands: Vec<RenderCommand>,
    window: Option<Arc<NativeWindow>>,
    mouse_captured: bool,
    input: InputState,
    comparative_embedding: DoomComparativeEmbedding,
    /// True when the renderer input is the complete declaration set emitted
    /// by the Slice 4B ordered source preparation rather than the global map
    /// shell. Candidate filtering, if any, is a later and separate stage.
    ordered_coverage_prepared: bool,
    /// The classic-plane presentation is reconstructed for exactly the
    /// source-spawn observer. Allowing ordinary free-look/movement would make
    /// geometry outside that retained view look like reconstruction evidence.
    fixed_reconstruction_camera: bool,
}

struct SceneInput {
    opaque_draws: Vec<StaticDrawPlanEntry>,
    opaque_uploads: Vec<StaticTextureUpload>,
    cutout_draws: Vec<StaticDrawPlanEntry>,
    cutout_uploads: Vec<StaticTextureUpload>,
    diagnostic_sky_draws: Vec<StaticDrawPlanEntry>,
    diagnostic_sky_records: Vec<String>,
    doom_sky_texture: PreparedStaticTexture,
    doom_sky_boundary_draws: Vec<DoomSkyBoundaryDepthDraw>,
    spawn_observer: SpawnObserver,
    walk_collision: DoomWalkCollisionWorld,
    walk_floors: DoomWalkFloorWorld,
    reject_report: DoomRejectReport,
    topology_report: DoomTopologyReport,
    membership_selection: DoomMembershipSelectionInput,
    activation_source: DoomLineActivationSource,
    door_geometry_source: DoomDynamicDoorGeometrySource,
}

#[derive(Clone, Debug)]
struct DoomSkyBoundaryDepthDraw {
    source_linedef: doom_map_provider::DoomSourceRecord,
    source_sidedef: doom_map_provider::DoomSourceRecord,
    source_sector: doom_map_provider::DoomSourceRecord,
    mesh: Mesh,
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

#[derive(Clone, Copy, Debug, PartialEq)]
enum DoomUseTraceResult {
    Special { distance: f64, linedef: u32 },
    BackSide { distance: f64, linedef: u32 },
    Blocked { distance: f64, linedef: u32 },
    NoIntercept,
}

fn main() -> PlatformResult<()> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let preserve_east = args.iter().any(|argument| argument == "--embedding-east");
    let preserve_north = args.iter().any(|argument| argument == "--embedding-north");
    let current_reflected = args
        .iter()
        .any(|argument| argument == "--embedding-current-reflected");
    let comparative_embedding = match (preserve_east, preserve_north, current_reflected) {
        (false, false, false) => DoomComparativeEmbedding::PreserveNorth,
        (false, false, true) => DoomComparativeEmbedding::CurrentReflected,
        (true, false, false) => DoomComparativeEmbedding::PreserveEast,
        (false, true, false) => DoomComparativeEmbedding::PreserveNorth,
        _ => return Err("choose only one comparative embedding".into()),
    };
    let include_cutouts = !args
        .iter()
        .any(|argument| argument == "--no-masked-cutouts");

    let diagnostic_sky = args
        .iter()
        .any(|argument| argument == "--diagnostic-sky-omissions");
    let doom_sky = !diagnostic_sky && !args.iter().any(|argument| argument == "--no-doom-sky");
    let source_sky_plane_depth = doom_sky
        && args
            .iter()
            .any(|argument| argument == "--source-sky-plane-depth");
    let source_sky_plane_depth_global_control = doom_sky
        && args
            .iter()
            .any(|argument| argument == "--source-sky-plane-depth-global-control");
    let spawn_observer = !args.iter().any(|argument| argument == "--overview-camera");
    let spawn_yaw_plus_90 = args
        .iter()
        .any(|argument| argument == "--spawn-yaw-plus-90");
    let walk_collision = !args
        .iter()
        .any(|argument| argument == "--no-walk-collision");
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
    let moving_floor_runtime_report = args
        .iter()
        .any(|argument| argument == "--moving-floor-runtime-report");
    let moving_floor_resource_replay_report = args
        .iter()
        .any(|argument| argument == "--moving-floor-resource-replay-report");
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
    let hut_wall_candidates_report = args
        .iter()
        .any(|argument| argument == "--hut-wall-candidates-report");
    let doom_seg_report = args.iter().any(|argument| argument == "--doom-seg-report");
    let doom_seg_clip_report = args
        .iter()
        .any(|argument| argument == "--doom-seg-clip-report");
    let doom_hut_clip_report = args
        .iter()
        .any(|argument| argument == "--doom-hut-clip-report");
    let doom_seg_clip_grid_report = args
        .iter()
        .any(|argument| argument == "--doom-seg-clip-2d-report");
    let doom_seg_clip_per_column_report = args
        .iter()
        .any(|argument| argument == "--doom-seg-clip-per-column-report");
    let doom_seg_per_column_turn_trace = args
        .iter()
        .any(|argument| argument == "--doom-seg-per-column-turn-trace");
    let doom_seg_per_column_position_trace = args
        .iter()
        .any(|argument| argument == "--doom-seg-per-column-position-trace");
    let doom_seg_per_column_failure_trace = args
        .iter()
        .any(|argument| argument == "--doom-seg-per-column-failure-trace");
    let doom_seg_per_column_order_trace = args
        .iter()
        .any(|argument| argument == "--doom-seg-per-column-order-trace");
    let doom_seg_classic_admission_trace = args
        .iter()
        .any(|argument| argument == "--doom-seg-classic-admission-trace");
    let doom_seg_classic_bsp_trace = args
        .iter()
        .any(|argument| argument == "--doom-seg-classic-bsp-trace");
    let doom_seg_classic_vertical_clip_trace = args
        .iter()
        .any(|argument| argument == "--doom-seg-classic-vertical-clip-trace");
    let doom_seg_classic_plane_identity_trace = args
        .iter()
        .any(|argument| argument == "--doom-seg-classic-plane-identity-trace");
    let doom_seg_classic_plane_span_trace = args
        .iter()
        .any(|argument| argument == "--doom-seg-classic-plane-span-trace");
    let doom_seg_classic_plane_presentation = args
        .iter()
        .any(|argument| argument == "--doom-seg-classic-plane-presentation");
    let doom_seg_classic_context_presentation = args
        .iter()
        .any(|argument| argument == "--doom-seg-classic-context-presentation");
    let doom_seg_ordered_coverage_report = args
        .iter()
        .any(|argument| argument == "--doom-seg-ordered-coverage-report");
    let doom_seg_ordered_coverage_pose_matrix = args
        .iter()
        .any(|argument| argument == "--doom-seg-ordered-coverage-pose-matrix");
    let doom_seg_ordered_coverage_presentation = args
        .iter()
        .any(|argument| argument == "--doom-seg-ordered-coverage-presentation");
    let doom_seg_clip_presentation = args
        .iter()
        .any(|argument| argument == "--doom-seg-clip-presentation");
    let doom_seg_per_column_presentation = args
        .iter()
        .any(|argument| argument == "--doom-seg-per-column-presentation");
    let doom_seg_per_column_dynamic = args
        .iter()
        .any(|argument| argument == "--doom-seg-per-column-dynamic");
    let doom_seg_classic_dynamic = args
        .iter()
        .any(|argument| argument == "--doom-seg-classic-dynamic");
    if [
        doom_seg_clip_presentation,
        doom_seg_per_column_presentation,
        doom_seg_per_column_dynamic,
        doom_seg_classic_dynamic,
        doom_seg_classic_plane_presentation,
        doom_seg_classic_context_presentation,
        doom_seg_ordered_coverage_presentation,
    ]
    .iter()
    .filter(|enabled| **enabled)
    .count()
        > 1
    {
        return Err("choose only one Stage 3B SEG presentation control".into());
    }
    let wall_source_report = args.iter().find_map(|argument| {
        argument
            .strip_prefix("--wall-source-report=")
            .and_then(|record| record.parse::<u32>().ok())
    });
    let look_ray_report = args
        .iter()
        .find_map(|argument| argument.strip_prefix("--look-ray-report="))
        .map(parse_source_look_ray)
        .transpose()?;
    args.retain(|argument| argument != "--masked-cutouts");
    args.retain(|argument| argument != "--no-masked-cutouts");
    args.retain(|argument| argument != "--diagnostic-sky-omissions");
    args.retain(|argument| argument != "--doom-sky");
    args.retain(|argument| argument != "--no-doom-sky");
    args.retain(|argument| argument != "--source-sky-plane-depth");
    args.retain(|argument| argument != "--source-sky-plane-depth-global-control");
    args.retain(|argument| argument != "--spawn-observer");
    args.retain(|argument| argument != "--overview-camera");
    args.retain(|argument| argument != "--spawn-yaw-plus-90");
    args.retain(|argument| argument != "--walk-collision");
    args.retain(|argument| argument != "--no-walk-collision");
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
    args.retain(|argument| argument != "--moving-floor-runtime-report");
    args.retain(|argument| argument != "--moving-floor-resource-replay-report");
    args.retain(|argument| argument != "--door-resource-replay-report");
    args.retain(|argument| argument != "--measure-two-frames");
    args.retain(|argument| argument != "--spatial-orientation-report");
    args.retain(|argument| argument != "--spatial-landmark-candidates-report");
    args.retain(|argument| argument != "--spatial-flat-uv-report");
    args.retain(|argument| argument != "--hut-wall-candidates-report");
    args.retain(|argument| argument != "--doom-seg-report");
    args.retain(|argument| argument != "--doom-seg-clip-report");
    args.retain(|argument| argument != "--doom-hut-clip-report");
    args.retain(|argument| argument != "--doom-seg-clip-2d-report");
    args.retain(|argument| argument != "--doom-seg-clip-per-column-report");
    args.retain(|argument| argument != "--doom-seg-per-column-turn-trace");
    args.retain(|argument| argument != "--doom-seg-per-column-position-trace");
    args.retain(|argument| argument != "--doom-seg-per-column-failure-trace");
    args.retain(|argument| argument != "--doom-seg-per-column-order-trace");
    args.retain(|argument| argument != "--doom-seg-classic-admission-trace");
    args.retain(|argument| argument != "--doom-seg-classic-bsp-trace");
    args.retain(|argument| argument != "--doom-seg-classic-vertical-clip-trace");
    args.retain(|argument| argument != "--doom-seg-classic-plane-identity-trace");
    args.retain(|argument| argument != "--doom-seg-classic-plane-span-trace");
    args.retain(|argument| argument != "--doom-seg-classic-plane-presentation");
    args.retain(|argument| argument != "--doom-seg-classic-context-presentation");
    args.retain(|argument| argument != "--doom-seg-ordered-coverage-report");
    args.retain(|argument| argument != "--doom-seg-ordered-coverage-pose-matrix");
    args.retain(|argument| argument != "--doom-seg-ordered-coverage-presentation");

    args.retain(|argument| argument != "--doom-seg-clip-presentation");
    args.retain(|argument| argument != "--doom-seg-per-column-presentation");
    args.retain(|argument| argument != "--doom-seg-per-column-dynamic");
    args.retain(|argument| argument != "--doom-seg-classic-dynamic");
    args.retain(|argument| !argument.starts_with("--wall-source-report="));
    args.retain(|argument| !argument.starts_with("--look-ray-report="));
    args.retain(|argument| argument != "--embedding-east");
    args.retain(|argument| argument != "--embedding-north");
    args.retain(|argument| argument != "--embedding-current-reflected");
    let [package, member] = args.as_slice() else {
        return Err(
            "usage: static_scene <canonical-doom-zip> <WAD-member-name> [--no-masked-cutouts] [--no-doom-sky|--diagnostic-sky-omissions] [--source-sky-plane-depth|--source-sky-plane-depth-global-control] [--overview-camera] [--spawn-yaw-plus-90] [--embedding-current-reflected|--embedding-east|--embedding-north] [--no-walk-collision] [--walk-collision-report] [--noclip] [--frustum-aabb] [--frustum-grid-8x4x8] [--doom-membership-union] [--doom-seg-per-column-dynamic|--doom-seg-classic-dynamic] [--candidate-report] [--candidate-turn-trace] [--candidate-position-trace] [--candidate-pathological-report] [--candidate-grid-report] [--candidate-temporal-report] [--doom-reject-report] [--doom-topology-report] [--doom-membership-report] [--doom-seg-report] [--doom-seg-classic-admission-trace|--doom-seg-classic-bsp-trace|--doom-seg-classic-vertical-clip-trace|--doom-seg-classic-plane-identity-trace|--doom-seg-classic-plane-span-trace|--doom-seg-classic-plane-presentation|--doom-seg-classic-context-presentation|--doom-seg-ordered-coverage-report|--doom-seg-ordered-coverage-pose-matrix|--doom-seg-ordered-coverage-presentation] [--flat-normal-report] [--special-activation-report] [--door-runtime-report] [--moving-floor-runtime-report|--moving-floor-resource-replay-report] [--door-resource-replay-report] [--spatial-orientation-report] [--spatial-landmark-candidates-report] [--spatial-flat-uv-report] [--hut-wall-candidates-report] [--wall-source-report=<linedef>] [--look-ray-report=<source-x,source-y,source-z,direction-x,direction-y,direction-z>] [--measure-two-frames]".into(),
        );
    };
    if (walk_collision || walk_collision_report) && !spawn_observer {
        return Err(
            "--walk-collision requires the source-spawn camera; omit --overview-camera".into(),
        );
    }
    if doom_seg_per_column_dynamic && !spawn_observer {
        return Err(
            "--doom-seg-per-column-dynamic requires the source-spawn observer; omit --overview-camera"
                .into(),
        );
    }
    if doom_seg_classic_dynamic && !spawn_observer {
        return Err(
            "--doom-seg-classic-dynamic requires the source-spawn observer; omit --overview-camera"
                .into(),
        );
    }
    if doom_seg_classic_plane_presentation && !spawn_observer {
        return Err(
            "--doom-seg-classic-plane-presentation requires the source-spawn observer; omit --overview-camera"
                .into(),
        );
    }
    if doom_seg_classic_context_presentation && !spawn_observer {
        return Err(
            "--doom-seg-classic-context-presentation requires the source-spawn observer; omit --overview-camera"
                .into(),
        );
    }
    if (doom_seg_ordered_coverage_report || doom_seg_ordered_coverage_presentation)
        && !spawn_observer
    {
        return Err(
            "ordered coverage comparison requires the source-spawn observer; omit --overview-camera"
                .into(),
        );
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
    if hut_wall_candidates_report {
        report_hut_wall_candidates(&scene);
        return Ok(());
    }
    if doom_seg_report {
        report_doom_seg_lowering(&scene)?;
        return Ok(());
    }
    if doom_seg_clip_report {
        report_doom_seg_screen_clip(&scene, false)?;
        return Ok(());
    }
    if doom_hut_clip_report {
        report_doom_seg_screen_clip(&scene, true)?;
        return Ok(());
    }
    if doom_seg_clip_grid_report {
        report_doom_seg_screen_grid(&scene, false)?;
        return Ok(());
    }
    if doom_seg_clip_per_column_report {
        report_doom_seg_screen_grid(&scene, true)?;
        return Ok(());
    }
    if doom_seg_per_column_turn_trace {
        report_doom_seg_per_column_turn_trace(&scene)?;
        return Ok(());
    }
    if doom_seg_per_column_position_trace {
        report_doom_seg_per_column_position_trace(&scene)?;
        return Ok(());
    }
    if doom_seg_per_column_failure_trace {
        report_doom_seg_per_column_failure_trace(&scene)?;
        return Ok(());
    }
    if doom_seg_per_column_order_trace {
        report_doom_seg_per_column_order_trace(&scene)?;

        return Ok(());
    }
    if doom_seg_classic_admission_trace {
        report_doom_seg_classic_admission_trace(&scene)?;
        return Ok(());
    }
    if doom_seg_classic_bsp_trace {
        report_doom_seg_classic_bsp_trace(&scene)?;
        return Ok(());
    }
    if doom_seg_classic_vertical_clip_trace {
        report_doom_seg_classic_vertical_clip_trace(&scene)?;
        return Ok(());
    }

    if doom_seg_classic_plane_identity_trace {
        report_doom_seg_classic_plane_identity_trace(&scene)?;
        return Ok(());
    }
    if doom_seg_classic_plane_span_trace {
        report_doom_seg_classic_plane_span_trace(&scene)?;
        return Ok(());
    }
    if doom_seg_ordered_coverage_report {
        let presentation = prepare_doom_seg_ordered_coverage_presentation(&scene)?;
        eprintln!(
            "E1M1 AR-0025 Slice 7 ordered-coverage report: wall-conservation=[retained-cells:{} reconstructed-triangles:{} lowered-triangles:{} source-degenerate-cells:{} source-unresolved-cells:{} lowering-degenerate-triangles:{} lowering-unresolved-triangles:{}]; grouped-wall-meshes={}; opaque-draws={}; cutout-draws={}; plane-conservation=[ordinary:{} reconstructed:{} rejected:{} lowered:{}]; sky-background-intervals={}; cutout-key-conservation={}/{}; coverage=[transitions:{} fail-open:{} reasons:{:?}]; bsp=[leaves:{} far-pruned:{} admitted-segs:{} solid-range-pruning:{}]; degenerate-omissions={}; unresolved-contributions={}; samples={:?}; meaning=one-fixed-source-observation-lowered-to-complete-prepared-declarations",
            presentation.retained_cells,
            presentation.reconstructed_triangles,
            presentation.lowered_wall_triangles,
            presentation.source_degenerate_cells,
            presentation.source_unresolved_cells,
            presentation.lowering_degenerate_triangles,
            presentation.lowering_unresolved_triangles,
            presentation.grouped_wall_meshes,
            presentation.opaque_draws.len(),
            presentation.cutout_draws.len(),
            presentation.ordinary_plane_intervals,
            presentation.reconstructed_plane_quads,
            presentation.rejected_plane_intervals,
            presentation.lowered_plane_quads,
            presentation.sky_plane_intervals,
            presentation.lowered_cutout_keys,
            presentation.source_cutout_keys,
            presentation.coverage_transitions,
            presentation.coverage_fail_open,
            presentation.coverage_fail_open_reasons,
            presentation.bsp_leaves_visited,
            presentation.bsp_far_children_pruned,
            presentation.bsp_admitted_segs,
            presentation.bsp_solid_range_pruning,
            presentation.degenerate_omissions,
            presentation.unresolved_cells,
            presentation.samples,
        );
        return Ok(());
    }
    if doom_seg_ordered_coverage_pose_matrix {
        report_doom_seg_ordered_coverage_pose_matrix(&scene)?;
        return Ok(());
    }
    if let Some(linedef) = wall_source_report {
        report_wall_source(&scene, linedef);
        return Ok(());
    }
    if doom_seg_clip_presentation {
        let presentation = prepare_doom_seg_clip_presentation(&scene, false)?;
        eprintln!(
            "E1M1 AR-0025 Stage 3B visible-SEG presentation: visible_intervals={}; source_triangles={}; submitted_draws={}; meaning=diagnostic-source-space-screen-span-comparison-not-historic-doom-parity",
            presentation.visible_intervals,
            presentation.source_triangles,
            presentation.draws.len(),
        );
        scene.opaque_draws = presentation.draws;
        scene.cutout_draws.clear();
    }
    if doom_seg_per_column_presentation {
        let presentation = prepare_doom_seg_per_column_presentation(&scene)?;
        eprintln!(
            "E1M1 AR-0025 Stage 3B per-column SEG comparison: selected_segs={}; submitted_wall_draws={}; meaning=diagnostic-source-space-grid-comparison-not-historic-doom-parity",

            presentation.selected_segs,
            presentation.wall_draws.len(),
        );
        scene
            .opaque_draws
            .retain(|draw| !matches!(draw.source, StaticDrawSource::Wall { .. }));
        scene.opaque_draws.extend(presentation.wall_draws);
    }
    if doom_seg_classic_plane_presentation {
        let presentation = prepare_doom_seg_classic_plane_presentation(&scene)?;
        eprintln!(
            "E1M1 AR-0025 Stage 3B classic-plane presentation: source-cells={}; grouped-meshes={}; triangles={}; meaning=fixed-source-spawn-doom-plane-comparison-not-visplane-parity-or-renderer-visibility",
            presentation.source_cells,
            presentation.grouped_meshes,
            presentation.triangles,
        );
        scene.opaque_draws = presentation.draws;
        scene.cutout_draws.clear();
    }
    if doom_seg_classic_context_presentation {
        let presentation = prepare_doom_seg_classic_context_presentation(&scene)?;
        eprintln!(
            "E1M1 AR-0025 Stage 3B classic-context presentation: plane-meshes={}; plane-triangles={}; wall-meshes={}; omitted-wall-triangles={}; total-draws={}; meaning=fixed-source-spawn-context-control-with-whole-seg-wall-tiers-not-visplane-or-historic-pixel-parity",
            presentation.plane_meshes,
            presentation.plane_triangles,
            presentation.wall_meshes,
            presentation.omitted_wall_triangles,
            presentation.draws.len(),
        );
        scene.opaque_draws = presentation.draws;
        scene.cutout_draws.clear();
    }
    if doom_seg_ordered_coverage_presentation {
        let presentation = prepare_doom_seg_ordered_coverage_presentation(&scene)?;
        eprintln!(
            "E1M1 AR-0025 Slice 7 ordered-coverage presentation: renderer-input=prepared-full-submission; wall-conservation=[retained-cells:{} reconstructed-triangles:{} lowered-triangles:{} source-degenerate-cells:{} source-unresolved-cells:{} lowering-degenerate-triangles:{} lowering-unresolved-triangles:{}]; grouped-wall-meshes={}; opaque-draws={}; cutout-draws={}; plane-conservation=[ordinary:{} reconstructed:{} rejected:{} lowered:{}]; sky-background-intervals={}; cutout-key-conservation={}/{}; coverage=[transitions:{} fail-open:{} reasons:{:?}]; bsp=[leaves:{} far-pruned:{} admitted-segs:{} solid-range-pruning:{}]; degenerate-omissions={}; unresolved-contributions={}; samples={:?}; meaning=one-fixed-source-observation-lowered-to-complete-prepared-declarations",
            presentation.retained_cells,
            presentation.reconstructed_triangles,
            presentation.lowered_wall_triangles,
            presentation.source_degenerate_cells,
            presentation.source_unresolved_cells,
            presentation.lowering_degenerate_triangles,
            presentation.lowering_unresolved_triangles,
            presentation.grouped_wall_meshes,
            presentation.opaque_draws.len(),
            presentation.cutout_draws.len(),
            presentation.ordinary_plane_intervals,
            presentation.reconstructed_plane_quads,
            presentation.rejected_plane_intervals,
            presentation.lowered_plane_quads,
            presentation.sky_plane_intervals,
            presentation.lowered_cutout_keys,
            presentation.source_cutout_keys,
            presentation.coverage_transitions,
            presentation.coverage_fail_open,
            presentation.coverage_fail_open_reasons,
            presentation.bsp_leaves_visited,
            presentation.bsp_far_children_pruned,
            presentation.bsp_admitted_segs,
            presentation.bsp_solid_range_pruning,
            presentation.degenerate_omissions,
            presentation.unresolved_cells,
            presentation.samples,
        );
        scene.opaque_draws = presentation.opaque_draws;
        scene.cutout_draws = presentation.cutout_draws;
    }
    let doom_seg_dynamic_selection = if doom_seg_per_column_dynamic || doom_seg_classic_dynamic {
        let selection = prepare_doom_seg_per_column_dynamic_scene(&mut scene)?;
        eprintln!(
            "E1M1 AR-0025 Stage 3B dynamic SEG control: mode={}; retained_seg_records={}; retained_flat_subsectors={}; unsupported_textures={:?}; meaning=source-local-draw-enable-experiment-not-renderer-visibility",
            if doom_seg_classic_dynamic { "classic-bsp" } else { "per-column-grid" },
            selection.draw_indices_by_seg.len(),
            selection.flat_indices_by_subsector.len(),
            selection.unsupported_textures,
        );
        Some(selection)
    } else {
        None
    };
    reembed_scene_for_comparison(&mut scene, comparative_embedding);
    if let Some(ray) = look_ray_report {
        report_source_look_ray(&scene, comparative_embedding, ray, include_cutouts);
        return Ok(());
    }
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
    if moving_floor_runtime_report {
        report_doom_moving_floor_runtime(&scene.activation_source);
        return Ok(());
    }
    if walk_collision_report {
        report_walk_collision(&scene);
        return Ok(());
    }
    let include_cutouts = include_cutouts && !doom_seg_clip_presentation;
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
        }
        // Paired-sky boundary meshes remain retained source evidence, but
        // E1M1 proved that presenting them unconditionally can hide valid
        // foreground geometry (the hut from the spawn-room window). The
        // synthetic paired-sky fixture remains the bounded mechanism control.
        + if source_sky_plane_depth_global_control {
            scene.diagnostic_sky_draws.len()
        } else if source_sky_plane_depth {
            1
        } else {
            0
        }
        + usize::from(doom_sky);

    let opaque_selected = vec![true; scene.opaque_draws.len()];
    let cutout_selected = vec![true; scene.cutout_draws.len()];
    let source_sky_plane_selected = vec![false; scene.diagnostic_sky_draws.len()];
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
        doom_sky_texture: scene.doom_sky_texture,
        doom_sky_mesh: build_doom_sky_cylinder(center, radius).map_err(io::Error::other)?,
        doom_sky_boundary_draws: scene.doom_sky_boundary_draws,
        doom_sky_enabled: doom_sky,
        source_sky_plane_depth_enabled: source_sky_plane_depth,
        source_sky_plane_depth_global_control,
        source_sky_plane_selected,
        cutout_mesh_base,
        include_cutouts,
        pipeline: PipelineHandle(0),
        cutout_pipeline: None,
        doom_sky_pipeline: None,
        doom_sky_boundary_pipeline: None,
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
        active_turbo_floors: Vec::new(),
        active_down_wait_up_platforms: Vec::new(),
        consumed_one_shot_cross_lines: BTreeSet::new(),
        moving_floor_tick_accumulator: 0.0,
        dirty_opaque_meshes: HashSet::new(),
        door_visual_diagnostic: None,
        door_geometry_diagnostic: None,
        dynamic_door_draws: BTreeSet::new(),
        dynamic_door_mesh_handles: BTreeMap::new(),
        next_dynamic_mesh_handle: cutout_mesh_base + cutout_selected.len() as u64,
        opaque_draw_enabled: opaque_selected.clone(),
        candidate_selection: if doom_seg_classic_dynamic {
            CandidateSelection::DoomClassicBsp
        } else if frustum_grid {
            CandidateSelection::UniformGrid8x4x8
        } else if doom_membership_union {
            CandidateSelection::DoomMembershipUnion
        } else if doom_seg_per_column_dynamic {
            CandidateSelection::DoomSegPerColumn
        } else if frustum_aabb {
            CandidateSelection::FrustumAabb
        } else {
            CandidateSelection::FullSubmission
        },
        doom_seg_dynamic_selection,
        frame_index: 0,
        exit_after_two_frames: measure_two_frames,
        opaque_selected,
        cutout_selected,
        commands,
        window: None,
        mouse_captured: false,
        input: InputState::default(),
        comparative_embedding,
        ordered_coverage_prepared: doom_seg_ordered_coverage_presentation,
        fixed_reconstruction_camera: doom_seg_classic_plane_presentation
            || doom_seg_classic_context_presentation
            || doom_seg_ordered_coverage_presentation,
    };
    if door_resource_replay_report {
        report_door_resource_replay(&mut app)?;
        return Ok(());
    }
    if moving_floor_resource_replay_report {
        report_moving_floor_resource_replay(&mut app)?;
        return Ok(());
    }
    run_window_with_app(
        WindowConfig {
            title: format!(
                "Tokimu DOOM E1M1 | {draw_count} draws | {comparative_embedding:?}{}",
                if app.fixed_reconstruction_camera {
                    " | fixed-source-spawn"
                } else {
                    ""
                }
            ),
            width: 1280,
            height: 800,
        },
        app,
    )
}

fn select_masked_middle_owning_sides(
    draws: &[StaticDrawPlanEntry],
    observer_position: Vec3,
    selected: &mut [bool],
    summary: &mut CandidateSelectionSummary,
    rejection_samples: &mut Vec<String>,
    retain_samples: bool,
) {
    for (index, (draw, selected)) in draws.iter().zip(selected.iter_mut()).enumerate() {
        if !*selected || mesh_owning_side_visible(&draw.mesh, observer_position) {
            continue;
        }
        *selected = false;
        summary.submitted = summary.submitted.saturating_sub(1);
        summary.rejected += 1;
        if retain_samples && rejection_samples.len() < 12 {
            rejection_samples.push(format!(
                "{}:doom-masked-middle-non-owning-side:index={index}",
                draw.source_label
            ));
        }
    }
}

fn mesh_owning_side_visible(mesh: &Mesh, observer_position: Vec3) -> bool {
    let (Some(position), Some(normal)) = (mesh.positions.first(), mesh.normals.first()) else {
        return true;
    };
    let position = Vec3::from_array(*position);
    let normal = Vec3::from_array(*normal);
    normal.dot(observer_position - position) >= 0.0
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
    let doom_sky_boundary_draws = lower_doom_paired_sky_boundary_triangles(&map)?
        .into_iter()
        .map(|triangle| DoomSkyBoundaryDepthDraw {
            source_linedef: triangle.source_linedef,
            source_sidedef: triangle.source_sidedef,
            source_sector: triangle.source_sector,
            mesh: Mesh::uniform_normal(
                triangle
                    .positions
                    .into_iter()
                    .map(|position| position.map(|component| component as f32))
                    .collect(),
                [0.0, 1.0, 0.0],
            ),
        })
        .collect::<Vec<_>>();
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
    let doom_sky_texture = prepare_e1m1_static_sky_panorama_texture(
        &read.bytes,
        &read.observation.wad,
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
        doom_sky_texture,
        doom_sky_boundary_draws,
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
    for draw in &mut scene.doom_sky_boundary_draws {
        reembed_comparative_mesh(&mut draw.mesh, embedding, false);
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

/// Builds the deliberately bounded Stage 3B presentation control from the
/// same fixed source-space screen-span experiment reported above. This is not
/// a Doom renderer reconstruction: it only gives the corpus a visually
/// inspectable representation of the retained SEG subintervals.
fn prepare_doom_seg_clip_presentation(
    scene: &SceneInput,
    hut_pose: bool,
) -> PlatformResult<DoomSegClipPresentation> {
    const COLUMNS: usize = 320;
    const HALF_FOV: f64 = std::f64::consts::FRAC_PI_4;

    let map = &scene.door_geometry_source.map;
    let viewer = scene.spawn_observer.source_position;
    let angle = if hut_pose {
        (-208.0_f64).atan2(1120.0)
    } else {
        f64::from(scene.spawn_observer.source_angle).to_radians()
    };
    let forward = [angle.cos(), angle.sin()];
    let right = [-forward[1], forward[0]];
    let order = resolve_doom_viewer_subsector_order(map, viewer)?;
    let order_by_source = order
        .iter()
        .enumerate()
        .map(|(rank, source)| (source.record_index, rank))
        .collect::<BTreeMap<_, _>>();
    let occluders = observe_doom_seg_occluders(map)?
        .into_iter()
        .map(|observation| (observation.source_seg.record_index, observation))
        .collect::<BTreeMap<_, _>>();
    let seg_triangles =
        lower_doom_seg_textured_wall_triangles(map, &scene.door_geometry_source.wall_extents)?;
    let mut triangles_by_seg = BTreeMap::<u32, Vec<_>>::new();
    for triangle in &seg_triangles {
        triangles_by_seg
            .entry(triangle.source_seg.record_index)
            .or_default()
            .push(triangle);
    }
    let mut ordered_segs = map
        .segs
        .iter()
        .filter_map(|seg| {
            let subsector = map.subsectors.iter().position(|subsector| {
                let start = usize::from(subsector.first_seg);
                let end = start + usize::from(subsector.seg_count);
                (start..end).any(|index| map.segs[index].source == seg.source)
            })?;
            Some((
                order_by_source.get(&map.subsectors[subsector].source.record_index)?,
                seg,
            ))
        })
        .collect::<Vec<_>>();
    ordered_segs.sort_by_key(|(rank, seg)| (**rank, seg.source.record_index));

    let project = |point: [i16; 2]| {
        let relative = [
            f64::from(point[0] - viewer[0]),
            f64::from(point[1] - viewer[1]),
        ];
        let depth = relative[0] * forward[0] + relative[1] * forward[1];
        let lateral = relative[0] * right[0] + relative[1] * right[1];
        (depth, lateral.atan2(depth))
    };
    let column = |angle: f64| {
        ((angle.clamp(-HALF_FOV, HALF_FOV) + HALF_FOV) / (2.0 * HALF_FOV) * COLUMNS as f64) as usize
    };

    let mut covered = vec![false; COLUMNS];
    let mut draws = Vec::new();
    let mut visible_intervals = 0usize;
    let mut source_triangles = 0usize;
    for (_, seg) in ordered_segs {
        let start = &map.vertices[usize::from(seg.start_vertex)];
        let end = &map.vertices[usize::from(seg.end_vertex)];
        let (start_depth, start_angle) = project([start.x, start.y]);
        let (end_depth, end_angle) = project([end.x, end.y]);
        if start_depth <= 0.0
            || end_depth <= 0.0
            || (start_angle.abs() > HALF_FOV && end_angle.abs() > HALF_FOV)
        {
            continue;
        }
        let start_column = column(start_angle).min(COLUMNS - 1);
        let end_column = column(end_angle).min(COLUMNS - 1);
        let (left, right_column) = (start_column.min(end_column), start_column.max(end_column));
        let span = right_column - left + 1;
        let line_interval = source_seg_linedef_interval(map, seg);
        for [run_start, run_end] in visible_column_runs(&covered[left..=right_column]) {
            let start_fraction = run_start as f64 / span as f64;
            let end_fraction = run_end as f64 / span as f64;
            let interval = [
                line_interval[0] + (line_interval[1] - line_interval[0]) * start_fraction,
                line_interval[0] + (line_interval[1] - line_interval[0]) * end_fraction,
            ];
            visible_intervals += 1;
            for triangle in triangles_by_seg
                .get(&seg.source.record_index)
                .into_iter()
                .flatten()
            {
                let extent = scene
                    .door_geometry_source
                    .wall_extents
                    .iter()
                    .find(|extent| extent.name == triangle.texture_name)
                    .cloned()
                    .ok_or_else(|| {
                        io::Error::other(format!(
                            "Stage 3B visible SEG `{}` has no texture extent",
                            triangle.texture_name
                        ))
                    })?;
                for clipped in clip_doom_seg_textured_wall_triangle_to_linedef_interval(
                    map, triangle, interval,
                )? {
                    let lowered = lower_static_seg_wall_triangle(&clipped, extent.clone())?;
                    let material = scene
                        .door_geometry_source
                        .wall_materials
                        .get(&lowered.wall.texture_name)
                        .copied()
                        .ok_or_else(|| {
                            io::Error::other(format!(
                                "Stage 3B visible SEG `{}` has no wall material",
                                lowered.wall.texture_name
                            ))
                        })?;
                    source_triangles += 1;
                    draws.push(StaticDrawPlanEntry {
                        source_label: format!(
                            "seg-clip:{}:{}:{:?}:{:?}:{}:{:.3}-{:.3}",
                            lowered.source_seg.record_index,
                            lowered.wall.source_linedef.record_index,
                            lowered.wall.side,
                            lowered.wall.role,
                            lowered.wall.texture_name,
                            interval[0],
                            interval[1],
                        ),
                        source: StaticDrawSource::Wall {
                            source_linedef: lowered.wall.source_linedef,
                            source_sidedef: lowered.wall.source_sidedef,
                            source_sector: lowered.wall.source_sector,

                            role: lowered.wall.role,
                        },
                        mesh: lowered.wall.mesh,
                        material,
                    });
                }
            }
        }
        let authority = occluders
            .get(&seg.source.record_index)
            .expect("every source SEG is classified");
        if authority.kind != doom_geometry_provider::DoomSegOccluderKind::Open {
            covered[left..=right_column].fill(true);
        }
    }

    Ok(DoomSegClipPresentation {
        draws,
        visible_intervals,
        source_triangles,
    })
}

/// Adapts the retained per-column source-grid observation into ordinary
/// source-labelled wall draws for a manual comparison. A SEG survives as a
/// whole piece when at least one of its bounded grid cells remains uncovered;
/// this deliberately fails open rather than claiming pixel-exact clipping.
fn prepare_doom_seg_per_column_presentation(
    scene: &SceneInput,
) -> PlatformResult<DoomSegPerColumnPresentation> {
    let observation = observe_doom_seg_screen_grid(
        &scene.door_geometry_source.map,
        scene.spawn_observer.position.y,
        true,
        scene.spawn_observer.source_position,
        f64::from(scene.spawn_observer.source_angle).to_radians(),
    )?;
    let map = &scene.door_geometry_source.map;
    let triangles =
        lower_doom_seg_textured_wall_triangles(map, &scene.door_geometry_source.wall_extents)?;
    let mut wall_draws = Vec::new();
    for triangle in triangles.iter().filter(|triangle| {
        observation
            .selected_seg_records
            .contains(&triangle.source_seg.record_index)
    }) {
        let extent = scene
            .door_geometry_source
            .wall_extents
            .iter()
            .find(|extent| extent.name == triangle.texture_name)
            .cloned()
            .ok_or_else(|| {
                io::Error::other(format!(
                    "Stage 3B selected SEG `{}` has no texture extent",
                    triangle.texture_name
                ))
            })?;
        let lowered = match lower_static_seg_wall_triangle(triangle, extent) {
            Ok(lowered) => lowered,
            Err(StaticFlatLoweringError::DegenerateTriangle) => continue,
            Err(error) => return Err(error.into()),
        };
        let material = scene
            .door_geometry_source
            .wall_materials
            .get(&lowered.wall.texture_name)
            .copied()
            .ok_or_else(|| {
                io::Error::other(format!(
                    "Stage 3B selected SEG `{}` has no wall material",
                    lowered.wall.texture_name
                ))
            })?;
        wall_draws.push(StaticDrawPlanEntry {
            source_label: format!(
                "seg-grid:{}:{}:{:?}:{:?}:{}",
                lowered.source_seg.record_index,
                lowered.wall.source_linedef.record_index,
                lowered.wall.side,
                lowered.wall.role,
                lowered.wall.texture_name,
            ),
            source: StaticDrawSource::Wall {
                source_linedef: lowered.wall.source_linedef,
                source_sidedef: lowered.wall.source_sidedef,
                source_sector: lowered.wall.source_sector,
                role: lowered.wall.role,
            },
            mesh: lowered.wall.mesh,
            material,
        });
    }
    Ok(DoomSegPerColumnPresentation {
        wall_draws,
        selected_segs: observation.selected_seg_records.len(),
    })
}

/// Prepares every SEG-derived wall once, retaining the original flat/cutout
/// draws. The runtime control later filters this stable set by source SEG
/// identity, so observer movement cannot cause mesh uploads or replacements.
fn prepare_doom_seg_per_column_dynamic_scene(
    scene: &mut SceneInput,
) -> PlatformResult<DoomSegDynamicSelectionInput> {
    let map = &scene.door_geometry_source.map;
    let triangles =
        lower_doom_seg_textured_wall_triangles(map, &scene.door_geometry_source.wall_extents)?;
    let unsupported_linedefs = triangles
        .iter()
        .filter(|triangle| {
            !scene
                .door_geometry_source
                .wall_materials
                .contains_key(&triangle.texture_name)
        })
        .map(|triangle| triangle.source_linedef.record_index)
        .collect::<BTreeSet<_>>();
    scene.opaque_draws.retain(|draw| match draw.source {
        StaticDrawSource::Wall { source_linedef, .. } => {
            unsupported_linedefs.contains(&source_linedef.record_index)
        }
        _ => true,
    });
    let mut unsupported_textures = BTreeSet::new();
    for triangle in triangles {
        let extent = scene
            .door_geometry_source
            .wall_extents
            .iter()
            .find(|extent| extent.name == triangle.texture_name)
            .cloned()
            .ok_or_else(|| {
                io::Error::other(format!(
                    "Stage 3B dynamic SEG `{}` has no texture extent",
                    triangle.texture_name
                ))
            })?;
        let lowered = match lower_static_seg_wall_triangle(&triangle, extent) {
            Ok(lowered) => lowered,
            // Preserve the established E1M1 rule: confirmed zero-area source
            // candidates are retained omissions, never fabricated normals.
            Err(StaticFlatLoweringError::DegenerateTriangle) => continue,
            Err(error) => return Err(error.into()),
        };
        let Some(material) = scene
            .door_geometry_source
            .wall_materials
            .get(&lowered.wall.texture_name)
            .copied()
        else {
            unsupported_textures.insert(lowered.wall.texture_name);
            continue;
        };
        scene.opaque_draws.push(StaticDrawPlanEntry {
            source_label: format!(
                "seg-dynamic:{}:{}:{:?}:{:?}:{}",
                lowered.source_seg.record_index,
                lowered.wall.source_linedef.record_index,
                lowered.wall.side,
                lowered.wall.role,
                lowered.wall.texture_name,
            ),
            source: StaticDrawSource::Wall {
                source_linedef: lowered.wall.source_linedef,

                source_sidedef: lowered.wall.source_sidedef,
                source_sector: lowered.wall.source_sector,
                role: lowered.wall.role,
            },
            mesh: lowered.wall.mesh,
            material,
        });
    }
    let mut draw_indices_by_seg = BTreeMap::<u32, Vec<usize>>::new();
    let mut flat_indices_by_subsector = BTreeMap::<u32, Vec<usize>>::new();
    for (index, draw) in scene.opaque_draws.iter().enumerate() {
        if let Some(seg) = draw
            .source_label
            .strip_prefix("seg-dynamic:")
            .and_then(|label| label.split(':').next())
            .and_then(|record| record.parse::<u32>().ok())
        {
            draw_indices_by_seg.entry(seg).or_default().push(index);
        }
        if let StaticDrawSource::Flat {
            source_subsector, ..
        } = draw.source
        {
            flat_indices_by_subsector
                .entry(source_subsector.record_index)
                .or_default()
                .push(index);
        }
    }
    Ok(DoomSegDynamicSelectionInput {
        draw_indices_by_seg,
        flat_indices_by_subsector,
        unsupported_textures,
    })
}

fn observe_doom_seg_classic_plane_identities(
    map: &DoomMapCore,
    plane_marks: &[DoomSegPlaneMarkObservation],
    traversal: &DoomSegClassicBspObservation,
) -> DoomSegClassicPlaneIdentityObservation {
    let sectors_by_record = map
        .sectors
        .iter()
        .map(|sector| (sector.source.record_index, sector))
        .collect::<BTreeMap<_, _>>();
    let marks_by_seg = plane_marks
        .iter()
        .map(|mark| (mark.source_seg.record_index, mark))
        .collect::<BTreeMap<_, _>>();
    let mut result = DoomSegClassicPlaneIdentityObservation::default();
    let mut floor_keys = BTreeSet::new();
    let mut ceiling_keys = BTreeSet::new();

    for source_seg in &traversal.admitted_seg_order {
        let Some(mark) = marks_by_seg.get(source_seg) else {
            continue;
        };
        let sector = sectors_by_record
            .get(&mark.front_sector.record_index)
            .expect("validated plane mark names an existing front sector");
        if mark.floor_marked {
            result.floor_mark_contributors += 1;
            let key = (
                sector.floor_height,
                sector.floor_texture.clone(),
                sector.light_level,
            );
            if floor_keys.insert(key.clone()) && result.samples.len() < 12 {
                result.samples.push(format!(
                    "floor-sector={} height={} flat={} light={}",
                    mark.front_sector.record_index, key.0, key.1, key.2,
                ));
            }
        }
        if mark.ceiling_marked {
            result.ceiling_mark_contributors += 1;
            let sky = sector.ceiling_texture == "F_SKY1";
            result.sky_ceiling_contributors += usize::from(sky);
            let key = if sky {
                (0, String::from("F_SKY1"), 0)
            } else {
                (
                    sector.ceiling_height,
                    sector.ceiling_texture.clone(),
                    sector.light_level,
                )
            };
            if ceiling_keys.insert(key.clone()) && result.samples.len() < 12 {
                result.samples.push(format!(
                    "ceiling-sector={} height={} flat={} light={} sky={sky}",
                    mark.front_sector.record_index, key.0, key.1, key.2,
                ));
            }
        }
    }
    result.unique_floor_keys = floor_keys.len();
    result.unique_ceiling_keys = ceiling_keys.len();
    result
}

#[cfg(test)]
fn doom_seg_classic_plane_key(
    kind: DoomSegClassicPlaneKind,
    sector: &doom_map_provider::DoomSector,
) -> DoomSegClassicPlaneKey {
    if kind == DoomSegClassicPlaneKind::Ceiling && sector.ceiling_texture == "F_SKY1" {
        DoomSegClassicPlaneKey {
            kind,
            height: 0,
            texture: String::from("F_SKY1"),
            light: 0,
        }
    } else {
        DoomSegClassicPlaneKey {
            kind,
            height: match kind {
                DoomSegClassicPlaneKind::Floor => sector.floor_height,
                DoomSegClassicPlaneKind::Ceiling => sector.ceiling_height,
            },
            texture: match kind {
                DoomSegClassicPlaneKind::Floor => sector.floor_texture.clone(),
                DoomSegClassicPlaneKind::Ceiling => sector.ceiling_texture.clone(),
            },
            light: sector.light_level,
        }
    }
}

#[cfg(test)]
fn retain_doom_seg_classic_plane_range(
    observation: &mut DoomSegClassicPlaneSpanObservation,
    key: DoomSegClassicPlaneKey,
    source_sector: u32,
    source_seg: u32,
    writes: &[(usize, usize, usize)],
    columns: usize,
) {
    let valid = writes
        .iter()
        .filter_map(|&(column, top, bottom)| {
            if top > bottom {
                observation.empty_after_clip += 1;
                None
            } else {
                Some((column, top, bottom))
            }
        })
        .collect::<Vec<_>>();
    let Some(minimum_column) = valid.iter().map(|(column, _, _)| *column).min() else {
        return;
    };
    let maximum_column = valid
        .iter()
        .map(|(column, _, _)| *column)
        .max()
        .expect("a minimum column proves at least one valid plane write");
    let instances = observation.keys.entry(key).or_default();
    let compatible = instances.iter().position(|instance| {
        let intersection_start = minimum_column.max(instance.minimum_column);
        let intersection_end = maximum_column.min(instance.maximum_column);
        intersection_start > intersection_end
            || instance.columns[intersection_start..=intersection_end]
                .iter()
                .all(Option::is_none)
    });
    let instance_index = compatible.unwrap_or_else(|| {
        if !instances.is_empty() {
            observation.collision_splits += 1;
        }
        instances.push(DoomSegClassicPlaneInstance {
            columns: vec![None; columns],
            column_sources: vec![None; columns],
            minimum_column,
            maximum_column,
            source_sectors: BTreeSet::new(),
            source_segs: BTreeSet::new(),
        });
        instances.len() - 1
    });
    let instance = &mut instances[instance_index];
    instance.source_sectors.insert(source_sector);
    instance.source_segs.insert(source_seg);
    instance.minimum_column = instance.minimum_column.min(minimum_column);
    instance.maximum_column = instance.maximum_column.max(maximum_column);
    for (column, top, bottom) in valid {
        let slot = &mut instance.columns[column];
        if slot.is_some() {
            observation.overlapping_writes += 1;
        } else {
            *slot = Some([top, bottom]);
            instance.column_sources[column] = Some([source_sector, source_seg]);
        }
    }
}

fn resolve_doom_seg_classic_plane_flats(
    scene: &SceneInput,
    spans: &DoomSegClassicPlaneSpanObservation,
) -> DoomSegClassicPlaneFlatResolution {
    let mut result = DoomSegClassicPlaneFlatResolution::default();
    for (key, instances) in &spans.keys {
        for (instance_index, instance) in instances.iter().enumerate() {
            if key.kind == DoomSegClassicPlaneKind::Ceiling && key.texture == "F_SKY1" {
                result.sky_instances += 1;
                if result.samples.len() < 12 {
                    result.samples.push(format!(
                        "kind={:?} flat={} instance={} sectors={:?} result=sky-presentation",
                        key.kind, key.texture, instance_index, instance.source_sectors,
                    ));
                }
                continue;
            }

            let expected_plane = match key.kind {
                DoomSegClassicPlaneKind::Floor => DoomSurfacePlane::Floor,
                DoomSegClassicPlaneKind::Ceiling => DoomSurfacePlane::Ceiling,
            };
            let candidates = scene
                .opaque_draws
                .iter()
                .filter(|draw| {
                    matches!(
                        draw.source,
                        StaticDrawSource::Flat {
                            source_sector,
                            plane,
                            ..
                        } if plane == expected_plane
                            && instance.source_sectors.contains(&source_sector.record_index)
                    )
                })
                .collect::<Vec<_>>();
            let triangles = candidates
                .iter()
                .map(|draw| draw.mesh.positions.len() / 3)
                .sum::<usize>();
            result.candidate_draws += candidates.len();
            result.candidate_triangles += triangles;
            if candidates.is_empty() {
                result.unresolved_instances += 1;
            } else {
                result.resolved_instances += 1;
            }
            if result.samples.len() < 12 {
                result.samples.push(format!(
                    "kind={:?} flat={} instance={} sectors={:?} segs={} candidate-draws={} candidate-triangles={}",
                    key.kind,
                    key.texture,
                    instance_index,
                    instance.source_sectors,
                    instance.source_segs.len(),
                    candidates.len(),
                    triangles,
                ));
            }
        }
    }
    result
}

fn observe_fixed_source_ordered_coverage(
    scene: &SceneInput,
) -> PlatformResult<(DoomSegClassicVerticalClipObservation, [i16; 2], f64, f64)> {
    let viewer = scene.spawn_observer.source_position;
    let heading = f64::from(scene.spawn_observer.source_angle).to_radians();
    let eye_height = scene.spawn_observer.position.y as f64;
    let traversal = observe_doom_seg_classic_bsp(
        &scene.door_geometry_source.map,
        viewer,
        heading,
        &BTreeSet::new(),
    )?;
    let lowerable_triangles = lower_doom_seg_textured_wall_triangles(
        &scene.door_geometry_source.map,
        &scene.door_geometry_source.wall_extents,
    )?;
    let plane_marks =
        observe_doom_seg_plane_marks(&scene.door_geometry_source.map, eye_height as i16)?;
    let vertical = observe_shared_doom_classic_vertical_clip_state(
        &scene.door_geometry_source.map,
        &lowerable_triangles,
        &plane_marks,
        &traversal,
        viewer,
        heading,
        eye_height,
    );
    Ok((vertical, viewer, heading, eye_height))
}

fn prepare_doom_ordered_coverage_observation(
    scene: &SceneInput,
) -> PlatformResult<DoomOrderedCoveragePreparation> {
    prepare_doom_ordered_coverage(
        &scene.door_geometry_source.map,
        &scene.door_geometry_source.wall_extents,
        scene.spawn_observer.source_position,
        f64::from(scene.spawn_observer.source_angle).to_radians(),
        scene.spawn_observer.position.y as f64,
        true,
    )
}

fn prepare_doom_seg_classic_plane_presentation(
    scene: &SceneInput,
) -> PlatformResult<DoomSegClassicPlanePresentation> {
    let (vertical, viewer, heading, eye_height) = observe_fixed_source_ordered_coverage(scene)?;
    let reconstruction = reconstruct_doom_seg_classic_plane_cells(
        &vertical.plane_spans,
        viewer,
        heading,
        eye_height,
    );
    lower_doom_seg_classic_plane_presentation(
        &scene.door_geometry_source.map,
        &scene.opaque_uploads,
        DoomComparativeEmbedding::CurrentReflected,
        reconstruction,
    )
}

/// Reconstructs the provider's retained per-column wall cells into grouped
/// ordinary meshes while preserving source and material identity. This is the
/// explicit Slice 7 falsification candidate: it may prove that source-derived
/// partial fragments are sufficient, or expose that the retained intervals are
/// still incomplete. It is not the default E1M1 preparation path.
fn prepare_doom_seg_ordered_coverage_presentation(
    scene: &SceneInput,
) -> PlatformResult<DoomSegOrderedCoveragePresentation> {
    struct WallGroup {
        source_seg: doom_map_provider::DoomSourceRecord,
        source_linedef: doom_map_provider::DoomSourceRecord,
        source_sidedef: doom_map_provider::DoomSourceRecord,
        source_sector: doom_map_provider::DoomSourceRecord,
        role: DoomWallTextureRole,
        texture_name: String,
        material: MaterialHandle,
        cutout: bool,
        positions: Vec<[f32; 3]>,
        normals: Vec<[f32; 3]>,
        texture_coordinates: Vec<[f32; 2]>,
    }

    let preparation = prepare_doom_ordered_coverage_observation(scene)?;
    let DoomOrderedCoveragePreparation {
        traversal,
        vertical,
        walls: reconstruction,
        planes: plane_reconstruction,
        ordinary_plane_intervals,
        sky_plane_intervals,
    } = preparation;
    let rejected_plane_intervals = plane_reconstruction.horizon_rejections
        + plane_reconstruction.behind_viewer_rejections
        + plane_reconstruction.degenerate_rejections;
    if ordinary_plane_intervals
        != plane_reconstruction.reconstructed_quads + rejected_plane_intervals
    {
        return Err(io::Error::other(format!(
            "ordered coverage plane conservation failed: retained ordinary intervals={ordinary_plane_intervals}, reconstructed={}, rejected={rejected_plane_intervals}",
            plane_reconstruction.reconstructed_quads,
        ))
        .into());
    }
    let reconstructed_plane_quads = plane_reconstruction.reconstructed_quads;
    let planes = lower_doom_seg_classic_plane_presentation(
        &scene.door_geometry_source.map,
        &scene.opaque_uploads,
        DoomComparativeEmbedding::CurrentReflected,
        plane_reconstruction,
    )?;
    let lowered_plane_quads = planes.triangles / 2;
    if lowered_plane_quads != reconstructed_plane_quads {
        return Err(io::Error::other(format!(
            "ordered coverage plane lowering lost contributions: reconstructed quads={reconstructed_plane_quads}, lowered quads={lowered_plane_quads}",
        ))
        .into());
    }

    let cutout_materials = scene
        .cutout_draws
        .iter()
        .filter_map(|draw| match draw.source {
            StaticDrawSource::Wall {
                source_linedef,
                source_sidedef,
                role,
                ..
            } => Some((
                (
                    source_linedef.record_index,
                    source_sidedef.record_index,
                    doom_wall_role_key(role),
                ),
                draw.material,
            )),
            StaticDrawSource::Flat { .. } => None,
        })
        .collect::<BTreeMap<_, _>>();
    let retained_middle_segs = vertical
        .ordered_wall_intervals
        .iter()
        .filter(|interval| {
            interval.role == DoomWallTextureRole::Middle && interval.retained_interval.is_some()
        })
        .map(|interval| interval.source_seg)
        .collect::<BTreeSet<_>>();
    let source_cutout_keys = retained_middle_segs
        .iter()
        .filter_map(|source_seg| {
            let seg = scene
                .door_geometry_source
                .map
                .segs
                .iter()
                .find(|seg| seg.source.record_index == *source_seg)?;
            let linedef = &scene.door_geometry_source.map.linedefs[usize::from(seg.linedef)];
            let sidedef_index = match seg.direction {
                0 => linedef.right_sidedef,
                1 => linedef.left_sidedef,
                _ => None,
            }?;
            let sidedef = &scene.door_geometry_source.map.sidedefs[usize::from(sidedef_index)];
            let key = (
                linedef.source.record_index,
                sidedef.source.record_index,
                doom_wall_role_key(DoomWallTextureRole::Middle),
            );
            cutout_materials.contains_key(&key).then_some(key)
        })
        .collect::<BTreeSet<_>>();
    let extents = scene
        .door_geometry_source
        .wall_extents
        .iter()
        .map(|extent| (extent.name.as_str(), extent.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut groups = BTreeMap::<(u32, u32, u8, String, bool), WallGroup>::new();
    let source_degenerate_cells = reconstruction.degenerate_cells;
    let source_unresolved_cells = reconstruction.unresolved_cells;
    let reconstructed_triangles = reconstruction.reconstructed_triangles.len();
    if reconstructed_triangles % 2 != 0
        || reconstruction.retained_cells
            != reconstructed_triangles / 2 + source_degenerate_cells + source_unresolved_cells
    {
        return Err(io::Error::other(format!(
            "ordered coverage wall reconstruction conservation failed: retained cells={}, reconstructed triangles={reconstructed_triangles}, source-degenerate cells={source_degenerate_cells}, source-unresolved cells={source_unresolved_cells}",
            reconstruction.retained_cells,
        ))
        .into());
    }
    let mut lowering_unresolved_triangles = 0;
    let mut lowering_degenerate_triangles = 0;
    let mut samples = reconstruction.samples.clone();

    for triangle in &reconstruction.reconstructed_triangles {
        let role_key = doom_wall_role_key(triangle.role);
        let cutout_key = (
            triangle.source_linedef.record_index,
            triangle.source_sidedef.record_index,
            role_key,
        );
        let (material, cutout) = if let Some(material) = cutout_materials.get(&cutout_key) {
            (*material, true)
        } else if let Some(material) = scene
            .door_geometry_source
            .wall_materials
            .get(&triangle.texture_name)
        {
            (*material, false)
        } else {
            lowering_unresolved_triangles += 1;
            if samples.len() < 12 {
                samples.push(format!(
                    "seg={}:linedef={}:texture={}:reason=material-unresolved",
                    triangle.source_seg.record_index,
                    triangle.source_linedef.record_index,
                    triangle.texture_name,
                ));
            }
            continue;
        };
        let Some(extent) = extents.get(triangle.texture_name.as_str()).cloned() else {
            lowering_unresolved_triangles += 1;
            if samples.len() < 12 {
                samples.push(format!(
                    "seg={}:linedef={}:texture={}:reason=extent-unresolved",
                    triangle.source_seg.record_index,
                    triangle.source_linedef.record_index,
                    triangle.texture_name,
                ));
            }
            continue;
        };

        let lowered = match lower_static_seg_wall_triangle(triangle, extent) {
            Ok(lowered) => lowered,
            Err(StaticFlatLoweringError::DegenerateTriangle) => {
                lowering_degenerate_triangles += 1;
                if samples.len() < 12 {
                    samples.push(format!(
                        "seg={}:linedef={}:texture={}:omitted=degenerate-reconstructed-fragment",
                        triangle.source_seg.record_index,
                        triangle.source_linedef.record_index,
                        triangle.texture_name,
                    ));
                }
                continue;
            }
            Err(error) => return Err(error.into()),
        };
        let key = (
            triangle.source_seg.record_index,
            triangle.source_sidedef.record_index,
            role_key,
            triangle.texture_name.clone(),
            cutout,
        );
        let group = groups.entry(key).or_insert_with(|| WallGroup {
            source_seg: triangle.source_seg,
            source_linedef: triangle.source_linedef,
            source_sidedef: triangle.source_sidedef,
            source_sector: triangle.source_sector,
            role: triangle.role,
            texture_name: triangle.texture_name.clone(),
            material,
            cutout,
            positions: Vec::new(),
            normals: Vec::new(),
            texture_coordinates: Vec::new(),
        });
        group.positions.extend(lowered.wall.mesh.positions);
        group.normals.extend(lowered.wall.mesh.normals);
        group
            .texture_coordinates
            .extend(lowered.wall.mesh.texture_coordinates);
    }

    let grouped_wall_meshes = groups.len();
    let lowered_wall_triangles = groups
        .values()
        .map(|group| group.positions.len() / 3)
        .sum::<usize>();
    if reconstructed_triangles
        != lowered_wall_triangles + lowering_degenerate_triangles + lowering_unresolved_triangles
    {
        return Err(io::Error::other(format!(
            "ordered coverage wall lowering conservation failed: reconstructed triangles={reconstructed_triangles}, lowered triangles={lowered_wall_triangles}, lowering-degenerate triangles={lowering_degenerate_triangles}, lowering-unresolved triangles={lowering_unresolved_triangles}",
        ))
        .into());
    }
    let mut opaque_draws = planes.draws;
    let mut cutout_draws = Vec::new();
    for (_, group) in groups {
        let mesh = Mesh::new(group.positions, group.normals)
            .with_texture_coordinates(group.texture_coordinates)?;
        let draw = StaticDrawPlanEntry {
            source_label: format!(
                "ordered-coverage-wall:{}:{}:{:?}:{}",
                group.source_seg.record_index,
                group.source_linedef.record_index,
                group.role,
                group.texture_name,
            ),
            source: StaticDrawSource::Wall {
                source_linedef: group.source_linedef,
                source_sidedef: group.source_sidedef,
                source_sector: group.source_sector,
                role: group.role,
            },
            mesh,

            material: group.material,
        };
        if group.cutout {
            cutout_draws.push(draw);
        } else {
            opaque_draws.push(draw);
        }
    }
    let lowered_cutout_keys = cutout_draws
        .iter()
        .filter_map(|draw| match draw.source {
            StaticDrawSource::Wall {
                source_linedef,
                source_sidedef,
                role,
                ..
            } => Some((
                source_linedef.record_index,
                source_sidedef.record_index,
                doom_wall_role_key(role),
            )),
            StaticDrawSource::Flat { .. } => None,
        })
        .collect::<BTreeSet<_>>();
    if lowered_cutout_keys != source_cutout_keys {
        let missing = source_cutout_keys
            .difference(&lowered_cutout_keys)
            .copied()
            .collect::<Vec<_>>();
        let fabricated = lowered_cutout_keys
            .difference(&source_cutout_keys)
            .copied()
            .collect::<Vec<_>>();
        return Err(io::Error::other(format!(
            "ordered coverage cutout conservation failed: missing={missing:?}, fabricated={fabricated:?}",
        ))
        .into());
    }

    Ok(DoomSegOrderedCoveragePresentation {
        opaque_draws,
        cutout_draws,
        retained_cells: reconstruction.retained_cells,
        reconstructed_triangles,
        lowered_wall_triangles,
        source_degenerate_cells,
        source_unresolved_cells,
        lowering_degenerate_triangles,
        lowering_unresolved_triangles,
        grouped_wall_meshes,
        ordinary_plane_intervals,
        sky_plane_intervals,
        reconstructed_plane_quads,
        rejected_plane_intervals,
        lowered_plane_quads,
        source_cutout_keys: source_cutout_keys.len(),
        lowered_cutout_keys: lowered_cutout_keys.len(),
        coverage_transitions: vertical.ordered_coverage_transitions.len(),
        coverage_fail_open: vertical.ordered_coverage_fail_open.len(),
        coverage_fail_open_reasons: DoomCoverageFailOpenSummary::default(),
        bsp_leaves_visited: traversal.leaves_visited,
        bsp_far_children_pruned: traversal.far_children_pruned,
        bsp_admitted_segs: traversal.admitted_seg_records.len(),
        bsp_solid_range_pruning: true,
        degenerate_omissions: source_degenerate_cells + lowering_degenerate_triangles,
        unresolved_cells: source_unresolved_cells + lowering_unresolved_triangles,
        samples,
    })
}

/// Adds the source-spawn BSP-admitted, already lowerable opaque wall tiers to
/// the reconstructed planes so a maintainer can judge plane gaps in context.
/// Wall tiers remain whole SEG fragments here; exact projected tier clipping
/// is deliberately not claimed by this intermediate visual control.
fn prepare_doom_seg_classic_context_presentation(
    scene: &SceneInput,
) -> PlatformResult<DoomSegClassicContextPresentation> {
    let planes = prepare_doom_seg_classic_plane_presentation(scene)?;
    let viewer = scene.spawn_observer.source_position;
    let heading = f64::from(scene.spawn_observer.source_angle).to_radians();
    let traversal = observe_doom_seg_classic_bsp(
        &scene.door_geometry_source.map,
        viewer,
        heading,
        &BTreeSet::new(),
    )?;
    let triangles = lower_doom_seg_textured_wall_triangles(
        &scene.door_geometry_source.map,
        &scene.door_geometry_source.wall_extents,
    )?;
    let mut draws = planes.draws;
    let mut wall_meshes = 0usize;
    let mut omitted_wall_triangles = 0usize;
    for triangle in triangles.iter().filter(|triangle| {
        traversal
            .admitted_seg_records
            .contains(&triangle.source_seg.record_index)
    }) {
        let Some(material) = scene
            .door_geometry_source
            .wall_materials
            .get(&triangle.texture_name)
            .copied()
        else {
            omitted_wall_triangles += 1;
            continue;
        };
        let extent = scene
            .door_geometry_source
            .wall_extents
            .iter()
            .find(|extent| extent.name == triangle.texture_name)
            .cloned()
            .ok_or_else(|| {
                io::Error::other(format!(
                    "classic context wall `{}` has no texture extent",
                    triangle.texture_name
                ))
            })?;
        let lowered = match lower_static_seg_wall_triangle(triangle, extent) {
            Ok(lowered) => lowered,
            Err(StaticFlatLoweringError::DegenerateTriangle) => {
                omitted_wall_triangles += 1;
                continue;
            }
            Err(error) => return Err(error.into()),
        };
        wall_meshes += 1;
        draws.push(StaticDrawPlanEntry {
            source_label: format!(
                "classic-context-wall:{}:{}:{}",
                lowered.source_seg.record_index,
                lowered.wall.source_linedef.record_index,
                lowered.wall.texture_name,
            ),
            source: StaticDrawSource::Wall {
                source_linedef: lowered.wall.source_linedef,
                source_sidedef: lowered.wall.source_sidedef,
                source_sector: lowered.wall.source_sector,
                role: lowered.wall.role,
            },
            mesh: lowered.wall.mesh,
            material,
        });
    }

    Ok(DoomSegClassicContextPresentation {
        plane_meshes: planes.grouped_meshes,
        plane_triangles: planes.triangles,
        wall_meshes,
        omitted_wall_triangles,
        draws,
    })
}

#[cfg(test)]
fn finalize_doom_seg_classic_plane_spans(observation: &mut DoomSegClassicPlaneSpanObservation) {
    observation.horizontal_spans = 0;
    observation.plane_instances = 0;
    observation.populated_columns = 0;
    observation.populated_cells = 0;
    observation.samples.clear();
    for (key, instances) in &observation.keys {
        let mut key_spans = 0usize;
        let mut key_columns = 0usize;
        let mut key_cells = 0usize;
        for instance in instances {
            observation.plane_instances += 1;
            let mut in_span = false;
            for column in &instance.columns {
                match column {
                    Some([top, bottom]) => {
                        if !in_span {
                            key_spans += 1;
                            in_span = true;
                        }
                        key_columns += 1;
                        key_cells += bottom - top + 1;
                    }
                    None => in_span = false,
                }
            }
        }

        observation.horizontal_spans += key_spans;
        observation.populated_columns += key_columns;
        observation.populated_cells += key_cells;
        if observation.samples.len() < 12 {
            observation.samples.push(format!(
                "kind={:?} height={} flat={} light={} instances={} spans={} columns={} cells={}",
                key.kind,
                key.height,
                key.texture,
                key.light,
                instances.len(),
                key_spans,
                key_columns,
                key_cells,
            ));
        }
    }
}

/// Bounded source-local observation of the clip boundaries that wall tiers
/// evolve after recursive BSP admission. The arrays are diagnostic only: no
/// renderer scissor, candidate selector, flat draw, or visplane consumes them.
#[cfg(test)]
#[allow(dead_code, clippy::too_many_arguments)]
fn legacy_observe_doom_seg_classic_vertical_clip_state(
    map: &DoomMapCore,
    triangles: &[DoomSegTexturedWallTriangle],
    plane_marks: &[DoomSegPlaneMarkObservation],
    traversal: &DoomSegClassicBspObservation,
    viewer: [i16; 2],
    heading: f64,
    eye_height: f64,
) -> DoomSegClassicVerticalClipObservation {
    let half_vertical_fov = classic_presentation_half_vertical_fov();
    let mut result = DoomSegClassicVerticalClipObservation {
        admitted_segs: traversal.admitted_seg_order.len(),
        ..Default::default()
    };
    let mut ceiling_clip = vec![0usize; CLASSIC_PRESENTATION_COLUMNS];
    let mut floor_clip = vec![CLASSIC_PRESENTATION_ROWS; CLASSIC_PRESENTATION_COLUMNS];
    let marks_by_seg = plane_marks
        .iter()
        .map(|mark| (mark.source_seg.record_index, mark))
        .collect::<BTreeMap<_, _>>();
    let segs_by_record = map
        .segs
        .iter()
        .map(|seg| (seg.source.record_index, seg))
        .collect::<BTreeMap<_, _>>();
    let sectors_by_record = map
        .sectors
        .iter()
        .map(|sector| (sector.source.record_index, sector))
        .collect::<BTreeMap<_, _>>();
    let mut tier_heights = BTreeMap::<(u32, u8), (DoomWallTextureRole, f64, f64)>::new();
    for triangle in triangles {
        if !traversal
            .admitted_seg_records
            .contains(&triangle.source_seg.record_index)
        {
            continue;
        }
        let role_key = match triangle.role {
            DoomWallTextureRole::Upper => 0,
            DoomWallTextureRole::Lower => 1,
            DoomWallTextureRole::Middle => 2,
        };
        let minimum = triangle
            .positions
            .iter()
            .map(|position| position[1])
            .fold(f64::INFINITY, f64::min);
        let maximum = triangle
            .positions
            .iter()
            .map(|position| position[1])
            .fold(f64::NEG_INFINITY, f64::max);
        tier_heights
            .entry((triangle.source_seg.record_index, role_key))
            .and_modify(|(_, stored_minimum, stored_maximum)| {
                *stored_minimum = stored_minimum.min(minimum);
                *stored_maximum = stored_maximum.max(maximum);
            })
            .or_insert((triangle.role, minimum, maximum));
    }
    let forward = [heading.cos(), heading.sin()];
    let right = [-forward[1], forward[0]];
    let project = |point: [i16; 2]| {
        let relative = [
            f64::from(point[0] - viewer[0]),
            f64::from(point[1] - viewer[1]),
        ];
        let depth = relative[0] * forward[0] + relative[1] * forward[1];
        let lateral = relative[0] * right[0] + relative[1] * right[1];
        (depth, lateral.atan2(depth))
    };
    let row = |angle: f64| {
        let normalized = (angle.tan() / half_vertical_fov.tan()).clamp(-1.0, 1.0);
        (((1.0 - normalized) * 0.5) * CLASSIC_PRESENTATION_ROWS as f64) as usize
    };
    for source_seg in &traversal.admitted_seg_order {
        let (Some(mark), Some(seg)) =
            (marks_by_seg.get(source_seg), segs_by_record.get(source_seg))
        else {
            continue;
        };
        let front_sector = sectors_by_record
            .get(&mark.front_sector.record_index)
            .expect("validated plane mark names an existing front sector");
        result.floor_plane_marks += usize::from(mark.floor_marked);
        result.ceiling_plane_marks += usize::from(mark.ceiling_marked);
        result.paired_sky_adjustments += usize::from(mark.paired_sky_ceiling_adjustment);
        let start = &map.vertices[usize::from(seg.start_vertex)];
        let end = &map.vertices[usize::from(seg.end_vertex)];
        let (start_depth, start_angle) = project([start.x, start.y]);
        let (end_depth, end_angle) = project([end.x, end.y]);
        if start_depth <= 0.0
            || end_depth <= 0.0
            || (start_angle.abs() > CLASSIC_PRESENTATION_HALF_HORIZONTAL_FOV
                && end_angle.abs() > CLASSIC_PRESENTATION_HALF_HORIZONTAL_FOV)
        {
            continue;
        }
        let [left, right_column] = source_fov_column_interval(
            start_angle,
            end_angle,
            CLASSIC_PRESENTATION_HALF_HORIZONTAL_FOV,
            CLASSIC_PRESENTATION_COLUMNS,
        );
        let has_upper = tier_heights.contains_key(&(*source_seg, 0));
        let has_lower = tier_heights.contains_key(&(*source_seg, 1));
        let has_middle = tier_heights.contains_key(&(*source_seg, 2));

        // Classic plane marking consumes the clip state that exists before
        // this wall range mutates it. Retain only bounded source-keyed cells;
        // later presentation lowering must remain a separate experiment.
        let mut ceiling_plane_writes = Vec::new();
        let mut floor_plane_writes = Vec::new();
        for x in left..=right_column {
            let normalized = -1.0 + ((x as f64 + 0.5) / CLASSIC_PRESENTATION_COLUMNS as f64) * 2.0;
            let local_angle = (normalized * CLASSIC_PRESENTATION_HALF_HORIZONTAL_FOV.tan()).atan();
            let ray = [
                forward[0] * local_angle.cos() + right[0] * local_angle.sin(),
                forward[1] * local_angle.cos() + right[1] * local_angle.sin(),
            ];
            let depth = source_ray_segment_depth(viewer, ray, [start.x, start.y], [end.x, end.y])
                .unwrap_or((start_depth + end_depth) * 0.5);
            let ceiling = row((f64::from(front_sector.ceiling_height) - eye_height).atan2(depth))
                .min(CLASSIC_PRESENTATION_ROWS - 1);
            let floor = row((f64::from(front_sector.floor_height) - eye_height).atan2(depth))
                .min(CLASSIC_PRESENTATION_ROWS - 1);

            let (ceiling, floor) = (ceiling.min(floor), ceiling.max(floor));
            if mark.ceiling_marked {
                let top = ceiling_clip[x].saturating_add(1);
                let bottom = ceiling.saturating_sub(1);
                ceiling_plane_writes.push((x, top, bottom));
            }
            if mark.floor_marked {
                let top = floor.saturating_add(1);
                let bottom = floor_clip[x].saturating_sub(1);
                floor_plane_writes.push((x, top, bottom));
            }
        }
        if !ceiling_plane_writes.is_empty() {
            retain_doom_seg_classic_plane_range(
                &mut result.plane_spans,
                doom_seg_classic_plane_key(DoomSegClassicPlaneKind::Ceiling, front_sector),
                mark.front_sector.record_index,
                *source_seg,
                &ceiling_plane_writes,
                CLASSIC_PRESENTATION_COLUMNS,
            );
        }
        if !floor_plane_writes.is_empty() {
            retain_doom_seg_classic_plane_range(
                &mut result.plane_spans,
                doom_seg_classic_plane_key(DoomSegClassicPlaneKind::Floor, front_sector),
                mark.front_sector.record_index,
                *source_seg,
                &floor_plane_writes,
                CLASSIC_PRESENTATION_COLUMNS,
            );
        }
        for role_key in 0..=2 {
            let Some((role, minimum, maximum)) = tier_heights.get(&(*source_seg, role_key)) else {
                continue;
            };
            match role {
                DoomWallTextureRole::Upper => result.upper_tier_spans += 1,
                DoomWallTextureRole::Lower => result.lower_tier_spans += 1,
                DoomWallTextureRole::Middle => result.middle_tier_spans += 1,
            }
            let mut center_trace = None;
            for x in left..=right_column {
                let normalized =
                    -1.0 + ((x as f64 + 0.5) / CLASSIC_PRESENTATION_COLUMNS as f64) * 2.0;
                let local_angle =
                    (normalized * CLASSIC_PRESENTATION_HALF_HORIZONTAL_FOV.tan()).atan();
                let ray = [
                    forward[0] * local_angle.cos() + right[0] * local_angle.sin(),
                    forward[1] * local_angle.cos() + right[1] * local_angle.sin(),
                ];
                let depth =
                    source_ray_segment_depth(viewer, ray, [start.x, start.y], [end.x, end.y])
                        .unwrap_or((start_depth + end_depth) * 0.5);
                let top =
                    row((maximum - eye_height).atan2(depth)).min(CLASSIC_PRESENTATION_ROWS - 1);
                let bottom =
                    row((minimum - eye_height).atan2(depth)).min(CLASSIC_PRESENTATION_ROWS - 1);
                let (top, bottom) = (top.min(bottom), top.max(bottom));
                let prior = [ceiling_clip[x], floor_clip[x]];
                match role {
                    DoomWallTextureRole::Upper => {
                        let next = ceiling_clip[x].max(bottom.saturating_add(1));
                        result.ceiling_clip_updates += usize::from(next != ceiling_clip[x]);
                        ceiling_clip[x] = next;
                    }
                    DoomWallTextureRole::Lower => {
                        let next = floor_clip[x].min(top);
                        result.floor_clip_updates += usize::from(next != floor_clip[x]);
                        floor_clip[x] = next;
                    }
                    DoomWallTextureRole::Middle => {}
                }
                if x == CLASSIC_PRESENTATION_COLUMNS / 2 {
                    center_trace = Some(format!(
                        "seg={source_seg} line={} tier={role:?} rows={top}..{bottom} clip-before={}..{} clip-after={}..{}",
                        seg.linedef, prior[0], prior[1], ceiling_clip[x], floor_clip[x],
                    ));
                }
            }
            if let Some(sample) = center_trace {
                if result.samples.len() < 12 {
                    result.samples.push(sample);
                }
            }
        }
        // The original wall loop also moves a clip boundary for a marked plane
        // when there is no corresponding upper/lower texture tier. A one-sided
        // middle is terminal, while a two-sided masked middle remains open.
        for x in left..=right_column {
            let normalized = -1.0 + ((x as f64 + 0.5) / CLASSIC_PRESENTATION_COLUMNS as f64) * 2.0;
            let local_angle = (normalized * CLASSIC_PRESENTATION_HALF_HORIZONTAL_FOV.tan()).atan();
            let ray = [
                forward[0] * local_angle.cos() + right[0] * local_angle.sin(),
                forward[1] * local_angle.cos() + right[1] * local_angle.sin(),
            ];
            let depth = source_ray_segment_depth(viewer, ray, [start.x, start.y], [end.x, end.y])
                .unwrap_or((start_depth + end_depth) * 0.5);
            let ceiling = row((f64::from(front_sector.ceiling_height) - eye_height).atan2(depth))
                .min(CLASSIC_PRESENTATION_ROWS - 1);
            let floor = row((f64::from(front_sector.floor_height) - eye_height).atan2(depth))
                .min(CLASSIC_PRESENTATION_ROWS - 1);
            let (ceiling, floor) = (ceiling.min(floor), ceiling.max(floor));
            if has_middle && mark.back_sector.is_none() {
                result.ceiling_clip_updates +=
                    usize::from(ceiling_clip[x] != CLASSIC_PRESENTATION_ROWS);
                result.floor_clip_updates += usize::from(floor_clip[x] != 0);
                ceiling_clip[x] = CLASSIC_PRESENTATION_ROWS;
                floor_clip[x] = 0;
            } else {
                if !has_upper && mark.ceiling_marked {
                    let next = ceiling_clip[x].max(ceiling.saturating_sub(1));
                    result.ceiling_clip_updates += usize::from(next != ceiling_clip[x]);
                    ceiling_clip[x] = next;
                }
                if !has_lower && mark.floor_marked {
                    let next = floor_clip[x].min(floor.saturating_add(1));
                    result.floor_clip_updates += usize::from(next != floor_clip[x]);
                    floor_clip[x] = next;
                }
            }
        }
    }
    finalize_doom_seg_classic_plane_spans(&mut result.plane_spans);
    result
}

/// Inventories the already lowerable wall tiers selected by the headless
/// source protocol. The roles remain Doom provider evidence; they are not a
/// renderer material taxonomy or a claim that all source wall tiers have been
/// classically clipped.
fn summarize_classic_bsp_wall_triangle_roles(
    triangles: &[DoomSegTexturedWallTriangle],
    admitted_seg_records: &BTreeSet<u32>,
) -> (usize, usize, usize) {
    triangles
        .iter()
        .fold((0, 0, 0), |(upper, lower, middle), triangle| {
            if !admitted_seg_records.contains(&triangle.source_seg.record_index) {
                return (upper, lower, middle);
            }
            match triangle.role {
                DoomWallTextureRole::Upper => (upper + 1, lower, middle),
                DoomWallTextureRole::Lower => (upper, lower + 1, middle),
                DoomWallTextureRole::Middle => (upper, lower, middle + 1),
            }
        })
}

/// Counts the source `R_StoreWallRange` plane-mark facts for admitted SEG
/// records. A mark is not a projected visplane span or a selected flat draw.
fn summarize_classic_bsp_plane_marks(
    plane_marks: &[DoomSegPlaneMarkObservation],
    admitted_seg_records: &BTreeSet<u32>,
) -> (usize, usize, usize) {
    plane_marks
        .iter()
        .fold((0, 0, 0), |(floors, ceilings, paired_sky), observation| {
            if !admitted_seg_records.contains(&observation.source_seg.record_index) {
                return (floors, ceilings, paired_sky);
            }
            (
                floors + usize::from(observation.floor_marked),
                ceilings + usize::from(observation.ceiling_marked),
                paired_sky + usize::from(observation.paired_sky_ceiling_adjustment),
            )
        })
}

/// Counts existing source-labelled static flat draws whose owning subsector was
/// reached by the headless Doom BSP protocol. These are not classic-Doom plane
/// spans and must not be submitted as a visibility result; the count merely
/// makes the currently unmodeled plane portion explicit.
fn count_classic_bsp_static_flat_draws(
    scene: &SceneInput,
    observation: &DoomSegClassicBspObservation,
) -> (usize, usize) {
    scene
        .opaque_draws
        .iter()
        .fold((0, 0), |(floors, ceilings), draw| match draw.source {
            StaticDrawSource::Flat {
                source_subsector,
                plane: doom_geometry_provider::DoomSurfacePlane::Floor,
                ..
            } if observation
                .visited_subsectors
                .contains(&(source_subsector.record_index as u16)) =>
            {
                (floors + 1, ceilings)
            }
            StaticDrawSource::Flat {
                source_subsector,
                plane: doom_geometry_provider::DoomSurfacePlane::Ceiling,
                ..
            } if observation
                .visited_subsectors
                .contains(&(source_subsector.record_index as u16)) =>
            {
                (floors, ceilings + 1)
            }
            _ => (floors, ceilings),
        })
}

fn observe_doom_seg_classic_bsp(
    map: &DoomMapCore,
    viewer: [i16; 2],
    heading: f64,
    watched_subsectors: &BTreeSet<u16>,
) -> PlatformResult<DoomSegClassicBspObservation> {
    Ok(observe_doom_classic_bsp(
        map,
        viewer,
        heading,
        watched_subsectors,
    )?)
}

#[cfg(test)]
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
fn visit_doom_seg_classic_bsp_child(
    map: &DoomMapCore,
    child: DoomBspChild,
    viewer: [i16; 2],
    heading: f64,
    occluders: &BTreeMap<u32, doom_geometry_provider::DoomSegOccluderObservation>,
    solid_ranges: &mut Vec<[usize; 2]>,
    ancestors: &mut Vec<u16>,
    watched_subsectors: &BTreeSet<u16>,
    observation: &mut DoomSegClassicBspObservation,
) -> PlatformResult<()> {
    match child {
        DoomBspChild::Subsector(index) => {
            let subsector = map.subsectors.get(usize::from(index)).ok_or_else(|| {
                io::Error::other(format!(
                    "Stage 3B classic BSP subsector {index} is unavailable"
                ))
            })?;
            observation.leaves_visited += 1;
            observation.visited_subsectors.insert(index);
            let first = usize::from(subsector.first_seg);
            let end = first + usize::from(subsector.seg_count);
            for seg in &map.segs[first..end] {
                admit_doom_seg_classic(
                    map,
                    seg,
                    viewer,
                    heading,
                    occluders,
                    solid_ranges,
                    observation,
                );
            }
            Ok(())
        }
        DoomBspChild::Node(index) => {
            if ancestors.contains(&index) {
                return Err(io::Error::other(format!(
                    "Stage 3B classic BSP cycle at node {index}"
                ))
                .into());
            }
            let node = map.nodes.get(usize::from(index)).ok_or_else(|| {
                io::Error::other(format!("Stage 3B classic BSP node {index} is unavailable"))
            })?;
            ancestors.push(index);
            let side = i64::from(node.delta_x) * i64::from(viewer[1] - node.y)
                - i64::from(node.delta_y) * i64::from(viewer[0] - node.x);
            let (near, far, far_bbox) = if side < 0 {
                (node.right_child, node.left_child, node.left_bbox)
            } else {
                (node.left_child, node.right_child, node.right_bbox)
            };
            visit_doom_seg_classic_bsp_child(
                map,
                near,
                viewer,
                heading,
                occluders,
                solid_ranges,
                ancestors,
                watched_subsectors,
                observation,
            )?;
            let watched_far = watched_subsectors
                .iter()
                .filter_map(|target| {
                    doom_bsp_child_contains_subsector(map, far, *target).then_some(*target)
                })
                .collect::<Vec<_>>();
            let far_projection = source_bbox_fov_column_interval(
                viewer,
                heading,
                far_bbox,
                std::f64::consts::FRAC_PI_4,
                320,
            );
            match far_projection {
                SourceBBoxProjection::OutsideFov => {
                    observation.far_children_outside_fov += 1;
                    record_watched_subsector_elision(
                        observation,
                        index,
                        "outside-fov",
                        &watched_far,
                        None,
                        None,
                    );
                }
                SourceBBoxProjection::Interval(interval) => {
                    if let Some(covering_range) = solid_ranges
                        .iter()
                        .find(|[first, last]| *first <= interval[0] && interval[1] <= *last)
                    {
                        observation.far_children_pruned += 1;
                        record_watched_subsector_elision(
                            observation,
                            index,
                            "solid-range",
                            &watched_far,
                            Some(interval),
                            Some(*covering_range),
                        );
                    } else {
                        visit_doom_seg_classic_bsp_child(
                            map,
                            far,
                            viewer,
                            heading,
                            occluders,
                            solid_ranges,
                            ancestors,
                            watched_subsectors,
                            observation,
                        )?;
                    }
                }
                SourceBBoxProjection::Uncertain => {
                    if matches!(far_projection, SourceBBoxProjection::Uncertain) {
                        observation.far_children_fail_open += 1;
                    }
                    visit_doom_seg_classic_bsp_child(
                        map,
                        far,
                        viewer,
                        heading,
                        occluders,
                        solid_ranges,
                        ancestors,
                        watched_subsectors,
                        observation,
                    )?;
                }
            }
            ancestors.pop();
            Ok(())
        }
    }
}

#[cfg(test)]
fn record_watched_subsector_elision(
    observation: &mut DoomSegClassicBspObservation,
    node: u16,
    reason: &str,
    subsectors: &[u16],
    interval: Option<[usize; 2]>,
    covering_range: Option<[usize; 2]>,
) {
    if !subsectors.is_empty() {
        observation.watched_subsector_elisions.push(format!(
            "node={node}:reason={reason}:subsectors={subsectors:?}:interval={interval:?}:covering-range={covering_range:?}"
        ));
    }
}

#[cfg(test)]
fn doom_bsp_child_contains_subsector(map: &DoomMapCore, child: DoomBspChild, target: u16) -> bool {
    let mut visited_nodes = HashSet::new();
    doom_bsp_child_contains_subsector_inner(map, child, target, &mut visited_nodes)
}

#[cfg(test)]
fn doom_bsp_child_contains_subsector_inner(
    map: &DoomMapCore,
    child: DoomBspChild,
    target: u16,
    visited_nodes: &mut HashSet<u16>,
) -> bool {
    match child {
        DoomBspChild::Subsector(index) => index == target,
        DoomBspChild::Node(index) => {
            if !visited_nodes.insert(index) {
                return false;
            }
            let contains = map.nodes.get(usize::from(index)).is_some_and(|node| {
                doom_bsp_child_contains_subsector_inner(
                    map,
                    node.right_child,
                    target,
                    visited_nodes,
                ) || doom_bsp_child_contains_subsector_inner(
                    map,
                    node.left_child,
                    target,
                    visited_nodes,
                )
            });
            visited_nodes.remove(&index);
            contains
        }
    }
}

#[cfg(test)]
fn admit_doom_seg_classic(
    map: &DoomMapCore,
    seg: &doom_map_provider::DoomSeg,
    viewer: [i16; 2],
    heading: f64,
    occluders: &BTreeMap<u32, doom_geometry_provider::DoomSegOccluderObservation>,
    solid_ranges: &mut Vec<[usize; 2]>,
    observation: &mut DoomSegClassicBspObservation,
) {
    const HALF_FOV: f64 = std::f64::consts::FRAC_PI_4;
    observation.source_segs_visited += 1;
    if seg.linedef == 247 {
        observation.hut_linedef_segs_visited += 1;
    }
    let start = &map.vertices[usize::from(seg.start_vertex)];
    let end = &map.vertices[usize::from(seg.end_vertex)];
    match source_seg_facing(viewer, [start.x, start.y], [end.x, end.y]) {
        SourceSegFacing::Back => {
            observation.backface_rejected += 1;
            return;
        }
        SourceSegFacing::EdgeOn => {
            observation.edge_on += 1;
            return;
        }
        SourceSegFacing::Front => {}
    }
    let forward = [heading.cos(), heading.sin()];
    let right = [-forward[1], forward[0]];
    let project = |point: [i16; 2]| {
        let relative = [
            f64::from(point[0] - viewer[0]),
            f64::from(point[1] - viewer[1]),
        ];
        let depth = relative[0] * forward[0] + relative[1] * forward[1];
        let lateral = relative[0] * right[0] + relative[1] * right[1];
        (depth, lateral.atan2(depth))
    };
    let (start_depth, start_angle) = project([start.x, start.y]);
    let (end_depth, end_angle) = project([end.x, end.y]);
    if (start_depth <= 0.0 && end_depth <= 0.0)
        || source_segment_outside_horizontal_fov(start_angle, end_angle, HALF_FOV)
    {
        observation.outside_fov_rejected += 1;
        return;
    }
    let authority = occluders
        .get(&seg.source.record_index)
        .expect("every source SEG is classified");
    let solid = authority.kind != doom_geometry_provider::DoomSegOccluderKind::Open;
    observation
        .admitted_seg_records
        .insert(seg.source.record_index);
    observation.admitted_seg_order.push(seg.source.record_index);
    if seg.linedef == 247 {
        observation.hut_linedef_segs_admitted += 1;
    }
    if solid && start_depth > 0.0 && end_depth > 0.0 {
        observation.solid_admitted += 1;
        let interval = source_fov_column_interval(start_angle, end_angle, HALF_FOV, 320);
        if merge_solid_range(solid_ranges, interval) {
            observation.solid_range_fully_covered += 1;
        } else {
            observation.solid_range_contributors += 1;
        }
    } else if solid {
        // A wall crossing the viewer plane must remain present, but its
        // unclipped behind-view endpoint cannot safely close a screen range.
        observation.near_plane_fail_open += 1;
    } else {
        observation.pass_admitted += 1;
    }
    if observation.samples.len() < 8 {
        observation.samples.push(format!(
            "seg={} line={} kind={:?} admission={}",
            seg.source.record_index,
            seg.linedef,
            authority.kind,
            if solid { "solid" } else { "pass" },
        ));
    }
}

fn observe_doom_seg_classic_admission(
    map: &DoomMapCore,
    viewer: [i16; 2],
    heading: f64,
) -> PlatformResult<DoomSegClassicAdmissionObservation> {
    const HALF_FOV: f64 = std::f64::consts::FRAC_PI_4;
    let forward = [heading.cos(), heading.sin()];
    let right = [-forward[1], forward[0]];
    let occluders = observe_doom_seg_occluders(map)?
        .into_iter()
        .map(|observation| (observation.source_seg.record_index, observation))
        .collect::<BTreeMap<_, _>>();
    let mut result = DoomSegClassicAdmissionObservation {
        source_segs: map.segs.len(),
        ..Default::default()
    };
    let order = resolve_doom_viewer_subsector_order(map, viewer)?;
    let order_by_source = order
        .iter()
        .enumerate()
        .map(|(rank, source)| (source.record_index, rank))
        .collect::<BTreeMap<_, _>>();
    let mut ordered_segs = map
        .segs
        .iter()
        .filter_map(|seg| {
            let subsector = map.subsectors.iter().find(|subsector| {
                let first = usize::from(subsector.first_seg);
                let end = first + usize::from(subsector.seg_count);
                (first..end).any(|index| map.segs[index].source == seg.source)
            })?;
            Some((*order_by_source.get(&subsector.source.record_index)?, seg))
        })
        .collect::<Vec<_>>();
    ordered_segs.sort_by_key(|(rank, seg)| (*rank, seg.source.record_index));
    let mut solid_ranges = Vec::<[usize; 2]>::new();
    for (rank, seg) in ordered_segs {
        let start = &map.vertices[usize::from(seg.start_vertex)];

        let end = &map.vertices[usize::from(seg.end_vertex)];
        let facing = source_seg_facing(viewer, [start.x, start.y], [end.x, end.y]);
        match facing {
            SourceSegFacing::Back => {
                result.backface_rejected += 1;
                continue;
            }
            SourceSegFacing::EdgeOn => {
                result.edge_on += 1;
                continue;
            }
            SourceSegFacing::Front => {}
        }
        let project_angle = |point: [i16; 2]| {
            let relative = [
                f64::from(point[0] - viewer[0]),
                f64::from(point[1] - viewer[1]),
            ];
            let depth = relative[0] * forward[0] + relative[1] * forward[1];
            let lateral = relative[0] * right[0] + relative[1] * right[1];
            (depth, lateral.atan2(depth))
        };
        let (start_depth, start_angle) = project_angle([start.x, start.y]);
        let (end_depth, end_angle) = project_angle([end.x, end.y]);

        if (start_depth <= 0.0 && end_depth <= 0.0)
            || source_segment_outside_horizontal_fov(start_angle, end_angle, HALF_FOV)
        {
            result.outside_fov_rejected += 1;
            continue;
        }
        let authority = occluders
            .get(&seg.source.record_index)
            .expect("every source SEG is classified");
        let solid = authority.kind != doom_geometry_provider::DoomSegOccluderKind::Open;
        if solid && start_depth > 0.0 && end_depth > 0.0 {
            result.solid_admitted += 1;
            let interval = source_fov_column_interval(start_angle, end_angle, HALF_FOV, 320);
            if merge_solid_range(&mut solid_ranges, interval) {
                result.solid_range_fully_covered += 1;
            } else {
                result.solid_range_contributors += 1;
            }
        } else if solid {
            result.near_plane_fail_open += 1;
        } else {
            result.pass_admitted += 1;
        }
        if result.samples.len() < 8 {
            result.samples.push(format!(
                "seg={} line={} rank={rank} facing=front start-depth={start_depth:.1} end-depth={end_depth:.1} kind={:?} admission={}",
                seg.source.record_index,
                seg.linedef,
                authority.kind,
                if solid { "solid" } else { "pass" },
            ));
        }
    }

    result.solid_range_covered_columns = solid_ranges
        .iter()
        .map(|[first, last]| last - first + 1)
        .sum();
    Ok(result)
}

/// Converts a bounded source horizontal-FOV interval to inclusive diagnostic
/// columns through a perspective plane. Classic Doom's `viewangletox` lookup
/// performs this same arc-to-plane kind of mapping; this bounded control does
/// not reproduce its exact fixed-point table.
fn source_fov_column_interval(
    first_angle: f64,
    second_angle: f64,
    half_fov: f64,
    columns: usize,
) -> [usize; 2] {
    let column = |angle: f64| {
        let normalized = angle.clamp(-half_fov, half_fov).tan() / half_fov.tan();
        (((normalized + 1.0) * 0.5) * columns as f64) as usize
    };
    let first = column(first_angle).min(columns - 1);
    let second = column(second_angle).min(columns - 1);
    [first.min(second), first.max(second)]
}

/// A segment is horizontally outside only when both endpoint bearings lie on
/// the same exterior side. Opposite exterior bearings cross the view and must
/// not be rejected merely because each endpoint is individually outside.
fn source_segment_outside_horizontal_fov(
    first_angle: f64,
    second_angle: f64,
    half_fov: f64,
) -> bool {
    (first_angle > half_fov && second_angle > half_fov)
        || (first_angle < -half_fov && second_angle < -half_fov)
}

/// Source-only far-child bbox outcome for the Stage 3B `R_CheckBBox` control.
/// It distinguishes a definitely outside FOV from geometry whose projection is
/// ambiguous and must remain fail-open.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceBBoxProjection {
    OutsideFov,
    Interval([usize; 2]),
    Uncertain,
}

/// Projects the two source bbox silhouette corners selected by the classic
/// `R_CheckBBox` position table. This is Doom-only protocol work, not a
/// generic bounding-box culler.
#[cfg(test)]
fn source_bbox_fov_column_interval(
    viewer: [i16; 2],
    heading: f64,
    bbox: [i16; 4],
    half_fov: f64,
    columns: usize,
) -> SourceBBoxProjection {
    let [top, bottom, left, right] = bbox;
    let box_x = if viewer[0] <= left {
        0
    } else if viewer[0] < right {
        1
    } else {
        2
    };
    let box_y = if viewer[1] >= top {
        0
    } else if viewer[1] > bottom {
        1
    } else {
        2
    };
    let box_position = box_y * 4 + box_x;
    if box_position == 5 {
        return SourceBBoxProjection::Uncertain;
    }
    // Matches `r_bsp.c`'s `checkcoord`: each value indexes Doom's decoded
    // bbox layout [top, bottom, left, right].
    const CHECK_COORD: [[usize; 4]; 12] = [
        [3, 0, 2, 1],
        [3, 0, 2, 0],
        [3, 1, 2, 0],
        [0, 0, 0, 0],
        [2, 0, 2, 1],
        [0, 0, 0, 0],
        [3, 1, 3, 0],
        [0, 0, 0, 0],
        [2, 0, 3, 1],
        [2, 1, 3, 1],
        [2, 1, 3, 0],
        [0, 0, 0, 0],
    ];
    let coordinates = CHECK_COORD[box_position];
    let source = [top, bottom, left, right];
    let points = [
        [source[coordinates[0]], source[coordinates[1]]],
        [source[coordinates[2]], source[coordinates[3]]],
    ];
    let forward = [heading.cos(), heading.sin()];
    let view_right = [-forward[1], forward[0]];
    let mut angles = Vec::with_capacity(2);
    for point in points {
        let relative = [
            f64::from(point[0] - viewer[0]),
            f64::from(point[1] - viewer[1]),
        ];
        let depth = relative[0] * forward[0] + relative[1] * forward[1];
        if depth <= 0.0 {
            return SourceBBoxProjection::Uncertain;
        }
        let lateral = relative[0] * view_right[0] + relative[1] * view_right[1];
        angles.push(lateral.atan2(depth));
    }
    let first_angle = angles[0];
    let second_angle = angles[1];
    let span = (first_angle - second_angle).abs();
    if span >= std::f64::consts::PI {
        return SourceBBoxProjection::Uncertain;
    }

    let minimum = first_angle.min(second_angle);
    let maximum = first_angle.max(second_angle);
    if maximum < -half_fov || minimum > half_fov {
        SourceBBoxProjection::OutsideFov
    } else {
        SourceBBoxProjection::Interval(source_fov_column_interval(
            minimum, maximum, half_fov, columns,
        ))
    }
}

/// Inserts one inclusive source screen interval into the current horizontal
/// solid-range union. Returns true when the interval was already fully closed.
/// This mirrors only the union property of Doom `solidsegs`, not its sentinel
/// representation, clipping details, or BSP bbox policy.
fn merge_solid_range(ranges: &mut Vec<[usize; 2]>, interval: [usize; 2]) -> bool {
    let fully_covered = ranges
        .iter()
        .any(|[first, last]| *first <= interval[0] && interval[1] <= *last);
    let mut merged = interval;
    let mut index = 0;
    while index < ranges.len() {
        let [first, last] = ranges[index];

        if last.saturating_add(1) < merged[0] || merged[1].saturating_add(1) < first {
            index += 1;
            continue;
        }
        merged[0] = merged[0].min(first);
        merged[1] = merged[1].max(last);
        ranges.remove(index);
    }
    ranges.insert(index, merged);
    fully_covered
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceSegFacing {
    Front,
    Back,
    EdgeOn,
}

/// Classic Doom treats the directed SEG's right side as its visible front.
/// This is source-data interpretation, intentionally separate from Tokimu
/// mesh normals, camera controls, or renderer culling.
fn source_seg_facing(viewer: [i16; 2], start: [i16; 2], end: [i16; 2]) -> SourceSegFacing {
    let segment = [i64::from(end[0] - start[0]), i64::from(end[1] - start[1])];
    let to_viewer = [
        i64::from(viewer[0] - start[0]),
        i64::from(viewer[1] - start[1]),
    ];
    let side = segment[0] * to_viewer[1] - segment[1] * to_viewer[0];
    if side < 0 {
        SourceSegFacing::Front
    } else if side > 0 {
        SourceSegFacing::Back
    } else {
        SourceSegFacing::EdgeOn
    }
}

fn observe_doom_seg_screen_grid(
    map: &DoomMapCore,
    eye_height: f32,
    per_column: bool,
    viewer: [i16; 2],
    angle: f64,
) -> PlatformResult<DoomSegScreenGridObservation> {
    observe_doom_seg_screen_grid_with_order(
        map,
        eye_height,
        per_column,
        viewer,
        angle,
        DoomSegScreenGridOrder::BspLeafThenSource,
    )
}

/// Runs the same bounded source grid with one declared diagnostic ordering.
/// The alternate nearest-segment order exists only to test whether coarse BSP
/// leaf order explains the retained depth inversions; it is not Doom parity.
fn observe_doom_seg_screen_grid_with_order(
    map: &DoomMapCore,
    eye_height: f32,
    per_column: bool,
    viewer: [i16; 2],
    angle: f64,
    ordering: DoomSegScreenGridOrder,
) -> PlatformResult<DoomSegScreenGridObservation> {
    const COLUMNS: usize = 320;
    const ROWS: usize = 200;
    const HALF_HORIZONTAL_FOV: f64 = std::f64::consts::FRAC_PI_4;
    let half_vertical_fov = ((ROWS as f64 / COLUMNS as f64) * HALF_HORIZONTAL_FOV.tan()).atan();
    let eye_height = f64::from(eye_height);
    let forward = [angle.cos(), angle.sin()];
    let right = [-forward[1], forward[0]];
    let order = resolve_doom_viewer_subsector_order(map, viewer)?;
    let order_by_source = order
        .iter()
        .enumerate()
        .map(|(rank, source)| (source.record_index, rank))
        .collect::<BTreeMap<_, _>>();
    let mut ordered_segs = map
        .segs
        .iter()
        .filter_map(|seg| {
            let subsector = map.subsectors.iter().position(|subsector| {
                let start = usize::from(subsector.first_seg);
                let end = start + usize::from(subsector.seg_count);
                (start..end).any(|index| map.segs[index].source == seg.source)
            })?;
            Some((
                order_by_source.get(&map.subsectors[subsector].source.record_index)?,
                seg,
            ))
        })
        .collect::<Vec<_>>();
    let candidates = resolve_doom_wall_candidates(map)?
        .into_iter()
        .map(|candidate| (candidate.source_linedef.record_index, candidate))
        .collect::<BTreeMap<_, _>>();
    let occluders = observe_doom_seg_occluders(map)?
        .into_iter()
        .map(|observation| (observation.source_seg.record_index, observation))
        .collect::<BTreeMap<_, _>>();
    let project = |point: [i16; 2]| {
        let relative = [
            f64::from(point[0] - viewer[0]),
            f64::from(point[1] - viewer[1]),
        ];
        let depth = relative[0] * forward[0] + relative[1] * forward[1];
        let lateral = relative[0] * right[0] + relative[1] * right[1];
        (depth, lateral.atan2(depth))
    };
    match ordering {
        DoomSegScreenGridOrder::BspLeafThenSource => {
            ordered_segs.sort_by_key(|(rank, seg)| (**rank, seg.source.record_index));
        }
        DoomSegScreenGridOrder::NearestSegmentToViewer => {
            ordered_segs.sort_by(|(_, left), (_, right)| {
                let left_start = &map.vertices[usize::from(left.start_vertex)];
                let left_end = &map.vertices[usize::from(left.end_vertex)];
                let right_start = &map.vertices[usize::from(right.start_vertex)];
                let right_end = &map.vertices[usize::from(right.end_vertex)];
                source_point_segment_distance_squared(
                    viewer,
                    [left_start.x, left_start.y],
                    [left_end.x, left_end.y],
                )
                .total_cmp(&source_point_segment_distance_squared(
                    viewer,
                    [right_start.x, right_start.y],
                    [right_end.x, right_end.y],
                ))
                .then_with(|| left.source.record_index.cmp(&right.source.record_index))
            });
        }
    }
    let column = |angle: f64| {
        ((angle.clamp(-HALF_HORIZONTAL_FOV, HALF_HORIZONTAL_FOV) + HALF_HORIZONTAL_FOV)
            / (2.0 * HALF_HORIZONTAL_FOV)
            * COLUMNS as f64) as usize
    };
    let row = |angle: f64| {
        ((half_vertical_fov - angle.clamp(-half_vertical_fov, half_vertical_fov))
            / (2.0 * half_vertical_fov)
            * ROWS as f64) as usize
    };

    let mut covered = vec![false; COLUMNS * ROWS];
    // This stays beside, rather than inside, the boolean coverage state so the
    // established falsified control retains its exact selection behavior.
    // It merely exposes cases where leaf/source order disagrees with local
    // ray depth for an attempted occluding write.
    let mut covering_depths = vec![None::<(f64, u32)>; COLUMNS * ROWS];
    let mut depth_order_inversions = 0usize;
    let mut depth_order_samples = Vec::new();
    let mut outside = 0usize;
    let mut fully_covered = 0usize;

    let mut partial = 0usize;
    let mut fully_visible = 0usize;
    let mut contributors = 0usize;
    let mut samples = Vec::new();
    let mut selected_seg_records = BTreeSet::new();
    for (rank, seg) in ordered_segs {
        let start = &map.vertices[usize::from(seg.start_vertex)];
        let end = &map.vertices[usize::from(seg.end_vertex)];
        let (start_depth, start_angle) = project([start.x, start.y]);
        let (end_depth, end_angle) = project([end.x, end.y]);
        if start_depth <= 0.0
            || end_depth <= 0.0
            || (start_angle.abs() > HALF_HORIZONTAL_FOV && end_angle.abs() > HALF_HORIZONTAL_FOV)
        {
            outside += 1;
            continue;
        }
        let candidate = candidates
            .get(&map.linedefs[usize::from(seg.linedef)].source.record_index)
            .expect("every SEG linedef has a resolved source wall");
        let (front, back) = match seg.direction {
            0 => (candidate.right.as_ref(), candidate.left.as_ref()),

            1 => (candidate.left.as_ref(), candidate.right.as_ref()),
            direction => {
                return Err(io::Error::other(format!(
                    "Stage 3B source SEG {} has unsupported direction {direction}",
                    seg.source.record_index
                ))
                .into())
            }
        };
        let front = front.expect("SEG direction names an existing owning side");
        let mut floor = map.sectors[usize::from(front.sector_index)].floor_height;
        let mut ceiling = map.sectors[usize::from(front.sector_index)].ceiling_height;
        if let Some(back) = back {
            let back_sector = &map.sectors[usize::from(back.sector_index)];
            floor = floor.min(back_sector.floor_height);
            ceiling = ceiling.max(back_sector.ceiling_height);
        }
        let left = column(start_angle).min(COLUMNS - 1);
        let right_column = column(end_angle).min(COLUMNS - 1);
        let (left, right_column) = (left.min(right_column), left.max(right_column));
        let rectangle_span = || {
            let vertical_angles = [
                (f64::from(floor) - eye_height).atan2(start_depth),
                (f64::from(ceiling) - eye_height).atan2(start_depth),
                (f64::from(floor) - eye_height).atan2(end_depth),
                (f64::from(ceiling) - eye_height).atan2(end_depth),
            ];
            let top = row(vertical_angles
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max))
            .min(ROWS - 1);
            let bottom = row(vertical_angles
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min))
            .min(ROWS - 1);
            [top.min(bottom), top.max(bottom)]
        };
        let vertical_spans = if per_column {
            (left..=right_column)
                .map(|x| {
                    let local_angle = -HALF_HORIZONTAL_FOV
                        + ((x as f64 + 0.5) / COLUMNS as f64) * (2.0 * HALF_HORIZONTAL_FOV);
                    let ray = [
                        forward[0] * local_angle.cos() + right[0] * local_angle.sin(),
                        forward[1] * local_angle.cos() + right[1] * local_angle.sin(),
                    ];
                    let depth =
                        source_ray_segment_depth(viewer, ray, [start.x, start.y], [end.x, end.y])
                            .unwrap_or_else(|| {
                                let fraction = if right_column == left {
                                    0.5
                                } else {
                                    (x - left) as f64 / (right_column - left) as f64
                                };
                                start_depth + (end_depth - start_depth) * fraction
                            });
                    let top = row((f64::from(ceiling) - eye_height).atan2(depth)).min(ROWS - 1);
                    let bottom = row((f64::from(floor) - eye_height).atan2(depth)).min(ROWS - 1);
                    ([top.min(bottom), top.max(bottom)], depth)
                })
                .collect::<Vec<_>>()
        } else {
            let depth = (start_depth + end_depth) * 0.5;
            vec![(rectangle_span(), depth); right_column - left + 1]
        };
        let mut cells = 0usize;
        let mut visible_cells = 0usize;
        for (offset, ([top, bottom], _depth)) in vertical_spans.iter().copied().enumerate() {
            let x = left + offset;
            for y in top..=bottom {
                cells += 1;
                visible_cells += usize::from(!covered[y * COLUMNS + x]);
            }
        }
        let result = if visible_cells == 0 {
            fully_covered += 1;
            "covered"
        } else if visible_cells == cells {
            fully_visible += 1;
            "visible"
        } else {
            partial += 1;
            "partial"
        };
        if visible_cells > 0 {
            selected_seg_records.insert(seg.source.record_index);
        }
        let authority = occluders
            .get(&seg.source.record_index)
            .expect("every source SEG is classified");
        let closes = authority.kind != doom_geometry_provider::DoomSegOccluderKind::Open;
        if closes {
            for (offset, ([top, bottom], depth)) in vertical_spans.iter().copied().enumerate() {
                let x = left + offset;
                for y in top..=bottom {
                    let cell = y * COLUMNS + x;
                    if let Some((prior_depth, prior_seg)) = covering_depths[cell] {
                        if depth + 0.01 < prior_depth {
                            depth_order_inversions += 1;
                            if depth_order_samples.len() < 8 {
                                depth_order_samples.push(format!(
                                    "cell=({x},{y}) prior-seg={prior_seg} prior-depth={prior_depth:.3} later-nearer-seg={} later-depth={depth:.3}",
                                    seg.source.record_index,
                                ));
                            }
                        }
                    }
                    // Retain the first closing SEG exactly as the existing
                    // boolean control does; do not let this audit repair the
                    // experiment while it is being measured.
                    covering_depths[cell].get_or_insert((depth, seg.source.record_index));
                    covered[cell] = true;
                }
            }
            contributors += 1;
        }
        if seg.linedef == 247 || samples.len() < 8 {
            let [top, bottom] = rectangle_span();
            samples.push(format!(
                "seg={} line={} rank={} horizontal=[{left}..{right_column}] enclosing-vertical=[{top}..{bottom}] mode={} visible={visible_cells}/{cells} authority={:?} result={result} contributor={closes}",
                seg.source.record_index,
                seg.linedef,
                rank,
                if per_column { "per-column" } else { "rectangle" },
                authority.kind
            ));
        }
    }
    Ok(DoomSegScreenGridObservation {
        selected_seg_records,
        outside,
        fully_covered,
        partial,
        fully_visible,
        contributors,
        covered_cells: covered.iter().filter(|covered| **covered).count(),
        depth_order_inversions,
        depth_order_samples,
        samples,
    })
}

/// Returns the positive depth at which a source-space camera ray meets one
/// source SEG, when that intersection lies on the finite SEG. This is retained
/// only for the Stage 3B per-column diagnostic grid; it does not define a
/// generic ray query or visibility capability.
fn source_ray_segment_depth(
    viewer: [i16; 2],
    ray: [f64; 2],
    start: [i16; 2],
    end: [i16; 2],
) -> Option<f64> {
    let offset = [
        f64::from(start[0] - viewer[0]),
        f64::from(start[1] - viewer[1]),
    ];
    let segment = [f64::from(end[0] - start[0]), f64::from(end[1] - start[1])];
    let cross = |left: [f64; 2], right: [f64; 2]| left[0] * right[1] - left[1] * right[0];
    let denominator = cross(ray, segment);
    if denominator.abs() <= f64::EPSILON {
        return None;
    }
    let depth = cross(offset, segment) / denominator;
    let progression = cross(offset, ray) / denominator;
    (depth > 0.0 && (0.0..=1.0).contains(&progression)).then_some(depth)
}

/// Squared source-space distance from a point to one finite SEG. This is only
/// a coarse ordering probe for Stage 3B; it does not claim camera-ray order,
/// source visibility, or generic spatial-query meaning.
fn source_point_segment_distance_squared(point: [i16; 2], start: [i16; 2], end: [i16; 2]) -> f64 {
    let offset = [
        f64::from(point[0] - start[0]),
        f64::from(point[1] - start[1]),
    ];
    let segment = [f64::from(end[0] - start[0]), f64::from(end[1] - start[1])];

    let length_squared = segment[0] * segment[0] + segment[1] * segment[1];
    let progression = if length_squared <= f64::EPSILON {
        0.0
    } else {
        ((offset[0] * segment[0] + offset[1] * segment[1]) / length_squared).clamp(0.0, 1.0)
    };
    let nearest = [
        f64::from(start[0]) + progression * segment[0],
        f64::from(start[1]) + progression * segment[1],
    ];
    let delta = [
        f64::from(point[0]) - nearest[0],
        f64::from(point[1]) - nearest[1],
    ];
    delta[0] * delta[0] + delta[1] * delta[1]
}

/// Returns contiguous not-yet-covered runs as offsets within one projected SEG
/// interval. The caller owns all screen-column meaning and source conversion.
fn visible_column_runs(covered: &[bool]) -> Vec<[usize; 2]> {
    let mut runs = Vec::new();
    let mut start = None;
    for (index, is_covered) in covered.iter().copied().enumerate() {
        match (start, is_covered) {
            (None, false) => start = Some(index),
            (Some(first), true) => {
                runs.push([first, index]);
                start = None;
            }
            _ => {}
        }
    }
    if let Some(first) = start {
        runs.push([first, covered.len()]);
    }
    runs
}

/// Maps a source SEG's endpoints onto the owning linedef progression. This is
/// Doom-only retained source math for the Stage 3B diagnostic lowering path.
fn source_seg_linedef_interval(map: &DoomMapCore, seg: &doom_map_provider::DoomSeg) -> [f64; 2] {
    let line = &map.linedefs[usize::from(seg.linedef)];
    let line_start = &map.vertices[usize::from(line.start_vertex)];
    let line_end = &map.vertices[usize::from(line.end_vertex)];
    let delta = [
        f64::from(line_end.x - line_start.x),
        f64::from(line_end.y - line_start.y),
    ];
    let length_squared = delta[0].mul_add(delta[0], delta[1] * delta[1]);
    let progression = |vertex: u16| {
        let point = &map.vertices[usize::from(vertex)];
        ((f64::from(point.x - line_start.x) * delta[0])
            + (f64::from(point.y - line_start.y) * delta[1]))
            / length_squared
    };
    let start = progression(seg.start_vertex);
    let end = progression(seg.end_vertex);
    [start.min(end), start.max(end)]
}

#[cfg(test)]
fn source_sky_sectors(spans: &DoomSegClassicPlaneSpanObservation) -> BTreeSet<u32> {
    spans
        .keys
        .iter()
        .filter(|(key, _)| key.kind == DoomSegClassicPlaneKind::Ceiling && key.texture == "F_SKY1")
        .flat_map(|(_, instances)| instances.iter())
        .flat_map(|instance| instance.source_sectors.iter().copied())
        .collect()
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
#[path = "static_scene/tests.rs"]
mod tests;
