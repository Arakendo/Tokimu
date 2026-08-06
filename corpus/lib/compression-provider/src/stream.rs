use std::io::{self, Read};

use crate::{
    CompressionCodec, CompressionError, CompressionObservation, CompressionResult, DecodeRequest,
};

pub(crate) fn collect_bounded(
    mut decoder: impl Read,
    request: DecodeRequest<'_>,
    map_error: impl Fn(io::Error) -> CompressionError,
) -> Result<Vec<u8>, CompressionError> {
    let mut output = Vec::new();
    let mut chunk = [0_u8; 8 * 1024];

    loop {
        let count = decoder.read(&mut chunk).map_err(&map_error)?;
        if count == 0 {
            break;
        }

        let next_len =
            output
                .len()
                .checked_add(count)
                .ok_or(CompressionError::OutputLimitExceeded {
                    actual_bytes: u64::MAX,
                    limit_bytes: request.limits.max_output_bytes,
                })?;
        request
            .limits
            .validate_output(request.input.len(), next_len)?;
        output.extend_from_slice(&chunk[..count]);
    }

    Ok(output)
}

pub(crate) fn result(
    codec: CompressionCodec,
    input_bytes: usize,
    bytes: Vec<u8>,
) -> CompressionResult {
    CompressionResult {
        observation: CompressionObservation::new(codec, input_bytes, bytes.len()),
        bytes,
    }
}
