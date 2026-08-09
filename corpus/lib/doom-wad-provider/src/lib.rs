//! Corpus-local, bounded inspection for Doom WAD containers.
//!
//! This provider deliberately stops at container observations. It neither
//! interprets map/resource semantics nor retains caller bytes.

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const HEADER_BYTES: usize = 12;
const DIRECTORY_ENTRY_BYTES: usize = 16;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WadReadLimits {
    pub max_input_bytes: usize,
    pub max_lumps: u32,
    pub max_lump_bytes: u32,
    pub max_total_lump_bytes: u64,
}

impl WadReadLimits {
    pub const fn new(
        max_input_bytes: usize,
        max_lumps: u32,
        max_lump_bytes: u32,
        max_total_lump_bytes: u64,
    ) -> Self {
        Self {
            max_input_bytes,
            max_lumps,
            max_lump_bytes,
            max_total_lump_bytes,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WadSourceIdentity {
    /// Caller-provided provenance label; it is never interpreted as a path.
    pub label: String,
    pub byte_len: usize,
    pub blake3: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WadKind {
    Iwad,
    Pwad,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WadNamespaceKind {
    Flats,
    Patches,
    Sprites,
}

impl WadNamespaceKind {
    const ALL: [Self; 3] = [Self::Flats, Self::Patches, Self::Sprites];

    const fn slot(self) -> usize {
        match self {
            Self::Flats => 0,
            Self::Patches => 1,
            Self::Sprites => 2,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WadNamespaceObservation {
    pub kind: WadNamespaceKind,
    pub start_marker_index: u32,
    pub end_marker_index: u32,
    /// Ordered source indices; marker records are not members.
    pub lump_indices: Vec<u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WadLumpObservation {
    pub index: u32,
    pub offset: u32,
    pub size: u32,
    /// ASCII WAD name with trailing NUL padding removed.
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WadManifest {
    pub source: WadSourceIdentity,
    pub kind: WadKind,
    pub directory_offset: u32,
    pub directory_bytes: u64,
    pub total_lump_bytes: u64,
    /// Directory order and duplicate names are intentionally retained.
    pub lumps: Vec<WadLumpObservation>,
    /// Doom WAD marker ranges projected without turning markers into files.
    pub namespaces: Vec<WadNamespaceObservation>,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum WadError {
    #[error("WAD input is {actual_bytes} bytes, exceeding the {limit_bytes}-byte limit")]
    InputLimitExceeded {
        limit_bytes: usize,
        actual_bytes: usize,
    },
    #[error("WAD header is truncated: expected {expected_bytes} bytes, found {actual_bytes}")]
    TruncatedHeader {
        expected_bytes: usize,
        actual_bytes: usize,
    },
    #[error("WAD signature {signature:?} is not IWAD or PWAD")]
    UnknownSignature { signature: [u8; 4] },
    #[error("WAD declares {actual_lumps} lumps, exceeding the {limit_lumps}-lump limit")]
    LumpCountLimitExceeded { limit_lumps: u32, actual_lumps: u32 },
    #[error(
        "WAD directory range offset={offset}, bytes={size} is outside a {input_bytes}-byte input"
    )]
    DirectoryOutOfBounds {
        offset: u32,
        size: u64,
        input_bytes: usize,
    },
    #[error("WAD lump {index} range offset={offset}, bytes={size} is outside a {input_bytes}-byte input")]
    LumpOutOfBounds {
        index: u32,
        offset: u32,
        size: u32,
        input_bytes: usize,
    },
    #[error("WAD lump {index} is {actual_bytes} bytes, exceeding the {limit_bytes}-byte limit")]
    LumpSizeLimitExceeded {
        index: u32,
        limit_bytes: u32,
        actual_bytes: u32,
    },
    #[error("WAD lump bytes total {actual_bytes} exceeds the {limit_bytes}-byte limit")]
    TotalLumpBytesLimitExceeded { limit_bytes: u64, actual_bytes: u64 },
    #[error("WAD lump {index} has malformed name bytes {bytes:?}")]
    MalformedLumpName { index: u32, bytes: [u8; 8] },
    #[error("WAD lumps {first_index} and {second_index} have overlapping byte ranges")]
    OverlappingLumpRanges { first_index: u32, second_index: u32 },
    #[error("WAD namespace {marker_kind:?} marker at lump {second_index} overlaps open {open_kind:?} namespace from lump {first_index}")]
    OverlappingNamespaceMarker {
        open_kind: WadNamespaceKind,
        marker_kind: WadNamespaceKind,
        first_index: u32,
        second_index: u32,
    },
    #[error("WAD namespace {kind:?} ends at lump {index} without an open marker")]
    UnmatchedNamespaceEnd { kind: WadNamespaceKind, index: u32 },
    #[error("WAD namespace {end_kind:?} ends at lump {second_index} while {open_kind:?} from lump {first_index} is open")]
    MismatchedNamespaceEnd {
        open_kind: WadNamespaceKind,
        end_kind: WadNamespaceKind,
        first_index: u32,
        second_index: u32,
    },
    #[error("WAD namespace {kind:?} starts at lump {index} but has no closing marker")]
    UnclosedNamespaceStart { kind: WadNamespaceKind, index: u32 },
}

/// Inspects a WAD byte resource without retaining it or assigning a Resource
/// Space identity. The caller owns source selection and byte transport.
pub fn inspect_wad(
    source_label: impl Into<String>,
    bytes: &[u8],
    limits: WadReadLimits,
) -> Result<WadManifest, WadError> {
    if bytes.len() > limits.max_input_bytes {
        return Err(WadError::InputLimitExceeded {
            limit_bytes: limits.max_input_bytes,
            actual_bytes: bytes.len(),
        });
    }
    if bytes.len() < HEADER_BYTES {
        return Err(WadError::TruncatedHeader {
            expected_bytes: HEADER_BYTES,
            actual_bytes: bytes.len(),
        });
    }

    let signature = [bytes[0], bytes[1], bytes[2], bytes[3]];
    let kind = match &signature {
        b"IWAD" => WadKind::Iwad,
        b"PWAD" => WadKind::Pwad,
        _ => return Err(WadError::UnknownSignature { signature }),
    };
    let lump_count = read_u32(bytes, 4);
    let directory_offset = read_u32(bytes, 8);
    if lump_count > limits.max_lumps {
        return Err(WadError::LumpCountLimitExceeded {
            limit_lumps: limits.max_lumps,
            actual_lumps: lump_count,
        });
    }

    let directory_bytes = u64::from(lump_count) * DIRECTORY_ENTRY_BYTES as u64;
    checked_range(directory_offset, directory_bytes, bytes.len()).ok_or(
        WadError::DirectoryOutOfBounds {
            offset: directory_offset,
            size: directory_bytes,
            input_bytes: bytes.len(),
        },
    )?;

    let mut total_lump_bytes = 0_u64;
    let mut lumps = Vec::with_capacity(lump_count as usize);
    for index in 0..lump_count {
        let entry_offset = directory_offset as usize + index as usize * DIRECTORY_ENTRY_BYTES;
        let offset = read_u32(bytes, entry_offset);
        let size = read_u32(bytes, entry_offset + 4);
        if size > limits.max_lump_bytes {
            return Err(WadError::LumpSizeLimitExceeded {
                index,
                limit_bytes: limits.max_lump_bytes,
                actual_bytes: size,
            });
        }
        if checked_range(offset, u64::from(size), bytes.len()).is_none() {
            return Err(WadError::LumpOutOfBounds {
                index,
                offset,
                size,
                input_bytes: bytes.len(),
            });
        }
        total_lump_bytes += u64::from(size);
        if total_lump_bytes > limits.max_total_lump_bytes {
            return Err(WadError::TotalLumpBytesLimitExceeded {
                limit_bytes: limits.max_total_lump_bytes,
                actual_bytes: total_lump_bytes,
            });
        }
        let name_bytes = bytes[entry_offset + 8..entry_offset + 16]
            .try_into()
            .expect("directory range was validated before entry decoding");
        lumps.push(WadLumpObservation {
            index,
            offset,
            size,
            name: decode_name(index, name_bytes)?,
        });
    }
    reject_overlaps(&lumps)?;
    let namespaces = project_namespaces(&lumps)?;

    let mut hasher = Hasher::new();
    hasher.update(bytes);
    Ok(WadManifest {
        source: WadSourceIdentity {
            label: source_label.into(),
            byte_len: bytes.len(),
            blake3: hasher.finalize().to_hex().to_string(),
        },
        kind,
        directory_offset,
        directory_bytes,
        total_lump_bytes,
        lumps,
        namespaces,
    })
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("fixed field"))
}

fn checked_range(offset: u32, size: u64, input_len: usize) -> Option<()> {
    let end = u64::from(offset).checked_add(size)?;
    (end <= input_len as u64).then_some(())
}

fn decode_name(index: u32, bytes: [u8; 8]) -> Result<String, WadError> {
    let padding_start = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    if padding_start == 0
        || bytes[padding_start..].iter().any(|byte| *byte != 0)
        || bytes[..padding_start]
            .iter()
            .any(|byte| !byte.is_ascii_graphic())
    {
        return Err(WadError::MalformedLumpName { index, bytes });
    }
    Ok(String::from_utf8_lossy(&bytes[..padding_start]).into_owned())
}

fn reject_overlaps(lumps: &[WadLumpObservation]) -> Result<(), WadError> {
    let mut ranges: Vec<_> = lumps
        .iter()
        .filter(|lump| lump.size != 0)
        .map(|lump| {
            (
                u64::from(lump.offset),
                u64::from(lump.offset) + u64::from(lump.size),
                lump.index,
            )
        })
        .collect();
    ranges.sort_unstable_by_key(|(start, end, index)| (*start, *end, *index));
    for pair in ranges.windows(2) {
        let (first_start, first_end, first_index) = pair[0];
        let (second_start, _, second_index) = pair[1];
        debug_assert!(first_start <= second_start);
        if second_start < first_end {
            return Err(WadError::OverlappingLumpRanges {
                first_index,
                second_index,
            });
        }
    }
    Ok(())
}

fn project_namespaces(
    lumps: &[WadLumpObservation],
) -> Result<Vec<WadNamespaceObservation>, WadError> {
    let mut open = [None; 3];
    let mut namespaces = Vec::new();
    for lump in lumps {
        let Some((kind, marker)) = namespace_marker(&lump.name) else {
            continue;
        };
        let slot = kind.slot();
        match marker {
            NamespaceMarker::Start => {
                if let Some((open_kind, first_index)) = open_namespace(&open) {
                    return Err(WadError::OverlappingNamespaceMarker {
                        open_kind,
                        marker_kind: kind,
                        first_index,
                        second_index: lump.index,
                    });
                }
                open[slot] = Some(lump.index);
            }
            NamespaceMarker::End => {
                let Some(start_marker_index) = open[slot].take() else {
                    if let Some((open_kind, first_index)) = open_namespace(&open) {
                        return Err(WadError::MismatchedNamespaceEnd {
                            open_kind,
                            end_kind: kind,
                            first_index,
                            second_index: lump.index,
                        });
                    }
                    return Err(WadError::UnmatchedNamespaceEnd {
                        kind,
                        index: lump.index,
                    });
                };
                namespaces.push(WadNamespaceObservation {
                    kind,
                    start_marker_index,
                    end_marker_index: lump.index,
                    lump_indices: ((start_marker_index + 1)..lump.index)
                        .filter(|index| namespace_marker(&lumps[*index as usize].name).is_none())
                        .collect(),
                });
            }
        }
    }
    for kind in WadNamespaceKind::ALL {
        if let Some(index) = open[kind.slot()] {
            return Err(WadError::UnclosedNamespaceStart { kind, index });
        }
    }
    Ok(namespaces)
}

fn open_namespace(open: &[Option<u32>; 3]) -> Option<(WadNamespaceKind, u32)> {
    WadNamespaceKind::ALL
        .into_iter()
        .find_map(|kind| open[kind.slot()].map(|index| (kind, index)))
}

#[derive(Clone, Copy)]
enum NamespaceMarker {
    Start,
    End,
}

fn namespace_marker(name: &str) -> Option<(WadNamespaceKind, NamespaceMarker)> {
    match name {
        "F_START" | "FF_START" => Some((WadNamespaceKind::Flats, NamespaceMarker::Start)),
        "F_END" | "FF_END" => Some((WadNamespaceKind::Flats, NamespaceMarker::End)),
        "P_START" | "PP_START" => Some((WadNamespaceKind::Patches, NamespaceMarker::Start)),
        "P_END" | "PP_END" => Some((WadNamespaceKind::Patches, NamespaceMarker::End)),
        "S_START" | "SS_START" => Some((WadNamespaceKind::Sprites, NamespaceMarker::Start)),
        "S_END" | "SS_END" => Some((WadNamespaceKind::Sprites, NamespaceMarker::End)),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
