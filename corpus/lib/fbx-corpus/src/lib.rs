//! Bounded, example-side structural evidence for the FBX corpus.
//!
//! This crate intentionally stops at FBX-native source records. It does not
//! define Tokimu model, mesh, animation, or rendering contracts.

mod animation;
mod ascii;
mod binary;
mod comparison;
mod error;
mod geometry;
mod material;
mod morph;
mod skeleton;
mod skinning;
mod source;
mod transform;

pub use animation::{
    animation_json, resolve_animations, FbxAnimationChannel, FbxAnimationEvidence,
    FbxAnimationLayer, FbxAnimationStack,
};
pub use ascii::{decode_ascii_fbx, decode_ascii_fbx_file, FbxAsciiDocument};
pub use binary::{
    decode_binary_fbx, decode_binary_fbx_file, source_records_json, FbxBinaryDocument,
    FbxByteOrder, FbxLimits, FbxProperty, FbxRecord, FbxRecordDocument,
};
pub use comparison::{
    compare_static_observations, comparison_json, FbxComparisonDifference, FbxComparisonReport,
    STATIC_OBSERVATION_COMPARISON_ALGORITHM, STATIC_OBSERVATION_COMPARISON_SCHEMA,
};
pub use error::{FbxError, FbxResult};
pub use geometry::{
    bounds_json, lower_static_geometry, meshes_json, topology_json, FbxBounds, FbxGeometryEvidence,
    FbxMaterialLayer, FbxNormalLayer, FbxPolygon, FbxStaticMesh, FbxUvLayer,
};
pub use material::{
    material_bindings_json, material_objects_json, material_slots_json, resolve_material_slots,
    resolve_materials, FbxMaterialBinding, FbxMaterialEvidence, FbxMaterialProperty,
    FbxMaterialSlotAssignment, FbxSourceMaterial, FbxSourceTexture,
};
pub use morph::{
    morph_json, resolve_morphs, FbxBlendShape, FbxBlendShapeChannel, FbxMorphEvidence,
    FbxMorphTarget,
};
pub use skeleton::{resolve_skeletons, skeleton_json, FbxSkeletonEvidence, FbxSkeletonJoint};
pub use skinning::{
    resolve_skins, skinning_json, FbxSkinCluster, FbxSkinEvidence, FbxSkinWeightSummary,
    FbxSourceSkin,
};
pub use source::{
    connections_json, objects_json, resolve_source_scene, source_scene_json, FbxConnection,
    FbxSourceDiagnostic, FbxSourceObject, FbxSourceScene, FbxSourceSceneNode,
};
pub use transform::{
    resolve_transforms, transforms_json, FbxAxisMetadata, FbxNodeTransform, FbxTransformEvidence,
};
