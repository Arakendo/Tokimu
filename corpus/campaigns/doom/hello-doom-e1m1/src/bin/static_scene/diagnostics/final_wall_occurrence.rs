//! Exact controls for final ordered walls over untouched global-full planes.

use super::super::*;
use super::ordered_causality::positive_wall_support_control;
use super::sky_transition_parity::source_ray_vectors;
use crate::render_strategies::final_wall_occurrence_global_planes;

#[derive(Clone, Copy, Debug)]
struct WallControl {
    name: &'static str,
    origin: [f64; 3],
    direction: [f64; 3],
    linedef: u32,
    expected_hit: bool,
}

const WALL_CONTROLS: [WallControl; 4] = [
    WallControl {
        name: "wall-241-finally-unsupported",
        origin: [2_042.021_240_234, -2_975.617_919_922, -20.0],
        direction: [0.613_750_577, -0.787_513_614, 0.055_970_095],
        linedef: 241,
        expected_hit: false,
    },
    WallControl {
        name: "hut-wall-160-sky-falsifier-survives",
        origin: [2076.0, -3560.0, 36.0],
        direction: [0.893_540_758, -0.447_666_839, -0.034_341_145],
        linedef: 160,
        expected_hit: true,
    },
    WallControl {
        name: "hut-wall-159-sky-falsifier-survives",
        origin: [2076.0, -3560.0, 36.0],
        direction: [0.942_283_736, -0.333_064_670, -0.034_194_817],
        linedef: 159,
        expected_hit: true,
    },
    WallControl {
        name: "far-left-wall-203-sky-falsifier-survives",
        origin: [2902.0, -3207.0, 9.0],
        direction: [-0.807_392_359, -0.589_416_674, 0.026_562_429],
        linedef: 203,
        expected_hit: true,
    },
];

pub(crate) fn report_final_wall_occurrence_global_planes(scene: &SceneInput) -> PlatformResult<()> {
    let (positive_origin, positive_direction, positive_linedef, _) =
        positive_wall_support_control();
    let positive = WallControl {
        name: "wall-135-finally-supported",
        origin: positive_origin,
        direction: positive_direction,
        linedef: positive_linedef,
        expected_hit: true,
    };
    let global_planes = scene
        .opaque_draws
        .iter()
        .filter(|draw| matches!(draw.source, StaticDrawSource::Flat { .. }))
        .cloned()
        .collect::<Vec<_>>();
    let mut rows = Vec::new();
    let mut passed = 0usize;

    for control in WALL_CONTROLS.into_iter().chain([positive]) {
        let viewer = [
            control.origin[0].round() as i16,
            control.origin[1].round() as i16,
        ];
        let heading = control.direction[1].atan2(control.direction[0]);
        let prepared = final_wall_occurrence_global_planes::prepare(
            scene,
            &scene.door_geometry_source.map,
            viewer,
            heading,
            control.origin[2].round() as i16,
        )?;
        let prepared_planes = prepared
            .opaque_draws
            .iter()
            .filter(|draw| matches!(draw.source, StaticDrawSource::Flat { .. }))
            .cloned()
            .collect::<Vec<_>>();
        if prepared_planes != global_planes {
            return Err(io::Error::other(format!(
                "{} changed global-full plane declarations",
                control.name
            ))
            .into());
        }
        let wall_draws = prepared
            .opaque_draws
            .iter()
            .chain(&prepared.cutout_draws)
            .filter(|draw| {
                matches!(
                    draw.source,
                    StaticDrawSource::Wall { source_linedef, .. }
                        if source_linedef.record_index == control.linedef
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        let (origin, direction) = source_ray_vectors(control.origin, control.direction);
        let hit = nearest_prepared_ray_hit(origin, direction, &wall_draws, None);
        let result = hit.is_some() == control.expected_hit;
        passed += usize::from(result);
        rows.push(format!(
            "case={}:linedef={}:expected={}:candidate={}:distance={}:matching-declarations={}:global-planes={}:plane-identity=unchanged:result={}",
            control.name,
            control.linedef,
            if control.expected_hit { "hit" } else { "none" },
            if hit.is_some() { "hit" } else { "none" },
            hit.map(|hit| format!("{:.3}", hit.distance))
                .unwrap_or_else(|| "none".to_owned()),
            wall_draws.len(),
            prepared_planes.len(),
            if result { "pass" } else { "fail" },
        ));
    }
    let mut fingerprint = 0xcbf29ce484222325u64;
    for row in &rows {
        for byte in row.bytes() {
            fingerprint ^= u64::from(byte);
            fingerprint = fingerprint.wrapping_mul(0x100000001b3);
        }
    }
    if passed != rows.len() {
        return Err(io::Error::other(format!(
            "final wall occurrence controls failed: {passed}/{}; rows=[{}]",
            rows.len(),
            rows.join(" | "),
        ))
        .into());
    }
    println!(
        "E1M1 final-wall-occurrence-global-planes Slice 0-1: controls={passed}/{}; global-planes={}; wall-241=absent; wall-135=present; sky-falsifier-walls=[159:present,160:present,203:present]; plane-identity=unchanged; renderer-vocabulary=ordinary-declarations-only; conservation=balanced; fingerprint={fingerprint:016x}; rows=[{}]",
        rows.len(),
        global_planes.len(),
        rows.join(" | "),
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wall_controls_include_negative_and_sky_falsifier_positives() {
        assert_eq!(
            WALL_CONTROLS
                .iter()
                .filter(|control| !control.expected_hit)
                .count(),
            1
        );
        assert_eq!(
            WALL_CONTROLS
                .iter()
                .filter(|control| control.expected_hit)
                .count(),
            3
        );
    }
}
