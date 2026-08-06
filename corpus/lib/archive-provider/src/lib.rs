//! Incubating, provider-neutral archive inspection and selected-entry reads.
//!
//! This boundary owns bounded container semantics. It deliberately has no
//! filesystem or Resource Space API, and archive entry names remain untrusted
//! until they pass normalization and safety checks.

mod model;
mod name;
mod seven_zip_provider;
mod tar_provider;
mod zip_provider;

pub use model::{
    ArchiveCompression, ArchiveEntryKind, ArchiveEntryObservation, ArchiveError, ArchiveFormat,
    ArchiveManifest, ArchiveProvider, ArchiveReadLimits, ArchiveReadResult, ArchiveWriteEntry,
    ArchiveWriteLimits, ArchiveWriteObservation, ArchiveWriteResult, ArchiveWriter,
};
pub use seven_zip_provider::SevenZipArchiveProvider;
pub use tar_provider::TarArchiveProvider;
pub use zip_provider::ZipArchiveProvider;

#[cfg(test)]
mod seeds;
#[cfg(test)]
mod tests;
