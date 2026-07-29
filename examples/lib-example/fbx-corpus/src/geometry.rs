use serde::{Deserialize, Serialize};

use crate::{FbxBinaryDocument, FbxError, FbxProperty, FbxRecord, FbxResult, FbxSourceScene};

/// Corpus-owned static geometry evidence. It deliberately retains polygon
/// topology before deriving renderer-friendly triangles.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxGeometryEvidence {
    pub meshes: Vec<FbxStaticMesh>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxStaticMesh {
    pub source_id: i64,
    pub name: String,
    pub source_offset: usize,
    pub control_points: Vec<[f64; 3]>,
    pub polygons: Vec<FbxPolygon>,
    pub triangles: Vec<[u32; 3]>,
    pub normal_layer: Option<FbxNormalLayer>,
    pub uv_layer: Option<FbxUvLayer>,
    pub bounds: FbxBounds,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxPolygon {
    pub control_point_indices: Vec<u32>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxNormalLayer {
    pub mapping: String,
    pub reference: String,
    pub values: Vec<[f64; 3]>,
    pub indices: Option<Vec<u32>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxUvLayer {
    pub mapping: String,
    pub reference: String,
    pub values: Vec<[f64; 2]>,
    pub indices: Option<Vec<u32>>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct FbxBounds {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

pub fn lower_static_geometry(
    document: &FbxBinaryDocument,
    scene: &FbxSourceScene,
) -> FbxResult<FbxGeometryEvidence> {
    let objects = top_level_record(document, "Objects")?;
    let meshes = scene
        .objects
        .iter()
        .filter(|object| object.kind == "Geometry" && object.class == "Mesh")
        .map(|object| {
            let record = objects
                .children
                .iter()
                .find(|record| record_id(record) == Some(object.source_id))
                .ok_or_else(|| {
                    geometry_error(
                        object.source_offset,
                        format!("missing source record for geometry {}", object.source_id),
                    )
                })?;
            decode_mesh(object.source_id, &object.name, record)
        })
        .collect::<FbxResult<Vec<_>>>()?;
    Ok(FbxGeometryEvidence { meshes })
}

pub fn meshes_json(evidence: &FbxGeometryEvidence) -> FbxResult<String> {
    Ok(serde_json::to_string_pretty(&evidence.meshes)?)
}

pub fn topology_json(evidence: &FbxGeometryEvidence) -> FbxResult<String> {
    #[derive(Serialize)]
    struct Topology<'a> {
        source_id: i64,
        polygons: &'a [FbxPolygon],
        triangles: &'a [[u32; 3]],
    }

    let topology = evidence
        .meshes
        .iter()
        .map(|mesh| Topology {
            source_id: mesh.source_id,
            polygons: &mesh.polygons,
            triangles: &mesh.triangles,
        })
        .collect::<Vec<_>>();
    Ok(serde_json::to_string_pretty(&topology)?)
}

pub fn bounds_json(evidence: &FbxGeometryEvidence) -> FbxResult<String> {
    #[derive(Serialize)]
    struct Bounds {
        source_id: i64,
        bounds: FbxBounds,
    }

    let bounds = evidence
        .meshes
        .iter()
        .map(|mesh| Bounds {
            source_id: mesh.source_id,
            bounds: mesh.bounds,
        })
        .collect::<Vec<_>>();
    Ok(serde_json::to_string_pretty(&bounds)?)
}

fn decode_mesh(source_id: i64, name: &str, record: &FbxRecord) -> FbxResult<FbxStaticMesh> {
    let vertices = f64_array(record, "Vertices")?;
    if vertices.len() % 3 != 0 {
        return Err(geometry_error(
            record.source_offset,
            format!(
                "Vertices has {} values, not a multiple of three",
                vertices.len()
            ),
        ));
    }
    let control_points = vertices
        .chunks_exact(3)
        .map(|point| [point[0], point[1], point[2]])
        .collect::<Vec<_>>();
    if control_points.is_empty() {
        return Err(geometry_error(
            record.source_offset,
            "mesh has no control points",
        ));
    }
    if control_points
        .iter()
        .flatten()
        .any(|value| !value.is_finite())
    {
        return Err(geometry_error(
            record.source_offset,
            "mesh control points contain a non-finite value",
        ));
    }

    let polygons = decode_polygons(
        i32_array(record, "PolygonVertexIndex")?,
        control_points.len(),
        record.source_offset,
    )?;
    let triangles = triangulate(&polygons, record.source_offset)?;
    let normal_layer = record
        .children
        .iter()
        .find(|child| child.name == "LayerElementNormal")
        .map(decode_normal_layer)
        .transpose()?;
    let uv_layer = record
        .children
        .iter()
        .find(|child| child.name == "LayerElementUV")
        .map(decode_uv_layer)
        .transpose()?;

    Ok(FbxStaticMesh {
        source_id,
        name: name.to_owned(),
        source_offset: record.source_offset,
        control_points: control_points.clone(),
        polygons,
        triangles,
        normal_layer,
        uv_layer,
        bounds: bounds(&control_points),
    })
}

fn decode_polygons(
    values: Vec<i32>,
    control_point_count: usize,
    offset: usize,
) -> FbxResult<Vec<FbxPolygon>> {
    let mut polygons = Vec::new();
    let mut current = Vec::new();
    for value in values {
        let (raw_index, end_polygon) = if value < 0 {
            (
                value
                    .checked_neg()
                    .and_then(|value| value.checked_sub(1))
                    .ok_or_else(|| {
                        geometry_error(
                            offset,
                            "polygon index uses the unsupported i32::MIN sentinel",
                        )
                    })?,
                true,
            )
        } else {
            (value, false)
        };
        let index = usize::try_from(raw_index)
            .map_err(|_| geometry_error(offset, format!("negative polygon index {raw_index}")))?;
        if index >= control_point_count {
            return Err(geometry_error(
                offset,
                format!("polygon index {index} exceeds {control_point_count} control points"),
            ));
        }
        current.push(index as u32);
        if end_polygon {
            if current.len() < 3 {
                return Err(geometry_error(
                    offset,
                    "polygon has fewer than three vertices",
                ));
            }
            polygons.push(FbxPolygon {
                control_point_indices: std::mem::take(&mut current),
            });
        }
    }
    if !current.is_empty() {
        return Err(geometry_error(
            offset,
            "polygon index list ends before a polygon terminator",
        ));
    }
    if polygons.is_empty() {
        return Err(geometry_error(offset, "mesh has no polygons"));
    }
    Ok(polygons)
}

fn triangulate(polygons: &[FbxPolygon], offset: usize) -> FbxResult<Vec<[u32; 3]>> {
    let mut triangles = Vec::new();
    for polygon in polygons {
        let Some((&first, rest)) = polygon.control_point_indices.split_first() else {
            return Err(geometry_error(offset, "empty polygon"));
        };
        for pair in rest.windows(2) {
            triangles.push([first, pair[0], pair[1]]);
        }
    }
    Ok(triangles)
}

fn decode_normal_layer(record: &FbxRecord) -> FbxResult<FbxNormalLayer> {
    let mapping = string_property(record, "MappingInformationType")?;
    let reference = string_property(record, "ReferenceInformationType")?;
    let values = f64_array(record, "Normals")?;
    if values.len() % 3 != 0 {
        return Err(geometry_error(
            record.source_offset,
            "Normals is not a multiple of three",
        ));
    }
    let indices = optional_indices(record, "NormalsIndex", values.len() / 3)?;
    Ok(FbxNormalLayer {
        mapping,
        reference,
        values: values
            .chunks_exact(3)
            .map(|value| [value[0], value[1], value[2]])
            .collect(),
        indices,
    })
}

fn decode_uv_layer(record: &FbxRecord) -> FbxResult<FbxUvLayer> {
    let mapping = string_property(record, "MappingInformationType")?;
    let reference = string_property(record, "ReferenceInformationType")?;
    let values = f64_array(record, "UV")?;
    if values.len() % 2 != 0 {
        return Err(geometry_error(
            record.source_offset,
            "UV is not a multiple of two",
        ));
    }
    let indices = optional_indices(record, "UVIndex", values.len() / 2)?;
    Ok(FbxUvLayer {
        mapping,
        reference,
        values: values
            .chunks_exact(2)
            .map(|value| [value[0], value[1]])
            .collect(),
        indices,
    })
}

fn optional_indices(
    record: &FbxRecord,
    name: &str,
    value_count: usize,
) -> FbxResult<Option<Vec<u32>>> {
    let Some(child) = record.children.iter().find(|child| child.name == name) else {
        return Ok(None);
    };
    let values = property_i32_array(child, 0, name)?;
    values
        .into_iter()
        .map(|value| {
            let index = usize::try_from(value).map_err(|_| {
                geometry_error(
                    child.source_offset,
                    format!("{name} contains negative index {value}"),
                )
            })?;
            if index >= value_count {
                return Err(geometry_error(
                    child.source_offset,
                    format!("{name} index {index} exceeds {value_count} values"),
                ));
            }
            Ok(index as u32)
        })
        .collect::<FbxResult<Vec<_>>>()
        .map(Some)
}

fn f64_array(record: &FbxRecord, child_name: &str) -> FbxResult<Vec<f64>> {
    let child = record
        .children
        .iter()
        .find(|child| child.name == child_name)
        .ok_or_else(|| {
            geometry_error(
                record.source_offset,
                format!("missing `{child_name}` record"),
            )
        })?;
    property_f64_array(child, 0, child_name)
}

fn i32_array(record: &FbxRecord, child_name: &str) -> FbxResult<Vec<i32>> {
    let child = record
        .children
        .iter()
        .find(|child| child.name == child_name)
        .ok_or_else(|| {
            geometry_error(
                record.source_offset,
                format!("missing `{child_name}` record"),
            )
        })?;
    property_i32_array(child, 0, child_name)
}

fn string_property(record: &FbxRecord, child_name: &str) -> FbxResult<String> {
    let child = record
        .children
        .iter()
        .find(|child| child.name == child_name)
        .ok_or_else(|| {
            geometry_error(
                record.source_offset,
                format!("missing `{child_name}` record"),
            )
        })?;
    match child.properties.first() {
        Some(FbxProperty::String(value)) => Ok(value.clone()),
        _ => Err(geometry_error(
            child.source_offset,
            format!("`{child_name}` does not contain a string property"),
        )),
    }
}

fn property_f64_array(record: &FbxRecord, index: usize, label: &str) -> FbxResult<Vec<f64>> {
    match record.properties.get(index) {
        Some(FbxProperty::F64Array(values)) => Ok(values.clone()),
        _ => Err(geometry_error(
            record.source_offset,
            format!("`{label}` does not contain an F64 array"),
        )),
    }
}

fn property_i32_array(record: &FbxRecord, index: usize, label: &str) -> FbxResult<Vec<i32>> {
    match record.properties.get(index) {
        Some(FbxProperty::I32Array(values)) => Ok(values.clone()),
        _ => Err(geometry_error(
            record.source_offset,
            format!("`{label}` does not contain an I32 array"),
        )),
    }
}

fn top_level_record<'a>(document: &'a FbxBinaryDocument, name: &str) -> FbxResult<&'a FbxRecord> {
    document
        .records
        .iter()
        .find(|record| record.name == name)
        .ok_or_else(|| geometry_error(0, format!("missing top-level `{name}` record")))
}

fn record_id(record: &FbxRecord) -> Option<i64> {
    match record.properties.first() {
        Some(FbxProperty::I64(value)) => Some(*value),
        _ => None,
    }
}

fn bounds(points: &[[f64; 3]]) -> FbxBounds {
    let mut min = points[0];
    let mut max = points[0];
    for point in &points[1..] {
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }
    FbxBounds { min, max }
}

fn geometry_error(offset: usize, reason: impl Into<String>) -> FbxError {
    FbxError::Geometry {
        offset,
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FbxConnection, FbxSourceObject, FbxSourceScene};

    #[test]
    fn triangulates_and_preserves_a_quad_polygon() {
        let document = document_with_mesh(
            vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0],
            vec![0, 1, 2, -4],
        );
        let evidence = lower_static_geometry(&document, &scene()).unwrap();
        let mesh = &evidence.meshes[0];

        assert_eq!(mesh.polygons[0].control_point_indices, vec![0, 1, 2, 3]);
        assert_eq!(mesh.triangles, vec![[0, 1, 2], [0, 2, 3]]);
        assert_eq!(mesh.bounds.min, [0.0, 0.0, 0.0]);
        assert_eq!(mesh.bounds.max, [1.0, 1.0, 0.0]);
    }

    #[test]
    fn rejects_polygon_indices_outside_control_points() {
        let document = document_with_mesh(
            vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            vec![0, 1, -5],
        );
        assert!(matches!(
            lower_static_geometry(&document, &scene()),
            Err(FbxError::Geometry { reason, .. }) if reason.contains("exceeds 3 control points")
        ));
    }

    fn scene() -> FbxSourceScene {
        FbxSourceScene {
            source_fingerprint: "test".into(),
            objects: vec![FbxSourceObject {
                source_id: 1,
                kind: "Geometry".into(),
                name: "Geometry::Test".into(),
                class: "Mesh".into(),
                source_offset: 1,
            }],
            connections: Vec::<FbxConnection>::new(),
            nodes: vec![],
            diagnostics: vec![],
        }
    }

    fn document_with_mesh(vertices: Vec<f64>, polygon_indices: Vec<i32>) -> FbxBinaryDocument {
        let geometry = FbxRecord {
            name: "Geometry".into(),
            source_offset: 1,
            end_offset: 2,
            property_byte_length: 0,
            properties: vec![
                FbxProperty::I64(1),
                FbxProperty::String("Geometry::Test".into()),
                FbxProperty::String("Mesh".into()),
            ],
            children: vec![
                array_record("Vertices", FbxProperty::F64Array(vertices)),
                array_record("PolygonVertexIndex", FbxProperty::I32Array(polygon_indices)),
            ],
        };
        FbxBinaryDocument {
            version: 7400,
            records: vec![FbxRecord {
                name: "Objects".into(),
                source_offset: 0,
                end_offset: 3,
                property_byte_length: 0,
                properties: vec![],
                children: vec![geometry],
            }],
            footer_offset: 3,
            source_bytes: 3,
            source_fingerprint: "test".into(),
        }
    }

    fn array_record(name: &str, property: FbxProperty) -> FbxRecord {
        FbxRecord {
            name: name.into(),
            source_offset: 2,
            end_offset: 3,
            property_byte_length: 0,
            properties: vec![property],
            children: vec![],
        }
    }
}
