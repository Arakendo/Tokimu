use crate::EditableRasterDocument;
use crc32fast::Hasher;
use flate2::{write::ZlibEncoder, Compression};
use serde::{Deserialize, Serialize};
use std::io::Write;
use thiserror::Error;

const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

/// Explicit output bound for the first local PNG export provider.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExportConfig {
    pub max_output_bytes: usize,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            max_output_bytes: 96 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportObservation {
    pub schema: u32,
    pub format: &'static str,
    pub width: u32,
    pub height: u32,
    pub output_bytes: usize,
    pub color_interpretation: &'static str,
    pub alpha_mode: &'static str,
    pub orientation: &'static str,
    pub source_revision: u64,
    pub pixel_fingerprint: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LosslessExport {
    pub bytes: Vec<u8>,
    pub observation: ExportObservation,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ExportError {
    #[error("PNG export scanline byte arithmetic overflowed")]
    ScanlineSizeOverflow,
    #[error(
        "PNG export requires {actual} bytes before compression, exceeding its {limit}-byte bound"
    )]
    SourceLimitExceeded { actual: usize, limit: usize },
    #[error("PNG export output is {actual} bytes, exceeding its {limit}-byte bound")]
    OutputLimitExceeded { actual: usize, limit: usize },
    #[error("PNG export compression failed: {0}")]
    Compression(String),
}

/// Encodes the current authoritative document as an RGBA8, top-down PNG.
///
/// The original source encoding is deliberately not consulted or preserved.
pub fn export_png(
    document: &EditableRasterDocument,
    config: ExportConfig,
) -> Result<LosslessExport, ExportError> {
    let scanline_bytes = document
        .row_stride()
        .checked_add(1)
        .ok_or(ExportError::ScanlineSizeOverflow)?;
    let source_bytes = scanline_bytes
        .checked_mul(document.height() as usize)
        .ok_or(ExportError::ScanlineSizeOverflow)?;
    if source_bytes > config.max_output_bytes {
        return Err(ExportError::SourceLimitExceeded {
            actual: source_bytes,
            limit: config.max_output_bytes,
        });
    }

    let mut scanlines = Vec::with_capacity(source_bytes);
    for row in document.pixels().chunks_exact(document.row_stride()) {
        scanlines.push(0); // Filter None keeps byte production deterministic.
        scanlines.extend_from_slice(row);
    }

    let compressed = compress(&scanlines)?;
    let mut bytes = Vec::with_capacity(PNG_SIGNATURE.len() + 64 + compressed.len());
    bytes.extend_from_slice(PNG_SIGNATURE);
    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&document.width().to_be_bytes());
    header.extend_from_slice(&document.height().to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]); // RGBA8, default PNG methods.
    append_chunk(&mut bytes, *b"IHDR", &header);
    append_chunk(&mut bytes, *b"IDAT", &compressed);
    append_chunk(&mut bytes, *b"IEND", &[]);
    if bytes.len() > config.max_output_bytes {
        return Err(ExportError::OutputLimitExceeded {
            actual: bytes.len(),
            limit: config.max_output_bytes,
        });
    }

    let document_observation = document.observation();
    Ok(LosslessExport {
        observation: ExportObservation {
            schema: 1,
            format: "png-rgba8",
            width: document.width(),
            height: document.height(),
            output_bytes: bytes.len(),
            color_interpretation: "color-srgb",
            alpha_mode: "straight",
            orientation: "top-down",
            source_revision: document.revision(),
            pixel_fingerprint: document_observation.pixel_fingerprint,
        },
        bytes,
    })
}

fn compress(source: &[u8]) -> Result<Vec<u8>, ExportError> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(source)
        .map_err(|error| ExportError::Compression(error.to_string()))?;
    encoder
        .finish()
        .map_err(|error| ExportError::Compression(error.to_string()))
}

fn append_chunk(output: &mut Vec<u8>, chunk_type: [u8; 4], data: &[u8]) {
    output.extend_from_slice(&(data.len() as u32).to_be_bytes());
    output.extend_from_slice(&chunk_type);
    output.extend_from_slice(data);
    let mut hasher = Hasher::new();
    hasher.update(&chunk_type);
    hasher.update(data);
    output.extend_from_slice(&hasher.finalize().to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::{export_png, ExportConfig, ExportError};
    use crate::{
        apply_command, DocumentConfig, EditableRasterDocument, PaintCommand, PixelPoint, Rgba8,
    };
    use raster_image_corpus::{decode_png, DecodeLimits, ImageOrientation};

    #[test]
    fn exported_png_round_trips_exact_rgba_pixels_and_metadata() {
        let document =
            EditableRasterDocument::blank(2, 1, Rgba8::TRANSPARENT, DocumentConfig::default())
                .unwrap();
        let mut document = document;
        apply_command(
            &mut document,
            &PaintCommand::PencilStroke {
                points: vec![PixelPoint { x: 1, y: 0 }],
                color: Rgba8 {
                    red: 12,
                    green: 34,
                    blue: 56,
                    alpha: 78,
                },
            },
        )
        .unwrap();
        let export = export_png(&document, ExportConfig::default()).unwrap();
        let decoded = decode_png(&export.bytes, DecodeLimits::default()).unwrap();

        assert_eq!(decoded.width, document.width());
        assert_eq!(decoded.height, document.height());
        assert_eq!(decoded.output_orientation, ImageOrientation::TopDown);
        assert_eq!(decoded.pixels, document.pixels());
        assert_eq!(&decoded.pixels[..4], &[0, 0, 0, 0]);
        assert_eq!(
            export.observation.pixel_fingerprint,
            document.observation().pixel_fingerprint
        );
    }

    #[test]
    fn output_bounds_reject_without_mutating_the_document() {
        let document =
            EditableRasterDocument::blank(2, 1, Rgba8::TRANSPARENT, DocumentConfig::default())
                .unwrap();
        let before = document.observation();

        assert_eq!(
            export_png(
                &document,
                ExportConfig {
                    max_output_bytes: 8,
                },
            ),
            Err(ExportError::SourceLimitExceeded {
                actual: 9,
                limit: 8,
            })
        );
        assert_eq!(document.observation(), before);
    }
}
