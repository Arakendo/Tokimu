use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    glb::parse_glb, gltf::ensure_version_2, scene::decode_scenes, CorpusError, CorpusResult,
    DecodedNode, DecodedScene, GltfSummary,
};

const MODE_TRIANGLES: u32 = 4;
const COMPONENT_U8: u32 = 5_121;
const COMPONENT_U16: u32 = 5_123;
const COMPONENT_U32: u32 = 5_125;
const COMPONENT_F32: u32 = 5_126;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct PrimitiveLocation {
    pub mesh: usize,
    pub primitive: usize,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct DecodedBounds {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DecodedPrimitive {
    pub location: PrimitiveLocation,
    pub mode: u32,
    pub material: Option<usize>,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tex_coords_0: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub bounds: DecodedBounds,
}

/// A bounded glTF animation profile retained as importer evidence.
///
/// The current corpus admits finite, monotonically ordered translation keys.
/// Rotation, scale, weights, and interpolation modes beyond linear remain
/// explicit future work.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DecodedTranslationChannel {
    pub node: usize,
    pub times: Vec<f32>,
    pub translations: Vec<[f32; 3]>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DecodedAnimation {
    pub name: Option<String>,
    pub channels: Vec<DecodedTranslationChannel>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DecodedModel {
    pub summary: GltfSummary,
    pub nodes: Vec<DecodedNode>,
    pub scenes: Vec<DecodedScene>,
    pub primitives: Vec<DecodedPrimitive>,
    pub animations: Vec<DecodedAnimation>,
}

pub fn decode_gltf(bytes: &[u8], base_path: impl AsRef<Path>) -> CorpusResult<DecodedModel> {
    let root: Value = serde_json::from_slice(bytes)?;
    let summary = GltfSummary::from_root(&root);
    ensure_version_2(&summary.asset_version)?;
    let buffers = load_gltf_buffers(&root, base_path.as_ref())?;
    decode_document(&root, &buffers)
}

pub fn decode_gltf_file(path: impl AsRef<Path>) -> CorpusResult<DecodedModel> {
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(|source| CorpusError::Read {
        path: path.to_owned(),
        source,
    })?;
    decode_gltf(&bytes, path.parent().unwrap_or_else(|| Path::new(".")))
}

pub fn decode_glb(bytes: &[u8]) -> CorpusResult<DecodedModel> {
    let parsed = parse_glb(bytes)?;
    let buffers = load_glb_buffers(&parsed.root, parsed.binary_chunk)?;
    decode_document(&parsed.root, &buffers)
}

pub fn decode_glb_file(path: impl AsRef<Path>) -> CorpusResult<DecodedModel> {
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(|source| CorpusError::Read {
        path: path.to_owned(),
        source,
    })?;
    decode_glb(&bytes)
}

fn decode_document(root: &Value, buffers: &[Vec<u8>]) -> CorpusResult<DecodedModel> {
    let summary = GltfSummary::from_root(root);
    let meshes = array(root, "meshes")?;
    let (nodes, scenes) = decode_scenes(root, meshes.len())?;
    let mut primitives = Vec::new();

    for (mesh_index, mesh) in meshes.iter().enumerate() {
        for (primitive_index, primitive) in array(mesh, "primitives")?.iter().enumerate() {
            let location = PrimitiveLocation {
                mesh: mesh_index,
                primitive: primitive_index,
            };
            let mode = primitive
                .get("mode")
                .and_then(Value::as_u64)
                .unwrap_or(MODE_TRIANGLES as u64) as u32;
            if mode != MODE_TRIANGLES {
                return Err(CorpusError::UnsupportedAccessor(format!(
                    "mesh {mesh_index} primitive {primitive_index} uses mode {mode}; only TRIANGLES is admitted"
                )));
            }

            let attributes = primitive
                .get("attributes")
                .and_then(Value::as_object)
                .ok_or_else(|| invalid("mesh primitive has no attributes object"))?;
            let position_accessor = index_value(attributes.get("POSITION"), "POSITION accessor")?;
            let positions = read_vec3_f32(root, buffers, position_accessor, "POSITION")?;
            if positions.is_empty() {
                return Err(invalid("POSITION accessor contains no vertices"));
            }
            let normals = match attributes.get("NORMAL") {
                Some(value) => {
                    let accessor = index_value(Some(value), "NORMAL accessor")?;
                    let values = read_vec3_f32(root, buffers, accessor, "NORMAL")?;
                    if values.len() != positions.len() {
                        return Err(invalid(format!(
                            "NORMAL count {} does not match POSITION count {}",
                            values.len(),
                            positions.len()
                        )));
                    }
                    values
                }
                None => Vec::new(),
            };
            let tex_coords_0 = match attributes.get("TEXCOORD_0") {
                Some(value) => {
                    let accessor = index_value(Some(value), "TEXCOORD_0 accessor")?;
                    let values = read_vec2_f32(root, buffers, accessor, "TEXCOORD_0")?;
                    if values.len() != positions.len() {
                        return Err(invalid(format!(
                            "TEXCOORD_0 count {} does not match POSITION count {}",
                            values.len(),
                            positions.len()
                        )));
                    }
                    values
                }
                None => Vec::new(),
            };
            let indices = match primitive.get("indices") {
                Some(value) => {
                    let accessor = index_value(Some(value), "index accessor")?;
                    read_indices(root, buffers, accessor)?
                }
                None => (0..positions.len())
                    .map(|index| {
                        u32::try_from(index)
                            .map_err(|_| invalid("vertex count exceeds 32-bit index range"))
                    })
                    .collect::<CorpusResult<Vec<_>>>()?,
            };
            if indices.len() % 3 != 0 {
                return Err(invalid(format!(
                    "triangle index count {} is not divisible by three",
                    indices.len()
                )));
            }
            if let Some(index) = indices
                .iter()
                .copied()
                .find(|index| *index as usize >= positions.len())
            {
                return Err(invalid(format!(
                    "index {index} exceeds vertex count {}",
                    positions.len()
                )));
            }

            let bounds = bounds(&positions)?;
            primitives.push(DecodedPrimitive {
                location,
                mode,
                material: primitive
                    .get("material")
                    .and_then(Value::as_u64)
                    .map(|value| value as usize),
                positions,
                normals,
                tex_coords_0,
                indices,
                bounds,
            });
        }
    }

    let animations = decode_translation_animations(root, buffers, nodes.len())?;

    Ok(DecodedModel {
        summary,
        nodes,
        scenes,
        primitives,
        animations,
    })
}

fn decode_translation_animations(
    root: &Value,
    buffers: &[Vec<u8>],
    node_count: usize,
) -> CorpusResult<Vec<DecodedAnimation>> {
    let animations = root
        .get("animations")
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice);
    animations
        .iter()
        .enumerate()
        .map(|(animation_index, animation)| {
            let samplers = array(animation, "samplers")?;
            let channels = array(animation, "channels")?
                .iter()
                .enumerate()
                .map(|(channel_index, channel)| {
                    let target = channel.get("target").and_then(Value::as_object).ok_or_else(|| {
                        invalid(format!("animation {animation_index} channel {channel_index} has no target"))
                    })?;
                    let path = target.get("path").and_then(Value::as_str).unwrap_or_default();
                    if path != "translation" {
                        return Err(CorpusError::UnsupportedAccessor(format!(
                            "animation {animation_index} channel {channel_index} targets `{path}`; only translation is admitted"
                        )));
                    }
                    let node = index_value(target.get("node"), "animation target node")?;
                    if node >= node_count {
                        return Err(invalid(format!(
                            "animation {animation_index} channel {channel_index} references missing node {node}"
                        )));
                    }
                    let sampler_index = index_value(channel.get("sampler"), "animation sampler")?;
                    let sampler = samplers.get(sampler_index).ok_or_else(|| {
                        invalid(format!("animation {animation_index} channel {channel_index} references missing sampler {sampler_index}"))
                    })?;
                    let interpolation = sampler.get("interpolation").and_then(Value::as_str).unwrap_or("LINEAR");
                    if interpolation != "LINEAR" {
                        return Err(CorpusError::UnsupportedAccessor(format!(
                            "animation {animation_index} sampler {sampler_index} uses `{interpolation}` interpolation; only LINEAR is admitted"
                        )));
                    }
                    let times = read_scalar_f32(
                        root,
                        buffers,
                        index_value(sampler.get("input"), "animation input accessor")?,
                        "animation input",
                    )?;
                    let translations = read_vec3_f32(
                        root,
                        buffers,
                        index_value(sampler.get("output"), "animation output accessor")?,
                        "animation translation output",
                    )?;
                    if times.is_empty() || times.len() != translations.len() {
                        return Err(invalid(format!(
                            "animation {animation_index} channel {channel_index} has {} times and {} translations",
                            times.len(), translations.len()
                        )));
                    }
                    if times.windows(2).any(|pair| pair[1] <= pair[0]) {
                        return Err(invalid(format!(
                            "animation {animation_index} channel {channel_index} key times are not strictly increasing"
                        )));
                    }
                    Ok(DecodedTranslationChannel { node, times, translations })
                })
                .collect::<CorpusResult<Vec<_>>>()?;
            Ok(DecodedAnimation {
                name: animation.get("name").and_then(Value::as_str).map(str::to_owned),
                channels,
            })
        })
        .collect()
}

fn load_gltf_buffers(root: &Value, base_path: &Path) -> CorpusResult<Vec<Vec<u8>>> {
    array(root, "buffers")?
        .iter()
        .enumerate()
        .map(|(index, buffer)| {
            let uri = buffer
                .get("uri")
                .and_then(Value::as_str)
                .ok_or_else(|| invalid(format!("buffer {index} has no URI outside a GLB")))?;
            if uri.starts_with("data:") || uri.contains("://") || Path::new(uri).is_absolute() {
                return Err(CorpusError::UnsupportedBufferUri(uri.to_owned()));
            }
            let path = base_path.join(uri);
            let bytes = fs::read(&path).map_err(|_| CorpusError::MissingBuffer(path.clone()))?;
            validate_buffer_length(buffer, index, &path, bytes.len())?;
            Ok(bytes)
        })
        .collect()
}

fn load_glb_buffers(root: &Value, binary_chunk: Option<&[u8]>) -> CorpusResult<Vec<Vec<u8>>> {
    let buffers = array(root, "buffers")?;
    let binary =
        binary_chunk.ok_or_else(|| invalid("GLB declares a buffer but has no BIN chunk"))?;
    let mut resolved = Vec::with_capacity(buffers.len());

    for (index, buffer) in buffers.iter().enumerate() {
        if let Some(uri) = buffer.get("uri").and_then(Value::as_str) {
            return Err(CorpusError::UnsupportedBufferUri(uri.to_owned()));
        }

        if index == 0 {
            validate_buffer_length(buffer, index, Path::new("<glb-bin>"), binary.len())?;
            resolved.push(binary.to_vec());
            continue;
        }

        let byte_length = buffer
            .get("byteLength")
            .and_then(Value::as_u64)
            .ok_or_else(|| invalid(format!("buffer {index} has no byteLength")))?;
        let byte_length = usize::try_from(byte_length)
            .map_err(|_| invalid(format!("buffer {index} exceeds addressable memory")))?;
        resolved.push(vec![0; byte_length]);
    }

    decode_meshopt_buffer_views(root, &mut resolved)?;
    Ok(resolved)
}

/// Reconstruct the bounded `EXT_meshopt_compression` profile used by the
/// hole-punch corpus asset. Compressed views remain an importer detail: once
/// reconstructed, ordinary accessors consume the declared logical buffer.
fn decode_meshopt_buffer_views(root: &Value, buffers: &mut [Vec<u8>]) -> CorpusResult<()> {
    for (view_index, view) in array(root, "bufferViews")?.iter().enumerate() {
        let Some(extension) = view
            .get("extensions")
            .and_then(|extensions| extensions.get("EXT_meshopt_compression"))
        else {
            continue;
        };

        let destination_buffer = index_value(view.get("buffer"), "bufferView buffer")?;
        let destination_offset =
            extension_optional_usize(view.get("byteOffset"), "bufferView byteOffset")?
                .unwrap_or_default();
        let destination_length =
            extension_required_usize(view.get("byteLength"), "bufferView byteLength")?;
        let destination_end = destination_offset
            .checked_add(destination_length)
            .ok_or_else(|| invalid(format!("bufferView {view_index} output range overflows")))?;

        let source_buffer = index_value(extension.get("buffer"), "meshopt source buffer")?;
        let source_offset =
            extension_optional_usize(extension.get("byteOffset"), "meshopt byteOffset")?
                .unwrap_or_default();
        let source_length =
            extension_required_usize(extension.get("byteLength"), "meshopt byteLength")?;
        let source_end = source_offset.checked_add(source_length).ok_or_else(|| {
            invalid(format!(
                "bufferView {view_index} compressed range overflows"
            ))
        })?;
        let count = extension_required_usize(extension.get("count"), "meshopt count")?;
        let stride = extension_required_usize(extension.get("byteStride"), "meshopt byteStride")?;
        let expected_length = count
            .checked_mul(stride)
            .ok_or_else(|| invalid(format!("bufferView {view_index} decoded length overflows")))?;
        if expected_length != destination_length {
            return Err(invalid(format!(
                "bufferView {view_index} decoded length {expected_length} does not match declared {destination_length}"
            )));
        }

        let source = buffers
            .get(source_buffer)
            .and_then(|buffer| buffer.get(source_offset..source_end))
            .ok_or_else(|| {
                invalid(format!(
                    "bufferView {view_index} compressed data exceeds buffer {source_buffer}"
                ))
            })?
            .to_vec();
        let destination = buffers
            .get_mut(destination_buffer)
            .and_then(|buffer| buffer.get_mut(destination_offset..destination_end))
            .ok_or_else(|| {
                invalid(format!(
                    "bufferView {view_index} output exceeds buffer {destination_buffer}"
                ))
            })?;

        let mode = extension
            .get("mode")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid(format!("bufferView {view_index} meshopt mode is missing")))?;
        let status = match mode {
            "ATTRIBUTES" => unsafe {
                meshopt::ffi::meshopt_decodeVertexBuffer(
                    destination.as_mut_ptr().cast(),
                    count,
                    stride,
                    source.as_ptr(),
                    source.len(),
                )
            },
            "TRIANGLES" => {
                let index_size = destination_length
                    .checked_div(count)
                    .filter(|size| *size == 2 || *size == 4)
                    .ok_or_else(|| {
                        invalid(format!(
                            "bufferView {view_index} meshopt triangle index width is unsupported"
                        ))
                    })?;
                unsafe {
                    meshopt::ffi::meshopt_decodeIndexBuffer(
                        destination.as_mut_ptr().cast(),
                        count,
                        index_size,
                        source.as_ptr(),
                        source.len(),
                    )
                }
            }
            other => {
                return Err(CorpusError::UnsupportedAccessor(format!(
                    "bufferView {view_index} uses unsupported EXT_meshopt_compression mode {other}"
                )));
            }
        };
        if status != 0 {
            return Err(invalid(format!(
                "bufferView {view_index} meshopt decode failed with status {status}"
            )));
        }

        match extension.get("filter").and_then(Value::as_str) {
            None | Some("NONE") => {}
            Some("EXPONENTIAL") => unsafe {
                meshopt::ffi::meshopt_decodeFilterExp(
                    destination.as_mut_ptr().cast(),
                    count,
                    stride,
                );
            },
            Some(filter) => {
                return Err(CorpusError::UnsupportedAccessor(format!(
                    "bufferView {view_index} uses unsupported EXT_meshopt_compression filter {filter}"
                )));
            }
        }
    }
    Ok(())
}

fn extension_required_usize(value: Option<&Value>, description: &str) -> CorpusResult<usize> {
    let value = value
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid(format!("{description} is missing or invalid")))?;
    usize::try_from(value).map_err(|_| invalid(format!("{description} exceeds addressable memory")))
}

fn extension_optional_usize(
    value: Option<&Value>,
    description: &str,
) -> CorpusResult<Option<usize>> {
    value
        .map(|value| extension_required_usize(Some(value), description))
        .transpose()
}

fn validate_buffer_length(
    buffer: &Value,
    index: usize,
    path: &Path,
    actual: usize,
) -> CorpusResult<()> {
    let expected = buffer
        .get("byteLength")
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid(format!("buffer {index} has no byteLength")))?;
    if (actual as u64) < expected {
        return Err(CorpusError::ShortBuffer {
            path: PathBuf::from(path),
            expected,
            actual: actual as u64,
        });
    }
    Ok(())
}

fn read_vec3_f32(
    root: &Value,
    buffers: &[Vec<u8>],
    accessor_index: usize,
    semantic: &str,
) -> CorpusResult<Vec<[f32; 3]>> {
    let view = AccessorView::new(root, buffers, accessor_index)?;
    if view.component_type != COMPONENT_F32 || view.accessor_type != "VEC3" {
        return Err(CorpusError::UnsupportedAccessor(format!(
            "{semantic} accessor {accessor_index} must be FLOAT VEC3, got component {} {}",
            view.component_type, view.accessor_type
        )));
    }
    let values = view
        .elements(12)?
        .map(|element| {
            let element = element?;
            let value = [
                read_f32(element, 0)?,
                read_f32(element, 4)?,
                read_f32(element, 8)?,
            ];
            if value.iter().any(|component| !component.is_finite()) {
                return Err(invalid(format!(
                    "{semantic} accessor {accessor_index} contains a non-finite value"
                )));
            }
            Ok(value)
        })
        .collect();
    values
}

fn read_vec2_f32(
    root: &Value,
    buffers: &[Vec<u8>],
    accessor_index: usize,
    semantic: &str,
) -> CorpusResult<Vec<[f32; 2]>> {
    let view = AccessorView::new(root, buffers, accessor_index)?;
    if view.component_type != COMPONENT_F32 || view.accessor_type != "VEC2" {
        return Err(CorpusError::UnsupportedAccessor(format!(
            "{semantic} accessor {accessor_index} must be FLOAT VEC2, got component {} {}",
            view.component_type, view.accessor_type
        )));
    }
    let values = view
        .elements(8)?
        .map(|element| {
            let element = element?;
            let value = [read_f32(element, 0)?, read_f32(element, 4)?];
            if value.iter().any(|component| !component.is_finite()) {
                return Err(invalid(format!(
                    "{semantic} accessor {accessor_index} contains a non-finite value"
                )));
            }
            Ok(value)
        })
        .collect();
    values
}

fn read_scalar_f32(
    root: &Value,
    buffers: &[Vec<u8>],
    accessor_index: usize,
    semantic: &str,
) -> CorpusResult<Vec<f32>> {
    let view = AccessorView::new(root, buffers, accessor_index)?;
    if view.component_type != COMPONENT_F32 || view.accessor_type != "SCALAR" {
        return Err(CorpusError::UnsupportedAccessor(format!(
            "{semantic} accessor {accessor_index} must be FLOAT SCALAR, got component {} {}",
            view.component_type, view.accessor_type
        )));
    }
    let values = view
        .elements(4)?
        .map(|element| {
            let value = read_f32(element?, 0)?;
            if value.is_finite() {
                Ok(value)
            } else {
                Err(invalid(format!(
                    "{semantic} accessor {accessor_index} contains a non-finite value"
                )))
            }
        })
        .collect();
    values
}

fn read_indices(
    root: &Value,
    buffers: &[Vec<u8>],
    accessor_index: usize,
) -> CorpusResult<Vec<u32>> {
    let view = AccessorView::new(root, buffers, accessor_index)?;
    if view.accessor_type != "SCALAR" {
        return Err(CorpusError::UnsupportedAccessor(format!(
            "index accessor {accessor_index} must be SCALAR, got {}",
            view.accessor_type
        )));
    }
    let component_size = match view.component_type {
        COMPONENT_U8 => 1,
        COMPONENT_U16 => 2,
        COMPONENT_U32 => 4,
        component => {
            return Err(CorpusError::UnsupportedAccessor(format!(
                "index accessor {accessor_index} uses component type {component}"
            )));
        }
    };
    let values = view
        .elements(component_size)?
        .map(|element| {
            let element = element?;
            match component_size {
                1 => Ok(u32::from(element[0])),
                2 => Ok(u32::from(u16::from_le_bytes(
                    element[..2].try_into().expect("element size is checked"),
                ))),
                4 => Ok(u32::from_le_bytes(
                    element[..4].try_into().expect("element size is checked"),
                )),
                _ => unreachable!("component size is constrained above"),
            }
        })
        .collect();
    values
}

struct AccessorView<'a> {
    bytes: &'a [u8],
    start: usize,
    count: usize,
    stride: usize,
    view_end: usize,
    component_type: u32,
    accessor_type: &'a str,
}

impl<'a> AccessorView<'a> {
    fn new(root: &'a Value, buffers: &'a [Vec<u8>], accessor_index: usize) -> CorpusResult<Self> {
        let accessor = array(root, "accessors")?
            .get(accessor_index)
            .ok_or_else(|| invalid(format!("missing accessor {accessor_index}")))?;
        if accessor.get("sparse").is_some() {
            return Err(CorpusError::UnsupportedAccessor(format!(
                "accessor {accessor_index} is sparse"
            )));
        }
        let view_index = usize_value(accessor, "bufferView")?;
        let view = array(root, "bufferViews")?
            .get(view_index)
            .ok_or_else(|| invalid(format!("missing bufferView {view_index}")))?;
        let buffer_index = usize_value(view, "buffer")?;
        let bytes = buffers
            .get(buffer_index)
            .ok_or_else(|| invalid(format!("missing buffer {buffer_index}")))?;
        let view_start = optional_usize(view, "byteOffset")?;
        let view_length = usize_value(view, "byteLength")?;
        let view_end = view_start
            .checked_add(view_length)
            .ok_or_else(|| range("bufferView range overflow"))?;
        if view_end > bytes.len() {
            return Err(range(format!(
                "bufferView {view_index} ends at {view_end}, buffer length is {}",
                bytes.len()
            )));
        }
        let accessor_offset = optional_usize(accessor, "byteOffset")?;
        let start = view_start
            .checked_add(accessor_offset)
            .ok_or_else(|| range("accessor offset overflow"))?;
        if start > view_end {
            return Err(range(format!(
                "accessor {accessor_index} starts at {start}, beyond bufferView end {view_end}"
            )));
        }
        let component_type = u32_value(accessor, "componentType")?;
        let component_size = component_size(component_type)?;
        let component_count = component_count(
            accessor
                .get("type")
                .and_then(Value::as_str)
                .ok_or_else(|| invalid(format!("accessor {accessor_index} has no type")))?,
        )?;
        let element_size = component_size
            .checked_mul(component_count)
            .ok_or_else(|| range("accessor element size overflow"))?;
        let stride = match view.get("byteStride") {
            Some(value) => usize_from_value(value, "byteStride")?,
            None => element_size,
        };
        if stride < element_size {
            return Err(range(format!(
                "accessor {accessor_index} stride {stride} is smaller than element size {element_size}"
            )));
        }

        Ok(Self {
            bytes,
            start,
            count: usize_value(accessor, "count")?,
            stride,
            view_end,
            component_type,
            accessor_type: accessor["type"]
                .as_str()
                .expect("accessor type is checked above"),
        })
    }

    fn elements(
        &'a self,
        element_size: usize,
    ) -> CorpusResult<impl Iterator<Item = CorpusResult<&'a [u8]>> + 'a> {
        if self.count > 0 {
            let final_offset = (self.count - 1)
                .checked_mul(self.stride)
                .ok_or_else(|| range("accessor iteration overflow"))?;
            let final_start = self
                .start
                .checked_add(final_offset)
                .ok_or_else(|| range("accessor iteration overflow"))?;
            let final_end = final_start
                .checked_add(element_size)
                .ok_or_else(|| range("accessor final element overflow"))?;
            if final_end > self.view_end || final_end > self.bytes.len() {
                return Err(range(format!(
                    "accessor final byte {final_end} exceeds view end {}",
                    self.view_end
                )));
            }
        }

        Ok((0..self.count).map(move |index| {
            let start = self
                .start
                .checked_add(
                    index
                        .checked_mul(self.stride)
                        .expect("the final accessor offset is checked above"),
                )
                .expect("the final accessor start is checked above");
            let end = start
                .checked_add(element_size)
                .expect("the final accessor end is checked above");
            self.bytes
                .get(start..end)
                .ok_or_else(|| range(format!("accessor element {index} is out of range")))
        }))
    }
}

fn bounds(positions: &[[f32; 3]]) -> CorpusResult<DecodedBounds> {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for position in positions {
        for axis in 0..3 {
            min[axis] = min[axis].min(position[axis]);
            max[axis] = max[axis].max(position[axis]);
        }
    }
    if min
        .iter()
        .chain(max.iter())
        .any(|component| !component.is_finite())
    {
        return Err(invalid("decoded bounds are not finite"));
    }
    Ok(DecodedBounds { min, max })
}

fn array<'a>(value: &'a Value, key: &str) -> CorpusResult<&'a Vec<Value>> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| invalid(format!("missing `{key}` array")))
}

fn index_value(value: Option<&Value>, label: &str) -> CorpusResult<usize> {
    let value = value
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid(format!("missing or invalid {label}")))?;
    usize::try_from(value).map_err(|_| invalid(format!("{label} exceeds platform range")))
}

fn usize_value(value: &Value, key: &str) -> CorpusResult<usize> {
    value
        .get(key)
        .ok_or_else(|| invalid(format!("missing `{key}`")))
        .and_then(|value| usize_from_value(value, key))
}

fn optional_usize(value: &Value, key: &str) -> CorpusResult<usize> {
    value
        .get(key)
        .map_or(Ok(0), |value| usize_from_value(value, key))
}

fn usize_from_value(value: &Value, label: &str) -> CorpusResult<usize> {
    let value = value
        .as_u64()
        .ok_or_else(|| invalid(format!("`{label}` is not an unsigned integer")))?;
    usize::try_from(value).map_err(|_| invalid(format!("`{label}` exceeds platform range")))
}

fn u32_value(value: &Value, key: &str) -> CorpusResult<u32> {
    let value = value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid(format!("missing or invalid `{key}`")))?;
    u32::try_from(value).map_err(|_| invalid(format!("`{key}` exceeds 32-bit range")))
}

fn component_size(component_type: u32) -> CorpusResult<usize> {
    match component_type {
        5_120 | COMPONENT_U8 => Ok(1),
        5_122 | COMPONENT_U16 => Ok(2),
        COMPONENT_U32 | COMPONENT_F32 => Ok(4),
        other => Err(CorpusError::UnsupportedAccessor(format!(
            "unknown component type {other}"
        ))),
    }
}

fn component_count(accessor_type: &str) -> CorpusResult<usize> {
    match accessor_type {
        "SCALAR" => Ok(1),
        "VEC2" => Ok(2),
        "VEC3" => Ok(3),
        "VEC4" | "MAT2" => Ok(4),
        "MAT3" => Ok(9),
        "MAT4" => Ok(16),
        other => Err(CorpusError::UnsupportedAccessor(format!(
            "unknown accessor type {other}"
        ))),
    }
}

fn read_f32(bytes: &[u8], offset: usize) -> CorpusResult<f32> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| range("truncated f32 component"))?;
    Ok(f32::from_le_bytes(
        value.try_into().expect("slice length is checked"),
    ))
}

fn invalid(message: impl Into<String>) -> CorpusError {
    CorpusError::InvalidDocument(message.into())
}

fn range(message: impl Into<String>) -> CorpusError {
    CorpusError::AccessorRange(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_index_outside_position_range() {
        let json = br#"{
            "asset":{"version":"2.0"},
            "buffers":[{"uri":"mesh.bin","byteLength":39}],
            "bufferViews":[
                {"buffer":0,"byteOffset":0,"byteLength":36},
                {"buffer":0,"byteOffset":36,"byteLength":3}
            ],
            "accessors":[
                {"bufferView":0,"componentType":5126,"count":3,"type":"VEC3"},
                {"bufferView":1,"componentType":5121,"count":3,"type":"SCALAR"}
            ],
            "meshes":[{"primitives":[{"attributes":{"POSITION":0},"indices":1}]}]
        }"#;
        let root: Value = serde_json::from_slice(json).expect("fixture JSON");
        let mut bytes = vec![0_u8; 39];
        bytes[36..].copy_from_slice(&[0, 1, 3]);
        let error = decode_document(&root, &[bytes]).expect_err("index must fail");
        assert!(error.to_string().contains("exceeds vertex count"));
    }

    #[test]
    fn rejects_accessor_that_exceeds_its_buffer_view() {
        let root: Value = serde_json::from_str(
            r#"{
                "bufferViews":[{"buffer":0,"byteLength":12}],
                "accessors":[
                    {"bufferView":0,"componentType":5126,"count":2,"type":"VEC3"}
                ]
            }"#,
        )
        .expect("fixture JSON");
        let buffers = [vec![0_u8; 24]];

        let error = read_vec3_f32(&root, &buffers, 0, "POSITION")
            .expect_err("accessor must remain inside its buffer view");

        assert!(matches!(error, CorpusError::AccessorRange(_)));
    }

    #[test]
    fn rejects_stride_smaller_than_element_size() {
        let root: Value = serde_json::from_str(
            r#"{
                "bufferViews":[{"buffer":0,"byteLength":12,"byteStride":8}],
                "accessors":[
                    {"bufferView":0,"componentType":5126,"count":1,"type":"VEC3"}
                ]
            }"#,
        )
        .expect("fixture JSON");
        let buffers = [vec![0_u8; 12]];

        let error =
            read_vec3_f32(&root, &buffers, 0, "POSITION").expect_err("stride must be rejected");

        assert!(matches!(error, CorpusError::AccessorRange(_)));
        assert!(error.to_string().contains("smaller than element size"));
    }

    #[test]
    fn recognizes_all_core_component_widths() {
        assert_eq!(component_size(5_120).expect("BYTE"), 1);
        assert_eq!(component_size(COMPONENT_U8).expect("UNSIGNED_BYTE"), 1);
        assert_eq!(component_size(5_122).expect("SHORT"), 2);
        assert_eq!(component_size(COMPONENT_U16).expect("UNSIGNED_SHORT"), 2);
        assert_eq!(component_size(COMPONENT_U32).expect("UNSIGNED_INT"), 4);
        assert_eq!(component_size(COMPONENT_F32).expect("FLOAT"), 4);
    }
}
