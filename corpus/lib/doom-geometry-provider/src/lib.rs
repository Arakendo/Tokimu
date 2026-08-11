//! Headless source-geometry observations for classic Doom maps.
//!
//! This provider resolves map-table references only. Mesh construction,
//! materials, renderer resources, and WAD acquisition remain outside it.

use std::collections::BTreeMap;

use doom_map_provider::{DoomBspChild, DoomMapCore, DoomSourceRecord};
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
/// UV mapping. Middle textures, pegging, and portals are deliberately deferred.
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
        if right_sector.ceiling_height > left_sector.ceiling_height {
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
        if left_sector.ceiling_height > right_sector.ceiling_height {
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
    Ok(lower_doom_subsector_surfaces(map, paths)?
        .into_iter()
        .filter(|triangle| triangle.texture_name == "F_SKY1")
        .map(|triangle| DoomSkySurfaceObservation {
            source_subsector: triangle.source_subsector,
            source_sector: triangle.source_sector,
            plane: triangle.plane,
            texture_name: triangle.texture_name,
        })
        .collect())
}

/// Retains raw sidedef texture axes before any Doom pegging policy is applied.
pub fn observe_doom_wall_texture_axes(
    map: &DoomMapCore,
) -> Result<Vec<DoomWallTextureAxisObservation>, DoomGeometryError> {
    let candidates = resolve_doom_wall_candidates(map)?;
    let mut observations = Vec::new();
    for candidate in candidates {
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
                if texture_name != "-" {
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
        if by_name.insert(extent.name.as_str(), extent).is_some() {
            return Err(DoomGeometryError::DuplicateTextureExtent {
                name: extent.name.clone(),
            });
        }
    }
    observe_doom_wall_texture_axes(map)?
        .into_iter()
        .map(|axis| {
            let extent = by_name.get(axis.texture_name.as_str()).ok_or(
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
    use doom_map_provider::{
        DoomBlockmapObservation, DoomBspChild, DoomLinedef, DoomMapCore, DoomNode,
        DoomRejectMatrix, DoomSector, DoomSeg, DoomSidedef, DoomSourceRecord, DoomSubsector,
        DoomVertex,
    };

    use super::{
        audit_doom_pegging_flags, audit_doom_subsector_bsp_paths,
        audit_doom_subsector_loop_closure, audit_doom_vertical_topology, audit_doom_wall_topology,
        doom_direction_to_tokimu, doom_point_to_tokimu, locate_doom_point_subsector,
        lower_doom_one_sided_walls, lower_doom_subsector_surfaces,
        lower_doom_textured_wall_triangles, lower_doom_two_sided_middle_walls,
        lower_doom_two_sided_wall_bands, observe_doom_sky_surfaces,
        observe_doom_two_sided_middle_textures, observe_doom_wall_texture_axes,
        resolve_doom_linedef_subsector_membership, resolve_doom_subsector_bsp_paths,
        resolve_doom_subsector_loops, resolve_doom_subsector_regions,
        resolve_doom_subsector_sector_ownership, resolve_doom_wall_candidates,
        resolve_doom_wall_texture_bindings, tokimu_direction_to_doom, tokimu_point_to_doom,
        DoomBspSide, DoomGeometryError, DoomLinedefSubsectorMembership, DoomSurfacePlane,
        DoomTextureExtent, DoomWallBand, DoomWallSideKind,
    };

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
