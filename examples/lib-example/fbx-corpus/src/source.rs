use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{FbxError, FbxProperty, FbxRecord, FbxRecordDocument, FbxResult};

/// Provider-local object evidence resolved from the binary `Objects` record.
///
/// `kind`, `class`, and `source_id` are deliberately retained as source
/// evidence. They are not Tokimu model identities.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FbxSourceObject {
    pub source_id: i64,
    pub kind: String,
    pub name: String,
    pub class: String,
    pub source_offset: usize,
}

/// A source `Connections/C` record. The relation remains textual so later
/// slices can add FBX-specific connection forms without inventing a false
/// provider-neutral enum.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FbxConnection {
    pub relation: String,
    pub child_id: i64,
    pub parent_id: i64,
    pub property: Option<String>,
    pub source_offset: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FbxSourceSceneNode {
    pub source_id: i64,
    pub name: String,
    pub class: String,
    pub parent_model_id: Option<i64>,
    pub geometry_ids: Vec<i64>,
    pub source_offset: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FbxSourceDiagnostic {
    pub source_offset: usize,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FbxSourceScene {
    pub source_fingerprint: String,
    pub objects: Vec<FbxSourceObject>,
    pub connections: Vec<FbxConnection>,
    pub nodes: Vec<FbxSourceSceneNode>,
    pub diagnostics: Vec<FbxSourceDiagnostic>,
}

pub fn resolve_source_scene(document: &impl FbxRecordDocument) -> FbxResult<FbxSourceScene> {
    let objects_record = unique_top_level_record(document, "Objects")?;
    let connections_record = unique_top_level_record(document, "Connections")?;
    let (objects, diagnostics) = decode_objects(objects_record)?;
    let object_map = object_map(&objects)?;
    let connections = decode_connections(connections_record)?;

    validate_connection_targets(&connections, &object_map)?;
    let nodes = build_scene_nodes(&objects, &connections)?;

    Ok(FbxSourceScene {
        source_fingerprint: document.source_fingerprint().to_owned(),
        objects,
        connections,
        nodes,
        diagnostics,
    })
}

pub fn objects_json(scene: &FbxSourceScene) -> FbxResult<String> {
    Ok(serde_json::to_string_pretty(&scene.objects)?)
}

pub fn connections_json(scene: &FbxSourceScene) -> FbxResult<String> {
    Ok(serde_json::to_string_pretty(&scene.connections)?)
}

pub fn source_scene_json(scene: &FbxSourceScene) -> FbxResult<String> {
    Ok(serde_json::to_string_pretty(scene)?)
}

fn unique_top_level_record<'a>(
    document: &'a impl FbxRecordDocument,
    name: &str,
) -> FbxResult<&'a FbxRecord> {
    let mut matches = document
        .records()
        .iter()
        .filter(|record| record.name == name);
    let Some(record) = matches.next() else {
        return Err(graph_error(
            0,
            format!("missing required top-level `{name}` record"),
        ));
    };
    if matches.next().is_some() {
        return Err(graph_error(
            record.source_offset,
            format!("more than one top-level `{name}` record"),
        ));
    }
    Ok(record)
}

fn decode_objects(
    record: &FbxRecord,
) -> FbxResult<(Vec<FbxSourceObject>, Vec<FbxSourceDiagnostic>)> {
    let mut objects = Vec::new();
    let mut diagnostics = Vec::new();

    for child in &record.children {
        let Some(FbxProperty::I64(source_id)) = child.properties.first() else {
            diagnostics.push(FbxSourceDiagnostic {
                source_offset: child.source_offset,
                message: format!(
                    "ignored `{}` source record because its first property is not an I64 object ID",
                    child.name
                ),
            });
            continue;
        };
        let Some(FbxProperty::String(name)) = child.properties.get(1) else {
            diagnostics.push(FbxSourceDiagnostic {
                source_offset: child.source_offset,
                message: format!(
                    "ignored `{}` object {source_id} because it has no string name",
                    child.name
                ),
            });
            continue;
        };
        let class = match child.properties.get(2) {
            Some(FbxProperty::String(class)) => class.clone(),
            _ => String::new(),
        };
        objects.push(FbxSourceObject {
            source_id: *source_id,
            kind: child.name.clone(),
            name: name.clone(),
            class,
            source_offset: child.source_offset,
        });
    }
    Ok((objects, diagnostics))
}

fn decode_connections(record: &FbxRecord) -> FbxResult<Vec<FbxConnection>> {
    let mut connections = Vec::new();
    for child in &record.children {
        if child.name != "C" {
            return Err(graph_error(
                child.source_offset,
                format!("unsupported connection record `{}`", child.name),
            ));
        }
        let relation = property_string(child, 0, "connection relation")?;
        let child_id = property_id(child, 1, "connection child ID")?;
        let parent_id = property_id(child, 2, "connection parent ID")?;
        let property = match child.properties.get(3) {
            None => None,
            Some(FbxProperty::String(value)) => Some(value.clone()),
            Some(_) => {
                return Err(graph_error(
                    child.source_offset,
                    "connection property must be a string when present",
                ));
            }
        };
        if child.properties.len() > 4 {
            return Err(graph_error(
                child.source_offset,
                "connection profile has more than four properties",
            ));
        }
        connections.push(FbxConnection {
            relation,
            child_id,
            parent_id,
            property,
            source_offset: child.source_offset,
        });
    }
    Ok(connections)
}

fn object_map(objects: &[FbxSourceObject]) -> FbxResult<BTreeMap<i64, &FbxSourceObject>> {
    let mut map = BTreeMap::new();
    for object in objects {
        if let Some(existing) = map.insert(object.source_id, object) {
            return Err(graph_error(
                object.source_offset,
                format!(
                    "duplicate source object ID {} also appears at byte {}",
                    object.source_id, existing.source_offset
                ),
            ));
        }
    }
    Ok(map)
}

fn validate_connection_targets(
    connections: &[FbxConnection],
    objects: &BTreeMap<i64, &FbxSourceObject>,
) -> FbxResult<()> {
    for connection in connections {
        if !objects.contains_key(&connection.child_id) {
            return Err(graph_error(
                connection.source_offset,
                format!("connection child ID {} is missing", connection.child_id),
            ));
        }
        if connection.parent_id != 0 && !objects.contains_key(&connection.parent_id) {
            return Err(graph_error(
                connection.source_offset,
                format!("connection parent ID {} is missing", connection.parent_id),
            ));
        }
    }
    Ok(())
}

fn build_scene_nodes(
    objects: &[FbxSourceObject],
    connections: &[FbxConnection],
) -> FbxResult<Vec<FbxSourceSceneNode>> {
    let model_ids = objects
        .iter()
        .filter(|object| object.kind == "Model")
        .map(|object| object.source_id)
        .collect::<BTreeSet<_>>();
    let geometry_ids = objects
        .iter()
        .filter(|object| object.kind == "Geometry")
        .map(|object| object.source_id)
        .collect::<BTreeSet<_>>();
    let mut parents = BTreeMap::<i64, (Option<i64>, usize)>::new();
    let mut geometries = BTreeMap::<i64, Vec<i64>>::new();

    for connection in connections
        .iter()
        .filter(|connection| connection.relation == "OO")
    {
        if model_ids.contains(&connection.child_id)
            && (connection.parent_id == 0 || model_ids.contains(&connection.parent_id))
        {
            if let Some((existing_parent, existing_offset)) = parents.insert(
                connection.child_id,
                (
                    (connection.parent_id != 0).then_some(connection.parent_id),
                    connection.source_offset,
                ),
            ) {
                return Err(graph_error(
                    connection.source_offset,
                    format!(
                        "model {} receives more than one parent ({existing_parent:?} at byte {existing_offset})",
                        connection.child_id
                    ),
                ));
            }
        }
        if geometry_ids.contains(&connection.child_id) && model_ids.contains(&connection.parent_id)
        {
            geometries
                .entry(connection.parent_id)
                .or_default()
                .push(connection.child_id);
        }
    }

    for model_id in &model_ids {
        detect_cycle(*model_id, &parents)?;
    }

    objects
        .iter()
        .filter(|object| object.kind == "Model")
        .map(|object| {
            let parent_model_id = parents
                .get(&object.source_id)
                .and_then(|(parent, _)| *parent);
            let mut geometry_ids = geometries.remove(&object.source_id).unwrap_or_default();
            geometry_ids.sort_unstable();
            Ok(FbxSourceSceneNode {
                source_id: object.source_id,
                name: object.name.clone(),
                class: object.class.clone(),
                parent_model_id,
                geometry_ids,
                source_offset: object.source_offset,
            })
        })
        .collect()
}

fn detect_cycle(start: i64, parents: &BTreeMap<i64, (Option<i64>, usize)>) -> FbxResult<()> {
    let mut visited = BTreeSet::new();
    let mut current = Some(start);
    while let Some(id) = current {
        if !visited.insert(id) {
            let offset = parents.get(&id).map_or(0, |(_, offset)| *offset);
            return Err(graph_error(
                offset,
                format!("model hierarchy contains a cycle through source ID {id}"),
            ));
        }
        current = parents.get(&id).and_then(|(parent, _)| *parent);
    }
    Ok(())
}

fn property_string(record: &FbxRecord, index: usize, label: &str) -> FbxResult<String> {
    match record.properties.get(index) {
        Some(FbxProperty::String(value)) => Ok(value.clone()),
        _ => Err(graph_error(
            record.source_offset,
            format!("{label} is missing or not a string"),
        )),
    }
}

fn property_id(record: &FbxRecord, index: usize, label: &str) -> FbxResult<i64> {
    match record.properties.get(index) {
        Some(FbxProperty::I64(value)) => Ok(*value),
        _ => Err(graph_error(
            record.source_offset,
            format!("{label} is missing or not an I64"),
        )),
    }
}

fn graph_error(offset: usize, reason: impl Into<String>) -> FbxError {
    FbxError::SourceGraph {
        offset,
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FbxBinaryDocument;

    #[test]
    fn reconstructs_model_geometry_and_parent_links() {
        let scene = resolve_source_scene(&document(
            vec![
                object(1, "Model", "Model::Root", "Null"),
                object(2, "Model", "Model::Cube", "Mesh"),
                object(3, "Geometry", "Geometry::Cube", "Mesh"),
            ],
            vec![connection("OO", 2, 1), connection("OO", 3, 2)],
        ))
        .unwrap();

        assert_eq!(scene.nodes.len(), 2);
        assert_eq!(scene.nodes[1].parent_model_id, Some(1));
        assert_eq!(scene.nodes[1].geometry_ids, vec![3]);
    }

    #[test]
    fn rejects_missing_connection_target() {
        let error = resolve_source_scene(&document(
            vec![object(1, "Model", "Model::Root", "Null")],
            vec![connection("OO", 9, 1)],
        ))
        .unwrap_err();
        assert!(
            matches!(error, FbxError::SourceGraph { reason, .. } if reason.contains("child ID 9"))
        );
    }

    #[test]
    fn rejects_duplicate_object_ids() {
        let error = resolve_source_scene(&document(
            vec![
                object(1, "Model", "Model::One", "Null"),
                object(1, "Model", "Model::Two", "Null"),
            ],
            vec![],
        ))
        .unwrap_err();
        assert!(
            matches!(error, FbxError::SourceGraph { reason, .. } if reason.contains("duplicate source object ID 1"))
        );
    }

    #[test]
    fn rejects_hierarchy_cycles() {
        let error = resolve_source_scene(&document(
            vec![
                object(1, "Model", "Model::One", "Null"),
                object(2, "Model", "Model::Two", "Null"),
            ],
            vec![connection("OO", 1, 2), connection("OO", 2, 1)],
        ))
        .unwrap_err();
        assert!(matches!(error, FbxError::SourceGraph { reason, .. } if reason.contains("cycle")));
    }

    fn document(objects: Vec<FbxRecord>, connections: Vec<FbxRecord>) -> FbxBinaryDocument {
        FbxBinaryDocument {
            version: 7400,
            byte_order: crate::FbxByteOrder::LittleEndian,
            records: vec![
                FbxRecord {
                    name: "Objects".into(),
                    source_offset: 10,
                    end_offset: 20,
                    property_byte_length: 0,
                    properties: vec![],
                    children: objects,
                },
                FbxRecord {
                    name: "Connections".into(),
                    source_offset: 20,
                    end_offset: 30,
                    property_byte_length: 0,
                    properties: vec![],
                    children: connections,
                },
            ],
            footer_offset: 30,
            source_bytes: 30,
            source_fingerprint: "test".into(),
        }
    }

    fn object(id: i64, kind: &str, name: &str, class: &str) -> FbxRecord {
        FbxRecord {
            name: kind.into(),
            source_offset: id as usize,
            end_offset: id as usize + 1,
            property_byte_length: 0,
            properties: vec![
                FbxProperty::I64(id),
                FbxProperty::String(name.into()),
                FbxProperty::String(class.into()),
            ],
            children: vec![],
        }
    }

    fn connection(relation: &str, child_id: i64, parent_id: i64) -> FbxRecord {
        FbxRecord {
            name: "C".into(),
            source_offset: 100 + child_id as usize,
            end_offset: 101 + child_id as usize,
            property_byte_length: 0,
            properties: vec![
                FbxProperty::String(relation.into()),
                FbxProperty::I64(child_id),
                FbxProperty::I64(parent_id),
            ],
            children: vec![],
        }
    }
}
