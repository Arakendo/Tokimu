use serde::{Deserialize, Serialize};

use crate::ResourceAddress;

macro_rules! stable_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        pub struct $name(u128);

        impl $name {
            pub const fn from_u128(value: u128) -> Self {
                Self(value)
            }

            pub const fn as_u128(self) -> u128 {
                self.0
            }
        }
    };
}

stable_id!(StoreId);
stable_id!(ResourceRootId);
stable_id!(FolderId);

/// Provider-neutral classification of how an application obtained a store.
///
/// This is diagnostic metadata, not an access mechanism. In particular, it
/// does not expose host paths, browser handles, URLs, or provider state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceStoreOrigin {
    /// The application did not retain an origin classification.
    #[default]
    Unspecified,
    /// The application generated the store's content.
    Generated,
    /// The application imported content across an external boundary.
    Imported,
    /// The store was created from a known test or corpus fixture.
    Fixture,
}

/// Advisory provenance attached to a logical store.
///
/// The optional label is deliberately non-unique and never participates in
/// store identity, create/open behavior, or resource addressing.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceStoreProvenance {
    origin: ResourceStoreOrigin,
    label: Option<String>,
}

impl ResourceStoreProvenance {
    pub const fn new(origin: ResourceStoreOrigin) -> Self {
        Self {
            origin,
            label: None,
        }
    }

    pub fn with_label(origin: ResourceStoreOrigin, label: impl Into<String>) -> Self {
        Self {
            origin,
            label: Some(label.into()),
        }
    }

    pub const fn origin(&self) -> ResourceStoreOrigin {
        self.origin
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
}

/// Stable logical-store identity plus an editable display name.
///
/// The display name is intentionally not unique and never participates in
/// resource identity or create/open conflict detection. Provenance is
/// similarly advisory so a host-specific acquisition mechanism never leaks
/// into the provider-neutral store contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceStoreDescriptor {
    id: StoreId,
    display_name: String,
    provenance: ResourceStoreProvenance,
}

impl ResourceStoreDescriptor {
    pub fn new(id: StoreId, display_name: impl Into<String>) -> Self {
        Self {
            id,
            display_name: display_name.into(),
            provenance: ResourceStoreProvenance::default(),
        }
    }

    pub fn with_provenance(mut self, provenance: ResourceStoreProvenance) -> Self {
        self.provenance = provenance;
        self
    }

    pub const fn id(&self) -> StoreId {
        self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub const fn provenance(&self) -> &ResourceStoreProvenance {
        &self.provenance
    }

    pub fn rename(&mut self, display_name: impl Into<String>) {
        self.display_name = display_name.into();
    }
}

/// Human-facing root information kept separate from stable root identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceRootDescriptor {
    id: ResourceRootId,
    display_name: String,
}

impl ResourceRootDescriptor {
    pub fn new(id: ResourceRootId, display_name: impl Into<String>) -> Self {
        Self {
            id,
            display_name: display_name.into(),
        }
    }

    pub const fn id(&self) -> ResourceRootId {
        self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn rename(&mut self, display_name: impl Into<String>) {
        self.display_name = display_name.into();
    }
}

/// Fully qualifies a logical address without treating content equality as
/// resource identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ResourceKey {
    store: StoreId,
    root: ResourceRootId,
    address: ResourceAddress,
}

impl ResourceKey {
    pub fn new(store: StoreId, root: ResourceRootId, address: ResourceAddress) -> Self {
        Self {
            store,
            root,
            address,
        }
    }

    pub const fn store(&self) -> StoreId {
        self.store
    }

    pub const fn root(&self) -> ResourceRootId {
        self.root
    }

    pub const fn address(&self) -> &ResourceAddress {
        &self.address
    }
}
