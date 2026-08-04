use std::collections::BTreeMap;

use thiserror::Error;

use crate::{AddressCasePolicy, InMemoryResourceSpace, ResourceStoreDescriptor, StoreId};

/// A bounded owner of independently addressable in-memory resource spaces.
///
/// The registry decides whether a stable store identity is created or opened.
/// It does not infer identity from a display name or from resource content.
#[derive(Debug, Default)]
pub struct InMemoryResourceSpaceRegistry {
    stores: BTreeMap<StoreId, RegisteredStore>,
}

impl InMemoryResourceSpaceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_new(
        &mut self,
        descriptor: ResourceStoreDescriptor,
        case_policy: AddressCasePolicy,
    ) -> Result<(), ResourceStoreRegistryError> {
        let store = descriptor.id();
        if self.stores.contains_key(&store) {
            return Err(ResourceStoreRegistryError::StoreAlreadyExists { store });
        }
        self.stores.insert(
            store,
            RegisteredStore {
                descriptor,
                space: InMemoryResourceSpace::new(store, case_policy),
            },
        );
        Ok(())
    }

    pub fn create_or_open(
        &mut self,
        descriptor: ResourceStoreDescriptor,
        case_policy: AddressCasePolicy,
    ) -> Result<StoreOpenOutcome, ResourceStoreRegistryError> {
        let store = descriptor.id();
        match self.stores.get(&store) {
            Some(existing) if existing.space.case_policy() != case_policy => {
                Err(ResourceStoreRegistryError::StorePolicyMismatch {
                    store,
                    existing: existing.space.case_policy(),
                    requested: case_policy,
                })
            }
            Some(_) => Ok(StoreOpenOutcome::OpenedExisting { store }),
            None => {
                self.create_new(descriptor, case_policy)?;
                Ok(StoreOpenOutcome::Created { store })
            }
        }
    }

    pub fn descriptor(
        &self,
        store: StoreId,
    ) -> Result<&ResourceStoreDescriptor, ResourceStoreRegistryError> {
        Ok(&self
            .stores
            .get(&store)
            .ok_or(ResourceStoreRegistryError::StoreNotFound { store })?
            .descriptor)
    }

    pub fn space(
        &self,
        store: StoreId,
    ) -> Result<&InMemoryResourceSpace, ResourceStoreRegistryError> {
        Ok(&self
            .stores
            .get(&store)
            .ok_or(ResourceStoreRegistryError::StoreNotFound { store })?
            .space)
    }

    pub fn space_mut(
        &mut self,
        store: StoreId,
    ) -> Result<&mut InMemoryResourceSpace, ResourceStoreRegistryError> {
        Ok(&mut self
            .stores
            .get_mut(&store)
            .ok_or(ResourceStoreRegistryError::StoreNotFound { store })?
            .space)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreOpenOutcome {
    Created { store: StoreId },
    OpenedExisting { store: StoreId },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ResourceStoreRegistryError {
    #[error("resource store {store:?} already exists")]
    StoreAlreadyExists { store: StoreId },
    #[error("resource store {store:?} does not exist")]
    StoreNotFound { store: StoreId },
    #[error(
        "resource store {store:?} already exists with case policy {existing:?}, not {requested:?}"
    )]
    StorePolicyMismatch {
        store: StoreId,
        existing: AddressCasePolicy,
        requested: AddressCasePolicy,
    },
}

#[derive(Debug)]
struct RegisteredStore {
    descriptor: ResourceStoreDescriptor,
    space: InMemoryResourceSpace,
}
