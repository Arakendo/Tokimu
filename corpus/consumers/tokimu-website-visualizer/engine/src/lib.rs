//! Browser-facing semantic host for the original Tokimu visualizer corpus.
//!
//! The session owns fixture selection, time progression, waveform lowering,
//! and diagnostics. JavaScript receives only serializable observations and is
//! deliberately not a MilkDrop evaluator or audio-analysis implementation.

use milkdrop_tools::{
    inspect_shader_entries, MilkDropClassicFrameControls, MilkDropSelectedRuntime,
    MilkDropShaderTranslationBlocker,
};
use serde::Serialize;
use visualizer_tools::{
    AudioBands, BeatObservation, SyntheticAudioFixture, SyntheticVisualizerConfig,
    SyntheticVisualizerInput, VisualizerViewport, VisualizerWaveform,
};
use wasm_bindgen::prelude::*;

const DEFAULT_WIDTH: u32 = 960;
const DEFAULT_HEIGHT: u32 = 540;
const SELECTED_MILKDROP_FIXTURE: &str =
    include_str!("../../../../focused/audio/hello-milkdrop/assets/tokimu-selected-fixture.milk");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VisualizerMode {
    Original,
    MilkDropSelected,
}

impl VisualizerMode {
    fn label(self) -> &'static str {
        match self {
            Self::Original => "original",
            Self::MilkDropSelected => "milkdrop-selected",
        }
    }
}

#[wasm_bindgen]
pub struct WasmVisualizerSession {
    source: SyntheticVisualizerInput,
    frame_index: u64,
    paused: bool,
    mode: VisualizerMode,
    milkdrop_runtime: MilkDropSelectedRuntime,
    milkdrop_controls: Option<MilkDropClassicFrameControls>,
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
        self.reset()?;
        Ok(())
    }

    pub fn set_mode(&mut self, mode: &str) -> Result<(), JsValue> {
        self.mode = parse_mode(mode).map_err(js_error)?;
        self.reset()?;
        Ok(())
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn reset(&mut self) -> Result<(), JsValue> {
        self.frame_index = 0;
        self.milkdrop_runtime.reset().map_err(js_error)?;
        self.milkdrop_controls = None;
        Ok(())
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
        let milkdrop = if self.mode == VisualizerMode::MilkDropSelected {
            if self
                .milkdrop_controls
                .as_ref()
                .is_none_or(|controls| controls.frame_index != frame.frame_index)
            {
                self.milkdrop_controls = Some(
                    self.milkdrop_runtime
                        .step_with_audio(
                            frame.frame_index,
                            frame.time_seconds,
                            selected_bands(&frame),
                            &frame.waveform,
                            &frame.spectrum,
                        )
                        .map_err(js_error)?,
                );
            }
            self.milkdrop_controls.as_ref().map(|controls| {
                MilkDropBrowserControls::from_runtime(controls, &self.milkdrop_runtime)
            })
        } else {
            None
        };
        let snapshot = VisualizerSnapshot {
            schema: 1,
            fixture: frame.fixture.label(),
            mode: self.mode.label(),
            frame_index: frame.frame_index,
            time_seconds: frame.time_seconds,
            paused: self.paused,
            bands: BrowserAudioBands::from(frame.bands),
            beat: frame.beat,
            waveform: waveform.points,
            milkdrop,
            diagnostics: VisualizerDiagnostics {
                audio_source: "synthetic-fixture",
                microphone_permission: "not-required",
                preset_evaluator: match self.mode {
                    VisualizerMode::Original => "not-present",
                    VisualizerMode::MilkDropSelected => {
                        "milkdrop-tools/selected-first-party-subset"
                    }
                },
                provider: match self.mode {
                    VisualizerMode::Original => "visualizer-tools",
                    VisualizerMode::MilkDropSelected => "visualizer-tools + milkdrop-tools",
                },
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
            mode: VisualizerMode::Original,
            milkdrop_runtime: MilkDropSelectedRuntime::from_source(SELECTED_MILKDROP_FIXTURE)
                .map_err(js_error)?,
            milkdrop_controls: None,
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct VisualizerSnapshot {
    schema: u32,
    fixture: &'static str,
    mode: &'static str,
    frame_index: u64,
    time_seconds: f32,
    paused: bool,
    bands: BrowserAudioBands,
    beat: BeatObservation,
    waveform: Vec<[f32; 2]>,
    milkdrop: Option<MilkDropBrowserControls>,
    diagnostics: VisualizerDiagnostics,
}

/// Browser-facing projection of provider-neutral audio observations.
///
/// Rust keeps the source fields idiomatic; the WASM contract uses the
/// camel-case names consumed by TypeScript.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BrowserAudioBands {
    sub_bass: f32,
    bass: f32,
    low_mid: f32,
    mid: f32,
    high_mid: f32,
    treble: f32,
}

impl From<AudioBands> for BrowserAudioBands {
    fn from(bands: AudioBands) -> Self {
        Self {
            sub_bass: bands.sub_bass,
            bass: bands.bass,
            low_mid: bands.low_mid,
            mid: bands.mid,
            high_mid: bands.high_mid,
            treble: bands.treble,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MilkDropBrowserControls {
    phase: f32,
    audio_energy: f32,
    decay: f32,
    zoom: f32,
    evaluated_assignments: usize,
    custom_wave_count: usize,
    custom_waves: Vec<MilkDropBrowserCustomWave>,
    custom_shape_count: usize,
    custom_shapes: Vec<MilkDropBrowserCustomShape>,
    shader_inspection: MilkDropBrowserShaderInspection,
}

/// A compact browser-safe summary of source inspection. It records only the
/// selected provider boundary; source is neither translated nor executed.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MilkDropBrowserShaderInspection {
    entries: usize,
    blockers: usize,
    texture_sampling_entries: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MilkDropBrowserCustomWave {
    points: Vec<[f32; 2]>,
    color: [f32; 4],
    dots: bool,
    thick: bool,
    additive: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MilkDropBrowserCustomShape {
    points: Vec<[f32; 2]>,
    color: [f32; 4],
    additive: bool,
    thick_outline: bool,
    textured: bool,
}

impl MilkDropBrowserControls {
    fn from_runtime(
        controls: &MilkDropClassicFrameControls,
        runtime: &MilkDropSelectedRuntime,
    ) -> Self {
        let inspections = inspect_shader_entries(runtime.document());
        let texture_sampling_entries = inspections
            .iter()
            .filter(|inspection| {
                inspection
                    .blockers
                    .contains(&MilkDropShaderTranslationBlocker::TextureRequirementsUnderReview)
            })
            .count();
        Self {
            phase: controls.phase,
            audio_energy: controls.audio_energy,
            decay: controls.decay,
            zoom: controls.zoom,
            evaluated_assignments: controls.evaluated_assignments,
            custom_wave_count: controls.custom_waves.len(),
            custom_waves: controls
                .custom_wave_frames
                .iter()
                .map(|frame| MilkDropBrowserCustomWave {
                    points: frame.points.clone(),
                    color: frame.wave.color,
                    dots: frame.wave.dots,
                    thick: frame.wave.thick,
                    additive: frame.wave.additive,
                })
                .collect(),
            custom_shape_count: controls.custom_shapes.len(),
            custom_shapes: controls
                .custom_shape_frames
                .iter()
                .map(|frame| MilkDropBrowserCustomShape {
                    points: frame.points.clone(),
                    color: frame.shape.color,
                    additive: frame.shape.additive,
                    thick_outline: frame.shape.thick_outline,
                    textured: frame.shape.textured,
                })
                .collect(),
            shader_inspection: MilkDropBrowserShaderInspection {
                entries: inspections.len(),
                blockers: inspections
                    .iter()
                    .map(|inspection| inspection.blockers.len())
                    .sum(),
                texture_sampling_entries,
            },
        }
    }
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

fn parse_mode(value: &str) -> Result<VisualizerMode, String> {
    match value {
        "original" => Ok(VisualizerMode::Original),
        "milkdrop-selected" => Ok(VisualizerMode::MilkDropSelected),
        _ => Err(format!("unknown visualizer mode `{value}`")),
    }
}

fn selected_bands(frame: &visualizer_tools::VisualizerFrameInput) -> [f32; 3] {
    [
        (frame.bands.sub_bass + frame.bands.bass) * 0.5,
        (frame.bands.low_mid + frame.bands.mid) * 0.5,
        (frame.bands.high_mid + frame.bands.treble) * 0.5,
    ]
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        parse_fixture, parse_mode, MilkDropBrowserControls, MilkDropSelectedRuntime,
        WasmVisualizerSession,
    };

    #[test]
    fn fixture_selection_is_bounded_to_known_synthetic_sources() {
        assert!(parse_fixture("frequency-sweep").is_ok());
        assert!(parse_fixture("microphone").is_err());
    }

    #[test]
    fn visualizer_mode_selection_is_bounded_to_the_admitted_browser_modes() {
        assert!(parse_mode("original").is_ok());
        assert!(parse_mode("milkdrop-selected").is_ok());
        assert!(parse_mode("projectm").is_err());
    }

    #[test]
    fn selected_mode_reports_shader_inspection_without_translation() {
        let mut runtime = MilkDropSelectedRuntime::from_source(super::SELECTED_MILKDROP_FIXTURE)
            .expect("the embedded selected fixture is valid");
        let controls = runtime
            .step_with_audio(1, 0.0, [0.0; 3], &[0.0; 8], &[0.0; 8])
            .expect("the selected fixture has bounded frame controls");
        let browser_controls = MilkDropBrowserControls::from_runtime(&controls, &runtime);

        assert_eq!(browser_controls.shader_inspection.entries, 1);
        assert!(browser_controls.shader_inspection.blockers >= 1);
        assert_eq!(
            browser_controls.shader_inspection.texture_sampling_entries,
            0
        );
    }

    #[test]
    fn browser_snapshot_projects_audio_bands_with_camel_case_names() {
        let mut session = WasmVisualizerSession::new().expect("the browser session is valid");
        let snapshot = session
            .step_json(960, 540)
            .expect("the first browser snapshot is serializable");
        let value: serde_json::Value =
            serde_json::from_str(&snapshot).expect("the snapshot is valid JSON");
        let bands = value["bands"].as_object().expect("the snapshot has bands");

        assert!(bands.contains_key("subBass"));
        assert!(bands.contains_key("lowMid"));
        assert!(bands.contains_key("highMid"));
        assert!(!bands.contains_key("sub_bass"));
    }
}
