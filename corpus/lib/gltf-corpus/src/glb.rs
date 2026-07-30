use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{gltf::ensure_version_2, CorpusError, CorpusResult, GltfSummary};

const GLB_MAGIC: u32 = 0x4654_6c67;
const JSON_CHUNK: u32 = 0x4e4f_534a;
const BIN_CHUNK: u32 = 0x004e_4942;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum GlbChunkKind {
    Json,
    Binary,
    Unknown(u32),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct GlbChunk {
    pub kind: GlbChunkKind,
    pub offset: usize,
    pub byte_length: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct GlbInspection {
    pub version: u32,
    pub declared_byte_length: usize,
    pub chunks: Vec<GlbChunk>,
    pub summary: GltfSummary,
}

pub fn inspect_glb_file(path: impl AsRef<Path>) -> CorpusResult<GlbInspection> {
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(|source| CorpusError::Read {
        path: path.to_owned(),
        source,
    })?;
    inspect_glb(&bytes)
}

pub fn inspect_glb(bytes: &[u8]) -> CorpusResult<GlbInspection> {
    Ok(parse_glb(bytes)?.inspection)
}

#[cfg_attr(not(feature = "decode"), allow(dead_code))]
pub(crate) struct ParsedGlb<'a> {
    pub inspection: GlbInspection,
    pub root: Value,
    pub binary_chunk: Option<&'a [u8]>,
}

pub(crate) fn parse_glb(bytes: &[u8]) -> CorpusResult<ParsedGlb<'_>> {
    if bytes.len() < 12 {
        return Err(invalid("header is shorter than 12 bytes"));
    }

    let magic = read_u32(bytes, 0)?;
    if magic != GLB_MAGIC {
        return Err(invalid("magic is not `glTF`"));
    }

    let version = read_u32(bytes, 4)?;
    if version != 2 {
        return Err(invalid(format!("unsupported container version {version}")));
    }

    let declared_byte_length = read_u32(bytes, 8)? as usize;
    if declared_byte_length != bytes.len() {
        return Err(invalid(format!(
            "declared length {declared_byte_length} does not match actual length {}",
            bytes.len()
        )));
    }

    let mut chunks = Vec::new();
    let mut cursor = 12;
    let mut json_root = None;
    let mut binary_chunk = None;
    while cursor < bytes.len() {
        if bytes.len() - cursor < 8 {
            return Err(invalid("truncated chunk header"));
        }
        let byte_length = read_u32(bytes, cursor)? as usize;
        let chunk_type = read_u32(bytes, cursor + 4)?;
        let data_offset = cursor + 8;
        let end = data_offset
            .checked_add(byte_length)
            .ok_or_else(|| invalid("chunk length overflow"))?;
        if end > bytes.len() {
            return Err(invalid("chunk extends beyond declared container length"));
        }

        let kind = match chunk_type {
            JSON_CHUNK => GlbChunkKind::Json,
            BIN_CHUNK => GlbChunkKind::Binary,
            other => GlbChunkKind::Unknown(other),
        };
        if matches!(kind, GlbChunkKind::Json) {
            if json_root.is_some() {
                return Err(invalid("container has more than one JSON chunk"));
            }
            let json = trim_json_padding(&bytes[data_offset..end]);
            json_root = Some(serde_json::from_slice::<Value>(json)?);
        } else if matches!(kind, GlbChunkKind::Binary) {
            if binary_chunk.is_some() {
                return Err(invalid("container has more than one BIN chunk"));
            }
            binary_chunk = Some(&bytes[data_offset..end]);
        }
        chunks.push(GlbChunk {
            kind,
            offset: data_offset,
            byte_length,
        });
        cursor = end;
    }

    let Some(first) = chunks.first() else {
        return Err(invalid("container has no chunks"));
    };
    if !matches!(first.kind, GlbChunkKind::Json) {
        return Err(invalid("first chunk is not JSON"));
    }
    let root = json_root.ok_or_else(|| invalid("container has no JSON chunk"))?;
    let summary = GltfSummary::from_root(&root);
    ensure_version_2(&summary.asset_version)?;

    Ok(ParsedGlb {
        inspection: GlbInspection {
            version,
            declared_byte_length,
            chunks,
            summary,
        },
        root,
        binary_chunk,
    })
}

fn read_u32(bytes: &[u8], offset: usize) -> CorpusResult<u32> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| invalid("truncated 32-bit field"))?;
    Ok(u32::from_le_bytes(
        value.try_into().expect("slice length is checked"),
    ))
}

fn trim_json_padding(bytes: &[u8]) -> &[u8] {
    let end = bytes
        .iter()
        .rposition(|byte| !matches!(byte, b' ' | 0))
        .map_or(0, |index| index + 1);
    &bytes[..end]
}

fn invalid(message: impl Into<String>) -> CorpusError {
    CorpusError::InvalidGlb(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_glb_magic() {
        let bytes = [0_u8; 12];
        assert!(matches!(
            inspect_glb(&bytes),
            Err(CorpusError::InvalidGlb(message)) if message.contains("magic")
        ));
    }

    #[test]
    fn rejects_declared_length_mismatch() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&GLB_MAGIC.to_le_bytes());
        bytes.extend_from_slice(&2_u32.to_le_bytes());
        bytes.extend_from_slice(&16_u32.to_le_bytes());
        assert!(matches!(
            inspect_glb(&bytes),
            Err(CorpusError::InvalidGlb(message)) if message.contains("declared length")
        ));
    }
}
