//! Corpus-local native audio output through CPAL.
//!
//! The callback consumes already-decoded PCM commands. Music parsing,
//! synthesis, gameplay cue selection, and authoritative transport state remain
//! outside this provider.

use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError},
        Arc, Mutex,
    },
    time::Duration,
};

use audio_tools::PcmClip;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    ErrorKind, FromSample, SampleFormat, SizedSample, Stream, StreamConfig,
};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeOutputConfig {
    pub command_capacity: usize,
    pub maximum_voices: usize,
}

impl Default for NativeOutputConfig {
    fn default() -> Self {
        Self {
            command_capacity: 32,
            maximum_voices: 8,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeOutputDescription {
    pub device_name: String,
    pub sample_rate_hz: u32,
    pub channels: u16,
    pub sample_format: String,
    pub buffer_size_frames: Option<u32>,
    pub nominal_buffer_latency_micros: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeOutputObservation {
    pub callback_count: u64,
    pub rendered_frames: u64,
    pub content_starvation_callbacks: u64,
    pub rejected_commands: u64,
    pub device_errors: u64,
    pub xrun_errors: u64,
    pub device_unavailable_errors: u64,
    pub last_device_error: Option<String>,
}

#[derive(Debug, Error)]
pub enum NativeOutputError {
    #[error("native audio command and voice capacities must both be nonzero")]
    ZeroCapacity,
    #[error("no default native audio output device is available")]
    NoDefaultOutputDevice,
    #[error("could not inspect the native output device: {0}")]
    DeviceInspection(String),
    #[error("could not obtain the default native output configuration: {0}")]
    DefaultConfiguration(String),
    #[error("native output sample format {0} is not supported by this corpus provider")]
    UnsupportedSampleFormat(String),
    #[error("could not build the native output stream: {0}")]
    BuildStream(String),
    #[error("could not start native audio output: {0}")]
    StartStream(String),
    #[error("could not pause native audio output: {0}")]
    PauseStream(String),
    #[error("native audio command queue is full; newest command was rejected")]
    CommandQueueFull,
    #[error("native audio command queue is disconnected")]
    CommandQueueDisconnected,
}

#[derive(Debug)]
enum PlaybackCommand {
    StartLoop(Arc<PcmClip>),
    PlayOneShot(Arc<PcmClip>),
    PauseContent,
    ResumeContent,
    StopAll,
}

#[derive(Debug)]
struct PlaybackVoice {
    clip: Arc<PcmClip>,
    source_frame: f64,
    looped: bool,
}

#[derive(Debug)]
struct CallbackState {
    receiver: Receiver<PlaybackCommand>,
    voices: Vec<PlaybackVoice>,
    maximum_voices: usize,
    paused: bool,
    loop_expected: bool,
    observations: Arc<AtomicObservations>,
}

impl CallbackState {
    fn new(
        receiver: Receiver<PlaybackCommand>,
        maximum_voices: usize,
        observations: Arc<AtomicObservations>,
    ) -> Self {
        Self {
            receiver,
            voices: Vec::with_capacity(maximum_voices),
            maximum_voices,
            paused: false,
            loop_expected: false,
            observations,
        }
    }

    fn drain_commands(&mut self) {
        loop {
            match self.receiver.try_recv() {
                Ok(PlaybackCommand::StartLoop(clip)) => {
                    self.voices.retain(|voice| !voice.looped);
                    self.loop_expected = true;
                    self.insert_voice(clip, true);
                }
                Ok(PlaybackCommand::PlayOneShot(clip)) => self.insert_voice(clip, false),
                Ok(PlaybackCommand::PauseContent) => self.paused = true,
                Ok(PlaybackCommand::ResumeContent) => self.paused = false,
                Ok(PlaybackCommand::StopAll) => {
                    self.voices.clear();
                    self.loop_expected = false;
                    self.paused = false;
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }
    }

    fn insert_voice(&mut self, clip: Arc<PcmClip>, looped: bool) {
        if self.voices.len() == self.maximum_voices {
            self.voices.remove(0);
        }
        self.voices.push(PlaybackVoice {
            clip,
            source_frame: 0.0,
            looped,
        });
    }

    fn render(&mut self, output: &mut [f32], output_rate_hz: u32, output_channels: usize) {
        self.drain_commands();
        let output_frames = output.len() / output_channels;
        self.observations
            .rendered_frames
            .fetch_add(output_frames as u64, Ordering::Relaxed);
        output.fill(0.0);
        if self.paused {
            return;
        }
        if self.loop_expected && self.voices.iter().all(|voice| !voice.looped) {
            self.observations
                .content_starvation_callbacks
                .fetch_add(1, Ordering::Relaxed);
        }

        for frame in output.chunks_exact_mut(output_channels) {
            for voice in &mut self.voices {
                mix_voice_frame(voice, frame);
            }
            for sample in frame {
                *sample = sample.clamp(-1.0, 1.0);
            }
            self.voices
                .retain_mut(|voice| advance_voice(voice, output_rate_hz));
        }
    }
}

fn mix_voice_frame(voice: &PlaybackVoice, output: &mut [f32]) {
    let source_channels = usize::from(voice.clip.channels());
    let source_frame = voice.source_frame.floor() as usize;
    if source_frame >= voice.clip.frames() {
        return;
    }
    let samples = voice.clip.interleaved_samples();
    for (channel, target) in output.iter_mut().enumerate() {
        let source_channel = if source_channels == 1 {
            0
        } else {
            channel.min(source_channels - 1)
        };
        *target += samples[source_frame * source_channels + source_channel];
    }
}

fn advance_voice(voice: &mut PlaybackVoice, output_rate_hz: u32) -> bool {
    voice.source_frame += f64::from(voice.clip.sample_rate_hz()) / f64::from(output_rate_hz);
    if voice.source_frame < voice.clip.frames() as f64 {
        true
    } else if voice.looped && voice.clip.frames() != 0 {
        voice.source_frame %= voice.clip.frames() as f64;
        true
    } else {
        false
    }
}

#[derive(Debug, Default)]
struct AtomicObservations {
    callback_count: AtomicU64,
    rendered_frames: AtomicU64,
    content_starvation_callbacks: AtomicU64,
    rejected_commands: AtomicU64,
    device_errors: AtomicU64,
    xrun_errors: AtomicU64,
    device_unavailable_errors: AtomicU64,
    last_device_error: Mutex<Option<String>>,
}

pub struct NativeAudioOutput {
    stream: Stream,
    sender: SyncSender<PlaybackCommand>,
    observations: Arc<AtomicObservations>,
    description: NativeOutputDescription,
}

impl NativeAudioOutput {
    pub fn open_default(config: NativeOutputConfig) -> Result<Self, NativeOutputError> {
        if config.command_capacity == 0 || config.maximum_voices == 0 {
            return Err(NativeOutputError::ZeroCapacity);
        }
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(NativeOutputError::NoDefaultOutputDevice)?;
        let device_name = device
            .description()
            .map_err(|error| NativeOutputError::DeviceInspection(error.to_string()))?
            .name()
            .to_owned();
        let supported = device
            .default_output_config()
            .map_err(|error| NativeOutputError::DefaultConfiguration(error.to_string()))?;
        let sample_format = supported.sample_format();
        let stream_config: StreamConfig = supported.config();
        let observations = Arc::new(AtomicObservations::default());
        let (sender, receiver) = mpsc::sync_channel(config.command_capacity);
        let state = CallbackState::new(receiver, config.maximum_voices, observations.clone());
        let stream = match sample_format {
            SampleFormat::F32 => build_stream::<f32>(&device, &stream_config, state)?,
            SampleFormat::I16 => build_stream::<i16>(&device, &stream_config, state)?,
            SampleFormat::U16 => build_stream::<u16>(&device, &stream_config, state)?,
            format => {
                return Err(NativeOutputError::UnsupportedSampleFormat(
                    format.to_string(),
                ))
            }
        };
        let buffer_size_frames = stream.buffer_size().ok();
        let nominal_buffer_latency_micros = buffer_size_frames
            .map(|frames| u64::from(frames) * 1_000_000 / u64::from(stream_config.sample_rate));
        let description = NativeOutputDescription {
            device_name,
            sample_rate_hz: stream_config.sample_rate,
            channels: stream_config.channels,
            sample_format: sample_format.to_string(),
            buffer_size_frames,
            nominal_buffer_latency_micros,
        };
        Ok(Self {
            stream,
            sender,
            observations,
            description,
        })
    }

    pub fn description(&self) -> &NativeOutputDescription {
        &self.description
    }

    pub fn play(&self) -> Result<(), NativeOutputError> {
        self.stream
            .play()
            .map_err(|error| NativeOutputError::StartStream(error.to_string()))
    }

    pub fn pause_device(&self) -> Result<(), NativeOutputError> {
        self.stream
            .pause()
            .map_err(|error| NativeOutputError::PauseStream(error.to_string()))
    }

    pub fn start_loop(&self, clip: Arc<PcmClip>) -> Result<(), NativeOutputError> {
        self.send(PlaybackCommand::StartLoop(clip))
    }

    pub fn play_one_shot(&self, clip: Arc<PcmClip>) -> Result<(), NativeOutputError> {
        self.send(PlaybackCommand::PlayOneShot(clip))
    }

    pub fn pause_content(&self) -> Result<(), NativeOutputError> {
        self.send(PlaybackCommand::PauseContent)
    }

    pub fn resume_content(&self) -> Result<(), NativeOutputError> {
        self.send(PlaybackCommand::ResumeContent)
    }

    pub fn stop(&self) -> Result<(), NativeOutputError> {
        self.send(PlaybackCommand::StopAll)
    }

    pub fn observe(&self) -> NativeOutputObservation {
        NativeOutputObservation {
            callback_count: self.observations.callback_count.load(Ordering::Relaxed),
            rendered_frames: self.observations.rendered_frames.load(Ordering::Relaxed),
            content_starvation_callbacks: self
                .observations
                .content_starvation_callbacks
                .load(Ordering::Relaxed),
            rejected_commands: self.observations.rejected_commands.load(Ordering::Relaxed),
            device_errors: self.observations.device_errors.load(Ordering::Relaxed),
            xrun_errors: self.observations.xrun_errors.load(Ordering::Relaxed),
            device_unavailable_errors: self
                .observations
                .device_unavailable_errors
                .load(Ordering::Relaxed),
            last_device_error: self
                .observations
                .last_device_error
                .lock()
                .expect("device error mutex is not poisoned")
                .clone(),
        }
    }

    fn send(&self, command: PlaybackCommand) -> Result<(), NativeOutputError> {
        match self.sender.try_send(command) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => {
                self.observations
                    .rejected_commands
                    .fetch_add(1, Ordering::Relaxed);
                Err(NativeOutputError::CommandQueueFull)
            }
            Err(TrySendError::Disconnected(_)) => Err(NativeOutputError::CommandQueueDisconnected),
        }
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    mut state: CallbackState,
) -> Result<Stream, NativeOutputError>
where
    T: SizedSample + FromSample<f32>,
{
    let observations = state.observations.clone();
    let sample_rate_hz = config.sample_rate;
    let channels = usize::from(config.channels);
    device
        .build_output_stream(
            *config,
            move |output: &mut [T], _| {
                // One scratch allocation here would violate the warm-callback
                // criterion, so conversion is performed through a fixed stack
                // chunk. CPAL callback sizes may vary.
                const SCRATCH_SAMPLES: usize = 16_384;
                let mut scratch = [0.0_f32; SCRATCH_SAMPLES];
                state
                    .observations
                    .callback_count
                    .fetch_add(1, Ordering::Relaxed);
                let aligned_chunk_samples = SCRATCH_SAMPLES - (SCRATCH_SAMPLES % channels);
                for chunk in output.chunks_mut(aligned_chunk_samples) {
                    let scratch = &mut scratch[..chunk.len()];
                    state.render(scratch, sample_rate_hz, channels);
                    for (target, sample) in chunk.iter_mut().zip(scratch.iter().copied()) {
                        *target = T::from_sample(sample);
                    }
                }
            },
            move |error| {
                observations.device_errors.fetch_add(1, Ordering::Relaxed);
                match error.kind() {
                    ErrorKind::Xrun => {
                        observations.xrun_errors.fetch_add(1, Ordering::Relaxed);
                    }
                    ErrorKind::DeviceNotAvailable | ErrorKind::StreamInvalidated => {
                        observations
                            .device_unavailable_errors
                            .fetch_add(1, Ordering::Relaxed);
                    }
                    _ => {}
                }
                if let Ok(mut last_error) = observations.last_device_error.lock() {
                    *last_error = Some(error.to_string());
                }
            },
            Some(Duration::from_millis(100)),
        )
        .map_err(|error| NativeOutputError::BuildStream(error.to_string()))
}

#[cfg(test)]
mod tests {
    use audio_tools::{PcmClip, PcmClipLimits};

    use super::*;

    fn clip(samples: &[f32]) -> Arc<PcmClip> {
        Arc::new(
            PcmClip::new(
                8_000,
                1,
                samples.to_vec(),
                PcmClipLimits {
                    maximum_frames: samples.len(),
                    maximum_channels: 1,
                    maximum_sample_rate_hz: 8_000,
                },
            )
            .unwrap(),
        )
    }

    #[test]
    fn headless_callback_path_loops_and_mixes_one_shot_without_allocation_growth() {
        let observations = Arc::new(AtomicObservations::default());
        let (sender, receiver) = mpsc::sync_channel(4);
        let mut state = CallbackState::new(receiver, 2, observations);
        sender
            .send(PlaybackCommand::StartLoop(clip(&[0.25, 0.5])))
            .unwrap();
        sender
            .send(PlaybackCommand::PlayOneShot(clip(&[0.5])))
            .unwrap();
        let initial_capacity = state.voices.capacity();
        let mut output = [0.0; 8];
        state.render(&mut output, 8_000, 2);
        assert_eq!(output, [0.75, 0.75, 0.5, 0.5, 0.25, 0.25, 0.5, 0.5]);
        assert_eq!(state.voices.capacity(), initial_capacity);
        assert_eq!(state.voices.len(), 1);
    }

    #[test]
    fn bounded_queue_rejects_newest_command_explicitly() {
        let observations = Arc::new(AtomicObservations::default());
        let (sender, _receiver) = mpsc::sync_channel(1);
        let output = NativeAudioOutputCommandTest {
            sender,
            observations: observations.clone(),
        };
        output.send(PlaybackCommand::PauseContent).unwrap();
        assert!(matches!(
            output.send(PlaybackCommand::ResumeContent),
            Err(NativeOutputError::CommandQueueFull)
        ));
        assert_eq!(observations.rejected_commands.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn sample_rate_adaptation_repeats_source_frames_at_double_output_rate() {
        let observations = Arc::new(AtomicObservations::default());
        let (sender, receiver) = mpsc::sync_channel(1);
        let mut state = CallbackState::new(receiver, 1, observations);
        let source = Arc::new(
            PcmClip::new(
                4_000,
                1,
                vec![0.25, 0.75],
                PcmClipLimits {
                    maximum_frames: 2,
                    maximum_channels: 1,
                    maximum_sample_rate_hz: 4_000,
                },
            )
            .unwrap(),
        );
        sender.send(PlaybackCommand::StartLoop(source)).unwrap();
        let mut output = [0.0; 4];
        state.render(&mut output, 8_000, 1);
        assert_eq!(output, [0.25, 0.25, 0.75, 0.75]);
    }

    struct NativeAudioOutputCommandTest {
        sender: SyncSender<PlaybackCommand>,
        observations: Arc<AtomicObservations>,
    }

    impl NativeAudioOutputCommandTest {
        fn send(&self, command: PlaybackCommand) -> Result<(), NativeOutputError> {
            match self.sender.try_send(command) {
                Ok(()) => Ok(()),
                Err(TrySendError::Full(_)) => {
                    self.observations
                        .rejected_commands
                        .fetch_add(1, Ordering::Relaxed);
                    Err(NativeOutputError::CommandQueueFull)
                }
                Err(TrySendError::Disconnected(_)) => {
                    Err(NativeOutputError::CommandQueueDisconnected)
                }
            }
        }
    }
}
