use super::seeds::{BROTLI_INPUT_SEEDS, GZIP_INPUT_SEEDS};
use super::*;

struct IdentityProvider;

impl CompressionProvider for IdentityProvider {
    fn supports(&self, _codec: CompressionCodec) -> bool {
        true
    }

    fn encode(&self, request: EncodeRequest<'_>) -> Result<CompressionResult, CompressionError> {
        Ok(CompressionResult {
            bytes: request.input.to_vec(),
            observation: CompressionObservation::new(
                request.codec,
                request.input.len(),
                request.input.len(),
            ),
        })
    }

    fn decode(&self, request: DecodeRequest<'_>) -> Result<CompressionResult, CompressionError> {
        decode_identity(request)
    }
}

fn decode_identity(request: DecodeRequest<'_>) -> Result<CompressionResult, CompressionError> {
    request.validate_input()?;
    request
        .limits
        .validate_output(request.input.len(), request.input.len())?;
    Ok(CompressionResult {
        bytes: request.input.to_vec(),
        observation: CompressionObservation::new(
            request.codec,
            request.input.len(),
            request.input.len(),
        ),
    })
}

#[test]
fn request_contract_has_no_filesystem_or_resource_space_dependency() {
    let provider = IdentityProvider;
    let input = b"bounded byte transformation";
    let result = provider
        .encode(EncodeRequest::new(CompressionCodec::Gzip, input))
        .expect("identity provider should satisfy the contract");

    assert_eq!(result.bytes, input);
    assert_eq!(result.observation.input_bytes, input.len() as u64);
    assert_eq!(result.observation.output_bytes, input.len() as u64);
}

#[test]
fn input_limit_is_checked_before_provider_work() {
    let provider = IdentityProvider;
    let request = DecodeRequest::new(
        CompressionCodec::Brotli,
        b"too large",
        DecodeLimits::new(3, 32),
    );

    assert_eq!(
        provider.decode(request),
        Err(CompressionError::InputLimitExceeded {
            actual_bytes: 9,
            limit_bytes: 3,
        })
    );
}

#[test]
fn output_and_expansion_limits_are_distinct() {
    let output_error = DecodeLimits::new(16, 4)
        .validate_output(2, 5)
        .expect_err("absolute output limit should reject output");
    assert!(matches!(
        output_error,
        CompressionError::OutputLimitExceeded { .. }
    ));

    let ratio_error = DecodeLimits::new(16, 100)
        .with_expansion_ratio(2)
        .validate_output(3, 7)
        .expect_err("expansion ratio should reject output");
    assert!(matches!(
        ratio_error,
        CompressionError::ExpansionLimitExceeded { .. }
    ));
}

#[test]
fn only_gzip_has_advisory_envelope_detection() {
    assert_eq!(
        detect_compression_envelope(&[0x1f, 0x8b, 0x08]),
        CompressionEnvelope::Gzip
    );
    assert_eq!(
        detect_compression_envelope(b"raw deflate or brotli is ambiguous"),
        CompressionEnvelope::Unknown
    );
}

#[test]
fn flate_provider_round_trips_gzip_and_raw_deflate() {
    let provider = FlateCompressionProvider;
    let cases = [
        Vec::new(),
        b"small".to_vec(),
        "Tokimu compression: 世界".as_bytes().to_vec(),
        (0_u8..=255).collect(),
        vec![0xA5; 4096],
    ];

    for codec in [CompressionCodec::Gzip, CompressionCodec::Deflate] {
        for input in &cases {
            let encoded = provider
                .encode(EncodeRequest::new(codec, input))
                .expect("supported codec should encode");
            let decoded = provider
                .decode(DecodeRequest::new(
                    codec,
                    &encoded.bytes,
                    DecodeLimits::new(16 * 1024, 16 * 1024),
                ))
                .expect("supported codec should decode");

            assert_eq!(&decoded.bytes, input);
            assert_eq!(decoded.observation.codec, codec);
        }
    }
}

#[test]
fn brotli_provider_round_trips_contract_matrix() {
    let provider = BrotliCompressionProvider;
    let cases = [
        Vec::new(),
        b"small".to_vec(),
        "Tokimu compression: 世界".as_bytes().to_vec(),
        (0_u8..=255).collect(),
        pseudo_random_bytes(4096),
        vec![0xA5; 4096],
    ];

    for goal in [
        CompressionGoal::Fast,
        CompressionGoal::Balanced,
        CompressionGoal::Small,
    ] {
        for input in &cases {
            let encoded = provider
                .encode(EncodeRequest::new(CompressionCodec::Brotli, input).with_goal(goal))
                .expect("Brotli should encode the contract matrix");
            let decoded = provider
                .decode(DecodeRequest::new(
                    CompressionCodec::Brotli,
                    &encoded.bytes,
                    DecodeLimits::new(16 * 1024, 16 * 1024),
                ))
                .expect("Brotli should decode the contract matrix");

            assert_eq!(&decoded.bytes, input);
            assert_eq!(decoded.observation.codec, CompressionCodec::Brotli);
        }
    }
}

#[test]
fn brotli_provider_enforces_streaming_output_limit() {
    let provider = BrotliCompressionProvider;
    let input = vec![b'A'; 4096];
    let encoded = provider
        .encode(EncodeRequest::new(CompressionCodec::Brotli, &input))
        .expect("fixture should encode");

    let error = provider
        .decode(DecodeRequest::new(
            CompressionCodec::Brotli,
            &encoded.bytes,
            DecodeLimits::new(1024, 64),
        ))
        .expect_err("bounded decode should reject expanded output");

    assert!(matches!(
        error,
        CompressionError::OutputLimitExceeded { .. }
    ));
}

#[test]
fn brotli_provider_rejects_other_codecs() {
    let provider = BrotliCompressionProvider;
    for codec in [CompressionCodec::Gzip, CompressionCodec::Deflate] {
        assert_eq!(
            provider.encode(EncodeRequest::new(codec, b"payload")),
            Err(CompressionError::UnsupportedCodec { codec })
        );
    }
}

#[test]
fn brotli_provider_rejects_malformed_input() {
    let provider = BrotliCompressionProvider;
    let error = provider
        .decode(DecodeRequest::new(
            CompressionCodec::Brotli,
            b"not a Brotli stream",
            DecodeLimits::new(1024, 4096),
        ))
        .expect_err("malformed input should fail");

    assert!(
        matches!(error, CompressionError::MalformedInput { .. }),
        "unexpected category: {error:?}"
    );
}

#[test]
fn named_codec_seeds_remain_codec_failures() {
    let limits = DecodeLimits::new(1024, 4096);

    for seed in GZIP_INPUT_SEEDS {
        let error = match FlateCompressionProvider.decode(DecodeRequest::new(
            CompressionCodec::Gzip,
            seed.bytes,
            limits,
        )) {
            Ok(_) => panic!("gzip seed `{}` decoded successfully", seed.id),
            Err(error) => error,
        };
        assert!(
            matches!(
                error,
                CompressionError::MalformedInput { .. } | CompressionError::TruncatedInput { .. }
            ),
            "gzip seed `{}` escaped the codec failure boundary: {error:?}",
            seed.id
        );
    }

    for seed in BROTLI_INPUT_SEEDS {
        let error = match BrotliCompressionProvider.decode(DecodeRequest::new(
            CompressionCodec::Brotli,
            seed.bytes,
            limits,
        )) {
            Ok(_) => panic!("Brotli seed `{}` decoded successfully", seed.id),
            Err(error) => error,
        };
        assert!(
            matches!(error, CompressionError::MalformedInput { .. }),
            "Brotli seed `{}` escaped the codec failure boundary: {error:?}",
            seed.id
        );
    }
}

#[test]
fn brotli_provider_reports_truncated_stream_as_malformed() {
    let provider = BrotliCompressionProvider;
    let input = pseudo_random_bytes(4096);
    let mut encoded = provider
        .encode(EncodeRequest::new(CompressionCodec::Brotli, &input))
        .expect("fixture should encode")
        .bytes;
    encoded.truncate(encoded.len() / 2);

    let error = provider
        .decode(DecodeRequest::new(
            CompressionCodec::Brotli,
            &encoded,
            DecodeLimits::new(16 * 1024, 16 * 1024),
        ))
        .expect_err("truncated input should fail");

    assert!(
        matches!(error, CompressionError::MalformedInput { .. }),
        "unexpected category: {error:?}"
    );
}

fn pseudo_random_bytes(length: usize) -> Vec<u8> {
    let mut state = 0x4D59_5DF4_D0F3_3173_u64;
    (0..length)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state as u8
        })
        .collect()
}

#[test]
fn flate_provider_enforces_limits_while_collecting_output() {
    let provider = FlateCompressionProvider;
    let input = vec![b'A'; 4096];
    let encoded = provider
        .encode(EncodeRequest::new(CompressionCodec::Gzip, &input))
        .expect("fixture should encode");

    let error = provider
        .decode(DecodeRequest::new(
            CompressionCodec::Gzip,
            &encoded.bytes,
            DecodeLimits::new(1024, 64),
        ))
        .expect_err("bounded decode should reject expanded output");

    assert!(matches!(
        error,
        CompressionError::OutputLimitExceeded { .. }
    ));
}

#[test]
fn flate_provider_rejects_brotli_without_guessing() {
    let provider = FlateCompressionProvider;
    let error = provider
        .encode(EncodeRequest::new(CompressionCodec::Brotli, b"payload"))
        .expect_err("flate provider must not claim Brotli");

    assert_eq!(
        error,
        CompressionError::UnsupportedCodec {
            codec: CompressionCodec::Brotli,
        }
    );
}

#[test]
fn flate_provider_classifies_malformed_and_truncated_input() {
    let provider = FlateCompressionProvider;
    let limits = DecodeLimits::new(1024, 4096);

    let malformed = provider
        .decode(DecodeRequest::new(
            CompressionCodec::Gzip,
            b"not a gzip stream",
            limits,
        ))
        .expect_err("malformed input should fail");
    assert!(
        matches!(malformed, CompressionError::MalformedInput { .. }),
        "unexpected category: {malformed:?}"
    );

    let encoded = provider
        .encode(EncodeRequest::new(
            CompressionCodec::Gzip,
            b"truncated stream fixture",
        ))
        .expect("fixture should encode");
    let truncated_bytes = &encoded.bytes[..encoded.bytes.len() / 2];
    let truncated = provider
        .decode(DecodeRequest::new(
            CompressionCodec::Gzip,
            truncated_bytes,
            limits,
        ))
        .expect_err("truncated input should fail");
    assert!(
        matches!(truncated, CompressionError::TruncatedInput { .. }),
        "unexpected category: {truncated:?}"
    );
}
