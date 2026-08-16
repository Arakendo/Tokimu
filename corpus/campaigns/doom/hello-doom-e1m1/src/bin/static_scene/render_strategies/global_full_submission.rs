use super::{AppliedRenderStrategy, CandidateSelection, SceneInput};
use tokimu::PlatformResult;

/// Strategy A is intentionally a no-op over the prepared global shell. Its
/// separate file makes the correctness control an explicit callable strategy.
pub(super) fn apply(_scene: &mut SceneInput) -> PlatformResult<AppliedRenderStrategy> {
    Ok(AppliedRenderStrategy {
        candidate_selection: CandidateSelection::FullSubmission,
        ordered_coverage_prepared: false,
        fixed_reconstruction_camera: false,
    })
}
