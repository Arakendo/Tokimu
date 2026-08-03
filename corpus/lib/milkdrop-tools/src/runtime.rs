use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    evaluate_selected_equations, lower_selected_custom_shapes, lower_selected_custom_waves,
    resolve_selected_custom_shapes, resolve_selected_custom_waves, resolve_selected_parameters,
    MilkDropCustomShape, MilkDropCustomShapeError, MilkDropCustomShapeFrame, MilkDropCustomWave,
    MilkDropCustomWaveError, MilkDropCustomWaveFrame, MilkDropEvaluationError,
    MilkDropEvaluationPhase, MilkDropEvaluationState, MilkDropParseError, MilkDropPresetDocument,
    MilkDropResolvedParameters,
};

/// Renderer-neutral controls produced by Tokimu's selected MilkDrop subset.
///
/// The four values intentionally match the bounded execution slot exercised by
/// `hello-audio-visualizer`; they are not a general MilkDrop uniform ABI.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MilkDropClassicFrameControls {
    pub schema: String,
    pub frame_index: u64,
    pub phase: f32,
    pub audio_energy: f32,
    pub decay: f32,
    pub zoom: f32,
    /// Selected custom-wave descriptions resolved from the preset source.
    /// Consumers may lower these through their own presentation contract.
    pub custom_waves: Vec<MilkDropCustomWave>,
    /// Selected literal custom waves lowered from explicitly supplied audio.
    ///
    /// This remains empty for [`MilkDropSelectedRuntime::step`], whose scalar
    /// frame API intentionally has no waveform or spectrum payload.
    pub custom_wave_frames: Vec<MilkDropCustomWaveFrame>,
    /// Selected literal custom-shape descriptions resolved from source.
    pub custom_shapes: Vec<MilkDropCustomShape>,
    /// Selected literal custom shapes lowered to normalized polygon points.
    ///
    /// The points have no mesh, fill, blend, or texture execution policy.
    pub custom_shape_frames: Vec<MilkDropCustomShapeFrame>,
    pub evaluated_assignments: usize,
    pub state: MilkDropEvaluationState,
}

impl MilkDropClassicFrameControls {
    pub fn shader_signal(&self) -> [f32; 4] {
        [self.phase, self.audio_energy, self.decay, self.zoom]
    }

    pub fn to_structural_json(&self) -> Result<String, MilkDropSelectedRuntimeError> {
        serde_json::to_string_pretty(self)
            .map_err(|error| MilkDropSelectedRuntimeError::Serialization(error.to_string()))
    }
}

/// Stateful executor for the deliberately selected MilkDrop 1-style subset.
///
/// Parsing and equation evaluation remain CPU-side and renderer-neutral. The
/// runtime injects only bounded frame/audio observations before evaluating the
/// selected per-frame equations. Per-pixel equations and custom source remain
/// deferred in the parsed document.
#[derive(Clone, Debug)]
pub struct MilkDropSelectedRuntime {
    document: MilkDropPresetDocument,
    parameters: MilkDropResolvedParameters,
    custom_waves: Vec<MilkDropCustomWave>,
    custom_shapes: Vec<MilkDropCustomShape>,
    state: MilkDropEvaluationState,
    last_frame_index: Option<u64>,
    evaluated_assignments: usize,
}

impl MilkDropSelectedRuntime {
    pub fn from_source(source: &str) -> Result<Self, MilkDropSelectedRuntimeError> {
        let document = MilkDropPresetDocument::parse(source)?;
        let parameters = resolve_selected_parameters(&document)?;
        let custom_waves = resolve_selected_custom_waves(&document)?;
        let custom_shapes = resolve_selected_custom_shapes(&document)?;
        let mut state = MilkDropEvaluationState::default();
        let evaluated_assignments = evaluate_selected_equations(
            &document,
            MilkDropEvaluationPhase::Initialization,
            &mut state,
        )?;
        Ok(Self {
            document,
            parameters,
            custom_waves,
            custom_shapes,
            state,
            last_frame_index: None,
            evaluated_assignments,
        })
    }

    pub fn document(&self) -> &MilkDropPresetDocument {
        &self.document
    }

    pub fn parameters(&self) -> &MilkDropResolvedParameters {
        &self.parameters
    }

    pub fn reset(&mut self) -> Result<(), MilkDropSelectedRuntimeError> {
        self.state = MilkDropEvaluationState::default();
        self.last_frame_index = None;
        self.evaluated_assignments = evaluate_selected_equations(
            &self.document,
            MilkDropEvaluationPhase::Initialization,
            &mut self.state,
        )?;
        Ok(())
    }

    pub fn step(
        &mut self,
        frame_index: u64,
        time_seconds: f32,
        bands: [f32; 3],
    ) -> Result<MilkDropClassicFrameControls, MilkDropSelectedRuntimeError> {
        self.step_inner(frame_index, time_seconds, bands, None)
    }

    /// Advances one frame and lowers selected literal custom waves from the
    /// supplied explicit audio observations.
    ///
    /// `wavecode` remains unexecuted. This method only applies admitted static
    /// custom-wave properties to the caller-provided waveform or spectrum.
    pub fn step_with_audio(
        &mut self,
        frame_index: u64,
        time_seconds: f32,
        bands: [f32; 3],
        waveform: &[f32],
        spectrum: &[f32],
    ) -> Result<MilkDropClassicFrameControls, MilkDropSelectedRuntimeError> {
        self.step_inner(frame_index, time_seconds, bands, Some((waveform, spectrum)))
    }

    fn step_inner(
        &mut self,
        frame_index: u64,
        time_seconds: f32,
        bands: [f32; 3],
        audio: Option<(&[f32], &[f32])>,
    ) -> Result<MilkDropClassicFrameControls, MilkDropSelectedRuntimeError> {
        if !time_seconds.is_finite() || bands.iter().any(|value| !value.is_finite()) {
            return Err(MilkDropSelectedRuntimeError::NonFiniteFrameInput);
        }

        if let Some(previous) = self.last_frame_index {
            if frame_index <= previous {
                return Err(MilkDropSelectedRuntimeError::NonIncreasingFrame {
                    previous,
                    next: frame_index,
                });
            }
        }
        self.last_frame_index = Some(frame_index);
        self.state
            .variables
            .insert("time".to_owned(), f64::from(time_seconds));
        self.state
            .variables
            .insert("frame".to_owned(), frame_index as f64);
        self.state
            .variables
            .insert("bass".to_owned(), f64::from(bands[0]));
        self.state
            .variables
            .insert("mid".to_owned(), f64::from(bands[1]));
        self.state
            .variables
            .insert("treb".to_owned(), f64::from(bands[2]));
        self.evaluated_assignments += evaluate_selected_equations(
            &self.document,
            MilkDropEvaluationPhase::PerFrame,
            &mut self.state,
        )?;

        let values = self.parameters.values;
        let decay = state_or_default(&self.state, "decay", values.decay)?;
        let zoom = state_or_default(&self.state, "zoom", values.zoom)?;
        if !(0.0..=1.0).contains(&decay) {
            return Err(MilkDropSelectedRuntimeError::ControlOutOfRange {
                control: "decay",
                value: decay,
                minimum: 0.0,
                maximum: 1.0,
            });
        }
        if !(0.25..=4.0).contains(&zoom) {
            return Err(MilkDropSelectedRuntimeError::ControlOutOfRange {
                control: "zoom",
                value: zoom,
                minimum: 0.25,
                maximum: 4.0,
            });
        }

        let rotation = state_or_default(&self.state, "rot", values.rotation)?;
        let warp_speed =
            state_or_default(&self.state, "warp_animspeed", values.warp_animation_speed)?;
        let phase = (time_seconds * warp_speed + rotation / std::f32::consts::TAU).rem_euclid(1.0);
        let audio_energy = ((bands[0] + bands[1] + bands[2]) / 3.0).clamp(0.0, 1.0);

        let custom_wave_frames = match audio {
            Some((waveform, spectrum)) => {
                lower_selected_custom_waves(&self.custom_waves, waveform, spectrum)?
            }
            None => Vec::new(),
        };
        let custom_shape_frames = lower_selected_custom_shapes(&self.custom_shapes)?;

        Ok(MilkDropClassicFrameControls {
            schema: "tokimu-milkdrop-classic-frame-v1".to_owned(),
            frame_index,
            phase,
            audio_energy,
            decay,
            zoom,
            custom_waves: self.custom_waves.clone(),
            custom_wave_frames,
            custom_shapes: self.custom_shapes.clone(),
            custom_shape_frames,
            evaluated_assignments: self.evaluated_assignments,
            state: self.state.clone(),
        })
    }
}

fn state_or_default(
    state: &MilkDropEvaluationState,
    name: &'static str,
    default: f32,
) -> Result<f32, MilkDropSelectedRuntimeError> {
    let value = state.value(name).unwrap_or(f64::from(default));
    if !value.is_finite() || value < f64::from(f32::MIN) || value > f64::from(f32::MAX) {
        return Err(MilkDropSelectedRuntimeError::InvalidControl {
            control: name,
            value,
        });
    }
    Ok(value as f32)
}

#[derive(Debug, Error)]
pub enum MilkDropSelectedRuntimeError {
    #[error(transparent)]
    Parse(#[from] MilkDropParseError),
    #[error(transparent)]
    Evaluation(#[from] MilkDropEvaluationError),
    #[error(transparent)]
    CustomWave(#[from] MilkDropCustomWaveError),
    #[error(transparent)]
    CustomShape(#[from] MilkDropCustomShapeError),
    #[error("MilkDrop frame input must be finite")]
    NonFiniteFrameInput,
    #[error("MilkDrop frame index must increase: previous={previous}, next={next}")]
    NonIncreasingFrame { previous: u64, next: u64 },
    #[error("MilkDrop control `{control}` resolved to invalid value {value}")]
    InvalidControl { control: &'static str, value: f64 },
    #[error(
        "MilkDrop control `{control}` resolved to {value}, outside admitted range {minimum}..={maximum}"
    )]
    ControlOutOfRange {
        control: &'static str,
        value: f32,
        minimum: f32,
        maximum: f32,
    },
    #[error("failed to serialize MilkDrop runtime evidence: {0}")]
    Serialization(String),
}

#[cfg(test)]
mod tests {
    use super::{MilkDropSelectedRuntime, MilkDropSelectedRuntimeError};

    const SOURCE: &str =
        "[preset00]\nfDecay=0.97\nfZoom=1.02\nper_frame_init_1=q1=0;\nper_frame_1=q1=q1+1;\n[wave_0]\nenabled=1\nsamples=64\nr=0.3\ng=0.7\nb=1\na=0.8";

    #[test]
    fn selected_runtime_executes_init_and_per_frame_equations() {
        let mut runtime = MilkDropSelectedRuntime::from_source(SOURCE).unwrap();
        let frame = runtime.step(1, 0.5, [0.3, 0.6, 0.0]).unwrap();

        assert_eq!(frame.frame_index, 1);
        assert_eq!(frame.state.value("q1"), Some(1.0));
        assert_eq!(frame.decay, 0.97);
        assert_eq!(frame.zoom, 1.02);
        assert!((frame.audio_energy - 0.3).abs() < f32::EPSILON);
        assert_eq!(frame.shader_signal(), [0.5, 0.3, 0.97, 1.02]);
        assert_eq!(frame.custom_waves.len(), 1);
        assert_eq!(frame.custom_waves[0].samples, 64);
        assert!(frame.custom_wave_frames.is_empty());
        assert!(frame.custom_shapes.is_empty());
        assert!(frame.custom_shape_frames.is_empty());
    }

    #[test]
    fn selected_runtime_reset_replays_initialization() {
        let mut runtime = MilkDropSelectedRuntime::from_source(SOURCE).unwrap();
        runtime.step(1, 0.5, [0.0; 3]).unwrap();
        runtime.reset().unwrap();
        let frame = runtime.step(1, 0.25, [0.0; 3]).unwrap();

        assert_eq!(frame.frame_index, 1);
        assert_eq!(frame.state.value("q1"), Some(1.0));
    }

    #[test]
    fn selected_runtime_rejects_execution_controls_outside_the_admitted_range() {
        let mut runtime = MilkDropSelectedRuntime::from_source("[preset00]\nfZoom=8").unwrap();
        assert!(matches!(
            runtime.step(0, 0.0, [0.0; 3]),
            Err(MilkDropSelectedRuntimeError::ControlOutOfRange {
                control: "zoom",
                ..
            })
        ));
    }

    #[test]
    fn selected_runtime_lowers_literal_custom_waves_only_when_audio_is_explicit() {
        let mut runtime = MilkDropSelectedRuntime::from_source(SOURCE).unwrap();
        let frame = runtime
            .step_with_audio(1, 0.5, [0.0; 3], &[-1.0, 1.0], &[0.0, 1.0])
            .unwrap();

        assert_eq!(frame.custom_wave_frames.len(), 1);
        assert_eq!(frame.custom_wave_frames[0].points.len(), 64);
    }

    #[test]
    fn selected_runtime_lowers_literal_custom_shapes_without_audio_input() {
        let source = "[preset00]\n[shape_0]\nenabled=1\nsides=5\nx=0.25\ny=0.75\nrad=0.2\nr=1\ng=0.5\nb=0.25\na=0.8";
        let mut runtime = MilkDropSelectedRuntime::from_source(source).unwrap();
        let frame = runtime.step(1, 0.5, [0.0; 3]).unwrap();

        assert_eq!(frame.custom_shapes.len(), 1);
        assert_eq!(frame.custom_shape_frames.len(), 1);
        assert_eq!(frame.custom_shape_frames[0].points.len(), 5);
        assert!(frame.custom_shape_frames[0]
            .points
            .iter()
            .flatten()
            .all(|value| value.is_finite()));
    }
}
