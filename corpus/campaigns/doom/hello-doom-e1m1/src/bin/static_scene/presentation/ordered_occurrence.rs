//! Continuous Doom source-occurrence observation for the E1M1 integration.
//!
//! This module deliberately does not consume Classic Doom's 320 diagnostic
//! columns. It walks source BSP leaves near-first, clips directed source SEGs
//! against continuous view half-spaces, and subtracts continuous projected
//! solid coverage. The resulting source-relative intervals are evidence for
//! later ordinary triangle lowering; they are not renderer scissors or a
//! generic visibility API.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    lower_static_flat_triangle, lower_static_seg_wall_triangle, FlatExtent, StaticDrawPlanEntry,
    StaticDrawSource, StaticFlatLoweringError, StaticTextureSourceKind, StaticTextureUpload,
};
use doom_geometry_provider::{
    clip_doom_seg_textured_wall_triangle_to_linedef_interval,
    lower_doom_seg_textured_wall_triangles, lower_doom_subsector_surfaces,
    observe_doom_classic_bsp, observe_doom_classic_vertical_clip_state, observe_doom_seg_occluders,
    observe_doom_seg_plane_marks, observe_doom_two_sided_middle_textures,
    resolve_doom_subsector_bsp_paths, DoomMiddleTextureObservation, DoomSegClassicPlaneKind,
    DoomSegOccluderKind, DoomSegOccluderObservation, DoomSegPlaneMarkObservation,
    DoomSegTexturedWallTriangle, DoomSurfacePlane, DoomSurfaceTriangle, DoomTextureExtent,
    DoomWallTextureRole,
};
use doom_map_provider::{DoomBspChild, DoomMapCore, DoomSector, DoomSeg};
use tokimu::MaterialHandle;

const HALF_FOV_TANGENT: f64 = 1.0;
const DEPTH_EPSILON: f64 = 1.0e-9;
const INTERVAL_EPSILON: f64 = 1.0e-9;
const CLASSIC_COLUMNS: usize = 320;
const CLASSIC_ROWS: usize = 200;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OrderedPlaneDomainCell {
    kind: OrderedPlaneKind,
    source_sector: u32,
    source_subsector: u32,
    source_height: i16,
    texture: String,
    light_level: i16,
    source_seg: u32,
    source_corners: [[f64; 2]; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct OrderedSourceOccurrence {
    pub(crate) source_seg: u32,
    pub(crate) source_linedef: u32,
    pub(crate) source_interval: [f64; 2],
    /// Continuous camera-horizontal interval retained by ordered coverage.
    /// This is provider-private preparation evidence, not a renderer scissor.
    pub(crate) view_interval: [f64; 2],
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum OrderedSourceDispositionKind {
    WholeRetained,
    TerminalRejected,
    PartialSeg,
    PartialPlane,
    UnresolvedFailOpen,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OrderedSourceDisposition {
    pub(crate) source_seg: u32,
    pub(crate) source_linedef: u32,
    pub(crate) kind: OrderedSourceDispositionKind,
    pub(crate) occurrence_count: usize,
    pub(crate) reason: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct OrderedSourceOccurrenceObservation {
    pub(crate) source_seg_records: usize,
    pub(crate) source_segs_visited: usize,
    pub(crate) whole_retained: usize,
    pub(crate) partial_retained: usize,
    pub(crate) whole_rejected: usize,
    pub(crate) unresolved_fail_open: usize,
    pub(crate) occurrences: Vec<OrderedSourceOccurrence>,
    pub(crate) dispositions: Vec<OrderedSourceDisposition>,
    pub(crate) fail_open_samples: Vec<String>,
}

/// Diagnostic-only identity used to correlate a retained LOOK hit with the
/// ordered Doom occurrence stream. It is not renderer vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrderedOccurrenceTraceTarget {
    Wall {
        source_linedef: u32,
    },
    Plane {
        source_subsector: u32,
        kind: OrderedPlaneKind,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct OrderedWallOccurrenceLoweringObservation {
    pub(crate) occurrences: usize,
    pub(crate) occurrences_with_wall_geometry: usize,
    pub(crate) occurrences_without_wall_geometry: usize,
    pub(crate) occurrences_unresolved: usize,
    pub(crate) matched_source_triangles: usize,
    pub(crate) matched_opaque_source_triangles: usize,
    pub(crate) matched_cutout_source_triangles: usize,
    pub(crate) material_resolved_source_triangles: usize,
    pub(crate) material_resolved_opaque_source_triangles: usize,
    pub(crate) material_resolved_cutout_source_triangles: usize,
    pub(crate) clipped_source_triangles: usize,
    pub(crate) clipped_opaque_triangles: usize,
    pub(crate) clipped_cutout_triangles: usize,
    pub(crate) lowered_wall_meshes: usize,
    pub(crate) lowered_opaque_meshes: usize,
    pub(crate) lowered_cutout_meshes: usize,
    pub(crate) degenerate_omissions: usize,
    pub(crate) unresolved_fail_open: usize,
    pub(crate) unresolved_samples: Vec<String>,
    pub(crate) source_dispositions: Vec<OrderedWallSourceDisposition>,
    pub(crate) prepared_declarations: Vec<PreparedOrderedWallDeclaration>,
    structural_fingerprint: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OrderedWallSourceDisposition {
    pub(crate) occurrence_ordinal: usize,
    pub(crate) source_seg: u32,
    pub(crate) source_triangle_ordinal: usize,
    pub(crate) cutout: bool,
    pub(crate) kind: OrderedSourceDispositionKind,
    pub(crate) output_count: usize,
    pub(crate) omission_count: usize,
    pub(crate) reason: String,
}

/// One ordinary Tokimu draw declaration produced from a Doom-owned retained
/// source occurrence. The occurrence metadata remains corpus-side evidence;
/// only `draw` is suitable for the renderer-facing declaration list.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PreparedOrderedWallDeclaration {
    pub(crate) occurrence: OrderedSourceOccurrence,
    pub(crate) occurrence_ordinal: usize,
    pub(crate) source_triangle_ordinal: usize,
    pub(crate) declaration_ordinal: usize,
    pub(crate) cutout: bool,
    pub(crate) draw: StaticDrawPlanEntry,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OrderedPreparedSubmissionObservation {
    pub(crate) source: OrderedSourceOccurrenceObservation,
    pub(crate) walls: OrderedWallOccurrenceLoweringObservation,
    pub(crate) planes: OrderedPlaneOccurrenceObservation,
    pub(crate) plane_lowering: OrderedPlaneLoweringObservation,
}

impl OrderedPreparedSubmissionObservation {
    pub(crate) fn verify_conservation(&self) -> Result<(), String> {
        self.source.verify_disposition_conservation()?;
        self.walls.verify_conservation()?;
        self.plane_lowering
            .verify_plane_disposition_conservation()?;

        let terminally_rejected = self
            .source
            .dispositions
            .iter()
            .filter(|disposition| {
                disposition.kind == OrderedSourceDispositionKind::TerminalRejected
            })
            .map(|disposition| disposition.source_seg)
            .collect::<BTreeSet<_>>();
        let reopened_wall =
            self.walls.prepared_declarations.iter().find(|declaration| {
                terminally_rejected.contains(&declaration.occurrence.source_seg)
            });
        let reopened_plane =
            self.planes.associations.iter().find(|association| {
                terminally_rejected.contains(&association.occurrence.source_seg)
            });
        if let Some(declaration) = reopened_wall {
            return Err(format!(
                "terminally rejected SEG {} re-entered through wall declaration {}",
                declaration.occurrence.source_seg, declaration.declaration_ordinal,
            ));
        }
        if let Some(association) = reopened_plane {
            return Err(format!(
                "terminally rejected SEG {} re-entered through {:?} plane association",
                association.occurrence.source_seg, association.kind,
            ));
        }

        let wall_dispositions = self
            .walls
            .source_dispositions
            .iter()
            .map(|disposition| {
                (
                    (
                        disposition.occurrence_ordinal,
                        disposition.source_seg,
                        disposition.source_triangle_ordinal,
                    ),
                    disposition,
                )
            })
            .collect::<BTreeMap<_, _>>();
        let mut wall_declaration_counts = BTreeMap::new();
        for declaration in &self.walls.prepared_declarations {
            let key = (
                declaration.occurrence_ordinal,
                declaration.occurrence.source_seg,
                declaration.source_triangle_ordinal,
            );
            let Some(disposition) = wall_dispositions.get(&key) else {
                return Err(format!(
                    "wall declaration {} has no source disposition for occurrence {} SEG {} triangle {}",
                    declaration.declaration_ordinal,
                    declaration.occurrence_ordinal,
                    declaration.occurrence.source_seg,
                    declaration.source_triangle_ordinal,
                ));
            };
            if matches!(
                disposition.kind,
                OrderedSourceDispositionKind::TerminalRejected
                    | OrderedSourceDispositionKind::UnresolvedFailOpen
            ) {
                return Err(format!(
                    "{:?} wall source occurrence {} SEG {} triangle {} re-entered through declaration {}",
                    disposition.kind,
                    declaration.occurrence_ordinal,
                    declaration.occurrence.source_seg,
                    declaration.source_triangle_ordinal,
                    declaration.declaration_ordinal,
                ));
            }
            *wall_declaration_counts.entry(key).or_insert(0usize) += 1;
        }
        for (key, disposition) in &wall_dispositions {
            let declaration_count = wall_declaration_counts.get(key).copied().unwrap_or(0);
            if declaration_count != disposition.output_count {
                return Err(format!(
                    "wall source occurrence {} SEG {} triangle {} declaration conservation failed: declarations={declaration_count}, disposition-outputs={}",
                    key.0, key.1, key.2, disposition.output_count,
                ));
            }
        }

        let plane_dispositions = self
            .plane_lowering
            .source_dispositions
            .iter()
            .map(|disposition| {
                (
                    (
                        disposition.plane_instance_ordinal,
                        disposition.source_subsector,
                        disposition.source_triangle_ordinal,
                    ),
                    disposition,
                )
            })
            .collect::<BTreeMap<_, _>>();
        let mut plane_declaration_counts = BTreeMap::new();
        for declaration in &self.plane_lowering.prepared_declarations {
            let key = (
                declaration.plane_instance_ordinal,
                declaration.source_subsector,
                declaration.source_triangle_ordinal,
            );
            let Some(disposition) = plane_dispositions.get(&key) else {
                return Err(format!(
                    "plane declaration has no source disposition for instance {} subsector {} triangle {}",
                    key.0, key.1, key.2,
                ));
            };
            if matches!(
                disposition.kind,
                OrderedSourceDispositionKind::TerminalRejected
                    | OrderedSourceDispositionKind::UnresolvedFailOpen
            ) {
                return Err(format!(
                    "{:?} plane source instance {} subsector {} triangle {} re-entered through a declaration",
                    disposition.kind, key.0, key.1, key.2,
                ));
            }
            *plane_declaration_counts.entry(key).or_insert(0usize) += 1;
        }
        for (key, disposition) in &plane_dispositions {
            let declaration_count = plane_declaration_counts.get(key).copied().unwrap_or(0);
            // Sky source triangles become Doom's background contribution and
            // deliberately do not lower to ordinary plane declarations.
            let sky_background = self
                .planes
                .plane_instances
                .get(disposition.plane_instance_ordinal)
                .is_some_and(|instance| instance.sky);
            let expected = if sky_background {
                0
            } else {
                disposition.output_count
            };
            if declaration_count != expected {
                return Err(format!(
                    "plane source instance {} subsector {} triangle {} declaration conservation failed: declarations={declaration_count}, expected={expected}, disposition-outputs={}",
                    key.0, key.1, key.2, disposition.output_count,
                ));
            }
        }

        let opaque_walls = self
            .walls
            .prepared_declarations
            .iter()
            .filter(|declaration| !declaration.cutout)
            .count();
        let cutout_walls = self
            .walls
            .prepared_declarations
            .iter()
            .filter(|declaration| declaration.cutout)
            .count();
        let floors = self
            .plane_lowering
            .prepared_declarations
            .iter()
            .filter(|declaration| {
                self.planes
                    .plane_instances
                    .get(declaration.plane_instance_ordinal)
                    .is_some_and(|instance| instance.kind == OrderedPlaneKind::Floor)
            })
            .count();
        let ceilings = self
            .plane_lowering
            .prepared_declarations
            .iter()
            .filter(|declaration| {
                self.planes
                    .plane_instances
                    .get(declaration.plane_instance_ordinal)
                    .is_some_and(|instance| instance.kind == OrderedPlaneKind::Ceiling)
            })
            .count();
        if opaque_walls != self.walls.lowered_opaque_meshes
            || cutout_walls != self.walls.lowered_cutout_meshes
            || floors + ceilings != self.plane_lowering.lowered_plane_meshes
        {
            return Err(format!(
                "ordered family conservation failed: opaque-walls={opaque_walls}/{}, cutout-walls={cutout_walls}/{}, floors+ceilings={}/{},",
                self.walls.lowered_opaque_meshes,
                self.walls.lowered_cutout_meshes,
                floors + ceilings,
                self.plane_lowering.lowered_plane_meshes,
            ));
        }
        Ok(())
    }

    pub(crate) fn family_conservation_report(&self) -> String {
        let opaque_walls = self
            .walls
            .prepared_declarations
            .iter()
            .filter(|declaration| !declaration.cutout)
            .count();
        let cutouts = self
            .walls
            .prepared_declarations
            .iter()
            .filter(|declaration| declaration.cutout)
            .count();
        let floors = self
            .plane_lowering
            .prepared_declarations
            .iter()
            .filter(|declaration| {
                self.planes
                    .plane_instances
                    .get(declaration.plane_instance_ordinal)
                    .is_some_and(|instance| instance.kind == OrderedPlaneKind::Floor)
            })
            .count();
        let ceilings = self
            .plane_lowering
            .prepared_declarations
            .iter()
            .filter(|declaration| {
                self.planes
                    .plane_instances
                    .get(declaration.plane_instance_ordinal)
                    .is_some_and(|instance| instance.kind == OrderedPlaneKind::Ceiling)
            })
            .count();
        format!(
            "source-segs={}; dispositions={}; terminal-rejected={}; wall-opaque={}; cutout={}; floor={}; ceiling={}; sky-source-triangles={}; unresolved={}; conservation={}",
            self.source.source_seg_records,
            self.source.dispositions.len(),
            self.source.whole_rejected,
            opaque_walls,
            cutouts,
            floors,
            ceilings,
            self.plane_lowering.sky_background_source_triangles,
            self.source.unresolved_fail_open
                + self.walls.unresolved_fail_open
                + self.planes.unresolved_fail_open
                + self.planes.plane_destination_unresolved_fail_open
                + self.plane_lowering.unresolved_fail_open,
            if self.verify_conservation().is_ok() { "balanced" } else { "unbalanced" },
        )
    }
}

/// Runs the coherent ordered-occurrence preparation seam for one immutable
/// Doom source/runtime snapshot. Application activation and timing policy stay
/// outside this function; callers supply the already-current map facts.
#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare_ordered_occurrence_submission(
    map: &DoomMapCore,
    viewer: [i16; 2],
    heading: f64,
    eye_height: i16,
    extents: &[DoomTextureExtent],
    opaque_materials: &BTreeMap<String, MaterialHandle>,
    cutout_materials: &BTreeMap<String, MaterialHandle>,
    opaque_uploads: &[StaticTextureUpload],
) -> Result<OrderedPreparedSubmissionObservation, String> {
    let source = observe_ordered_source_occurrences(map, viewer, heading)?;
    let walls = observe_ordered_wall_occurrence_lowering(
        map,
        extents,
        opaque_materials,
        cutout_materials,
        &source,
    )?;
    let planes =
        observe_ordered_plane_occurrences(map, viewer, heading, eye_height, &source, &walls)?;
    let plane_domain_cells =
        observe_ordered_plane_domain_cells(map, viewer, heading, eye_height, extents)?;
    let plane_lowering =
        lower_ordered_plane_destinations(map, opaque_uploads, &planes, &plane_domain_cells)?;
    let prepared = OrderedPreparedSubmissionObservation {
        source,
        walls,
        planes,
        plane_lowering,
    };
    prepared.verify_conservation()?;
    Ok(prepared)
}

/// Renderer-ready declarations from one immutable Doom source/view/snapshot
/// preparation. The source protocol and its detailed accounting remain private
/// to this corpus crate; hosts receive only ordinary declarations plus a
/// bounded conservation report.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedOrderedOccurrenceDeclarations {
    pub opaque_draws: Vec<StaticDrawPlanEntry>,
    pub cutout_draws: Vec<StaticDrawPlanEntry>,
    pub conservation_report: String,
}

/// Shared Doom-private preparation entry point for native and browser corpus
/// hosts. This is intentionally not a renderer API: it accepts decoded Doom
/// source facts and returns ordinary Tokimu declarations for one current view.
#[allow(dead_code, clippy::too_many_arguments)]
pub fn prepare_ordered_occurrence_declarations(
    map: &DoomMapCore,
    viewer: [i16; 2],
    heading: f64,
    eye_height: i16,
    extents: &[DoomTextureExtent],
    opaque_materials: &BTreeMap<String, MaterialHandle>,
    cutout_materials: &BTreeMap<String, MaterialHandle>,
    opaque_uploads: &[StaticTextureUpload],
) -> Result<PreparedOrderedOccurrenceDeclarations, String> {
    let prepared = prepare_ordered_occurrence_submission(
        map,
        viewer,
        heading,
        eye_height,
        extents,
        opaque_materials,
        cutout_materials,
        opaque_uploads,
    )?;
    Ok(PreparedOrderedOccurrenceDeclarations {
        opaque_draws: prepared
            .walls
            .prepared_declarations
            .iter()
            .filter(|declaration| !declaration.cutout)
            .map(|declaration| declaration.draw.clone())
            .chain(
                prepared
                    .plane_lowering
                    .prepared_declarations
                    .iter()
                    .map(|declaration| declaration.draw.clone()),
            )
            .collect(),
        cutout_draws: prepared
            .walls
            .prepared_declarations
            .iter()
            .filter(|declaration| declaration.cutout)
            .map(|declaration| declaration.draw.clone())
            .collect(),
        conservation_report: prepared.family_conservation_report(),
    })
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum OrderedPlaneKind {
    Floor,
    Ceiling,
}

/// One source plane association retained by the same continuous occurrence
/// observation as the wall manifest. This is deliberately not yet a renderer
/// declaration: a source plane mark identifies ownership, but continuous
/// vertical coverage still has to be prepared before ordinary mesh lowering.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OrderedPlaneOccurrence {
    pub(crate) occurrence: OrderedSourceOccurrence,
    pub(crate) source_subsector: u32,
    pub(crate) kind: OrderedPlaneKind,
    pub(crate) source_sector: u32,
    pub(crate) source_height: i16,
    pub(crate) texture: String,
    pub(crate) light_level: i16,
    pub(crate) sky: bool,
}

/// One source plane identity reached through retained ordered occurrences.
/// The admitted subsectors retain region provenance without claiming that a
/// whole subsector polygon is visible or ready for renderer submission.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OrderedPreparedPlaneInstance {
    pub(crate) kind: OrderedPlaneKind,
    pub(crate) source_sector: u32,
    pub(crate) source_height: i16,
    pub(crate) texture: String,
    pub(crate) light_level: i16,
    pub(crate) sky: bool,
    pub(crate) occurrence_references: usize,
    pub(crate) source_subsectors: Vec<u32>,
}

/// Exact source-region geometry destination for one prepared plane instance.
/// This proves that retained source meaning has somewhere to lower without
/// claiming that the entire region is viewer-visible or renderer-ready.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OrderedPreparedPlaneDestination {
    pub(crate) plane_instance_ordinal: usize,
    pub(crate) kind: OrderedPlaneKind,
    pub(crate) source_sector: u32,
    pub(crate) source_subsector: u32,
    pub(crate) source_triangles: usize,
    pub(crate) occurrence_references: usize,
    /// Disjoint camera-horizontal domains authorized by the source
    /// occurrences reaching this exact plane destination.
    pub(crate) view_intervals: Vec<[f64; 2]>,
}

/// One ordinary non-sky plane declaration produced from an exact retained
/// source-region destination. Sky destinations are accounted separately and
/// resolve to Doom's background presentation rather than depth-writing plane
/// geometry.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PreparedOrderedPlaneDeclaration {
    pub(crate) plane_instance_ordinal: usize,
    pub(crate) source_subsector: u32,
    pub(crate) source_triangle_ordinal: usize,
    pub(crate) draw: StaticDrawPlanEntry,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OrderedPlaneSourceDisposition {
    pub(crate) plane_instance_ordinal: usize,
    pub(crate) source_subsector: u32,
    pub(crate) source_triangle_ordinal: usize,
    pub(crate) kind: OrderedSourceDispositionKind,
    pub(crate) output_count: usize,
    pub(crate) reason: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct OrderedPlaneLoweringObservation {
    pub(crate) destination_references: usize,
    pub(crate) destination_source_triangles: usize,
    pub(crate) ordinary_destination_references: usize,
    pub(crate) ordinary_source_triangles: usize,
    pub(crate) ordinary_source_triangles_with_survivors: usize,
    pub(crate) ordinary_source_triangles_fully_rejected: usize,
    pub(crate) clipped_plane_triangles: usize,
    pub(crate) partial_plane_domain_cells: usize,
    pub(crate) partial_plane_domain_fragments: usize,
    pub(crate) sky_background_destination_references: usize,
    pub(crate) sky_background_source_triangles: usize,
    pub(crate) lowered_plane_meshes: usize,
    pub(crate) lowered_plane_triangles: usize,
    pub(crate) degenerate_omissions: usize,
    pub(crate) unresolved_fail_open: usize,
    pub(crate) source_dispositions: Vec<OrderedPlaneSourceDisposition>,
    pub(crate) prepared_declarations: Vec<PreparedOrderedPlaneDeclaration>,
    pub(crate) unresolved_samples: Vec<String>,
}

impl OrderedPlaneLoweringObservation {
    pub(crate) fn report(&self) -> String {
        format!(
            "destination-references={}; destination-source-triangles={}; ordinary-destination-references={}; ordinary-source-triangles={}; ordinary-source-triangles-with-survivors={}; ordinary-source-triangles-fully-rejected={}; clipped-plane-triangles={}; partial-plane-domain-cells={}; partial-plane-domain-fragments={}; sky-background-destination-references={}; sky-background-source-triangles={}; lowered-plane-meshes={}; lowered-plane-triangles={}; degenerate-omissions={}; unresolved-fail-open={}; source-dispositions={}; disposition-conservation={}; destination-conservation={}; source-triangle-conservation={}; fragment-conservation={}; declaration-conservation={}; sky-realization=doom-background-not-depth-writing-plane; whole-region-visibility-claimed=false; unresolved-samples=[{}]",
            self.destination_references,
            self.destination_source_triangles,
            self.ordinary_destination_references,
            self.ordinary_source_triangles,
            self.ordinary_source_triangles_with_survivors,
            self.ordinary_source_triangles_fully_rejected,
            self.clipped_plane_triangles,
            self.partial_plane_domain_cells,
            self.partial_plane_domain_fragments,
            self.sky_background_destination_references,
            self.sky_background_source_triangles,
            self.lowered_plane_meshes,
            self.lowered_plane_triangles,
            self.degenerate_omissions,
            self.unresolved_fail_open,
            self.source_dispositions.len(),
            if self.plane_disposition_conservation_is_balanced() {
                "balanced"
            } else {
                "unbalanced"
            },
            if self.destination_references
                == self.ordinary_destination_references
                    + self.sky_background_destination_references
                    + self.unresolved_fail_open
            {
                "balanced"
            } else {
                "unbalanced"
            },
            if self.ordinary_source_triangles
                == self.ordinary_source_triangles_with_survivors
                    + self.ordinary_source_triangles_fully_rejected
            {
                "balanced"
            } else {
                "unbalanced"
            },
            if self.clipped_plane_triangles
                == self.lowered_plane_triangles + self.degenerate_omissions
            {
                "balanced"
            } else {
                "unbalanced"
            },
            if self.lowered_plane_meshes == self.prepared_declarations.len() {
                "balanced"
            } else {
                "unbalanced"
            },
            self.unresolved_samples.join(" | "),
        )
    }

    fn retain_unresolved(&mut self, reason: String) {
        self.unresolved_fail_open += 1;
        if self.unresolved_samples.len() < 12 {
            self.unresolved_samples.push(reason);
        }
    }

    fn plane_disposition_conservation_is_balanced(&self) -> bool {
        let identities = self
            .source_dispositions
            .iter()
            .map(|disposition| {
                (
                    disposition.plane_instance_ordinal,
                    disposition.source_subsector,
                    disposition.source_triangle_ordinal,
                )
            })
            .collect::<BTreeSet<_>>();
        self.source_dispositions.len() == self.destination_source_triangles
            && identities.len() == self.source_dispositions.len()
            && self
                .source_dispositions
                .iter()
                .all(|disposition| match disposition.kind {
                    OrderedSourceDispositionKind::TerminalRejected => disposition.output_count == 0,
                    OrderedSourceDispositionKind::WholeRetained
                    | OrderedSourceDispositionKind::PartialPlane => disposition.output_count > 0,
                    OrderedSourceDispositionKind::UnresolvedFailOpen => {
                        disposition.output_count == 0
                    }
                    OrderedSourceDispositionKind::PartialSeg => false,
                })
    }

    fn verify_plane_disposition_conservation(&self) -> Result<(), String> {
        if self.plane_disposition_conservation_is_balanced() {
            return Ok(());
        }
        let mut identity_counts = BTreeMap::new();
        for disposition in &self.source_dispositions {
            *identity_counts
                .entry((
                    disposition.plane_instance_ordinal,
                    disposition.source_subsector,
                    disposition.source_triangle_ordinal,
                ))
                .or_insert(0usize) += 1;
        }
        let duplicates = identity_counts
            .iter()
            .filter(|(_, count)| **count > 1)
            .take(4)
            .map(|(identity, count)| format!("{identity:?}x{count}"))
            .collect::<Vec<_>>()
            .join("|");
        let invalid = self
            .source_dispositions
            .iter()
            .filter(|disposition| match disposition.kind {
                OrderedSourceDispositionKind::TerminalRejected => disposition.output_count != 0,
                OrderedSourceDispositionKind::WholeRetained
                | OrderedSourceDispositionKind::PartialPlane => disposition.output_count == 0,
                OrderedSourceDispositionKind::UnresolvedFailOpen => disposition.output_count != 0,
                OrderedSourceDispositionKind::PartialSeg => true,
            })
            .take(4)
            .map(|disposition| {
                format!(
                    "({},{},{})={:?}/outputs={}/{}",
                    disposition.plane_instance_ordinal,
                    disposition.source_subsector,
                    disposition.source_triangle_ordinal,
                    disposition.kind,
                    disposition.output_count,
                    disposition.reason,
                )
            })
            .collect::<Vec<_>>()
            .join("|");
        Err(format!(
            "ordered plane disposition conservation failed: source-triangles={} dispositions={} unique={} duplicates=[{}] invalid=[{}]",
            self.destination_source_triangles,
            self.source_dispositions.len(),
            identity_counts.len(),
            duplicates,
            invalid,
        ))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct OrderedPlaneOccurrenceObservation {
    pub(crate) viewer: [i16; 2],
    pub(crate) heading: f64,
    pub(crate) occurrences: usize,
    pub(crate) occurrences_with_marked_planes: usize,
    pub(crate) occurrences_without_marked_planes: usize,
    pub(crate) floor_associations: usize,
    pub(crate) ceiling_associations: usize,
    pub(crate) sky_ceiling_associations: usize,
    pub(crate) paired_sky_adjustments: usize,
    pub(crate) distinct_floor_planes: usize,
    pub(crate) distinct_ceiling_planes: usize,
    pub(crate) distinct_sky_ceiling_planes: usize,
    pub(crate) one_sided_boundaries: usize,
    pub(crate) open_two_sided_boundaries: usize,
    pub(crate) closed_two_sided_boundaries: usize,
    pub(crate) wall_consumer_references: usize,
    pub(crate) plane_consumer_references: usize,
    pub(crate) plane_instance_occurrence_references: usize,
    pub(crate) plane_instance_subsector_references: usize,
    pub(crate) plane_destination_references: usize,
    pub(crate) plane_destination_source_triangles: usize,
    pub(crate) plane_destination_unresolved_fail_open: usize,
    pub(crate) boundaries: Vec<OrderedPreparedBoundary>,
    pub(crate) plane_instances: Vec<OrderedPreparedPlaneInstance>,
    pub(crate) plane_destinations: Vec<OrderedPreparedPlaneDestination>,
    pub(crate) plane_destination_unresolved_samples: Vec<String>,
    pub(crate) unresolved_fail_open: usize,
    pub(crate) unresolved_samples: Vec<String>,
    pub(crate) associations: Vec<OrderedPlaneOccurrence>,
}

/// The one continuous vertical boundary shared by all consumers of a retained
/// Doom source occurrence. This remains Doom-private evidence: it is neither a
/// renderer clip rectangle nor a generic scene boundary.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OrderedPreparedBoundary {
    pub(crate) occurrence: OrderedSourceOccurrence,
    pub(crate) front_sector: u32,
    pub(crate) back_sector: Option<u32>,
    pub(crate) front_vertical: [i16; 2],
    pub(crate) back_vertical: Option<[i16; 2]>,
    pub(crate) opening: Option<[i16; 2]>,
    pub(crate) paired_sky_ceiling_adjustment: bool,
    pub(crate) wall_consumers: usize,
    pub(crate) plane_consumers: usize,
}

impl OrderedPlaneOccurrenceObservation {
    pub(crate) fn report(&self) -> String {
        format!(
            "occurrences={}; with-marked-planes={}; without-marked-planes={}; associations={}; floor-associations={}; ceiling-associations={}; sky-ceiling-associations={}; paired-sky-adjustments={}; distinct-floor-planes={}; distinct-ceiling-planes={}; distinct-sky-ceiling-planes={}; plane-instances={}; plane-instance-occurrence-references={}; plane-instance-subsector-references={}; plane-destination-references={}; plane-destination-source-triangles={}; plane-destination-unresolved-fail-open={}; boundaries={}; one-sided-boundaries={}; open-two-sided-boundaries={}; closed-two-sided-boundaries={}; wall-consumer-references={}; plane-consumer-references={}; unresolved-fail-open={}; occurrence-conservation={}; association-conservation={}; plane-instance-conservation={}; plane-destination-conservation={}; boundary-conservation={}; consumer-conservation={}; continuous-vertical-coverage-ready=true; legacy-screen-columns-used=false; unresolved-samples=[{}]; plane-destination-unresolved-samples=[{}]",
            self.occurrences,
            self.occurrences_with_marked_planes,
            self.occurrences_without_marked_planes,
            self.associations.len(),
            self.floor_associations,
            self.ceiling_associations,
            self.sky_ceiling_associations,
            self.paired_sky_adjustments,
            self.distinct_floor_planes,
            self.distinct_ceiling_planes,
            self.distinct_sky_ceiling_planes,
            self.plane_instances.len(),
            self.plane_instance_occurrence_references,
            self.plane_instance_subsector_references,
            self.plane_destination_references,
            self.plane_destination_source_triangles,
            self.plane_destination_unresolved_fail_open,
            self.boundaries.len(),
            self.one_sided_boundaries,
            self.open_two_sided_boundaries,
            self.closed_two_sided_boundaries,
            self.wall_consumer_references,
            self.plane_consumer_references,
            self.unresolved_fail_open,
            if self.occurrences
                == self.occurrences_with_marked_planes
                    + self.occurrences_without_marked_planes
                    + self.unresolved_fail_open
            {
                "balanced"
            } else {
                "unbalanced"
            },
            if self.associations.len() == self.floor_associations + self.ceiling_associations {
                "balanced"
            } else {
                "unbalanced"
            },
            if self.associations.len() == self.plane_instance_occurrence_references {
                "balanced"
            } else {
                "unbalanced"
            },
            if self.plane_instance_subsector_references
                == self.plane_destination_references
                    + self.plane_destination_unresolved_fail_open
            {
                "balanced"
            } else {
                "unbalanced"
            },
            if self.boundaries.len() + self.unresolved_fail_open == self.occurrences {
                "balanced"
            } else {
                "unbalanced"
            },
            if self.wall_consumer_references
                == self
                    .boundaries
                    .iter()
                    .map(|boundary| boundary.wall_consumers)
                    .sum::<usize>()
                && self.plane_consumer_references
                    == self
                        .boundaries
                        .iter()
                        .map(|boundary| boundary.plane_consumers)
                        .sum::<usize>()
            {
                "balanced"
            } else {
                "unbalanced"
            },
            self.unresolved_samples.join(" | "),
            self.plane_destination_unresolved_samples.join(" | "),
        )
    }

    fn retain_unresolved(&mut self, reason: String) {
        self.unresolved_fail_open += 1;
        if self.unresolved_samples.len() < 12 {
            self.unresolved_samples.push(reason);
        }
    }
}

impl OrderedWallOccurrenceLoweringObservation {
    pub(crate) fn report(&self) -> String {
        format!(
            "occurrences={}; with-wall-geometry={}; without-wall-geometry={}; unresolved-occurrences={}; matched-source-triangles={}; matched-opaque-source-triangles={}; matched-cutout-source-triangles={}; material-resolved-source-triangles={}; material-resolved-opaque-source-triangles={}; material-resolved-cutout-source-triangles={}; clipped-source-triangles={}; clipped-opaque-triangles={}; clipped-cutout-triangles={}; lowered-wall-meshes={}; lowered-opaque-meshes={}; lowered-cutout-meshes={}; source-dispositions={}; prepared-declarations={}; prepared-opaque-declarations={}; prepared-cutout-declarations={}; degenerate-omissions={}; unresolved-fail-open={}; structural-fingerprint={:016x}; conservation={}; category-conservation={}; material-conservation={}; disposition-conservation={}; declaration-conservation={}; unresolved-samples=[{}]",
            self.occurrences,
            self.occurrences_with_wall_geometry,
            self.occurrences_without_wall_geometry,
            self.occurrences_unresolved,
            self.matched_source_triangles,
            self.matched_opaque_source_triangles,
            self.matched_cutout_source_triangles,
            self.material_resolved_source_triangles,
            self.material_resolved_opaque_source_triangles,
            self.material_resolved_cutout_source_triangles,
            self.clipped_source_triangles,
            self.clipped_opaque_triangles,
            self.clipped_cutout_triangles,
            self.lowered_wall_meshes,
            self.lowered_opaque_meshes,
            self.lowered_cutout_meshes,
            self.source_dispositions.len(),
            self.prepared_declarations.len(),
            self.prepared_declarations
                .iter()
                .filter(|declaration| !declaration.cutout)
                .count(),
            self.prepared_declarations
                .iter()
                .filter(|declaration| declaration.cutout)
                .count(),
            self.degenerate_omissions,
            self.unresolved_fail_open,
            self.structural_fingerprint,
            if self.occurrences
                == self.occurrences_with_wall_geometry
                    + self.occurrences_without_wall_geometry
                    + self.occurrences_unresolved
            {
                "balanced"
            } else {
                "unbalanced"
            },
            if self.matched_source_triangles
                == self.matched_opaque_source_triangles + self.matched_cutout_source_triangles
                && self.clipped_source_triangles
                    == self.clipped_opaque_triangles + self.clipped_cutout_triangles
                && self.lowered_wall_meshes
                    == self.lowered_opaque_meshes + self.lowered_cutout_meshes
            {
                "balanced"
            } else {
                "unbalanced"
            },
            if self.material_resolved_source_triangles
                    == self.material_resolved_opaque_source_triangles
                        + self.material_resolved_cutout_source_triangles
            {
                "balanced"
            } else {
                "unbalanced"
            },
            if self.source_dispositions.len() == self.matched_source_triangles
                && self
                    .source_dispositions
                    .iter()
                    .map(|disposition| {
                        (
                            disposition.occurrence_ordinal,
                            disposition.source_seg,
                            disposition.source_triangle_ordinal,
                        )
                    })
                    .collect::<BTreeSet<_>>()
                    .len()
                    == self.source_dispositions.len()
            {
                "balanced"
            } else {
                "unbalanced"
            },
            if self.lowered_wall_meshes == self.prepared_declarations.len()
                && self.lowered_opaque_meshes
                    == self
                        .prepared_declarations
                        .iter()
                        .filter(|declaration| !declaration.cutout)
                        .count()
                && self.lowered_cutout_meshes
                    == self
                        .prepared_declarations
                        .iter()
                        .filter(|declaration| declaration.cutout)
                        .count()
            {
                "balanced"
            } else {
                "unbalanced"
            },
            self.unresolved_samples.join(" | "),
        )
    }

    fn verify_conservation(&self) -> Result<(), String> {
        let unique_source_triangles = self
            .source_dispositions
            .iter()
            .map(|disposition| {
                (
                    disposition.occurrence_ordinal,
                    disposition.source_seg,
                    disposition.source_triangle_ordinal,
                )
            })
            .collect::<BTreeSet<_>>()
            .len();
        let disposition_outputs = self
            .source_dispositions
            .iter()
            .map(|disposition| disposition.output_count)
            .sum::<usize>();
        let disposition_omissions = self
            .source_dispositions
            .iter()
            .map(|disposition| disposition.omission_count)
            .sum::<usize>();
        let balanced = self.occurrences
            == self.occurrences_with_wall_geometry
                + self.occurrences_without_wall_geometry
                + self.occurrences_unresolved
            && self.matched_source_triangles
                == self.matched_opaque_source_triangles + self.matched_cutout_source_triangles
            && self.clipped_source_triangles
                == self.clipped_opaque_triangles + self.clipped_cutout_triangles
            && self.lowered_wall_meshes == self.lowered_opaque_meshes + self.lowered_cutout_meshes
            && self.material_resolved_source_triangles
                == self.material_resolved_opaque_source_triangles
                    + self.material_resolved_cutout_source_triangles
            && self.source_dispositions.len() == self.matched_source_triangles
            && unique_source_triangles == self.source_dispositions.len()
            && disposition_outputs == self.lowered_wall_meshes
            && disposition_omissions == self.degenerate_omissions
            && self
                .source_dispositions
                .iter()
                .all(|disposition| match disposition.kind {
                    OrderedSourceDispositionKind::WholeRetained
                    | OrderedSourceDispositionKind::PartialSeg => {
                        disposition.output_count + disposition.omission_count > 0
                    }
                    OrderedSourceDispositionKind::TerminalRejected
                    | OrderedSourceDispositionKind::UnresolvedFailOpen => {
                        disposition.output_count == 0 && disposition.omission_count == 0
                    }
                    OrderedSourceDispositionKind::PartialPlane => false,
                })
            && self.lowered_wall_meshes == self.prepared_declarations.len();
        balanced.then_some(()).ok_or_else(|| {
            format!(
                "ordered wall conservation failed: occurrences={} with={} without={} unresolved-occurrences={} matched={} dispositions={} unique={} unresolved-dispositions={} resolved={} lowered={} disposition-outputs={} omissions={}/{} declarations={}",
                self.occurrences,
                self.occurrences_with_wall_geometry,
                self.occurrences_without_wall_geometry,
                self.occurrences_unresolved,
                self.matched_source_triangles,
                self.source_dispositions.len(),
                unique_source_triangles,
                self.unresolved_fail_open,
                self.material_resolved_source_triangles,
                self.lowered_wall_meshes,
                disposition_outputs,
                disposition_omissions,
                self.degenerate_omissions,
                self.prepared_declarations.len(),
            )
        })
    }
}

impl OrderedSourceOccurrenceObservation {
    pub(crate) fn report(&self) -> String {
        let samples = self
            .occurrences
            .iter()
            .take(12)
            .map(|occurrence| {
                format!(
                    "seg={}/linedef={}/source=[{:.6},{:.6}]",
                    occurrence.source_seg,
                    occurrence.source_linedef,
                    occurrence.source_interval[0],
                    occurrence.source_interval[1],
                )
            })
            .collect::<Vec<_>>()
            .join(" | ");
        format!(
            "source-seg-records={}; source-segs-visited={}; whole-retained={}; partial-retained={}; whole-rejected={}; unresolved-fail-open={}; occurrences={}; dispositions={}; disposition-conservation={}; occurrence-fingerprint={:016x}; continuous-source-domain=true; diagnostic-columns-authoritative=false; occurrence-samples=[{}]; fail-open-samples=[{}]",
            self.source_seg_records,
            self.source_segs_visited,
            self.whole_retained,
            self.partial_retained,
            self.whole_rejected,
            self.unresolved_fail_open,
            self.occurrences.len(),
            self.dispositions.len(),
            if self.disposition_conservation_is_balanced() {
                "balanced"
            } else {
                "unbalanced"
            },
            self.occurrence_fingerprint(),
            samples,
            self.fail_open_samples.join(" | "),
        )
    }

    fn disposition_conservation_is_balanced(&self) -> bool {
        let unique_segs = self
            .dispositions
            .iter()
            .map(|disposition| disposition.source_seg)
            .collect::<BTreeSet<_>>()
            .len();
        let retained_occurrences = self
            .dispositions
            .iter()
            .map(|disposition| disposition.occurrence_count)
            .sum::<usize>();
        self.dispositions.len() == self.source_segs_visited
            && unique_segs == self.source_segs_visited
            && self.source_segs_visited == self.source_seg_records
            && self.dispositions.len()
                == self.whole_retained
                    + self.partial_retained
                    + self.whole_rejected
                    + self.unresolved_fail_open
            && retained_occurrences == self.occurrences.len()
            && self
                .dispositions
                .iter()
                .all(|disposition| match disposition.kind {
                    OrderedSourceDispositionKind::WholeRetained
                    | OrderedSourceDispositionKind::PartialSeg
                    | OrderedSourceDispositionKind::UnresolvedFailOpen => {
                        disposition.occurrence_count > 0
                    }
                    OrderedSourceDispositionKind::TerminalRejected => {
                        disposition.occurrence_count == 0
                    }
                    OrderedSourceDispositionKind::PartialPlane => false,
                })
    }

    fn verify_disposition_conservation(&self) -> Result<(), String> {
        if self.disposition_conservation_is_balanced() {
            Ok(())
        } else {
            Err(format!(
                "ordered source disposition conservation failed: records={} visited={} dispositions={} unique={} retained-occurrences={} emitted-occurrences={}",
                self.source_seg_records,
                self.source_segs_visited,
                self.dispositions.len(),
                self.dispositions
                    .iter()
                    .map(|disposition| disposition.source_seg)
                    .collect::<BTreeSet<_>>()
                    .len(),
                self.dispositions
                    .iter()
                    .map(|disposition| disposition.occurrence_count)
                    .sum::<usize>(),
                self.occurrences.len(),
            ))
        }
    }

    fn occurrence_fingerprint(&self) -> u64 {
        let mut fingerprint = 0xcbf2_9ce4_8422_2325_u64;
        for occurrence in &self.occurrences {
            for value in [
                u64::from(occurrence.source_seg),
                u64::from(occurrence.source_linedef),
                occurrence.source_interval[0].to_bits(),
                occurrence.source_interval[1].to_bits(),
            ] {
                fingerprint ^= value;
                fingerprint = fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        fingerprint
    }
}

pub(crate) fn observe_ordered_source_occurrences(
    map: &DoomMapCore,
    viewer: [i16; 2],
    heading: f64,
) -> Result<OrderedSourceOccurrenceObservation, String> {
    let root = map
        .nodes
        .len()
        .checked_sub(1)
        .ok_or_else(|| "ordered occurrence observation requires a BSP root".to_owned())?;
    let occluders = observe_doom_seg_occluders(map)
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|observation| (observation.source_seg.record_index, observation))
        .collect::<BTreeMap<_, _>>();
    let mut observation = OrderedSourceOccurrenceObservation {
        source_seg_records: map.segs.len(),
        ..OrderedSourceOccurrenceObservation::default()
    };
    let mut covered = Vec::<[f64; 2]>::new();
    let mut ancestors = Vec::new();
    visit_child(
        map,
        DoomBspChild::Node(root as u16),
        viewer,
        heading,
        &occluders,
        &mut covered,
        &mut ancestors,
        &mut observation,
    )?;
    observation.verify_disposition_conservation()?;
    Ok(observation)
}

/// Formats the finite horizontal source/view domains retained by the ordered
/// Doom protocol for a diagnostic ray. This is attribution evidence only: a
/// wall linedef may have several SEG occurrences, and no vertical authority
/// domain is inferred from a horizontal occurrence.
pub(crate) fn format_ordered_occurrence_domain_trace(
    map: &DoomMapCore,
    viewer: [i16; 2],
    heading: f64,
    eye_height: i16,
    candidate: Option<OrderedOccurrenceTraceTarget>,
    authority: Option<OrderedOccurrenceTraceTarget>,
) -> String {
    let observation = match observe_ordered_source_occurrences(map, viewer, heading) {
        Ok(observation) => observation,
        Err(error) => {
            return format!("ordered_occurrence_domains=unavailable:observation:{error}");
        }
    };
    let empty_walls = OrderedWallOccurrenceLoweringObservation::default();
    let planes = match observe_ordered_plane_occurrences(
        map,
        viewer,
        heading,
        eye_height,
        &observation,
        &empty_walls,
    ) {
        Ok(planes) => planes,
        Err(error) => {
            return format!("ordered_occurrence_domains=unavailable:plane-observation:{error}");
        }
    };
    let matching = |target: Option<OrderedOccurrenceTraceTarget>| {
        target.map_or_else(
            || String::from("not-applicable"),
            |target| match target {
                OrderedOccurrenceTraceTarget::Wall { source_linedef } => {
                    let occurrences = observation
                        .occurrences
                        .iter()
                        .filter(|occurrence| occurrence.source_linedef == source_linedef)
                        .map(|occurrence| {
                            let vertical = planes
                                .boundaries
                                .iter()
                                .find(|boundary| boundary.occurrence == *occurrence)
                                .map_or_else(
                                    || String::from("vertical=unavailable"),
                                    |boundary| {
                                        format!(
                                            "front={:?}:back={:?}:opening={:?}",
                                            boundary.front_vertical,
                                            boundary.back_vertical,
                                            boundary.opening,
                                        )
                                    },
                                );
                            format!(
                                "seg{}:source[{:.6},{:.6}]:view[{:.6},{:.6}]:{vertical}",
                                occurrence.source_seg,
                                occurrence.source_interval[0],
                                occurrence.source_interval[1],
                                occurrence.view_interval[0],
                                occurrence.view_interval[1],
                            )
                        })
                        .collect::<Vec<_>>();
                    if occurrences.is_empty() {
                        format!("linedef{source_linedef}:source-protocol-rejected")
                    } else {
                        format!("linedef{source_linedef}:[{}]", occurrences.join("|"))
                    }
                }
                OrderedOccurrenceTraceTarget::Plane {
                    source_subsector,
                    kind,
                } => {
                    let destinations = planes
                        .plane_destinations
                        .iter()
                        .filter(|destination| {
                            destination.source_subsector == source_subsector
                                && destination.kind == kind
                        })
                        .map(|destination| {
                            let source_height = planes
                                .plane_instances
                                .get(destination.plane_instance_ordinal)
                                .map(|instance| instance.source_height)
                                .map_or_else(
                                    || String::from("unresolved"),
                                    |height| height.to_string(),
                                );
                            format!(
                                "subsector{}:{kind:?}:height{}:view{:?}:triangles{}",
                                destination.source_subsector,
                                source_height,
                                destination.view_intervals,
                                destination.source_triangles,
                            )
                        })
                        .collect::<Vec<_>>();
                    if destinations.is_empty() {
                        format!("subsector{source_subsector}:{kind:?}:source-protocol-rejected")
                    } else {
                        destinations.join("|")
                    }
                }
            },
        )
    };
    format!(
        "ordered_occurrence_domains=candidate:{};authority:{}; meaning=finite-doom-source-view-and-vertical-occurrence-attribution",
        matching(candidate),
        matching(authority),
    )
}

/// Correlates the continuous source occurrences with the provider's existing
/// SEG-granular wall triangles and exercises their ordinary Tokimu mesh
/// lowering. This is deliberately observation-only: it proves that retained
/// horizontal source domains have a bounded wall destination without yet
/// replacing any declaration in the E1M1 scene.
pub(crate) fn observe_ordered_wall_occurrence_lowering(
    map: &DoomMapCore,
    extents: &[DoomTextureExtent],
    opaque_materials: &BTreeMap<String, MaterialHandle>,
    cutout_materials: &BTreeMap<String, MaterialHandle>,
    source: &OrderedSourceOccurrenceObservation,
) -> Result<OrderedWallOccurrenceLoweringObservation, String> {
    let triangles =
        lower_doom_seg_textured_wall_triangles(map, extents).map_err(|error| error.to_string())?;
    let masked_middles =
        observe_doom_two_sided_middle_textures(map).map_err(|error| error.to_string())?;
    let mut triangles_by_seg = BTreeMap::<u32, Vec<_>>::new();
    for triangle in &triangles {
        triangles_by_seg
            .entry(triangle.source_seg.record_index)
            .or_default()
            .push(triangle);
    }

    let mut observation = OrderedWallOccurrenceLoweringObservation {
        occurrences: source.occurrences.len(),
        structural_fingerprint: 0xcbf2_9ce4_8422_2325,
        ..OrderedWallOccurrenceLoweringObservation::default()
    };
    for (occurrence_ordinal, occurrence) in source.occurrences.iter().enumerate() {
        let Some(seg) = map
            .segs
            .iter()
            .find(|seg| seg.source.record_index == occurrence.source_seg)
        else {
            observation.occurrences_unresolved += 1;
            observation.retain_unresolved(format!(
                "seg={}:source-record-unavailable",
                occurrence.source_seg
            ));
            continue;
        };
        let Some(source_triangles) = triangles_by_seg.get(&occurrence.source_seg) else {
            observation.occurrences_without_wall_geometry += 1;
            observation.fingerprint_occurrence(occurrence, 0);
            continue;
        };
        let linedef_interval =
            match occurrence_linedef_interval(map, seg, occurrence.source_interval) {
                Ok(interval) => interval,
                Err(reason) => {
                    observation.occurrences_unresolved += 1;
                    for (source_triangle_ordinal, triangle) in source_triangles.iter().enumerate() {
                        let cutout = is_masked_middle(triangle, &masked_middles);
                        observation.matched_source_triangles += 1;
                        if cutout {
                            observation.matched_cutout_source_triangles += 1;
                        } else {
                            observation.matched_opaque_source_triangles += 1;
                        }
                        observation.retain_wall_disposition(OrderedWallSourceDisposition {
                            occurrence_ordinal,
                            source_seg: occurrence.source_seg,
                            source_triangle_ordinal,
                            cutout,
                            kind: OrderedSourceDispositionKind::UnresolvedFailOpen,
                            output_count: 0,
                            omission_count: 0,
                            reason: format!("linedef-domain:{reason}"),
                        });
                    }
                    continue;
                }
            };
        let mut clipped_for_occurrence = 0usize;
        let mut occurrence_unresolved = false;
        observation.matched_source_triangles += source_triangles.len();
        for (source_triangle_ordinal, triangle) in source_triangles.iter().enumerate() {
            let cutout = is_masked_middle(triangle, &masked_middles);
            if cutout {
                observation.matched_cutout_source_triangles += 1;
            } else {
                observation.matched_opaque_source_triangles += 1;
            }
            let material = if cutout {
                cutout_materials.get(&triangle.texture_name)
            } else {
                opaque_materials.get(&triangle.texture_name)
            };
            let Some(material) = material else {
                observation.retain_wall_disposition(OrderedWallSourceDisposition {
                    occurrence_ordinal,
                    source_seg: occurrence.source_seg,
                    source_triangle_ordinal,
                    cutout,
                    kind: OrderedSourceDispositionKind::UnresolvedFailOpen,
                    output_count: 0,
                    omission_count: 0,
                    reason: format!(
                        "missing-{}-material:{}",
                        if cutout { "cutout" } else { "opaque" },
                        triangle.texture_name,
                    ),
                });
                occurrence_unresolved = true;
                continue;
            };
            observation.material_resolved_source_triangles += 1;
            if cutout {
                observation.material_resolved_cutout_source_triangles += 1;
            } else {
                observation.material_resolved_opaque_source_triangles += 1;
            }
            observation.fingerprint_values([material.0]);
            let clipped = match clip_doom_seg_textured_wall_triangle_to_linedef_interval(
                map,
                triangle,
                linedef_interval,
            ) {
                Ok(clipped) => clipped,
                Err(error) => {
                    observation.retain_wall_disposition(OrderedWallSourceDisposition {
                        occurrence_ordinal,
                        source_seg: occurrence.source_seg,
                        source_triangle_ordinal,
                        cutout,
                        kind: OrderedSourceDispositionKind::UnresolvedFailOpen,
                        output_count: 0,
                        omission_count: 0,
                        reason: format!("clip-failed:{error}"),
                    });
                    occurrence_unresolved = true;
                    continue;
                }
            };
            if clipped.is_empty() {
                observation.retain_wall_disposition(OrderedWallSourceDisposition {
                    occurrence_ordinal,
                    source_seg: occurrence.source_seg,
                    source_triangle_ordinal,
                    cutout,
                    kind: OrderedSourceDispositionKind::TerminalRejected,
                    output_count: 0,
                    omission_count: 0,
                    reason: "source-interval-produced-no-fragment".to_owned(),
                });
                continue;
            }

            // A source triangle is committed atomically. No fragment becomes a
            // declaration until every retained fragment has resolved its
            // texture extent and completed ordinary mesh lowering.
            let mut pending = Vec::new();
            let mut pending_omissions = 0usize;
            let mut pending_error = None;
            for fragment in &clipped {
                let Some(extent) = extents
                    .iter()
                    .find(|extent| extent.name == fragment.texture_name)
                    .cloned()
                else {
                    pending_error = Some(format!("missing-extent:{}", fragment.texture_name));
                    break;
                };
                match lower_static_seg_wall_triangle(fragment, extent) {
                    Ok(lowered) => pending.push((fragment, lowered.wall.mesh)),
                    Err(StaticFlatLoweringError::DegenerateTriangle) => {
                        pending_omissions += 1;
                    }
                    Err(error) => {
                        pending_error = Some(format!("ordinary-lowering-failed:{error}"));
                        break;
                    }
                }
            }
            if let Some(reason) = pending_error {
                observation.retain_wall_disposition(OrderedWallSourceDisposition {
                    occurrence_ordinal,
                    source_seg: occurrence.source_seg,
                    source_triangle_ordinal,
                    cutout,
                    kind: OrderedSourceDispositionKind::UnresolvedFailOpen,
                    output_count: 0,
                    omission_count: 0,
                    reason,
                });
                occurrence_unresolved = true;
                continue;
            }

            for fragment in &clipped {
                observation.fingerprint_triangle(fragment);
            }
            observation.clipped_source_triangles += clipped.len();
            if cutout {
                observation.clipped_cutout_triangles += clipped.len();
            } else {
                observation.clipped_opaque_triangles += clipped.len();
            }
            clipped_for_occurrence += clipped.len();
            observation.degenerate_omissions += pending_omissions;
            let output_count = pending.len();
            for (fragment, mesh) in pending {
                let declaration_ordinal = observation.prepared_declarations.len();
                observation
                    .prepared_declarations
                    .push(PreparedOrderedWallDeclaration {
                        occurrence: *occurrence,
                        occurrence_ordinal,
                        source_triangle_ordinal,
                        declaration_ordinal,
                        cutout,
                        draw: StaticDrawPlanEntry {
                            mesh,
                            material: *material,
                            source_label: format!(
                                "ordered-occurrence:seg:{}:linedef:{}:fragment:{}:{}",
                                occurrence.source_seg,
                                occurrence.source_linedef,
                                declaration_ordinal,
                                fragment.texture_name,
                            ),
                            source: StaticDrawSource::Wall {
                                source_linedef: fragment.source_linedef,
                                source_sidedef: fragment.source_sidedef,
                                source_sector: fragment.source_sector,
                                role: fragment.role,
                            },
                        },
                    });
            }
            observation.lowered_wall_meshes += output_count;
            if cutout {
                observation.lowered_cutout_meshes += output_count;
            } else {
                observation.lowered_opaque_meshes += output_count;
            }
            let whole = approximately_equal(linedef_interval[0], 0.0)
                && approximately_equal(linedef_interval[1], 1.0);
            observation
                .source_dispositions
                .push(OrderedWallSourceDisposition {
                    occurrence_ordinal,
                    source_seg: occurrence.source_seg,
                    source_triangle_ordinal,
                    cutout,
                    kind: if whole {
                        OrderedSourceDispositionKind::WholeRetained
                    } else {
                        OrderedSourceDispositionKind::PartialSeg
                    },
                    output_count,
                    omission_count: pending_omissions,
                    reason: if whole {
                        "whole-source-triangle-retained"
                    } else {
                        "source-relative-partial-seg-retained"
                    }
                    .to_owned(),
                });
        }
        if occurrence_unresolved {
            observation.occurrences_unresolved += 1;
            continue;
        }
        observation.occurrences_with_wall_geometry += 1;
        observation.fingerprint_occurrence(occurrence, clipped_for_occurrence);
    }
    observation.verify_conservation()?;
    Ok(observation)
}

/// Correlates retained continuous source occurrences with Doom-owned plane
/// mark facts. It intentionally stops at source plane identity: those facts do
/// not by themselves authorize a world-space plane mesh or a legacy 320-column
/// reconstruction.
pub(crate) fn observe_ordered_plane_occurrences(
    map: &DoomMapCore,
    viewer: [i16; 2],
    heading: f64,
    eye_height: i16,
    source: &OrderedSourceOccurrenceObservation,
    walls: &OrderedWallOccurrenceLoweringObservation,
) -> Result<OrderedPlaneOccurrenceObservation, String> {
    let marks = observe_doom_seg_plane_marks(map, eye_height).map_err(|error| error.to_string())?;
    let marks_by_seg = marks
        .iter()
        .map(|mark| (mark.source_seg.record_index, mark))
        .collect::<BTreeMap<_, _>>();
    let sectors_by_source = map
        .sectors
        .iter()
        .map(|sector| (sector.source.record_index, sector))
        .collect::<BTreeMap<_, _>>();
    let subsector_by_seg = resolve_subsector_by_seg(map)?;
    let mut result = OrderedPlaneOccurrenceObservation {
        viewer,
        heading,
        occurrences: source.occurrences.len(),
        ..OrderedPlaneOccurrenceObservation::default()
    };
    let mut floor_keys = BTreeSet::new();
    let mut ceiling_keys = BTreeSet::new();
    let mut sky_keys = BTreeSet::new();

    for occurrence in &source.occurrences {
        let Some(source_subsector) = subsector_by_seg.get(&occurrence.source_seg).copied() else {
            result.retain_unresolved(format!(
                "seg={}:owning-subsector-unavailable",
                occurrence.source_seg
            ));
            continue;
        };
        let Some(mark) = marks_by_seg.get(&occurrence.source_seg) else {
            result.retain_unresolved(format!(
                "seg={}:plane-mark-unavailable",
                occurrence.source_seg
            ));
            continue;
        };
        if mark.source_linedef.record_index != occurrence.source_linedef {
            result.retain_unresolved(format!(
                "seg={}:plane-mark-linedef-mismatch:{}!={}",
                occurrence.source_seg, mark.source_linedef.record_index, occurrence.source_linedef,
            ));
            continue;
        }
        let Some(sector) = sectors_by_source.get(&mark.front_sector.record_index) else {
            result.retain_unresolved(format!(
                "seg={}:front-sector-unavailable:{}",
                occurrence.source_seg, mark.front_sector.record_index,
            ));
            continue;
        };
        let back_sector = match mark.back_sector {
            Some(source) => {
                let Some(sector) = sectors_by_source.get(&source.record_index) else {
                    result.retain_unresolved(format!(
                        "seg={}:back-sector-unavailable:{}",
                        occurrence.source_seg, source.record_index,
                    ));
                    continue;
                };
                Some(*sector)
            }
            None => None,
        };
        let wall_consumers = walls
            .prepared_declarations
            .iter()
            .filter(|declaration| declaration.occurrence == *occurrence)
            .count();
        let plane_consumers = usize::from(mark.floor_marked) + usize::from(mark.ceiling_marked);
        let boundary = match prepare_occurrence_boundary(
            *occurrence,
            mark,
            sector,
            back_sector,
            wall_consumers,
            plane_consumers,
        ) {
            Ok(boundary) => boundary,
            Err(reason) => {
                result.retain_unresolved(reason);
                continue;
            }
        };
        match boundary.back_vertical {
            None => result.one_sided_boundaries += 1,
            Some(_) if boundary.opening.is_some() => result.open_two_sided_boundaries += 1,
            Some(_) => result.closed_two_sided_boundaries += 1,
        }
        result.wall_consumer_references += wall_consumers;
        result.plane_consumer_references += plane_consumers;
        result.boundaries.push(boundary);
        let mut marked = false;
        if mark.floor_marked {
            marked = true;
            result.floor_associations += 1;
            floor_keys.insert((
                sector.source.record_index,
                sector.floor_height,
                sector.floor_texture.clone(),
                sector.light_level,
            ));
            result.associations.push(OrderedPlaneOccurrence {
                occurrence: *occurrence,
                source_subsector,
                kind: OrderedPlaneKind::Floor,
                source_sector: sector.source.record_index,
                source_height: sector.floor_height,
                texture: sector.floor_texture.clone(),
                light_level: sector.light_level,
                sky: false,
            });
        }
        if mark.ceiling_marked {
            marked = true;
            let sky = sector.ceiling_texture == "F_SKY1";
            result.ceiling_associations += 1;
            result.sky_ceiling_associations += usize::from(sky);
            let key = (
                sector.source.record_index,
                sector.ceiling_height,
                sector.ceiling_texture.clone(),
                sector.light_level,
            );
            ceiling_keys.insert(key.clone());
            if sky {
                sky_keys.insert(key);
            }
            result.associations.push(OrderedPlaneOccurrence {
                occurrence: *occurrence,
                source_subsector,
                kind: OrderedPlaneKind::Ceiling,
                source_sector: sector.source.record_index,
                source_height: sector.ceiling_height,
                texture: sector.ceiling_texture.clone(),
                light_level: sector.light_level,
                sky,
            });
        }
        result.paired_sky_adjustments += usize::from(mark.paired_sky_ceiling_adjustment);
        if marked {
            result.occurrences_with_marked_planes += 1;
        } else {
            result.occurrences_without_marked_planes += 1;
        }
    }

    result.distinct_floor_planes = floor_keys.len();
    result.distinct_ceiling_planes = ceiling_keys.len();
    result.distinct_sky_ceiling_planes = sky_keys.len();
    result.plane_instances = group_ordered_plane_instances(&result.associations);
    result.plane_instance_occurrence_references = result
        .plane_instances
        .iter()
        .map(|instance| instance.occurrence_references)
        .sum();
    result.plane_instance_subsector_references = result
        .plane_instances
        .iter()
        .map(|instance| instance.source_subsectors.len())
        .sum();
    correlate_plane_geometry_destinations(map, &mut result)?;
    Ok(result)
}

fn correlate_plane_geometry_destinations(
    map: &DoomMapCore,
    observation: &mut OrderedPlaneOccurrenceObservation,
) -> Result<(), String> {
    type DestinationKey = (u32, u32, OrderedPlaneKind);
    let paths = resolve_doom_subsector_bsp_paths(map).map_err(|error| error.to_string())?;
    let surfaces = lower_doom_subsector_surfaces(map, &paths).map_err(|error| error.to_string())?;
    let mut surfaces_by_destination = BTreeMap::<DestinationKey, Vec<_>>::new();
    for surface in &surfaces {
        let kind = match surface.plane {
            DoomSurfacePlane::Floor => OrderedPlaneKind::Floor,
            DoomSurfacePlane::Ceiling => OrderedPlaneKind::Ceiling,
        };
        surfaces_by_destination
            .entry((
                surface.source_subsector.record_index,
                surface.source_sector.record_index,
                kind,
            ))
            .or_default()
            .push(surface);
    }

    for (plane_instance_ordinal, instance) in observation.plane_instances.iter().enumerate() {
        for &source_subsector in &instance.source_subsectors {
            let key = (source_subsector, instance.source_sector, instance.kind);
            let Some(destination_surfaces) = surfaces_by_destination.get(&key) else {
                observation.plane_destination_unresolved_fail_open += 1;
                if observation.plane_destination_unresolved_samples.len() < 12 {
                    observation.plane_destination_unresolved_samples.push(format!(
                        "instance={plane_instance_ordinal}:sector={}:subsector={source_subsector}:kind={:?}:source-region-unavailable",
                        instance.source_sector, instance.kind,
                    ));
                }
                continue;
            };
            let expected_height = f64::from(instance.source_height);
            let values_match = destination_surfaces.iter().all(|surface| {
                surface.texture_name == instance.texture
                    && surface
                        .positions
                        .iter()
                        .all(|position| position[1] == expected_height)
            });
            if !values_match {
                observation.plane_destination_unresolved_fail_open += 1;
                if observation.plane_destination_unresolved_samples.len() < 12 {
                    observation.plane_destination_unresolved_samples.push(format!(
                        "instance={plane_instance_ordinal}:sector={}:subsector={source_subsector}:kind={:?}:source-values-mismatch",
                        instance.source_sector, instance.kind,
                    ));
                }
                continue;
            }
            let occurrence_view_intervals = observation
                .associations
                .iter()
                .filter(|association| {
                    association.source_subsector == source_subsector
                        && association.kind == instance.kind
                        && association.source_sector == instance.source_sector
                        && association.source_height == instance.source_height
                        && association.texture == instance.texture
                        && association.light_level == instance.light_level
                        && association.sky == instance.sky
                })
                .map(|association| association.occurrence.view_interval)
                .collect::<Vec<_>>();
            let occurrence_references = occurrence_view_intervals.len();
            let view_intervals = merge_intervals(occurrence_view_intervals);
            if view_intervals.is_empty() {
                observation.plane_destination_unresolved_fail_open += 1;
                if observation.plane_destination_unresolved_samples.len() < 12 {
                    observation.plane_destination_unresolved_samples.push(format!(
                        "instance={plane_instance_ordinal}:sector={}:subsector={source_subsector}:kind={:?}:occurrence-domain-unavailable",
                        instance.source_sector, instance.kind,
                    ));
                }
                continue;
            }
            observation.plane_destination_references += 1;
            observation.plane_destination_source_triangles += destination_surfaces.len();
            observation
                .plane_destinations
                .push(OrderedPreparedPlaneDestination {
                    plane_instance_ordinal,
                    kind: instance.kind,
                    source_sector: instance.source_sector,
                    source_subsector,
                    source_triangles: destination_surfaces.len(),
                    occurrence_references,
                    view_intervals,
                });
        }
    }
    Ok(())
}

/// Reconstructs the exact bounded plane cells retained by Doom's ordered
/// vertical-coverage protocol. These are source-private presentation support,
/// not renderer pixels or a generic clipping primitive.
fn observe_ordered_plane_domain_cells(
    map: &DoomMapCore,
    viewer: [i16; 2],
    heading: f64,
    eye_height: i16,
    extents: &[DoomTextureExtent],
) -> Result<Vec<OrderedPlaneDomainCell>, String> {
    const MINIMUM_TANGENT: f64 = 1.0e-9;
    let traversal = observe_doom_classic_bsp(map, viewer, heading, &BTreeSet::new())
        .map_err(|error| error.to_string())?;
    let wall_triangles =
        lower_doom_seg_textured_wall_triangles(map, extents).map_err(|error| error.to_string())?;
    let plane_marks =
        observe_doom_seg_plane_marks(map, eye_height).map_err(|error| error.to_string())?;
    let vertical = observe_doom_classic_vertical_clip_state(
        map,
        &wall_triangles,
        &plane_marks,
        &traversal,
        viewer,
        heading,
        f64::from(eye_height),
    );
    let mut subsector_by_seg = BTreeMap::new();
    for subsector in &map.subsectors {
        let start = usize::from(subsector.first_seg);
        let end = start + usize::from(subsector.seg_count);
        let segs = map.segs.get(start..end).ok_or_else(|| {
            format!(
                "subsector {} has invalid SEG range {start}..{end}",
                subsector.source.record_index,
            )
        })?;
        for seg in segs {
            subsector_by_seg.insert(seg.source.record_index, subsector.source.record_index);
        }
    }

    let half_vertical_fov =
        (std::f64::consts::FRAC_PI_4.tan() / (CLASSIC_COLUMNS as f64 / CLASSIC_ROWS as f64)).atan();
    let forward = [heading.cos(), heading.sin()];
    let right = [-forward[1], forward[0]];
    let viewer_f64 = [f64::from(viewer[0]), f64::from(viewer[1])];
    let horizontal_angle = |column_boundary: f64| {
        let normalized = -1.0 + (column_boundary / CLASSIC_COLUMNS as f64) * 2.0;
        (normalized * std::f64::consts::FRAC_PI_4.tan()).atan()
    };
    let vertical_angle = |row_boundary: f64| {
        let normalized = 1.0 - (row_boundary / CLASSIC_ROWS as f64) * 2.0;
        (normalized * half_vertical_fov.tan()).atan()
    };
    let mut cells = Vec::new();
    for (key, instances) in &vertical.plane_spans.keys {
        if key.kind == DoomSegClassicPlaneKind::Ceiling && key.texture == "F_SKY1" {
            continue;
        }
        let kind = match key.kind {
            DoomSegClassicPlaneKind::Floor => OrderedPlaneKind::Floor,
            DoomSegClassicPlaneKind::Ceiling => OrderedPlaneKind::Ceiling,
        };
        for instance in instances {
            for (column, rows) in instance.columns.iter().enumerate() {
                let Some([top, bottom]) = rows else {
                    continue;
                };
                let Some([source_sector, source_seg]) = instance.column_sources[column] else {
                    continue;
                };
                let Some(&source_subsector) = subsector_by_seg.get(&source_seg) else {
                    continue;
                };
                let plane_delta = f64::from(key.height - eye_height);
                let boundaries = [
                    (column as f64, *top as f64),
                    ((column + 1) as f64, *top as f64),
                    ((column + 1) as f64, (*bottom + 1) as f64),
                    (column as f64, (*bottom + 1) as f64),
                ];
                let mut corners = [[0.0; 2]; 4];
                let mut valid = true;
                for (corner, (column_boundary, row_boundary)) in corners.iter_mut().zip(boundaries)
                {
                    let tangent = vertical_angle(row_boundary).tan();
                    if tangent.abs() <= MINIMUM_TANGENT {
                        valid = false;
                        break;
                    }
                    let forward_distance = plane_delta / tangent;
                    if !forward_distance.is_finite() || forward_distance <= 0.0 {
                        valid = false;
                        break;
                    }
                    let angle = horizontal_angle(column_boundary);
                    let radial_distance = forward_distance / angle.cos();
                    let ray = [
                        forward[0] * angle.cos() + right[0] * angle.sin(),
                        forward[1] * angle.cos() + right[1] * angle.sin(),
                    ];
                    *corner = [
                        viewer_f64[0] + ray[0] * radial_distance,
                        viewer_f64[1] + ray[1] * radial_distance,
                    ];
                }
                if valid {
                    cells.push(OrderedPlaneDomainCell {
                        kind,
                        source_sector,
                        source_subsector,
                        source_height: key.height,
                        texture: key.texture.clone(),
                        light_level: key.light,
                        source_seg,
                        source_corners: corners,
                    });
                }
            }
        }
    }
    Ok(cells)
}

/// Lowers the exact source-region destinations retained by the ordered plane
/// observation. This is a conservative region candidate, not a claim that an
/// entire subsector is visible through every contributing occurrence.
///
/// Sky destinations deliberately do not become world-space depth geometry.
/// They resolve to the existing Doom sky background presentation and remain
/// counted separately so later source-owned coverage preparation can refine
/// their visible intervals without changing renderer vocabulary.
pub(crate) fn lower_ordered_plane_destinations(
    map: &DoomMapCore,
    opaque_uploads: &[StaticTextureUpload],
    source: &OrderedPlaneOccurrenceObservation,
    plane_domain_cells: &[OrderedPlaneDomainCell],
) -> Result<OrderedPlaneLoweringObservation, String> {
    type DestinationKey = (u32, u32, OrderedPlaneKind);
    let paths = resolve_doom_subsector_bsp_paths(map).map_err(|error| error.to_string())?;
    let surfaces = lower_doom_subsector_surfaces(map, &paths).map_err(|error| error.to_string())?;
    let mut surfaces_by_destination = BTreeMap::<DestinationKey, Vec<_>>::new();
    for surface in &surfaces {
        let kind = match surface.plane {
            DoomSurfacePlane::Floor => OrderedPlaneKind::Floor,
            DoomSurfacePlane::Ceiling => OrderedPlaneKind::Ceiling,
        };
        surfaces_by_destination
            .entry((
                surface.source_subsector.record_index,
                surface.source_sector.record_index,
                kind,
            ))
            .or_default()
            .push(surface);
    }
    let flat_materials = opaque_uploads
        .iter()
        .filter(|upload| upload.source_kind == StaticTextureSourceKind::Flat)
        .map(|upload| (upload.source_name.as_str(), upload.material))
        .collect::<BTreeMap<_, _>>();
    let mut result = OrderedPlaneLoweringObservation {
        destination_references: source.plane_destinations.len(),
        destination_source_triangles: source.plane_destination_source_triangles,
        ..OrderedPlaneLoweringObservation::default()
    };

    for destination in &source.plane_destinations {
        let Some(instance) = source
            .plane_instances
            .get(destination.plane_instance_ordinal)
        else {
            retain_unresolved_plane_source_dispositions(
                &mut result,
                destination,
                "destination-instance-unavailable",
            );
            result.retain_unresolved(format!(
                "instance={}:destination-instance-unavailable",
                destination.plane_instance_ordinal
            ));
            continue;
        };
        if instance.sky {
            result.sky_background_destination_references += 1;
            result.sky_background_source_triangles += destination.source_triangles;
            for source_triangle_ordinal in 0..destination.source_triangles {
                result
                    .source_dispositions
                    .push(OrderedPlaneSourceDisposition {
                        plane_instance_ordinal: destination.plane_instance_ordinal,
                        source_subsector: destination.source_subsector,
                        source_triangle_ordinal,
                        kind: OrderedSourceDispositionKind::WholeRetained,
                        output_count: 1,
                        reason: "retained-as-doom-sky-background".to_owned(),
                    });
            }
            continue;
        }
        let key = (
            destination.source_subsector,
            destination.source_sector,
            destination.kind,
        );
        let Some(destination_surfaces) = surfaces_by_destination.get(&key) else {
            retain_unresolved_plane_source_dispositions(
                &mut result,
                destination,
                "source-region-unavailable",
            );
            result.retain_unresolved(format!(
                "instance={}:sector={}:subsector={}:kind={:?}:source-region-unavailable",
                destination.plane_instance_ordinal,
                destination.source_sector,
                destination.source_subsector,
                destination.kind,
            ));
            continue;
        };
        let Some(&material) = flat_materials.get(instance.texture.as_str()) else {
            retain_unresolved_plane_source_dispositions(
                &mut result,
                destination,
                "material-unavailable",
            );
            result.retain_unresolved(format!(
                "instance={}:sector={}:subsector={}:flat={}:material-unavailable",
                destination.plane_instance_ordinal,
                destination.source_sector,
                destination.source_subsector,
                instance.texture,
            ));
            continue;
        };
        result.ordinary_destination_references += 1;
        result.ordinary_source_triangles += destination_surfaces.len();
        for (source_triangle_ordinal, surface) in destination_surfaces.iter().enumerate() {
            let interval_clipped_surfaces = destination
                .view_intervals
                .iter()
                .flat_map(|interval| {
                    clip_plane_triangle_to_view_interval(
                        surface,
                        source.viewer,
                        source.heading,
                        *interval,
                    )
                })
                .collect::<Vec<_>>();
            if interval_clipped_surfaces.is_empty() {
                result.ordinary_source_triangles_fully_rejected += 1;
                result
                    .source_dispositions
                    .push(OrderedPlaneSourceDisposition {
                        plane_instance_ordinal: destination.plane_instance_ordinal,
                        source_subsector: destination.source_subsector,
                        source_triangle_ordinal,
                        kind: OrderedSourceDispositionKind::TerminalRejected,
                        output_count: 0,
                        reason: "outside-authorized-plane-view-intervals".to_owned(),
                    });
                continue;
            }
            result.ordinary_source_triangles_with_survivors += 1;
            let disposition_kind =
                if plane_triangle_is_unchanged(surface, &interval_clipped_surfaces) {
                    OrderedSourceDispositionKind::WholeRetained
                } else {
                    OrderedSourceDispositionKind::PartialPlane
                };
            let clipped_surfaces = if disposition_kind == OrderedSourceDispositionKind::PartialPlane
            {
                let matching_cells = plane_domain_cells
                    .iter()
                    .filter(|cell| plane_domain_cell_matches(cell, destination, instance))
                    .collect::<Vec<_>>();
                result.partial_plane_domain_cells += matching_cells.len();
                let fragments = matching_cells
                    .into_iter()
                    .flat_map(|cell| clip_plane_triangle_to_domain_cell(surface, cell))
                    .collect::<Vec<_>>();
                result.partial_plane_domain_fragments += fragments.len();
                fragments
            } else {
                interval_clipped_surfaces
            };
            if clipped_surfaces.is_empty() {
                result.ordinary_source_triangles_with_survivors -= 1;
                result.ordinary_source_triangles_fully_rejected += 1;
                result
                    .source_dispositions
                    .push(OrderedPlaneSourceDisposition {
                        plane_instance_ordinal: destination.plane_instance_ordinal,
                        source_subsector: destination.source_subsector,
                        source_triangle_ordinal,
                        kind: OrderedSourceDispositionKind::TerminalRejected,
                        output_count: 0,
                        reason: "outside-authoritative-plane-domain-cells".to_owned(),
                    });
                continue;
            }
            result.clipped_plane_triangles += clipped_surfaces.len();
            let declarations_before = result.prepared_declarations.len();
            let mut combined_draw = None::<StaticDrawPlanEntry>;
            for clipped_surface in &clipped_surfaces {
                match lower_static_flat_triangle(clipped_surface, FlatExtent::E1M1) {
                    Ok(flat) => {
                        result.lowered_plane_triangles += 1;
                        if let Some(draw) = combined_draw.as_mut() {
                            draw.mesh.positions.extend(flat.mesh.positions);
                            draw.mesh.normals.extend(flat.mesh.normals);
                            draw.mesh
                                .texture_coordinates
                                .extend(flat.mesh.texture_coordinates);
                        } else {
                            combined_draw = Some(StaticDrawPlanEntry {
                                source_label: String::new(),
                                source: StaticDrawSource::Flat {
                                    source_subsector: flat.source.subsector,
                                    source_sector: flat.source.sector,
                                    plane: flat.source.plane,
                                },
                                mesh: flat.mesh,
                                material,
                            });
                        }
                    }
                    Err(StaticFlatLoweringError::DegenerateTriangle) => {
                        result.degenerate_omissions += 1;
                    }
                    Err(error) => {
                        return Err(format!(
                            "instance={}:sector={}:subsector={}:plane lowering failed: {error}",
                            destination.plane_instance_ordinal,
                            destination.source_sector,
                            destination.source_subsector,
                        ));
                    }
                }
            }
            if let Some(mut draw) = combined_draw {
                let declaration_ordinal = result.prepared_declarations.len();
                draw.source_label = format!(
                    "ordered-plane:{:?}:{}:{}:{}:{}",
                    destination.kind,
                    destination.source_sector,
                    destination.source_subsector,
                    instance.texture,
                    declaration_ordinal,
                );
                result
                    .prepared_declarations
                    .push(PreparedOrderedPlaneDeclaration {
                        plane_instance_ordinal: destination.plane_instance_ordinal,
                        source_subsector: destination.source_subsector,
                        source_triangle_ordinal,
                        draw,
                    });
                result.lowered_plane_meshes += 1;
            }
            let output_count = result.prepared_declarations.len() - declarations_before;
            result
                .source_dispositions
                .push(OrderedPlaneSourceDisposition {
                    plane_instance_ordinal: destination.plane_instance_ordinal,
                    source_subsector: destination.source_subsector,
                    source_triangle_ordinal,
                    kind: if output_count == 0 {
                        OrderedSourceDispositionKind::TerminalRejected
                    } else {
                        disposition_kind
                    },
                    output_count,
                    reason: if output_count == 0 {
                        "all-authorized-plane-fragments-degenerate"
                    } else if disposition_kind == OrderedSourceDispositionKind::WholeRetained {
                        "whole-plane-triangle-retained"
                    } else {
                        "partial-plane-triangle-retained"
                    }
                    .to_owned(),
                });
        }
    }
    result.verify_plane_disposition_conservation()?;
    Ok(result)
}

fn retain_unresolved_plane_source_dispositions(
    result: &mut OrderedPlaneLoweringObservation,
    destination: &OrderedPreparedPlaneDestination,
    reason: &str,
) {
    for source_triangle_ordinal in 0..destination.source_triangles {
        result
            .source_dispositions
            .push(OrderedPlaneSourceDisposition {
                plane_instance_ordinal: destination.plane_instance_ordinal,
                source_subsector: destination.source_subsector,
                source_triangle_ordinal,
                kind: OrderedSourceDispositionKind::UnresolvedFailOpen,
                output_count: 0,
                reason: reason.to_owned(),
            });
    }
}

fn plane_triangle_is_unchanged(
    source: &DoomSurfaceTriangle,
    clipped: &[DoomSurfaceTriangle],
) -> bool {
    clipped.len() == 1
        && source
            .positions
            .iter()
            .zip(clipped[0].positions.iter())
            .all(|(source, clipped)| {
                source
                    .iter()
                    .zip(clipped.iter())
                    .all(|(source, clipped)| approximately_equal(*source, *clipped))
            })
}

fn merge_intervals(mut intervals: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    intervals.retain(|interval| {
        interval.iter().all(|value| value.is_finite())
            && interval[0] + INTERVAL_EPSILON < interval[1]
    });
    intervals.sort_by(|left, right| left[0].total_cmp(&right[0]));
    let mut merged = Vec::<[f64; 2]>::new();
    for interval in intervals {
        if let Some(last) = merged.last_mut() {
            if interval[0] <= last[1] + INTERVAL_EPSILON {
                last[1] = last[1].max(interval[1]);
                continue;
            }
        }
        merged.push(interval);
    }
    merged
}

fn clip_plane_triangle_to_view_interval(
    triangle: &DoomSurfaceTriangle,
    viewer: [i16; 2],
    heading: f64,
    interval: [f64; 2],
) -> Vec<DoomSurfaceTriangle> {
    if !interval.iter().all(|value| value.is_finite())
        || interval[0] + INTERVAL_EPSILON >= interval[1]
    {
        return Vec::new();
    }
    let forward = [heading.cos(), heading.sin()];
    let right = [-forward[1], forward[0]];
    let view_coordinates = |position: [f64; 3]| {
        let relative = [
            position[0] - f64::from(viewer[0]),
            position[2] - f64::from(viewer[1]),
        ];
        [
            relative[0] * forward[0] + relative[1] * forward[1],
            relative[0] * right[0] + relative[1] * right[1],
        ]
    };
    let left = interval[0] * HALF_FOV_TANGENT;
    let right_bound = interval[1] * HALF_FOV_TANGENT;
    let mut polygon = clip_polygon_to_half_space(&triangle.positions, |position| {
        view_coordinates(position)[0] - DEPTH_EPSILON
    });
    polygon = clip_polygon_to_half_space(&polygon, |position| {
        let view = view_coordinates(position);
        view[1] - left * view[0]
    });
    polygon = clip_polygon_to_half_space(&polygon, |position| {
        let view = view_coordinates(position);
        right_bound * view[0] - view[1]
    });
    if polygon.len() < 3 {
        return Vec::new();
    }
    (1..polygon.len() - 1)
        .map(|index| DoomSurfaceTriangle {
            source_subsector: triangle.source_subsector,
            source_sector: triangle.source_sector,
            plane: triangle.plane,
            texture_name: triangle.texture_name.clone(),
            positions: [polygon[0], polygon[index], polygon[index + 1]],
        })
        .collect()
}

fn plane_domain_cell_matches(
    cell: &OrderedPlaneDomainCell,
    destination: &OrderedPreparedPlaneDestination,
    instance: &OrderedPreparedPlaneInstance,
) -> bool {
    cell.kind == destination.kind
        && cell.source_sector == destination.source_sector
        && cell.source_subsector == destination.source_subsector
        && cell.source_height == instance.source_height
        && cell.texture == instance.texture
        && cell.light_level == instance.light_level
        && cell.source_seg != u32::MAX
}

/// Intersects one inferred source-region triangle with one exact Doom plane
/// coverage cell. The result remains ordinary finite plane geometry while the
/// cell, rather than a SEG parameter or global plane mesh, owns partial
/// presentation support.
fn clip_plane_triangle_to_domain_cell(
    triangle: &DoomSurfaceTriangle,
    cell: &OrderedPlaneDomainCell,
) -> Vec<DoomSurfaceTriangle> {
    let twice_area = cell
        .source_corners
        .iter()
        .zip(cell.source_corners.iter().cycle().skip(1))
        .take(cell.source_corners.len())
        .map(|(left, right)| left[0] * right[1] - right[0] * left[1])
        .sum::<f64>();
    if !twice_area.is_finite() || twice_area.abs() <= INTERVAL_EPSILON {
        return Vec::new();
    }
    let orientation = twice_area.signum();
    let mut polygon = triangle.positions.to_vec();
    for index in 0..cell.source_corners.len() {
        let start = cell.source_corners[index];
        let end = cell.source_corners[(index + 1) % cell.source_corners.len()];
        polygon = clip_polygon_to_half_space(&polygon, |position| {
            orientation
                * ((end[0] - start[0]) * (position[2] - start[1])
                    - (end[1] - start[1]) * (position[0] - start[0]))
        });
        if polygon.len() < 3 {
            return Vec::new();
        }
    }
    (1..polygon.len() - 1)
        .map(|index| DoomSurfaceTriangle {
            source_subsector: triangle.source_subsector,
            source_sector: triangle.source_sector,
            plane: triangle.plane,
            texture_name: triangle.texture_name.clone(),
            positions: [polygon[0], polygon[index], polygon[index + 1]],
        })
        .collect()
}

fn clip_polygon_to_half_space(
    polygon: &[[f64; 3]],
    evaluate: impl Fn([f64; 3]) -> f64,
) -> Vec<[f64; 3]> {
    let Some(&last) = polygon.last() else {
        return Vec::new();
    };
    let mut result = Vec::with_capacity(polygon.len() + 1);
    let mut previous = last;
    let mut previous_value = evaluate(previous);
    for &current in polygon {
        let current_value = evaluate(current);
        let previous_inside = previous_value >= -INTERVAL_EPSILON;
        let current_inside = current_value >= -INTERVAL_EPSILON;
        if previous_inside != current_inside {
            let denominator = previous_value - current_value;
            if denominator.abs() > INTERVAL_EPSILON {
                let t = (previous_value / denominator).clamp(0.0, 1.0);
                result.push([
                    previous[0] + (current[0] - previous[0]) * t,
                    previous[1] + (current[1] - previous[1]) * t,
                    previous[2] + (current[2] - previous[2]) * t,
                ]);
            }
        }
        if current_inside {
            result.push(current);
        }
        previous = current;
        previous_value = current_value;
    }
    result
}

fn resolve_subsector_by_seg(map: &DoomMapCore) -> Result<BTreeMap<u32, u32>, String> {
    let mut result = BTreeMap::new();
    for subsector in &map.subsectors {
        let start = usize::from(subsector.first_seg);
        let end = start + usize::from(subsector.seg_count);
        let segs = map.segs.get(start..end).ok_or_else(|| {
            format!(
                "subsector={} has invalid SEG range {}..{}",
                subsector.source.record_index, start, end
            )
        })?;
        for seg in segs {
            if let Some(previous) =
                result.insert(seg.source.record_index, subsector.source.record_index)
            {
                if previous != subsector.source.record_index {
                    return Err(format!(
                        "seg={} belongs to multiple subsectors: {} and {}",
                        seg.source.record_index, previous, subsector.source.record_index
                    ));
                }
            }
        }
    }
    Ok(result)
}

fn group_ordered_plane_instances(
    associations: &[OrderedPlaneOccurrence],
) -> Vec<OrderedPreparedPlaneInstance> {
    type PlaneKey = (OrderedPlaneKind, u32, i16, String, i16, bool);
    let mut grouped = BTreeMap::<PlaneKey, (usize, BTreeSet<u32>)>::new();
    for association in associations {
        let entry = grouped
            .entry((
                association.kind,
                association.source_sector,
                association.source_height,
                association.texture.clone(),
                association.light_level,
                association.sky,
            ))
            .or_default();
        entry.0 += 1;
        entry.1.insert(association.source_subsector);
    }
    grouped
        .into_iter()
        .map(
            |(
                (kind, source_sector, source_height, texture, light_level, sky),
                (occurrence_references, source_subsectors),
            )| OrderedPreparedPlaneInstance {
                kind,
                source_sector,
                source_height,
                texture,
                light_level,
                sky,
                occurrence_references,
                source_subsectors: source_subsectors.into_iter().collect(),
            },
        )
        .collect()
}

fn prepare_occurrence_boundary(
    occurrence: OrderedSourceOccurrence,
    mark: &DoomSegPlaneMarkObservation,
    front: &DoomSector,
    back: Option<&DoomSector>,
    wall_consumers: usize,
    plane_consumers: usize,
) -> Result<OrderedPreparedBoundary, String> {
    if front.floor_height > front.ceiling_height {
        return Err(format!(
            "seg={}:front-sector-reversed-heights:{}>{}",
            occurrence.source_seg, front.floor_height, front.ceiling_height,
        ));
    }
    if let Some(back) = back {
        if back.floor_height > back.ceiling_height {
            return Err(format!(
                "seg={}:back-sector-reversed-heights:{}>{}",
                occurrence.source_seg, back.floor_height, back.ceiling_height,
            ));
        }
    }
    let back_vertical = back.map(|sector| [sector.floor_height, sector.ceiling_height]);
    let opening = back.and_then(|back| {
        let lower = front.floor_height.max(back.floor_height);
        let upper = front.ceiling_height.min(back.ceiling_height);
        (lower < upper).then_some([lower, upper])
    });
    Ok(OrderedPreparedBoundary {
        occurrence,
        front_sector: front.source.record_index,
        back_sector: back.map(|sector| sector.source.record_index),
        front_vertical: [front.floor_height, front.ceiling_height],
        back_vertical,
        opening,
        paired_sky_ceiling_adjustment: mark.paired_sky_ceiling_adjustment,
        wall_consumers,
        plane_consumers,
    })
}

fn is_masked_middle(
    triangle: &DoomSegTexturedWallTriangle,
    masked_middles: &[DoomMiddleTextureObservation],
) -> bool {
    triangle.role == DoomWallTextureRole::Middle
        && masked_middles.iter().any(|middle| {
            middle.source_linedef == triangle.source_linedef
                && middle.source_sidedef == triangle.source_sidedef
                && middle.side == triangle.side
                && middle.texture_name == triangle.texture_name
        })
}

impl OrderedWallOccurrenceLoweringObservation {
    fn retain_unresolved(&mut self, reason: String) {
        self.unresolved_fail_open += 1;
        if self.unresolved_samples.len() < 12 {
            self.unresolved_samples.push(reason);
        }
    }

    fn retain_wall_disposition(&mut self, disposition: OrderedWallSourceDisposition) {
        debug_assert_eq!(
            disposition.kind,
            OrderedSourceDispositionKind::UnresolvedFailOpen
        );
        self.retain_unresolved(format!(
            "seg={}:triangle={}:{}",
            disposition.source_seg, disposition.source_triangle_ordinal, disposition.reason,
        ));
        self.source_dispositions.push(disposition);
    }

    fn fingerprint_occurrence(&mut self, occurrence: &OrderedSourceOccurrence, pieces: usize) {
        self.fingerprint_values([
            u64::from(occurrence.source_seg),
            u64::from(occurrence.source_linedef),
            occurrence.source_interval[0].to_bits(),
            occurrence.source_interval[1].to_bits(),
            pieces as u64,
        ]);
    }

    fn fingerprint_triangle(
        &mut self,
        triangle: &doom_geometry_provider::DoomSegTexturedWallTriangle,
    ) {
        self.fingerprint_values(
            triangle
                .positions
                .iter()
                .flatten()
                .chain(triangle.texture_coordinates.iter().flatten())
                .map(|value| value.to_bits()),
        );
    }

    fn fingerprint_values(&mut self, values: impl IntoIterator<Item = u64>) {
        for value in values {
            self.structural_fingerprint ^= value;
            self.structural_fingerprint = self
                .structural_fingerprint
                .wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
}

fn occurrence_linedef_interval(
    map: &DoomMapCore,
    seg: &DoomSeg,
    source_interval: [f64; 2],
) -> Result<[f64; 2], String> {
    if !source_interval.iter().all(|value| value.is_finite())
        || source_interval[0] < 0.0
        || source_interval[1] > 1.0
        || source_interval[0] > source_interval[1]
    {
        return Err("invalid-source-interval".to_owned());
    }
    let linedef = map
        .linedefs
        .get(usize::from(seg.linedef))
        .ok_or_else(|| "linedef-unavailable".to_owned())?;
    let line_start = map
        .vertices
        .get(usize::from(linedef.start_vertex))
        .ok_or_else(|| "linedef-start-unavailable".to_owned())?;
    let line_end = map
        .vertices
        .get(usize::from(linedef.end_vertex))
        .ok_or_else(|| "linedef-end-unavailable".to_owned())?;
    let seg_start = map
        .vertices
        .get(usize::from(seg.start_vertex))
        .ok_or_else(|| "seg-start-unavailable".to_owned())?;
    let seg_end = map
        .vertices
        .get(usize::from(seg.end_vertex))
        .ok_or_else(|| "seg-end-unavailable".to_owned())?;
    let delta = [
        f64::from(line_end.x) - f64::from(line_start.x),
        f64::from(line_end.y) - f64::from(line_start.y),
    ];
    let length_squared = delta[0].mul_add(delta[0], delta[1] * delta[1]);
    if length_squared <= INTERVAL_EPSILON {
        return Err("degenerate-linedef".to_owned());
    }
    let progression = |point: [i16; 2]| {
        ((f64::from(point[0]) - f64::from(line_start.x)) * delta[0]
            + (f64::from(point[1]) - f64::from(line_start.y)) * delta[1])
            / length_squared
    };
    let start = progression([seg_start.x, seg_start.y]);
    let end = progression([seg_end.x, seg_end.y]);
    let mapped = [
        start + (end - start) * source_interval[0],
        start + (end - start) * source_interval[1],
    ];
    Ok([
        mapped[0].min(mapped[1]).clamp(0.0, 1.0),
        mapped[0].max(mapped[1]).clamp(0.0, 1.0),
    ])
}

#[allow(clippy::too_many_arguments)]
fn visit_child(
    map: &DoomMapCore,
    child: DoomBspChild,
    viewer: [i16; 2],
    heading: f64,
    occluders: &BTreeMap<u32, DoomSegOccluderObservation>,
    covered: &mut Vec<[f64; 2]>,
    ancestors: &mut Vec<u16>,
    observation: &mut OrderedSourceOccurrenceObservation,
) -> Result<(), String> {
    match child {
        DoomBspChild::Subsector(index) => {
            let subsector = map
                .subsectors
                .get(usize::from(index))
                .ok_or_else(|| format!("BSP subsector {index} is out of bounds"))?;
            let first = usize::from(subsector.first_seg);
            let end = first + usize::from(subsector.seg_count);
            let segs = map
                .segs
                .get(first..end)
                .ok_or_else(|| format!("subsector {index} SEG range is out of bounds"))?;
            for seg in segs {
                observe_seg(map, seg, viewer, heading, occluders, covered, observation)?;
            }
            Ok(())
        }
        DoomBspChild::Node(index) => {
            if ancestors.contains(&index) {
                return Err(format!("BSP cycle at node {index}"));
            }
            let node = map
                .nodes
                .get(usize::from(index))
                .ok_or_else(|| format!("BSP node {index} is out of bounds"))?;
            ancestors.push(index);
            let viewer_from_partition = [
                i64::from(viewer[0]) - i64::from(node.x),
                i64::from(viewer[1]) - i64::from(node.y),
            ];
            let side = i64::from(node.delta_x) * viewer_from_partition[1]
                - i64::from(node.delta_y) * viewer_from_partition[0];
            let (near, far) = if side < 0 {
                (node.right_child, node.left_child)
            } else {
                (node.left_child, node.right_child)
            };
            visit_child(
                map,
                near,
                viewer,
                heading,
                occluders,
                covered,
                ancestors,
                observation,
            )?;
            // Traversing the far child even after full coverage is an
            // intentional fail-open baseline. Continuous coverage may reject
            // its individual source contributions, but coarse BSP pruning is
            // not yet being claimed by this realization.
            visit_child(
                map,
                far,
                viewer,
                heading,
                occluders,
                covered,
                ancestors,
                observation,
            )?;
            ancestors.pop();
            Ok(())
        }
    }
}

fn observe_seg(
    map: &DoomMapCore,
    seg: &DoomSeg,
    viewer: [i16; 2],
    heading: f64,
    occluders: &BTreeMap<u32, DoomSegOccluderObservation>,
    covered: &mut Vec<[f64; 2]>,
    observation: &mut OrderedSourceOccurrenceObservation,
) -> Result<(), String> {
    observation.source_segs_visited += 1;
    let start = map
        .vertices
        .get(usize::from(seg.start_vertex))
        .ok_or_else(|| format!("SEG {} start vertex is missing", seg.source.record_index))?;
    let end = map
        .vertices
        .get(usize::from(seg.end_vertex))
        .ok_or_else(|| format!("SEG {} end vertex is missing", seg.source.record_index))?;
    let linedef = map
        .linedefs
        .get(usize::from(seg.linedef))
        .ok_or_else(|| format!("SEG {} linedef is missing", seg.source.record_index))?;
    let segment = [
        i64::from(end.x) - i64::from(start.x),
        i64::from(end.y) - i64::from(start.y),
    ];
    let to_viewer = [
        i64::from(viewer[0]) - i64::from(start.x),
        i64::from(viewer[1]) - i64::from(start.y),
    ];
    let facing = segment[0] * to_viewer[1] - segment[1] * to_viewer[0];
    if facing >= 0 {
        observation.whole_rejected += 1;
        retain_disposition(
            observation,
            seg.source.record_index,
            linedef.source.record_index,
            OrderedSourceDispositionKind::TerminalRejected,
            0,
            "back-facing",
        );
        return Ok(());
    }

    let forward = [heading.cos(), heading.sin()];
    let right = [-forward[1], forward[0]];
    let view_point = |point: [i16; 2]| {
        let relative = [
            f64::from(point[0]) - f64::from(viewer[0]),
            f64::from(point[1]) - f64::from(viewer[1]),
        ];
        [
            relative[0] * forward[0] + relative[1] * forward[1],
            relative[0] * right[0] + relative[1] * right[1],
        ]
    };
    let first = view_point([start.x, start.y]);
    let second = view_point([end.x, end.y]);
    if first[0] <= DEPTH_EPSILON || second[0] <= DEPTH_EPSILON {
        retain_full_occurrence_fail_open(
            observation,
            seg.source.record_index,
            linedef.source.record_index,
            format!("seg={}:near-plane-ambiguous", seg.source.record_index),
        );
        return Ok(());
    }
    let Some(source_domain) = clip_source_domain_to_view(first, second) else {
        observation.whole_rejected += 1;
        retain_disposition(
            observation,
            seg.source.record_index,
            linedef.source.record_index,
            OrderedSourceDispositionKind::TerminalRejected,
            0,
            "outside-horizontal-view",
        );
        return Ok(());
    };
    let projected = [
        projected_x(first, second, source_domain[0]),
        projected_x(first, second, source_domain[1]),
    ];
    if !projected.iter().all(|value| value.is_finite()) {
        retain_full_occurrence_fail_open(
            observation,
            seg.source.record_index,
            linedef.source.record_index,
            format!("seg={}:non-finite-projection", seg.source.record_index),
        );
        return Ok(());
    }
    let projected_domain = [
        projected[0].min(projected[1]),
        projected[0].max(projected[1]),
    ];
    let survivors = subtract_covered(projected_domain, covered);
    if survivors.is_empty() {
        observation.whole_rejected += 1;
        retain_disposition(
            observation,
            seg.source.record_index,
            linedef.source.record_index,
            OrderedSourceDispositionKind::TerminalRejected,
            0,
            "covered-by-earlier-solid-range",
        );
    } else {
        let mut source_survivors = Vec::with_capacity(survivors.len());
        for survivor in survivors {
            let first_t = source_t_for_projected_x(first, second, survivor[0]);
            let second_t = source_t_for_projected_x(first, second, survivor[1]);
            if !first_t.is_finite() || !second_t.is_finite() {
                retain_full_occurrence_fail_open(
                    observation,
                    seg.source.record_index,
                    linedef.source.record_index,
                    format!(
                        "seg={}:source-domain-inversion-failed",
                        seg.source.record_index
                    ),
                );
                return Ok(());
            }
            source_survivors.push((
                [
                    first_t
                        .min(second_t)
                        .clamp(source_domain[0], source_domain[1]),
                    first_t
                        .max(second_t)
                        .clamp(source_domain[0], source_domain[1]),
                ],
                survivor,
            ));
        }
        let whole = source_survivors.len() == 1
            && approximately_equal(source_survivors[0].0[0], 0.0)
            && approximately_equal(source_survivors[0].0[1], 1.0);
        if whole {
            observation.whole_retained += 1;
        } else {
            observation.partial_retained += 1;
        }
        retain_disposition(
            observation,
            seg.source.record_index,
            linedef.source.record_index,
            if whole {
                OrderedSourceDispositionKind::WholeRetained
            } else {
                OrderedSourceDispositionKind::PartialSeg
            },
            source_survivors.len(),
            if whole {
                "whole-source-domain-retained"
            } else {
                "partial-source-domain-retained"
            },
        );
        observation
            .occurrences
            .extend(
                source_survivors
                    .into_iter()
                    .map(|(source_interval, view_interval)| OrderedSourceOccurrence {
                        source_seg: seg.source.record_index,
                        source_linedef: linedef.source.record_index,
                        source_interval,
                        view_interval,
                    }),
            );
    }

    let authority = occluders.get(&seg.source.record_index).ok_or_else(|| {
        format!(
            "SEG {} has no occluder classification",
            seg.source.record_index
        )
    })?;
    if authority.kind != DoomSegOccluderKind::Open {
        merge_covered(covered, projected_domain);
    }
    Ok(())
}

fn clip_source_domain_to_view(first: [f64; 2], second: [f64; 2]) -> Option<[f64; 2]> {
    let mut domain = [0.0, 1.0];
    // q >= -1  => lateral + depth >= 0
    // q <=  1  => depth - lateral >= 0
    for (start, end) in [
        (first[1] + first[0], second[1] + second[0]),
        (first[0] - first[1], second[0] - second[1]),
    ] {
        clip_linear_nonnegative(&mut domain, start, end)?;
    }
    (domain[1] - domain[0] > INTERVAL_EPSILON).then_some(domain)
}

fn clip_linear_nonnegative(domain: &mut [f64; 2], start: f64, end: f64) -> Option<()> {
    let delta = end - start;
    if delta.abs() <= INTERVAL_EPSILON {
        return (start >= 0.0).then_some(());
    }
    let crossing = -start / delta;
    if delta > 0.0 {
        domain[0] = domain[0].max(crossing);
    } else {
        domain[1] = domain[1].min(crossing);
    }
    (domain[0] <= domain[1]).then_some(())
}

fn projected_x(first: [f64; 2], second: [f64; 2], t: f64) -> f64 {
    let depth = first[0] + (second[0] - first[0]) * t;
    let lateral = first[1] + (second[1] - first[1]) * t;
    (lateral / depth) / HALF_FOV_TANGENT
}

fn source_t_for_projected_x(first: [f64; 2], second: [f64; 2], projected: f64) -> f64 {
    let depth_delta = second[0] - first[0];
    let lateral_delta = second[1] - first[1];
    let denominator = projected * HALF_FOV_TANGENT * depth_delta - lateral_delta;
    (first[1] - projected * HALF_FOV_TANGENT * first[0]) / denominator
}

fn subtract_covered(interval: [f64; 2], covered: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let mut survivors = vec![interval];
    for occluded in covered {
        let mut next = Vec::new();
        for survivor in survivors {
            if occluded[1] <= survivor[0] + INTERVAL_EPSILON
                || occluded[0] >= survivor[1] - INTERVAL_EPSILON
            {
                next.push(survivor);
                continue;
            }
            if survivor[0] + INTERVAL_EPSILON < occluded[0] {
                next.push([survivor[0], occluded[0].min(survivor[1])]);
            }
            if occluded[1] + INTERVAL_EPSILON < survivor[1] {
                next.push([occluded[1].max(survivor[0]), survivor[1]]);
            }
        }
        survivors = next;
    }
    survivors
}

fn merge_covered(covered: &mut Vec<[f64; 2]>, interval: [f64; 2]) {
    covered.push(interval);
    covered.sort_by(|left, right| left[0].total_cmp(&right[0]));
    let mut merged = Vec::<[f64; 2]>::new();
    for range in covered.drain(..) {
        if let Some(last) = merged.last_mut() {
            if range[0] <= last[1] + INTERVAL_EPSILON {
                last[1] = last[1].max(range[1]);
                continue;
            }
        }
        merged.push(range);
    }
    *covered = merged;
}

fn retain_fail_open(observation: &mut OrderedSourceOccurrenceObservation, reason: String) {
    observation.unresolved_fail_open += 1;
    if observation.fail_open_samples.len() < 12 {
        observation.fail_open_samples.push(reason);
    }
}

fn retain_full_occurrence_fail_open(
    observation: &mut OrderedSourceOccurrenceObservation,
    source_seg: u32,
    source_linedef: u32,
    reason: String,
) {
    retain_disposition(
        observation,
        source_seg,
        source_linedef,
        OrderedSourceDispositionKind::UnresolvedFailOpen,
        1,
        &reason,
    );
    observation.occurrences.push(OrderedSourceOccurrence {
        source_seg,
        source_linedef,
        source_interval: [0.0, 1.0],
        view_interval: [-1.0, 1.0],
    });
    retain_fail_open(observation, reason);
}

fn retain_disposition(
    observation: &mut OrderedSourceOccurrenceObservation,
    source_seg: u32,
    source_linedef: u32,
    kind: OrderedSourceDispositionKind,
    occurrence_count: usize,
    reason: impl Into<String>,
) {
    observation.dispositions.push(OrderedSourceDisposition {
        source_seg,
        source_linedef,
        kind,
        occurrence_count,
        reason: reason.into(),
    });
}

fn approximately_equal(left: f64, right: f64) -> bool {
    (left - right).abs() <= INTERVAL_EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;
    use doom_geometry_provider::DoomWallSideKind;
    use doom_map_provider::DoomSourceRecord;

    fn source(record_index: u32) -> DoomSourceRecord {
        DoomSourceRecord {
            lump_index: 1,
            record_index,
        }
    }

    fn sector(record_index: u32, floor_height: i16, ceiling_height: i16) -> DoomSector {
        DoomSector {
            source: source(record_index),
            floor_height,
            ceiling_height,
            floor_texture: "FLOOR".to_owned(),
            ceiling_texture: "CEILING".to_owned(),
            light_level: 160,
            special: 0,
            tag: 0,
        }
    }

    fn occurrence() -> OrderedSourceOccurrence {
        OrderedSourceOccurrence {
            source_seg: 3,
            source_linedef: 5,
            source_interval: [0.25, 0.75],
            view_interval: [-0.25, 0.25],
        }
    }

    fn mark(back_sector: Option<u32>, paired_sky: bool) -> DoomSegPlaneMarkObservation {
        DoomSegPlaneMarkObservation {
            source_seg: source(3),
            source_linedef: source(5),
            side: DoomWallSideKind::Right,
            front_sector: source(7),
            back_sector: back_sector.map(source),
            floor_marked: true,
            ceiling_marked: true,
            paired_sky_ceiling_adjustment: paired_sky,
        }
    }

    #[test]
    fn continuous_coverage_can_split_one_projected_source_domain() {
        let survivors = subtract_covered([-0.8, 0.8], &[[-0.5, 0.5]]);
        assert_eq!(survivors, vec![[-0.8, -0.5], [0.5, 0.8]]);
    }

    #[test]
    fn continuous_coverage_merges_without_pixel_quantization() {
        let mut covered = vec![[-0.8, -0.2], [0.3, 0.6]];
        merge_covered(&mut covered, [-0.25, 0.35]);
        assert_eq!(covered, vec![[-0.8, 0.6]]);
    }

    #[test]
    fn source_dispositions_conserve_unique_segs_and_emitted_occurrences() {
        let observation = OrderedSourceOccurrenceObservation {
            source_seg_records: 4,
            source_segs_visited: 4,
            whole_retained: 1,
            partial_retained: 1,
            whole_rejected: 1,
            unresolved_fail_open: 1,
            occurrences: vec![
                OrderedSourceOccurrence {
                    source_seg: 1,
                    source_linedef: 11,
                    source_interval: [0.0, 1.0],
                    view_interval: [-1.0, 1.0],
                },
                OrderedSourceOccurrence {
                    source_seg: 2,
                    source_linedef: 12,
                    source_interval: [0.0, 0.25],
                    view_interval: [-1.0, -0.5],
                },
                OrderedSourceOccurrence {
                    source_seg: 2,
                    source_linedef: 12,
                    source_interval: [0.75, 1.0],
                    view_interval: [0.5, 1.0],
                },
                OrderedSourceOccurrence {
                    source_seg: 4,
                    source_linedef: 14,
                    source_interval: [0.0, 1.0],
                    view_interval: [-1.0, 1.0],
                },
            ],
            dispositions: vec![
                OrderedSourceDisposition {
                    source_seg: 1,
                    source_linedef: 11,
                    kind: OrderedSourceDispositionKind::WholeRetained,
                    occurrence_count: 1,
                    reason: "whole".to_owned(),
                },
                OrderedSourceDisposition {
                    source_seg: 2,
                    source_linedef: 12,
                    kind: OrderedSourceDispositionKind::PartialSeg,
                    occurrence_count: 2,
                    reason: "split".to_owned(),
                },
                OrderedSourceDisposition {
                    source_seg: 3,
                    source_linedef: 13,
                    kind: OrderedSourceDispositionKind::TerminalRejected,
                    occurrence_count: 0,
                    reason: "covered".to_owned(),
                },
                OrderedSourceDisposition {
                    source_seg: 4,
                    source_linedef: 14,
                    kind: OrderedSourceDispositionKind::UnresolvedFailOpen,
                    occurrence_count: 1,
                    reason: "near-plane".to_owned(),
                },
            ],
            ..OrderedSourceOccurrenceObservation::default()
        };

        assert!(observation.disposition_conservation_is_balanced());
        assert!(observation.verify_disposition_conservation().is_ok());
    }

    #[test]
    fn terminal_source_rejection_cannot_own_an_occurrence() {
        let observation = OrderedSourceOccurrenceObservation {
            source_seg_records: 1,
            source_segs_visited: 1,
            whole_rejected: 1,
            occurrences: vec![OrderedSourceOccurrence {
                source_seg: 3,
                source_linedef: 13,
                source_interval: [0.0, 1.0],
                view_interval: [-1.0, 1.0],
            }],
            dispositions: vec![OrderedSourceDisposition {
                source_seg: 3,
                source_linedef: 13,
                kind: OrderedSourceDispositionKind::TerminalRejected,
                occurrence_count: 1,
                reason: "invalid-reentry".to_owned(),
            }],
            ..OrderedSourceOccurrenceObservation::default()
        };

        assert!(!observation.disposition_conservation_is_balanced());
        assert!(observation.verify_disposition_conservation().is_err());
    }

    #[test]
    fn plane_source_dispositions_conserve_unique_source_triangles() {
        let observation = OrderedPlaneLoweringObservation {
            destination_source_triangles: 4,
            source_dispositions: vec![
                OrderedPlaneSourceDisposition {
                    plane_instance_ordinal: 0,
                    source_subsector: 7,
                    source_triangle_ordinal: 0,
                    kind: OrderedSourceDispositionKind::WholeRetained,
                    output_count: 1,
                    reason: "whole".to_owned(),
                },
                OrderedPlaneSourceDisposition {
                    plane_instance_ordinal: 0,
                    source_subsector: 7,
                    source_triangle_ordinal: 1,
                    kind: OrderedSourceDispositionKind::PartialPlane,
                    output_count: 2,
                    reason: "split".to_owned(),
                },
                OrderedPlaneSourceDisposition {
                    plane_instance_ordinal: 1,
                    source_subsector: 8,
                    source_triangle_ordinal: 0,
                    kind: OrderedSourceDispositionKind::TerminalRejected,
                    output_count: 0,
                    reason: "covered".to_owned(),
                },
                OrderedPlaneSourceDisposition {
                    plane_instance_ordinal: 2,
                    source_subsector: 9,
                    source_triangle_ordinal: 0,
                    kind: OrderedSourceDispositionKind::UnresolvedFailOpen,
                    output_count: 0,
                    reason: "unresolved".to_owned(),
                },
            ],
            ..OrderedPlaneLoweringObservation::default()
        };

        assert!(observation.plane_disposition_conservation_is_balanced());
        assert!(observation.verify_plane_disposition_conservation().is_ok());
    }

    #[test]
    fn rejected_plane_source_triangle_cannot_own_lowered_output() {
        let observation = OrderedPlaneLoweringObservation {
            destination_source_triangles: 1,
            source_dispositions: vec![OrderedPlaneSourceDisposition {
                plane_instance_ordinal: 0,
                source_subsector: 7,
                source_triangle_ordinal: 0,
                kind: OrderedSourceDispositionKind::TerminalRejected,
                output_count: 1,
                reason: "invalid-reentry".to_owned(),
            }],
            ..OrderedPlaneLoweringObservation::default()
        };

        assert!(!observation.plane_disposition_conservation_is_balanced());
        assert!(observation.verify_plane_disposition_conservation().is_err());
    }

    #[test]
    fn plane_occurrence_domains_merge_before_geometry_clipping() {
        assert_eq!(
            merge_intervals(vec![[-0.8, -0.2], [0.3, 0.6], [-0.25, 0.35]]),
            vec![[-0.8, 0.6]]
        );
    }

    #[test]
    fn plane_triangle_is_clipped_to_the_authorized_view_wedge() {
        let triangle = DoomSurfaceTriangle {
            source_subsector: source(11),
            source_sector: source(12),
            plane: DoomSurfacePlane::Floor,
            texture_name: "FLOOR".to_owned(),
            positions: [[10.0, 0.0, -10.0], [10.0, 0.0, 10.0], [20.0, 0.0, 0.0]],
        };
        let fragments = clip_plane_triangle_to_view_interval(&triangle, [0, 0], 0.0, [-0.2, 0.2]);
        assert!(!fragments.is_empty());
        for position in fragments
            .iter()
            .flat_map(|fragment| fragment.positions.iter())
        {
            let projected = position[2] / position[0];
            assert!((-0.2 - INTERVAL_EPSILON..=0.2 + INTERVAL_EPSILON).contains(&projected));
        }
    }

    #[test]
    fn partial_plane_triangle_is_bounded_by_its_own_plane_domain_cell() {
        let triangle = DoomSurfaceTriangle {
            source_subsector: source(11),
            source_sector: source(12),
            plane: DoomSurfacePlane::Ceiling,
            texture_name: "CEILING".to_owned(),
            positions: [[0.0, 64.0, 0.0], [8.0, 64.0, 0.0], [0.0, 64.0, 8.0]],
        };
        let cell = OrderedPlaneDomainCell {
            kind: OrderedPlaneKind::Ceiling,
            source_sector: 12,
            source_subsector: 11,
            source_height: 64,
            texture: "CEILING".to_owned(),
            light_level: 160,
            source_seg: 5,
            source_corners: [[2.0, 2.0], [6.0, 2.0], [6.0, 6.0], [2.0, 6.0]],
        };

        let fragments = clip_plane_triangle_to_domain_cell(&triangle, &cell);
        assert!(!fragments.is_empty());
        for position in fragments
            .iter()
            .flat_map(|fragment| fragment.positions.iter())
        {
            assert!((2.0 - INTERVAL_EPSILON..=6.0 + INTERVAL_EPSILON).contains(&position[0]));
            assert!((2.0 - INTERVAL_EPSILON..=6.0 + INTERVAL_EPSILON).contains(&position[2]));
        }
    }

    #[test]
    fn plane_triangle_outside_the_authorized_view_wedge_is_rejected() {
        let triangle = DoomSurfaceTriangle {
            source_subsector: source(11),
            source_sector: source(12),
            plane: DoomSurfacePlane::Ceiling,
            texture_name: "CEILING".to_owned(),
            positions: [[10.0, 64.0, 5.0], [10.0, 64.0, 8.0], [20.0, 64.0, 12.0]],
        };
        assert!(
            clip_plane_triangle_to_view_interval(&triangle, [0, 0], 0.0, [-0.2, 0.2]).is_empty()
        );
    }

    #[test]
    fn source_projection_inverse_recovers_original_parameter() {
        let first = [2.0, -1.0];
        let second = [5.0, 2.0];
        for expected in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let projected = projected_x(first, second, expected);
            let actual = source_t_for_projected_x(first, second, projected);
            assert!((actual - expected).abs() < 1.0e-9);
        }
    }

    #[test]
    fn fov_clip_returns_continuous_source_domain() {
        let clipped = clip_source_domain_to_view([2.0, -4.0], [2.0, 0.0]).unwrap();
        assert!((clipped[0] - 0.5).abs() < 1.0e-9);
        assert_eq!(clipped[1], 1.0);
    }

    #[test]
    fn wall_category_conservation_requires_every_triangle_to_have_one_policy() {
        let observation = OrderedWallOccurrenceLoweringObservation {
            matched_source_triangles: 7,
            matched_opaque_source_triangles: 5,
            matched_cutout_source_triangles: 2,
            material_resolved_source_triangles: 7,
            material_resolved_opaque_source_triangles: 5,
            material_resolved_cutout_source_triangles: 2,
            clipped_source_triangles: 4,
            clipped_opaque_triangles: 3,
            clipped_cutout_triangles: 1,
            lowered_wall_meshes: 4,
            lowered_opaque_meshes: 3,
            lowered_cutout_meshes: 1,
            ..OrderedWallOccurrenceLoweringObservation::default()
        };

        assert!(observation
            .report()
            .contains("category-conservation=balanced"));
        assert!(observation
            .report()
            .contains("material-conservation=balanced"));
    }

    #[test]
    fn plane_occurrence_report_distinguishes_partition_and_association_conservation() {
        let observation = OrderedPlaneOccurrenceObservation {
            occurrences: 3,
            occurrences_with_marked_planes: 2,
            occurrences_without_marked_planes: 1,
            floor_associations: 2,
            ceiling_associations: 1,
            associations: vec![
                OrderedPlaneOccurrence {
                    occurrence: OrderedSourceOccurrence {
                        source_seg: 1,
                        source_linedef: 2,
                        source_interval: [0.0, 1.0],
                        view_interval: [-0.5, 0.5],
                    },
                    source_subsector: 4,
                    kind: OrderedPlaneKind::Floor,
                    source_sector: 3,
                    source_height: 0,
                    texture: "FLOOR".to_owned(),
                    light_level: 160,
                    sky: false,
                };
                3
            ],
            ..OrderedPlaneOccurrenceObservation::default()
        };

        let report = observation.report();
        assert!(report.contains("occurrence-conservation=balanced"));
        assert!(report.contains("association-conservation=balanced"));
        assert!(report.contains("legacy-screen-columns-used=false"));
    }

    #[test]
    fn two_sided_boundary_uses_shared_max_floor_and_min_ceiling() {
        let front = sector(7, 0, 128);
        let back = sector(8, 24, 96);
        let boundary = prepare_occurrence_boundary(
            occurrence(),
            &mark(Some(8), false),
            &front,
            Some(&back),
            3,
            2,
        )
        .unwrap();

        assert_eq!(boundary.front_vertical, [0, 128]);
        assert_eq!(boundary.back_vertical, Some([24, 96]));
        assert_eq!(boundary.opening, Some([24, 96]));
        assert_eq!(boundary.wall_consumers, 3);
        assert_eq!(boundary.plane_consumers, 2);
    }

    #[test]
    fn one_sided_boundary_is_solid_without_fabricating_an_opening() {
        let front = sector(7, 0, 128);
        let boundary =
            prepare_occurrence_boundary(occurrence(), &mark(None, false), &front, None, 1, 2)
                .unwrap();

        assert_eq!(boundary.back_vertical, None);
        assert_eq!(boundary.opening, None);
    }

    #[test]
    fn paired_sky_is_metadata_and_does_not_mutate_the_shared_opening() {
        let front = sector(7, 0, 128);
        let back = sector(8, 24, 96);
        let boundary = prepare_occurrence_boundary(
            occurrence(),
            &mark(Some(8), true),
            &front,
            Some(&back),
            1,
            1,
        )
        .unwrap();

        assert_eq!(boundary.opening, Some([24, 96]));
        assert!(boundary.paired_sky_ceiling_adjustment);
    }

    #[test]
    fn reversed_sector_heights_fail_open_instead_of_fabricating_coverage() {
        let front = sector(7, 128, 0);
        let error =
            prepare_occurrence_boundary(occurrence(), &mark(None, false), &front, None, 1, 2)
                .unwrap_err();

        assert!(error.contains("front-sector-reversed-heights"));
    }

    #[test]
    fn exact_plane_identity_groups_occurrences_but_retains_subsector_provenance() {
        let associations = [4_u32, 5, 4]
            .into_iter()
            .enumerate()
            .map(|(ordinal, source_subsector)| OrderedPlaneOccurrence {
                occurrence: OrderedSourceOccurrence {
                    source_seg: ordinal as u32,
                    source_linedef: 8,
                    source_interval: [0.0, 1.0],
                    view_interval: [-0.5, 0.5],
                },
                source_subsector,
                kind: OrderedPlaneKind::Floor,
                source_sector: 3,
                source_height: 0,
                texture: "FLOOR".to_owned(),
                light_level: 160,
                sky: false,
            })
            .collect::<Vec<_>>();

        let instances = group_ordered_plane_instances(&associations);

        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].occurrence_references, 3);
        assert_eq!(instances[0].source_subsectors, vec![4, 5]);
    }

    #[test]
    fn equal_plane_values_in_different_sectors_remain_distinct_instances() {
        let associations = [3_u32, 9]
            .into_iter()
            .map(|source_sector| OrderedPlaneOccurrence {
                occurrence: occurrence(),
                source_subsector: source_sector + 1,
                kind: OrderedPlaneKind::Ceiling,
                source_sector,
                source_height: 128,
                texture: "CEILING".to_owned(),
                light_level: 160,
                sky: false,
            })
            .collect::<Vec<_>>();

        let instances = group_ordered_plane_instances(&associations);

        assert_eq!(instances.len(), 2);
        assert_eq!(instances[0].source_sector, 3);
        assert_eq!(instances[1].source_sector, 9);
    }
}
