//! Native first-frame proof for the Slice 5B static E1M1 presentation policy.
//!
//! The WAD is read only at this corpus edge. `tokimu-render` receives ordinary
//! meshes, texture bytes, materials, and one explicit opaque 3D pipeline.

use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    env, fs, io,
    process::Command,
    sync::Arc,
    time::Instant,
};

use archive_provider::{ArchiveFormat, ArchiveReadLimits, ZipArchiveProvider};
use doom_geometry_provider::{
    clip_doom_seg_textured_wall_triangle_to_linedef_interval, doom_point_to_tokimu,
    locate_doom_point_subsector, lower_doom_paired_sky_boundary_triangles,
    lower_doom_sector_bounded_subsector_surfaces, lower_doom_seg_textured_wall_triangles,
    lower_doom_source_bounded_subsector_surfaces, lower_doom_subsector_surfaces,
    lower_doom_textured_wall_triangles, observe_doom_classic_bsp,
    observe_doom_classic_bsp_suppressing_solid_range_source_seg,
    observe_doom_classic_bsp_without_solid_range_pruning,
    observe_doom_classic_vertical_clip_state as observe_shared_doom_classic_vertical_clip_state,
    observe_doom_seg_occluders, observe_doom_seg_plane_marks, observe_doom_sky_surfaces,
    observe_doom_two_sided_middle_textures, project_doom_sector_runtime_heights,
    reconstruct_doom_ordered_wall_fragments, resolve_doom_linedef_subsector_membership,
    resolve_doom_subsector_bsp_paths, resolve_doom_subsector_regions,
    resolve_doom_subsector_sector_ownership, resolve_doom_viewer_subsector_order,
    resolve_doom_wall_candidates, DoomClassicBspObservation, DoomSectorRuntimeHeightSnapshot,
    DoomSegClassicPlaneKind, DoomSegClassicPlaneSpanObservation,
    DoomSegClassicVerticalClipObservation, DoomSegPlaneMarkObservation,
    DoomSegTexturedWallTriangle, DoomSourceBoundedSurfaceAudit, DoomSurfacePlane,
    DoomTextureExtent, DoomWallTextureRole,
};
#[cfg(test)]
use doom_geometry_provider::{DoomSegClassicPlaneInstance, DoomSegClassicPlaneKey};
use doom_map_provider::{
    decode_doom_map_core, resolve_doom_player_one_start, DoomBspChild, DoomMapCore, DoomSector,
    DoomThing,
};
use doom_raster_provider::{
    decode_doom_raster_globals, decode_doom_sprite_frame_rotations, decode_doom_sprite_patch,
    indexed_image_from_doom_patch, lower_doom_indexed_image, DoomFlatDecodeLimits,
    DoomPatchDecodeLimits, DoomRasterDecodeLimits, DoomSpriteFrameRotation,
    DoomTextureComposeLimits, DoomTextureDecodeLimits,
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
    resolve_doom_line_activation, resolve_doom_shareware_switch_texture, DoomDownWaitUpStayPhase,
    DoomDownWaitUpStayPolicy, DoomDownWaitUpStayRuntime, DoomLineActivation,
    DoomLineActivationIntent, DoomLineActivationRequest, DoomLineActivationResolution,
    DoomLineActivationSource, DoomManualDoorPhase, DoomManualDoorPolicy, DoomManualDoorRuntime,
    DoomSwitchTextureChange, DoomSwitchTextureSlot, DoomTurboLowerFloorPhase,
    DoomTurboLowerFloorPolicy, DoomTurboLowerFloorRuntime,
};
pub use hello_doom_e1m1::{
    assemble_experimental_masked_middle_cutouts, assemble_static_opaque_flats,
    assemble_static_opaque_walls, build_experimental_cutout_draw_plan,
    build_experimental_cutout_texture_uploads, build_static_draw_plan,
    build_static_texture_uploads, classify_static_draw_frustum_rejection,
    classify_static_draw_sphere_frustum_rejection, doom_heading_forward,
    lower_static_seg_wall_triangle, lower_static_wall_triangle,
    observe_doom_ground_frame_with_embedding, observer_direction, observer_right,
    observer_yaw_from_forward, prepare_e1m1_flat_textures,
    prepare_e1m1_static_sky_panorama_texture, prepare_e1m1_wall_texture_extents,
    prepare_e1m1_wall_textures, prepared_e1m1_masked_middle_texture_names,
    reembed_comparative_mesh, DoomComparativeEmbedding, PreparedE1m1Flats,
    PreparedE1m1MaskedMiddleCutouts, PreparedE1m1Walls, PreparedStaticTexture, StaticDrawAabb,
    StaticDrawPlanEntry, StaticDrawSource, StaticFlatLoweringError, StaticTextureEligibility,
    StaticTextureUpload,
};
pub use hello_doom_e1m1::{lower_static_flat_triangle, FlatExtent, StaticTextureSourceKind};
use hello_doom_visibility_conformance::{
    model_authoritative_sky_regions, observe_authoritative_sky_source_depth_approximation,
    prepare_authoritative_sky_source_depth_declarations,
    prepare_authoritative_sky_submission_local_geometry, AuthoritativeSkyViewIdentity,
    DoomFixtureViewer, DoomVisibilityFixture, SubmissionIdentity, SubmissionLocalGeometryLimits,
};
use raster_image_corpus::{decode_png, prepare_renderer_texture, DecodeLimits, TextureUse};
use resource_space::{
    AddressCasePolicy, FolderId, InMemoryResourceSpace, ResourceMetadata, ResourceName,
    ResourceRootDescriptor, ResourceRootId, StoreId,
};
use resource_space_archive::InspectArchiveResourceRequest;
use tokimu::experimental_submission_local_geometry::{
    ExperimentalLocalGeometryDraw, ExperimentalSubmissionIdentity,
    ExperimentalSubmissionLocalGeometry, ExperimentalSubmissionLocalGeometryBuilder,
};
use tokimu::{
    run_window_with_app, BlendMode, Camera, CameraHandle, CategoricalCutout, ClearCommand, Color,
    ColorWriteMask, CullMode, CutoutComparison, CutoutThreshold, DepthTest, DrawMeshCommand,
    FrameOutcome, Instance2d, Material, MaterialHandle, Mesh, MeshHandle, NativeWindow, Pipeline,
    PipelineHandle, PipelineKind, PipelineRenderState, PlatformEventHandler, PlatformInputEvent,
    PlatformResult, RenderCommand, Renderer, Rgba8TextureColorSpace, Rgba8TextureDescriptor,
    StencilMode, Texture, TextureAddressMode, TextureFilter, TextureHandle, TextureSampler,
    WgpuBackend, WindowConfig,
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

#[path = "static_scene/audio.rs"]
mod audio;
#[path = "static_scene/candidate_selection/mod.rs"]
mod candidate_selection;
#[path = "static_scene/controls/mod.rs"]
mod controls;
#[path = "static_scene/diagnostics/mod.rs"]
mod diagnostics;
#[path = "static_scene/presentation/mod.rs"]
mod presentation;
#[path = "static_scene/render_strategies/mod.rs"]
mod render_strategies;
#[path = "static_scene/runtime/mod.rs"]
mod runtime;
#[path = "static_scene/startup/mod.rs"]
mod startup;

use audio::*;
use candidate_selection::*;
use controls::{inspection_movement_delta, release_navigation_keys};
use diagnostics::*;
use presentation::*;
use render_strategies::TrialRenderStrategy;
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
const DOOM_SKY_BOUNDARY_MESH_BASE: u64 = 9_001_000;
const DOOM_SOURCE_SKY_PLANE_MESH_BASE: u64 = 9_002_000;
const DOOM_VIEWER_SKY_SPAN_MESH: MeshHandle = MeshHandle(9_003_000);
const CANDIDATE1_SKY_DEPTH_MATERIAL: MaterialHandle = MaterialHandle(9_004_000);
const CANDIDATE1_CLIP_CAMERA: CameraHandle = CameraHandle(9_004_000);
const ORDERED_COVERAGE_CUTOUT_MESH_BASE: u64 = 8_000_000;
const ORDERED_COVERAGE_DYNAMIC_MESH_BASE: u64 = 8_500_000;
const DOOM_THING_SPRITE_TEXTURE_BASE: u64 = 9_100_000;
const DOOM_THING_SPRITE_MATERIAL_BASE: u64 = 9_110_000;
const DOOM_THING_SPRITE_MESH_BASE: u64 = 9_200_000;
const WALK_SPEED: f32 = 240.0;
const RUN_SPEED_MULTIPLIER: f32 = 2.0;
const WALK_RADIUS: f32 = 16.0;
// id Software's released `p_local.h` declares USERANGE as 64 map units.

// This remains a Doom-corpus interaction policy, not a generic Tokimu reach.
const CLASSIC_USE_RANGE: f32 = 64.0;
const DOOM_TIC_SECONDS: f64 = 1.0 / 35.0;

struct App {
    map_name: String,
    import_warnings: Vec<String>,
    runtime_warnings: Vec<String>,
    preparation_timings: DoomPreparationTimings,
    upload_cpu_us: Option<u128>,
    last_frame_cpu_us: Option<u128>,
    available_maps: Vec<String>,
    launch_arguments: Vec<String>,
    map_rotation_exit_requested: bool,
    source_exit_level_requested: bool,
    live_audio: Option<DoomLiveAudio>,
    discovered_secret_sectors: BTreeSet<u32>,
    secret_sector_total: usize,
    renderer: Option<WgpuBackend>,
    render_strategy_name: &'static str,
    render_strategy_stages: &'static str,
    topology_inventory: TopologyContributionInventory,
    /// Doom-private shadow classification visualization. Geometry membership
    /// remains the unchanged global-full inventory in every focus mode.
    bsp_diagnostic_enabled: bool,
    bsp_diagnostic_focus: BspDiagnosticFocus,
    draws: Vec<StaticDrawPlanEntry>,
    uploads: Vec<StaticTextureUpload>,
    cutout_draws: Vec<StaticDrawPlanEntry>,

    cutout_uploads: Vec<StaticTextureUpload>,
    thing_sprites: Vec<DoomThingSprite>,
    sprite_frames: Vec<DoomSpriteFrameRotation>,
    sprite_uploads: Vec<DoomSpriteTextureUpload>,
    sprite_meshes: Vec<Mesh>,
    sprite_selected_materials: Vec<MaterialHandle>,
    sprite_last_viewer_source_position: Option<[f32; 2]>,
    thing_sprite_states: Vec<hello_doom_e1m1::things::DoomThingRuntimeState>,
    thing_sprite_tick_accumulator: f64,
    thing_sprite_total_ticks: u64,
    thing_sprite_active: Vec<bool>,
    monster_chase_live: bool,
    monster_runtime_states: Vec<Option<DoomMonsterRuntimeState>>,
    monster_sight_world: hello_doom_e1m1::perception::DoomMonsterSightWorld,
    actor_movement_world: hello_doom_e1m1::collision::DoomActorMovementWorld,
    player_inventory: hello_doom_e1m1::things::DoomPlayerInventory,
    thing_combat_states: Vec<Option<hello_doom_e1m1::combat::DoomCombatActorState>>,
    play_random: hello_doom_e1m1::combat::DoomPlayRandom,
    diagnostic_sky_draws: Vec<StaticDrawPlanEntry>,
    diagnostic_sky_enabled: bool,
    diagnostic_sky_records: Vec<String>,
    doom_sky_texture: PreparedStaticTexture,
    doom_sky_mesh: Mesh,
    doom_sky_boundary_draws: Vec<DoomSkyBoundaryDepthDraw>,
    doom_sky_enabled: bool,
    source_sky_plane_depth_enabled: bool,
    source_sky_plane_depth_global_control: bool,
    candidate1_sky_depth_enabled: bool,
    skywall_parity_enabled: bool,
    source_sky_plane_selected: Vec<bool>,
    cutout_mesh_base: u64,
    include_cutouts: bool,
    pipeline: PipelineHandle,
    opaque_depth_prepass_pipeline: Option<PipelineHandle>,
    one_sided_wall_depth_prepass_pipeline: Option<PipelineHandle>,
    cutout_pipeline: Option<PipelineHandle>,
    cutout_depth_prepass_pipeline: Option<PipelineHandle>,
    sprite_pipeline: Option<PipelineHandle>,
    sprite_depth_prepass_pipeline: Option<PipelineHandle>,
    doom_sky_pipeline: Option<PipelineHandle>,
    doom_sky_boundary_pipeline: Option<PipelineHandle>,
    diagnostic_sky_pipeline: Option<PipelineHandle>,
    candidate1_sky_depth_pipeline: Option<PipelineHandle>,
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
    active_switch_textures: Vec<DoomSwitchTextureChange>,
    scrolling_wall_sidedefs: BTreeSet<u32>,
    wall_material_inverse_widths: BTreeMap<u64, f32>,
    scrolling_wall_tick_accumulator: f64,
    scrolling_wall_total_ticks: u64,
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
    /// Experimental Doom-private walkabout mode that retains complete global
    /// contributions only for source owners reached by ordered coverage.
    source_covered_domain_filter: bool,
    /// Experimental live realization of exact final source-cell support as
    /// finite ordinary world-space declarations.
    source_occurrence_support_filter: bool,
    /// Experimental family-isolated preparation: final ordered wall
    /// fragments over untouched global-full planes.
    final_wall_occurrence_filter: bool,
    /// Immutable decoded/source preparation input retained by strategies B/C.
    /// The runtime changes only the explicit viewer pose passed to the Doom
    /// preparation; it does not mutate this source snapshot or teach the
    /// renderer about Doom traversal.
    ordered_coverage_source: Option<Box<SceneInput>>,
    /// Identity of the last completely installed Doom-private preparation.
    /// It prevents stationary frames from rebuilding and re-uploading an
    /// identical declaration set; assignment occurs only after a successful
    /// prepare-then-replace transaction.
    ordered_preparation_identity: Option<OrderedPreparationIdentity>,
    /// The classic-plane presentation is reconstructed for exactly the
    /// source-spawn observer. Allowing ordinary free-look/movement would make
    /// geometry outside that retained view look like reconstruction evidence.
    fixed_reconstruction_camera: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct OrderedPreparationIdentity {
    source_position: [i16; 2],
    source_heading_bits: u64,
    eye_height: i16,
    door_ceilings: Vec<(u32, i16)>,
    turbo_floors: Vec<(u32, i16)>,
    platform_floors: Vec<(u32, i16)>,
}

#[derive(Clone)]
struct SceneInput {
    map_name: String,
    import_warnings: Vec<String>,
    preparation_timings: DoomPreparationTimings,
    audio_assets: Option<DoomLiveAudioAssets>,
    available_maps: Vec<String>,
    things: Vec<DoomThing>,
    sprite_frames: Vec<DoomSpriteFrameRotation>,
    thing_sprites: Vec<DoomThingSprite>,
    sprite_uploads: Vec<DoomSpriteTextureUpload>,
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
    monster_sight_world: hello_doom_e1m1::perception::DoomMonsterSightWorld,
    actor_movement_world: hello_doom_e1m1::collision::DoomActorMovementWorld,
    reject_report: DoomRejectReport,
    topology_report: DoomTopologyReport,
    bsp_bounds_audit: Option<DoomBspBoundsAudit>,
    source_bounded_surface_audit: Option<DoomSourceBoundedSurfaceAudit>,
    membership_selection: DoomMembershipSelectionInput,
    activation_source: DoomLineActivationSource,
    door_geometry_source: DoomDynamicDoorGeometrySource,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct DoomPreparationTimings {
    wad_parse_us: u128,
    map_decode_us: u128,
    lowering_us: u128,
}

#[derive(Clone, Debug)]
struct DoomThingSprite {
    source: doom_map_provider::DoomSourceRecord,
    kind: u16,
    source_position: [i16; 2],
    source_angle: u16,
    floor_height: i16,
    source_sector: u32,
    sprite: &'static str,
    initial_frame: char,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DoomMonsterRuntimeState {
    source_position: [f32; 2],
    source_angle_degrees: f32,
    floor_height: i16,
    source_sector: u32,
    awake: bool,
    look_tics: u16,
    chase_tics: u16,
    chase_state_index: u8,
    escape_heading_degrees: Option<f32>,
    escape_steps_remaining: u8,
}

/// Corpus-private mutable gameplay payload. Imported WAD/map records and
/// renderer resources are deliberately absent from this snapshot.
#[derive(Clone, Debug, PartialEq)]
struct DoomGameplaySnapshot {
    player_inventory: hello_doom_e1m1::things::DoomPlayerInventory,
    thing_sprite_states: Vec<hello_doom_e1m1::things::DoomThingRuntimeState>,
    thing_sprite_total_ticks: u64,
    thing_sprite_active: Vec<bool>,
    thing_combat_states: Vec<Option<hello_doom_e1m1::combat::DoomCombatActorState>>,
    play_random: hello_doom_e1m1::combat::DoomPlayRandom,
    monster_runtime_states: Vec<Option<DoomMonsterRuntimeState>>,
}

#[derive(Clone, Debug)]
struct DoomSpriteTextureUpload {
    source_lump_index: u32,
    source_name: String,
    width: u16,
    height: u16,
    left_offset: i16,
    top_offset: i16,
    texture: TextureHandle,
    material: MaterialHandle,
    descriptor: Rgba8TextureDescriptor,
    rgba8: Vec<u8>,
    material_value: Material,
}

impl DoomSpriteTextureUpload {
    fn opaque_row_bounds(&self) -> Option<[usize; 2]> {
        let width = usize::from(self.width);
        let mut first = None;
        let mut last = None;
        for (pixel, rgba) in self.rgba8.chunks_exact(4).enumerate() {
            if rgba[3] == 0 {
                continue;
            }
            let row = pixel / width;
            first.get_or_insert(row);
            last = Some(row);
        }
        first.zip(last).map(|(first, last)| [first, last])
    }

    /// A software-rendered Doom sprite may cover pixels below its Thing
    /// origin and still be composited over a floor visplane. A physical GPU
    /// billboard would instead intersect the floor depth. Lift only enough to
    /// put the lowest covered texel edge on the owning floor; transparent
    /// patch padding and already floor-aligned sprites do not move.
    fn floor_clearance_lift(&self) -> f32 {
        self.opaque_row_bounds().map_or(0.0, |[_, last]| {
            ((last + 1) as f32 - f32::from(self.top_offset)).max(0.0)
        })
    }
}

fn build_doom_thing_sprite_mesh(
    upload: &DoomSpriteTextureUpload,
    mirrored: bool,
    viewer_source_position: [f32; 2],
    source_position: [f32; 2],
    floor_height: i16,
    embedding: DoomComparativeEmbedding,
) -> PlatformResult<Mesh> {
    let thing_source = source_position;
    let mut toward_viewer = Vec3::new(
        viewer_source_position[0] - thing_source[0],
        0.0,
        viewer_source_position[1] - thing_source[1],
    );
    if toward_viewer.length_squared() <= f32::EPSILON {
        toward_viewer = Vec3::new(0.0, 0.0, 1.0);
    } else {
        toward_viewer = toward_viewer.normalize();
    }
    let source_right = Vec3::new(toward_viewer.z, 0.0, -toward_viewer.x);
    let left = -f32::from(upload.left_offset);
    let right = f32::from(upload.width) - f32::from(upload.left_offset);
    let floor_lift = upload.floor_clearance_lift();
    let top = f32::from(floor_height) + f32::from(upload.top_offset) + floor_lift;
    let bottom = top - f32::from(upload.height);
    let source_center = Vec3::new(thing_source[0], 0.0, thing_source[1]);
    let source_positions = [
        source_center + source_right * left + Vec3::Y * top,
        source_center + source_right * left + Vec3::Y * bottom,
        source_center + source_right * right + Vec3::Y * bottom,
        source_center + source_right * left + Vec3::Y * top,
        source_center + source_right * right + Vec3::Y * bottom,
        source_center + source_right * right + Vec3::Y * top,
    ];
    let positions = source_positions
        .map(|position| {
            embedding
                .lift_direction([position.x, position.z], position.y)
                .to_array()
        })
        .to_vec();
    let normal = embedding
        .lift_direction([toward_viewer.x, toward_viewer.z], 0.0)
        .normalize_or_zero()
        .to_array();
    let (left_u, right_u) = if mirrored { (1.0, 0.0) } else { (0.0, 1.0) };
    Mesh::uniform_normal(positions, normal)
        .with_texture_coordinates(vec![
            [left_u, 0.0],
            [left_u, 1.0],
            [right_u, 1.0],
            [left_u, 0.0],
            [right_u, 1.0],
            [right_u, 0.0],
        ])
        .map_err(io::Error::other)
        .map_err(Into::into)
}

#[derive(Clone, Debug)]
struct DoomSkyBoundaryDepthDraw {
    source_linedef: doom_map_provider::DoomSourceRecord,
    source_sidedef: doom_map_provider::DoomSourceRecord,
    source_sector: doom_map_provider::DoomSourceRecord,
    mesh: Mesh,
}

fn diagnostic_skywall_mesh(positions: Vec<[f32; 3]>) -> PlatformResult<Mesh> {
    let mut horizontal_axis = None;
    'vertices: for (index, start) in positions.iter().enumerate() {
        for end in positions.iter().skip(index + 1) {
            let mut dx = end[0] - start[0];
            let mut dz = end[2] - start[2];
            let length = (dx * dx + dz * dz).sqrt();
            if length <= f32::EPSILON {
                continue;
            }
            dx /= length;
            dz /= length;
            if dx < 0.0 || (dx.abs() <= f32::EPSILON && dz < 0.0) {
                dx = -dx;
                dz = -dz;
            }
            horizontal_axis = Some([dx, dz]);
            break 'vertices;
        }
    }
    let [dx, dz] = horizontal_axis.unwrap_or([1.0, 0.0]);
    let texture_coordinates = positions
        .iter()
        .map(|position| {
            [
                (position[0] * dx + position[2] * dz) / 64.0,
                -position[1] / 64.0,
            ]
        })
        .collect();
    Mesh::uniform_normal(positions, [0.0, 1.0, 0.0])
        .with_texture_coordinates(texture_coordinates)
        .map_err(io::Error::other)
        .map_err(Into::into)
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
    startup::run()
}

fn arguments_for_rotated_map(arguments: &[String], map_name: &str) -> Vec<String> {
    let mut replaced = false;
    let mut rotated = arguments
        .iter()
        .map(|argument| {
            if argument.starts_with("--map=") {
                replaced = true;
                format!("--map={map_name}")
            } else {
                argument.clone()
            }
        })
        .collect::<Vec<_>>();
    if !replaced {
        rotated.push(format!("--map={map_name}"));
    }
    rotated
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

fn is_source_one_sided_wall(draw: &StaticDrawPlanEntry, map: &DoomMapCore) -> bool {
    let StaticDrawSource::Wall { source_linedef, .. } = draw.source else {
        return false;
    };
    map.linedefs
        .get(source_linedef.record_index as usize)
        .filter(|linedef| linedef.source == source_linedef)
        .is_some_and(|linedef| linedef.right_sidedef.is_some() ^ linedef.left_sidedef.is_some())
}

/// Classic-compatible placement lookup for map-authored Things only. The
/// general collision/topology locator intentionally rejects points on a BSP
/// partition; `R_PointOnSide` instead chooses the left child on equality.
fn locate_doom_thing_subsector(
    map: &DoomMapCore,
    point: [i16; 2],
) -> PlatformResult<doom_map_provider::DoomSourceRecord> {
    let mut child = DoomBspChild::Node(
        u16::try_from(map.nodes.len().saturating_sub(1))
            .map_err(|_| io::Error::other("Doom node root does not fit u16"))?,
    );
    for _ in 0..=map.nodes.len() {
        match child {
            DoomBspChild::Subsector(index) => {
                return map
                    .subsectors
                    .get(usize::from(index))
                    .map(|subsector| subsector.source)
                    .ok_or_else(|| io::Error::other("Thing BSP descent reached missing subsector"))
                    .map_err(Into::into);
            }
            DoomBspChild::Node(index) => {
                let node = map
                    .nodes
                    .get(usize::from(index))
                    .ok_or_else(|| io::Error::other("Thing BSP descent reached missing node"))?;
                let side = i64::from(node.delta_x) * i64::from(point[1] - node.y)
                    - i64::from(node.delta_y) * i64::from(point[0] - node.x);
                child = if side < 0 {
                    node.right_child
                } else {
                    node.left_child
                };
            }
        }
    }
    Err(io::Error::other("Thing BSP descent encountered a cycle").into())
}

fn prepare_scene(
    package: &str,
    member: &str,
    map_name: &str,
    audit_bsp_bounds: bool,
    source_boundary_trim: bool,
    sector_boundary_trim: bool,
    prepare_audio: bool,
) -> PlatformResult<SceneInput> {
    let mut import_warnings = Vec::new();
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
    let wad_parse_started = Instant::now();
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
    let wad_parse_us = wad_parse_started.elapsed().as_micros();
    let audio_assets = if prepare_audio {
        match prepare_doom_live_audio_assets(&read.bytes, &read.observation.wad, map_name) {
            Ok(assets) => Some(assets),
            Err(error) => {
                let warning =
                    format!("live-audio-preparation-unavailable:{error}; gameplay-continues=true");
                eprintln!("{map_name} {warning}");
                import_warnings.push(warning);
                None
            }
        }
    } else {
        None
    };
    let map_decode_started = Instant::now();
    let selection = select_doom_episode_map(&read.observation.wad, map_name)?;
    let map = decode_doom_map_core(&read.bytes, &selection, MAP_LIMITS)?;
    let map_decode_us = map_decode_started.elapsed().as_micros();
    let lowering_started = Instant::now();
    let unsupported_linedefs = map
        .linedefs
        .iter()
        .filter(|linedef| !matches!(linedef.special, 0 | 1 | 11 | 36 | 48 | 88))
        .map(|linedef| (linedef.source.record_index, linedef.special))
        .collect::<Vec<_>>();
    if let Some(warning) = bounded_source_warning(
        "unsupported-linedef-specials",
        "special",
        &unsupported_linedefs,
    ) {
        import_warnings.push(warning);
    }
    let unsupported_things = map
        .things
        .iter()
        .filter(|thing| hello_doom_e1m1::things::classify_e1m1_thing_kind(thing.kind).is_none())
        .map(|thing| (thing.source.record_index, thing.kind))
        .collect::<Vec<_>>();
    if let Some(warning) =
        bounded_source_warning("unsupported-thing-kinds", "kind", &unsupported_things)
    {
        import_warnings.push(warning);
    }
    let sprite_frames = decode_doom_sprite_frame_rotations(&read.observation.wad)?;
    let available_maps = read
        .observation
        .wad
        .lumps
        .iter()
        .map(|lump| lump.name.as_str())
        .filter(|name| matches!(name.as_bytes(), [b'E', b'1'..=b'9', b'M', b'1'..=b'9']))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let doom_sky_boundary_draws = lower_doom_paired_sky_boundary_triangles(&map)?
        .into_iter()
        .map(|triangle| {
            Ok(DoomSkyBoundaryDepthDraw {
                source_linedef: triangle.source_linedef,
                source_sidedef: triangle.source_sidedef,
                source_sector: triangle.source_sector,
                mesh: diagnostic_skywall_mesh(
                    triangle
                        .positions
                        .into_iter()
                        .map(|position| position.map(|component| component as f32))
                        .collect(),
                )?,
            })
        })
        .collect::<PlatformResult<Vec<_>>>()?;
    let walk_collision = DoomWalkCollisionWorld::from_map(&map);
    let walk_floors = DoomWalkFloorWorld::from_map(&map)?;
    let monster_sight_world = hello_doom_e1m1::perception::DoomMonsterSightWorld::from_map(&map);
    let actor_movement_world = hello_doom_e1m1::collision::DoomActorMovementWorld::from_map(&map)?;
    let start = resolve_doom_player_one_start(&map.things)?;
    let paths = resolve_doom_subsector_bsp_paths(&map)?;
    let location = locate_doom_point_subsector(start.position, &paths)?;
    let ownership = resolve_doom_subsector_sector_ownership(&map)?;
    let mut thing_sprites = Vec::new();
    for thing in &map.things {
        let Some(classification) = hello_doom_e1m1::things::classify_e1m1_thing_kind(thing.kind)
        else {
            continue;
        };
        let (Some(sprite), Some(frame)) =
            (classification.initial_sprite, classification.initial_frame)
        else {
            continue;
        };
        let source_subsector = match locate_doom_point_subsector([thing.x, thing.y], &paths) {
            Ok(location) => location.source_subsector,
            Err(_) => locate_doom_thing_subsector(&map, [thing.x, thing.y])?,
        };
        let owner = ownership
            .iter()
            .find(|entry| entry.source_subsector == source_subsector)
            .ok_or_else(|| {
                io::Error::other(format!(
                    "Thing {} has no source-sector ownership",
                    thing.source.record_index
                ))
            })?;
        let floor_height = map.sectors[usize::from(owner.sector_index)].floor_height;
        thing_sprites.push(DoomThingSprite {
            source: thing.source,
            kind: thing.kind,
            source_position: [thing.x, thing.y],
            source_angle: thing.angle,
            floor_height,
            source_sector: owner.source_sector.record_index,
            sprite,
            initial_frame: frame,
        });
    }
    let required_sprite_lumps = thing_sprites
        .iter()
        .flat_map(|thing| {
            let program = hello_doom_e1m1::things::e1m1_thing_state_program(thing.kind);
            let required_frames = program.required_frames(thing.initial_frame);
            sprite_frames.iter().filter(move |candidate| {
                candidate.sprite.eq_ignore_ascii_case(thing.sprite)
                    && required_frames.contains(&candidate.frame)
            })
        })
        .map(|candidate| candidate.source_lump_index)
        .collect::<BTreeSet<_>>();
    let raster_globals =
        decode_doom_raster_globals(&read.bytes, &read.observation.wad, RASTER_LIMITS)?;
    let mut sprite_uploads = Vec::with_capacity(required_sprite_lumps.len());
    for (upload_index, source_lump_index) in required_sprite_lumps.into_iter().enumerate() {
        let source_lump = read
            .observation
            .wad
            .lumps
            .get(source_lump_index as usize)
            .ok_or_else(|| io::Error::other("sprite frame refers to missing source lump"))?;
        let patch = decode_doom_sprite_patch(
            &read.bytes,
            &read.observation.wad,
            &source_lump.name,
            PATCH_LIMITS,
        )?;
        if patch.source_lump_index != source_lump_index {
            return Err(io::Error::other(format!(
                "sprite lump name {} resolves to {}, expected source lump {}",
                source_lump.name, patch.source_lump_index, source_lump_index
            ))
            .into());
        }
        let indexed = indexed_image_from_doom_patch(&patch);
        let lowered = lower_doom_indexed_image(&indexed, &raster_globals.palettes[0])?;
        let texture = TextureHandle(DOOM_THING_SPRITE_TEXTURE_BASE + upload_index as u64);
        let material = MaterialHandle(DOOM_THING_SPRITE_MATERIAL_BASE + upload_index as u64);
        let descriptor = Rgba8TextureDescriptor::new(
            u32::from(patch.width),
            u32::from(patch.height),
            Rgba8TextureColorSpace::Srgb,
        );
        sprite_uploads.push(DoomSpriteTextureUpload {
            source_lump_index,
            source_name: source_lump.name.clone(),
            width: patch.width,
            height: patch.height,
            left_offset: patch.left_offset,
            top_offset: patch.top_offset,
            texture,
            material,
            descriptor,
            rgba8: lowered.pixels,
            material_value: Material::new(
                format!("doom-thing-sprite:{}", source_lump.name),
                Color::rgb(1.0, 1.0, 1.0),
            )
            .with_texture(texture)
            .with_texture_sampler(TextureSampler {
                filter: TextureFilter::Point,
                address_u: TextureAddressMode::Clamp,
                address_v: TextureAddressMode::Clamp,
            }),
        });
    }
    let regions = resolve_doom_subsector_regions(&map, &paths)?;
    let inferred_region_bounds = regions
        .iter()
        .map(|region| {
            region.vertices.iter().fold(None, |bounds, [x, y]| {
                Some(bounds.map_or([*x, *y, *x, *y], |current: [f64; 4]| {
                    [
                        current[0].min(*x),
                        current[1].min(*y),
                        current[2].max(*x),
                        current[3].max(*y),
                    ]
                }))
            })
        })
        .collect::<Vec<_>>();
    let bsp_bounds_audit = audit_bsp_bounds
        .then(|| audit_doom_bsp_bounds(&read.bytes, &selection, &map, &inferred_region_bounds))
        .transpose()?;
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
    let source_bounded_surface_bake = if source_boundary_trim {
        let result = if sector_boundary_trim {
            lower_doom_sector_bounded_subsector_surfaces(&map, &paths)
        } else {
            lower_doom_source_bounded_subsector_surfaces(&map, &paths)
        };
        match result {
            Ok(bake) => Some(bake),
            Err(error) => {
                let warning = format!(
                    "source-boundary-surface-trim-unavailable:{error}; fallback=finite-bsp-path-subsector-surfaces"
                );
                eprintln!("{} {warning}", map.map_name);
                import_warnings.push(warning);
                None
            }
        }
    } else {
        None
    };
    let sky_surfaces = source_bounded_surface_bake
        .as_ref()
        .map(|_| observe_doom_sky_surfaces(&map, &paths))
        .transpose()?;
    let flats = if let (Some(bake), Some(sky)) =
        (source_bounded_surface_bake.as_ref(), sky_surfaces.as_ref())
    {
        PreparedE1m1Flats {
            map_name: map.map_name.clone(),
            flat_assembly: assemble_static_opaque_flats(&bake.surfaces, sky, FlatExtent::E1M1)?,
        }
    } else {
        let surfaces = lower_doom_subsector_surfaces(&map, &paths)?;
        let sky = observe_doom_sky_surfaces(&map, &paths)?;
        PreparedE1m1Flats {
            map_name: map.map_name.clone(),
            flat_assembly: assemble_static_opaque_flats(&surfaces, &sky, FlatExtent::E1M1)?,
        }
    };
    let wall_extents =
        prepare_e1m1_wall_texture_extents(&read.bytes, &read.observation.wad, TEXTURE_LIMITS)?;
    let source_walls = lower_doom_textured_wall_triangles(&map, &wall_extents)?;
    let masked_middles = observe_doom_two_sided_middle_textures(&map)?;
    let walls = PreparedE1m1Walls {
        map_name: map.map_name.clone(),
        wall_assembly: assemble_static_opaque_walls(&source_walls, &masked_middles, &wall_extents)?,
    };
    let cutouts = PreparedE1m1MaskedMiddleCutouts {
        map_name: map.map_name.clone(),
        assembly: assemble_experimental_masked_middle_cutouts(
            &source_walls,
            &masked_middles,
            &wall_extents,
        )?,
    };
    let flat_textures = prepare_e1m1_flat_textures(
        &read.bytes,
        &read.observation.wad,
        &flats,
        RASTER_LIMITS,
        FLAT_LIMITS,
    )?;
    let mut names = hello_doom_e1m1::prepared_e1m1_wall_texture_names(&walls);
    names.extend(manual_door_dynamic_wall_texture_names(
        &map,
        &activation_source,
        &wall_extents,
    )?);
    names.extend(
        activation_source
            .linedefs
            .iter()
            .filter(|linedef| linedef.special == 11)
            .filter_map(|linedef| {
                resolve_doom_shareware_switch_texture(&activation_source, linedef.source)
                    .1
                    .map(|change| change.after_texture)
            }),
    );
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
    let diagnostic_sky_flats = if let (Some(bake), Some(sky)) =
        (source_bounded_surface_bake.as_ref(), sky_surfaces.as_ref())
    {
        let mut lowered = Vec::new();
        for surface in &bake.surfaces {
            if !sky.iter().any(|observation| {
                observation.source_subsector == surface.source_subsector
                    && observation.source_sector == surface.source_sector
                    && observation.plane == surface.plane
            }) {
                continue;
            }
            match lower_static_flat_triangle(surface, FlatExtent::E1M1) {
                Ok(flat) => lowered.push(flat),
                Err(StaticFlatLoweringError::DegenerateTriangle) => {}
                Err(error) => return Err(error.into()),
            }
        }
        lowered
    } else {
        let surfaces = lower_doom_subsector_surfaces(&map, &paths)?;
        let sky = observe_doom_sky_surfaces(&map, &paths)?;
        let mut lowered = Vec::new();
        for surface in &surfaces {
            if !sky.iter().any(|observation| {
                observation.source_subsector == surface.source_subsector
                    && observation.source_sector == surface.source_sector
                    && observation.plane == surface.plane
            }) {
                continue;
            }
            match lower_static_flat_triangle(surface, FlatExtent::E1M1) {
                Ok(flat) => lowered.push(flat),
                Err(StaticFlatLoweringError::DegenerateTriangle) => {}
                Err(error) => return Err(error.into()),
            }
        }
        lowered
    };
    let diagnostic_sky_draws = diagnostic_sky_flats
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
    let preparation_timings = DoomPreparationTimings {
        wad_parse_us,
        map_decode_us,
        lowering_us: lowering_started.elapsed().as_micros(),
    };
    Ok(SceneInput {
        map_name: map.map_name.clone(),
        import_warnings,
        preparation_timings,
        audio_assets,
        available_maps,
        things: map.things.clone(),
        sprite_frames,
        thing_sprites,
        sprite_uploads,
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
        monster_sight_world,
        actor_movement_world,
        reject_report,
        topology_report,
        bsp_bounds_audit,
        source_bounded_surface_audit: source_bounded_surface_bake.map(|bake| bake.audit),
        membership_selection,
        activation_source,
        door_geometry_source: DoomDynamicDoorGeometrySource {
            map,
            wall_extents,
            wall_materials,
        },
    })
}

fn bounded_source_warning(
    family: &str,
    value_label: &str,
    records: &[(u32, u16)],
) -> Option<String> {
    (!records.is_empty()).then(|| {
        let samples = records
            .iter()
            .take(8)
            .map(|(record, value)| format!("{record}:{value_label}{value}"))
            .collect::<Vec<_>>()
            .join("|");
        format!("{family}:count={}:samples={samples}", records.len())
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
    reembed_draws_for_comparison(&mut scene.opaque_draws, embedding);
    reembed_draws_for_comparison(&mut scene.cutout_draws, embedding);
    reembed_draws_for_comparison(&mut scene.diagnostic_sky_draws, embedding);
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

/// One frame's Doom-private Candidate 1 realization.
///
/// The renderer batch contains no persistent mesh identity. The remaining
/// fields are retained only so E1M1 can prove conservation at the corpus
/// boundary before the batch is submitted.
struct Candidate1SkyDepthBatch {
    batch: ExperimentalSubmissionLocalGeometry,
    source_regions: usize,
    declarations: usize,
    vertices: usize,
    triangles: usize,
    structural_fingerprint: String,
}

fn reembed_draws_for_comparison(
    draws: &mut [StaticDrawPlanEntry],
    embedding: DoomComparativeEmbedding,
) {
    if embedding == DoomComparativeEmbedding::CurrentReflected {
        return;
    }
    for draw in draws {
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
