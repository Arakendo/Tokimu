use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{FbxBinaryDocument, FbxError, FbxProperty, FbxRecord, FbxResult, FbxSourceScene};

/// Corpus-owned transform evidence for the intentionally narrow FBX v1
/// profile: local translation, XYZ Euler rotation, and scale.
///
/// The matrices retain the source coordinate system. Axis conversion is
/// recorded as source metadata rather than being silently applied.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxTransformEvidence {
    pub axes: FbxAxisMetadata,
    pub nodes: Vec<FbxNodeTransform>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxAxisMetadata {
    pub up_axis: Option<i32>,
    pub up_axis_sign: Option<i32>,
    pub front_axis: Option<i32>,
    pub front_axis_sign: Option<i32>,
    pub coord_axis: Option<i32>,
    pub coord_axis_sign: Option<i32>,
    pub unit_scale_factor: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxNodeTransform {
    pub source_id: i64,
    pub parent_model_id: Option<i64>,
    pub local_translation: [f64; 3],
    pub local_rotation_degrees_xyz: [f64; 3],
    pub local_scale: [f64; 3],
    pub local_matrix: [f64; 16],
    pub world_matrix: [f64; 16],
    pub source_offset: usize,
}

pub fn resolve_transforms(
    document: &FbxBinaryDocument,
    scene: &FbxSourceScene,
) -> FbxResult<FbxTransformEvidence> {
    let objects = top_level(document, "Objects")?;
    let records = objects
        .children
        .iter()
        .filter_map(|record| record_id(record).map(|id| (id, record)))
        .collect::<BTreeMap<_, _>>();
    let axes = decode_axis_metadata(document)?;
    let mut local = BTreeMap::new();

    for node in &scene.nodes {
        let record = records.get(&node.source_id).ok_or_else(|| {
            transform_error(
                node.source_offset,
                format!("missing model record for source ID {}", node.source_id),
            )
        })?;
        local.insert(
            node.source_id,
            decode_local_transform(record, node.parent_model_id)?,
        );
    }

    let mut world = BTreeMap::new();
    for node in &scene.nodes {
        resolve_world(node.source_id, &local, &mut world)?;
    }

    let nodes = scene
        .nodes
        .iter()
        .map(|node| {
            let transform = local.get(&node.source_id).expect("local transform exists");
            Ok(FbxNodeTransform {
                source_id: node.source_id,
                parent_model_id: node.parent_model_id,
                local_translation: transform.translation,
                local_rotation_degrees_xyz: transform.rotation,
                local_scale: transform.scale,
                local_matrix: transform.matrix,
                world_matrix: *world.get(&node.source_id).expect("world transform exists"),
                source_offset: node.source_offset,
            })
        })
        .collect::<FbxResult<Vec<_>>>()?;
    Ok(FbxTransformEvidence { axes, nodes })
}

pub fn transforms_json(evidence: &FbxTransformEvidence) -> FbxResult<String> {
    Ok(serde_json::to_string_pretty(evidence)?)
}

#[derive(Clone, Copy)]
struct LocalTransform {
    parent_model_id: Option<i64>,
    translation: [f64; 3],
    rotation: [f64; 3],
    scale: [f64; 3],
    matrix: [f64; 16],
    source_offset: usize,
}

fn decode_local_transform(
    record: &FbxRecord,
    parent_model_id: Option<i64>,
) -> FbxResult<LocalTransform> {
    let mut translation = [0.0, 0.0, 0.0];
    let mut rotation = [0.0, 0.0, 0.0];
    let mut scale = [1.0, 1.0, 1.0];
    let Some(properties) = record
        .children
        .iter()
        .find(|child| child.name == "Properties70")
    else {
        return Ok(LocalTransform {
            parent_model_id,
            translation,
            rotation,
            scale,
            matrix: trs_matrix(translation, rotation, scale),
            source_offset: record.source_offset,
        });
    };

    for property in &properties.children {
        if property.name != "P" {
            return Err(transform_error(
                property.source_offset,
                format!("unsupported Properties70 child `{}`", property.name),
            ));
        }
        let name = property_name(property)?;
        match name.as_str() {
            "Lcl Translation" => translation = vec3_property(property)?,
            "Lcl Rotation" => rotation = vec3_property(property)?,
            "Lcl Scaling" => scale = vec3_property(property)?,
            "PreRotation"
            | "PostRotation"
            | "RotationPivot"
            | "ScalingPivot"
            | "RotationOffset"
            | "ScalingOffset"
            | "GeometricTranslation"
            | "GeometricRotation"
            | "GeometricScaling" => {
                let expected = if name == "GeometricScaling" {
                    [1.0, 1.0, 1.0]
                } else {
                    [0.0, 0.0, 0.0]
                };
                let actual = vec3_property(property)?;
                if actual != expected {
                    return Err(transform_error(
                        property.source_offset,
                        format!("non-default `{name}` is outside the v1 TRS profile"),
                    ));
                }
            }
            "RotationOrder" => {
                let order = number_property(property, 4)? as i32;
                if order != 0 {
                    return Err(transform_error(
                        property.source_offset,
                        format!("rotation order {order} is outside the v1 XYZ profile"),
                    ));
                }
            }
            _ => {}
        }
    }
    if !translation
        .iter()
        .chain(rotation.iter())
        .chain(scale.iter())
        .all(|value| value.is_finite())
    {
        return Err(transform_error(
            record.source_offset,
            "local TRS contains non-finite values",
        ));
    }
    Ok(LocalTransform {
        parent_model_id,
        translation,
        rotation,
        scale,
        matrix: trs_matrix(translation, rotation, scale),
        source_offset: record.source_offset,
    })
}

fn resolve_world(
    id: i64,
    local: &BTreeMap<i64, LocalTransform>,
    resolved: &mut BTreeMap<i64, [f64; 16]>,
) -> FbxResult<[f64; 16]> {
    if let Some(matrix) = resolved.get(&id) {
        return Ok(*matrix);
    }
    let transform = local
        .get(&id)
        .ok_or_else(|| transform_error(0, format!("missing local transform for source ID {id}")))?;
    let matrix = match transform.parent_model_id {
        Some(parent) => multiply(resolve_world(parent, local, resolved)?, transform.matrix),
        None => transform.matrix,
    };
    if !matrix.iter().all(|value| value.is_finite()) {
        return Err(transform_error(
            transform.source_offset,
            "world transform is non-finite",
        ));
    }
    resolved.insert(id, matrix);
    Ok(matrix)
}

fn decode_axis_metadata(document: &FbxBinaryDocument) -> FbxResult<FbxAxisMetadata> {
    let Some(global) = document
        .records
        .iter()
        .find(|record| record.name == "GlobalSettings")
    else {
        return Ok(FbxAxisMetadata {
            up_axis: None,
            up_axis_sign: None,
            front_axis: None,
            front_axis_sign: None,
            coord_axis: None,
            coord_axis_sign: None,
            unit_scale_factor: None,
        });
    };
    let Some(properties) = global
        .children
        .iter()
        .find(|child| child.name == "Properties70")
    else {
        return Ok(FbxAxisMetadata {
            up_axis: None,
            up_axis_sign: None,
            front_axis: None,
            front_axis_sign: None,
            coord_axis: None,
            coord_axis_sign: None,
            unit_scale_factor: None,
        });
    };
    let value = |name: &str| {
        properties
            .children
            .iter()
            .find(|child| child.name == "P" && property_name(child).ok().as_deref() == Some(name))
            .map(|child| number_property(child, 4))
            .transpose()
    };
    Ok(FbxAxisMetadata {
        up_axis: value("UpAxis")?.map(|value| value as i32),
        up_axis_sign: value("UpAxisSign")?.map(|value| value as i32),
        front_axis: value("FrontAxis")?.map(|value| value as i32),
        front_axis_sign: value("FrontAxisSign")?.map(|value| value as i32),
        coord_axis: value("CoordAxis")?.map(|value| value as i32),
        coord_axis_sign: value("CoordAxisSign")?.map(|value| value as i32),
        unit_scale_factor: value("UnitScaleFactor")?,
    })
}

fn property_name(record: &FbxRecord) -> FbxResult<String> {
    match record.properties.first() {
        Some(FbxProperty::String(value)) => Ok(value.clone()),
        _ => Err(transform_error(
            record.source_offset,
            "Properties70 entry has no string name",
        )),
    }
}

fn vec3_property(record: &FbxRecord) -> FbxResult<[f64; 3]> {
    Ok([
        number_property(record, 4)?,
        number_property(record, 5)?,
        number_property(record, 6)?,
    ])
}

fn number_property(record: &FbxRecord, index: usize) -> FbxResult<f64> {
    let value = match record.properties.get(index) {
        Some(FbxProperty::I16(value)) => *value as f64,
        Some(FbxProperty::I32(value)) => *value as f64,
        Some(FbxProperty::I64(value)) => *value as f64,
        Some(FbxProperty::F32(value)) => *value as f64,
        Some(FbxProperty::F64(value)) => *value,
        _ => {
            return Err(transform_error(
                record.source_offset,
                format!("Properties70 value {index} is missing or non-numeric"),
            ))
        }
    };
    if !value.is_finite() {
        return Err(transform_error(
            record.source_offset,
            "Properties70 contains a non-finite number",
        ));
    }
    Ok(value)
}

fn top_level<'a>(document: &'a FbxBinaryDocument, name: &str) -> FbxResult<&'a FbxRecord> {
    document
        .records
        .iter()
        .find(|record| record.name == name)
        .ok_or_else(|| transform_error(0, format!("missing top-level `{name}` record")))
}

fn record_id(record: &FbxRecord) -> Option<i64> {
    match record.properties.first() {
        Some(FbxProperty::I64(value)) => Some(*value),
        _ => None,
    }
}

fn trs_matrix(translation: [f64; 3], rotation: [f64; 3], scale: [f64; 3]) -> [f64; 16] {
    let [rx, ry, rz] = rotation.map(f64::to_radians);
    let (sx, cx) = rx.sin_cos();
    let (sy, cy) = ry.sin_cos();
    let (sz, cz) = rz.sin_cos();
    let rotation = [
        cy * cz,
        cy * sz,
        -sy,
        0.0,
        sx * sy * cz - cx * sz,
        sx * sy * sz + cx * cz,
        sx * cy,
        0.0,
        cx * sy * cz + sx * sz,
        cx * sy * sz - sx * cz,
        cx * cy,
        0.0,
        translation[0],
        translation[1],
        translation[2],
        1.0,
    ];
    let scaling = [
        scale[0], 0.0, 0.0, 0.0, 0.0, scale[1], 0.0, 0.0, 0.0, 0.0, scale[2], 0.0, 0.0, 0.0, 0.0,
        1.0,
    ];
    multiply(rotation, scaling)
}

fn multiply(left: [f64; 16], right: [f64; 16]) -> [f64; 16] {
    let mut output = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            output[column * 4 + row] = (0..4)
                .map(|index| left[index * 4 + row] * right[column * 4 + index])
                .sum();
        }
    }
    output
}

fn transform_error(offset: usize, reason: impl Into<String>) -> FbxError {
    FbxError::Transform {
        offset,
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composes_parent_translation_with_child_translation() {
        let local = BTreeMap::from([
            (
                1,
                LocalTransform {
                    parent_model_id: None,
                    translation: [2.0, 0.0, 0.0],
                    rotation: [0.0; 3],
                    scale: [1.0; 3],
                    matrix: trs_matrix([2.0, 0.0, 0.0], [0.0; 3], [1.0; 3]),
                    source_offset: 1,
                },
            ),
            (
                2,
                LocalTransform {
                    parent_model_id: Some(1),
                    translation: [0.0, 3.0, 0.0],
                    rotation: [0.0; 3],
                    scale: [1.0; 3],
                    matrix: trs_matrix([0.0, 3.0, 0.0], [0.0; 3], [1.0; 3]),
                    source_offset: 2,
                },
            ),
        ]);
        let mut resolved = BTreeMap::new();
        let world = resolve_world(2, &local, &mut resolved).unwrap();
        assert_eq!([world[12], world[13], world[14]], [2.0, 3.0, 0.0]);
    }

    #[test]
    fn rejects_non_default_rotation_pivot() {
        let property = FbxRecord {
            name: "P".into(),
            source_offset: 1,
            end_offset: 2,
            property_byte_length: 0,
            properties: vec![
                FbxProperty::String("RotationPivot".into()),
                FbxProperty::String("Vector3D".into()),
                FbxProperty::String(String::new()),
                FbxProperty::String("A".into()),
                FbxProperty::F64(1.0),
                FbxProperty::F64(0.0),
                FbxProperty::F64(0.0),
            ],
            children: vec![],
        };
        let record = FbxRecord {
            name: "Model".into(),
            source_offset: 0,
            end_offset: 3,
            property_byte_length: 0,
            properties: vec![],
            children: vec![FbxRecord {
                name: "Properties70".into(),
                source_offset: 0,
                end_offset: 3,
                property_byte_length: 0,
                properties: vec![],
                children: vec![property],
            }],
        };
        assert!(matches!(
            decode_local_transform(&record, None),
            Err(FbxError::Transform { .. })
        ));
    }
}
