use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use zune_jpeg::{
    zune_core::{
        bytestream::ZCursor, colorspace::ColorSpace as ZuneColorSpace, options::DecoderOptions,
    },
    JpegDecoder,
};

use crate::{
    AlphaMode, ColorSpace, DecodeLimits, DecodedImage, ImageOrientation, PixelFormat,
    RasterImageError,
};

const SOI: &[u8; 2] = b"\xFF\xD8";
const EOI: &[u8; 2] = b"\xFF\xD9";
const SOF0: u8 = 0xC0;
const SOS: u8 = 0xDA;
const APP0: u8 = 0xE0;
const APP1: u8 = 0xE1;
const APP2: u8 = 0xE2;
const JFIF_SIGNATURE: &[u8; 5] = b"JFIF\0";
const EXIF_SIGNATURE: &[u8; 6] = b"Exif\0\0";
const ICC_SIGNATURE: &[u8; 12] = b"ICC_PROFILE\0";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct JfifMetadata {
    pub version_major: u8,
    pub version_minor: u8,
    pub density_units: u8,
    pub x_density: u16,
    pub y_density: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct JpegIccMetadata {
    pub chunks_seen: usize,
    pub declared_chunks: u8,
    pub complete: bool,
}

/// The bounded JPEG source color model observed from the frame component count.
///
/// This is source evidence only. The current adapter always normalizes pixels
/// to RGBA8 and does not perform a color-management conversion.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum JpegColorModel {
    Grayscale,
    Ycbcr,
    Cmyk,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct JpegInspection {
    pub frame_marker: u8,
    pub width: u32,
    pub height: u32,
    pub precision: u8,
    pub components: u8,
    pub color_model: JpegColorModel,
    pub jfif: Option<JfifMetadata>,
    pub exif_orientation: Option<u16>,
    pub icc_profile: Option<JpegIccMetadata>,
}

#[derive(Default)]
struct IccAccumulator {
    declared_chunks: Option<u8>,
    sequences: BTreeSet<u8>,
}

pub fn decode_jpeg(source: &[u8], limits: DecodeLimits) -> Result<DecodedImage, RasterImageError> {
    let inspection = inspect_jpeg(source, limits)?;
    validate_profile(&inspection)?;

    let row_stride = (inspection.width as usize)
        .checked_mul(4)
        .ok_or(RasterImageError::DecodedSizeOverflow)?;
    let decoded_len = row_stride
        .checked_mul(inspection.height as usize)
        .ok_or(RasterImageError::DecodedSizeOverflow)?;
    if decoded_len > limits.max_decoded_bytes {
        return Err(RasterImageError::DecodedLimitExceeded {
            actual: decoded_len,
            limit: limits.max_decoded_bytes,
        });
    }

    let options = DecoderOptions::new_safe()
        .set_strict_mode(true)
        .set_max_width(limits.max_width as usize)
        .set_max_height(limits.max_height as usize)
        .jpeg_set_out_colorspace(ZuneColorSpace::RGBA);
    let mut decoder = JpegDecoder::new_with_options(ZCursor::new(source), options);
    let pixels = decoder
        .decode()
        .map_err(|error| RasterImageError::JpegDecode(error.to_string()))?;
    let info = decoder
        .info()
        .ok_or_else(|| RasterImageError::JpegDecode("decoder omitted image metadata".to_owned()))?;
    let actual_width = u32::from(info.width);
    let actual_height = u32::from(info.height);
    if actual_width != inspection.width || actual_height != inspection.height {
        return Err(RasterImageError::JpegHeaderMismatch {
            expected_width: inspection.width,
            expected_height: inspection.height,
            actual_width,
            actual_height,
        });
    }

    let source_row_stride = (inspection.width as usize)
        .checked_mul(inspection.components as usize)
        .ok_or(RasterImageError::DecodedSizeOverflow)?;
    let image = DecodedImage {
        width: inspection.width,
        height: inspection.height,
        row_stride,
        pixel_format: PixelFormat::Rgba8,
        color_space: ColorSpace::Unspecified,
        alpha_mode: AlphaMode::Opaque,
        source_orientation: ImageOrientation::TopDown,
        output_orientation: ImageOrientation::TopDown,
        source_bit_depth: u16::from(inspection.precision),
        source_row_stride,
        pixels,
    };
    image.validate()?;
    Ok(image)
}

pub fn inspect_jpeg(
    source: &[u8],
    limits: DecodeLimits,
) -> Result<JpegInspection, RasterImageError> {
    if source.len() > limits.max_source_bytes {
        return Err(RasterImageError::SourceLimitExceeded {
            actual: source.len(),
            limit: limits.max_source_bytes,
        });
    }

    if !source.starts_with(SOI) {
        return Err(RasterImageError::InvalidJpegSignature);
    }
    // The bounded corpus profile requires a complete JPEG stream. The header
    // walk stops at SOS, so validate the entropy-coded tail explicitly before
    // delegating to the provider decoder.
    if !source.ends_with(EOI) {
        return Err(RasterImageError::MissingJpegEnd);
    }

    let mut offset = SOI.len();
    let mut frame = None;
    let mut jfif = None;
    let mut exif_orientation = None;
    let mut icc = IccAccumulator::default();
    while offset < source.len() {
        while source.get(offset) == Some(&0xFF) {
            offset += 1;
        }
        let marker = *source
            .get(offset)
            .ok_or(RasterImageError::TruncatedJpegMarker { offset })?;
        offset += 1;

        if marker == 0x00 {
            continue;
        }
        if marker == SOS {
            break;
        }
        if marker == 0xD9 {
            break;
        }
        if marker == 0x01 || (0xD0..=0xD8).contains(&marker) {
            continue;
        }

        let length_offset = offset;
        let segment_len = read_be_u16(source, length_offset)? as usize;
        if segment_len < 2 {
            return Err(RasterImageError::TruncatedJpegMarker {
                offset: length_offset,
            });
        }
        let segment_end = offset
            .checked_add(segment_len)
            .ok_or(RasterImageError::DecodedSizeOverflow)?;
        if segment_end > source.len() {
            return Err(RasterImageError::TruncatedJpegMarker {
                offset: length_offset,
            });
        }

        let payload = &source[offset + 2..segment_end];
        match marker {
            APP0 if payload.starts_with(JFIF_SIGNATURE) => {
                jfif = Some(parse_jfif(payload)?);
            }
            APP1 if payload.starts_with(EXIF_SIGNATURE) => {
                exif_orientation = parse_exif_orientation(payload)?;
            }
            APP2 if payload.starts_with(ICC_SIGNATURE) => {
                observe_icc_chunk(payload, &mut icc)?;
            }
            _ => {}
        }

        if is_start_of_frame(marker) && frame.is_none() {
            if segment_len < 8 {
                return Err(RasterImageError::TruncatedJpegMarker {
                    offset: length_offset,
                });
            }
            let components = source[offset + 7];
            let expected_len = 8_usize
                .checked_add(
                    (components as usize)
                        .checked_mul(3)
                        .ok_or(RasterImageError::DecodedSizeOverflow)?,
                )
                .ok_or(RasterImageError::DecodedSizeOverflow)?;
            if segment_len != expected_len {
                return Err(RasterImageError::InvalidJpegFrameLength {
                    declared: segment_len,
                    expected: expected_len,
                });
            }
            frame = Some(JpegInspection {
                frame_marker: marker,
                precision: source[offset + 2],
                height: u32::from(read_be_u16(source, offset + 3)?),
                width: u32::from(read_be_u16(source, offset + 5)?),
                components,
                color_model: color_model_for_components(components)?,
                jfif: None,
                exif_orientation: None,
                icc_profile: None,
            });
        }
        offset = segment_end;
    }

    let mut inspection = frame.ok_or(RasterImageError::MissingJpegFrame)?;
    validate_dimensions(&inspection, limits)?;
    inspection.jfif = jfif;
    inspection.exif_orientation = exif_orientation;
    inspection.icc_profile = finish_icc(icc)?;
    Ok(inspection)
}

fn validate_dimensions(
    inspection: &JpegInspection,
    limits: DecodeLimits,
) -> Result<(), RasterImageError> {
    if inspection.width == 0 || inspection.height == 0 {
        return Err(RasterImageError::InvalidJpegDimensions);
    }
    if inspection.width > limits.max_width || inspection.height > limits.max_height {
        return Err(RasterImageError::DimensionLimitExceeded {
            width: inspection.width,
            height: inspection.height,
            max_width: limits.max_width,
            max_height: limits.max_height,
        });
    }
    Ok(())
}

fn validate_profile(inspection: &JpegInspection) -> Result<(), RasterImageError> {
    if inspection.precision != 8 {
        return Err(RasterImageError::UnsupportedJpegPrecision(
            inspection.precision,
        ));
    }
    if inspection.frame_marker != SOF0 {
        return Err(RasterImageError::UnsupportedJpegFrame {
            marker: inspection.frame_marker,
        });
    }
    if matches!(inspection.color_model, JpegColorModel::Cmyk) {
        return Err(RasterImageError::UnsupportedJpegComponents(
            inspection.components,
        ));
    }
    Ok(())
}

fn color_model_for_components(components: u8) -> Result<JpegColorModel, RasterImageError> {
    match components {
        1 => Ok(JpegColorModel::Grayscale),
        3 => Ok(JpegColorModel::Ycbcr),
        4 => Ok(JpegColorModel::Cmyk),
        _ => Err(RasterImageError::UnsupportedJpegComponents(components)),
    }
}

fn parse_jfif(payload: &[u8]) -> Result<JfifMetadata, RasterImageError> {
    if payload.len() < 14 {
        return Err(RasterImageError::InvalidJpegMetadata(
            "JFIF application segment is shorter than its fixed header",
        ));
    }
    Ok(JfifMetadata {
        version_major: payload[5],
        version_minor: payload[6],
        density_units: payload[7],
        x_density: u16::from_be_bytes([payload[8], payload[9]]),
        y_density: u16::from_be_bytes([payload[10], payload[11]]),
    })
}

fn parse_exif_orientation(payload: &[u8]) -> Result<Option<u16>, RasterImageError> {
    let tiff = payload
        .get(EXIF_SIGNATURE.len()..)
        .ok_or(RasterImageError::InvalidJpegMetadata(
            "EXIF application segment omitted its TIFF header",
        ))?;
    if tiff.len() < 8 {
        return Err(RasterImageError::InvalidJpegMetadata(
            "EXIF TIFF header is truncated",
        ));
    }

    let little_endian = match &tiff[..2] {
        b"II" => true,
        b"MM" => false,
        _ => {
            return Err(RasterImageError::InvalidJpegMetadata(
                "EXIF TIFF byte order is invalid",
            ))
        }
    };
    if read_tiff_u16(tiff, 2, little_endian)? != 42 {
        return Err(RasterImageError::InvalidJpegMetadata(
            "EXIF TIFF magic is invalid",
        ));
    }
    let ifd_offset = read_tiff_u32(tiff, 4, little_endian)? as usize;
    let entry_count = read_tiff_u16(tiff, ifd_offset, little_endian)? as usize;
    let entries_start = ifd_offset
        .checked_add(2)
        .ok_or(RasterImageError::DecodedSizeOverflow)?;

    for index in 0..entry_count {
        let entry = entries_start
            .checked_add(
                index
                    .checked_mul(12)
                    .ok_or(RasterImageError::DecodedSizeOverflow)?,
            )
            .ok_or(RasterImageError::DecodedSizeOverflow)?;
        let tag = read_tiff_u16(tiff, entry, little_endian)?;
        if tag != 0x0112 {
            continue;
        }
        let field_type = read_tiff_u16(tiff, entry + 2, little_endian)?;
        let count = read_tiff_u32(tiff, entry + 4, little_endian)?;
        if field_type != 3 || count != 1 {
            return Err(RasterImageError::InvalidJpegMetadata(
                "EXIF orientation must be one SHORT value",
            ));
        }
        let orientation = read_tiff_u16(tiff, entry + 8, little_endian)?;
        if !(1..=8).contains(&orientation) {
            return Err(RasterImageError::InvalidJpegMetadata(
                "EXIF orientation is outside the defined range",
            ));
        }
        return Ok(Some(orientation));
    }
    Ok(None)
}

fn observe_icc_chunk(
    payload: &[u8],
    accumulator: &mut IccAccumulator,
) -> Result<(), RasterImageError> {
    if payload.len() < ICC_SIGNATURE.len() + 2 {
        return Err(RasterImageError::InvalidJpegMetadata(
            "ICC application segment omitted its sequence fields",
        ));
    }
    let sequence = payload[ICC_SIGNATURE.len()];
    let declared = payload[ICC_SIGNATURE.len() + 1];
    if sequence == 0 || declared == 0 || sequence > declared {
        return Err(RasterImageError::InvalidJpegMetadata(
            "ICC chunk sequence is invalid",
        ));
    }
    if let Some(previous) = accumulator.declared_chunks {
        if previous != declared {
            return Err(RasterImageError::InvalidJpegMetadata(
                "ICC chunks disagree about the declared chunk count",
            ));
        }
    } else {
        accumulator.declared_chunks = Some(declared);
    }
    if !accumulator.sequences.insert(sequence) {
        return Err(RasterImageError::InvalidJpegMetadata(
            "ICC chunk sequence is duplicated",
        ));
    }
    Ok(())
}

fn finish_icc(accumulator: IccAccumulator) -> Result<Option<JpegIccMetadata>, RasterImageError> {
    let Some(declared_chunks) = accumulator.declared_chunks else {
        return Ok(None);
    };
    let complete = accumulator.sequences.len() == usize::from(declared_chunks)
        && (1..=declared_chunks).all(|sequence| accumulator.sequences.contains(&sequence));
    Ok(Some(JpegIccMetadata {
        chunks_seen: accumulator.sequences.len(),
        declared_chunks,
        complete,
    }))
}

fn is_start_of_frame(marker: u8) -> bool {
    matches!(
        marker,
        0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF
    )
}

fn read_be_u16(source: &[u8], offset: usize) -> Result<u16, RasterImageError> {
    let bytes = source
        .get(offset..offset + 2)
        .ok_or(RasterImageError::TruncatedJpegMarker { offset })?;
    Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
}

fn read_tiff_u16(
    source: &[u8],
    offset: usize,
    little_endian: bool,
) -> Result<u16, RasterImageError> {
    let bytes = source
        .get(offset..offset + 2)
        .ok_or(RasterImageError::InvalidJpegMetadata(
            "EXIF TIFF value is truncated",
        ))?;
    Ok(if little_endian {
        u16::from_le_bytes([bytes[0], bytes[1]])
    } else {
        u16::from_be_bytes([bytes[0], bytes[1]])
    })
}

fn read_tiff_u32(
    source: &[u8],
    offset: usize,
    little_endian: bool,
) -> Result<u32, RasterImageError> {
    let bytes = source
        .get(offset..offset + 4)
        .ok_or(RasterImageError::InvalidJpegMetadata(
            "EXIF TIFF value is truncated",
        ))?;
    Ok(if little_endian {
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    } else {
        u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    })
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Cursor, path::PathBuf};

    use jpeg_decoder::{Decoder as ReferenceDecoder, PixelFormat as ReferencePixelFormat};

    use super::{
        decode_jpeg, inspect_jpeg, JfifMetadata, JpegColorModel, JpegIccMetadata, APP1, APP2,
        ICC_SIGNATURE, SOF0, SOI,
    };
    use crate::{DecodeLimits, RasterImageError};

    fn fixture(name: &str) -> Vec<u8> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages")
            .join(name);
        fs::read(path).unwrap()
    }

    fn external_fixture(name: &str) -> Vec<u8> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../third-party/fixtures/raster-images/upstream/jpeg-decoder")
            .join(name);
        fs::read(path).unwrap()
    }

    fn decode_reference_rgba(source: &[u8]) -> (u32, u32, Vec<u8>) {
        let mut decoder = ReferenceDecoder::new(Cursor::new(source));
        let pixels = decoder.decode().unwrap();
        let info = decoder.info().unwrap();
        let width = u32::from(info.width);
        let height = u32::from(info.height);
        let rgba = match info.pixel_format {
            ReferencePixelFormat::L8 => pixels
                .into_iter()
                .flat_map(|value| [value, value, value, 255])
                .collect(),
            ReferencePixelFormat::RGB24 => pixels
                .chunks_exact(3)
                .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], 255])
                .collect(),
            format => panic!("reference decoder returned unsupported format {format:?}"),
        };
        (width, height, rgba)
    }

    fn maximum_rgb_delta(actual: &[u8], reference: &[u8]) -> u8 {
        actual
            .chunks_exact(4)
            .zip(reference.chunks_exact(4))
            .flat_map(|(actual, reference)| {
                (0..3).map(move |channel| actual[channel].abs_diff(reference[channel]))
            })
            .max()
            .unwrap_or_default()
    }

    fn mean_rgb_delta(actual: &[u8], reference: &[u8]) -> f64 {
        let mut total = 0_u64;
        let mut channels = 0_u64;
        for (actual, reference) in actual.chunks_exact(4).zip(reference.chunks_exact(4)) {
            for channel in 0..3 {
                total += u64::from(actual[channel].abs_diff(reference[channel]));
                channels += 1;
            }
        }
        if channels == 0 {
            0.0
        } else {
            total as f64 / channels as f64
        }
    }

    fn insert_app_segment(source: &[u8], marker: u8, payload: &[u8]) -> Vec<u8> {
        let segment_len = u16::try_from(payload.len() + 2).unwrap();
        let mut output = Vec::with_capacity(source.len() + payload.len() + 4);
        output.extend_from_slice(SOI);
        output.extend_from_slice(&[0xFF, marker]);
        output.extend_from_slice(&segment_len.to_be_bytes());
        output.extend_from_slice(payload);
        output.extend_from_slice(&source[SOI.len()..]);
        output
    }

    #[test]
    fn preflight_reads_baseline_frame() {
        let inspection = inspect_jpeg(&fixture("testorig.jpg"), DecodeLimits::default()).unwrap();
        assert_eq!(inspection.frame_marker, SOF0);
        assert_eq!((inspection.width, inspection.height), (227, 149));
        assert_eq!(inspection.precision, 8);
        assert_eq!(inspection.components, 3);
        assert_eq!(inspection.color_model, JpegColorModel::Ycbcr);
        assert_eq!(
            inspection.jfif,
            Some(JfifMetadata {
                version_major: 1,
                version_minor: 1,
                density_units: 0,
                x_density: 1,
                y_density: 1,
            })
        );
        assert_eq!(inspection.exif_orientation, None);
        assert_eq!(inspection.icc_profile, None);
    }

    #[test]
    fn observes_exif_orientation_and_complete_icc_chunks() {
        let exif = [
            b'E', b'x', b'i', b'f', 0, 0, b'I', b'I', 42, 0, 8, 0, 0, 0, 1, 0, 0x12, 0x01, 3, 0, 1,
            0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0,
        ];
        let mut icc = ICC_SIGNATURE.to_vec();
        icc.extend_from_slice(&[1, 1, 0xCA, 0xFE]);
        let source = insert_app_segment(&fixture("testorig.jpg"), APP2, &icc);
        let source = insert_app_segment(&source, APP1, &exif);

        let inspection = inspect_jpeg(&source, DecodeLimits::default()).unwrap();
        assert_eq!(inspection.exif_orientation, Some(6));
        assert_eq!(
            inspection.icc_profile,
            Some(JpegIccMetadata {
                chunks_seen: 1,
                declared_chunks: 1,
                complete: true,
            })
        );
    }

    #[test]
    fn reports_incomplete_icc_and_rejects_invalid_metadata() {
        let mut incomplete_icc = ICC_SIGNATURE.to_vec();
        incomplete_icc.extend_from_slice(&[1, 2, 0xCA, 0xFE]);
        let source = insert_app_segment(&fixture("testorig.jpg"), APP2, &incomplete_icc);
        assert_eq!(
            inspect_jpeg(&source, DecodeLimits::default())
                .unwrap()
                .icc_profile,
            Some(JpegIccMetadata {
                chunks_seen: 1,
                declared_chunks: 2,
                complete: false,
            })
        );

        let invalid_exif = [b'E', b'x', b'i', b'f', 0, 0, b'?', b'?', 0, 0, 0, 0, 0, 0];
        let source = insert_app_segment(&fixture("testorig.jpg"), APP1, &invalid_exif);
        assert_eq!(
            inspect_jpeg(&source, DecodeLimits::default()),
            Err(RasterImageError::InvalidJpegMetadata(
                "EXIF TIFF byte order is invalid"
            ))
        );
    }

    #[test]
    fn decodes_admitted_baseline_fixtures_to_rgba() {
        for (name, expected_fingerprint) in [
            ("testorig.jpg", "230001e9c18f35c3"),
            ("testimgint.jpg", "9b3db4663a9f8009"),
        ] {
            let inspection = inspect_jpeg(&fixture(name), DecodeLimits::default()).unwrap();
            assert_eq!(inspection.color_model, JpegColorModel::Ycbcr, "{name}");
            let image = decode_jpeg(&fixture(name), DecodeLimits::default()).unwrap();
            assert_eq!((image.width, image.height), (227, 149));
            assert_eq!(image.pixels.len(), 227 * 149 * 4);
            assert!(image.pixels.chunks_exact(4).all(|pixel| pixel[3] == 255));
            assert_eq!(image.pixel_fingerprint(), expected_fingerprint);
        }
    }

    #[test]
    fn decodes_admitted_baseline_grayscale_fixture_to_opaque_rgba() {
        let source = external_fixture("grayscale_square.jpg");
        let inspection = inspect_jpeg(&source, DecodeLimits::default()).unwrap();
        assert_eq!(inspection.frame_marker, SOF0);
        assert_eq!((inspection.width, inspection.height), (10, 10));
        assert_eq!(inspection.precision, 8);
        assert_eq!(inspection.components, 1);
        assert_eq!(inspection.color_model, JpegColorModel::Grayscale);

        let image = decode_jpeg(&source, DecodeLimits::default()).unwrap();
        assert_eq!((image.width, image.height), (10, 10));
        assert_eq!(image.pixels.len(), 10 * 10 * 4);
        assert!(image
            .pixels
            .chunks_exact(4)
            .all(|pixel| { pixel[0] == pixel[1] && pixel[1] == pixel[2] && pixel[3] == 255 }));
        assert_eq!(image.pixel_fingerprint(), "10d9b58400c38503");
    }

    #[test]
    fn baseline_fixture_output_agrees_with_independent_reference_decoder() {
        // The comparison is deliberately test-only. `zune-jpeg` remains the
        // production provider; `jpeg-decoder` is a named independent oracle.
        for (name, source) in [
            ("testorig.jpg", fixture("testorig.jpg")),
            ("testimgint.jpg", fixture("testimgint.jpg")),
            (
                "grayscale_square.jpg",
                external_fixture("grayscale_square.jpg"),
            ),
        ] {
            let decoded = decode_jpeg(&source, DecodeLimits::default()).unwrap();
            let (width, height, reference) = decode_reference_rgba(&source);
            assert_eq!((decoded.width, decoded.height), (width, height), "{name}");
            assert_eq!(decoded.pixels.len(), reference.len(), "{name}");
            assert!(decoded.pixels.chunks_exact(4).all(|pixel| pixel[3] == 255));

            let maximum_delta = maximum_rgb_delta(&decoded.pixels, &reference);
            let mean_delta = mean_rgb_delta(&decoded.pixels, &reference);
            eprintln!(
                "{name}: zune-jpeg/jpeg-decoder maximum RGB delta = {maximum_delta}, mean = {mean_delta:.4}"
            );
            assert!(
                maximum_delta <= 4,
                "{name}: decoded RGB differs from the reference by {maximum_delta}, exceeding the reviewed 4-level bound"
            );
            assert!(
                mean_delta <= 0.5,
                "{name}: decoded RGB mean absolute error is {mean_delta:.4}, exceeding the reviewed 0.5-level bound"
            );
        }
    }

    #[test]
    fn rejects_arithmetic_frame_before_provider_decode() {
        assert_eq!(
            decode_jpeg(&fixture("testimgari.jpg"), DecodeLimits::default()),
            Err(RasterImageError::UnsupportedJpegFrame { marker: 0xC9 })
        );
    }

    #[test]
    fn rejects_twelve_bit_frame_before_provider_decode() {
        assert_eq!(
            decode_jpeg(&fixture("monkey12.jpg"), DecodeLimits::default()),
            Err(RasterImageError::UnsupportedJpegPrecision(12))
        );
    }

    #[test]
    fn classifies_grayscale_source_frames_without_decoding_them() {
        let mut source = fixture("testorig.jpg");
        let marker = source
            .windows(2)
            .position(|bytes| bytes == [0xFF, SOF0])
            .unwrap();
        let frame = marker + 2;
        source[frame + 7] = 1;
        source.drain(frame + 11..frame + 17);
        let frame_length = 11_u16.to_be_bytes();
        source[frame..frame + 2].copy_from_slice(&frame_length);

        let inspection = inspect_jpeg(&source, DecodeLimits::default()).unwrap();
        assert_eq!(inspection.components, 1);
        assert_eq!(inspection.color_model, JpegColorModel::Grayscale);
    }

    #[test]
    fn classifies_cmyk_frames_and_rejects_them_before_provider_decode() {
        let mut source = fixture("testorig.jpg");
        let marker = source
            .windows(2)
            .position(|bytes| bytes == [0xFF, SOF0])
            .unwrap();
        let frame = marker + 2;
        source[frame + 7] = 4;
        source.splice(frame + 17..frame + 17, [0, 0, 0]);
        source[frame..frame + 2].copy_from_slice(&20_u16.to_be_bytes());

        let inspection = inspect_jpeg(&source, DecodeLimits::default()).unwrap();
        assert_eq!(inspection.components, 4);
        assert_eq!(inspection.color_model, JpegColorModel::Cmyk);
        assert_eq!(
            decode_jpeg(&source, DecodeLimits::default()),
            Err(RasterImageError::UnsupportedJpegComponents(4))
        );
    }

    #[test]
    fn rejects_source_and_dimension_limits_before_decode() {
        let source = fixture("testorig.jpg");
        assert_eq!(
            decode_jpeg(
                &source,
                DecodeLimits {
                    max_source_bytes: source.len() - 1,
                    ..DecodeLimits::default()
                }
            ),
            Err(RasterImageError::SourceLimitExceeded {
                actual: source.len(),
                limit: source.len() - 1,
            })
        );
        assert_eq!(
            decode_jpeg(
                &source,
                DecodeLimits {
                    max_width: 226,
                    ..DecodeLimits::default()
                }
            ),
            Err(RasterImageError::DimensionLimitExceeded {
                width: 227,
                height: 149,
                max_width: 226,
                max_height: DecodeLimits::default().max_height,
            })
        );
    }

    #[test]
    fn rejects_decoded_size_limit_before_provider_decode() {
        let source = fixture("testorig.jpg");
        let required = 227 * 149 * 4;
        assert_eq!(
            decode_jpeg(
                &source,
                DecodeLimits {
                    max_decoded_bytes: required - 1,
                    ..DecodeLimits::default()
                }
            ),
            Err(RasterImageError::DecodedLimitExceeded {
                actual: required,
                limit: required - 1,
            })
        );
    }

    #[test]
    fn rejects_invalid_and_truncated_marker_framing() {
        assert_eq!(
            decode_jpeg(b"not a jpeg", DecodeLimits::default()),
            Err(RasterImageError::InvalidJpegSignature)
        );
        assert_eq!(
            decode_jpeg(b"\xFF\xD8\xFF\xE0\x00\x10", DecodeLimits::default()),
            Err(RasterImageError::MissingJpegEnd)
        );

        let source = fixture("testorig.jpg");
        assert_eq!(
            decode_jpeg(&source[..source.len() - 1], DecodeLimits::default()),
            Err(RasterImageError::MissingJpegEnd)
        );
    }

    #[test]
    fn rejects_progressive_profile_explicitly() {
        let mut source = fixture("testorig.jpg");
        let marker = source
            .windows(2)
            .position(|bytes| bytes == [0xFF, SOF0])
            .unwrap();
        source[marker + 1] = 0xC2;

        assert_eq!(
            decode_jpeg(&source, DecodeLimits::default()),
            Err(RasterImageError::UnsupportedJpegFrame { marker: 0xC2 })
        );
    }
}
