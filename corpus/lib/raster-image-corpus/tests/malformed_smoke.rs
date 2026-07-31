use std::{
    fs,
    panic::{catch_unwind, AssertUnwindSafe},
    path::PathBuf,
};

use raster_image_corpus::{decode_bmp, decode_jpeg, decode_png, DecodeLimits, RasterImageError};

struct Case {
    name: &'static str,
    source: &'static str,
    decode: fn(&[u8], DecodeLimits) -> Result<raster_image_corpus::DecodedImage, RasterImageError>,
}

#[test]
fn representative_truncations_reject_without_panicking() {
    let cases = [
        Case {
            name: "png",
            source: "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/basn6a08.png",
            decode: decode_png,
        },
        Case {
            name: "jpeg",
            source: "../../../third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages/testorig.jpg",
            decode: decode_jpeg,
        },
        Case {
            name: "bmp",
            source: "../../../third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages/shira_bird8.bmp",
            decode: decode_bmp,
        },
    ];

    for case in cases {
        let source = fixture(case.source);
        for cut in [0, 1, source.len() / 2, source.len() - 1] {
            let truncated = &source[..cut];
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                (case.decode)(truncated, DecodeLimits::default())
            }));
            assert!(
                outcome.is_ok(),
                "{} panicked while inspecting a {}-byte truncation",
                case.name,
                cut
            );
            assert!(
                outcome.unwrap().is_err(),
                "{} accepted a {}-byte truncation",
                case.name,
                cut
            );
        }
    }
}

#[test]
fn adversarial_headers_and_payloads_reject_without_panicking() {
    let mut png_dimension = fixture(
        "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/basn6a08.png",
    );
    png_dimension[16..20].copy_from_slice(&u32::MAX.to_be_bytes());
    rewrite_png_chunk_crc(&mut png_dimension, 8);

    let mut png_payload = fixture(
        "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/basn6a08.png",
    );
    let idat_offset = find_png_chunk(&png_payload, b"IDAT").expect("selected fixture has IDAT");
    let payload_offset = idat_offset + 8;
    png_payload[payload_offset] ^= 0xFF;
    rewrite_png_chunk_crc(&mut png_payload, idat_offset);

    let mut jpeg_dimension = fixture(
        "../../../third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages/testorig.jpg",
    );
    let sof_offset = jpeg_dimension
        .windows(2)
        .position(|bytes| bytes == [0xFF, 0xC0])
        .expect("selected fixture has SOF0");
    jpeg_dimension[sof_offset + 5..sof_offset + 7].copy_from_slice(&u16::MAX.to_be_bytes());

    let mut bmp_offset = fixture(
        "../../../third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages/shira_bird8.bmp",
    );
    bmp_offset[10..14].copy_from_slice(&14_u32.to_le_bytes());

    let cases = [
        ("png oversized IHDR", decode_png as DecodeFn, png_dimension),
        (
            "png corrupt deflate payload",
            decode_png as DecodeFn,
            png_payload,
        ),
        (
            "jpeg oversized SOF0",
            decode_jpeg as DecodeFn,
            jpeg_dimension,
        ),
        (
            "bmp pixel offset inside header",
            decode_bmp as DecodeFn,
            bmp_offset,
        ),
    ];
    for (name, decode, source) in cases {
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            decode(&source, DecodeLimits::default())
        }));
        assert!(outcome.is_ok(), "{name} panicked");
        assert!(outcome.unwrap().is_err(), "{name} decoded successfully");
    }
}

#[test]
fn deterministic_source_mutations_never_panic() {
    let cases = [
        (
            "png",
            decode_png as DecodeFn,
            fixture(
                "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/basn6a08.png",
            ),
        ),
        (
            "jpeg",
            decode_jpeg as DecodeFn,
            fixture(
                "../../../third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages/testorig.jpg",
            ),
        ),
        (
            "bmp",
            decode_bmp as DecodeFn,
            fixture(
                "../../../third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages/shira_bird8.bmp",
            ),
        ),
    ];

    for (name, decode, source) in cases {
        for offset in mutation_offsets(source.len()) {
            let mut mutated = source.clone();
            mutated[offset] ^= 0x5A;
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                decode(&mutated, DecodeLimits::default())
            }));
            assert!(
                outcome.is_ok(),
                "{name} panicked after mutating byte {offset}"
            );
        }
    }
}

type DecodeFn =
    fn(&[u8], DecodeLimits) -> Result<raster_image_corpus::DecodedImage, RasterImageError>;

fn find_png_chunk(source: &[u8], kind: &[u8; 4]) -> Option<usize> {
    let mut offset = 8;
    while offset + 12 <= source.len() {
        let payload_len = u32::from_be_bytes(source[offset..offset + 4].try_into().ok()?) as usize;
        let payload_end = offset.checked_add(8)?.checked_add(payload_len)?;
        if payload_end.checked_add(4)? > source.len() {
            return None;
        }
        if source[offset + 4..offset + 8] == *kind {
            return Some(offset);
        }
        offset = payload_end + 4;
    }
    None
}

fn rewrite_png_chunk_crc(source: &mut [u8], chunk_offset: usize) {
    let payload_len = u32::from_be_bytes(
        source[chunk_offset..chunk_offset + 4]
            .try_into()
            .expect("PNG chunk length"),
    ) as usize;
    let crc_start = chunk_offset + 4;
    let crc_end = chunk_offset + 8 + payload_len;
    let crc = png_crc32(&source[crc_start..crc_end]);
    source[crc_end..crc_end + 4].copy_from_slice(&crc.to_be_bytes());
}

fn png_crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFF_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 0 {
                crc >> 1
            } else {
                (crc >> 1) ^ 0xEDB8_8320
            };
        }
    }
    !crc
}

fn mutation_offsets(source_len: usize) -> impl Iterator<Item = usize> {
    let probes = [
        0,
        1,
        2,
        7,
        8,
        source_len / 8,
        source_len / 4,
        source_len / 2,
        source_len.saturating_sub(2),
        source_len.saturating_sub(1),
    ];
    probes
        .into_iter()
        .filter(move |offset| *offset < source_len)
}

fn fixture(relative_path: &str) -> Vec<u8> {
    fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path))
        .unwrap_or_else(|error| panic!("read {relative_path}: {error}"))
}
