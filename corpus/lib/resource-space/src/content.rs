use serde::{Deserialize, Serialize};

/// Named diagnostic fingerprint of immutable resource bytes.
///
/// A matching fingerprint is useful for reporting and candidate
/// deduplication, but it never defines resource identity. Callers requiring
/// exact equality must compare the retained bytes after a fingerprint match.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentFingerprint {
    algorithm: ContentFingerprintAlgorithm,
    digest: [u8; 32],
}

impl ContentFingerprint {
    pub fn blake3(bytes: &[u8]) -> Self {
        Self {
            algorithm: ContentFingerprintAlgorithm::Blake3,
            digest: *blake3::hash(bytes).as_bytes(),
        }
    }

    pub const fn algorithm(&self) -> ContentFingerprintAlgorithm {
        self.algorithm
    }

    pub const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    /// Checks whether `bytes` produce this fingerprint under its named
    /// algorithm. This is not a substitute for an exact byte comparison.
    pub fn matches_bytes(&self, bytes: &[u8]) -> bool {
        match self.algorithm {
            ContentFingerprintAlgorithm::Blake3 => Self::blake3(bytes) == *self,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentFingerprintAlgorithm {
    Blake3,
}
