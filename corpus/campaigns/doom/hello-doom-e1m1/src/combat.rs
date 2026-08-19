//! Corpus-local deterministic combat collision primitives.
//!
//! The caller supplies the nearest finite world-surface distance. This module
//! owns actor-cylinder and projectile-sweep ordering only; it does not make
//! renderer declarations authoritative gameplay state or introduce a generic
//! Tokimu physics contract.

const COLLISION_EPSILON: f32 = 0.000_1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DoomCombatActor {
    pub source_thing: u32,
    pub kind: u16,
    pub position: [f32; 3],
    pub radius: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DoomCombatHit {
    World { distance: f32 },
    Actor { source_thing: u32, distance: f32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DoomCombatActorState {
    pub source_thing: u32,
    pub kind: u16,
    pub spawn_health: i32,
    pub health: i32,
    pub dead: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoomDamageOutcome {
    Hurt { remaining_health: i32 },
    Killed,
    AlreadyDead,
}

impl DoomCombatActorState {
    pub fn new(source_thing: u32, kind: u16) -> Option<Self> {
        let spawn_health = match kind {
            3004 => 20,
            9 => 30,
            3001 => 60,
            2035 => 20,
            _ => return None,
        };
        Some(Self {
            source_thing,
            kind,
            spawn_health,
            health: spawn_health,
            dead: false,
        })
    }

    pub fn apply_damage(&mut self, damage: i32) -> DoomDamageOutcome {
        if self.dead {
            return DoomDamageOutcome::AlreadyDead;
        }
        self.health = self.health.saturating_sub(damage.max(0));
        if self.health <= 0 {
            self.health = 0;
            self.dead = true;
            DoomDamageOutcome::Killed
        } else {
            DoomDamageOutcome::Hurt {
                remaining_health: self.health,
            }
        }
    }

    pub fn respawn(&mut self) {
        self.health = self.spawn_health;
        self.dead = false;
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DoomPlayRandom {
    index: u8,
}

impl DoomPlayRandom {
    pub fn next_byte(&mut self) -> u8 {
        self.index = self.index.wrapping_add(1);
        DOOM_RANDOM_TABLE[usize::from(self.index)]
    }

    pub fn pistol_damage(&mut self) -> i32 {
        5 * (i32::from(self.next_byte()) % 3 + 1)
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }
}

#[rustfmt::skip]
const DOOM_RANDOM_TABLE: [u8; 256] = [
      0,   8, 109, 220, 222, 241, 149, 107,  75, 248, 254, 140,  16,  66,  74,  21,
    211,  47,  80, 242, 154,  27, 205, 128, 161,  89,  77,  36,  95, 110,  85,  48,
    212, 140, 211, 249,  22,  79, 200,  50,  28, 188,  52, 140, 202, 120,  68, 145,
     62,  70, 184, 190,  91, 197, 152, 224, 149, 104,  25, 178, 252, 182, 202, 182,
    141, 197,   4,  81, 181, 242, 145,  42,  39, 227, 156, 198, 225, 193, 219,  93,
    122, 175, 249,   0, 175, 143,  70, 239,  46, 246, 163,  53, 163, 109, 168, 135,
      2, 235,  25,  92,  20, 145, 138,  77,  69, 166,  78, 176, 173, 212, 166, 113,
     94, 161,  41,  50, 239,  49, 111, 164,  70,  60,   2,  37, 171,  75, 136, 156,
     11,  56,  42, 146, 138, 229,  73, 146,  77,  61,  98, 196, 135, 106,  63, 197,
    195,  86,  96, 203, 113, 101, 170, 247, 181, 113,  80, 250, 108,   7, 255, 237,
    129, 226,  79, 107, 112, 166, 103, 241,  24, 223, 239, 120, 198,  58,  60,  82,
    128,   3, 184,  66, 143, 224, 145, 224,  81, 206, 163,  45,  63,  90, 168, 114,
     59,  33, 159,  95,  28, 139, 123,  98, 125, 196,  15,  70, 194, 253,  54,  14,
    109, 226,  71,  17, 161,  93, 186,  87, 244, 138,  20,  52, 123, 251,  26,  36,
     17,  46,  52, 231, 232,  76,  31, 221,  84,  37, 216, 165, 212, 106, 197, 242,
     98,  43,  39, 175, 254, 145, 190,  84, 118, 222, 187, 136, 120, 163, 236, 249,
];

pub fn trace_hitscan(
    origin: [f32; 3],
    direction: [f32; 3],
    maximum_distance: f32,
    world_distance: Option<f32>,
    actors: &[DoomCombatActor],
) -> Option<DoomCombatHit> {
    let direction = normalize3(direction)?;
    let maximum_distance = valid_maximum(maximum_distance)?;
    let world_distance = world_distance.filter(|distance| {
        distance.is_finite() && *distance >= 0.0 && *distance <= maximum_distance
    });
    let actor_hit = actors
        .iter()
        .filter_map(|actor| {
            ray_vertical_cylinder_distance(origin, direction, *actor)
                .filter(|distance| *distance <= maximum_distance)
                .map(|distance| (distance, actor.source_thing))
        })
        .min_by(|left, right| {
            left.0
                .total_cmp(&right.0)
                .then_with(|| left.1.cmp(&right.1))
        });

    match (world_distance, actor_hit) {
        (None, None) => None,
        (Some(distance), None) => Some(DoomCombatHit::World { distance }),
        (None, Some((distance, source_thing))) => Some(DoomCombatHit::Actor {
            source_thing,
            distance,
        }),
        (Some(world), Some((actor, source_thing))) if actor + COLLISION_EPSILON < world => {
            Some(DoomCombatHit::Actor {
                source_thing,
                distance: actor,
            })
        }
        (Some(distance), Some(_)) => Some(DoomCombatHit::World { distance }),
    }
}

/// Sweeps a finite projectile cylinder along one movement delta. The caller's
/// world distance is measured along the same normalized path and therefore
/// competes directly with expanded actor cylinders.
pub fn sweep_projectile(
    origin: [f32; 3],
    delta: [f32; 3],
    projectile_radius: f32,
    projectile_height: f32,
    world_distance: Option<f32>,
    actors: &[DoomCombatActor],
) -> Option<DoomCombatHit> {
    if !projectile_radius.is_finite()
        || projectile_radius < 0.0
        || !projectile_height.is_finite()
        || projectile_height < 0.0
    {
        return None;
    }
    let distance = length3(delta);
    let direction = normalize3(delta)?;
    let expanded = actors
        .iter()
        .map(|actor| DoomCombatActor {
            radius: actor.radius + projectile_radius,
            position: [
                actor.position[0],
                actor.position[1] - projectile_height,
                actor.position[2],
            ],
            height: actor.height + projectile_height,
            ..*actor
        })
        .collect::<Vec<_>>();
    trace_hitscan(origin, direction, distance, world_distance, &expanded)
}

fn ray_vertical_cylinder_distance(
    origin: [f32; 3],
    direction: [f32; 3],
    actor: DoomCombatActor,
) -> Option<f32> {
    if actor.radius <= 0.0 || actor.height <= 0.0 {
        return None;
    }
    let ox = origin[0] - actor.position[0];
    let oz = origin[2] - actor.position[2];
    if ox * ox + oz * oz <= actor.radius * actor.radius
        && origin[1] >= actor.position[1]
        && origin[1] <= actor.position[1] + actor.height
    {
        return Some(0.0);
    }
    let a = direction[0] * direction[0] + direction[2] * direction[2];
    let mut candidates = Vec::with_capacity(4);
    if a > COLLISION_EPSILON {
        let b = 2.0 * (ox * direction[0] + oz * direction[2]);
        let c = ox * ox + oz * oz - actor.radius * actor.radius;
        let discriminant = b * b - 4.0 * a * c;
        if discriminant >= 0.0 {
            let root = discriminant.sqrt();
            candidates.push((-b - root) / (2.0 * a));
            candidates.push((-b + root) / (2.0 * a));
        }
    }
    if direction[1].abs() > COLLISION_EPSILON {
        candidates.push((actor.position[1] - origin[1]) / direction[1]);
        candidates.push((actor.position[1] + actor.height - origin[1]) / direction[1]);
    }
    candidates
        .into_iter()
        .filter(|distance| distance.is_finite() && *distance >= 0.0)
        .filter(|distance| {
            let point = add3(origin, scale3(direction, *distance));
            let horizontal =
                (point[0] - actor.position[0]).powi(2) + (point[2] - actor.position[2]).powi(2);
            horizontal <= actor.radius * actor.radius + COLLISION_EPSILON
                && point[1] + COLLISION_EPSILON >= actor.position[1]
                && point[1] <= actor.position[1] + actor.height + COLLISION_EPSILON
        })
        .min_by(f32::total_cmp)
}

fn valid_maximum(value: f32) -> Option<f32> {
    (value.is_finite() && value >= 0.0).then_some(value)
}

fn normalize3(value: [f32; 3]) -> Option<[f32; 3]> {
    let length = length3(value);
    (length.is_finite() && length > COLLISION_EPSILON).then(|| scale3(value, 1.0 / length))
}

fn length3(value: [f32; 3]) -> f32 {
    (value[0] * value[0] + value[1] * value[1] + value[2] * value[2]).sqrt()
}

fn add3(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] + right[0], left[1] + right[1], left[2] + right[2]]
}

fn scale3(value: [f32; 3], scale: f32) -> [f32; 3] {
    [value[0] * scale, value[1] * scale, value[2] * scale]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn actor(source_thing: u32, position: [f32; 3]) -> DoomCombatActor {
        DoomCombatActor {
            source_thing,
            kind: 3004,
            position,
            radius: 20.0,
            height: 56.0,
        }
    }

    #[test]
    fn hitscan_selects_nearest_actor_with_stable_source_tie_break() {
        let hit = trace_hitscan(
            [0.0, 28.0, 0.0],
            [1.0, 0.0, 0.0],
            2048.0,
            None,
            &[actor(9, [100.0, 0.0, 0.0]), actor(3, [100.0, 0.0, 0.0])],
        );
        assert_eq!(
            hit,
            Some(DoomCombatHit::Actor {
                source_thing: 3,
                distance: 80.0
            })
        );
    }

    #[test]
    fn nearer_world_surface_occludes_actor_and_wins_a_tie() {
        let actors = [actor(3, [100.0, 0.0, 0.0])];
        assert_eq!(
            trace_hitscan(
                [0.0, 28.0, 0.0],
                [1.0, 0.0, 0.0],
                2048.0,
                Some(50.0),
                &actors
            ),
            Some(DoomCombatHit::World { distance: 50.0 })
        );
        assert!(matches!(
            trace_hitscan(
                [0.0, 28.0, 0.0],
                [1.0, 0.0, 0.0],
                2048.0,
                Some(80.0),
                &actors
            ),
            Some(DoomCombatHit::World { .. })
        ));
    }

    #[test]
    fn pitch_must_intersect_actor_vertical_interval() {
        let actors = [actor(3, [100.0, 0.0, 0.0])];
        assert!(trace_hitscan([0.0, 28.0, 0.0], [1.0, 1.0, 0.0], 2048.0, None, &actors).is_none());
    }

    #[test]
    fn projectile_sweep_expands_actor_by_projectile_volume() {
        let hit = sweep_projectile(
            [0.0, 24.0, 25.0],
            [100.0, 0.0, 0.0],
            6.0,
            8.0,
            None,
            &[actor(7, [100.0, 0.0, 0.0])],
        );
        assert!(matches!(
            hit,
            Some(DoomCombatHit::Actor {
                source_thing: 7,
                ..
            })
        ));
    }

    #[test]
    fn deterministic_pistol_damage_replay_kills_and_respawns_actor() {
        let replay = || {
            let mut random = DoomPlayRandom::default();
            let mut target = DoomCombatActorState::new(17, 3004).unwrap();
            let collision = [actor(17, [100.0, 0.0, 0.0])];
            let fire = |random: &mut DoomPlayRandom, target: &mut DoomCombatActorState| {
                assert_eq!(
                    trace_hitscan([0.0, 28.0, 0.0], [1.0, 0.0, 0.0], 2048.0, None, &collision,),
                    Some(DoomCombatHit::Actor {
                        source_thing: 17,
                        distance: 80.0,
                    })
                );
                target.apply_damage(random.pistol_damage())
            };
            let first = fire(&mut random, &mut target);
            let second = fire(&mut random, &mut target);
            (random, target, first, second)
        };
        let left = replay();
        let right = replay();
        assert_eq!(left, right);
        assert_eq!(
            left.2,
            DoomDamageOutcome::Hurt {
                remaining_health: 5
            }
        );
        assert_eq!(left.3, DoomDamageOutcome::Killed);
        let mut target = left.1;
        target.respawn();
        assert_eq!(target.health, 20);
        assert!(!target.dead);
    }
}
