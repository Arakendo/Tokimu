use serde::{Deserialize, Serialize};

use crate::{FbxResult, FbxSkinEvidence, FbxSourceScene};

/// Provider-local skeleton evidence reconstructed from FBX `LimbNode` models
/// and the skin clusters that reference them.
///
/// This preserves source hierarchy and cluster relationships for inspection.
/// It does not define a Tokimu skeleton, bind pose, or runtime deformation
/// contract.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FbxSkeletonEvidence {
    pub joints: Vec<FbxSkeletonJoint>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FbxSkeletonJoint {
    pub source_id: i64,
    pub name: String,
    pub parent_joint_source_id: Option<i64>,
    pub cluster_ids: Vec<i64>,
    pub source_offset: usize,
}

pub fn resolve_skeletons(
    scene: &FbxSourceScene,
    skins: &FbxSkinEvidence,
) -> FbxResult<FbxSkeletonEvidence> {
    let joint_ids = scene
        .nodes
        .iter()
        .filter(|node| node.class == "LimbNode")
        .map(|node| node.source_id)
        .collect::<std::collections::BTreeSet<_>>();
    let mut joints = scene
        .nodes
        .iter()
        .filter(|node| joint_ids.contains(&node.source_id))
        .map(|node| FbxSkeletonJoint {
            source_id: node.source_id,
            name: node.name.clone(),
            parent_joint_source_id: node
                .parent_model_id
                .filter(|parent| joint_ids.contains(parent)),
            cluster_ids: skins
                .clusters
                .iter()
                .filter(|cluster| cluster.joint_model_id == node.source_id)
                .map(|cluster| cluster.source_id)
                .collect(),
            source_offset: node.source_offset,
        })
        .collect::<Vec<_>>();
    joints.sort_by_key(|joint| joint.source_id);
    Ok(FbxSkeletonEvidence { joints })
}

pub fn skeleton_json(evidence: &FbxSkeletonEvidence) -> FbxResult<String> {
    Ok(serde_json::to_string_pretty(evidence)?)
}
