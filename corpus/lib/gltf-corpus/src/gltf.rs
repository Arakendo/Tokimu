use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{CorpusError, CorpusResult, GltfSummary};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct BufferReference {
    pub index: usize,
    pub uri: Option<String>,
    pub byte_length: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct ResolvedBuffer {
    pub index: usize,
    pub path: PathBuf,
    pub declared_byte_length: u64,
    pub actual_byte_length: u64,
}

/// Source-level identity of a glTF primitive and its declared topology.
///
/// This remains inspection evidence. It does not claim that the topology can
/// be lowered into Tokimu geometry.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct PrimitiveTopology {
    pub mesh: usize,
    pub primitive: usize,
    pub mode: u32,
}

/// Provider-native PBR material metadata retained as corpus evidence.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MaterialReference {
    pub index: usize,
    pub name: Option<String>,
    pub base_color_factor: Option<[f32; 4]>,
    pub base_color_texture: Option<usize>,
}

/// Source-level link from a glTF texture object to its image and sampler.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct TextureReference {
    pub index: usize,
    pub source: Option<usize>,
    pub sampler: Option<usize>,
}

/// Source-level image identity. Image bytes remain owned by the importer.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct ImageReference {
    pub index: usize,
    pub uri: Option<String>,
    pub mime_type: Option<String>,
    pub buffer_view: Option<usize>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GltfInspection {
    pub summary: GltfSummary,
    pub buffers: Vec<BufferReference>,
    pub resolved_buffers: Vec<ResolvedBuffer>,
    pub primitive_topologies: Vec<PrimitiveTopology>,
    pub materials: Vec<MaterialReference>,
    pub textures: Vec<TextureReference>,
    pub images: Vec<ImageReference>,
}

pub fn inspect_gltf(bytes: &[u8]) -> CorpusResult<GltfInspection> {
    let root: Value = serde_json::from_slice(bytes)?;
    let summary = GltfSummary::from_root(&root);
    ensure_version_2(&summary.asset_version)?;

    let buffers = root
        .get("buffers")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(index, buffer)| BufferReference {
            index,
            uri: buffer.get("uri").and_then(Value::as_str).map(str::to_owned),
            byte_length: buffer
                .get("byteLength")
                .and_then(Value::as_u64)
                .unwrap_or_default(),
        })
        .collect();
    let primitive_topologies = root
        .get("meshes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .flat_map(|(mesh, value)| {
            value
                .get("primitives")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .enumerate()
                .map(move |(primitive, value)| PrimitiveTopology {
                    mesh,
                    primitive,
                    mode: value
                        .get("mode")
                        .and_then(Value::as_u64)
                        .and_then(|mode| u32::try_from(mode).ok())
                        // glTF defaults omitted primitive modes to TRIANGLES.
                        .unwrap_or(4),
                })
        })
        .collect();
    let images = root
        .get("images")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(index, image)| {
            Ok(ImageReference {
                index,
                uri: image.get("uri").and_then(Value::as_str).map(str::to_owned),
                mime_type: image
                    .get("mimeType")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                buffer_view: optional_index(image, "bufferView")?,
            })
        })
        .collect::<CorpusResult<Vec<_>>>()?;
    let textures = root
        .get("textures")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(index, texture)| {
            let source = optional_index(texture, "source")?;
            if source.is_some_and(|source| source >= images.len()) {
                return Err(invalid(format!(
                    "texture {index} references missing image {}",
                    source.expect("checked above")
                )));
            }
            Ok(TextureReference {
                index,
                source,
                sampler: optional_index(texture, "sampler")?,
            })
        })
        .collect::<CorpusResult<Vec<_>>>()?;
    let materials = root
        .get("materials")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(index, material)| {
            let pbr = material.get("pbrMetallicRoughness").unwrap_or(&Value::Null);
            let base_color_texture = pbr
                .get("baseColorTexture")
                .map(|texture| required_index(texture, "index"))
                .transpose()?;
            if base_color_texture.is_some_and(|texture| texture >= textures.len()) {
                return Err(invalid(format!(
                    "material {index} references missing texture {}",
                    base_color_texture.expect("checked above")
                )));
            }
            Ok(MaterialReference {
                index,
                name: material
                    .get("name")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                base_color_factor: color_factor(pbr.get("baseColorFactor"))?,
                base_color_texture,
            })
        })
        .collect::<CorpusResult<Vec<_>>>()?;

    Ok(GltfInspection {
        summary,
        buffers,
        resolved_buffers: Vec::new(),
        primitive_topologies,
        materials,
        textures,
        images,
    })
}

pub fn inspect_gltf_file(path: impl AsRef<Path>) -> CorpusResult<GltfInspection> {
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(|source| CorpusError::Read {
        path: path.to_owned(),
        source,
    })?;
    let mut inspection = inspect_gltf(&bytes)?;
    let parent = path.parent().unwrap_or_else(|| Path::new("."));

    for buffer in &inspection.buffers {
        let Some(uri) = buffer.uri.as_deref() else {
            continue;
        };
        if uri.starts_with("data:") || uri.contains("://") || Path::new(uri).is_absolute() {
            return Err(CorpusError::UnsupportedBufferUri(uri.to_owned()));
        }

        let buffer_path = parent.join(uri);
        let metadata = fs::metadata(&buffer_path)
            .map_err(|_| CorpusError::MissingBuffer(buffer_path.clone()))?;
        if metadata.len() < buffer.byte_length {
            return Err(CorpusError::ShortBuffer {
                path: buffer_path,
                expected: buffer.byte_length,
                actual: metadata.len(),
            });
        }
        inspection.resolved_buffers.push(ResolvedBuffer {
            index: buffer.index,
            path: buffer_path,
            declared_byte_length: buffer.byte_length,
            actual_byte_length: metadata.len(),
        });
    }

    Ok(inspection)
}

pub(crate) fn ensure_version_2(version: &str) -> CorpusResult<()> {
    if version == "2.0" {
        Ok(())
    } else {
        Err(CorpusError::UnsupportedVersion(version.to_owned()))
    }
}

fn optional_index(value: &Value, key: &str) -> CorpusResult<Option<usize>> {
    value
        .get(key)
        .map(|value| {
            let value = value
                .as_u64()
                .ok_or_else(|| invalid(format!("`{key}` is not an unsigned integer")))?;
            usize::try_from(value).map_err(|_| invalid(format!("`{key}` exceeds platform range")))
        })
        .transpose()
}

fn required_index(value: &Value, key: &str) -> CorpusResult<usize> {
    optional_index(value, key)?.ok_or_else(|| invalid(format!("missing `{key}`")))
}

fn color_factor(value: Option<&Value>) -> CorpusResult<Option<[f32; 4]>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let values = value
        .as_array()
        .ok_or_else(|| invalid("baseColorFactor is not an array"))?;
    let values: Vec<f32> = values
        .iter()
        .map(|value| {
            let value = value
                .as_f64()
                .ok_or_else(|| invalid("baseColorFactor component is not numeric"))?;
            let value = value as f32;
            if !value.is_finite() {
                return Err(invalid("baseColorFactor component is not finite"));
            }
            Ok(value)
        })
        .collect::<CorpusResult<_>>()?;
    values
        .try_into()
        .map(Some)
        .map_err(|_: Vec<f32>| invalid("baseColorFactor must contain four components"))
}

fn invalid(message: impl Into<String>) -> CorpusError {
    CorpusError::InvalidDocument(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_material_texture_reference_outside_source_inventory() {
        let json = br#"{
            "asset":{"version":"2.0"},
            "materials":[{
                "pbrMetallicRoughness":{"baseColorTexture":{"index":1}}
            }],
            "textures":[{"source":0}],
            "images":[{"uri":"texture.png"}]
        }"#;

        let error = inspect_gltf(json).expect_err("missing texture reference must fail");

        assert!(matches!(error, CorpusError::InvalidDocument(_)));
        assert!(error.to_string().contains("missing texture 1"));
    }
}
