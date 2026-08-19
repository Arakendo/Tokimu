use crate::{AudioValueError, NoteSequence, SequenceEvent};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportState {
    Stopped,
    Playing,
    Paused,
    Finished,
}

impl TransportState {
    const fn label(self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Playing => "playing",
            Self::Paused => "paused",
            Self::Finished => "finished",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SequenceTransport {
    state: TransportState,
    position_units: u64,
    next_event: usize,
}

impl Default for SequenceTransport {
    fn default() -> Self {
        Self {
            state: TransportState::Stopped,
            position_units: 0,
            next_event: 0,
        }
    }
}

impl SequenceTransport {
    pub const fn state(&self) -> TransportState {
        self.state
    }

    pub const fn position_units(&self) -> u64 {
        self.position_units
    }

    pub fn start(&mut self) {
        self.state = TransportState::Playing;
        self.position_units = 0;
        self.next_event = 0;
    }

    pub fn pause(&mut self) -> Result<(), AudioValueError> {
        if self.state != TransportState::Playing {
            return Err(self.invalid_transition("pause"));
        }
        self.state = TransportState::Paused;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), AudioValueError> {
        if self.state != TransportState::Paused {
            return Err(self.invalid_transition("resume"));
        }
        self.state = TransportState::Playing;
        Ok(())
    }

    pub fn stop(&mut self) {
        self.state = TransportState::Stopped;
        self.position_units = 0;
        self.next_event = 0;
    }

    pub fn advance(
        &mut self,
        sequence: &NoteSequence,
        delta_units: u64,
        maximum_dispatch: usize,
    ) -> Result<Vec<SequenceEvent>, AudioValueError> {
        if self.state != TransportState::Playing {
            return Err(self.invalid_transition("advance"));
        }
        let target = self
            .position_units
            .saturating_add(delta_units)
            .min(sequence.duration_units());
        let start = self.next_event;
        let mut end = start;
        while end < sequence.events().len() && sequence.events()[end].time_units <= target {
            end += 1;
            if end - start > maximum_dispatch {
                return Err(AudioValueError::SequenceEventLimitExceeded {
                    events: end - start,
                    maximum_events: maximum_dispatch,
                });
            }
        }
        self.next_event = end;
        self.position_units = target;
        if target == sequence.duration_units() {
            self.state = TransportState::Finished;
        }
        Ok(sequence.events()[start..end].to_vec())
    }

    fn invalid_transition(&self, operation: &'static str) -> AudioValueError {
        AudioValueError::InvalidTransportTransition {
            operation,
            state: self.state.label(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{NoteSequenceLimits, SequenceEventKind, SequenceTimebase};

    use super::*;

    fn fixture() -> NoteSequence {
        NoteSequence::new(
            SequenceTimebase::new(10, 100).expect("valid timebase"),
            1,
            10,
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
                    time_units: 5,
                    order: 1,
                    channel: 0,
                    kind: SequenceEventKind::NoteOff { note: 60 },
                },
            ],
            NoteSequenceLimits {
                maximum_events: 4,
                maximum_channels: 2,
                maximum_time_units: 100,
                maximum_units_per_second: 100,
            },
        )
        .expect("fixture")
    }

    #[test]
    fn transport_lifecycle_and_fixed_step_dispatch_are_deterministic() {
        let sequence = fixture();
        let mut transport = SequenceTransport::default();
        transport.start();
        assert_eq!(transport.advance(&sequence, 0, 4).unwrap().len(), 1);
        assert_eq!(transport.advance(&sequence, 4, 4).unwrap().len(), 0);
        transport.pause().unwrap();
        assert_eq!(transport.state(), TransportState::Paused);
        transport.resume().unwrap();
        assert_eq!(transport.advance(&sequence, 1, 4).unwrap().len(), 1);
        assert_eq!(transport.advance(&sequence, 5, 4).unwrap().len(), 0);
        assert_eq!(transport.state(), TransportState::Finished);
        transport.stop();
        assert_eq!(transport, SequenceTransport::default());
    }

    #[test]
    fn dispatch_limit_failure_does_not_partially_advance_transport() {
        let sequence = fixture();
        let mut transport = SequenceTransport::default();
        transport.start();
        let before = transport.clone();
        assert!(matches!(
            transport.advance(&sequence, 10, 1),
            Err(AudioValueError::SequenceEventLimitExceeded { .. })
        ));
        assert_eq!(transport, before);
    }
}
