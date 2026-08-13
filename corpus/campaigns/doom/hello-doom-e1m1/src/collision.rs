//! Corpus-local first-walk collision evidence for classic Doom source maps.
//!
//! This owns no renderer, runtime, or global spatial contract. It selects only
//! one-sided and explicitly blocking source lines for the initial E1M1 walk.
//! Two-sided clearance, doors, and lifts remain separate. The first floor
//! transition policy is deliberately explicit: a 24-unit upward step and a
//! 56-unit vertical clearance are corpus-local classic-Doom evidence, not a
//! generic character-controller contract.

use std::collections::BTreeSet;

use doom_geometry_provider::{
    locate_doom_point_subsector, resolve_doom_subsector_bsp_paths,
    resolve_doom_subsector_sector_ownership, DoomGeometryError, DoomSubsectorBspPath,
    DoomSubsectorSectorOwnership,
};
use doom_map_provider::{DoomMapCore, DoomSector, DoomSourceRecord};
use tokimu_core::math::Vec3;

use crate::DoomComparativeEmbedding;

const CLASSIC_BLOCKMAP_SPAN: f32 = 128.0;
const MAX_SUBSTEP_DISTANCE: f32 = 4.0;
const MAX_OVERLAP_PASSES: usize = 3;
const PUSH_EPSILON: f32 = 0.001;
const DOOM_LINE_BLOCKING: u16 = 0x0001;
const CLASSIC_MAX_STEP_UP: i16 = 24;
const CLASSIC_PLAYER_HEIGHT: i16 = 56;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DoomWalkWall {
    pub source_linedef: u32,
    pub start: [f32; 2],
    pub end: [f32; 2],
}

#[derive(Clone, Debug)]
pub struct DoomWalkCollisionWorld {
    walls: Vec<DoomWalkWall>,
    blockmap_origin: [f32; 2],
    blockmap_dimensions: [usize; 2],
    blockmap_cells: Vec<Vec<u32>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DoomWalkMoveObservation {
    pub requested_delta: [f32; 2],
    pub resolved_position: [f32; 2],
    pub contacted_linedefs: Vec<u32>,
    pub broad_phase_candidates: usize,
    pub used_full_wall_fallback: bool,
}

/// Corpus-local source lookup for floor/ceiling transitions. This is separate
/// from horizontal disc contacts: BSP ownership identifies a prospective
/// source sector, then the caller decides whether the vertical transition is
/// eligible.
#[derive(Clone, Debug)]
pub struct DoomWalkFloorWorld {
    paths: Vec<DoomSubsectorBspPath>,
    ownership: Vec<DoomSubsectorSectorOwnership>,
    sectors: Vec<DoomSector>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoomWalkFloorResolution {
    Accepted {
        source_sector: DoomSourceRecord,
        floor_height: i16,
        ceiling_height: i16,
    },
    StepTooHigh {
        source_sector: DoomSourceRecord,
        current_floor_height: i16,
        candidate_floor_height: i16,
        maximum_step_up: i16,
    },
    InsufficientClearance {
        source_sector: DoomSourceRecord,
        floor_height: i16,
        ceiling_height: i16,
        required_clearance: i16,
    },
    PointOutsideUniqueSubsector {
        point: [i16; 2],
    },
}

/// Deterministic corpus probe chosen from the nearest known blocking source
/// line. It exists only to retain E1M1 collision evidence without requiring
/// an interactive capture to reproduce a contact.
#[derive(Clone, Debug, PartialEq)]
pub struct DoomWalkNearestWallProbe {
    pub source_linedef: u32,
    pub distance_before_move: f32,
    pub observation: DoomWalkMoveObservation,
}

impl DoomWalkCollisionWorld {
    /// One-sided linedefs block. Two-sided lines enter only with classic
    /// explicit blocking set; vertical clearance is intentionally deferred.
    pub fn from_map(map: &DoomMapCore) -> Self {
        let walls = map
            .linedefs
            .iter()
            .enumerate()
            .filter_map(|(index, line)| {
                let one_sided = line.right_sidedef.is_none() || line.left_sidedef.is_none();
                if !one_sided && line.flags & DOOM_LINE_BLOCKING == 0 {
                    return None;
                }
                let start = map.vertices.get(line.start_vertex as usize)?;
                let end = map.vertices.get(line.end_vertex as usize)?;
                Some(DoomWalkWall {
                    source_linedef: index as u32,
                    start: [f32::from(start.x), f32::from(start.y)],
                    end: [f32::from(end.x), f32::from(end.y)],
                })
            })
            .collect();
        Self {
            walls,
            blockmap_origin: [
                f32::from(map.blockmap.origin_x),
                f32::from(map.blockmap.origin_y),
            ],
            blockmap_dimensions: [
                usize::from(map.blockmap.columns),
                usize::from(map.blockmap.rows),
            ],
            blockmap_cells: map
                .blockmap
                .cell_linedefs
                .iter()
                .map(|cell| cell.linedefs.iter().map(|line| u32::from(*line)).collect())
                .collect(),
        }
    }

    pub fn blocking_wall_count(&self) -> usize {
        self.walls.len()
    }

    pub fn probe_nearest_blocking_wall(
        &self,
        start: [f32; 2],
        radius: f32,
    ) -> Option<DoomWalkNearestWallProbe> {
        let (wall, closest, distance) = self
            .walls
            .iter()
            .map(|wall| {
                let closest = closest_point_on_segment(start, wall.start, wall.end);
                (wall, closest, length(subtract(start, closest)))
            })
            .min_by(|left, right| left.2.total_cmp(&right.2))?;
        let direction = normalize(subtract(closest, start))
            .unwrap_or_else(|| stable_wall_normal(wall.start, wall.end, [1.0, 0.0]));
        let observation = self.move_disc(start, scale(direction, distance + radius + 8.0), radius);
        Some(DoomWalkNearestWallProbe {
            source_linedef: wall.source_linedef,
            distance_before_move: distance,
            observation,
        })
    }

    pub fn move_disc(
        &self,
        start: [f32; 2],
        requested_delta: [f32; 2],
        radius: f32,
    ) -> DoomWalkMoveObservation {
        assert!(
            radius.is_finite() && radius > 0.0,
            "walk radius must be positive and finite"
        );
        let steps = (length(requested_delta) / MAX_SUBSTEP_DISTANCE)
            .ceil()
            .max(1.0) as usize;
        let step = scale(requested_delta, 1.0 / steps as f32);
        let mut position = start;
        let mut contacted = BTreeSet::new();
        let mut candidates_seen = BTreeSet::new();
        let mut fallback = false;

        for _ in 0..steps {
            let target = add(position, step);
            let (candidates, used_fallback) =
                self.candidates_for_swept_disc(position, target, radius);
            fallback |= used_fallback;
            candidates_seen.extend(candidates.iter().copied());
            position = target;
            for _ in 0..MAX_OVERLAP_PASSES {
                let mut corrected = false;
                for candidate in &candidates {
                    let Some(wall) = self
                        .walls
                        .iter()
                        .find(|wall| wall.source_linedef == *candidate)
                    else {
                        continue;
                    };
                    let closest = closest_point_on_segment(position, wall.start, wall.end);
                    let separation = subtract(position, closest);
                    let distance = length(separation);
                    if distance >= radius {
                        continue;
                    }
                    let normal = if distance > f32::EPSILON {
                        scale(separation, 1.0 / distance)
                    } else {
                        stable_wall_normal(wall.start, wall.end, step)
                    };
                    position = add(position, scale(normal, radius - distance + PUSH_EPSILON));
                    contacted.insert(wall.source_linedef);
                    corrected = true;
                }
                if !corrected {
                    break;
                }
            }
        }
        DoomWalkMoveObservation {
            requested_delta,
            resolved_position: position,
            contacted_linedefs: contacted.into_iter().collect(),
            broad_phase_candidates: candidates_seen.len(),
            used_full_wall_fallback: fallback,
        }
    }

    /// Runs the unchanged source-space collision world behind one AR-0028
    /// candidate embedding. Contact identities and blockmap lookup remain Doom
    /// source facts; only the caller-facing position and delta are converted.
    pub fn move_disc_in_embedding(
        &self,
        embedding: DoomComparativeEmbedding,
        world_start: [f32; 2],
        world_delta: [f32; 2],
        radius: f32,
    ) -> DoomWalkMoveObservation {
        let (source_start, _) =
            embedding.lower_direction(Vec3::new(world_start[0], 0.0, world_start[1]));
        let (source_delta, _) =
            embedding.lower_direction(Vec3::new(world_delta[0], 0.0, world_delta[1]));
        let source = self.move_disc(source_start, source_delta, radius);
        let resolved = embedding.lift_direction(source.resolved_position, 0.0);
        DoomWalkMoveObservation {
            requested_delta: world_delta,
            resolved_position: [resolved.x, resolved.z],
            contacted_linedefs: source.contacted_linedefs,
            broad_phase_candidates: source.broad_phase_candidates,
            used_full_wall_fallback: source.used_full_wall_fallback,
        }
    }

    fn candidates_for_swept_disc(
        &self,
        start: [f32; 2],
        end: [f32; 2],
        radius: f32,
    ) -> (Vec<u32>, bool) {
        let min = [start[0].min(end[0]) - radius, start[1].min(end[1]) - radius];
        let max = [start[0].max(end[0]) + radius, start[1].max(end[1]) + radius];
        let mut candidates = BTreeSet::new();
        if let (Some(low), Some(high)) = (self.blockmap_cell(min), self.blockmap_cell(max)) {
            for row in low[1]..=high[1] {
                for column in low[0]..=high[0] {
                    let index = row * self.blockmap_dimensions[0] + column;
                    if let Some(cell) = self.blockmap_cells.get(index) {
                        candidates.extend(cell.iter().copied());
                    }
                }
            }
        }
        let candidates = candidates
            .into_iter()
            .filter(|candidate| {
                self.walls
                    .iter()
                    .any(|wall| wall.source_linedef == *candidate)
            })
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            return (
                self.walls.iter().map(|wall| wall.source_linedef).collect(),
                true,
            );
        }
        (candidates, false)
    }

    fn blockmap_cell(&self, point: [f32; 2]) -> Option<[usize; 2]> {
        let column =
            ((point[0] - self.blockmap_origin[0]) / CLASSIC_BLOCKMAP_SPAN).floor() as isize;
        let row = ((point[1] - self.blockmap_origin[1]) / CLASSIC_BLOCKMAP_SPAN).floor() as isize;
        if column < 0
            || row < 0
            || column >= self.blockmap_dimensions[0] as isize
            || row >= self.blockmap_dimensions[1] as isize
        {
            return None;
        }
        Some([column as usize, row as usize])
    }
}

impl DoomWalkFloorWorld {
    pub fn from_map(map: &DoomMapCore) -> Result<Self, DoomGeometryError> {
        Ok(Self {
            paths: resolve_doom_subsector_bsp_paths(map)?,
            ownership: resolve_doom_subsector_sector_ownership(map)?,
            sectors: map.sectors.clone(),
        })
    }

    /// Evaluates the floor at a candidate point, preserving an explicit
    /// rejection when BSP ownership is ambiguous. Descents are permitted;
    /// only upward steps above the retained corpus limit are blocked.
    pub fn resolve_transition(
        &self,
        point: [f32; 2],
        current_floor_height: i16,
    ) -> DoomWalkFloorResolution {
        self.resolve_transition_with_ceiling_overrides(point, current_floor_height, &[])
    }

    /// Resolves a candidate world point through the unchanged Doom BSP/source
    /// ownership under one AR-0028 embedding.
    pub fn resolve_transition_in_embedding(
        &self,
        embedding: DoomComparativeEmbedding,
        world_point: [f32; 2],
        current_floor_height: i16,
        ceiling_overrides: &[(DoomSourceRecord, i16)],
    ) -> DoomWalkFloorResolution {
        let (source_point, _) =
            embedding.lower_direction(Vec3::new(world_point[0], 0.0, world_point[1]));
        self.resolve_transition_with_ceiling_overrides(
            source_point,
            current_floor_height,
            ceiling_overrides,
        )
    }

    /// Applies a bounded caller-owned ceiling overlay after source-sector
    /// ownership is resolved. The immutable WAD sector still supplies the
    /// floor and default ceiling; active corpus runtime state may replace only
    /// the ceiling for its retained source-sector identity.
    pub fn resolve_transition_with_ceiling_overrides(
        &self,
        point: [f32; 2],
        current_floor_height: i16,
        ceiling_overrides: &[(DoomSourceRecord, i16)],
    ) -> DoomWalkFloorResolution {
        let point = [
            round_map_coordinate(point[0]),
            round_map_coordinate(point[1]),
        ];
        let Ok(location) = locate_doom_point_subsector(point, &self.paths) else {
            return DoomWalkFloorResolution::PointOutsideUniqueSubsector { point };
        };
        let Some(ownership) = self
            .ownership
            .iter()
            .find(|ownership| ownership.source_subsector == location.source_subsector)
        else {
            return DoomWalkFloorResolution::PointOutsideUniqueSubsector { point };
        };
        let Some(sector) = self.sectors.get(usize::from(ownership.sector_index)) else {
            return DoomWalkFloorResolution::PointOutsideUniqueSubsector { point };
        };
        let ceiling_height = source_ceiling_height(sector, ceiling_overrides);
        classify_floor_transition(
            sector.source,
            sector.floor_height,
            ceiling_height,
            current_floor_height,
        )
    }
}

fn source_ceiling_height(
    sector: &DoomSector,
    ceiling_overrides: &[(DoomSourceRecord, i16)],
) -> i16 {
    ceiling_overrides
        .iter()
        .find_map(|(source, height)| (*source == sector.source).then_some(*height))
        .unwrap_or(sector.ceiling_height)
}

fn classify_floor_transition(
    source_sector: DoomSourceRecord,
    floor_height: i16,
    ceiling_height: i16,
    current_floor_height: i16,
) -> DoomWalkFloorResolution {
    if ceiling_height - floor_height < CLASSIC_PLAYER_HEIGHT {
        return DoomWalkFloorResolution::InsufficientClearance {
            source_sector,
            floor_height,
            ceiling_height,
            required_clearance: CLASSIC_PLAYER_HEIGHT,
        };
    }
    if floor_height - current_floor_height > CLASSIC_MAX_STEP_UP {
        return DoomWalkFloorResolution::StepTooHigh {
            source_sector,
            current_floor_height,
            candidate_floor_height: floor_height,
            maximum_step_up: CLASSIC_MAX_STEP_UP,
        };
    }
    DoomWalkFloorResolution::Accepted {
        source_sector,
        floor_height,
        ceiling_height,
    }
}

fn round_map_coordinate(value: f32) -> i16 {
    value
        .round()
        .clamp(f32::from(i16::MIN), f32::from(i16::MAX)) as i16
}

fn closest_point_on_segment(point: [f32; 2], start: [f32; 2], end: [f32; 2]) -> [f32; 2] {
    let edge = subtract(end, start);
    let length_squared = dot(edge, edge);
    if length_squared <= f32::EPSILON {
        return start;
    }
    add(
        start,
        scale(
            edge,
            (dot(subtract(point, start), edge) / length_squared).clamp(0.0, 1.0),
        ),
    )
}

fn stable_wall_normal(start: [f32; 2], end: [f32; 2], movement: [f32; 2]) -> [f32; 2] {
    let edge = subtract(end, start);
    let candidate = normalize([-edge[1], edge[0]]).unwrap_or([1.0, 0.0]);
    if dot(candidate, movement) > 0.0 {
        scale(candidate, -1.0)
    } else {
        candidate
    }
}
fn add(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] + b[0], a[1] + b[1]]
}
fn subtract(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
fn scale(value: [f32; 2], scalar: f32) -> [f32; 2] {
    [value[0] * scalar, value[1] * scalar]
}
fn dot(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}
fn length(value: [f32; 2]) -> f32 {
    dot(value, value).sqrt()
}
fn normalize(value: [f32; 2]) -> Option<[f32; 2]> {
    let length = length(value);
    (length > f32::EPSILON).then(|| scale(value, 1.0 / length))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn world(walls: Vec<DoomWalkWall>) -> DoomWalkCollisionWorld {
        DoomWalkCollisionWorld {
            walls,
            blockmap_origin: [-128.0, -128.0],
            blockmap_dimensions: [2, 2],
            blockmap_cells: vec![vec![0], vec![0], vec![0], vec![0]],
        }
    }
    #[test]
    fn disc_stops_before_a_blocking_wall() {
        let world = world(vec![DoomWalkWall {
            source_linedef: 0,
            start: [0.0, -64.0],
            end: [0.0, 64.0],
        }]);
        let result = world.move_disc([-20.0, 0.0], [32.0, 0.0], 4.0);
        assert!(result.resolved_position[0] <= -4.0 + 0.01);
        assert_eq!(result.contacted_linedefs, vec![0]);
    }
    #[test]
    fn disc_slides_along_a_blocking_wall() {
        let world = world(vec![DoomWalkWall {
            source_linedef: 7,
            start: [0.0, -64.0],
            end: [0.0, 64.0],
        }]);
        let result = world.move_disc([-20.0, -20.0], [32.0, 32.0], 4.0);
        assert!(result.resolved_position[0] <= -4.0 + 0.01);
        assert!(result.resolved_position[1] > -5.0);
        assert_eq!(result.contacted_linedefs, vec![7]);
    }
    #[test]
    fn empty_blockmap_candidates_fall_back_to_known_blocking_walls() {
        let mut world = world(vec![DoomWalkWall {
            source_linedef: 2,
            start: [0.0, -64.0],
            end: [0.0, 64.0],
        }]);
        world.blockmap_cells = vec![Vec::new(); 4];
        let result = world.move_disc([-20.0, 0.0], [32.0, 0.0], 4.0);
        assert!(result.used_full_wall_fallback);
        assert_eq!(result.contacted_linedefs, vec![2]);
    }

    #[test]
    fn nearest_wall_probe_retains_the_contacting_source_linedef() {
        let world = world(vec![DoomWalkWall {
            source_linedef: 9,
            start: [0.0, -64.0],
            end: [0.0, 64.0],
        }]);
        let probe = world
            .probe_nearest_blocking_wall([-20.0, 0.0], 4.0)
            .expect("one blocking wall exists");
        assert_eq!(probe.source_linedef, 9);
        assert_eq!(probe.observation.contacted_linedefs, vec![9]);
    }

    #[test]
    fn candidate_embeddings_preserve_collision_contacts_and_resolved_source_position() {
        let world = world(vec![DoomWalkWall {
            source_linedef: 17,
            start: [0.0, -64.0],
            end: [0.0, 64.0],
        }]);
        let source_start = [-20.0, -20.0];
        let source_delta = [32.0, 32.0];
        let source = world.move_disc(source_start, source_delta, 4.0);

        for embedding in [
            DoomComparativeEmbedding::PreserveEast,
            DoomComparativeEmbedding::PreserveNorth,
        ] {
            let lifted_start = embedding.lift_direction(source_start, 0.0);
            let lifted_delta = embedding.lift_direction(source_delta, 0.0);
            let candidate = world.move_disc_in_embedding(
                embedding,
                [lifted_start.x, lifted_start.z],
                [lifted_delta.x, lifted_delta.z],
                4.0,
            );
            let (resolved_source, _) = embedding.lower_direction(Vec3::new(
                candidate.resolved_position[0],
                0.0,
                candidate.resolved_position[1],
            ));

            assert_eq!(candidate.contacted_linedefs, source.contacted_linedefs);
            assert_eq!(
                candidate.broad_phase_candidates,
                source.broad_phase_candidates
            );
            assert_eq!(
                candidate.used_full_wall_fallback,
                source.used_full_wall_fallback
            );
            assert!((resolved_source[0] - source.resolved_position[0]).abs() < 0.000_1);
            assert!((resolved_source[1] - source.resolved_position[1]).abs() < 0.000_1);
        }
    }

    fn source_sector(record_index: u32) -> DoomSourceRecord {
        DoomSourceRecord {
            lump_index: 4,
            record_index,
        }
    }

    #[test]
    fn floor_policy_allows_a_classic_step_and_a_descent() {
        assert!(matches!(
            classify_floor_transition(source_sector(3), 24, 128, 0),
            DoomWalkFloorResolution::Accepted {
                floor_height: 24,
                ..
            }
        ));
        assert!(matches!(
            classify_floor_transition(source_sector(4), -64, 64, 0),
            DoomWalkFloorResolution::Accepted {
                floor_height: -64,
                ..
            }
        ));
    }

    #[test]
    fn floor_policy_rejects_a_tall_step_and_insufficient_clearance() {
        assert!(matches!(
            classify_floor_transition(source_sector(5), 25, 128, 0),
            DoomWalkFloorResolution::StepTooHigh {
                maximum_step_up: 24,
                ..
            }
        ));
        assert!(matches!(
            classify_floor_transition(source_sector(6), 0, 55, 0),
            DoomWalkFloorResolution::InsufficientClearance {
                required_clearance: 56,
                ..
            }
        ));
    }

    #[test]
    fn active_runtime_ceiling_override_replaces_only_the_matching_source_sector() {
        let sector = DoomSector {
            floor_height: 0,
            ceiling_height: 0,
            floor_texture: "FLOOR0_1".to_owned(),
            ceiling_texture: "CEIL1_1".to_owned(),
            light_level: 160,
            special: 0,
            tag: 0,
            source: source_sector(4),
        };
        assert_eq!(source_ceiling_height(&sector, &[]), 0);
        assert_eq!(
            source_ceiling_height(&sector, &[(source_sector(4), 68)]),
            68
        );
        assert_eq!(source_ceiling_height(&sector, &[(source_sector(5), 68)]), 0);
    }
}
