//! Corpus-local render-strategy selection for the Slice 7 A/B/C comparison.
//!
//! These strategies describe which declaration domain reaches `tokimu-render`.
//! They are executable study alternatives, not renderer API or engine policy.

mod global_full_submission;
pub(crate) mod legacy_comparisons;
mod prepared_frustum_filtered;
mod prepared_full_submission;

use super::{CandidateSelection, SceneInput};
use tokimu::PlatformResult;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TrialRenderStrategy {
    /// A: submit the original global E1M1 shell.
    GlobalFullSubmission,
    /// B: submit every declaration retained by one ordered Doom preparation.
    PreparedFullSubmission,
    /// C: conservatively frustum-filter B's prepared declarations.
    PreparedFrustumFiltered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AppliedRenderStrategy {
    pub(crate) candidate_selection: CandidateSelection,
    pub(crate) ordered_coverage_prepared: bool,
    pub(crate) fixed_reconstruction_camera: bool,
}

impl TrialRenderStrategy {
    pub(crate) fn from_args(
        args: &[String],
        compatibility_prepared_flag: bool,
        compatibility_frustum_flag: bool,
    ) -> PlatformResult<Option<Self>> {
        let explicit = args
            .iter()
            .filter_map(|argument| argument.strip_prefix("--render-strategy="))
            .map(|name| match name {
                "a" | "global-full-submission" => Ok(Self::GlobalFullSubmission),
                "b" | "prepared-full-submission" => Ok(Self::PreparedFullSubmission),
                "c" | "prepared-frustum-filtered" => Ok(Self::PreparedFrustumFiltered),
                _ => Err(format!(
                    "unknown render strategy `{name}`; expected a/global-full-submission, b/prepared-full-submission, or c/prepared-frustum-filtered"
                )),
            })
            .collect::<Result<Vec<_>, _>>()?;

        let explicit = match explicit.as_slice() {
            [] => None,
            [strategy] => Some(*strategy),
            _ => return Err("choose only one --render-strategy value".into()),
        };

        if let Some(strategy) = explicit {
            if compatibility_prepared_flag {
                return Err(
                    "--render-strategy replaces --doom-seg-ordered-coverage-presentation; choose one selection form"
                        .into(),
                );
            }
            if compatibility_frustum_flag && strategy != Self::PreparedFrustumFiltered {
                return Err(
                    "--frustum-aabb is implicit in render strategy C and conflicts with explicit A/B"
                        .into(),
                );
            }
            return Ok(Some(strategy));
        }

        if compatibility_prepared_flag {
            return Ok(Some(if compatibility_frustum_flag {
                Self::PreparedFrustumFiltered
            } else {
                Self::PreparedFullSubmission
            }));
        }

        Ok(None)
    }

    pub(crate) fn apply(self, scene: &mut SceneInput) -> PlatformResult<AppliedRenderStrategy> {
        match self {
            Self::GlobalFullSubmission => global_full_submission::apply(scene),
            Self::PreparedFullSubmission => prepared_full_submission::apply(scene),
            Self::PreparedFrustumFiltered => prepared_frustum_filtered::apply(scene),
        }
    }
}

pub(crate) fn remove_cli_args(args: &mut Vec<String>) {
    args.retain(|argument| !argument.starts_with("--render-strategy="));
}

#[cfg(test)]
mod tests {
    use super::TrialRenderStrategy;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn explicit_names_select_the_three_slice_7_dataflows() {
        assert_eq!(
            TrialRenderStrategy::from_args(&args(&["--render-strategy=a"]), false, false).unwrap(),
            Some(TrialRenderStrategy::GlobalFullSubmission)
        );
        assert_eq!(
            TrialRenderStrategy::from_args(
                &args(&["--render-strategy=prepared-full-submission"]),
                false,
                false,
            )
            .unwrap(),
            Some(TrialRenderStrategy::PreparedFullSubmission)
        );
        assert_eq!(
            TrialRenderStrategy::from_args(&args(&["--render-strategy=c"]), false, false).unwrap(),
            Some(TrialRenderStrategy::PreparedFrustumFiltered)
        );
    }

    #[test]
    fn old_ordered_coverage_flag_remains_a_compatibility_alias() {
        assert_eq!(
            TrialRenderStrategy::from_args(&[], true, false).unwrap(),
            Some(TrialRenderStrategy::PreparedFullSubmission)
        );
        assert_eq!(
            TrialRenderStrategy::from_args(&[], true, true).unwrap(),
            Some(TrialRenderStrategy::PreparedFrustumFiltered)
        );
    }

    #[test]
    fn no_trial_selection_preserves_other_existing_candidate_controls() {
        assert_eq!(
            TrialRenderStrategy::from_args(&[], false, true).unwrap(),
            None
        );
    }
}
