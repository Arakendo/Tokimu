//! Corpus-local render-strategy selection for the Slice 7 A/B/C comparison.
//!
//! These strategies describe which declaration domain reaches `tokimu-render`.
//! They are executable study alternatives, not renderer API or engine policy.

mod global_full_submission;
pub(crate) mod legacy_comparisons;
mod ordered_occurrence_prepared_full;
mod prepared_frustum_filtered;
mod prepared_full_submission;
pub(crate) mod source_covered_global_shell;
mod topology_admitted_frustum;
mod topology_admitted_full;

use super::{CandidateSelection, SceneInput};
use crate::presentation::OrderedPreparedSubmissionObservation;
use tokimu::PlatformResult;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TrialRenderStrategy {
    /// A: submit the original global E1M1 shell.
    GlobalFullSubmission,
    /// Historical Slice 7 B: submit declarations reconstructed from one
    /// bounded ordered Doom preparation.
    PreparedFullSubmission,
    /// Historical Slice 7 C: frustum-filter the reconstructed declarations.
    PreparedFrustumFiltered,
    /// Current topology-admission study B: retain original geometry after a
    /// Doom-owned contribution decision. The first slice is deliberately an
    /// all-fail-open identity pass.
    TopologyAdmittedFull,
    /// Current topology-admission study C: apply the existing conservative
    /// AABB/frustum selector only after topology admission.
    TopologyAdmittedFrustum,
    /// Ordered-occurrence Slice 6 fixed-view integration. Doom preparation
    /// replaces the global shell with all ordinary wall and plane declarations
    /// produced for the source-spawn view; no generic camera filter follows.
    OrderedOccurrencePreparedFull,
    /// Doom-private free-look experiment: retain complete original geometry
    /// only for source domains reached by ordered horizontal BSP coverage.
    SourceCoveredGlobalShell,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AppliedRenderStrategy {
    pub(crate) candidate_selection: CandidateSelection,
    pub(crate) ordered_coverage_prepared: bool,
    pub(crate) source_covered_domain_filter: bool,
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
                "a" | "global-full" | "global-full-submission" => {
                    Ok(Self::GlobalFullSubmission)
                }
                "b" | "prepared-full-submission" => Ok(Self::PreparedFullSubmission),
                "c" | "prepared-frustum-filtered" => Ok(Self::PreparedFrustumFiltered),
                "topology-admitted-full" => Ok(Self::TopologyAdmittedFull),
                "topology-admitted-frustum" => Ok(Self::TopologyAdmittedFrustum),
                "ordered-occurrence-prepared-full" => {
                    Ok(Self::OrderedOccurrencePreparedFull)
                }
                "source-covered-global-shell" => Ok(Self::SourceCoveredGlobalShell),
                _ => Err(format!(
                    "unknown render strategy `{name}`; expected global-full, ordered-occurrence-prepared-full, source-covered-global-shell, topology-admitted-full, topology-admitted-frustum, or an explicit historical a/b/c compatibility name"
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
            if compatibility_frustum_flag
                && !matches!(
                    strategy,
                    Self::PreparedFrustumFiltered | Self::TopologyAdmittedFrustum
                )
            {
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
            Self::TopologyAdmittedFull => topology_admitted_full::apply(scene),
            Self::TopologyAdmittedFrustum => topology_admitted_frustum::apply(scene),
            Self::OrderedOccurrencePreparedFull => ordered_occurrence_prepared_full::apply(scene),
            Self::SourceCoveredGlobalShell => source_covered_global_shell::apply(scene),
        }
    }

    pub(crate) const fn resolved_name(self) -> &'static str {
        match self {
            Self::GlobalFullSubmission => "global-full",
            Self::PreparedFullSubmission => "legacy-prepared-full-submission",
            Self::PreparedFrustumFiltered => "legacy-prepared-frustum-filtered",
            Self::TopologyAdmittedFull => "topology-admitted-full",
            Self::TopologyAdmittedFrustum => "topology-admitted-frustum",
            Self::OrderedOccurrencePreparedFull => "ordered-occurrence-prepared-full",
            Self::SourceCoveredGlobalShell => "source-covered-global-shell",
        }
    }

    pub(crate) const fn ordered_stages(self) -> &'static str {
        match self {
            Self::GlobalFullSubmission => "original-complete-geometry>renderer-full-submission",
            Self::PreparedFullSubmission => {
                "legacy-ordered-reconstruction>renderer-full-submission"
            }
            Self::PreparedFrustumFiltered => {
                "legacy-ordered-reconstruction>generic-frustum>renderer"
            }
            Self::TopologyAdmittedFull => {
                "original-contribution-inventory>doom-topology-admission-fail-open>renderer-full-submission"
            }
            Self::TopologyAdmittedFrustum => {
                "original-contribution-inventory>doom-topology-admission-fail-open>generic-frustum>renderer"
            }
            Self::OrderedOccurrencePreparedFull => {
                "source-occurrence-inventory>doom-ordered-wall-plane-preparation>ordinary-declarations>renderer-full-submission"
            }
            Self::SourceCoveredGlobalShell => {
                "original-complete-geometry>doom-ordered-reached-source-domains>exclusive-unvisited-domain-suppression>renderer-full-submission"
            }
        }
    }

    pub(crate) const fn is_ordered_occurrence_integration(self) -> bool {
        matches!(self, Self::OrderedOccurrencePreparedFull)
    }

    pub(crate) const fn is_source_covered_walkabout(self) -> bool {
        matches!(self, Self::SourceCoveredGlobalShell)
    }

    pub(crate) const fn uses_live_doom_preparation(self) -> bool {
        self.is_ordered_occurrence_integration() || self.is_source_covered_walkabout()
    }
}

pub(crate) fn remove_cli_args(args: &mut Vec<String>) {
    args.retain(|argument| !argument.starts_with("--render-strategy="));
}

pub(crate) fn replace_ordered_occurrence_declarations(
    scene: &mut SceneInput,
    prepared: &OrderedPreparedSubmissionObservation,
) -> PlatformResult<()> {
    ordered_occurrence_prepared_full::replace_declarations(scene, prepared)
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
    fn topology_study_names_do_not_alias_historical_prepared_strategies() {
        assert_eq!(
            TrialRenderStrategy::from_args(
                &args(&["--render-strategy=topology-admitted-full"]),
                false,
                false,
            )
            .unwrap(),
            Some(TrialRenderStrategy::TopologyAdmittedFull)
        );
        assert_eq!(
            TrialRenderStrategy::from_args(
                &args(&["--render-strategy=topology-admitted-frustum"]),
                false,
                false,
            )
            .unwrap(),
            Some(TrialRenderStrategy::TopologyAdmittedFrustum)
        );
        assert_ne!(
            TrialRenderStrategy::TopologyAdmittedFull,
            TrialRenderStrategy::PreparedFullSubmission
        );
    }

    #[test]
    fn ordered_occurrence_strategy_is_distinct_from_legacy_and_topology_candidates() {
        assert_eq!(
            TrialRenderStrategy::from_args(
                &args(&["--render-strategy=ordered-occurrence-prepared-full"]),
                false,
                false,
            )
            .unwrap(),
            Some(TrialRenderStrategy::OrderedOccurrencePreparedFull)
        );
        assert_ne!(
            TrialRenderStrategy::OrderedOccurrencePreparedFull,
            TrialRenderStrategy::PreparedFullSubmission
        );
        assert_ne!(
            TrialRenderStrategy::OrderedOccurrencePreparedFull,
            TrialRenderStrategy::TopologyAdmittedFull
        );
        assert!(
            TrialRenderStrategy::OrderedOccurrencePreparedFull.is_ordered_occurrence_integration()
        );
    }

    #[test]
    fn source_covered_walkabout_has_an_explicit_non_aliasing_name() {
        assert_eq!(
            TrialRenderStrategy::from_args(
                &args(&["--render-strategy=source-covered-global-shell"]),
                false,
                false,
            )
            .unwrap(),
            Some(TrialRenderStrategy::SourceCoveredGlobalShell)
        );
        assert!(TrialRenderStrategy::SourceCoveredGlobalShell.uses_live_doom_preparation());
        assert!(!TrialRenderStrategy::SourceCoveredGlobalShell.is_ordered_occurrence_integration());
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
