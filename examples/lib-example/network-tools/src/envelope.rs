use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u16 = 1;
pub const DEFAULT_FRAME_LIMIT: usize = 64 * 1024;
pub const DEFAULT_PAYLOAD_LIMIT: usize = 60 * 1024;
pub(crate) const MAX_SCHEMA_ID_BYTES: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaIdentity {
    pub id: String,
    pub version: u16,
}

impl SchemaIdentity {
    pub fn new(id: impl Into<String>, version: u16) -> Self {
        Self {
            id: id.into(),
            version,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageKind {
    ObservationSnapshot,
    ClientInput,
}

impl MessageKind {
    pub(crate) fn wire_name(self) -> &'static str {
        match self {
            Self::ObservationSnapshot => "observation_snapshot",
            Self::ClientInput => "client_input",
        }
    }

    pub(crate) fn from_wire_name(value: &str) -> Option<Self> {
        match value {
            "observation_snapshot" => Some(Self::ObservationSnapshot),
            "client_input" => Some(Self::ClientInput),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplicationEnvelope {
    pub protocol_version: u16,
    pub schema: SchemaIdentity,
    pub sequence: u64,
    pub message_kind: MessageKind,
    pub payload: Vec<u8>,
}

impl ReplicationEnvelope {
    pub fn new(
        schema: SchemaIdentity,
        sequence: u64,
        message_kind: MessageKind,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            schema,
            sequence,
            message_kind,
            payload,
        }
    }

    pub fn observation(schema: SchemaIdentity, sequence: u64, payload: Vec<u8>) -> Self {
        Self::new(schema, sequence, MessageKind::ObservationSnapshot, payload)
    }

    pub fn client_input(schema: SchemaIdentity, sequence: u64, payload: Vec<u8>) -> Self {
        Self::new(schema, sequence, MessageKind::ClientInput, payload)
    }
}
