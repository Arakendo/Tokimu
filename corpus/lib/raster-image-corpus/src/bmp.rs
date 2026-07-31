use crate::{
    AlphaMode, ColorSpace, DecodeLimits, DecodedImage, ImageOrientation, PixelFormat,
    RasterImageError,
};

const BMP_FILE_HEADER_SIZE: usize = 14;
const BITMAP_INFO_HEADER_SIZE: usize = 40;
const BI_RGB: u32 = 0;

pub fn decode_bmp(bytes: &[u8], limits: DecodeLimits) -> Result<DecodedImage, RasterImageError> {
    if bytes.len() > limits.max_source_bytes {
        return Err(RasterImageError::SourceLimitExceeded {
            actual: bytes.len(),
            limit: limits.max_source_bytes,
        });
    }
    require_len(bytes, 54, "BMP and BITMAPINFO headers")?;
    if &bytes[0..2] != b"BM" {
        return Err(RasterImageError::InvalidBmpSignature);
    }

    let declared_file_size = read_u32(bytes, 2)? as usize;
    if declared_file_size < 54 || declared_file_size > bytes.len() {
        return Err(RasterImageError::InvalidDeclaredFileSize {
            declared: declared_file_size,
            actual: bytes.len(),
        });
    }

    let pixel_offset = read_u32(bytes, 10)? as usize;
    let dib_size = read_u32(bytes, BMP_FILE_HEADER_SIZE)?;
    if dib_size < BITMAP_INFO_HEADER_SIZE as u32 {
        return Err(RasterImageError::UnsupportedBmpHeader(dib_size));
    }
    let dib_end = BMP_FILE_HEADER_SIZE
        .checked_add(dib_size as usize)
        .ok_or(RasterImageError::DecodedSizeOverflow)?;
    require_len(bytes, dib_end, "declared BMP DIB header")?;
    if pixel_offset < dib_end {
        return Err(RasterImageError::InvalidBmpPixelOffset {
            offset: pixel_offset,
            minimum: dib_end,
        });
    }

    let source_width = read_i32(bytes, 18)?;
    if source_width <= 0 {
        return Err(RasterImageError::InvalidBmpWidth(source_width));
    }
    let source_height = read_i32(bytes, 22)?;
    if source_height == 0 || source_height == i32::MIN {
        return Err(RasterImageError::InvalidBmpHeight(source_height));
    }

    let planes = read_u16(bytes, 26)?;
    if planes != 1 {
        return Err(RasterImageError::InvalidBmpPlanes(planes));
    }
    let bit_depth = read_u16(bytes, 28)?;
    if bit_depth != 8 && bit_depth != 24 && bit_depth != 32 {
        return Err(RasterImageError::UnsupportedBmpBitDepth(bit_depth));
    }
    let compression = read_u32(bytes, 30)?;
    if compression != BI_RGB {
        return Err(RasterImageError::UnsupportedBmpCompression(compression));
    }

    let width = source_width as u32;
    let height = source_height.unsigned_abs();
    if width > limits.max_width || height > limits.max_height {
        return Err(RasterImageError::DimensionLimitExceeded {
            width,
            height,
            max_width: limits.max_width,
            max_height: limits.max_height,
        });
    }

    let palette = if bit_depth == 8 {
        Some(parse_palette(
            bytes,
            dib_end,
            pixel_offset,
            read_u32(bytes, 46)?,
        )?)
    } else {
        None
    };
    let bytes_per_pixel = usize::from(bit_depth / 8);
    let source_row_stride = (width as usize)
        .checked_mul(bytes_per_pixel)
        .and_then(|row| row.checked_add(3))
        .map(|row| row & !3)
        .ok_or(RasterImageError::DecodedSizeOverflow)?;
    let source_pixel_bytes = source_row_stride
        .checked_mul(height as usize)
        .ok_or(RasterImageError::DecodedSizeOverflow)?;
    let source_end = pixel_offset
        .checked_add(source_pixel_bytes)
        .ok_or(RasterImageError::DecodedSizeOverflow)?;
    let available_len = declared_file_size.min(bytes.len());
    if source_end > available_len {
        return Err(RasterImageError::TruncatedBmpPixels {
            expected_end: source_end,
            actual: available_len,
        });
    }

    let row_stride = (width as usize)
        .checked_mul(4)
        .ok_or(RasterImageError::DecodedSizeOverflow)?;
    let decoded_len = row_stride
        .checked_mul(height as usize)
        .ok_or(RasterImageError::DecodedSizeOverflow)?;
    if decoded_len > limits.max_decoded_bytes {
        return Err(RasterImageError::DecodedLimitExceeded {
            actual: decoded_len,
            limit: limits.max_decoded_bytes,
        });
    }

    let source_orientation = if source_height < 0 {
        ImageOrientation::TopDown
    } else {
        ImageOrientation::BottomUp
    };
    let mut pixels = vec![0_u8; decoded_len];
    for output_y in 0..height as usize {
        let source_y = match source_orientation {
            ImageOrientation::TopDown => output_y,
            ImageOrientation::BottomUp => height as usize - 1 - output_y,
        };
        let source_row = pixel_offset + source_y * source_row_stride;
        let output_row = output_y * row_stride;
        for x in 0..width as usize {
            let source = source_row + x * bytes_per_pixel;
            let output = output_row + x * 4;
            match bit_depth {
                8 => {
                    let index = usize::from(bytes[source]);
                    let entries = palette.as_ref().expect("8-bit BMP requires a palette");
                    let [red, green, blue] =
                        *entries
                            .get(index)
                            .ok_or(RasterImageError::InvalidBmpPaletteIndex {
                                index,
                                entries: entries.len(),
                            })?;
                    pixels[output..output + 4].copy_from_slice(&[red, green, blue, 255]);
                }
                24 => {
                    pixels[output] = bytes[source + 2];
                    pixels[output + 1] = bytes[source + 1];
                    pixels[output + 2] = bytes[source];
                    pixels[output + 3] = 255;
                }
                32 => {
                    pixels[output] = bytes[source + 2];
                    pixels[output + 1] = bytes[source + 1];
                    pixels[output + 2] = bytes[source];
                    pixels[output + 3] = bytes[source + 3];
                }
                _ => unreachable!("bit depth was validated before decoding"),
            }
        }
    }

    let image = DecodedImage {
        width,
        height,
        row_stride,
        pixel_format: PixelFormat::Rgba8,
        color_space: ColorSpace::Unspecified,
        alpha_mode: if bit_depth == 32 {
            AlphaMode::Unspecified
        } else {
            AlphaMode::Opaque
        },
        source_orientation,
        output_orientation: ImageOrientation::TopDown,
        source_bit_depth: bit_depth,
        source_row_stride,
        pixels,
    };
    image.validate()?;
    Ok(image)
}

fn parse_palette(
    bytes: &[u8],
    palette_start: usize,
    pixel_offset: usize,
    colors_used: u32,
) -> Result<Vec<[u8; 3]>, RasterImageError> {
    let entries = if colors_used == 0 {
        256
    } else {
        usize::try_from(colors_used).map_err(|_| RasterImageError::DecodedSizeOverflow)?
    };
    let expected = entries
        .checked_mul(4)
        .ok_or(RasterImageError::DecodedSizeOverflow)?;
    let actual = pixel_offset.saturating_sub(palette_start);
    if entries > 256 || expected > actual {
        return Err(RasterImageError::InvalidBmpPalette { expected, actual });
    }

    Ok((0..entries)
        .map(|index| {
            let offset = palette_start + index * 4;
            [bytes[offset + 2], bytes[offset + 1], bytes[offset]]
        })
        .collect())
}

fn require_len(
    bytes: &[u8],
    expected: usize,
    context: &'static str,
) -> Result<(), RasterImageError> {
    if bytes.len() < expected {
        return Err(RasterImageError::Truncated {
            context,
            expected,
            actual: bytes.len(),
        });
    }
    Ok(())
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, RasterImageError> {
    let end = offset + 2;
    require_len(bytes, end, "BMP u16 field")?;
    Ok(u16::from_le_bytes([bytes[offset], bytes[offset + 1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, RasterImageError> {
    let end = offset + 4;
    require_len(bytes, end, "BMP u32 field")?;
    Ok(u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]))
}

fn read_i32(bytes: &[u8], offset: usize) -> Result<i32, RasterImageError> {
    Ok(read_u32(bytes, offset)? as i32)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;

    fn external_fixture(name: &str) -> Vec<u8> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages")
            .join(name);
        fs::read(path).unwrap()
    }

    #[test]
    fn decodes_admitted_external_bmp_fixtures() {
        for (name, dimensions, expected_fingerprint) in [
            ("shira_bird8.bmp", (192, 144), "3a823684e670e6cc"),
            ("vgl_6434_0018a.bmp", (119, 96), "813cdb6b834c4453"),
            ("vgl_6548_0026a.bmp", (128, 95), "d087bf84e7d56e2d"),
        ] {
            let image = decode_bmp(&external_fixture(name), DecodeLimits::default()).expect(name);
            assert_eq!((image.width, image.height), dimensions);
            assert_eq!(image.source_orientation, ImageOrientation::BottomUp);
            assert_eq!(image.output_orientation, ImageOrientation::TopDown);
            assert_eq!(image.source_bit_depth, 24);
            assert_eq!(image.pixel_fingerprint(), expected_fingerprint);
        }
    }

    #[test]
    fn decodes_bottom_up_24_bit_rows_with_padding() {
        let bytes = make_bmp(
            2,
            2,
            24,
            &[
                // Bottom source row: blue, white, then two padding bytes.
                255, 0, 0, 255, 255, 255, 0, 0,
                // Top source row: red, green, then two padding bytes.
                0, 0, 255, 0, 255, 0, 0, 0,
            ],
        );
        let image = decode_bmp(&bytes, DecodeLimits::default()).expect("BMP should decode");

        assert_eq!(image.width, 2);
        assert_eq!(image.height, 2);
        assert_eq!(image.source_orientation, ImageOrientation::BottomUp);
        assert_eq!(image.output_orientation, ImageOrientation::TopDown);
        assert_eq!(image.alpha_mode, AlphaMode::Opaque);
        assert_eq!(
            image.pixels,
            [
                255, 0, 0, 255, 0, 255, 0, 255, // top row
                0, 0, 255, 255, 255, 255, 255, 255, // bottom row
            ]
        );
        assert_eq!(image.pixel_fingerprint(), "8a4318bc590ba10d");
    }

    #[test]
    fn decodes_top_down_32_bit_and_preserves_alpha_bytes() {
        let bytes = make_bmp(2, -1, 32, &[30, 20, 10, 40, 70, 60, 50, 80]);
        let image = decode_bmp(&bytes, DecodeLimits::default()).expect("BMP should decode");

        assert_eq!(image.source_orientation, ImageOrientation::TopDown);
        assert_eq!(image.alpha_mode, AlphaMode::Unspecified);
        assert_eq!(image.pixels, [10, 20, 30, 40, 50, 60, 70, 80]);
    }

    #[test]
    fn decodes_bottom_up_8_bit_palette_rows_with_padding() {
        let bytes = make_indexed_bmp(
            2,
            2,
            &[[0, 0, 0], [255, 0, 0], [0, 255, 0], [0, 0, 255]],
            &[
                // Bottom source row: blue, black, then row padding.
                3, 0, 0, 0, // Top source row: red, green, then row padding.
                1, 2, 0, 0,
            ],
        );
        let image = decode_bmp(&bytes, DecodeLimits::default()).expect("indexed BMP should decode");

        assert_eq!(image.source_bit_depth, 8);
        assert_eq!(image.alpha_mode, AlphaMode::Opaque);
        assert_eq!(image.source_orientation, ImageOrientation::BottomUp);
        assert_eq!(
            image.pixels,
            [
                255, 0, 0, 255, 0, 255, 0, 255, // top row
                0, 0, 255, 255, 0, 0, 0, 255, // bottom row
            ]
        );
    }

    #[test]
    fn rejects_indexed_bmp_palette_indices_outside_the_declared_table() {
        let bytes = make_indexed_bmp(1, 1, &[[0, 0, 0], [255, 0, 0]], &[2, 0, 0, 0]);
        assert_eq!(
            decode_bmp(&bytes, DecodeLimits::default()),
            Err(RasterImageError::InvalidBmpPaletteIndex {
                index: 2,
                entries: 2,
            })
        );
    }

    #[test]
    fn rejects_indexed_bmp_palette_that_ends_before_the_pixel_offset() {
        let mut bytes = make_indexed_bmp(1, 1, &[[0, 0, 0], [255, 0, 0]], &[0, 0, 0, 0]);
        bytes[10..14].copy_from_slice(&60_u32.to_le_bytes());

        assert_eq!(
            decode_bmp(&bytes, DecodeLimits::default()),
            Err(RasterImageError::InvalidBmpPalette {
                expected: 8,
                actual: 6,
            })
        );
    }

    #[test]
    fn rejects_indexed_bmp_palette_with_more_than_256_entries() {
        let mut bytes = make_indexed_bmp(1, 1, &[[0, 0, 0]], &[0, 0, 0, 0]);
        bytes[46..50].copy_from_slice(&257_u32.to_le_bytes());

        assert_eq!(
            decode_bmp(&bytes, DecodeLimits::default()),
            Err(RasterImageError::InvalidBmpPalette {
                expected: 1_028,
                actual: 4,
            })
        );
    }

    #[test]
    fn rejects_unsupported_compression() {
        let mut bytes = make_bmp(1, 1, 24, &[0, 0, 0, 0]);
        bytes[30..34].copy_from_slice(&1_u32.to_le_bytes());
        assert_eq!(
            decode_bmp(&bytes, DecodeLimits::default()),
            Err(RasterImageError::UnsupportedBmpCompression(1))
        );
    }

    #[test]
    fn rejects_truncated_pixel_rows() {
        let mut bytes = make_bmp(2, 2, 24, &[0; 16]);
        bytes.truncate(bytes.len() - 1);
        let file_size = bytes.len() as u32;
        bytes[2..6].copy_from_slice(&file_size.to_le_bytes());
        assert!(matches!(
            decode_bmp(&bytes, DecodeLimits::default()),
            Err(RasterImageError::TruncatedBmpPixels { .. })
        ));
    }

    #[test]
    fn rejects_dimensions_before_decoded_allocation() {
        let bytes = make_bmp(2, 1, 24, &[0; 8]);
        let limits = DecodeLimits {
            max_width: 1,
            ..DecodeLimits::default()
        };
        assert!(matches!(
            decode_bmp(&bytes, limits),
            Err(RasterImageError::DimensionLimitExceeded { .. })
        ));
    }

    #[test]
    fn rejects_pixel_offset_inside_the_dib_header() {
        let mut bytes = make_bmp(1, 1, 24, &[0; 4]);
        bytes[10..14].copy_from_slice(&40_u32.to_le_bytes());
        assert_eq!(
            decode_bmp(&bytes, DecodeLimits::default()),
            Err(RasterImageError::InvalidBmpPixelOffset {
                offset: 40,
                minimum: 54,
            })
        );
    }

    #[test]
    fn rejects_source_before_parsing_when_limit_is_exceeded() {
        let bytes = make_bmp(1, 1, 24, &[0; 4]);
        let limits = DecodeLimits {
            max_source_bytes: bytes.len() - 1,
            ..DecodeLimits::default()
        };
        assert_eq!(
            decode_bmp(&bytes, limits),
            Err(RasterImageError::SourceLimitExceeded {
                actual: bytes.len(),
                limit: bytes.len() - 1,
            })
        );
    }

    fn make_bmp(width: i32, height: i32, bit_depth: u16, pixels: &[u8]) -> Vec<u8> {
        let file_size = 54 + pixels.len();
        let mut bytes = Vec::with_capacity(file_size);
        bytes.extend_from_slice(b"BM");
        bytes.extend_from_slice(&(file_size as u32).to_le_bytes());
        bytes.extend_from_slice(&[0; 4]);
        bytes.extend_from_slice(&54_u32.to_le_bytes());
        bytes.extend_from_slice(&40_u32.to_le_bytes());
        bytes.extend_from_slice(&width.to_le_bytes());
        bytes.extend_from_slice(&height.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&bit_depth.to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        bytes.extend_from_slice(&(pixels.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&[0; 16]);
        bytes.extend_from_slice(pixels);
        bytes
    }

    fn make_indexed_bmp(width: i32, height: i32, palette: &[[u8; 3]], pixels: &[u8]) -> Vec<u8> {
        let pixel_offset = 54 + palette.len() * 4;
        let file_size = pixel_offset + pixels.len();
        let mut bytes = Vec::with_capacity(file_size);
        bytes.extend_from_slice(b"BM");
        bytes.extend_from_slice(&(file_size as u32).to_le_bytes());
        bytes.extend_from_slice(&[0; 4]);
        bytes.extend_from_slice(&(pixel_offset as u32).to_le_bytes());
        bytes.extend_from_slice(&40_u32.to_le_bytes());
        bytes.extend_from_slice(&width.to_le_bytes());
        bytes.extend_from_slice(&height.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&8_u16.to_le_bytes());
        bytes.extend_from_slice(&BI_RGB.to_le_bytes());
        bytes.extend_from_slice(&(pixels.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&[0; 8]);
        bytes.extend_from_slice(&(palette.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        for [red, green, blue] in palette {
            bytes.extend_from_slice(&[*blue, *green, *red, 0]);
        }
        bytes.extend_from_slice(pixels);
        bytes
    }
}
