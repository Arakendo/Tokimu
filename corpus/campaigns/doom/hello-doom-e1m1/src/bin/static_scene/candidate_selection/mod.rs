//! Corpus-local candidate-selection models and conservative generic filters.
//!
//! Doom source preparation remains upstream of this subject. These helpers
//! only select from already-declared presentation work and never repair or
//! reinterpret missing source contributions.

mod conservative;
mod doom;
mod model;

pub(super) use conservative::{
    draw_bounds, draw_spheres, select_current_candidates, summarize_candidate_selection,
    summarize_grouped_aabb_selection,
};
pub(super) use doom::{
    membership_draw_selected, select_membership_candidates, select_seg_classic_bsp_candidates,
    select_seg_per_column_candidates, DoomMembershipSelectionInput, DoomSegDynamicSelectionInput,
};
pub(super) use model::{
    candidate_selection_label, CandidateSelection, CandidateSelectionSummary, UniformGridAabbIndex,
};
