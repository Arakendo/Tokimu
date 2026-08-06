//! Explicit archive operations over retained Resource Space bytes.
//!
//! Resource Space continues to own logical identity and retained bytes. The
//! archive provider owns container parsing, name validation, and bounded entry
//! decoding. Ordinary resource reads never inspect or extract archives.

use archive_provider::{
    ArchiveEntryObservation, ArchiveError, ArchiveFormat, ArchiveManifest, ArchiveProvider,
    ArchiveReadLimits, ArchiveReadResult, ArchiveWriteEntry, ArchiveWriteLimits,
    ArchiveWriteObservation, ArchiveWriter,
};
use resource_space::{
    ContentFingerprint, FolderId, InMemoryResourceSpace, ResourceEntry, ResourceKey,
    ResourceMetadata, ResourceName, ResourceSpaceError, VisibilityQuery,
};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

/// Caller-selected behavior when the selected destination already exists.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveResourceCollisionPolicy {
    Reject,
    Replace,
}

/// Identifies one retained resource and the bounded interpretation requested
/// by its caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectArchiveResourceRequest {
    pub source_folder: FolderId,
    pub source_name: ResourceName,
    pub format: ArchiveFormat,
    pub limits: ArchiveReadLimits,
}

/// Provider-neutral evidence from inspecting one retained archive resource.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceArchiveInspection {
    source: ResourceKey,
    source_fingerprint: ContentFingerprint,
    manifest: ArchiveManifest,
}

impl ResourceArchiveInspection {
    pub const fn source(&self) -> &ResourceKey {
        &self.source
    }

    pub const fn source_fingerprint(&self) -> &ContentFingerprint {
        &self.source_fingerprint
    }

    pub const fn manifest(&self) -> &ArchiveManifest {
        &self.manifest
    }
}

/// A provider-neutral, read-only projection of a retained archive resource.
///
/// This is deliberately not a Resource Space folder: it retains no extracted
/// payloads, cannot mutate the source archive, and becomes stale when the
/// source fingerprint no longer matches. Callers must use an explicit copy or
/// subtree-import operation to materialize entries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveDerivedView {
    source: ResourceKey,
    source_fingerprint: ContentFingerprint,
    source_folder: FolderId,
    source_name: ResourceName,
    format: ArchiveFormat,
    entries: Vec<ArchiveDerivedEntry>,
}

impl ArchiveDerivedView {
    pub const fn source(&self) -> &ResourceKey {
        &self.source
    }

    pub const fn source_fingerprint(&self) -> &ContentFingerprint {
        &self.source_fingerprint
    }

    pub const fn format(&self) -> ArchiveFormat {
        self.format
    }

    pub const fn entries(&self) -> &Vec<ArchiveDerivedEntry> {
        &self.entries
    }

    pub const fn is_read_only(&self) -> bool {
        true
    }
}

/// One qualified entry in an [`ArchiveDerivedView`]. Its identity is always
/// scoped by the view's source resource and source fingerprint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveDerivedEntry {
    entry: ArchiveEntryObservation,
}

impl ArchiveDerivedEntry {
    pub const fn entry(&self) -> &ArchiveEntryObservation {
        &self.entry
    }

    pub const fn normalized_name(&self) -> &String {
        &self.entry.normalized_name
    }
}

/// Copies one validated regular-file entry into one explicit logical
/// destination. The entry name never chooses the destination implicitly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CopyArchiveEntryRequest {
    pub source_folder: FolderId,
    pub source_name: ResourceName,
    pub format: ArchiveFormat,
    pub entry_name: String,
    pub limits: ArchiveReadLimits,
    pub destination_folder: FolderId,
    pub destination_name: ResourceName,
    pub collision: ArchiveResourceCollisionPolicy,
    pub metadata: ResourceMetadata,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArchiveResourceMutation {
    Inserted,
    Replaced,
}

/// Evidence linking an immutable source archive and validated entry to the
/// materialized logical resource.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveEntryCopyObservation {
    source: ResourceKey,
    source_fingerprint: ContentFingerprint,
    entry: ArchiveEntryObservation,
    result: ResourceKey,
    result_fingerprint: ContentFingerprint,
    mutation: ArchiveResourceMutation,
}

impl ArchiveEntryCopyObservation {
    pub const fn source(&self) -> &ResourceKey {
        &self.source
    }

    pub const fn source_fingerprint(&self) -> &ContentFingerprint {
        &self.source_fingerprint
    }

    pub const fn entry(&self) -> &ArchiveEntryObservation {
        &self.entry
    }

    pub const fn result(&self) -> &ResourceKey {
        &self.result
    }

    pub const fn result_fingerprint(&self) -> &ContentFingerprint {
        &self.result_fingerprint
    }

    pub const fn mutation(&self) -> ArchiveResourceMutation {
        self.mutation
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveEntryCopyResult {
    entry: ResourceEntry,
    observation: ArchiveEntryCopyObservation,
}

/// Materializes every admitted regular-file entry beneath one caller-selected
/// folder. The archive chooses neither the destination root nor folder IDs.
/// All payloads and names are validated before Resource Space mutation begins.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportArchiveSubtreeRequest {
    pub source_folder: FolderId,
    pub source_name: ResourceName,
    pub format: ArchiveFormat,
    pub limits: ArchiveReadLimits,
    pub destination_parent: FolderId,
    pub destination_root_name: ResourceName,
    pub first_folder_id: FolderId,
    pub metadata: ResourceMetadata,
}

/// Provider-neutral evidence from an explicit archive subtree materialization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveSubtreeImportObservation {
    source: ResourceKey,
    source_fingerprint: ContentFingerprint,
    destination_root: FolderId,
    folders: u32,
    resources: u32,
    retained_bytes: u64,
}

impl ArchiveSubtreeImportObservation {
    pub const fn source(&self) -> &ResourceKey {
        &self.source
    }

    pub const fn source_fingerprint(&self) -> &ContentFingerprint {
        &self.source_fingerprint
    }

    pub const fn destination_root(&self) -> FolderId {
        self.destination_root
    }

    pub const fn folders(&self) -> u32 {
        self.folders
    }

    pub const fn resources(&self) -> u32 {
        self.resources
    }

    pub const fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }
}

/// Exports one explicit Resource Space subtree through a caller-selected
/// archive writer. The bridge owns lowering logical folders and resources into
/// ordered archive entries; it does not allocate a destination resource or
/// infer a host path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportResourceSubtreeRequest {
    pub source_folder: FolderId,
    pub format: ArchiveFormat,
    pub limits: ArchiveWriteLimits,
    pub file_compression: archive_provider::ArchiveCompression,
    pub visibility: VisibilityQuery,
}

/// Provider-neutral evidence from one completed subtree export.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceSubtreeExport {
    source_folder: FolderId,
    folders: u32,
    resources: u32,
    archive: ArchiveWriteObservation,
    bytes: Vec<u8>,
}

impl ResourceSubtreeExport {
    pub const fn source_folder(&self) -> FolderId {
        self.source_folder
    }

    pub const fn folders(&self) -> u32 {
        self.folders
    }

    pub const fn resources(&self) -> u32 {
        self.resources
    }

    pub const fn archive(&self) -> &ArchiveWriteObservation {
        &self.archive
    }

    pub const fn bytes(&self) -> &Vec<u8> {
        &self.bytes
    }
}

impl ArchiveEntryCopyResult {
    pub const fn entry(&self) -> &ResourceEntry {
        &self.entry
    }

    pub const fn observation(&self) -> &ArchiveEntryCopyObservation {
        &self.observation
    }
}

#[derive(Debug, Error)]
pub enum ResourceArchiveBridgeError {
    #[error("archive provider does not support requested format {format:?}")]
    ProviderUnavailable { format: ArchiveFormat },
    #[error("archive writer does not support requested format {format:?}")]
    WriterUnavailable { format: ArchiveFormat },
    #[error("source folder {folder:?} was not found")]
    SourceFolderNotFound { folder: FolderId },
    #[error("destination parent folder {folder:?} was not found")]
    DestinationParentNotFound { folder: FolderId },
    #[error("resource-space lookup failed for source `{name}`: {error}")]
    Lookup {
        name: ResourceName,
        #[source]
        error: Box<ResourceSpaceError>,
    },
    #[error("source resource `{name}` was not found in the selected logical folder")]
    MissingSource { name: ResourceName },
    #[error("destination resource `{name}` already exists and collision policy is reject")]
    DestinationExists { name: ResourceName },
    #[error("destination folder `{name}` already exists beneath the selected parent")]
    DestinationFolderExists { name: ResourceName },
    #[error("folder id {folder:?} is already allocated")]
    DestinationFolderIdExists { folder: FolderId },
    #[error("archive entry `{entry}` has no materializable file name")]
    EmptyEntryName { entry: String },
    #[error(
        "archive path component `{name}` is not valid for the destination Resource Space: {error}"
    )]
    InvalidDestinationName { name: String, error: String },
    #[error(
        "archive subtree needs more folder identifiers than the caller-provided range permits"
    )]
    FolderIdRangeExhausted,
    #[error("logical {kind} name `{name}` cannot be lowered to one archive path component")]
    UnsafeArchivePathComponent { kind: &'static str, name: String },
    #[error("archive operation failed: {0}")]
    Archive(#[from] ArchiveError),
    #[error("resource-space mutation failed for destination `{name}`: {error}")]
    Store {
        name: ResourceName,
        #[source]
        error: Box<ResourceSpaceError>,
    },
    #[error("archive-derived view is stale because its source resource changed")]
    DerivedViewStale,
}

/// Lowers the selected folder and its visible caller-selected descendants into
/// deterministic, depth-first archive entries. The source root itself is not
/// emitted as a synthetic archive directory; only its direct children are.
pub fn export_resource_subtree<W: ArchiveWriter>(
    space: &InMemoryResourceSpace,
    request: ExportResourceSubtreeRequest,
    writer: &W,
) -> Result<ResourceSubtreeExport, ResourceArchiveBridgeError> {
    if !writer.supports_write(request.format) {
        return Err(ResourceArchiveBridgeError::WriterUnavailable {
            format: request.format,
        });
    }
    if space.folder(request.source_folder).is_none() {
        return Err(ResourceArchiveBridgeError::SourceFolderNotFound {
            folder: request.source_folder,
        });
    }

    let mut entries = Vec::new();
    let mut counts = SubtreeCounts::default();
    collect_subtree_entries(
        space,
        request.source_folder,
        "",
        request.visibility,
        request.file_compression,
        &mut entries,
        &mut counts,
    )?;
    let result = writer.write_archive(request.format, &entries, request.limits)?;
    Ok(ResourceSubtreeExport {
        source_folder: request.source_folder,
        folders: counts.folders,
        resources: counts.resources,
        archive: result.observation,
        bytes: result.bytes,
    })
}

pub fn inspect_archive_resource<P: ArchiveProvider>(
    space: &InMemoryResourceSpace,
    request: InspectArchiveResourceRequest,
    provider: &P,
) -> Result<ResourceArchiveInspection, ResourceArchiveBridgeError> {
    require_provider(provider, request.format)?;
    let source = source_resource(space, request.source_folder, &request.source_name)?;
    let manifest = provider.inspect(request.format, source.bytes().as_ref(), request.limits)?;
    Ok(ResourceArchiveInspection {
        source: source.key().clone(),
        source_fingerprint: source.content_fingerprint(),
        manifest,
    })
}

/// Opens a read-only, bounded archive projection without materializing its
/// contents into Resource Space. Reopening the view after its source changes
/// yields a new fingerprint; callers compare that value to invalidate any
/// earlier projection they retained.
pub fn open_archive_derived_view<P: ArchiveProvider>(
    space: &InMemoryResourceSpace,
    request: InspectArchiveResourceRequest,
    provider: &P,
) -> Result<ArchiveDerivedView, ResourceArchiveBridgeError> {
    let source_folder = request.source_folder;
    let source_name = request.source_name.clone();
    let inspection = inspect_archive_resource(space, request, provider)?;
    Ok(ArchiveDerivedView {
        source: inspection.source,
        source_fingerprint: inspection.source_fingerprint,
        source_folder,
        source_name,
        format: inspection.manifest.format,
        entries: inspection
            .manifest
            .entries
            .into_iter()
            .map(|entry| ArchiveDerivedEntry { entry })
            .collect(),
    })
}

/// Reads one entry through a derived view without materializing it. The view is
/// valid only while the retained source archive still has the fingerprint that
/// was observed when the view opened.
pub fn read_archive_derived_entry<P: ArchiveProvider>(
    space: &InMemoryResourceSpace,
    view: &ArchiveDerivedView,
    normalized_name: &str,
    limits: ArchiveReadLimits,
    provider: &P,
) -> Result<ArchiveReadResult, ResourceArchiveBridgeError> {
    require_provider(provider, view.format)?;
    let source = source_resource(space, view.source_folder, &view.source_name)?;
    if source.key() != &view.source || source.content_fingerprint() != view.source_fingerprint {
        return Err(ResourceArchiveBridgeError::DerivedViewStale);
    }
    provider
        .read_entry(
            view.format,
            source.bytes().as_ref(),
            normalized_name,
            limits,
        )
        .map_err(ResourceArchiveBridgeError::Archive)
}

/// Reads and validates the selected entry completely before mutating Resource
/// Space. A failed provider operation or rejected collision leaves logical
/// state unchanged.
pub fn copy_archive_entry<P: ArchiveProvider>(
    space: &mut InMemoryResourceSpace,
    request: CopyArchiveEntryRequest,
    provider: &P,
) -> Result<ArchiveEntryCopyResult, ResourceArchiveBridgeError> {
    require_provider(provider, request.format)?;
    let source = source_resource(space, request.source_folder, &request.source_name)?;
    let source_key = source.key().clone();
    let source_fingerprint = source.content_fingerprint();
    let read = provider.read_entry(
        request.format,
        source.bytes().as_ref(),
        &request.entry_name,
        request.limits,
    )?;

    let existing = space
        .resource(request.destination_folder, &request.destination_name)
        .map_err(|error| ResourceArchiveBridgeError::Lookup {
            name: request.destination_name.clone(),
            error: Box::new(error),
        })?;
    let (entry, mutation) = match (existing, request.collision) {
        (Some(_), ArchiveResourceCollisionPolicy::Reject) => {
            return Err(ResourceArchiveBridgeError::DestinationExists {
                name: request.destination_name,
            });
        }
        (Some(_), ArchiveResourceCollisionPolicy::Replace) => (
            space
                .replace_resource(
                    request.destination_folder,
                    &request.destination_name,
                    read.bytes,
                    request.metadata,
                )
                .map_err(|error| ResourceArchiveBridgeError::Store {
                    name: request.destination_name.clone(),
                    error: Box::new(error),
                })?,
            ArchiveResourceMutation::Replaced,
        ),
        (None, _) => (
            space
                .insert_resource(
                    request.destination_folder,
                    request.destination_name.clone(),
                    read.bytes,
                    request.metadata,
                )
                .map_err(|error| ResourceArchiveBridgeError::Store {
                    name: request.destination_name,
                    error: Box::new(error),
                })?,
            ArchiveResourceMutation::Inserted,
        ),
    };

    let observation = ArchiveEntryCopyObservation {
        source: source_key,
        source_fingerprint,
        entry: read.entry,
        result: entry.key().clone(),
        result_fingerprint: entry.content_fingerprint(),
        mutation,
    };
    Ok(ArchiveEntryCopyResult { entry, observation })
}

/// Imports all admitted regular-file entries beneath a newly created explicit
/// destination root. This is an eager materialization operation, not an
/// archive mount: after success, ordinary Resource Space reads return retained
/// entry bytes without consulting the archive provider again.
pub fn import_archive_subtree<P: ArchiveProvider>(
    space: &mut InMemoryResourceSpace,
    request: ImportArchiveSubtreeRequest,
    provider: &P,
) -> Result<ArchiveSubtreeImportObservation, ResourceArchiveBridgeError> {
    require_provider(provider, request.format)?;
    if space.folder(request.destination_parent).is_none() {
        return Err(ResourceArchiveBridgeError::DestinationParentNotFound {
            folder: request.destination_parent,
        });
    }
    ensure_folder_available(
        space,
        request.destination_parent,
        &request.destination_root_name,
    )?;

    let source = source_resource(space, request.source_folder, &request.source_name)?;
    let source_key = source.key().clone();
    let source_fingerprint = source.content_fingerprint();
    let manifest = provider.inspect(request.format, source.bytes().as_ref(), request.limits)?;

    let mut staged_files = Vec::new();
    let mut folder_paths = BTreeSet::new();
    for entry in &manifest.entries {
        let components = archive_entry_components(space, &entry.normalized_name)?;
        match entry.kind {
            archive_provider::ArchiveEntryKind::Directory => {
                folder_paths.insert(component_path(&components));
            }
            archive_provider::ArchiveEntryKind::RegularFile => {
                if components.is_empty() {
                    return Err(ResourceArchiveBridgeError::EmptyEntryName {
                        entry: entry.normalized_name.clone(),
                    });
                }
                for index in 1..components.len() {
                    folder_paths.insert(component_path(&components[..index]));
                }
                let read = provider.read_entry(
                    request.format,
                    source.bytes().as_ref(),
                    &entry.normalized_name,
                    request.limits,
                )?;
                staged_files.push(StagedArchiveFile {
                    components,
                    bytes: read.bytes,
                });
            }
        }
    }

    let folder_count = folder_paths.len().saturating_add(1);
    let mut next_folder = request.first_folder_id.as_u128();
    let destination_root = FolderId::from_u128(next_folder);
    let mut folder_ids = BTreeMap::new();
    for path in &folder_paths {
        next_folder = next_folder
            .checked_add(1)
            .ok_or(ResourceArchiveBridgeError::FolderIdRangeExhausted)?;
        folder_ids.insert(path.clone(), FolderId::from_u128(next_folder));
    }

    for folder in std::iter::once(destination_root).chain(folder_ids.values().copied()) {
        if space.folder(folder).is_some() {
            return Err(ResourceArchiveBridgeError::DestinationFolderIdExists { folder });
        }
    }

    space
        .create_folder(
            destination_root,
            request.destination_parent,
            request.destination_root_name,
            request.metadata.clone(),
        )
        .map_err(|error| store_error("archive root", error))?;

    for path in &folder_paths {
        let components = archive_entry_components(space, path)?;
        let name = components.last().cloned().ok_or_else(|| {
            ResourceArchiveBridgeError::EmptyEntryName {
                entry: path.clone(),
            }
        })?;
        let parent = if components.len() == 1 {
            destination_root
        } else {
            let parent_path = component_path(&components[..components.len() - 1]);
            *folder_ids
                .get(&parent_path)
                .expect("validated parent archive folder must be planned")
        };
        let folder = *folder_ids
            .get(path)
            .expect("every planned archive folder has an allocated identifier");
        space
            .create_folder(folder, parent, name, request.metadata.clone())
            .map_err(|error| store_error("archive folder", error))?;
    }

    let mut retained_bytes = 0_u64;
    for file in staged_files {
        let name = file
            .components
            .last()
            .cloned()
            .expect("validated regular archive file has a final component");
        let parent = if file.components.len() == 1 {
            destination_root
        } else {
            let parent_path = component_path(&file.components[..file.components.len() - 1]);
            *folder_ids
                .get(&parent_path)
                .expect("validated parent archive folder must be planned")
        };
        retained_bytes = retained_bytes.saturating_add(file.bytes.len() as u64);
        space
            .insert_resource(parent, name, file.bytes, request.metadata.clone())
            .map_err(|error| store_error("archive entry", error))?;
    }

    Ok(ArchiveSubtreeImportObservation {
        source: source_key,
        source_fingerprint,
        destination_root,
        folders: folder_count as u32,
        resources: manifest
            .entries
            .iter()
            .filter(|entry| entry.kind == archive_provider::ArchiveEntryKind::RegularFile)
            .count() as u32,
        retained_bytes,
    })
}

fn require_provider<P: ArchiveProvider>(
    provider: &P,
    format: ArchiveFormat,
) -> Result<(), ResourceArchiveBridgeError> {
    if provider.supports(format) {
        Ok(())
    } else {
        Err(ResourceArchiveBridgeError::ProviderUnavailable { format })
    }
}

struct StagedArchiveFile {
    components: Vec<ResourceName>,
    bytes: Vec<u8>,
}

fn ensure_folder_available(
    space: &InMemoryResourceSpace,
    parent: FolderId,
    name: &ResourceName,
) -> Result<(), ResourceArchiveBridgeError> {
    if space
        .list_folders(parent, VisibilityQuery::All)
        .map_err(|error| store_error("archive root", error))?
        .iter()
        .any(|folder| folder.name() == Some(name))
    {
        return Err(ResourceArchiveBridgeError::DestinationFolderExists { name: name.clone() });
    }
    if space
        .resource(parent, name)
        .map_err(|error| ResourceArchiveBridgeError::Lookup {
            name: name.clone(),
            error: Box::new(error),
        })?
        .is_some()
    {
        return Err(ResourceArchiveBridgeError::DestinationExists { name: name.clone() });
    }
    Ok(())
}

fn archive_entry_components(
    space: &InMemoryResourceSpace,
    path: &str,
) -> Result<Vec<ResourceName>, ResourceArchiveBridgeError> {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    trimmed
        .split('/')
        .map(|component| {
            space.resource_name(component).map_err(|error| {
                ResourceArchiveBridgeError::InvalidDestinationName {
                    name: component.to_owned(),
                    error: error.to_string(),
                }
            })
        })
        .collect()
}

fn component_path(components: &[ResourceName]) -> String {
    components
        .iter()
        .map(ResourceName::as_str)
        .collect::<Vec<_>>()
        .join("/")
}

#[derive(Default)]
struct SubtreeCounts {
    folders: u32,
    resources: u32,
}

fn collect_subtree_entries(
    space: &InMemoryResourceSpace,
    folder: FolderId,
    prefix: &str,
    visibility: VisibilityQuery,
    file_compression: archive_provider::ArchiveCompression,
    entries: &mut Vec<ArchiveWriteEntry>,
    counts: &mut SubtreeCounts,
) -> Result<(), ResourceArchiveBridgeError> {
    for resource in space
        .list_resources(folder, visibility)
        .map_err(|error| store_error("subtree", error))?
    {
        let name = archive_component("resource", resource.name())?;
        entries.push(ArchiveWriteEntry::file(
            format!("{prefix}{name}"),
            resource.bytes().as_ref().to_vec(),
            file_compression,
        ));
        counts.resources = counts.resources.saturating_add(1);
    }

    for child in space
        .list_folders(folder, visibility)
        .map_err(|error| store_error("subtree", error))?
    {
        let name = archive_component(
            "folder",
            child
                .name()
                .expect("non-root child folders always have a name"),
        )?;
        let child_prefix = format!("{prefix}{name}/");
        entries.push(ArchiveWriteEntry::directory(child_prefix.clone()));
        counts.folders = counts.folders.saturating_add(1);
        collect_subtree_entries(
            space,
            child.id(),
            &child_prefix,
            visibility,
            file_compression,
            entries,
            counts,
        )?;
    }
    Ok(())
}

fn archive_component<'a>(
    kind: &'static str,
    name: &'a ResourceName,
) -> Result<&'a str, ResourceArchiveBridgeError> {
    let value = name.as_str();
    if value.contains(['/', '\\']) {
        return Err(ResourceArchiveBridgeError::UnsafeArchivePathComponent {
            kind,
            name: value.to_owned(),
        });
    }
    Ok(value)
}

fn store_error(name: &str, error: ResourceSpaceError) -> ResourceArchiveBridgeError {
    ResourceArchiveBridgeError::Store {
        name: ResourceName::parse(name, resource_space::AddressCasePolicy::Sensitive)
            .expect("fixed diagnostic name is valid"),
        error: Box::new(error),
    }
}

fn source_resource(
    space: &InMemoryResourceSpace,
    folder: FolderId,
    name: &ResourceName,
) -> Result<ResourceEntry, ResourceArchiveBridgeError> {
    space
        .resource(folder, name)
        .map_err(|error| ResourceArchiveBridgeError::Lookup {
            name: name.clone(),
            error: Box::new(error),
        })?
        .ok_or_else(|| ResourceArchiveBridgeError::MissingSource { name: name.clone() })
}

#[cfg(test)]
mod tests;
