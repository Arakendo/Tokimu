//! Corpus-local monster sight observations.
//!
//! Doom's `REJECT` matrix is only a negative prefilter. A permitted sector
//! pair still needs a source-space trace through finite linedefs and vertical
//! openings. This module retains those decisions separately so later monster
//! movement cannot confuse renderer visibility with gameplay sight.

use doom_map_provider::{DoomMapCore, DoomRejectMatrix};

const INTERSECTION_EPSILON: f32 = 0.000_1;
const CLASSIC_MELEE_RANGE: f32 = 64.0;

#[derive(Clone, Copy, Debug, PartialEq)]
struct DoomSightLine {
    source_linedef: u32,
    start: [f32; 2],
    end: [f32; 2],
    opening: Option<[f32; 2]>,
}

#[derive(Clone, Debug)]
pub struct DoomMonsterSightWorld {
    reject: Option<DoomRejectMatrix>,
    lines: Vec<DoomSightLine>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoomMonsterPerception {
    PlayerDead,
    RejectUnavailable,
    RejectForbidden,
    SightBlocked { source_linedef: u32 },
    OutsideFrontArc,
    Acquired { crossed_openings: usize },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DoomMonsterPerceptionQuery {
    pub monster_sector: usize,
    pub monster_position: [f32; 2],
    pub monster_floor: f32,
    pub monster_height: f32,
    pub monster_angle_degrees: f32,
    pub player_sector: usize,
    pub player_position: [f32; 2],
    pub player_floor: f32,
    pub player_height: f32,
    pub player_alive: bool,
    pub all_around: bool,
}

impl DoomMonsterSightWorld {
    pub fn from_map(map: &DoomMapCore) -> Self {
        let lines = map
            .linedefs
            .iter()
            .filter_map(|line| {
                let start = map.vertices.get(usize::from(line.start_vertex))?;
                let end = map.vertices.get(usize::from(line.end_vertex))?;
                let opening =
                    line.right_sidedef
                        .zip(line.left_sidedef)
                        .and_then(|(right, left)| {
                            let right = map.sidedefs.get(usize::from(right))?;
                            let left = map.sidedefs.get(usize::from(left))?;
                            let right = map.sectors.get(usize::from(right.sector))?;
                            let left = map.sectors.get(usize::from(left.sector))?;
                            Some([
                                f32::from(right.floor_height.max(left.floor_height)),
                                f32::from(right.ceiling_height.min(left.ceiling_height)),
                            ])
                        });
                Some(DoomSightLine {
                    source_linedef: line.source.record_index,
                    start: [f32::from(start.x), f32::from(start.y)],
                    end: [f32::from(end.x), f32::from(end.y)],
                    opening,
                })
            })
            .collect();
        Self {
            reject: Some(map.reject.clone()),
            lines,
        }
    }

    pub fn observe(&self, query: DoomMonsterPerceptionQuery) -> DoomMonsterPerception {
        if !query.player_alive {
            return DoomMonsterPerception::PlayerDead;
        }
        if let Some(reject) = &self.reject {
            match reject.forbids_monster_sight(query.monster_sector, query.player_sector) {
                Ok(true) => return DoomMonsterPerception::RejectForbidden,
                Ok(false) => {}
                Err(_) => return DoomMonsterPerception::RejectUnavailable,
            }
        }
        let sight_start = query.monster_floor + query.monster_height * 0.75;
        let mut bottom_slope = query.player_floor - sight_start;
        let mut top_slope = query.player_floor + query.player_height - sight_start;
        let mut crossings = self
            .lines
            .iter()
            .filter_map(|line| {
                segment_crossing_fraction(
                    query.monster_position,
                    query.player_position,
                    line.start,
                    line.end,
                )
                .map(|fraction| (fraction, *line))
            })
            .collect::<Vec<_>>();
        crossings.sort_by(|left, right| {
            left.0
                .total_cmp(&right.0)
                .then_with(|| left.1.source_linedef.cmp(&right.1.source_linedef))
        });
        let mut crossed_openings = 0;
        for (fraction, line) in crossings {
            let Some([open_bottom, open_top]) = line.opening else {
                return DoomMonsterPerception::SightBlocked {
                    source_linedef: line.source_linedef,
                };
            };
            if open_bottom >= open_top {
                return DoomMonsterPerception::SightBlocked {
                    source_linedef: line.source_linedef,
                };
            }
            bottom_slope = bottom_slope.max((open_bottom - sight_start) / fraction);
            top_slope = top_slope.min((open_top - sight_start) / fraction);
            if top_slope <= bottom_slope {
                return DoomMonsterPerception::SightBlocked {
                    source_linedef: line.source_linedef,
                };
            }
            crossed_openings += 1;
        }
        if !query.all_around
            && !inside_front_arc(
                query.monster_position,
                query.monster_angle_degrees,
                query.player_position,
            )
        {
            return DoomMonsterPerception::OutsideFrontArc;
        }
        DoomMonsterPerception::Acquired { crossed_openings }
    }
}

fn segment_crossing_fraction(
    ray_start: [f32; 2],
    ray_end: [f32; 2],
    line_start: [f32; 2],
    line_end: [f32; 2],
) -> Option<f32> {
    let ray = subtract(ray_end, ray_start);
    let line = subtract(line_end, line_start);
    let denominator = cross(ray, line);
    if denominator.abs() <= INTERSECTION_EPSILON {
        return None;
    }
    let offset = subtract(line_start, ray_start);
    let ray_fraction = cross(offset, line) / denominator;
    let line_fraction = cross(offset, ray) / denominator;
    (ray_fraction > INTERSECTION_EPSILON
        && ray_fraction < 1.0 - INTERSECTION_EPSILON
        && line_fraction >= -INTERSECTION_EPSILON
        && line_fraction <= 1.0 + INTERSECTION_EPSILON)
        .then_some(ray_fraction)
}

fn inside_front_arc(monster: [f32; 2], angle_degrees: f32, player: [f32; 2]) -> bool {
    let delta = subtract(player, monster);
    let distance = delta[0].abs().max(delta[1].abs()) + 0.5 * delta[0].abs().min(delta[1].abs());
    if distance <= CLASSIC_MELEE_RANGE {
        return true;
    }
    let radians = angle_degrees.to_radians();
    let forward = [radians.cos(), radians.sin()];
    forward[0] * delta[0] + forward[1] * delta[1] >= 0.0
}

fn subtract(left: [f32; 2], right: [f32; 2]) -> [f32; 2] {
    [left[0] - right[0], left[1] - right[1]]
}

fn cross(left: [f32; 2], right: [f32; 2]) -> f32 {
    left[0] * right[1] - left[1] * right[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query() -> DoomMonsterPerceptionQuery {
        DoomMonsterPerceptionQuery {
            monster_sector: 0,
            monster_position: [0.0, 0.0],
            monster_floor: 0.0,
            monster_height: 56.0,
            monster_angle_degrees: 0.0,
            player_sector: 0,
            player_position: [100.0, 0.0],
            player_floor: 0.0,
            player_height: 56.0,
            player_alive: true,
            all_around: false,
        }
    }

    fn world(lines: Vec<DoomSightLine>) -> DoomMonsterSightWorld {
        DoomMonsterSightWorld {
            reject: None,
            lines,
        }
    }

    #[test]
    fn one_sided_line_blocks_sight() {
        assert_eq!(
            world(vec![DoomSightLine {
                source_linedef: 7,
                start: [50.0, -10.0],
                end: [50.0, 10.0],
                opening: None,
            }])
            .observe(query()),
            DoomMonsterPerception::SightBlocked { source_linedef: 7 }
        );
    }

    #[test]
    fn finite_opening_clips_vertical_sight_slope() {
        let mut raised_player = query();
        raised_player.player_floor = 80.0;
        assert_eq!(
            world(vec![DoomSightLine {
                source_linedef: 8,
                start: [50.0, -10.0],
                end: [50.0, 10.0],
                opening: Some([0.0, 40.0]),
            }])
            .observe(raised_player),
            DoomMonsterPerception::SightBlocked { source_linedef: 8 }
        );
    }

    #[test]
    fn open_sight_still_honors_front_arc_and_melee_exception() {
        let empty = world(Vec::new());
        let mut behind = query();
        behind.player_position = [-100.0, 0.0];
        assert_eq!(
            empty.observe(behind),
            DoomMonsterPerception::OutsideFrontArc
        );
        behind.player_position = [-32.0, 0.0];
        assert_eq!(
            empty.observe(behind),
            DoomMonsterPerception::Acquired {
                crossed_openings: 0
            }
        );
    }
}
