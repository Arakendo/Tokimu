use audio_tools::{
    AudioValueError, InstrumentKey, NoteSequence, NoteSequenceLimits, SequenceControl,
    SequenceEvent, SequenceEventKind, SequenceTimebase,
};
use doom_wad_provider::WadManifest;
use thiserror::Error;

const MUS_HEADER_BYTES: usize = 14;
const MUS_TIME_UNITS_PER_SECOND: u32 = 140;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomMusDecodeLimits {
    pub maximum_score_bytes: usize,
    pub maximum_events: usize,
    pub maximum_duration_units: u64,
    pub maximum_instruments: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomMusScore {
    pub source_lump_index: u32,
    pub source_name: String,
    pub score_bytes: usize,
    pub primary_channels: u16,
    pub secondary_channels: u16,
    pub instruments: Vec<u16>,
    pub sequence: NoteSequence,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum DoomMusDecodeError {
    #[error("Doom MUS lump {name} is unavailable")]
    MissingLump { name: String },
    #[error("Doom MUS lump {name} range is outside the retained WAD bytes")]
    LumpOutOfBounds { name: String },
    #[error("Doom MUS lump {name} is truncated at byte {offset}")]
    Truncated { name: String, offset: usize },
    #[error("Doom MUS lump {name} does not begin with MUS\\x1a")]
    InvalidMagic { name: String },
    #[error("Doom MUS lump {name} score range is invalid")]
    InvalidScoreRange { name: String },
    #[error("Doom MUS lump {name} score has {bytes} bytes, exceeding limit {limit}")]
    ScoreLimitExceeded {
        name: String,
        bytes: usize,
        limit: usize,
    },
    #[error("Doom MUS lump {name} has {instruments} instruments, exceeding limit {limit}")]
    InstrumentLimitExceeded {
        name: String,
        instruments: usize,
        limit: usize,
    },
    #[error("Doom MUS lump {name} uses unsupported event type {event_type} at byte {offset}")]
    UnsupportedEvent {
        name: String,
        event_type: u8,
        offset: usize,
    },
    #[error("Doom MUS lump {name} uses unsupported controller {controller} at byte {offset}")]
    UnsupportedController {
        name: String,
        controller: u8,
        offset: usize,
    },
    #[error("Doom MUS lump {name} time delay overflows at byte {offset}")]
    TimeOverflow { name: String, offset: usize },
    #[error("Doom MUS lump {name} has no score-end event")]
    MissingScoreEnd { name: String },
    #[error("Doom MUS lump {name} cannot lower into bounded note semantics: {source}")]
    InvalidSequence {
        name: String,
        #[source]
        source: AudioValueError,
    },
}

pub fn decode_doom_mus_score(
    wad_bytes: &[u8],
    manifest: &WadManifest,
    name: &str,
    limits: DoomMusDecodeLimits,
) -> Result<DoomMusScore, DoomMusDecodeError> {
    let lump = manifest
        .lumps
        .iter()
        .rev()
        .find(|lump| lump.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| DoomMusDecodeError::MissingLump {
            name: name.to_owned(),
        })?;
    let start = usize::try_from(lump.offset).expect("u32 offset fits usize");
    let size = usize::try_from(lump.size).expect("u32 size fits usize");
    let bytes = start
        .checked_add(size)
        .filter(|end| *end <= wad_bytes.len())
        .map(|end| &wad_bytes[start..end])
        .ok_or_else(|| DoomMusDecodeError::LumpOutOfBounds {
            name: lump.name.clone(),
        })?;
    if bytes.len() < MUS_HEADER_BYTES {
        return Err(DoomMusDecodeError::Truncated {
            name: lump.name.clone(),
            offset: bytes.len(),
        });
    }
    if bytes[..4] != *b"MUS\x1a" {
        return Err(DoomMusDecodeError::InvalidMagic {
            name: lump.name.clone(),
        });
    }

    let score_length = usize::from(read_u16(bytes, 4));
    let score_start = usize::from(read_u16(bytes, 6));
    let primary_channels = read_u16(bytes, 8);
    let secondary_channels = read_u16(bytes, 10);
    let instrument_count = usize::from(read_u16(bytes, 12));
    if score_length > limits.maximum_score_bytes {
        return Err(DoomMusDecodeError::ScoreLimitExceeded {
            name: lump.name.clone(),
            bytes: score_length,
            limit: limits.maximum_score_bytes,
        });
    }
    if instrument_count > limits.maximum_instruments {
        return Err(DoomMusDecodeError::InstrumentLimitExceeded {
            name: lump.name.clone(),
            instruments: instrument_count,
            limit: limits.maximum_instruments,
        });
    }
    let instrument_end = MUS_HEADER_BYTES
        .checked_add(instrument_count.saturating_mul(2))
        .ok_or_else(|| DoomMusDecodeError::InvalidScoreRange {
            name: lump.name.clone(),
        })?;
    let score_end = score_start
        .checked_add(score_length)
        .filter(|end| *end <= bytes.len())
        .ok_or_else(|| DoomMusDecodeError::InvalidScoreRange {
            name: lump.name.clone(),
        })?;
    if instrument_end > score_start || score_start < MUS_HEADER_BYTES {
        return Err(DoomMusDecodeError::InvalidScoreRange {
            name: lump.name.clone(),
        });
    }
    let instruments = (0..instrument_count)
        .map(|index| read_u16(bytes, MUS_HEADER_BYTES + index * 2))
        .collect::<Vec<_>>();

    let mut cursor = score_start;
    let mut time_units = 0_u64;
    let mut events = Vec::new();
    let mut velocities = [127_u8; 16];
    let mut found_end = false;
    while cursor < score_end {
        let descriptor_offset = cursor;
        let descriptor = take(bytes, &mut cursor, score_end, &lump.name)?;
        let source_channel = descriptor & 0x0f;
        let event_type = (descriptor >> 4) & 0x07;
        let kind = match event_type {
            0 => {
                let note = take(bytes, &mut cursor, score_end, &lump.name)? & 0x7f;
                Some(SequenceEventKind::NoteOff { note })
            }
            1 => {
                let note_byte = take(bytes, &mut cursor, score_end, &lump.name)?;
                if note_byte & 0x80 != 0 {
                    velocities[usize::from(source_channel)] =
                        take(bytes, &mut cursor, score_end, &lump.name)? & 0x7f;
                }
                Some(SequenceEventKind::NoteOn {
                    note: note_byte & 0x7f,
                    velocity: velocities[usize::from(source_channel)],
                })
            }
            2 => {
                let wheel = take(bytes, &mut cursor, score_end, &lump.name)?;
                Some(SequenceEventKind::PitchBend {
                    bend: i16::from(wheel) * 64 - 8192,
                })
            }
            3 => {
                let controller_offset = cursor;
                let controller = take(bytes, &mut cursor, score_end, &lump.name)?;
                let control = map_system_control(controller).ok_or_else(|| {
                    DoomMusDecodeError::UnsupportedController {
                        name: lump.name.clone(),
                        controller,
                        offset: controller_offset,
                    }
                })?;
                Some(SequenceEventKind::Control { control, value: 0 })
            }
            4 => {
                let controller_offset = cursor;
                let controller = take(bytes, &mut cursor, score_end, &lump.name)?;
                let value = take(bytes, &mut cursor, score_end, &lump.name)? & 0x7f;
                if controller == 0 {
                    Some(SequenceEventKind::Instrument {
                        instrument: InstrumentKey::new(format!("doom.mus.program.{value}"))
                            .map_err(|source| DoomMusDecodeError::InvalidSequence {
                                name: lump.name.clone(),
                                source,
                            })?,
                    })
                } else {
                    let control = map_valued_control(controller).ok_or_else(|| {
                        DoomMusDecodeError::UnsupportedController {
                            name: lump.name.clone(),
                            controller,
                            offset: controller_offset,
                        }
                    })?;
                    Some(SequenceEventKind::Control { control, value })
                }
            }
            6 => {
                found_end = true;
                None
            }
            _ => {
                return Err(DoomMusDecodeError::UnsupportedEvent {
                    name: lump.name.clone(),
                    event_type,
                    offset: descriptor_offset,
                });
            }
        };
        if let Some(kind) = kind {
            if events.len() >= limits.maximum_events {
                return Err(DoomMusDecodeError::InvalidSequence {
                    name: lump.name.clone(),
                    source: AudioValueError::SequenceEventLimitExceeded {
                        events: events.len() + 1,
                        maximum_events: limits.maximum_events,
                    },
                });
            }
            events.push(SequenceEvent {
                time_units,
                order: u32::try_from(events.len()).map_err(|_| {
                    DoomMusDecodeError::InvalidSequence {
                        name: lump.name.clone(),
                        source: AudioValueError::SequenceEventLimitExceeded {
                            events: events.len() + 1,
                            maximum_events: u32::MAX as usize,
                        },
                    }
                })?,
                channel: source_channel,
                kind,
            });
        }
        if found_end {
            break;
        }
        if descriptor & 0x80 != 0 {
            let delay = read_time_delay(bytes, &mut cursor, score_end, &lump.name)?;
            time_units =
                time_units
                    .checked_add(delay)
                    .ok_or_else(|| DoomMusDecodeError::TimeOverflow {
                        name: lump.name.clone(),
                        offset: cursor,
                    })?;
            if time_units > limits.maximum_duration_units {
                return Err(DoomMusDecodeError::InvalidSequence {
                    name: lump.name.clone(),
                    source: AudioValueError::SequenceDurationLimitExceeded {
                        duration_units: time_units,
                        maximum_time_units: limits.maximum_duration_units,
                    },
                });
            }
        }
    }
    if !found_end {
        return Err(DoomMusDecodeError::MissingScoreEnd {
            name: lump.name.clone(),
        });
    }
    let sequence_limits = NoteSequenceLimits {
        maximum_events: limits.maximum_events,
        maximum_channels: 16,
        maximum_time_units: limits.maximum_duration_units,
        maximum_units_per_second: MUS_TIME_UNITS_PER_SECOND,
    };
    let sequence = NoteSequence::new(
        SequenceTimebase::new(MUS_TIME_UNITS_PER_SECOND, MUS_TIME_UNITS_PER_SECOND)
            .expect("fixed MUS timebase is valid"),
        16,
        time_units,
        events,
        sequence_limits,
    )
    .map_err(|source| DoomMusDecodeError::InvalidSequence {
        name: lump.name.clone(),
        source,
    })?;
    Ok(DoomMusScore {
        source_lump_index: lump.index,
        source_name: lump.name.clone(),
        score_bytes: score_length,
        primary_channels,
        secondary_channels,
        instruments,
        sequence,
    })
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn take(
    bytes: &[u8],
    cursor: &mut usize,
    end: usize,
    name: &str,
) -> Result<u8, DoomMusDecodeError> {
    if *cursor >= end {
        return Err(DoomMusDecodeError::Truncated {
            name: name.to_owned(),
            offset: *cursor,
        });
    }
    let byte = bytes[*cursor];
    *cursor += 1;
    Ok(byte)
}

fn read_time_delay(
    bytes: &[u8],
    cursor: &mut usize,
    end: usize,
    name: &str,
) -> Result<u64, DoomMusDecodeError> {
    let mut delay = 0_u64;
    for _ in 0..10 {
        let byte = take(bytes, cursor, end, name)?;
        delay = delay
            .checked_mul(128)
            .and_then(|value| value.checked_add(u64::from(byte & 0x7f)))
            .ok_or_else(|| DoomMusDecodeError::TimeOverflow {
                name: name.to_owned(),
                offset: *cursor - 1,
            })?;
        if byte & 0x80 == 0 {
            return Ok(delay);
        }
    }
    Err(DoomMusDecodeError::TimeOverflow {
        name: name.to_owned(),
        offset: *cursor,
    })
}

fn map_valued_control(controller: u8) -> Option<SequenceControl> {
    Some(match controller {
        1 => SequenceControl::BankSelect,
        2 => SequenceControl::Modulation,
        3 => SequenceControl::Volume,
        4 => SequenceControl::Pan,
        5 => SequenceControl::Expression,
        6 => SequenceControl::Reverb,
        7 => SequenceControl::Chorus,
        8 => SequenceControl::Sustain,
        9 => SequenceControl::SoftPedal,
        _ => return None,
    })
}

fn map_system_control(controller: u8) -> Option<SequenceControl> {
    Some(match controller {
        10 => SequenceControl::AllSoundsOff,
        11 => SequenceControl::AllNotesOff,
        12 => SequenceControl::Mono,
        13 => SequenceControl::Poly,
        14 => SequenceControl::ResetControllers,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use doom_wad_provider::{WadKind, WadLumpObservation, WadSourceIdentity};

    use super::*;

    fn manifest(bytes: usize) -> WadManifest {
        WadManifest {
            source: WadSourceIdentity {
                label: "fixture".to_owned(),
                byte_len: bytes,
                blake3: "fixture".to_owned(),
            },
            kind: WadKind::Iwad,
            directory_offset: 0,
            directory_bytes: 0,
            total_lump_bytes: bytes as u64,
            lumps: vec![WadLumpObservation {
                index: 7,
                offset: 0,
                size: bytes as u32,
                name: "D_TEST".to_owned(),
            }],
            namespaces: Vec::new(),
        }
    }

    const LIMITS: DoomMusDecodeLimits = DoomMusDecodeLimits {
        maximum_score_bytes: 64,
        maximum_events: 16,
        maximum_duration_units: 1_000,
        maximum_instruments: 8,
    };

    #[test]
    fn bounded_mus_score_lowers_to_generic_note_sequence() {
        let mut bytes = vec![b'M', b'U', b'S', 0x1a, 8, 0, 14, 0, 1, 0, 0, 0, 0, 0];
        bytes.extend_from_slice(&[0x90, 0xbc, 100, 10, 0x80, 60, 5, 0x60]);
        let score = decode_doom_mus_score(&bytes, &manifest(bytes.len()), "D_TEST", LIMITS)
            .expect("synthetic MUS score");
        assert_eq!(score.sequence.timebase().units_per_second(), 140);
        assert_eq!(score.sequence.duration_units(), 15);
        assert_eq!(score.sequence.events().len(), 2);
        assert!(matches!(
            score.sequence.events()[0].kind,
            SequenceEventKind::NoteOn {
                note: 60,
                velocity: 100
            }
        ));
        assert!(matches!(
            score.sequence.events()[1].kind,
            SequenceEventKind::NoteOff { note: 60 }
        ));
    }

    #[test]
    fn malformed_mus_score_fails_without_reading_past_bounds() {
        let mut bytes = vec![b'M', b'U', b'S', 0x1a, 3, 0, 14, 0, 1, 0, 0, 0, 0, 0];
        bytes.extend_from_slice(&[0x90, 0xbc, 100]);
        assert!(matches!(
            decode_doom_mus_score(&bytes, &manifest(bytes.len()), "D_TEST", LIMITS),
            Err(DoomMusDecodeError::Truncated { .. })
        ));
    }
}
