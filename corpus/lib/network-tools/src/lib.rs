//! Incubating, provider-neutral networking contracts for corpus examples.
//!
//! Applications own payload meaning. This crate owns only bounded envelope,
//! codec, sequence, and transport proof machinery.

mod codec;
mod envelope;
mod sequence;
mod transport;

pub use codec::{decode_payload, encode_payload, CodecError, JsonEnvelopeCodec};
pub use envelope::{
    MessageKind, ReplicationEnvelope, SchemaIdentity, DEFAULT_FRAME_LIMIT, DEFAULT_PAYLOAD_LIMIT,
    PROTOCOL_VERSION,
};
pub use sequence::{ObservationSequenceTracker, SequenceDecision};
pub use transport::{InjectedFailure, LoopbackTransport, Transport, TransportError};

#[cfg(test)]
mod tests;
