use std::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
};

use thiserror::Error;

use crate::resource::StoredResource;
use crate::{
    AddressCasePolicy, FolderId, ResourceAddress, ResourceAddressError, ResourceEntry, ResourceKey,
    ResourceMetadata, ResourceMutationObservation, ResourceMutationOutcome, ResourceName,
    ResourceRootDescriptor, ResourceRootId, ResourceSearchQuery, ResourceSpaceLimits,
    ResourceSpaceSummary, ResourceVisibility, StoreId, VisibilityQuery,
};

/// A provider-owned in-memory hierarchy that retains immutable resource bytes.
///
/// This provider proves logical root, folder, and resource semantics without
/// implying filesystem behavior or selecting a platform storage mechanism.
#[derive(Debug)]
pub struct InMemoryResourceSpace {
    store_id: StoreId,
    case_policy: AddressCasePolicy,
    limits: ResourceSpaceLimits,
    roots: BTreeMap<ResourceRootId, RootEntry>,
    folders: BTreeMap<FolderId, FolderEntry>,
    resources: BTreeMap<(FolderId, ResourceName), StoredResource>,
    mutation_observation_capacity: usize,
    next_mutation_sequence: u64,
    mutation_observations: VecDeque<ResourceMutationObservation>,
}

impl InMemoryResourceSpace {
    pub fn new(store_id: StoreId, case_policy: AddressCasePolicy) -> Self {
        Self::with_limits(store_id, case_policy, ResourceSpaceLimits::default())
    }

    pub fn with_limits(
        store_id: StoreId,
        case_policy: AddressCasePolicy,
        limits: ResourceSpaceLimits,
    ) -> Self {
        Self {
            store_id,
            case_policy,
            limits,
            roots: BTreeMap::new(),
            folders: BTreeMap::new(),
            resources: BTreeMap::new(),
            mutation_observation_capacity: 0,
            next_mutation_sequence: 1,
            mutation_observations: VecDeque::new(),
        }
    }

    pub const fn store_id(&self) -> StoreId {
        self.store_id
    }

    pub const fn case_policy(&self) -> AddressCasePolicy {
        self.case_policy
    }

    pub const fn limits(&self) -> ResourceSpaceLimits {
        self.limits
    }

    /// Starts a new bounded mutation-observation capture session.
    ///
    /// Observation is disabled by default. Enabling it clears observations
    /// from any earlier session and restarts local sequence numbering at one.
    /// Capture never changes whether a later mutation succeeds or fails.
    pub fn enable_mutation_observations(
        &mut self,
        capacity: usize,
    ) -> Result<(), ResourceSpaceError> {
        if capacity == 0 {
            return Err(ResourceSpaceError::MutationObservationCapacityZero);
        }
        self.mutation_observation_capacity = capacity;
        self.next_mutation_sequence = 1;
        self.mutation_observations.clear();
        Ok(())
    }

    /// Disables mutation observation and releases all retained records.
    pub fn disable_mutation_observations(&mut self) {
        self.mutation_observation_capacity = 0;
        self.next_mutation_sequence = 1;
        self.mutation_observations.clear();
    }

    pub const fn mutation_observations_enabled(&self) -> bool {
        self.mutation_observation_capacity != 0
    }

    /// Returns the retained-record limit for the active capture session.
    pub const fn mutation_observation_capacity(&self) -> Option<usize> {
        if self.mutation_observation_capacity == 0 {
            None
        } else {
            Some(self.mutation_observation_capacity)
        }
    }

    pub fn mutation_observations(
        &self,
    ) -> impl ExactSizeIterator<Item = &ResourceMutationObservation> {
        self.mutation_observations.iter()
    }

    /// Removes and returns observations in ascending local sequence order.
    pub fn drain_mutation_observations(&mut self) -> Vec<ResourceMutationObservation> {
        self.mutation_observations.drain(..).collect()
    }

    /// Returns current retained hierarchy and byte counts without exposing the
    /// provider's backing collections.
    pub fn summary(&self) -> ResourceSpaceSummary {
        ResourceSpaceSummary::new(
            self.roots.len(),
            self.folders.len(),
            self.resources.len(),
            self.resources
                .values()
                .map(|resource| resource.bytes.len())
                .sum(),
        )
    }

    pub fn create_root(
        &mut self,
        descriptor: ResourceRootDescriptor,
        root_folder: FolderId,
        metadata: ResourceMetadata,
    ) -> Result<(), ResourceSpaceError> {
        let root_id = descriptor.id();
        if self.roots.contains_key(&root_id) {
            return Err(ResourceSpaceError::RootAlreadyExists { root: root_id });
        }
        if self.folders.contains_key(&root_folder) {
            return Err(ResourceSpaceError::FolderIdAlreadyExists {
                folder: root_folder,
            });
        }

        self.folders.insert(
            root_folder,
            FolderEntry {
                id: root_folder,
                store: self.store_id,
                root: root_id,
                parent: None,
                name: None,
                metadata,
            },
        );
        self.roots.insert(
            root_id,
            RootEntry {
                descriptor,
                root_folder,
            },
        );
        self.record_mutation(ResourceMutationOutcome::RootCreated {
            root: root_id,
            root_folder,
        });
        Ok(())
    }

    pub fn root(&self, root: ResourceRootId) -> Option<&ResourceRootDescriptor> {
        self.roots.get(&root).map(|entry| &entry.descriptor)
    }

    pub fn root_folder(&self, root: ResourceRootId) -> Option<FolderId> {
        self.roots.get(&root).map(|entry| entry.root_folder)
    }

    /// Removes a root only when its distinguished root folder has no children
    /// or directly retained resources.
    /// This keeps root removal from silently deleting an unrelated subtree.
    pub fn remove_empty_root(
        &mut self,
        root: ResourceRootId,
    ) -> Result<ResourceRootDescriptor, ResourceSpaceError> {
        let root_folder = self
            .roots
            .get(&root)
            .ok_or(ResourceSpaceError::RootNotFound { root })?
            .root_folder;
        if self
            .folders
            .values()
            .any(|candidate| candidate.parent == Some(root_folder))
            || self
                .resources
                .keys()
                .any(|(parent, _)| *parent == root_folder)
        {
            return Err(ResourceSpaceError::RootNotEmpty { root });
        }

        self.folders.remove(&root_folder);
        let descriptor = self
            .roots
            .remove(&root)
            .expect("root was found before removal")
            .descriptor;
        self.record_mutation(ResourceMutationOutcome::RootRemoved { root, root_folder });
        Ok(descriptor)
    }

    pub fn rename_root(
        &mut self,
        root: ResourceRootId,
        display_name: impl Into<String>,
    ) -> Result<(), ResourceSpaceError> {
        let entry = self
            .roots
            .get_mut(&root)
            .ok_or(ResourceSpaceError::RootNotFound { root })?;
        entry.descriptor.rename(display_name);
        self.record_mutation(ResourceMutationOutcome::RootRenamed { root });
        Ok(())
    }

    pub fn folder(&self, folder: FolderId) -> Option<&FolderEntry> {
        self.folders.get(&folder)
    }

    /// Parses a folder name using this space's declared case policy.
    pub fn resource_name(&self, value: &str) -> Result<ResourceName, ResourceAddressError> {
        ResourceName::parse(value, self.case_policy)
    }

    pub fn create_folder(
        &mut self,
        id: FolderId,
        parent: FolderId,
        name: ResourceName,
        metadata: ResourceMetadata,
    ) -> Result<(), ResourceSpaceError> {
        if self.folders.contains_key(&id) {
            return Err(ResourceSpaceError::FolderIdAlreadyExists { folder: id });
        }
        let parent_entry = self
            .folders
            .get(&parent)
            .ok_or(ResourceSpaceError::FolderNotFound { folder: parent })?;
        let root = parent_entry.root;
        let name = self.require_policy_name(name)?;
        self.ensure_name_available(parent, &name, None)?;

        self.folders.insert(
            id,
            FolderEntry {
                id,
                store: self.store_id,
                root,
                parent: Some(parent),
                name: Some(name),
                metadata,
            },
        );
        self.record_mutation(ResourceMutationOutcome::FolderCreated { folder: id, parent });
        Ok(())
    }

    pub fn insert_resource(
        &mut self,
        parent: FolderId,
        name: ResourceName,
        bytes: impl Into<Arc<[u8]>>,
        metadata: ResourceMetadata,
    ) -> Result<ResourceEntry, ResourceSpaceError> {
        let parent_entry = self
            .folders
            .get(&parent)
            .ok_or(ResourceSpaceError::FolderNotFound { folder: parent })?;
        let root = parent_entry.root;
        let name = self.require_policy_name(name)?;
        self.ensure_name_available(parent, &name, None)?;

        let bytes = bytes.into();
        self.ensure_retention_allows(bytes.len(), None)?;
        let stored = StoredResource {
            parent,
            name: name.clone(),
            bytes,
            metadata,
        };
        self.resources.insert((parent, name), stored.clone());
        let entry = self.resource_entry(root, stored);
        self.record_mutation(ResourceMutationOutcome::ResourceInserted {
            key: entry.key().clone(),
            byte_len: entry.byte_len(),
        });
        Ok(entry)
    }

    pub fn resource(
        &self,
        parent: FolderId,
        name: &ResourceName,
    ) -> Result<Option<ResourceEntry>, ResourceSpaceError> {
        let parent_entry = self
            .folders
            .get(&parent)
            .ok_or(ResourceSpaceError::FolderNotFound { folder: parent })?;
        let canonical_name = self.require_policy_name(name.clone())?;
        Ok(self
            .resources
            .get(&(parent, canonical_name))
            .cloned()
            .map(|stored| self.resource_entry(parent_entry.root, stored)))
    }

    pub fn replace_resource(
        &mut self,
        parent: FolderId,
        name: &ResourceName,
        bytes: impl Into<Arc<[u8]>>,
        metadata: ResourceMetadata,
    ) -> Result<ResourceEntry, ResourceSpaceError> {
        let parent_entry = self
            .folders
            .get(&parent)
            .ok_or(ResourceSpaceError::FolderNotFound { folder: parent })?;
        let canonical_name = self.require_policy_name(name.clone())?;
        let bytes = bytes.into();
        let previous_len = self
            .resources
            .get(&(parent, canonical_name.clone()))
            .ok_or_else(|| ResourceSpaceError::ResourceNotFound {
                parent,
                name: canonical_name.clone(),
            })?
            .bytes
            .len();
        self.ensure_retention_allows(bytes.len(), Some(previous_len))?;
        let stored = {
            let stored = self
                .resources
                .get_mut(&(parent, canonical_name.clone()))
                .ok_or_else(|| ResourceSpaceError::ResourceNotFound {
                    parent,
                    name: canonical_name.clone(),
                })?;
            stored.bytes = bytes;
            stored.metadata = metadata;
            stored.clone()
        };
        let entry = self.resource_entry(parent_entry.root, stored);
        self.record_mutation(ResourceMutationOutcome::ResourceReplaced {
            key: entry.key().clone(),
            previous_byte_len: previous_len,
            byte_len: entry.byte_len(),
        });
        Ok(entry)
    }

    /// Changes only the explicit visibility metadata of one retained resource.
    ///
    /// Visibility is logical Resource Space state, rather than an inferred
    /// filename convention or provider-specific hidden-file attribute.
    pub fn set_resource_visibility(
        &mut self,
        parent: FolderId,
        name: &ResourceName,
        visibility: ResourceVisibility,
    ) -> Result<ResourceEntry, ResourceSpaceError> {
        let parent_entry = self
            .folders
            .get(&parent)
            .ok_or(ResourceSpaceError::FolderNotFound { folder: parent })?;
        let root = parent_entry.root;
        let canonical_name = self.require_policy_name(name.clone())?;
        let stored = {
            let stored = self
                .resources
                .get_mut(&(parent, canonical_name.clone()))
                .ok_or_else(|| ResourceSpaceError::ResourceNotFound {
                    parent,
                    name: canonical_name.clone(),
                })?;
            stored.metadata.visibility = visibility;
            stored.clone()
        };
        let entry = self.resource_entry(root, stored);
        self.record_mutation(ResourceMutationOutcome::ResourceVisibilityChanged {
            key: entry.key().clone(),
            visibility,
        });
        Ok(entry)
    }

    pub fn remove_resource(
        &mut self,
        parent: FolderId,
        name: &ResourceName,
    ) -> Result<ResourceEntry, ResourceSpaceError> {
        let parent_entry = self
            .folders
            .get(&parent)
            .ok_or(ResourceSpaceError::FolderNotFound { folder: parent })?;
        let canonical_name = self.require_policy_name(name.clone())?;
        let stored = self
            .resources
            .remove(&(parent, canonical_name.clone()))
            .ok_or(ResourceSpaceError::ResourceNotFound {
                parent,
                name: canonical_name,
            })?;
        let entry = self.resource_entry(parent_entry.root, stored);
        self.record_mutation(ResourceMutationOutcome::ResourceRemoved {
            key: entry.key().clone(),
            byte_len: entry.byte_len(),
        });
        Ok(entry)
    }

    /// Returns whether a directly named resource exists beneath `parent`.
    pub fn contains_resource(
        &self,
        parent: FolderId,
        name: &ResourceName,
    ) -> Result<bool, ResourceSpaceError> {
        if !self.folders.contains_key(&parent) {
            return Err(ResourceSpaceError::FolderNotFound { folder: parent });
        }
        let canonical_name = self.require_policy_name(name.clone())?;
        Ok(self.resources.contains_key(&(parent, canonical_name)))
    }

    /// Copies a resource's immutable content into a new direct child entry.
    pub fn copy_resource(
        &mut self,
        source_parent: FolderId,
        source_name: &ResourceName,
        destination_parent: FolderId,
        destination_name: ResourceName,
    ) -> Result<ResourceEntry, ResourceSpaceError> {
        let source_name = self.require_policy_name(source_name.clone())?;
        let destination_name = self.require_policy_name(destination_name)?;
        let source = self
            .resources
            .get(&(source_parent, source_name.clone()))
            .cloned()
            .ok_or(ResourceSpaceError::ResourceNotFound {
                parent: source_parent,
                name: source_name,
            })?;
        let destination_root = self
            .folders
            .get(&destination_parent)
            .ok_or(ResourceSpaceError::FolderNotFound {
                folder: destination_parent,
            })?
            .root;
        self.ensure_name_available(destination_parent, &destination_name, None)?;
        self.ensure_retention_allows(source.bytes.len(), None)?;

        let source_root = self
            .folders
            .get(&source_parent)
            .expect("stored resources always retain a valid parent folder")
            .root;
        let source_key = self
            .resource_entry(source_root, source.clone())
            .key()
            .clone();
        let stored = StoredResource {
            parent: destination_parent,
            name: destination_name.clone(),
            bytes: source.bytes,
            metadata: source.metadata,
        };
        self.resources
            .insert((destination_parent, destination_name), stored.clone());
        let entry = self.resource_entry(destination_root, stored);
        self.record_mutation(ResourceMutationOutcome::ResourceCopied {
            source: source_key,
            destination: entry.key().clone(),
            byte_len: entry.byte_len(),
        });
        Ok(entry)
    }

    /// Moves a resource to a new direct parent without changing its retained bytes.
    pub fn move_resource(
        &mut self,
        source_parent: FolderId,
        source_name: &ResourceName,
        destination_parent: FolderId,
        destination_name: ResourceName,
    ) -> Result<ResourceEntry, ResourceSpaceError> {
        let source_name = self.require_policy_name(source_name.clone())?;
        let destination_name = self.require_policy_name(destination_name)?;
        let source = self
            .resources
            .get(&(source_parent, source_name.clone()))
            .cloned()
            .ok_or(ResourceSpaceError::ResourceNotFound {
                parent: source_parent,
                name: source_name.clone(),
            })?;
        let destination_root = self
            .folders
            .get(&destination_parent)
            .ok_or(ResourceSpaceError::FolderNotFound {
                folder: destination_parent,
            })?
            .root;

        if source_parent == destination_parent && source_name == destination_name {
            return Ok(self.resource_entry(destination_root, source));
        }

        self.ensure_name_available(destination_parent, &destination_name, None)?;
        let source_root = self
            .folders
            .get(&source_parent)
            .expect("stored resources always retain a valid parent folder")
            .root;
        let source_key = self
            .resource_entry(source_root, source.clone())
            .key()
            .clone();
        let stored = StoredResource {
            parent: destination_parent,
            name: destination_name.clone(),
            bytes: source.bytes,
            metadata: source.metadata,
        };
        self.resources.remove(&(source_parent, source_name));
        self.resources
            .insert((destination_parent, destination_name), stored.clone());
        let entry = self.resource_entry(destination_root, stored);
        self.record_mutation(ResourceMutationOutcome::ResourceMoved {
            source: source_key,
            destination: entry.key().clone(),
            byte_len: entry.byte_len(),
        });
        Ok(entry)
    }

    pub fn list_children(
        &self,
        parent: FolderId,
        visibility: VisibilityQuery,
    ) -> Result<Vec<FolderEntry>, ResourceSpaceError> {
        if !self.folders.contains_key(&parent) {
            return Err(ResourceSpaceError::FolderNotFound { folder: parent });
        }

        let mut children = self
            .folders
            .values()
            .filter(|folder| folder.parent == Some(parent))
            .filter(|folder| visibility.includes(folder.metadata.visibility))
            .cloned()
            .collect::<Vec<_>>();
        children.sort_by(|left, right| left.name().cmp(&right.name()).then(left.id.cmp(&right.id)));
        Ok(children)
    }

    /// Lists direct child folders. This is an explicit alias for the original
    /// folder-only `list_children` operation while resource enumeration grows.
    pub fn list_folders(
        &self,
        parent: FolderId,
        visibility: VisibilityQuery,
    ) -> Result<Vec<FolderEntry>, ResourceSpaceError> {
        self.list_children(parent, visibility)
    }

    /// Lists direct child resources in normalized-name order.
    pub fn list_resources(
        &self,
        parent: FolderId,
        visibility: VisibilityQuery,
    ) -> Result<Vec<ResourceEntry>, ResourceSpaceError> {
        let parent_entry = self
            .folders
            .get(&parent)
            .ok_or(ResourceSpaceError::FolderNotFound { folder: parent })?;
        let mut resources = self
            .resources
            .iter()
            .filter(|((candidate_parent, _), stored)| {
                *candidate_parent == parent && visibility.includes(stored.metadata.visibility)
            })
            .map(|(_, stored)| self.resource_entry(parent_entry.root, stored.clone()))
            .collect::<Vec<_>>();
        resources.sort_by(|left, right| left.name().cmp(right.name()));
        Ok(resources)
    }

    /// Searches a selected folder and its descendants using a bounded literal
    /// query. This is intentionally distinct from `list_resources`, which
    /// enumerates direct children only.
    pub fn search_resources(
        &self,
        scope: FolderId,
        query: &ResourceSearchQuery,
    ) -> Result<Vec<ResourceEntry>, ResourceSpaceError> {
        if query.max_results() == 0 {
            return Err(ResourceSpaceError::SearchResultLimitZero);
        }
        if !self.folders.contains_key(&scope) {
            return Err(ResourceSpaceError::FolderNotFound { folder: scope });
        }

        let mut matches = self
            .resources
            .values()
            .filter(|stored| self.is_descendant_of(stored.parent, scope))
            .filter(|stored| {
                query
                    .visibility_query()
                    .includes(stored.metadata.visibility)
            })
            .filter(|stored| {
                query
                    .name_prefix()
                    .is_none_or(|prefix| stored.name.as_str().starts_with(prefix))
            })
            .filter(|stored| {
                query
                    .name_suffix()
                    .is_none_or(|suffix| stored.name.as_str().ends_with(suffix))
            })
            .filter(|stored| {
                query.media_type().is_none_or(|media_type| {
                    stored.metadata.media_type.as_deref() == Some(media_type)
                })
            })
            .map(|stored| {
                let root = self
                    .folders
                    .get(&stored.parent)
                    .expect("stored resources always retain a valid parent folder")
                    .root;
                self.resource_entry(root, stored.clone())
            })
            .collect::<Vec<_>>();
        matches.sort_by(|left, right| left.key().cmp(right.key()));
        matches.truncate(query.max_results());
        Ok(matches)
    }

    pub fn rename_folder(
        &mut self,
        folder: FolderId,
        name: ResourceName,
    ) -> Result<(), ResourceSpaceError> {
        let name = self.require_policy_name(name)?;
        let entry = self
            .folders
            .get(&folder)
            .ok_or(ResourceSpaceError::FolderNotFound { folder })?;
        let parent = entry
            .parent
            .ok_or(ResourceSpaceError::RootFolderImmutable { folder })?;
        self.ensure_name_available(parent, &name, Some(folder))?;
        self.folders
            .get_mut(&folder)
            .expect("folder was found before collision validation")
            .name = Some(name);
        self.record_mutation(ResourceMutationOutcome::FolderRenamed { folder });
        Ok(())
    }

    pub fn move_folder(
        &mut self,
        folder: FolderId,
        new_parent: FolderId,
    ) -> Result<(), ResourceSpaceError> {
        let entry = self
            .folders
            .get(&folder)
            .ok_or(ResourceSpaceError::FolderNotFound { folder })?;
        if entry.parent.is_none() {
            return Err(ResourceSpaceError::RootFolderImmutable { folder });
        }
        let destination = self
            .folders
            .get(&new_parent)
            .ok_or(ResourceSpaceError::FolderNotFound { folder: new_parent })?;
        if entry.root != destination.root {
            return Err(ResourceSpaceError::CrossRootMove {
                folder,
                source_root: entry.root,
                destination_root: destination.root,
            });
        }
        if folder == new_parent || self.is_descendant_of(new_parent, folder) {
            return Err(ResourceSpaceError::FolderMoveCycle { folder, new_parent });
        }
        let name = entry
            .name
            .as_ref()
            .expect("ordinary folders always have a name");
        self.ensure_name_available(new_parent, name, Some(folder))?;
        let source_parent = entry
            .parent
            .expect("root folders are rejected before move validation");
        self.folders
            .get_mut(&folder)
            .expect("folder was found before move validation")
            .parent = Some(new_parent);
        self.record_mutation(ResourceMutationOutcome::FolderMoved {
            folder,
            source_parent,
            destination_parent: new_parent,
        });
        Ok(())
    }

    pub fn remove_empty_folder(
        &mut self,
        folder: FolderId,
    ) -> Result<FolderEntry, ResourceSpaceError> {
        let entry = self
            .folders
            .get(&folder)
            .ok_or(ResourceSpaceError::FolderNotFound { folder })?;
        if entry.parent.is_none() {
            return Err(ResourceSpaceError::RootFolderImmutable { folder });
        }
        if self
            .folders
            .values()
            .any(|candidate| candidate.parent == Some(folder))
            || self.resources.keys().any(|(parent, _)| *parent == folder)
        {
            return Err(ResourceSpaceError::FolderNotEmpty { folder });
        }
        let entry = self
            .folders
            .remove(&folder)
            .expect("folder was found before removal");
        let parent = entry
            .parent
            .expect("root folders are rejected before removal");
        self.record_mutation(ResourceMutationOutcome::FolderRemoved { folder, parent });
        Ok(entry)
    }

    fn record_mutation(&mut self, outcome: ResourceMutationOutcome) {
        if self.mutation_observation_capacity == 0 {
            return;
        }
        if self.mutation_observations.len() == self.mutation_observation_capacity {
            self.mutation_observations.pop_front();
        }
        self.mutation_observations
            .push_back(ResourceMutationObservation::new(
                self.next_mutation_sequence,
                outcome,
            ));
        self.next_mutation_sequence = self.next_mutation_sequence.saturating_add(1);
    }

    fn ensure_name_available(
        &self,
        parent: FolderId,
        name: &ResourceName,
        except: Option<FolderId>,
    ) -> Result<(), ResourceSpaceError> {
        if self.folders.values().any(|candidate| {
            candidate.parent == Some(parent)
                && except != Some(candidate.id)
                && candidate.name.as_ref() == Some(name)
        }) || self.resources.contains_key(&(parent, name.clone()))
        {
            return Err(ResourceSpaceError::ChildNameConflict {
                parent,
                name: name.clone(),
            });
        }
        Ok(())
    }

    fn ensure_retention_allows(
        &self,
        incoming_len: usize,
        replaced_len: Option<usize>,
    ) -> Result<(), ResourceSpaceError> {
        if let Some(max_bytes_per_entry) = self.limits.max_bytes_per_entry {
            if incoming_len > max_bytes_per_entry {
                return Err(ResourceSpaceError::EntryByteLimitExceeded {
                    limit: max_bytes_per_entry,
                    attempted: incoming_len,
                });
            }
        }
        if replaced_len.is_none() {
            if let Some(max_entries) = self.limits.max_entries {
                let attempted = self.resources.len().saturating_add(1);
                if attempted > max_entries {
                    return Err(ResourceSpaceError::EntryLimitExceeded {
                        limit: max_entries,
                        attempted,
                    });
                }
            }
        }
        if let Some(max_total_bytes) = self.limits.max_total_bytes {
            let retained = self.total_retained_bytes();
            let attempted = retained
                .saturating_sub(replaced_len.unwrap_or_default())
                .saturating_add(incoming_len);
            if attempted > max_total_bytes {
                return Err(ResourceSpaceError::TotalByteLimitExceeded {
                    limit: max_total_bytes,
                    attempted,
                });
            }
        }
        Ok(())
    }

    fn total_retained_bytes(&self) -> usize {
        self.resources
            .values()
            .map(|resource| resource.bytes.len())
            .sum()
    }

    fn require_policy_name(&self, name: ResourceName) -> Result<ResourceName, ResourceSpaceError> {
        let normalized = self.resource_name(name.as_str()).expect(
            "a resource name parsed under one case policy remains valid under another case policy",
        );
        if normalized != name {
            return Err(ResourceSpaceError::NamePolicyMismatch { name });
        }
        Ok(normalized)
    }

    fn is_descendant_of(&self, candidate: FolderId, ancestor: FolderId) -> bool {
        let mut current = Some(candidate);
        while let Some(folder) = current {
            if folder == ancestor {
                return true;
            }
            current = self.folders.get(&folder).and_then(|entry| entry.parent);
        }
        false
    }

    fn resource_entry(&self, root: ResourceRootId, stored: StoredResource) -> ResourceEntry {
        let address = self.resource_address(stored.parent, stored.name.clone());
        ResourceEntry::new(
            ResourceKey::new(self.store_id, root, address),
            stored.parent,
            stored.name,
            stored.bytes,
            stored.metadata,
        )
    }

    fn resource_address(&self, parent: FolderId, name: ResourceName) -> ResourceAddress {
        let mut segments = vec![name];
        let mut current = parent;
        loop {
            let folder = self
                .folders
                .get(&current)
                .expect("resource parent belongs to this resource space");
            let Some(parent) = folder.parent else {
                break;
            };
            segments.push(
                folder
                    .name
                    .clone()
                    .expect("ordinary folders always have a name"),
            );
            current = parent;
        }
        segments.reverse();
        ResourceAddress::from_segments(segments)
            .expect("a resource entry always has a file-name segment")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderEntry {
    id: FolderId,
    store: StoreId,
    root: ResourceRootId,
    parent: Option<FolderId>,
    name: Option<ResourceName>,
    metadata: ResourceMetadata,
}

impl FolderEntry {
    pub const fn id(&self) -> FolderId {
        self.id
    }

    pub const fn store(&self) -> StoreId {
        self.store
    }

    pub const fn root(&self) -> ResourceRootId {
        self.root
    }

    pub const fn parent(&self) -> Option<FolderId> {
        self.parent
    }

    pub fn name(&self) -> Option<&ResourceName> {
        self.name.as_ref()
    }

    pub const fn metadata(&self) -> &ResourceMetadata {
        &self.metadata
    }

    pub const fn is_root_folder(&self) -> bool {
        self.parent.is_none()
    }
}

#[derive(Debug)]
struct RootEntry {
    descriptor: ResourceRootDescriptor,
    root_folder: FolderId,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ResourceSpaceError {
    #[error("mutation observation requires a nonzero retained-record capacity")]
    MutationObservationCapacityZero,
    #[error("resource search requires a nonzero result limit")]
    SearchResultLimitZero,
    #[error("resource root {root:?} already exists")]
    RootAlreadyExists { root: ResourceRootId },
    #[error("resource root {root:?} does not exist")]
    RootNotFound { root: ResourceRootId },
    #[error("resource root {root:?} is not empty")]
    RootNotEmpty { root: ResourceRootId },
    #[error("folder {folder:?} already exists")]
    FolderIdAlreadyExists { folder: FolderId },
    #[error("folder {folder:?} does not exist")]
    FolderNotFound { folder: FolderId },
    #[error("folder {folder:?} is a root folder and cannot be moved, renamed, or removed")]
    RootFolderImmutable { folder: FolderId },
    #[error("folder {folder:?} is not empty")]
    FolderNotEmpty { folder: FolderId },
    #[error("folder {folder:?} cannot move beneath itself or one of its descendants")]
    FolderMoveCycle {
        folder: FolderId,
        new_parent: FolderId,
    },
    #[error(
        "folder {folder:?} cannot move from root {source_root:?} to root {destination_root:?}"
    )]
    CrossRootMove {
        folder: FolderId,
        source_root: ResourceRootId,
        destination_root: ResourceRootId,
    },
    #[error("child name {name} already exists beneath parent {parent:?}")]
    ChildNameConflict {
        parent: FolderId,
        name: ResourceName,
    },
    #[error("resource {name} does not exist beneath parent {parent:?}")]
    ResourceNotFound {
        parent: FolderId,
        name: ResourceName,
    },
    #[error("resource entry limit {limit} would be exceeded by {attempted} entries")]
    EntryLimitExceeded { limit: usize, attempted: usize },
    #[error("resource entry byte limit {limit} would be exceeded by {attempted} bytes")]
    EntryByteLimitExceeded { limit: usize, attempted: usize },
    #[error("resource total-byte limit {limit} would be exceeded by {attempted} bytes")]
    TotalByteLimitExceeded { limit: usize, attempted: usize },
    #[error("folder name {name} was not normalized for this resource space's case policy")]
    NamePolicyMismatch { name: ResourceName },
}
