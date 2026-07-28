use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{CorpusError, CorpusResult};

pub type TransformMatrix = [f32; 16];

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DecodedNode {
    pub index: usize,
    pub name: Option<String>,
    pub mesh: Option<usize>,
    pub children: Vec<usize>,
    /// Column-major glTF local transform.
    pub local_transform: TransformMatrix,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DecodedSceneNode {
    pub node: usize,
    /// Column-major world transform relative to the scene root.
    pub world_transform: TransformMatrix,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DecodedScene {
    pub index: usize,
    pub name: Option<String>,
    pub roots: Vec<usize>,
    pub traversal: Vec<DecodedSceneNode>,
}

pub(crate) fn decode_scenes(
    root: &Value,
    mesh_count: usize,
) -> CorpusResult<(Vec<DecodedNode>, Vec<DecodedScene>)> {
    let nodes = root
        .get("nodes")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .enumerate()
        .map(|(index, node)| decode_node(node, index, mesh_count))
        .collect::<CorpusResult<Vec<_>>>()?;

    for node in &nodes {
        for child in &node.children {
            if *child >= nodes.len() {
                return Err(invalid(format!(
                    "node {} references missing child {child}",
                    node.index
                )));
            }
        }
    }

    let scenes = root
        .get("scenes")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .enumerate()
        .map(|(index, scene)| decode_scene(scene, index, &nodes))
        .collect::<CorpusResult<Vec<_>>>()?;

    Ok((nodes, scenes))
}

fn decode_node(node: &Value, index: usize, mesh_count: usize) -> CorpusResult<DecodedNode> {
    let mesh = node
        .get("mesh")
        .map(|value| index_value(value, "mesh"))
        .transpose()?;
    if let Some(mesh) = mesh.filter(|mesh| *mesh >= mesh_count) {
        return Err(invalid(format!(
            "node {index} references missing mesh {mesh}"
        )));
    }

    let children =
        node.get("children")
            .and_then(Value::as_array)
            .map_or(Ok(Vec::new()), |children| {
                children
                    .iter()
                    .map(|value| index_value(value, "child"))
                    .collect()
            })?;

    Ok(DecodedNode {
        index,
        name: node.get("name").and_then(Value::as_str).map(str::to_owned),
        mesh,
        children,
        local_transform: local_transform(node)?,
    })
}

fn decode_scene(scene: &Value, index: usize, nodes: &[DecodedNode]) -> CorpusResult<DecodedScene> {
    let roots = scene
        .get("nodes")
        .and_then(Value::as_array)
        .map_or(Ok(Vec::new()), |roots| {
            roots
                .iter()
                .map(|value| index_value(value, "scene root node"))
                .collect()
        })?;

    let mut traversal = Vec::new();
    for root in &roots {
        if *root >= nodes.len() {
            return Err(invalid(format!(
                "scene {index} references missing root node {root}"
            )));
        }
        let mut active = vec![false; nodes.len()];
        visit_node(*root, identity(), nodes, &mut active, &mut traversal)?;
    }

    Ok(DecodedScene {
        index,
        name: scene.get("name").and_then(Value::as_str).map(str::to_owned),
        roots,
        traversal,
    })
}

fn visit_node(
    index: usize,
    parent_transform: TransformMatrix,
    nodes: &[DecodedNode],
    active: &mut [bool],
    traversal: &mut Vec<DecodedSceneNode>,
) -> CorpusResult<()> {
    if active[index] {
        return Err(invalid(format!(
            "node hierarchy contains a cycle at node {index}"
        )));
    }
    active[index] = true;
    let world_transform = multiply(parent_transform, nodes[index].local_transform);
    traversal.push(DecodedSceneNode {
        node: index,
        world_transform,
    });
    for child in &nodes[index].children {
        visit_node(*child, world_transform, nodes, active, traversal)?;
    }
    active[index] = false;
    Ok(())
}

fn local_transform(node: &Value) -> CorpusResult<TransformMatrix> {
    let has_matrix = node.get("matrix").is_some();
    let has_trs = ["translation", "rotation", "scale"]
        .iter()
        .any(|key| node.get(*key).is_some());
    if has_matrix && has_trs {
        return Err(invalid("node defines both matrix and TRS properties"));
    }
    if let Some(matrix) = node.get("matrix") {
        return matrix_value(matrix, 16, "matrix")
            .map(|values| values.try_into().expect("16 values"));
    }

    let translation = node
        .get("translation")
        .map(|value| matrix_value(value, 3, "translation"))
        .transpose()?
        .unwrap_or_else(|| vec![0.0, 0.0, 0.0]);
    let rotation = node
        .get("rotation")
        .map(|value| matrix_value(value, 4, "rotation"))
        .transpose()?
        .unwrap_or_else(|| vec![0.0, 0.0, 0.0, 1.0]);
    let scale = node
        .get("scale")
        .map(|value| matrix_value(value, 3, "scale"))
        .transpose()?
        .unwrap_or_else(|| vec![1.0, 1.0, 1.0]);
    Ok(trs_matrix(
        [translation[0], translation[1], translation[2]],
        [rotation[0], rotation[1], rotation[2], rotation[3]],
        [scale[0], scale[1], scale[2]],
    ))
}

fn matrix_value(value: &Value, expected: usize, label: &str) -> CorpusResult<Vec<f32>> {
    let values = value
        .as_array()
        .ok_or_else(|| invalid(format!("node {label} is not an array")))?;
    if values.len() != expected {
        return Err(invalid(format!(
            "node {label} must contain {expected} values, got {}",
            values.len()
        )));
    }
    values
        .iter()
        .map(|value| {
            let value = value
                .as_f64()
                .ok_or_else(|| invalid(format!("node {label} value is not numeric")))?;
            let value = value as f32;
            if value.is_finite() {
                Ok(value)
            } else {
                Err(invalid(format!("node {label} contains a non-finite value")))
            }
        })
        .collect()
}

fn index_value(value: &Value, label: &str) -> CorpusResult<usize> {
    let value = value
        .as_u64()
        .ok_or_else(|| invalid(format!("node {label} is not an unsigned integer")))?;
    usize::try_from(value).map_err(|_| invalid(format!("node {label} exceeds platform range")))
}

fn identity() -> TransformMatrix {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

fn trs_matrix(translation: [f32; 3], rotation: [f32; 4], scale: [f32; 3]) -> TransformMatrix {
    let [x, y, z, w] = rotation;
    let [sx, sy, sz] = scale;
    [
        (1.0 - 2.0 * (y * y + z * z)) * sx,
        (2.0 * (x * y + z * w)) * sx,
        (2.0 * (x * z - y * w)) * sx,
        0.0,
        (2.0 * (x * y - z * w)) * sy,
        (1.0 - 2.0 * (x * x + z * z)) * sy,
        (2.0 * (y * z + x * w)) * sy,
        0.0,
        (2.0 * (x * z + y * w)) * sz,
        (2.0 * (y * z - x * w)) * sz,
        (1.0 - 2.0 * (x * x + y * y)) * sz,
        0.0,
        translation[0],
        translation[1],
        translation[2],
        1.0,
    ]
}

fn multiply(left: TransformMatrix, right: TransformMatrix) -> TransformMatrix {
    let mut result = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            result[column * 4 + row] = (0..4)
                .map(|index| left[index * 4 + row] * right[column * 4 + index])
                .sum();
        }
    }
    result
}

fn invalid(message: impl Into<String>) -> CorpusError {
    CorpusError::InvalidDocument(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trs_transform_is_column_major() {
        let transform = trs_matrix([2.0, 3.0, 4.0], [0.0, 0.0, 0.0, 1.0], [5.0, 6.0, 7.0]);
        assert_eq!(transform[0], 5.0);
        assert_eq!(transform[5], 6.0);
        assert_eq!(transform[10], 7.0);
        assert_eq!(&transform[12..15], &[2.0, 3.0, 4.0]);
    }

    #[test]
    fn rejects_cyclic_node_hierarchy() {
        let root: Value = serde_json::json!({
            "nodes": [{"children": [1]}, {"children": [0]}],
            "scenes": [{"nodes": [0]}]
        });
        let error = decode_scenes(&root, 0).expect_err("cycles must be rejected");
        assert!(error.to_string().contains("cycle"));
    }
}
