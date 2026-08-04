use serde::{Deserialize, Serialize};

use crate::{FolderId, ResourceKey, ResourceRootId};

/// Structured result of a successful logical resource-space mutation.
///
/// Outcomes describe provider-neutral identities and byte counts only. They do
/// not expose backing collections, filesystem paths, persistence transactions,
/// or provider-specific handles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceMutationOutcome {
    RootCreated {
        root: ResourceRootId,
        root_folder: FolderId,
    },
    RootRenamed {
        root: ResourceRootId,
    },
    RootRemoved {
        root: ResourceRootId,
        root_folder: FolderId,
    },
    FolderCreated {
        folder: FolderId,
        parent: FolderId,
    },
    FolderRenamed {
        folder: FolderId,
    },
    FolderMoved {
        folder: FolderId,
        source_parent: FolderId,
        destination_parent: FolderId,
    },
    FolderRemoved {
        folder: FolderId,
        parent: FolderId,
    },
    ResourceInserted {
        key: ResourceKey,
        byte_len: usize,
    },
    ResourceReplaced {
        key: ResourceKey,
        previous_byte_len: usize,
        byte_len: usize,
    },
    ResourceRemoved {
        key: ResourceKey,
        byte_len: usize,
    },
    ResourceCopied {
        source: ResourceKey,
        destination: ResourceKey,
        byte_len: usize,
    },
    ResourceMoved {
        source: ResourceKey,
        destination: ResourceKey,
        byte_len: usize,
    },
}

/// One locally ordered diagnostic observation retained by a resource space.
///
/// Sequence numbers start at one whenever observation is enabled. They order
/// the retained observations from that capture session; they are not durable
/// revisions and do not participate in resource identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceMutationObservation {
    sequence: u64,
    outcome: ResourceMutationOutcome,
}

impl ResourceMutationObservation {
    pub(crate) const fn new(sequence: u64, outcome: ResourceMutationOutcome) -> Self {
        Self { sequence, outcome }
    }

    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    pub const fn outcome(&self) -> &ResourceMutationOutcome {
        &self.outcome
    }
}
