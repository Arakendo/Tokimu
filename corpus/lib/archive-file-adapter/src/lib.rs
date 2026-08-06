//! Native-only archive file composition.
//!
//! Archive providers own container bytes and entry semantics. This adapter owns
//! the explicitly requested host-file boundary: bounded file reads and atomic
//! create-new publication. It deliberately does not implement replacement,
//! backup naming, directory traversal, or WASM file mechanisms.

use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use archive_provider::{
    ArchiveError, ArchiveFormat, ArchiveManifest, ArchiveProvider, ArchiveReadLimits,
};
use thiserror::Error;

static TEMPORARY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Native filesystem composition for archive bytes.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeArchiveFileAdapter;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeFileRead {
    bytes: Vec<u8>,
    observation: NativeFileReadObservation,
}

impl NativeFileRead {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    pub fn observation(&self) -> &NativeFileReadObservation {
        &self.observation
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeFileReadObservation {
    path: PathBuf,
    bytes: u64,
}

impl NativeFileReadObservation {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub const fn bytes(&self) -> u64 {
        self.bytes
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeArchiveInspection {
    pub file: NativeFileReadObservation,
    pub manifest: ArchiveManifest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeFileWriteObservation {
    destination: PathBuf,
    bytes: u64,
    publication: NativeFilePublication,
}

impl NativeFileWriteObservation {
    pub fn destination(&self) -> &Path {
        &self.destination
    }

    pub const fn bytes(&self) -> u64 {
        self.bytes
    }

    pub const fn publication(&self) -> NativeFilePublication {
        self.publication
    }
}

/// The only publication behavior currently admitted by this adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeFilePublication {
    AtomicCreateNew,
}

#[derive(Debug, Error)]
pub enum NativeArchiveFileError {
    #[error("native input path `{path}` is not a regular file")]
    NotRegularFile { path: PathBuf },
    #[error("native file `{path}` is {actual_bytes} bytes; limit is {limit_bytes} bytes")]
    InputLimitExceeded {
        path: PathBuf,
        actual_bytes: u64,
        limit_bytes: u64,
    },
    #[error("native output is {actual_bytes} bytes; limit is {limit_bytes} bytes")]
    OutputLimitExceeded { actual_bytes: u64, limit_bytes: u64 },
    #[error("native output destination `{path}` already exists; replacement is not implicit")]
    DestinationExists { path: PathBuf },
    #[error("native output destination `{path}` has no file name")]
    DestinationHasNoFileName { path: PathBuf },
    #[error("native {operation} failed for `{path}`: {source}")]
    HostIo {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Archive(#[from] ArchiveError),
}

impl NativeArchiveFileAdapter {
    /// Reads one regular host file without allowing it to exceed `max_bytes`.
    pub fn read(
        &self,
        path: impl AsRef<Path>,
        max_bytes: u64,
    ) -> Result<NativeFileRead, NativeArchiveFileError> {
        let path = path.as_ref().to_path_buf();
        let metadata = fs::metadata(&path).map_err(|source| NativeArchiveFileError::HostIo {
            operation: "inspect input metadata",
            path: path.clone(),
            source,
        })?;
        if !metadata.is_file() {
            return Err(NativeArchiveFileError::NotRegularFile { path });
        }

        let file = File::open(&path).map_err(|source| NativeArchiveFileError::HostIo {
            operation: "open input",
            path: path.clone(),
            source,
        })?;
        let bounded = max_bytes.saturating_add(1);
        let mut reader = file.take(bounded);
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .map_err(|source| NativeArchiveFileError::HostIo {
                operation: "read input",
                path: path.clone(),
                source,
            })?;
        let actual_bytes = bytes.len() as u64;
        if actual_bytes > max_bytes {
            return Err(NativeArchiveFileError::InputLimitExceeded {
                path,
                actual_bytes,
                limit_bytes: max_bytes,
            });
        }

        Ok(NativeFileRead {
            observation: NativeFileReadObservation {
                path,
                bytes: actual_bytes,
            },
            bytes,
        })
    }

    /// Reads bounded host bytes, then delegates all archive semantics to the
    /// selected provider.
    pub fn inspect_archive<P: ArchiveProvider>(
        &self,
        provider: &P,
        format: ArchiveFormat,
        path: impl AsRef<Path>,
        limits: ArchiveReadLimits,
    ) -> Result<NativeArchiveInspection, NativeArchiveFileError> {
        let file = self.read(path, limits.max_archive_bytes)?;
        let manifest = provider.inspect(format, file.bytes(), limits)?;
        Ok(NativeArchiveInspection {
            file: file.observation,
            manifest,
        })
    }

    /// Atomically publishes bytes only when `destination` does not yet exist.
    ///
    /// A same-directory temporary file is written and synced first. A hard-link
    /// creation then publishes the destination without replacing an existing
    /// file. Replacement, backups, and recursive output belong to an explicit
    /// higher-level platform policy rather than this adapter.
    pub fn publish_new(
        &self,
        destination: impl AsRef<Path>,
        bytes: &[u8],
        max_bytes: u64,
    ) -> Result<NativeFileWriteObservation, NativeArchiveFileError> {
        let destination = destination.as_ref().to_path_buf();
        let actual_bytes = bytes.len() as u64;
        if actual_bytes > max_bytes {
            return Err(NativeArchiveFileError::OutputLimitExceeded {
                actual_bytes,
                limit_bytes: max_bytes,
            });
        }
        if destination.exists() {
            return Err(NativeArchiveFileError::DestinationExists { path: destination });
        }

        let temporary = temporary_path(&destination)?;
        let write_result = (|| -> Result<(), NativeArchiveFileError> {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
                .map_err(|source| NativeArchiveFileError::HostIo {
                    operation: "create temporary output",
                    path: temporary.clone(),
                    source,
                })?;
            file.write_all(bytes)
                .map_err(|source| NativeArchiveFileError::HostIo {
                    operation: "write temporary output",
                    path: temporary.clone(),
                    source,
                })?;
            file.sync_all()
                .map_err(|source| NativeArchiveFileError::HostIo {
                    operation: "sync temporary output",
                    path: temporary.clone(),
                    source,
                })?;
            fs::hard_link(&temporary, &destination).map_err(|source| {
                if source.kind() == std::io::ErrorKind::AlreadyExists {
                    NativeArchiveFileError::DestinationExists {
                        path: destination.clone(),
                    }
                } else {
                    NativeArchiveFileError::HostIo {
                        operation: "publish create-new output",
                        path: destination.clone(),
                        source,
                    }
                }
            })?;
            Ok(())
        })();

        let cleanup = fs::remove_file(&temporary);
        write_result?;
        cleanup.map_err(|source| NativeArchiveFileError::HostIo {
            operation: "remove published temporary output",
            path: temporary,
            source,
        })?;

        Ok(NativeFileWriteObservation {
            destination,
            bytes: actual_bytes,
            publication: NativeFilePublication::AtomicCreateNew,
        })
    }
}

fn temporary_path(destination: &Path) -> Result<PathBuf, NativeArchiveFileError> {
    let name = destination
        .file_name()
        .ok_or_else(|| NativeArchiveFileError::DestinationHasNoFileName {
            path: destination.to_path_buf(),
        })?
        .to_string_lossy();
    let sequence = TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let temporary_name = format!(".{name}.tokimu-partial-{}-{sequence}", std::process::id());
    Ok(destination.with_file_name(temporary_name))
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use archive_provider::{
        ArchiveEntryKind, ArchiveError, ArchiveWriteEntry, ArchiveWriteLimits, ArchiveWriter,
        ZipArchiveProvider,
    };

    use super::{
        ArchiveFormat, NativeArchiveFileAdapter, NativeArchiveFileError, NativeFilePublication,
    };

    fn test_path(name: &str) -> PathBuf {
        let sequence = super::TEMPORARY_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "tokimu-archive-file-adapter-{name}-{}-{sequence}",
            std::process::id()
        ))
    }

    #[test]
    fn native_publish_is_create_new_and_cleans_its_temporary_file() {
        let adapter = NativeArchiveFileAdapter;
        let path = test_path("publish.zip");
        let observation = adapter.publish_new(&path, b"archive bytes", 64).unwrap();

        assert_eq!(
            observation.publication(),
            NativeFilePublication::AtomicCreateNew
        );
        assert_eq!(fs::read(&path).unwrap(), b"archive bytes");
        let partial_prefix = ".publish.zip.tokimu-partial-";
        assert!(!fs::read_dir(path.parent().unwrap())
            .unwrap()
            .flatten()
            .any(|entry| entry
                .file_name()
                .to_string_lossy()
                .starts_with(partial_prefix)));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn native_publish_never_replaces_an_existing_destination() {
        let adapter = NativeArchiveFileAdapter;
        let path = test_path("collision.zip");
        fs::write(&path, b"original").unwrap();

        let error = adapter.publish_new(&path, b"replacement", 64).unwrap_err();
        assert!(matches!(
            error,
            NativeArchiveFileError::DestinationExists { .. }
        ));
        assert_eq!(fs::read(&path).unwrap(), b"original");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn native_reads_enforce_a_bound_before_archive_inspection() {
        let adapter = NativeArchiveFileAdapter;
        let path = test_path("limit.zip");
        fs::write(&path, b"too large").unwrap();

        let error = adapter.read(&path, 3).unwrap_err();
        assert!(matches!(
            error,
            NativeArchiveFileError::InputLimitExceeded { .. }
        ));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn native_publish_enforces_an_output_bound_before_creating_a_file() {
        let adapter = NativeArchiveFileAdapter;
        let path = test_path("output-limit.zip");

        let error = adapter.publish_new(&path, b"too large", 3).unwrap_err();
        assert!(matches!(
            error,
            NativeArchiveFileError::OutputLimitExceeded { .. }
        ));
        assert!(!path.exists());
    }

    #[test]
    fn native_missing_input_is_reported_as_a_host_failure() {
        let adapter = NativeArchiveFileAdapter;
        let path = test_path("missing.zip");

        let error = adapter.read(&path, 64).unwrap_err();
        assert!(matches!(
            error,
            NativeArchiveFileError::HostIo {
                operation: "inspect input metadata",
                ..
            }
        ));
    }

    #[test]
    fn native_directories_are_not_treated_as_archive_files() {
        let adapter = NativeArchiveFileAdapter;
        let path = test_path("directory");
        fs::create_dir(&path).unwrap();

        let error = adapter.read(&path, 64).unwrap_err();
        assert!(matches!(
            error,
            NativeArchiveFileError::NotRegularFile { .. }
        ));
        fs::remove_dir(path).unwrap();
    }

    #[test]
    fn native_inspection_delegates_container_semantics_to_the_provider() {
        let provider = ZipArchiveProvider;
        let archive = provider
            .write_archive(
                ArchiveFormat::Zip,
                &[ArchiveWriteEntry::file(
                    "notes.txt",
                    b"Tokimu archive adapter",
                    archive_provider::ArchiveCompression::Stored,
                )],
                ArchiveWriteLimits::default(),
            )
            .unwrap();
        let adapter = NativeArchiveFileAdapter;
        let path = test_path("inspect.zip");
        adapter
            .publish_new(&path, &archive.bytes, 1024 * 1024)
            .unwrap();

        let inspection = adapter
            .inspect_archive(&provider, ArchiveFormat::Zip, &path, Default::default())
            .unwrap();
        assert_eq!(inspection.file.bytes(), archive.bytes.len() as u64);
        assert_eq!(inspection.manifest.entries.len(), 1);
        assert_eq!(
            inspection.manifest.entries[0].kind,
            ArchiveEntryKind::RegularFile
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn native_inspection_preserves_archive_failures_after_file_reading() {
        let adapter = NativeArchiveFileAdapter;
        let path = test_path("malformed.zip");
        fs::write(&path, b"not a zip archive").unwrap();

        let error = adapter
            .inspect_archive(
                &ZipArchiveProvider,
                ArchiveFormat::Zip,
                &path,
                Default::default(),
            )
            .unwrap_err();
        assert!(matches!(
            error,
            NativeArchiveFileError::Archive(
                ArchiveError::MalformedArchive { .. } | ArchiveError::TruncatedArchive { .. }
            )
        ));
        fs::remove_file(path).unwrap();
    }
}
