use hello_doom_e1m1::StaticDrawAabb;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CandidateSelection {
    FullSubmission,
    FrustumAabb,
    /// Fixed corpus evidence configuration, not a renderer or application
    /// selection contract. AR-0025 compares this grid with the AABB baseline.
    UniformGrid8x4x8,
    DoomMembershipUnion,
    /// Source-specific, fail-open per-column SEG control for AR-0025 only.
    DoomSegPerColumn,
    /// Doom-owned, corpus-local BSP/solid-range control. Walls follow admitted
    /// source SEGs; flats follow leaves reached by the same traversal. This is
    /// deliberately not renderer visibility or historic pixel parity.
    DoomClassicBsp,
}

pub(crate) fn candidate_selection_label(
    selection: CandidateSelection,
    ordered_coverage_prepared: bool,
) -> &'static str {
    match (selection, ordered_coverage_prepared) {
        (CandidateSelection::FullSubmission, false) => "global-full-submission",
        (CandidateSelection::FullSubmission, true) => "prepared-full-submission",
        (CandidateSelection::FrustumAabb, true) => "prepared-frustum-filtered",
        (CandidateSelection::FrustumAabb, false) => "global-frustum-aabb",
        (CandidateSelection::UniformGrid8x4x8, _) => "uniform-grid-8x4x8",
        (CandidateSelection::DoomMembershipUnion, _) => "doom-membership-union",
        (CandidateSelection::DoomSegPerColumn, _) => "doom-seg-per-column-dynamic",
        (CandidateSelection::DoomClassicBsp, _) => "doom-seg-classic-dynamic",
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CandidateSelectionSummary {
    pub(crate) candidates: usize,
    pub(crate) rejected: usize,
    pub(crate) submitted: usize,
    pub(crate) uncertain_bounds: usize,
    pub(crate) rejected_by_plane: [usize; 6],
}

impl CandidateSelectionSummary {
    pub(crate) fn merge(&mut self, other: Self) {
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

/// Corpus-only aggregate of a contiguous caller-owned draw range. The range is
/// a comparative selection unit, not a renderer batch, material group, or
/// source identity.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GroupCandidateSelectionSummary {
    pub(crate) groups: usize,
    pub(crate) rejected_groups: usize,
    pub(crate) submitted_groups: usize,
    pub(crate) submitted_draws: usize,
    pub(crate) uncertain_groups: usize,
}

/// Corpus-local static uniform grid for AR-0025 Stage 2. It owns neither scene
/// membership nor rendering: callers retain the ordered draw list and the grid
/// only proposes which declared bounds need exact AABB/frustum testing.
#[derive(Debug)]
pub(crate) struct UniformGridAabbIndex {
    pub(crate) bounds: StaticDrawAabb,
    pub(crate) dimensions: [usize; 3],
    pub(crate) cells: Vec<Vec<usize>>,
    pub(crate) uncertain_draws: Vec<usize>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct UniformGridSelectionSummary {
    pub(crate) cells_tested: usize,
    pub(crate) cells_rejected: usize,
    pub(crate) grid_candidates: usize,
    pub(crate) exact_tests: usize,
    pub(crate) submitted: usize,
    pub(crate) rejected: usize,
    pub(crate) uncertain_bounds: usize,
    pub(crate) rejected_by_plane: [usize; 6],
}

#[cfg(test)]
mod tests {
    use super::{candidate_selection_label, CandidateSelection};

    #[test]
    fn labels_distinguish_global_and_prepared_submission_stages() {
        assert_eq!(
            candidate_selection_label(CandidateSelection::FullSubmission, false),
            "global-full-submission"
        );
        assert_eq!(
            candidate_selection_label(CandidateSelection::FullSubmission, true),
            "prepared-full-submission"
        );
        assert_eq!(
            candidate_selection_label(CandidateSelection::FrustumAabb, true),
            "prepared-frustum-filtered"
        );
        assert_eq!(
            candidate_selection_label(CandidateSelection::FrustumAabb, false),
            "global-frustum-aabb"
        );
    }
}
