use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PixelFormat {
    Rgba8,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ColorSpace {
    Srgb,
    #[default]
    Unspecified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AlphaMode {
    Opaque,
    Straight,
    Unspecified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImageOrientation {
    TopDown,
    BottomUp,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DecodeLimits {
    pub max_source_bytes: usize,
    pub max_width: u32,
    pub max_height: u32,
    pub max_decoded_bytes: usize,
}

impl Default for DecodeLimits {
    fn default() -> Self {
        Self {
            max_source_bytes: 64 * 1024 * 1024,
            max_width: 16_384,
            max_height: 16_384,
            max_decoded_bytes: 256 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub row_stride: usize,
    pub pixel_format: PixelFormat,
    pub color_space: ColorSpace,
    pub alpha_mode: AlphaMode,
    pub source_orientation: ImageOrientation,
    pub output_orientation: ImageOrientation,
    pub source_bit_depth: u16,
    pub source_row_stride: usize,
    pub pixels: Vec<u8>,
}

impl DecodedImage {
    pub fn validate(&self) -> Result<(), RasterImageError> {
        let expected_stride = usize::try_from(self.width)
            .ok()
            .and_then(|width| width.checked_mul(4))
            .ok_or(RasterImageError::DecodedSizeOverflow)?;
        if self.row_stride != expected_stride {
            return Err(RasterImageError::InvalidDecodedStride {
                expected: expected_stride,
                actual: self.row_stride,
            });
        }
        let expected_len = expected_stride
            .checked_mul(self.height as usize)
            .ok_or(RasterImageError::DecodedSizeOverflow)?;
        if self.pixels.len() != expected_len {
            return Err(RasterImageError::InvalidDecodedLength {
                expected: expected_len,
                actual: self.pixels.len(),
            });
        }
        Ok(())
    }

    /// Returns a stable FNV-1a fingerprint of the decoded pixel bytes.
    pub fn pixel_fingerprint(&self) -> String {
        let mut hash = 0xcbf29ce484222325_u64;
        for byte in &self.pixels {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        format!("{hash:016x}")
    }

    pub fn artifact(&self) -> Result<DecodedImageArtifact, RasterImageError> {
        self.validate()?;
        Ok(DecodedImageArtifact {
            schema: 1,
            artifact_kind: "decoded-image",
            width: self.width,
            height: self.height,
            row_stride: self.row_stride,
            decoded_bytes: self.pixels.len(),
            pixel_format: self.pixel_format,
            color_space: self.color_space,
            alpha_mode: self.alpha_mode,
            source_orientation: self.source_orientation,
            output_orientation: self.output_orientation,
            source_bit_depth: self.source_bit_depth,
            source_row_stride: self.source_row_stride,
            fingerprint_algorithm: "fnv1a64",
            pixel_fingerprint: self.pixel_fingerprint(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DecodedImageArtifact {
    pub schema: u32,
    pub artifact_kind: &'static str,
    pub width: u32,
    pub height: u32,
    pub row_stride: usize,
    pub decoded_bytes: usize,
    pub pixel_format: PixelFormat,
    pub color_space: ColorSpace,
    pub alpha_mode: AlphaMode,
    pub source_orientation: ImageOrientation,
    pub output_orientation: ImageOrientation,
    pub source_bit_depth: u16,
    pub source_row_stride: usize,
    pub fingerprint_algorithm: &'static str,
    pub pixel_fingerprint: String,
}

impl DecodedImageArtifact {
    pub fn to_pretty_json(&self) -> Result<String, RasterImageError> {
        serde_json::to_string_pretty(self)
            .map(|json| format!("{json}\n"))
            .map_err(|error| RasterImageError::ArtifactSerialization(error.to_string()))
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum RasterImageError {
    #[error("source contains {actual} bytes, exceeding the configured limit of {limit}")]
    SourceLimitExceeded { actual: usize, limit: usize },
    #[error(
        "BMP source is too short for {context}: expected at least {expected} bytes, got {actual}"
    )]
    Truncated {
        context: &'static str,
        expected: usize,
        actual: usize,
    },
    #[error("source does not begin with the BMP signature")]
    InvalidBmpSignature,
    #[error("BMP declares file size {declared}, but the source has {actual} bytes")]
    InvalidDeclaredFileSize { declared: usize, actual: usize },
    #[error("unsupported BMP DIB header size {0}; the first profile requires at least 40 bytes")]
    UnsupportedBmpHeader(u32),
    #[error("BMP planes must equal 1, got {0}")]
    InvalidBmpPlanes(u16),
    #[error("unsupported BMP bit depth {0}; the first profile accepts 8, 24, or 32")]
    UnsupportedBmpBitDepth(u16),
    #[error("unsupported BMP compression mode {0}; the first profile accepts BI_RGB")]
    UnsupportedBmpCompression(u32),
    #[error("BMP width must be positive, got {0}")]
    InvalidBmpWidth(i32),
    #[error("BMP height must be non-zero and representable, got {0}")]
    InvalidBmpHeight(i32),
    #[error(
        "image dimensions {width}x{height} exceed the configured limit {max_width}x{max_height}"
    )]
    DimensionLimitExceeded {
        width: u32,
        height: u32,
        max_width: u32,
        max_height: u32,
    },
    #[error("decoded image size arithmetic overflowed")]
    DecodedSizeOverflow,
    #[error("decoded image requires {actual} bytes, exceeding the configured limit of {limit}")]
    DecodedLimitExceeded { actual: usize, limit: usize },
    #[error("BMP pixel offset {offset} precedes the end of its DIB header at {minimum}")]
    InvalidBmpPixelOffset { offset: usize, minimum: usize },
    #[error(
        "BMP pixel data ends at byte {expected_end}, beyond the available source length {actual}"
    )]
    TruncatedBmpPixels { expected_end: usize, actual: usize },
    #[error(
        "BMP palette requires {expected} bytes before pixel data, but only {actual} are available"
    )]
    InvalidBmpPalette { expected: usize, actual: usize },
    #[error("BMP palette index {index} exceeds {entries} entries")]
    InvalidBmpPaletteIndex { index: usize, entries: usize },
    #[error("decoded row stride should be {expected} bytes, got {actual}")]
    InvalidDecodedStride { expected: usize, actual: usize },
    #[error("decoded image should contain {expected} bytes, got {actual}")]
    InvalidDecodedLength { expected: usize, actual: usize },
    #[error("decoded-image artifact serialization failed: {0}")]
    ArtifactSerialization(String),
    #[error("source does not begin with the PNG signature")]
    InvalidPngSignature,
    #[error("PNG chunk framing is truncated at byte {offset}")]
    TruncatedPngChunk { offset: usize },
    #[error("PNG chunk {chunk} has an invalid CRC")]
    InvalidPngCrc { chunk: String },
    #[error("PNG IHDR must be the first chunk and occur exactly once")]
    InvalidPngHeaderOrder,
    #[error("PNG IHDR must contain 13 bytes, got {0}")]
    InvalidPngHeaderLength(usize),
    #[error("PNG dimensions must be non-zero")]
    InvalidPngDimensions,
    #[error("unsupported PNG bit depth {bit_depth} for color type {color_type}")]
    UnsupportedPngBitDepth { bit_depth: u8, color_type: u8 },
    #[error("unsupported PNG color type {0}")]
    UnsupportedPngColorType(u8),
    #[error("unsupported PNG compression method {0}")]
    UnsupportedPngCompression(u8),
    #[error("unsupported PNG filter method {0}")]
    UnsupportedPngFilterMethod(u8),
    #[error("unsupported PNG interlace method {0}; the first profile is non-interlaced")]
    UnsupportedPngInterlace(u8),
    #[error("PNG palette is required before indexed pixel data")]
    MissingPngPalette,
    #[error("PNG palette length {0} is invalid")]
    InvalidPngPaletteLength(usize),
    #[error("PNG transparency chunk is invalid for color type {color_type}")]
    InvalidPngTransparency { color_type: u8 },
    #[error("PNG IDAT chunks must be consecutive")]
    NonConsecutivePngData,
    #[error("PNG is missing IDAT or IEND")]
    IncompletePng,
    #[error("unsupported critical PNG chunk {0}")]
    UnsupportedCriticalPngChunk(String),
    #[error("PNG decompression failed: {0}")]
    PngDecompression(String),
    #[error("PNG decompressed to {actual} bytes; expected exactly {expected}")]
    InvalidPngDecodedLength { expected: usize, actual: usize },
    #[error("PNG scanline uses unsupported filter type {0}")]
    UnsupportedPngScanlineFilter(u8),
    #[error("PNG palette index {index} exceeds {entries} entries")]
    InvalidPngPaletteIndex { index: usize, entries: usize },
    #[error("invalid PNG metadata: {0}")]
    InvalidPngMetadata(&'static str),
    #[error("source does not begin with the JPEG SOI marker")]
    InvalidJpegSignature,
    #[error("JPEG marker framing is truncated at byte {offset}")]
    TruncatedJpegMarker { offset: usize },
    #[error("JPEG does not end with the required EOI marker")]
    MissingJpegEnd,
    #[error("JPEG does not contain a start-of-frame marker before image data")]
    MissingJpegFrame,
    #[error("JPEG start-of-frame length is {declared} bytes; expected {expected}")]
    InvalidJpegFrameLength { declared: usize, expected: usize },
    #[error("JPEG frame dimensions must be non-zero")]
    InvalidJpegDimensions,
    #[error("unsupported JPEG frame marker 0x{marker:02X}; the first profile accepts SOF0")]
    UnsupportedJpegFrame { marker: u8 },
    #[error("unsupported JPEG sample precision {0}; the first profile accepts 8-bit samples")]
    UnsupportedJpegPrecision(u8),
    #[error("unsupported JPEG component count {0}; the first profile accepts grayscale or YCbCr")]
    UnsupportedJpegComponents(u8),
    #[error("invalid JPEG metadata: {0}")]
    InvalidJpegMetadata(&'static str),
    #[error("JPEG decoder failed: {0}")]
    JpegDecode(String),
    #[error(
        "JPEG decoder reported {actual_width}x{actual_height}; preflight declared {expected_width}x{expected_height}"
    )]
    JpegHeaderMismatch {
        expected_width: u32,
        expected_height: u32,
        actual_width: u32,
        actual_height: u32,
    },
}

#[cfg(test)]
mod tests {
    use super::{
        AlphaMode, ColorSpace, DecodedImage, ImageOrientation, PixelFormat, RasterImageError,
    };

    fn image() -> DecodedImage {
        DecodedImage {
            width: 1,
            height: 2,
            row_stride: 4,
            pixel_format: PixelFormat::Rgba8,
            color_space: ColorSpace::Unspecified,
            alpha_mode: AlphaMode::Opaque,
            source_orientation: ImageOrientation::BottomUp,
            output_orientation: ImageOrientation::TopDown,
            source_bit_depth: 24,
            source_row_stride: 4,
            pixels: vec![1, 2, 3, 255, 4, 5, 6, 255],
        }
    }

    #[test]
    fn artifact_is_deterministic_and_omits_raw_pixels() {
        let image = image();
        let first = image.artifact().unwrap().to_pretty_json().unwrap();
        let second = image.artifact().unwrap().to_pretty_json().unwrap();

        assert_eq!(first, second);
        assert!(first.ends_with('\n'));
        assert!(first.contains("\"artifact_kind\": \"decoded-image\""));
        assert!(first.contains("\"fingerprint_algorithm\": \"fnv1a64\""));
        assert!(first.contains(&image.pixel_fingerprint()));
        assert!(!first.contains("\"pixels\""));
    }

    #[test]
    fn artifact_rejects_invalid_decoded_evidence() {
        let mut image = image();
        image.row_stride = 8;

        assert_eq!(
            image.artifact(),
            Err(RasterImageError::InvalidDecodedStride {
                expected: 4,
                actual: 8,
            })
        );
    }
}
