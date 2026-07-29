use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum CorpusError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid GLB: {0}")]
    InvalidGlb(String),
    #[error("invalid glTF document: {0}")]
    InvalidDocument(String),
    #[error("unsupported glTF accessor: {0}")]
    UnsupportedAccessor(String),
    #[error("accessor data exceeds its buffer or buffer view: {0}")]
    AccessorRange(String),
    #[error("unsupported glTF version `{0}`")]
    UnsupportedVersion(String),
    #[error("external buffer URI `{0}` is not a local relative path")]
    UnsupportedBufferUri(String),
    #[error("missing external buffer {0}")]
    MissingBuffer(PathBuf),
    #[error(
        "external buffer {path} is shorter than declared: expected at least {expected}, got {actual}"
    )]
    ShortBuffer {
        path: PathBuf,
        expected: u64,
        actual: u64,
    },
}

pub type CorpusResult<T> = Result<T, CorpusError>;
