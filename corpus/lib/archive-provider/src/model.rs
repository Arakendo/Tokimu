use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchiveFormat {
    Zip,
    Tar,
    SevenZip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchiveEntryKind {
    RegularFile,
    Directory,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchiveCompression {
    Stored,
    Deflate,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArchiveReadLimits {
    pub max_archive_bytes: u64,
    pub max_entries: u32,
    pub max_entry_bytes: u64,
    pub max_total_output_bytes: u64,
    pub max_path_bytes: u32,
}

impl ArchiveReadLimits {
    pub const fn new(
        max_archive_bytes: u64,
        max_entries: u32,
        max_entry_bytes: u64,
        max_total_output_bytes: u64,
        max_path_bytes: u32,
    ) -> Self {
        Self {
            max_archive_bytes,
            max_entries,
            max_entry_bytes,
            max_total_output_bytes,
            max_path_bytes,
        }
    }
}

impl Default for ArchiveReadLimits {
    fn default() -> Self {
        Self::new(
            64 * 1024 * 1024,
            10_000,
            64 * 1024 * 1024,
            64 * 1024 * 1024,
            4096,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArchiveEntryObservation {
    pub index: u32,
    pub original_name: String,
    pub normalized_name: String,
    pub kind: ArchiveEntryKind,
    pub compression: ArchiveCompression,
    pub compressed_bytes: u64,
    pub uncompressed_bytes: u64,
    /// ZIP provides CRC-32 metadata; formats without an equivalent portable
    /// entry checksum report `None` rather than inventing a value.
    pub crc32: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArchiveManifest {
    pub format: ArchiveFormat,
    pub archive_bytes: u64,
    pub total_uncompressed_bytes: u64,
    pub entries: Vec<ArchiveEntryObservation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveReadResult {
    pub entry: ArchiveEntryObservation,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArchiveWriteLimits {
    pub max_archive_bytes: u64,
    pub max_entries: u32,
    pub max_entry_bytes: u64,
    pub max_total_input_bytes: u64,
    pub max_path_bytes: u32,
}

impl ArchiveWriteLimits {
    pub const fn new(
        max_archive_bytes: u64,
        max_entries: u32,
        max_entry_bytes: u64,
        max_total_input_bytes: u64,
        max_path_bytes: u32,
    ) -> Self {
        Self {
            max_archive_bytes,
            max_entries,
            max_entry_bytes,
            max_total_input_bytes,
            max_path_bytes,
        }
    }
}

impl Default for ArchiveWriteLimits {
    fn default() -> Self {
        Self::new(
            64 * 1024 * 1024,
            10_000,
            64 * 1024 * 1024,
            256 * 1024 * 1024,
            4096,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveWriteEntry {
    pub name: String,
    pub kind: ArchiveEntryKind,
    pub compression: ArchiveCompression,
    pub bytes: Vec<u8>,
}

impl ArchiveWriteEntry {
    pub fn file(
        name: impl Into<String>,
        bytes: impl Into<Vec<u8>>,
        compression: ArchiveCompression,
    ) -> Self {
        Self {
            name: name.into(),
            kind: ArchiveEntryKind::RegularFile,
            compression,
            bytes: bytes.into(),
        }
    }

    pub fn directory(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: ArchiveEntryKind::Directory,
            compression: ArchiveCompression::Stored,
            bytes: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArchiveWriteObservation {
    pub format: ArchiveFormat,
    pub archive_bytes: u64,
    pub entry_count: u32,
    pub total_input_bytes: u64,
    pub deterministic_metadata: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveWriteResult {
    pub observation: ArchiveWriteObservation,
    pub bytes: Vec<u8>,
}

pub trait ArchiveProvider {
    fn supports(&self, format: ArchiveFormat) -> bool;

    fn inspect(
        &self,
        format: ArchiveFormat,
        archive: &[u8],
        limits: ArchiveReadLimits,
    ) -> Result<ArchiveManifest, ArchiveError>;

    fn read_entry(
        &self,
        format: ArchiveFormat,
        archive: &[u8],
        normalized_name: &str,
        limits: ArchiveReadLimits,
    ) -> Result<ArchiveReadResult, ArchiveError>;
}

pub trait ArchiveWriter {
    fn supports_write(&self, format: ArchiveFormat) -> bool;

    fn write_archive(
        &self,
        format: ArchiveFormat,
        entries: &[ArchiveWriteEntry],
        limits: ArchiveWriteLimits,
    ) -> Result<ArchiveWriteResult, ArchiveError>;
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ArchiveError {
    #[error("archive format {format:?} is unsupported")]
    UnsupportedFormat { format: ArchiveFormat },
    #[error("archive input is {actual_bytes} bytes; limit is {limit_bytes} bytes")]
    ArchiveLimitExceeded { actual_bytes: u64, limit_bytes: u64 },
    #[error("archive contains {actual_entries} entries; limit is {limit_entries}")]
    EntryCountLimitExceeded {
        actual_entries: u64,
        limit_entries: u32,
    },
    #[error("archive entry `{name}` declares {actual_bytes} bytes; limit is {limit_bytes} bytes")]
    EntryLimitExceeded {
        name: String,
        actual_bytes: u64,
        limit_bytes: u64,
    },
    #[error("archive declares {actual_bytes} total output bytes; limit is {limit_bytes} bytes")]
    TotalOutputLimitExceeded { actual_bytes: u64, limit_bytes: u64 },
    #[error("archive entry name is unsafe: `{name}` ({reason})")]
    UnsafeEntryName { name: String, reason: String },
    #[error("archive entries normalize to duplicate name `{normalized_name}`")]
    DuplicateEntryName { normalized_name: String },
    #[error("archive entry `{name}` is encrypted")]
    EncryptedEntry { name: String },
    #[error("archive entry `{name}` has unsupported kind: {kind}")]
    UnsupportedEntryKind { name: String, kind: String },
    #[error("archive entry `{name}` was not found")]
    EntryNotFound { name: String },
    #[error("archive integrity check failed: {diagnostic}")]
    IntegrityFailure { diagnostic: String },
    #[error("archive input is truncated: {diagnostic}")]
    TruncatedArchive { diagnostic: String },
    #[error("archive input is malformed: {diagnostic}")]
    MalformedArchive { diagnostic: String },
    #[error("archive provider failed: {diagnostic}")]
    ProviderFailure { diagnostic: String },
    #[error("archive output reached its {limit_bytes} byte limit")]
    OutputLimitExceeded { limit_bytes: u64 },
    #[error("archive entry `{name}` uses unsupported write compression {compression:?}")]
    UnsupportedWriteCompression {
        name: String,
        compression: ArchiveCompression,
    },
    #[error("archive entry `{name}` has invalid write content: {reason}")]
    InvalidWriteEntry { name: String, reason: String },
}
