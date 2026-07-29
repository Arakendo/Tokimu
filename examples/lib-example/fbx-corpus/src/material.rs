use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    FbxBinaryDocument, FbxConnection, FbxError, FbxProperty, FbxRecord, FbxResult, FbxSourceScene,
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

pub fn resolve_materials(
    document: &FbxBinaryDocument,
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

fn top_level<'a>(document: &'a FbxBinaryDocument, name: &str) -> FbxResult<&'a FbxRecord> {
    document
        .records
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
}
