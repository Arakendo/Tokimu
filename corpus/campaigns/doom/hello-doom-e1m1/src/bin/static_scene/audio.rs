//! Opt-in Doom application audio composition.
//!
//! Source decoding and synthesis happen before the device opens. The live
//! object owns only resolved clips, application cue dispatch, and the
//! corpus-local native output provider.

use std::{collections::BTreeMap, sync::Arc};

use audio_tools::{PcmClip, PcmClipLimits, SoundEmission};
use cpal_audio_output_provider::{NativeAudioOutput, NativeOutputConfig};
use doom_audio_provider::{
    decode_doom_mus_score, decode_doom_sound_effect, DoomMusDecodeLimits, DoomSoundDecodeLimits,
};
use doom_wad_provider::WadManifest;
use hello_doom_e1m1::sound::{
    doom_sound_lump_for_clip, request_doom_sound, DoomGameplaySoundEvent,
};
use simple_audio_synth_provider::{synthesize_sequence, SimpleSynthConfig};

#[derive(Clone)]
pub(crate) struct DoomLiveAudioAssets {
    music: Arc<PcmClip>,
    music_lump: String,
    clips: BTreeMap<String, Arc<PcmClip>>,
    unavailable_cues: Vec<String>,
}

pub(crate) fn prepare_doom_live_audio_assets(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    map_name: &str,
) -> Result<DoomLiveAudioAssets, String> {
    let music_lump = format!("D_{map_name}");
    let score = decode_doom_mus_score(
        wad_bytes,
        manifest,
        &music_lump,
        DoomMusDecodeLimits {
            maximum_score_bytes: 1024 * 1024,
            maximum_events: 100_000,
            maximum_duration_units: 140 * 60 * 30,
            maximum_instruments: 256,
        },
    )
    .map_err(|error| error.to_string())?;
    let render_time_units = score.sequence.duration_units().min(140 * 30);
    let maximum_frames = usize::try_from(
        u128::from(render_time_units)
            .saturating_mul(22_050)
            .div_ceil(u128::from(score.sequence.timebase().units_per_second())),
    )
    .unwrap_or(usize::MAX);
    let music = synthesize_sequence(
        &score.sequence,
        SimpleSynthConfig {
            sample_rate_hz: 22_050,
            render_time_units,
            maximum_frames,
            maximum_voices: 64,
            master_gain: 0.08,
        },
    )
    .map_err(|error| error.to_string())?;

    let mut clips = BTreeMap::new();
    let mut unavailable_cues = Vec::new();
    for clip_key in ["weapon.pistol", "monster.alert.zombieman"] {
        let Some(lump) = doom_sound_lump_for_clip(clip_key) else {
            unavailable_cues.push(format!("{clip_key}:no-source-mapping"));
            continue;
        };
        match decode_doom_sound_effect(
            wad_bytes,
            manifest,
            lump,
            DoomSoundDecodeLimits {
                maximum_samples: 1_000_000,
                maximum_sample_rate_hz: 48_000,
            },
        ) {
            Ok(sound) => match sound.to_pcm_clip(PcmClipLimits {
                maximum_frames: 1_000_000,
                maximum_channels: 2,
                maximum_sample_rate_hz: 48_000,
            }) {
                Ok(clip) => {
                    clips.insert(clip_key.to_owned(), Arc::new(clip));
                }
                Err(error) => unavailable_cues.push(format!("{clip_key}:{error}")),
            },
            Err(error) => unavailable_cues.push(format!("{clip_key}:{error}")),
        }
    }
    Ok(DoomLiveAudioAssets {
        music: Arc::new(music.clip),
        music_lump,
        clips,
        unavailable_cues,
    })
}

pub(crate) struct DoomLiveAudio {
    output: NativeAudioOutput,
    clips: BTreeMap<String, Arc<PcmClip>>,
    submitted_cues: u64,
    unavailable_cues: u64,
    observed_device_errors: u64,
    observed_starvation_callbacks: u64,
}

impl DoomLiveAudio {
    pub(crate) fn open(assets: DoomLiveAudioAssets) -> Result<(Self, String), String> {
        let output = NativeAudioOutput::open_default(NativeOutputConfig::default())
            .map_err(|error| error.to_string())?;
        output.play().map_err(|error| error.to_string())?;
        output
            .start_loop(assets.music)
            .map_err(|error| error.to_string())?;
        let description = output.description();
        let diagnostic = format!(
            "audio=enabled; provider=cpal-corpus-local; device={}; sample-rate-hz={}; channels={}; buffer-size-frames={:?}; nominal-buffer-latency-us={:?}; music={}; music-policy=bounded-30-second-preview-loop; resolved-cues={}; unavailable-cues={:?}; spatialization=not-implemented-listener-relative-mix",
            description.device_name,
            description.sample_rate_hz,
            description.channels,
            description.buffer_size_frames,
            description.nominal_buffer_latency_micros,
            assets.music_lump,
            assets.clips.len(),
            assets.unavailable_cues,
        );
        Ok((
            Self {
                output,
                clips: assets.clips,
                submitted_cues: 0,
                unavailable_cues: 0,
                observed_device_errors: 0,
                observed_starvation_callbacks: 0,
            },
            diagnostic,
        ))
    }

    pub(crate) fn emit(&mut self, event: DoomGameplaySoundEvent) -> Result<String, String> {
        let request = request_doom_sound(event).map_err(|error| error.to_string())?;
        let emission = match request.emission {
            SoundEmission::ListenerRelative => "listener-relative".to_owned(),
            SoundEmission::Spatial { position } => {
                format!("spatial-request={position:?}; realized=listener-relative")
            }
        };
        let Some(clip) = self.clips.get(request.clip.as_str()) else {
            self.unavailable_cues = self.unavailable_cues.saturating_add(1);
            return Err(format!(
                "audio cue unavailable: clip={}; emission={emission}",
                request.clip.as_str()
            ));
        };
        self.output
            .play_one_shot(clip.clone())
            .map_err(|error| error.to_string())?;
        self.submitted_cues = self.submitted_cues.saturating_add(1);
        Ok(format!(
            "audio cue: clip={}; emission={emission}; submitted-cues={}",
            request.clip.as_str(),
            self.submitted_cues
        ))
    }

    pub(crate) fn poll_diagnostic(&mut self) -> Option<String> {
        let observation = self.output.observe();
        if observation.device_errors > self.observed_device_errors {
            self.observed_device_errors = observation.device_errors;
            return Some(format!(
                "audio device observation: errors={}; xrun-errors={}; device-unavailable-errors={}; last-error={:?}; gameplay-continues=true",
                observation.device_errors,
                observation.xrun_errors,
                observation.device_unavailable_errors,
                observation.last_device_error,
            ));
        }
        if observation.content_starvation_callbacks > self.observed_starvation_callbacks {
            self.observed_starvation_callbacks = observation.content_starvation_callbacks;
            return Some(format!(
                "audio content starvation: callbacks={}; gameplay-continues=true",
                observation.content_starvation_callbacks,
            ));
        }
        None
    }
}
