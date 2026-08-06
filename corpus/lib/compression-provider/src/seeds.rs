//! Deterministic malformed byte prefixes for codec regression tests.
//!
//! These seeds identify failures at the codec boundary. They are intentionally
//! small so later fuzzing can reuse them without coupling to a provider's
//! internal stream representation.

pub(super) struct CompressionInputSeed {
    pub(super) id: &'static str,
    pub(super) bytes: &'static [u8],
}

pub(super) const GZIP_INPUT_SEEDS: &[CompressionInputSeed] = &[
    CompressionInputSeed {
        id: "empty-input",
        bytes: b"",
    },
    CompressionInputSeed {
        id: "gzip-magic-only",
        bytes: b"\x1f\x8b",
    },
    CompressionInputSeed {
        id: "gzip-header-without-body",
        bytes: b"\x1f\x8b\x08\x00\x00\x00\x00\x00\x00\xff",
    },
    CompressionInputSeed {
        id: "non-gzip-bytes",
        bytes: b"Tokimu codec seed: not gzip",
    },
];

pub(super) const BROTLI_INPUT_SEEDS: &[CompressionInputSeed] = &[
    CompressionInputSeed {
        id: "empty-input",
        bytes: b"",
    },
    CompressionInputSeed {
        id: "single-zero-byte",
        bytes: b"\x00",
    },
    CompressionInputSeed {
        id: "truncated-prefix",
        bytes: b"\x83\x01",
    },
    CompressionInputSeed {
        id: "non-brotli-bytes",
        bytes: b"Tokimu codec seed: not brotli",
    },
];
