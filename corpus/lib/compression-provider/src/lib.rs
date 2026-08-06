//! Incubating, provider-neutral byte-compression contracts.
//!
//! This crate owns bounded byte transformation semantics only. Archive entry
//! names, Resource Space identity, host files, and provider-native tuning stay
//! outside this boundary.

mod brotli;
mod detection;
mod flate;
mod model;
#[cfg(test)]
mod seeds;
mod stream;

pub use brotli::BrotliCompressionProvider;
pub use detection::{detect_compression_envelope, CompressionEnvelope};
pub use flate::FlateCompressionProvider;
pub use model::{
    CompressionCodec, CompressionError, CompressionGoal, CompressionObservation,
    CompressionProvider, CompressionResult, DecodeLimits, DecodeRequest, EncodeRequest,
};

#[cfg(test)]
mod tests;
