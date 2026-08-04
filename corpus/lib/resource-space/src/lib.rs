//! Incubating logical resource-space semantics.
//!
//! The crate owns provider-neutral identity and addressing. It does not own
//! filesystem behavior, persistence, format parsing, or asset lifecycle.

mod address;
mod content;
mod identity;
mod limits;
mod metadata;
mod mutation;
mod query;
mod registry;
mod resource;
mod space;
mod summary;

pub use address::{AddressCasePolicy, ResourceAddress, ResourceAddressError, ResourceName};
pub use content::{ContentFingerprint, ContentFingerprintAlgorithm};
pub use identity::{
    FolderId, ResourceKey, ResourceRootDescriptor, ResourceRootId, ResourceStoreDescriptor,
    ResourceStoreOrigin, ResourceStoreProvenance, StoreId,
};
pub use limits::ResourceSpaceLimits;
pub use metadata::{ResourceMetadata, ResourceTimestamp, ResourceVisibility, VisibilityQuery};
pub use mutation::{ResourceMutationObservation, ResourceMutationOutcome};
pub use query::ResourceSearchQuery;
pub use registry::{InMemoryResourceSpaceRegistry, ResourceStoreRegistryError, StoreOpenOutcome};
pub use resource::ResourceEntry;
pub use space::{FolderEntry, InMemoryResourceSpace, ResourceSpaceError};
pub use summary::ResourceSpaceSummary;

#[cfg(test)]
mod tests;
