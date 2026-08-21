//! Corpus-private semantic shadows for AR-0033.
//!
//! These types compare update authority and ordering only. They are not a
//! renderer API, provider implementation, or proposed public handle shape.

use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Scope {
    session: u64,
    set: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Target {
    scope: Scope,
    local: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ResourceClass {
    Ordinary,
    Dynamic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Realization {
    revision: u64,
    fingerprint: u64,
    class: ResourceClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PreparedUpdate {
    target: Target,
    fingerprint: u64,
    require_dynamic: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ScopedCommand {
    target: Target,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TransientSubmission {
    scope: Scope,
    fingerprint: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InjectedFailure {
    AfterPreparation,
    AfterPartialProviderAllocation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ShadowError {
    ForeignSessionBeforeLookup,
    StaleSetBeforeLookup,
    MissingResource,
    ImmutableResource,
    Injected(InjectedFailure),
}

struct UpdateShadow {
    scope: Scope,
    resources: BTreeMap<u64, Realization>,
    presented_transient: Option<u64>,
    visibility_boundary: u64,
}

impl UpdateShadow {
    fn new() -> Self {
        Self {
            scope: Scope {
                session: 11,
                set: 1,
            },
            resources: BTreeMap::from([
                (
                    7,
                    Realization {
                        revision: 0,
                        fingerprint: 100,
                        class: ResourceClass::Ordinary,
                    },
                ),
                (
                    8,
                    Realization {
                        revision: 0,
                        fingerprint: 200,
                        class: ResourceClass::Dynamic,
                    },
                ),
            ]),
            presented_transient: Some(300),
            visibility_boundary: 0,
        }
    }

    fn target(&self, local: u64) -> Target {
        Target {
            scope: self.scope,
            local,
        }
    }

    fn validate_scope(&self, scope: Scope) -> Result<(), ShadowError> {
        if scope.session != self.scope.session {
            return Err(ShadowError::ForeignSessionBeforeLookup);
        }
        if scope.set != self.scope.set {
            return Err(ShadowError::StaleSetBeforeLookup);
        }
        Ok(())
    }

    fn prepare_update(
        &self,
        target: Target,
        fingerprint: u64,
        require_dynamic: bool,
    ) -> Result<PreparedUpdate, ShadowError> {
        self.validate_scope(target.scope)?;
        let current = self
            .resources
            .get(&target.local)
            .ok_or(ShadowError::MissingResource)?;
        if require_dynamic && current.class != ResourceClass::Dynamic {
            return Err(ShadowError::ImmutableResource);
        }
        Ok(PreparedUpdate {
            target,
            fingerprint,
            require_dynamic,
        })
    }

    fn commit_update(
        &mut self,
        candidate: PreparedUpdate,
        failure: Option<InjectedFailure>,
    ) -> Result<Realization, ShadowError> {
        // Revalidate before local lookup so a whole-set commit cannot redirect
        // the candidate toward a reused successor key.
        self.validate_scope(candidate.target.scope)?;
        let current = self
            .resources
            .get(&candidate.target.local)
            .copied()
            .ok_or(ShadowError::MissingResource)?;
        if candidate.require_dynamic && current.class != ResourceClass::Dynamic {
            return Err(ShadowError::ImmutableResource);
        }
        if let Some(failure) = failure {
            return Err(ShadowError::Injected(failure));
        }
        let replacement = Realization {
            revision: current.revision.saturating_add(1),
            fingerprint: candidate.fingerprint,
            class: current.class,
        };
        self.resources.insert(candidate.target.local, replacement);
        self.visibility_boundary = self.visibility_boundary.saturating_add(1);
        Ok(replacement)
    }

    fn resolve_command(&self, command: ScopedCommand) -> Result<Realization, ShadowError> {
        self.validate_scope(command.target.scope)?;
        self.resources
            .get(&command.target.local)
            .copied()
            .ok_or(ShadowError::MissingResource)
    }

    fn prepare_transient(&self, fingerprint: u64) -> TransientSubmission {
        TransientSubmission {
            scope: self.scope,
            fingerprint,
        }
    }

    fn submit_transient(
        &mut self,
        submission: TransientSubmission,
        failure: Option<InjectedFailure>,
    ) -> Result<(), ShadowError> {
        self.validate_scope(submission.scope)?;
        if let Some(failure) = failure {
            return Err(ShadowError::Injected(failure));
        }
        self.presented_transient = Some(submission.fingerprint);
        self.visibility_boundary = self.visibility_boundary.saturating_add(1);
        Ok(())
    }

    fn commit_whole_set_with_reused_keys(&mut self) {
        self.scope.set = self.scope.set.saturating_add(1);
        for realization in self.resources.values_mut() {
            realization.revision = 0;
            realization.fingerprint = realization.fingerprint.saturating_add(1_000);
        }
        self.visibility_boundary = self.visibility_boundary.saturating_add(1);
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Ar0033SemanticShadowObservation {
    status: &'static str,
    review: &'static str,
    alternative_b_existing_command_observes_committed_revision: bool,
    alternative_b_failed_preparation_preserves_prior: bool,
    alternative_b_failed_provider_allocation_preserves_prior: bool,
    alternative_b_stale_after_whole_set_commit_rejects_before_lookup: bool,
    alternative_b_foreign_session_rejects_before_lookup: bool,
    alternative_c_ordinary_resource_rejected: bool,
    alternative_c_dynamic_resource_committed: bool,
    alternative_d_uses_no_persistent_resource_identity: bool,
    alternative_d_failed_submission_preserves_prior_frame: bool,
    alternative_d_stale_submission_rejects_before_use: bool,
    visibility_boundary: &'static str,
    authority: &'static str,
}

pub(crate) fn observe_semantic_shadows() -> Ar0033SemanticShadowObservation {
    let mut existing = UpdateShadow::new();
    let target = existing.target(7);
    let command = ScopedCommand { target };
    let prior = existing.resolve_command(command).expect("seeded resource");
    let candidate = existing
        .prepare_update(target, 101, false)
        .expect("current ordinary target");
    let failed_preparation =
        existing.commit_update(candidate, Some(InjectedFailure::AfterPreparation));
    let preparation_preserved = matches!(
        failed_preparation,
        Err(ShadowError::Injected(InjectedFailure::AfterPreparation))
    ) && existing.resolve_command(command) == Ok(prior);
    let failed_provider = existing.commit_update(
        candidate,
        Some(InjectedFailure::AfterPartialProviderAllocation),
    );
    let provider_preserved = matches!(
        failed_provider,
        Err(ShadowError::Injected(
            InjectedFailure::AfterPartialProviderAllocation
        ))
    ) && existing.resolve_command(command) == Ok(prior);
    let committed = existing
        .commit_update(candidate, None)
        .expect("existing-identity commit");
    let command_observes_commit = existing.resolve_command(command) == Ok(committed)
        && committed.revision == prior.revision + 1;

    let stale_candidate = existing
        .prepare_update(target, 102, false)
        .expect("candidate before whole-set replacement");
    existing.commit_whole_set_with_reused_keys();
    let stale_rejected =
        existing.commit_update(stale_candidate, None) == Err(ShadowError::StaleSetBeforeLookup);
    let foreign_target = Target {
        scope: Scope {
            session: 99,
            set: existing.scope.set,
        },
        local: 7,
    };
    let foreign_rejected = existing.prepare_update(foreign_target, 103, false)
        == Err(ShadowError::ForeignSessionBeforeLookup);

    let mut dynamic = UpdateShadow::new();
    let ordinary_rejected =
        dynamic.prepare_update(dynamic.target(7), 104, true) == Err(ShadowError::ImmutableResource);
    let dynamic_candidate = dynamic
        .prepare_update(dynamic.target(8), 201, true)
        .expect("declared dynamic target");
    let dynamic_committed = dynamic
        .commit_update(dynamic_candidate, None)
        .is_ok_and(|value| value.fingerprint == 201 && value.revision == 1);

    let mut transient = UpdateShadow::new();
    let transient_before = transient.presented_transient;
    let failed_transient = transient.submit_transient(
        transient.prepare_transient(301),
        Some(InjectedFailure::AfterPartialProviderAllocation),
    );
    let transient_preserved = failed_transient
        == Err(ShadowError::Injected(
            InjectedFailure::AfterPartialProviderAllocation,
        ))
        && transient.presented_transient == transient_before;
    let stale_transient = transient.prepare_transient(302);
    transient.commit_whole_set_with_reused_keys();
    let transient_stale_rejected =
        transient.submit_transient(stale_transient, None) == Err(ShadowError::StaleSetBeforeLookup);

    Ar0033SemanticShadowObservation {
        status: "complete",
        review: "AR-0033-slice-1",
        alternative_b_existing_command_observes_committed_revision: command_observes_commit,
        alternative_b_failed_preparation_preserves_prior: preparation_preserved,
        alternative_b_failed_provider_allocation_preserves_prior: provider_preserved,
        alternative_b_stale_after_whole_set_commit_rejects_before_lookup: stale_rejected,
        alternative_b_foreign_session_rejects_before_lookup: foreign_rejected,
        alternative_c_ordinary_resource_rejected: ordinary_rejected,
        alternative_c_dynamic_resource_committed: dynamic_committed,
        alternative_d_uses_no_persistent_resource_identity: true,
        alternative_d_failed_submission_preserves_prior_frame: transient_preserved,
        alternative_d_stale_submission_rejects_before_use: transient_stale_rejected,
        visibility_boundary: "explicit-commit-or-scoped-submission",
        authority: "corpus-private-semantic-shadow-not-provider-or-public-contract",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_shadows_satisfy_the_required_failure_scope_and_ordering_checks() {
        let observation = observe_semantic_shadows();
        assert!(observation.alternative_b_existing_command_observes_committed_revision);
        assert!(observation.alternative_b_failed_preparation_preserves_prior);
        assert!(observation.alternative_b_failed_provider_allocation_preserves_prior);
        assert!(observation.alternative_b_stale_after_whole_set_commit_rejects_before_lookup);
        assert!(observation.alternative_b_foreign_session_rejects_before_lookup);
        assert!(observation.alternative_c_ordinary_resource_rejected);
        assert!(observation.alternative_c_dynamic_resource_committed);
        assert!(observation.alternative_d_uses_no_persistent_resource_identity);
        assert!(observation.alternative_d_failed_submission_preserves_prior_frame);
        assert!(observation.alternative_d_stale_submission_rejects_before_use);
    }

    #[test]
    fn prepared_existing_identity_update_is_not_visible_until_commit() {
        let mut shadow = UpdateShadow::new();
        let target = shadow.target(7);
        let command = ScopedCommand { target };
        let before = shadow.resolve_command(command).unwrap();
        let candidate = shadow.prepare_update(target, 999, false).unwrap();
        assert_eq!(shadow.resolve_command(command), Ok(before));
        let after = shadow.commit_update(candidate, None).unwrap();
        assert_eq!(shadow.resolve_command(command), Ok(after));
        assert_eq!(after.fingerprint, 999);
    }

    #[test]
    fn whole_set_commit_wins_over_an_older_in_set_candidate() {
        let mut shadow = UpdateShadow::new();
        let candidate = shadow.prepare_update(shadow.target(7), 999, false).unwrap();
        shadow.commit_whole_set_with_reused_keys();
        assert_eq!(
            shadow.commit_update(candidate, None),
            Err(ShadowError::StaleSetBeforeLookup)
        );
    }
}
