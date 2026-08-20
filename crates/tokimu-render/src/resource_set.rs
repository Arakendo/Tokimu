//! Provider-neutral render resource-set replacement semantics.
//!
//! ADR-0018 admits atomic staged replacement without prescribing provider
//! allocation, synchronization, reclamation, or individual handle encoding.

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use thiserror::Error;

use crate::{RenderCommand, Renderer};

/// Opaque identity of one authoritative render resource set.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RenderResourceSetId(u64);

impl RenderResourceSetId {
    const INITIAL: Self = Self(1);

    /// Returns a diagnostic value. Callers cannot construct identities from it.
    pub const fn diagnostic_value(self) -> u64 {
        self.0
    }
}

#[derive(Debug)]
pub(crate) struct RenderResourceSetAuthority {
    next_id: AtomicU64,
}

impl RenderResourceSetAuthority {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            next_id: AtomicU64::new(RenderResourceSetId::INITIAL.0 + 1),
        })
    }

    pub(crate) const fn initial_id() -> RenderResourceSetId {
        RenderResourceSetId::INITIAL
    }

    pub(crate) fn allocate_id(&self) -> Result<RenderResourceSetId, RenderCommandSetError> {
        self.next_id
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
                next.checked_add(1)
            })
            .map(RenderResourceSetId)
            .map_err(|_| RenderCommandSetError::IdentityExhausted)
    }
}

/// Ordinary render commands scoped to the resource set that authored them.
///
/// The private authority token prevents a batch from another renderer session
/// from aliasing the current set even if both expose the same diagnostic ID.
#[derive(Clone, Debug)]
pub struct RenderCommandSet {
    authority: Arc<RenderResourceSetAuthority>,
    resource_set: RenderResourceSetId,
    commands: Vec<RenderCommand>,
}

impl RenderCommandSet {
    pub(crate) fn new(
        authority: Arc<RenderResourceSetAuthority>,
        resource_set: RenderResourceSetId,
        commands: &[RenderCommand],
    ) -> Self {
        Self {
            authority,
            resource_set,
            commands: commands.to_vec(),
        }
    }

    pub const fn resource_set(&self) -> RenderResourceSetId {
        self.resource_set
    }

    pub fn commands(&self) -> &[RenderCommand] {
        &self.commands
    }

    pub(crate) fn validate_for(
        &self,
        authority: &Arc<RenderResourceSetAuthority>,
        current: RenderResourceSetId,
    ) -> Result<(), RenderCommandSetError> {
        if !Arc::ptr_eq(&self.authority, authority) {
            return Err(RenderCommandSetError::ForeignAuthority);
        }
        if self.resource_set != current {
            return Err(RenderCommandSetError::StaleResourceSet {
                requested: self.resource_set,
                current,
            });
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum RenderCommandSetError {
    #[error("render command set belongs to a different resource-set authority")]
    ForeignAuthority,
    #[error("render command set {requested:?} is retired; current resource set is {current:?}")]
    StaleResourceSet {
        requested: RenderResourceSetId,
        current: RenderResourceSetId,
    },
    #[error("render resource-set identity space is exhausted")]
    IdentityExhausted,
}

/// Provider-neutral lifecycle for atomic staged replacement.
///
/// The associated candidate remains provider-owned. Resource upload and
/// validation mechanics therefore do not leak into this contract. Dropping a
/// candidate before commit must leave the current set authoritative.
pub trait RenderResourceSetLifecycle: Renderer {
    type Candidate;
    type Error;
    type CommitObservation;

    fn begin_resource_set_stage(&self) -> Result<Self::Candidate, Self::Error>;

    fn commit_resource_set_stage(
        &mut self,
        candidate: Self::Candidate,
    ) -> Result<Self::CommitObservation, Self::Error>;

    fn scope_render_commands(&self, commands: &[RenderCommand]) -> RenderCommandSet;

    fn submit_render_command_set(
        &mut self,
        command_set: &RenderCommandSet,
    ) -> Result<(), Self::Error>;

    /// Stages, populates, and commits one candidate without exposing partial
    /// replacement. A population or commit error leaves the current set as the
    /// only authoritative set.
    fn replace_resource_set<F>(
        &mut self,
        populate: F,
    ) -> Result<Self::CommitObservation, Self::Error>
    where
        F: FnOnce(&mut Self::Candidate) -> Result<(), Self::Error>,
    {
        let mut candidate = self.begin_resource_set_stage()?;
        populate(&mut candidate)?;
        self.commit_resource_set_stage(candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ClearCommand, Color, RenderStats};

    struct MockCandidate {
        id: RenderResourceSetId,
    }

    struct MockRenderer {
        authority: Arc<RenderResourceSetAuthority>,
        current: RenderResourceSetId,
        submitted: usize,
    }

    impl MockRenderer {
        fn new() -> Self {
            Self {
                authority: RenderResourceSetAuthority::new(),
                current: RenderResourceSetAuthority::initial_id(),
                submitted: 0,
            }
        }
    }

    impl Renderer for MockRenderer {
        fn name(&self) -> &'static str {
            "resource-set-lifecycle-mock"
        }

        fn clear_color(&self) -> Color {
            Color::BLACK
        }

        fn begin_frame(&mut self) {}

        fn submit(&mut self, commands: &[RenderCommand]) {
            self.submitted += commands.len();
        }

        fn end_frame(&mut self) -> RenderStats {
            RenderStats::default()
        }
    }

    impl RenderResourceSetLifecycle for MockRenderer {
        type Candidate = MockCandidate;
        type Error = RenderCommandSetError;
        type CommitObservation = RenderResourceSetId;

        fn begin_resource_set_stage(&self) -> Result<Self::Candidate, Self::Error> {
            Ok(MockCandidate {
                id: self.authority.allocate_id()?,
            })
        }

        fn commit_resource_set_stage(
            &mut self,
            candidate: Self::Candidate,
        ) -> Result<Self::CommitObservation, Self::Error> {
            self.current = candidate.id;
            Ok(self.current)
        }

        fn scope_render_commands(&self, commands: &[RenderCommand]) -> RenderCommandSet {
            RenderCommandSet::new(Arc::clone(&self.authority), self.current, commands)
        }

        fn submit_render_command_set(
            &mut self,
            command_set: &RenderCommandSet,
        ) -> Result<(), Self::Error> {
            command_set.validate_for(&self.authority, self.current)?;
            self.submit(command_set.commands());
            Ok(())
        }
    }

    fn command() -> RenderCommand {
        RenderCommand::Clear(ClearCommand {
            color: Color::BLACK,
        })
    }

    #[test]
    fn retained_commands_reject_after_authority_advances_to_a_successor() {
        let authority = RenderResourceSetAuthority::new();
        let set_a = RenderResourceSetAuthority::initial_id();
        let retained_a = RenderCommandSet::new(Arc::clone(&authority), set_a, &[command()]);
        let set_b = authority.allocate_id().expect("successor identity");

        assert_eq!(
            retained_a.validate_for(&authority, set_b),
            Err(RenderCommandSetError::StaleResourceSet {
                requested: set_a,
                current: set_b,
            })
        );
    }

    #[test]
    fn successor_commands_with_reused_local_handles_remain_current() {
        let authority = RenderResourceSetAuthority::new();
        let set_b = authority.allocate_id().expect("successor identity");
        let current_b = RenderCommandSet::new(Arc::clone(&authority), set_b, &[command()]);

        assert_eq!(current_b.validate_for(&authority, set_b), Ok(()));
        assert_eq!(current_b.commands(), &[command()]);
    }

    #[test]
    fn equal_numeric_set_ids_from_another_authority_do_not_alias() {
        let authority_a = RenderResourceSetAuthority::new();
        let authority_b = RenderResourceSetAuthority::new();
        let set = RenderResourceSetAuthority::initial_id();
        let foreign = RenderCommandSet::new(authority_a, set, &[command()]);

        assert_eq!(
            foreign.validate_for(&authority_b, set),
            Err(RenderCommandSetError::ForeignAuthority)
        );
    }

    #[test]
    fn lifecycle_default_replacement_preserves_current_on_failure_then_commits_atomically() {
        let mut renderer = MockRenderer::new();
        let retained_a = renderer.scope_render_commands(&[command()]);

        let failed =
            renderer.replace_resource_set(|_| Err(RenderCommandSetError::IdentityExhausted));
        assert_eq!(failed, Err(RenderCommandSetError::IdentityExhausted));
        assert_eq!(renderer.current, retained_a.resource_set());
        assert_eq!(renderer.submit_render_command_set(&retained_a), Ok(()));

        let committed_b = renderer
            .replace_resource_set(|_| Ok(()))
            .expect("complete candidate commits");
        assert_eq!(renderer.current, committed_b);
        assert_eq!(
            renderer.submit_render_command_set(&retained_a),
            Err(RenderCommandSetError::StaleResourceSet {
                requested: retained_a.resource_set(),
                current: committed_b,
            })
        );
    }
}
