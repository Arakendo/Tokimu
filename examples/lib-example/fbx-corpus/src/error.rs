use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum FbxError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("FBX input has {actual} bytes, exceeding the limit of {limit}")]
    InputTooLarge { actual: usize, limit: usize },
    #[error("FBX input is truncated at byte {offset} while reading {context}")]
    Truncated {
        offset: usize,
        context: &'static str,
    },
    #[error("invalid binary FBX signature")]
    InvalidSignature,
    #[error("unsupported binary FBX version {version}")]
    UnsupportedVersion { version: u32 },
    #[error("FBX record count exceeds the configured limit of {limit}")]
    RecordLimit { limit: usize },
    #[error("FBX record nesting exceeds the configured limit of {limit}")]
    DepthLimit { limit: usize },
    #[error("FBX property count exceeds the configured limit of {limit}")]
    PropertyLimit { limit: usize },
    #[error(
        "FBX array at byte {offset} declares {actual} elements, exceeding the limit of {limit}"
    )]
    ArrayLimit {
        offset: usize,
        actual: usize,
        limit: usize,
    },
    #[error("FBX blob at byte {offset} declares {actual} bytes, exceeding the limit of {limit}")]
    BlobLimit {
        offset: usize,
        actual: usize,
        limit: usize,
    },
    #[error("invalid FBX record at byte {offset}: {reason}")]
    InvalidRecord { offset: usize, reason: String },
    #[error("unsupported FBX property type 0x{code:02x} at byte {offset}")]
    UnsupportedProperty { offset: usize, code: u8 },
    #[error("unsupported FBX array encoding {encoding} at byte {offset}")]
    UnsupportedArrayEncoding { offset: usize, encoding: u32 },
    #[error("FBX array decompression failed at byte {offset}: {source}")]
    ArrayDecompression {
        offset: usize,
        source: std::io::Error,
    },
    #[error(
        "FBX array at byte {offset} decoded to {actual} bytes, but {expected} bytes were declared"
    )]
    InvalidArrayLength {
        offset: usize,
        expected: usize,
        actual: usize,
    },
    #[error("failed to serialize FBX source-record artifact: {0}")]
    Serialize(#[from] serde_json::Error),
}

pub type FbxResult<T> = Result<T, FbxError>;
