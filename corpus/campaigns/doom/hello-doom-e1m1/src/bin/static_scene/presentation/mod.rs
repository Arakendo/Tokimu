//! Doom-owned viewer-relative presentation preparation for the E1M1 corpus.
//!
//! This private subject module owns the intermediate observations produced
//! before ordinary Tokimu render declarations. It does not define renderer
//! visibility policy or public Doom-provider API.

mod lowering;
mod model;
mod preparation;
mod sky_cylinder;
mod sky_span;
mod viewport;

pub(crate) use lowering::{doom_wall_role_key, lower_doom_seg_classic_plane_presentation};

pub(crate) use model::{
    DoomCoverageFailOpenSummary, DoomOrderedCoveragePreparation,
    DoomSegClassicAdmissionObservation, DoomSegClassicContextPresentation,
    DoomSegClassicPlaneFlatResolution, DoomSegClassicPlaneIdentityObservation,
    DoomSegClassicPlanePresentation, DoomSegClipPresentation, DoomSegOrderedCoveragePresentation,
    DoomSegPerColumnPresentation, DoomSegScreenGridObservation, DoomSegScreenGridOrder,
};
pub(crate) use preparation::{
    prepare_doom_ordered_coverage, reconstruct_doom_seg_classic_plane_cells,
    reconstruct_doom_seg_classic_sky_cells,
};
pub(crate) use sky_cylinder::build_doom_sky_cylinder;
pub(crate) use sky_span::prepare_viewer_relative_source_sky_span_mesh;
pub(crate) use viewport::classic_presentation_half_vertical_fov;
#[cfg(test)]
pub(crate) use viewport::{
    CLASSIC_PRESENTATION_COLUMNS, CLASSIC_PRESENTATION_HALF_HORIZONTAL_FOV,
    CLASSIC_PRESENTATION_ROWS,
};
