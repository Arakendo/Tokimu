//! Browser-facing semantic host for the original Tokimu visualizer corpus.
//!
//! The session owns fixture selection, time progression, waveform lowering,
//! and diagnostics. JavaScript receives only serializable observations and is
//! deliberately not a MilkDrop evaluator or audio-analysis implementation.

use serde::Serialize;
use visualizer_tools::{
    AudioBands, BeatObservation, SyntheticAudioFixture, SyntheticVisualizerConfig,
    SyntheticVisualizerInput, VisualizerViewport, VisualizerWaveform,
};
use wasm_bindgen::prelude::*;

const DEFAULT_WIDTH: u32 = 960;
const DEFAULT_HEIGHT: u32 = 540;

#[wasm_bindgen]
pub struct WasmVisualizerSession {
    source: SyntheticVisualizerInput,
    frame_index: u64,
    paused: bool,
}

#[wasm_bindgen]
impl WasmVisualizerSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Self, JsValue> {
        Self::with_fixture(SyntheticAudioFixture::FrequencySweep)
    }

    pub fn set_fixture(&mut self, fixture: &str) -> Result<(), JsValue> {
        let fixture = parse_fixture(fixture).map_err(js_error)?;
        self.source.set_fixture(fixture);
        self.frame_index = 0;
        Ok(())
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn reset(&mut self) {
        self.frame_index = 0;
    }

    /// Advances exactly one fixed semantic frame unless paused.
    pub fn step_json(&mut self, width: u32, height: u32) -> Result<String, JsValue> {
        let viewport = VisualizerViewport::new(
            if width == 0 { DEFAULT_WIDTH } else { width },
            if height == 0 { DEFAULT_HEIGHT } else { height },
        )
        .map_err(js_error)?;
        let frame = self
            .source
            .frame(self.frame_index, viewport)
            .map_err(js_error)?;
        let waveform = VisualizerWaveform::from_frame(&frame, 0.82).map_err(js_error)?;
        let snapshot = VisualizerSnapshot {
            schema: 1,
            fixture: frame.fixture.label(),
            frame_index: frame.frame_index,
            time_seconds: frame.time_seconds,
            paused: self.paused,
            bands: frame.bands,
            beat: frame.beat,
            waveform: waveform.points,
            diagnostics: VisualizerDiagnostics {
                audio_source: "synthetic-fixture",
                microphone_permission: "not-required",
                preset_evaluator: "not-present",
                provider: "visualizer-tools",
            },
        };
        if !self.paused {
            self.frame_index = self.frame_index.saturating_add(1);
        }
        serde_json::to_string(&snapshot).map_err(|error| js_error(error.to_string()))
    }
}

impl WasmVisualizerSession {
    fn with_fixture(fixture: SyntheticAudioFixture) -> Result<Self, JsValue> {
        let source = SyntheticVisualizerInput::new(fixture, SyntheticVisualizerConfig::default())
            .map_err(js_error)?;
        Ok(Self {
            source,
            frame_index: 0,
            paused: false,
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct VisualizerSnapshot {
    schema: u32,
    fixture: &'static str,
    frame_index: u64,
    time_seconds: f32,
    paused: bool,
    bands: AudioBands,
    beat: BeatObservation,
    waveform: Vec<[f32; 2]>,
    diagnostics: VisualizerDiagnostics,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct VisualizerDiagnostics {
    audio_source: &'static str,
    microphone_permission: &'static str,
    preset_evaluator: &'static str,
    provider: &'static str,
}

fn parse_fixture(value: &str) -> Result<SyntheticAudioFixture, String> {
    SyntheticAudioFixture::ALL
        .into_iter()
        .find(|fixture| fixture.label() == value)
        .ok_or_else(|| format!("unknown synthetic visualizer fixture `{value}`"))
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

#[cfg(test)]
mod tests {
    use super::parse_fixture;

    #[test]
    fn fixture_selection_is_bounded_to_known_synthetic_sources() {
        assert!(parse_fixture("frequency-sweep").is_ok());
        assert!(parse_fixture("microphone").is_err());
    }
}
