//! Consumer-local durable representation of Resource Space state.
//!
//! Tosumu stores opaque, versioned logical values. This module owns the
//! mapping between those values and Resource Space's public hierarchy; neither
//! the bridge envelope nor Tosumu's public API exposes the other's internals.

use std::path::Path;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use resource_space::{
    AddressCasePolicy, FolderId, InMemoryResourceSpaceRegistry, ResourceMetadata,
    ResourceRootDescriptor, ResourceRootId, ResourceStoreDescriptor, StoreId, VisibilityQuery,
};
use serde::{Deserialize, Serialize};
use tosumu_core::KvStore;

const SNAPSHOT_KEY: &[u8] = b"tokimu.resource-space.snapshot.v1";
const SNAPSHOT_SCHEMA: u32 = 1;

pub(super) struct TosumuProvider {
    store: KvStore,
}

impl TosumuProvider {
    pub(super) fn open(path: &Path) -> Result<(Self, Option<PersistedResourceSpace>), String> {
        let store = if path.exists() {
            KvStore::open(path)
        } else {
            KvStore::create(path)
        }
        .map_err(|error| format!("could not open Tosumu store: {error}"))?;
        let provider = Self { store };
        let snapshot = provider
            .store
            .get(SNAPSHOT_KEY)
            .map_err(|error| format!("could not read Tosumu resource snapshot: {error}"))?
            .map(|bytes| {
                serde_json::from_slice(&bytes)
                    .map_err(|error| format!("Tosumu resource snapshot is invalid: {error}"))
            })
            .transpose()?;
        Ok((provider, snapshot))
    }

    pub(super) fn save(&mut self, snapshot: &PersistedResourceSpace) -> Result<(), String> {
        let bytes = serde_json::to_vec(snapshot)
            .map_err(|error| format!("could not serialize Resource Space snapshot: {error}"))?;
        self.store
            .transaction(|transaction| transaction.put(SNAPSHOT_KEY, &bytes))
            .map_err(|error| format!("could not commit Tosumu resource snapshot: {error}"))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct PersistedResourceSpace {
    schema: u32,
    descriptor: ResourceStoreDescriptor,
    case_policy: AddressCasePolicy,
    root: ResourceRootDescriptor,
    root_folder_id: String,
    root_metadata: ResourceMetadata,
    next_folder_id: String,
    folders: Vec<PersistedFolder>,
    resources: Vec<PersistedResource>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedFolder {
    id: String,
    parent_id: String,
    name: String,
    metadata: ResourceMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedResource {
    parent_id: String,
    name: String,
    bytes_base64: String,
    metadata: ResourceMetadata,
}

impl PersistedResourceSpace {
    pub(super) fn capture(
        registry: &InMemoryResourceSpaceRegistry,
        store: StoreId,
        root: ResourceRootId,
        root_folder: FolderId,
        next_folder: u128,
    ) -> Result<Self, String> {
        let space = registry.space(store).map_err(|error| error.to_string())?;
        let descriptor = registry
            .descriptor(store)
            .map_err(|error| error.to_string())?
            .clone();
        let root_descriptor = space
            .root(root)
            .ok_or_else(|| "Resource Space root is missing during persistence".to_owned())?
            .clone();
        let root_metadata = space
            .folder(root_folder)
            .ok_or_else(|| "Resource Space root folder is missing during persistence".to_owned())?
            .metadata()
            .clone();

        let mut folders = Vec::new();
        let mut resources = Vec::new();
        let mut pending = vec![root_folder];
        while let Some(parent) = pending.pop() {
            for folder in space
                .list_folders(parent, VisibilityQuery::All)
                .map_err(|error| error.to_string())?
            {
                let id = folder.id();
                folders.push(PersistedFolder {
                    id: id.as_u128().to_string(),
                    parent_id: parent.as_u128().to_string(),
                    name: folder
                        .name()
                        .expect("non-root folders have names")
                        .as_str()
                        .to_owned(),
                    metadata: folder.metadata().clone(),
                });
                pending.push(id);
            }
            for resource in space
                .list_resources(parent, VisibilityQuery::All)
                .map_err(|error| error.to_string())?
            {
                resources.push(PersistedResource {
                    parent_id: parent.as_u128().to_string(),
                    name: resource.name().as_str().to_owned(),
                    bytes_base64: STANDARD.encode(resource.bytes()),
                    metadata: resource.metadata().clone(),
                });
            }
        }

        Ok(Self {
            schema: SNAPSHOT_SCHEMA,
            descriptor,
            case_policy: space.case_policy(),
            root: root_descriptor,
            root_folder_id: root_folder.as_u128().to_string(),
            root_metadata,
            next_folder_id: next_folder.to_string(),
            folders,
            resources,
        })
    }

    pub(super) fn restore(
        self,
        registry: &mut InMemoryResourceSpaceRegistry,
    ) -> Result<RestoredSession, String> {
        if self.schema != SNAPSHOT_SCHEMA {
            return Err(format!(
                "unsupported Tosumu Resource Space snapshot schema {}",
                self.schema
            ));
        }
        let store = self.descriptor.id();
        let root = self.root.id();
        let root_folder = parse_id(&self.root_folder_id, "root_folder_id")?;
        let next_folder = parse_u128(&self.next_folder_id, "next_folder_id")?;
        registry
            .create_new(self.descriptor, self.case_policy)
            .map_err(|error| error.to_string())?;
        let space = registry
            .space_mut(store)
            .map_err(|error| error.to_string())?;
        space
            .create_root(self.root, root_folder, self.root_metadata)
            .map_err(|error| error.to_string())?;
        for folder in self.folders {
            let id = parse_id(&folder.id, "folder.id")?;
            let parent = parse_id(&folder.parent_id, "folder.parent_id")?;
            let name = space
                .resource_name(&folder.name)
                .map_err(|error| error.to_string())?;
            space
                .create_folder(id, parent, name, folder.metadata)
                .map_err(|error| error.to_string())?;
        }
        for resource in self.resources {
            let parent = parse_id(&resource.parent_id, "resource.parent_id")?;
            let name = space
                .resource_name(&resource.name)
                .map_err(|error| error.to_string())?;
            let bytes = STANDARD
                .decode(resource.bytes_base64)
                .map_err(|_| "resource bytes are not valid Base64".to_owned())?;
            space
                .insert_resource(parent, name, bytes, resource.metadata)
                .map_err(|error| error.to_string())?;
        }
        Ok(RestoredSession {
            store,
            root,
            root_folder,
            next_folder,
        })
    }

    pub(super) fn case_policy(&self) -> AddressCasePolicy {
        self.case_policy
    }

    pub(super) fn store_id(&self) -> StoreId {
        self.descriptor.id()
    }

    pub(super) fn root_id(&self) -> ResourceRootId {
        self.root.id()
    }

    pub(super) fn root_folder_id(&self) -> Result<FolderId, String> {
        parse_id(&self.root_folder_id, "root_folder_id")
    }
}

pub(super) struct RestoredSession {
    pub(super) store: StoreId,
    pub(super) root: ResourceRootId,
    pub(super) root_folder: FolderId,
    pub(super) next_folder: u128,
}

fn parse_id(value: &str, label: &str) -> Result<FolderId, String> {
    Ok(FolderId::from_u128(parse_u128(value, label)?))
}

fn parse_u128(value: &str, label: &str) -> Result<u128, String> {
    value
        .parse()
        .map_err(|_| format!("persisted {label} is not a decimal u128"))
}
