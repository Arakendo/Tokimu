use crate::ReplicationEnvelope;

/// Per-observation-stream sequence policy.
///
/// A tracker accepts monotonic observations, reports gaps, and ignores frames
/// that cannot advance the accepted sequence. It does not buffer, recover, or
/// imply reliable transport delivery.
#[derive(Debug, Default)]
pub struct ObservationSequenceTracker {
    last_accepted: Option<u64>,
}

impl ObservationSequenceTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn last_accepted(&self) -> Option<u64> {
        self.last_accepted
    }

    pub fn observe(&mut self, envelope: &ReplicationEnvelope) -> SequenceDecision {
        let schema_id = envelope.schema.id.clone();
        let schema_version = envelope.schema.version;
        let received = envelope.sequence;

        match self.last_accepted {
            None => {
                self.last_accepted = Some(received);
                SequenceDecision::AcceptedFirst {
                    schema_id,
                    schema_version,
                    received,
                }
            }
            Some(last_accepted) if received == last_accepted => {
                SequenceDecision::IgnoredDuplicate {
                    schema_id,
                    schema_version,
                    received,
                    last_accepted,
                }
            }
            Some(last_accepted) if received < last_accepted => {
                SequenceDecision::IgnoredStaleOrOutOfOrder {
                    schema_id,
                    schema_version,
                    received,
                    last_accepted,
                }
            }
            Some(last_accepted) => {
                // This branch is reachable only when received is greater than
                // last_accepted. Saturating addition cannot turn sequence
                // exhaustion into a wrapped in-order value.
                let expected = last_accepted.saturating_add(1);
                self.last_accepted = Some(received);
                if received == expected {
                    SequenceDecision::AcceptedInOrder {
                        schema_id,
                        schema_version,
                        received,
                    }
                } else {
                    SequenceDecision::AcceptedWithGap {
                        schema_id,
                        schema_version,
                        expected,
                        received,
                    }
                }
            }
        }
    }
}

/// A bounded diagnostic outcome for one envelope sequence observation.
///
/// Schema identity and numbers identify the frame without logging its payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SequenceDecision {
    AcceptedFirst {
        schema_id: String,
        schema_version: u16,
        received: u64,
    },
    AcceptedInOrder {
        schema_id: String,
        schema_version: u16,
        received: u64,
    },
    AcceptedWithGap {
        schema_id: String,
        schema_version: u16,
        expected: u64,
        received: u64,
    },
    IgnoredDuplicate {
        schema_id: String,
        schema_version: u16,
        received: u64,
        last_accepted: u64,
    },
    IgnoredStaleOrOutOfOrder {
        schema_id: String,
        schema_version: u16,
        received: u64,
        last_accepted: u64,
    },
}

impl SequenceDecision {
    pub fn is_accepted(&self) -> bool {
        matches!(
            self,
            Self::AcceptedFirst { .. }
                | Self::AcceptedInOrder { .. }
                | Self::AcceptedWithGap { .. }
        )
    }

    pub fn schema_id(&self) -> &str {
        match self {
            Self::AcceptedFirst { schema_id, .. }
            | Self::AcceptedInOrder { schema_id, .. }
            | Self::AcceptedWithGap { schema_id, .. }
            | Self::IgnoredDuplicate { schema_id, .. }
            | Self::IgnoredStaleOrOutOfOrder { schema_id, .. } => schema_id,
        }
    }

    pub fn schema_version(&self) -> u16 {
        match self {
            Self::AcceptedFirst { schema_version, .. }
            | Self::AcceptedInOrder { schema_version, .. }
            | Self::AcceptedWithGap { schema_version, .. }
            | Self::IgnoredDuplicate { schema_version, .. }
            | Self::IgnoredStaleOrOutOfOrder { schema_version, .. } => *schema_version,
        }
    }
}
