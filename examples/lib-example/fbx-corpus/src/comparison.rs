use serde::{Deserialize, Serialize};

use crate::{FbxGeometryEvidence, FbxSourceScene, FbxTransformEvidence};

/// Stable identity for the bounded, encoding-neutral static observation view.
pub const STATIC_OBSERVATION_COMPARISON_SCHEMA: &str = "fbx-static-observation-v1";
pub const STATIC_OBSERVATION_COMPARISON_ALGORITHM: &str = "ordered-observation-v1";

/// Provider-local differential evidence for two FBX inputs representing one
/// logical scene. A mismatch is evidence, not an engine failure classification.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FbxComparisonReport {
    pub schema: String,
    pub algorithm: String,
    pub equivalent: bool,
    pub first_difference: Option<FbxComparisonDifference>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FbxComparisonDifference {
    pub stage: String,
    pub observation: String,
    pub left: String,
    pub right: String,
}

/// Compares source graph shape, static mesh data, and source-transform values.
///
/// Object IDs, source labels, byte offsets, and fingerprints remain encoding
/// evidence and are intentionally excluded. Their values may differ between
/// equivalent binary and ASCII exports.
pub fn compare_static_observations(
    left_scene: &FbxSourceScene,
    left_geometry: &FbxGeometryEvidence,
    left_transforms: &FbxTransformEvidence,
    right_scene: &FbxSourceScene,
    right_geometry: &FbxGeometryEvidence,
    right_transforms: &FbxTransformEvidence,
) -> FbxComparisonReport {
    let first_difference = compare_source_graph(left_scene, right_scene)
        .or_else(|| compare_geometry(left_geometry, right_geometry))
        .or_else(|| compare_transforms(left_transforms, right_transforms));
    FbxComparisonReport {
        schema: STATIC_OBSERVATION_COMPARISON_SCHEMA.into(),
        algorithm: STATIC_OBSERVATION_COMPARISON_ALGORITHM.into(),
        equivalent: first_difference.is_none(),
        first_difference,
    }
}

pub fn comparison_json(report: &FbxComparisonReport) -> crate::FbxResult<String> {
    Ok(serde_json::to_string_pretty(report)?)
}

fn compare_source_graph(
    left: &FbxSourceScene,
    right: &FbxSourceScene,
) -> Option<FbxComparisonDifference> {
    compare_slice_length(
        "source",
        "object count",
        left.objects.len(),
        right.objects.len(),
    )
    .or_else(|| {
        left.objects
            .iter()
            .map(|object| (&object.kind, &object.class))
            .zip(
                right
                    .objects
                    .iter()
                    .map(|object| (&object.kind, &object.class)),
            )
            .enumerate()
            .find_map(|(index, (left, right))| {
                (left != right)
                    .then(|| difference("source", format!("object {index}"), left, right))
            })
    })
    .or_else(|| {
        compare_slice_length(
            "source",
            "connection count",
            left.connections.len(),
            right.connections.len(),
        )
    })
    .or_else(|| {
        left.connections
            .iter()
            .map(|connection| (&connection.relation, connection.property.as_deref()))
            .zip(
                right
                    .connections
                    .iter()
                    .map(|connection| (&connection.relation, connection.property.as_deref())),
            )
            .enumerate()
            .find_map(|(index, (left, right))| {
                (left != right)
                    .then(|| difference("source", format!("connection {index}"), left, right))
            })
    })
    .or_else(|| compare_slice_length("source", "node count", left.nodes.len(), right.nodes.len()))
}

fn compare_geometry(
    left: &FbxGeometryEvidence,
    right: &FbxGeometryEvidence,
) -> Option<FbxComparisonDifference> {
    compare_slice_length(
        "geometry",
        "mesh count",
        left.meshes.len(),
        right.meshes.len(),
    )
    .or_else(|| {
        left.meshes
            .iter()
            .zip(&right.meshes)
            .enumerate()
            .find_map(|(index, (left, right))| {
                compare_value(
                    "geometry",
                    format!("mesh {index} control points"),
                    &left.control_points,
                    &right.control_points,
                )
                .or_else(|| {
                    compare_value(
                        "geometry",
                        format!("mesh {index} polygons"),
                        &left.polygons,
                        &right.polygons,
                    )
                })
                .or_else(|| {
                    compare_value(
                        "geometry",
                        format!("mesh {index} triangles"),
                        &left.triangles,
                        &right.triangles,
                    )
                })
                .or_else(|| {
                    compare_value(
                        "geometry",
                        format!("mesh {index} normals"),
                        &left.normal_layer,
                        &right.normal_layer,
                    )
                })
                .or_else(|| {
                    compare_value(
                        "geometry",
                        format!("mesh {index} UVs"),
                        &left.uv_layer,
                        &right.uv_layer,
                    )
                })
                .or_else(|| {
                    compare_value(
                        "geometry",
                        format!("mesh {index} materials"),
                        &left.material_layer,
                        &right.material_layer,
                    )
                })
                .or_else(|| {
                    compare_value(
                        "geometry",
                        format!("mesh {index} bounds"),
                        &left.bounds,
                        &right.bounds,
                    )
                })
            })
    })
}

fn compare_transforms(
    left: &FbxTransformEvidence,
    right: &FbxTransformEvidence,
) -> Option<FbxComparisonDifference> {
    compare_value("transform", "axis metadata", &left.axes, &right.axes)
        .or_else(|| {
            compare_slice_length(
                "transform",
                "node count",
                left.nodes.len(),
                right.nodes.len(),
            )
        })
        .or_else(|| {
            left.nodes
                .iter()
                .zip(&right.nodes)
                .enumerate()
                .find_map(|(index, (left, right))| {
                    compare_value(
                        "transform",
                        format!("node {index} translation"),
                        &left.local_translation,
                        &right.local_translation,
                    )
                    .or_else(|| {
                        compare_value(
                            "transform",
                            format!("node {index} rotation"),
                            &left.local_rotation_degrees_xyz,
                            &right.local_rotation_degrees_xyz,
                        )
                    })
                    .or_else(|| {
                        compare_value(
                            "transform",
                            format!("node {index} scale"),
                            &left.local_scale,
                            &right.local_scale,
                        )
                    })
                    .or_else(|| {
                        compare_value(
                            "transform",
                            format!("node {index} local matrix"),
                            &left.local_matrix,
                            &right.local_matrix,
                        )
                    })
                    .or_else(|| {
                        compare_value(
                            "transform",
                            format!("node {index} world matrix"),
                            &left.world_matrix,
                            &right.world_matrix,
                        )
                    })
                })
        })
}

fn compare_slice_length(
    stage: &str,
    observation: &str,
    left: usize,
    right: usize,
) -> Option<FbxComparisonDifference> {
    (left != right).then(|| difference(stage, observation, left, right))
}

fn compare_value<T: std::fmt::Debug + PartialEq>(
    stage: &str,
    observation: impl Into<String>,
    left: &T,
    right: &T,
) -> Option<FbxComparisonDifference> {
    (left != right).then(|| difference(stage, observation, left, right))
}

fn difference(
    stage: impl Into<String>,
    observation: impl Into<String>,
    left: impl std::fmt::Debug,
    right: impl std::fmt::Debug,
) -> FbxComparisonDifference {
    FbxComparisonDifference {
        stage: stage.into(),
        observation: observation.into(),
        left: format!("{left:?}"),
        right: format!("{right:?}"),
    }
}
