//! Bounded, example-side structural evidence for the FBX corpus.
//!
//! This crate intentionally stops at FBX-native source records. It does not
//! define Tokimu model, mesh, animation, or rendering contracts.

mod binary;
mod error;

pub use binary::{
    decode_binary_fbx, decode_binary_fbx_file, source_records_json, FbxBinaryDocument, FbxLimits,
    FbxProperty, FbxRecord,
};
pub use error::{FbxError, FbxResult};
