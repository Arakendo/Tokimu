//! Audible corpus proof for Doom music and an independent sound-effect cue.

use std::{env, fs, process::ExitCode, sync::Arc, thread, time::Duration};

use archive_provider::{ArchiveFormat, ArchiveReadLimits, ZipArchiveProvider};
use audio_tools::{
    NoteSequence, NoteSequenceLimits, PcmClipLimits, SequenceEvent, SequenceEventKind,
    SequenceTimebase,
};
use cpal_audio_output_provider::{NativeAudioOutput, NativeOutputConfig};
use doom_audio_provider::{
    decode_doom_mus_score, decode_doom_sound_effect, DoomMusDecodeLimits, DoomSoundDecodeLimits,
};
use doom_wad_package::{read_wad_package_member, InspectWadPackageRequest};
use doom_wad_provider::WadReadLimits;
use resource_space::{
    AddressCasePolicy, FolderId, InMemoryResourceSpace, ResourceMetadata, ResourceName,
    ResourceRootDescriptor, ResourceRootId, StoreId,
};
use resource_space_archive::InspectArchiveResourceRequest;
use simple_audio_synth_provider::{synthesize_sequence, SimpleSynthConfig};

const WAD_LIMITS: WadReadLimits =
    WadReadLimits::new(64 * 1024 * 1024, 8_192, 16 * 1024 * 1024, 64 * 1024 * 1024);

fn main() -> ExitCode {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let [package, member] = args.as_slice() else {
        eprintln!(
            "usage: doom_audio_playback <canonical-doom-zip> <WAD-member-name>\n\
             plays a short looping D_E1M1 preview, triggers DSPISTOL, then exercises pause/resume/stop"
        );
        return ExitCode::from(2);
    };
    match run(package, member) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Doom audio playback failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(package: &str, member: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (wad_bytes, manifest) = load_wad(package, member)?;
    let score = decode_doom_mus_score(
        &wad_bytes,
        &manifest,
        "D_E1M1",
        DoomMusDecodeLimits {
            maximum_score_bytes: 1024 * 1024,
            maximum_events: 100_000,
            maximum_duration_units: 140 * 60 * 30,
            maximum_instruments: 256,
        },
    )?;
    let music = synthesize_sequence(
        &score.sequence,
        SimpleSynthConfig {
            sample_rate_hz: 22_050,
            render_time_units: 140 * 5,
            maximum_frames: 22_050 * 5,
            maximum_voices: 64,
            master_gain: 0.08,
        },
    )?;
    let pistol = decode_doom_sound_effect(
        &wad_bytes,
        &manifest,
        "DSPISTOL",
        DoomSoundDecodeLimits {
            maximum_samples: 1_000_000,
            maximum_sample_rate_hz: 48_000,
        },
    )?
    .to_pcm_clip(PcmClipLimits {
        maximum_frames: 1_000_000,
        maximum_channels: 2,
        maximum_sample_rate_hz: 48_000,
    })?;
    let note_cue_sequence = NoteSequence::new(
        SequenceTimebase::new(140, 140)?,
        1,
        35,
        vec![
            SequenceEvent {
                time_units: 0,
                order: 0,
                channel: 0,
                kind: SequenceEventKind::NoteOn {
                    note: 69,
                    velocity: 110,
                },
            },
            SequenceEvent {
                time_units: 35,
                order: 1,
                channel: 0,
                kind: SequenceEventKind::NoteOff { note: 69 },
            },
        ],
        NoteSequenceLimits {
            maximum_events: 2,
            maximum_channels: 1,
            maximum_time_units: 35,
            maximum_units_per_second: 140,
        },
    )?;
    let note_cue = synthesize_sequence(
        &note_cue_sequence,
        SimpleSynthConfig {
            sample_rate_hz: 22_050,
            render_time_units: 35,
            maximum_frames: 5_513,
            maximum_voices: 1,
            master_gain: 0.15,
        },
    )?;

    let session = DoomAudioSession::open(
        Arc::new(music.clip),
        Arc::new(pistol),
        Arc::new(note_cue.clip),
    )?;
    println!(
        "Doom native audio opened: device={}; sample-rate-hz={}; channels={}; sample-format={}; buffer-size-frames={:?}; nominal-buffer-latency-us={:?}; provider=cpal-corpus-local; source={member}; music=D_E1M1; cues=DSPISTOL+authored-note-69",
        session.output.description().device_name,
        session.output.description().sample_rate_hz,
        session.output.description().channels,
        session.output.description().sample_format,
        session.output.description().buffer_size_frames,
        session.output.description().nominal_buffer_latency_micros,
    );
    session.start_music()?;
    thread::sleep(Duration::from_millis(800));
    session.trigger_pistol()?;
    thread::sleep(Duration::from_millis(300));
    session.trigger_note_cue()?;
    thread::sleep(Duration::from_millis(300));
    session.pause()?;
    thread::sleep(Duration::from_millis(250));
    session.resume()?;
    thread::sleep(Duration::from_millis(600));
    session.stop()?;
    thread::sleep(Duration::from_millis(100));
    session.output.pause_device()?;
    let observation = session.output.observe();
    println!(
        "Doom native audio observation: lifecycle=start-music>one-shot-pistol>one-shot-note-cue>pause>resume>stop; callbacks={}; rendered-frames={}; content-starvation-callbacks={}; rejected-commands={}; device-errors={}; xrun-errors={}; device-unavailable-errors={}; last-device-error={:?}; device-handle-exposed=false; callback-midi-parsing=false; callback-application-policy=false",
        observation.callback_count,
        observation.rendered_frames,
        observation.content_starvation_callbacks,
        observation.rejected_commands,
        observation.device_errors,
        observation.xrun_errors,
        observation.device_unavailable_errors,
        observation.last_device_error,
    );
    Ok(())
}

struct DoomAudioSession {
    output: NativeAudioOutput,
    music: Arc<audio_tools::PcmClip>,
    pistol: Arc<audio_tools::PcmClip>,
    note_cue: Arc<audio_tools::PcmClip>,
}

impl DoomAudioSession {
    fn open(
        music: Arc<audio_tools::PcmClip>,
        pistol: Arc<audio_tools::PcmClip>,
        note_cue: Arc<audio_tools::PcmClip>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let output = NativeAudioOutput::open_default(NativeOutputConfig::default())?;
        output.play()?;
        Ok(Self {
            output,
            music,
            pistol,
            note_cue,
        })
    }

    fn start_music(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.output.start_loop(self.music.clone())?;
        Ok(())
    }

    fn trigger_pistol(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.output.play_one_shot(self.pistol.clone())?;
        Ok(())
    }

    fn trigger_note_cue(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.output.play_one_shot(self.note_cue.clone())?;
        Ok(())
    }

    fn pause(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.output.pause_content()?;
        Ok(())
    }

    fn resume(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.output.resume_content()?;
        Ok(())
    }

    fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.output.stop()?;
        Ok(())
    }
}

fn load_wad(
    package: &str,
    member: &str,
) -> Result<(Vec<u8>, doom_wad_provider::WadManifest), Box<dyn std::error::Error>> {
    let mut space =
        InMemoryResourceSpace::new(StoreId::from_u128(5_221), AddressCasePolicy::Sensitive);
    let folder = FolderId::from_u128(5_222);
    space.create_root(
        ResourceRootDescriptor::new(ResourceRootId::from_u128(5_223), "Doom audio package"),
        folder,
        ResourceMetadata::default(),
    )?;
    let resource_name =
        ResourceName::parse("canonical-doom-package.zip", AddressCasePolicy::Sensitive)?;
    space.insert_resource(
        folder,
        resource_name.clone(),
        fs::read(package)?,
        ResourceMetadata::default(),
    )?;
    let read = read_wad_package_member(
        &space,
        InspectWadPackageRequest {
            archive: InspectArchiveResourceRequest {
                source_folder: folder,
                source_name: resource_name,
                format: ArchiveFormat::Zip,
                limits: ArchiveReadLimits::new(
                    64 * 1024 * 1024,
                    2048,
                    16 * 1024 * 1024,
                    64 * 1024 * 1024,
                    4096,
                ),
            },
            member_name: member.to_owned(),
            wad_source_label: format!("{package}:{member}"),
            wad_limits: WAD_LIMITS,
        },
        &ZipArchiveProvider,
    )?;
    Ok((read.bytes, read.observation.wad))
}
