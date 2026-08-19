//! Headless Doom MUS, transport, and bounded synthesis observation.

use std::{env, fs, process::ExitCode};

use archive_provider::{ArchiveFormat, ArchiveReadLimits, ZipArchiveProvider};
use audio_tools::{SequenceEventKind, SequenceTransport, TransportState};
use doom_audio_provider::{decode_doom_mus_score, DoomMusDecodeLimits};
use doom_wad_package::{read_wad_package_member, InspectWadPackageRequest};
use doom_wad_provider::WadReadLimits;
use resource_space::{
    AddressCasePolicy, FolderId, InMemoryResourceSpace, ResourceMetadata, ResourceName,
    ResourceRootDescriptor, ResourceRootId, StoreId,
};
use resource_space_archive::InspectArchiveResourceRequest;
use simple_audio_synth_provider::{encode_pcm16_wave, synthesize_sequence, SimpleSynthConfig};

const WAD_LIMITS: WadReadLimits =
    WadReadLimits::new(64 * 1024 * 1024, 8_192, 16 * 1024 * 1024, 64 * 1024 * 1024);
const MUS_LIMITS: DoomMusDecodeLimits = DoomMusDecodeLimits {
    maximum_score_bytes: 1024 * 1024,
    maximum_events: 100_000,
    maximum_duration_units: 140 * 60 * 30,
    maximum_instruments: 256,
};

fn main() -> ExitCode {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let (package, member, music_lump, wave_output) = match args.as_slice() {
        [package, member, music_lump] => (package, member, music_lump, None),
        [package, member, music_lump, wave_output] => {
            (package, member, music_lump, Some(wave_output.as_str()))
        }
        _ => {
            eprintln!(
                "usage: doom_music_report <canonical-doom-zip> <WAD-member-name> <music-lump> [preview.wav]"
            );
            return ExitCode::from(2);
        }
    };
    match report(package, member, music_lump, wave_output) {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Doom music report failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn report(
    package: &str,
    member: &str,
    music_lump: &str,
    wave_output: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let package_bytes = fs::read(package)?;
    let mut space =
        InMemoryResourceSpace::new(StoreId::from_u128(5_211), AddressCasePolicy::Sensitive);
    let folder = FolderId::from_u128(5_212);
    space.create_root(
        ResourceRootDescriptor::new(ResourceRootId::from_u128(5_213), "Doom music package"),
        folder,
        ResourceMetadata::default(),
    )?;
    let resource_name =
        ResourceName::parse("canonical-doom-package.zip", AddressCasePolicy::Sensitive)?;
    space.insert_resource(
        folder,
        resource_name.clone(),
        package_bytes,
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
    let score = decode_doom_mus_score(&read.bytes, &read.observation.wad, music_lump, MUS_LIMITS)?;

    let mut transport = SequenceTransport::default();
    transport.start();
    let mut dispatched = transport.advance(&score.sequence, 0, 4096)?.len();
    transport.pause()?;
    transport.resume()?;
    while transport.state() == TransportState::Playing {
        dispatched += transport.advance(&score.sequence, 35, 4096)?.len();
    }
    let finished_position = transport.position_units();
    transport.stop();

    let preview_units = u64::from(score.sequence.timebase().units_per_second()) * 5;
    let synthesis = synthesize_sequence(
        &score.sequence,
        SimpleSynthConfig {
            sample_rate_hz: 22_050,
            render_time_units: preview_units,
            maximum_frames: 22_050 * 5,
            maximum_voices: 64,
            master_gain: 0.08,
        },
    )?;
    let wave = encode_pcm16_wave(&synthesis.clip)?;
    if let Some(output) = wave_output {
        fs::write(output, &wave)?;
    }
    let wave_fingerprint = fingerprint(&wave);
    let (notes_on, notes_off, instruments, controls, bends) = score.sequence.events().iter().fold(
        (0_usize, 0_usize, 0_usize, 0_usize, 0_usize),
        |mut counts, event| {
            match event.kind {
                SequenceEventKind::NoteOn { .. } => counts.0 += 1,
                SequenceEventKind::NoteOff { .. } => counts.1 += 1,
                SequenceEventKind::Instrument { .. } => counts.2 += 1,
                SequenceEventKind::Control { .. } => counts.3 += 1,
                SequenceEventKind::PitchBend { .. } => counts.4 += 1,
            }
            counts
        },
    );

    Ok(format!(
        "Doom music report: source={member}; lump={}; index={}; score-bytes={}; primary-channels={}; secondary-channels={}; declared-instruments={}; timebase-hz={}; duration-units={}; duration-ms={:.3}; events={}; note-on={notes_on}; note-off={notes_off}; instruments={instruments}; controls={controls}; pitch-bends={bends}; sequence-fingerprint={:016x}\ntransport: clock=application-supplied-fixed-step; step-units=35; lifecycle=start>pause>resume>finish>stop; dispatched-events={dispatched}; conservation={}; finished-position={finished_position}; reset-position={}; reset-state={:?}\nsynthesis-preview: provider=tokimu-authored-triangle-oscillator; render-units={}; sample-rate-hz={}; channels={}; frames={}; dispatched-events={}; peak={:.6}; clipped-samples={}; maximum-active-voices={}; voice-steals={}; substituted-instruments={}; ignored-controls={}; sample-fingerprint={:016x}; wave-bytes={}; wave-fingerprint={wave_fingerprint:016x}; wave-output={}; audio-device=false; playback=false; renderer=false",
        score.source_name,
        score.source_lump_index,
        score.score_bytes,
        score.primary_channels,
        score.secondary_channels,
        score.instruments.len(),
        score.sequence.timebase().units_per_second(),
        score.sequence.duration_units(),
        score.sequence.duration_units() as f64
            * 1000.0
            / f64::from(score.sequence.timebase().units_per_second()),
        score.sequence.events().len(),
        score.sequence.structural_fingerprint(),
        if dispatched == score.sequence.events().len() {
            "exact"
        } else {
            "mismatch"
        },
        transport.position_units(),
        transport.state(),
        synthesis.rendered_time_units,
        synthesis.clip.sample_rate_hz(),
        synthesis.clip.channels(),
        synthesis.clip.frames(),
        synthesis.dispatched_events,
        synthesis.peak,
        synthesis.clipped_samples,
        synthesis.maximum_active_voices,
        synthesis.voice_steals,
        synthesis.substituted_instruments,
        synthesis.ignored_controls,
        synthesis.sample_fingerprint,
        wave.len(),
        wave_output.unwrap_or("not-written"),
    ))
}

fn fingerprint(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}
