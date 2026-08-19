//! Source-only classification of the `THINGS` kinds present in shareware E1M1.
//!
//! This is deliberately a selected-corpus table, not a generic Doom gameplay
//! registry. It preserves the distinction between map-authored spawn records
//! and runtime-created objects such as projectiles.

use doom_raster_provider::DoomSpriteFrameRotation;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DoomThingFamily {
    PlayerStart,
    MultiplayerStart,
    Monster,
    WeaponPickup,
    AmmoPickup,
    HealthPickup,
    ArmorPickup,
    Decoration,
    ExplosiveProp,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomThingKindClassification {
    pub kind: u16,
    pub name: &'static str,
    pub family: DoomThingFamily,
    pub initial_sprite: Option<&'static str>,
    pub initial_frame: Option<char>,
}

const fn thing(
    kind: u16,
    name: &'static str,
    family: DoomThingFamily,
    initial_sprite: Option<&'static str>,
) -> DoomThingKindClassification {
    DoomThingKindClassification {
        kind,
        name,
        family,
        initial_sprite,
        initial_frame: match initial_sprite {
            Some(_) => Some('A'),
            None => None,
        },
    }
}

/// Exact numeric kinds observed in the reviewed E1M1 `THINGS` lump.
pub const E1M1_THING_KINDS: &[DoomThingKindClassification] = &[
    thing(1, "player-one-start", DoomThingFamily::PlayerStart, None),
    thing(
        2,
        "player-two-start",
        DoomThingFamily::MultiplayerStart,
        None,
    ),
    thing(
        3,
        "player-three-start",
        DoomThingFamily::MultiplayerStart,
        None,
    ),
    thing(
        4,
        "player-four-start",
        DoomThingFamily::MultiplayerStart,
        None,
    ),
    thing(9, "shotgun-guy", DoomThingFamily::Monster, Some("SPOS")),
    DoomThingKindClassification {
        kind: 10,
        name: "bloody-mess",
        family: DoomThingFamily::Decoration,
        initial_sprite: Some("PLAY"),
        initial_frame: Some('W'),
    },
    thing(
        11,
        "deathmatch-start",
        DoomThingFamily::MultiplayerStart,
        None,
    ),
    DoomThingKindClassification {
        kind: 12,
        name: "bloody-mess",
        family: DoomThingFamily::Decoration,
        initial_sprite: Some("PLAY"),
        initial_frame: Some('W'),
    },
    DoomThingKindClassification {
        kind: 15,
        name: "dead-player",
        family: DoomThingFamily::Decoration,
        initial_sprite: Some("PLAY"),
        initial_frame: Some('N'),
    },
    thing(
        24,
        "pool-of-gibs",
        DoomThingFamily::Decoration,
        Some("POL5"),
    ),
    thing(35, "candelabra", DoomThingFamily::Decoration, Some("CBRA")),
    thing(48, "tech-pillar", DoomThingFamily::Decoration, Some("ELEC")),
    thing(2001, "shotgun", DoomThingFamily::WeaponPickup, Some("SHOT")),
    thing(
        2002,
        "chaingun",
        DoomThingFamily::WeaponPickup,
        Some("MGUN"),
    ),
    thing(
        2003,
        "rocket-launcher",
        DoomThingFamily::WeaponPickup,
        Some("LAUN"),
    ),
    thing(2007, "ammo-clip", DoomThingFamily::AmmoPickup, Some("CLIP")),
    thing(
        2008,
        "shotgun-shells",
        DoomThingFamily::AmmoPickup,
        Some("SHEL"),
    ),
    thing(
        2011,
        "stimpack",
        DoomThingFamily::HealthPickup,
        Some("STIM"),
    ),
    thing(2012, "medikit", DoomThingFamily::HealthPickup, Some("MEDI")),
    thing(
        2014,
        "health-bonus",
        DoomThingFamily::HealthPickup,
        Some("BON1"),
    ),
    thing(
        2015,
        "armor-bonus",
        DoomThingFamily::ArmorPickup,
        Some("BON2"),
    ),
    thing(
        2018,
        "green-armor",
        DoomThingFamily::ArmorPickup,
        Some("ARM1"),
    ),
    thing(
        2019,
        "blue-armor",
        DoomThingFamily::ArmorPickup,
        Some("ARM2"),
    ),
    thing(
        2028,
        "floor-lamp",
        DoomThingFamily::Decoration,
        Some("COLU"),
    ),
    thing(
        2035,
        "exploding-barrel",
        DoomThingFamily::ExplosiveProp,
        Some("BAR1"),
    ),
    thing(
        2046,
        "rocket-box",
        DoomThingFamily::AmmoPickup,
        Some("BROK"),
    ),
    thing(2048, "ammo-box", DoomThingFamily::AmmoPickup, Some("AMMO")),
    thing(2049, "shell-box", DoomThingFamily::AmmoPickup, Some("SBOX")),
    thing(3001, "imp", DoomThingFamily::Monster, Some("TROO")),
    thing(3004, "zombieman", DoomThingFamily::Monster, Some("POSS")),
];

pub fn classify_e1m1_thing_kind(kind: u16) -> Option<DoomThingKindClassification> {
    E1M1_THING_KINDS
        .binary_search_by_key(&kind, |classification| classification.kind)
        .ok()
        .map(|index| E1M1_THING_KINDS[index])
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomSpritePatchSelection {
    pub source_lump_index: u32,
    pub source_rotation: u8,
    pub mirrored: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DoomSpritePatchSelectionError {
    MissingFrame {
        sprite: String,
        frame: char,
    },
    MissingRotation {
        sprite: String,
        frame: char,
        rotation: u8,
    },
    AmbiguousRotation {
        sprite: String,
        frame: char,
        rotation: u8,
        source_lumps: Vec<u32>,
    },
}

/// Selects the classic eight-way source rotation for a map Thing. The result
/// is 1..=8; rotation-zero sprite frames bypass this value during patch lookup.
pub fn select_doom_sprite_view_rotation(
    viewer: [f64; 2],
    thing: [f64; 2],
    thing_angle_degrees: f64,
) -> u8 {
    let viewer_to_thing = (thing[1] - viewer[1])
        .atan2(thing[0] - viewer[0])
        .to_degrees();
    let relative = (viewer_to_thing - thing_angle_degrees + 202.5).rem_euclid(360.0);
    (relative / 45.0).floor() as u8 + 1
}

pub fn resolve_doom_sprite_patch(
    frames: &[DoomSpriteFrameRotation],
    sprite: &str,
    frame: char,
    view_rotation: u8,
) -> Result<DoomSpritePatchSelection, DoomSpritePatchSelectionError> {
    let matching_frame = frames
        .iter()
        .filter(|candidate| candidate.sprite.eq_ignore_ascii_case(sprite))
        .filter(|candidate| candidate.frame == frame)
        .collect::<Vec<_>>();
    if matching_frame.is_empty() {
        return Err(DoomSpritePatchSelectionError::MissingFrame {
            sprite: sprite.to_owned(),
            frame,
        });
    }
    let rotation = if matching_frame
        .iter()
        .any(|candidate| candidate.rotation == 0)
    {
        0
    } else {
        view_rotation
    };
    let matching_rotation = matching_frame
        .into_iter()
        .filter(|candidate| candidate.rotation == rotation)
        .collect::<Vec<_>>();
    match matching_rotation.as_slice() {
        [] => Err(DoomSpritePatchSelectionError::MissingRotation {
            sprite: sprite.to_owned(),
            frame,
            rotation,
        }),
        [selected] => Ok(DoomSpritePatchSelection {
            source_lump_index: selected.source_lump_index,
            source_rotation: selected.rotation,
            mirrored: selected.mirrored,
        }),
        duplicates => Err(DoomSpritePatchSelectionError::AmbiguousRotation {
            sprite: sprite.to_owned(),
            frame,
            rotation,
            source_lumps: duplicates
                .iter()
                .map(|candidate| candidate.source_lump_index)
                .collect(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn selected_table_is_sorted_unique_and_classifies_boundaries() {
        assert!(E1M1_THING_KINDS
            .windows(2)
            .all(|pair| pair[0].kind < pair[1].kind));
        assert_eq!(
            classify_e1m1_thing_kind(1).map(|value| value.family),
            Some(DoomThingFamily::PlayerStart)
        );
        assert_eq!(
            classify_e1m1_thing_kind(3004).map(|value| value.family),
            Some(DoomThingFamily::Monster)
        );
        assert_eq!(classify_e1m1_thing_kind(999), None);
    }

    #[test]
    fn canonical_e1m1_inventory_is_fully_classified_by_family() {
        let inventory = [
            (1, 1),
            (2, 1),
            (3, 1),
            (4, 1),
            (9, 16),
            (10, 2),
            (11, 5),
            (12, 2),
            (15, 4),
            (24, 7),
            (35, 2),
            (48, 2),
            (2001, 1),
            (2002, 1),
            (2003, 1),
            (2007, 2),
            (2008, 3),
            (2011, 1),
            (2012, 3),
            (2014, 13),
            (2015, 25),
            (2018, 1),
            (2019, 1),
            (2028, 8),
            (2035, 6),
            (2046, 3),
            (2048, 6),
            (2049, 6),
            (3001, 4),
            (3004, 9),
        ];
        let mut families = BTreeMap::new();
        for (kind, count) in inventory {
            let classification = classify_e1m1_thing_kind(kind).expect("classified E1M1 kind");
            *families.entry(classification.family).or_insert(0) += count;
        }

        assert_eq!(families.get(&DoomThingFamily::PlayerStart), Some(&1));
        assert_eq!(families.get(&DoomThingFamily::MultiplayerStart), Some(&8));
        assert_eq!(families.get(&DoomThingFamily::Monster), Some(&29));
        assert_eq!(families.get(&DoomThingFamily::WeaponPickup), Some(&3));
        assert_eq!(families.get(&DoomThingFamily::AmmoPickup), Some(&20));
        assert_eq!(families.get(&DoomThingFamily::HealthPickup), Some(&17));
        assert_eq!(families.get(&DoomThingFamily::ArmorPickup), Some(&27));
        assert_eq!(families.get(&DoomThingFamily::Decoration), Some(&27));
        assert_eq!(families.get(&DoomThingFamily::ExplosiveProp), Some(&6));
        assert_eq!(families.values().sum::<usize>(), 138);
    }

    #[test]
    fn view_rotation_and_paired_lump_mirroring_are_explicit() {
        assert_eq!(
            select_doom_sprite_view_rotation([1.0, 0.0], [0.0, 0.0], 0.0),
            1
        );
        assert_eq!(
            select_doom_sprite_view_rotation([-1.0, 0.0], [0.0, 0.0], 0.0),
            5
        );

        let frames = vec![
            DoomSpriteFrameRotation {
                source_lump_index: 10,
                sprite: "TROO".to_owned(),
                frame: 'A',
                rotation: 2,
                mirrored: false,
            },
            DoomSpriteFrameRotation {
                source_lump_index: 10,
                sprite: "TROO".to_owned(),
                frame: 'A',
                rotation: 8,
                mirrored: true,
            },
            DoomSpriteFrameRotation {
                source_lump_index: 20,
                sprite: "BON1".to_owned(),
                frame: 'A',
                rotation: 0,
                mirrored: false,
            },
        ];
        assert_eq!(
            resolve_doom_sprite_patch(&frames, "TROO", 'A', 8),
            Ok(DoomSpritePatchSelection {
                source_lump_index: 10,
                source_rotation: 8,
                mirrored: true,
            })
        );
        assert_eq!(
            resolve_doom_sprite_patch(&frames, "BON1", 'A', 6)
                .expect("rotation-zero applies to every view")
                .source_rotation,
            0
        );
    }
}
