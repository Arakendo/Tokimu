//! Headless source-geometry observations for classic Doom maps.
//!
//! This provider resolves map-table references only. Mesh construction,
//! materials, renderer resources, and WAD acquisition remain outside it.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use doom_map_provider::{DoomBspChild, DoomMapCore, DoomSector, DoomSourceRecord};
use thiserror::Error;

/// One original sidedef and its owning sector, retained before wall semantics
/// such as texture selection or pegging are admitted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomWallSide {
    pub source_sidedef: DoomSourceRecord,
    pub source_sector: DoomSourceRecord,
    pub sector_index: u16,
    pub x_offset: i16,
    pub y_offset: i16,
    pub upper_texture: String,
    pub lower_texture: String,
    pub middle_texture: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomWallCandidate {
    pub source_linedef: DoomSourceRecord,
    pub linedef_flags: u16,
    pub start: [i16; 2],
    pub end: [i16; 2],
    pub right: Option<DoomWallSide>,
    pub left: Option<DoomWallSide>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomWallTopologyAudit {
    pub candidates: usize,
    pub one_sided: usize,
    pub two_sided: usize,
    pub same_sector_two_sided: usize,
}

/// One temporary sector-height observation projected over immutable decoded
/// Doom source. It is a corpus/provider input to later geometry or visibility
/// preparation, not a moving-sector state machine and not a renderer command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomSectorRuntimeHeightSnapshot {
    pub source_sector: DoomSourceRecord,
    pub floor_height: Option<i16>,
    pub ceiling_height: Option<i16>,
}

/// Applies bounded runtime height facts to a clone of decoded source records.
/// An unavailable source identity is rejected explicitly: treating it as a
/// no-op would make stale application runtime state indistinguishable from an
/// intentional static source view.
pub fn project_doom_sector_runtime_heights(
    map: &DoomMapCore,
    snapshots: &[DoomSectorRuntimeHeightSnapshot],
) -> Result<DoomMapCore, DoomGeometryError> {
    let mut projected = map.clone();
    for snapshot in snapshots {
        let sector = projected
            .sectors
            .iter_mut()
            .find(|sector| sector.source == snapshot.source_sector)
            .ok_or(DoomGeometryError::RuntimeSnapshotSectorUnavailable {
                source_sector: snapshot.source_sector,
            })?;
        if let Some(floor_height) = snapshot.floor_height {
            sector.floor_height = floor_height;
        }
        if let Some(ceiling_height) = snapshot.ceiling_height {
            sector.ceiling_height = ceiling_height;
        }
    }
    Ok(projected)
}

/// Raw vertical-clearance evidence retained before wall visibility, middle
/// texture clipping, or player movement policy is admitted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomVerticalTopologyAudit {
    pub sectors: usize,
    pub sectors_without_positive_clearance: usize,
    pub two_sided_openings: usize,
    pub two_sided_openings_without_positive_clearance: usize,
}

/// A closed subsector boundary reconstructed deterministically from `SEGS`.
/// This is topology evidence, not a triangulated floor or ceiling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomSubsectorLoop {
    pub source_subsector: DoomSourceRecord,
    pub source_segs: Vec<DoomSubsectorBoundaryEdge>,
    pub vertices: Vec<[i16; 2]>,
}

/// Counts the results of attempting a strict `SEGS`-only boundary recovery.
/// Failures are retained rather than hidden behind the first rejected leaf.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomSubsectorLoopClosureAudit {
    pub subsectors: usize,
    pub closed_loops: usize,
    pub rejected: Vec<DoomGeometryError>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomSubsectorBoundaryEdge {
    pub source_seg: DoomSourceRecord,
    pub reversed_from_source: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoomBspSide {
    Right,
    Left,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomBspPathStep {
    pub source_node: DoomSourceRecord,
    pub side: DoomBspSide,
    pub origin: [i16; 2],
    pub delta: [i16; 2],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomSubsectorBspPath {
    pub source_subsector: DoomSourceRecord,
    pub steps: Vec<DoomBspPathStep>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomBspPathAudit {
    pub subsectors: usize,
    pub minimum_depth: usize,
    pub maximum_depth: usize,
}

/// A bounded convex region inferred from a subsector's BSP partition path.
/// It is an intermediate topology observation, not floor or ceiling geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct DoomSubsectorRegion {
    pub source_subsector: DoomSourceRecord,
    pub vertices: Vec<[f64; 2]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DoomSubsectorRegionAudit {
    pub regions: usize,
    pub seg_endpoints: usize,
    pub endpoints_outside_paths: usize,
    pub maximum_outside_distance: f64,
}

/// The sector implied by every source seg surrounding a subsector.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomSubsectorSectorOwnership {
    pub source_subsector: DoomSourceRecord,
    pub source_sector: DoomSourceRecord,
    pub sector_index: u16,
}

/// The source BSP leaves that contain one original linedef through their
/// `SEGS`. Membership is intentionally one-to-many: a partitioned linedef can
/// occur in multiple subsectors. This is Doom-topology evidence, not renderer
/// scene membership or a visibility decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomLinedefSubsectorMembership {
    pub source_linedef: DoomSourceRecord,
    pub source_subsectors: Vec<DoomSourceRecord>,
}

/// The uniquely located source BSP leaf for an integer map point.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomPointSubsectorObservation {
    pub point: [i16; 2],
    pub source_subsector: DoomSourceRecord,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoomSurfacePlane {
    Floor,
    Ceiling,
}

/// Explicit corpus-provider lift from classic Doom's map plane plus height to
/// the current Tokimu-facing 3D embedding. This is Doom source conversion,
/// not a renderer or global-world convention.
pub fn doom_point_to_tokimu(source_xy: [f64; 2], height: f64) -> [f64; 3] {
    [source_xy[0], height, source_xy[1]]
}

/// Exact inverse of [`doom_point_to_tokimu`] for retained round-trip evidence.
pub fn tokimu_point_to_doom(world: [f64; 3]) -> ([f64; 2], f64) {
    ([world[0], world[2]], world[1])
}

/// Lifts a Doom planar direction plus vertical component without applying
/// translation or normalization.
pub fn doom_direction_to_tokimu(source_xy: [f64; 2], vertical: f64) -> [f64; 3] {
    [source_xy[0], vertical, source_xy[1]]
}

/// Exact inverse of [`doom_direction_to_tokimu`].
pub fn tokimu_direction_to_doom(world: [f64; 3]) -> ([f64; 2], f64) {
    ([world[0], world[2]], world[1])
}

/// One renderer-neutral triangle from a bounded BSP leaf surface.
#[derive(Clone, Debug, PartialEq)]
pub struct DoomSurfaceTriangle {
    pub source_subsector: DoomSourceRecord,
    pub source_sector: DoomSourceRecord,
    pub plane: DoomSurfacePlane,
    pub texture_name: String,
    /// Doom map X/Z with the owning sector floor or ceiling as Y.
    pub positions: [[f64; 3]; 3],
}

/// Audit for the Doom-private source-boundary-refined plane bake. Local SEG
/// evidence first narrows each BSP leaf. A balanced directed boundary graph
/// reconstructed from the owning sector's LINEDEF/SIDEDEF topology can then
/// trim non-convex shells and holes which one leaf's local SEGs do not fully
/// describe. Unavailable sector topology fails open to the local result. A
/// leaf for which every candidate region has zero area emits no plane
/// triangles and is retained explicitly in the audit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomSourceBoundedSurfaceAudit {
    pub subsectors: usize,
    pub stitched_seg_loops: usize,
    pub stitched_loop_refinements: usize,
    pub seg_half_plane_regions: usize,
    pub seg_half_plane_refinements: usize,
    pub bsp_path_fallbacks: usize,
    pub bsp_path_fallback_subsectors: Vec<DoomSourceRecord>,
    pub sector_boundary_supported_subsectors: usize,
    pub sector_boundary_refinements: usize,
    pub sector_boundary_fragments: usize,
    pub sector_boundary_omissions: usize,
    pub sector_boundary_omission_subsectors: Vec<DoomSourceRecord>,
    pub sector_boundary_unavailable_subsectors: Vec<DoomSourceRecord>,
    pub degenerate_region_omissions: usize,
    pub degenerate_region_subsectors: Vec<DoomSourceRecord>,
    pub surface_triangles: usize,
}

/// Renderer-neutral plane triangles plus evidence for the boundary source
/// selected for each subsector.
#[derive(Clone, Debug, PartialEq)]
pub struct DoomSourceBoundedSurfaceBake {
    pub surfaces: Vec<DoomSurfaceTriangle>,
    pub audit: DoomSourceBoundedSurfaceAudit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoomWallSideKind {
    /// WAD sidedef slot 0: the original Doom linedef's front side.
    Right,
    /// WAD sidedef slot 1: the original Doom linedef's back side.
    Left,
}

/// One renderer-neutral triangle for the full height of a one-sided wall.
#[derive(Clone, Debug, PartialEq)]
pub struct DoomWallTriangle {
    pub source_linedef: DoomSourceRecord,
    pub source_sidedef: DoomSourceRecord,
    pub source_sector: DoomSourceRecord,
    pub side: DoomWallSideKind,
    /// Doom map X/Z with the owning sector floor or ceiling as Y.
    pub positions: [[f64; 3]; 3],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoomWallBand {
    Upper,
    Lower,
}

/// One renderer-neutral triangle for a height discontinuity at a two-sided wall.
#[derive(Clone, Debug, PartialEq)]
pub struct DoomTwoSidedWallTriangle {
    pub source_linedef: DoomSourceRecord,
    pub source_sidedef: DoomSourceRecord,
    pub source_sector: DoomSourceRecord,
    pub side: DoomWallSideKind,
    pub band: DoomWallBand,
    pub texture_name: String,
    /// Doom map X/Z with the band heights as Y.
    pub positions: [[f64; 3]; 3],
}

/// One renderer-neutral depth-coverage triangle for a height discontinuity
/// between adjacent classic-sky ceilings.
///
/// Doom does not present the corresponding upper wall texture, but the
/// viewer-relative sky aperture still prevents unrelated, farther map
/// geometry from showing through that span. This retained source identity is
/// deliberately separate from ordinary visible wall lowering.
#[derive(Clone, Debug, PartialEq)]
pub struct DoomPairedSkyBoundaryTriangle {
    pub source_linedef: DoomSourceRecord,
    pub source_sidedef: DoomSourceRecord,
    pub source_sector: DoomSourceRecord,
    pub side: DoomWallSideKind,
    /// Doom map X/Z with the adjacent ceiling heights as Y.
    pub positions: [[f64; 3]; 3],
}

/// An authored two-sided middle texture and the source opening it occupies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomMiddleTextureObservation {
    pub source_linedef: DoomSourceRecord,
    pub source_sidedef: DoomSourceRecord,
    pub source_sector: DoomSourceRecord,
    pub side: DoomWallSideKind,
    pub texture_name: String,
    pub opening_floor: i16,
    pub opening_ceiling: i16,
}

/// One clipped triangle for an authored two-sided middle texture. The initial
/// policy emits only the positive shared vertical opening and does not choose
/// alpha testing, blending, or portal behavior.
#[derive(Clone, Debug, PartialEq)]
pub struct DoomTwoSidedMiddleWallTriangle {
    pub source_linedef: DoomSourceRecord,
    pub source_sidedef: DoomSourceRecord,
    pub source_sector: DoomSourceRecord,
    pub side: DoomWallSideKind,
    pub texture_name: String,
    pub opening_floor: i16,
    pub opening_ceiling: i16,
    pub positions: [[f64; 3]; 3],
}

/// A Doom-specific sky surface classification retained outside generic mesh data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomSkySurfaceObservation {
    pub source_subsector: DoomSourceRecord,
    pub source_sector: DoomSourceRecord,
    pub plane: DoomSurfacePlane,
    pub texture_name: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoomWallTextureRole {
    Upper,
    Lower,
    Middle,
}

/// The stable, side-local horizontal axis for one authored wall texture.
///
/// `u_start` and `u_end` correspond to the linedef's stored start and end
/// vertices after Doom's 2D coordinates are lifted into Tokimu's 3D frame.
/// That lift reverses the horizontal screen direction of a right/front
/// sidedef, so its texture axis decreases along the stored linedef; a
/// left/back sidedef advances along it. This preserves readable source art
/// without asking a generic mesh or renderer consumer to understand Doom
/// sidedefs.
#[derive(Clone, Debug, PartialEq)]
pub struct DoomWallTextureAxisObservation {
    pub source_linedef: DoomSourceRecord,
    pub linedef_flags: u16,
    pub source_sidedef: DoomSourceRecord,
    pub side: DoomWallSideKind,
    pub role: DoomWallTextureRole,
    pub texture_name: String,
    pub u_start: f64,
    pub u_end: f64,
    pub v_offset: i16,
}

/// Plain texture dimensions supplied across the raster-to-geometry boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomTextureExtent {
    pub name: String,
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DoomWallTextureBinding {
    pub axis: DoomWallTextureAxisObservation,
    pub texture_width: u16,
    pub texture_height: u16,
}

/// The classic source renderer's vertical texture anchor after the sidedef row
/// offset. It is renderer-neutral: a later lowering chooses normalized V
/// direction from this retained world-space anchor.
#[derive(Clone, Debug, PartialEq)]
pub struct DoomWallTexturePlacement {
    pub binding: DoomWallTextureBinding,
    pub source_sector: DoomSourceRecord,
    pub texture_mid_y: i32,
}

/// One ordinary UV-bearing wall triangle, still free of material and renderer
/// objects. Middle-texture geometry is intentionally absent.
#[derive(Clone, Debug, PartialEq)]
pub struct DoomTexturedWallTriangle {
    pub source_linedef: DoomSourceRecord,
    pub source_sidedef: DoomSourceRecord,
    pub source_sector: DoomSourceRecord,
    pub side: DoomWallSideKind,
    pub role: DoomWallTextureRole,
    pub texture_name: String,
    pub positions: [[f64; 3]; 3],
    pub texture_coordinates: [[f64; 2]; 3],
}

/// A corpus-only textured wall triangle clipped to one source `SEG`.
///
/// This intentionally retains both the original wall identity and the BSP
/// fragment that produced the candidate. It is not a renderer fragment type:
/// callers may compare it with whole-linedef lowering before deciding whether
/// viewer-relative Doom presentation is worth pursuing.
#[derive(Clone, Debug, PartialEq)]
pub struct DoomSegTexturedWallTriangle {
    pub source_seg: DoomSourceRecord,
    pub source_linedef: DoomSourceRecord,
    pub source_sidedef: DoomSourceRecord,
    pub source_sector: DoomSourceRecord,
    pub side: DoomWallSideKind,
    pub role: DoomWallTextureRole,
    pub texture_name: String,
    pub positions: [[f64; 3]; 3],
    pub texture_coordinates: [[f64; 2]; 3],
}

/// Doom-source classification of whether a SEG's sector relationship can
/// close a classic screen-span interval. This is deliberately distinct from
/// screen projection and interval bookkeeping: the classification is owned by
/// the Doom provider, not generic visibility code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoomSegOccluderKind {
    OneSided,
    BackSectorClosed,
    OpeningClosed,
    Open,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomSegOccluderObservation {
    pub source_seg: DoomSourceRecord,
    pub source_linedef: DoomSourceRecord,
    pub side: DoomWallSideKind,
    pub kind: DoomSegOccluderKind,
}

/// Source-local plane-mark facts for one directed SEG at a declared viewer
/// height. This mirrors only the `R_StoreWallRange` decision to mark an
/// adjacent floor or ceiling plane; it does not project columns, form a visplane,
/// clip a span, or select a renderer draw.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomSegPlaneMarkObservation {
    pub source_seg: DoomSourceRecord,
    pub source_linedef: DoomSourceRecord,
    pub side: DoomWallSideKind,
    pub front_sector: DoomSourceRecord,
    pub back_sector: Option<DoomSourceRecord>,
    pub floor_marked: bool,
    pub ceiling_marked: bool,
    pub paired_sky_ceiling_adjustment: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomPeggingFlagAudit {
    pub upper_axes: usize,
    pub lower_axes: usize,
    pub upper_unpegged: usize,
    pub lower_unpegged: usize,
}

struct DoomWallBandRequest<'a> {
    side: DoomWallSideKind,
    band: DoomWallBand,
    bottom: i16,
    top: i16,
    texture_name: &'a str,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum DoomGeometryError {
    #[error("runtime height snapshot refers to unavailable sector record {source_sector:?}")]
    RuntimeSnapshotSectorUnavailable { source_sector: DoomSourceRecord },
    #[error("linedef {linedef_index} has neither a right nor left sidedef")]
    MissingBothSidedefs { linedef_index: u32 },
    #[error("linedef {linedef_index} has identical start and end vertices")]
    DegenerateLinedef { linedef_index: u32 },
    #[error("subsector {subsector_index} has only {seg_count} boundary segs")]
    SubsectorTooSmall {
        subsector_index: u32,
        seg_count: u16,
    },
    #[error(
        "subsector {subsector_index} boundary is not continuous between seg records {previous_seg_index} and {next_seg_index}"
    )]
    SubsectorBoundaryOpen {
        subsector_index: u32,
        previous_seg_index: u32,
        next_seg_index: u32,
    },
    #[error(
        "subsector {subsector_index} has multiple next boundary segs after seg record {previous_seg_index}"
    )]
    SubsectorBoundaryAmbiguous {
        subsector_index: u32,
        previous_seg_index: u32,
    },
    #[error("subsector {subsector_index} contains zero-length seg record {seg_index}")]
    DegenerateSeg {
        subsector_index: u32,
        seg_index: u32,
    },
    #[error("map has {subsectors} subsectors but no BSP root node")]
    MissingBspRoot { subsectors: usize },
    #[error("BSP traversal reaches subsector index {subsector_index}, but only {available} exist")]
    BspSubsectorOutOfBounds {
        subsector_index: u16,
        available: usize,
    },
    #[error("BSP traversal reaches node index {node_index}, but only {available} exist")]
    BspNodeOutOfBounds { node_index: u16, available: usize },
    #[error("BSP traversal revisits node index {node_index}")]
    BspCycle { node_index: u16 },
    #[error("map has no vertices from which to establish a bounded BSP clip region")]
    MissingMapBounds,
    #[error("BSP path for subsector {subsector_index} clips to an empty region")]
    EmptySubsectorRegion { subsector_index: u32 },
    #[error("subsector {subsector_index} seg record {seg_index} has no owning sidedef")]
    SubsectorSegMissingOwningSide {
        subsector_index: u32,
        seg_index: u32,
    },
    #[error(
        "subsector {subsector_index} mixes sector {first_sector} with sector {observed_sector} at seg record {seg_index}"
    )]
    SubsectorMixedSectors {
        subsector_index: u32,
        first_sector: u16,
        observed_sector: u16,
        seg_index: u32,
    },
    #[error("texture extent `{name}` is duplicated")]
    DuplicateTextureExtent { name: String },
    #[error("wall texture `{name}` has no supplied extent at linedef record {linedef_index}")]
    MissingTextureExtent { name: String, linedef_index: u32 },
    #[error("subsector {subsector_index} candidate region has zero area")]
    DegenerateSubsectorRegion { subsector_index: u32 },
    #[error("point ({x}, {y}) lies on or outside the retained BSP partition boundary")]
    PointNotInsideUniqueSubsector { x: i16, y: i16 },
    #[error("seg record {seg_index} has unsupported sidedef direction {direction}")]
    UnsupportedSegDirection { seg_index: u32, direction: u16 },
    #[error("linedef clipping interval is not finite or is outside 0 through 1")]
    InvalidLinedefInterval,
}

/// Bounded source-only result of a near-first Doom BSP traversal. It records
/// Doom SEG admission and horizontal solid-range coverage before mesh or
/// renderer work begins. It is corpus/provider evidence, not a generic
/// visibility result or renderer scheduling policy.
#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct DoomClassicBspObservation {
    pub leaves_visited: usize,
    pub visited_subsectors: BTreeSet<u16>,
    pub source_segs_visited: usize,
    pub far_children_pruned: usize,
    pub far_children_outside_fov: usize,
    pub far_children_fail_open: usize,
    pub backface_rejected: usize,
    pub edge_on: usize,
    pub outside_fov_rejected: usize,
    pub near_plane_fail_open: usize,
    pub solid_admitted: usize,
    pub pass_admitted: usize,
    pub solid_range_contributors: usize,
    pub solid_range_fully_covered: usize,
    pub solid_range_covered_columns: usize,
    pub admitted_seg_records: BTreeSet<u32>,
    /// Source protocol order matters to later Doom-local wall-tier and plane
    /// observations. A set alone cannot retain the preceding authority.
    pub admitted_seg_order: Vec<u32>,
    /// E1M1's retained hut control uses linedef 247 as a corpus-local watched
    /// identity. General fixtures use `watched_subsector_elisions` instead.
    pub hut_linedef_segs_visited: usize,
    pub hut_linedef_segs_admitted: usize,
    pub watched_subsector_elisions: Vec<String>,
    /// Structured target-specific elisions with provenance for the nearer
    /// solid source events that established the covering range. This is
    /// diagnostic history, not historical Doom storage or renderer policy.
    pub watched_elision_provenance: Vec<DoomClassicWatchedElisionProvenance>,
    /// Ordered solid-range mutations retained for causal replay. The ordinary
    /// `solidsegs`-equivalent union remains the decision input.
    pub solid_range_events: Vec<DoomClassicSolidRangeEvent>,
    /// Exact solid mutations suppressed by a diagnostic counterfactual. An
    /// ordinary replay leaves this empty.
    pub suppressed_solid_range_mutations: Vec<DoomClassicSuppressedSolidRangeMutation>,
    pub samples: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomClassicWatchedElisionProvenance {
    pub event_ordinal: usize,
    pub node: u16,
    pub reason: String,
    pub subsectors: Vec<u16>,
    pub projected_interval: Option<[usize; 2]>,
    pub covering_range: Option<[usize; 2]>,
    pub covering_source_segs: BTreeSet<u32>,
    pub covering_source_linedefs: BTreeSet<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomClassicSolidRangeEvent {
    pub event_ordinal: usize,
    pub source_seg: u32,
    pub source_linedef: u32,
    pub input_interval: [usize; 2],
    pub fully_covered_before: bool,
    pub merged_range: [usize; 2],
    pub contributing_source_segs: BTreeSet<u32>,
    pub contributing_source_linedefs: BTreeSet<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomClassicSuppressedSolidRangeMutation {
    pub source_seg: u32,
    pub source_linedef: u32,
    pub input_interval: [usize; 2],
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DoomClassicSolidRangeProvenance {
    interval: [usize; 2],
    source_segs: BTreeSet<u32>,
    source_linedefs: BTreeSet<u32>,
}

#[derive(Default)]
struct DoomClassicSolidRangeState {
    intervals: Vec<[usize; 2]>,
    provenance: Vec<DoomClassicSolidRangeProvenance>,
    suppressed_source_seg: Option<u32>,
}

/// Bounded Doom-source observation of per-column vertical clip state after
/// classic BSP admission. This is corpus/provider evidence only: it forms no
/// renderer scissor, visplane, mesh, or public visibility contract.
#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct DoomSegClassicVerticalClipObservation {
    pub admitted_segs: usize,
    pub upper_tier_spans: usize,
    pub lower_tier_spans: usize,
    pub middle_tier_spans: usize,
    pub floor_plane_marks: usize,
    pub ceiling_plane_marks: usize,
    pub paired_sky_adjustments: usize,
    pub ceiling_clip_updates: usize,
    pub floor_clip_updates: usize,
    /// Ordered, source-labelled mutations of the bounded diagnostic columns.
    /// These facts expose the Doom provider's preparation protocol for corpus
    /// review; they are not renderer commands or a public span API.
    pub ordered_coverage_transitions: Vec<DoomOrderedCoverageTransition>,
    /// Cases where the provider could not prove a coverage mutation. The
    /// corresponding diagnostic column remains unchanged.
    pub ordered_coverage_fail_open: Vec<DoomOrderedCoverageFailOpen>,
    /// Source wall-tier intervals after applying the coverage established by
    /// earlier near-to-far contributions. A missing retained interval means
    /// the tier was source-valid but could not re-enter the currently open
    /// vertical range. These are diagnostic cells, not renderer fragments.
    pub ordered_wall_intervals: Vec<DoomOrderedWallInterval>,
    /// Bounded final per-column clip facts with the source SEG tiers that
    /// contributed to them. These are Doom-provider diagnostic cells, not
    /// renderer pixels, a scissor contract, or an admission of a visplane API.
    pub column_traces: Vec<DoomSegClassicVerticalColumnTrace>,
    pub plane_spans: DoomSegClassicPlaneSpanObservation,
    pub samples: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoomOrderedCoverageTransitionReason {
    CeilingPlaneMarked,
    FloorPlaneMarked,
    PairedSkyBoundaryRetained,
    UpperTierRaised,
    LowerTierLowered,
    OneSidedMiddleClosed,
    CeilingMarkClosed,
    FloorMarkClosed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomOrderedCoverageTransition {
    pub source_seg: u32,
    pub source_linedef: u32,
    pub front_sector: u32,
    pub back_sector: Option<u32>,
    pub column: usize,
    pub upper_before: usize,
    pub lower_before: usize,
    pub upper_after: usize,
    pub lower_after: usize,
    pub retained_plane_interval: Option<[usize; 2]>,
    pub reason: DoomOrderedCoverageTransitionReason,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoomOrderedCoverageFailOpenReason {
    MissingPlaneMark,
    MissingSourceSeg,
    ProjectionBehindViewer,
    ProjectionOutsideHorizontalFov,
    RaySegmentDepthUnresolved,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomOrderedCoverageFailOpen {
    pub source_seg: u32,
    pub source_linedef: Option<u32>,
    pub column: Option<usize>,
    pub reason: DoomOrderedCoverageFailOpenReason,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomOrderedWallInterval {
    pub source_seg: u32,
    pub source_linedef: u32,
    pub column: usize,
    pub role: DoomWallTextureRole,
    pub raw_interval: [usize; 2],
    pub open_interval_before: Option<[usize; 2]>,
    pub retained_interval: Option<[usize; 2]>,
}

/// Bounded reconstruction of ordered wall cells as ordinary source-labelled
/// triangles for corpus presentation. The provider owns the Doom projection
/// and source interpolation; consumers still receive plain textured geometry.
///
/// This is deliberately a fixed 320x200 diagnostic reconstruction rather than
/// a public renderer span/scissor contract or a claim of historic pixel parity.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct DoomOrderedWallFragmentReconstruction {
    pub retained_cells: usize,
    pub reconstructed_triangles: Vec<DoomSegTexturedWallTriangle>,
    /// Retained source cells whose owning wall tier has no vertical extent.
    /// These are explicit presentation omissions, not unresolved failures.
    pub degenerate_cells: usize,
    pub unresolved_cells: usize,
    pub samples: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomSegClassicVerticalColumnTrace {
    pub column: usize,
    pub ceiling_clip: usize,
    pub floor_clip: usize,
    pub upper_source_segs: BTreeSet<u32>,
    pub lower_source_segs: BTreeSet<u32>,
    pub middle_source_segs: BTreeSet<u32>,
    /// Paired-sky source boundaries whose horizontal interval reaches this
    /// diagnostic cell. This records a Doom-specific potential depth boundary;
    /// it is deliberately not folded into upper-wall authority or generic
    /// occlusion state.
    pub paired_sky_boundary_source_segs: BTreeSet<u32>,
}

/// Bounded plane-span state keyed only by Doom source plane identity. The
/// intervals are diagnostic screen cells, not renderer pixels or flat draws.
#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct DoomSegClassicPlaneSpanObservation {
    pub keys: BTreeMap<DoomSegClassicPlaneKey, Vec<DoomSegClassicPlaneInstance>>,
    pub plane_instances: usize,
    pub collision_splits: usize,
    pub horizontal_spans: usize,
    pub populated_columns: usize,
    pub populated_cells: usize,
    pub overlapping_writes: usize,
    pub empty_after_clip: usize,
    pub samples: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomSegClassicPlaneInstance {
    pub columns: Vec<Option<[usize; 2]>>,
    pub column_sources: Vec<Option<[u32; 2]>>,
    pub minimum_column: usize,
    pub maximum_column: usize,
    pub source_sectors: BTreeSet<u32>,
    pub source_segs: BTreeSet<u32>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DoomSegClassicPlaneKind {
    Floor,
    Ceiling,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DoomSegClassicPlaneKey {
    pub kind: DoomSegClassicPlaneKind,
    pub height: i16,
    pub texture: String,
    pub light: i16,
}

/// Runs the bounded Doom near-first BSP/solid-range control shared by E1M1 and
/// synthetic corpus fixtures. It intentionally stops before plane spans,
/// texture/material selection, mesh lowering, or renderer submission.
pub fn observe_doom_classic_bsp(
    map: &DoomMapCore,
    viewer: [i16; 2],
    heading: f64,
    watched_subsectors: &BTreeSet<u16>,
) -> Result<DoomClassicBspObservation, DoomGeometryError> {
    observe_doom_classic_bsp_with_order(
        map,
        viewer,
        heading,
        watched_subsectors,
        DoomClassicBspTraversalOrder::NearFirst,
        DoomClassicBspReplayControl::ordinary(),
    )
}

/// Diagnostic control for the Doom corpus. It retains near-first BSP order and
/// source SEG admission, but does not let accumulated solid horizontal ranges
/// prune a far child. This isolates the effect of the coarse 320-column prune;
/// it is not a production visibility mode or renderer candidate contract.
pub fn observe_doom_classic_bsp_without_solid_range_pruning(
    map: &DoomMapCore,
    viewer: [i16; 2],
    heading: f64,
    watched_subsectors: &BTreeSet<u16>,
) -> Result<DoomClassicBspObservation, DoomGeometryError> {
    observe_doom_classic_bsp_with_order(
        map,
        viewer,
        heading,
        watched_subsectors,
        DoomClassicBspTraversalOrder::NearFirst,
        DoomClassicBspReplayControl::without_solid_range_pruning(),
    )
}

/// Diagnostic intervention that suppresses only the named source SEG's solid
/// range mutation while retaining its traversal and admission. This is a
/// causal shadow, not a selectable presentation policy.
pub fn observe_doom_classic_bsp_suppressing_solid_range_source_seg(
    map: &DoomMapCore,
    viewer: [i16; 2],
    heading: f64,
    watched_subsectors: &BTreeSet<u16>,
    source_seg: u32,
) -> Result<DoomClassicBspObservation, DoomGeometryError> {
    observe_doom_classic_bsp_with_order(
        map,
        viewer,
        heading,
        watched_subsectors,
        DoomClassicBspTraversalOrder::NearFirst,
        DoomClassicBspReplayControl::suppressing(source_seg),
    )
}

/// Internal corpus/provider control only. Callers cannot select Doom BSP
/// traversal order; production remains explicitly near-first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DoomClassicBspTraversalOrder {
    NearFirst,
    #[cfg(test)]
    FarFirstControl,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DoomClassicBspPruning {
    SolidRanges,
    OutsideFovOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DoomClassicBspReplayControl {
    pruning: DoomClassicBspPruning,
    suppressed_solid_range_source_seg: Option<u32>,
}

impl DoomClassicBspReplayControl {
    const fn ordinary() -> Self {
        Self {
            pruning: DoomClassicBspPruning::SolidRanges,
            suppressed_solid_range_source_seg: None,
        }
    }

    const fn without_solid_range_pruning() -> Self {
        Self {
            pruning: DoomClassicBspPruning::OutsideFovOnly,
            suppressed_solid_range_source_seg: None,
        }
    }

    const fn suppressing(source_seg: u32) -> Self {
        Self {
            pruning: DoomClassicBspPruning::SolidRanges,
            suppressed_solid_range_source_seg: Some(source_seg),
        }
    }
}

fn observe_doom_classic_bsp_with_order(
    map: &DoomMapCore,
    viewer: [i16; 2],
    heading: f64,
    watched_subsectors: &BTreeSet<u16>,
    traversal_order: DoomClassicBspTraversalOrder,
    control: DoomClassicBspReplayControl,
) -> Result<DoomClassicBspObservation, DoomGeometryError> {
    let root = map
        .nodes
        .len()
        .checked_sub(1)
        .ok_or(DoomGeometryError::MissingBspRoot {
            subsectors: map.subsectors.len(),
        })?;
    let occluders = observe_doom_seg_occluders(map)?
        .into_iter()
        .map(|observation| (observation.source_seg.record_index, observation))
        .collect::<BTreeMap<_, _>>();
    let mut observation = DoomClassicBspObservation::default();
    let mut solid_ranges = DoomClassicSolidRangeState {
        suppressed_source_seg: control.suppressed_solid_range_source_seg,
        ..DoomClassicSolidRangeState::default()
    };
    let mut ancestors = Vec::new();
    visit_doom_classic_bsp_child(
        map,
        DoomBspChild::Node(root as u16),
        viewer,
        heading,
        &occluders,
        &mut solid_ranges,
        &mut ancestors,
        watched_subsectors,
        &mut observation,
        traversal_order,
        control,
    )?;
    observation.solid_range_covered_columns = solid_ranges
        .intervals
        .iter()
        .map(|[first, last]| last - first + 1)
        .sum();
    Ok(observation)
}

#[cfg(test)]
fn observe_doom_classic_bsp_far_first_control(
    map: &DoomMapCore,
    viewer: [i16; 2],
    heading: f64,
    watched_subsectors: &BTreeSet<u16>,
) -> Result<DoomClassicBspObservation, DoomGeometryError> {
    observe_doom_classic_bsp_with_order(
        map,
        viewer,
        heading,
        watched_subsectors,
        DoomClassicBspTraversalOrder::FarFirstControl,
        DoomClassicBspReplayControl::ordinary(),
    )
}

// Doom stores `ceilingclip` as the last closed row (initially -1) and
// `floorclip` as the first closed row (initially view height). This provider
// normalizes the ceiling value to the first open row so the state remains
// unsigned. Keep all inclusive/exclusive translation in these helpers.
fn classic_open_rows(first_open: usize, first_closed: usize) -> Option<[usize; 2]> {
    let last_open = first_closed.checked_sub(1)?;
    (first_open <= last_open).then_some([first_open, last_open])
}

fn classic_ceiling_plane_rows(first_open: usize, projected_ceiling: usize) -> Option<[usize; 2]> {
    let last_plane_row = projected_ceiling.checked_sub(1)?;
    (first_open <= last_plane_row).then_some([first_open, last_plane_row])
}

fn classic_ceiling_after_mark_without_upper(first_open: usize, projected_ceiling: usize) -> usize {
    first_open.max(projected_ceiling)
}

/// Observes the Doom-owned vertical clip and plane-span facts for SEG wall
/// tiers already admitted by [`observe_doom_classic_bsp`]. The caller supplies
/// ordinary source wall triangles so texture/material resolution remains at
/// the corpus edge. No renderer object enters or leaves this function.
#[allow(clippy::too_many_arguments)]
pub fn observe_doom_classic_vertical_clip_state(
    map: &DoomMapCore,
    triangles: &[DoomSegTexturedWallTriangle],
    plane_marks: &[DoomSegPlaneMarkObservation],
    traversal: &DoomClassicBspObservation,
    viewer: [i16; 2],
    heading: f64,
    eye_height: f64,
) -> DoomSegClassicVerticalClipObservation {
    const COLUMNS: usize = 320;
    const ROWS: usize = 200;
    const HALF_HORIZONTAL_FOV: f64 = std::f64::consts::FRAC_PI_4;
    let half_vertical_fov = ((ROWS as f64 / COLUMNS as f64) * HALF_HORIZONTAL_FOV.tan()).atan();
    let mut result = DoomSegClassicVerticalClipObservation {
        admitted_segs: traversal.admitted_seg_order.len(),
        ..Default::default()
    };
    let mut ceiling_clip = vec![0usize; COLUMNS];
    let mut floor_clip = vec![ROWS; COLUMNS];
    let mut upper_sources = vec![BTreeSet::new(); COLUMNS];
    let mut lower_sources = vec![BTreeSet::new(); COLUMNS];
    let mut middle_sources = vec![BTreeSet::new(); COLUMNS];
    let mut paired_sky_boundary_sources = vec![BTreeSet::new(); COLUMNS];
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
        (((1.0 - normalized) * 0.5) * ROWS as f64) as usize
    };
    for source_seg in &traversal.admitted_seg_order {
        let Some(mark) = marks_by_seg.get(source_seg) else {
            result
                .ordered_coverage_fail_open
                .push(DoomOrderedCoverageFailOpen {
                    source_seg: *source_seg,
                    source_linedef: None,
                    column: None,
                    reason: DoomOrderedCoverageFailOpenReason::MissingPlaneMark,
                });
            continue;
        };
        let Some(seg) = segs_by_record.get(source_seg) else {
            result
                .ordered_coverage_fail_open
                .push(DoomOrderedCoverageFailOpen {
                    source_seg: *source_seg,
                    source_linedef: Some(mark.source_linedef.record_index),
                    column: None,
                    reason: DoomOrderedCoverageFailOpenReason::MissingSourceSeg,
                });
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
        if start_depth <= 0.0 || end_depth <= 0.0 {
            result
                .ordered_coverage_fail_open
                .push(DoomOrderedCoverageFailOpen {
                    source_seg: *source_seg,
                    source_linedef: Some(u32::from(seg.linedef)),
                    column: None,
                    reason: DoomOrderedCoverageFailOpenReason::ProjectionBehindViewer,
                });
            continue;
        }
        if source_segment_outside_horizontal_fov(start_angle, end_angle, HALF_HORIZONTAL_FOV) {
            result
                .ordered_coverage_fail_open
                .push(DoomOrderedCoverageFailOpen {
                    source_seg: *source_seg,
                    source_linedef: Some(u32::from(seg.linedef)),
                    column: None,
                    reason: DoomOrderedCoverageFailOpenReason::ProjectionOutsideHorizontalFov,
                });
            continue;
        }
        let [left, right_column] =
            source_fov_column_interval(start_angle, end_angle, HALF_HORIZONTAL_FOV, COLUMNS);
        if mark.paired_sky_ceiling_adjustment {
            for (offset, source_set) in paired_sky_boundary_sources[left..=right_column]
                .iter_mut()
                .enumerate()
            {
                source_set.insert(*source_seg);
                let column = left + offset;
                result
                    .ordered_coverage_transitions
                    .push(DoomOrderedCoverageTransition {
                        source_seg: *source_seg,
                        source_linedef: u32::from(seg.linedef),
                        front_sector: mark.front_sector.record_index,
                        back_sector: mark.back_sector.map(|source| source.record_index),
                        column,
                        upper_before: ceiling_clip[column],
                        lower_before: floor_clip[column],
                        upper_after: ceiling_clip[column],
                        lower_after: floor_clip[column],
                        retained_plane_interval: None,
                        reason: DoomOrderedCoverageTransitionReason::PairedSkyBoundaryRetained,
                    });
            }
        }
        let has_upper = tier_heights.contains_key(&(*source_seg, 0));
        let has_lower = tier_heights.contains_key(&(*source_seg, 1));
        let has_middle = tier_heights.contains_key(&(*source_seg, 2));
        let mut ceiling_plane_writes = Vec::new();
        let mut floor_plane_writes = Vec::new();
        for x in left..=right_column {
            let normalized = -1.0 + ((x as f64 + 0.5) / COLUMNS as f64) * 2.0;
            let local_angle = (normalized * HALF_HORIZONTAL_FOV.tan()).atan();
            let ray = [
                forward[0] * local_angle.cos() + right[0] * local_angle.sin(),
                forward[1] * local_angle.cos() + right[1] * local_angle.sin(),
            ];
            let Some(radial_depth) =
                source_ray_segment_depth(viewer, ray, [start.x, start.y], [end.x, end.y])
            else {
                result
                    .ordered_coverage_fail_open
                    .push(DoomOrderedCoverageFailOpen {
                        source_seg: *source_seg,
                        source_linedef: Some(u32::from(seg.linedef)),
                        column: Some(x),
                        reason: DoomOrderedCoverageFailOpenReason::RaySegmentDepthUnresolved,
                    });
                continue;
            };
            // Ray/SEG intersection returns distance along the normalized
            // horizontal ray. Rectilinear perspective rows are defined by
            // distance along the camera-forward axis instead.
            let forward_depth = radial_depth * local_angle.cos();
            let ceiling =
                row((f64::from(front_sector.ceiling_height) - eye_height).atan2(forward_depth))
                    .min(ROWS - 1);
            let floor =
                row((f64::from(front_sector.floor_height) - eye_height).atan2(forward_depth))
                    .min(ROWS - 1);
            let (ceiling, floor) = (ceiling.min(floor), ceiling.max(floor));
            if mark.ceiling_marked {
                // Doom stores the last closed ceiling row and initializes it
                // to -1. This observer stores the equivalent first open row
                // so it can remain unsigned; therefore the first retained
                // ceiling row is the clip value itself, not clip + 1.
                let retained = classic_ceiling_plane_rows(ceiling_clip[x], ceiling);
                if let Some([top, bottom]) = retained {
                    ceiling_plane_writes.push((x, top, bottom));
                    result
                        .ordered_coverage_transitions
                        .push(DoomOrderedCoverageTransition {
                            source_seg: *source_seg,
                            source_linedef: u32::from(seg.linedef),
                            front_sector: mark.front_sector.record_index,
                            back_sector: mark.back_sector.map(|source| source.record_index),
                            column: x,
                            upper_before: ceiling_clip[x],
                            lower_before: floor_clip[x],
                            upper_after: ceiling_clip[x],
                            lower_after: floor_clip[x],
                            retained_plane_interval: Some([top, bottom]),
                            reason: DoomOrderedCoverageTransitionReason::CeilingPlaneMarked,
                        });
                }
            }
            if mark.floor_marked {
                let top = floor.saturating_add(1);
                let bottom = floor_clip[x].saturating_sub(1);
                floor_plane_writes.push((x, top, bottom));
                if top <= bottom {
                    result
                        .ordered_coverage_transitions
                        .push(DoomOrderedCoverageTransition {
                            source_seg: *source_seg,
                            source_linedef: u32::from(seg.linedef),
                            front_sector: mark.front_sector.record_index,
                            back_sector: mark.back_sector.map(|source| source.record_index),
                            column: x,
                            upper_before: ceiling_clip[x],
                            lower_before: floor_clip[x],
                            upper_after: ceiling_clip[x],
                            lower_after: floor_clip[x],
                            retained_plane_interval: Some([top, bottom]),
                            reason: DoomOrderedCoverageTransitionReason::FloorPlaneMarked,
                        });
                }
            }
        }
        if !ceiling_plane_writes.is_empty() {
            retain_classic_plane_range(
                &mut result.plane_spans,
                classic_plane_key(DoomSegClassicPlaneKind::Ceiling, front_sector),
                mark.front_sector.record_index,
                *source_seg,
                &ceiling_plane_writes,
                COLUMNS,
            );
        }
        if !floor_plane_writes.is_empty() {
            retain_classic_plane_range(
                &mut result.plane_spans,
                classic_plane_key(DoomSegClassicPlaneKind::Floor, front_sector),
                mark.front_sector.record_index,
                *source_seg,
                &floor_plane_writes,
                COLUMNS,
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
                let normalized = -1.0 + ((x as f64 + 0.5) / COLUMNS as f64) * 2.0;
                let local_angle = (normalized * HALF_HORIZONTAL_FOV.tan()).atan();
                let ray = [
                    forward[0] * local_angle.cos() + right[0] * local_angle.sin(),
                    forward[1] * local_angle.cos() + right[1] * local_angle.sin(),
                ];
                let Some(radial_depth) =
                    source_ray_segment_depth(viewer, ray, [start.x, start.y], [end.x, end.y])
                else {
                    result
                        .ordered_coverage_fail_open
                        .push(DoomOrderedCoverageFailOpen {
                            source_seg: *source_seg,
                            source_linedef: Some(u32::from(seg.linedef)),
                            column: Some(x),
                            reason: DoomOrderedCoverageFailOpenReason::RaySegmentDepthUnresolved,
                        });
                    continue;
                };
                let forward_depth = radial_depth * local_angle.cos();
                let top = row((maximum - eye_height).atan2(forward_depth)).min(ROWS - 1);
                let bottom = row((minimum - eye_height).atan2(forward_depth)).min(ROWS - 1);
                let (top, bottom) = (top.min(bottom), top.max(bottom));
                let prior = [ceiling_clip[x], floor_clip[x]];
                // `ceiling_clip` is the first open row and `floor_clip` is the
                // first closed row. This is the unsigned normalization of
                // Doom's inclusive `ceilingclip` / `floorclip` pair.
                let open_interval_before = classic_open_rows(prior[0], prior[1]);
                let Some([open_top, open_bottom]) = open_interval_before else {
                    result.ordered_wall_intervals.push(DoomOrderedWallInterval {
                        source_seg: *source_seg,
                        source_linedef: u32::from(seg.linedef),
                        column: x,
                        role: *role,
                        raw_interval: [top, bottom],
                        open_interval_before: None,
                        retained_interval: None,
                    });
                    continue;
                };
                let retained_top = top.max(open_top);
                let retained_bottom = bottom.min(open_bottom);
                let retained_interval =
                    (retained_top <= retained_bottom).then_some([retained_top, retained_bottom]);
                result.ordered_wall_intervals.push(DoomOrderedWallInterval {
                    source_seg: *source_seg,
                    source_linedef: u32::from(seg.linedef),
                    column: x,
                    role: *role,
                    raw_interval: [top, bottom],
                    open_interval_before,
                    retained_interval,
                });
                match role {
                    DoomWallTextureRole::Upper => {
                        let Some(retained) = retained_interval else {
                            continue;
                        };
                        upper_sources[x].insert(*source_seg);
                        let next = ceiling_clip[x].max(retained[1].saturating_add(1));
                        result.ceiling_clip_updates += usize::from(next != ceiling_clip[x]);
                        ceiling_clip[x] = next;
                        if next != prior[0] {
                            result.ordered_coverage_transitions.push(
                                DoomOrderedCoverageTransition {
                                    source_seg: *source_seg,
                                    source_linedef: u32::from(seg.linedef),
                                    front_sector: mark.front_sector.record_index,
                                    back_sector: mark.back_sector.map(|source| source.record_index),
                                    column: x,
                                    upper_before: prior[0],
                                    lower_before: prior[1],
                                    upper_after: next,
                                    lower_after: prior[1],
                                    retained_plane_interval: None,
                                    reason: DoomOrderedCoverageTransitionReason::UpperTierRaised,
                                },
                            );
                        }
                    }
                    DoomWallTextureRole::Lower => {
                        let Some(retained) = retained_interval else {
                            continue;
                        };
                        lower_sources[x].insert(*source_seg);
                        let next = floor_clip[x].min(retained[0]);
                        result.floor_clip_updates += usize::from(next != floor_clip[x]);
                        floor_clip[x] = next;
                        if next != prior[1] {
                            result.ordered_coverage_transitions.push(
                                DoomOrderedCoverageTransition {
                                    source_seg: *source_seg,
                                    source_linedef: u32::from(seg.linedef),
                                    front_sector: mark.front_sector.record_index,
                                    back_sector: mark.back_sector.map(|source| source.record_index),
                                    column: x,
                                    upper_before: prior[0],
                                    lower_before: prior[1],
                                    upper_after: prior[0],
                                    lower_after: next,
                                    retained_plane_interval: None,
                                    reason: DoomOrderedCoverageTransitionReason::LowerTierLowered,
                                },
                            );
                        }
                    }
                    DoomWallTextureRole::Middle => {
                        if retained_interval.is_some() {
                            middle_sources[x].insert(*source_seg);
                        }
                    }
                }
                if x == COLUMNS / 2 {
                    center_trace = Some(format!("seg={source_seg} line={} tier={role:?} rows={top}..{bottom} clip-before={}..{} clip-after={}..{}", seg.linedef, prior[0], prior[1], ceiling_clip[x], floor_clip[x]));
                }
            }
            if let Some(sample) = center_trace {
                if result.samples.len() < 12 {
                    result.samples.push(sample);
                }
            }
        }
        for x in left..=right_column {
            let normalized = -1.0 + ((x as f64 + 0.5) / COLUMNS as f64) * 2.0;
            let local_angle = (normalized * HALF_HORIZONTAL_FOV.tan()).atan();
            let ray = [
                forward[0] * local_angle.cos() + right[0] * local_angle.sin(),
                forward[1] * local_angle.cos() + right[1] * local_angle.sin(),
            ];
            let Some(radial_depth) =
                source_ray_segment_depth(viewer, ray, [start.x, start.y], [end.x, end.y])
            else {
                result
                    .ordered_coverage_fail_open
                    .push(DoomOrderedCoverageFailOpen {
                        source_seg: *source_seg,
                        source_linedef: Some(u32::from(seg.linedef)),
                        column: Some(x),
                        reason: DoomOrderedCoverageFailOpenReason::RaySegmentDepthUnresolved,
                    });
                continue;
            };
            let forward_depth = radial_depth * local_angle.cos();
            let ceiling =
                row((f64::from(front_sector.ceiling_height) - eye_height).atan2(forward_depth))
                    .min(ROWS - 1);
            let floor =
                row((f64::from(front_sector.floor_height) - eye_height).atan2(forward_depth))
                    .min(ROWS - 1);
            let (ceiling, floor) = (ceiling.min(floor), ceiling.max(floor));
            if has_middle && mark.back_sector.is_none() && middle_sources[x].contains(source_seg) {
                let prior = [ceiling_clip[x], floor_clip[x]];
                result.ceiling_clip_updates += usize::from(ceiling_clip[x] != ROWS);
                result.floor_clip_updates += usize::from(floor_clip[x] != 0);
                ceiling_clip[x] = ROWS;
                floor_clip[x] = 0;
                if prior != [ROWS, 0] {
                    result
                        .ordered_coverage_transitions
                        .push(DoomOrderedCoverageTransition {
                            source_seg: *source_seg,
                            source_linedef: u32::from(seg.linedef),
                            front_sector: mark.front_sector.record_index,
                            back_sector: None,
                            column: x,
                            upper_before: prior[0],
                            lower_before: prior[1],
                            upper_after: ROWS,
                            lower_after: 0,
                            retained_plane_interval: None,
                            reason: DoomOrderedCoverageTransitionReason::OneSidedMiddleClosed,
                        });
                }
            } else {
                if !has_upper && mark.ceiling_marked {
                    let prior = ceiling_clip[x];
                    // Doom assigns `ceilingclip = yl - 1` here. In the
                    // normalized first-open representation that is `yl`.
                    let next = classic_ceiling_after_mark_without_upper(ceiling_clip[x], ceiling);
                    result.ceiling_clip_updates += usize::from(next != ceiling_clip[x]);
                    ceiling_clip[x] = next;
                    if next != prior {
                        result
                            .ordered_coverage_transitions
                            .push(DoomOrderedCoverageTransition {
                                source_seg: *source_seg,
                                source_linedef: u32::from(seg.linedef),
                                front_sector: mark.front_sector.record_index,
                                back_sector: mark.back_sector.map(|source| source.record_index),
                                column: x,
                                upper_before: prior,
                                lower_before: floor_clip[x],
                                upper_after: next,
                                lower_after: floor_clip[x],
                                retained_plane_interval: None,
                                reason: DoomOrderedCoverageTransitionReason::CeilingMarkClosed,
                            });
                    }
                }
                if !has_lower && mark.floor_marked {
                    let prior = floor_clip[x];
                    let next = floor_clip[x].min(floor.saturating_add(1));
                    result.floor_clip_updates += usize::from(next != floor_clip[x]);
                    floor_clip[x] = next;
                    if next != prior {
                        result
                            .ordered_coverage_transitions
                            .push(DoomOrderedCoverageTransition {
                                source_seg: *source_seg,
                                source_linedef: u32::from(seg.linedef),
                                front_sector: mark.front_sector.record_index,
                                back_sector: mark.back_sector.map(|source| source.record_index),
                                column: x,
                                upper_before: ceiling_clip[x],
                                lower_before: prior,
                                upper_after: ceiling_clip[x],
                                lower_after: next,
                                retained_plane_interval: None,
                                reason: DoomOrderedCoverageTransitionReason::FloorMarkClosed,
                            });
                    }
                }
            }
        }
    }
    finalize_classic_plane_spans(&mut result.plane_spans);
    result.column_traces = (0..COLUMNS)
        .filter_map(|column| {
            let upper_source_segs = std::mem::take(&mut upper_sources[column]);
            let lower_source_segs = std::mem::take(&mut lower_sources[column]);
            let middle_source_segs = std::mem::take(&mut middle_sources[column]);
            let paired_sky_boundary_source_segs =
                std::mem::take(&mut paired_sky_boundary_sources[column]);
            (ceiling_clip[column] != 0
                || floor_clip[column] != ROWS
                || !upper_source_segs.is_empty()
                || !lower_source_segs.is_empty()
                || !middle_source_segs.is_empty()
                || !paired_sky_boundary_source_segs.is_empty())
            .then_some(DoomSegClassicVerticalColumnTrace {
                column,
                ceiling_clip: ceiling_clip[column],
                floor_clip: floor_clip[column],
                upper_source_segs,
                lower_source_segs,
                middle_source_segs,
                paired_sky_boundary_source_segs,
            })
        })
        .collect();
    result
}

fn classic_plane_key(kind: DoomSegClassicPlaneKind, sector: &DoomSector) -> DoomSegClassicPlaneKey {
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

fn retain_classic_plane_range(
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
        .expect("a minimum column proves a valid plane write");
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

fn finalize_classic_plane_spans(observation: &mut DoomSegClassicPlaneSpanObservation) {
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
                key_cells
            ));
        }
    }
}

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

#[allow(clippy::too_many_arguments)]
fn visit_doom_classic_bsp_child(
    map: &DoomMapCore,
    child: DoomBspChild,
    viewer: [i16; 2],
    heading: f64,
    occluders: &BTreeMap<u32, DoomSegOccluderObservation>,
    solid_ranges: &mut DoomClassicSolidRangeState,
    ancestors: &mut Vec<u16>,
    watched_subsectors: &BTreeSet<u16>,
    observation: &mut DoomClassicBspObservation,
    traversal_order: DoomClassicBspTraversalOrder,
    control: DoomClassicBspReplayControl,
) -> Result<(), DoomGeometryError> {
    match child {
        DoomBspChild::Subsector(index) => {
            let subsector = map.subsectors.get(usize::from(index)).ok_or(
                DoomGeometryError::BspSubsectorOutOfBounds {
                    subsector_index: index,
                    available: map.subsectors.len(),
                },
            )?;
            observation.leaves_visited += 1;
            observation.visited_subsectors.insert(index);
            let first = usize::from(subsector.first_seg);
            let end = first + usize::from(subsector.seg_count);
            for seg in &map.segs[first..end] {
                admit_doom_classic_seg(
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
                return Err(DoomGeometryError::BspCycle { node_index: index });
            }
            let node =
                map.nodes
                    .get(usize::from(index))
                    .ok_or(DoomGeometryError::BspNodeOutOfBounds {
                        node_index: index,
                        available: map.nodes.len(),
                    })?;
            ancestors.push(index);
            let side = i64::from(node.delta_x) * i64::from(viewer[1] - node.y)
                - i64::from(node.delta_y) * i64::from(viewer[0] - node.x);
            let (near, _near_bbox, far, far_bbox) = if side < 0 {
                (
                    node.right_child,
                    node.right_bbox,
                    node.left_child,
                    node.left_bbox,
                )
            } else {
                (
                    node.left_child,
                    node.left_bbox,
                    node.right_child,
                    node.right_bbox,
                )
            };
            let (first, second, second_bbox) = match traversal_order {
                DoomClassicBspTraversalOrder::NearFirst => (near, far, far_bbox),
                #[cfg(test)]
                DoomClassicBspTraversalOrder::FarFirstControl => (far, near, _near_bbox),
            };
            visit_doom_classic_bsp_child(
                map,
                first,
                viewer,
                heading,
                occluders,
                solid_ranges,
                ancestors,
                watched_subsectors,
                observation,
                traversal_order,
                control,
            )?;
            let watched_far = watched_subsectors
                .iter()
                .filter_map(|target| {
                    doom_bsp_child_contains_subsector(map, second, *target).then_some(*target)
                })
                .collect::<Vec<_>>();
            match source_bbox_fov_column_interval(
                viewer,
                heading,
                second_bbox,
                std::f64::consts::FRAC_PI_4,
                320,
            ) {
                SourceBBoxProjection::OutsideFov => {
                    observation.far_children_outside_fov += 1;
                    record_watched_subsector_elision(
                        observation,
                        index,
                        "outside-fov",
                        &watched_far,
                        None,
                        None,
                        None,
                    );
                }
                SourceBBoxProjection::Interval(interval) => {
                    if let Some(covering_range) =
                        (control.pruning == DoomClassicBspPruning::SolidRanges)
                            .then(|| {
                                solid_ranges.intervals.iter().find(|[first, last]| {
                                    *first <= interval[0] && interval[1] <= *last
                                })
                            })
                            .flatten()
                    {
                        observation.far_children_pruned += 1;
                        record_watched_subsector_elision(
                            observation,
                            index,
                            "solid-range",
                            &watched_far,
                            Some(interval),
                            Some(*covering_range),
                            solid_ranges.provenance.iter().find(|provenance| {
                                provenance.interval[0] <= interval[0]
                                    && interval[1] <= provenance.interval[1]
                            }),
                        );
                    } else {
                        visit_doom_classic_bsp_child(
                            map,
                            second,
                            viewer,
                            heading,
                            occluders,
                            solid_ranges,
                            ancestors,
                            watched_subsectors,
                            observation,
                            traversal_order,
                            control,
                        )?;
                    }
                }
                SourceBBoxProjection::Uncertain => {
                    observation.far_children_fail_open += 1;
                    visit_doom_classic_bsp_child(
                        map,
                        second,
                        viewer,
                        heading,
                        occluders,
                        solid_ranges,
                        ancestors,
                        watched_subsectors,
                        observation,
                        traversal_order,
                        control,
                    )?;
                }
            }
            ancestors.pop();
            Ok(())
        }
    }
}

fn record_watched_subsector_elision(
    observation: &mut DoomClassicBspObservation,
    node: u16,
    reason: &str,
    subsectors: &[u16],
    interval: Option<[usize; 2]>,
    covering_range: Option<[usize; 2]>,
    covering_provenance: Option<&DoomClassicSolidRangeProvenance>,
) {
    if !subsectors.is_empty() {
        let event_ordinal = observation.watched_elision_provenance.len();
        observation.watched_subsector_elisions.push(format!(
            "node={node}:reason={reason}:subsectors={subsectors:?}:interval={interval:?}:covering-range={covering_range:?}"
        ));
        observation
            .watched_elision_provenance
            .push(DoomClassicWatchedElisionProvenance {
                event_ordinal,
                node,
                reason: reason.to_owned(),
                subsectors: subsectors.to_vec(),
                projected_interval: interval,
                covering_range,
                covering_source_segs: covering_provenance
                    .map(|provenance| provenance.source_segs.clone())
                    .unwrap_or_default(),
                covering_source_linedefs: covering_provenance
                    .map(|provenance| provenance.source_linedefs.clone())
                    .unwrap_or_default(),
            });
    }
}

fn doom_bsp_child_contains_subsector(map: &DoomMapCore, child: DoomBspChild, target: u16) -> bool {
    let mut visited_nodes = HashSet::new();
    doom_bsp_child_contains_subsector_inner(map, child, target, &mut visited_nodes)
}

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

fn admit_doom_classic_seg(
    map: &DoomMapCore,
    seg: &doom_map_provider::DoomSeg,
    viewer: [i16; 2],
    heading: f64,
    occluders: &BTreeMap<u32, DoomSegOccluderObservation>,
    solid_ranges: &mut DoomClassicSolidRangeState,
    observation: &mut DoomClassicBspObservation,
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
    let solid = authority.kind != DoomSegOccluderKind::Open;
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
        if solid_ranges.suppressed_source_seg == Some(seg.source.record_index) {
            observation.suppressed_solid_range_mutations.push(
                DoomClassicSuppressedSolidRangeMutation {
                    source_seg: seg.source.record_index,
                    source_linedef: u32::from(seg.linedef),
                    input_interval: interval,
                },
            );
            return;
        }
        let fully_covered = merge_solid_range(&mut solid_ranges.intervals, interval);
        let merged = merge_solid_range_provenance(
            &mut solid_ranges.provenance,
            interval,
            seg.source.record_index,
            u32::from(seg.linedef),
        );
        debug_assert_eq!(
            solid_ranges.intervals,
            solid_ranges
                .provenance
                .iter()
                .map(|provenance| provenance.interval)
                .collect::<Vec<_>>()
        );
        observation
            .solid_range_events
            .push(DoomClassicSolidRangeEvent {
                event_ordinal: observation.solid_range_events.len(),
                source_seg: seg.source.record_index,
                source_linedef: u32::from(seg.linedef),
                input_interval: interval,
                fully_covered_before: fully_covered,
                merged_range: merged.interval,
                contributing_source_segs: merged.source_segs.clone(),
                contributing_source_linedefs: merged.source_linedefs.clone(),
            });
        if fully_covered {
            observation.solid_range_fully_covered += 1;
        } else {
            observation.solid_range_contributors += 1;
        }
    } else if solid {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceBBoxProjection {
    OutsideFov,
    Interval([usize; 2]),
    Uncertain,
}

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
    let source = [top, bottom, left, right];
    let coordinates = CHECK_COORD[box_position];
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
    let span = (angles[0] - angles[1]).abs();
    if span >= std::f64::consts::PI {
        return SourceBBoxProjection::Uncertain;
    }
    let minimum = angles[0].min(angles[1]);
    let maximum = angles[0].max(angles[1]);
    if maximum < -half_fov || minimum > half_fov {
        SourceBBoxProjection::OutsideFov
    } else {
        SourceBBoxProjection::Interval(source_fov_column_interval(
            minimum, maximum, half_fov, columns,
        ))
    }
}

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

fn source_segment_outside_horizontal_fov(
    first_angle: f64,
    second_angle: f64,
    half_fov: f64,
) -> bool {
    (first_angle > half_fov && second_angle > half_fov)
        || (first_angle < -half_fov && second_angle < -half_fov)
}

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

fn merge_solid_range_provenance(
    ranges: &mut Vec<DoomClassicSolidRangeProvenance>,
    interval: [usize; 2],
    source_seg: u32,
    source_linedef: u32,
) -> DoomClassicSolidRangeProvenance {
    let mut merged = DoomClassicSolidRangeProvenance {
        interval,
        source_segs: BTreeSet::from([source_seg]),
        source_linedefs: BTreeSet::from([source_linedef]),
    };
    let mut index = 0;
    while index < ranges.len() {
        let candidate = &ranges[index];
        if candidate.interval[1].saturating_add(1) < merged.interval[0]
            || merged.interval[1].saturating_add(1) < candidate.interval[0]
        {
            index += 1;
            continue;
        }
        let candidate = ranges.remove(index);
        merged.interval[0] = merged.interval[0].min(candidate.interval[0]);
        merged.interval[1] = merged.interval[1].max(candidate.interval[1]);
        merged.source_segs.extend(candidate.source_segs);
        merged.source_linedefs.extend(candidate.source_linedefs);
    }
    ranges.insert(index, merged.clone());
    merged
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceSegFacing {
    Front,
    Back,
    EdgeOn,
}

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

/// Resolves linedef endpoints and sidedef-sector ownership for later lowering.
pub fn resolve_doom_wall_candidates(
    map: &DoomMapCore,
) -> Result<Vec<DoomWallCandidate>, DoomGeometryError> {
    map.linedefs
        .iter()
        .enumerate()
        .map(|(linedef_index, linedef)| {
            let right = resolve_side(map, linedef.right_sidedef);
            let left = resolve_side(map, linedef.left_sidedef);
            if right.is_none() && left.is_none() {
                return Err(DoomGeometryError::MissingBothSidedefs {
                    linedef_index: linedef_index as u32,
                });
            }
            let start = {
                let vertex = &map.vertices[usize::from(linedef.start_vertex)];
                [vertex.x, vertex.y]
            };
            let end = {
                let vertex = &map.vertices[usize::from(linedef.end_vertex)];
                [vertex.x, vertex.y]
            };
            if start == end {
                return Err(DoomGeometryError::DegenerateLinedef {
                    linedef_index: linedef_index as u32,
                });
            }
            Ok(DoomWallCandidate {
                source_linedef: linedef.source,
                linedef_flags: linedef.flags,
                start,
                end,
                right,
                left,
            })
        })
        .collect()
}

/// Counts the basic sidedef topology that later wall lowering must handle.
pub fn audit_doom_wall_topology(candidates: &[DoomWallCandidate]) -> DoomWallTopologyAudit {
    let one_sided = candidates
        .iter()
        .filter(|candidate| candidate.right.is_none() || candidate.left.is_none())
        .count();
    let two_sided = candidates.len() - one_sided;
    let same_sector_two_sided = candidates
        .iter()
        .filter_map(|candidate| candidate.right.as_ref().zip(candidate.left.as_ref()))
        .filter(|(right, left)| right.sector_index == left.sector_index)
        .count();
    DoomWallTopologyAudit {
        candidates: candidates.len(),
        one_sided,
        two_sided,
        same_sector_two_sided,
    }
}

/// Audits source vertical intervals without treating a closed opening as an
/// error. Such records are diagnostic pressure for later portal and movement
/// policy, not grounds for rewriting the map.
pub fn audit_doom_vertical_topology(
    map: &DoomMapCore,
) -> Result<DoomVerticalTopologyAudit, DoomGeometryError> {
    let candidates = resolve_doom_wall_candidates(map)?;
    let two_sided_openings = candidates
        .iter()
        .filter_map(|candidate| candidate.right.as_ref().zip(candidate.left.as_ref()))
        .collect::<Vec<_>>();
    Ok(DoomVerticalTopologyAudit {
        sectors: map.sectors.len(),
        sectors_without_positive_clearance: map
            .sectors
            .iter()
            .filter(|sector| sector.floor_height >= sector.ceiling_height)
            .count(),
        two_sided_openings: two_sided_openings.len(),
        two_sided_openings_without_positive_clearance: two_sided_openings
            .iter()
            .filter(|(right, left)| {
                let opening_floor = map.sectors[usize::from(right.sector_index)]
                    .floor_height
                    .max(map.sectors[usize::from(left.sector_index)].floor_height);
                let opening_ceiling = map.sectors[usize::from(right.sector_index)]
                    .ceiling_height
                    .min(map.sectors[usize::from(left.sector_index)].ceiling_height);
                opening_floor >= opening_ceiling
            })
            .count(),
    })
}

/// Resolves classic texture pegging into a source-traceable vertical anchor.
/// The owned candidate side is the source renderer's front side; the optional
/// opposite candidate side is back. No renderer or view-dependent clipping is
/// admitted here.
pub fn resolve_doom_wall_texture_placements(
    map: &DoomMapCore,
    extents: &[DoomTextureExtent],
) -> Result<Vec<DoomWallTexturePlacement>, DoomGeometryError> {
    let candidates = resolve_doom_wall_candidates(map)?;
    resolve_doom_wall_texture_bindings(map, extents)?
        .into_iter()
        .map(|binding| {
            let candidate = candidates
                .iter()
                .find(|candidate| candidate.source_linedef == binding.axis.source_linedef)
                .expect("wall texture binding originates from a resolved candidate");
            let (front, back) = match binding.axis.side {
                DoomWallSideKind::Right => (candidate.right.as_ref(), candidate.left.as_ref()),
                DoomWallSideKind::Left => (candidate.left.as_ref(), candidate.right.as_ref()),
            };
            let front = front.expect("wall texture axis has an owning candidate side");
            let texture_height = i32::from(binding.texture_height);
            let texture_mid_y = match binding.axis.role {
                DoomWallTextureRole::Upper => {
                    if binding.axis.linedef_flags & 0x0008 != 0 {
                        i32::from(map.sectors[usize::from(front.sector_index)].ceiling_height)
                    } else {
                        i32::from(
                            map.sectors[usize::from(
                                back.expect("upper texture requires two sides").sector_index,
                            )]
                            .ceiling_height,
                        ) + texture_height
                    }
                }
                DoomWallTextureRole::Lower => {
                    if binding.axis.linedef_flags & 0x0010 != 0 {
                        i32::from(map.sectors[usize::from(front.sector_index)].ceiling_height)
                    } else {
                        i32::from(
                            map.sectors[usize::from(
                                back.expect("lower texture requires two sides").sector_index,
                            )]
                            .floor_height,
                        )
                    }
                }
                DoomWallTextureRole::Middle => match back {
                    Some(back) if binding.axis.linedef_flags & 0x0010 != 0 => {
                        i32::from(
                            map.sectors[usize::from(front.sector_index)]
                                .floor_height
                                .max(map.sectors[usize::from(back.sector_index)].floor_height),
                        ) + texture_height
                    }
                    Some(back) => i32::from(
                        map.sectors[usize::from(front.sector_index)]
                            .ceiling_height
                            .min(map.sectors[usize::from(back.sector_index)].ceiling_height),
                    ),
                    None if binding.axis.linedef_flags & 0x0010 != 0 => {
                        i32::from(map.sectors[usize::from(front.sector_index)].floor_height)
                            + texture_height
                    }
                    None => i32::from(map.sectors[usize::from(front.sector_index)].ceiling_height),
                },
            } + i32::from(binding.axis.v_offset);
            Ok(DoomWallTexturePlacement {
                source_sector: front.source_sector,
                binding,
                texture_mid_y,
            })
        })
        .collect()
}

/// Resolves every classic Doom subsector as a closed, ordered boundary.
///
/// The map decoder has already bounded and validated the seg ranges and
/// vertex indices. A broken loop remains an explicit import error rather than
/// becoming an arbitrary floor or ceiling polygon.
pub fn resolve_doom_subsector_loops(
    map: &DoomMapCore,
) -> Result<Vec<DoomSubsectorLoop>, DoomGeometryError> {
    map.subsectors
        .iter()
        .enumerate()
        .map(|(subsector_index, _)| resolve_doom_subsector_loop(map, subsector_index))
        .collect()
}

/// Audits every subsector independently under the strict `SEGS`-only loop
/// rule, preserving all source-indexed rejections for diagnostics.
pub fn audit_doom_subsector_loop_closure(map: &DoomMapCore) -> DoomSubsectorLoopClosureAudit {
    let mut audit = DoomSubsectorLoopClosureAudit {
        subsectors: map.subsectors.len(),
        closed_loops: 0,
        rejected: Vec::new(),
    };
    for subsector_index in 0..map.subsectors.len() {
        match resolve_doom_subsector_loop(map, subsector_index) {
            Ok(_) => audit.closed_loops += 1,
            Err(error) => audit.rejected.push(error),
        }
    }
    audit
}

fn resolve_doom_subsector_loop(
    map: &DoomMapCore,
    subsector_index: usize,
) -> Result<DoomSubsectorLoop, DoomGeometryError> {
    let subsector = &map.subsectors[subsector_index];
    if subsector.seg_count < 3 {
        return Err(DoomGeometryError::SubsectorTooSmall {
            subsector_index: subsector_index as u32,
            seg_count: subsector.seg_count,
        });
    }
    let first = usize::from(subsector.first_seg);
    let end = first + usize::from(subsector.seg_count);
    let segs = &map.segs[first..end];
    let first_start = point_for_vertex(map, segs[0].start_vertex);
    let mut vertices = Vec::with_capacity(segs.len());
    let mut source_segs = Vec::with_capacity(segs.len());
    let mut used = vec![false; segs.len()];
    let mut current_index = 0;
    let mut current_reversed = false;
    for offset in 0..segs.len() {
        let seg = &segs[current_index];
        let source_start = point_for_vertex(map, seg.start_vertex);
        let source_end = point_for_vertex(map, seg.end_vertex);
        if source_start == source_end {
            return Err(DoomGeometryError::DegenerateSeg {
                subsector_index: subsector_index as u32,
                seg_index: seg.source.record_index,
            });
        }
        let (start, expected_start) = if current_reversed {
            (source_end, source_start)
        } else {
            (source_start, source_end)
        };
        vertices.push(start);
        source_segs.push(DoomSubsectorBoundaryEdge {
            source_seg: seg.source,
            reversed_from_source: current_reversed,
        });
        used[current_index] = true;
        if offset + 1 == segs.len() {
            if expected_start != first_start {
                return Err(DoomGeometryError::SubsectorBoundaryOpen {
                    subsector_index: subsector_index as u32,
                    previous_seg_index: seg.source.record_index,
                    next_seg_index: segs[0].source.record_index,
                });
            }
            continue;
        }
        let next = segs
            .iter()
            .enumerate()
            .flat_map(|(index, candidate)| {
                if used[index] {
                    return Vec::new();
                }
                let candidate_start = point_for_vertex(map, candidate.start_vertex);
                let candidate_end = point_for_vertex(map, candidate.end_vertex);
                let mut matches = Vec::new();
                if candidate_start == expected_start {
                    matches.push((index, false));
                }
                if candidate_end == expected_start {
                    matches.push((index, true));
                }
                matches
            })
            .collect::<Vec<_>>();
        match next.as_slice() {
            [(index, reversed)] => {
                current_index = *index;
                current_reversed = *reversed;
            }
            [] => {
                return Err(DoomGeometryError::SubsectorBoundaryOpen {
                    subsector_index: subsector_index as u32,
                    previous_seg_index: seg.source.record_index,
                    next_seg_index: segs[0].source.record_index,
                });
            }
            _ => {
                return Err(DoomGeometryError::SubsectorBoundaryAmbiguous {
                    subsector_index: subsector_index as u32,
                    previous_seg_index: seg.source.record_index,
                });
            }
        }
    }
    Ok(DoomSubsectorLoop {
        source_subsector: subsector.source,
        source_segs,
        vertices,
    })
}

/// Establishes the root-to-leaf BSP ownership path for each subsector.
///
/// A subsector's `SEGS` can describe only its map-wall portions; the retained
/// partition path makes the additional half-plane evidence explicit without
/// pretending that it has already been converted into floor geometry.
pub fn resolve_doom_subsector_bsp_paths(
    map: &DoomMapCore,
) -> Result<Vec<DoomSubsectorBspPath>, DoomGeometryError> {
    if map.subsectors.is_empty() {
        return Ok(Vec::new());
    }
    let root = map
        .nodes
        .len()
        .checked_sub(1)
        .ok_or(DoomGeometryError::MissingBspRoot {
            subsectors: map.subsectors.len(),
        })? as u16;
    let mut paths = vec![None; map.subsectors.len()];
    let mut ancestors = Vec::new();
    visit_bsp_node(map, root, &mut Vec::new(), &mut ancestors, &mut paths)?;
    paths
        .into_iter()
        .enumerate()
        .map(|(subsector_index, steps)| {
            let steps = steps.ok_or(DoomGeometryError::BspSubsectorOutOfBounds {
                subsector_index: subsector_index as u16,
                available: map.subsectors.len(),
            })?;
            Ok(DoomSubsectorBspPath {
                source_subsector: map.subsectors[subsector_index].source,
                steps,
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

/// Retains classic Doom's viewer-relative near-first BSP leaf order without
/// claiming that traversal alone determines presentation visibility. Screen
/// span clipping and occluder authority remain later, separate questions.
pub fn resolve_doom_viewer_subsector_order(
    map: &DoomMapCore,
    viewer: [i16; 2],
) -> Result<Vec<DoomSourceRecord>, DoomGeometryError> {
    if map.subsectors.is_empty() {
        return Ok(Vec::new());
    }
    let root = map
        .nodes
        .len()
        .checked_sub(1)
        .ok_or(DoomGeometryError::MissingBspRoot {
            subsectors: map.subsectors.len(),
        })? as u16;
    let mut order = Vec::with_capacity(map.subsectors.len());
    let mut ancestors = Vec::new();
    visit_viewer_bsp_child(
        map,
        DoomBspChild::Node(root),
        viewer,
        &mut ancestors,
        &mut order,
    )?;
    Ok(order)
}

fn visit_viewer_bsp_child(
    map: &DoomMapCore,
    child: DoomBspChild,
    viewer: [i16; 2],
    ancestors: &mut Vec<u16>,
    order: &mut Vec<DoomSourceRecord>,
) -> Result<(), DoomGeometryError> {
    match child {
        DoomBspChild::Subsector(index) => {
            let subsector = map.subsectors.get(usize::from(index)).ok_or(
                DoomGeometryError::BspSubsectorOutOfBounds {
                    subsector_index: index,
                    available: map.subsectors.len(),
                },
            )?;
            order.push(subsector.source);
            Ok(())
        }
        DoomBspChild::Node(index) => {
            if ancestors.contains(&index) {
                return Err(DoomGeometryError::BspCycle { node_index: index });
            }
            let node = &map.nodes[usize::from(index)];
            ancestors.push(index);
            let distance = f64::from(node.delta_x) * f64::from(viewer[1] - node.y)
                - f64::from(node.delta_y) * f64::from(viewer[0] - node.x);
            let (near, far) = if distance < 0.0 {
                (node.right_child, node.left_child)
            } else {
                (node.left_child, node.right_child)
            };
            visit_viewer_bsp_child(map, near, viewer, ancestors, order)?;
            visit_viewer_bsp_child(map, far, viewer, ancestors, order)?;
            ancestors.pop();
            Ok(())
        }
    }
}

/// Summarizes how much BSP partition evidence each leaf accumulates.
pub fn audit_doom_subsector_bsp_paths(paths: &[DoomSubsectorBspPath]) -> DoomBspPathAudit {
    let minimum_depth = paths.iter().map(|path| path.steps.len()).min().unwrap_or(0);
    let maximum_depth = paths.iter().map(|path| path.steps.len()).max().unwrap_or(0);
    DoomBspPathAudit {
        subsectors: paths.len(),
        minimum_depth,
        maximum_depth,
    }
}

/// Locates an integer point only when its retained BSP path is unambiguous.
/// A point on any partition plane is rejected rather than assigned by a hidden
/// tie-break rule; collision/spawn policy must make that choice explicitly.
pub fn locate_doom_point_subsector(
    point: [i16; 2],
    paths: &[DoomSubsectorBspPath],
) -> Result<DoomPointSubsectorObservation, DoomGeometryError> {
    let matches = paths
        .iter()
        .filter(|path| {
            path.steps.iter().all(|step| {
                let distance = partition_distance([f64::from(point[0]), f64::from(point[1])], step);
                match step.side {
                    DoomBspSide::Right => distance < 0.0,
                    DoomBspSide::Left => distance > 0.0,
                }
            })
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [path] => Ok(DoomPointSubsectorObservation {
            point,
            source_subsector: path.source_subsector,
        }),
        _ => Err(DoomGeometryError::PointNotInsideUniqueSubsector {
            x: point[0],
            y: point[1],
        }),
    }
}

/// Clips the finite map extent against each root-to-leaf BSP path.
///
/// The classic node convention used here treats its `right` child as the
/// non-positive cross-product half-plane and its `left` child as the
/// non-negative half-plane. This has no renderer interpretation yet.
pub fn resolve_doom_subsector_regions(
    map: &DoomMapCore,
    paths: &[DoomSubsectorBspPath],
) -> Result<Vec<DoomSubsectorRegion>, DoomGeometryError> {
    let bounds = map_bounds(map)?;
    paths
        .iter()
        .enumerate()
        .map(|(subsector_index, path)| {
            let mut vertices = bounds.to_vec();
            for step in &path.steps {
                vertices = clip_convex_region(&vertices, step);
            }
            if vertices.len() < 3 {
                return Err(DoomGeometryError::EmptySubsectorRegion {
                    subsector_index: subsector_index as u32,
                });
            }
            Ok(DoomSubsectorRegion {
                source_subsector: path.source_subsector,
                vertices,
            })
        })
        .collect()
}

/// Counts source seg endpoints which do not satisfy the idealized integer BSP
/// half-planes exactly. This is diagnostic evidence for later rounding policy,
/// not a reason to mutate or discard the original map records.
pub fn audit_doom_subsector_region_endpoints(
    map: &DoomMapCore,
    paths: &[DoomSubsectorBspPath],
) -> DoomSubsectorRegionAudit {
    let mut seg_endpoints = 0;
    let mut endpoints_outside_paths = 0;
    let mut maximum_outside_distance = 0.0_f64;
    for (subsector, path) in map.subsectors.iter().zip(paths) {
        let first = usize::from(subsector.first_seg);
        let end = first + usize::from(subsector.seg_count);
        for seg in &map.segs[first..end] {
            for vertex_index in [seg.start_vertex, seg.end_vertex] {
                seg_endpoints += 1;
                let point = point_for_vertex(map, vertex_index).map(f64::from);
                let outside_distance = path
                    .steps
                    .iter()
                    .map(|step| {
                        outside_partition_distance(partition_distance(point, step), step.side)
                    })
                    .fold(0.0_f64, f64::max);
                if outside_distance > 1.0e-9 {
                    endpoints_outside_paths += 1;
                    maximum_outside_distance = maximum_outside_distance.max(outside_distance);
                }
            }
        }
    }
    DoomSubsectorRegionAudit {
        regions: paths.len(),
        seg_endpoints,
        endpoints_outside_paths,
        maximum_outside_distance,
    }
}

/// Resolves the sector each classic Doom subsector belongs to from its seg
/// direction and the corresponding linedef side.
pub fn resolve_doom_subsector_sector_ownership(
    map: &DoomMapCore,
) -> Result<Vec<DoomSubsectorSectorOwnership>, DoomGeometryError> {
    map.subsectors
        .iter()
        .enumerate()
        .map(|(subsector_index, subsector)| {
            let first = usize::from(subsector.first_seg);
            let end = first + usize::from(subsector.seg_count);
            let mut ownership = None;
            for seg in &map.segs[first..end] {
                let linedef = &map.linedefs[usize::from(seg.linedef)];
                let sidedef_index = match seg.direction {
                    0 => linedef.right_sidedef,
                    1 => linedef.left_sidedef,
                    _ => None,
                }
                .ok_or(DoomGeometryError::SubsectorSegMissingOwningSide {
                    subsector_index: subsector_index as u32,
                    seg_index: seg.source.record_index,
                })?;
                let sidedef = &map.sidedefs[usize::from(sidedef_index)];
                let sector = &map.sectors[usize::from(sidedef.sector)];
                match ownership {
                    Some((first_sector, _)) if first_sector != sidedef.sector => {
                        return Err(DoomGeometryError::SubsectorMixedSectors {
                            subsector_index: subsector_index as u32,
                            first_sector,
                            observed_sector: sidedef.sector,
                            seg_index: seg.source.record_index,
                        });
                    }
                    Some(_) => {}
                    None => ownership = Some((sidedef.sector, sector.source)),
                }
            }
            let (sector_index, source_sector) =
                ownership.ok_or(DoomGeometryError::SubsectorSegMissingOwningSide {
                    subsector_index: subsector_index as u32,
                    seg_index: 0,
                })?;
            Ok(DoomSubsectorSectorOwnership {
                source_subsector: subsector.source,
                source_sector,
                sector_index,
            })
        })
        .collect()
}

/// Resolves the source BSP-leaf membership of every linedef from `SSECTORS`
/// and their `SEGS`, preserving source order and a one-to-many result.
pub fn resolve_doom_linedef_subsector_membership(
    map: &DoomMapCore,
) -> Vec<DoomLinedefSubsectorMembership> {
    let mut memberships = vec![Vec::new(); map.linedefs.len()];
    for subsector in &map.subsectors {
        let first = usize::from(subsector.first_seg);
        let end = first + usize::from(subsector.seg_count);
        for seg in &map.segs[first..end] {
            let membership = &mut memberships[usize::from(seg.linedef)];
            if !membership.contains(&subsector.source) {
                membership.push(subsector.source);
            }
        }
    }
    map.linedefs
        .iter()
        .zip(memberships)
        .map(
            |(linedef, source_subsectors)| DoomLinedefSubsectorMembership {
                source_linedef: linedef.source,
                source_subsectors,
            },
        )
        .collect()
}

/// Lowers each bounded BSP leaf into floor and ceiling triangle candidates.
///
/// This preserves the BSP-leaf partition rather than attempting to merge
/// sectors or infer texture semantics. It is deliberately renderer-neutral.
pub fn lower_doom_subsector_surfaces(
    map: &DoomMapCore,
    paths: &[DoomSubsectorBspPath],
) -> Result<Vec<DoomSurfaceTriangle>, DoomGeometryError> {
    let regions = resolve_doom_subsector_regions(map, paths)?;
    let ownership = resolve_doom_subsector_sector_ownership(map)?;
    let mut triangles = Vec::new();
    for (subsector_index, (region, ownership)) in regions.iter().zip(&ownership).enumerate() {
        if polygon_signed_area(&region.vertices).abs() <= f64::EPSILON {
            return Err(DoomGeometryError::DegenerateSubsectorRegion {
                subsector_index: subsector_index as u32,
            });
        }
        let sector = &map.sectors[usize::from(ownership.sector_index)];
        for (plane, height, texture_name, reverse_winding) in [
            (
                DoomSurfacePlane::Floor,
                sector.floor_height,
                sector.floor_texture.as_str(),
                true,
            ),
            (
                DoomSurfacePlane::Ceiling,
                sector.ceiling_height,
                sector.ceiling_texture.as_str(),
                false,
            ),
        ] {
            for index in 1..region.vertices.len() - 1 {
                let points = [
                    region.vertices[0],
                    region.vertices[index],
                    region.vertices[index + 1],
                ];
                let positions = if reverse_winding {
                    [
                        [points[0][0], f64::from(height), points[0][1]],
                        [points[2][0], f64::from(height), points[2][1]],
                        [points[1][0], f64::from(height), points[1][1]],
                    ]
                } else {
                    points.map(|point| doom_point_to_tokimu(point, f64::from(height)))
                };
                triangles.push(DoomSurfaceTriangle {
                    source_subsector: region.source_subsector,
                    source_sector: ownership.source_sector,
                    plane,
                    texture_name: texture_name.to_owned(),
                    positions,
                });
            }
        }
    }
    Ok(triangles)
}

#[derive(Clone, Copy, Debug)]
struct DoomSectorBoundaryEdge {
    source_linedef: DoomSourceRecord,
    start_vertex: u16,
    end_vertex: u16,
    start: [i16; 2],
    end: [i16; 2],
}

fn resolve_doom_sector_boundary_support(
    map: &DoomMapCore,
) -> Vec<Option<Vec<DoomSectorBoundaryEdge>>> {
    let mut sector_edges = vec![Vec::new(); map.sectors.len()];
    for linedef in &map.linedefs {
        let right_sector = linedef
            .right_sidedef
            .and_then(|index| map.sidedefs.get(usize::from(index)))
            .map(|sidedef| sidedef.sector);
        let left_sector = linedef
            .left_sidedef
            .and_then(|index| map.sidedefs.get(usize::from(index)))
            .map(|sidedef| sidedef.sector);
        if right_sector == left_sector {
            continue;
        }
        let Some(start) = map.vertices.get(usize::from(linedef.start_vertex)) else {
            continue;
        };
        let Some(end) = map.vertices.get(usize::from(linedef.end_vertex)) else {
            continue;
        };
        if let Some(sector) =
            right_sector.and_then(|index| sector_edges.get_mut(usize::from(index)))
        {
            sector.push(DoomSectorBoundaryEdge {
                source_linedef: linedef.source,
                start_vertex: linedef.start_vertex,
                end_vertex: linedef.end_vertex,
                start: [start.x, start.y],
                end: [end.x, end.y],
            });
        }
        if let Some(sector) = left_sector.and_then(|index| sector_edges.get_mut(usize::from(index)))
        {
            sector.push(DoomSectorBoundaryEdge {
                source_linedef: linedef.source,
                start_vertex: linedef.end_vertex,
                end_vertex: linedef.start_vertex,
                start: [end.x, end.y],
                end: [start.x, start.y],
            });
        }
    }

    sector_edges
        .into_iter()
        .map(|edges| {
            if edges.len() < 3 {
                return None;
            }
            let mut degrees = BTreeMap::<u16, [usize; 2]>::new();
            for edge in &edges {
                degrees.entry(edge.start_vertex).or_default()[0] += 1;
                degrees.entry(edge.end_vertex).or_default()[1] += 1;
            }
            degrees
                .values()
                .all(|degree| degree[0] == degree[1])
                .then_some(edges)
        })
        .collect()
}

fn refine_convex_region_to_sector_boundary(
    vertices: &[[f64; 2]],
    edges: &[DoomSectorBoundaryEdge],
) -> Option<Vec<Vec<[f64; 2]>>> {
    const MAXIMUM_FRAGMENTS_PER_SUBSECTOR: usize = 4096;
    const AREA_EPSILON: f64 = 1.0e-9;
    let mut fragments = vec![vertices.to_vec()];
    for edge in edges {
        let mut split_fragments = Vec::with_capacity(fragments.len());
        for fragment in fragments {
            if !sector_edge_intersects_convex_region(edge, &fragment) {
                split_fragments.push(fragment);
                continue;
            }
            let step = DoomBspPathStep {
                source_node: edge.source_linedef,
                side: DoomBspSide::Right,
                origin: edge.start,
                delta: [
                    edge.end[0].checked_sub(edge.start[0])?,
                    edge.end[1].checked_sub(edge.start[1])?,
                ],
            };
            let distances = fragment
                .iter()
                .map(|point| partition_distance(*point, &step));
            let (mut positive, mut negative) = (false, false);
            for distance in distances {
                positive |= distance > 1.0e-7;
                negative |= distance < -1.0e-7;
            }
            if !(positive && negative) {
                split_fragments.push(fragment);
                continue;
            }
            for side in [DoomBspSide::Right, DoomBspSide::Left] {
                let piece = clip_convex_region(&fragment, &DoomBspPathStep { side, ..step });
                if piece.len() >= 3 && polygon_signed_area(&piece).abs() > AREA_EPSILON {
                    split_fragments.push(piece);
                }
            }
        }
        if split_fragments.len() > MAXIMUM_FRAGMENTS_PER_SUBSECTOR {
            return None;
        }
        fragments = split_fragments;
    }
    Some(
        fragments
            .into_iter()
            .filter(|fragment| {
                let inverse_count = 1.0 / fragment.len() as f64;
                let centroid = fragment.iter().fold([0.0, 0.0], |sum, point| {
                    [sum[0] + point[0], sum[1] + point[1]]
                });
                point_in_directed_sector_boundary(
                    [centroid[0] * inverse_count, centroid[1] * inverse_count],
                    edges,
                )
            })
            .collect(),
    )
}

fn sector_edge_intersects_convex_region(
    edge: &DoomSectorBoundaryEdge,
    vertices: &[[f64; 2]],
) -> bool {
    let start = edge.start.map(f64::from);
    let end = edge.end.map(f64::from);
    point_in_convex_polygon(start, vertices)
        || point_in_convex_polygon(end, vertices)
        || vertices
            .iter()
            .copied()
            .zip(vertices.iter().copied().cycle().skip(1))
            .take(vertices.len())
            .any(|(polygon_start, polygon_end)| {
                closed_segments_intersect(start, end, polygon_start, polygon_end)
            })
}

fn point_in_convex_polygon(point: [f64; 2], vertices: &[[f64; 2]]) -> bool {
    let mut positive = false;
    let mut negative = false;
    for (start, end) in vertices
        .iter()
        .copied()
        .zip(vertices.iter().copied().cycle().skip(1))
        .take(vertices.len())
    {
        let cross = cross_2d(start, end, point);
        positive |= cross > 1.0e-7;
        negative |= cross < -1.0e-7;
        if positive && negative {
            return false;
        }
    }
    true
}

fn closed_segments_intersect(
    first_start: [f64; 2],
    first_end: [f64; 2],
    second_start: [f64; 2],
    second_end: [f64; 2],
) -> bool {
    let first_a = cross_2d(first_start, first_end, second_start);
    let first_b = cross_2d(first_start, first_end, second_end);
    let second_a = cross_2d(second_start, second_end, first_start);
    let second_b = cross_2d(second_start, second_end, first_end);
    (first_a.abs() <= 1.0e-7 && point_on_closed_segment(second_start, first_start, first_end))
        || (first_b.abs() <= 1.0e-7 && point_on_closed_segment(second_end, first_start, first_end))
        || (second_a.abs() <= 1.0e-7
            && point_on_closed_segment(first_start, second_start, second_end))
        || (second_b.abs() <= 1.0e-7
            && point_on_closed_segment(first_end, second_start, second_end))
        || ((first_a > 0.0) != (first_b > 0.0) && (second_a > 0.0) != (second_b > 0.0))
}

fn point_in_directed_sector_boundary(point: [f64; 2], edges: &[DoomSectorBoundaryEdge]) -> bool {
    let mut winding = 0_i32;
    for edge in edges {
        let start = edge.start.map(f64::from);
        let end = edge.end.map(f64::from);
        if cross_2d(start, end, point).abs() <= 1.0e-7 && point_on_closed_segment(point, start, end)
        {
            return true;
        }
        if start[1] <= point[1] {
            if end[1] > point[1] && cross_2d(start, end, point) > 1.0e-7 {
                winding += 1;
            }
        } else if end[1] <= point[1] && cross_2d(start, end, point) < -1.0e-7 {
            winding -= 1;
        }
    }
    winding != 0
}

fn cross_2d(start: [f64; 2], end: [f64; 2], point: [f64; 2]) -> f64 {
    (end[0] - start[0]) * (point[1] - start[1]) - (end[1] - start[1]) * (point[0] - start[0])
}

fn point_on_closed_segment(point: [f64; 2], start: [f64; 2], end: [f64; 2]) -> bool {
    point[0] >= start[0].min(end[0]) - 1.0e-7
        && point[0] <= start[0].max(end[0]) + 1.0e-7
        && point[1] >= start[1].min(end[1]) - 1.0e-7
        && point[1] <= start[1].max(end[1]) + 1.0e-7
}

/// Lowers Doom floor and ceiling surfaces from the narrowest validated source
/// boundary available for each subsector.
///
/// A subsector's `SEGS` may be stored in an order or direction other than
/// boundary order. This bake therefore joins decoded endpoints by identity.
/// A joined cycle is authoritative only when it consumes every SEG exactly
/// once, is convex, and remains inside the BSP path for that leaf. Otherwise
/// the existing BSP-path region is retained. This is finite surface-support
/// recovery, not visibility, reachability, or source BSP pruning.
pub fn lower_doom_source_bounded_subsector_surfaces(
    map: &DoomMapCore,
    paths: &[DoomSubsectorBspPath],
) -> Result<DoomSourceBoundedSurfaceBake, DoomGeometryError> {
    lower_doom_bounded_subsector_surfaces(map, paths, false)
}

/// Extends [`lower_doom_source_bounded_subsector_surfaces`] with a complete
/// directed LINEDEF/SIDEDEF boundary graph for each sector. This candidate can
/// trim concave shells and holes which are not recoverable from one leaf's
/// local SEGs. Only balanced closed sector graphs participate; every
/// unavailable or empty result fails open to the local source/BSP region.
pub fn lower_doom_sector_bounded_subsector_surfaces(
    map: &DoomMapCore,
    paths: &[DoomSubsectorBspPath],
) -> Result<DoomSourceBoundedSurfaceBake, DoomGeometryError> {
    lower_doom_bounded_subsector_surfaces(map, paths, true)
}

fn lower_doom_bounded_subsector_surfaces(
    map: &DoomMapCore,
    paths: &[DoomSubsectorBspPath],
    sector_boundary_trim: bool,
) -> Result<DoomSourceBoundedSurfaceBake, DoomGeometryError> {
    const AREA_EPSILON: f64 = 1.0e-9;

    let regions = resolve_doom_subsector_regions(map, paths)?;
    let ownership = resolve_doom_subsector_sector_ownership(map)?;
    let sector_boundaries = sector_boundary_trim.then(|| resolve_doom_sector_boundary_support(map));
    let mut surfaces = Vec::new();
    let mut stitched_seg_loops = 0;
    let mut stitched_loop_refinements = 0;
    let mut seg_half_plane_regions = 0;
    let mut seg_half_plane_refinements = 0;
    let mut bsp_path_fallbacks = 0;
    let mut bsp_path_fallback_subsectors = Vec::new();
    let mut sector_boundary_supported_subsectors = 0;
    let mut sector_boundary_refinements = 0;
    let mut sector_boundary_fragments = 0;
    let mut sector_boundary_omissions = 0;
    let mut sector_boundary_omission_subsectors = Vec::new();
    let mut sector_boundary_unavailable_subsectors = Vec::new();
    let mut degenerate_region_omissions = 0;
    let mut degenerate_region_subsectors = Vec::new();

    for (subsector_index, ((subsector, region), ownership)) in map
        .subsectors
        .iter()
        .zip(&regions)
        .zip(&ownership)
        .enumerate()
    {
        let path = &paths[subsector_index];
        let stitched = stitch_subsector_seg_loop(map, subsector).filter(|vertices| {
            polygon_signed_area(vertices).abs() > AREA_EPSILON
                && is_convex_polygon(vertices)
                && vertices.iter().all(|point| {
                    path.steps.iter().all(|step| {
                        is_inside_partition(partition_distance(*point, step), step.side, 1.0e-9)
                    })
                })
        });
        let vertices = if let Some(vertices) = stitched {
            stitched_seg_loops += 1;
            if (polygon_signed_area(&vertices).abs() - polygon_signed_area(&region.vertices).abs())
                .abs()
                > AREA_EPSILON
            {
                stitched_loop_refinements += 1;
            }
            vertices
        } else if let Some(vertices) =
            clip_subsector_region_to_seg_half_planes(map, subsector, &region.vertices)
        {
            seg_half_plane_regions += 1;
            if (polygon_signed_area(&vertices).abs() - polygon_signed_area(&region.vertices).abs())
                .abs()
                > AREA_EPSILON
            {
                seg_half_plane_refinements += 1;
            }
            vertices
        } else {
            bsp_path_fallbacks += 1;
            bsp_path_fallback_subsectors.push(region.source_subsector);
            region.vertices.clone()
        };

        if polygon_signed_area(&vertices).abs() <= AREA_EPSILON {
            degenerate_region_omissions += 1;
            degenerate_region_subsectors.push(region.source_subsector);
            continue;
        }
        let original_area = polygon_signed_area(&vertices).abs();
        let refined_regions = sector_boundaries.as_ref().map(|boundaries| {
            boundaries
                .get(usize::from(ownership.sector_index))
                .and_then(Option::as_deref)
                .and_then(|edges| refine_convex_region_to_sector_boundary(&vertices, edges))
        });
        let prepared_regions = match refined_regions {
            Some(Some(regions)) if !regions.is_empty() => {
                sector_boundary_supported_subsectors += 1;
                sector_boundary_fragments += regions.len();
                let refined_area = regions
                    .iter()
                    .map(|region| polygon_signed_area(region).abs())
                    .sum::<f64>();
                if (refined_area - original_area).abs() > AREA_EPSILON || regions.len() > 1 {
                    sector_boundary_refinements += 1;
                }
                regions
            }
            Some(Some(_)) => {
                // A complete disappearance is too strong to install from a
                // derived sector graph. Retain the local source/BSP result and
                // expose the disagreement for corpus review.
                sector_boundary_omissions += 1;
                sector_boundary_omission_subsectors.push(region.source_subsector);
                vec![vertices]
            }
            Some(None) => {
                sector_boundary_unavailable_subsectors.push(region.source_subsector);
                vec![vertices]
            }
            None => vec![vertices],
        };
        for prepared_region in prepared_regions {
            append_subsector_surface_triangles(
                &mut surfaces,
                &prepared_region,
                region.source_subsector,
                ownership,
                &map.sectors[usize::from(ownership.sector_index)],
            );
        }
    }

    Ok(DoomSourceBoundedSurfaceBake {
        audit: DoomSourceBoundedSurfaceAudit {
            subsectors: map.subsectors.len(),
            stitched_seg_loops,
            stitched_loop_refinements,
            seg_half_plane_regions,
            seg_half_plane_refinements,
            bsp_path_fallbacks,
            bsp_path_fallback_subsectors,
            sector_boundary_supported_subsectors,
            sector_boundary_refinements,
            sector_boundary_fragments,
            sector_boundary_omissions,
            sector_boundary_omission_subsectors,
            sector_boundary_unavailable_subsectors,
            degenerate_region_omissions,
            degenerate_region_subsectors,
            surface_triangles: surfaces.len(),
        },
        surfaces,
    })
}

fn clip_subsector_region_to_seg_half_planes(
    map: &DoomMapCore,
    subsector: &doom_map_provider::DoomSubsector,
    bsp_region: &[[f64; 2]],
) -> Option<Vec<[f64; 2]>> {
    const NODE_BUILDER_VERTEX_TOLERANCE: f64 = 1.0;

    let first = usize::from(subsector.first_seg);
    let end = first.checked_add(usize::from(subsector.seg_count))?;
    let segs = map.segs.get(first..end)?;
    if segs.is_empty() {
        return None;
    }

    // Decoded Doom SEGs face along their stored start/end direction with the
    // owning subsector on the right. Their supporting lines can therefore
    // complete a leaf whose other edges exist only in its BSP path. The
    // finite BSP region remains the initial domain, so these constraints can
    // shrink but never expand the established leaf.
    let mut steps = Vec::with_capacity(segs.len());
    for seg in segs {
        let start = point_for_vertex(map, seg.start_vertex);
        let end = point_for_vertex(map, seg.end_vertex);
        steps.push(DoomBspPathStep {
            source_node: seg.source,
            side: DoomBspSide::Right,
            origin: start,
            delta: [end[0].checked_sub(start[0])?, end[1].checked_sub(start[1])?],
        });
    }

    // Reject contradictory orientation instead of allowing clipping order to
    // manufacture a plausible polygon. Doom node builders store split
    // vertices at integer coordinates, so two supporting lines which meet at
    // the same conceptual leaf corner can miss by less than one map unit.
    // Admit only that bounded perpendicular quantization error; the exact
    // half-planes still clip the result inside the finite BSP-path region.
    if segs.iter().any(|seg| {
        [seg.start_vertex, seg.end_vertex]
            .into_iter()
            .map(|vertex| point_for_vertex(map, vertex).map(f64::from))
            .any(|point| {
                steps.iter().any(|step| {
                    let length = f64::from(step.delta[0]).hypot(f64::from(step.delta[1]));
                    !is_inside_partition(
                        partition_distance(point, step),
                        step.side,
                        length * NODE_BUILDER_VERTEX_TOLERANCE + 1.0e-7,
                    )
                })
            })
    }) {
        return None;
    }

    let mut vertices = bsp_region.to_vec();
    for step in &steps {
        vertices = clip_convex_region(&vertices, step);
        if vertices.len() < 3 {
            return None;
        }
    }
    (polygon_signed_area(&vertices).abs() > 1.0e-9 && is_convex_polygon(&vertices))
        .then_some(vertices)
}

fn stitch_subsector_seg_loop(
    map: &DoomMapCore,
    subsector: &doom_map_provider::DoomSubsector,
) -> Option<Vec<[f64; 2]>> {
    let first = usize::from(subsector.first_seg);
    let end = first.checked_add(usize::from(subsector.seg_count))?;
    let segs = map.segs.get(first..end)?;
    if segs.len() < 3 {
        return None;
    }

    let mut adjacency = BTreeMap::<u16, Vec<(u16, usize)>>::new();
    for (edge_index, seg) in segs.iter().enumerate() {
        if seg.start_vertex == seg.end_vertex {
            return None;
        }
        adjacency
            .entry(seg.start_vertex)
            .or_default()
            .push((seg.end_vertex, edge_index));
        adjacency
            .entry(seg.end_vertex)
            .or_default()
            .push((seg.start_vertex, edge_index));
    }
    if adjacency.len() < 3 || adjacency.values().any(|neighbors| neighbors.len() != 2) {
        return None;
    }

    let start = *adjacency.keys().next()?;
    let mut current = start;
    let mut used_edges = BTreeSet::new();
    let mut vertices = Vec::with_capacity(adjacency.len());
    loop {
        let vertex = map.vertices.get(usize::from(current))?;
        vertices.push([f64::from(vertex.x), f64::from(vertex.y)]);
        let mut candidates = adjacency
            .get(&current)?
            .iter()
            .copied()
            .filter(|(_, edge_index)| !used_edges.contains(edge_index))
            .collect::<Vec<_>>();
        candidates.sort_unstable();
        if vertices.len() > 1 && candidates.len() != 1 {
            return None;
        }
        let (next, edge_index) = *candidates.first()?;
        used_edges.insert(edge_index);
        current = next;
        if current == start {
            return (used_edges.len() == segs.len() && vertices.len() == adjacency.len())
                .then_some(vertices);
        }
    }
}

fn is_convex_polygon(vertices: &[[f64; 2]]) -> bool {
    const EPSILON: f64 = 1.0e-9;
    if vertices.len() < 3 {
        return false;
    }
    let mut sign = 0.0_f64;
    for index in 0..vertices.len() {
        let a = vertices[index];
        let b = vertices[(index + 1) % vertices.len()];
        let c = vertices[(index + 2) % vertices.len()];
        let cross = (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0]);
        if cross.abs() <= EPSILON {
            continue;
        }
        if sign != 0.0 && cross.signum() != sign.signum() {
            return false;
        }
        sign = cross;
    }
    sign != 0.0
}

fn append_subsector_surface_triangles(
    triangles: &mut Vec<DoomSurfaceTriangle>,
    vertices: &[[f64; 2]],
    source_subsector: DoomSourceRecord,
    ownership: &DoomSubsectorSectorOwnership,
    sector: &DoomSector,
) {
    let counter_clockwise = polygon_signed_area(vertices) > 0.0;
    for (plane, height, texture_name, face_up) in [
        (
            DoomSurfacePlane::Floor,
            sector.floor_height,
            sector.floor_texture.as_str(),
            true,
        ),
        (
            DoomSurfacePlane::Ceiling,
            sector.ceiling_height,
            sector.ceiling_texture.as_str(),
            false,
        ),
    ] {
        for index in 1..vertices.len() - 1 {
            let points = [vertices[0], vertices[index], vertices[index + 1]];
            let reverse = face_up == counter_clockwise;
            let positions = if reverse {
                [
                    doom_point_to_tokimu(points[0], f64::from(height)),
                    doom_point_to_tokimu(points[2], f64::from(height)),
                    doom_point_to_tokimu(points[1], f64::from(height)),
                ]
            } else {
                points.map(|point| doom_point_to_tokimu(point, f64::from(height)))
            };
            triangles.push(DoomSurfaceTriangle {
                source_subsector,
                source_sector: ownership.source_sector,
                plane,
                texture_name: texture_name.to_owned(),
                positions,
            });
        }
    }
}

/// Lowers one-sided walls as full-height, untextured triangle candidates.
///
/// This admits neither middle/upper/lower texture semantics nor two-sided
/// openings. Those require separate source rules.
pub fn lower_doom_one_sided_walls(
    map: &DoomMapCore,
) -> Result<Vec<DoomWallTriangle>, DoomGeometryError> {
    let candidates = resolve_doom_wall_candidates(map)?;
    let mut triangles = Vec::new();
    for candidate in candidates {
        let (side, ownership) = match (candidate.right.as_ref(), candidate.left.as_ref()) {
            (Some(right), None) => (DoomWallSideKind::Right, right),
            (None, Some(left)) => (DoomWallSideKind::Left, left),
            _ => continue,
        };
        let sector = &map.sectors[usize::from(ownership.sector_index)];
        let source_start = candidate.start.map(f64::from);
        let source_end = candidate.end.map(f64::from);
        let start_floor = doom_point_to_tokimu(source_start, f64::from(sector.floor_height));
        let end_floor = doom_point_to_tokimu(source_end, f64::from(sector.floor_height));
        let start_ceiling = doom_point_to_tokimu(source_start, f64::from(sector.ceiling_height));
        let end_ceiling = doom_point_to_tokimu(source_end, f64::from(sector.ceiling_height));
        let positions =
            doom_wall_quad_triangles(side, start_floor, end_floor, start_ceiling, end_ceiling);
        triangles.extend(positions.map(|positions| DoomWallTriangle {
            source_linedef: candidate.source_linedef,
            source_sidedef: ownership.source_sidedef,
            source_sector: ownership.source_sector,
            side,
            positions,
        }));
    }
    Ok(triangles)
}

/// Lowers upper and lower height discontinuities on two-sided walls.
///
/// The source texture name is retained but not interpreted as a material or
/// UV mapping. The Doom-owned `F_SKY1` adjacency rule omits upper bands between
/// two sky ceilings; middle textures, pegging, and portals remain separate.
pub fn lower_doom_two_sided_wall_bands(
    map: &DoomMapCore,
) -> Result<Vec<DoomTwoSidedWallTriangle>, DoomGeometryError> {
    let candidates = resolve_doom_wall_candidates(map)?;
    let mut triangles = Vec::new();
    for candidate in candidates {
        let (Some(right), Some(left)) = (candidate.right.as_ref(), candidate.left.as_ref()) else {
            continue;
        };
        let right_sector = &map.sectors[usize::from(right.sector_index)];
        let left_sector = &map.sectors[usize::from(left.sector_index)];
        // Classic Doom treats adjacent F_SKY1 ceilings as one continuous sky
        // opening. The geometric height discontinuity still exists in the
        // source sectors, but its upper wall texture is not presented. Keep
        // this source-format rule here rather than asking generic geometry,
        // material, or visibility consumers to recognize Doom sky names.
        let suppress_upper_band =
            right_sector.ceiling_texture == "F_SKY1" && left_sector.ceiling_texture == "F_SKY1";
        if !suppress_upper_band && right_sector.ceiling_height > left_sector.ceiling_height {
            append_two_sided_band(
                &mut triangles,
                &candidate,
                right,
                DoomWallBandRequest {
                    side: DoomWallSideKind::Right,
                    band: DoomWallBand::Upper,
                    bottom: left_sector.ceiling_height,
                    top: right_sector.ceiling_height,
                    texture_name: &right.upper_texture,
                },
            );
        }
        if !suppress_upper_band && left_sector.ceiling_height > right_sector.ceiling_height {
            append_two_sided_band(
                &mut triangles,
                &candidate,
                left,
                DoomWallBandRequest {
                    side: DoomWallSideKind::Left,
                    band: DoomWallBand::Upper,
                    bottom: right_sector.ceiling_height,
                    top: left_sector.ceiling_height,
                    texture_name: &left.upper_texture,
                },
            );
        }
        if right_sector.floor_height < left_sector.floor_height {
            append_two_sided_band(
                &mut triangles,
                &candidate,
                right,
                DoomWallBandRequest {
                    side: DoomWallSideKind::Right,
                    band: DoomWallBand::Lower,
                    bottom: right_sector.floor_height,
                    top: left_sector.floor_height,
                    texture_name: &right.lower_texture,
                },
            );
        }
        if left_sector.floor_height < right_sector.floor_height {
            append_two_sided_band(
                &mut triangles,
                &candidate,
                left,
                DoomWallBandRequest {
                    side: DoomWallSideKind::Left,
                    band: DoomWallBand::Lower,
                    bottom: left_sector.floor_height,
                    top: right_sector.floor_height,
                    texture_name: &left.lower_texture,
                },
            );
        }
    }
    Ok(triangles)
}

/// Retains the non-colored depth boundary implied by two adjacent `F_SKY1`
/// ceilings whose heights differ.
///
/// This does not restore the omitted upper wall as visible geometry. It gives
/// a Doom presentation consumer the exact source span needed to keep the sky
/// aperture in front of unrelated farther map geometry.
pub fn lower_doom_paired_sky_boundary_triangles(
    map: &DoomMapCore,
) -> Result<Vec<DoomPairedSkyBoundaryTriangle>, DoomGeometryError> {
    let candidates = resolve_doom_wall_candidates(map)?;
    let mut triangles = Vec::new();
    for candidate in candidates {
        let (Some(right), Some(left)) = (candidate.right.as_ref(), candidate.left.as_ref()) else {
            continue;
        };
        let right_sector = &map.sectors[usize::from(right.sector_index)];
        let left_sector = &map.sectors[usize::from(left.sector_index)];
        if right_sector.ceiling_texture != "F_SKY1"
            || left_sector.ceiling_texture != "F_SKY1"
            || right_sector.ceiling_height == left_sector.ceiling_height
        {
            continue;
        }

        let (ownership, side, bottom, top) =
            if right_sector.ceiling_height > left_sector.ceiling_height {
                (
                    right,
                    DoomWallSideKind::Right,
                    left_sector.ceiling_height,
                    right_sector.ceiling_height,
                )
            } else {
                (
                    left,
                    DoomWallSideKind::Left,
                    right_sector.ceiling_height,
                    left_sector.ceiling_height,
                )
            };
        let source_start = candidate.start.map(f64::from);
        let source_end = candidate.end.map(f64::from);
        let start_bottom = doom_point_to_tokimu(source_start, f64::from(bottom));
        let end_bottom = doom_point_to_tokimu(source_end, f64::from(bottom));
        let start_top = doom_point_to_tokimu(source_start, f64::from(top));
        let end_top = doom_point_to_tokimu(source_end, f64::from(top));
        triangles.extend(
            doom_wall_quad_triangles(side, start_bottom, end_bottom, start_top, end_top).map(
                |positions| DoomPairedSkyBoundaryTriangle {
                    source_linedef: candidate.source_linedef,
                    source_sidedef: ownership.source_sidedef,
                    source_sector: ownership.source_sector,
                    side,
                    positions,
                },
            ),
        );
    }
    Ok(triangles)
}

/// Lowers authored two-sided middle textures into their positive shared
/// vertical opening. Closed or inverted openings produce no geometry. This is
/// a bounded clipping policy only; transparency and portal behavior remain at
/// the later material/presentation boundary.
pub fn lower_doom_two_sided_middle_walls(
    map: &DoomMapCore,
) -> Result<Vec<DoomTwoSidedMiddleWallTriangle>, DoomGeometryError> {
    let candidates = resolve_doom_wall_candidates(map)?;
    let mut triangles = Vec::new();
    for candidate in candidates {
        let (Some(right), Some(left)) = (candidate.right.as_ref(), candidate.left.as_ref()) else {
            continue;
        };
        let right_sector = &map.sectors[usize::from(right.sector_index)];
        let left_sector = &map.sectors[usize::from(left.sector_index)];
        let opening_floor = right_sector.floor_height.max(left_sector.floor_height);
        let opening_ceiling = right_sector.ceiling_height.min(left_sector.ceiling_height);
        if opening_floor >= opening_ceiling {
            continue;
        }
        for (side, ownership) in [
            (DoomWallSideKind::Right, right),
            (DoomWallSideKind::Left, left),
        ] {
            if ownership.middle_texture == "-" {
                continue;
            }
            append_two_sided_middle_wall(
                &mut triangles,
                &candidate,
                ownership,
                side,
                opening_floor,
                opening_ceiling,
            );
        }
    }
    Ok(triangles)
}

/// Lowers the currently admitted ordinary wall geometry with stable Doom
/// texture-space coordinates. Coordinates remain in source texels: no
/// renderer-specific normalization, wrapping, or material object is chosen.
///
/// Two-sided middle walls use their explicitly admitted shared-opening clip.
/// No material object, alpha mode, wrapping, or portal behavior is chosen.
pub fn lower_doom_textured_wall_triangles(
    map: &DoomMapCore,
    extents: &[DoomTextureExtent],
) -> Result<Vec<DoomTexturedWallTriangle>, DoomGeometryError> {
    let candidates = resolve_doom_wall_candidates(map)?;
    let placements = resolve_doom_wall_texture_placements(map, extents)?;
    let mut triangles = Vec::new();

    for triangle in lower_doom_one_sided_walls(map)? {
        append_textured_wall_triangle(
            &mut triangles,
            &candidates,
            &placements,
            triangle.source_linedef,
            triangle.source_sidedef,
            triangle.source_sector,
            triangle.side,
            DoomWallTextureRole::Middle,
            triangle.positions,
        );
    }
    for triangle in lower_doom_two_sided_wall_bands(map)? {
        let role = match triangle.band {
            DoomWallBand::Upper => DoomWallTextureRole::Upper,
            DoomWallBand::Lower => DoomWallTextureRole::Lower,
        };
        append_textured_wall_triangle(
            &mut triangles,
            &candidates,
            &placements,
            triangle.source_linedef,
            triangle.source_sidedef,
            triangle.source_sector,
            triangle.side,
            role,
            triangle.positions,
        );
    }
    for triangle in lower_doom_two_sided_middle_walls(map)? {
        append_textured_wall_triangle(
            &mut triangles,
            &candidates,
            &placements,
            triangle.source_linedef,
            triangle.source_sidedef,
            triangle.source_sector,
            triangle.side,
            DoomWallTextureRole::Middle,
            triangle.positions,
        );
    }
    Ok(triangles)
}

/// Re-expresses the existing textured-wall lowering at the source `SEG`
/// granularity retained by Doom's BSP. This is deliberately a corpus-only
/// representation experiment: it neither changes whole-linedef lowering nor
/// grants generic geometry or render code knowledge of `SEG`s.
///
/// Each emitted triangle keeps the original linedef/sidedef/side identity and
/// interpolates the already-resolved source-texel coordinates at the SEG
/// endpoints. Consequently a texture continues across adjacent SEG splits
/// instead of restarting at each BSP fragment.
pub fn lower_doom_seg_textured_wall_triangles(
    map: &DoomMapCore,
    extents: &[DoomTextureExtent],
) -> Result<Vec<DoomSegTexturedWallTriangle>, DoomGeometryError> {
    let whole_walls = lower_doom_textured_wall_triangles(map, extents)?;
    let candidates = resolve_doom_wall_candidates(map)?;
    let mut triangles = Vec::new();

    for seg in &map.segs {
        let side = match seg.direction {
            0 => DoomWallSideKind::Right,
            1 => DoomWallSideKind::Left,
            direction => {
                return Err(DoomGeometryError::UnsupportedSegDirection {
                    seg_index: seg.source.record_index,
                    direction,
                });
            }
        };
        let linedef = &map.linedefs[usize::from(seg.linedef)];
        let candidate = candidates
            .iter()
            .find(|candidate| candidate.source_linedef == linedef.source)
            .expect("validated SEG linedef has a resolved wall candidate");
        let start = &map.vertices[usize::from(seg.start_vertex)];
        let end = &map.vertices[usize::from(seg.end_vertex)];
        let interval = seg_interval_on_linedef(candidate, [start.x, start.y], [end.x, end.y]);

        for wall in whole_walls
            .iter()
            .filter(|wall| wall.source_linedef == linedef.source && wall.side == side)
        {
            for clipped in clip_textured_triangle_to_linedef_interval(
                wall.positions,
                wall.texture_coordinates,
                interval,
                candidate,
            ) {
                triangles.push(DoomSegTexturedWallTriangle {
                    source_seg: seg.source,
                    source_linedef: wall.source_linedef,
                    source_sidedef: wall.source_sidedef,
                    source_sector: wall.source_sector,
                    side: wall.side,
                    role: wall.role,
                    texture_name: wall.texture_name.clone(),
                    positions: clipped.0,
                    texture_coordinates: clipped.1,
                });
            }
        }
    }
    Ok(triangles)
}

/// Clips one already-lowered source `SEG` wall triangle to a subinterval of
/// its owning linedef. This stays in the Doom provider because the interval is
/// source-space evidence from the Stage 3B screen-span experiment, not a
/// renderer clipping API.
///
/// Returned triangles retain their SEG, linedef, sidedef, role, and source
/// texel coordinates, so a presentation comparison can preserve identity and
/// texture phase while isolating only the observed source interval.
pub fn clip_doom_seg_textured_wall_triangle_to_linedef_interval(
    map: &DoomMapCore,
    triangle: &DoomSegTexturedWallTriangle,
    interval: [f64; 2],
) -> Result<Vec<DoomSegTexturedWallTriangle>, DoomGeometryError> {
    if !interval[0].is_finite()
        || !interval[1].is_finite()
        || interval[0] < 0.0
        || interval[1] > 1.0
        || interval[0] > interval[1]
    {
        return Err(DoomGeometryError::InvalidLinedefInterval);
    }
    let candidate = resolve_doom_wall_candidates(map)?
        .into_iter()
        .find(|candidate| candidate.source_linedef == triangle.source_linedef)
        .expect("validated SEG triangle retains a resolved wall candidate");
    Ok(clip_textured_triangle_to_linedef_interval(
        triangle.positions,
        triangle.texture_coordinates,
        interval,
        &candidate,
    )
    .into_iter()
    .map(
        |(positions, texture_coordinates)| DoomSegTexturedWallTriangle {
            source_seg: triangle.source_seg,
            source_linedef: triangle.source_linedef,
            source_sidedef: triangle.source_sidedef,
            source_sector: triangle.source_sector,
            side: triangle.side,
            role: triangle.role,
            texture_name: triangle.texture_name.clone(),
            positions,
            texture_coordinates,
        },
    )
    .collect())
}

/// Reconstructs the retained ordered wall cells as ordinary textured
/// triangles. This lets a corpus consumer test whether Doom-owned partial
/// coverage can be realized through the existing mesh renderer without
/// exporting columns, scissors, or visplane state as renderer vocabulary.
///
/// Cell boundaries are intersected with the owning source SEG and clamped to
/// its endpoints. The clamp is intentional: the diagnostic column containing
/// an endpoint extends half a cell beyond the projected finite segment.
pub fn reconstruct_doom_ordered_wall_fragments(
    map: &DoomMapCore,
    source_triangles: &[DoomSegTexturedWallTriangle],
    observation: &DoomSegClassicVerticalClipObservation,
    viewer: [i16; 2],
    heading: f64,
    eye_height: f64,
) -> DoomOrderedWallFragmentReconstruction {
    const COLUMNS: usize = 320;
    const ROWS: usize = 200;
    const HALF_HORIZONTAL_FOV: f64 = std::f64::consts::FRAC_PI_4;

    let half_vertical_fov = ((ROWS as f64 / COLUMNS as f64) * HALF_HORIZONTAL_FOV.tan()).atan();
    let forward = [heading.cos(), heading.sin()];
    let right = [-forward[1], forward[0]];
    let segs_by_record = map
        .segs
        .iter()
        .map(|seg| (seg.source.record_index, seg))
        .collect::<BTreeMap<_, _>>();
    let triangles_by_key = source_triangles.iter().fold(
        BTreeMap::<(u32, u8), Vec<&DoomSegTexturedWallTriangle>>::new(),
        |mut triangles, triangle| {
            triangles
                .entry((
                    triangle.source_seg.record_index,
                    wall_role_key(triangle.role),
                ))
                .or_default()
                .push(triangle);
            triangles
        },
    );
    let mut result = DoomOrderedWallFragmentReconstruction::default();

    for interval in &observation.ordered_wall_intervals {
        let Some([top, bottom]) = interval.retained_interval else {
            continue;
        };
        result.retained_cells += 1;
        let key = (interval.source_seg, wall_role_key(interval.role));
        let (Some(seg), Some(reference_triangles)) = (
            segs_by_record.get(&interval.source_seg),
            triangles_by_key.get(&key),
        ) else {
            result.unresolved_cells += 1;
            retain_fragment_sample(
                &mut result.samples,
                format!(
                    "seg={} linedef={} column={} role={:?} unresolved=source-identity",
                    interval.source_seg, interval.source_linedef, interval.column, interval.role
                ),
            );
            continue;
        };
        let start = &map.vertices[usize::from(seg.start_vertex)];
        let end = &map.vertices[usize::from(seg.end_vertex)];
        let source_start = [f64::from(start.x), f64::from(start.y)];
        let source_end = [f64::from(end.x), f64::from(end.y)];
        let ray_for_column_edge = |edge: usize| {
            let normalized = -1.0 + (edge as f64 / COLUMNS as f64) * 2.0;
            let local_angle = (normalized * HALF_HORIZONTAL_FOV.tan()).atan();
            (
                [
                    forward[0] * local_angle.cos() + right[0] * local_angle.sin(),
                    forward[1] * local_angle.cos() + right[1] * local_angle.sin(),
                ],
                local_angle.cos(),
            )
        };
        let (left_ray, left_forward_scale) = ray_for_column_edge(interval.column);
        let Some((left_source, left_radial_depth)) =
            source_ray_segment_intersection(viewer, left_ray, source_start, source_end)
        else {
            result.unresolved_cells += 1;
            retain_fragment_sample(
                &mut result.samples,
                format!(
                    "seg={} linedef={} column={} role={:?} unresolved=left-ray",
                    interval.source_seg, interval.source_linedef, interval.column, interval.role
                ),
            );
            continue;
        };
        let (right_ray, right_forward_scale) = ray_for_column_edge(interval.column + 1);
        let Some((right_source, right_radial_depth)) =
            source_ray_segment_intersection(viewer, right_ray, source_start, source_end)
        else {
            result.unresolved_cells += 1;
            retain_fragment_sample(
                &mut result.samples,
                format!(
                    "seg={} linedef={} column={} role={:?} unresolved=right-ray",
                    interval.source_seg, interval.source_linedef, interval.column, interval.role
                ),
            );
            continue;
        };
        let left_forward_depth = left_radial_depth * left_forward_scale;
        let right_forward_depth = right_radial_depth * right_forward_scale;

        let minimum = reference_triangles
            .iter()
            .flat_map(|triangle| triangle.positions)
            .map(|position| position[1])
            .fold(f64::INFINITY, f64::min);
        let maximum = reference_triangles
            .iter()
            .flat_map(|triangle| triangle.positions)
            .map(|position| position[1])
            .fold(f64::NEG_INFINITY, f64::max);
        if (maximum - minimum).abs() <= f64::EPSILON {
            result.degenerate_cells += 1;
            retain_fragment_sample(
                &mut result.samples,
                format!(
                    "seg={} linedef={} column={} role={:?} omitted=zero-height-source-tier",
                    interval.source_seg, interval.source_linedef, interval.column, interval.role
                ),
            );
            continue;
        }
        let height_for_row = |row: usize, depth: f64| {
            let normalized = 1.0 - (row as f64 / ROWS as f64) * 2.0;
            (eye_height + normalized * half_vertical_fov.tan() * depth).clamp(minimum, maximum)
        };
        let positions = [
            doom_point_to_tokimu(left_source, height_for_row(top, left_forward_depth)),
            doom_point_to_tokimu(right_source, height_for_row(top, right_forward_depth)),
            doom_point_to_tokimu(
                right_source,
                height_for_row(bottom + 1, right_forward_depth),
            ),
            doom_point_to_tokimu(left_source, height_for_row(bottom + 1, left_forward_depth)),
        ];
        let Some(texture_coordinates) = positions
            .iter()
            .map(|position| interpolate_wall_texture_coordinate(reference_triangles, *position))
            .collect::<Option<Vec<_>>>()
        else {
            result.unresolved_cells += 1;
            retain_fragment_sample(
                &mut result.samples,
                format!(
                    "seg={} linedef={} column={} role={:?} unresolved=texture-interpolation",
                    interval.source_seg, interval.source_linedef, interval.column, interval.role
                ),
            );
            continue;
        };
        let reference = reference_triangles[0];
        for indices in [[0, 1, 2], [0, 2, 3]] {
            let mut triangle_positions = indices.map(|index| positions[index]);
            let mut triangle_uvs = indices.map(|index| texture_coordinates[index]);
            if dot3(
                triangle_normal64(triangle_positions),
                triangle_normal64(reference.positions),
            ) < 0.0
            {
                triangle_positions.swap(1, 2);
                triangle_uvs.swap(1, 2);
            }
            result
                .reconstructed_triangles
                .push(DoomSegTexturedWallTriangle {
                    source_seg: reference.source_seg,
                    source_linedef: reference.source_linedef,
                    source_sidedef: reference.source_sidedef,
                    source_sector: reference.source_sector,
                    side: reference.side,
                    role: reference.role,
                    texture_name: reference.texture_name.clone(),
                    positions: triangle_positions,
                    texture_coordinates: triangle_uvs,
                });
        }
    }

    result
}

fn wall_role_key(role: DoomWallTextureRole) -> u8 {
    match role {
        DoomWallTextureRole::Upper => 0,
        DoomWallTextureRole::Lower => 1,
        DoomWallTextureRole::Middle => 2,
    }
}

fn retain_fragment_sample(samples: &mut Vec<String>, sample: String) {
    if samples.len() < 12 {
        samples.push(sample);
    }
}

fn source_ray_segment_intersection(
    viewer: [i16; 2],
    ray: [f64; 2],
    start: [f64; 2],
    end: [f64; 2],
) -> Option<([f64; 2], f64)> {
    let viewer = viewer.map(f64::from);
    let offset = [start[0] - viewer[0], start[1] - viewer[1]];
    let segment = [end[0] - start[0], end[1] - start[1]];
    let cross = |left: [f64; 2], right: [f64; 2]| left[0] * right[1] - left[1] * right[0];
    let denominator = cross(ray, segment);
    if denominator.abs() <= f64::EPSILON {
        return None;
    }
    let depth = cross(offset, segment) / denominator;
    let progression = (cross(offset, ray) / denominator).clamp(0.0, 1.0);
    (depth > 0.0).then_some((
        [
            start[0] + segment[0] * progression,
            start[1] + segment[1] * progression,
        ],
        depth,
    ))
}

fn interpolate_wall_texture_coordinate(
    triangles: &[&DoomSegTexturedWallTriangle],
    position: [f64; 3],
) -> Option<[f64; 2]> {
    triangles.iter().find_map(|triangle| {
        let origin = triangle.positions[0];
        let first = subtract3(triangle.positions[1], origin);
        let second = subtract3(triangle.positions[2], origin);
        let relative = subtract3(position, origin);
        let first_first = dot3(first, first);
        let first_second = dot3(first, second);
        let second_second = dot3(second, second);
        let relative_first = dot3(relative, first);
        let relative_second = dot3(relative, second);
        let denominator = first_first * second_second - first_second * first_second;
        if denominator.abs() <= f64::EPSILON {
            return None;
        }
        let first_weight =
            (relative_first * second_second - relative_second * first_second) / denominator;
        let second_weight =
            (relative_second * first_first - relative_first * first_second) / denominator;
        let origin_weight = 1.0 - first_weight - second_weight;
        const TOLERANCE: f64 = 1.0e-6;
        (origin_weight >= -TOLERANCE && first_weight >= -TOLERANCE && second_weight >= -TOLERANCE)
            .then(|| {
                [
                    origin_weight * triangle.texture_coordinates[0][0]
                        + first_weight * triangle.texture_coordinates[1][0]
                        + second_weight * triangle.texture_coordinates[2][0],
                    origin_weight * triangle.texture_coordinates[0][1]
                        + first_weight * triangle.texture_coordinates[1][1]
                        + second_weight * triangle.texture_coordinates[2][1],
                ]
            })
    })
}

fn subtract3(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn dot3(left: [f64; 3], right: [f64; 3]) -> f64 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn triangle_normal64(positions: [[f64; 3]; 3]) -> [f64; 3] {
    let first = subtract3(positions[1], positions[0]);
    let second = subtract3(positions[2], positions[0]);
    [
        first[1] * second[2] - first[2] * second[1],
        first[2] * second[0] - first[0] * second[2],
        first[0] * second[1] - first[1] * second[0],
    ]
}

/// Observes only the source sector-height conditions under which classic Doom
/// may close a screen interval for a SEG. It neither projects the SEG nor
/// writes any coverage state; masked middles and sky behavior remain separate
/// presentation rules.
pub fn observe_doom_seg_occluders(
    map: &DoomMapCore,
) -> Result<Vec<DoomSegOccluderObservation>, DoomGeometryError> {
    let candidates = resolve_doom_wall_candidates(map)?;
    map.segs
        .iter()
        .map(|seg| {
            let side = match seg.direction {
                0 => DoomWallSideKind::Right,
                1 => DoomWallSideKind::Left,
                direction => {
                    return Err(DoomGeometryError::UnsupportedSegDirection {
                        seg_index: seg.source.record_index,
                        direction,
                    });
                }
            };
            let linedef = &map.linedefs[usize::from(seg.linedef)];
            let candidate = candidates
                .iter()
                .find(|candidate| candidate.source_linedef == linedef.source)
                .expect("validated SEG linedef has a resolved wall candidate");
            let (front, back) = match side {
                DoomWallSideKind::Right => (candidate.right.as_ref(), candidate.left.as_ref()),
                DoomWallSideKind::Left => (candidate.left.as_ref(), candidate.right.as_ref()),
            };
            let front = front.expect("SEG direction names an existing owning side");
            let kind = match back {
                None => DoomSegOccluderKind::OneSided,
                Some(back) => {
                    let front_sector = &map.sectors[usize::from(front.sector_index)];
                    let back_sector = &map.sectors[usize::from(back.sector_index)];
                    if back_sector.floor_height >= back_sector.ceiling_height {
                        DoomSegOccluderKind::BackSectorClosed
                    } else if back_sector.floor_height >= front_sector.ceiling_height
                        || back_sector.ceiling_height <= front_sector.floor_height
                    {
                        DoomSegOccluderKind::OpeningClosed
                    } else {
                        DoomSegOccluderKind::Open
                    }
                }
            };
            Ok(DoomSegOccluderObservation {
                source_seg: seg.source,
                source_linedef: linedef.source,
                side,
                kind,
            })
        })
        .collect()
}

/// Observes the source sector relationships that let classic Doom mark floor
/// and ceiling planes while storing an admitted wall range. The caller supplies
/// the source-space viewer height. The result deliberately stops before any
/// screen projection, per-column clip state, or plane-span construction.
pub fn observe_doom_seg_plane_marks(
    map: &DoomMapCore,
    source_view_height: i16,
) -> Result<Vec<DoomSegPlaneMarkObservation>, DoomGeometryError> {
    let candidates = resolve_doom_wall_candidates(map)?;
    map.segs
        .iter()
        .map(|seg| {
            let side = match seg.direction {
                0 => DoomWallSideKind::Right,
                1 => DoomWallSideKind::Left,
                direction => {
                    return Err(DoomGeometryError::UnsupportedSegDirection {
                        seg_index: seg.source.record_index,
                        direction,
                    });
                }
            };
            let linedef = &map.linedefs[usize::from(seg.linedef)];
            let candidate = candidates
                .iter()
                .find(|candidate| candidate.source_linedef == linedef.source)
                .expect("validated SEG linedef has a resolved wall candidate");
            let (front, back) = match side {
                DoomWallSideKind::Right => (candidate.right.as_ref(), candidate.left.as_ref()),
                DoomWallSideKind::Left => (candidate.left.as_ref(), candidate.right.as_ref()),
            };
            let front = front.expect("SEG direction names an existing owning side");
            let front_sector = &map.sectors[usize::from(front.sector_index)];
            let back_sector = back.map(|back| &map.sectors[usize::from(back.sector_index)]);
            let paired_sky_ceiling_adjustment = back_sector.is_some_and(|back_sector| {
                front_sector.ceiling_texture == "F_SKY1" && back_sector.ceiling_texture == "F_SKY1"
            });

            let (mut floor_marked, mut ceiling_marked) = match back_sector {
                None => (true, true),
                Some(back_sector) => {
                    let effective_front_ceiling = if paired_sky_ceiling_adjustment {
                        back_sector.ceiling_height
                    } else {
                        front_sector.ceiling_height
                    };
                    let closed_opening = back_sector.ceiling_height <= front_sector.floor_height
                        || back_sector.floor_height >= front_sector.ceiling_height;
                    (
                        closed_opening
                            || back_sector.floor_height != front_sector.floor_height
                            || back_sector.floor_texture != front_sector.floor_texture
                            || back_sector.light_level != front_sector.light_level,
                        closed_opening
                            || back_sector.ceiling_height != effective_front_ceiling
                            || back_sector.ceiling_texture != front_sector.ceiling_texture
                            || back_sector.light_level != front_sector.light_level,
                    )
                }
            };
            if front_sector.floor_height >= source_view_height {
                floor_marked = false;
            }
            if front_sector.ceiling_height <= source_view_height
                && front_sector.ceiling_texture != "F_SKY1"
            {
                ceiling_marked = false;
            }
            Ok(DoomSegPlaneMarkObservation {
                source_seg: seg.source,
                source_linedef: linedef.source,
                side,
                front_sector: front.source_sector,
                back_sector: back.map(|back| back.source_sector),
                floor_marked,
                ceiling_marked,
                paired_sky_ceiling_adjustment,
            })
        })
        .collect()
}

fn seg_interval_on_linedef(
    candidate: &DoomWallCandidate,
    start: [i16; 2],
    end: [i16; 2],
) -> [f64; 2] {
    let progression = |point: [i16; 2]| {
        let delta_x = f64::from(candidate.end[0] - candidate.start[0]);
        let delta_z = f64::from(candidate.end[1] - candidate.start[1]);
        let length_squared = delta_x.mul_add(delta_x, delta_z * delta_z);
        ((f64::from(point[0] - candidate.start[0]) * delta_x)
            + (f64::from(point[1] - candidate.start[1]) * delta_z))
            / length_squared
    };
    let start = progression(start);
    let end = progression(end);
    [start.min(end), start.max(end)]
}

type TexturedTriangle = ([[f64; 3]; 3], [[f64; 2]; 3]);

fn clip_textured_triangle_to_linedef_interval(
    positions: [[f64; 3]; 3],
    coordinates: [[f64; 2]; 3],
    interval: [f64; 2],
    candidate: &DoomWallCandidate,
) -> Vec<TexturedTriangle> {
    let mut polygon = positions
        .into_iter()
        .zip(coordinates)
        .map(|(position, coordinate)| {
            (
                position,
                coordinate,
                linedef_progression(candidate, position),
            )
        })
        .collect::<Vec<_>>();
    polygon = clip_textured_polygon(&polygon, interval[0], true);
    polygon = clip_textured_polygon(&polygon, interval[1], false);
    if polygon.len() < 3 {
        return Vec::new();
    }
    (1..polygon.len() - 1)
        .map(|index| {
            let triangle = [polygon[0], polygon[index], polygon[index + 1]];
            (
                triangle.map(|vertex| vertex.0),
                triangle.map(|vertex| vertex.1),
            )
        })
        .collect()
}

fn clip_textured_polygon(
    polygon: &[([f64; 3], [f64; 2], f64)],
    boundary: f64,
    keep_greater: bool,
) -> Vec<([f64; 3], [f64; 2], f64)> {
    let mut output = Vec::new();
    for (previous, current) in polygon
        .iter()
        .zip(polygon.iter().cycle().skip(1))
        .take(polygon.len())
    {
        let previous_inside = if keep_greater {
            previous.2 >= boundary
        } else {
            previous.2 <= boundary
        };
        let current_inside = if keep_greater {
            current.2 >= boundary
        } else {
            current.2 <= boundary
        };
        if previous_inside != current_inside {
            let t = (boundary - previous.2) / (current.2 - previous.2);
            output.push((
                interpolate3(previous.0, current.0, t),
                interpolate2(previous.1, current.1, t),
                boundary,
            ));
        }
        if current_inside {
            output.push(*current);
        }
    }
    output
}

fn linedef_progression(candidate: &DoomWallCandidate, position: [f64; 3]) -> f64 {
    let delta_x = f64::from(candidate.end[0] - candidate.start[0]);
    let delta_z = f64::from(candidate.end[1] - candidate.start[1]);
    let length_squared = delta_x.mul_add(delta_x, delta_z * delta_z);
    ((position[0] - f64::from(candidate.start[0])) * delta_x
        + (position[2] - f64::from(candidate.start[1])) * delta_z)
        / length_squared
}

fn interpolate3(start: [f64; 3], end: [f64; 3], t: f64) -> [f64; 3] {
    [
        start[0] + (end[0] - start[0]) * t,
        start[1] + (end[1] - start[1]) * t,
        start[2] + (end[2] - start[2]) * t,
    ]
}

fn interpolate2(start: [f64; 2], end: [f64; 2], t: f64) -> [f64; 2] {
    [
        start[0] + (end[0] - start[0]) * t,
        start[1] + (end[1] - start[1]) * t,
    ]
}

#[allow(clippy::too_many_arguments)]
fn append_textured_wall_triangle(
    triangles: &mut Vec<DoomTexturedWallTriangle>,
    candidates: &[DoomWallCandidate],
    placements: &[DoomWallTexturePlacement],
    source_linedef: DoomSourceRecord,
    source_sidedef: DoomSourceRecord,
    source_sector: DoomSourceRecord,
    side: DoomWallSideKind,
    role: DoomWallTextureRole,
    positions: [[f64; 3]; 3],
) {
    let Some(placement) = placements.iter().find(|placement| {
        placement.binding.axis.source_linedef == source_linedef
            && placement.binding.axis.source_sidedef == source_sidedef
            && placement.binding.axis.side == side
            && placement.binding.axis.role == role
    }) else {
        return;
    };
    let candidate = candidates
        .iter()
        .find(|candidate| candidate.source_linedef == source_linedef)
        .expect("lowered wall triangle originates from a resolved candidate");
    triangles.push(DoomTexturedWallTriangle {
        source_linedef,
        source_sidedef,
        source_sector,
        side,
        role,
        texture_name: placement.binding.axis.texture_name.clone(),
        positions,
        texture_coordinates: positions
            .map(|position| doom_wall_texture_coordinate(candidate, placement, position)),
    });
}

fn doom_wall_texture_coordinate(
    candidate: &DoomWallCandidate,
    placement: &DoomWallTexturePlacement,
    position: [f64; 3],
) -> [f64; 2] {
    let start_x = f64::from(candidate.start[0]);
    let start_z = f64::from(candidate.start[1]);
    let delta_x = f64::from(candidate.end[0]) - f64::from(candidate.start[0]);
    let delta_z = f64::from(candidate.end[1]) - f64::from(candidate.start[1]);
    let length_squared = delta_x.mul_add(delta_x, delta_z * delta_z);
    let progression =
        ((position[0] - start_x) * delta_x + (position[2] - start_z) * delta_z) / length_squared;
    let u = placement.binding.axis.u_start
        + progression * (placement.binding.axis.u_end - placement.binding.axis.u_start);
    let v = f64::from(placement.texture_mid_y) - position[1];
    [u, v]
}

/// Reports authored middle textures on two-sided wall openings.
///
/// It deliberately does not create middle geometry: texture dimensions,
/// pegging, clipping, and transparency policy have not yet been admitted.
pub fn observe_doom_two_sided_middle_textures(
    map: &DoomMapCore,
) -> Result<Vec<DoomMiddleTextureObservation>, DoomGeometryError> {
    let candidates = resolve_doom_wall_candidates(map)?;
    let mut observations = Vec::new();
    for candidate in candidates {
        let (Some(right), Some(left)) = (candidate.right.as_ref(), candidate.left.as_ref()) else {
            continue;
        };
        let right_sector = &map.sectors[usize::from(right.sector_index)];
        let left_sector = &map.sectors[usize::from(left.sector_index)];
        let opening_floor = right_sector.floor_height.max(left_sector.floor_height);
        let opening_ceiling = right_sector.ceiling_height.min(left_sector.ceiling_height);
        if opening_floor >= opening_ceiling {
            continue;
        }
        for (side, ownership) in [
            (DoomWallSideKind::Right, right),
            (DoomWallSideKind::Left, left),
        ] {
            if ownership.middle_texture != "-" {
                observations.push(DoomMiddleTextureObservation {
                    source_linedef: candidate.source_linedef,
                    source_sidedef: ownership.source_sidedef,
                    source_sector: ownership.source_sector,
                    side,
                    texture_name: ownership.middle_texture.clone(),
                    opening_floor,
                    opening_ceiling,
                });
            }
        }
    }
    Ok(observations)
}

/// Observes classic Doom `F_SKY1` floor/ceiling source semantics without
/// assigning any renderer behavior to the generated surface triangles.
pub fn observe_doom_sky_surfaces(
    map: &DoomMapCore,
    paths: &[DoomSubsectorBspPath],
) -> Result<Vec<DoomSkySurfaceObservation>, DoomGeometryError> {
    let regions = resolve_doom_subsector_regions(map, paths)?;
    let ownership = resolve_doom_subsector_sector_ownership(map)?;
    let mut observations = Vec::new();
    for ((subsector, region), ownership) in map.subsectors.iter().zip(regions).zip(ownership) {
        let sector = &map.sectors[usize::from(ownership.sector_index)];
        for (plane, texture_name) in [
            (DoomSurfacePlane::Floor, sector.floor_texture.as_str()),
            (DoomSurfacePlane::Ceiling, sector.ceiling_texture.as_str()),
        ] {
            if texture_name == "F_SKY1" {
                for _ in 1..region.vertices.len().saturating_sub(1) {
                    observations.push(DoomSkySurfaceObservation {
                        source_subsector: subsector.source,
                        source_sector: ownership.source_sector,
                        plane,
                        texture_name: texture_name.to_owned(),
                    });
                }
            }
        }
    }
    Ok(observations)
}

/// Retains raw sidedef texture axes before any Doom pegging policy is applied.
pub fn observe_doom_wall_texture_axes(
    map: &DoomMapCore,
) -> Result<Vec<DoomWallTextureAxisObservation>, DoomGeometryError> {
    let candidates = resolve_doom_wall_candidates(map)?;
    let mut observations = Vec::new();
    for candidate in candidates {
        let two_sided = candidate.right.is_some() && candidate.left.is_some();
        for (side, ownership) in [
            (DoomWallSideKind::Right, candidate.right.as_ref()),
            (DoomWallSideKind::Left, candidate.left.as_ref()),
        ] {
            let Some(ownership) = ownership else { continue };
            let length = line_length(candidate.start, candidate.end);
            for (role, texture_name) in [
                (DoomWallTextureRole::Upper, ownership.upper_texture.as_str()),
                (DoomWallTextureRole::Lower, ownership.lower_texture.as_str()),
                (
                    DoomWallTextureRole::Middle,
                    ownership.middle_texture.as_str(),
                ),
            ] {
                let role_can_present = role == DoomWallTextureRole::Middle || two_sided;
                if role_can_present && texture_name != "-" {
                    observations.push(DoomWallTextureAxisObservation {
                        source_linedef: candidate.source_linedef,
                        linedef_flags: candidate.linedef_flags,
                        source_sidedef: ownership.source_sidedef,
                        side,
                        role,
                        texture_name: texture_name.to_owned(),
                        u_start: match side {
                            DoomWallSideKind::Right => f64::from(ownership.x_offset) + length,
                            DoomWallSideKind::Left => f64::from(ownership.x_offset),
                        },
                        u_end: match side {
                            DoomWallSideKind::Right => f64::from(ownership.x_offset),
                            DoomWallSideKind::Left => f64::from(ownership.x_offset) + length,
                        },
                        v_offset: ownership.y_offset,
                    });
                }
            }
        }
    }
    Ok(observations)
}

/// Resolves authored wall texture axes against provider-supplied plain extents.
/// No UV normalization, V placement, or pegging transform is chosen here.
pub fn resolve_doom_wall_texture_bindings(
    map: &DoomMapCore,
    extents: &[DoomTextureExtent],
) -> Result<Vec<DoomWallTextureBinding>, DoomGeometryError> {
    let mut by_name = BTreeMap::new();
    for extent in extents {
        if by_name
            .insert(extent.name.to_ascii_uppercase(), extent)
            .is_some()
        {
            return Err(DoomGeometryError::DuplicateTextureExtent {
                name: extent.name.clone(),
            });
        }
    }
    observe_doom_wall_texture_axes(map)?
        .into_iter()
        .map(|axis| {
            let extent = by_name.get(&axis.texture_name.to_ascii_uppercase()).ok_or(
                DoomGeometryError::MissingTextureExtent {
                    name: axis.texture_name.clone(),
                    linedef_index: axis.source_linedef.record_index,
                },
            )?;
            Ok(DoomWallTextureBinding {
                axis,
                texture_width: extent.width,
                texture_height: extent.height,
            })
        })
        .collect()
}

/// Audits the classic source flag bits before any pegging transform is chosen.
pub fn audit_doom_pegging_flags(
    map: &DoomMapCore,
) -> Result<DoomPeggingFlagAudit, DoomGeometryError> {
    const DONT_PEG_TOP: u16 = 0x0008;
    const DONT_PEG_BOTTOM: u16 = 0x0010;
    let mut audit = DoomPeggingFlagAudit {
        upper_axes: 0,
        lower_axes: 0,
        upper_unpegged: 0,
        lower_unpegged: 0,
    };
    for axis in observe_doom_wall_texture_axes(map)? {
        match axis.role {
            DoomWallTextureRole::Upper => {
                audit.upper_axes += 1;
                if axis.linedef_flags & DONT_PEG_TOP != 0 {
                    audit.upper_unpegged += 1;
                }
            }
            DoomWallTextureRole::Lower => {
                audit.lower_axes += 1;
                if axis.linedef_flags & DONT_PEG_BOTTOM != 0 {
                    audit.lower_unpegged += 1;
                }
            }
            DoomWallTextureRole::Middle => {}
        }
    }
    Ok(audit)
}

fn visit_bsp_node(
    map: &DoomMapCore,
    node_index: u16,
    path: &mut Vec<DoomBspPathStep>,
    ancestors: &mut Vec<u16>,
    paths: &mut [Option<Vec<DoomBspPathStep>>],
) -> Result<(), DoomGeometryError> {
    if ancestors.contains(&node_index) {
        return Err(DoomGeometryError::BspCycle { node_index });
    }
    let node = &map.nodes[usize::from(node_index)];
    ancestors.push(node_index);
    for (side, child) in [
        (DoomBspSide::Right, node.right_child),
        (DoomBspSide::Left, node.left_child),
    ] {
        path.push(DoomBspPathStep {
            source_node: node.source,
            side,
            origin: [node.x, node.y],
            delta: [node.delta_x, node.delta_y],
        });
        match child {
            DoomBspChild::Node(child_index) => {
                visit_bsp_node(map, child_index, path, ancestors, paths)?
            }
            DoomBspChild::Subsector(subsector_index) => {
                let available = paths.len();
                let entry = paths.get_mut(usize::from(subsector_index)).ok_or(
                    DoomGeometryError::BspSubsectorOutOfBounds {
                        subsector_index,
                        available,
                    },
                )?;
                *entry = Some(path.clone());
            }
        }
        path.pop();
    }
    ancestors.pop();
    Ok(())
}

fn map_bounds(map: &DoomMapCore) -> Result<[[f64; 2]; 4], DoomGeometryError> {
    let first = map
        .vertices
        .first()
        .ok_or(DoomGeometryError::MissingMapBounds)?;
    let (minimum_x, maximum_x, minimum_y, maximum_y) = map.vertices.iter().fold(
        (first.x, first.x, first.y, first.y),
        |(minimum_x, maximum_x, minimum_y, maximum_y), vertex| {
            (
                minimum_x.min(vertex.x),
                maximum_x.max(vertex.x),
                minimum_y.min(vertex.y),
                maximum_y.max(vertex.y),
            )
        },
    );
    Ok([
        [f64::from(minimum_x), f64::from(minimum_y)],
        [f64::from(maximum_x), f64::from(minimum_y)],
        [f64::from(maximum_x), f64::from(maximum_y)],
        [f64::from(minimum_x), f64::from(maximum_y)],
    ])
}

fn clip_convex_region(vertices: &[[f64; 2]], step: &DoomBspPathStep) -> Vec<[f64; 2]> {
    const EPSILON: f64 = 1.0e-9;
    let mut clipped = Vec::with_capacity(vertices.len() + 1);
    for (previous, current) in vertices
        .iter()
        .copied()
        .zip(vertices.iter().copied().cycle().skip(1))
        .take(vertices.len())
    {
        let previous_distance = partition_distance(previous, step);
        let current_distance = partition_distance(current, step);
        let previous_inside = is_inside_partition(previous_distance, step.side, EPSILON);
        let current_inside = is_inside_partition(current_distance, step.side, EPSILON);
        if previous_inside != current_inside {
            let fraction = previous_distance / (previous_distance - current_distance);
            clipped.push([
                previous[0] + (current[0] - previous[0]) * fraction,
                previous[1] + (current[1] - previous[1]) * fraction,
            ]);
        }
        if current_inside {
            clipped.push(current);
        }
    }
    clipped
}

fn partition_distance(point: [f64; 2], step: &DoomBspPathStep) -> f64 {
    f64::from(step.delta[0]) * (point[1] - f64::from(step.origin[1]))
        - f64::from(step.delta[1]) * (point[0] - f64::from(step.origin[0]))
}

fn is_inside_partition(distance: f64, side: DoomBspSide, epsilon: f64) -> bool {
    match side {
        DoomBspSide::Right => distance <= epsilon,
        DoomBspSide::Left => distance >= -epsilon,
    }
}

/// Splits one vertical wall quad into two triangles whose geometric normal
/// faces the owning classic-Doom sidedef.
///
/// Tokimu's experimental map embedding is `(doom_x, height, doom_y)`. With
/// that embedding, the WAD's side 0/right/front side is the source-direction
/// right normal `(delta_y, 0, -delta_x)`. Keeping this conversion in one
/// helper prevents one-sided walls, height bands, and masked middles from
/// silently adopting different winding conventions.
fn doom_wall_quad_triangles(
    side: DoomWallSideKind,
    start_bottom: [f64; 3],
    end_bottom: [f64; 3],
    start_top: [f64; 3],
    end_top: [f64; 3],
) -> [[[f64; 3]; 3]; 2] {
    match side {
        DoomWallSideKind::Right => [
            [start_bottom, end_top, end_bottom],
            [start_bottom, start_top, end_top],
        ],
        DoomWallSideKind::Left => [
            [end_bottom, start_top, start_bottom],
            [end_bottom, end_top, start_top],
        ],
    }
}

fn append_two_sided_band(
    triangles: &mut Vec<DoomTwoSidedWallTriangle>,
    candidate: &DoomWallCandidate,
    ownership: &DoomWallSide,
    request: DoomWallBandRequest<'_>,
) {
    let source_start = candidate.start.map(f64::from);
    let source_end = candidate.end.map(f64::from);
    let start_bottom = doom_point_to_tokimu(source_start, f64::from(request.bottom));
    let end_bottom = doom_point_to_tokimu(source_end, f64::from(request.bottom));
    let start_top = doom_point_to_tokimu(source_start, f64::from(request.top));
    let end_top = doom_point_to_tokimu(source_end, f64::from(request.top));
    let positions =
        doom_wall_quad_triangles(request.side, start_bottom, end_bottom, start_top, end_top);
    triangles.extend(positions.map(|positions| DoomTwoSidedWallTriangle {
        source_linedef: candidate.source_linedef,
        source_sidedef: ownership.source_sidedef,
        source_sector: ownership.source_sector,
        side: request.side,
        band: request.band,
        texture_name: request.texture_name.to_owned(),
        positions,
    }));
}

fn append_two_sided_middle_wall(
    triangles: &mut Vec<DoomTwoSidedMiddleWallTriangle>,
    candidate: &DoomWallCandidate,
    ownership: &DoomWallSide,
    side: DoomWallSideKind,
    opening_floor: i16,
    opening_ceiling: i16,
) {
    let source_start = candidate.start.map(f64::from);
    let source_end = candidate.end.map(f64::from);
    let start_bottom = doom_point_to_tokimu(source_start, f64::from(opening_floor));
    let end_bottom = doom_point_to_tokimu(source_end, f64::from(opening_floor));
    let start_top = doom_point_to_tokimu(source_start, f64::from(opening_ceiling));
    let end_top = doom_point_to_tokimu(source_end, f64::from(opening_ceiling));
    let positions = doom_wall_quad_triangles(side, start_bottom, end_bottom, start_top, end_top);
    triangles.extend(positions.map(|positions| DoomTwoSidedMiddleWallTriangle {
        source_linedef: candidate.source_linedef,
        source_sidedef: ownership.source_sidedef,
        source_sector: ownership.source_sector,
        side,
        texture_name: ownership.middle_texture.clone(),
        opening_floor,
        opening_ceiling,
        positions,
    }));
}

fn outside_partition_distance(distance: f64, side: DoomBspSide) -> f64 {
    match side {
        DoomBspSide::Right => distance.max(0.0),
        DoomBspSide::Left => (-distance).max(0.0),
    }
}

fn polygon_signed_area(vertices: &[[f64; 2]]) -> f64 {
    vertices
        .iter()
        .copied()
        .zip(vertices.iter().copied().cycle().skip(1))
        .take(vertices.len())
        .map(|(start, end)| start[0] * end[1] - end[0] * start[1])
        .sum::<f64>()
        * 0.5
}

fn line_length(start: [i16; 2], end: [i16; 2]) -> f64 {
    let x = f64::from(end[0]) - f64::from(start[0]);
    let y = f64::from(end[1]) - f64::from(start[1]);
    x.hypot(y)
}

fn resolve_side(map: &DoomMapCore, sidedef_index: Option<u16>) -> Option<DoomWallSide> {
    let sidedef_index = sidedef_index?;
    let sidedef = &map.sidedefs[usize::from(sidedef_index)];
    let sector = &map.sectors[usize::from(sidedef.sector)];
    Some(DoomWallSide {
        source_sidedef: sidedef.source,
        source_sector: sector.source,
        sector_index: sidedef.sector,
        x_offset: sidedef.x_offset,
        y_offset: sidedef.y_offset,
        upper_texture: sidedef.upper_texture.clone(),
        lower_texture: sidedef.lower_texture.clone(),
        middle_texture: sidedef.middle_texture.clone(),
    })
}

fn point_for_vertex(map: &DoomMapCore, vertex_index: u16) -> [i16; 2] {
    let vertex = &map.vertices[usize::from(vertex_index)];
    [vertex.x, vertex.y]
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use doom_map_provider::{
        DoomBlockmapObservation, DoomBspChild, DoomLinedef, DoomMapCore, DoomNode,
        DoomRejectMatrix, DoomSector, DoomSeg, DoomSidedef, DoomSourceRecord, DoomSubsector,
        DoomVertex,
    };

    use super::{
        audit_doom_pegging_flags, audit_doom_subsector_bsp_paths,
        audit_doom_subsector_loop_closure, audit_doom_vertical_topology, audit_doom_wall_topology,
        classic_ceiling_after_mark_without_upper, classic_ceiling_plane_rows, classic_open_rows,
        clip_doom_seg_textured_wall_triangle_to_linedef_interval, doom_direction_to_tokimu,
        doom_point_to_tokimu, locate_doom_point_subsector, lower_doom_one_sided_walls,
        lower_doom_paired_sky_boundary_triangles, lower_doom_sector_bounded_subsector_surfaces,
        lower_doom_seg_textured_wall_triangles, lower_doom_source_bounded_subsector_surfaces,
        lower_doom_subsector_surfaces, lower_doom_textured_wall_triangles,
        lower_doom_two_sided_middle_walls, lower_doom_two_sided_wall_bands,
        observe_doom_classic_bsp, observe_doom_classic_bsp_far_first_control,
        observe_doom_classic_bsp_suppressing_solid_range_source_seg,
        observe_doom_classic_bsp_without_solid_range_pruning, observe_doom_seg_occluders,
        observe_doom_seg_plane_marks, observe_doom_sky_surfaces,
        observe_doom_two_sided_middle_textures, observe_doom_wall_texture_axes,
        reconstruct_doom_ordered_wall_fragments, resolve_doom_linedef_subsector_membership,
        resolve_doom_subsector_bsp_paths, resolve_doom_subsector_loops,
        resolve_doom_subsector_regions, resolve_doom_subsector_sector_ownership,
        resolve_doom_viewer_subsector_order, resolve_doom_wall_candidates,
        resolve_doom_wall_texture_bindings, tokimu_direction_to_doom, tokimu_point_to_doom,
        DoomBspSide, DoomClassicSuppressedSolidRangeMutation, DoomGeometryError,
        DoomLinedefSubsectorMembership, DoomOrderedWallInterval,
        DoomSegClassicVerticalClipObservation, DoomSurfacePlane, DoomTextureExtent, DoomWallBand,
        DoomWallSideKind, DoomWallTextureRole,
    };

    #[test]
    fn unsigned_classic_clip_state_matches_dooms_signed_inclusive_rows() {
        // Doom initializes ceilingclip to -1 and floorclip to viewheight.
        // The provider's unsigned form stores first-open / first-closed.
        assert_eq!(classic_open_rows(0, 200), Some([0, 199]));
        assert_eq!(classic_ceiling_plane_rows(0, 40), Some([0, 39]));

        // An upper wall ending on row 63 leaves row 64 as the first open
        // row. The ordinary max in the wall loop performs this transition.
        assert_eq!(63usize.saturating_add(1), 64);
        assert_eq!(classic_open_rows(64, 200), Some([64, 199]));

        // Doom's no-upper path assigns last-closed = yl - 1. In normalized
        // state that means first-open = yl, without a compensating -1.
        assert_eq!(classic_ceiling_after_mark_without_upper(0, 40), 40);
        assert_eq!(classic_open_rows(40, 200), Some([40, 199]));

        // A terminal one-sided wall closes the normalized interval.
        assert_eq!(classic_open_rows(200, 0), None);
    }

    #[test]
    fn retains_both_side_and_sector_sources_before_wall_lowering() {
        let candidates = resolve_doom_wall_candidates(&map_with_linedef(Some(0), Some(1))).unwrap();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].start, [10, 20]);
        assert_eq!(candidates[0].end, [30, 40]);
        assert_eq!(
            candidates[0]
                .right
                .as_ref()
                .unwrap()
                .source_sidedef
                .record_index,
            0
        );
        assert_eq!(
            candidates[0]
                .right
                .as_ref()
                .unwrap()
                .source_sector
                .record_index,
            0
        );
        assert_eq!(
            candidates[0]
                .left
                .as_ref()
                .unwrap()
                .source_sidedef
                .record_index,
            1
        );
        assert_eq!(
            candidates[0]
                .left
                .as_ref()
                .unwrap()
                .source_sector
                .record_index,
            1
        );

        assert_eq!(
            audit_doom_wall_topology(&candidates),
            super::DoomWallTopologyAudit {
                candidates: 1,
                one_sided: 0,
                two_sided: 1,
                same_sector_two_sided: 0,
            }
        );
    }

    #[test]
    fn rejects_linedef_without_either_side() {
        assert_eq!(
            resolve_doom_wall_candidates(&map_with_linedef(None, None)),
            Err(DoomGeometryError::MissingBothSidedefs { linedef_index: 0 })
        );
    }

    #[test]
    fn audits_closed_source_openings_without_inventing_a_repair() {
        let mut map = map_with_linedef(Some(0), Some(1));
        map.sectors[1].floor_height = 128;
        map.sectors[1].ceiling_height = 160;
        map.sectors[1].ceiling_height = 128;

        assert_eq!(
            audit_doom_vertical_topology(&map).unwrap(),
            super::DoomVerticalTopologyAudit {
                sectors: 2,
                sectors_without_positive_clearance: 1,
                two_sided_openings: 1,
                two_sided_openings_without_positive_clearance: 1,
            }
        );
    }

    #[test]
    fn rejects_zero_length_linedef() {
        let mut map = map_with_linedef(Some(0), None);
        map.vertices[1] = map.vertices[0].clone();
        assert_eq!(
            resolve_doom_wall_candidates(&map),
            Err(DoomGeometryError::DegenerateLinedef { linedef_index: 0 })
        );
    }

    #[test]
    fn retains_closed_subsector_boundary_in_source_order() {
        let mut map = map_with_linedef(Some(0), None);
        map.vertices.push(DoomVertex {
            source: source(2),
            x: 10,
            y: 40,
        });
        map.segs = vec![seg(0, 0, 1), seg(1, 1, 2), seg(2, 2, 0)];
        map.subsectors = vec![DoomSubsector {
            source: source(0),
            seg_count: 3,
            first_seg: 0,
        }];

        let loops = resolve_doom_subsector_loops(&map).unwrap();
        assert_eq!(loops[0].vertices, vec![[10, 20], [30, 40], [10, 40]]);
        assert_eq!(
            loops[0]
                .source_segs
                .iter()
                .map(|edge| edge.source_seg.record_index)
                .collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
    }

    #[test]
    fn rejects_open_subsector_boundary() {
        let mut map = map_with_linedef(Some(0), None);
        map.vertices.push(DoomVertex {
            source: source(2),
            x: 10,
            y: 40,
        });
        map.vertices.push(DoomVertex {
            source: source(3),
            x: 50,
            y: 60,
        });
        map.segs = vec![seg(0, 0, 1), seg(1, 2, 0), seg(2, 2, 3)];
        map.subsectors = vec![DoomSubsector {
            source: source(0),
            seg_count: 3,
            first_seg: 0,
        }];

        assert_eq!(
            resolve_doom_subsector_loops(&map),
            Err(DoomGeometryError::SubsectorBoundaryOpen {
                subsector_index: 0,
                previous_seg_index: 0,
                next_seg_index: 0,
            })
        );
    }

    #[test]
    fn source_bounded_surfaces_stitch_shuffled_directed_segs() {
        let mut map = map_with_linedef(Some(0), None);
        map.vertices = vec![
            DoomVertex {
                source: source(0),
                x: 0,
                y: 0,
            },
            DoomVertex {
                source: source(1),
                x: 64,
                y: 0,
            },
            DoomVertex {
                source: source(2),
                x: 64,
                y: 32,
            },
            DoomVertex {
                source: source(3),
                x: 0,
                y: 32,
            },
            // Expands the old map-bounds/BSP-path fallback without becoming
            // part of this subsector's source boundary.
            DoomVertex {
                source: source(4),
                x: 256,
                y: 256,
            },
        ];
        map.segs = vec![seg(0, 0, 1), seg(1, 1, 2), seg(2, 3, 0), seg(3, 2, 3)];
        map.subsectors = vec![DoomSubsector {
            source: source(0),
            seg_count: 4,
            first_seg: 0,
        }];
        let paths = vec![super::DoomSubsectorBspPath {
            source_subsector: source(0),
            steps: Vec::new(),
        }];

        let bake = lower_doom_source_bounded_subsector_surfaces(&map, &paths).unwrap();

        assert_eq!(bake.audit.subsectors, 1);
        assert_eq!(bake.audit.stitched_seg_loops, 1);
        assert_eq!(bake.audit.stitched_loop_refinements, 1);
        assert_eq!(bake.audit.bsp_path_fallbacks, 0);
        assert_eq!(bake.surfaces.len(), 4);
        assert!(bake
            .surfaces
            .iter()
            .flat_map(|surface| surface.positions)
            .all(|position| (0.0..=64.0).contains(&position[0])
                && (0.0..=32.0).contains(&position[2])));
        assert_normal_direction(bake.surfaces[0].positions, [0.0, 1.0, 0.0]);
        assert_normal_direction(bake.surfaces[2].positions, [0.0, -1.0, 0.0]);
    }

    #[test]
    fn source_bounded_surfaces_fail_open_to_bsp_region_for_open_segs() {
        let mut map = map_with_linedef(Some(0), None);
        map.vertices.push(DoomVertex {
            source: source(2),
            x: 10,
            y: 40,
        });
        map.segs = vec![seg(0, 0, 1), seg(1, 1, 2), seg(2, 2, 1)];
        map.subsectors = vec![DoomSubsector {
            source: source(0),
            seg_count: 3,
            first_seg: 0,
        }];
        let paths = vec![super::DoomSubsectorBspPath {
            source_subsector: source(0),
            steps: Vec::new(),
        }];

        let bake = lower_doom_source_bounded_subsector_surfaces(&map, &paths).unwrap();

        assert_eq!(bake.audit.stitched_seg_loops, 0);
        assert_eq!(bake.audit.bsp_path_fallbacks, 1);
        assert_eq!(bake.surfaces.len(), 4);

        let sector_bake = lower_doom_sector_bounded_subsector_surfaces(&map, &paths).unwrap();
        assert_eq!(
            sector_bake.audit.sector_boundary_unavailable_subsectors,
            vec![source(0)]
        );
        assert_eq!(sector_bake.surfaces, bake.surfaces);
    }

    #[test]
    fn source_bounded_surfaces_combine_seg_and_implicit_bsp_boundaries() {
        let mut map = map_with_linedef(Some(0), None);
        map.vertices = vec![
            DoomVertex {
                source: source(0),
                x: 64,
                y: 0,
            },
            DoomVertex {
                source: source(1),
                x: 0,
                y: 0,
            },
            DoomVertex {
                source: source(2),
                x: 64,
                y: 32,
            },
            DoomVertex {
                source: source(3),
                x: 256,
                y: 256,
            },
        ];
        map.segs = vec![seg(0, 0, 1), seg(1, 2, 0)];
        map.subsectors = vec![DoomSubsector {
            source: source(0),
            seg_count: 2,
            first_seg: 0,
        }];
        let paths = vec![super::DoomSubsectorBspPath {
            source_subsector: source(0),
            steps: Vec::new(),
        }];

        let bake = lower_doom_source_bounded_subsector_surfaces(&map, &paths).unwrap();

        assert_eq!(bake.audit.stitched_seg_loops, 0);
        assert_eq!(bake.audit.seg_half_plane_regions, 1);
        assert_eq!(bake.audit.seg_half_plane_refinements, 1);
        assert_eq!(bake.audit.bsp_path_fallbacks, 0);
        assert!(bake
            .surfaces
            .iter()
            .flat_map(|surface| surface.positions)
            .all(|position| position[0] <= 64.0));
    }

    #[test]
    fn source_bounded_surfaces_admit_one_unit_node_builder_corner_quantization() {
        let mut map = map_with_linedef(Some(0), None);
        map.vertices = vec![
            DoomVertex {
                source: source(0),
                x: 1568,
                y: 1657,
            },
            DoomVertex {
                source: source(1),
                x: 2560,
                y: 1616,
            },
            DoomVertex {
                source: source(2),
                x: 2560,
                y: 1467,
            },
            DoomVertex {
                source: source(3),
                x: 2304,
                y: 1488,
            },
            DoomVertex {
                source: source(4),
                x: 1568,
                y: 1546,
            },
            // Reproduces the oversized untrimmed extent without becoming a
            // source SEG endpoint.
            DoomVertex {
                source: source(5),
                x: 1800,
                y: 2752,
            },
        ];
        map.segs = vec![seg(0, 0, 1), seg(1, 1, 2), seg(2, 3, 4)];
        map.subsectors = vec![DoomSubsector {
            source: source(0),
            seg_count: 3,
            first_seg: 0,
        }];
        let paths = vec![super::DoomSubsectorBspPath {
            source_subsector: source(0),
            steps: Vec::new(),
        }];

        let bake = lower_doom_source_bounded_subsector_surfaces(&map, &paths).unwrap();

        assert_eq!(bake.audit.stitched_seg_loops, 0);
        assert_eq!(bake.audit.seg_half_plane_regions, 1);
        assert_eq!(bake.audit.bsp_path_fallbacks, 0);
        assert!(bake
            .surfaces
            .iter()
            .flat_map(|surface| surface.positions)
            .all(|position| position[2] <= 1657.0));
    }

    #[test]
    fn source_bounded_surfaces_trim_a_concave_sector_from_directed_linedefs() {
        let mut map = map_with_linedef(Some(0), None);
        map.vertices = [(0, 0), (0, 64), (32, 64), (32, 32), (64, 32), (64, 0)]
            .into_iter()
            .enumerate()
            .map(|(index, (x, y))| DoomVertex {
                source: source(index as u32),
                x,
                y,
            })
            .collect();
        let boundary = [(0_u16, 1_u16), (1, 2), (2, 3), (3, 4), (4, 5), (5, 0)];
        map.linedefs = boundary
            .iter()
            .enumerate()
            .map(|(index, &(start_vertex, end_vertex))| DoomLinedef {
                source: source(index as u32),
                start_vertex,
                end_vertex,
                flags: 0,
                special: 0,
                tag: 0,
                right_sidedef: Some(0),
                left_sidedef: None,
            })
            .collect();
        map.segs = boundary
            .iter()
            .enumerate()
            .map(|(index, &(start_vertex, end_vertex))| DoomSeg {
                source: source(index as u32),
                start_vertex,
                end_vertex,
                angle: 0,
                linedef: index as u16,
                direction: 0,
                offset: 0,
            })
            .collect();
        map.subsectors = vec![DoomSubsector {
            source: source(0),
            seg_count: boundary.len() as u16,
            first_seg: 0,
        }];
        let paths = vec![super::DoomSubsectorBspPath {
            source_subsector: source(0),
            steps: Vec::new(),
        }];

        let bake = lower_doom_sector_bounded_subsector_surfaces(&map, &paths).unwrap();
        let surface_area = bake
            .surfaces
            .iter()
            .map(|triangle| {
                let [a, b, c] = triangle.positions;
                ((b[0] - a[0]) * (c[2] - a[2]) - (b[2] - a[2]) * (c[0] - a[0])).abs() * 0.5
            })
            .sum::<f64>();

        assert_eq!(bake.audit.bsp_path_fallbacks, 1);
        assert_eq!(bake.audit.sector_boundary_supported_subsectors, 1);
        assert_eq!(bake.audit.sector_boundary_refinements, 1);
        // The clockwise L shell has area 3,072. Floor and ceiling each own
        // that finite support; the missing upper-right quadrant stays empty.
        assert!((surface_area - 6144.0).abs() <= 1.0e-7);
    }

    #[test]
    fn audits_each_strict_subsector_loop_rejection() {
        let mut map = map_with_linedef(Some(0), None);
        map.vertices.push(DoomVertex {
            source: source(2),
            x: 10,
            y: 40,
        });
        map.vertices.push(DoomVertex {
            source: source(3),
            x: 50,
            y: 60,
        });
        map.segs = vec![seg(0, 0, 1), seg(1, 2, 0), seg(2, 2, 3)];
        map.subsectors = vec![DoomSubsector {
            source: source(0),
            seg_count: 3,
            first_seg: 0,
        }];

        assert_eq!(
            audit_doom_subsector_loop_closure(&map),
            super::DoomSubsectorLoopClosureAudit {
                subsectors: 1,
                closed_loops: 0,
                rejected: vec![DoomGeometryError::SubsectorBoundaryOpen {
                    subsector_index: 0,
                    previous_seg_index: 0,
                    next_seg_index: 0,
                }],
            }
        );
    }

    #[test]
    fn retains_root_to_leaf_partition_paths_for_subsectors() {
        let mut map = map_with_linedef(Some(0), None);
        map.subsectors = vec![
            DoomSubsector {
                source: source(0),
                seg_count: 0,
                first_seg: 0,
            },
            DoomSubsector {
                source: source(1),
                seg_count: 0,
                first_seg: 0,
            },
        ];
        map.nodes = vec![DoomNode {
            source: source(0),
            x: 0,
            y: 0,
            delta_x: 64,
            delta_y: 0,
            right_bbox: [0; 4],
            left_bbox: [0; 4],
            right_child: DoomBspChild::Subsector(0),
            left_child: DoomBspChild::Subsector(1),
        }];

        let paths = resolve_doom_subsector_bsp_paths(&map).unwrap();
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].steps[0].side, DoomBspSide::Right);
        assert_eq!(paths[1].steps[0].side, DoomBspSide::Left);
        assert_eq!(
            audit_doom_subsector_bsp_paths(&paths),
            super::DoomBspPathAudit {
                subsectors: 2,
                minimum_depth: 1,
                maximum_depth: 1,
            }
        );
    }

    #[test]
    fn locates_only_an_unambiguous_point_in_a_retained_bsp_leaf() {
        let mut map = map_with_linedef(Some(0), None);
        map.subsectors = vec![
            DoomSubsector {
                source: source(0),
                seg_count: 0,
                first_seg: 0,
            },
            DoomSubsector {
                source: source(1),
                seg_count: 0,
                first_seg: 0,
            },
        ];
        map.nodes = vec![DoomNode {
            source: source(0),
            x: 0,
            y: 0,
            delta_x: 64,
            delta_y: 0,
            right_bbox: [0; 4],
            left_bbox: [0; 4],
            right_child: DoomBspChild::Subsector(0),
            left_child: DoomBspChild::Subsector(1),
        }];
        let paths = resolve_doom_subsector_bsp_paths(&map).unwrap();

        assert_eq!(
            locate_doom_point_subsector([5, -1], &paths)
                .unwrap()
                .source_subsector,
            source(0)
        );
        assert_eq!(
            locate_doom_point_subsector([5, 1], &paths)
                .unwrap()
                .source_subsector,
            source(1)
        );
        assert!(matches!(
            locate_doom_point_subsector([5, 0], &paths),
            Err(DoomGeometryError::PointNotInsideUniqueSubsector { x: 5, y: 0 })
        ));
    }

    #[test]
    fn viewer_bsp_order_visits_the_near_leaf_before_the_far_leaf() {
        let mut map = map_with_linedef(Some(0), None);
        map.subsectors = vec![
            DoomSubsector {
                source: source(3),
                seg_count: 0,
                first_seg: 0,
            },
            DoomSubsector {
                source: source(7),
                seg_count: 0,
                first_seg: 0,
            },
        ];
        map.nodes = vec![DoomNode {
            source: source(9),
            x: 0,
            y: 0,
            delta_x: 64,
            delta_y: 0,
            right_bbox: [0; 4],
            left_bbox: [0; 4],
            right_child: DoomBspChild::Subsector(0),
            left_child: DoomBspChild::Subsector(1),
        }];

        assert_eq!(
            resolve_doom_viewer_subsector_order(&map, [0, -1]).unwrap(),
            vec![source(3), source(7)]
        );
        assert_eq!(
            resolve_doom_viewer_subsector_order(&map, [0, 1]).unwrap(),
            vec![source(7), source(3)]
        );
    }

    #[test]
    fn clips_leaf_regions_using_the_retained_bsp_side() {
        let mut map = map_with_linedef(Some(0), None);
        map.vertices = vec![
            DoomVertex {
                source: source(0),
                x: 0,
                y: 0,
            },
            DoomVertex {
                source: source(1),
                x: 100,
                y: 0,
            },
            DoomVertex {
                source: source(2),
                x: 100,
                y: 100,
            },
            DoomVertex {
                source: source(3),
                x: 0,
                y: 100,
            },
        ];
        map.subsectors = vec![
            DoomSubsector {
                source: source(0),
                seg_count: 0,
                first_seg: 0,
            },
            DoomSubsector {
                source: source(1),
                seg_count: 0,
                first_seg: 0,
            },
        ];
        map.nodes = vec![DoomNode {
            source: source(0),
            x: 50,
            y: 0,
            delta_x: 0,
            delta_y: 100,
            right_bbox: [0; 4],
            left_bbox: [0; 4],
            right_child: DoomBspChild::Subsector(0),
            left_child: DoomBspChild::Subsector(1),
        }];

        let paths = resolve_doom_subsector_bsp_paths(&map).unwrap();
        let regions = resolve_doom_subsector_regions(&map, &paths).unwrap();
        assert!(regions[0].vertices.iter().all(|point| point[0] >= 50.0));
        assert!(regions[1].vertices.iter().all(|point| point[0] <= 50.0));
    }

    #[test]
    fn resolves_subsector_sector_from_seg_direction() {
        let mut map = map_with_linedef(Some(0), None);
        map.segs = vec![seg(0, 0, 1)];
        map.subsectors = vec![DoomSubsector {
            source: source(0),
            seg_count: 1,
            first_seg: 0,
        }];

        let ownership = resolve_doom_subsector_sector_ownership(&map).unwrap();
        assert_eq!(ownership[0].sector_index, 0);
        assert_eq!(ownership[0].source_sector.record_index, 0);
    }

    #[test]
    fn retains_one_to_many_linedef_subsector_membership_from_source_segs() {
        let mut map = map_with_linedef(Some(0), None);
        map.segs = vec![seg(0, 0, 1), seg(1, 1, 0)];
        map.subsectors = vec![
            DoomSubsector {
                source: source(3),
                seg_count: 1,
                first_seg: 0,
            },
            DoomSubsector {
                source: source(7),
                seg_count: 1,
                first_seg: 1,
            },
        ];

        assert_eq!(
            resolve_doom_linedef_subsector_membership(&map),
            vec![DoomLinedefSubsectorMembership {
                source_linedef: source(0),
                source_subsectors: vec![source(3), source(7)],
            }]
        );
    }

    #[test]
    fn lowers_bsp_leaf_regions_to_floor_and_ceiling_triangles() {
        let mut map = map_with_linedef(Some(0), None);
        map.vertices = vec![
            DoomVertex {
                source: source(0),
                x: 0,
                y: 0,
            },
            DoomVertex {
                source: source(1),
                x: 100,
                y: 0,
            },
            DoomVertex {
                source: source(2),
                x: 100,
                y: 100,
            },
            DoomVertex {
                source: source(3),
                x: 0,
                y: 100,
            },
        ];
        map.segs = vec![seg(0, 0, 1), seg(1, 1, 2)];
        map.subsectors = vec![
            DoomSubsector {
                source: source(0),
                seg_count: 1,
                first_seg: 0,
            },
            DoomSubsector {
                source: source(1),
                seg_count: 1,
                first_seg: 1,
            },
        ];
        map.nodes = vec![DoomNode {
            source: source(0),
            x: 50,
            y: 0,
            delta_x: 0,
            delta_y: 100,
            right_bbox: [0; 4],
            left_bbox: [0; 4],
            right_child: DoomBspChild::Subsector(0),
            left_child: DoomBspChild::Subsector(1),
        }];

        let paths = resolve_doom_subsector_bsp_paths(&map).unwrap();
        let triangles = lower_doom_subsector_surfaces(&map, &paths).unwrap();
        assert_eq!(triangles.len(), 8);
        assert_eq!(
            triangles
                .iter()
                .filter(|triangle| triangle.plane == DoomSurfacePlane::Floor)
                .count(),
            4
        );
        assert!(triangles
            .iter()
            .all(|triangle| triangle.source_sector.record_index == 0));
        assert!(triangles
            .iter()
            .all(|triangle| triangle.source_subsector.record_index <= 1));
        assert!(triangles.iter().all(|triangle| {
            triangle.texture_name == "FLOOR0_1" || triangle.texture_name == "CEIL1_1"
        }));
        assert!(triangles.iter().all(|triangle| {
            triangle
                .positions
                .iter()
                .all(|position| position[1] == 0.0 || position[1] == 128.0)
        }));
        for triangle in &triangles {
            let normal_y = (triangle.positions[1][2] - triangle.positions[0][2])
                * (triangle.positions[2][0] - triangle.positions[0][0])
                - (triangle.positions[1][0] - triangle.positions[0][0])
                    * (triangle.positions[2][2] - triangle.positions[0][2]);
            match triangle.plane {
                DoomSurfacePlane::Floor => assert!(normal_y > 0.0),
                DoomSurfacePlane::Ceiling => assert!(normal_y < 0.0),
            }
        }
    }

    #[test]
    fn lowers_one_sided_wall_to_two_source_traceable_triangles() {
        let triangles = lower_doom_one_sided_walls(&map_with_linedef(Some(0), None)).unwrap();

        assert_eq!(triangles.len(), 2);
        assert!(triangles
            .iter()
            .all(|triangle| triangle.side == DoomWallSideKind::Right));
        assert!(triangles
            .iter()
            .all(|triangle| triangle.source_linedef.record_index == 0));
        assert!(triangles
            .iter()
            .all(|triangle| triangle.source_sidedef.record_index == 0));
        assert!(triangles
            .iter()
            .all(|triangle| triangle.source_sector.record_index == 0));
        assert!(triangles.iter().all(|triangle| triangle
            .positions
            .iter()
            .all(|position| { position[1] == 0.0 || position[1] == 128.0 })));
    }

    #[test]
    fn wall_winding_faces_the_owning_wad_sidedef() {
        let right = lower_doom_one_sided_walls(&map_with_linedef(Some(0), None)).unwrap();
        let left = lower_doom_one_sided_walls(&map_with_linedef(None, Some(1))).unwrap();

        // The fixture runs from (10, 20) to (30, 40). In Tokimu's present
        // `(doom_x, height, doom_y)` embedding, WAD side 0/right/front faces
        // `(delta_y, 0, -delta_x)` and side 1/left/back faces its inverse.
        assert_normal_direction(right[0].positions, [20.0, 0.0, -20.0]);
        assert_normal_direction(left[0].positions, [-20.0, 0.0, 20.0]);
    }

    #[test]
    fn doom_point_and_direction_lifts_round_trip_exactly() {
        let source_point = ([1056.0, -3616.0], 36.0);
        let world_point = doom_point_to_tokimu(source_point.0, source_point.1);
        assert_eq!(world_point, [1056.0, 36.0, -3616.0]);
        assert_eq!(tokimu_point_to_doom(world_point), source_point);

        let source_direction = ([20.0, -40.0], 12.0);
        let world_direction = doom_direction_to_tokimu(source_direction.0, source_direction.1);
        assert_eq!(world_direction, [20.0, 12.0, -40.0]);
        assert_eq!(tokimu_direction_to_doom(world_direction), source_direction);
    }

    #[test]
    fn lowers_one_sided_wall_with_source_texel_coordinates() {
        let mut map = map_with_linedef(Some(0), None);
        map.sidedefs[0].x_offset = 7;
        map.sidedefs[0].y_offset = -3;
        map.sidedefs[0].middle_texture = "WALL".to_owned();

        let triangles = lower_doom_textured_wall_triangles(
            &map,
            &[DoomTextureExtent {
                name: "WALL".to_owned(),
                width: 64,
                height: 128,
            }],
        )
        .unwrap();

        assert_eq!(triangles.len(), 2);
        assert!(triangles
            .iter()
            .all(|triangle| triangle.role == super::DoomWallTextureRole::Middle));
        assert!(triangles
            .iter()
            .all(|triangle| triangle.texture_name == "WALL"));
        let coordinates = triangles
            .iter()
            .flat_map(|triangle| triangle.texture_coordinates)
            .collect::<Vec<_>>();
        assert!(coordinates.contains(&[7.0, 125.0]));
        assert!(coordinates.contains(&[7.0, -3.0]));
        assert!(coordinates.iter().any(|coordinate| coordinate[0] > 30.0));
    }

    #[test]
    fn seg_granular_wall_lowering_preserves_source_uv_continuity() {
        let mut map = map_with_linedef(Some(0), None);
        map.sidedefs[0].x_offset = 7;
        map.sidedefs[0].middle_texture = "WALL".to_owned();
        map.vertices = vec![
            DoomVertex {
                source: source(0),
                x: 0,
                y: 0,
            },
            DoomVertex {
                source: source(1),
                x: 100,
                y: 0,
            },
            DoomVertex {
                source: source(2),
                x: 50,
                y: 0,
            },
        ];
        map.linedefs[0].start_vertex = 0;
        map.linedefs[0].end_vertex = 1;
        map.segs = vec![seg(11, 0, 2), seg(12, 2, 1)];

        let triangles = lower_doom_seg_textured_wall_triangles(
            &map,
            &[DoomTextureExtent {
                name: "WALL".to_owned(),
                width: 64,
                height: 128,
            }],
        )
        .unwrap();

        assert!(triangles
            .iter()
            .any(|triangle| triangle.source_seg.record_index == 11));
        assert!(triangles
            .iter()
            .any(|triangle| triangle.source_seg.record_index == 12));
        let seam_u = triangles
            .iter()
            .flat_map(|triangle| triangle.positions.iter().zip(triangle.texture_coordinates))
            .filter_map(|(position, coordinate)| {
                (position[0] == 50.0 && position[2] == 0.0).then_some(coordinate[0])
            })
            .collect::<Vec<_>>();
        assert!(!seam_u.is_empty());
        assert!(seam_u.iter().all(|u| (*u - 57.0).abs() < f64::EPSILON));
        assert!(triangles
            .iter()
            .all(|triangle| triangle.source_linedef == source(0)));
        assert!(triangles
            .iter()
            .all(|triangle| triangle.source_sidedef == source(0)));
    }

    #[test]
    fn seg_subinterval_clipping_preserves_identity_and_interpolates_source_texels() {
        let mut map = map_with_linedef(Some(0), None);
        map.sidedefs[0].x_offset = 7;
        map.sidedefs[0].middle_texture = "WALL".to_owned();
        map.vertices = vec![
            DoomVertex {
                source: source(0),
                x: 0,
                y: 0,
            },
            DoomVertex {
                source: source(1),
                x: 100,
                y: 0,
            },
            DoomVertex {
                source: source(2),
                x: 50,
                y: 0,
            },
        ];
        map.linedefs[0].start_vertex = 0;
        map.linedefs[0].end_vertex = 1;
        map.segs = vec![seg(11, 0, 2), seg(12, 2, 1)];
        let triangles = lower_doom_seg_textured_wall_triangles(
            &map,
            &[DoomTextureExtent {
                name: "WALL".to_owned(),
                width: 64,
                height: 128,
            }],
        )
        .unwrap();
        let first = triangles
            .iter()
            .find(|triangle| triangle.source_seg.record_index == 11)
            .unwrap();
        let clipped =
            clip_doom_seg_textured_wall_triangle_to_linedef_interval(&map, first, [0.125, 0.375])
                .unwrap();

        assert!(!clipped.is_empty());
        assert!(clipped
            .iter()
            .all(|triangle| triangle.source_seg == first.source_seg));
        assert!(clipped
            .iter()
            .all(|triangle| triangle.source_linedef == first.source_linedef));
        assert!(clipped
            .iter()
            .flat_map(|triangle| triangle.positions)
            .all(|position| {
                (12.5 - f64::EPSILON..=37.5 + f64::EPSILON).contains(&position[0])
            }));
        let original_u = first
            .texture_coordinates
            .iter()
            .map(|coordinate| coordinate[0])
            .collect::<Vec<_>>();
        let minimum_u = original_u.iter().copied().fold(f64::INFINITY, f64::min);
        let maximum_u = original_u.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        assert!(clipped
            .iter()
            .flat_map(|triangle| triangle.texture_coordinates)
            .all(|coordinate| coordinate[0].is_finite()
                && coordinate[0] >= minimum_u
                && coordinate[0] <= maximum_u));
    }

    #[test]
    fn classifies_seg_occluder_authority_from_source_opening_heights() {
        let mut map = map_with_linedef(Some(0), Some(1));
        map.segs = vec![seg(4, 0, 1)];
        assert_eq!(
            observe_doom_seg_occluders(&map).unwrap()[0].kind,
            super::DoomSegOccluderKind::Open
        );
        map.sectors[1].floor_height = 128;
        map.sectors[1].ceiling_height = 160;
        assert_eq!(
            observe_doom_seg_occluders(&map).unwrap()[0].kind,
            super::DoomSegOccluderKind::OpeningClosed
        );
        map.sectors[1].ceiling_height = 128;
        assert_eq!(
            observe_doom_seg_occluders(&map).unwrap()[0].kind,
            super::DoomSegOccluderKind::BackSectorClosed
        );
    }

    #[test]
    fn observes_plane_mark_eligibility_before_column_clipping() {
        let mut one_sided = map_with_linedef(Some(0), None);
        one_sided.segs = vec![seg(4, 0, 1)];
        let mark = observe_doom_seg_plane_marks(&one_sided, 36).unwrap()[0];
        assert!(mark.floor_marked);
        assert!(mark.ceiling_marked);
        assert_eq!(mark.back_sector, None);

        let mut two_sided = map_with_linedef(Some(0), Some(1));
        two_sided.segs = vec![seg(5, 0, 1)];
        let mark = observe_doom_seg_plane_marks(&two_sided, 36).unwrap()[0];
        assert!(!mark.floor_marked);
        assert!(!mark.ceiling_marked);

        two_sided.sectors[1].floor_height = 8;
        let mark = observe_doom_seg_plane_marks(&two_sided, 36).unwrap()[0];
        assert!(mark.floor_marked);
        assert!(!mark.ceiling_marked);

        two_sided.sectors[0].ceiling_texture = "F_SKY1".to_owned();
        two_sided.sectors[1].ceiling_texture = "F_SKY1".to_owned();
        two_sided.sectors[1].ceiling_height = 160;
        let mark = observe_doom_seg_plane_marks(&two_sided, 36).unwrap()[0];
        assert!(mark.paired_sky_ceiling_adjustment);
        assert!(!mark.ceiling_marked);
    }

    #[test]
    fn lowers_left_sided_wall_with_the_forward_source_u_axis() {
        let mut map = map_with_linedef(None, Some(0));
        map.sidedefs[0].x_offset = 7;
        map.sidedefs[0].middle_texture = "WALL".to_owned();

        let triangles = lower_doom_textured_wall_triangles(
            &map,
            &[DoomTextureExtent {
                name: "WALL".to_owned(),
                width: 64,
                height: 128,
            }],
        )
        .unwrap();

        let line_length = (800.0_f64).sqrt();
        let texture_u_at = |position: [f64; 3]| {
            triangles
                .iter()
                .flat_map(|triangle| triangle.positions.iter().zip(triangle.texture_coordinates))
                .find_map(|(candidate, coordinate)| {
                    (*candidate == position).then_some(coordinate[0])
                })
                .expect("both linedef endpoints occur in the lowered wall")
        };

        // In Tokimu's lifted 3D frame, a left/back sidedef advances in the
        // stored linedef direction, preserving its horizontal screen axis.
        assert_eq!(texture_u_at([10.0, 0.0, 20.0]), 7.0);
        assert_eq!(texture_u_at([30.0, 0.0, 40.0]), 7.0 + line_length);
    }

    #[test]
    fn one_sided_wall_ignores_inactive_tiers_and_resolves_texture_case_insensitively() {
        let mut map = map_with_linedef(Some(0), None);
        map.sidedefs[0].upper_texture = "INACTIVE_UPPER".to_owned();
        map.sidedefs[0].lower_texture = "INACTIVE_LOWER".to_owned();
        map.sidedefs[0].middle_texture = "wall".to_owned();

        let axes = observe_doom_wall_texture_axes(&map).unwrap();
        assert_eq!(axes.len(), 1);
        assert_eq!(axes[0].role, DoomWallTextureRole::Middle);

        let triangles = lower_doom_textured_wall_triangles(
            &map,
            &[DoomTextureExtent {
                name: "WALL".to_owned(),
                width: 64,
                height: 128,
            }],
        )
        .unwrap();
        assert_eq!(triangles.len(), 2);
        assert!(triangles
            .iter()
            .all(|triangle| triangle.texture_name == "wall"));
    }

    #[test]
    fn right_and_left_sidedefs_retain_opposed_source_u_axes() {
        let mut map = map_with_linedef(Some(0), Some(1));
        map.sidedefs[0].x_offset = 7;
        map.sidedefs[0].middle_texture = "RIGHT_LABEL".to_owned();
        map.sidedefs[1].x_offset = 11;
        map.sidedefs[1].middle_texture = "LEFT_LABEL".to_owned();

        let axes = observe_doom_wall_texture_axes(&map).unwrap();
        let right = axes
            .iter()
            .find(|axis| axis.texture_name == "RIGHT_LABEL")
            .unwrap();
        let left = axes
            .iter()
            .find(|axis| axis.texture_name == "LEFT_LABEL")
            .unwrap();
        let length = (800.0_f64).sqrt();

        assert_eq!(right.side, DoomWallSideKind::Right);
        assert_eq!(right.u_start, 7.0 + length);
        assert_eq!(right.u_end, 7.0);
        assert_eq!(left.side, DoomWallSideKind::Left);
        assert_eq!(left.u_start, 11.0);
        assert_eq!(left.u_end, 11.0 + length);
    }

    #[test]
    fn lowers_two_sided_upper_and_lower_height_bands() {
        let mut map = map_with_linedef(Some(0), Some(1));
        map.sectors[1].floor_height = 32;
        map.sectors[1].ceiling_height = 96;

        let triangles = lower_doom_two_sided_wall_bands(&map).unwrap();
        assert_eq!(triangles.len(), 4);
        assert!(triangles
            .iter()
            .all(|triangle| triangle.side == DoomWallSideKind::Right));
        assert_eq!(
            triangles
                .iter()
                .filter(|triangle| triangle.band == DoomWallBand::Upper)
                .count(),
            2
        );
        assert!(triangles
            .iter()
            .all(|triangle| triangle.source_linedef.record_index == 0));
        assert!(triangles
            .iter()
            .all(|triangle| triangle.source_sidedef.record_index == 0));
        assert!(triangles
            .iter()
            .all(|triangle| triangle.source_sector.record_index == 0));
        assert_eq!(
            triangles
                .iter()
                .filter(|triangle| triangle.band == DoomWallBand::Lower)
                .count(),
            2
        );
    }

    #[test]
    fn omits_upper_band_between_adjacent_classic_sky_ceilings() {
        let mut map = map_with_linedef(Some(0), Some(1));
        map.sectors[0].ceiling_texture = "F_SKY1".to_owned();
        map.sectors[1].ceiling_texture = "F_SKY1".to_owned();
        map.sectors[1].floor_height = 32;
        map.sectors[1].ceiling_height = 96;

        let triangles = lower_doom_two_sided_wall_bands(&map).unwrap();

        assert_eq!(triangles.len(), 2);
        assert!(triangles
            .iter()
            .all(|triangle| triangle.band == DoomWallBand::Lower));
    }

    #[test]
    fn retains_depth_boundary_between_adjacent_classic_sky_ceilings() {
        let mut map = map_with_linedef(Some(0), Some(1));
        map.sectors[0].ceiling_texture = "F_SKY1".to_owned();
        map.sectors[1].ceiling_texture = "F_SKY1".to_owned();
        map.sectors[1].ceiling_height = 96;

        let triangles = lower_doom_paired_sky_boundary_triangles(&map).unwrap();

        assert_eq!(triangles.len(), 2);
        assert!(triangles
            .iter()
            .all(|triangle| triangle.side == DoomWallSideKind::Right));
        assert!(triangles
            .iter()
            .flat_map(|triangle| triangle.positions)
            .all(|position| position[1] == 96.0 || position[1] == 128.0));
    }

    #[test]
    fn does_not_invent_paired_sky_boundary_without_height_difference() {
        let mut map = map_with_linedef(Some(0), Some(1));
        map.sectors[0].ceiling_texture = "F_SKY1".to_owned();
        map.sectors[1].ceiling_texture = "F_SKY1".to_owned();

        assert!(lower_doom_paired_sky_boundary_triangles(&map)
            .unwrap()
            .is_empty());

        map.sectors[1].ceiling_height = 96;
        map.sectors[1].ceiling_texture = "CEIL1_1".to_owned();
        assert!(lower_doom_paired_sky_boundary_triangles(&map)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn retains_upper_band_when_only_one_ceiling_is_classic_sky() {
        let mut map = map_with_linedef(Some(0), Some(1));
        map.sectors[0].ceiling_texture = "F_SKY1".to_owned();
        map.sectors[1].ceiling_height = 96;

        let triangles = lower_doom_two_sided_wall_bands(&map).unwrap();

        assert_eq!(triangles.len(), 2);
        assert!(triangles
            .iter()
            .all(|triangle| triangle.band == DoomWallBand::Upper));
    }

    #[test]
    fn observes_authored_two_sided_middle_texture_without_lowering_it() {
        let mut map = map_with_linedef(Some(0), Some(1));
        map.sidedefs[0].middle_texture = "MIDTEX".to_owned();

        let observations = observe_doom_two_sided_middle_textures(&map).unwrap();
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].side, DoomWallSideKind::Right);
        assert_eq!(observations[0].texture_name, "MIDTEX");
        assert_eq!(observations[0].opening_floor, 0);
        assert_eq!(observations[0].opening_ceiling, 128);
    }

    #[test]
    fn lowers_authored_two_sided_middle_texture_to_shared_opening() {
        let mut map = map_with_linedef(Some(0), Some(1));
        map.sidedefs[0].middle_texture = "MIDTEX".to_owned();
        map.sectors[1].floor_height = 32;
        map.sectors[1].ceiling_height = 96;

        let triangles = lower_doom_two_sided_middle_walls(&map).unwrap();
        assert_eq!(triangles.len(), 2);
        assert!(triangles
            .iter()
            .all(|triangle| triangle.texture_name == "MIDTEX"));
        assert!(triangles
            .iter()
            .all(|triangle| { triangle.opening_floor == 32 && triangle.opening_ceiling == 96 }));
        assert!(triangles.iter().all(|triangle| triangle
            .positions
            .iter()
            .all(|position| position[1] == 32.0 || position[1] == 96.0)));
    }

    #[test]
    fn observes_f_sky1_without_assigning_mesh_behavior() {
        let mut map = map_with_linedef(Some(0), None);
        map.vertices = vec![
            DoomVertex {
                source: source(0),
                x: 0,
                y: 0,
            },
            DoomVertex {
                source: source(1),
                x: 100,
                y: 0,
            },
            DoomVertex {
                source: source(2),
                x: 100,
                y: 100,
            },
            DoomVertex {
                source: source(3),
                x: 0,
                y: 100,
            },
        ];
        map.segs = vec![seg(0, 0, 1), seg(1, 1, 2)];
        map.subsectors = vec![
            DoomSubsector {
                source: source(0),
                seg_count: 1,
                first_seg: 0,
            },
            DoomSubsector {
                source: source(1),
                seg_count: 1,
                first_seg: 1,
            },
        ];
        map.nodes = vec![DoomNode {
            source: source(0),
            x: 50,
            y: 0,
            delta_x: 0,
            delta_y: 100,
            right_bbox: [0; 4],
            left_bbox: [0; 4],
            right_child: DoomBspChild::Subsector(0),
            left_child: DoomBspChild::Subsector(1),
        }];
        map.sectors[0].ceiling_texture = "F_SKY1".to_owned();

        let paths = resolve_doom_subsector_bsp_paths(&map).unwrap();
        let sky = observe_doom_sky_surfaces(&map, &paths).unwrap();
        assert_eq!(sky.len(), 4);
        assert!(sky
            .iter()
            .all(|observation| observation.plane == DoomSurfacePlane::Ceiling));
    }

    #[test]
    fn retains_raw_sidedef_texture_axis_without_pegging() {
        let mut map = map_with_linedef(Some(0), None);
        map.sidedefs[0].middle_texture = "WALL".to_owned();
        map.sidedefs[0].x_offset = 7;
        map.sidedefs[0].y_offset = -3;
        map.linedefs[0].flags = 0x0018;

        let axes = observe_doom_wall_texture_axes(&map).unwrap();
        assert_eq!(axes.len(), 1);
        assert_eq!(axes[0].texture_name, "WALL");
        assert_eq!(axes[0].u_start, 7.0 + (800.0_f64).sqrt());
        assert_eq!(axes[0].u_end, 7.0);
        assert_eq!(axes[0].v_offset, -3);
        assert_eq!(axes[0].linedef_flags, 0x0018);
        let audit = audit_doom_pegging_flags(&map).unwrap();
        assert_eq!(audit.upper_axes, 0);
        assert_eq!(audit.lower_axes, 0);
    }

    #[test]
    fn resolves_texture_extent_without_admitting_pegging() {
        let mut map = map_with_linedef(Some(0), None);
        map.sidedefs[0].middle_texture = "WALL".to_owned();
        let bindings = resolve_doom_wall_texture_bindings(
            &map,
            &[DoomTextureExtent {
                name: "WALL".to_owned(),
                width: 64,
                height: 128,
            }],
        )
        .unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].texture_width, 64);
        assert_eq!(bindings[0].texture_height, 128);
    }

    #[test]
    fn far_first_control_exposes_why_near_first_is_a_doom_provider_invariant() {
        let map = near_solid_far_bsp_map();
        let watched = BTreeSet::from([1]);
        let near_first =
            observe_doom_classic_bsp(&map, [0, -96], std::f64::consts::FRAC_PI_2, &watched)
                .unwrap();
        let far_first = observe_doom_classic_bsp_far_first_control(
            &map,
            [0, -96],
            std::f64::consts::FRAC_PI_2,
            &watched,
        )
        .unwrap();

        // The production traversal visits the near solid first and can prove
        // that the watched far leaf is fully covered. The test-only reversed
        // traversal admits the far SEG before the same range exists. This is
        // a deliberately explained coverage difference, not a selectable
        // renderer or application policy.
        assert_eq!(near_first.admitted_seg_order, vec![0]);
        assert_eq!(near_first.far_children_pruned, 1);
        assert_eq!(near_first.solid_range_events.len(), 1);
        let covering_event = &near_first.solid_range_events[0];
        assert_eq!(covering_event.event_ordinal, 0);
        assert_eq!(covering_event.source_seg, 0);
        assert_eq!(covering_event.source_linedef, 0);
        assert!(!covering_event.fully_covered_before);
        assert_eq!(covering_event.contributing_source_segs, BTreeSet::from([0]));
        assert_eq!(
            covering_event.contributing_source_linedefs,
            BTreeSet::from([0])
        );

        assert_eq!(near_first.watched_elision_provenance.len(), 1);
        let watched_elision = &near_first.watched_elision_provenance[0];
        assert_eq!(watched_elision.event_ordinal, 0);
        assert_eq!(watched_elision.reason, "solid-range");
        assert_eq!(watched_elision.subsectors, vec![1]);
        assert_eq!(watched_elision.covering_source_segs, BTreeSet::from([0]));
        assert_eq!(
            watched_elision.covering_source_linedefs,
            BTreeSet::from([0])
        );

        assert_eq!(far_first.admitted_seg_order, vec![1, 0]);
        assert_eq!(far_first.far_children_pruned, 0);
        assert!(far_first.admitted_seg_records.contains(&1));

        let without_solid_pruning = observe_doom_classic_bsp_without_solid_range_pruning(
            &map,
            [0, -96],
            std::f64::consts::FRAC_PI_2,
            &watched,
        )
        .unwrap();
        assert!(without_solid_pruning.visited_subsectors.contains(&1));

        let suppressed_near_solid = observe_doom_classic_bsp_suppressing_solid_range_source_seg(
            &map,
            [0, -96],
            std::f64::consts::FRAC_PI_2,
            &watched,
            0,
        )
        .unwrap();
        assert!(suppressed_near_solid.visited_subsectors.contains(&1));
        assert_eq!(
            suppressed_near_solid.suppressed_solid_range_mutations,
            vec![DoomClassicSuppressedSolidRangeMutation {
                source_seg: 0,
                source_linedef: 0,
                input_interval: [0, 319],
            }]
        );
    }

    #[test]
    fn reconstructs_one_retained_ordered_cell_as_source_labelled_triangles() {
        let mut map = near_solid_far_bsp_map();
        map.sidedefs[0].middle_texture = "WALL".to_owned();
        let extents = [DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 128,
            height: 128,
        }];
        let source_triangles = lower_doom_seg_textured_wall_triangles(&map, &extents).unwrap();
        let observation = DoomSegClassicVerticalClipObservation {
            ordered_wall_intervals: vec![DoomOrderedWallInterval {
                source_seg: 0,
                source_linedef: 0,
                column: 160,
                role: DoomWallTextureRole::Middle,
                raw_interval: [40, 159],
                open_interval_before: Some([0, 199]),
                retained_interval: Some([40, 159]),
            }],
            ..DoomSegClassicVerticalClipObservation::default()
        };

        let reconstruction = reconstruct_doom_ordered_wall_fragments(
            &map,
            &source_triangles,
            &observation,
            [0, -96],
            std::f64::consts::FRAC_PI_2,
            41.0,
        );

        assert_eq!(reconstruction.retained_cells, 1);
        assert_eq!(reconstruction.degenerate_cells, 0);
        assert_eq!(reconstruction.unresolved_cells, 0);
        assert_eq!(reconstruction.reconstructed_triangles.len(), 2);
        assert!(reconstruction
            .reconstructed_triangles
            .iter()
            .all(|triangle| triangle.source_seg.record_index == 0
                && triangle.source_linedef.record_index == 0
                && triangle.source_sidedef.record_index == 0
                && triangle.texture_name == "WALL"));
        assert!(reconstruction
            .reconstructed_triangles
            .iter()
            .flat_map(|triangle| triangle.texture_coordinates)
            .all(|uv| uv.into_iter().all(f64::is_finite)));
    }

    #[test]
    fn off_center_wall_cell_round_trips_through_rectilinear_projection() {
        let mut map = near_solid_far_bsp_map();
        map.sidedefs[0].middle_texture = "WALL".to_owned();
        let extents = [DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 128,
            height: 128,
        }];
        let source_triangles = lower_doom_seg_textured_wall_triangles(&map, &extents).unwrap();
        let observation = DoomSegClassicVerticalClipObservation {
            ordered_wall_intervals: vec![DoomOrderedWallInterval {
                source_seg: 0,
                source_linedef: 0,
                column: 80,
                role: DoomWallTextureRole::Middle,
                raw_interval: [40, 159],
                open_interval_before: Some([0, 199]),
                retained_interval: Some([40, 159]),
            }],
            ..DoomSegClassicVerticalClipObservation::default()
        };

        let reconstruction = reconstruct_doom_ordered_wall_fragments(
            &map,
            &source_triangles,
            &observation,
            [0, -96],
            std::f64::consts::FRAC_PI_2,
            41.0,
        );
        let half_vertical_fov = ((200.0_f64 / 320.0) * std::f64::consts::FRAC_PI_4.tan()).atan();

        for triangle in &reconstruction.reconstructed_triangles {
            for world in triangle.positions {
                let (source, height) = tokimu_point_to_doom(world);
                let relative = [source[0], source[1] + 96.0];
                let forward_depth = relative[1];
                let lateral = -relative[0];
                let column = ((lateral / forward_depth + 1.0) * 0.5) * 320.0;
                let row =
                    (1.0 - (height - 41.0) / forward_depth / half_vertical_fov.tan()) * 0.5 * 200.0;

                assert!((column - 80.0).abs() < 1.0e-9 || (column - 81.0).abs() < 1.0e-9);
                assert!((row - 40.0).abs() < 1.0e-9 || (row - 160.0).abs() < 1.0e-9);
            }
        }
    }

    #[test]
    fn retains_zero_height_ordered_cells_as_explicit_degenerate_omissions() {
        let mut map = near_solid_far_bsp_map();
        map.sidedefs[0].middle_texture = "WALL".to_owned();
        map.sectors[0].ceiling_height = map.sectors[0].floor_height;
        let extents = [DoomTextureExtent {
            name: "WALL".to_owned(),
            width: 128,
            height: 128,
        }];
        let source_triangles = lower_doom_seg_textured_wall_triangles(&map, &extents).unwrap();
        let observation = DoomSegClassicVerticalClipObservation {
            ordered_wall_intervals: vec![DoomOrderedWallInterval {
                source_seg: 0,
                source_linedef: 0,
                column: 160,
                role: DoomWallTextureRole::Middle,
                raw_interval: [100, 100],
                open_interval_before: Some([0, 199]),
                retained_interval: Some([100, 100]),
            }],
            ..DoomSegClassicVerticalClipObservation::default()
        };

        let reconstruction = reconstruct_doom_ordered_wall_fragments(
            &map,
            &source_triangles,
            &observation,
            [0, -96],
            std::f64::consts::FRAC_PI_2,
            41.0,
        );

        assert_eq!(reconstruction.retained_cells, 1);
        assert_eq!(reconstruction.degenerate_cells, 1);
        assert_eq!(reconstruction.unresolved_cells, 0);
        assert!(reconstruction.reconstructed_triangles.is_empty());
        assert!(reconstruction.samples[0].contains("zero-height-source-tier"));
    }

    fn map_with_linedef(right_sidedef: Option<u16>, left_sidedef: Option<u16>) -> DoomMapCore {
        let source = |record_index| DoomSourceRecord {
            lump_index: 1,
            record_index,
        };
        DoomMapCore {
            map_name: "TEST".to_owned(),
            things: Vec::new(),
            vertices: vec![
                DoomVertex {
                    source: source(0),
                    x: 10,
                    y: 20,
                },
                DoomVertex {
                    source: source(1),
                    x: 30,
                    y: 40,
                },
            ],
            linedefs: vec![DoomLinedef {
                source: source(0),
                start_vertex: 0,
                end_vertex: 1,
                flags: 0,
                special: 0,
                tag: 0,
                right_sidedef,
                left_sidedef,
            }],
            sidedefs: vec![
                DoomSidedef {
                    source: source(0),
                    x_offset: 0,
                    y_offset: 0,
                    upper_texture: "-".to_owned(),
                    lower_texture: "-".to_owned(),
                    middle_texture: "-".to_owned(),
                    sector: 0,
                },
                DoomSidedef {
                    source: source(1),
                    x_offset: 0,
                    y_offset: 0,
                    upper_texture: "-".to_owned(),
                    lower_texture: "-".to_owned(),
                    middle_texture: "-".to_owned(),
                    sector: 1,
                },
            ],
            sectors: vec![sector(source(0)), sector(source(1))],
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

    fn near_solid_far_bsp_map() -> DoomMapCore {
        let mut map = map_with_linedef(Some(0), None);
        map.vertices = vec![
            DoomVertex {
                source: source(0),
                x: -128,
                y: 0,
            },
            DoomVertex {
                source: source(1),
                x: 128,
                y: 0,
            },
            DoomVertex {
                source: source(2),
                x: -24,
                y: 64,
            },
            DoomVertex {
                source: source(3),
                x: 24,
                y: 64,
            },
        ];
        map.linedefs = vec![
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
                right_sidedef: Some(0),
                left_sidedef: None,
            },
        ];
        map.segs = vec![
            DoomSeg {
                source: source(0),
                start_vertex: 0,
                end_vertex: 1,
                angle: 0,
                linedef: 0,
                direction: 0,
                offset: 0,
            },
            DoomSeg {
                source: source(1),
                start_vertex: 2,
                end_vertex: 3,
                angle: 0,
                linedef: 1,
                direction: 0,
                offset: 0,
            },
        ];
        map.subsectors = vec![
            DoomSubsector {
                source: source(0),
                first_seg: 0,
                seg_count: 1,
            },
            DoomSubsector {
                source: source(1),
                first_seg: 1,
                seg_count: 1,
            },
        ];
        map.nodes = vec![DoomNode {
            source: source(0),
            x: 0,
            y: 0,
            delta_x: 0,
            delta_y: 64,
            right_bbox: [64, 64, 24, -24],
            left_bbox: [0, 0, 128, -128],
            right_child: DoomBspChild::Subsector(1),
            left_child: DoomBspChild::Subsector(0),
        }];
        map
    }

    fn sector(source: DoomSourceRecord) -> DoomSector {
        DoomSector {
            source,
            floor_height: 0,
            ceiling_height: 128,
            floor_texture: "FLOOR0_1".to_owned(),
            ceiling_texture: "CEIL1_1".to_owned(),
            light_level: 160,
            special: 0,
            tag: 0,
        }
    }

    fn source(record_index: u32) -> DoomSourceRecord {
        DoomSourceRecord {
            lump_index: 1,
            record_index,
        }
    }

    fn assert_normal_direction(positions: [[f64; 3]; 3], expected: [f64; 3]) {
        let first_edge = subtract(positions[1], positions[0]);
        let second_edge = subtract(positions[2], positions[0]);
        let normal = [
            first_edge[1] * second_edge[2] - first_edge[2] * second_edge[1],
            first_edge[2] * second_edge[0] - first_edge[0] * second_edge[2],
            first_edge[0] * second_edge[1] - first_edge[1] * second_edge[0],
        ];
        let alignment = normal[0] * expected[0] + normal[1] * expected[1] + normal[2] * expected[2];
        assert!(
            alignment > 0.0,
            "normal {normal:?} did not face {expected:?}"
        );
    }

    fn subtract(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
        [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
    }

    fn seg(record_index: u32, start_vertex: u16, end_vertex: u16) -> DoomSeg {
        DoomSeg {
            source: source(record_index),
            start_vertex,
            end_vertex,
            angle: 0,
            linedef: 0,
            direction: 0,
            offset: 0,
        }
    }
}
