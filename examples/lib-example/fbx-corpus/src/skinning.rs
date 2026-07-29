use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    FbxError, FbxGeometryEvidence, FbxProperty, FbxRecord, FbxRecordDocument, FbxResult,
    FbxSourceScene,
};

/// Provider-local skinning evidence. It retains FBX clusters and bind matrices
/// for inspection; it does not define Tokimu joint, weight, or evaluation
/// semantics.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxSkinEvidence {
    pub skins: Vec<FbxSourceSkin>,
    pub clusters: Vec<FbxSkinCluster>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxSourceSkin {
    pub source_id: i64,
    pub name: String,
    pub geometry_id: i64,
    pub cluster_ids: Vec<i64>,
    pub skinning_type: Option<String>,
    pub weight_summary: FbxSkinWeightSummary,
    pub source_offset: usize,
}

/// Aggregated source influence totals. These are observations, not a claim
/// that every FBX link mode must normalize weights to one.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxSkinWeightSummary {
    pub control_point_count: usize,
    pub influenced_control_point_count: usize,
    pub minimum_influenced_sum: Option<f64>,
    pub maximum_influenced_sum: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxSkinCluster {
    pub source_id: i64,
    pub skin_id: i64,
    pub joint_model_id: i64,
    /// FBX source link mode retained for corpus evidence. Tokimu does not
    /// evaluate link-mode behavior or normalize it into runtime skinning.
    pub link_mode: Option<String>,
    pub control_point_indices: Vec<u32>,
    pub weights: Vec<f64>,
    pub transform: [f64; 16],
    pub transform_link: [f64; 16],
    pub source_offset: usize,
}

pub fn resolve_skins(
    document: &impl FbxRecordDocument,
    scene: &FbxSourceScene,
    geometry: &FbxGeometryEvidence,
) -> FbxResult<FbxSkinEvidence> {
    let objects = top_level(document, "Objects")?;
    let records = objects
        .children
        .iter()
        .filter_map(|record| record_id(record).map(|id| (id, record)))
        .collect::<BTreeMap<_, _>>();
    let geometry_points = geometry
        .meshes
        .iter()
        .map(|mesh| (mesh.source_id, mesh.control_points.len()))
        .collect::<BTreeMap<_, _>>();
    let skin_ids = scene
        .objects
        .iter()
        .filter(|object| object.kind == "Deformer" && object.class == "Skin")
        .map(|object| object.source_id)
        .collect::<BTreeSet<_>>();
    let cluster_ids = scene
        .objects
        .iter()
        .filter(|object| object.kind == "Deformer" && object.class == "Cluster")
        .map(|object| object.source_id)
        .collect::<BTreeSet<_>>();

    let skin_to_geometry = scene
        .connections
        .iter()
        .filter(|connection| {
            connection.relation == "OO"
                && skin_ids.contains(&connection.child_id)
                && geometry_points.contains_key(&connection.parent_id)
        })
        .map(|connection| (connection.child_id, connection.parent_id))
        .collect::<BTreeMap<_, _>>();
    let cluster_to_skin = scene
        .connections
        .iter()
        .filter(|connection| {
            connection.relation == "OO"
                && cluster_ids.contains(&connection.child_id)
                && skin_ids.contains(&connection.parent_id)
        })
        .map(|connection| (connection.child_id, connection.parent_id))
        .collect::<BTreeMap<_, _>>();
    let cluster_to_joint = scene
        .connections
        .iter()
        .filter(|connection| {
            connection.relation == "OO"
                && cluster_ids.contains(&connection.parent_id)
                && scene
                    .objects
                    .iter()
                    .any(|object| object.source_id == connection.child_id && object.kind == "Model")
        })
        .map(|connection| (connection.parent_id, connection.child_id))
        .collect::<BTreeMap<_, _>>();

    let mut skins = Vec::new();
    let mut clusters = Vec::new();
    for skin_id in skin_ids {
        let record = records.get(&skin_id).ok_or_else(|| {
            skinning_error(0, format!("missing source record for skin {skin_id}"))
        })?;
        let geometry_id = skin_to_geometry.get(&skin_id).copied().ok_or_else(|| {
            skinning_error(
                record.source_offset,
                format!("skin {skin_id} has no geometry connection"),
            )
        })?;
        let point_count = *geometry_points.get(&geometry_id).ok_or_else(|| {
            skinning_error(
                record.source_offset,
                format!("skin {skin_id} references unknown geometry {geometry_id}"),
            )
        })?;
        let mut attached_clusters = cluster_to_skin
            .iter()
            .filter_map(|(cluster_id, attached_skin)| {
                (*attached_skin == skin_id).then_some(*cluster_id)
            })
            .collect::<Vec<_>>();
        attached_clusters.sort_unstable();
        if attached_clusters.is_empty() {
            return Err(skinning_error(
                record.source_offset,
                format!("skin {skin_id} has no cluster connections"),
            ));
        }
        for cluster_id in &attached_clusters {
            let cluster_record = records.get(cluster_id).ok_or_else(|| {
                skinning_error(0, format!("missing source record for cluster {cluster_id}"))
            })?;
            let joint_model_id = cluster_to_joint.get(cluster_id).copied().ok_or_else(|| {
                skinning_error(
                    cluster_record.source_offset,
                    format!("cluster {cluster_id} has no model connection"),
                )
            })?;
            let indices = i32_array(cluster_record, "Indexes")?;
            let weights = f64_array(cluster_record, "Weights")?;
            if indices.len() != weights.len() {
                return Err(skinning_error(
                    cluster_record.source_offset,
                    format!(
                        "cluster {cluster_id} has {} indices and {} weights",
                        indices.len(),
                        weights.len()
                    ),
                ));
            }
            let control_point_indices = indices
                .into_iter()
                .map(|index| {
                    let index = u32::try_from(index).map_err(|_| {
                        skinning_error(cluster_record.source_offset, "cluster index is negative")
                    })?;
                    if index as usize >= point_count {
                        return Err(skinning_error(
                            cluster_record.source_offset,
                            format!("cluster index {index} exceeds geometry control-point count {point_count}"),
                        ));
                    }
                    Ok(index)
                })
                .collect::<FbxResult<Vec<_>>>()?;
            if weights
                .iter()
                .any(|weight| !weight.is_finite() || *weight < 0.0)
            {
                return Err(skinning_error(
                    cluster_record.source_offset,
                    "cluster weights contain a negative or non-finite value",
                ));
            }
            clusters.push(FbxSkinCluster {
                source_id: *cluster_id,
                skin_id,
                joint_model_id,
                link_mode: optional_string(cluster_record, "Link_Mode")?,
                control_point_indices,
                weights,
                transform: matrix(cluster_record, "Transform")?,
                transform_link: matrix(cluster_record, "TransformLink")?,
                source_offset: cluster_record.source_offset,
            });
        }
        let weight_summary = summarize_weights(&clusters, skin_id, point_count);
        skins.push(FbxSourceSkin {
            source_id: skin_id,
            name: object_name(scene, skin_id),
            geometry_id,
            cluster_ids: attached_clusters,
            skinning_type: optional_string(record, "SkinningType")?,
            weight_summary,
            source_offset: record.source_offset,
        });
    }
    clusters.sort_by_key(|cluster| cluster.source_id);
    skins.sort_by_key(|skin| skin.source_id);
    Ok(FbxSkinEvidence { skins, clusters })
}

pub fn skinning_json(evidence: &FbxSkinEvidence) -> FbxResult<String> {
    Ok(serde_json::to_string_pretty(evidence)?)
}

fn summarize_weights(
    clusters: &[FbxSkinCluster],
    skin_id: i64,
    control_point_count: usize,
) -> FbxSkinWeightSummary {
    let mut sums = vec![0.0; control_point_count];
    for cluster in clusters.iter().filter(|cluster| cluster.skin_id == skin_id) {
        for (&index, &weight) in cluster.control_point_indices.iter().zip(&cluster.weights) {
            sums[index as usize] += weight;
        }
    }
    let influenced = sums
        .into_iter()
        .filter(|sum| *sum > 0.0)
        .collect::<Vec<_>>();
    let minimum_influenced_sum = influenced.iter().copied().reduce(f64::min);
    let maximum_influenced_sum = influenced.iter().copied().fold(0.0, f64::max);
    FbxSkinWeightSummary {
        control_point_count,
        influenced_control_point_count: influenced.len(),
        minimum_influenced_sum,
        maximum_influenced_sum,
    }
}

fn top_level<'a>(document: &'a impl FbxRecordDocument, name: &str) -> FbxResult<&'a FbxRecord> {
    document
        .records()
        .iter()
        .find(|record| record.name == name)
        .ok_or_else(|| skinning_error(0, format!("missing top-level `{name}` record")))
}

fn record_id(record: &FbxRecord) -> Option<i64> {
    match record.properties.first() {
        Some(FbxProperty::I64(value)) => Some(*value),
        _ => None,
    }
}

fn object_name(scene: &FbxSourceScene, source_id: i64) -> String {
    scene
        .objects
        .iter()
        .find(|object| object.source_id == source_id)
        .map(|object| object.name.clone())
        .unwrap_or_default()
}

fn i32_array(record: &FbxRecord, name: &str) -> FbxResult<Vec<i32>> {
    match child(record, name)?.properties.first() {
        Some(FbxProperty::I32Array(values)) => Ok(values.clone()),
        _ => Err(skinning_error(
            record.source_offset,
            format!("`{name}` is not an I32 array"),
        )),
    }
}

fn f64_array(record: &FbxRecord, name: &str) -> FbxResult<Vec<f64>> {
    match child(record, name)?.properties.first() {
        Some(FbxProperty::F64Array(values)) => Ok(values.clone()),
        Some(FbxProperty::F32Array(values)) => {
            Ok(values.iter().map(|value| *value as f64).collect())
        }
        _ => Err(skinning_error(
            record.source_offset,
            format!("`{name}` is not a floating-point array"),
        )),
    }
}

fn matrix(record: &FbxRecord, name: &str) -> FbxResult<[f64; 16]> {
    let values = f64_array(record, name)?;
    values.try_into().map_err(|values: Vec<f64>| {
        skinning_error(
            record.source_offset,
            format!("`{name}` has {} values, expected 16", values.len()),
        )
    })
}

fn optional_string(record: &FbxRecord, name: &str) -> FbxResult<Option<String>> {
    let Some(record) = record.children.iter().find(|child| child.name == name) else {
        return Ok(None);
    };
    match record.properties.first() {
        Some(FbxProperty::String(value)) => Ok(Some(value.clone())),
        _ => Err(skinning_error(
            record.source_offset,
            format!("`{name}` is not a string"),
        )),
    }
}

fn child<'a>(record: &'a FbxRecord, name: &str) -> FbxResult<&'a FbxRecord> {
    record
        .children
        .iter()
        .find(|child| child.name == name)
        .ok_or_else(|| skinning_error(record.source_offset, format!("missing `{name}` record")))
}

fn skinning_error(offset: usize, reason: impl Into<String>) -> FbxError {
    FbxError::Skinning {
        offset,
        reason: reason.into(),
    }
}
