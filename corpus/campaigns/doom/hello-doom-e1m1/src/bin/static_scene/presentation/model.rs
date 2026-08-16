use std::collections::BTreeSet;

use doom_geometry_provider::{
    DoomClassicBspObservation, DoomOrderedWallFragmentReconstruction, DoomSegClassicPlaneKey,
    DoomSegClassicVerticalClipObservation,
};
use hello_doom_e1m1::StaticDrawPlanEntry;

/// Corpus-only output of the Stage 3B diagnostic span experiment.
pub(crate) struct DoomSegClipPresentation {
    pub(crate) draws: Vec<StaticDrawPlanEntry>,
    pub(crate) visible_intervals: usize,
    pub(crate) source_triangles: usize,
}

/// Retained source-only result of one bounded Stage 3B projected-grid control.
pub(crate) struct DoomSegScreenGridObservation {
    pub(crate) selected_seg_records: BTreeSet<u32>,
    pub(crate) outside: usize,
    pub(crate) fully_covered: usize,
    pub(crate) partial: usize,
    pub(crate) fully_visible: usize,
    pub(crate) contributors: usize,
    pub(crate) covered_cells: usize,
    pub(crate) depth_order_inversions: usize,
    pub(crate) depth_order_samples: Vec<String>,
    pub(crate) samples: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DoomSegScreenGridOrder {
    BspLeafThenSource,
    NearestSegmentToViewer,
}

#[derive(Default)]
pub(crate) struct DoomSegClassicAdmissionObservation {
    pub(crate) source_segs: usize,
    pub(crate) backface_rejected: usize,
    pub(crate) outside_fov_rejected: usize,
    pub(crate) near_plane_fail_open: usize,
    pub(crate) edge_on: usize,
    pub(crate) solid_admitted: usize,
    pub(crate) pass_admitted: usize,
    pub(crate) solid_range_contributors: usize,
    pub(crate) solid_range_fully_covered: usize,
    pub(crate) solid_range_covered_columns: usize,
    pub(crate) samples: Vec<String>,
}

#[derive(Default)]
pub(crate) struct DoomSegClassicPlaneFlatResolution {
    pub(crate) resolved_instances: usize,
    pub(crate) unresolved_instances: usize,
    pub(crate) sky_instances: usize,
    pub(crate) candidate_draws: usize,
    pub(crate) candidate_triangles: usize,
    pub(crate) samples: Vec<String>,
}

/// Headless reconstruction of retained diagnostic screen cells on their
/// source-height planes. The resulting quads prove that viewer-relative plane
/// geometry can be recovered without selecting whole subsector meshes. They
/// are not uploaded meshes, historic Doom pixels, or renderer visibility.
#[derive(Default)]
pub(crate) struct DoomSegClassicPlaneCellReconstruction {
    pub(crate) source_cells: usize,
    pub(crate) reconstructed_quads: usize,
    pub(crate) reconstructed_triangles: usize,
    pub(crate) horizon_rejections: usize,
    pub(crate) behind_viewer_rejections: usize,
    pub(crate) degenerate_rejections: usize,
    pub(crate) maximum_source_distance: f64,
    pub(crate) cells: Vec<DoomSegClassicPlaneCell>,
    pub(crate) samples: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct DoomSegClassicPlaneCell {
    pub(crate) key: DoomSegClassicPlaneKey,
    pub(crate) source_height: i16,
    pub(crate) source_sector: u32,
    pub(crate) source_seg: u32,
    pub(crate) source_corners: [[f64; 2]; 4],
}

pub(crate) struct DoomSegClassicPlanePresentation {
    pub(crate) draws: Vec<StaticDrawPlanEntry>,
    pub(crate) source_cells: usize,
    pub(crate) grouped_meshes: usize,
    pub(crate) triangles: usize,
}

pub(crate) struct DoomSegClassicContextPresentation {
    pub(crate) draws: Vec<StaticDrawPlanEntry>,
    pub(crate) plane_meshes: usize,
    pub(crate) plane_triangles: usize,
    pub(crate) wall_meshes: usize,
    pub(crate) omitted_wall_triangles: usize,
}

/// Fixed-source-spawn comparison that consumes the Doom provider's retained
/// per-column wall intervals as source-labelled ordinary meshes. The
/// intervals and reconstruction remain Doom-owned; `tokimu-render` receives
/// only the resulting opaque/cutout draws.
pub(crate) struct DoomSegOrderedCoveragePresentation {
    pub(crate) opaque_draws: Vec<StaticDrawPlanEntry>,
    pub(crate) cutout_draws: Vec<StaticDrawPlanEntry>,
    pub(crate) retained_cells: usize,
    pub(crate) reconstructed_triangles: usize,
    pub(crate) lowered_wall_triangles: usize,
    pub(crate) source_degenerate_cells: usize,
    pub(crate) source_unresolved_cells: usize,
    pub(crate) lowering_degenerate_triangles: usize,
    pub(crate) lowering_unresolved_triangles: usize,
    pub(crate) grouped_wall_meshes: usize,
    pub(crate) ordinary_plane_intervals: usize,
    pub(crate) sky_plane_intervals: usize,
    pub(crate) reconstructed_plane_quads: usize,
    pub(crate) rejected_plane_intervals: usize,
    pub(crate) lowered_plane_quads: usize,
    pub(crate) source_cutout_keys: usize,
    pub(crate) lowered_cutout_keys: usize,
    pub(crate) coverage_transitions: usize,
    pub(crate) coverage_fail_open: usize,
    pub(crate) coverage_fail_open_reasons: DoomCoverageFailOpenSummary,
    pub(crate) bsp_leaves_visited: usize,
    pub(crate) bsp_far_children_pruned: usize,
    pub(crate) bsp_admitted_segs: usize,
    pub(crate) bsp_solid_range_pruning: bool,
    pub(crate) degenerate_omissions: usize,
    pub(crate) unresolved_cells: usize,
    pub(crate) samples: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct DoomCoverageFailOpenSummary {
    pub(crate) missing_plane_mark: usize,
    pub(crate) missing_source_seg: usize,
    pub(crate) projection_behind_viewer: usize,
    pub(crate) projection_outside_horizontal_fov: usize,
    pub(crate) ray_segment_depth_unresolved: usize,
    pub(crate) unique_source_segs: usize,
    pub(crate) unique_columns: usize,
}

/// One fixed-view Slice 4B source observation. Walls and planes are derived
/// from the same traversal and vertical-coverage state before either is
/// lowered into renderer declarations.
pub(crate) struct DoomOrderedCoveragePreparation {
    pub(crate) traversal: DoomClassicBspObservation,
    pub(crate) vertical: DoomSegClassicVerticalClipObservation,
    pub(crate) walls: DoomOrderedWallFragmentReconstruction,
    pub(crate) planes: DoomSegClassicPlaneCellReconstruction,
    pub(crate) ordinary_plane_intervals: usize,
    pub(crate) sky_plane_intervals: usize,
}

#[derive(Default)]
pub(crate) struct DoomSegClassicPlaneIdentityObservation {
    pub(crate) floor_mark_contributors: usize,
    pub(crate) ceiling_mark_contributors: usize,
    pub(crate) unique_floor_keys: usize,
    pub(crate) unique_ceiling_keys: usize,
    pub(crate) sky_ceiling_contributors: usize,
    pub(crate) samples: Vec<String>,
}

pub(crate) struct DoomSegPerColumnPresentation {
    pub(crate) wall_draws: Vec<StaticDrawPlanEntry>,
    pub(crate) selected_segs: usize,
}
