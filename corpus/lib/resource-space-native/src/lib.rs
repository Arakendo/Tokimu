//! Explicit native filesystem import and export for Resource Space.
//!
//! Host paths, hidden-file conventions, and sandbox containment belong here,
//! not in the provider-neutral resource-space semantic contract.

use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

use resource_space::{
    FolderId, InMemoryResourceSpace, ResourceMetadata, ResourceSpaceError, ResourceVisibility,
    VisibilityQuery,
};
use thiserror::Error;

/// How a native import treats dot-prefixed and platform-hidden entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeHiddenEntryPolicy {
    Include,
    Skip,
    Reject,
}

/// Explicit import behavior for native directory traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeImportPolicy {
    pub hidden_entries: NativeHiddenEntryPolicy,
    pub preserve_empty_directories: bool,
    pub reject_symbolic_links: bool,
}

impl Default for NativeImportPolicy {
    fn default() -> Self {
        Self {
            hidden_entries: NativeHiddenEntryPolicy::Include,
            preserve_empty_directories: true,
            reject_symbolic_links: true,
        }
    }
}

/// Caller-owned folder ID allocation for one import session.
///
/// The semantic store deliberately does not generate global identity. The
/// adapter therefore accepts an explicit local source for imported folders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeImportIds {
    next_folder: u128,
}

impl NativeImportIds {
    pub const fn starting_at(next_folder: u128) -> Self {
        Self { next_folder }
    }

    fn next_folder(&mut self) -> Result<FolderId, NativeResourceAdapterError> {
        let current = self.next_folder;
        self.next_folder = self
            .next_folder
            .checked_add(1)
            .ok_or(NativeResourceAdapterError::FolderIdExhausted)?;
        Ok(FolderId::from_u128(current))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeImportDiagnosticKind {
    HiddenSkipped,
    HiddenRejected,
    SymbolicLinkRejected,
    InvalidLogicalName,
    FolderRejected,
    ResourceRejected,
    Io,
}

/// A bounded per-entry observation from native import.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeImportDiagnostic {
    kind: NativeImportDiagnosticKind,
    path: PathBuf,
    message: String,
}

impl NativeImportDiagnostic {
    pub const fn kind(&self) -> &NativeImportDiagnosticKind {
        &self.kind
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Default)]
pub struct NativeImportReport {
    imported_folders: usize,
    imported_resources: usize,
    diagnostics: Vec<NativeImportDiagnostic>,
}

impl NativeImportReport {
    pub const fn imported_folders(&self) -> usize {
        self.imported_folders
    }

    pub const fn imported_resources(&self) -> usize {
        self.imported_resources
    }

    pub fn diagnostics(&self) -> &[NativeImportDiagnostic] {
        &self.diagnostics
    }
}

#[derive(Debug, Default)]
pub struct NativeExportReport {
    exported_folders: usize,
    exported_resources: usize,
}

impl NativeExportReport {
    pub const fn exported_folders(&self) -> usize {
        self.exported_folders
    }

    pub const fn exported_resources(&self) -> usize {
        self.exported_resources
    }
}

#[derive(Debug, Error)]
pub enum NativeResourceAdapterError {
    #[error("native import source {path} is not a directory")]
    ImportSourceNotDirectory { path: PathBuf },
    #[error("approved export root {path} is not a directory")]
    ExportRootNotDirectory { path: PathBuf },
    #[error("native import folder IDs are exhausted")]
    FolderIdExhausted,
    #[error("native filesystem operation failed for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error(transparent)]
    ResourceSpace(#[from] ResourceSpaceError),
}

/// Imports one selected native directory into an explicit logical target.
///
/// The source directory itself maps to `target_folder`; only its children are
/// represented beneath that target. Every rejected host entry is reported
/// rather than silently disappearing.
pub fn import_directory(
    space: &mut InMemoryResourceSpace,
    target_folder: FolderId,
    source_directory: impl AsRef<Path>,
    policy: NativeImportPolicy,
    ids: &mut NativeImportIds,
) -> Result<NativeImportReport, NativeResourceAdapterError> {
    let source_directory = source_directory.as_ref();
    if !source_directory.is_dir() {
        return Err(NativeResourceAdapterError::ImportSourceNotDirectory {
            path: source_directory.to_path_buf(),
        });
    }
    if space.folder(target_folder).is_none() {
        return Err(ResourceSpaceError::FolderNotFound {
            folder: target_folder,
        }
        .into());
    }

    let mut report = NativeImportReport::default();
    import_children(
        space,
        target_folder,
        source_directory,
        policy,
        ids,
        &mut report,
    )?;
    Ok(report)
}

/// Exports an explicit logical folder beneath an approved native directory.
///
/// Export never derives its destination from a resource address supplied by a
/// caller. Each parent is created or canonicalized and verified to remain
/// within `approved_root` before a file is written.
pub fn export_folder(
    space: &InMemoryResourceSpace,
    source_folder: FolderId,
    approved_root: impl AsRef<Path>,
) -> Result<NativeExportReport, NativeResourceAdapterError> {
    if space.folder(source_folder).is_none() {
        return Err(ResourceSpaceError::FolderNotFound {
            folder: source_folder,
        }
        .into());
    }

    let approved_root = approved_root.as_ref();
    if !approved_root.is_dir() {
        return Err(NativeResourceAdapterError::ExportRootNotDirectory {
            path: approved_root.to_path_buf(),
        });
    }
    let canonical_root = canonical_directory(approved_root)?;
    let mut report = NativeExportReport::default();
    export_children(
        space,
        source_folder,
        &canonical_root,
        &canonical_root,
        &mut report,
    )?;
    Ok(report)
}

fn import_children(
    space: &mut InMemoryResourceSpace,
    target_folder: FolderId,
    source_directory: &Path,
    policy: NativeImportPolicy,
    ids: &mut NativeImportIds,
    report: &mut NativeImportReport,
) -> Result<(), NativeResourceAdapterError> {
    let mut children = fs::read_dir(source_directory)
        .map_err(|source| NativeResourceAdapterError::Io {
            path: source_directory.to_path_buf(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| NativeResourceAdapterError::Io {
            path: source_directory.to_path_buf(),
            source,
        })?;
    children.sort_by_key(|entry| entry.file_name());

    for child in children {
        let path = child.path();
        let file_name = child.file_name();
        let file_name = file_name.to_string_lossy();
        let metadata = child
            .metadata()
            .map_err(|source| NativeResourceAdapterError::Io {
                path: path.clone(),
                source,
            })?;
        if is_hidden(&path, &file_name, &metadata) {
            match policy.hidden_entries {
                NativeHiddenEntryPolicy::Include => {}
                NativeHiddenEntryPolicy::Skip => {
                    push_diagnostic(
                        report,
                        NativeImportDiagnosticKind::HiddenSkipped,
                        path,
                        "hidden host entry skipped by explicit import policy",
                    );
                    continue;
                }
                NativeHiddenEntryPolicy::Reject => {
                    push_diagnostic(
                        report,
                        NativeImportDiagnosticKind::HiddenRejected,
                        path,
                        "hidden host entry rejected by explicit import policy",
                    );
                    continue;
                }
            }
        }

        let file_type = child
            .file_type()
            .map_err(|source| NativeResourceAdapterError::Io {
                path: path.clone(),
                source,
            })?;
        if policy.reject_symbolic_links && file_type.is_symlink() {
            push_diagnostic(
                report,
                NativeImportDiagnosticKind::SymbolicLinkRejected,
                path,
                "symbolic link rejected by explicit import policy",
            );
            continue;
        }

        let name = match space.resource_name(&file_name) {
            Ok(name) => name,
            Err(error) => {
                push_diagnostic(
                    report,
                    NativeImportDiagnosticKind::InvalidLogicalName,
                    path,
                    error.to_string(),
                );
                continue;
            }
        };
        let resource_metadata = ResourceMetadata {
            visibility: if is_hidden(&path, &file_name, &metadata) {
                ResourceVisibility::Hidden
            } else {
                ResourceVisibility::Visible
            },
            media_type: media_type_for_path(&path),
            ..ResourceMetadata::default()
        };

        if metadata.is_dir() {
            let folder_id = ids.next_folder()?;
            if let Err(error) =
                space.create_folder(folder_id, target_folder, name, resource_metadata)
            {
                push_diagnostic(
                    report,
                    NativeImportDiagnosticKind::FolderRejected,
                    path,
                    error.to_string(),
                );
                continue;
            }
            report.imported_folders += 1;
            import_children(space, folder_id, &child.path(), policy, ids, report)?;
            if !policy.preserve_empty_directories
                && space
                    .list_folders(folder_id, VisibilityQuery::All)?
                    .is_empty()
                && space
                    .list_resources(folder_id, VisibilityQuery::All)?
                    .is_empty()
            {
                space.remove_empty_folder(folder_id)?;
                report.imported_folders = report
                    .imported_folders
                    .checked_sub(1)
                    .expect("created folder count remains positive while pruning");
            }
        } else if metadata.is_file() {
            let bytes = match fs::read(&path) {
                Ok(bytes) => bytes,
                Err(error) => {
                    push_diagnostic(
                        report,
                        NativeImportDiagnosticKind::Io,
                        path,
                        error.to_string(),
                    );
                    continue;
                }
            };
            if let Err(error) = space.insert_resource(
                target_folder,
                name,
                Arc::<[u8]>::from(bytes),
                resource_metadata,
            ) {
                push_diagnostic(
                    report,
                    NativeImportDiagnosticKind::ResourceRejected,
                    path,
                    error.to_string(),
                );
                continue;
            }
            report.imported_resources += 1;
        }
    }
    Ok(())
}

fn export_children(
    space: &InMemoryResourceSpace,
    source_folder: FolderId,
    output_directory: &Path,
    approved_root: &Path,
    report: &mut NativeExportReport,
) -> Result<(), NativeResourceAdapterError> {
    verify_contained(output_directory, approved_root)?;
    for resource in space.list_resources(source_folder, VisibilityQuery::All)? {
        let output_path = output_directory.join(resource.name().as_str());
        verify_output_parent(&output_path, approved_root)?;
        fs::write(&output_path, resource.bytes()).map_err(|source| {
            NativeResourceAdapterError::Io {
                path: output_path,
                source,
            }
        })?;
        report.exported_resources += 1;
    }
    for folder in space.list_folders(source_folder, VisibilityQuery::All)? {
        let name = folder.name().expect("non-root child folders have a name");
        let child_output = output_directory.join(name.as_str());
        fs::create_dir_all(&child_output).map_err(|source| NativeResourceAdapterError::Io {
            path: child_output.clone(),
            source,
        })?;
        let child_output = canonical_directory(&child_output)?;
        verify_contained(&child_output, approved_root)?;
        report.exported_folders += 1;
        export_children(space, folder.id(), &child_output, approved_root, report)?;
    }
    Ok(())
}

fn canonical_directory(path: &Path) -> Result<PathBuf, NativeResourceAdapterError> {
    fs::canonicalize(path).map_err(|source| NativeResourceAdapterError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn verify_output_parent(
    path: &Path,
    approved_root: &Path,
) -> Result<(), NativeResourceAdapterError> {
    let parent = path
        .parent()
        .expect("joined output paths retain their parent");
    verify_contained(parent, approved_root)
}

fn verify_contained(path: &Path, approved_root: &Path) -> Result<(), NativeResourceAdapterError> {
    let canonical = canonical_directory(path)?;
    if canonical.starts_with(approved_root) {
        Ok(())
    } else {
        Err(NativeResourceAdapterError::Io {
            path: canonical,
            source: io::Error::new(
                io::ErrorKind::PermissionDenied,
                "export path escaped approved root",
            ),
        })
    }
}

fn push_diagnostic(
    report: &mut NativeImportReport,
    kind: NativeImportDiagnosticKind,
    path: PathBuf,
    message: impl Into<String>,
) {
    report.diagnostics.push(NativeImportDiagnostic {
        kind,
        path,
        message: message.into(),
    });
}

fn media_type_for_path(path: &Path) -> Option<String> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    let media_type = match extension.as_str() {
        "gltf" => "model/gltf+json",
        "glb" => "model/gltf-binary",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "json" => "application/json",
        "xml" => "application/xml",
        _ => return None,
    };
    Some(media_type.to_owned())
}

fn is_hidden(_path: &Path, file_name: &str, metadata: &fs::Metadata) -> bool {
    if file_name.starts_with('.') {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
        metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0
    }
    #[cfg(not(windows))]
    {
        let _ = metadata;
        false
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use resource_space::{AddressCasePolicy, ResourceRootDescriptor, ResourceRootId, StoreId};

    use super::*;

    fn unique_temp_directory(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        std::env::temp_dir().join(format!("tokimu-resource-space-{label}-{nonce}"))
    }

    fn fixture_space() -> (InMemoryResourceSpace, FolderId) {
        let mut space =
            InMemoryResourceSpace::new(StoreId::from_u128(1), AddressCasePolicy::Sensitive);
        let root = FolderId::from_u128(2);
        space
            .create_root(
                ResourceRootDescriptor::new(ResourceRootId::from_u128(3), "fixture"),
                root,
                ResourceMetadata::default(),
            )
            .expect("root");
        (space, root)
    }

    #[test]
    fn import_preserves_empty_directories_and_explicit_hidden_policy() {
        let source = unique_temp_directory("import");
        fs::create_dir_all(source.join("empty")).expect("empty folder");
        fs::write(source.join("visible.svg"), b"<svg/>").expect("visible fixture");
        fs::write(source.join(".hidden.txt"), b"hidden").expect("hidden fixture");

        let (mut space, root) = fixture_space();
        let mut ids = NativeImportIds::starting_at(10);
        let report = import_directory(
            &mut space,
            root,
            &source,
            NativeImportPolicy {
                hidden_entries: NativeHiddenEntryPolicy::Skip,
                ..NativeImportPolicy::default()
            },
            &mut ids,
        )
        .expect("import");

        assert_eq!(report.imported_folders(), 1);
        assert_eq!(report.imported_resources(), 1);
        assert!(report
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.kind() == &NativeImportDiagnosticKind::HiddenSkipped));
        assert_eq!(
            space
                .list_folders(root, VisibilityQuery::All)
                .expect("folders")
                .len(),
            1
        );
        assert_eq!(
            space
                .list_resources(root, VisibilityQuery::All)
                .expect("resources")
                .len(),
            1
        );

        fs::remove_dir_all(&source).expect("remove uniquely-created temp fixture");
    }

    #[test]
    fn import_can_discard_empty_directories_when_policy_requests_it() {
        let source = unique_temp_directory("discard-empty");
        fs::create_dir_all(source.join("empty/nested")).expect("empty nested folder");
        fs::write(source.join("visible.svg"), b"<svg/>").expect("visible fixture");

        let (mut space, root) = fixture_space();
        let mut ids = NativeImportIds::starting_at(10);
        let report = import_directory(
            &mut space,
            root,
            &source,
            NativeImportPolicy {
                preserve_empty_directories: false,
                ..NativeImportPolicy::default()
            },
            &mut ids,
        )
        .expect("import");

        assert_eq!(report.imported_folders(), 0);
        assert_eq!(report.imported_resources(), 1);
        assert!(space
            .list_folders(root, VisibilityQuery::All)
            .expect("folders")
            .is_empty());

        fs::remove_dir_all(&source).expect("remove uniquely-created temp fixture");
    }

    #[test]
    fn export_stays_under_the_explicit_approved_root() {
        let output = unique_temp_directory("export");
        fs::create_dir_all(&output).expect("output root");
        let (mut space, root) = fixture_space();
        let child = FolderId::from_u128(4);
        space
            .create_folder(
                child,
                root,
                space.resource_name("assets").expect("folder name"),
                ResourceMetadata::default(),
            )
            .expect("child folder");
        space
            .insert_resource(
                child,
                space.resource_name("scene.json").expect("resource name"),
                Arc::<[u8]>::from(&b"{}"[..]),
                ResourceMetadata::default(),
            )
            .expect("resource");

        let report = export_folder(&space, root, &output).expect("export");
        assert_eq!(report.exported_folders(), 1);
        assert_eq!(report.exported_resources(), 1);
        assert_eq!(
            fs::read(output.join("assets/scene.json")).expect("exported bytes"),
            b"{}"
        );

        fs::remove_dir_all(&output).expect("remove uniquely-created temp fixture");
    }
}
