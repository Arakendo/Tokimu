use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

use crate::envelope::{
    MessageKind, ReplicationEnvelope, SchemaIdentity, DEFAULT_FRAME_LIMIT, DEFAULT_PAYLOAD_LIMIT,
    MAX_SCHEMA_ID_BYTES, PROTOCOL_VERSION,
};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CodecError {
    #[error("schema identity must not be empty")]
    EmptySchemaId,
    #[error("schema identity is {actual} bytes; maximum is {limit}")]
    SchemaIdTooLong { actual: usize, limit: usize },
    #[error("payload is {actual} bytes; maximum is {limit}")]
    PayloadTooLarge { actual: usize, limit: usize },
    #[error("frame is {actual} bytes; maximum is {limit}")]
    FrameTooLarge { actual: usize, limit: usize },
    #[error("unsupported protocol version {found}; expected {expected}")]
    UnsupportedProtocolVersion { expected: u16, found: u16 },
    #[error(
        "unsupported schema {found_id}@{found_version}; expected {expected_id}@{expected_version}"
    )]
    UnsupportedSchema {
        expected_id: String,
        expected_version: u16,
        found_id: String,
        found_version: u16,
    },
    #[error("unknown message kind `{0}`")]
    UnknownMessageKind(String),
    #[error("malformed envelope: {0}")]
    MalformedEnvelope(String),
    #[error("malformed application payload: {0}")]
    MalformedPayload(String),
}

#[derive(Clone, Debug)]
pub struct JsonEnvelopeCodec {
    expected_schema: SchemaIdentity,
    frame_limit: usize,
    payload_limit: usize,
}

impl JsonEnvelopeCodec {
    pub fn new(expected_schema: SchemaIdentity) -> Self {
        Self {
            expected_schema,
            frame_limit: DEFAULT_FRAME_LIMIT,
            payload_limit: DEFAULT_PAYLOAD_LIMIT,
        }
    }

    pub fn with_limits(mut self, frame_limit: usize, payload_limit: usize) -> Self {
        self.frame_limit = frame_limit;
        self.payload_limit = payload_limit;
        self
    }

    pub fn encode(&self, envelope: &ReplicationEnvelope) -> Result<Vec<u8>, CodecError> {
        self.validate(envelope)?;
        let wire = WireEnvelope {
            protocol_version: envelope.protocol_version,
            schema_id: envelope.schema.id.clone(),
            schema_version: envelope.schema.version,
            sequence: envelope.sequence,
            message_kind: envelope.message_kind.wire_name().to_owned(),
            payload: envelope.payload.clone(),
        };
        let frame = serde_json::to_vec(&wire)
            .map_err(|error| CodecError::MalformedEnvelope(error.to_string()))?;
        if frame.len() > self.frame_limit {
            return Err(CodecError::FrameTooLarge {
                actual: frame.len(),
                limit: self.frame_limit,
            });
        }
        Ok(frame)
    }

    pub fn decode(&self, frame: &[u8]) -> Result<ReplicationEnvelope, CodecError> {
        if frame.len() > self.frame_limit {
            return Err(CodecError::FrameTooLarge {
                actual: frame.len(),
                limit: self.frame_limit,
            });
        }
        let wire: WireEnvelope = serde_json::from_slice(frame)
            .map_err(|error| CodecError::MalformedEnvelope(error.to_string()))?;
        let message_kind = MessageKind::from_wire_name(&wire.message_kind)
            .ok_or_else(|| CodecError::UnknownMessageKind(wire.message_kind.clone()))?;
        let envelope = ReplicationEnvelope {
            protocol_version: wire.protocol_version,
            schema: SchemaIdentity::new(wire.schema_id, wire.schema_version),
            sequence: wire.sequence,
            message_kind,
            payload: wire.payload,
        };
        self.validate(&envelope)?;
        Ok(envelope)
    }

    fn validate(&self, envelope: &ReplicationEnvelope) -> Result<(), CodecError> {
        if envelope.protocol_version != PROTOCOL_VERSION {
            return Err(CodecError::UnsupportedProtocolVersion {
                expected: PROTOCOL_VERSION,
                found: envelope.protocol_version,
            });
        }
        if envelope.schema.id.is_empty() {
            return Err(CodecError::EmptySchemaId);
        }
        if envelope.schema.id.len() > MAX_SCHEMA_ID_BYTES {
            return Err(CodecError::SchemaIdTooLong {
                actual: envelope.schema.id.len(),
                limit: MAX_SCHEMA_ID_BYTES,
            });
        }
        if envelope.schema != self.expected_schema {
            return Err(CodecError::UnsupportedSchema {
                expected_id: self.expected_schema.id.clone(),
                expected_version: self.expected_schema.version,
                found_id: envelope.schema.id.clone(),
                found_version: envelope.schema.version,
            });
        }
        if envelope.payload.len() > self.payload_limit {
            return Err(CodecError::PayloadTooLarge {
                actual: envelope.payload.len(),
                limit: self.payload_limit,
            });
        }
        Ok(())
    }
}

pub fn encode_payload<T: Serialize>(payload: &T, limit: usize) -> Result<Vec<u8>, CodecError> {
    let bytes = serde_json::to_vec(payload)
        .map_err(|error| CodecError::MalformedPayload(error.to_string()))?;
    if bytes.len() > limit {
        return Err(CodecError::PayloadTooLarge {
            actual: bytes.len(),
            limit,
        });
    }
    Ok(bytes)
}

pub fn decode_payload<T: DeserializeOwned>(payload: &[u8], limit: usize) -> Result<T, CodecError> {
    if payload.len() > limit {
        return Err(CodecError::PayloadTooLarge {
            actual: payload.len(),
            limit,
        });
    }
    serde_json::from_slice(payload).map_err(|error| CodecError::MalformedPayload(error.to_string()))
}

#[derive(Debug, Deserialize, Serialize)]
struct WireEnvelope {
    protocol_version: u16,
    schema_id: String,
    schema_version: u16,
    sequence: u64,
    message_kind: String,
    payload: Vec<u8>,
}
