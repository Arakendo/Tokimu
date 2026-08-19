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
}
