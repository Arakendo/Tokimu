use std::io::{self, Write};

use flate2::{
    read::{DeflateDecoder, GzDecoder},
    write::{DeflateEncoder, GzEncoder},
    Compression,
};

use crate::{
    stream::{collect_bounded, result},
    CompressionCodec, CompressionError, CompressionGoal, CompressionProvider, CompressionResult,
    DecodeRequest, EncodeRequest,
};

/// Pure-Rust GZip and raw-Deflate provider backed by `flate2`.
#[derive(Clone, Copy, Debug, Default)]
pub struct FlateCompressionProvider;

impl CompressionProvider for FlateCompressionProvider {
    fn supports(&self, codec: CompressionCodec) -> bool {
        matches!(codec, CompressionCodec::Gzip | CompressionCodec::Deflate)
    }

    fn encode(&self, request: EncodeRequest<'_>) -> Result<CompressionResult, CompressionError> {
        if !self.supports(request.codec) {
            return Err(CompressionError::UnsupportedCodec {
                codec: request.codec,
            });
        }

        let bytes = match request.codec {
            CompressionCodec::Gzip => encode_gzip(request.input, request.goal),
            CompressionCodec::Deflate => encode_deflate(request.input, request.goal),
            CompressionCodec::Brotli => unreachable!("unsupported codec returned above"),
        }?;

        Ok(result(request.codec, request.input.len(), bytes))
    }

    fn decode(&self, request: DecodeRequest<'_>) -> Result<CompressionResult, CompressionError> {
        if !self.supports(request.codec) {
            return Err(CompressionError::UnsupportedCodec {
                codec: request.codec,
            });
        }
        request.validate_input()?;

        if request.codec == CompressionCodec::Gzip && !request.input.starts_with(&[0x1f, 0x8b]) {
            return Err(CompressionError::MalformedInput {
                diagnostic: "GZip: missing 1f 8b envelope signature".to_owned(),
            });
        }

        let bytes = match request.codec {
            CompressionCodec::Gzip => {
                collect_bounded(GzDecoder::new(request.input), request, |error| {
                    map_decode_error(error, "GZip")
                })
            }
            CompressionCodec::Deflate => {
                collect_bounded(DeflateDecoder::new(request.input), request, |error| {
                    map_decode_error(error, "raw Deflate")
                })
            }
            CompressionCodec::Brotli => unreachable!("unsupported codec returned above"),
        }?;

        Ok(result(request.codec, request.input.len(), bytes))
    }
}

fn compression(goal: CompressionGoal) -> Compression {
    match goal {
        CompressionGoal::Fast => Compression::fast(),
        CompressionGoal::Balanced => Compression::default(),
        CompressionGoal::Small => Compression::best(),
    }
}

fn encode_gzip(input: &[u8], goal: CompressionGoal) -> Result<Vec<u8>, CompressionError> {
    let mut encoder = GzEncoder::new(Vec::new(), compression(goal));
    encoder.write_all(input).map_err(map_provider_error)?;
    encoder.finish().map_err(map_provider_error)
}

fn encode_deflate(input: &[u8], goal: CompressionGoal) -> Result<Vec<u8>, CompressionError> {
    let mut encoder = DeflateEncoder::new(Vec::new(), compression(goal));
    encoder.write_all(input).map_err(map_provider_error)?;
    encoder.finish().map_err(map_provider_error)
}

fn map_decode_error(error: io::Error, envelope: &str) -> CompressionError {
    let diagnostic = format!("{envelope}: {error}");
    match error.kind() {
        io::ErrorKind::UnexpectedEof => CompressionError::TruncatedInput { diagnostic },
        io::ErrorKind::InvalidData => CompressionError::MalformedInput { diagnostic },
        _ => CompressionError::ProviderFailure { diagnostic },
    }
}

fn map_provider_error(error: io::Error) -> CompressionError {
    CompressionError::ProviderFailure {
        diagnostic: error.to_string(),
    }
}
