use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub struct GltfSummary {
    pub asset_version: String,
    pub default_scene: Option<u64>,
    pub scenes: usize,
    pub nodes: usize,
    pub meshes: usize,
    pub primitives: usize,
    pub accessors: usize,
    pub buffer_views: usize,
    pub buffers: usize,
    pub materials: usize,
    pub textures: usize,
    pub images: usize,
    pub animations: usize,
    pub skins: usize,
    pub cameras: usize,
    pub extensions_used: Vec<String>,
    pub extensions_required: Vec<String>,
}

impl GltfSummary {
    pub(crate) fn from_root(root: &Value) -> Self {
        Self {
            asset_version: root
                .pointer("/asset/version")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            default_scene: root.get("scene").and_then(Value::as_u64),
            scenes: array_len(root, "scenes"),
            nodes: array_len(root, "nodes"),
            meshes: array_len(root, "meshes"),
            primitives: root
                .get("meshes")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(|mesh| array_len(mesh, "primitives"))
                .sum(),
            accessors: array_len(root, "accessors"),
            buffer_views: array_len(root, "bufferViews"),
            buffers: array_len(root, "buffers"),
            materials: array_len(root, "materials"),
            textures: array_len(root, "textures"),
            images: array_len(root, "images"),
            animations: array_len(root, "animations"),
            skins: array_len(root, "skins"),
            cameras: array_len(root, "cameras"),
            extensions_used: string_array(root, "extensionsUsed"),
            extensions_required: string_array(root, "extensionsRequired"),
        }
    }
}

fn array_len(value: &Value, key: &str) -> usize {
    value.get(key).and_then(Value::as_array).map_or(0, Vec::len)
}

fn string_array(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}
