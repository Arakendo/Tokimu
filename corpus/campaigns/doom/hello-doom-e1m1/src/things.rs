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

/// Corpus-private visual state programs retained from the released Doom state
/// table. These programs advance source frames only. In particular, the
/// monster idle states' `A_Look` action is retained as deferred gameplay work
/// and is never executed by this presentation clock.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoomThingStateProgram {
    Hold,
    MonsterIdle,
    BarrelIdle,
    BonusLoop,
    GreenArmorLoop,
    BlueArmorLoop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomThingStateFrame {
    pub frame: char,
    pub tics: Option<u16>,
    pub full_bright: bool,
    pub gameplay_action_deferred: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomThingRuntimeState {
    pub program: DoomThingStateProgram,
    pub state_index: usize,
    pub remaining_tics: Option<u16>,
    pub elapsed_tics: u64,
}

pub const fn e1m1_thing_state_program(kind: u16) -> DoomThingStateProgram {
    match kind {
        9 | 3001 | 3004 => DoomThingStateProgram::MonsterIdle,
        2035 => DoomThingStateProgram::BarrelIdle,
        2014 | 2015 => DoomThingStateProgram::BonusLoop,
        2018 => DoomThingStateProgram::GreenArmorLoop,
        2019 => DoomThingStateProgram::BlueArmorLoop,
        _ => DoomThingStateProgram::Hold,
    }
}

impl DoomThingStateProgram {
    pub const fn state_count(self) -> usize {
        match self {
            Self::Hold => 1,
            Self::MonsterIdle | Self::BarrelIdle | Self::GreenArmorLoop | Self::BlueArmorLoop => 2,
            Self::BonusLoop => 6,
        }
    }

    pub const fn frame(self, initial_frame: char, state_index: usize) -> DoomThingStateFrame {
        match self {
            Self::Hold => DoomThingStateFrame {
                frame: initial_frame,
                tics: None,
                full_bright: false,
                gameplay_action_deferred: false,
            },
            Self::MonsterIdle => DoomThingStateFrame {
                frame: if state_index % 2 == 0 { 'A' } else { 'B' },
                tics: Some(10),
                full_bright: false,
                gameplay_action_deferred: true,
            },
            Self::BarrelIdle => DoomThingStateFrame {
                frame: if state_index % 2 == 0 { 'A' } else { 'B' },
                tics: Some(6),
                full_bright: false,
                gameplay_action_deferred: false,
            },
            Self::BonusLoop => {
                const FRAMES: [char; 6] = ['A', 'B', 'C', 'D', 'C', 'B'];
                DoomThingStateFrame {
                    frame: FRAMES[state_index % FRAMES.len()],
                    tics: Some(6),
                    full_bright: false,
                    gameplay_action_deferred: false,
                }
            }
            Self::GreenArmorLoop => DoomThingStateFrame {
                frame: if state_index % 2 == 0 { 'A' } else { 'B' },
                tics: Some(if state_index % 2 == 0 { 6 } else { 7 }),
                full_bright: state_index % 2 == 1,
                gameplay_action_deferred: false,
            },
            Self::BlueArmorLoop => DoomThingStateFrame {
                frame: if state_index % 2 == 0 { 'A' } else { 'B' },
                tics: Some(6),
                full_bright: state_index % 2 == 1,
                gameplay_action_deferred: false,
            },
        }
    }

    pub fn required_frames(self, initial_frame: char) -> Vec<char> {
        if self == Self::MonsterIdle {
            // The opt-in corpus chase candidate reuses the same source sprite
            // family after A_Look succeeds. Load the complete A-D run cycle
            // up front without mutating the imported Thing or stabilizing a
            // generic animation contract.
            return vec!['A', 'B', 'C', 'D'];
        }
        let mut frames = (0..self.state_count())
            .map(|state_index| self.frame(initial_frame, state_index).frame)
            .collect::<Vec<_>>();
        frames.sort_unstable();
        frames.dedup();
        frames
    }
}

/// Retained E1M1 run-state cadence from Doom's generated `info.c` table.
/// The former human uses four tics; the sergeant and imp use three.
pub const fn e1m1_monster_chase_tics(kind: u16) -> Option<u16> {
    match kind {
        3004 => Some(4),
        9 | 3001 => Some(3),
        _ => None,
    }
}

impl DoomThingRuntimeState {
    pub const fn new(program: DoomThingStateProgram, initial_frame: char) -> Self {
        Self {
            program,
            state_index: 0,
            remaining_tics: program.frame(initial_frame, 0).tics,
            elapsed_tics: 0,
        }
    }

    pub const fn frame(self, initial_frame: char) -> DoomThingStateFrame {
        self.program.frame(initial_frame, self.state_index)
    }

    /// Advances an integer number of Doom tics and reports whether the visible
    /// frame changed at least once. Chunking does not affect the final state.
    pub fn advance(&mut self, initial_frame: char, tics: u64) -> bool {
        let initial = self.frame(initial_frame).frame;
        for _ in 0..tics {
            self.elapsed_tics = self.elapsed_tics.saturating_add(1);
            let Some(remaining) = self.remaining_tics else {
                continue;
            };
            if remaining > 1 {
                self.remaining_tics = Some(remaining - 1);
                continue;
            }
            self.state_index = (self.state_index + 1) % self.program.state_count();
            self.remaining_tics = self.frame(initial_frame).tics;
        }
        self.frame(initial_frame).frame != initial
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DoomWeapon {
    Fist,
    Pistol,
    Shotgun,
    Chaingun,
    RocketLauncher,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomPlayerInventory {
    pub health: u16,
    pub armor_points: u16,
    pub armor_type: u8,
    /// Bullets, shells, rockets, and cells, in this corpus consumer's explicit
    /// order (the released enum places cells before rockets).
    pub ammo: [u16; 4],
    pub weapons: [bool; 5],
    /// Blue/yellow/red cards followed by blue/yellow/red skulls.
    pub keys: [bool; 6],
    pub item_count: u32,
}

impl Default for DoomPlayerInventory {
    fn default() -> Self {
        Self {
            health: 100,
            armor_points: 0,
            armor_type: 0,
            ammo: [50, 0, 0, 0],
            weapons: [true, true, false, false, false],
            keys: [false; 6],
            item_count: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoomPickupOutcome {
    Collected,
    NotNeeded,
    NotPickup,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoomPlayerDamageOutcome {
    Alive { health: u16, armor_points: u16 },
    Killed,
}

pub fn e1m1_pickup_touches_player(
    player_xy: [f32; 2],
    player_floor_z: f32,
    pickup_xy: [i16; 2],
    pickup_floor_z: i16,
) -> bool {
    const PLAYER_RADIUS: f32 = 16.0;
    const PICKUP_RADIUS: f32 = 20.0;
    let dx = f32::from(pickup_xy[0]) - player_xy[0];
    let dy = f32::from(pickup_xy[1]) - player_xy[1];
    let vertical_delta = f32::from(pickup_floor_z) - player_floor_z;
    dx * dx + dy * dy <= (PLAYER_RADIUS + PICKUP_RADIUS).powi(2)
        && (-8.0..=56.0).contains(&vertical_delta)
}

/// Source dimensions for the shootable actors present in E1M1. These are
/// collision facts, not billboard extents.
pub const fn e1m1_combat_actor_dimensions(kind: u16) -> Option<[f32; 2]> {
    match kind {
        9 | 3001 | 3004 => Some([20.0, 56.0]),
        2035 => Some([10.0, 42.0]),
        _ => None,
    }
}

impl DoomPlayerInventory {
    /// Applies Classic Doom's green/blue armor fractions before reducing
    /// health. Damage sources and death presentation remain caller-owned.
    pub fn apply_damage(&mut self, damage: u16) -> DoomPlayerDamageOutcome {
        let saved = match self.armor_type {
            1 => damage / 3,
            2 => damage / 2,
            _ => 0,
        }
        .min(self.armor_points);
        self.armor_points -= saved;
        if self.armor_points == 0 {
            self.armor_type = 0;
        }
        self.health = self.health.saturating_sub(damage - saved);
        if self.health == 0 {
            DoomPlayerDamageOutcome::Killed
        } else {
            DoomPlayerDamageOutcome::Alive {
                health: self.health,
                armor_points: self.armor_points,
            }
        }
    }

    fn give_ammo(&mut self, ammo_index: usize, clip_loads: u16) -> bool {
        const CLIP_AMMO: [u16; 4] = [10, 4, 1, 20];
        const MAX_AMMO: [u16; 4] = [200, 50, 50, 300];
        let old = self.ammo[ammo_index];
        self.ammo[ammo_index] = self.ammo[ammo_index]
            .saturating_add(CLIP_AMMO[ammo_index] * clip_loads)
            .min(MAX_AMMO[ammo_index]);
        self.ammo[ammo_index] != old
    }

    fn give_weapon(&mut self, weapon: DoomWeapon, ammo_index: usize) -> bool {
        let index = weapon as usize;
        let gave_weapon = !self.weapons[index];
        self.weapons[index] = true;
        self.give_ammo(ammo_index, 2) || gave_weapon
    }

    /// Applies the admitted single-player E1M1 pickup kinds. Source difficulty
    /// doubling, dropped-item flags, sounds, messages, and weapon switching are
    /// intentionally outside this first deterministic inventory transition.
    pub fn try_collect_e1m1_kind(&mut self, kind: u16) -> DoomPickupOutcome {
        let collected = match kind {
            2001 => self.give_weapon(DoomWeapon::Shotgun, 1),
            2002 => self.give_weapon(DoomWeapon::Chaingun, 0),
            2003 => self.give_weapon(DoomWeapon::RocketLauncher, 2),
            2007 => self.give_ammo(0, 1),
            2008 => self.give_ammo(1, 1),
            2046 => self.give_ammo(2, 5),
            2048 => self.give_ammo(0, 5),
            2049 => self.give_ammo(1, 5),
            2011 if self.health < 100 => {
                self.health = self.health.saturating_add(10).min(100);
                true
            }
            2012 if self.health < 100 => {
                self.health = self.health.saturating_add(25).min(100);
                true
            }
            2014 => {
                self.health = self.health.saturating_add(1).min(200);
                true
            }
            2015 => {
                self.armor_points = self.armor_points.saturating_add(1).min(200);
                if self.armor_type == 0 {
                    self.armor_type = 1;
                }
                true
            }
            2018 if self.armor_points < 100 => {
                self.armor_points = 100;
                self.armor_type = 1;
                true
            }
            2019 if self.armor_points < 200 => {
                self.armor_points = 200;
                self.armor_type = 2;
                true
            }
            5 | 6 | 13 | 38 | 39 | 40 => {
                let key_index = match kind {
                    5 => 0,
                    6 => 1,
                    13 => 2,
                    40 => 3,
                    39 => 4,
                    38 => 5,
                    _ => unreachable!(),
                };
                self.keys[key_index] = true;
                true
            }
            2011 | 2012 | 2018 | 2019 => false,
            _ => return DoomPickupOutcome::NotPickup,
        };
        if collected {
            self.item_count = self.item_count.saturating_add(1);
            DoomPickupOutcome::Collected
        } else {
            DoomPickupOutcome::NotNeeded
        }
    }
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
    fn source_state_programs_preserve_reviewed_frame_cadence() {
        let monster = e1m1_thing_state_program(3004);
        assert_eq!(monster.required_frames('A'), vec!['A', 'B', 'C', 'D']);
        assert_eq!(monster.frame('A', 0).tics, Some(10));
        assert!(monster.frame('A', 0).gameplay_action_deferred);
        assert_eq!(e1m1_monster_chase_tics(3004), Some(4));
        assert_eq!(e1m1_monster_chase_tics(9), Some(3));
        assert_eq!(e1m1_monster_chase_tics(3001), Some(3));

        let bonus = e1m1_thing_state_program(2014);
        let frames = (0..bonus.state_count())
            .map(|index| bonus.frame('A', index).frame)
            .collect::<Vec<_>>();
        assert_eq!(frames, vec!['A', 'B', 'C', 'D', 'C', 'B']);

        let armor = e1m1_thing_state_program(2018);
        assert_eq!(armor.frame('A', 0).tics, Some(6));
        assert_eq!(armor.frame('A', 1).tics, Some(7));
        assert!(armor.frame('A', 1).full_bright);
        assert_eq!(e1m1_thing_state_program(2001), DoomThingStateProgram::Hold);
    }

    #[test]
    fn runtime_state_is_deterministic_across_tick_chunking() {
        let program = e1m1_thing_state_program(2014);
        let mut single = DoomThingRuntimeState::new(program, 'A');
        let mut chunked = single;
        single.advance('A', 49);
        for ticks in [3, 11, 1, 20, 14] {
            chunked.advance('A', ticks);
        }
        assert_eq!(single, chunked);
        assert_eq!(single.frame('A').frame, 'C');

        let mut held = DoomThingRuntimeState::new(DoomThingStateProgram::Hold, 'W');
        assert!(!held.advance('W', 10_000));
        assert_eq!(held.frame('W').frame, 'W');
    }

    #[test]
    fn e1m1_pickup_transitions_are_bounded_and_deterministic() {
        let mut player = DoomPlayerInventory {
            health: 82,
            ..DoomPlayerInventory::default()
        };
        assert_eq!(
            player.try_collect_e1m1_kind(2011),
            DoomPickupOutcome::Collected
        );
        assert_eq!(player.health, 92);
        assert_eq!(
            player.try_collect_e1m1_kind(2012),
            DoomPickupOutcome::Collected
        );
        assert_eq!(player.health, 100);
        assert_eq!(
            player.try_collect_e1m1_kind(2012),
            DoomPickupOutcome::NotNeeded
        );
        assert_eq!(
            player.try_collect_e1m1_kind(2001),
            DoomPickupOutcome::Collected
        );
        assert!(player.weapons[DoomWeapon::Shotgun as usize]);
        assert_eq!(player.ammo[1], 8);
        assert_eq!(
            player.try_collect_e1m1_kind(35),
            DoomPickupOutcome::NotPickup
        );
        assert_eq!(
            player.try_collect_e1m1_kind(5),
            DoomPickupOutcome::Collected
        );
        assert!(player.keys[0]);
    }

    #[test]
    fn identical_pickup_replay_produces_identical_inventory() {
        let replay = [2014, 2015, 2048, 2001, 2049, 2018, 2019, 2007];
        let run = || {
            let mut state = DoomPlayerInventory::default();
            for kind in replay {
                state.try_collect_e1m1_kind(kind);
            }
            state
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn player_damage_applies_classic_armor_fractions_and_death() {
        let mut green = DoomPlayerInventory {
            armor_points: 100,
            armor_type: 1,
            ..DoomPlayerInventory::default()
        };
        assert_eq!(
            green.apply_damage(30),
            DoomPlayerDamageOutcome::Alive {
                health: 80,
                armor_points: 90,
            }
        );

        let mut blue = DoomPlayerInventory {
            armor_points: 5,
            armor_type: 2,
            health: 10,
            ..DoomPlayerInventory::default()
        };
        assert_eq!(blue.apply_damage(20), DoomPlayerDamageOutcome::Killed);
        assert_eq!(blue.health, 0);
        assert_eq!(blue.armor_points, 0);
        assert_eq!(blue.armor_type, 0);

        blue = DoomPlayerInventory::default();
        assert_eq!(blue.health, 100);
        assert_eq!(blue.armor_points, 0);
    }

    #[test]
    fn pickup_contact_preserves_horizontal_and_vertical_boundaries() {
        assert!(e1m1_pickup_touches_player([0.0, 0.0], 0.0, [36, 0], 56));
        assert!(!e1m1_pickup_touches_player([0.0, 0.0], 0.0, [37, 0], 56));
        assert!(!e1m1_pickup_touches_player([0.0, 0.0], 0.0, [0, 0], -9));
        assert_eq!(e1m1_combat_actor_dimensions(3001), Some([20.0, 56.0]));
        assert_eq!(e1m1_combat_actor_dimensions(2035), Some([10.0, 42.0]));
        assert_eq!(e1m1_combat_actor_dimensions(2014), None);
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
