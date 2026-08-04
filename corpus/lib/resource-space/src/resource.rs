use std::sync::Arc;

use crate::{ContentFingerprint, FolderId, ResourceKey, ResourceMetadata, ResourceName};

/// An immutable byte resource located beneath an explicit folder.
///
/// The key is derived from the current folder hierarchy. Moving or renaming a
/// folder intentionally changes the address portion of descendant keys while
/// preserving their retained byte content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceEntry {
    key: ResourceKey,
    parent: FolderId,
    name: ResourceName,
    bytes: Arc<[u8]>,
    metadata: ResourceMetadata,
}

impl ResourceEntry {
    pub(crate) fn new(
        key: ResourceKey,
        parent: FolderId,
        name: ResourceName,
        bytes: Arc<[u8]>,
        metadata: ResourceMetadata,
    ) -> Self {
        Self {
            key,
            parent,
            name,
            bytes,
            metadata,
        }
    }

    pub fn key(&self) -> &ResourceKey {
        &self.key
    }

    pub const fn parent(&self) -> FolderId {
        self.parent
    }

    pub fn name(&self) -> &ResourceName {
        &self.name
    }

    pub fn bytes(&self) -> &Arc<[u8]> {
        &self.bytes
    }

    pub fn byte_len(&self) -> usize {
        self.bytes.len()
    }

    pub const fn metadata(&self) -> &ResourceMetadata {
        &self.metadata
    }

    /// Returns a named diagnostic fingerprint for this entry's immutable
    /// retained bytes. It does not participate in resource identity.
    pub fn content_fingerprint(&self) -> ContentFingerprint {
        ContentFingerprint::blake3(&self.bytes)
    }

    /// Returns exact byte equality without making equal content imply equal
    /// logical resource identity.
    pub fn has_same_content_as(&self, other: &Self) -> bool {
        self.content_fingerprint() == other.content_fingerprint() && self.bytes == other.bytes
    }
}

#[derive(Debug, Clone)]
pub(crate) struct StoredResource {
    pub parent: FolderId,
    pub name: ResourceName,
    pub bytes: Arc<[u8]>,
    pub metadata: ResourceMetadata,
}
