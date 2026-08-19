//! Corpus-local mapping from Doom gameplay events to logical sound requests.
//!
//! Source-format decoding belongs to `doom-audio-provider`; playback, mixing,
//! devices, and a stable Tokimu audio contract remain deliberately absent.

use audio_tools::{AudioValueError, SoundClipKey, SoundEmission, SoundRequest};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DoomGameplaySoundEvent {
    PlayerPistolFired,
    MonsterAlert {
        source_thing: u32,
        source_position: [f32; 3],
    },
}

pub fn request_doom_sound(event: DoomGameplaySoundEvent) -> Result<SoundRequest, AudioValueError> {
    let (clip, emission) = match event {
        DoomGameplaySoundEvent::PlayerPistolFired => {
            ("weapon.pistol", SoundEmission::ListenerRelative)
        }
        DoomGameplaySoundEvent::MonsterAlert {
            source_thing: _,
            source_position,
        } => (
            "monster.alert.zombieman",
            SoundEmission::Spatial {
                position: source_position,
            },
        ),
    };
    SoundRequest::new(SoundClipKey::new(clip)?, emission)
}

/// Corpus-private source resolution remains downstream of semantic request
/// creation and upstream of any future playback provider.
pub fn doom_sound_lump_for_clip(clip_key: &str) -> Option<&'static str> {
    match clip_key {
        "weapon.pistol" => Some("DSPISTOL"),
        "monster.alert.zombieman" => Some("DSPOSACT"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gameplay_events_lower_to_logical_requests_before_lump_resolution() {
        let pistol = request_doom_sound(DoomGameplaySoundEvent::PlayerPistolFired)
            .expect("fixed event maps to valid request");
        assert_eq!(pistol.clip.as_str(), "weapon.pistol");
        assert_eq!(pistol.emission, SoundEmission::ListenerRelative);
        assert_eq!(
            doom_sound_lump_for_clip(pistol.clip.as_str()),
            Some("DSPISTOL")
        );

        let alert = request_doom_sound(DoomGameplaySoundEvent::MonsterAlert {
            source_thing: 10,
            source_position: [128.0, -64.0, 0.0],
        })
        .expect("fixed event maps to valid request");
        assert_eq!(
            alert.emission,
            SoundEmission::Spatial {
                position: [128.0, -64.0, 0.0]
            }
        );
        assert_eq!(
            doom_sound_lump_for_clip(alert.clip.as_str()),
            Some("DSPOSACT")
        );
    }
}
