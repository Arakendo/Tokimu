//! Deterministic generated-polygon investigations.
//!
//! These cases intentionally remain outside the reviewed catalog. They provide
//! repeatable inputs for probing geometry behavior without turning every seed
//! into a permanent golden contract.

use crate::{
    cases::PATH_STAGES,
    evidence::segment_intersections,
    geometry::{format_mesh_summary, validate_mesh},
    reports::failed_stage,
    CaseReport, CorpusStage, StageReport, StageStatus,
};
use ui_tools::{tessellate_general_fill_with_rule, VectorContour, VectorFillRule, VectorPath};

/// Runs one deterministic generated polygon without adding it to the reviewed
/// case list. Generated cases are investigation inputs, not golden contracts.
pub fn run_generated_case(seed: u64, index: usize) -> CaseReport {
    let path = generated_path(seed, index);
    let id = format!("generated/{seed}/{index}");
    let intersections = segment_intersections(&path);
    let mut report = CaseReport {
        id,
        producer: "generated/seeded-polygon".to_owned(),
        selected_stages: PATH_STAGES.to_vec(),
        stages: vec![StageReport {
            stage: CorpusStage::Source,
            status: StageStatus::Ready,
            summary: format!(
                "seed={seed} index={index} contours={} points={}",
                path.contours.len(),
                path.contours
                    .iter()
                    .map(|contour| contour.points.len())
                    .sum::<usize>()
            ),
        }],
        diagnostics: Vec::new(),
    };

    if !intersections.is_empty() {
        let message = format!(
            "generated polygon has {} self-intersection(s)",
            intersections.len()
        );
        report
            .stages
            .push(failed_stage(CorpusStage::Vector, &message));
        report.diagnostics.push(message);
        return report;
    }

    report.stages.push(StageReport {
        stage: CorpusStage::Vector,
        status: StageStatus::Ready,
        summary: format!(
            "finite={} intersections={}",
            path.is_finite(),
            intersections.len()
        ),
    });
    match tessellate_general_fill_with_rule(&path, VectorFillRule::EvenOdd) {
        Ok(triangles) => {
            let validation = validate_mesh(&triangles);
            if validation.finite && validation.complete_triangles {
                report.stages.push(StageReport {
                    stage: CorpusStage::Mesh,
                    status: StageStatus::Ready,
                    summary: format_mesh_summary(&validation),
                });
            } else {
                let message = format!(
                    "generated mesh validation failed: {}",
                    format_mesh_summary(&validation)
                );
                report
                    .stages
                    .push(failed_stage(CorpusStage::Mesh, &message));
                report.diagnostics.push(message);
            }
        }
        Err(error) => {
            report.stages.push(failed_stage(CorpusStage::Mesh, &error));
            report
                .diagnostics
                .push(format!("generated mesh tessellation failed: {error}"));
        }
    }
    report
}

fn generated_path(seed: u64, index: usize) -> VectorPath {
    let mut state = seed
        .wrapping_add((index as u64).wrapping_mul(0x9e3779b97f4a7c15))
        .max(1);
    let point_count = 5 + (next_random(&mut state) % 4) as usize;
    let mut points = Vec::with_capacity(point_count);
    for point_index in 0..point_count {
        let angle = std::f32::consts::TAU * point_index as f32 / point_count as f32;
        let radius = 0.28 + next_unit(&mut state) * 0.16;
        points.push([0.5 + angle.cos() * radius, 0.5 + angle.sin() * radius]);
    }
    VectorPath::new(vec![VectorContour::new(points, true)])
}

fn next_random(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn next_unit(state: &mut u64) -> f32 {
    (next_random(state) as f64 / u64::MAX as f64) as f32
}
