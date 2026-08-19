//! Headless Doom sound decode and semantic-event observation.
//!
//! No audio device, playback backend, renderer, or window is initialized.

use std::{env, fs, process::ExitCode};

use archive_provider::{ArchiveFormat, ArchiveReadLimits, ZipArchiveProvider};
use audio_tools::{PcmClipLimits, SoundEmission};
use doom_audio_provider::{decode_doom_sound_effect, DoomSoundDecodeLimits, DoomSoundEffect};
use doom_wad_package::{read_wad_package_member, InspectWadPackageRequest};
use doom_wad_provider::WadReadLimits;
use hello_doom_e1m1::sound::{
    doom_sound_lump_for_clip, request_doom_sound, DoomGameplaySoundEvent,
};
use resource_space::{
    AddressCasePolicy, FolderId, InMemoryResourceSpace, ResourceMetadata, ResourceName,
    ResourceRootDescriptor, ResourceRootId, StoreId,
};
use resource_space_archive::InspectArchiveResourceRequest;

const WAD_LIMITS: WadReadLimits =
    WadReadLimits::new(64 * 1024 * 1024, 8_192, 16 * 1024 * 1024, 64 * 1024 * 1024);
const SOUND_LIMITS: DoomSoundDecodeLimits = DoomSoundDecodeLimits {
    maximum_samples: 1_000_000,
    maximum_sample_rate_hz: 48_000,
};

fn main() -> ExitCode {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let [package, member] = args.as_slice() else {
        eprintln!("usage: doom_sound_report <canonical-doom-zip> <WAD-member-name>");
        return ExitCode::from(2);
    };
    match report(package, member) {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Doom sound report failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn report(package: &str, member: &str) -> Result<String, Box<dyn std::error::Error>> {
    let package_bytes = fs::read(package)?;
    let mut space =
        InMemoryResourceSpace::new(StoreId::from_u128(5_201), AddressCasePolicy::Sensitive);
    let folder = FolderId::from_u128(5_202);
    space.create_root(
        ResourceRootDescriptor::new(ResourceRootId::from_u128(5_203), "Doom sound package"),
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

    let events = [
        DoomGameplaySoundEvent::PlayerPistolFired,
        DoomGameplaySoundEvent::MonsterAlert {
            source_thing: 10,
            source_position: [128.0, -64.0, 0.0],
        },
    ];
    let requests = events
        .map(request_doom_sound)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
    let mut decoded = Vec::new();
    for request in &requests {
        let lump = doom_sound_lump_for_clip(request.clip.as_str())
            .ok_or_else(|| format!("no Doom source mapping for {}", request.clip.as_str()))?;
        decoded.push(decode_doom_sound_effect(
            &read.bytes,
            &read.observation.wad,
            lump,
            SOUND_LIMITS,
        )?);
    }

    let mut lines = vec![format!(
        "Doom sound report: source={member}; decoded-clips={}; semantic-requests={}; source-format=format-3-unsigned-u8-mono; normalized-pcm=finite-f32-minus-one-inclusive-to-one-exclusive; audio-device=false; playback=false; renderer=false; clock=none",
        decoded.len(),
        requests.len()
    )];
    for sound in &decoded {
        lines.push(format_sound(sound));
    }
    for (index, request) in requests.iter().enumerate() {
        let emission = match request.emission {
            SoundEmission::ListenerRelative => "listener-relative".to_owned(),
            SoundEmission::Spatial { position } => {
                let source_thing = match events[index] {
                    DoomGameplaySoundEvent::MonsterAlert { source_thing, .. } => source_thing,
                    DoomGameplaySoundEvent::PlayerPistolFired => {
                        unreachable!("pistol request is listener-relative")
                    }
                };
                format!("spatial-source:thing={source_thing}:position={position:?}")
            }
        };
        lines.push(format!(
            "sound request {index}: clip-key={}; emission={emission}; resolved-source={}",
            request.clip.as_str(),
            doom_sound_lump_for_clip(request.clip.as_str())
                .expect("retained request has source resolution")
        ));
    }
    Ok(lines.join("\n"))
}

fn format_sound(sound: &DoomSoundEffect) -> String {
    let clip = sound
        .to_pcm_clip(PcmClipLimits {
            maximum_frames: usize::try_from(SOUND_LIMITS.maximum_samples)
                .expect("u32 sample limit fits usize"),
            maximum_channels: 2,
            maximum_sample_rate_hz: u32::from(SOUND_LIMITS.maximum_sample_rate_hz),
        })
        .expect("decoded bounded Doom sound maps to bounded PCM clip");
    let minimum = clip
        .interleaved_samples()
        .iter()
        .copied()
        .fold(f32::INFINITY, f32::min);
    let maximum = clip
        .interleaved_samples()
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    format!(
        "sound source: lump={}; index={}; format={}; sample-rate-hz={}; samples={}; duration-ms={:.3}; normalized-min={minimum:.6}; normalized-max={maximum:.6}; sample-fingerprint={:016x}",
        sound.source_name,
        sound.source_lump_index,
        sound.format,
        sound.sample_rate_hz,
        sound.samples.len(),
        sound.duration_seconds() * 1000.0,
        sound.sample_fingerprint(),
    )
}
