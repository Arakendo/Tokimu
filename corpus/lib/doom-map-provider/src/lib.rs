//! Bounded, source-indexed decoding for the fixed-record core of a classic
//! Doom episode map.
//!
//! This crate does not lower geometry or mutate runtime state. It reports map
//! records and validates the cross-table references needed before a later
//! presentation or simulation layer can interpret them.

use std::collections::{BTreeMap, BTreeSet};

use doom_wad_package::{DoomMapLumpObservation, DoomMapSelection, RequiredDoomMapLump};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomMapDecodeLimits {
    pub max_things: usize,
    pub max_vertices: usize,
    pub max_linedefs: usize,
    pub max_sidedefs: usize,
    pub max_sectors: usize,
    pub max_segs: usize,
    pub max_subsectors: usize,
    pub max_nodes: usize,
    pub max_reject_bytes: usize,
    pub max_blockmap_bytes: usize,
    pub max_blockmap_cells: usize,
    pub max_blockmap_linedef_refs: usize,
    pub max_total_record_bytes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomSourceRecord {
    pub lump_index: u32,
    pub record_index: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomThing {
    pub source: DoomSourceRecord,
    pub x: i16,
    pub y: i16,
    pub angle: u16,
    pub kind: u16,
    pub flags: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomVertex {
    pub source: DoomSourceRecord,
    pub x: i16,
    pub y: i16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomLinedef {
    pub source: DoomSourceRecord,
    pub start_vertex: u16,
    pub end_vertex: u16,
    pub flags: u16,
    pub special: u16,
    pub tag: u16,
    pub right_sidedef: Option<u16>,
    pub left_sidedef: Option<u16>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomSidedef {
    pub source: DoomSourceRecord,
    pub x_offset: i16,
    pub y_offset: i16,
    pub upper_texture: String,
    pub lower_texture: String,
    pub middle_texture: String,
    pub sector: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomSector {
    pub source: DoomSourceRecord,
    pub floor_height: i16,
    pub ceiling_height: i16,
    pub floor_texture: String,
    pub ceiling_texture: String,
    pub light_level: i16,
    pub special: u16,
    pub tag: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomSeg {
    pub source: DoomSourceRecord,
    pub start_vertex: u16,
    pub end_vertex: u16,
    pub angle: u16,
    pub linedef: u16,
    pub direction: u16,
    pub offset: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomSubsector {
    pub source: DoomSourceRecord,
    pub seg_count: u16,
    pub first_seg: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoomBspChild {
    Node(u16),
    Subsector(u16),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomNode {
    pub source: DoomSourceRecord,
    pub x: i16,
    pub y: i16,
    pub delta_x: i16,
    pub delta_y: i16,
    pub right_bbox: [i16; 4],
    pub left_bbox: [i16; 4],
    pub right_child: DoomBspChild,
    pub left_child: DoomBspChild,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomRejectObservation {
    pub lump_index: u32,
    pub byte_len: usize,
    pub required_min_bytes: usize,
}

/// Source-faithful Doom `REJECT` information.
///
/// Classic Doom uses this bit matrix as a monster-sight prefilter: a set bit
/// says a monster in the row sector cannot sight a player in the column
/// sector. It is deliberately not named or exposed as rendering visibility.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomRejectMatrix {
    pub observation: DoomRejectObservation,
    sector_count: usize,
    bits: Vec<u8>,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum DoomRejectLookupError {
    #[error("REJECT {role} sector {sector} is outside its {sector_count}-sector matrix")]
    SectorOutOfBounds {
        role: &'static str,
        sector: usize,
        sector_count: usize,
    },
}

impl DoomRejectMatrix {
    /// Returns the original Doom REJECT meaning for one monster/player sector
    /// pair. The matrix is row-major and least-significant-bit first.
    pub fn forbids_monster_sight(
        &self,
        monster_sector: usize,
        player_sector: usize,
    ) -> Result<bool, DoomRejectLookupError> {
        if monster_sector >= self.sector_count {
            return Err(DoomRejectLookupError::SectorOutOfBounds {
                role: "monster",
                sector: monster_sector,
                sector_count: self.sector_count,
            });
        }
        if player_sector >= self.sector_count {
            return Err(DoomRejectLookupError::SectorOutOfBounds {
                role: "player",
                sector: player_sector,
                sector_count: self.sector_count,
            });
        }
        let bit_index = monster_sector * self.sector_count + player_sector;
        let byte = self.bits[bit_index / 8];
        Ok(byte & (1 << (bit_index % 8)) != 0)
    }

    pub fn sector_count(&self) -> usize {
        self.sector_count
    }
}

impl Default for DoomRejectMatrix {
    fn default() -> Self {
        Self {
            observation: DoomRejectObservation {
                lump_index: 0,
                byte_len: 0,
                required_min_bytes: 0,
            },
            sector_count: 0,
            bits: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomBlockmapObservation {
    pub lump_index: u32,
    pub origin_x: i16,
    pub origin_y: i16,
    pub columns: u16,
    pub rows: u16,
    pub cells: usize,
    pub unique_linedef_lists: usize,
    pub linedef_references: usize,
    /// Row-major cells retaining their source `LINEDEFS` candidates. This is
    /// broad-phase evidence only; it does not assert collision behavior.
    pub cell_linedefs: Vec<DoomBlockmapCell>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomBlockmapCell {
    pub cell_index: usize,
    pub column: u16,
    pub row: u16,
    pub linedefs: Vec<u16>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoomMapCore {
    pub map_name: String,
    pub things: Vec<DoomThing>,
    pub vertices: Vec<DoomVertex>,
    pub linedefs: Vec<DoomLinedef>,
    pub sidedefs: Vec<DoomSidedef>,
    pub sectors: Vec<DoomSector>,
    pub segs: Vec<DoomSeg>,
    pub subsectors: Vec<DoomSubsector>,
    pub nodes: Vec<DoomNode>,
    pub reject: DoomRejectMatrix,
    pub blockmap: DoomBlockmapObservation,
}

/// The source-traceable player-one start selected from a classic Doom `THINGS`
/// table. This is an import observation, not runtime-owned player state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomPlayerOneStart {
    pub source: DoomSourceRecord,
    pub position: [i16; 2],
    pub angle: u16,
    pub flags: u16,
}

/// Finds the containing source BLOCKMAP cell, if a point lies inside its
/// bounded row-major grid. This is an import observation, not a collision
/// query or a movement policy.
pub fn locate_doom_blockmap_cell(
    blockmap: &DoomBlockmapObservation,
    point: [i16; 2],
) -> Option<&DoomBlockmapCell> {
    const CLASSIC_BLOCKMAP_CELL_SPAN: i32 = 128;
    let column =
        (i32::from(point[0]) - i32::from(blockmap.origin_x)).div_euclid(CLASSIC_BLOCKMAP_CELL_SPAN);
    let row =
        (i32::from(point[1]) - i32::from(blockmap.origin_y)).div_euclid(CLASSIC_BLOCKMAP_CELL_SPAN);
    if column < 0
        || row < 0
        || column >= i32::from(blockmap.columns)
        || row >= i32::from(blockmap.rows)
    {
        return None;
    }
    let index = usize::try_from(row).ok()? * usize::from(blockmap.columns)
        + usize::try_from(column).ok()?;
    blockmap.cell_linedefs.get(index)
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum DoomPlayerStartError {
    #[error("map contains no classic player-one start (THING type 1)")]
    MissingPlayerOneStart,
    #[error(
        "map contains multiple classic player-one starts (THING type 1): records {record_indices:?}"
    )]
    DuplicatePlayerOneStarts { record_indices: Vec<u32> },
}

/// Resolves the one classic Doom player-one start (`THING` type `1`).
///
/// Cooperative/deathmatch starts and the later runtime spawn policy remain
/// outside this narrow source observation.
pub fn resolve_doom_player_one_start(
    things: &[DoomThing],
) -> Result<DoomPlayerOneStart, DoomPlayerStartError> {
    let starts = things
        .iter()
        .filter(|thing| thing.kind == 1)
        .collect::<Vec<_>>();
    match starts.as_slice() {
        [] => Err(DoomPlayerStartError::MissingPlayerOneStart),
        [start] => Ok(DoomPlayerOneStart {
            source: start.source,
            position: [start.x, start.y],
            angle: start.angle,
            flags: start.flags,
        }),
        starts => Err(DoomPlayerStartError::DuplicatePlayerOneStarts {
            record_indices: starts
                .iter()
                .map(|start| start.source.record_index)
                .collect(),
        }),
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum DoomMapDecodeError {
    #[error("selected map is missing required observation for {required:?}")]
    MissingSelectedLump { required: RequiredDoomMapLump },
    #[error(
        "{table} lump range offset={offset}, bytes={size} is outside a {wad_bytes}-byte WAD input"
    )]
    LumpOutOfBounds {
        table: &'static str,
        offset: u32,
        size: u32,
        wad_bytes: usize,
    },
    #[error(
        "{table} has {actual_bytes} bytes, not a multiple of its {record_bytes}-byte record size"
    )]
    PartialRecord {
        table: &'static str,
        actual_bytes: usize,
        record_bytes: usize,
    },
    #[error(
        "{table} declares {actual_records} records, exceeding the {limit_records}-record limit"
    )]
    RecordCountLimitExceeded {
        table: &'static str,
        actual_records: usize,
        limit_records: usize,
    },
    #[error(
        "fixed map records total {actual_bytes} bytes, exceeding the {limit_bytes}-byte limit"
    )]
    TotalRecordBytesLimitExceeded {
        actual_bytes: usize,
        limit_bytes: usize,
    },
    #[error("{table} record {record_index} contains malformed {field} name bytes {bytes:?}")]
    MalformedName {
        table: &'static str,
        record_index: u32,
        field: &'static str,
        bytes: [u8; 8],
    },
    #[error("linedef record {record_index} references {field} index {index}, but {available} entries exist")]
    LinedefReferenceOutOfBounds {
        record_index: u32,
        field: &'static str,
        index: u16,
        available: usize,
    },
    #[error("sidedef record {record_index} references sector index {index}, but {available} entries exist")]
    SidedefSectorOutOfBounds {
        record_index: u32,
        index: u16,
        available: usize,
    },
    #[error("seg record {record_index} has unsupported direction value {direction}")]
    SegDirectionInvalid { record_index: u32, direction: u16 },
    #[error("{table} record {record_index} references {field} index {index}, but {available} entries exist")]
    SpatialReferenceOutOfBounds {
        table: &'static str,
        record_index: u32,
        field: &'static str,
        index: u16,
        available: usize,
    },
    #[error("subsector record {record_index} range first_seg={first_seg}, seg_count={seg_count} exceeds {available} segs")]
    SubsectorSegRangeOutOfBounds {
        record_index: u32,
        first_seg: u16,
        seg_count: u16,
        available: usize,
    },
    #[error(
        "REJECT has {actual_bytes} bytes, but {required_bytes} are required for {sectors} sectors"
    )]
    RejectTooShort {
        actual_bytes: usize,
        required_bytes: usize,
        sectors: usize,
    },
    #[error("{table} has {actual_bytes} bytes, exceeding the {limit_bytes}-byte limit")]
    AuxiliaryLumpLimitExceeded {
        table: &'static str,
        actual_bytes: usize,
        limit_bytes: usize,
    },
    #[error("BLOCKMAP is {actual_bytes} bytes, smaller than its 8-byte header")]
    BlockmapHeaderTooShort { actual_bytes: usize },
    #[error("BLOCKMAP declares {columns} by {rows} cells, exceeding the {limit_cells}-cell limit")]
    BlockmapCellLimitExceeded {
        columns: u16,
        rows: u16,
        limit_cells: usize,
    },
    #[error("BLOCKMAP has {actual_bytes} bytes but needs {required_bytes} for its offset table")]
    BlockmapOffsetTableTruncated {
        actual_bytes: usize,
        required_bytes: usize,
    },
    #[error("BLOCKMAP cell {cell_index} points to word offset {word_offset}, outside its {byte_len}-byte lump")]
    BlockmapListOffsetOutOfBounds {
        cell_index: usize,
        word_offset: u16,
        byte_len: usize,
    },
    #[error("BLOCKMAP cell {cell_index} points to word offset {word_offset} inside its header or offset table")]
    BlockmapListOffsetInsideTable { cell_index: usize, word_offset: u16 },
    #[error("BLOCKMAP list at word offset {word_offset} lacks the required leading zero")]
    BlockmapListMissingLeadingZero { word_offset: u16 },
    #[error("BLOCKMAP list at word offset {word_offset} has no 0xffff terminator")]
    BlockmapListUnterminated { word_offset: u16 },
    #[error("BLOCKMAP list at word offset {word_offset} references linedef {linedef}, but {available} exist")]
    BlockmapLinedefOutOfBounds {
        word_offset: u16,
        linedef: u16,
        available: usize,
    },
    #[error("BLOCKMAP has more than its {limit_refs}-reference limit")]
    BlockmapReferenceLimitExceeded { limit_refs: usize },
}

/// Decodes the fixed-record map core and validates its immediate references.
pub fn decode_doom_map_core(
    wad_bytes: &[u8],
    selection: &DoomMapSelection,
    limits: DoomMapDecodeLimits,
) -> Result<DoomMapCore, DoomMapDecodeError> {
    let things_lump = selected_lump(selection, RequiredDoomMapLump::Things)?;
    let linedefs_lump = selected_lump(selection, RequiredDoomMapLump::Linedefs)?;
    let sidedefs_lump = selected_lump(selection, RequiredDoomMapLump::Sidedefs)?;
    let vertices_lump = selected_lump(selection, RequiredDoomMapLump::Vertexes)?;
    let sectors_lump = selected_lump(selection, RequiredDoomMapLump::Sectors)?;
    let segs_lump = selected_lump(selection, RequiredDoomMapLump::Segs)?;
    let subsectors_lump = selected_lump(selection, RequiredDoomMapLump::Subsectors)?;
    let nodes_lump = selected_lump(selection, RequiredDoomMapLump::Nodes)?;
    let reject_lump = selected_lump(selection, RequiredDoomMapLump::Reject)?;
    let blockmap_lump = selected_lump(selection, RequiredDoomMapLump::Blockmap)?;
    let things_bytes = lump_bytes(wad_bytes, things_lump, "THINGS")?;
    let linedefs_bytes = lump_bytes(wad_bytes, linedefs_lump, "LINEDEFS")?;
    let sidedefs_bytes = lump_bytes(wad_bytes, sidedefs_lump, "SIDEDEFS")?;
    let vertices_bytes = lump_bytes(wad_bytes, vertices_lump, "VERTEXES")?;
    let sectors_bytes = lump_bytes(wad_bytes, sectors_lump, "SECTORS")?;
    let segs_bytes = lump_bytes(wad_bytes, segs_lump, "SEGS")?;
    let subsectors_bytes = lump_bytes(wad_bytes, subsectors_lump, "SSECTORS")?;
    let nodes_bytes = lump_bytes(wad_bytes, nodes_lump, "NODES")?;
    let reject_bytes = lump_bytes(wad_bytes, reject_lump, "REJECT")?;
    let blockmap_bytes = lump_bytes(wad_bytes, blockmap_lump, "BLOCKMAP")?;
    let total_bytes = things_bytes.len()
        + linedefs_bytes.len()
        + sidedefs_bytes.len()
        + vertices_bytes.len()
        + sectors_bytes.len()
        + segs_bytes.len()
        + subsectors_bytes.len()
        + nodes_bytes.len()
        + reject_bytes.len()
        + blockmap_bytes.len();
    if total_bytes > limits.max_total_record_bytes {
        return Err(DoomMapDecodeError::TotalRecordBytesLimitExceeded {
            actual_bytes: total_bytes,
            limit_bytes: limits.max_total_record_bytes,
        });
    }

    let things = decode_things(things_bytes, things_lump.source.index, limits.max_things)?;
    let vertices = decode_vertices(
        vertices_bytes,
        vertices_lump.source.index,
        limits.max_vertices,
    )?;
    let linedefs = decode_linedefs(
        linedefs_bytes,
        linedefs_lump.source.index,
        limits.max_linedefs,
    )?;
    let sidedefs = decode_sidedefs(
        sidedefs_bytes,
        sidedefs_lump.source.index,
        limits.max_sidedefs,
    )?;
    let sectors = decode_sectors(sectors_bytes, sectors_lump.source.index, limits.max_sectors)?;
    let segs = decode_segs(segs_bytes, segs_lump.source.index, limits.max_segs)?;
    let subsectors = decode_subsectors(
        subsectors_bytes,
        subsectors_lump.source.index,
        limits.max_subsectors,
    )?;
    let nodes = decode_nodes(nodes_bytes, nodes_lump.source.index, limits.max_nodes)?;
    validate_references(&linedefs, &vertices, &sidedefs, &sectors)?;
    validate_spatial_references(&segs, &subsectors, &nodes, &vertices, &linedefs)?;
    let reject = validate_reject(
        reject_bytes,
        reject_lump.source.index,
        sectors.len(),
        limits,
    )?;
    let blockmap = validate_blockmap(
        blockmap_bytes,
        blockmap_lump.source.index,
        linedefs.len(),
        limits,
    )?;

    Ok(DoomMapCore {
        map_name: selection.map_name.clone(),
        things,
        vertices,
        linedefs,
        sidedefs,
        sectors,
        segs,
        subsectors,
        nodes,
        reject,
        blockmap,
    })
}

fn selected_lump(
    selection: &DoomMapSelection,
    required: RequiredDoomMapLump,
) -> Result<&DoomMapLumpObservation, DoomMapDecodeError> {
    selection
        .required_lumps
        .iter()
        .find(|lump| lump.kind == required)
        .ok_or(DoomMapDecodeError::MissingSelectedLump { required })
}

fn lump_bytes<'a>(
    wad_bytes: &'a [u8],
    lump: &DoomMapLumpObservation,
    table: &'static str,
) -> Result<&'a [u8], DoomMapDecodeError> {
    let start = lump.source.offset as usize;
    let end = start.checked_add(lump.source.size as usize).ok_or(
        DoomMapDecodeError::LumpOutOfBounds {
            table,
            offset: lump.source.offset,
            size: lump.source.size,
            wad_bytes: wad_bytes.len(),
        },
    )?;
    wad_bytes
        .get(start..end)
        .ok_or(DoomMapDecodeError::LumpOutOfBounds {
            table,
            offset: lump.source.offset,
            size: lump.source.size,
            wad_bytes: wad_bytes.len(),
        })
}

fn record_count(
    bytes: &[u8],
    table: &'static str,
    record_bytes: usize,
    limit: usize,
) -> Result<usize, DoomMapDecodeError> {
    if !bytes.len().is_multiple_of(record_bytes) {
        return Err(DoomMapDecodeError::PartialRecord {
            table,
            actual_bytes: bytes.len(),
            record_bytes,
        });
    }
    let count = bytes.len() / record_bytes;
    if count > limit {
        return Err(DoomMapDecodeError::RecordCountLimitExceeded {
            table,
            actual_records: count,
            limit_records: limit,
        });
    }
    Ok(count)
}

fn decode_things(
    bytes: &[u8],
    lump_index: u32,
    limit: usize,
) -> Result<Vec<DoomThing>, DoomMapDecodeError> {
    let count = record_count(bytes, "THINGS", 10, limit)?;
    Ok((0..count)
        .map(|record_index| {
            let bytes = &bytes[record_index * 10..][..10];
            DoomThing {
                source: DoomSourceRecord {
                    lump_index,
                    record_index: record_index as u32,
                },
                x: read_i16(bytes, 0),
                y: read_i16(bytes, 2),
                angle: read_u16(bytes, 4),
                kind: read_u16(bytes, 6),
                flags: read_u16(bytes, 8),
            }
        })
        .collect())
}

fn decode_vertices(
    bytes: &[u8],
    lump_index: u32,
    limit: usize,
) -> Result<Vec<DoomVertex>, DoomMapDecodeError> {
    let count = record_count(bytes, "VERTEXES", 4, limit)?;
    Ok((0..count)
        .map(|record_index| {
            let bytes = &bytes[record_index * 4..][..4];
            DoomVertex {
                source: DoomSourceRecord {
                    lump_index,
                    record_index: record_index as u32,
                },
                x: read_i16(bytes, 0),
                y: read_i16(bytes, 2),
            }
        })
        .collect())
}

fn decode_linedefs(
    bytes: &[u8],
    lump_index: u32,
    limit: usize,
) -> Result<Vec<DoomLinedef>, DoomMapDecodeError> {
    let count = record_count(bytes, "LINEDEFS", 14, limit)?;
    Ok((0..count)
        .map(|record_index| {
            let bytes = &bytes[record_index * 14..][..14];
            DoomLinedef {
                source: DoomSourceRecord {
                    lump_index,
                    record_index: record_index as u32,
                },
                start_vertex: read_u16(bytes, 0),
                end_vertex: read_u16(bytes, 2),
                flags: read_u16(bytes, 4),
                special: read_u16(bytes, 6),
                tag: read_u16(bytes, 8),
                right_sidedef: optional_index(read_u16(bytes, 10)),
                left_sidedef: optional_index(read_u16(bytes, 12)),
            }
        })
        .collect())
}

fn decode_sidedefs(
    bytes: &[u8],
    lump_index: u32,
    limit: usize,
) -> Result<Vec<DoomSidedef>, DoomMapDecodeError> {
    let count = record_count(bytes, "SIDEDEFS", 30, limit)?;
    (0..count)
        .map(|record_index| {
            let bytes = &bytes[record_index * 30..][..30];
            Ok(DoomSidedef {
                source: DoomSourceRecord {
                    lump_index,
                    record_index: record_index as u32,
                },
                x_offset: read_i16(bytes, 0),
                y_offset: read_i16(bytes, 2),
                upper_texture: decode_name(
                    "SIDEDEFS",
                    record_index as u32,
                    "upper_texture",
                    bytes,
                    4,
                )?,
                lower_texture: decode_name(
                    "SIDEDEFS",
                    record_index as u32,
                    "lower_texture",
                    bytes,
                    12,
                )?,
                middle_texture: decode_name(
                    "SIDEDEFS",
                    record_index as u32,
                    "middle_texture",
                    bytes,
                    20,
                )?,
                sector: read_u16(bytes, 28),
            })
        })
        .collect()
}

fn decode_sectors(
    bytes: &[u8],
    lump_index: u32,
    limit: usize,
) -> Result<Vec<DoomSector>, DoomMapDecodeError> {
    let count = record_count(bytes, "SECTORS", 26, limit)?;
    (0..count)
        .map(|record_index| {
            let bytes = &bytes[record_index * 26..][..26];
            Ok(DoomSector {
                source: DoomSourceRecord {
                    lump_index,
                    record_index: record_index as u32,
                },
                floor_height: read_i16(bytes, 0),
                ceiling_height: read_i16(bytes, 2),
                floor_texture: decode_name(
                    "SECTORS",
                    record_index as u32,
                    "floor_texture",
                    bytes,
                    4,
                )?,
                ceiling_texture: decode_name(
                    "SECTORS",
                    record_index as u32,
                    "ceiling_texture",
                    bytes,
                    12,
                )?,
                light_level: read_i16(bytes, 20),
                special: read_u16(bytes, 22),
                tag: read_u16(bytes, 24),
            })
        })
        .collect()
}

fn decode_segs(
    bytes: &[u8],
    lump_index: u32,
    limit: usize,
) -> Result<Vec<DoomSeg>, DoomMapDecodeError> {
    let count = record_count(bytes, "SEGS", 12, limit)?;
    Ok((0..count)
        .map(|record_index| {
            let bytes = &bytes[record_index * 12..][..12];
            DoomSeg {
                source: DoomSourceRecord {
                    lump_index,
                    record_index: record_index as u32,
                },
                start_vertex: read_u16(bytes, 0),
                end_vertex: read_u16(bytes, 2),
                angle: read_u16(bytes, 4),
                linedef: read_u16(bytes, 6),
                direction: read_u16(bytes, 8),
                offset: read_u16(bytes, 10),
            }
        })
        .collect())
}

fn decode_subsectors(
    bytes: &[u8],
    lump_index: u32,
    limit: usize,
) -> Result<Vec<DoomSubsector>, DoomMapDecodeError> {
    let count = record_count(bytes, "SSECTORS", 4, limit)?;
    Ok((0..count)
        .map(|record_index| {
            let bytes = &bytes[record_index * 4..][..4];
            DoomSubsector {
                source: DoomSourceRecord {
                    lump_index,
                    record_index: record_index as u32,
                },
                seg_count: read_u16(bytes, 0),
                first_seg: read_u16(bytes, 2),
            }
        })
        .collect())
}

fn decode_nodes(
    bytes: &[u8],
    lump_index: u32,
    limit: usize,
) -> Result<Vec<DoomNode>, DoomMapDecodeError> {
    let count = record_count(bytes, "NODES", 28, limit)?;
    Ok((0..count)
        .map(|record_index| {
            let bytes = &bytes[record_index * 28..][..28];
            DoomNode {
                source: DoomSourceRecord {
                    lump_index,
                    record_index: record_index as u32,
                },
                x: read_i16(bytes, 0),
                y: read_i16(bytes, 2),
                delta_x: read_i16(bytes, 4),
                delta_y: read_i16(bytes, 6),
                right_bbox: [
                    read_i16(bytes, 8),
                    read_i16(bytes, 10),
                    read_i16(bytes, 12),
                    read_i16(bytes, 14),
                ],
                left_bbox: [
                    read_i16(bytes, 16),
                    read_i16(bytes, 18),
                    read_i16(bytes, 20),
                    read_i16(bytes, 22),
                ],
                right_child: decode_bsp_child(read_u16(bytes, 24)),
                left_child: decode_bsp_child(read_u16(bytes, 26)),
            }
        })
        .collect())
}

fn validate_references(
    linedefs: &[DoomLinedef],
    vertices: &[DoomVertex],
    sidedefs: &[DoomSidedef],
    sectors: &[DoomSector],
) -> Result<(), DoomMapDecodeError> {
    for line in linedefs {
        for (field, index) in [
            ("start_vertex", Some(line.start_vertex)),
            ("end_vertex", Some(line.end_vertex)),
            ("right_sidedef", line.right_sidedef),
            ("left_sidedef", line.left_sidedef),
        ] {
            let available = if field.ends_with("vertex") {
                vertices.len()
            } else {
                sidedefs.len()
            };
            if let Some(index) = index {
                if index as usize >= available {
                    return Err(DoomMapDecodeError::LinedefReferenceOutOfBounds {
                        record_index: line.source.record_index,
                        field,
                        index,
                        available,
                    });
                }
            }
        }
    }
    for side in sidedefs {
        if side.sector as usize >= sectors.len() {
            return Err(DoomMapDecodeError::SidedefSectorOutOfBounds {
                record_index: side.source.record_index,
                index: side.sector,
                available: sectors.len(),
            });
        }
    }
    Ok(())
}

fn validate_spatial_references(
    segs: &[DoomSeg],
    subsectors: &[DoomSubsector],
    nodes: &[DoomNode],
    vertices: &[DoomVertex],
    linedefs: &[DoomLinedef],
) -> Result<(), DoomMapDecodeError> {
    for seg in segs {
        for (field, index, available) in [
            ("start_vertex", seg.start_vertex, vertices.len()),
            ("end_vertex", seg.end_vertex, vertices.len()),
            ("linedef", seg.linedef, linedefs.len()),
        ] {
            if index as usize >= available {
                return Err(DoomMapDecodeError::SpatialReferenceOutOfBounds {
                    table: "SEGS",
                    record_index: seg.source.record_index,
                    field,
                    index,
                    available,
                });
            }
        }
        if seg.direction > 1 {
            return Err(DoomMapDecodeError::SegDirectionInvalid {
                record_index: seg.source.record_index,
                direction: seg.direction,
            });
        }
    }
    for subsector in subsectors {
        let end = usize::from(subsector.first_seg) + usize::from(subsector.seg_count);
        if end > segs.len() {
            return Err(DoomMapDecodeError::SubsectorSegRangeOutOfBounds {
                record_index: subsector.source.record_index,
                first_seg: subsector.first_seg,
                seg_count: subsector.seg_count,
                available: segs.len(),
            });
        }
    }
    for node in nodes {
        for (field, child) in [
            ("right_child", node.right_child),
            ("left_child", node.left_child),
        ] {
            let (index, available) = match child {
                DoomBspChild::Node(index) => (index, nodes.len()),
                DoomBspChild::Subsector(index) => (index, subsectors.len()),
            };
            if index as usize >= available {
                return Err(DoomMapDecodeError::SpatialReferenceOutOfBounds {
                    table: "NODES",
                    record_index: node.source.record_index,
                    field,
                    index,
                    available,
                });
            }
        }
    }
    Ok(())
}

fn validate_reject(
    bytes: &[u8],
    lump_index: u32,
    sectors: usize,
    limits: DoomMapDecodeLimits,
) -> Result<DoomRejectMatrix, DoomMapDecodeError> {
    if bytes.len() > limits.max_reject_bytes {
        return Err(DoomMapDecodeError::AuxiliaryLumpLimitExceeded {
            table: "REJECT",
            actual_bytes: bytes.len(),
            limit_bytes: limits.max_reject_bytes,
        });
    }
    let required_min_bytes = sectors
        .checked_mul(sectors)
        .and_then(|bits| bits.checked_add(7))
        .map(|bits| bits / 8)
        .expect("sector count is bounded by an addressable input allocation");
    if bytes.len() < required_min_bytes {
        return Err(DoomMapDecodeError::RejectTooShort {
            actual_bytes: bytes.len(),
            required_bytes: required_min_bytes,
            sectors,
        });
    }
    Ok(DoomRejectMatrix {
        observation: DoomRejectObservation {
            lump_index,
            byte_len: bytes.len(),
            required_min_bytes,
        },
        sector_count: sectors,
        bits: bytes.to_vec(),
    })
}

fn validate_blockmap(
    bytes: &[u8],
    lump_index: u32,
    linedefs: usize,
    limits: DoomMapDecodeLimits,
) -> Result<DoomBlockmapObservation, DoomMapDecodeError> {
    if bytes.len() > limits.max_blockmap_bytes {
        return Err(DoomMapDecodeError::AuxiliaryLumpLimitExceeded {
            table: "BLOCKMAP",
            actual_bytes: bytes.len(),
            limit_bytes: limits.max_blockmap_bytes,
        });
    }
    if bytes.len() < 8 {
        return Err(DoomMapDecodeError::BlockmapHeaderTooShort {
            actual_bytes: bytes.len(),
        });
    }
    let origin_x = read_i16(bytes, 0);
    let origin_y = read_i16(bytes, 2);
    let columns = read_u16(bytes, 4);
    let rows = read_u16(bytes, 6);
    let cells = usize::from(columns) * usize::from(rows);
    if cells > limits.max_blockmap_cells {
        return Err(DoomMapDecodeError::BlockmapCellLimitExceeded {
            columns,
            rows,
            limit_cells: limits.max_blockmap_cells,
        });
    }
    let offset_table_end = 8 + cells * 2;
    if bytes.len() < offset_table_end {
        return Err(DoomMapDecodeError::BlockmapOffsetTableTruncated {
            actual_bytes: bytes.len(),
            required_bytes: offset_table_end,
        });
    }
    let mut offsets = BTreeSet::new();
    for cell_index in 0..cells {
        let word_offset = read_u16(bytes, 8 + cell_index * 2);
        let byte_offset = usize::from(word_offset) * 2;
        if byte_offset >= bytes.len() {
            return Err(DoomMapDecodeError::BlockmapListOffsetOutOfBounds {
                cell_index,
                word_offset,
                byte_len: bytes.len(),
            });
        }
        if byte_offset < offset_table_end {
            return Err(DoomMapDecodeError::BlockmapListOffsetInsideTable {
                cell_index,
                word_offset,
            });
        }
        offsets.insert(word_offset);
    }
    let mut linedef_references = 0;
    let mut lists = BTreeMap::new();
    for word_offset in &offsets {
        let mut byte_offset = usize::from(*word_offset) * 2;
        if read_u16(bytes, byte_offset) != 0 {
            return Err(DoomMapDecodeError::BlockmapListMissingLeadingZero {
                word_offset: *word_offset,
            });
        }
        byte_offset += 2;
        let mut list = Vec::new();
        loop {
            if byte_offset + 2 > bytes.len() {
                return Err(DoomMapDecodeError::BlockmapListUnterminated {
                    word_offset: *word_offset,
                });
            }
            let linedef = read_u16(bytes, byte_offset);
            byte_offset += 2;
            if linedef == u16::MAX {
                break;
            }
            if linedef as usize >= linedefs {
                return Err(DoomMapDecodeError::BlockmapLinedefOutOfBounds {
                    word_offset: *word_offset,
                    linedef,
                    available: linedefs,
                });
            }
            list.push(linedef);
            linedef_references += 1;
            if linedef_references > limits.max_blockmap_linedef_refs {
                return Err(DoomMapDecodeError::BlockmapReferenceLimitExceeded {
                    limit_refs: limits.max_blockmap_linedef_refs,
                });
            }
        }
        lists.insert(*word_offset, list);
    }
    let cell_linedefs = (0..cells)
        .map(|cell_index| {
            let word_offset = read_u16(bytes, 8 + cell_index * 2);
            DoomBlockmapCell {
                cell_index,
                column: (cell_index % usize::from(columns)) as u16,
                row: (cell_index / usize::from(columns)) as u16,
                linedefs: lists
                    .get(&word_offset)
                    .expect("validated BLOCKMAP offset has a decoded list")
                    .clone(),
            }
        })
        .collect();
    Ok(DoomBlockmapObservation {
        lump_index,
        origin_x,
        origin_y,
        columns,
        rows,
        cells,
        unique_linedef_lists: offsets.len(),
        linedef_references,
        cell_linedefs,
    })
}

fn decode_name(
    table: &'static str,
    record_index: u32,
    field: &'static str,
    record: &[u8],
    offset: usize,
) -> Result<String, DoomMapDecodeError> {
    let bytes: [u8; 8] = record[offset..offset + 8]
        .try_into()
        .expect("fixed-record name field is in range");
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
        return Err(DoomMapDecodeError::MalformedName {
            table,
            record_index,
            field,
            bytes,
        });
    }
    Ok(String::from_utf8_lossy(&bytes[..padding_start]).into_owned())
}

fn read_i16(bytes: &[u8], offset: usize) -> i16 {
    i16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("fixed field"))
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("fixed field"))
}

fn optional_index(value: u16) -> Option<u16> {
    (value != u16::MAX).then_some(value)
}

fn decode_bsp_child(value: u16) -> DoomBspChild {
    if value & 0x8000 != 0 {
        DoomBspChild::Subsector(value & 0x7fff)
    } else {
        DoomBspChild::Node(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doom_wad_package::select_doom_episode_map;
    use doom_wad_provider::{inspect_wad, WadReadLimits};

    fn limits() -> DoomMapDecodeLimits {
        DoomMapDecodeLimits {
            max_things: 16,
            max_vertices: 16,
            max_linedefs: 16,
            max_sidedefs: 16,
            max_sectors: 16,
            max_segs: 16,
            max_subsectors: 16,
            max_nodes: 16,
            max_reject_bytes: 1024,
            max_blockmap_bytes: 1024,
            max_blockmap_cells: 16,
            max_blockmap_linedef_refs: 32,
            max_total_record_bytes: 4096,
        }
    }

    fn name_bytes(name: &str) -> [u8; 8] {
        let mut bytes = [0_u8; 8];
        bytes[..name.len()].copy_from_slice(name.as_bytes());
        bytes
    }

    fn record_sidedef(sector: u16) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0_i16.to_le_bytes());
        bytes.extend_from_slice(&0_i16.to_le_bytes());
        bytes.extend_from_slice(&name_bytes("-"));
        bytes.extend_from_slice(&name_bytes("-"));
        bytes.extend_from_slice(&name_bytes("STARTAN3"));
        bytes.extend_from_slice(&sector.to_le_bytes());
        bytes
    }

    fn record_sector() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0_i16.to_le_bytes());
        bytes.extend_from_slice(&128_i16.to_le_bytes());
        bytes.extend_from_slice(&name_bytes("FLOOR0_1"));
        bytes.extend_from_slice(&name_bytes("CEIL1_1"));
        bytes.extend_from_slice(&160_i16.to_le_bytes());
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        bytes
    }

    fn wad_bytes(start_vertex: u16, thing_bytes: Vec<u8>) -> Vec<u8> {
        let mut linedef = Vec::new();
        linedef.extend_from_slice(&start_vertex.to_le_bytes());
        linedef.extend_from_slice(&1_u16.to_le_bytes());
        linedef.extend_from_slice(&0_u16.to_le_bytes());
        linedef.extend_from_slice(&0_u16.to_le_bytes());
        linedef.extend_from_slice(&0_u16.to_le_bytes());
        linedef.extend_from_slice(&0_u16.to_le_bytes());
        linedef.extend_from_slice(&u16::MAX.to_le_bytes());

        let mut vertices = Vec::new();
        vertices.extend_from_slice(&0_i16.to_le_bytes());
        vertices.extend_from_slice(&0_i16.to_le_bytes());
        vertices.extend_from_slice(&128_i16.to_le_bytes());
        vertices.extend_from_slice(&0_i16.to_le_bytes());
        let mut seg = Vec::new();
        seg.extend_from_slice(&0_u16.to_le_bytes());
        seg.extend_from_slice(&1_u16.to_le_bytes());
        seg.extend_from_slice(&0_u16.to_le_bytes());
        seg.extend_from_slice(&0_u16.to_le_bytes());
        seg.extend_from_slice(&0_u16.to_le_bytes());
        seg.extend_from_slice(&0_u16.to_le_bytes());
        let mut subsector = Vec::new();
        subsector.extend_from_slice(&1_u16.to_le_bytes());
        subsector.extend_from_slice(&0_u16.to_le_bytes());
        let mut node = vec![0_u8; 24];
        node.extend_from_slice(&0x8000_u16.to_le_bytes());
        node.extend_from_slice(&0x8000_u16.to_le_bytes());
        let entries = vec![
            ("E1M1", Vec::new()),
            ("THINGS", thing_bytes),
            ("LINEDEFS", linedef),
            ("SIDEDEFS", record_sidedef(0)),
            ("VERTEXES", vertices),
            ("SEGS", seg),
            ("SSECTORS", subsector),
            ("NODES", node),
            ("SECTORS", record_sector()),
            ("REJECT", vec![0]),
            ("BLOCKMAP", blockmap_bytes()),
        ];
        let directory_offset = 12 + entries.iter().map(|(_, bytes)| bytes.len()).sum::<usize>();
        let mut wad = Vec::new();
        wad.extend_from_slice(b"IWAD");
        wad.extend_from_slice(&(entries.len() as u32).to_le_bytes());
        wad.extend_from_slice(&(directory_offset as u32).to_le_bytes());
        let mut offset = 12_u32;
        for (_, bytes) in &entries {
            wad.extend_from_slice(bytes);
        }
        for (name, bytes) in &entries {
            wad.extend_from_slice(&offset.to_le_bytes());
            wad.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            wad.extend_from_slice(&name_bytes(name));
            offset += bytes.len() as u32;
        }
        wad
    }

    fn blockmap_bytes() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0_i16.to_le_bytes());
        bytes.extend_from_slice(&0_i16.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&5_u16.to_le_bytes());
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        bytes.extend_from_slice(&u16::MAX.to_le_bytes());
        bytes
    }

    fn selected_fixture(start_vertex: u16, thing_bytes: Vec<u8>) -> (Vec<u8>, DoomMapSelection) {
        let wad = wad_bytes(start_vertex, thing_bytes);
        let manifest = inspect_wad(
            "synthetic/map-core.wad",
            &wad,
            WadReadLimits::new(4096, 32, 2048, 4096),
        )
        .expect("synthetic fixture should inspect");
        let selection =
            select_doom_episode_map(&manifest, "E1M1").expect("synthetic map block should select");
        (wad, selection)
    }

    #[test]
    fn decodes_fixed_record_core_with_source_indices() {
        let mut thing = Vec::new();
        thing.extend_from_slice(&10_i16.to_le_bytes());
        thing.extend_from_slice(&20_i16.to_le_bytes());
        thing.extend_from_slice(&90_u16.to_le_bytes());
        thing.extend_from_slice(&1_u16.to_le_bytes());
        thing.extend_from_slice(&7_u16.to_le_bytes());
        let (wad, selection) = selected_fixture(0, thing);

        let core = decode_doom_map_core(&wad, &selection, limits())
            .expect("bounded fixed records should decode");
        assert_eq!(core.map_name, "E1M1");
        assert_eq!(core.things[0].source.record_index, 0);
        assert_eq!(core.things[0].x, 10);
        assert_eq!(core.vertices.len(), 2);
        assert_eq!(core.linedefs[0].left_sidedef, None);
        assert_eq!(core.sidedefs[0].middle_texture, "STARTAN3");
        assert_eq!(core.sectors[0].ceiling_height, 128);
        assert_eq!(core.segs.len(), 1);
        assert_eq!(core.subsectors[0].seg_count, 1);
        assert_eq!(core.nodes[0].right_child, DoomBspChild::Subsector(0));
        assert_eq!(core.reject.observation.required_min_bytes, 1);
        assert!(!core
            .reject
            .forbids_monster_sight(0, 0)
            .expect("one-sector REJECT lookup remains in bounds"));
        assert_eq!(core.blockmap.cells, 1);
        assert_eq!(core.blockmap.cell_linedefs[0].linedefs, vec![0]);
        assert_eq!(
            locate_doom_blockmap_cell(&core.blockmap, [127, 127])
                .expect("point remains inside the one source cell")
                .cell_index,
            0
        );
        assert!(locate_doom_blockmap_cell(&core.blockmap, [128, 0]).is_none());
    }

    #[test]
    fn partial_records_and_invalid_cross_references_are_rejected() {
        let (partial_wad, selection) = selected_fixture(0, vec![0]);
        assert!(matches!(
            decode_doom_map_core(&partial_wad, &selection, limits()),
            Err(DoomMapDecodeError::PartialRecord {
                table: "THINGS",
                ..
            })
        ));

        let (invalid_wad, selection) = selected_fixture(4, vec![0; 10]);
        assert!(matches!(
            decode_doom_map_core(&invalid_wad, &selection, limits()),
            Err(DoomMapDecodeError::LinedefReferenceOutOfBounds {
                field: "start_vertex",
                index: 4,
                available: 2,
                ..
            })
        ));

        let (mut invalid_spatial_wad, selection) = selected_fixture(0, vec![0; 10]);
        let seg_offset = selection
            .required_lumps
            .iter()
            .find(|lump| lump.kind == RequiredDoomMapLump::Segs)
            .expect("synthetic selection includes SEGS")
            .source
            .offset as usize;
        invalid_spatial_wad[seg_offset + 8..seg_offset + 10].copy_from_slice(&2_u16.to_le_bytes());
        assert!(matches!(
            decode_doom_map_core(&invalid_spatial_wad, &selection, limits()),
            Err(DoomMapDecodeError::SegDirectionInvalid {
                record_index: 0,
                direction: 2,
            })
        ));
    }

    #[test]
    fn reject_and_blockmap_failures_remain_structured() {
        assert!(matches!(
            validate_reject(&[], 10, 1, limits()),
            Err(DoomMapDecodeError::RejectTooShort {
                actual_bytes: 0,
                required_bytes: 1,
                sectors: 1,
            })
        ));

        let mut blockmap = blockmap_bytes();
        blockmap[10..12].copy_from_slice(&1_u16.to_le_bytes());
        assert!(matches!(
            validate_blockmap(&blockmap, 11, 1, limits()),
            Err(DoomMapDecodeError::BlockmapListMissingLeadingZero { word_offset: 5 })
        ));
    }

    #[test]
    fn reject_preserves_doom_row_major_lsb_first_monster_sight_meaning() {
        // Three sectors need nine bits. Set bit 3 (monster sector 1, player
        // sector 0) and bit 8 (monster sector 2, player sector 2).
        let reject = validate_reject(&[0b0000_1000, 0b0000_0001], 10, 3, limits())
            .expect("complete synthetic REJECT matrix should validate");

        assert!(!reject
            .forbids_monster_sight(0, 1)
            .expect("matrix lookup remains in bounds"));
        assert!(reject
            .forbids_monster_sight(1, 0)
            .expect("bit three is monster one/player zero"));
        assert!(reject
            .forbids_monster_sight(2, 2)
            .expect("bit eight is monster two/player two"));
        assert!(matches!(
            reject.forbids_monster_sight(3, 0),
            Err(DoomRejectLookupError::SectorOutOfBounds {
                role: "monster",
                sector: 3,
                sector_count: 3,
            })
        ));
    }

    #[test]
    fn player_one_start_requires_exactly_one_source_thing() {
        let thing = |record_index, kind| DoomThing {
            source: DoomSourceRecord {
                lump_index: 7,
                record_index,
            },
            x: 10,
            y: -20,
            angle: 90,
            kind,
            flags: 7,
        };

        assert!(matches!(
            resolve_doom_player_one_start(&[thing(0, 3004)]),
            Err(DoomPlayerStartError::MissingPlayerOneStart)
        ));
        assert!(matches!(
            resolve_doom_player_one_start(&[thing(3, 1), thing(5, 1)]),
            Err(DoomPlayerStartError::DuplicatePlayerOneStarts { record_indices })
                if record_indices == [3, 5]
        ));
        assert_eq!(
            resolve_doom_player_one_start(&[thing(3, 1)]),
            Ok(DoomPlayerOneStart {
                source: DoomSourceRecord {
                    lump_index: 7,
                    record_index: 3,
                },
                position: [10, -20],
                angle: 90,
                flags: 7,
            })
        );
    }
}
