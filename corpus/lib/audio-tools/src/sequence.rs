use crate::{AudioValueError, SoundClipKey};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NoteSequenceLimits {
    pub maximum_events: usize,
    pub maximum_channels: u8,
    pub maximum_time_units: u64,
    pub maximum_units_per_second: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SequenceTimebase {
    units_per_second: u32,
}

impl SequenceTimebase {
    pub fn new(
        units_per_second: u32,
        maximum_units_per_second: u32,
    ) -> Result<Self, AudioValueError> {
        if units_per_second == 0 || units_per_second > maximum_units_per_second {
            return Err(AudioValueError::InvalidSequenceTimebase {
                units_per_second,
                maximum_units_per_second,
            });
        }
        Ok(Self { units_per_second })
    }

    pub const fn units_per_second(self) -> u32 {
        self.units_per_second
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct InstrumentKey(SoundClipKey);

impl InstrumentKey {
    pub fn new(value: impl Into<String>) -> Result<Self, AudioValueError> {
        SoundClipKey::new(value).map(Self)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SequenceControl {
    BankSelect,
    Modulation,
    Volume,
    Pan,
    Expression,
    Reverb,
    Chorus,
    Sustain,
    SoftPedal,
    AllSoundsOff,
    AllNotesOff,
    Mono,
    Poly,
    ResetControllers,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SequenceEventKind {
    NoteOn {
        note: u8,
        velocity: u8,
    },
    NoteOff {
        note: u8,
    },
    Instrument {
        instrument: InstrumentKey,
    },
    Control {
        control: SequenceControl,
        value: u8,
    },
    /// Signed bend in 1/8192 of the provider-neutral full-scale bend range.
    PitchBend {
        bend: i16,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SequenceEvent {
    pub time_units: u64,
    pub order: u32,
    pub channel: u8,
    pub kind: SequenceEventKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NoteSequence {
    timebase: SequenceTimebase,
    channels: u8,
    duration_units: u64,
    events: Vec<SequenceEvent>,
}

impl NoteSequence {
    pub fn new(
        timebase: SequenceTimebase,
        channels: u8,
        duration_units: u64,
        events: Vec<SequenceEvent>,
        limits: NoteSequenceLimits,
    ) -> Result<Self, AudioValueError> {
        if channels == 0 || channels > limits.maximum_channels {
            return Err(AudioValueError::InvalidChannelCount {
                channels,
                maximum_channels: limits.maximum_channels,
            });
        }
        if events.len() > limits.maximum_events {
            return Err(AudioValueError::SequenceEventLimitExceeded {
                events: events.len(),
                maximum_events: limits.maximum_events,
            });
        }
        if duration_units > limits.maximum_time_units {
            return Err(AudioValueError::SequenceDurationLimitExceeded {
                duration_units,
                maximum_time_units: limits.maximum_time_units,
            });
        }
        for (index, event) in events.iter().enumerate() {
            if event.channel >= channels {
                return Err(AudioValueError::InvalidSequenceChannel {
                    event_index: index,
                    channel: event.channel,
                    channels,
                });
            }
            if event.time_units > duration_units {
                return Err(AudioValueError::SequenceEventAfterDuration { event_index: index });
            }
            if index > 0 {
                let previous = &events[index - 1];
                if (event.time_units, event.order) <= (previous.time_units, previous.order) {
                    return Err(AudioValueError::UnorderedSequenceEvent { event_index: index });
                }
            }
            validate_event(index, &event.kind)?;
        }
        Ok(Self {
            timebase,
            channels,
            duration_units,
            events,
        })
    }

    pub const fn timebase(&self) -> SequenceTimebase {
        self.timebase
    }

    pub const fn channels(&self) -> u8 {
        self.channels
    }

    pub const fn duration_units(&self) -> u64 {
        self.duration_units
    }

    pub fn events(&self) -> &[SequenceEvent] {
        &self.events
    }

    pub fn structural_fingerprint(&self) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        fingerprint(&mut hash, &self.timebase.units_per_second.to_le_bytes());
        fingerprint(&mut hash, &[self.channels]);
        fingerprint(&mut hash, &self.duration_units.to_le_bytes());
        for event in &self.events {
            fingerprint(&mut hash, &event.time_units.to_le_bytes());
            fingerprint(&mut hash, &event.order.to_le_bytes());
            fingerprint(&mut hash, &[event.channel]);
            match &event.kind {
                SequenceEventKind::NoteOn { note, velocity } => {
                    fingerprint(&mut hash, &[0, *note, *velocity]);
                }
                SequenceEventKind::NoteOff { note } => fingerprint(&mut hash, &[1, *note]),
                SequenceEventKind::Instrument { instrument } => {
                    fingerprint(&mut hash, &[2]);
                    fingerprint(&mut hash, instrument.as_str().as_bytes());
                }
                SequenceEventKind::Control { control, value } => {
                    fingerprint(&mut hash, &[3, control_code(*control), *value]);
                }
                SequenceEventKind::PitchBend { bend } => {
                    fingerprint(&mut hash, &[4]);
                    fingerprint(&mut hash, &bend.to_le_bytes());
                }
            }
        }
        hash
    }
}

fn validate_event(index: usize, event: &SequenceEventKind) -> Result<(), AudioValueError> {
    let invalid = match event {
        SequenceEventKind::NoteOn { note, velocity } => (*note > 127)
            .then_some(i32::from(*note))
            .or_else(|| (*velocity > 127).then_some(i32::from(*velocity))),
        SequenceEventKind::NoteOff { note } => (*note > 127).then_some(i32::from(*note)),
        SequenceEventKind::Instrument { .. } => None,
        SequenceEventKind::Control { value, .. } => (*value > 127).then_some(i32::from(*value)),
        SequenceEventKind::PitchBend { bend } => {
            (!(-8192..=8191).contains(bend)).then_some(i32::from(*bend))
        }
    };
    if let Some(value) = invalid {
        return Err(AudioValueError::InvalidSequenceValue {
            event_index: index,
            value,
        });
    }
    Ok(())
}

fn control_code(control: SequenceControl) -> u8 {
    match control {
        SequenceControl::BankSelect => 0,
        SequenceControl::Modulation => 1,
        SequenceControl::Volume => 2,
        SequenceControl::Pan => 3,
        SequenceControl::Expression => 4,
        SequenceControl::Reverb => 5,
        SequenceControl::Chorus => 6,
        SequenceControl::Sustain => 7,
        SequenceControl::SoftPedal => 8,
        SequenceControl::AllSoundsOff => 9,
        SequenceControl::AllNotesOff => 10,
        SequenceControl::Mono => 11,
        SequenceControl::Poly => 12,
        SequenceControl::ResetControllers => 13,
    }
}

fn fingerprint(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash = (*hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIMITS: NoteSequenceLimits = NoteSequenceLimits {
        maximum_events: 8,
        maximum_channels: 16,
        maximum_time_units: 1_000,
        maximum_units_per_second: 1_000,
    };

    #[test]
    fn note_sequence_has_explicit_time_and_simultaneous_order() {
        let sequence = NoteSequence::new(
            SequenceTimebase::new(140, 1_000).expect("valid timebase"),
            2,
            140,
            vec![
                SequenceEvent {
                    time_units: 0,
                    order: 0,
                    channel: 0,
                    kind: SequenceEventKind::NoteOn {
                        note: 60,
                        velocity: 100,
                    },
                },
                SequenceEvent {
                    time_units: 0,
                    order: 1,
                    channel: 1,
                    kind: SequenceEventKind::NoteOn {
                        note: 64,
                        velocity: 100,
                    },
                },
            ],
            LIMITS,
        )
        .expect("bounded sequence");
        assert_eq!(sequence.timebase().units_per_second(), 140);
        assert_eq!(sequence.events().len(), 2);
        assert_eq!(sequence.structural_fingerprint(), 0x5fbf_aec6_90a8_e871);
    }

    #[test]
    fn unordered_and_invalid_events_fail_explicitly() {
        let result = NoteSequence::new(
            SequenceTimebase::new(140, 1_000).expect("valid timebase"),
            1,
            10,
            vec![
                SequenceEvent {
                    time_units: 2,
                    order: 1,
                    channel: 0,
                    kind: SequenceEventKind::NoteOff { note: 60 },
                },
                SequenceEvent {
                    time_units: 1,
                    order: 2,
                    channel: 0,
                    kind: SequenceEventKind::NoteOff { note: 60 },
                },
            ],
            LIMITS,
        );
        assert_eq!(
            result,
            Err(AudioValueError::UnorderedSequenceEvent { event_index: 1 })
        );
    }
}
