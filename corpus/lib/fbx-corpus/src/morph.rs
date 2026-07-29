use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    FbxBinaryDocument, FbxError, FbxGeometryEvidence, FbxProperty, FbxRecord, FbxResult,
    FbxSourceScene,
};

/// Provider-local blend-shape evidence. It preserves FBX target topology and
/// source position vectors without defining Tokimu morph weights or evaluation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxMorphEvidence {
    pub blend_shapes: Vec<FbxBlendShape>,
    pub channels: Vec<FbxBlendShapeChannel>,
    pub targets: Vec<FbxMorphTarget>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FbxBlendShape {
    pub source_id: i64,
    pub geometry_id: i64,
    pub channel_ids: Vec<i64>,
    pub source_offset: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FbxBlendShapeChannel {
    pub source_id: i64,
    pub blend_shape_id: i64,
    pub target_ids: Vec<i64>,
    pub source_offset: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxMorphTarget {
    pub source_id: i64,
    pub channel_id: i64,
    pub control_point_indices: Vec<u32>,
    /// Raw `Shape.Vertices` triples paired with `Indexes`.
    ///
    /// This evidence intentionally does not claim whether a given exporter
    /// encoded absolute target positions or deformation deltas. That requires
    /// a later evaluated-morph slice against the base geometry.
    pub position_values: Vec<[f64; 3]>,
    pub source_offset: usize,
}

pub fn resolve_morphs(
    document: &FbxBinaryDocument,
    scene: &FbxSourceScene,
    geometry: &FbxGeometryEvidence,
) -> FbxResult<FbxMorphEvidence> {
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
    let blend_shape_ids = scene
        .objects
        .iter()
        .filter(|object| object.kind == "Deformer" && object.class == "BlendShape")
        .map(|object| object.source_id)
        .collect::<BTreeSet<_>>();
    let channel_ids = scene
        .objects
        .iter()
        .filter(|object| object.kind == "Deformer" && object.class == "BlendShapeChannel")
        .map(|object| object.source_id)
        .collect::<BTreeSet<_>>();
    let target_ids = scene
        .objects
        .iter()
        .filter(|object| object.kind == "Geometry" && object.class == "Shape")
        .map(|object| object.source_id)
        .collect::<BTreeSet<_>>();

    let blend_to_geometry = scene
        .connections
        .iter()
        .filter(|connection| {
            connection.relation == "OO"
                && blend_shape_ids.contains(&connection.child_id)
                && geometry_points.contains_key(&connection.parent_id)
        })
        .map(|connection| (connection.child_id, connection.parent_id))
        .collect::<BTreeMap<_, _>>();
    let channel_to_blend = scene
        .connections
        .iter()
        .filter(|connection| {
            connection.relation == "OO"
                && channel_ids.contains(&connection.child_id)
                && blend_shape_ids.contains(&connection.parent_id)
        })
        .map(|connection| (connection.child_id, connection.parent_id))
        .collect::<BTreeMap<_, _>>();
    let target_to_channel = scene
        .connections
        .iter()
        .filter(|connection| {
            connection.relation == "OO"
                && target_ids.contains(&connection.child_id)
                && channel_ids.contains(&connection.parent_id)
        })
        .map(|connection| (connection.child_id, connection.parent_id))
        .collect::<BTreeMap<_, _>>();

    let mut blend_shapes = Vec::new();
    let mut channels = Vec::new();
    let mut targets = Vec::new();
    for blend_shape_id in blend_shape_ids {
        let record = records.get(&blend_shape_id).ok_or_else(|| {
            morph_error(
                0,
                format!("missing source record for blend shape {blend_shape_id}"),
            )
        })?;
        let geometry_id = blend_to_geometry
            .get(&blend_shape_id)
            .copied()
            .ok_or_else(|| {
                morph_error(
                    record.source_offset,
                    format!("blend shape {blend_shape_id} has no base geometry connection"),
                )
            })?;
        let point_count = *geometry_points.get(&geometry_id).ok_or_else(|| {
            morph_error(
                record.source_offset,
                format!("blend shape {blend_shape_id} references unknown geometry {geometry_id}"),
            )
        })?;
        let mut attached_channels = channel_to_blend
            .iter()
            .filter_map(|(channel_id, blend)| (*blend == blend_shape_id).then_some(*channel_id))
            .collect::<Vec<_>>();
        attached_channels.sort_unstable();
        if attached_channels.is_empty() {
            return Err(morph_error(
                record.source_offset,
                format!("blend shape {blend_shape_id} has no channel connections"),
            ));
        }
        for channel_id in &attached_channels {
            let channel_record = records.get(channel_id).ok_or_else(|| {
                morph_error(
                    0,
                    format!("missing source record for blend-shape channel {channel_id}"),
                )
            })?;
            let mut attached_targets = target_to_channel
                .iter()
                .filter_map(|(target_id, channel)| (*channel == *channel_id).then_some(*target_id))
                .collect::<Vec<_>>();
            attached_targets.sort_unstable();
            if attached_targets.is_empty() {
                return Err(morph_error(
                    channel_record.source_offset,
                    format!("blend-shape channel {channel_id} has no target connections"),
                ));
            }
            for target_id in &attached_targets {
                let target_record = records.get(target_id).ok_or_else(|| {
                    morph_error(
                        0,
                        format!("missing source record for morph target {target_id}"),
                    )
                })?;
                let indices = i32_array(target_record, "Indexes")?;
                let values = f64_array(target_record, "Vertices")?;
                if values.len() % 3 != 0 {
                    return Err(morph_error(
                        target_record.source_offset,
                        format!(
                            "morph target {target_id} has {} delta values, not a multiple of three",
                            values.len()
                        ),
                    ));
                }
                let position_values = values
                    .chunks_exact(3)
                    .map(|delta| [delta[0], delta[1], delta[2]])
                    .collect::<Vec<_>>();
                if indices.len() != position_values.len() {
                    return Err(morph_error(
                        target_record.source_offset,
                        format!(
                            "morph target {target_id} has {} indices and {} position vectors",
                            indices.len(),
                            position_values.len()
                        ),
                    ));
                }
                let control_point_indices = indices
                    .into_iter()
                    .map(|index| {
                        let index = u32::try_from(index).map_err(|_| {
                            morph_error(target_record.source_offset, "morph target index is negative")
                        })?;
                        if index as usize >= point_count {
                            return Err(morph_error(
                                target_record.source_offset,
                                format!("morph target index {index} exceeds base control-point count {point_count}"),
                            ));
                        }
                        Ok(index)
                    })
                    .collect::<FbxResult<Vec<_>>>()?;
                if position_values
                    .iter()
                    .flatten()
                    .any(|value| !value.is_finite())
                {
                    return Err(morph_error(
                        target_record.source_offset,
                        "morph target position vectors contain a non-finite value",
                    ));
                }
                targets.push(FbxMorphTarget {
                    source_id: *target_id,
                    channel_id: *channel_id,
                    control_point_indices,
                    position_values,
                    source_offset: target_record.source_offset,
                });
            }
            channels.push(FbxBlendShapeChannel {
                source_id: *channel_id,
                blend_shape_id,
                target_ids: attached_targets,
                source_offset: channel_record.source_offset,
            });
        }
        blend_shapes.push(FbxBlendShape {
            source_id: blend_shape_id,
            geometry_id,
            channel_ids: attached_channels,
            source_offset: record.source_offset,
        });
    }
    blend_shapes.sort_by_key(|blend_shape| blend_shape.source_id);
    channels.sort_by_key(|channel| channel.source_id);
    targets.sort_by_key(|target| target.source_id);
    Ok(FbxMorphEvidence {
        blend_shapes,
        channels,
        targets,
    })
}

pub fn morph_json(evidence: &FbxMorphEvidence) -> FbxResult<String> {
    Ok(serde_json::to_string_pretty(evidence)?)
}

fn top_level<'a>(document: &'a FbxBinaryDocument, name: &str) -> FbxResult<&'a FbxRecord> {
    document
        .records
        .iter()
        .find(|record| record.name == name)
        .ok_or_else(|| morph_error(0, format!("missing top-level `{name}` record")))
}

fn record_id(record: &FbxRecord) -> Option<i64> {
    match record.properties.first() {
        Some(FbxProperty::I64(value)) => Some(*value),
        _ => None,
    }
}

fn i32_array(record: &FbxRecord, name: &str) -> FbxResult<Vec<i32>> {
    match child(record, name)?.properties.first() {
        Some(FbxProperty::I32Array(values)) => Ok(values.clone()),
        _ => Err(morph_error(
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
        _ => Err(morph_error(
            record.source_offset,
            format!("`{name}` is not a floating-point array"),
        )),
    }
}

fn child<'a>(record: &'a FbxRecord, name: &str) -> FbxResult<&'a FbxRecord> {
    record
        .children
        .iter()
        .find(|child| child.name == name)
        .ok_or_else(|| morph_error(record.source_offset, format!("missing `{name}` record")))
}

fn morph_error(offset: usize, reason: impl Into<String>) -> FbxError {
    FbxError::Morph {
        offset,
        reason: reason.into(),
    }
}
