use super::{AppliedRenderStrategy, CandidateSelection, SceneInput};
use tokimu::PlatformResult;

/// First executable topology-admission candidate.
///
/// Slice 1 deliberately fails every undecided contribution open, so this
/// stage does not mutate or remove original geometry. The separate inventory
/// proves identity and structural conservation before source rejection earns
/// implementation.
pub(super) fn apply(_scene: &mut SceneInput) -> PlatformResult<AppliedRenderStrategy> {
    Ok(AppliedRenderStrategy {
        candidate_selection: CandidateSelection::FullSubmission,
        ordered_coverage_prepared: false,
        source_covered_domain_filter: false,
        source_occurrence_support_filter: false,
        fixed_reconstruction_camera: false,
    })
}
