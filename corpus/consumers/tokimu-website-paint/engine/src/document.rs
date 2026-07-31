use raster_image_corpus::{DecodedImage, ImageOrientation, PixelFormat};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const BYTES_PER_PIXEL: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Rgba8 {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Rgba8 {
    pub const TRANSPARENT: Self = Self {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 0,
    };

    pub(crate) const fn bytes(self) -> [u8; BYTES_PER_PIXEL] {
        [self.red, self.green, self.blue, self.alpha]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DocumentConfig {
    pub max_width: u32,
    pub max_height: u32,
    pub max_pixel_bytes: usize,
}

impl Default for DocumentConfig {
    fn default() -> Self {
        Self {
            max_width: 4_096,
            max_height: 4_096,
            max_pixel_bytes: 64 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentObservation {
    pub schema: u32,
    pub width: u32,
    pub height: u32,
    pub row_stride: usize,
    pub pixel_bytes: usize,
    pub pixel_format: &'static str,
    pub color_interpretation: &'static str,
    pub alpha_mode: &'static str,
    pub orientation: &'static str,
    pub revision: u64,
    pub dirty: bool,
    pub fingerprint_algorithm: &'static str,
    pub pixel_fingerprint: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditableRasterDocument {
    width: u32,
    height: u32,
    row_stride: usize,
    pixels: Vec<u8>,
    revision: u64,
    dirty: bool,
}

impl EditableRasterDocument {
    pub fn blank(
        width: u32,
        height: u32,
        color: Rgba8,
        config: DocumentConfig,
    ) -> Result<Self, DocumentError> {
        let (row_stride, pixel_bytes) = validate_dimensions(width, height, config)?;
        let mut pixels = Vec::with_capacity(pixel_bytes);
        for _ in 0..u64::from(width) * u64::from(height) {
            pixels.extend_from_slice(&color.bytes());
        }

        Ok(Self {
            width,
            height,
            row_stride,
            pixels,
            revision: 0,
            dirty: false,
        })
    }

    pub fn from_decoded(
        source: &DecodedImage,
        config: DocumentConfig,
    ) -> Result<Self, DocumentError> {
        source
            .validate()
            .map_err(|error| DocumentError::InvalidDecodedImage(error.to_string()))?;
        if source.pixel_format != PixelFormat::Rgba8 {
            return Err(DocumentError::UnsupportedPixelFormat);
        }
        if source.output_orientation != ImageOrientation::TopDown {
            return Err(DocumentError::UnsupportedOrientation);
        }

        let (row_stride, pixel_bytes) = validate_dimensions(source.width, source.height, config)?;
        if source.row_stride != row_stride || source.pixels.len() != pixel_bytes {
            return Err(DocumentError::InvalidDecodedImage(
                "decoded storage does not match document dimensions".to_owned(),
            ));
        }

        Ok(Self {
            width: source.width,
            height: source.height,
            row_stride,
            pixels: source.pixels.clone(),
            revision: 0,
            dirty: false,
        })
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub const fn row_stride(&self) -> usize {
        self.row_stride
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub(crate) fn state_snapshot(&self) -> DocumentStateSnapshot {
        DocumentStateSnapshot {
            pixels: self.pixels.clone(),
            revision: self.revision,
            dirty: self.dirty,
        }
    }

    pub(crate) fn restore_state(&mut self, snapshot: DocumentStateSnapshot) {
        debug_assert_eq!(snapshot.pixels.len(), self.pixels.len());
        self.pixels = snapshot.pixels;
        self.revision = snapshot.revision;
        self.dirty = snapshot.dirty;
    }

    pub const fn revision(&self) -> u64 {
        self.revision
    }

    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn pixel(&self, x: u32, y: u32) -> Result<Rgba8, DocumentError> {
        let offset = self.pixel_offset(x, y)?;
        Ok(Rgba8 {
            red: self.pixels[offset],
            green: self.pixels[offset + 1],
            blue: self.pixels[offset + 2],
            alpha: self.pixels[offset + 3],
        })
    }

    pub fn observation(&self) -> DocumentObservation {
        DocumentObservation {
            schema: 1,
            width: self.width,
            height: self.height,
            row_stride: self.row_stride,
            pixel_bytes: self.pixels.len(),
            pixel_format: "rgba8",
            color_interpretation: "color-srgb",
            alpha_mode: "straight",
            orientation: "top-down",
            revision: self.revision,
            dirty: self.dirty,
            fingerprint_algorithm: "fnv1a64",
            pixel_fingerprint: pixel_fingerprint(&self.pixels),
        }
    }

    pub(crate) fn pixel_offset(&self, x: u32, y: u32) -> Result<usize, DocumentError> {
        if x >= self.width || y >= self.height {
            return Err(DocumentError::CoordinateOutOfBounds {
                x,
                y,
                width: self.width,
                height: self.height,
            });
        }

        Ok(y as usize * self.row_stride + x as usize * BYTES_PER_PIXEL)
    }

    pub(crate) fn replace_pixel_if_different(
        &mut self,
        x: u32,
        y: u32,
        color: Rgba8,
    ) -> Result<bool, DocumentError> {
        let offset = self.pixel_offset(x, y)?;
        if self.pixels[offset..offset + BYTES_PER_PIXEL] == color.bytes() {
            return Ok(false);
        }
        self.pixels[offset..offset + BYTES_PER_PIXEL].copy_from_slice(&color.bytes());
        Ok(true)
    }

    pub(crate) fn commit_edit(&mut self) {
        self.revision += 1;
        self.dirty = true;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DocumentStateSnapshot {
    pub pixels: Vec<u8>,
    pub revision: u64,
    pub dirty: bool,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum DocumentError {
    #[error("document dimensions must be non-zero, got {width}x{height}")]
    EmptyDimensions { width: u32, height: u32 },
    #[error(
        "document dimensions {width}x{height} exceed the configured limit {max_width}x{max_height}"
    )]
    DimensionLimitExceeded {
        width: u32,
        height: u32,
        max_width: u32,
        max_height: u32,
    },
    #[error("document pixel size arithmetic overflowed")]
    PixelSizeOverflow,
    #[error("document requires {actual} pixel bytes, exceeding the configured limit of {limit}")]
    PixelByteLimitExceeded { actual: usize, limit: usize },
    #[error("decoded image is invalid: {0}")]
    InvalidDecodedImage(String),
    #[error("decoded image pixel format is not supported by the first Paint profile")]
    UnsupportedPixelFormat,
    #[error("decoded image output orientation must be top-down")]
    UnsupportedOrientation,
    #[error("coordinate ({x}, {y}) is outside document bounds {width}x{height}")]
    CoordinateOutOfBounds {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
}

fn validate_dimensions(
    width: u32,
    height: u32,
    config: DocumentConfig,
) -> Result<(usize, usize), DocumentError> {
    if width == 0 || height == 0 {
        return Err(DocumentError::EmptyDimensions { width, height });
    }
    if width > config.max_width || height > config.max_height {
        return Err(DocumentError::DimensionLimitExceeded {
            width,
            height,
            max_width: config.max_width,
            max_height: config.max_height,
        });
    }

    let row_stride = usize::try_from(width)
        .ok()
        .and_then(|value| value.checked_mul(BYTES_PER_PIXEL))
        .ok_or(DocumentError::PixelSizeOverflow)?;
    let pixel_bytes = row_stride
        .checked_mul(height as usize)
        .ok_or(DocumentError::PixelSizeOverflow)?;
    if pixel_bytes > config.max_pixel_bytes {
        return Err(DocumentError::PixelByteLimitExceeded {
            actual: pixel_bytes,
            limit: config.max_pixel_bytes,
        });
    }
    Ok((row_stride, pixel_bytes))
}

fn pixel_fingerprint(pixels: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in pixels {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::{DocumentConfig, DocumentError, EditableRasterDocument, Rgba8};
    use raster_image_corpus::{
        decode_bmp, decode_jpeg, decode_png, AlphaMode, ColorSpace, DecodeLimits, DecodedImage,
        ImageOrientation, PixelFormat, RasterImageError,
    };
    use std::{fs, path::PathBuf};

    type DecodeFn = fn(&[u8], DecodeLimits) -> Result<DecodedImage, RasterImageError>;

    fn decoded_2x1() -> DecodedImage {
        DecodedImage {
            width: 2,
            height: 1,
            row_stride: 8,
            pixel_format: PixelFormat::Rgba8,
            color_space: ColorSpace::Srgb,
            alpha_mode: AlphaMode::Straight,
            source_orientation: ImageOrientation::TopDown,
            output_orientation: ImageOrientation::TopDown,
            source_bit_depth: 8,
            source_row_stride: 8,
            pixels: vec![255, 0, 0, 128, 0, 0, 255, 255],
        }
    }

    #[test]
    fn blank_document_is_bounded_and_observable() {
        let document =
            EditableRasterDocument::blank(2, 2, Rgba8::TRANSPARENT, DocumentConfig::default())
                .unwrap();
        let observation = document.observation();

        assert_eq!(document.row_stride(), 8);
        assert_eq!(document.pixels(), &[0; 16]);
        assert_eq!(observation.pixel_format, "rgba8");
        assert_eq!(observation.color_interpretation, "color-srgb");
        assert_eq!(observation.alpha_mode, "straight");
        assert_eq!(observation.orientation, "top-down");
        assert_eq!(observation.revision, 0);
        assert!(!observation.dirty);
    }

    #[test]
    fn odd_sized_documents_preserve_straight_alpha_pixels() {
        let color = Rgba8 {
            red: 12,
            green: 34,
            blue: 56,
            alpha: 78,
        };
        let document =
            EditableRasterDocument::blank(3, 5, color, DocumentConfig::default()).unwrap();

        assert_eq!(document.row_stride(), 12);
        assert_eq!(document.pixels().len(), 60);
        assert_eq!(document.pixel(2, 4).unwrap(), color);
        assert_eq!(document.observation().alpha_mode, "straight");
    }

    #[test]
    fn importing_copies_decoder_evidence_before_mutation() {
        let source = decoded_2x1();
        let source_pixels = source.pixels.clone();
        let mut document =
            EditableRasterDocument::from_decoded(&source, DocumentConfig::default()).unwrap();

        document
            .replace_pixel_if_different(
                0,
                0,
                Rgba8 {
                    red: 1,
                    green: 2,
                    blue: 3,
                    alpha: 4,
                },
            )
            .unwrap();
        document.commit_edit();

        assert_eq!(source.pixels, source_pixels);
        assert_ne!(document.pixels(), source.pixels);
        assert_eq!(document.revision(), 1);
        assert!(document.is_dirty());
    }

    #[test]
    fn invalid_dimensions_and_coordinates_fail_deterministically() {
        assert_eq!(
            EditableRasterDocument::blank(0, 1, Rgba8::TRANSPARENT, DocumentConfig::default()),
            Err(DocumentError::EmptyDimensions {
                width: 0,
                height: 1
            })
        );

        let document =
            EditableRasterDocument::blank(1, 1, Rgba8::TRANSPARENT, DocumentConfig::default())
                .unwrap();
        assert_eq!(
            document.pixel(1, 0),
            Err(DocumentError::CoordinateOutOfBounds {
                x: 1,
                y: 0,
                width: 1,
                height: 1
            })
        );
    }

    #[test]
    fn admitted_png_jpeg_and_bmp_enter_one_document_boundary() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..");
        let cases: [(&str, PathBuf, DecodeFn); 3] = [
            (
                "png",
                root.join(
                    "third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/basn6a08.png",
                ),
                decode_png,
            ),
            (
                "jpeg",
                root.join(
                    "third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages/testorig.jpg",
                ),
                decode_jpeg,
            ),
            (
                "bmp",
                root.join(
                    "third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages/shira_bird8.bmp",
                ),
                decode_bmp,
            ),
        ];

        for (name, path, decode) in cases {
            let source = fs::read(&path)
                .unwrap_or_else(|error| panic!("failed to read {name} fixture {path:?}: {error}"));
            let decoded = decode(&source, DecodeLimits::default()).unwrap();
            let document =
                EditableRasterDocument::from_decoded(&decoded, DocumentConfig::default()).unwrap();

            assert_eq!(document.width(), decoded.width, "{name}");
            assert_eq!(document.height(), decoded.height, "{name}");
            assert_eq!(document.pixels(), decoded.pixels, "{name}");
            assert_eq!(document.observation().pixel_format, "rgba8", "{name}");
        }
    }

    #[test]
    fn pixel_hash_is_stable_and_content_sensitive() {
        let source = decoded_2x1();
        let first =
            EditableRasterDocument::from_decoded(&source, DocumentConfig::default()).unwrap();
        let second =
            EditableRasterDocument::from_decoded(&source, DocumentConfig::default()).unwrap();
        let mut changed = second.clone();
        assert!(changed
            .replace_pixel_if_different(1, 0, Rgba8::TRANSPARENT)
            .unwrap());
        changed.commit_edit();

        assert_eq!(
            first.observation().pixel_fingerprint,
            second.observation().pixel_fingerprint
        );
        assert_ne!(
            first.observation().pixel_fingerprint,
            changed.observation().pixel_fingerprint
        );
    }
}
