use std::collections::HashSet;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use zip::result::ZipError;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};

use crate::name::normalize_entry_name;
use crate::{
    ArchiveCompression, ArchiveEntryKind, ArchiveEntryObservation, ArchiveError, ArchiveFormat,
    ArchiveManifest, ArchiveProvider, ArchiveReadLimits, ArchiveReadResult, ArchiveWriteEntry,
    ArchiveWriteLimits, ArchiveWriteObservation, ArchiveWriteResult, ArchiveWriter,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct ZipArchiveProvider;

impl ArchiveProvider for ZipArchiveProvider {
    fn supports(&self, format: ArchiveFormat) -> bool {
        format == ArchiveFormat::Zip
    }

    fn inspect(
        &self,
        format: ArchiveFormat,
        archive: &[u8],
        limits: ArchiveReadLimits,
    ) -> Result<ArchiveManifest, ArchiveError> {
        require_zip(format)?;
        validate_archive_size(archive, limits)?;
        inspect_zip(archive, limits)
    }

    fn read_entry(
        &self,
        format: ArchiveFormat,
        archive: &[u8],
        normalized_name: &str,
        limits: ArchiveReadLimits,
    ) -> Result<ArchiveReadResult, ArchiveError> {
        require_zip(format)?;
        validate_archive_size(archive, limits)?;
        let manifest = inspect_zip(archive, limits)?;
        let expected = manifest
            .entries
            .into_iter()
            .find(|entry| entry.normalized_name == normalized_name)
            .ok_or_else(|| ArchiveError::EntryNotFound {
                name: normalized_name.to_owned(),
            })?;
        if expected.kind != ArchiveEntryKind::RegularFile {
            return Err(ArchiveError::UnsupportedEntryKind {
                name: expected.normalized_name,
                kind: "directory".to_owned(),
            });
        }

        let mut zip = open_zip(archive)?;
        let mut file = zip
            .by_index(expected.index as usize)
            .map_err(|error| map_zip_error(error, Some(expected.original_name.as_str())))?;
        if file.encrypted() {
            return Err(ArchiveError::EncryptedEntry {
                name: expected.original_name,
            });
        }

        let capacity = usize::try_from(expected.uncompressed_bytes).map_err(|_| {
            ArchiveError::EntryLimitExceeded {
                name: expected.normalized_name.clone(),
                actual_bytes: expected.uncompressed_bytes,
                limit_bytes: limits.max_entry_bytes,
            }
        })?;
        let mut bytes = Vec::with_capacity(capacity);
        let mut bounded = (&mut file).take(limits.max_entry_bytes.saturating_add(1));
        bounded
            .read_to_end(&mut bytes)
            .map_err(|error| map_read_error(error, expected.original_name.as_str()))?;
        if bytes.len() as u64 > limits.max_entry_bytes {
            return Err(ArchiveError::EntryLimitExceeded {
                name: expected.normalized_name,
                actual_bytes: bytes.len() as u64,
                limit_bytes: limits.max_entry_bytes,
            });
        }

        Ok(ArchiveReadResult {
            entry: expected,
            bytes,
        })
    }
}

impl ArchiveWriter for ZipArchiveProvider {
    fn supports_write(&self, format: ArchiveFormat) -> bool {
        format == ArchiveFormat::Zip
    }

    fn write_archive(
        &self,
        format: ArchiveFormat,
        entries: &[ArchiveWriteEntry],
        limits: ArchiveWriteLimits,
    ) -> Result<ArchiveWriteResult, ArchiveError> {
        require_zip(format)?;
        validate_write_entries(entries, limits)?;

        let cursor = BoundedCursor::new(limits.max_archive_bytes);
        let mut zip = ZipWriter::new(cursor);
        for entry in entries {
            let normalized_name = normalize_entry_name(&entry.name, limits.max_path_bytes)?;
            let options = deterministic_options(entry)?;
            match entry.kind {
                ArchiveEntryKind::RegularFile => {
                    zip.start_file(&normalized_name, options)
                        .map_err(|error| map_write_error(error, limits.max_archive_bytes))?;
                    zip.write_all(&entry.bytes)
                        .map_err(|error| map_write_io_error(error, limits.max_archive_bytes))?;
                }
                ArchiveEntryKind::Directory => {
                    zip.add_directory(&normalized_name, options)
                        .map_err(|error| map_write_error(error, limits.max_archive_bytes))?;
                }
            }
        }

        let bytes = zip
            .finish()
            .map_err(|error| map_write_error(error, limits.max_archive_bytes))?
            .into_inner();
        let total_input_bytes = entries.iter().map(|entry| entry.bytes.len() as u64).sum();
        Ok(ArchiveWriteResult {
            observation: ArchiveWriteObservation {
                format,
                archive_bytes: bytes.len() as u64,
                entry_count: entries.len() as u32,
                total_input_bytes,
                deterministic_metadata: true,
            },
            bytes,
        })
    }
}

fn validate_write_entries(
    entries: &[ArchiveWriteEntry],
    limits: ArchiveWriteLimits,
) -> Result<(), ArchiveError> {
    if entries.len() as u64 > u64::from(limits.max_entries) {
        return Err(ArchiveError::EntryCountLimitExceeded {
            actual_entries: entries.len() as u64,
            limit_entries: limits.max_entries,
        });
    }

    let mut names = HashSet::new();
    let mut total_input_bytes = 0_u64;
    let mut estimated_archive_bytes = 128_u64;
    for entry in entries {
        let normalized_name = normalize_entry_name(&entry.name, limits.max_path_bytes)?;
        if !names.insert(normalized_name.clone()) {
            return Err(ArchiveError::DuplicateEntryName { normalized_name });
        }
        if entry.bytes.len() as u64 > limits.max_entry_bytes {
            return Err(ArchiveError::EntryLimitExceeded {
                name: normalized_name,
                actual_bytes: entry.bytes.len() as u64,
                limit_bytes: limits.max_entry_bytes,
            });
        }
        total_input_bytes = total_input_bytes.saturating_add(entry.bytes.len() as u64);
        if total_input_bytes > limits.max_total_input_bytes {
            return Err(ArchiveError::TotalOutputLimitExceeded {
                actual_bytes: total_input_bytes,
                limit_bytes: limits.max_total_input_bytes,
            });
        }
        let compression_overhead = match entry.compression {
            ArchiveCompression::Deflate => {
                (entry.bytes.len() as u64 / (16 * 1024) + 1).saturating_mul(5)
            }
            ArchiveCompression::Stored | ArchiveCompression::Other => 0,
        };
        estimated_archive_bytes = estimated_archive_bytes
            .saturating_add(entry.bytes.len() as u64)
            .saturating_add(compression_overhead)
            .saturating_add((normalized_name.len() as u64).saturating_mul(2))
            .saturating_add(160);
        if estimated_archive_bytes > limits.max_archive_bytes {
            return Err(ArchiveError::OutputLimitExceeded {
                limit_bytes: limits.max_archive_bytes,
            });
        }

        match entry.kind {
            ArchiveEntryKind::RegularFile if normalized_name.ends_with('/') => {
                return Err(ArchiveError::InvalidWriteEntry {
                    name: entry.name.clone(),
                    reason: "regular file names must not end with `/`".to_owned(),
                });
            }
            ArchiveEntryKind::Directory if !entry.bytes.is_empty() => {
                return Err(ArchiveError::InvalidWriteEntry {
                    name: entry.name.clone(),
                    reason: "directory entries cannot contain bytes".to_owned(),
                });
            }
            _ => {}
        }
    }
    Ok(())
}

fn deterministic_options(entry: &ArchiveWriteEntry) -> Result<SimpleFileOptions, ArchiveError> {
    let compression = match entry.compression {
        ArchiveCompression::Stored => CompressionMethod::Stored,
        ArchiveCompression::Deflate => CompressionMethod::Deflated,
        ArchiveCompression::Other => {
            return Err(ArchiveError::UnsupportedWriteCompression {
                name: entry.name.clone(),
                compression: entry.compression,
            });
        }
    };
    let permissions = match entry.kind {
        ArchiveEntryKind::RegularFile => 0o644,
        ArchiveEntryKind::Directory => 0o755,
    };
    Ok(SimpleFileOptions::default()
        .compression_method(compression)
        .last_modified_time(DateTime::default())
        .unix_permissions(permissions))
}

const OUTPUT_LIMIT_DIAGNOSTIC: &str = "archive output limit exceeded";

struct BoundedCursor {
    inner: Cursor<Vec<u8>>,
    max_bytes: u64,
}

impl BoundedCursor {
    fn new(max_bytes: u64) -> Self {
        Self {
            inner: Cursor::new(Vec::new()),
            max_bytes,
        }
    }

    fn into_inner(self) -> Vec<u8> {
        self.inner.into_inner()
    }
}

impl Write for BoundedCursor {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        let end = self.inner.position().saturating_add(buffer.len() as u64);
        if end > self.max_bytes {
            return Err(std::io::Error::other(OUTPUT_LIMIT_DIAGNOSTIC));
        }
        self.inner.write(buffer)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl Seek for BoundedCursor {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        let previous = self.inner.position();
        let next = self.inner.seek(position)?;
        if next > self.max_bytes {
            self.inner.set_position(previous);
            return Err(std::io::Error::other(OUTPUT_LIMIT_DIAGNOSTIC));
        }
        Ok(next)
    }
}

fn map_write_error(error: ZipError, limit_bytes: u64) -> ArchiveError {
    match error {
        ZipError::Io(error) => map_write_io_error(error, limit_bytes),
        other => ArchiveError::ProviderFailure {
            diagnostic: other.to_string(),
        },
    }
}

fn map_write_io_error(error: std::io::Error, limit_bytes: u64) -> ArchiveError {
    if error.to_string().contains(OUTPUT_LIMIT_DIAGNOSTIC) {
        ArchiveError::OutputLimitExceeded { limit_bytes }
    } else {
        ArchiveError::ProviderFailure {
            diagnostic: error.to_string(),
        }
    }
}

fn inspect_zip(archive: &[u8], limits: ArchiveReadLimits) -> Result<ArchiveManifest, ArchiveError> {
    let mut zip = open_zip(archive)?;
    if zip.len() as u64 > u64::from(limits.max_entries) {
        return Err(ArchiveError::EntryCountLimitExceeded {
            actual_entries: zip.len() as u64,
            limit_entries: limits.max_entries,
        });
    }

    let mut names = HashSet::new();
    let mut entries = Vec::with_capacity(zip.len());
    let mut total_uncompressed_bytes = 0_u64;
    for index in 0..zip.len() {
        let name_hint = zip.name_for_index(index).map(str::to_owned);
        let file = zip
            .by_index(index)
            .map_err(|error| map_zip_error(error, name_hint.as_deref()))?;
        let original_name = file.name().to_owned();
        if file.encrypted() {
            return Err(ArchiveError::EncryptedEntry {
                name: original_name,
            });
        }
        let normalized_name = normalize_entry_name(&original_name, limits.max_path_bytes)?;
        if !names.insert(normalized_name.clone()) {
            return Err(ArchiveError::DuplicateEntryName { normalized_name });
        }

        let kind = if file.is_dir() {
            ArchiveEntryKind::Directory
        } else if file.is_file() && !file.is_symlink() {
            ArchiveEntryKind::RegularFile
        } else {
            return Err(ArchiveError::UnsupportedEntryKind {
                name: original_name,
                kind: if file.is_symlink() {
                    "symlink"
                } else {
                    "non-regular"
                }
                .to_owned(),
            });
        };
        if file.size() > limits.max_entry_bytes {
            return Err(ArchiveError::EntryLimitExceeded {
                name: normalized_name,
                actual_bytes: file.size(),
                limit_bytes: limits.max_entry_bytes,
            });
        }
        total_uncompressed_bytes = total_uncompressed_bytes.saturating_add(file.size());
        if total_uncompressed_bytes > limits.max_total_output_bytes {
            return Err(ArchiveError::TotalOutputLimitExceeded {
                actual_bytes: total_uncompressed_bytes,
                limit_bytes: limits.max_total_output_bytes,
            });
        }

        entries.push(ArchiveEntryObservation {
            index: u32::try_from(index).unwrap_or(u32::MAX),
            original_name,
            normalized_name,
            kind,
            compression: compression(file.compression()),
            compressed_bytes: file.compressed_size(),
            uncompressed_bytes: file.size(),
            crc32: Some(file.crc32()),
        });
    }

    Ok(ArchiveManifest {
        format: ArchiveFormat::Zip,
        archive_bytes: archive.len() as u64,
        total_uncompressed_bytes,
        entries,
    })
}

fn open_zip(archive: &[u8]) -> Result<ZipArchive<Cursor<&[u8]>>, ArchiveError> {
    ZipArchive::new(Cursor::new(archive)).map_err(|error| map_zip_error(error, None))
}

fn require_zip(format: ArchiveFormat) -> Result<(), ArchiveError> {
    if format == ArchiveFormat::Zip {
        Ok(())
    } else {
        Err(ArchiveError::UnsupportedFormat { format })
    }
}

fn validate_archive_size(archive: &[u8], limits: ArchiveReadLimits) -> Result<(), ArchiveError> {
    let actual_bytes = archive.len() as u64;
    if actual_bytes > limits.max_archive_bytes {
        return Err(ArchiveError::ArchiveLimitExceeded {
            actual_bytes,
            limit_bytes: limits.max_archive_bytes,
        });
    }
    Ok(())
}

fn compression(method: CompressionMethod) -> ArchiveCompression {
    match method {
        CompressionMethod::Stored => ArchiveCompression::Stored,
        CompressionMethod::Deflated => ArchiveCompression::Deflate,
        _ => ArchiveCompression::Other,
    }
}

fn map_zip_error(error: ZipError, name: Option<&str>) -> ArchiveError {
    match error {
        ZipError::UnsupportedArchive(message) if message == ZipError::PASSWORD_REQUIRED => {
            ArchiveError::EncryptedEntry {
                name: name.unwrap_or("<unknown>").to_owned(),
            }
        }
        ZipError::InvalidArchive(message) => {
            let diagnostic = message.into_owned();
            if looks_truncated(&diagnostic) {
                ArchiveError::TruncatedArchive { diagnostic }
            } else {
                ArchiveError::MalformedArchive { diagnostic }
            }
        }
        ZipError::Io(error) => map_read_error(error, name.unwrap_or("<archive>")),
        other => ArchiveError::ProviderFailure {
            diagnostic: other.to_string(),
        },
    }
}

fn map_read_error(error: std::io::Error, name: &str) -> ArchiveError {
    let diagnostic = format!("{name}: {error}");
    if error.kind() == std::io::ErrorKind::UnexpectedEof {
        ArchiveError::TruncatedArchive { diagnostic }
    } else if error.kind() == std::io::ErrorKind::InvalidData
        || diagnostic.to_ascii_lowercase().contains("crc")
        || diagnostic.to_ascii_lowercase().contains("checksum")
    {
        ArchiveError::IntegrityFailure { diagnostic }
    } else {
        ArchiveError::ProviderFailure { diagnostic }
    }
}

fn looks_truncated(diagnostic: &str) -> bool {
    let diagnostic = diagnostic.to_ascii_lowercase();
    diagnostic.contains("eof")
        || diagnostic.contains("end of central directory")
        || diagnostic.contains("could not find central directory")
}
