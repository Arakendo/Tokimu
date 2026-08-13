use std::{collections::BTreeMap, path::PathBuf};

use gltf_corpus::{decode_glb, decode_glb_file, DecodedAnimation};
use serde::{Deserialize, Serialize};

use crate::{ObservationDiagnostic, Position};

const FIXED_STEP_SECONDS: f32 = 1.0 / 60.0;
const HOLE_PUNCH_SOURCE: &str = "corpus/assets/CheckLicense/hole_punch1.glb";

/// Provider-neutral evidence for one bounded translation clip.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AnimationClipObservation {
    pub id: usize,
    pub name: String,
    pub duration_seconds: f32,
    pub translation_channels: usize,
    pub animated_nodes: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackMode {
    Stopped,
    Playing,
    Paused,
    Completed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PlaybackPolicy {
    /// The source clips are assembly steps. This remains application policy,
    /// rather than an importer behavior inferred from their names.
    pub hold_completed_steps: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlaybackState {
    pub selected_clip: usize,
    pub mode: PlaybackMode,
    pub local_time_seconds: f32,
    pub speed: f32,
    pub looping: bool,
    pub policy: PlaybackPolicy,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum PlaybackCommand {
    Play { clip: usize },
    Pause,
    Resume,
    Stop,
    Seek { seconds: f32 },
    SetSpeed { speed: f32 },
    SetLooping { looping: bool },
    NextStep,
    Reset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackDisposition {
    Accepted,
    RejectedUnknownClip,
    RejectedInvalidTime,
    RejectedInvalidSpeed,
    RejectedUnsupported,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlaybackCommandResult {
    pub disposition: PlaybackDisposition,
    pub state: PlaybackState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<ObservationDiagnostic>,
}

pub fn load_hole_punch_catalog() -> Result<Vec<AnimationClipObservation>, String> {
    let model = decode_glb_file(hole_punch_path()).map_err(|error| error.to_string())?;
    catalog_from_animations(&model.animations)
}

/// A checked provider-neutral observation fixture for consumers that cannot
/// compile the optional meshopt decoder yet (including the current WASM
/// toolchain). Native tests compare this record with the source GLB decode.
///
/// This is not a substitute for importing GLB bytes. It isolates runtime
/// observation and playback-contract validation from that separate provider
/// capability while preserving the exact observed catalog.
pub fn verified_hole_punch_catalog_fixture() -> Vec<AnimationClipObservation> {
    vec![
        AnimationClipObservation {
            id: 0,
            name: "step1".to_owned(),
            duration_seconds: 2.0,
            translation_channels: 1,
            animated_nodes: vec![25],
        },
        AnimationClipObservation {
            id: 1,
            name: "step2".to_owned(),
            duration_seconds: 2.0,
            translation_channels: 2,
            animated_nodes: vec![21, 23],
        },
        AnimationClipObservation {
            id: 2,
            name: "step3".to_owned(),
            duration_seconds: 2.0,
            translation_channels: 1,
            animated_nodes: vec![19],
        },
        AnimationClipObservation {
            id: 3,
            name: "step4".to_owned(),
            duration_seconds: 2.0,
            translation_channels: 1,
            animated_nodes: vec![17],
        },
        AnimationClipObservation {
            id: 4,
            name: "step5".to_owned(),
            duration_seconds: 2.0,
            translation_channels: 1,
            animated_nodes: vec![11],
        },
    ]
}

/// Decode a bounded catalog from caller-provided GLB bytes.
///
/// Fixture acquisition remains outside this seam, allowing a WASM consumer to
/// embed known source bytes without moving GLB parsing into TypeScript.
pub fn catalog_from_glb_bytes(bytes: &[u8]) -> Result<Vec<AnimationClipObservation>, String> {
    let model = decode_glb(bytes).map_err(|error| error.to_string())?;
    catalog_from_animations(&model.animations)
}

fn catalog_from_animations(
    animations: &[DecodedAnimation],
) -> Result<Vec<AnimationClipObservation>, String> {
    let catalog = animations
        .iter()
        .enumerate()
        .map(|(id, animation)| AnimationClipObservation {
            id,
            name: animation
                .name
                .clone()
                .unwrap_or_else(|| format!("clip-{id}")),
            duration_seconds: animation_duration(animation),
            translation_channels: animation.channels.len(),
            animated_nodes: animation
                .channels
                .iter()
                .map(|channel| channel.node)
                .collect(),
        })
        .collect::<Vec<_>>();
    if catalog.is_empty() {
        return Err("hole_punch1.glb exposes no admitted translation clips".to_owned());
    }
    Ok(catalog)
}

impl PlaybackState {
    pub fn initial(policy: PlaybackPolicy) -> Self {
        Self {
            selected_clip: 0,
            mode: PlaybackMode::Stopped,
            local_time_seconds: 0.0,
            speed: 1.0,
            looping: false,
            policy,
        }
    }

    /// Apply a provider-neutral command. A rejected command retains an exact
    /// clone of the prior state.
    pub fn apply_command(
        &mut self,
        catalog: &[AnimationClipObservation],
        command: PlaybackCommand,
    ) -> PlaybackCommandResult {
        let before = self.clone();
        let outcome = match command {
            PlaybackCommand::Play { clip } => {
                if catalog.get(clip).is_none() {
                    Err((
                        PlaybackDisposition::RejectedUnknownClip,
                        "unknown_animation_clip",
                        format!("clip {clip} is outside the catalog"),
                    ))
                } else {
                    self.selected_clip = clip;
                    self.local_time_seconds = 0.0;
                    self.mode = PlaybackMode::Playing;
                    Ok(())
                }
            }
            PlaybackCommand::Pause if self.mode == PlaybackMode::Playing => {
                self.mode = PlaybackMode::Paused;
                Ok(())
            }
            PlaybackCommand::Pause => Err((
                PlaybackDisposition::RejectedUnsupported,
                "pause_not_playing",
                "pause requires an actively playing clip".to_owned(),
            )),
            PlaybackCommand::Resume if self.mode == PlaybackMode::Paused => {
                self.mode = PlaybackMode::Playing;
                Ok(())
            }
            PlaybackCommand::Resume => Err((
                PlaybackDisposition::RejectedUnsupported,
                "resume_not_paused",
                "resume requires a paused clip".to_owned(),
            )),
            PlaybackCommand::Stop => {
                self.mode = PlaybackMode::Stopped;
                self.local_time_seconds = 0.0;
                Ok(())
            }
            PlaybackCommand::Seek { seconds } => {
                let Some(clip) = catalog.get(self.selected_clip) else {
                    return rejected_playback(
                        before,
                        PlaybackDisposition::RejectedUnknownClip,
                        "unknown_animation_clip",
                        "selected clip is outside the catalog".to_owned(),
                    );
                };
                if !seconds.is_finite() || !(0.0..=clip.duration_seconds).contains(&seconds) {
                    Err((
                        PlaybackDisposition::RejectedInvalidTime,
                        "invalid_animation_seek",
                        format!(
                            "seek time {seconds:?} is outside 0..={} for clip {}",
                            clip.duration_seconds, clip.id
                        ),
                    ))
                } else {
                    self.local_time_seconds = seconds;
                    Ok(())
                }
            }
            PlaybackCommand::SetSpeed { speed } if speed.is_finite() && speed > 0.0 => {
                self.speed = speed;
                Ok(())
            }
            PlaybackCommand::SetSpeed { speed } => Err((
                PlaybackDisposition::RejectedInvalidSpeed,
                "invalid_animation_speed",
                format!("playback speed must be finite and positive, received {speed:?}"),
            )),
            PlaybackCommand::SetLooping { looping } => {
                self.looping = looping;
                Ok(())
            }
            PlaybackCommand::NextStep => {
                if self.selected_clip + 1 >= catalog.len() {
                    Err((
                        PlaybackDisposition::RejectedUnsupported,
                        "next_step_unavailable",
                        "the selected clip is already the final assembly step".to_owned(),
                    ))
                } else {
                    self.selected_clip += 1;
                    self.local_time_seconds = 0.0;
                    self.mode = PlaybackMode::Paused;
                    Ok(())
                }
            }
            PlaybackCommand::Reset => {
                self.selected_clip = 0;
                self.local_time_seconds = 0.0;
                self.mode = PlaybackMode::Stopped;
                Ok(())
            }
        };

        match outcome {
            Ok(()) => PlaybackCommandResult {
                disposition: PlaybackDisposition::Accepted,
                state: self.clone(),
                diagnostic: None,
            },
            Err((disposition, code, message)) => {
                rejected_playback(before, disposition, code, message)
            }
        }
    }

    pub fn advance_fixed_step(&mut self, catalog: &[AnimationClipObservation]) {
        if self.mode != PlaybackMode::Playing {
            return;
        }
        let Some(clip) = catalog.get(self.selected_clip) else {
            self.mode = PlaybackMode::Stopped;
            return;
        };
        let duration = clip.duration_seconds.max(FIXED_STEP_SECONDS);
        self.local_time_seconds += FIXED_STEP_SECONDS * self.speed;
        if self.local_time_seconds < duration {
            return;
        }
        if self.looping {
            self.local_time_seconds = self.local_time_seconds.rem_euclid(duration);
        } else {
            self.local_time_seconds = duration;
            self.mode = PlaybackMode::Completed;
        }
    }
}

/// Sample only the current provider-neutral translation evidence. The
/// application chooses whether already-completed assembly clips remain held.
pub fn sample_hole_punch_translations(
    state: &PlaybackState,
) -> Result<BTreeMap<usize, Position>, String> {
    let model = decode_glb_file(hole_punch_path()).map_err(|error| error.to_string())?;
    let Some(current) = model.animations.get(state.selected_clip) else {
        return Err(format!(
            "selected clip {} is unavailable",
            state.selected_clip
        ));
    };
    let mut translations = BTreeMap::new();
    if state.policy.hold_completed_steps {
        for animation in model.animations.iter().take(state.selected_clip) {
            insert_final_translations(&mut translations, animation);
        }
    }
    insert_sampled_translations(&mut translations, current, state.local_time_seconds);
    Ok(translations)
}

fn rejected_playback(
    state: PlaybackState,
    disposition: PlaybackDisposition,
    code: &'static str,
    message: String,
) -> PlaybackCommandResult {
    PlaybackCommandResult {
        disposition,
        state,
        diagnostic: Some(ObservationDiagnostic {
            code,
            owner: "application_playback_adapter",
            message,
        }),
    }
}

fn insert_final_translations(output: &mut BTreeMap<usize, Position>, animation: &DecodedAnimation) {
    for channel in &animation.channels {
        if let Some(translation) = channel.translations.last() {
            output.insert(channel.node, position(*translation));
        }
    }
}

fn insert_sampled_translations(
    output: &mut BTreeMap<usize, Position>,
    animation: &DecodedAnimation,
    time: f32,
) {
    for channel in &animation.channels {
        if let Some(translation) = sample_translation(&channel.times, &channel.translations, time) {
            output.insert(channel.node, position(translation));
        }
    }
}

fn sample_translation(times: &[f32], translations: &[[f32; 3]], time: f32) -> Option<[f32; 3]> {
    let first = *translations.first()?;
    if time <= *times.first()? {
        return Some(first);
    }
    for index in 1..times.len().min(translations.len()) {
        if time <= times[index] {
            let start = times[index - 1];
            let duration = times[index] - start;
            if duration <= 0.0 {
                return Some(translations[index]);
            }
            let amount = ((time - start) / duration).clamp(0.0, 1.0);
            return Some(interpolate(
                translations[index - 1],
                translations[index],
                amount,
            ));
        }
    }
    translations.last().copied()
}

fn interpolate(from: [f32; 3], to: [f32; 3], amount: f32) -> [f32; 3] {
    [
        from[0] + (to[0] - from[0]) * amount,
        from[1] + (to[1] - from[1]) * amount,
        from[2] + (to[2] - from[2]) * amount,
    ]
}

fn position(value: [f32; 3]) -> Position {
    Position {
        x: value[0],
        y: value[1],
        z: value[2],
    }
}

fn animation_duration(animation: &DecodedAnimation) -> f32 {
    animation
        .channels
        .iter()
        .filter_map(|channel| channel.times.last().copied())
        .fold(0.0, f32::max)
}

fn hole_punch_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../..")
        .join(HOLE_PUNCH_SOURCE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hole_punch_catalog_is_named_and_deterministic() {
        let first = load_hole_punch_catalog().unwrap();
        let second = load_hole_punch_catalog().unwrap();
        assert_eq!(first, second);
        assert_eq!(
            first
                .iter()
                .map(|clip| clip.name.as_str())
                .collect::<Vec<_>>(),
            ["step1", "step2", "step3", "step4", "step5"]
        );
        assert_eq!(first, verified_hole_punch_catalog_fixture());
    }

    #[test]
    fn pause_seek_reset_and_fixed_steps_are_explicit() {
        let catalog = load_hole_punch_catalog().unwrap();
        let mut state = PlaybackState::initial(PlaybackPolicy {
            hold_completed_steps: true,
        });
        assert_eq!(
            state
                .apply_command(&catalog, PlaybackCommand::Play { clip: 0 })
                .disposition,
            PlaybackDisposition::Accepted
        );
        state.advance_fixed_step(&catalog);
        let after_step = state.local_time_seconds;
        state.apply_command(&catalog, PlaybackCommand::Pause);
        state.advance_fixed_step(&catalog);
        assert_eq!(state.local_time_seconds, after_step);
        assert_eq!(
            state
                .apply_command(&catalog, PlaybackCommand::Seek { seconds: 0.0 })
                .disposition,
            PlaybackDisposition::Accepted
        );
        let first_sample = sample_hole_punch_translations(&state).unwrap();
        assert!(!first_sample.is_empty());
        state.apply_command(&catalog, PlaybackCommand::Reset);
        assert_eq!(state.mode, PlaybackMode::Stopped);
        assert_eq!(state.selected_clip, 0);
        assert_eq!(state.local_time_seconds, 0.0);
    }

    #[test]
    fn speed_loop_and_step_navigation_are_bounded_commands() {
        let catalog = load_hole_punch_catalog().unwrap();
        let mut state = PlaybackState::initial(PlaybackPolicy {
            hold_completed_steps: true,
        });
        assert_eq!(
            state
                .apply_command(&catalog, PlaybackCommand::SetSpeed { speed: 2.0 })
                .disposition,
            PlaybackDisposition::Accepted
        );
        assert_eq!(
            state
                .apply_command(&catalog, PlaybackCommand::SetLooping { looping: true })
                .disposition,
            PlaybackDisposition::Accepted
        );
        assert_eq!(
            state
                .apply_command(&catalog, PlaybackCommand::NextStep)
                .disposition,
            PlaybackDisposition::Accepted
        );
        assert_eq!(state.selected_clip, 1);
        state.selected_clip = catalog.len() - 1;
        let result = state.apply_command(&catalog, PlaybackCommand::NextStep);
        assert_eq!(result.disposition, PlaybackDisposition::RejectedUnsupported);
    }

    #[test]
    fn playback_rejections_retain_the_prior_state() {
        let catalog = load_hole_punch_catalog().unwrap();
        let mut state = PlaybackState::initial(PlaybackPolicy {
            hold_completed_steps: false,
        });
        let before = state.clone();
        let result = state.apply_command(&catalog, PlaybackCommand::SetSpeed { speed: 0.0 });
        assert_eq!(
            result.disposition,
            PlaybackDisposition::RejectedInvalidSpeed
        );
        assert_eq!(state, before);
        let result = state.apply_command(&catalog, PlaybackCommand::Play { clip: 99 });
        assert_eq!(result.disposition, PlaybackDisposition::RejectedUnknownClip);
        assert_eq!(state, before);
    }

    #[test]
    fn hold_policy_is_explicit_in_the_sampled_state() {
        let catalog = load_hole_punch_catalog().unwrap();
        let mut state = PlaybackState::initial(PlaybackPolicy {
            hold_completed_steps: true,
        });
        state.apply_command(&catalog, PlaybackCommand::Play { clip: 1 });
        let held = sample_hole_punch_translations(&state).unwrap();
        state.policy.hold_completed_steps = false;
        let unheld = sample_hole_punch_translations(&state).unwrap();
        assert!(held.len() >= unheld.len());
        assert_ne!(held, unheld);
    }
}
