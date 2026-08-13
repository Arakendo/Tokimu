//! Corpus-local resolution of selected classic Doom linedef activation intent.
//!
//! This module classifies a source request before any runtime-owned moving
//! sector, application transition, or presentation policy exists. It does not
//! emulate Doom's dispatcher and does not mutate imported map records.

use doom_map_provider::{DoomLinedef, DoomMapCore, DoomSector, DoomSidedef, DoomSourceRecord};

/// Immutable source records needed to resolve a linedef activation. This is a
/// Doom-corpus data view, not runtime state or a generic trigger graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomLineActivationSource {
    pub linedefs: Vec<DoomLinedef>,
    pub sidedefs: Vec<DoomSidedef>,
    pub sectors: Vec<DoomSector>,
}

impl DoomLineActivationSource {
    pub fn from_map(map: &DoomMapCore) -> Self {
        Self {
            linedefs: map.linedefs.clone(),
            sidedefs: map.sidedefs.clone(),
            sectors: map.sectors.clone(),
        }
    }
}

/// The source interaction which caused a line activation request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoomLineActivation {
    Use,
    Cross,
}

/// A deterministic, source-addressed request from the Doom application layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomLineActivationRequest {
    pub source_linedef: DoomSourceRecord,
    pub activation: DoomLineActivation,
}

/// The smallest demonstrated request intent. Later runtime/application owners
/// may consume this intent; resolution alone performs no state transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoomLineActivationIntent {
    /// Classic manual doors accept front-side use and target the opposite
    /// sidedef's sector, rather than selecting a sector by their zero tag.
    RaiseDoor {
        target_sector: DoomSourceRecord,
    },
    ExitLevel {
        tag: u16,
    },
    LowerFloorTurbo {
        tag: u16,
    },
    PlatformDownWaitUpStay {
        tag: u16,
    },
}

/// Explicit result of resolving one request against immutable source lines.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoomLineActivationResolution {
    Accepted {
        source_linedef: DoomSourceRecord,
        special: u16,
        intent: DoomLineActivationIntent,
    },
    NoSpecial {
        source_linedef: DoomSourceRecord,
    },
    WrongActivation {
        source_linedef: DoomSourceRecord,
        special: u16,
        requested: DoomLineActivation,
        required: DoomLineActivation,
    },
    UnsupportedSpecial {
        source_linedef: DoomSourceRecord,
        special: u16,
    },
    UnknownLinedef {
        source_linedef: DoomSourceRecord,
    },
    MissingManualDoorTarget {
        source_linedef: DoomSourceRecord,
        missing_left_sidedef: Option<u16>,
    },
    InvalidManualDoorTarget {
        source_linedef: DoomSourceRecord,
        sidedef_index: u16,
        sector_index: u16,
    },
}

/// Corpus-local timing and clearance policy for the observed manual-door
/// special. These are map units and simulation ticks, not renderer units or a
/// generic animation profile. The defaults mirror the released Doom source's
/// normal vertical-door speed, wait, and four-unit top clearance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomManualDoorPolicy {
    pub speed_per_tick: i16,
    pub wait_ticks: u16,
    pub top_clearance: i16,
}

impl DoomManualDoorPolicy {
    pub const CLASSIC_NORMAL: Self = Self {
        speed_per_tick: 2,
        wait_ticks: 150,
        top_clearance: 4,
    };
}

/// Runtime-owned phase of one selected manual-door sector. The source map is
/// immutable; a later world/presentation lowerer may consume the height
/// observation without reparsing source WAD bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoomManualDoorPhase {
    Opening,
    Waiting { remaining_ticks: u16 },
    Closing,
    Closed,
}

/// One corpus-local moving-ceiling state machine. It deliberately contains no
/// mesh, renderer handle, or imported mutable sector record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomManualDoorRuntime {
    pub target_sector: DoomSourceRecord,
    pub closed_ceiling_height: i16,
    pub open_ceiling_height: i16,
    pub current_ceiling_height: i16,
    pub phase: DoomManualDoorPhase,
    pub policy: DoomManualDoorPolicy,
}

/// Explicit inability to create a manual-door runtime from retained source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoomManualDoorStartError {
    TargetSectorUnavailable { target_sector: DoomSourceRecord },
    NoAdjacentCeiling { target_sector: DoomSourceRecord },
    InvalidPolicy { speed_per_tick: i16 },
}

/// A one-tick transition observation for retained corpus diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomManualDoorTick {
    pub target_sector: DoomSourceRecord,
    pub before_height: i16,
    pub after_height: i16,
    pub before_phase: DoomManualDoorPhase,
    pub after_phase: DoomManualDoorPhase,
}

impl DoomManualDoorRuntime {
    /// Starts a selected manual-door sector from immutable source information.
    /// The opening destination is the lowest neighboring ceiling minus the
    /// retained four-unit clearance policy, matching the classic normal-door
    /// calculation without adopting a general moving-geometry contract.
    pub fn start(
        source: &DoomLineActivationSource,
        target_sector: DoomSourceRecord,
        policy: DoomManualDoorPolicy,
    ) -> Result<Self, DoomManualDoorStartError> {
        if policy.speed_per_tick <= 0 {
            return Err(DoomManualDoorStartError::InvalidPolicy {
                speed_per_tick: policy.speed_per_tick,
            });
        }
        let Some((target_index, target)) = source
            .sectors
            .iter()
            .enumerate()
            .find(|(_, sector)| sector.source == target_sector)
        else {
            return Err(DoomManualDoorStartError::TargetSectorUnavailable { target_sector });
        };
        let lowest_neighbor_ceiling = source
            .linedefs
            .iter()
            .filter_map(|line| neighboring_sector_index(line, target_index, &source.sidedefs))
            .filter_map(|index| source.sectors.get(index))
            .map(|sector| sector.ceiling_height)
            .min()
            .ok_or(DoomManualDoorStartError::NoAdjacentCeiling { target_sector })?;
        Ok(Self {
            target_sector,
            closed_ceiling_height: target.ceiling_height,
            open_ceiling_height: lowest_neighbor_ceiling - policy.top_clearance,
            current_ceiling_height: target.ceiling_height,
            phase: DoomManualDoorPhase::Opening,
            policy,
        })
    }

    /// Advances only this runtime-owned state by one deterministic tick.
    pub fn advance_tick(&mut self) -> DoomManualDoorTick {
        let before_height = self.current_ceiling_height;
        let before_phase = self.phase;
        match self.phase {
            DoomManualDoorPhase::Opening => {
                self.current_ceiling_height = self
                    .current_ceiling_height
                    .saturating_add(self.policy.speed_per_tick)
                    .min(self.open_ceiling_height);
                if self.current_ceiling_height == self.open_ceiling_height {
                    self.phase = DoomManualDoorPhase::Waiting {
                        remaining_ticks: self.policy.wait_ticks,
                    };
                }
            }
            DoomManualDoorPhase::Waiting { remaining_ticks } if remaining_ticks > 1 => {
                self.phase = DoomManualDoorPhase::Waiting {
                    remaining_ticks: remaining_ticks - 1,
                };
            }
            DoomManualDoorPhase::Waiting { .. } => {
                self.phase = DoomManualDoorPhase::Closing;
            }
            DoomManualDoorPhase::Closing => {
                self.current_ceiling_height = self
                    .current_ceiling_height
                    .saturating_sub(self.policy.speed_per_tick)
                    .max(self.closed_ceiling_height);
                if self.current_ceiling_height == self.closed_ceiling_height {
                    self.phase = DoomManualDoorPhase::Closed;
                }
            }
            DoomManualDoorPhase::Closed => {}
        }
        DoomManualDoorTick {
            target_sector: self.target_sector,
            before_height,
            after_height: self.current_ceiling_height,
            before_phase,
            after_phase: self.phase,
        }
    }
}

/// Returns the sector on the other side of a two-sided source line which
/// touches `target_sector_index`. One-sided lines and malformed sidedef
/// references contribute no adjacency rather than inventing a destination.
fn neighboring_sector_index(
    line: &DoomLinedef,
    target_sector_index: usize,
    sidedefs: &[DoomSidedef],
) -> Option<usize> {
    let right = line
        .right_sidedef
        .and_then(|index| sidedefs.get(usize::from(index)))?;
    let left = line
        .left_sidedef
        .and_then(|index| sidedefs.get(usize::from(index)))?;
    let right_sector = usize::from(right.sector);
    let left_sector = usize::from(left.sector);
    match (
        right_sector == target_sector_index,
        left_sector == target_sector_index,
    ) {
        (true, false) => Some(left_sector),
        (false, true) => Some(right_sector),
        _ => None,
    }
}

/// Resolves only the classic E1M1 lines presently classified by Slice 8.
///
/// The output intentionally distinguishes a valid future runtime/application
/// request from an immediate implementation. No door, floor, platform, or map
/// transition is performed here.
pub fn resolve_doom_line_activation(
    source: &DoomLineActivationSource,
    request: DoomLineActivationRequest,
) -> DoomLineActivationResolution {
    let Some(linedef) = source
        .linedefs
        .iter()
        .find(|linedef| linedef.source == request.source_linedef)
    else {
        return DoomLineActivationResolution::UnknownLinedef {
            source_linedef: request.source_linedef,
        };
    };
    let source_linedef = linedef.source;
    let special = linedef.special;
    if special == 0 {
        return DoomLineActivationResolution::NoSpecial { source_linedef };
    }

    let (required, intent) = match special {
        1 => {
            let Some(sidedef_index) = linedef.left_sidedef else {
                return DoomLineActivationResolution::MissingManualDoorTarget {
                    source_linedef,
                    missing_left_sidedef: None,
                };
            };
            let Some(sidedef) = source.sidedefs.get(usize::from(sidedef_index)) else {
                return DoomLineActivationResolution::MissingManualDoorTarget {
                    source_linedef,
                    missing_left_sidedef: Some(sidedef_index),
                };
            };
            let Some(sector) = source.sectors.get(usize::from(sidedef.sector)) else {
                return DoomLineActivationResolution::InvalidManualDoorTarget {
                    source_linedef,
                    sidedef_index,
                    sector_index: sidedef.sector,
                };
            };
            (
                DoomLineActivation::Use,
                DoomLineActivationIntent::RaiseDoor {
                    target_sector: sector.source,
                },
            )
        }
        11 => (
            DoomLineActivation::Cross,
            DoomLineActivationIntent::ExitLevel { tag: linedef.tag },
        ),
        36 => (
            DoomLineActivation::Cross,
            DoomLineActivationIntent::LowerFloorTurbo { tag: linedef.tag },
        ),
        88 => (
            DoomLineActivation::Cross,
            DoomLineActivationIntent::PlatformDownWaitUpStay { tag: linedef.tag },
        ),
        _ => {
            return DoomLineActivationResolution::UnsupportedSpecial {
                source_linedef,
                special,
            };
        }
    };
    if request.activation != required {
        return DoomLineActivationResolution::WrongActivation {
            source_linedef,
            special,
            requested: request.activation,
            required,
        };
    }
    DoomLineActivationResolution::Accepted {
        source_linedef,
        special,
        intent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(record_index: u32) -> DoomSourceRecord {
        DoomSourceRecord {
            lump_index: 17,
            record_index,
        }
    }

    fn line(record_index: u32, special: u16, tag: u16) -> DoomLinedef {
        DoomLinedef {
            source: source(record_index),
            start_vertex: 0,
            end_vertex: 1,
            flags: 0,
            special,
            tag,
            right_sidedef: None,
            left_sidedef: Some(0),
        }
    }

    fn source_with(line: DoomLinedef) -> DoomLineActivationSource {
        DoomLineActivationSource {
            linedefs: vec![line],
            sidedefs: vec![DoomSidedef {
                source: source(8),
                x_offset: 0,
                y_offset: 0,
                upper_texture: "-".to_owned(),
                lower_texture: "-".to_owned(),
                middle_texture: "-".to_owned(),
                sector: 0,
            }],
            sectors: vec![DoomSector {
                source: source(12),
                floor_height: 0,
                ceiling_height: 128,
                floor_texture: "FLOOR0_1".to_owned(),
                ceiling_texture: "CEIL1_1".to_owned(),
                light_level: 160,
                special: 0,
                tag: 0,
            }],
        }
    }

    #[test]
    fn use_request_retains_door_source_and_tag_without_mutating_source() {
        let source_data = source_with(line(4, 1, 0));
        assert_eq!(
            resolve_doom_line_activation(
                &source_data,
                DoomLineActivationRequest {
                    source_linedef: source(4),
                    activation: DoomLineActivation::Use,
                },
            ),
            DoomLineActivationResolution::Accepted {
                source_linedef: source(4),
                special: 1,
                intent: DoomLineActivationIntent::RaiseDoor {
                    target_sector: source(12),
                },
            }
        );
        assert_eq!(source_data.linedefs[0].special, 1);
    }

    #[test]
    fn crossing_special_does_not_look_like_a_use_special() {
        let source_data = source_with(line(5, 88, 7));
        assert_eq!(
            resolve_doom_line_activation(
                &source_data,
                DoomLineActivationRequest {
                    source_linedef: source(5),
                    activation: DoomLineActivation::Use,
                },
            ),
            DoomLineActivationResolution::WrongActivation {
                source_linedef: source(5),
                special: 88,
                requested: DoomLineActivation::Use,
                required: DoomLineActivation::Cross,
            }
        );
    }

    #[test]
    fn unclassified_special_and_unknown_source_remain_explicit() {
        let source_data = source_with(line(6, 48, 0));
        assert!(matches!(
            resolve_doom_line_activation(
                &source_data,
                DoomLineActivationRequest {
                    source_linedef: source(6),
                    activation: DoomLineActivation::Use,
                },
            ),
            DoomLineActivationResolution::UnsupportedSpecial { special: 48, .. }
        ));
        assert!(matches!(
            resolve_doom_line_activation(
                &source_data,
                DoomLineActivationRequest {
                    source_linedef: source(99),
                    activation: DoomLineActivation::Use,
                },
            ),
            DoomLineActivationResolution::UnknownLinedef { .. }
        ));
    }

    #[test]
    fn manual_door_requires_an_opposite_sidedef_target() {
        let mut line = line(7, 1, 0);
        line.left_sidedef = None;
        assert!(matches!(
            resolve_doom_line_activation(
                &source_with(line),
                DoomLineActivationRequest {
                    source_linedef: source(7),
                    activation: DoomLineActivation::Use,
                },
            ),
            DoomLineActivationResolution::MissingManualDoorTarget { .. }
        ));
    }

    fn door_source() -> DoomLineActivationSource {
        DoomLineActivationSource {
            linedefs: vec![DoomLinedef {
                source: source(4),
                start_vertex: 0,
                end_vertex: 1,
                flags: 0,
                special: 1,
                tag: 0,
                right_sidedef: Some(0),
                left_sidedef: Some(1),
            }],
            sidedefs: vec![
                DoomSidedef {
                    source: source(8),
                    x_offset: 0,
                    y_offset: 0,
                    upper_texture: "-".to_owned(),
                    lower_texture: "-".to_owned(),
                    middle_texture: "-".to_owned(),
                    sector: 0,
                },
                DoomSidedef {
                    source: source(9),
                    x_offset: 0,
                    y_offset: 0,
                    upper_texture: "-".to_owned(),
                    lower_texture: "-".to_owned(),
                    middle_texture: "-".to_owned(),
                    sector: 1,
                },
            ],
            sectors: vec![
                DoomSector {
                    source: source(12),
                    floor_height: 0,
                    ceiling_height: 200,
                    floor_texture: "FLOOR0_1".to_owned(),
                    ceiling_texture: "CEIL1_1".to_owned(),
                    light_level: 160,
                    special: 0,
                    tag: 0,
                },
                DoomSector {
                    source: source(13),
                    floor_height: 0,
                    ceiling_height: 128,
                    floor_texture: "FLOOR0_1".to_owned(),
                    ceiling_texture: "CEIL1_1".to_owned(),
                    light_level: 160,
                    special: 0,
                    tag: 0,
                },
            ],
        }
    }

    #[test]
    fn manual_door_runtime_owns_height_without_mutating_imported_sector() {
        let source_data = door_source();
        let mut door = DoomManualDoorRuntime::start(
            &source_data,
            source(13),
            DoomManualDoorPolicy {
                speed_per_tick: 4,
                wait_ticks: 2,
                top_clearance: 4,
            },
        )
        .expect("target sector has one adjacent ceiling");
        assert_eq!(door.closed_ceiling_height, 128);
        assert_eq!(door.open_ceiling_height, 196);
        let first = door.advance_tick();
        assert_eq!(first.after_height, 132);
        assert_eq!(source_data.sectors[1].ceiling_height, 128);

        for _ in 0..16 {
            door.advance_tick();
        }
        assert_eq!(door.current_ceiling_height, 196);
        assert_eq!(
            door.phase,
            DoomManualDoorPhase::Waiting { remaining_ticks: 2 }
        );
        door.advance_tick();
        door.advance_tick();
        assert_eq!(door.phase, DoomManualDoorPhase::Closing);
        for _ in 0..17 {
            door.advance_tick();
        }
        assert_eq!(door.current_ceiling_height, 128);
        assert_eq!(door.phase, DoomManualDoorPhase::Closed);
    }

    #[test]
    fn manual_door_runtime_rejects_missing_adjacency_and_invalid_speed() {
        let source_data = source_with(line(4, 1, 0));
        assert!(matches!(
            DoomManualDoorRuntime::start(
                &source_data,
                source(12),
                DoomManualDoorPolicy::CLASSIC_NORMAL,
            ),
            Err(DoomManualDoorStartError::NoAdjacentCeiling { .. })
        ));
        assert!(matches!(
            DoomManualDoorRuntime::start(
                &door_source(),
                source(13),
                DoomManualDoorPolicy {
                    speed_per_tick: 0,
                    ..DoomManualDoorPolicy::CLASSIC_NORMAL
                },
            ),
            Err(DoomManualDoorStartError::InvalidPolicy { .. })
        ));
    }
}
