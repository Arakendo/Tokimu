use serde::{Deserialize, Serialize};

/// Self-identifying compression envelopes recognized without decoding.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompressionEnvelope {
    Gzip,
    Unknown,
}

/// Performs bounded advisory envelope detection.
///
/// Raw Brotli and Deflate payloads intentionally remain `Unknown`; arbitrary
/// bytes cannot identify those codecs reliably.
pub fn detect_compression_envelope(bytes: &[u8]) -> CompressionEnvelope {
    if bytes.starts_with(&[0x1f, 0x8b]) {
        CompressionEnvelope::Gzip
    } else {
        CompressionEnvelope::Unknown
    }
}
