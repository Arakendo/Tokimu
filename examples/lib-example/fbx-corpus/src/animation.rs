use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{FbxBinaryDocument, FbxError, FbxProperty, FbxRecord, FbxResult, FbxSourceScene};

/// Provider-local animation evidence. Times are retained as native FBX ticks;
/// this layer does not define Tokimu playback or interpolation semantics.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxAnimationEvidence {
    pub stacks: Vec<FbxAnimationStack>,
    pub layers: Vec<FbxAnimationLayer>,
    pub channels: Vec<FbxAnimationChannel>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FbxAnimationStack {
    pub source_id: i64,
    pub name: String,
    pub source_offset: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FbxAnimationLayer {
    pub source_id: i64,
    pub name: String,
    pub stack_id: Option<i64>,
    pub source_offset: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxAnimationChannel {
    pub curve_id: i64,
    pub curve_node_id: i64,
    pub stack_id: Option<i64>,
    pub layer_id: Option<i64>,
    pub target_id: Option<i64>,
    pub target_property: Option<String>,
    pub component: Option<String>,
    pub key_times: Vec<i64>,
    pub key_values: Vec<f64>,
    /// Raw FBX key attribute flags, when present in the source record.
    ///
    /// The flags retain provider-local interpolation intent for corpus
    /// inspection. This evidence does not define Tokimu playback behavior.
    pub key_attr_flags: Option<Vec<i32>>,
    pub source_offset: usize,
}

pub fn resolve_animations(
    document: &FbxBinaryDocument,
    scene: &FbxSourceScene,
) -> FbxResult<FbxAnimationEvidence> {
    let objects = top_level(document, "Objects")?;
    let records = objects
        .children
        .iter()
        .filter_map(|record| record_id(record).map(|id| (id, record)))
        .collect::<BTreeMap<_, _>>();
    let objects_by_id = scene
        .objects
        .iter()
        .map(|object| (object.source_id, object))
        .collect::<BTreeMap<_, _>>();
    let stacks = scene
        .objects
        .iter()
        .filter(|object| object.kind == "AnimationStack")
        .map(|object| FbxAnimationStack {
            source_id: object.source_id,
            name: object.name.clone(),
            source_offset: object.source_offset,
        })
        .collect::<Vec<_>>();
    let stack_ids = stacks
        .iter()
        .map(|stack| stack.source_id)
        .collect::<BTreeSet<_>>();
    let layer_to_stack = scene
        .connections
        .iter()
        .filter(|connection| {
            connection.relation == "OO" && stack_ids.contains(&connection.parent_id)
        })
        .map(|connection| (connection.child_id, connection.parent_id))
        .collect::<BTreeMap<_, _>>();
    let layers = scene
        .objects
        .iter()
        .filter(|object| object.kind == "AnimationLayer")
        .map(|object| FbxAnimationLayer {
            source_id: object.source_id,
            name: object.name.clone(),
            stack_id: layer_to_stack.get(&object.source_id).copied(),
            source_offset: object.source_offset,
        })
        .collect::<Vec<_>>();
    let layer_ids = layers
        .iter()
        .map(|layer| layer.source_id)
        .collect::<BTreeSet<_>>();
    let curve_nodes = scene
        .objects
        .iter()
        .filter(|object| object.kind == "AnimationCurveNode")
        .map(|object| object.source_id)
        .collect::<BTreeSet<_>>();
    let curves = scene
        .objects
        .iter()
        .filter(|object| object.kind == "AnimationCurve")
        .map(|object| object.source_id)
        .collect::<BTreeSet<_>>();

    let curve_to_node = scene
        .connections
        .iter()
        .filter(|connection| {
            curves.contains(&connection.child_id) && curve_nodes.contains(&connection.parent_id)
        })
        .map(|connection| {
            (
                connection.child_id,
                (connection.parent_id, connection.property.clone()),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let node_to_layer = scene
        .connections
        .iter()
        .filter(|connection| {
            curve_nodes.contains(&connection.child_id) && layer_ids.contains(&connection.parent_id)
        })
        .map(|connection| (connection.child_id, connection.parent_id))
        .collect::<BTreeMap<_, _>>();
    let node_targets = scene
        .connections
        .iter()
        .filter(|connection| {
            curve_nodes.contains(&connection.child_id)
                && objects_by_id.contains_key(&connection.parent_id)
        })
        .filter(|connection| !layer_ids.contains(&connection.parent_id))
        .map(|connection| {
            (
                connection.child_id,
                (connection.parent_id, connection.property.clone()),
            )
        })
        .collect::<BTreeMap<_, _>>();

    let mut channels = Vec::new();
    for curve_id in curves {
        let (node_id, component) = curve_to_node.get(&curve_id).cloned().ok_or_else(|| {
            animation_error(
                0,
                format!("animation curve {curve_id} has no curve-node connection"),
            )
        })?;
        let record = records.get(&curve_id).ok_or_else(|| {
            animation_error(
                0,
                format!("missing source record for animation curve {curve_id}"),
            )
        })?;
        let key_times = i64_array(record, "KeyTime")?;
        let key_values = key_values(record)?;
        if key_times.len() != key_values.len() {
            return Err(animation_error(
                record.source_offset,
                format!(
                    "animation curve {curve_id} has {} times and {} values",
                    key_times.len(),
                    key_values.len()
                ),
            ));
        }
        if key_times.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(animation_error(
                record.source_offset,
                "animation key times are not strictly ordered",
            ));
        }
        if key_values.iter().any(|value| !value.is_finite()) {
            return Err(animation_error(
                record.source_offset,
                "animation curve has non-finite values",
            ));
        }
        // FBX exporters do not consistently encode one attribute flag per
        // key. Preserve the raw array without assigning it per-key meaning.
        let key_attr_flags = optional_i32_array(record, "KeyAttrFlags")?;
        let layer_id = node_to_layer.get(&node_id).copied();
        let stack_id = layer_id.and_then(|layer| layer_to_stack.get(&layer).copied());
        let (target_id, target_property) = node_targets
            .get(&node_id)
            .map(|(target, property)| (Some(*target), property.clone()))
            .unwrap_or((None, None));
        channels.push(FbxAnimationChannel {
            curve_id,
            curve_node_id: node_id,
            stack_id,
            layer_id,
            target_id,
            target_property,
            component,
            key_times,
            key_values,
            key_attr_flags,
            source_offset: record.source_offset,
        });
    }
    channels.sort_by_key(|channel| channel.curve_id);
    Ok(FbxAnimationEvidence {
        stacks,
        layers,
        channels,
    })
}

pub fn animation_json(evidence: &FbxAnimationEvidence) -> FbxResult<String> {
    Ok(serde_json::to_string_pretty(evidence)?)
}

fn top_level<'a>(document: &'a FbxBinaryDocument, name: &str) -> FbxResult<&'a FbxRecord> {
    document
        .records
        .iter()
        .find(|record| record.name == name)
        .ok_or_else(|| animation_error(0, format!("missing top-level `{name}` record")))
}

fn record_id(record: &FbxRecord) -> Option<i64> {
    match record.properties.first() {
        Some(FbxProperty::I64(value)) => Some(*value),
        _ => None,
    }
}

fn i64_array(record: &FbxRecord, name: &str) -> FbxResult<Vec<i64>> {
    let child = record
        .children
        .iter()
        .find(|child| child.name == name)
        .ok_or_else(|| animation_error(record.source_offset, format!("missing `{name}` record")))?;
    match child.properties.first() {
        Some(FbxProperty::I64Array(values)) => Ok(values.clone()),
        _ => Err(animation_error(
            child.source_offset,
            format!("`{name}` is not an I64 array"),
        )),
    }
}

fn optional_i32_array(record: &FbxRecord, name: &str) -> FbxResult<Option<Vec<i32>>> {
    let Some(child) = record.children.iter().find(|child| child.name == name) else {
        return Ok(None);
    };
    match child.properties.first() {
        Some(FbxProperty::I32Array(values)) => Ok(Some(values.clone())),
        _ => Err(animation_error(
            child.source_offset,
            format!("`{name}` is not an I32 array"),
        )),
    }
}

fn key_values(record: &FbxRecord) -> FbxResult<Vec<f64>> {
    let child = record
        .children
        .iter()
        .find(|child| child.name == "KeyValueFloat")
        .ok_or_else(|| animation_error(record.source_offset, "missing `KeyValueFloat` record"))?;
    match child.properties.first() {
        Some(FbxProperty::F32Array(values)) => {
            Ok(values.iter().map(|value| *value as f64).collect())
        }
        Some(FbxProperty::F64Array(values)) => Ok(values.clone()),
        _ => Err(animation_error(
            child.source_offset,
            "`KeyValueFloat` is not a floating-point array",
        )),
    }
}

fn animation_error(offset: usize, reason: impl Into<String>) -> FbxError {
    FbxError::Animation {
        offset,
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_monotonic_key_times() {
        let record = FbxRecord {
            name: "AnimationCurve".into(),
            source_offset: 1,
            end_offset: 2,
            property_byte_length: 0,
            properties: vec![],
            children: vec![],
        };
        assert!(matches!(
            i64_array(&record, "KeyTime"),
            Err(FbxError::Animation { .. })
        ));
    }
}
