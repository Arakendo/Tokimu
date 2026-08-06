use std::io::{self, Read};

use brotli::{CompressorReader, Decompressor};

use crate::{
    stream::{collect_bounded, result},
    CompressionCodec, CompressionError, CompressionGoal, CompressionProvider, CompressionResult,
    DecodeRequest, EncodeRequest,
};

const BUFFER_BYTES: usize = 8 * 1024;
const WINDOW_BITS: u32 = 22;

/// Safe-Rust raw-Brotli provider backed by `brotli`.
#[derive(Clone, Copy, Debug, Default)]
pub struct BrotliCompressionProvider;

impl CompressionProvider for BrotliCompressionProvider {
    fn supports(&self, codec: CompressionCodec) -> bool {
        codec == CompressionCodec::Brotli
    }

    fn encode(&self, request: EncodeRequest<'_>) -> Result<CompressionResult, CompressionError> {
        ensure_brotli(request.codec)?;

        let mut encoder = CompressorReader::new(
            request.input,
            BUFFER_BYTES,
            quality(request.goal),
            WINDOW_BITS,
        );
        let mut bytes = Vec::new();
        encoder
            .read_to_end(&mut bytes)
            .map_err(map_provider_error)?;

        Ok(result(request.codec, request.input.len(), bytes))
    }

    fn decode(&self, request: DecodeRequest<'_>) -> Result<CompressionResult, CompressionError> {
        ensure_brotli(request.codec)?;
        request.validate_input()?;

        let bytes = collect_bounded(
            Decompressor::new(request.input, BUFFER_BYTES),
            request,
            map_decode_error,
        )?;
        Ok(result(request.codec, request.input.len(), bytes))
    }
}

fn ensure_brotli(codec: CompressionCodec) -> Result<(), CompressionError> {
    if codec == CompressionCodec::Brotli {
        Ok(())
    } else {
        Err(CompressionError::UnsupportedCodec { codec })
    }
}

fn quality(goal: CompressionGoal) -> u32 {
    match goal {
        CompressionGoal::Fast => 2,
        CompressionGoal::Balanced => 6,
        CompressionGoal::Small => 11,
    }
}

fn map_decode_error(error: io::Error) -> CompressionError {
    let diagnostic = format!("raw Brotli: {error}");
    match error.kind() {
        io::ErrorKind::UnexpectedEof => CompressionError::TruncatedInput { diagnostic },
        io::ErrorKind::InvalidData => CompressionError::MalformedInput { diagnostic },
        _ => CompressionError::ProviderFailure { diagnostic },
    }
}

fn map_provider_error(error: io::Error) -> CompressionError {
    CompressionError::ProviderFailure {
        diagnostic: format!("raw Brotli: {error}"),
    }
}
