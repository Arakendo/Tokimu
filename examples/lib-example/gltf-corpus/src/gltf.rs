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

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct GltfInspection {
    pub summary: GltfSummary,
    pub buffers: Vec<BufferReference>,
    pub resolved_buffers: Vec<ResolvedBuffer>,
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

    Ok(GltfInspection {
        summary,
        buffers,
        resolved_buffers: Vec::new(),
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
