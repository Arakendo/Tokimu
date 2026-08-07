// The corpus model remains private to this package. WASM exports bounded
// evidence only; browser code never receives Ratatui or font-provider types.
// The corpus binary and WASM facade intentionally share one private fixture
// model; the facade exports only opaque raster evidence, not every fixture path.
#![allow(dead_code)]

include!("main.rs");

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn independent_fixture_cpu_summary() -> Result<String, JsValue> {
    independent_fixture_summary().map_err(|error| JsValue::from_str(&error.to_string()))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn independent_fixture_cpu_width() -> Result<u32, JsValue> {
    independent_fixture_raster()
        .map(|raster| raster.width)
        .map_err(|error| JsValue::from_str(&error))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn independent_fixture_cpu_height() -> Result<u32, JsValue> {
    independent_fixture_raster()
        .map(|raster| raster.height)
        .map_err(|error| JsValue::from_str(&error))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn independent_fixture_cpu_rgba() -> Result<Vec<u8>, JsValue> {
    independent_fixture_raster()
        .map(|raster| raster.rgba)
        .map_err(|error| JsValue::from_str(&error))
}

#[cfg(target_arch = "wasm32")]
fn wasm_fixture(producer: &str) -> Result<FixtureProducer, JsValue> {
    FixtureProducer::parse(producer).map_err(|error| JsValue::from_str(&error))
}

/// Returns a provider-selected, fully rasterized CPU surface. The browser sees
/// pixels only and cannot receive the producer's layout or font state.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn terminal_fixture_cpu_summary(producer: &str) -> Result<String, JsValue> {
    wasm_fixture(producer)?
        .summary()
        .map_err(|error| JsValue::from_str(&error))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn terminal_fixture_cpu_width(producer: &str) -> Result<u32, JsValue> {
    wasm_fixture(producer)?
        .raster()
        .map(|raster| raster.width)
        .map_err(|error| JsValue::from_str(&error))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn terminal_fixture_cpu_height(producer: &str) -> Result<u32, JsValue> {
    wasm_fixture(producer)?
        .raster()
        .map(|raster| raster.height)
        .map_err(|error| JsValue::from_str(&error))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn terminal_fixture_cpu_rgba(producer: &str) -> Result<Vec<u8>, JsValue> {
    wasm_fixture(producer)?
        .raster()
        .map(|raster| raster.rgba)
        .map_err(|error| JsValue::from_str(&error))
}
