use std::{fs, path::PathBuf};

use raster_image_corpus::{
    decode_bmp, decode_jpeg, decode_png, DecodeLimits, DecodedImage, RasterImageError,
};

struct Case {
    name: &'static str,
    source: &'static str,
    decode: fn(&[u8], DecodeLimits) -> Result<DecodedImage, RasterImageError>,
}

#[test]
fn selected_rasters_enforce_the_same_source_dimension_and_output_budgets() {
    let cases = [
        Case {
            name: "png",
            source: "../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/basn2c08.png",
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
        let image = (case.decode)(&source, DecodeLimits::default())
            .unwrap_or_else(|error| panic!("{} should decode: {error}", case.name));
        let decoded_bytes = image.pixels.len();

        assert_eq!(
            (case.decode)(
                &source,
                DecodeLimits {
                    max_source_bytes: source.len() - 1,
                    ..DecodeLimits::default()
                },
            ),
            Err(RasterImageError::SourceLimitExceeded {
                actual: source.len(),
                limit: source.len() - 1,
            }),
            "{} must reject before parsing beyond its source budget",
            case.name
        );

        assert_eq!(
            (case.decode)(
                &source,
                DecodeLimits {
                    max_width: image.width - 1,
                    ..DecodeLimits::default()
                },
            ),
            Err(RasterImageError::DimensionLimitExceeded {
                width: image.width,
                height: image.height,
                max_width: image.width - 1,
                max_height: DecodeLimits::default().max_height,
            }),
            "{} must reject declared dimensions before decode allocation",
            case.name
        );

        assert_eq!(
            (case.decode)(
                &source,
                DecodeLimits {
                    max_decoded_bytes: decoded_bytes - 1,
                    ..DecodeLimits::default()
                },
            ),
            Err(RasterImageError::DecodedLimitExceeded {
                actual: decoded_bytes,
                limit: decoded_bytes - 1,
            }),
            "{} must reject output beyond its decoded-byte budget",
            case.name
        );
    }
}

fn fixture(relative_path: &str) -> Vec<u8> {
    fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path))
        .unwrap_or_else(|error| panic!("read {relative_path}: {error}"))
}
