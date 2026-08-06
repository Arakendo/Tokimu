use std::collections::HashSet;
use std::io::{Cursor, Seek, SeekFrom, Write};

use sevenz_rust2::{
    Archive, ArchiveEntry, ArchiveReader, ArchiveWriter as SevenZipWriter, Password,
};

use crate::name::normalize_entry_name;
use crate::{
    ArchiveCompression, ArchiveEntryKind, ArchiveEntryObservation, ArchiveError, ArchiveFormat,
    ArchiveManifest, ArchiveProvider, ArchiveReadLimits, ArchiveReadResult, ArchiveWriteEntry,
    ArchiveWriteLimits, ArchiveWriteObservation, ArchiveWriteResult, ArchiveWriter,
};

/// Bounded 7z adapter. The provider accepts and returns bytes so it remains
/// usable from native callers and browser/WASM consumers without a filesystem
/// dependency.
///
/// Writing creates a fresh archive only. Update, password, multi-volume, and
/// metadata-parity behavior remain explicit non-goals of this compatibility
/// provider.
#[derive(Clone, Copy, Debug, Default)]
pub struct SevenZipArchiveProvider;

impl ArchiveProvider for SevenZipArchiveProvider {
    fn supports(&self, format: ArchiveFormat) -> bool {
        format == ArchiveFormat::SevenZip
    }

    fn inspect(
        &self,
        format: ArchiveFormat,
        archive: &[u8],
        limits: ArchiveReadLimits,
    ) -> Result<ArchiveManifest, ArchiveError> {
        require_seven_zip(format)?;
        validate_archive_size(archive, limits)?;
        inspect_seven_zip(archive, limits)
    }

    fn read_entry(
        &self,
        format: ArchiveFormat,
        archive: &[u8],
        normalized_name: &str,
        limits: ArchiveReadLimits,
    ) -> Result<ArchiveReadResult, ArchiveError> {
        require_seven_zip(format)?;
        validate_archive_size(archive, limits)?;
        let manifest = inspect_seven_zip(archive, limits)?;
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

        let mut reader = open_seven_zip_reader(archive)?;
        let bytes = reader
            .read_file(expected.original_name.as_str())
            .map_err(map_seven_zip_error)?;
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

impl ArchiveWriter for SevenZipArchiveProvider {
    fn supports_write(&self, format: ArchiveFormat) -> bool {
        format == ArchiveFormat::SevenZip
    }

    fn write_archive(
        &self,
        format: ArchiveFormat,
        entries: &[ArchiveWriteEntry],
        limits: ArchiveWriteLimits,
    ) -> Result<ArchiveWriteResult, ArchiveError> {
        require_seven_zip(format)?;
        validate_write_entries(entries, limits)?;

        let output = BoundedCursor::new(limits.max_archive_bytes);
        let mut writer = SevenZipWriter::new(output)
            .map_err(|error| map_seven_zip_write_error(error, limits.max_archive_bytes))?;
        for entry in entries {
            let normalized_name = normalize_entry_name(&entry.name, limits.max_path_bytes)?;
            let archive_entry = match entry.kind {
                ArchiveEntryKind::RegularFile => ArchiveEntry::new_file(&normalized_name),
                ArchiveEntryKind::Directory => ArchiveEntry::new_directory(&normalized_name),
            };
            let content = match entry.kind {
                ArchiveEntryKind::RegularFile => Some(Cursor::new(entry.bytes.as_slice())),
                ArchiveEntryKind::Directory => None,
            };
            writer
                .push_archive_entry(archive_entry, content)
                .map_err(|error| map_seven_zip_write_error(error, limits.max_archive_bytes))?;
        }
        let bytes = writer
            .finish()
            .map_err(|error| map_seven_zip_write_io_error(error, limits.max_archive_bytes))?
            .into_inner();
        let total_input_bytes = entries.iter().map(|entry| entry.bytes.len() as u64).sum();
        Ok(ArchiveWriteResult {
            observation: ArchiveWriteObservation {
                format,
                archive_bytes: bytes.len() as u64,
                entry_count: entries.len() as u32,
                total_input_bytes,
                // The provider controls timestamps and metadata. Byte-level
                // equivalence is tested rather than inferred from this flag.
                deterministic_metadata: true,
            },
            bytes,
        })
    }
}

fn require_seven_zip(format: ArchiveFormat) -> Result<(), ArchiveError> {
    if format == ArchiveFormat::SevenZip {
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
    // The provider writes a header and end records in addition to entry data.
    // Reserve a conservative fixed allowance for each admitted entry before it
    // receives any input bytes.
    let mut estimated_archive_bytes = 512_u64;
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
        if entry.kind == ArchiveEntryKind::RegularFile
            && entry.compression != ArchiveCompression::Other
        {
            return Err(ArchiveError::UnsupportedWriteCompression {
                name: entry.name.clone(),
                compression: entry.compression,
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
        estimated_archive_bytes = estimated_archive_bytes
            .saturating_add(entry.bytes.len() as u64)
            .saturating_add(normalized_name.len() as u64)
            .saturating_add(256);
        if estimated_archive_bytes > limits.max_archive_bytes {
            return Err(ArchiveError::OutputLimitExceeded {
                limit_bytes: limits.max_archive_bytes,
            });
        }
    }
    Ok(())
}

fn inspect_seven_zip(
    archive: &[u8],
    limits: ArchiveReadLimits,
) -> Result<ArchiveManifest, ArchiveError> {
    let parsed = open_seven_zip(archive)?;
    if parsed.files.len() as u64 > u64::from(limits.max_entries) {
        return Err(ArchiveError::EntryCountLimitExceeded {
            actual_entries: parsed.files.len() as u64,
            limit_entries: limits.max_entries,
        });
    }

    let mut names = HashSet::new();
    let mut total_uncompressed_bytes = 0_u64;
    let mut entries = Vec::with_capacity(parsed.files.len());
    for (index, file) in parsed.files.iter().enumerate() {
        if file.is_anti_item {
            return Err(ArchiveError::UnsupportedEntryKind {
                name: file.name.clone(),
                kind: "anti-item".to_owned(),
            });
        }
        let normalized_name = normalize_entry_name(&file.name, limits.max_path_bytes)?;
        if !names.insert(normalized_name.clone()) {
            return Err(ArchiveError::DuplicateEntryName { normalized_name });
        }
        if file.size > limits.max_entry_bytes {
            return Err(ArchiveError::EntryLimitExceeded {
                name: normalized_name,
                actual_bytes: file.size,
                limit_bytes: limits.max_entry_bytes,
            });
        }
        total_uncompressed_bytes = total_uncompressed_bytes.saturating_add(file.size);
        if total_uncompressed_bytes > limits.max_total_output_bytes {
            return Err(ArchiveError::TotalOutputLimitExceeded {
                actual_bytes: total_uncompressed_bytes,
                limit_bytes: limits.max_total_output_bytes,
            });
        }
        entries.push(ArchiveEntryObservation {
            index: index as u32,
            original_name: file.name.clone(),
            normalized_name,
            kind: if file.is_directory {
                ArchiveEntryKind::Directory
            } else {
                ArchiveEntryKind::RegularFile
            },
            compression: ArchiveCompression::Other,
            compressed_bytes: file.compressed_size,
            uncompressed_bytes: file.size,
            crc32: file.has_crc.then_some(file.crc as u32),
        });
    }

    Ok(ArchiveManifest {
        format: ArchiveFormat::SevenZip,
        archive_bytes: archive.len() as u64,
        total_uncompressed_bytes,
        entries,
    })
}

fn open_seven_zip(archive: &[u8]) -> Result<Archive, ArchiveError> {
    let mut source = Cursor::new(archive);
    Archive::read(&mut source, &Password::empty()).map_err(map_seven_zip_error)
}

fn open_seven_zip_reader(archive: &[u8]) -> Result<ArchiveReader<Cursor<&[u8]>>, ArchiveError> {
    ArchiveReader::new(Cursor::new(archive), Password::empty()).map_err(map_seven_zip_error)
}

fn map_seven_zip_error(error: sevenz_rust2::Error) -> ArchiveError {
    let diagnostic = error.to_string();
    let lower = diagnostic.to_ascii_lowercase();
    if lower.contains("password") || lower.contains("encrypt") {
        ArchiveError::EncryptedEntry {
            name: "<archive>".to_owned(),
        }
    } else if lower.contains("unexpected eof") || lower.contains("truncated") {
        ArchiveError::TruncatedArchive { diagnostic }
    } else {
        ArchiveError::MalformedArchive { diagnostic }
    }
}

fn map_seven_zip_write_error(error: sevenz_rust2::Error, limit_bytes: u64) -> ArchiveError {
    let diagnostic = error.to_string();
    if diagnostic.contains(OUTPUT_LIMIT_DIAGNOSTIC) {
        ArchiveError::OutputLimitExceeded { limit_bytes }
    } else {
        ArchiveError::ProviderFailure { diagnostic }
    }
}

fn map_seven_zip_write_io_error(error: std::io::Error, limit_bytes: u64) -> ArchiveError {
    if error.to_string().contains(OUTPUT_LIMIT_DIAGNOSTIC) {
        ArchiveError::OutputLimitExceeded { limit_bytes }
    } else {
        ArchiveError::ProviderFailure {
            diagnostic: error.to_string(),
        }
    }
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

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn fixture() -> Vec<u8> {
        fixture_with_file("docs/readme.txt", b"Tokimu 7z provider evidence")
    }

    fn fixture_with_file(name: &str, bytes: &[u8]) -> Vec<u8> {
        let mut writer = SevenZipWriter::new(Cursor::new(Vec::new()))
            .expect("7z fixture writer should initialize");
        writer
            .push_archive_entry(ArchiveEntry::new_file(name), Some(Cursor::new(bytes)))
            .expect("7z fixture entry should write");
        writer
            .finish()
            .expect("7z fixture should finish")
            .into_inner()
    }

    #[test]
    fn inspects_and_reads_a_generated_seven_zip_entry() {
        let archive = fixture();
        let provider = SevenZipArchiveProvider;
        let manifest = provider
            .inspect(
                ArchiveFormat::SevenZip,
                &archive,
                ArchiveReadLimits::default(),
            )
            .expect("7z fixture should inspect");

        assert_eq!(manifest.format, ArchiveFormat::SevenZip);
        assert_eq!(manifest.entries.len(), 1);
        assert_eq!(manifest.entries[0].normalized_name, "docs/readme.txt");

        let entry = provider
            .read_entry(
                ArchiveFormat::SevenZip,
                &archive,
                "docs/readme.txt",
                ArchiveReadLimits::default(),
            )
            .expect("7z fixture entry should read");
        assert_eq!(entry.bytes, b"Tokimu 7z provider evidence");
    }

    #[test]
    fn rejects_unsafe_entry_names_during_inspection() {
        let archive = fixture_with_file("../escape.txt", b"unsafe");
        let error = SevenZipArchiveProvider
            .inspect(
                ArchiveFormat::SevenZip,
                &archive,
                ArchiveReadLimits::default(),
            )
            .expect_err("parent traversal must not enter a manifest");

        assert!(matches!(error, ArchiveError::UnsafeEntryName { .. }));
    }

    #[test]
    fn rejects_declared_entries_that_exceed_read_limits() {
        let archive = fixture_with_file("large.bin", b"0123456789");
        let limits = ArchiveReadLimits {
            max_entry_bytes: 4,
            ..ArchiveReadLimits::default()
        };
        let error = SevenZipArchiveProvider
            .inspect(ArchiveFormat::SevenZip, &archive, limits)
            .expect_err("oversized entries must fail before extraction");

        assert!(matches!(
            error,
            ArchiveError::EntryLimitExceeded {
                name,
                actual_bytes: 10,
                limit_bytes: 4,
            } if name == "large.bin"
        ));
    }

    #[test]
    fn writer_creates_a_bounded_seven_zip_archive_and_round_trips() {
        let entries = [
            ArchiveWriteEntry::directory("docs"),
            ArchiveWriteEntry::file(
                "docs/readme.txt",
                b"Tokimu bounded 7z".to_vec(),
                ArchiveCompression::Other,
            ),
        ];
        let provider = SevenZipArchiveProvider;
        let first = provider
            .write_archive(
                ArchiveFormat::SevenZip,
                &entries,
                ArchiveWriteLimits::default(),
            )
            .expect("7z writer should create a fresh archive");
        let second = provider
            .write_archive(
                ArchiveFormat::SevenZip,
                &entries,
                ArchiveWriteLimits::default(),
            )
            .expect("equivalent 7z writer input should create an archive");

        assert_eq!(first.bytes, second.bytes);
        assert!(first.observation.deterministic_metadata);
        assert_eq!(first.observation.entry_count, 2);

        let manifest = provider
            .inspect(
                ArchiveFormat::SevenZip,
                &first.bytes,
                ArchiveReadLimits::default(),
            )
            .expect("written 7z should inspect");
        assert_eq!(manifest.entries.len(), 2);
        assert_eq!(manifest.entries[0].kind, ArchiveEntryKind::Directory);
        let read = provider
            .read_entry(
                ArchiveFormat::SevenZip,
                &first.bytes,
                "docs/readme.txt",
                ArchiveReadLimits::default(),
            )
            .expect("written 7z file should read");
        assert_eq!(read.bytes, b"Tokimu bounded 7z");
    }

    #[test]
    fn writer_rejects_zip_style_compression_and_output_limit() {
        let provider = SevenZipArchiveProvider;
        let deflated = [ArchiveWriteEntry::file(
            "value.txt",
            b"value".to_vec(),
            ArchiveCompression::Deflate,
        )];
        assert!(matches!(
            provider.write_archive(
                ArchiveFormat::SevenZip,
                &deflated,
                ArchiveWriteLimits::default(),
            ),
            Err(ArchiveError::UnsupportedWriteCompression { .. })
        ));

        let entry = [ArchiveWriteEntry::file(
            "value.txt",
            vec![7; 32],
            ArchiveCompression::Other,
        )];
        assert!(matches!(
            provider.write_archive(
                ArchiveFormat::SevenZip,
                &entry,
                ArchiveWriteLimits::new(16, 2, 64, 64, 64),
            ),
            Err(ArchiveError::OutputLimitExceeded { limit_bytes: 16 })
        ));
    }
}
