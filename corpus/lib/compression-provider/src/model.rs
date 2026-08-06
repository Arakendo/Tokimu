use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompressionCodec {
    Gzip,
    Brotli,
    Deflate,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompressionGoal {
    Fast,
    #[default]
    Balanced,
    Small,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DecodeLimits {
    pub max_input_bytes: u64,
    pub max_output_bytes: u64,
    pub max_expansion_ratio: Option<u32>,
}

impl DecodeLimits {
    pub const fn new(max_input_bytes: u64, max_output_bytes: u64) -> Self {
        Self {
            max_input_bytes,
            max_output_bytes,
            max_expansion_ratio: None,
        }
    }

    pub const fn with_expansion_ratio(mut self, max_expansion_ratio: u32) -> Self {
        self.max_expansion_ratio = Some(max_expansion_ratio);
        self
    }

    pub fn validate_input(self, input_bytes: usize) -> Result<(), CompressionError> {
        let input_bytes = u64::try_from(input_bytes).unwrap_or(u64::MAX);
        if input_bytes > self.max_input_bytes {
            return Err(CompressionError::InputLimitExceeded {
                actual_bytes: input_bytes,
                limit_bytes: self.max_input_bytes,
            });
        }
        Ok(())
    }

    pub fn validate_output(
        self,
        input_bytes: usize,
        output_bytes: usize,
    ) -> Result<(), CompressionError> {
        let input_bytes = u64::try_from(input_bytes).unwrap_or(u64::MAX);
        let output_bytes = u64::try_from(output_bytes).unwrap_or(u64::MAX);

        if output_bytes > self.max_output_bytes {
            return Err(CompressionError::OutputLimitExceeded {
                actual_bytes: output_bytes,
                limit_bytes: self.max_output_bytes,
            });
        }

        if let Some(limit_ratio) = self.max_expansion_ratio {
            let permitted = input_bytes.saturating_mul(u64::from(limit_ratio));
            if output_bytes > permitted {
                return Err(CompressionError::ExpansionLimitExceeded {
                    input_bytes,
                    output_bytes,
                    limit_ratio,
                });
            }
        }
        Ok(())
    }
}

impl Default for DecodeLimits {
    fn default() -> Self {
        Self::new(16 * 1024 * 1024, 64 * 1024 * 1024).with_expansion_ratio(100)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EncodeRequest<'a> {
    pub codec: CompressionCodec,
    pub input: &'a [u8],
    pub goal: CompressionGoal,
}

impl<'a> EncodeRequest<'a> {
    pub fn new(codec: CompressionCodec, input: &'a [u8]) -> Self {
        Self {
            codec,
            input,
            goal: CompressionGoal::default(),
        }
    }

    pub const fn with_goal(mut self, goal: CompressionGoal) -> Self {
        self.goal = goal;
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DecodeRequest<'a> {
    pub codec: CompressionCodec,
    pub input: &'a [u8],
    pub limits: DecodeLimits,
}

impl<'a> DecodeRequest<'a> {
    pub const fn new(codec: CompressionCodec, input: &'a [u8], limits: DecodeLimits) -> Self {
        Self {
            codec,
            input,
            limits,
        }
    }

    pub fn validate_input(&self) -> Result<(), CompressionError> {
        self.limits.validate_input(self.input.len())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompressionObservation {
    pub codec: CompressionCodec,
    pub input_bytes: u64,
    pub output_bytes: u64,
}

impl CompressionObservation {
    pub fn new(codec: CompressionCodec, input_bytes: usize, output_bytes: usize) -> Self {
        Self {
            codec,
            input_bytes: u64::try_from(input_bytes).unwrap_or(u64::MAX),
            output_bytes: u64::try_from(output_bytes).unwrap_or(u64::MAX),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompressionResult {
    pub bytes: Vec<u8>,
    pub observation: CompressionObservation,
}

/// A byte-codec mechanism below Tokimu's provider-neutral contract.
pub trait CompressionProvider {
    fn supports(&self, codec: CompressionCodec) -> bool;

    fn encode(&self, request: EncodeRequest<'_>) -> Result<CompressionResult, CompressionError>;

    fn decode(&self, request: DecodeRequest<'_>) -> Result<CompressionResult, CompressionError>;
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum CompressionError {
    #[error("compression codec {codec:?} is unsupported")]
    UnsupportedCodec { codec: CompressionCodec },
    #[error("compression input is {actual_bytes} bytes; limit is {limit_bytes} bytes")]
    InputLimitExceeded { actual_bytes: u64, limit_bytes: u64 },
    #[error("compression output is {actual_bytes} bytes; limit is {limit_bytes} bytes")]
    OutputLimitExceeded { actual_bytes: u64, limit_bytes: u64 },
    #[error(
        "decoded output expanded from {input_bytes} to {output_bytes} bytes; ratio limit is {limit_ratio}:1"
    )]
    ExpansionLimitExceeded {
        input_bytes: u64,
        output_bytes: u64,
        limit_ratio: u32,
    },
    #[error("compressed input is malformed: {diagnostic}")]
    MalformedInput { diagnostic: String },
    #[error("compressed input is truncated: {diagnostic}")]
    TruncatedInput { diagnostic: String },
    #[error("compression provider failed: {diagnostic}")]
    ProviderFailure { diagnostic: String },
}
