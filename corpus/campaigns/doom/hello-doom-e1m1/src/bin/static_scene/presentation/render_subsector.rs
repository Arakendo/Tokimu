//! Doom-private persistent render-subsector geometry for the AR-0030 study.
//!
//! This module constructs source-attributed finite world-space surfaces. It
//! does not decide view participation and exports no renderer or stable engine
//! contract.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use doom_geometry_provider::{
    doom_point_to_tokimu, lower_doom_seg_textured_wall_triangles, observe_doom_seg_occluders,
    resolve_doom_subsector_bsp_paths, resolve_doom_subsector_regions,
    resolve_doom_subsector_sector_ownership, resolve_doom_viewer_subsector_order,
    DoomSegOccluderKind, DoomSegTexturedWallTriangle, DoomSurfacePlane, DoomSurfaceTriangle,
    DoomTextureExtent, DoomWallTextureRole,
};
use doom_map_provider::{DoomMapCore, DoomSourceRecord};
use hello_doom_e1m1::{
    classify_static_draw_frustum_rejection, lower_static_flat_triangle,
    lower_static_seg_wall_triangle, FlatExtent, StaticDrawAabb, StaticDrawFrustumRejection,
    StaticDrawPlanEntry, StaticDrawSource, StaticTextureSourceKind, StaticTextureUpload,
};
use tokimu_core::math::{try_projection_perspective_rh_gl, try_view_look_at_rh, Vec3, Vec4};

const DOMAIN_EPSILON: f64 = 1.0e-7;
const FINGERPRINT_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FINGERPRINT_PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RenderSubsectorBoundaryAuthority {
    OrderedSegLoop,
    OrderedSegLoopRefinesBspPath,
    BspPathImplicitBoundary,
    UnresolvedDomainMismatch,
}

impl RenderSubsectorBoundaryAuthority {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::OrderedSegLoop => "ordered-seg-loop",
            Self::OrderedSegLoopRefinesBspPath => "ordered-seg-loop-refines-bsp-path",
            Self::BspPathImplicitBoundary => "bsp-path-implicit-boundary",
            Self::UnresolvedDomainMismatch => "unresolved-domain-mismatch",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RenderSubsectorPlaneRole {
    Ordinary,
    Sky,
}

impl RenderSubsectorPlaneRole {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Ordinary => "ordinary",
            Self::Sky => "sky",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderSubsectorTriangle {
    pub(crate) source_subsector: DoomSourceRecord,
    pub(crate) source_sector: DoomSourceRecord,
    pub(crate) plane: DoomSurfacePlane,
    pub(crate) role: RenderSubsectorPlaneRole,
    pub(crate) texture_name: String,
    pub(crate) positions: [[f64; 3]; 3],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RenderSubsectorWallSource {
    pub(crate) source_seg: DoomSourceRecord,
    pub(crate) source_linedef: DoomSourceRecord,
    pub(crate) source_sidedef: DoomSourceRecord,
    pub(crate) source_sector: DoomSourceRecord,
    pub(crate) direction: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderSubsector {
    pub(crate) source_subsector: DoomSourceRecord,
    pub(crate) source_sector: DoomSourceRecord,
    pub(crate) render_sector: DoomSourceRecord,
    pub(crate) sector_index: u16,
    pub(crate) boundary_authority: RenderSubsectorBoundaryAuthority,
    pub(crate) ordered_seg_gaps: usize,
    pub(crate) boundary: Vec<[f64; 2]>,
    pub(crate) boundary_fingerprint: u64,
    pub(crate) wall_sources: Vec<RenderSubsectorWallSource>,
    pub(crate) wall_fingerprint: u64,
    pub(crate) wall_tier_triangles: Vec<DoomSegTexturedWallTriangle>,
    pub(crate) wall_tier_fingerprint: u64,
    pub(crate) floor_height: i16,
    pub(crate) ceiling_height: i16,
    pub(crate) runtime_height_revision: u64,
    pub(crate) floor_role: RenderSubsectorPlaneRole,
    pub(crate) ceiling_role: RenderSubsectorPlaneRole,
    pub(crate) triangles: Vec<RenderSubsectorTriangle>,
    pub(crate) triangle_fingerprint: u64,
    pub(crate) unresolved_reason: Option<&'static str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RenderSubsectorPreparedViewIdentity {
    pub(crate) map_fingerprint: u64,
    pub(crate) camera_fingerprint: u64,
    pub(crate) runtime_height_fingerprint: u64,
    pub(crate) prepared_view_fingerprint: u64,
    pub(crate) viewport: [u32; 2],
    pub(crate) vertical_fov_degrees_bits: u32,
    pub(crate) pitch_degrees_bits: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderSubsectorInventory {
    pub(crate) strategy: &'static str,
    pub(crate) identity: RenderSubsectorPreparedViewIdentity,
    pub(crate) source_subsectors: usize,
    pub(crate) subsectors: Vec<RenderSubsector>,
    pub(crate) ordered_seg_loops: usize,
    pub(crate) ordered_seg_refinements: usize,
    pub(crate) bsp_path_boundaries: usize,
    pub(crate) unresolved_boundaries: usize,
    pub(crate) source_plane_units: usize,
    pub(crate) represented_plane_units: usize,
    pub(crate) triangles: usize,
    pub(crate) containment_failures: usize,
    pub(crate) winding_failures: usize,
    pub(crate) degenerate_triangles: usize,
    pub(crate) source_wall_segs: usize,
    pub(crate) represented_wall_segs: usize,
    pub(crate) source_wall_tier_triangles: usize,
    pub(crate) represented_wall_tier_triangles: usize,
    pub(crate) ordinary_plane_units: usize,
    pub(crate) sky_plane_units: usize,
    pub(crate) zero_clearance_subsectors: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RenderSubsectorViewPose {
    pub(crate) label: &'static str,
    pub(crate) source_position: [f64; 2],
    pub(crate) eye_height: f64,
    pub(crate) heading_degrees: f32,
    pub(crate) pitch_degrees: f32,
    pub(crate) viewport: [u32; 2],
    pub(crate) vertical_fov_degrees: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RenderSubsectorShadowDisposition {
    RetainedGeometry,
    OutsideFrustum,
    Unresolved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RenderSubsectorSurfaceShadowDisposition {
    RetainedGeometry,
    OutsideFrustum,
    SourceCovered,
    Unresolved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RenderSubsectorPlaneShadowEntry {
    pub(crate) source_subsector: DoomSourceRecord,
    pub(crate) plane: DoomSurfacePlane,
    pub(crate) disposition: RenderSubsectorSurfaceShadowDisposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RenderSubsectorWallTierShadowEntry {
    pub(crate) source_subsector: DoomSourceRecord,
    pub(crate) source_seg: DoomSourceRecord,
    pub(crate) wall_tier_ordinal: usize,
    pub(crate) role: DoomWallTextureRole,
    pub(crate) disposition: RenderSubsectorSurfaceShadowDisposition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RenderSubsectorShadowEntry {
    pub(crate) source_subsector: DoomSourceRecord,
    pub(crate) near_first_rank: usize,
    pub(crate) disposition: RenderSubsectorShadowDisposition,
    pub(crate) reason: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RenderSubsectorShadowObservation {
    pub(crate) label: &'static str,
    pub(crate) view_fingerprint: u64,
    pub(crate) near_first_fingerprint: u64,
    pub(crate) result_fingerprint: u64,
    pub(crate) entries: Vec<RenderSubsectorShadowEntry>,
    pub(crate) retained: usize,
    pub(crate) outside_frustum: usize,
    pub(crate) unresolved: usize,
    pub(crate) brute_retained: usize,
    pub(crate) false_negatives: usize,
    pub(crate) false_positives: usize,
    pub(crate) plane_retained: usize,
    pub(crate) plane_outside_frustum: usize,
    pub(crate) plane_source_covered: usize,
    pub(crate) plane_unresolved: usize,
    pub(crate) wall_tiers_retained: usize,
    pub(crate) wall_tiers_outside_frustum: usize,
    pub(crate) wall_tiers_source_covered: usize,
    pub(crate) wall_tiers_unresolved: usize,
    pub(crate) plane_horizontal_aabb_false_positives: usize,
    pub(crate) wall_horizontal_aabb_false_positives: usize,
    pub(crate) source_coverage_fingerprint: u64,
    pub(crate) unresolved_surface_samples: Vec<String>,
    pub(crate) plane_entries: Vec<RenderSubsectorPlaneShadowEntry>,
    pub(crate) wall_tier_entries: Vec<RenderSubsectorWallTierShadowEntry>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PreparedRenderSubsectorSurface {
    Plane(DoomSurfacePlane),
    WallTier {
        source_seg: DoomSourceRecord,
        role: DoomWallTextureRole,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PreparedRenderSubsectorDeclaration {
    pub(crate) source_subsector: DoomSourceRecord,
    pub(crate) source_triangle_ordinal: usize,
    pub(crate) surface: PreparedRenderSubsectorSurface,
    pub(crate) cutout: bool,
    pub(crate) draw: StaticDrawPlanEntry,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PreparedRenderSubsectorView {
    pub(crate) shadow: RenderSubsectorShadowObservation,
    pub(crate) declarations: Vec<PreparedRenderSubsectorDeclaration>,
    pub(crate) source_plane_triangles: usize,
    pub(crate) source_wall_tier_triangles: usize,
    pub(crate) ordinary_plane_declarations: usize,
    pub(crate) opaque_wall_declarations: usize,
    pub(crate) cutout_wall_declarations: usize,
    pub(crate) sky_background_triangles: usize,
    pub(crate) outside_frustum_triangles: usize,
    pub(crate) source_covered_triangles: usize,
    pub(crate) unresolved_fail_open_triangles: usize,
    pub(crate) declaration_fingerprint: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RenderSubsectorConnectivityRole {
    ImplicitPartition,
    ClosedSolid,
    PositiveOpening,
    MaskedMiddleOpening,
    PairedSkyOpening,
    UnresolvedFailOpen,
}

impl RenderSubsectorConnectivityRole {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::ImplicitPartition => "implicit-partition",
            Self::ClosedSolid => "closed-solid",
            Self::PositiveOpening => "positive-opening",
            Self::MaskedMiddleOpening => "masked-middle-opening",
            Self::PairedSkyOpening => "paired-sky-opening",
            Self::UnresolvedFailOpen => "unresolved-fail-open",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderSubsectorConnectivityEdge {
    pub(crate) from_subsector: u32,
    pub(crate) to_subsector: u32,
    pub(crate) source_linedef: Option<DoomSourceRecord>,
    pub(crate) role: RenderSubsectorConnectivityRole,
    pub(crate) shared_interval: [[f64; 2]; 2],
    pub(crate) opening_bottom: i16,
    pub(crate) opening_top: i16,
    pub(crate) runtime_height_revision: u64,
    pub(crate) reason: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderSubsectorConnectivityGraph {
    pub(crate) edges: Vec<RenderSubsectorConnectivityEdge>,
    pub(crate) adjacency: Vec<Vec<usize>>,
    pub(crate) role_counts: BTreeMap<&'static str, usize>,
    pub(crate) isolated_subsectors: usize,
    pub(crate) fingerprint: u64,
    pub(crate) aperture_fingerprint: u64,
    pub(crate) source_correlated_relationships: usize,
    pub(crate) traversable_relationships: usize,
    pub(crate) closed_relationships: usize,
    pub(crate) aperture_containment_failures: usize,
    pub(crate) zero_clearance_relationships: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RenderSubsectorConnectivityObservation {
    pub(crate) start_subsector: u32,
    pub(crate) sky_terminal: bool,
    pub(crate) reachable: BTreeSet<u32>,
    pub(crate) predecessor: Vec<Option<usize>>,
    pub(crate) traversed_edges: usize,
    pub(crate) terminal_closed_edges: usize,
    pub(crate) terminal_sky_edges: usize,
    pub(crate) fail_open_edges: usize,
    pub(crate) fingerprint: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DoomViewWindow {
    pub(crate) minimum_ndc: [f32; 2],
    pub(crate) maximum_ndc: [f32; 2],
    pub(crate) depth: [f32; 2],
}

impl DoomViewWindow {
    pub(crate) fn contains_sample(self, sample_ndc: [f32; 2]) -> bool {
        sample_ndc[0] >= self.minimum_ndc[0] - 1.0e-5
            && sample_ndc[0] <= self.maximum_ndc[0] + 1.0e-5
            && sample_ndc[1] >= self.minimum_ndc[1] - 1.0e-5
            && sample_ndc[1] <= self.maximum_ndc[1] + 1.0e-5
    }

    fn contains_window(self, other: Self) -> bool {
        other.minimum_ndc[0] >= self.minimum_ndc[0] - 1.0e-5
            && other.maximum_ndc[0] <= self.maximum_ndc[0] + 1.0e-5
            && other.minimum_ndc[1] >= self.minimum_ndc[1] - 1.0e-5
            && other.maximum_ndc[1] <= self.maximum_ndc[1] + 1.0e-5
    }

    fn intersect(self, other: Self) -> Option<Self> {
        let minimum_ndc = [
            self.minimum_ndc[0].max(other.minimum_ndc[0]),
            self.minimum_ndc[1].max(other.minimum_ndc[1]),
        ];
        let maximum_ndc = [
            self.maximum_ndc[0].min(other.maximum_ndc[0]),
            self.maximum_ndc[1].min(other.maximum_ndc[1]),
        ];
        if minimum_ndc[0] >= maximum_ndc[0] - 1.0e-6 || minimum_ndc[1] >= maximum_ndc[1] - 1.0e-6 {
            return None;
        }
        Some(Self {
            minimum_ndc,
            maximum_ndc,
            depth: [other.depth[0], other.depth[1]],
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DoomViewTransferState {
    pub(crate) cell: u32,
    pub(crate) window: DoomViewWindow,
    pub(crate) predecessor_state: Option<usize>,
    pub(crate) aperture_edge: Option<usize>,
    pub(crate) lineage_fingerprint: u64,
    pub(crate) near_plane_fail_open: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DoomViewTransferObservation {
    pub(crate) label: &'static str,
    pub(crate) start_cell: u32,
    pub(crate) sky_terminal: bool,
    pub(crate) states: Vec<DoomViewTransferState>,
    pub(crate) states_by_cell: Vec<Vec<usize>>,
    pub(crate) attempted_states: usize,
    pub(crate) dominated_states: usize,
    pub(crate) terminal_closed: usize,
    pub(crate) terminal_sky: usize,
    pub(crate) outside_view: usize,
    pub(crate) near_plane_fail_open: usize,
    pub(crate) maximum_depth: usize,
    pub(crate) repeated_destination_cells: usize,
    pub(crate) maximum_occurrences_per_cell: usize,
    pub(crate) fingerprint: u64,
}

impl PreparedRenderSubsectorView {
    pub(crate) fn verify_conservation(&self) -> Result<(), String> {
        let source = self.source_plane_triangles + self.source_wall_tier_triangles;
        let accounted = self.declarations.len()
            + self.sky_background_triangles
            + self.outside_frustum_triangles
            + self.source_covered_triangles;
        if source != accounted {
            return Err(format!(
                "prepared render-subsector conservation failed: source={source} accounted={accounted}"
            ));
        }
        if self.declarations.len()
            != self.ordinary_plane_declarations
                + self.opaque_wall_declarations
                + self.cutout_wall_declarations
        {
            return Err("prepared render-subsector declaration categories disagree".to_owned());
        }
        if self.unresolved_fail_open_triangles > self.declarations.len() {
            return Err(
                "prepared render-subsector unresolved fail-open count exceeds declarations"
                    .to_owned(),
            );
        }
        Ok(())
    }
}

impl RenderSubsectorInventory {
    pub(crate) fn conserved(&self) -> bool {
        self.subsectors.len() == self.source_subsectors
            && self.source_plane_units == self.represented_plane_units
            && self.source_wall_segs == self.represented_wall_segs
            && self.source_wall_tier_triangles == self.represented_wall_tier_triangles
            && self.unresolved_boundaries == 0
            && self.containment_failures == 0
            && self.winding_failures == 0
            && self.degenerate_triangles == 0
    }
}

#[allow(clippy::too_many_arguments)] // The study keeps each identity fact explicit instead of introducing a premature config contract.
pub(crate) fn build_render_subsector_inventory(
    map: &DoomMapCore,
    wall_extents: &[DoomTextureExtent],
    source_position: [i16; 2],
    source_angle: u16,
    eye_height: i16,
    viewport: [u32; 2],
    vertical_fov_degrees: f32,
    pitch_degrees: f32,
) -> Result<RenderSubsectorInventory, String> {
    let paths = resolve_doom_subsector_bsp_paths(map).map_err(|error| error.to_string())?;
    let regions = resolve_doom_subsector_regions(map, &paths).map_err(|error| error.to_string())?;
    let ownership =
        resolve_doom_subsector_sector_ownership(map).map_err(|error| error.to_string())?;
    let source_wall_tier_triangles = lower_doom_seg_textured_wall_triangles(map, wall_extents)
        .map_err(|error| error.to_string())?;
    if regions.len() != map.subsectors.len() || ownership.len() != map.subsectors.len() {
        return Err(format!(
            "render-subsector inputs disagree: source={} regions={} ownership={}",
            map.subsectors.len(),
            regions.len(),
            ownership.len()
        ));
    }

    let mut subsectors = Vec::with_capacity(map.subsectors.len());
    let mut ordered_seg_loops = 0;
    let mut ordered_seg_refinements = 0;
    let mut bsp_path_boundaries = 0;
    let mut unresolved_boundaries = 0;
    let mut represented_plane_units = 0;
    let mut triangles = 0;
    let mut containment_failures = 0;
    let mut winding_failures = 0;
    let mut degenerate_triangles = 0;
    let mut source_wall_segs = 0;
    let mut represented_wall_segs = 0;
    let mut represented_wall_tier_triangles = 0;
    let mut ordinary_plane_units = 0;
    let mut sky_plane_units = 0;
    let mut zero_clearance_subsectors = 0;

    for (subsector_index, ((source, region), owner)) in map
        .subsectors
        .iter()
        .zip(&regions)
        .zip(&ownership)
        .enumerate()
    {
        if source.source != region.source_subsector || source.source != owner.source_subsector {
            return Err(format!(
                "render-subsector {subsector_index} source correlation disagrees"
            ));
        }
        let first = usize::from(source.first_seg);
        let end = first + usize::from(source.seg_count);
        let segs = &map.segs[first..end];
        source_wall_segs += segs.len();
        let (ordered_loop, ordered_seg_gaps) = ordered_seg_loop(map, segs);
        let (boundary_authority, boundary, unresolved_reason) = match ordered_loop {
            Some(loop_vertices) if polygons_cover_same_domain(&loop_vertices, &region.vertices) => {
                ordered_seg_loops += 1;
                (
                    RenderSubsectorBoundaryAuthority::OrderedSegLoop,
                    loop_vertices,
                    None,
                )
            }
            Some(loop_vertices)
                if loop_vertices
                    .iter()
                    .all(|point| point_in_convex_domain(*point, &region.vertices)) =>
            {
                // A closed source SEG loop is a stronger finite surface domain
                // than unused partition space enclosed only by the BSP path.
                // The path still proves the loop belongs to this leaf.
                ordered_seg_refinements += 1;
                (
                    RenderSubsectorBoundaryAuthority::OrderedSegLoopRefinesBspPath,
                    loop_vertices,
                    None,
                )
            }
            Some(_) => {
                unresolved_boundaries += 1;
                (
                    RenderSubsectorBoundaryAuthority::UnresolvedDomainMismatch,
                    region.vertices.clone(),
                    Some("ordered-seg-loop-disagrees-with-bsp-path-domain"),
                )
            }
            None => {
                bsp_path_boundaries += 1;
                (
                    RenderSubsectorBoundaryAuthority::BspPathImplicitBoundary,
                    region.vertices.clone(),
                    None,
                )
            }
        };

        let sector = &map.sectors[usize::from(owner.sector_index)];
        let floor_role = plane_role(&sector.floor_texture);
        let ceiling_role = plane_role(&sector.ceiling_texture);
        for role in [floor_role, ceiling_role] {
            match role {
                RenderSubsectorPlaneRole::Ordinary => ordinary_plane_units += 1,
                RenderSubsectorPlaneRole::Sky => sky_plane_units += 1,
            }
        }
        if sector.floor_height >= sector.ceiling_height {
            zero_clearance_subsectors += 1;
        }
        let mut surface_triangles = if unresolved_reason.is_none() {
            triangulate_render_subsector(
                source.source,
                owner.source_sector,
                &boundary,
                sector.floor_height,
                sector.ceiling_height,
                &sector.floor_texture,
                &sector.ceiling_texture,
            )
        } else {
            Vec::new()
        };
        if unresolved_reason.is_none() {
            represented_plane_units += 2;
        }
        for triangle in &surface_triangles {
            let area = triangle_area(triangle.positions);
            if area <= DOMAIN_EPSILON {
                degenerate_triangles += 1;
            }
            if !triangle
                .positions
                .iter()
                .all(|position| point_in_convex_domain([position[0], position[2]], &boundary))
            {
                containment_failures += 1;
            }
            let normal_y = triangle_normal_y(triangle.positions);
            let winding_correct = match triangle.plane {
                DoomSurfacePlane::Floor => normal_y > DOMAIN_EPSILON,
                DoomSurfacePlane::Ceiling => normal_y < -DOMAIN_EPSILON,
            };
            if !winding_correct {
                winding_failures += 1;
            }
        }
        triangles += surface_triangles.len();

        let wall_sources = segs
            .iter()
            .map(|seg| {
                let linedef = &map.linedefs[usize::from(seg.linedef)];
                let sidedef_index = match seg.direction {
                    0 => linedef.right_sidedef,
                    1 => linedef.left_sidedef,
                    _ => None,
                }
                .ok_or_else(|| {
                    format!(
                        "render-subsector {subsector_index} SEG {} has no owning sidedef",
                        seg.source.record_index
                    )
                })?;
                let sidedef = &map.sidedefs[usize::from(sidedef_index)];
                let owning_sector = &map.sectors[usize::from(sidedef.sector)];
                if owning_sector.source != owner.source_sector {
                    return Err(format!(
                        "render-subsector {subsector_index} SEG {} sector disagrees with ownership",
                        seg.source.record_index
                    ));
                }
                Ok(RenderSubsectorWallSource {
                    source_seg: seg.source,
                    source_linedef: linedef.source,
                    source_sidedef: sidedef.source,
                    source_sector: owning_sector.source,
                    direction: seg.direction,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        represented_wall_segs += wall_sources.len();
        let wall_tier_triangles = source_wall_tier_triangles
            .iter()
            .filter(|triangle| segs.iter().any(|seg| seg.source == triangle.source_seg))
            .cloned()
            .collect::<Vec<_>>();
        if wall_tier_triangles
            .iter()
            .any(|triangle| triangle.source_sector != owner.source_sector)
        {
            return Err(format!(
                "render-subsector {subsector_index} wall tier sector disagrees with ownership"
            ));
        }
        represented_wall_tier_triangles += wall_tier_triangles.len();

        let boundary_fingerprint = fingerprint_boundary(source.source, &boundary);
        let wall_fingerprint = fingerprint_walls(&wall_sources);
        let wall_tier_fingerprint = fingerprint_wall_tiers(&wall_tier_triangles);
        let runtime_height_revision = fingerprint_runtime_height(
            owner.source_sector,
            sector.floor_height,
            sector.ceiling_height,
        );
        let triangle_fingerprint = fingerprint_triangles(&surface_triangles);
        subsectors.push(RenderSubsector {
            source_subsector: source.source,
            source_sector: owner.source_sector,
            render_sector: owner.source_sector,
            sector_index: owner.sector_index,
            boundary_authority,
            ordered_seg_gaps,
            boundary,
            boundary_fingerprint,
            wall_sources,
            wall_fingerprint,
            wall_tier_triangles,
            wall_tier_fingerprint,
            floor_height: sector.floor_height,
            ceiling_height: sector.ceiling_height,
            runtime_height_revision,
            floor_role,
            ceiling_role,
            triangles: std::mem::take(&mut surface_triangles),
            triangle_fingerprint,
            unresolved_reason,
        });
    }

    let map_fingerprint = fingerprint_map(map);
    let runtime_height_fingerprint = fingerprint_all_runtime_heights(map);
    let camera_fingerprint = fingerprint_camera(
        source_position,
        source_angle,
        eye_height,
        viewport,
        vertical_fov_degrees,
        pitch_degrees,
    );
    let mut prepared_view_fingerprint = FINGERPRINT_OFFSET;
    hash_u64(&mut prepared_view_fingerprint, map_fingerprint);
    hash_u64(&mut prepared_view_fingerprint, camera_fingerprint);
    hash_u64(&mut prepared_view_fingerprint, runtime_height_fingerprint);

    Ok(RenderSubsectorInventory {
        strategy: "render-subsector-actual-camera-shadow",
        identity: RenderSubsectorPreparedViewIdentity {
            map_fingerprint,
            camera_fingerprint,
            runtime_height_fingerprint,
            prepared_view_fingerprint,
            viewport,
            vertical_fov_degrees_bits: vertical_fov_degrees.to_bits(),
            pitch_degrees_bits: pitch_degrees.to_bits(),
        },
        source_subsectors: map.subsectors.len(),
        subsectors,
        ordered_seg_loops,
        ordered_seg_refinements,
        bsp_path_boundaries,
        unresolved_boundaries,
        source_plane_units: map.subsectors.len() * 2,
        represented_plane_units,
        triangles,
        containment_failures,
        winding_failures,
        degenerate_triangles,
        source_wall_segs,
        represented_wall_segs,
        source_wall_tier_triangles: source_wall_tier_triangles.len(),
        represented_wall_tier_triangles,
        ordinary_plane_units,
        sky_plane_units,
        zero_clearance_subsectors,
    })
}

pub(crate) fn build_render_subsector_connectivity_graph(
    map: &DoomMapCore,
    inventory: &RenderSubsectorInventory,
) -> Result<RenderSubsectorConnectivityGraph, String> {
    if inventory.subsectors.len() != map.subsectors.len() {
        return Err("render-subsector connectivity requires a complete inventory".to_owned());
    }
    let mut edges = Vec::new();
    let mut adjacency = vec![Vec::new(); inventory.subsectors.len()];
    let mut role_counts = BTreeMap::new();
    let mut fingerprint = FINGERPRINT_OFFSET;
    let mut aperture_fingerprint = FINGERPRINT_OFFSET;
    let mut source_correlated_relationships = 0;
    let mut traversable_relationships = 0;
    let mut closed_relationships = 0;
    let mut aperture_containment_failures = 0;
    let mut zero_clearance_relationships = 0;

    for left_index in 0..inventory.subsectors.len() {
        for right_index in left_index + 1..inventory.subsectors.len() {
            let left = &inventory.subsectors[left_index];
            let right = &inventory.subsectors[right_index];
            for left_edge in polygon_edges(&left.boundary) {
                for right_edge in polygon_edges(&right.boundary) {
                    let Some(shared_interval) = shared_collinear_interval(left_edge, right_edge)
                    else {
                        continue;
                    };
                    let mut linedefs = boundary_linedefs(map, left_index, shared_interval);
                    linedefs.extend(boundary_linedefs(map, right_index, shared_interval));
                    linedefs.sort_unstable();
                    linedefs.dedup();
                    let (source_linedef, role, reason) = match linedefs.as_slice() {
                        [] => (
                            None,
                            RenderSubsectorConnectivityRole::ImplicitPartition,
                            "shared-finite-boundary-without-authored-linedef",
                        ),
                        [linedef_index] => {
                            let linedef = &map.linedefs[*linedef_index];
                            let (role, reason) = classify_connectivity_linedef(map, linedef);
                            (Some(linedef.source), role, reason)
                        }
                        _ => (
                            None,
                            RenderSubsectorConnectivityRole::UnresolvedFailOpen,
                            "multiple-authored-linedefs-overlap-shared-boundary",
                        ),
                    };
                    let opening_bottom = left.floor_height.max(right.floor_height);
                    let opening_top = left.ceiling_height.min(right.ceiling_height);
                    let runtime_height_revision = left.runtime_height_revision.rotate_left(17)
                        ^ right.runtime_height_revision;
                    let positive_clearance = opening_bottom < opening_top;
                    let traversable =
                        role != RenderSubsectorConnectivityRole::ClosedSolid && positive_clearance;
                    source_correlated_relationships += usize::from(source_linedef.is_some());
                    traversable_relationships += usize::from(traversable);
                    closed_relationships += usize::from(!traversable);
                    zero_clearance_relationships += usize::from(!positive_clearance);
                    aperture_containment_failures += usize::from(
                        !interval_on_polygon_boundary(shared_interval, &left.boundary)
                            || !interval_on_polygon_boundary(shared_interval, &right.boundary),
                    );
                    *role_counts.entry(role.label()).or_default() += 1;
                    hash_u64(&mut fingerprint, left_index as u64);
                    hash_u64(&mut fingerprint, right_index as u64);
                    hash_bytes(&mut fingerprint, role.label().as_bytes());
                    if let Some(source) = source_linedef {
                        hash_source(&mut fingerprint, source);
                    }
                    for point in shared_interval {
                        for coordinate in point {
                            hash_u64(&mut fingerprint, coordinate.to_bits());
                        }
                    }
                    hash_u64(&mut aperture_fingerprint, left_index as u64);
                    hash_u64(&mut aperture_fingerprint, right_index as u64);
                    hash_bytes(&mut aperture_fingerprint, role.label().as_bytes());
                    hash_bytes(&mut aperture_fingerprint, &opening_bottom.to_le_bytes());
                    hash_bytes(&mut aperture_fingerprint, &opening_top.to_le_bytes());
                    hash_u64(&mut aperture_fingerprint, runtime_height_revision);
                    if let Some(source) = source_linedef {
                        hash_source(&mut aperture_fingerprint, source);
                    }
                    for point in shared_interval {
                        for coordinate in point {
                            hash_u64(&mut aperture_fingerprint, coordinate.to_bits());
                        }
                    }
                    let forward_index = edges.len();
                    edges.push(RenderSubsectorConnectivityEdge {
                        from_subsector: left_index as u32,
                        to_subsector: right_index as u32,
                        source_linedef,
                        role,
                        shared_interval,
                        opening_bottom,
                        opening_top,
                        runtime_height_revision,
                        reason,
                    });
                    adjacency[left_index].push(forward_index);
                    let reverse_index = edges.len();
                    edges.push(RenderSubsectorConnectivityEdge {
                        from_subsector: right_index as u32,
                        to_subsector: left_index as u32,
                        source_linedef,
                        role,
                        shared_interval: [shared_interval[1], shared_interval[0]],
                        opening_bottom,
                        opening_top,
                        runtime_height_revision,
                        reason,
                    });
                    adjacency[right_index].push(reverse_index);
                }
            }
        }
    }
    for outgoing in &mut adjacency {
        outgoing.sort_by_key(|edge_index| {
            let edge = &edges[*edge_index];
            (
                edge.to_subsector,
                edge.source_linedef.map(|source| source.record_index),
                edge.role.label(),
            )
        });
    }
    let isolated_subsectors = adjacency.iter().filter(|edges| edges.is_empty()).count();
    Ok(RenderSubsectorConnectivityGraph {
        edges,
        adjacency,
        role_counts,
        isolated_subsectors,
        fingerprint,
        aperture_fingerprint,
        source_correlated_relationships,
        traversable_relationships,
        closed_relationships,
        aperture_containment_failures,
        zero_clearance_relationships,
    })
}

pub(crate) fn observe_render_subsector_connectivity(
    graph: &RenderSubsectorConnectivityGraph,
    start_subsector: u32,
    sky_terminal: bool,
) -> Result<RenderSubsectorConnectivityObservation, String> {
    let start = start_subsector as usize;
    if start >= graph.adjacency.len() {
        return Err(format!(
            "connectivity start subsector {start_subsector} is outside {} cells",
            graph.adjacency.len()
        ));
    }
    let mut reachable = BTreeSet::from([start_subsector]);
    let mut predecessor = vec![None; graph.adjacency.len()];
    let mut queue = VecDeque::from([start_subsector]);
    let mut traversed_edges = 0;
    let mut terminal_closed_edges = 0;
    let mut terminal_sky_edges = 0;
    let mut fail_open_edges = 0;
    while let Some(source) = queue.pop_front() {
        for edge_index in &graph.adjacency[source as usize] {
            let edge = &graph.edges[*edge_index];
            match edge.role {
                RenderSubsectorConnectivityRole::ClosedSolid => {
                    terminal_closed_edges += 1;
                    continue;
                }
                RenderSubsectorConnectivityRole::PairedSkyOpening if sky_terminal => {
                    terminal_sky_edges += 1;
                    continue;
                }
                RenderSubsectorConnectivityRole::UnresolvedFailOpen => {
                    fail_open_edges += 1;
                }
                RenderSubsectorConnectivityRole::ImplicitPartition
                | RenderSubsectorConnectivityRole::PositiveOpening
                | RenderSubsectorConnectivityRole::MaskedMiddleOpening
                | RenderSubsectorConnectivityRole::PairedSkyOpening => {}
            }
            traversed_edges += 1;
            if reachable.insert(edge.to_subsector) {
                predecessor[edge.to_subsector as usize] = Some(*edge_index);
                queue.push_back(edge.to_subsector);
            }
        }
    }
    let mut fingerprint = FINGERPRINT_OFFSET;
    hash_u64(&mut fingerprint, start_subsector as u64);
    hash_bytes(
        &mut fingerprint,
        if sky_terminal {
            b"sky-terminal"
        } else {
            b"sky-annotated-open"
        },
    );
    for source in &reachable {
        hash_u64(&mut fingerprint, u64::from(*source));
    }
    Ok(RenderSubsectorConnectivityObservation {
        start_subsector,
        sky_terminal,
        reachable,
        predecessor,
        traversed_edges,
        terminal_closed_edges,
        terminal_sky_edges,
        fail_open_edges,
        fingerprint,
    })
}

pub(crate) fn render_subsector_connectivity_path<'a>(
    graph: &'a RenderSubsectorConnectivityGraph,
    observation: &RenderSubsectorConnectivityObservation,
    target_subsector: u32,
) -> Option<Vec<&'a RenderSubsectorConnectivityEdge>> {
    if !observation.reachable.contains(&target_subsector) {
        return None;
    }
    let mut result = Vec::new();
    let mut current = target_subsector;
    while current != observation.start_subsector {
        let edge_index = observation.predecessor[current as usize]?;
        let edge = &graph.edges[edge_index];
        result.push(edge);
        current = edge.from_subsector;
    }
    result.reverse();
    Some(result)
}

pub(crate) fn observe_doom_view_transfer(
    inventory: &RenderSubsectorInventory,
    graph: &RenderSubsectorConnectivityGraph,
    pose: RenderSubsectorViewPose,
    start_cell: u32,
    sky_terminal: bool,
) -> Result<DoomViewTransferObservation, String> {
    const MAX_STATES: usize = 250_000;
    if start_cell as usize >= graph.adjacency.len() {
        return Err(format!(
            "{} view-transfer start cell {} is outside {} cells",
            pose.label,
            start_cell,
            graph.adjacency.len()
        ));
    }
    let view_projection = render_subsector_view_projection(inventory, pose)?;
    let initial = DoomViewTransferState {
        cell: start_cell,
        window: DoomViewWindow {
            minimum_ndc: [-1.0, -1.0],
            maximum_ndc: [1.0, 1.0],
            depth: [0.0, f32::INFINITY],
        },
        predecessor_state: None,
        aperture_edge: None,
        lineage_fingerprint: fingerprint_view_pose(pose),
        near_plane_fail_open: false,
    };
    let mut states = vec![initial];
    let mut state_depths = vec![0_usize];
    let mut states_by_cell = vec![Vec::new(); graph.adjacency.len()];
    states_by_cell[start_cell as usize].push(0);
    let mut queue = VecDeque::from([0_usize]);
    let mut attempted_states = 0;
    let mut dominated_states = 0;
    let mut terminal_closed = 0;
    let mut terminal_sky = 0;
    let mut outside_view = 0;
    let mut near_plane_fail_open = 0;
    let mut maximum_depth = 0;

    while let Some(state_index) = queue.pop_front() {
        let state = states[state_index].clone();
        for edge_index in &graph.adjacency[state.cell as usize] {
            let edge = &graph.edges[*edge_index];
            if edge.role == RenderSubsectorConnectivityRole::ClosedSolid
                || edge.opening_bottom >= edge.opening_top
            {
                terminal_closed += 1;
                continue;
            }
            if sky_terminal && edge.role == RenderSubsectorConnectivityRole::PairedSkyOpening {
                terminal_sky += 1;
                continue;
            }
            let Some((aperture_window, aperture_near_plane_fail_open)) =
                project_aperture_window(edge, view_projection)
            else {
                outside_view += 1;
                continue;
            };
            let Some(window) = state.window.intersect(aperture_window) else {
                outside_view += 1;
                continue;
            };
            attempted_states += 1;
            let target = edge.to_subsector as usize;
            if states_by_cell[target]
                .iter()
                .any(|existing| states[*existing].window.contains_window(window))
            {
                dominated_states += 1;
                continue;
            }
            if states.len() >= MAX_STATES {
                return Err(format!(
                    "{} view-transfer exceeded {MAX_STATES} states without a conservative dominance result",
                    pose.label
                ));
            }
            let mut lineage_fingerprint = state.lineage_fingerprint;
            hash_u64(&mut lineage_fingerprint, *edge_index as u64);
            hash_view_window(&mut lineage_fingerprint, window);
            let new_state = DoomViewTransferState {
                cell: edge.to_subsector,
                window,
                predecessor_state: Some(state_index),
                aperture_edge: Some(*edge_index),
                lineage_fingerprint,
                near_plane_fail_open: aperture_near_plane_fail_open,
            };
            near_plane_fail_open += usize::from(aperture_near_plane_fail_open);
            let depth = state_depths[state_index] + 1;
            maximum_depth = maximum_depth.max(depth);
            let new_index = states.len();
            states.push(new_state);
            state_depths.push(depth);
            states_by_cell[target].push(new_index);
            queue.push_back(new_index);
        }
    }
    let repeated_destination_cells = states_by_cell
        .iter()
        .filter(|occurrences| occurrences.len() > 1)
        .count();
    let maximum_occurrences_per_cell = states_by_cell
        .iter()
        .map(Vec::len)
        .max()
        .unwrap_or_default();
    let mut fingerprint = FINGERPRINT_OFFSET;
    hash_u64(&mut fingerprint, start_cell as u64);
    hash_bytes(
        &mut fingerprint,
        if sky_terminal {
            b"bounded-sky-terminal"
        } else {
            b"bounded-sky-open"
        },
    );
    for state in &states {
        hash_u64(&mut fingerprint, state.cell as u64);
        hash_u64(&mut fingerprint, state.lineage_fingerprint);
        hash_view_window(&mut fingerprint, state.window);
        hash_bytes(&mut fingerprint, &[u8::from(state.near_plane_fail_open)]);
    }
    Ok(DoomViewTransferObservation {
        label: pose.label,
        start_cell,
        sky_terminal,
        states,
        states_by_cell,
        attempted_states,
        dominated_states,
        terminal_closed,
        terminal_sky,
        outside_view,
        near_plane_fail_open,
        maximum_depth,
        repeated_destination_cells,
        maximum_occurrences_per_cell,
        fingerprint,
    })
}

pub(crate) fn doom_view_transfer_chain<'a>(
    graph: &'a RenderSubsectorConnectivityGraph,
    observation: &'a DoomViewTransferObservation,
    state_index: usize,
) -> Option<
    Vec<(
        &'a DoomViewTransferState,
        &'a RenderSubsectorConnectivityEdge,
    )>,
> {
    let mut result = Vec::new();
    let mut current = state_index;
    loop {
        let state = observation.states.get(current)?;
        let (Some(predecessor), Some(edge_index)) = (state.predecessor_state, state.aperture_edge)
        else {
            break;
        };
        result.push((state, graph.edges.get(edge_index)?));
        current = predecessor;
    }
    result.reverse();
    Some(result)
}

fn project_aperture_window(
    edge: &RenderSubsectorConnectivityEdge,
    view_projection: tokimu_core::math::Mat4,
) -> Option<(DoomViewWindow, bool)> {
    if edge.opening_bottom >= edge.opening_top {
        return None;
    }
    let corners = [
        [
            edge.shared_interval[0][0] as f32,
            f32::from(edge.opening_bottom),
            edge.shared_interval[0][1] as f32,
        ],
        [
            edge.shared_interval[1][0] as f32,
            f32::from(edge.opening_bottom),
            edge.shared_interval[1][1] as f32,
        ],
        [
            edge.shared_interval[1][0] as f32,
            f32::from(edge.opening_top),
            edge.shared_interval[1][1] as f32,
        ],
        [
            edge.shared_interval[0][0] as f32,
            f32::from(edge.opening_top),
            edge.shared_interval[0][1] as f32,
        ],
    ];
    let clips = corners
        .map(|position| view_projection * Vec4::new(position[0], position[1], position[2], 1.0));
    if clips.iter().any(|clip| !clip.is_finite()) {
        return None;
    }
    let front = clips.iter().filter(|clip| clip.w > 1.0e-5).count();
    if front == 0 {
        return None;
    }
    if front != clips.len() {
        return Some((
            DoomViewWindow {
                minimum_ndc: [-1.0, -1.0],
                maximum_ndc: [1.0, 1.0],
                depth: [0.0, f32::INFINITY],
            },
            true,
        ));
    }
    let mut minimum_ndc = [f32::INFINITY; 2];
    let mut maximum_ndc = [f32::NEG_INFINITY; 2];
    let mut depth = [f32::INFINITY, f32::NEG_INFINITY];
    for clip in clips {
        let ndc = clip.truncate() / clip.w;
        minimum_ndc[0] = minimum_ndc[0].min(ndc.x);
        minimum_ndc[1] = minimum_ndc[1].min(ndc.y);
        maximum_ndc[0] = maximum_ndc[0].max(ndc.x);
        maximum_ndc[1] = maximum_ndc[1].max(ndc.y);
        depth[0] = depth[0].min(clip.w);
        depth[1] = depth[1].max(clip.w);
    }
    let window = DoomViewWindow {
        minimum_ndc: [minimum_ndc[0].max(-1.0), minimum_ndc[1].max(-1.0)],
        maximum_ndc: [maximum_ndc[0].min(1.0), maximum_ndc[1].min(1.0)],
        depth,
    };
    (window.minimum_ndc[0] < window.maximum_ndc[0] - 1.0e-6
        && window.minimum_ndc[1] < window.maximum_ndc[1] - 1.0e-6)
        .then_some((window, false))
}

fn hash_view_window(hash: &mut u64, window: DoomViewWindow) {
    for value in window
        .minimum_ndc
        .into_iter()
        .chain(window.maximum_ndc)
        .chain(window.depth)
    {
        hash_bytes(hash, &value.to_bits().to_le_bytes());
    }
}

fn polygon_edges(polygon: &[[f64; 2]]) -> impl Iterator<Item = [[f64; 2]; 2]> + '_ {
    polygon
        .iter()
        .copied()
        .zip(polygon.iter().copied().cycle().skip(1))
        .take(polygon.len())
        .map(|(start, end)| [start, end])
}

fn interval_on_polygon_boundary(interval: [[f64; 2]; 2], polygon: &[[f64; 2]]) -> bool {
    polygon_edges(polygon).any(|edge| {
        let Some(overlap) = shared_collinear_interval(edge, interval) else {
            return false;
        };
        (point_distance_squared(overlap[0], interval[0]) <= DOMAIN_EPSILON
            && point_distance_squared(overlap[1], interval[1]) <= DOMAIN_EPSILON)
            || (point_distance_squared(overlap[0], interval[1]) <= DOMAIN_EPSILON
                && point_distance_squared(overlap[1], interval[0]) <= DOMAIN_EPSILON)
    })
}

fn point_distance_squared(left: [f64; 2], right: [f64; 2]) -> f64 {
    let delta = [left[0] - right[0], left[1] - right[1]];
    delta[0] * delta[0] + delta[1] * delta[1]
}

fn shared_collinear_interval(first: [[f64; 2]; 2], second: [[f64; 2]; 2]) -> Option<[[f64; 2]; 2]> {
    let direction = [first[1][0] - first[0][0], first[1][1] - first[0][1]];
    let length_squared = direction[0] * direction[0] + direction[1] * direction[1];
    if length_squared <= DOMAIN_EPSILON {
        return None;
    }
    let cross = |point: [f64; 2]| {
        direction[0] * (point[1] - first[0][1]) - direction[1] * (point[0] - first[0][0])
    };
    if cross(second[0]).abs() > DOMAIN_EPSILON || cross(second[1]).abs() > DOMAIN_EPSILON {
        return None;
    }
    let parameter = |point: [f64; 2]| {
        ((point[0] - first[0][0]) * direction[0] + (point[1] - first[0][1]) * direction[1])
            / length_squared
    };
    let second_parameters = [parameter(second[0]), parameter(second[1])];
    let start = 0.0_f64.max(second_parameters[0].min(second_parameters[1]));
    let end = 1.0_f64.min(second_parameters[0].max(second_parameters[1]));
    if end - start <= DOMAIN_EPSILON {
        return None;
    }
    let point = |value: f64| {
        [
            first[0][0] + direction[0] * value,
            first[0][1] + direction[1] * value,
        ]
    };
    Some([point(start), point(end)])
}

fn boundary_linedefs(
    map: &DoomMapCore,
    subsector_index: usize,
    shared_interval: [[f64; 2]; 2],
) -> Vec<usize> {
    let subsector = &map.subsectors[subsector_index];
    let first = usize::from(subsector.first_seg);
    let end = first + usize::from(subsector.seg_count);
    map.segs[first..end]
        .iter()
        .filter_map(|seg| {
            let start = &map.vertices[usize::from(seg.start_vertex)];
            let end = &map.vertices[usize::from(seg.end_vertex)];
            shared_collinear_interval(
                [
                    [f64::from(start.x), f64::from(start.y)],
                    [f64::from(end.x), f64::from(end.y)],
                ],
                shared_interval,
            )
            .map(|_| usize::from(seg.linedef))
        })
        .collect()
}

fn classify_connectivity_linedef(
    map: &DoomMapCore,
    linedef: &doom_map_provider::DoomLinedef,
) -> (RenderSubsectorConnectivityRole, &'static str) {
    let (Some(right_index), Some(left_index)) = (linedef.right_sidedef, linedef.left_sidedef)
    else {
        return (
            RenderSubsectorConnectivityRole::ClosedSolid,
            "one-sided-linedef",
        );
    };
    let right = &map.sidedefs[usize::from(right_index)];
    let left = &map.sidedefs[usize::from(left_index)];
    let right_sector = &map.sectors[usize::from(right.sector)];
    let left_sector = &map.sectors[usize::from(left.sector)];
    let opening_floor = right_sector.floor_height.max(left_sector.floor_height);
    let opening_ceiling = right_sector.ceiling_height.min(left_sector.ceiling_height);
    if opening_floor >= opening_ceiling {
        return (
            RenderSubsectorConnectivityRole::ClosedSolid,
            "two-sided-opening-has-no-positive-clearance",
        );
    }
    if right_sector.ceiling_texture == "F_SKY1" && left_sector.ceiling_texture == "F_SKY1" {
        return (
            RenderSubsectorConnectivityRole::PairedSkyOpening,
            "positive-opening-between-paired-sky-ceilings",
        );
    }
    if right.middle_texture != "-" || left.middle_texture != "-" {
        return (
            RenderSubsectorConnectivityRole::MaskedMiddleOpening,
            "positive-opening-with-authored-middle-surface",
        );
    }
    (
        RenderSubsectorConnectivityRole::PositiveOpening,
        "positive-two-sided-opening",
    )
}

pub(crate) fn observe_render_subsector_actual_camera(
    map: &DoomMapCore,
    inventory: &RenderSubsectorInventory,
    pose: RenderSubsectorViewPose,
) -> Result<RenderSubsectorShadowObservation, String> {
    if pose.viewport[0] == 0 || pose.viewport[1] == 0 {
        return Err(format!("{} has an empty viewport", pose.label));
    }
    let rounded_position = [
        finite_i16(pose.source_position[0], "source x")?,
        finite_i16(pose.source_position[1], "source y")?,
    ];
    let order = resolve_doom_viewer_subsector_order(map, rounded_position)
        .map_err(|error| error.to_string())?;
    if order.len() != inventory.subsectors.len() {
        return Err(format!(
            "{} near-first order has {} leaves for {} render subsectors",
            pose.label,
            order.len(),
            inventory.subsectors.len()
        ));
    }
    let mut rank_by_subsector = vec![None; map.subsectors.len()];
    let mut near_first_fingerprint = FINGERPRINT_OFFSET;
    for (rank, source) in order.iter().enumerate() {
        let index = source.record_index as usize;
        let slot = rank_by_subsector.get_mut(index).ok_or_else(|| {
            format!(
                "{} order names out-of-range subsector {}",
                pose.label, source.record_index
            )
        })?;
        if slot.replace(rank).is_some() {
            return Err(format!(
                "{} order repeats subsector {}",
                pose.label, source.record_index
            ));
        }
        hash_source(&mut near_first_fingerprint, *source);
    }

    let view_projection = render_subsector_view_projection(inventory, pose)?;
    let view_fingerprint = fingerprint_view_pose(pose);
    let mut entries = Vec::with_capacity(inventory.subsectors.len());
    let mut retained = 0;
    let mut outside_frustum = 0;
    let mut unresolved = 0;
    let mut brute_retained = 0;
    let mut false_negatives = 0;
    let mut false_positives = 0;

    for subsector in &inventory.subsectors {
        let rank = rank_by_subsector
            .get(subsector.source_subsector.record_index as usize)
            .and_then(|rank| *rank)
            .ok_or_else(|| {
                format!(
                    "{} has no order rank for subsector {}",
                    pose.label, subsector.source_subsector.record_index
                )
            })?;
        let candidate_outside = render_subsector_bounds(subsector)
            .and_then(|bounds| classify_static_draw_frustum_rejection(bounds, view_projection));
        let brute_intersects = subsector_geometry_intersects_frustum(subsector, view_projection);
        brute_retained += usize::from(brute_intersects);
        let (disposition, reason) = if subsector.unresolved_reason.is_some() {
            unresolved += 1;
            (
                RenderSubsectorShadowDisposition::Unresolved,
                "persistent-geometry-unresolved",
            )
        } else if let Some(rejection) = candidate_outside {
            outside_frustum += 1;
            if brute_intersects {
                false_negatives += 1;
            }
            (
                RenderSubsectorShadowDisposition::OutsideFrustum,
                frustum_rejection_reason(rejection),
            )
        } else {
            retained += 1;
            if !brute_intersects {
                false_positives += 1;
            }
            (
                RenderSubsectorShadowDisposition::RetainedGeometry,
                "actual-render-subsector-aabb-intersects-frustum",
            )
        };
        entries.push(RenderSubsectorShadowEntry {
            source_subsector: subsector.source_subsector,
            near_first_rank: rank,
            disposition,
            reason,
        });
    }
    entries.sort_by_key(|entry| entry.near_first_rank);
    let mut result_fingerprint = FINGERPRINT_OFFSET;
    for entry in &entries {
        hash_source(&mut result_fingerprint, entry.source_subsector);
        hash_u64(&mut result_fingerprint, entry.near_first_rank as u64);
        hash_bytes(
            &mut result_fingerprint,
            match entry.disposition {
                RenderSubsectorShadowDisposition::RetainedGeometry => b"retained",
                RenderSubsectorShadowDisposition::OutsideFrustum => b"outside",
                RenderSubsectorShadowDisposition::Unresolved => b"unresolved",
            },
        );
        hash_bytes(&mut result_fingerprint, entry.reason.as_bytes());
    }
    let coverage = observe_surface_coverage(map, inventory, pose, view_projection, &entries)?;

    Ok(RenderSubsectorShadowObservation {
        label: pose.label,
        view_fingerprint,
        near_first_fingerprint,
        result_fingerprint,
        entries,
        retained,
        outside_frustum,
        unresolved,
        brute_retained,
        false_negatives,
        false_positives,
        plane_retained: coverage.plane_retained,
        plane_outside_frustum: coverage.plane_outside_frustum,
        plane_source_covered: coverage.plane_source_covered,
        plane_unresolved: coverage.plane_unresolved,
        wall_tiers_retained: coverage.wall_tiers_retained,
        wall_tiers_outside_frustum: coverage.wall_tiers_outside_frustum,
        wall_tiers_source_covered: coverage.wall_tiers_source_covered,
        wall_tiers_unresolved: coverage.wall_tiers_unresolved,
        plane_horizontal_aabb_false_positives: coverage.plane_horizontal_aabb_false_positives,
        wall_horizontal_aabb_false_positives: coverage.wall_horizontal_aabb_false_positives,
        source_coverage_fingerprint: coverage.fingerprint,
        unresolved_surface_samples: coverage.unresolved_samples,
        plane_entries: coverage.plane_entries,
        wall_tier_entries: coverage.wall_tier_entries,
    })
}

pub(crate) fn prepare_render_subsector_view(
    map: &DoomMapCore,
    inventory: &RenderSubsectorInventory,
    pose: RenderSubsectorViewPose,
    wall_extents: &[DoomTextureExtent],
    opaque_uploads: &[StaticTextureUpload],
    cutout_uploads: &[StaticTextureUpload],
    cutout_draws: &[StaticDrawPlanEntry],
) -> Result<PreparedRenderSubsectorView, String> {
    let shadow = observe_render_subsector_actual_camera(map, inventory, pose)?;
    let flat_materials = opaque_uploads
        .iter()
        .filter(|upload| upload.source_kind == StaticTextureSourceKind::Flat)
        .map(|upload| (upload.source_name.as_str(), upload.material))
        .collect::<BTreeMap<_, _>>();
    let wall_materials = opaque_uploads
        .iter()
        .filter(|upload| upload.source_kind == StaticTextureSourceKind::Wall)
        .map(|upload| (upload.source_name.as_str(), upload.material))
        .collect::<BTreeMap<_, _>>();
    let cutout_materials = cutout_uploads
        .iter()
        .map(|upload| (upload.source_name.as_str(), upload.material))
        .collect::<BTreeMap<_, _>>();
    let cutout_sources = cutout_draws
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
                wall_role_key(role),
            )),
            StaticDrawSource::Flat { .. } => None,
        })
        .collect::<BTreeSet<_>>();
    let plane_dispositions = shadow
        .plane_entries
        .iter()
        .map(|entry| {
            (
                (entry.source_subsector.record_index, plane_key(entry.plane)),
                entry.disposition,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let wall_dispositions = shadow
        .wall_tier_entries
        .iter()
        .map(|entry| {
            (
                (entry.source_subsector.record_index, entry.wall_tier_ordinal),
                entry.disposition,
            )
        })
        .collect::<BTreeMap<_, _>>();
    if plane_dispositions.len() != shadow.plane_entries.len()
        || wall_dispositions.len() != shadow.wall_tier_entries.len()
    {
        return Err(format!(
            "{} surface shadow identity is not unique",
            pose.label
        ));
    }

    let mut result = PreparedRenderSubsectorView {
        shadow,
        declarations: Vec::new(),
        source_plane_triangles: inventory.triangles,
        source_wall_tier_triangles: inventory.represented_wall_tier_triangles,
        ordinary_plane_declarations: 0,
        opaque_wall_declarations: 0,
        cutout_wall_declarations: 0,
        sky_background_triangles: 0,
        outside_frustum_triangles: 0,
        source_covered_triangles: 0,
        unresolved_fail_open_triangles: 0,
        declaration_fingerprint: FINGERPRINT_OFFSET,
    };

    for subsector in &inventory.subsectors {
        for (source_triangle_ordinal, triangle) in subsector.triangles.iter().enumerate() {
            let disposition = *plane_dispositions
                .get(&(
                    subsector.source_subsector.record_index,
                    plane_key(triangle.plane),
                ))
                .ok_or_else(|| {
                    format!(
                        "{} lacks {:?} shadow disposition for subsector {}",
                        pose.label, triangle.plane, subsector.source_subsector.record_index
                    )
                })?;
            if triangle.role == RenderSubsectorPlaneRole::Sky {
                match disposition {
                    RenderSubsectorSurfaceShadowDisposition::RetainedGeometry
                    | RenderSubsectorSurfaceShadowDisposition::Unresolved => {
                        result.sky_background_triangles += 1;
                        result.unresolved_fail_open_triangles += usize::from(
                            disposition == RenderSubsectorSurfaceShadowDisposition::Unresolved,
                        );
                    }
                    RenderSubsectorSurfaceShadowDisposition::OutsideFrustum => {
                        result.outside_frustum_triangles += 1;
                    }
                    RenderSubsectorSurfaceShadowDisposition::SourceCovered => {
                        result.source_covered_triangles += 1;
                    }
                }
                continue;
            }
            match disposition {
                RenderSubsectorSurfaceShadowDisposition::OutsideFrustum => {
                    result.outside_frustum_triangles += 1;
                    continue;
                }
                RenderSubsectorSurfaceShadowDisposition::SourceCovered => {
                    result.source_covered_triangles += 1;
                    continue;
                }
                RenderSubsectorSurfaceShadowDisposition::Unresolved => {
                    result.unresolved_fail_open_triangles += 1;
                }
                RenderSubsectorSurfaceShadowDisposition::RetainedGeometry => {}
            }
            let material = *flat_materials
                .get(triangle.texture_name.as_str())
                .ok_or_else(|| {
                    format!(
                        "{} has no opaque flat material for {}",
                        pose.label, triangle.texture_name
                    )
                })?;
            let lowered = lower_static_flat_triangle(
                &DoomSurfaceTriangle {
                    source_subsector: triangle.source_subsector,
                    source_sector: triangle.source_sector,
                    plane: triangle.plane,
                    texture_name: triangle.texture_name.clone(),
                    positions: triangle.positions,
                },
                FlatExtent::E1M1,
            )
            .map_err(|error| error.to_string())?;
            let declaration = PreparedRenderSubsectorDeclaration {
                source_subsector: subsector.source_subsector,
                source_triangle_ordinal,
                surface: PreparedRenderSubsectorSurface::Plane(triangle.plane),
                cutout: false,
                draw: StaticDrawPlanEntry {
                    mesh: lowered.mesh,
                    material,
                    source_label: format!(
                        "flat:{}:{}",
                        triangle.source_sector.record_index, triangle.texture_name
                    ),
                    source: StaticDrawSource::Flat {
                        source_subsector: triangle.source_subsector,
                        source_sector: triangle.source_sector,
                        plane: triangle.plane,
                    },
                },
            };
            hash_prepared_declaration(&mut result.declaration_fingerprint, &declaration);
            result.ordinary_plane_declarations += 1;
            result.declarations.push(declaration);
        }

        for (source_triangle_ordinal, triangle) in subsector.wall_tier_triangles.iter().enumerate()
        {
            let disposition = *wall_dispositions
                .get(&(
                    subsector.source_subsector.record_index,
                    source_triangle_ordinal,
                ))
                .ok_or_else(|| {
                    format!(
                        "{} lacks wall-tier shadow disposition for subsector {} triangle {}",
                        pose.label,
                        subsector.source_subsector.record_index,
                        source_triangle_ordinal
                    )
                })?;
            match disposition {
                RenderSubsectorSurfaceShadowDisposition::OutsideFrustum => {
                    result.outside_frustum_triangles += 1;
                    continue;
                }
                RenderSubsectorSurfaceShadowDisposition::SourceCovered => {
                    result.source_covered_triangles += 1;
                    continue;
                }
                RenderSubsectorSurfaceShadowDisposition::Unresolved => {
                    result.unresolved_fail_open_triangles += 1;
                }
                RenderSubsectorSurfaceShadowDisposition::RetainedGeometry => {}
            }
            let extent = wall_extents
                .iter()
                .find(|extent| extent.name == triangle.texture_name)
                .cloned()
                .ok_or_else(|| {
                    format!(
                        "{} has no wall extent for {}",
                        pose.label, triangle.texture_name
                    )
                })?;
            let lowered = lower_static_seg_wall_triangle(triangle, extent)
                .map_err(|error| error.to_string())?;
            let cutout = cutout_sources.contains(&(
                triangle.source_linedef.record_index,
                triangle.source_sidedef.record_index,
                wall_role_key(triangle.role),
            ));
            let material = if cutout {
                *cutout_materials
                    .get(triangle.texture_name.as_str())
                    .expect("cutout membership was checked")
            } else {
                *wall_materials
                    .get(triangle.texture_name.as_str())
                    .ok_or_else(|| {
                        format!(
                            "{} has no opaque wall material for {}",
                            pose.label, triangle.texture_name
                        )
                    })?
            };
            let declaration = PreparedRenderSubsectorDeclaration {
                source_subsector: subsector.source_subsector,
                source_triangle_ordinal,
                surface: PreparedRenderSubsectorSurface::WallTier {
                    source_seg: triangle.source_seg,
                    role: triangle.role,
                },
                cutout,
                draw: StaticDrawPlanEntry {
                    mesh: lowered.wall.mesh,
                    material,
                    source_label: format!(
                        "wall:{}:{}",
                        triangle.source_linedef.record_index, triangle.texture_name
                    ),
                    source: StaticDrawSource::Wall {
                        source_linedef: triangle.source_linedef,
                        source_sidedef: triangle.source_sidedef,
                        source_sector: triangle.source_sector,
                        role: triangle.role,
                    },
                },
            };
            hash_prepared_declaration(&mut result.declaration_fingerprint, &declaration);
            if cutout {
                result.cutout_wall_declarations += 1;
            } else {
                result.opaque_wall_declarations += 1;
            }
            result.declarations.push(declaration);
        }
    }
    result.verify_conservation()?;
    Ok(result)
}

const fn plane_key(plane: DoomSurfacePlane) -> u8 {
    match plane {
        DoomSurfacePlane::Floor => 0,
        DoomSurfacePlane::Ceiling => 1,
    }
}

const fn wall_role_key(role: DoomWallTextureRole) -> u8 {
    match role {
        DoomWallTextureRole::Upper => 0,
        DoomWallTextureRole::Lower => 1,
        DoomWallTextureRole::Middle => 2,
    }
}

fn hash_prepared_declaration(hash: &mut u64, declaration: &PreparedRenderSubsectorDeclaration) {
    hash_source(hash, declaration.source_subsector);
    hash_u64(hash, declaration.source_triangle_ordinal as u64);
    hash_u64(hash, declaration.draw.material.0);
    hash_bytes(hash, &[u8::from(declaration.cutout)]);
    hash_bytes(hash, declaration.draw.source_label.as_bytes());
    for position in &declaration.draw.mesh.positions {
        for coordinate in position {
            hash_bytes(hash, &coordinate.to_bits().to_le_bytes());
        }
    }
}

#[derive(Default)]
struct SurfaceCoverageObservation {
    plane_retained: usize,
    plane_outside_frustum: usize,
    plane_source_covered: usize,
    plane_unresolved: usize,
    wall_tiers_retained: usize,
    wall_tiers_outside_frustum: usize,
    wall_tiers_source_covered: usize,
    wall_tiers_unresolved: usize,
    plane_horizontal_aabb_false_positives: usize,
    wall_horizontal_aabb_false_positives: usize,
    fingerprint: u64,
    unresolved_samples: Vec<String>,
    plane_entries: Vec<RenderSubsectorPlaneShadowEntry>,
    wall_tier_entries: Vec<RenderSubsectorWallTierShadowEntry>,
}

fn observe_surface_coverage(
    map: &DoomMapCore,
    inventory: &RenderSubsectorInventory,
    pose: RenderSubsectorViewPose,
    view_projection: tokimu_core::math::Mat4,
    entries: &[RenderSubsectorShadowEntry],
) -> Result<SurfaceCoverageObservation, String> {
    let occluders = observe_doom_seg_occluders(map).map_err(|error| error.to_string())?;
    if occluders.len() != map.segs.len() {
        return Err(format!(
            "{} occluder inventory has {} records for {} SEGs",
            pose.label,
            occluders.len(),
            map.segs.len()
        ));
    }
    let aspect = pose.viewport[0] as f64 / pose.viewport[1] as f64;
    let half_horizontal_fov =
        ((f64::from(pose.vertical_fov_degrees).to_radians() * 0.5).tan() * aspect).atan();
    let heading = f64::from(pose.heading_degrees).to_radians();
    let mut solid_coverage = Vec::<[f64; 2]>::new();
    let mut result = SurfaceCoverageObservation {
        fingerprint: FINGERPRINT_OFFSET,
        ..SurfaceCoverageObservation::default()
    };

    for entry in entries {
        let source_index = entry.source_subsector.record_index as usize;
        let subsector = inventory.subsectors.get(source_index).ok_or_else(|| {
            format!(
                "{} coverage order names unavailable render subsector {}",
                pose.label, entry.source_subsector.record_index
            )
        })?;
        let plane_interval = polygon_horizontal_interval(
            &subsector.boundary,
            pose.source_position,
            heading,
            half_horizontal_fov,
        );
        for plane in [DoomSurfacePlane::Floor, DoomSurfacePlane::Ceiling] {
            let actual_intersects = subsector
                .triangles
                .iter()
                .filter(|triangle| triangle.plane == plane)
                .filter_map(|triangle| triangle_bounds(triangle.positions))
                .any(|bounds| {
                    classify_static_draw_frustum_rejection(bounds, view_projection).is_none()
                });
            let disposition = if !actual_intersects {
                result.plane_outside_frustum += 1;
                RenderSubsectorSurfaceShadowDisposition::OutsideFrustum
            } else if let Some(interval) = plane_interval {
                if interval_fully_covered(interval, &solid_coverage) {
                    result.plane_source_covered += 1;
                    RenderSubsectorSurfaceShadowDisposition::SourceCovered
                } else {
                    result.plane_retained += 1;
                    RenderSubsectorSurfaceShadowDisposition::RetainedGeometry
                }
            } else {
                // The triangle AABB is conservative. The exact finite convex
                // boundary being wholly outside the horizontal frustum is
                // sufficient proof that this plane cannot participate.
                result.plane_horizontal_aabb_false_positives += 1;
                result.plane_outside_frustum += 1;
                RenderSubsectorSurfaceShadowDisposition::OutsideFrustum
            };
            result.plane_entries.push(RenderSubsectorPlaneShadowEntry {
                source_subsector: subsector.source_subsector,
                plane,
                disposition,
            });
            hash_source(&mut result.fingerprint, subsector.source_subsector);
            hash_bytes(
                &mut result.fingerprint,
                match plane {
                    DoomSurfacePlane::Floor => b"floor",
                    DoomSurfacePlane::Ceiling => b"ceiling",
                },
            );
            hash_bytes(
                &mut result.fingerprint,
                surface_disposition_label(disposition),
            );
        }

        let source_subsector = &map.subsectors[source_index];
        let first = usize::from(source_subsector.first_seg);
        let end = first + usize::from(source_subsector.seg_count);
        for seg in &map.segs[first..end] {
            let start = &map.vertices[usize::from(seg.start_vertex)];
            let end = &map.vertices[usize::from(seg.end_vertex)];
            let start_position = [f64::from(start.x), f64::from(start.y)];
            let end_position = [f64::from(end.x), f64::from(end.y)];
            let solid_range_eligible = source_seg_solid_range_eligible(
                pose.source_position,
                heading,
                start_position,
                end_position,
            );
            let interval =
                source_seg_front_facing(pose.source_position, start_position, end_position)
                    .then(|| {
                        segment_horizontal_interval(
                            start_position,
                            end_position,
                            pose.source_position,
                            heading,
                            half_horizontal_fov,
                        )
                    })
                    .flatten();
            let covered_before =
                interval.is_some_and(|interval| interval_fully_covered(interval, &solid_coverage));
            for (wall_tier_ordinal, triangle) in subsector
                .wall_tier_triangles
                .iter()
                .enumerate()
                .filter(|(_, triangle)| triangle.source_seg == seg.source)
            {
                let actual_intersects = triangle_bounds(triangle.positions).is_some_and(|bounds| {
                    classify_static_draw_frustum_rejection(bounds, view_projection).is_none()
                });
                let disposition = if !actual_intersects {
                    result.wall_tiers_outside_frustum += 1;
                    RenderSubsectorSurfaceShadowDisposition::OutsideFrustum
                } else if interval.is_none() {
                    // As above, an exact finite SEG outside the horizontal
                    // frustum resolves an AABB-only overlap as outside.
                    result.wall_horizontal_aabb_false_positives += 1;
                    result.wall_tiers_outside_frustum += 1;
                    RenderSubsectorSurfaceShadowDisposition::OutsideFrustum
                } else if covered_before {
                    result.wall_tiers_source_covered += 1;
                    RenderSubsectorSurfaceShadowDisposition::SourceCovered
                } else {
                    result.wall_tiers_retained += 1;
                    RenderSubsectorSurfaceShadowDisposition::RetainedGeometry
                };
                result
                    .wall_tier_entries
                    .push(RenderSubsectorWallTierShadowEntry {
                        source_subsector: subsector.source_subsector,
                        source_seg: triangle.source_seg,
                        wall_tier_ordinal,
                        role: triangle.role,
                        disposition,
                    });
                hash_source(&mut result.fingerprint, triangle.source_seg);
                hash_bytes(
                    &mut result.fingerprint,
                    surface_disposition_label(disposition),
                );
            }
            let occluder = occluders
                .get(seg.source.record_index as usize)
                .ok_or_else(|| {
                    format!(
                        "{} has no occluder record for SEG {}",
                        pose.label, seg.source.record_index
                    )
                })?;
            if occluder.source_seg != seg.source {
                return Err(format!(
                    "{} occluder correlation disagrees for SEG {}",
                    pose.label, seg.source.record_index
                ));
            }
            if solid_range_eligible && occluder.kind != DoomSegOccluderKind::Open {
                if let Some(interval) = interval {
                    merge_angular_interval(&mut solid_coverage, interval);
                }
            }
        }
    }
    Ok(result)
}

fn source_seg_front_facing(viewer: [f64; 2], start: [f64; 2], end: [f64; 2]) -> bool {
    let segment = [end[0] - start[0], end[1] - start[1]];
    let to_viewer = [viewer[0] - start[0], viewer[1] - start[1]];
    segment[0] * to_viewer[1] - segment[1] * to_viewer[0] < -DOMAIN_EPSILON
}

fn source_seg_solid_range_eligible(
    viewer: [f64; 2],
    heading: f64,
    start: [f64; 2],
    end: [f64; 2],
) -> bool {
    let forward = [heading.cos(), heading.sin()];
    let depth =
        |point: [f64; 2]| (point[0] - viewer[0]) * forward[0] + (point[1] - viewer[1]) * forward[1];
    depth(start) > DOMAIN_EPSILON && depth(end) > DOMAIN_EPSILON
}

fn surface_disposition_label(
    disposition: RenderSubsectorSurfaceShadowDisposition,
) -> &'static [u8] {
    match disposition {
        RenderSubsectorSurfaceShadowDisposition::RetainedGeometry => b"retained",
        RenderSubsectorSurfaceShadowDisposition::OutsideFrustum => b"outside",
        RenderSubsectorSurfaceShadowDisposition::SourceCovered => b"source-covered",
        RenderSubsectorSurfaceShadowDisposition::Unresolved => b"unresolved",
    }
}

fn polygon_horizontal_interval(
    polygon: &[[f64; 2]],
    viewer: [f64; 2],
    heading: f64,
    half_fov: f64,
) -> Option<[f64; 2]> {
    if point_in_convex_domain(viewer, polygon) {
        return Some([-half_fov, half_fov]);
    }
    let mut result: Option<[f64; 2]> = None;
    for index in 0..polygon.len() {
        let Some(interval) = segment_horizontal_interval(
            polygon[index],
            polygon[(index + 1) % polygon.len()],
            viewer,
            heading,
            half_fov,
        ) else {
            continue;
        };
        result = Some(result.map_or(interval, |current| {
            [current[0].min(interval[0]), current[1].max(interval[1])]
        }));
    }
    result
}

fn segment_horizontal_interval(
    start: [f64; 2],
    end: [f64; 2],
    viewer: [f64; 2],
    heading: f64,
    half_fov: f64,
) -> Option<[f64; 2]> {
    let forward = [heading.cos(), heading.sin()];
    let right = [-forward[1], forward[0]];
    let camera_point = |point: [f64; 2]| {
        let delta = [point[0] - viewer[0], point[1] - viewer[1]];
        [
            delta[0] * right[0] + delta[1] * right[1],
            delta[0] * forward[0] + delta[1] * forward[1],
        ]
    };
    let start = camera_point(start);
    let end = camera_point(end);
    let delta = [end[0] - start[0], end[1] - start[1]];
    let tangent = half_fov.tan();
    let mut minimum_t = 0.0_f64;
    let mut maximum_t = 1.0_f64;
    // forward > 0, right >= -forward*tan(fov), right <= forward*tan(fov)
    for (a, b, c) in [
        (0.0, 1.0, 1.0e-6),
        (1.0, tangent, 0.0),
        (-1.0, tangent, 0.0),
    ] {
        let initial = a * start[0] + b * start[1] - c;
        let change = a * delta[0] + b * delta[1];
        if change.abs() <= f64::EPSILON {
            if initial < 0.0 {
                return None;
            }
            continue;
        }
        let crossing = -initial / change;
        if change > 0.0 {
            minimum_t = minimum_t.max(crossing);
        } else {
            maximum_t = maximum_t.min(crossing);
        }
        if minimum_t > maximum_t {
            return None;
        }
    }
    let angle = |parameter: f64| {
        let right = start[0] + delta[0] * parameter;
        let forward = start[1] + delta[1] * parameter;
        right.atan2(forward).clamp(-half_fov, half_fov)
    };
    let first = angle(minimum_t);
    let second = angle(maximum_t);
    Some([first.min(second), first.max(second)])
}

fn interval_fully_covered(interval: [f64; 2], coverage: &[[f64; 2]]) -> bool {
    coverage.iter().any(|covered| {
        covered[0] <= interval[0] + DOMAIN_EPSILON && covered[1] >= interval[1] - DOMAIN_EPSILON
    })
}

fn merge_angular_interval(coverage: &mut Vec<[f64; 2]>, interval: [f64; 2]) {
    coverage.push(interval);
    coverage.sort_by(|left, right| left[0].total_cmp(&right[0]));
    let mut merged = Vec::<[f64; 2]>::with_capacity(coverage.len());
    for interval in coverage.drain(..) {
        if let Some(last) = merged.last_mut() {
            if interval[0] <= last[1] + DOMAIN_EPSILON {
                last[1] = last[1].max(interval[1]);
                continue;
            }
        }
        merged.push(interval);
    }
    *coverage = merged;
}

fn render_subsector_view_projection(
    inventory: &RenderSubsectorInventory,
    pose: RenderSubsectorViewPose,
) -> Result<tokimu_core::math::Mat4, String> {
    let world_eye = Vec3::from_array(
        doom_point_to_tokimu(pose.source_position, pose.eye_height).map(|value| value as f32),
    );
    let heading = pose.heading_degrees.to_radians();
    let pitch = pose.pitch_degrees.to_radians();
    let forward = Vec3::new(
        heading.cos() * pitch.cos(),
        pitch.sin(),
        heading.sin() * pitch.cos(),
    );
    let view = try_view_look_at_rh(world_eye, world_eye + forward * 128.0, Vec3::Y)
        .ok_or_else(|| format!("{} view is non-finite or degenerate", pose.label))?;
    let far = inventory
        .subsectors
        .iter()
        .flat_map(|subsector| {
            subsector.boundary.iter().map(|point| {
                Vec3::new(
                    point[0] as f32,
                    f32::from(subsector.ceiling_height),
                    point[1] as f32,
                )
                .distance(world_eye)
            })
        })
        .fold(1.0_f32, f32::max)
        * 1.25;
    let aspect = pose.viewport[0] as f32 / pose.viewport[1] as f32;
    let projection = try_projection_perspective_rh_gl(
        pose.vertical_fov_degrees.to_radians(),
        aspect,
        0.1,
        far.max(1.0),
    )
    .ok_or_else(|| format!("{} projection is invalid", pose.label))?;
    Ok(projection * view)
}

fn render_subsector_bounds(subsector: &RenderSubsector) -> Option<StaticDrawAabb> {
    let mut minimum = Vec3::splat(f32::INFINITY);
    let mut maximum = Vec3::splat(f32::NEG_INFINITY);
    for point in &subsector.boundary {
        for height in [subsector.floor_height, subsector.ceiling_height] {
            let position = Vec3::new(point[0] as f32, f32::from(height), point[1] as f32);
            minimum = minimum.min(position);
            maximum = maximum.max(position);
        }
    }
    StaticDrawAabb::from_minimum_maximum(minimum, maximum)
}

fn subsector_geometry_intersects_frustum(
    subsector: &RenderSubsector,
    view_projection: tokimu_core::math::Mat4,
) -> bool {
    subsector
        .triangles
        .iter()
        .map(|triangle| triangle.positions)
        .chain(
            subsector
                .wall_tier_triangles
                .iter()
                .map(|triangle| triangle.positions),
        )
        .filter_map(triangle_bounds)
        .any(|bounds| classify_static_draw_frustum_rejection(bounds, view_projection).is_none())
}

fn triangle_bounds(positions: [[f64; 3]; 3]) -> Option<StaticDrawAabb> {
    StaticDrawAabb::from_positions(
        &positions.map(|position| position.map(|coordinate| coordinate as f32)),
    )
}

fn frustum_rejection_reason(rejection: StaticDrawFrustumRejection) -> &'static str {
    match rejection {
        StaticDrawFrustumRejection::Left => "actual-render-subsector-aabb-outside-left",
        StaticDrawFrustumRejection::Right => "actual-render-subsector-aabb-outside-right",
        StaticDrawFrustumRejection::Bottom => "actual-render-subsector-aabb-outside-bottom",
        StaticDrawFrustumRejection::Top => "actual-render-subsector-aabb-outside-top",
        StaticDrawFrustumRejection::Near => "actual-render-subsector-aabb-outside-near",
        StaticDrawFrustumRejection::Far => "actual-render-subsector-aabb-outside-far",
    }
}

fn finite_i16(value: f64, label: &str) -> Result<i16, String> {
    if !value.is_finite() || value < f64::from(i16::MIN) || value > f64::from(i16::MAX) {
        return Err(format!(
            "{label} {value} is outside the Doom coordinate domain"
        ));
    }
    Ok(value.round() as i16)
}

fn fingerprint_view_pose(pose: RenderSubsectorViewPose) -> u64 {
    let mut hash = FINGERPRINT_OFFSET;
    for coordinate in pose.source_position {
        hash_u64(&mut hash, coordinate.to_bits());
    }
    hash_u64(&mut hash, pose.eye_height.to_bits());
    hash_bytes(&mut hash, &pose.heading_degrees.to_bits().to_le_bytes());
    hash_bytes(&mut hash, &pose.pitch_degrees.to_bits().to_le_bytes());
    for extent in pose.viewport {
        hash_bytes(&mut hash, &extent.to_le_bytes());
    }
    hash_bytes(
        &mut hash,
        &pose.vertical_fov_degrees.to_bits().to_le_bytes(),
    );
    hash
}

fn ordered_seg_loop(
    map: &DoomMapCore,
    segs: &[doom_map_provider::DoomSeg],
) -> (Option<Vec<[f64; 2]>>, usize) {
    if segs.len() < 3 {
        return (None, segs.len().max(1));
    }
    let points = segs
        .iter()
        .map(|seg| {
            let start = &map.vertices[usize::from(seg.start_vertex)];
            let end = &map.vertices[usize::from(seg.end_vertex)];
            ([start.x, start.y], [end.x, end.y])
        })
        .collect::<Vec<_>>();
    let mut gaps = 0;
    for index in 0..points.len() {
        let next = (index + 1) % points.len();
        if points[index].1 != points[next].0 {
            gaps += 1;
        }
    }
    if gaps == 0 {
        (
            Some(
                points
                    .into_iter()
                    .map(|(start, _)| start.map(f64::from))
                    .collect(),
            ),
            0,
        )
    } else {
        (None, gaps)
    }
}

fn polygons_cover_same_domain(left: &[[f64; 2]], right: &[[f64; 2]]) -> bool {
    if left.len() < 3 || right.len() < 3 {
        return false;
    }
    let left_area = polygon_signed_area(left).abs();
    let right_area = polygon_signed_area(right).abs();
    let area_tolerance = left_area.max(right_area).max(1.0) * DOMAIN_EPSILON;
    (left_area - right_area).abs() <= area_tolerance
        && left
            .iter()
            .all(|point| point_in_convex_domain(*point, right))
        && right
            .iter()
            .all(|point| point_in_convex_domain(*point, left))
}

fn point_in_convex_domain(point: [f64; 2], polygon: &[[f64; 2]]) -> bool {
    let mut positive = false;
    let mut negative = false;
    for index in 0..polygon.len() {
        let start = polygon[index];
        let end = polygon[(index + 1) % polygon.len()];
        let cross = (end[0] - start[0]) * (point[1] - start[1])
            - (end[1] - start[1]) * (point[0] - start[0]);
        positive |= cross > DOMAIN_EPSILON;
        negative |= cross < -DOMAIN_EPSILON;
        if positive && negative {
            return false;
        }
    }
    true
}

fn polygon_signed_area(vertices: &[[f64; 2]]) -> f64 {
    vertices
        .iter()
        .zip(vertices.iter().cycle().skip(1))
        .take(vertices.len())
        .map(|(left, right)| left[0] * right[1] - right[0] * left[1])
        .sum::<f64>()
        * 0.5
}

#[allow(clippy::too_many_arguments)]
fn triangulate_render_subsector(
    source_subsector: DoomSourceRecord,
    source_sector: DoomSourceRecord,
    boundary: &[[f64; 2]],
    floor_height: i16,
    ceiling_height: i16,
    floor_texture: &str,
    ceiling_texture: &str,
) -> Vec<RenderSubsectorTriangle> {
    let boundary = remove_collinear_vertices(boundary);
    if boundary.len() < 3 {
        return Vec::new();
    }
    let counter_clockwise = polygon_signed_area(&boundary) > 0.0;
    let mut triangles = Vec::with_capacity((boundary.len().saturating_sub(2)) * 2);
    for (plane, height, texture_name, role) in [
        (
            DoomSurfacePlane::Floor,
            floor_height,
            floor_texture,
            plane_role(floor_texture),
        ),
        (
            DoomSurfacePlane::Ceiling,
            ceiling_height,
            ceiling_texture,
            plane_role(ceiling_texture),
        ),
    ] {
        for index in 1..boundary.len() - 1 {
            let (second, third) = match (plane, counter_clockwise) {
                (DoomSurfacePlane::Floor, true) | (DoomSurfacePlane::Ceiling, false) => {
                    (boundary[index + 1], boundary[index])
                }
                (DoomSurfacePlane::Floor, false) | (DoomSurfacePlane::Ceiling, true) => {
                    (boundary[index], boundary[index + 1])
                }
            };
            let to_world = |point: [f64; 2]| [point[0], f64::from(height), point[1]];
            triangles.push(RenderSubsectorTriangle {
                source_subsector,
                source_sector,
                plane,
                role,
                texture_name: texture_name.to_owned(),
                positions: [to_world(boundary[0]), to_world(second), to_world(third)],
            });
        }
    }
    triangles
}

fn remove_collinear_vertices(boundary: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let mut result = boundary.to_vec();
    loop {
        if result.len() <= 3 {
            return result;
        }
        let removable = (0..result.len()).find(|&index| {
            let previous = result[(index + result.len() - 1) % result.len()];
            let current = result[index];
            let next = result[(index + 1) % result.len()];
            let cross = (current[0] - previous[0]) * (next[1] - current[1])
                - (current[1] - previous[1]) * (next[0] - current[0]);
            cross.abs() <= DOMAIN_EPSILON
        });
        let Some(index) = removable else {
            return result;
        };
        result.remove(index);
    }
}

fn plane_role(texture_name: &str) -> RenderSubsectorPlaneRole {
    if texture_name.eq_ignore_ascii_case("F_SKY1") {
        RenderSubsectorPlaneRole::Sky
    } else {
        RenderSubsectorPlaneRole::Ordinary
    }
}

fn triangle_normal_y(positions: [[f64; 3]; 3]) -> f64 {
    let left = [
        positions[1][0] - positions[0][0],
        positions[1][1] - positions[0][1],
        positions[1][2] - positions[0][2],
    ];
    let right = [
        positions[2][0] - positions[0][0],
        positions[2][1] - positions[0][1],
        positions[2][2] - positions[0][2],
    ];
    left[2] * right[0] - left[0] * right[2]
}

fn triangle_area(positions: [[f64; 3]; 3]) -> f64 {
    triangle_normal_y(positions).abs() * 0.5
}

fn fingerprint_boundary(source: DoomSourceRecord, boundary: &[[f64; 2]]) -> u64 {
    let mut hash = FINGERPRINT_OFFSET;
    hash_source(&mut hash, source);
    hash_u64(&mut hash, boundary.len() as u64);
    for point in boundary {
        hash_u64(&mut hash, point[0].to_bits());
        hash_u64(&mut hash, point[1].to_bits());
    }
    hash
}

fn fingerprint_walls(walls: &[RenderSubsectorWallSource]) -> u64 {
    let mut hash = FINGERPRINT_OFFSET;
    for wall in walls {
        hash_source(&mut hash, wall.source_seg);
        hash_source(&mut hash, wall.source_linedef);
        hash_source(&mut hash, wall.source_sidedef);
        hash_source(&mut hash, wall.source_sector);
        hash_u64(&mut hash, u64::from(wall.direction));
    }
    hash
}

fn fingerprint_wall_tiers(triangles: &[DoomSegTexturedWallTriangle]) -> u64 {
    let mut hash = FINGERPRINT_OFFSET;
    for triangle in triangles {
        hash_source(&mut hash, triangle.source_seg);
        hash_source(&mut hash, triangle.source_linedef);
        hash_source(&mut hash, triangle.source_sidedef);
        hash_source(&mut hash, triangle.source_sector);
        hash_bytes(
            &mut hash,
            match triangle.role {
                DoomWallTextureRole::Upper => b"upper",
                DoomWallTextureRole::Lower => b"lower",
                DoomWallTextureRole::Middle => b"middle",
            },
        );
        hash_bytes(&mut hash, triangle.texture_name.as_bytes());
        for position in triangle.positions {
            for coordinate in position {
                hash_u64(&mut hash, coordinate.to_bits());
            }
        }
        for coordinates in triangle.texture_coordinates {
            for coordinate in coordinates {
                hash_u64(&mut hash, coordinate.to_bits());
            }
        }
    }
    hash
}

fn fingerprint_triangles(triangles: &[RenderSubsectorTriangle]) -> u64 {
    let mut hash = FINGERPRINT_OFFSET;
    for triangle in triangles {
        hash_source(&mut hash, triangle.source_subsector);
        hash_source(&mut hash, triangle.source_sector);
        hash_bytes(
            &mut hash,
            match triangle.plane {
                DoomSurfacePlane::Floor => b"floor",
                DoomSurfacePlane::Ceiling => b"ceiling",
            },
        );
        hash_bytes(&mut hash, triangle.role.label().as_bytes());
        hash_bytes(&mut hash, triangle.texture_name.as_bytes());
        for position in triangle.positions {
            for coordinate in position {
                hash_u64(&mut hash, coordinate.to_bits());
            }
        }
    }
    hash
}

fn fingerprint_runtime_height(
    source_sector: DoomSourceRecord,
    floor_height: i16,
    ceiling_height: i16,
) -> u64 {
    let mut hash = FINGERPRINT_OFFSET;
    hash_source(&mut hash, source_sector);
    hash_bytes(&mut hash, &floor_height.to_le_bytes());
    hash_bytes(&mut hash, &ceiling_height.to_le_bytes());
    hash
}

fn fingerprint_all_runtime_heights(map: &DoomMapCore) -> u64 {
    let mut hash = FINGERPRINT_OFFSET;
    for sector in &map.sectors {
        hash_u64(
            &mut hash,
            fingerprint_runtime_height(sector.source, sector.floor_height, sector.ceiling_height),
        );
    }
    hash
}

fn fingerprint_map(map: &DoomMapCore) -> u64 {
    let mut hash = FINGERPRINT_OFFSET;
    hash_bytes(&mut hash, map.map_name.as_bytes());
    for vertex in &map.vertices {
        hash_source(&mut hash, vertex.source);
        hash_bytes(&mut hash, &vertex.x.to_le_bytes());
        hash_bytes(&mut hash, &vertex.y.to_le_bytes());
    }
    for seg in &map.segs {
        hash_source(&mut hash, seg.source);
        hash_bytes(&mut hash, &seg.start_vertex.to_le_bytes());
        hash_bytes(&mut hash, &seg.end_vertex.to_le_bytes());
        hash_bytes(&mut hash, &seg.linedef.to_le_bytes());
        hash_bytes(&mut hash, &seg.direction.to_le_bytes());
    }
    for subsector in &map.subsectors {
        hash_source(&mut hash, subsector.source);
        hash_bytes(&mut hash, &subsector.first_seg.to_le_bytes());
        hash_bytes(&mut hash, &subsector.seg_count.to_le_bytes());
    }
    for node in &map.nodes {
        hash_source(&mut hash, node.source);
        for value in [node.x, node.y, node.delta_x, node.delta_y] {
            hash_bytes(&mut hash, &value.to_le_bytes());
        }
    }
    hash_u64(&mut hash, fingerprint_all_runtime_heights(map));
    hash
}

fn fingerprint_camera(
    source_position: [i16; 2],
    source_angle: u16,
    eye_height: i16,
    viewport: [u32; 2],
    vertical_fov_degrees: f32,
    pitch_degrees: f32,
) -> u64 {
    let mut hash = FINGERPRINT_OFFSET;
    for coordinate in source_position {
        hash_bytes(&mut hash, &coordinate.to_le_bytes());
    }
    hash_bytes(&mut hash, &source_angle.to_le_bytes());
    hash_bytes(&mut hash, &eye_height.to_le_bytes());
    for extent in viewport {
        hash_bytes(&mut hash, &extent.to_le_bytes());
    }
    hash_bytes(&mut hash, &vertical_fov_degrees.to_bits().to_le_bytes());
    hash_bytes(&mut hash, &pitch_degrees.to_bits().to_le_bytes());
    hash
}

fn hash_source(hash: &mut u64, source: DoomSourceRecord) {
    hash_bytes(hash, &source.lump_index.to_le_bytes());
    hash_bytes(hash, &source.record_index.to_le_bytes());
}

fn hash_u64(hash: &mut u64, value: u64) {
    hash_bytes(hash, &value.to_le_bytes());
}

fn hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(FINGERPRINT_PRIME);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        interval_fully_covered, merge_angular_interval, point_in_convex_domain,
        polygons_cover_same_domain, segment_horizontal_interval, shared_collinear_interval,
        source_seg_front_facing, source_seg_solid_range_eligible, triangle_normal_y,
        triangulate_render_subsector, DoomSourceRecord, DoomSurfacePlane, DoomViewWindow,
    };

    #[test]
    fn domain_comparison_ignores_collinear_boundary_refinement() {
        let square = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]];
        let refined = [[0.0, 0.0], [2.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]];
        assert!(polygons_cover_same_domain(&square, &refined));
        assert!(point_in_convex_domain([4.0, 2.0], &square));
        assert!(!point_in_convex_domain([4.1, 2.0], &square));
    }

    #[test]
    fn floor_and_ceiling_triangles_face_opposite_vertical_directions() {
        let source = DoomSourceRecord {
            lump_index: 1,
            record_index: 2,
        };
        let triangles = triangulate_render_subsector(
            source,
            source,
            &[[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]],
            0,
            8,
            "FLOOR",
            "CEIL",
        );
        assert_eq!(triangles.len(), 4);
        assert!(triangles
            .iter()
            .filter(|triangle| triangle.plane == DoomSurfacePlane::Floor)
            .all(|triangle| triangle_normal_y(triangle.positions) > 0.0));
        assert!(triangles
            .iter()
            .filter(|triangle| triangle.plane == DoomSurfacePlane::Ceiling)
            .all(|triangle| triangle_normal_y(triangle.positions) < 0.0));
    }

    #[test]
    fn actual_horizontal_projection_clips_front_segments_and_rejects_rear_segments() {
        let half_fov = 45.0_f64.to_radians();
        let interval =
            segment_horizontal_interval([10.0, -1.0], [10.0, 1.0], [0.0, 0.0], 0.0, half_fov)
                .expect("front segment interval");
        assert!(interval[0] < 0.0);
        assert!(interval[1] > 0.0);
        assert!(segment_horizontal_interval(
            [-10.0, -1.0],
            [-10.0, 1.0],
            [0.0, 0.0],
            0.0,
            half_fov,
        )
        .is_none());
    }

    #[test]
    fn angular_coverage_merges_only_touching_intervals() {
        let mut coverage = vec![[-0.5, -0.1], [0.2, 0.4]];
        merge_angular_interval(&mut coverage, [-0.2, 0.25]);
        assert_eq!(coverage, vec![[-0.5, 0.4]]);
        assert!(interval_fully_covered([-0.3, 0.3], &coverage));
        assert!(!interval_fully_covered([-0.6, 0.3], &coverage));
    }

    #[test]
    fn source_solid_ranges_require_front_facing_segments_fully_beyond_the_near_plane() {
        let viewer = [0.0, 0.0];
        assert!(source_seg_front_facing(viewer, [10.0, 1.0], [10.0, -1.0]));
        assert!(source_seg_solid_range_eligible(
            viewer,
            0.0,
            [10.0, 1.0],
            [10.0, -1.0]
        ));
        assert!(!source_seg_solid_range_eligible(
            viewer,
            0.0,
            [10.0, 1.0],
            [-1.0, -1.0]
        ));
        assert!(!source_seg_front_facing(viewer, [10.0, -1.0], [10.0, 1.0]));
    }

    #[test]
    fn shared_boundary_overlap_accepts_refinement_but_not_point_contact() {
        assert_eq!(
            shared_collinear_interval([[0.0, 0.0], [4.0, 0.0]], [[3.0, 0.0], [1.0, 0.0]]),
            Some([[1.0, 0.0], [3.0, 0.0]])
        );
        assert_eq!(
            shared_collinear_interval([[0.0, 0.0], [1.0, 0.0]], [[1.0, 0.0], [2.0, 0.0]]),
            None
        );
        assert_eq!(
            shared_collinear_interval([[0.0, 0.0], [4.0, 0.0]], [[1.0, 1.0], [3.0, 1.0]]),
            None
        );
    }

    #[test]
    fn transferred_view_window_intersection_is_bounded_and_conservative() {
        let parent = DoomViewWindow {
            minimum_ndc: [-0.8, -0.6],
            maximum_ndc: [0.7, 0.9],
            depth: [0.0, 100.0],
        };
        let aperture = DoomViewWindow {
            minimum_ndc: [-0.2, -0.9],
            maximum_ndc: [0.9, 0.4],
            depth: [12.0, 18.0],
        };
        let transferred = parent.intersect(aperture).expect("overlap");
        assert_eq!(transferred.minimum_ndc, [-0.2, -0.6]);
        assert_eq!(transferred.maximum_ndc, [0.7, 0.4]);
        assert_eq!(transferred.depth, aperture.depth);
        assert!(parent.contains_window(transferred));
        assert!(transferred.contains_sample([0.0, 0.0]));
        assert!(!transferred.contains_sample([0.8, 0.0]));
        assert!(parent
            .intersect(DoomViewWindow {
                minimum_ndc: [0.8, -0.1],
                maximum_ndc: [1.0, 0.1],
                depth: [2.0, 3.0],
            })
            .is_none());
    }
}
