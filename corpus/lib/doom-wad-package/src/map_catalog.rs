//! Doom episode-map cataloguing over a bounded WAD directory observation.
//!
//! This is intentionally not map-record decoding. It identifies one reviewed
//! `E#M#` block and validates the required classic Doom lump names before a
//! later provider interprets the bytes as things, linedefs, sectors, or nodes.

use doom_wad_provider::{WadLumpObservation, WadManifest};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequiredDoomMapLump {
    Things,
    Linedefs,
    Sidedefs,
    Vertexes,
    Segs,
    Subsectors,
    Nodes,
    Sectors,
    Reject,
    Blockmap,
}

impl RequiredDoomMapLump {
    const ALL: [Self; 10] = [
        Self::Things,
        Self::Linedefs,
        Self::Sidedefs,
        Self::Vertexes,
        Self::Segs,
        Self::Subsectors,
        Self::Nodes,
        Self::Sectors,
        Self::Reject,
        Self::Blockmap,
    ];

    const fn name(self) -> &'static str {
        match self {
            Self::Things => "THINGS",
            Self::Linedefs => "LINEDEFS",
            Self::Sidedefs => "SIDEDEFS",
            Self::Vertexes => "VERTEXES",
            Self::Segs => "SEGS",
            Self::Subsectors => "SSECTORS",
            Self::Nodes => "NODES",
            Self::Sectors => "SECTORS",
            Self::Reject => "REJECT",
            Self::Blockmap => "BLOCKMAP",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomMapLumpObservation {
    pub kind: RequiredDoomMapLump,
    pub source: WadLumpObservation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomMapSelection {
    pub map_name: String,
    pub marker: WadLumpObservation,
    /// First directory index after the map marker and final index before the
    /// following `E#M#` marker, or the end of the directory.
    pub local_range: std::ops::Range<u32>,
    pub required_lumps: Vec<DoomMapLumpObservation>,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum DoomMapSelectionError {
    #[error("`{map_name}` is not an admitted Doom episode map name")]
    InvalidMapName { map_name: String },
    #[error("Doom map marker `{map_name}` was not found")]
    MissingMapMarker { map_name: String },
    #[error("Doom map marker `{map_name}` appears at both lumps {first_index} and {second_index}")]
    DuplicateMapMarker {
        map_name: String,
        first_index: u32,
        second_index: u32,
    },
    #[error("Doom map `{map_name}` is missing required lump {required:?}")]
    MissingRequiredLump {
        map_name: String,
        required: RequiredDoomMapLump,
    },
    #[error("Doom map `{map_name}` has duplicate required lump {required:?} at {first_index} and {second_index}")]
    DuplicateRequiredLump {
        map_name: String,
        required: RequiredDoomMapLump,
        first_index: u32,
        second_index: u32,
    },
    #[error("Doom map `{map_name}` puts {following:?} at lump {following_index} before {preceding:?} at lump {preceding_index}")]
    ReorderedRequiredLumps {
        map_name: String,
        preceding: RequiredDoomMapLump,
        preceding_index: u32,
        following: RequiredDoomMapLump,
        following_index: u32,
    },
}

/// Selects a classic Doom episode map from the manifest's source order.
///
/// Map-local membership is bounded by the next recognized `E#M#` marker or the
/// directory end. Required classic map records must each occur exactly once and
/// in their admitted order. This validates WAD-level map selection only; it
/// does not decode a record payload.
pub fn select_doom_episode_map(
    manifest: &WadManifest,
    map_name: &str,
) -> Result<DoomMapSelection, DoomMapSelectionError> {
    if !is_episode_map_marker(map_name) {
        return Err(DoomMapSelectionError::InvalidMapName {
            map_name: map_name.to_owned(),
        });
    }

    let mut markers = manifest
        .lumps
        .iter()
        .filter(|lump| lump.name == map_name)
        .filter(|lump| is_episode_map_marker(&lump.name));
    let marker =
        markers
            .next()
            .cloned()
            .ok_or_else(|| DoomMapSelectionError::MissingMapMarker {
                map_name: map_name.to_owned(),
            })?;
    if let Some(duplicate) = markers.next() {
        return Err(DoomMapSelectionError::DuplicateMapMarker {
            map_name: map_name.to_owned(),
            first_index: marker.index,
            second_index: duplicate.index,
        });
    }

    let local_end = manifest
        .lumps
        .iter()
        .skip(marker.index as usize + 1)
        .find(|lump| is_episode_map_marker(&lump.name))
        .map_or(manifest.lumps.len() as u32, |lump| lump.index);
    let local_range = (marker.index + 1)..local_end;
    let local_lumps = &manifest.lumps[local_range.start as usize..local_range.end as usize];
    let mut required_lumps = Vec::with_capacity(RequiredDoomMapLump::ALL.len());
    let mut previous = None;
    for required in RequiredDoomMapLump::ALL {
        let mut matches = local_lumps
            .iter()
            .filter(|lump| lump.name == required.name());
        let source =
            matches
                .next()
                .cloned()
                .ok_or_else(|| DoomMapSelectionError::MissingRequiredLump {
                    map_name: map_name.to_owned(),
                    required,
                })?;
        if let Some(duplicate) = matches.next() {
            return Err(DoomMapSelectionError::DuplicateRequiredLump {
                map_name: map_name.to_owned(),
                required,
                first_index: source.index,
                second_index: duplicate.index,
            });
        }
        if let Some((preceding, preceding_index)) = previous {
            if source.index < preceding_index {
                return Err(DoomMapSelectionError::ReorderedRequiredLumps {
                    map_name: map_name.to_owned(),
                    preceding,
                    preceding_index,
                    following: required,
                    following_index: source.index,
                });
            }
        }
        previous = Some((required, source.index));
        required_lumps.push(DoomMapLumpObservation {
            kind: required,
            source,
        });
    }

    Ok(DoomMapSelection {
        map_name: map_name.to_owned(),
        marker,
        local_range,
        required_lumps,
    })
}

fn is_episode_map_marker(name: &str) -> bool {
    let bytes = name.as_bytes();
    matches!(
        bytes,
        [b'E', episode @ b'1'..=b'9', b'M', map @ b'1'..=b'9']
            if *episode != 0 && *map != 0
    )
}
