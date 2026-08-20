//! Provisional provider-neutral command scoping for ADR-0018 conformance.
//!
//! The final public transaction and handle representation remain undecided.
//! This experiment proves that a retained command batch can carry enough
//! resource-set authority to reject before its ordinary handles are resolved
//! against a successor set.

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use thiserror::Error;

use crate::RenderCommand;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ExperimentalRenderResourceSetId(u64);

impl ExperimentalRenderResourceSetId {
    const INITIAL: Self = Self(1);

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Debug)]
pub(crate) struct ExperimentalRenderResourceSetAuthority {
    next_id: AtomicU64,
}

impl ExperimentalRenderResourceSetAuthority {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            next_id: AtomicU64::new(ExperimentalRenderResourceSetId::INITIAL.0 + 1),
        })
    }

    pub(crate) const fn initial_id() -> ExperimentalRenderResourceSetId {
        ExperimentalRenderResourceSetId::INITIAL
    }

    pub(crate) fn allocate_id(
        &self,
    ) -> Result<ExperimentalRenderResourceSetId, ExperimentalRenderCommandSetError> {
        self.next_id
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
                next.checked_add(1)
            })
            .map(ExperimentalRenderResourceSetId)
            .map_err(|_| ExperimentalRenderCommandSetError::IdentityExhausted)
    }
}

/// A provisional command batch scoped to one authoritative render resource set.
///
/// The authority token is intentionally private. Callers can retain and submit
/// the batch, but cannot forge its set membership from a numeric identifier.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct ExperimentalRenderCommandSet {
    authority: Arc<ExperimentalRenderResourceSetAuthority>,
    resource_set: ExperimentalRenderResourceSetId,
    commands: Vec<RenderCommand>,
}

impl ExperimentalRenderCommandSet {
    pub(crate) fn new(
        authority: Arc<ExperimentalRenderResourceSetAuthority>,
        resource_set: ExperimentalRenderResourceSetId,
        commands: &[RenderCommand],
    ) -> Self {
        Self {
            authority,
            resource_set,
            commands: commands.to_vec(),
        }
    }

    pub const fn resource_set(&self) -> ExperimentalRenderResourceSetId {
        self.resource_set
    }

    pub fn commands(&self) -> &[RenderCommand] {
        &self.commands
    }

    pub(crate) fn validate_for(
        &self,
        authority: &Arc<ExperimentalRenderResourceSetAuthority>,
        current: ExperimentalRenderResourceSetId,
    ) -> Result<(), ExperimentalRenderCommandSetError> {
        if !Arc::ptr_eq(&self.authority, authority) {
            return Err(ExperimentalRenderCommandSetError::ForeignAuthority);
        }
        if self.resource_set != current {
            return Err(ExperimentalRenderCommandSetError::StaleResourceSet {
                requested: self.resource_set,
                current,
            });
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ExperimentalRenderCommandSetError {
    #[error("render command set belongs to a different resource-set authority")]
    ForeignAuthority,
    #[error("render command set {requested:?} is retired; current resource set is {current:?}")]
    StaleResourceSet {
        requested: ExperimentalRenderResourceSetId,
        current: ExperimentalRenderResourceSetId,
    },
    #[error("render resource-set identity space is exhausted")]
    IdentityExhausted,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ClearCommand, Color};

    fn command() -> RenderCommand {
        RenderCommand::Clear(ClearCommand {
            color: Color::BLACK,
        })
    }

    #[test]
    fn retained_commands_reject_after_authority_advances_to_a_successor() {
        let authority = ExperimentalRenderResourceSetAuthority::new();
        let set_a = ExperimentalRenderResourceSetAuthority::initial_id();
        let retained_a =
            ExperimentalRenderCommandSet::new(Arc::clone(&authority), set_a, &[command()]);
        let set_b = authority.allocate_id().expect("successor identity");

        assert_eq!(
            retained_a.validate_for(&authority, set_b),
            Err(ExperimentalRenderCommandSetError::StaleResourceSet {
                requested: set_a,
                current: set_b,
            })
        );
    }

    #[test]
    fn successor_commands_with_reused_local_handles_remain_current() {
        let authority = ExperimentalRenderResourceSetAuthority::new();
        let set_b = authority.allocate_id().expect("successor identity");
        let current_b =
            ExperimentalRenderCommandSet::new(Arc::clone(&authority), set_b, &[command()]);

        assert_eq!(current_b.validate_for(&authority, set_b), Ok(()));
        assert_eq!(current_b.commands(), &[command()]);
    }

    #[test]
    fn equal_numeric_set_ids_from_another_authority_do_not_alias() {
        let authority_a = ExperimentalRenderResourceSetAuthority::new();
        let authority_b = ExperimentalRenderResourceSetAuthority::new();
        let set = ExperimentalRenderResourceSetAuthority::initial_id();
        let foreign = ExperimentalRenderCommandSet::new(authority_a, set, &[command()]);

        assert_eq!(
            foreign.validate_for(&authority_b, set),
            Err(ExperimentalRenderCommandSetError::ForeignAuthority)
        );
    }
}
