//! Format-specific structural evidence for the Khronos glTF corpus.
//!
//! This crate intentionally stops before Tokimu model or mesh lowering. It
//! makes glTF and GLB source boundaries observable without admitting importer
//! details into an engine capability.

mod decode;
mod error;
mod glb;
mod gltf;
mod summary;

pub use decode::{
    decode_glb, decode_glb_file, decode_gltf, decode_gltf_file, DecodedBounds, DecodedModel,
    DecodedPrimitive, PrimitiveLocation,
};
pub use error::{CorpusError, CorpusResult};
pub use glb::{inspect_glb, inspect_glb_file, GlbChunk, GlbChunkKind, GlbInspection};
pub use gltf::{inspect_gltf, inspect_gltf_file, BufferReference, GltfInspection, ResolvedBuffer};
pub use summary::GltfSummary;
