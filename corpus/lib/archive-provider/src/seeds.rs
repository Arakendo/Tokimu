//! Small deterministic adversarial inputs retained beside the archive contract.
//!
//! These are named regression seeds, not a general fuzzing framework. Future
//! fuzz targets can reuse the same boundary cases without treating provider
//! implementation details as portable archive semantics.

pub(super) struct ArchiveInputSeed {
    pub(super) id: &'static str,
    pub(super) bytes: &'static [u8],
}

pub(super) const ZIP_INPUT_SEEDS: &[ArchiveInputSeed] = &[
    ArchiveInputSeed {
        id: "empty-input",
        bytes: b"",
    },
    ArchiveInputSeed {
        id: "local-header-without-payload",
        bytes: b"PK\x03\x04\x14\x00\x00\x00",
    },
    ArchiveInputSeed {
        id: "central-directory-prefix-only",
        bytes: b"PK\x01\x02",
    },
    ArchiveInputSeed {
        id: "non-archive-bytes",
        bytes: b"Tokimu archive seed: not a container",
    },
];

pub(super) const UNSAFE_ENTRY_NAME_SEEDS: &[&str] = &[
    "../outside.txt",
    "/absolute.txt",
    "C:\\drive-rooted.txt",
    "nested/../../outside.txt",
    "././../outside.txt",
];
