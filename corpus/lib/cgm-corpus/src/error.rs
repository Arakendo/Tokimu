use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum CgmError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("CGM input has {actual} bytes, exceeding the limit of {limit}")]
    InputTooLarge { actual: usize, limit: usize },
    #[error("CGM input is truncated at byte {offset} while reading {context}")]
    Truncated {
        offset: usize,
        context: &'static str,
    },
    #[error(
        "unsupported CGM encoding at byte 0: expected binary BEGIN METAFILE, found class {class}, id {id}"
    )]
    UnsupportedEncoding { class: u8, id: u8 },
    #[error("CGM element count exceeds the configured limit of {limit}")]
    ElementLimit { limit: usize },
    #[error(
        "CGM element at byte {offset} declares {actual} parameter bytes, exceeding the limit of {limit}"
    )]
    ParameterLimit {
        offset: usize,
        actual: usize,
        limit: usize,
    },
    #[error("CGM element at byte {offset} exceeds the partition limit of {limit}")]
    PartitionLimit { offset: usize, limit: usize },
    #[error("invalid CGM string at byte {offset}: {reason}")]
    InvalidString { offset: usize, reason: String },
    #[error("invalid CGM lifecycle at byte {offset}: {reason}")]
    InvalidLifecycle { offset: usize, reason: String },
    #[error("CGM VDC type {value} at byte {offset} is outside the initial integer-VDC profile")]
    UnsupportedVdcType { offset: usize, value: u16 },
    #[error(
        "CGM integer precision {value} at byte {offset} is outside the initial 16-bit profile"
    )]
    UnsupportedIntegerPrecision { offset: usize, value: u16 },
    #[error(
        "CGM {kind} precision {value} at byte {offset} is outside the initial 8-bit color profile"
    )]
    UnsupportedColorPrecision {
        offset: usize,
        kind: &'static str,
        value: u16,
    },
    #[error("CGM VDC extent at byte {offset} is malformed: {reason}")]
    InvalidVdcExtent { offset: usize, reason: String },
    #[error("CGM primitive at byte {offset} is malformed: {reason}")]
    InvalidPrimitive { offset: usize, reason: String },
    #[error("CGM picture {picture:?} has no usable VDC extent for primitive lowering")]
    MissingVdcExtent { picture: String },
    #[error("CGM primitive at byte {offset} is not admitted to the first vector-lowering profile: {kind}")]
    UnsupportedPrimitiveLowering { offset: usize, kind: &'static str },
    #[error("CGM document ended before END METAFILE")]
    MissingEndMetafile,
    #[error("CGM input contains {count} trailing bytes after END METAFILE")]
    TrailingData { count: usize },
}

pub type CgmResult<T> = Result<T, CgmError>;
