//! Bounded, example-side structural evidence for the FBX corpus.
//!
//! This crate intentionally stops at FBX-native source records. It does not
//! define Tokimu model, mesh, animation, or rendering contracts.

mod binary;
mod error;
mod geometry;
mod material;
mod source;
mod transform;

pub use binary::{
    decode_binary_fbx, decode_binary_fbx_file, source_records_json, FbxBinaryDocument, FbxLimits,
    FbxProperty, FbxRecord,
};
pub use error::{FbxError, FbxResult};
pub use geometry::{
    bounds_json, lower_static_geometry, meshes_json, topology_json, FbxBounds, FbxGeometryEvidence,
    FbxNormalLayer, FbxPolygon, FbxStaticMesh, FbxUvLayer,
};
pub use material::{
    material_bindings_json, material_objects_json, resolve_materials, FbxMaterialBinding,
    FbxMaterialEvidence, FbxMaterialProperty, FbxSourceMaterial, FbxSourceTexture,
};
pub use source::{
    connections_json, objects_json, resolve_source_scene, source_scene_json, FbxConnection,
    FbxSourceDiagnostic, FbxSourceObject, FbxSourceScene, FbxSourceSceneNode,
};
pub use transform::{
    resolve_transforms, transforms_json, FbxAxisMetadata, FbxNodeTransform, FbxTransformEvidence,
};
