use std::collections::HashSet;
use std::io::{Cursor, Read, Write};

use tar::{Archive, Builder, EntryType, Header};

use crate::name::normalize_entry_name;
use crate::{
    ArchiveCompression, ArchiveEntryKind, ArchiveEntryObservation, ArchiveError, ArchiveFormat,
    ArchiveManifest, ArchiveProvider, ArchiveReadLimits, ArchiveReadResult, ArchiveWriteEntry,
    ArchiveWriteLimits, ArchiveWriteObservation, ArchiveWriteResult, ArchiveWriter,
};

/// Bounded TAR reader and writer for portable regular files and directories.
///
/// Links, sparse files, and extended headers remain provider-level diagnostics
/// rather than becoming Resource Space semantics.
#[derive(Clone, Copy, Debug, Default)]
pub struct TarArchiveProvider;

impl ArchiveProvider for TarArchiveProvider {
    fn supports(&self, format: ArchiveFormat) -> bool {
        format == ArchiveFormat::Tar
    }

    fn inspect(
        &self,
        format: ArchiveFormat,
        archive: &[u8],
        limits: ArchiveReadLimits,
    ) -> Result<ArchiveManifest, ArchiveError> {
        require_tar(format)?;
        validate_archive_size(archive, limits)?;
        inspect_tar(archive, limits)
    }

    fn read_entry(
        &self,
        format: ArchiveFormat,
        archive: &[u8],
        normalized_name: &str,
        limits: ArchiveReadLimits,
    ) -> Result<ArchiveReadResult, ArchiveError> {
        require_tar(format)?;
        validate_archive_size(archive, limits)?;
        let manifest = inspect_tar(archive, limits)?;
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

        let mut tar = Archive::new(Cursor::new(archive));
        let entries = tar.entries().map_err(map_tar_io_error)?;
        for (index, entry) in entries.enumerate() {
            let mut entry = entry.map_err(map_tar_io_error)?;
            if index as u32 != expected.index {
                continue;
            }
            let capacity = usize::try_from(expected.uncompressed_bytes).map_err(|_| {
                ArchiveError::EntryLimitExceeded {
                    name: expected.normalized_name.clone(),
                    actual_bytes: expected.uncompressed_bytes,
                    limit_bytes: usize::MAX as u64,
                }
            })?;
            let mut bytes = Vec::with_capacity(capacity);
            let mut bounded = (&mut entry).take(limits.max_entry_bytes.saturating_add(1));
            bounded.read_to_end(&mut bytes).map_err(map_tar_io_error)?;
            if bytes.len() as u64 > limits.max_entry_bytes {
                return Err(ArchiveError::EntryLimitExceeded {
                    name: expected.normalized_name,
                    actual_bytes: bytes.len() as u64,
                    limit_bytes: limits.max_entry_bytes,
                });
            }
            return Ok(ArchiveReadResult {
                entry: expected,
                bytes,
            });
        }
        Err(ArchiveError::EntryNotFound {
            name: normalized_name.to_owned(),
        })
    }
}

impl ArchiveWriter for TarArchiveProvider {
    fn supports_write(&self, format: ArchiveFormat) -> bool {
        format == ArchiveFormat::Tar
    }

    fn write_archive(
        &self,
        format: ArchiveFormat,
        entries: &[ArchiveWriteEntry],
        limits: ArchiveWriteLimits,
    ) -> Result<ArchiveWriteResult, ArchiveError> {
        require_tar(format)?;
        validate_write_entries(entries, limits)?;
        let mut builder = Builder::new(BoundedWriter::new(limits.max_archive_bytes));
        for entry in entries {
            let normalized_name = normalize_entry_name(&entry.name, limits.max_path_bytes)?;
            let mut header = deterministic_header(entry, &normalized_name)?;
            match entry.kind {
                ArchiveEntryKind::RegularFile => builder
                    .append_data(&mut header, normalized_name, Cursor::new(&entry.bytes))
                    .map_err(|error| map_tar_write_error(error, limits.max_archive_bytes))?,
                ArchiveEntryKind::Directory => builder
                    .append_data(&mut header, normalized_name, std::io::empty())
                    .map_err(|error| map_tar_write_error(error, limits.max_archive_bytes))?,
            }
        }
        let bytes = builder
            .into_inner()
            .map_err(|error| map_tar_write_error(error, limits.max_archive_bytes))?
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

fn inspect_tar(archive: &[u8], limits: ArchiveReadLimits) -> Result<ArchiveManifest, ArchiveError> {
    let mut tar = Archive::new(Cursor::new(archive));
    let entries = tar.entries().map_err(map_tar_io_error)?;
    let mut names = HashSet::new();
    let mut observed = Vec::new();
    let mut total_uncompressed_bytes = 0_u64;
    for (index, entry) in entries.enumerate() {
        if index as u64 >= u64::from(limits.max_entries) {
            return Err(ArchiveError::EntryCountLimitExceeded {
                actual_entries: index as u64 + 1,
                limit_entries: limits.max_entries,
            });
        }
        let mut entry = entry.map_err(map_tar_io_error)?;
        let mut original_name = entry_name(&entry)?;
        if entry.pax_extensions().map_err(map_tar_io_error)?.is_some() {
            return Err(ArchiveError::UnsupportedEntryKind {
                name: original_name,
                kind: "pax-extended".to_owned(),
            });
        }
        let kind = entry_kind(entry.header().entry_type(), &original_name)?;
        if kind == ArchiveEntryKind::Directory && !original_name.ends_with('/') {
            original_name.push('/');
        }
        let normalized_name = normalize_entry_name(&original_name, limits.max_path_bytes)?;
        if !names.insert(normalized_name.clone()) {
            return Err(ArchiveError::DuplicateEntryName { normalized_name });
        }
        let size = entry.size();
        if size > limits.max_entry_bytes {
            return Err(ArchiveError::EntryLimitExceeded {
                name: normalized_name,
                actual_bytes: size,
                limit_bytes: limits.max_entry_bytes,
            });
        }
        total_uncompressed_bytes = total_uncompressed_bytes.saturating_add(size);
        if total_uncompressed_bytes > limits.max_total_output_bytes {
            return Err(ArchiveError::TotalOutputLimitExceeded {
                actual_bytes: total_uncompressed_bytes,
                limit_bytes: limits.max_total_output_bytes,
            });
        }
        observed.push(ArchiveEntryObservation {
            index: index as u32,
            original_name,
            normalized_name,
            kind,
            compression: ArchiveCompression::Stored,
            compressed_bytes: size,
            uncompressed_bytes: size,
            crc32: None,
        });
    }
    Ok(ArchiveManifest {
        format: ArchiveFormat::Tar,
        archive_bytes: archive.len() as u64,
        total_uncompressed_bytes,
        entries: observed,
    })
}

fn entry_name<R: Read>(entry: &tar::Entry<'_, R>) -> Result<String, ArchiveError> {
    let path = entry.path().map_err(map_tar_io_error)?;
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| ArchiveError::MalformedArchive {
            diagnostic: "TAR entry name is not valid UTF-8".to_owned(),
        })
}

fn entry_kind(kind: EntryType, name: &str) -> Result<ArchiveEntryKind, ArchiveError> {
    if kind.is_file() {
        Ok(ArchiveEntryKind::RegularFile)
    } else if kind.is_dir() {
        Ok(ArchiveEntryKind::Directory)
    } else {
        let label = if kind.is_symlink() {
            "symlink".to_owned()
        } else if kind.is_hard_link() {
            "hard-link".to_owned()
        } else if kind.is_gnu_sparse() {
            "gnu-sparse".to_owned()
        } else if kind.is_gnu_longname() || kind.is_gnu_longlink() {
            "gnu-extended".to_owned()
        } else if kind.is_pax_global_extensions() || kind.is_pax_local_extensions() {
            "pax-extended".to_owned()
        } else {
            format!("type-{}", kind.as_byte() as char)
        };
        Err(ArchiveError::UnsupportedEntryKind {
            name: name.to_owned(),
            kind: label,
        })
    }
}

fn deterministic_header(entry: &ArchiveWriteEntry, name: &str) -> Result<Header, ArchiveError> {
    if entry.compression != ArchiveCompression::Stored {
        return Err(ArchiveError::UnsupportedWriteCompression {
            name: entry.name.clone(),
            compression: entry.compression,
        });
    }
    if entry.kind == ArchiveEntryKind::RegularFile && name.ends_with('/') {
        return Err(ArchiveError::InvalidWriteEntry {
            name: entry.name.clone(),
            reason: "regular file names must not end with `/`".to_owned(),
        });
    }
    if entry.kind == ArchiveEntryKind::Directory && !entry.bytes.is_empty() {
        return Err(ArchiveError::InvalidWriteEntry {
            name: entry.name.clone(),
            reason: "directory entries cannot contain bytes".to_owned(),
        });
    }
    let mut header = Header::new_ustar();
    header.set_size(entry.bytes.len() as u64);
    header.set_mode(match entry.kind {
        ArchiveEntryKind::RegularFile => 0o644,
        ArchiveEntryKind::Directory => 0o755,
    });
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header
        .set_username("")
        .and_then(|_| header.set_groupname(""))
        .map_err(|error| map_tar_write_error(error, u64::MAX))?;
    header.set_entry_type(match entry.kind {
        ArchiveEntryKind::RegularFile => EntryType::Regular,
        ArchiveEntryKind::Directory => EntryType::Directory,
    });
    header.set_cksum();
    Ok(header)
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
    let mut estimated_archive_bytes = 1024_u64;
    for entry in entries {
        let normalized_name = normalize_entry_name(&entry.name, limits.max_path_bytes)?;
        if !names.insert(normalized_name.clone()) {
            return Err(ArchiveError::DuplicateEntryName { normalized_name });
        }
        if entry.bytes.len() as u64 > limits.max_entry_bytes {
            return Err(ArchiveError::EntryLimitExceeded {
                name: entry.name.clone(),
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
        let data_blocks = (entry.bytes.len() as u64).saturating_add(511) / 512;
        estimated_archive_bytes = estimated_archive_bytes
            .saturating_add(512)
            .saturating_add(data_blocks.saturating_mul(512));
        if estimated_archive_bytes > limits.max_archive_bytes {
            return Err(ArchiveError::OutputLimitExceeded {
                limit_bytes: limits.max_archive_bytes,
            });
        }
    }
    Ok(())
}

fn require_tar(format: ArchiveFormat) -> Result<(), ArchiveError> {
    if format == ArchiveFormat::Tar {
        Ok(())
    } else {
        Err(ArchiveError::UnsupportedFormat { format })
    }
}

fn validate_archive_size(archive: &[u8], limits: ArchiveReadLimits) -> Result<(), ArchiveError> {
    if archive.len() as u64 > limits.max_archive_bytes {
        return Err(ArchiveError::ArchiveLimitExceeded {
            actual_bytes: archive.len() as u64,
            limit_bytes: limits.max_archive_bytes,
        });
    }
    Ok(())
}

fn map_tar_io_error(error: std::io::Error) -> ArchiveError {
    let diagnostic = error.to_string();
    if error.kind() == std::io::ErrorKind::UnexpectedEof
        || diagnostic.to_ascii_lowercase().contains("eof")
    {
        ArchiveError::TruncatedArchive { diagnostic }
    } else if error.kind() == std::io::ErrorKind::InvalidData
        || diagnostic.to_ascii_lowercase().contains("checksum")
    {
        ArchiveError::IntegrityFailure { diagnostic }
    } else {
        ArchiveError::MalformedArchive { diagnostic }
    }
}

fn map_tar_write_error(error: std::io::Error, limit_bytes: u64) -> ArchiveError {
    if error.to_string().contains("archive output limit exceeded") {
        ArchiveError::OutputLimitExceeded { limit_bytes }
    } else {
        ArchiveError::ProviderFailure {
            diagnostic: error.to_string(),
        }
    }
}

struct BoundedWriter {
    bytes: Vec<u8>,
    max_bytes: u64,
}

impl BoundedWriter {
    fn new(max_bytes: u64) -> Self {
        Self {
            bytes: Vec::new(),
            max_bytes,
        }
    }

    fn into_inner(self) -> Vec<u8> {
        self.bytes
    }
}

impl Write for BoundedWriter {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        if self.bytes.len().saturating_add(buffer.len()) as u64 > self.max_bytes {
            return Err(std::io::Error::other("archive output limit exceeded"));
        }
        self.bytes.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
