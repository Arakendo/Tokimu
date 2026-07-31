use std::io::Read;

use crc32fast::Hasher;
use flate2::read::ZlibDecoder;
use serde::{Deserialize, Serialize};

use crate::{
    AlphaMode, ColorSpace, DecodeLimits, DecodedImage, ImageOrientation, PixelFormat,
    RasterImageError,
};

const SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

#[derive(Default)]
struct PngState {
    header: Option<Header>,
    palette: Option<Vec<[u8; 3]>>,
    transparency: Option<Vec<u8>>,
    compressed: Vec<u8>,
    saw_data: bool,
    ended_data: bool,
    saw_end: bool,
    color_space: ColorSpace,
    srgb_rendering_intent: Option<u8>,
    gamma_times_100000: Option<u32>,
    icc_profile: Option<PngIccMetadata>,
}

#[derive(Clone, Copy)]
struct Header {
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: u8,
    channels: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PngIccMetadata {
    pub profile_name: String,
    pub compressed_bytes: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PngInspection {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: u8,
    pub srgb_rendering_intent: Option<u8>,
    pub gamma_times_100000: Option<u32>,
    pub icc_profile: Option<PngIccMetadata>,
}

pub fn decode_png(source: &[u8], limits: DecodeLimits) -> Result<DecodedImage, RasterImageError> {
    let (state, header) = parse_png(source, limits)?;
    decode_pixels(state, header, limits)
}

pub fn inspect_png(source: &[u8], limits: DecodeLimits) -> Result<PngInspection, RasterImageError> {
    let (state, header) = parse_png(source, limits)?;
    Ok(PngInspection {
        width: header.width,
        height: header.height,
        bit_depth: header.bit_depth,
        color_type: header.color_type,
        srgb_rendering_intent: state.srgb_rendering_intent,
        gamma_times_100000: state.gamma_times_100000,
        icc_profile: state.icc_profile,
    })
}

fn parse_png(source: &[u8], limits: DecodeLimits) -> Result<(PngState, Header), RasterImageError> {
    if source.len() > limits.max_source_bytes {
        return Err(RasterImageError::SourceLimitExceeded {
            actual: source.len(),
            limit: limits.max_source_bytes,
        });
    }
    if !source.starts_with(SIGNATURE) {
        return Err(RasterImageError::InvalidPngSignature);
    }

    let mut state = PngState::default();
    let mut offset = SIGNATURE.len();
    while offset < source.len() {
        let chunk_start = offset;
        let length = read_u32(source, offset)? as usize;
        let data_start = offset
            .checked_add(8)
            .ok_or(RasterImageError::DecodedSizeOverflow)?;
        let data_end = data_start
            .checked_add(length)
            .ok_or(RasterImageError::DecodedSizeOverflow)?;
        let chunk_end = data_end
            .checked_add(4)
            .ok_or(RasterImageError::DecodedSizeOverflow)?;
        if chunk_end > source.len() {
            return Err(RasterImageError::TruncatedPngChunk {
                offset: chunk_start,
            });
        }

        let kind = &source[offset + 4..offset + 8];
        let data = &source[data_start..data_end];
        validate_crc(kind, data, &source[data_end..chunk_end])?;
        process_chunk(&mut state, kind, data, limits)?;
        offset = chunk_end;
        if state.saw_end {
            if offset != source.len() {
                return Err(RasterImageError::TruncatedPngChunk { offset });
            }
            break;
        }
    }

    let header = state
        .header
        .ok_or(RasterImageError::InvalidPngHeaderOrder)?;
    if !state.saw_data || !state.saw_end {
        return Err(RasterImageError::IncompletePng);
    }
    Ok((state, header))
}

fn process_chunk(
    state: &mut PngState,
    kind: &[u8],
    data: &[u8],
    limits: DecodeLimits,
) -> Result<(), RasterImageError> {
    if state.saw_end {
        return Err(RasterImageError::IncompletePng);
    }
    if kind != b"IDAT" && state.saw_data {
        state.ended_data = true;
    }

    match kind {
        b"IHDR" => {
            if state.header.is_some() || state.saw_data {
                return Err(RasterImageError::InvalidPngHeaderOrder);
            }
            state.header = Some(parse_header(data, limits)?);
        }
        b"PLTE" => {
            require_header(state)?;
            if state.saw_data
                || state.palette.is_some()
                || data.is_empty()
                || !data.len().is_multiple_of(3)
                || data.len() > 768
            {
                return Err(RasterImageError::InvalidPngPaletteLength(data.len()));
            }
            state.palette = Some(
                data.chunks_exact(3)
                    .map(|entry| [entry[0], entry[1], entry[2]])
                    .collect(),
            );
        }
        b"tRNS" => {
            let header = require_header(state)?;
            if state.saw_data
                || state.transparency.is_some()
                || !valid_transparency_length(header, state, data.len())
            {
                return Err(RasterImageError::InvalidPngTransparency {
                    color_type: header.color_type,
                });
            }
            state.transparency = Some(data.to_vec());
        }
        b"sRGB" => {
            require_header(state)?;
            if state.saw_data || state.palette.is_some() {
                return Err(RasterImageError::InvalidPngMetadata(
                    "sRGB must precede PLTE and IDAT",
                ));
            }
            if state.srgb_rendering_intent.is_some() {
                return Err(RasterImageError::InvalidPngMetadata("duplicate sRGB chunk"));
            }
            if state.icc_profile.is_some() {
                return Err(RasterImageError::InvalidPngMetadata(
                    "sRGB and iCCP cannot both occur",
                ));
            }
            let intent = *data.first().filter(|_| data.len() == 1).ok_or(
                RasterImageError::InvalidPngMetadata("sRGB must contain one rendering-intent byte"),
            )?;
            if intent > 3 {
                return Err(RasterImageError::InvalidPngMetadata(
                    "sRGB rendering intent must be in 0..=3",
                ));
            }
            state.srgb_rendering_intent = Some(intent);
            state.color_space = ColorSpace::Srgb;
        }
        b"gAMA" => {
            require_header(state)?;
            if state.saw_data || state.palette.is_some() {
                return Err(RasterImageError::InvalidPngMetadata(
                    "gAMA must precede PLTE and IDAT",
                ));
            }
            if state.gamma_times_100000.is_some() {
                return Err(RasterImageError::InvalidPngMetadata("duplicate gAMA chunk"));
            }
            let gamma =
                u32::from_be_bytes(data.try_into().map_err(|_| {
                    RasterImageError::InvalidPngMetadata("gAMA must be four bytes")
                })?);
            if gamma == 0 {
                return Err(RasterImageError::InvalidPngMetadata(
                    "gAMA value must be non-zero",
                ));
            }
            state.gamma_times_100000 = Some(gamma);
        }
        b"iCCP" => {
            require_header(state)?;
            if state.saw_data || state.palette.is_some() {
                return Err(RasterImageError::InvalidPngMetadata(
                    "iCCP must precede PLTE and IDAT",
                ));
            }
            if state.icc_profile.is_some() {
                return Err(RasterImageError::InvalidPngMetadata("duplicate iCCP chunk"));
            }
            if state.srgb_rendering_intent.is_some() {
                return Err(RasterImageError::InvalidPngMetadata(
                    "sRGB and iCCP cannot both occur",
                ));
            }
            state.icc_profile = Some(parse_iccp(data)?);
        }
        b"IDAT" => {
            let header = require_header(state)?;
            if state.ended_data {
                return Err(RasterImageError::NonConsecutivePngData);
            }
            if header.color_type == 3 && state.palette.is_none() {
                return Err(RasterImageError::MissingPngPalette);
            }
            let combined = state
                .compressed
                .len()
                .checked_add(data.len())
                .ok_or(RasterImageError::DecodedSizeOverflow)?;
            if combined > limits.max_source_bytes {
                return Err(RasterImageError::SourceLimitExceeded {
                    actual: combined,
                    limit: limits.max_source_bytes,
                });
            }
            state.compressed.extend_from_slice(data);
            state.saw_data = true;
        }
        b"IEND" => {
            require_header(state)?;
            if !data.is_empty() {
                return Err(RasterImageError::IncompletePng);
            }
            state.saw_end = true;
        }
        _ if kind[0].is_ascii_uppercase() => {
            return Err(RasterImageError::UnsupportedCriticalPngChunk(
                String::from_utf8_lossy(kind).into_owned(),
            ));
        }
        _ => {
            require_header(state)?;
        }
    }
    Ok(())
}

fn parse_iccp(data: &[u8]) -> Result<PngIccMetadata, RasterImageError> {
    let separator =
        data.iter()
            .position(|byte| *byte == 0)
            .ok_or(RasterImageError::InvalidPngMetadata(
                "iCCP profile name is not terminated",
            ))?;
    if separator == 0 || separator > 79 {
        return Err(RasterImageError::InvalidPngMetadata(
            "iCCP profile name must contain 1..=79 bytes",
        ));
    }
    let compression_method =
        *data
            .get(separator + 1)
            .ok_or(RasterImageError::InvalidPngMetadata(
                "iCCP compression method is missing",
            ))?;
    if compression_method != 0 {
        return Err(RasterImageError::InvalidPngMetadata(
            "iCCP compression method must be zero",
        ));
    }
    let compressed_bytes = data.len().saturating_sub(separator + 2);
    if compressed_bytes == 0 {
        return Err(RasterImageError::InvalidPngMetadata(
            "iCCP compressed profile is empty",
        ));
    }

    Ok(PngIccMetadata {
        profile_name: data[..separator]
            .iter()
            .map(|byte| char::from(*byte))
            .collect(),
        compressed_bytes,
    })
}

fn parse_header(data: &[u8], limits: DecodeLimits) -> Result<Header, RasterImageError> {
    if data.len() != 13 {
        return Err(RasterImageError::InvalidPngHeaderLength(data.len()));
    }
    let width = u32::from_be_bytes(data[0..4].try_into().unwrap());
    let height = u32::from_be_bytes(data[4..8].try_into().unwrap());
    if width == 0 || height == 0 {
        return Err(RasterImageError::InvalidPngDimensions);
    }
    if width > limits.max_width || height > limits.max_height {
        return Err(RasterImageError::DimensionLimitExceeded {
            width,
            height,
            max_width: limits.max_width,
            max_height: limits.max_height,
        });
    }
    let bit_depth = data[8];
    let color_type = data[9];
    let channels = match color_type {
        0 => 1,
        2 => 3,
        3 => 1,
        4 => 2,
        6 => 4,
        other => return Err(RasterImageError::UnsupportedPngColorType(other)),
    };
    let supported_bit_depth = match color_type {
        0 | 3 => matches!(bit_depth, 1 | 2 | 4 | 8),
        2 | 4 | 6 => bit_depth == 8,
        _ => false,
    };
    if !supported_bit_depth {
        return Err(RasterImageError::UnsupportedPngBitDepth {
            bit_depth,
            color_type,
        });
    }
    if data[10] != 0 {
        return Err(RasterImageError::UnsupportedPngCompression(data[10]));
    }
    if data[11] != 0 {
        return Err(RasterImageError::UnsupportedPngFilterMethod(data[11]));
    }
    if data[12] != 0 {
        return Err(RasterImageError::UnsupportedPngInterlace(data[12]));
    }
    Ok(Header {
        width,
        height,
        bit_depth,
        color_type,
        channels,
    })
}

fn decode_pixels(
    state: PngState,
    header: Header,
    limits: DecodeLimits,
) -> Result<DecodedImage, RasterImageError> {
    let bits_per_row = (header.width as usize)
        .checked_mul(header.channels)
        .and_then(|samples| samples.checked_mul(usize::from(header.bit_depth)))
        .ok_or(RasterImageError::DecodedSizeOverflow)?;
    let source_stride = bits_per_row
        .checked_add(7)
        .ok_or(RasterImageError::DecodedSizeOverflow)?
        / 8;
    let bytes_per_pixel = header
        .channels
        .checked_mul(usize::from(header.bit_depth))
        .ok_or(RasterImageError::DecodedSizeOverflow)?
        .div_ceil(8)
        .max(1);
    let encoded_stride = source_stride
        .checked_add(1)
        .ok_or(RasterImageError::DecodedSizeOverflow)?;
    let expected = encoded_stride
        .checked_mul(header.height as usize)
        .ok_or(RasterImageError::DecodedSizeOverflow)?;
    if expected > limits.max_decoded_bytes {
        return Err(RasterImageError::DecodedLimitExceeded {
            actual: expected,
            limit: limits.max_decoded_bytes,
        });
    }

    let mut decoder = ZlibDecoder::new(state.compressed.as_slice());
    let mut filtered = Vec::with_capacity(expected);
    decoder
        .by_ref()
        .take((expected as u64) + 1)
        .read_to_end(&mut filtered)
        .map_err(|error| RasterImageError::PngDecompression(error.to_string()))?;
    if filtered.len() != expected {
        return Err(RasterImageError::InvalidPngDecodedLength {
            expected,
            actual: filtered.len(),
        });
    }

    let raw = unfilter(&filtered, source_stride, header.height, bytes_per_pixel)?;
    let output_stride = (header.width as usize)
        .checked_mul(4)
        .ok_or(RasterImageError::DecodedSizeOverflow)?;
    let output_len = output_stride
        .checked_mul(header.height as usize)
        .ok_or(RasterImageError::DecodedSizeOverflow)?;
    if output_len > limits.max_decoded_bytes {
        return Err(RasterImageError::DecodedLimitExceeded {
            actual: output_len,
            limit: limits.max_decoded_bytes,
        });
    }
    let mut pixels = Vec::with_capacity(output_len);
    expand_pixels(&raw, header, &state, &mut pixels)?;

    let alpha_mode = if matches!(header.color_type, 4 | 6) || state.transparency.is_some() {
        AlphaMode::Straight
    } else {
        AlphaMode::Opaque
    };
    let image = DecodedImage {
        width: header.width,
        height: header.height,
        row_stride: output_stride,
        pixel_format: PixelFormat::Rgba8,
        color_space: state.color_space,
        alpha_mode,
        source_orientation: ImageOrientation::TopDown,
        output_orientation: ImageOrientation::TopDown,
        source_bit_depth: u16::from(header.bit_depth),
        source_row_stride: source_stride,
        pixels,
    };
    image.validate()?;
    Ok(image)
}

fn unfilter(
    filtered: &[u8],
    row_stride: usize,
    height: u32,
    bytes_per_pixel: usize,
) -> Result<Vec<u8>, RasterImageError> {
    let mut raw = vec![0_u8; row_stride * height as usize];
    let encoded_stride = row_stride + 1;
    for row in 0..height as usize {
        let filter = filtered[row * encoded_stride];
        let source = &filtered[row * encoded_stride + 1..(row + 1) * encoded_stride];
        let row_start = row * row_stride;
        for column in 0..row_stride {
            let left = if column >= bytes_per_pixel {
                raw[row_start + column - bytes_per_pixel]
            } else {
                0
            };
            let above = if row > 0 {
                raw[row_start + column - row_stride]
            } else {
                0
            };
            let upper_left = if row > 0 && column >= bytes_per_pixel {
                raw[row_start + column - row_stride - bytes_per_pixel]
            } else {
                0
            };
            let reconstructed = match filter {
                0 => source[column],
                1 => source[column].wrapping_add(left),
                2 => source[column].wrapping_add(above),
                3 => source[column].wrapping_add(((u16::from(left) + u16::from(above)) / 2) as u8),
                4 => source[column].wrapping_add(paeth(left, above, upper_left)),
                other => return Err(RasterImageError::UnsupportedPngScanlineFilter(other)),
            };
            raw[row_start + column] = reconstructed;
        }
    }
    Ok(raw)
}

fn expand_pixels(
    raw: &[u8],
    header: Header,
    state: &PngState,
    output: &mut Vec<u8>,
) -> Result<(), RasterImageError> {
    if header.bit_depth < 8 {
        return expand_packed_pixels(raw, header, state, output);
    }

    for sample in raw.chunks_exact(header.channels) {
        let rgba = match header.color_type {
            0 => {
                let alpha = match state.transparency.as_deref() {
                    Some([high, low])
                        if u16::from_be_bytes([*high, *low]) == u16::from(sample[0]) =>
                    {
                        0
                    }
                    _ => 255,
                };
                [sample[0], sample[0], sample[0], alpha]
            }
            2 => {
                let alpha = match state.transparency.as_deref() {
                    Some(value)
                        if value.len() == 6
                            && u16::from_be_bytes([value[0], value[1]]) == u16::from(sample[0])
                            && u16::from_be_bytes([value[2], value[3]]) == u16::from(sample[1])
                            && u16::from_be_bytes([value[4], value[5]]) == u16::from(sample[2]) =>
                    {
                        0
                    }
                    _ => 255,
                };
                [sample[0], sample[1], sample[2], alpha]
            }
            3 => {
                let index = usize::from(sample[0]);
                let palette = state
                    .palette
                    .as_ref()
                    .ok_or(RasterImageError::MissingPngPalette)?;
                let color = palette
                    .get(index)
                    .ok_or(RasterImageError::InvalidPngPaletteIndex {
                        index,
                        entries: palette.len(),
                    })?;
                let alpha = state
                    .transparency
                    .as_ref()
                    .and_then(|values| values.get(index))
                    .copied()
                    .unwrap_or(255);
                [color[0], color[1], color[2], alpha]
            }
            4 => [sample[0], sample[0], sample[0], sample[1]],
            6 => [sample[0], sample[1], sample[2], sample[3]],
            other => return Err(RasterImageError::UnsupportedPngColorType(other)),
        };
        output.extend_from_slice(&rgba);
    }
    Ok(())
}

fn expand_packed_pixels(
    raw: &[u8],
    header: Header,
    state: &PngState,
    output: &mut Vec<u8>,
) -> Result<(), RasterImageError> {
    let samples_per_byte = 8 / usize::from(header.bit_depth);
    let sample_mask = (1_u8 << header.bit_depth) - 1;
    let max_sample = u16::from(sample_mask);
    let source_stride = raw.len() / header.height as usize;

    for row in raw.chunks_exact(source_stride) {
        for column in 0..header.width as usize {
            let byte = row[column / samples_per_byte];
            let shift = 8 - usize::from(header.bit_depth) * (column % samples_per_byte + 1);
            let sample = (byte >> shift) & sample_mask;
            let rgba = match header.color_type {
                0 => {
                    let value = ((u16::from(sample) * 255) / max_sample) as u8;
                    let alpha = match state.transparency.as_deref() {
                        Some([high, low])
                            if u16::from_be_bytes([*high, *low]) == u16::from(sample) =>
                        {
                            0
                        }
                        _ => 255,
                    };
                    [value, value, value, alpha]
                }
                3 => {
                    let index = usize::from(sample);
                    let palette = state
                        .palette
                        .as_ref()
                        .ok_or(RasterImageError::MissingPngPalette)?;
                    let color =
                        palette
                            .get(index)
                            .ok_or(RasterImageError::InvalidPngPaletteIndex {
                                index,
                                entries: palette.len(),
                            })?;
                    let alpha = state
                        .transparency
                        .as_ref()
                        .and_then(|values| values.get(index))
                        .copied()
                        .unwrap_or(255);
                    [color[0], color[1], color[2], alpha]
                }
                other => return Err(RasterImageError::UnsupportedPngColorType(other)),
            };
            output.extend_from_slice(&rgba);
        }
    }
    Ok(())
}

fn valid_transparency_length(header: Header, state: &PngState, length: usize) -> bool {
    match header.color_type {
        0 => length == 2,
        2 => length == 6,
        3 => state
            .palette
            .as_ref()
            .is_some_and(|palette| length <= palette.len()),
        _ => false,
    }
}

fn require_header(state: &PngState) -> Result<Header, RasterImageError> {
    state.header.ok_or(RasterImageError::InvalidPngHeaderOrder)
}

fn validate_crc(kind: &[u8], data: &[u8], expected: &[u8]) -> Result<(), RasterImageError> {
    let mut hasher = Hasher::new();
    hasher.update(kind);
    hasher.update(data);
    let actual = hasher.finalize();
    let expected = u32::from_be_bytes(expected.try_into().unwrap());
    if actual != expected {
        return Err(RasterImageError::InvalidPngCrc {
            chunk: String::from_utf8_lossy(kind).into_owned(),
        });
    }
    Ok(())
}

fn read_u32(source: &[u8], offset: usize) -> Result<u32, RasterImageError> {
    let end = offset
        .checked_add(4)
        .ok_or(RasterImageError::DecodedSizeOverflow)?;
    let bytes = source
        .get(offset..end)
        .ok_or(RasterImageError::TruncatedPngChunk { offset })?;
    Ok(u32::from_be_bytes(bytes.try_into().unwrap()))
}

fn paeth(left: u8, above: u8, upper_left: u8) -> u8 {
    let left = i32::from(left);
    let above = i32::from(above);
    let upper_left = i32::from(upper_left);
    let estimate = left + above - upper_left;
    let left_distance = (estimate - left).abs();
    let above_distance = (estimate - above).abs();
    let corner_distance = (estimate - upper_left).abs();
    if left_distance <= above_distance && left_distance <= corner_distance {
        left as u8
    } else if above_distance <= corner_distance {
        above as u8
    } else {
        upper_left as u8
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::{Read, Write},
        path::PathBuf,
    };

    use crc32fast::Hasher;
    use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};

    use super::{decode_png, inspect_png, paeth, PngIccMetadata, PngInspection};
    use crate::{AlphaMode, ColorSpace, DecodeLimits, RasterImageError};

    struct Fixture<'a> {
        width: u32,
        height: u32,
        color_type: u8,
        channels: usize,
        raw: &'a [u8],
        filters: &'a [u8],
        palette: Option<&'a [u8]>,
        transparency: Option<&'a [u8]>,
        srgb: bool,
    }

    fn png_suite_fixture(name: &str) -> Vec<u8> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite")
            .join(name);
        fs::read(path).unwrap()
    }

    #[test]
    fn decodes_selected_png_suite_eight_bit_profiles() {
        for (name, expected_fingerprint) in [
            ("basn0g08.png", "11ed8979ce4d7b4d"),
            ("basn2c08.png", "6d0a594462868f25"),
            ("basn3p08.png", "3733a8885d80db25"),
            ("basn4a08.png", "4b4e70fc7720494d"),
            ("basn6a08.png", "f9ed41b6375b125d"),
            ("tp1n3p08.png", "812010caba91968b"),
        ] {
            let image = decode_png(&png_suite_fixture(name), DecodeLimits::default()).unwrap();
            assert_eq!((image.width, image.height), (32, 32), "{name}");
            assert_eq!(image.row_stride, 32 * 4, "{name}");
            assert_eq!(image.pixels.len(), 32 * 32 * 4, "{name}");
            assert_eq!(image.pixel_fingerprint(), expected_fingerprint, "{name}");
        }
    }

    #[test]
    fn selected_png_suite_filter_encodings_match_documented_profiles() {
        let cases = [
            ("f00n2c08.png", 0, "bc7cde3bbb7a5582"),
            ("f01n2c08.png", 1, "698cb503a52cf3ba"),
            ("f02n2c08.png", 2, "85049d42112cb08c"),
            ("f03n2c08.png", 3, "0cc6d97779210b6c"),
            ("f04n2c08.png", 4, "82f3b8a0ed489346"),
        ];
        for (name, expected_filter, expected_fingerprint) in cases {
            let source = png_suite_fixture(name);
            let filtered = inflate_idat(&source);
            assert_eq!(filtered.len(), 32 * (1 + 32 * 3), "{name}");
            for row in filtered.chunks_exact(1 + 32 * 3) {
                assert_eq!(row[0], expected_filter, "{name}");
            }

            let image = decode_png(&source, DecodeLimits::default()).unwrap();
            assert_eq!((image.width, image.height), (32, 32), "{name}");
            assert_eq!(image.pixel_fingerprint(), expected_fingerprint, "{name}");
        }
    }

    fn inflate_idat(source: &[u8]) -> Vec<u8> {
        let mut offset = 8;
        let mut compressed = Vec::new();
        while offset < source.len() {
            let length =
                u32::from_be_bytes(source[offset..offset + 4].try_into().unwrap()) as usize;
            let chunk_type = &source[offset + 4..offset + 8];
            let data_start = offset + 8;
            let data_end = data_start + length;
            if chunk_type == b"IDAT" {
                compressed.extend_from_slice(&source[data_start..data_end]);
            }
            offset = data_end + 4;
        }

        let mut filtered = Vec::new();
        ZlibDecoder::new(compressed.as_slice())
            .read_to_end(&mut filtered)
            .unwrap();
        filtered
    }

    #[test]
    fn selected_png_suite_packed_profiles_decode_and_adam7_stops_explicitly() {
        assert_eq!(
            decode_png(&png_suite_fixture("basi6a08.png"), DecodeLimits::default()),
            Err(RasterImageError::UnsupportedPngInterlace(1))
        );
        assert_eq!(
            decode_png(&png_suite_fixture("s07i3p02.png"), DecodeLimits::default()),
            Err(RasterImageError::UnsupportedPngInterlace(1))
        );
        for (name, dimensions, expected_fingerprint) in [
            ("s01n3p01.png", (1, 1), "4a3d077f9b55736b"),
            ("s33n3p04.png", (33, 33), "ced74e9e9e28435b"),
        ] {
            let image = decode_png(&png_suite_fixture(name), DecodeLimits::default()).unwrap();
            assert_eq!((image.width, image.height), dimensions, "{name}");
            assert_eq!(image.pixel_fingerprint(), expected_fingerprint, "{name}");
        }
    }

    #[test]
    fn selected_png_suite_corrupt_inputs_fail_at_png_boundary() {
        for name in ["x00n0g01.png", "xcrn0g04.png", "xlfn0g04.png"] {
            assert!(
                decode_png(&png_suite_fixture(name), DecodeLimits::default()).is_err(),
                "{name} unexpectedly decoded"
            );
        }
    }

    #[test]
    fn decodes_all_supported_eight_bit_color_types() {
        let cases = [
            (
                0,
                1,
                vec![17],
                None,
                None,
                vec![17, 17, 17, 255],
                AlphaMode::Opaque,
            ),
            (
                2,
                3,
                vec![1, 2, 3],
                None,
                None,
                vec![1, 2, 3, 255],
                AlphaMode::Opaque,
            ),
            (
                3,
                1,
                vec![1],
                Some(vec![9, 8, 7, 6, 5, 4]),
                Some(vec![255, 23]),
                vec![6, 5, 4, 23],
                AlphaMode::Straight,
            ),
            (
                4,
                2,
                vec![31, 47],
                None,
                None,
                vec![31, 31, 31, 47],
                AlphaMode::Straight,
            ),
            (
                6,
                4,
                vec![11, 12, 13, 14],
                None,
                None,
                vec![11, 12, 13, 14],
                AlphaMode::Straight,
            ),
        ];

        for (color_type, channels, raw, palette, transparency, expected, alpha_mode) in cases {
            let bytes = fixture_png(Fixture {
                width: 1,
                height: 1,
                color_type,
                channels,
                raw: &raw,
                filters: &[0],
                palette: palette.as_deref(),
                transparency: transparency.as_deref(),
                srgb: false,
            });
            let image = decode_png(&bytes, DecodeLimits::default()).unwrap();
            assert_eq!(image.pixels, expected);
            assert_eq!(image.alpha_mode, alpha_mode);
        }
    }

    #[test]
    fn expands_packed_grayscale_samples_at_one_two_and_four_bits() {
        for (bit_depth, raw, expected) in [
            (
                1,
                vec![0b0101_0000],
                vec![
                    0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255,
                ],
            ),
            (
                2,
                vec![0b00_01_10_11],
                vec![
                    0, 0, 0, 255, 85, 85, 85, 255, 170, 170, 170, 255, 255, 255, 255, 255,
                ],
            ),
            (
                4,
                vec![0x0f, 0x78],
                vec![
                    0, 0, 0, 255, 255, 255, 255, 255, 119, 119, 119, 255, 136, 136, 136, 255,
                ],
            ),
        ] {
            let bytes = packed_fixture_png(4, bit_depth, 0, &raw, None);
            let image = decode_png(&bytes, DecodeLimits::default()).unwrap();
            assert_eq!(image.pixels, expected, "{bit_depth}-bit");
            assert_eq!(image.source_bit_depth, u16::from(bit_depth));
        }
    }

    #[test]
    fn reconstructs_all_five_scanline_filters_to_identical_pixels() {
        let raw = [1, 2, 3, 20, 30, 40, 7, 8, 9, 50, 60, 70];
        let expected = [1, 2, 3, 255, 20, 30, 40, 255, 7, 8, 9, 255, 50, 60, 70, 255];
        let mut fingerprints = Vec::new();
        for filter in 0..=4 {
            let bytes = fixture_png(Fixture {
                width: 2,
                height: 2,
                color_type: 2,
                channels: 3,
                raw: &raw,
                filters: &[filter, filter],
                palette: None,
                transparency: None,
                srgb: false,
            });
            let image = decode_png(&bytes, DecodeLimits::default()).unwrap();
            assert_eq!(image.pixels, expected);
            fingerprints.push(image.pixel_fingerprint());
        }
        assert!(fingerprints.windows(2).all(|pair| pair[0] == pair[1]));
    }

    #[test]
    fn observes_srgb_without_converting_pixels() {
        let raw = [10, 20, 30];
        let bytes = fixture_png(Fixture {
            width: 1,
            height: 1,
            color_type: 2,
            channels: 3,
            raw: &raw,
            filters: &[0],
            palette: None,
            transparency: None,
            srgb: true,
        });
        let image = decode_png(&bytes, DecodeLimits::default()).unwrap();
        assert_eq!(image.color_space, ColorSpace::Srgb);
        assert_eq!(image.pixels, [10, 20, 30, 255]);
    }

    #[test]
    fn observes_gamma_and_icc_without_converting_pixels() {
        let raw = [10, 20, 30];
        let base = fixture_png(Fixture {
            width: 1,
            height: 1,
            color_type: 2,
            channels: 3,
            raw: &raw,
            filters: &[0],
            palette: None,
            transparency: None,
            srgb: false,
        });
        let mut with_metadata = base.clone();
        insert_chunk_before(
            &mut with_metadata,
            b"IDAT",
            b"gAMA",
            &45_455_u32.to_be_bytes(),
        );

        let mut profile_encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        profile_encoder.write_all(b"bounded ICC evidence").unwrap();
        let compressed_profile = profile_encoder.finish().unwrap();
        let mut iccp = b"Tokimu test profile\0\0".to_vec();
        iccp.extend_from_slice(&compressed_profile);
        insert_chunk_before(&mut with_metadata, b"IDAT", b"iCCP", &iccp);

        assert_eq!(
            inspect_png(&with_metadata, DecodeLimits::default()).unwrap(),
            PngInspection {
                width: 1,
                height: 1,
                bit_depth: 8,
                color_type: 2,
                srgb_rendering_intent: None,
                gamma_times_100000: Some(45_455),
                icc_profile: Some(PngIccMetadata {
                    profile_name: "Tokimu test profile".to_owned(),
                    compressed_bytes: compressed_profile.len(),
                }),
            }
        );

        let base_image = decode_png(&base, DecodeLimits::default()).unwrap();
        let metadata_image = decode_png(&with_metadata, DecodeLimits::default()).unwrap();
        assert_eq!(metadata_image.pixels, base_image.pixels);
        assert_eq!(metadata_image.color_space, ColorSpace::Unspecified);
    }

    #[test]
    fn rejects_malformed_or_conflicting_png_color_metadata() {
        let raw = [10, 20, 30];
        let base = fixture_png(Fixture {
            width: 1,
            height: 1,
            color_type: 2,
            channels: 3,
            raw: &raw,
            filters: &[0],
            palette: None,
            transparency: None,
            srgb: false,
        });

        let mut zero_gamma = base.clone();
        insert_chunk_before(&mut zero_gamma, b"IDAT", b"gAMA", &[0, 0, 0, 0]);
        assert_eq!(
            inspect_png(&zero_gamma, DecodeLimits::default()),
            Err(RasterImageError::InvalidPngMetadata(
                "gAMA value must be non-zero"
            ))
        );

        let mut invalid_iccp = base.clone();
        insert_chunk_before(
            &mut invalid_iccp,
            b"IDAT",
            b"iCCP",
            b"Tokimu profile\0\x01compressed",
        );
        assert_eq!(
            inspect_png(&invalid_iccp, DecodeLimits::default()),
            Err(RasterImageError::InvalidPngMetadata(
                "iCCP compression method must be zero"
            ))
        );

        let mut conflicting = fixture_png(Fixture {
            width: 1,
            height: 1,
            color_type: 2,
            channels: 3,
            raw: &raw,
            filters: &[0],
            palette: None,
            transparency: None,
            srgb: true,
        });
        insert_chunk_before(
            &mut conflicting,
            b"IDAT",
            b"iCCP",
            b"Tokimu profile\0\0compressed",
        );
        assert_eq!(
            inspect_png(&conflicting, DecodeLimits::default()),
            Err(RasterImageError::InvalidPngMetadata(
                "sRGB and iCCP cannot both occur"
            ))
        );
    }

    #[test]
    fn rejects_crc_corruption() {
        let raw = [10, 20, 30];
        let mut bytes = fixture_png(Fixture {
            width: 1,
            height: 1,
            color_type: 2,
            channels: 3,
            raw: &raw,
            filters: &[0],
            palette: None,
            transparency: None,
            srgb: false,
        });
        let idat = bytes
            .windows(4)
            .position(|window| window == b"IDAT")
            .unwrap();
        bytes[idat + 4] ^= 0x01;

        assert!(matches!(
            decode_png(&bytes, DecodeLimits::default()),
            Err(RasterImageError::InvalidPngCrc { .. })
        ));
    }

    #[test]
    fn rejects_truncated_chunks_and_unsupported_bit_depth() {
        let raw = [1, 2, 3];
        let mut bytes = fixture_png(Fixture {
            width: 1,
            height: 1,
            color_type: 2,
            channels: 3,
            raw: &raw,
            filters: &[0],
            palette: None,
            transparency: None,
            srgb: false,
        });
        bytes.truncate(bytes.len() - 2);
        assert!(matches!(
            decode_png(&bytes, DecodeLimits::default()),
            Err(RasterImageError::TruncatedPngChunk { .. })
        ));

        let mut unsupported = fixture_png(Fixture {
            width: 1,
            height: 1,
            color_type: 2,
            channels: 3,
            raw: &raw,
            filters: &[0],
            palette: None,
            transparency: None,
            srgb: false,
        });
        unsupported[24] = 16;
        rewrite_chunk_crc(&mut unsupported, 8);
        assert_eq!(
            decode_png(&unsupported, DecodeLimits::default()),
            Err(RasterImageError::UnsupportedPngBitDepth {
                bit_depth: 16,
                color_type: 2,
            })
        );
    }

    #[test]
    fn rejects_output_beyond_decode_policy_before_expansion() {
        let raw = [1, 2, 3];
        let bytes = fixture_png(Fixture {
            width: 1,
            height: 1,
            color_type: 2,
            channels: 3,
            raw: &raw,
            filters: &[0],
            palette: None,
            transparency: None,
            srgb: false,
        });
        let limits = DecodeLimits {
            max_decoded_bytes: 3,
            ..DecodeLimits::default()
        };
        assert_eq!(
            decode_png(&bytes, limits),
            Err(RasterImageError::DecodedLimitExceeded {
                actual: 4,
                limit: 3,
            })
        );
    }

    #[test]
    fn ignores_unknown_ancillary_metadata_without_changing_pixels() {
        let raw = [10, 20, 30];
        let base = fixture_png(Fixture {
            width: 1,
            height: 1,
            color_type: 2,
            channels: 3,
            raw: &raw,
            filters: &[0],
            palette: None,
            transparency: None,
            srgb: false,
        });
        let mut with_metadata = base.clone();
        insert_chunk_before(&mut with_metadata, b"IDAT", b"ruSt", b"corpus evidence");

        let base_image = decode_png(&base, DecodeLimits::default()).unwrap();
        let metadata_image = decode_png(&with_metadata, DecodeLimits::default()).unwrap();
        assert_eq!(metadata_image.pixels, base_image.pixels);
        assert_eq!(
            metadata_image.pixel_fingerprint(),
            base_image.pixel_fingerprint()
        );
    }

    #[test]
    fn rejects_unknown_critical_chunks_without_treating_them_as_metadata() {
        let raw = [10, 20, 30];
        let mut bytes = fixture_png(Fixture {
            width: 1,
            height: 1,
            color_type: 2,
            channels: 3,
            raw: &raw,
            filters: &[0],
            palette: None,
            transparency: None,
            srgb: false,
        });
        insert_chunk_before(&mut bytes, b"IDAT", b"RuSt", b"must not be ignored");

        assert_eq!(
            decode_png(&bytes, DecodeLimits::default()),
            Err(RasterImageError::UnsupportedCriticalPngChunk(
                "RuSt".to_owned()
            ))
        );
    }

    #[test]
    fn rejects_duplicate_palette_and_transparency_chunks() {
        let raw = [0_u8];
        let mut duplicate_palette = fixture_png(Fixture {
            width: 1,
            height: 1,
            color_type: 3,
            channels: 1,
            raw: &raw,
            filters: &[0],
            palette: Some(&[10, 20, 30]),
            transparency: None,
            srgb: false,
        });
        insert_chunk_before(&mut duplicate_palette, b"IDAT", b"PLTE", &[10, 20, 30]);
        assert_eq!(
            decode_png(&duplicate_palette, DecodeLimits::default()),
            Err(RasterImageError::InvalidPngPaletteLength(3))
        );

        let mut duplicate_transparency = fixture_png(Fixture {
            width: 1,
            height: 1,
            color_type: 3,
            channels: 1,
            raw: &raw,
            filters: &[0],
            palette: Some(&[10, 20, 30]),
            transparency: Some(&[255]),
            srgb: false,
        });
        insert_chunk_before(&mut duplicate_transparency, b"IDAT", b"tRNS", &[255]);
        assert_eq!(
            decode_png(&duplicate_transparency, DecodeLimits::default()),
            Err(RasterImageError::InvalidPngTransparency { color_type: 3 })
        );
    }

    #[test]
    fn rejects_nonconsecutive_data_and_malformed_palette() {
        let raw = [10, 20, 30];
        let mut nonconsecutive = fixture_png(Fixture {
            width: 1,
            height: 1,
            color_type: 2,
            channels: 3,
            raw: &raw,
            filters: &[0],
            palette: None,
            transparency: None,
            srgb: false,
        });
        insert_chunk_before(&mut nonconsecutive, b"IEND", b"ruSt", b"boundary");
        insert_chunk_before(&mut nonconsecutive, b"IEND", b"IDAT", &[]);
        assert_eq!(
            decode_png(&nonconsecutive, DecodeLimits::default()),
            Err(RasterImageError::NonConsecutivePngData)
        );

        let indexed = [0];
        let missing_palette = fixture_png(Fixture {
            width: 1,
            height: 1,
            color_type: 3,
            channels: 1,
            raw: &indexed,
            filters: &[0],
            palette: None,
            transparency: None,
            srgb: false,
        });
        assert_eq!(
            decode_png(&missing_palette, DecodeLimits::default()),
            Err(RasterImageError::MissingPngPalette)
        );

        let mut malformed_palette = fixture_png(Fixture {
            width: 1,
            height: 1,
            color_type: 3,
            channels: 1,
            raw: &indexed,
            filters: &[0],
            palette: Some(&[1, 2, 3]),
            transparency: None,
            srgb: false,
        });
        let palette_kind = malformed_palette
            .windows(4)
            .position(|window| window == b"PLTE")
            .unwrap();
        let chunk_start = palette_kind - 4;
        malformed_palette.splice(chunk_start..chunk_start + 15, {
            let mut replacement = Vec::new();
            append_chunk(&mut replacement, b"PLTE", &[1, 2]);
            replacement
        });
        assert_eq!(
            decode_png(&malformed_palette, DecodeLimits::default()),
            Err(RasterImageError::InvalidPngPaletteLength(2))
        );

        let rgba = [1, 2, 3, 4];
        let invalid_transparency = fixture_png(Fixture {
            width: 1,
            height: 1,
            color_type: 6,
            channels: 4,
            raw: &rgba,
            filters: &[0],
            palette: None,
            transparency: Some(&[0, 0]),
            srgb: false,
        });
        assert_eq!(
            decode_png(&invalid_transparency, DecodeLimits::default()),
            Err(RasterImageError::InvalidPngTransparency { color_type: 6 })
        );
    }

    #[test]
    fn decodes_consecutive_idat_partitions_identically() {
        let raw = [
            10, 20, 30, 40, 50, 60, // first row
            70, 80, 90, 100, 110, 120, // second row
        ];
        let source = fixture_png(Fixture {
            width: 2,
            height: 2,
            color_type: 2,
            channels: 3,
            raw: &raw,
            filters: &[0, 4],
            palette: None,
            transparency: None,
            srgb: false,
        });
        let mut partitioned = source.clone();
        split_first_idat(&mut partitioned);

        assert_eq!(
            decode_png(&partitioned, DecodeLimits::default()).unwrap(),
            decode_png(&source, DecodeLimits::default()).unwrap(),
            "legal consecutive IDAT chunks must decode exactly like one unsplit stream"
        );
    }

    fn fixture_png(fixture: Fixture<'_>) -> Vec<u8> {
        assert_eq!(fixture.filters.len(), fixture.height as usize);
        let stride = fixture.width as usize * fixture.channels;
        assert_eq!(fixture.raw.len(), stride * fixture.height as usize);

        let mut filtered = Vec::with_capacity((stride + 1) * fixture.height as usize);
        for row in 0..fixture.height as usize {
            let filter = fixture.filters[row];
            filtered.push(filter);
            let row_start = row * stride;
            for column in 0..stride {
                let value = fixture.raw[row_start + column];
                let left = if column >= fixture.channels {
                    fixture.raw[row_start + column - fixture.channels]
                } else {
                    0
                };
                let above = if row > 0 {
                    fixture.raw[row_start + column - stride]
                } else {
                    0
                };
                let upper_left = if row > 0 && column >= fixture.channels {
                    fixture.raw[row_start + column - stride - fixture.channels]
                } else {
                    0
                };
                let predictor = match filter {
                    0 => 0,
                    1 => left,
                    2 => above,
                    3 => ((u16::from(left) + u16::from(above)) / 2) as u8,
                    4 => paeth(left, above, upper_left),
                    _ => unreachable!(),
                };
                filtered.push(value.wrapping_sub(predictor));
            }
        }

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&filtered).unwrap();
        let compressed = encoder.finish().unwrap();

        let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut header = Vec::new();
        header.extend_from_slice(&fixture.width.to_be_bytes());
        header.extend_from_slice(&fixture.height.to_be_bytes());
        header.extend_from_slice(&[8, fixture.color_type, 0, 0, 0]);
        append_chunk(&mut bytes, b"IHDR", &header);
        if fixture.srgb {
            append_chunk(&mut bytes, b"sRGB", &[0]);
        }
        if let Some(palette) = fixture.palette {
            append_chunk(&mut bytes, b"PLTE", palette);
        }
        if let Some(transparency) = fixture.transparency {
            append_chunk(&mut bytes, b"tRNS", transparency);
        }
        append_chunk(&mut bytes, b"IDAT", &compressed);
        append_chunk(&mut bytes, b"IEND", &[]);
        bytes
    }

    fn packed_fixture_png(
        width: u32,
        bit_depth: u8,
        color_type: u8,
        raw: &[u8],
        palette: Option<&[u8]>,
    ) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&[0]).unwrap();
        encoder.write_all(raw).unwrap();
        let compressed = encoder.finish().unwrap();

        let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut header = Vec::new();
        header.extend_from_slice(&width.to_be_bytes());
        header.extend_from_slice(&1_u32.to_be_bytes());
        header.extend_from_slice(&[bit_depth, color_type, 0, 0, 0]);
        append_chunk(&mut bytes, b"IHDR", &header);
        if let Some(palette) = palette {
            append_chunk(&mut bytes, b"PLTE", palette);
        }
        append_chunk(&mut bytes, b"IDAT", &compressed);
        append_chunk(&mut bytes, b"IEND", &[]);
        bytes
    }

    fn append_chunk(output: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
        output.extend_from_slice(&(data.len() as u32).to_be_bytes());
        output.extend_from_slice(kind);
        output.extend_from_slice(data);
        let mut hasher = Hasher::new();
        hasher.update(kind);
        hasher.update(data);
        output.extend_from_slice(&hasher.finalize().to_be_bytes());
    }

    fn insert_chunk_before(
        bytes: &mut Vec<u8>,
        target_kind: &[u8; 4],
        kind: &[u8; 4],
        data: &[u8],
    ) {
        let target = bytes
            .windows(4)
            .position(|window| window == target_kind)
            .expect("target chunk should exist")
            - 4;
        let mut chunk = Vec::new();
        append_chunk(&mut chunk, kind, data);
        bytes.splice(target..target, chunk);
    }

    fn split_first_idat(bytes: &mut Vec<u8>) {
        let kind_start = bytes
            .windows(4)
            .position(|window| window == b"IDAT")
            .expect("fixture should contain IDAT");
        let chunk_start = kind_start - 4;
        let length =
            u32::from_be_bytes(bytes[chunk_start..kind_start].try_into().unwrap()) as usize;
        assert!(length > 1, "fixture IDAT should be splittable");

        let data_start = kind_start + 4;
        let data_end = data_start + length;
        let split_at = length / 2;
        let mut replacement = Vec::with_capacity(length + 24);
        append_chunk(
            &mut replacement,
            b"IDAT",
            &bytes[data_start..data_start + split_at],
        );
        append_chunk(
            &mut replacement,
            b"IDAT",
            &bytes[data_start + split_at..data_end],
        );
        bytes.splice(chunk_start..data_end + 4, replacement);
    }

    fn rewrite_chunk_crc(bytes: &mut [u8], chunk_start: usize) {
        let length =
            u32::from_be_bytes(bytes[chunk_start..chunk_start + 4].try_into().unwrap()) as usize;
        let kind_start = chunk_start + 4;
        let data_end = kind_start + 4 + length;
        let mut hasher = Hasher::new();
        hasher.update(&bytes[kind_start..data_end]);
        bytes[data_end..data_end + 4].copy_from_slice(&hasher.finalize().to_be_bytes());
    }
}
