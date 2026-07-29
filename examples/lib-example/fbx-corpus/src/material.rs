use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    FbxConnection, FbxError, FbxGeometryEvidence, FbxProperty, FbxRecord, FbxRecordDocument,
    FbxResult, FbxSourceScene,
};

/// Source-level material evidence. These records intentionally preserve FBX
/// names and connection topology without defining Tokimu shading semantics.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxMaterialEvidence {
    pub materials: Vec<FbxSourceMaterial>,
    pub textures: Vec<FbxSourceTexture>,
    pub bindings: Vec<FbxMaterialBinding>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxSourceMaterial {
    pub source_id: i64,
    pub name: String,
    pub class: String,
    pub properties: Vec<FbxMaterialProperty>,
    pub source_offset: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxSourceTexture {
    pub source_id: i64,
    pub name: String,
    pub class: String,
    pub file_name: Option<String>,
    pub relative_file_name: Option<String>,
    pub source_offset: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxMaterialProperty {
    pub name: String,
    pub values: Vec<FbxMaterialValue>,
    pub source_offset: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum FbxMaterialValue {
    Bool(bool),
    Integer(i64),
    Number(f64),
    Text(String),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FbxMaterialBinding {
    pub relation: String,
    pub child_id: i64,
    pub parent_id: i64,
    pub property: Option<String>,
    pub source_offset: usize,
}

/// A material-slot assignment resolved only for the `ByPolygon` or `AllSame`
/// `IndexToDirect` source profiles. It is imported-model evidence, not a
/// renderer material binding.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FbxMaterialSlotAssignment {
    pub geometry_id: i64,
    pub model_id: i64,
    pub material_ids: Vec<i64>,
    pub polygon_material_slots: Vec<u32>,
    pub source_offset: usize,
}

pub fn resolve_materials(
    document: &impl FbxRecordDocument,
    scene: &FbxSourceScene,
) -> FbxResult<FbxMaterialEvidence> {
    let objects = top_level(document, "Objects")?;
    let records = objects
        .children
        .iter()
        .filter_map(|record| record_id(record).map(|id| (id, record)))
        .collect::<BTreeMap<_, _>>();
    let materials = scene
        .objects
        .iter()
        .filter(|object| object.kind == "Material")
        .map(|object| {
            let record = records.get(&object.source_id).ok_or_else(|| {
                material_error(
                    object.source_offset,
                    format!("missing material record for source ID {}", object.source_id),
                )
            })?;
            Ok(FbxSourceMaterial {
                source_id: object.source_id,
                name: object.name.clone(),
                class: object.class.clone(),
                properties: decode_properties(record)?,
                source_offset: object.source_offset,
            })
        })
        .collect::<FbxResult<Vec<_>>>()?;
    let textures = scene
        .objects
        .iter()
        .filter(|object| object.kind == "Texture")
        .map(|object| {
            let record = records.get(&object.source_id).ok_or_else(|| {
                material_error(
                    object.source_offset,
                    format!("missing texture record for source ID {}", object.source_id),
                )
            })?;
            Ok(FbxSourceTexture {
                source_id: object.source_id,
                name: object.name.clone(),
                class: object.class.clone(),
                file_name: child_string(record, "FileName")?,
                relative_file_name: child_string(record, "RelativeFilename")?,
                source_offset: object.source_offset,
            })
        })
        .collect::<FbxResult<Vec<_>>>()?;
    let source_ids = materials
        .iter()
        .map(|material| material.source_id)
        .chain(textures.iter().map(|texture| texture.source_id))
        .collect::<std::collections::BTreeSet<_>>();
    let bindings = scene
        .connections
        .iter()
        .filter(|connection| {
            source_ids.contains(&connection.child_id) || source_ids.contains(&connection.parent_id)
        })
        .map(binding)
        .collect();
    Ok(FbxMaterialEvidence {
        materials,
        textures,
        bindings,
    })
}

pub fn material_objects_json(evidence: &FbxMaterialEvidence) -> FbxResult<String> {
    #[derive(Serialize)]
    struct Objects<'a> {
        materials: &'a [FbxSourceMaterial],
        textures: &'a [FbxSourceTexture],
    }
    Ok(serde_json::to_string_pretty(&Objects {
        materials: &evidence.materials,
        textures: &evidence.textures,
    })?)
}

pub fn material_bindings_json(evidence: &FbxMaterialEvidence) -> FbxResult<String> {
    Ok(serde_json::to_string_pretty(&evidence.bindings)?)
}

pub fn resolve_material_slots(
    scene: &FbxSourceScene,
    geometry: &FbxGeometryEvidence,
) -> FbxResult<Vec<FbxMaterialSlotAssignment>> {
    let model_materials = material_ids_by_model(scene)?;

    let mut assignments = Vec::new();
    for mesh in &geometry.meshes {
        let Some(layer) = &mesh.material_layer else {
            continue;
        };
        if layer.reference != "IndexToDirect" {
            return Err(material_error(
                mesh.source_offset,
                format!(
                    "geometry {} uses unsupported material reference mode {}/{}",
                    mesh.source_id, layer.mapping, layer.reference
                ),
            ));
        }
        for node in scene
            .nodes
            .iter()
            .filter(|node| node.geometry_ids.contains(&mesh.source_id))
        {
            let ids = model_materials
                .get(&node.source_id)
                .cloned()
                .unwrap_or_default();
            if ids.is_empty() {
                return Err(material_error(
                    node.source_offset,
                    format!(
                        "model {} references material-layer geometry {} without material bindings",
                        node.source_id, mesh.source_id
                    ),
                ));
            }
            let source_slots = match layer.mapping.as_str() {
                "ByPolygon" if layer.indices.len() == mesh.polygons.len() => layer.indices.clone(),
                "AllSame" if layer.indices.len() == 1 => {
                    vec![layer.indices[0]; mesh.polygons.len()]
                }
                "ByPolygon" => {
                    return Err(material_error(
                        mesh.source_offset,
                        format!(
                            "geometry {} has {} material slots for {} polygons",
                            mesh.source_id,
                            layer.indices.len(),
                            mesh.polygons.len()
                        ),
                    ))
                }
                mapping => {
                    return Err(material_error(
                        mesh.source_offset,
                        format!(
                            "geometry {} uses unsupported material mapping `{mapping}`",
                            mesh.source_id
                        ),
                    ))
                }
            };
            let polygon_material_slots = source_slots
                .iter()
                .map(|index| {
                    let index = usize::try_from(*index).map_err(|_| {
                        material_error(mesh.source_offset, "negative material slot index")
                    })?;
                    if index >= ids.len() {
                        return Err(material_error(
                            mesh.source_offset,
                            format!(
                                "polygon material slot {index} exceeds {} bound materials",
                                ids.len()
                            ),
                        ));
                    }
                    Ok(index as u32)
                })
                .collect::<FbxResult<Vec<_>>>()?;
            assignments.push(FbxMaterialSlotAssignment {
                geometry_id: mesh.source_id,
                model_id: node.source_id,
                material_ids: ids,
                polygon_material_slots,
                source_offset: mesh.source_offset,
            });
        }
    }
    Ok(assignments)
}

/// FBX material slots are indexed through their `Connections` record order.
/// Preserve that source order instead of sorting by object ID, which would
/// make a deterministic but semantically incorrect slot table.
fn material_ids_by_model(scene: &FbxSourceScene) -> FbxResult<BTreeMap<i64, Vec<i64>>> {
    let material_ids = scene
        .objects
        .iter()
        .filter(|object| object.kind == "Material")
        .map(|object| object.source_id)
        .collect::<std::collections::BTreeSet<_>>();
    let mut model_materials = BTreeMap::<i64, Vec<i64>>::new();
    for connection in &scene.connections {
        if connection.relation == "OO"
            && material_ids.contains(&connection.child_id)
            && scene
                .nodes
                .iter()
                .any(|node| node.source_id == connection.parent_id)
        {
            let materials = model_materials.entry(connection.parent_id).or_default();
            if materials.contains(&connection.child_id) {
                return Err(material_error(
                    connection.source_offset,
                    format!(
                        "model {} repeats material connection for source ID {}",
                        connection.parent_id, connection.child_id
                    ),
                ));
            }
            materials.push(connection.child_id);
        }
    }
    Ok(model_materials)
}

pub fn material_slots_json(assignments: &[FbxMaterialSlotAssignment]) -> FbxResult<String> {
    Ok(serde_json::to_string_pretty(assignments)?)
}

fn decode_properties(record: &FbxRecord) -> FbxResult<Vec<FbxMaterialProperty>> {
    let Some(properties) = record
        .children
        .iter()
        .find(|child| child.name == "Properties70")
    else {
        return Ok(Vec::new());
    };
    properties
        .children
        .iter()
        .filter(|child| child.name == "P")
        .map(|property| {
            let name = match property.properties.first() {
                Some(FbxProperty::String(value)) => value.clone(),
                _ => {
                    return Err(material_error(
                        property.source_offset,
                        "material property has no string name",
                    ))
                }
            };
            let values = property
                .properties
                .iter()
                .skip(4)
                .map(material_value)
                .collect::<FbxResult<Vec<_>>>()?;
            Ok(FbxMaterialProperty {
                name,
                values,
                source_offset: property.source_offset,
            })
        })
        .collect()
}

fn material_value(value: &FbxProperty) -> FbxResult<FbxMaterialValue> {
    match value {
        FbxProperty::Bool(value) => Ok(FbxMaterialValue::Bool(*value)),
        FbxProperty::I16(value) => Ok(FbxMaterialValue::Integer(*value as i64)),
        FbxProperty::I32(value) => Ok(FbxMaterialValue::Integer(*value as i64)),
        FbxProperty::I64(value) => Ok(FbxMaterialValue::Integer(*value)),
        FbxProperty::F32(value) if value.is_finite() => Ok(FbxMaterialValue::Number(*value as f64)),
        FbxProperty::F64(value) if value.is_finite() => Ok(FbxMaterialValue::Number(*value)),
        FbxProperty::String(value) => Ok(FbxMaterialValue::Text(value.clone())),
        _ => Err(material_error(
            0,
            "material property uses unsupported or non-finite value",
        )),
    }
}

fn child_string(record: &FbxRecord, name: &str) -> FbxResult<Option<String>> {
    let Some(child) = record.children.iter().find(|child| child.name == name) else {
        return Ok(None);
    };
    match child.properties.first() {
        Some(FbxProperty::String(value)) => Ok(Some(value.clone())),
        _ => Err(material_error(
            child.source_offset,
            format!("`{name}` is not a string"),
        )),
    }
}

fn binding(connection: &FbxConnection) -> FbxMaterialBinding {
    FbxMaterialBinding {
        relation: connection.relation.clone(),
        child_id: connection.child_id,
        parent_id: connection.parent_id,
        property: connection.property.clone(),
        source_offset: connection.source_offset,
    }
}

fn top_level<'a>(document: &'a impl FbxRecordDocument, name: &str) -> FbxResult<&'a FbxRecord> {
    document
        .records()
        .iter()
        .find(|record| record.name == name)
        .ok_or_else(|| material_error(0, format!("missing top-level `{name}` record")))
}

fn record_id(record: &FbxRecord) -> Option<i64> {
    match record.properties.first() {
        Some(FbxProperty::I64(value)) => Some(*value),
        _ => None,
    }
}

fn material_error(offset: usize, reason: impl Into<String>) -> FbxError {
    FbxError::Material {
        offset,
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source_object(id: i64, kind: &str) -> crate::FbxSourceObject {
        crate::FbxSourceObject {
            source_id: id,
            kind: kind.into(),
            name: format!("{kind}-{id}"),
            class: String::new(),
            source_offset: id as usize,
        }
    }

    #[test]
    fn preserves_supported_source_property_values() {
        let values = [FbxProperty::F64(1.0), FbxProperty::String("Lambert".into())]
            .iter()
            .map(material_value)
            .collect::<FbxResult<Vec<_>>>()
            .unwrap();
        assert_eq!(
            values,
            vec![
                FbxMaterialValue::Number(1.0),
                FbxMaterialValue::Text("Lambert".into())
            ]
        );
    }

    #[test]
    fn preserves_material_connection_order_for_slot_tables() {
        let scene = FbxSourceScene {
            source_fingerprint: "test".into(),
            objects: vec![
                source_object(30, "Model"),
                source_object(100, "Material"),
                source_object(10, "Material"),
            ],
            connections: vec![
                FbxConnection {
                    relation: "OO".into(),
                    child_id: 100,
                    parent_id: 30,
                    property: None,
                    source_offset: 1,
                },
                FbxConnection {
                    relation: "OO".into(),
                    child_id: 10,
                    parent_id: 30,
                    property: None,
                    source_offset: 2,
                },
            ],
            nodes: vec![crate::FbxSourceSceneNode {
                source_id: 30,
                name: "model".into(),
                class: String::new(),
                parent_model_id: None,
                geometry_ids: vec![],
                source_offset: 0,
            }],
            diagnostics: vec![],
        };

        assert_eq!(material_ids_by_model(&scene).unwrap()[&30], vec![100, 10]);
    }
}
