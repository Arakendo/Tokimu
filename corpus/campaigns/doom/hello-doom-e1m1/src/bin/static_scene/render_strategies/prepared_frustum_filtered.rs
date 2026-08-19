use super::{prepared_full_submission, AppliedRenderStrategy, CandidateSelection, SceneInput};
use tokimu::PlatformResult;

/// Strategy C is explicitly B followed by Tokimu's conservative AABB/frustum
/// filter. The generic filter cannot repair or participate in Doom preparation.
pub(super) fn apply(scene: &mut SceneInput) -> PlatformResult<AppliedRenderStrategy> {
    prepared_full_submission::apply_prepared_declarations(scene)?;
    Ok(AppliedRenderStrategy {
        candidate_selection: CandidateSelection::FrustumAabb,
        ordered_coverage_prepared: true,
        source_covered_domain_filter: false,
        source_occurrence_support_filter: false,
        final_wall_occurrence_filter: false,
        fixed_reconstruction_camera: false,
    })
}
