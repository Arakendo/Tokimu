use crate::{PaintCommand, PaintSession, PaintSessionConfig, PixelPoint, Rgba8};
use raster_image_corpus::{decode_bmp, decode_jpeg, decode_png, DecodeLimits, DecodedImage};
use wasm_bindgen::prelude::*;

/// Browser-facing adapter for the Paint consumer.
///
/// Commands and observations are small JSON control records. Preview and
/// export use explicit byte copies so a Canvas, DOM object, or decoder-native
/// object never becomes authoritative application state.
#[wasm_bindgen]
pub struct WasmPaintSession {
    session: PaintSession,
}

#[wasm_bindgen]
impl WasmPaintSession {
    #[wasm_bindgen(constructor)]
    pub fn new_blank(
        width: u32,
        height: u32,
        red: u8,
        green: u8,
        blue: u8,
        alpha: u8,
    ) -> Result<Self, JsValue> {
        PaintSession::new_blank(
            width,
            height,
            Rgba8 {
                red,
                green,
                blue,
                alpha,
            },
            PaintSessionConfig::default(),
        )
        .map(|session| Self { session })
        .map_err(js_error)
    }

    /// Opens one admitted encoded source through the Rust raster provider.
    #[wasm_bindgen]
    pub fn open(bytes: Vec<u8>, format: &str) -> Result<Self, JsValue> {
        let decoded = decode_source(&bytes, format).map_err(js_error)?;
        PaintSession::open_decoded(&decoded, PaintSessionConfig::default())
            .map(|session| Self { session })
            .map_err(js_error)
    }

    /// Applies a small semantic command record. Pixel and export buffers are
    /// deliberately not transported through this JSON path.
    pub fn apply_json(&mut self, command_json: &str) -> Result<String, JsValue> {
        let command = parse_command(command_json).map_err(js_error)?;
        let observation = self.session.apply(&command).map_err(js_error)?;
        json(observation).map_err(js_error)
    }

    pub fn observation_json(&self) -> Result<String, JsValue> {
        json(self.session.observation()).map_err(js_error)
    }

    pub fn undo_json(&mut self) -> Result<String, JsValue> {
        self.session.undo().map_err(js_error)?;
        json(self.session.observation()).map_err(js_error)
    }

    pub fn redo_json(&mut self) -> Result<String, JsValue> {
        self.session.redo().map_err(js_error)?;
        json(self.session.observation()).map_err(js_error)
    }

    pub fn reset_json(&mut self) -> Result<String, JsValue> {
        let observation = self.session.reset().map_err(js_error)?;
        json(observation).map_err(js_error)
    }

    pub fn sample_rgba(&self, x: u32, y: u32) -> Result<Vec<u8>, JsValue> {
        self.session
            .sample(PixelPoint { x, y })
            .map(|color| vec![color.red, color.green, color.blue, color.alpha])
            .map_err(js_error)
    }

    pub fn preview_bytes(&self) -> Result<Vec<u8>, JsValue> {
        self.session
            .preview()
            .map(|preview| preview.pixels)
            .map_err(js_error)
    }

    pub fn preview_observation_json(&self) -> Result<String, JsValue> {
        let preview = self.session.preview().map_err(js_error)?;
        serde_json::to_string(&PreviewObservation {
            schema: 1,
            width: preview.width,
            height: preview.height,
            row_stride: preview.row_stride,
            pixel_bytes: preview.pixels.len(),
            pixel_fingerprint: preview.pixel_fingerprint,
        })
        .map_err(|error| JsValue::from_str(&format!("preview observation failed: {error}")))
    }

    pub fn export_png_bytes(&self) -> Result<Vec<u8>, JsValue> {
        self.session
            .export_png()
            .map(|export| export.bytes)
            .map_err(js_error)
    }

    pub fn export_observation_json(&self) -> Result<String, JsValue> {
        let export = self.session.export_png().map_err(js_error)?;
        json(export.observation).map_err(js_error)
    }

    pub fn dispose(&mut self) {
        self.session.dispose();
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PreviewObservation {
    schema: u32,
    width: u32,
    height: u32,
    row_stride: usize,
    pixel_bytes: usize,
    pixel_fingerprint: String,
}

fn decode_source(bytes: &[u8], format: &str) -> Result<DecodedImage, String> {
    let limits = DecodeLimits::default();
    match format.trim().to_ascii_lowercase().as_str() {
        "png" | "image/png" => decode_png(bytes, limits),
        "jpeg" | "jpg" | "image/jpeg" => decode_jpeg(bytes, limits),
        "bmp" | "image/bmp" => decode_bmp(bytes, limits),
        _ => return Err(format!("unsupported Paint source format `{format}`")),
    }
    .map_err(|error| error.to_string())
}

fn parse_command(command_json: &str) -> Result<PaintCommand, String> {
    serde_json::from_str(command_json).map_err(|error| format!("invalid Paint command: {error}"))
}

fn json<T: serde::Serialize>(value: T) -> Result<String, String> {
    serde_json::to_string(&value)
        .map_err(|error| format!("Paint observation serialization failed: {error}"))
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{decode_source, parse_command};

    #[test]
    fn unsupported_source_format_is_rejected_before_decode() {
        assert!(decode_source(&[], "tiff").is_err());
    }

    #[test]
    fn malformed_control_json_is_rejected_before_document_mutation() {
        assert!(parse_command("{not-json}").is_err());
        assert!(parse_command(r#"{\"kind\":\"unknown\"}"#).is_err());
    }
}
