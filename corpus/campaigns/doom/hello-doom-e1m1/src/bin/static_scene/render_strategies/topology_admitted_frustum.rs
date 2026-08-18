use super::{AppliedRenderStrategy, CandidateSelection, SceneInput};
use tokimu::PlatformResult;

/// Conservative post-filter control over the current all-fail-open topology
/// inventory. Source admission remains an earlier, Doom-owned stage.
pub(super) fn apply(_scene: &mut SceneInput) -> PlatformResult<AppliedRenderStrategy> {
    Ok(AppliedRenderStrategy {
        candidate_selection: CandidateSelection::FrustumAabb,
        ordered_coverage_prepared: false,
        source_covered_domain_filter: false,
        source_occurrence_support_filter: false,
        fixed_reconstruction_camera: false,
    })
}
