//! Synthetic topology cases and their structural coverage probes.

use crate::{
    evidence::segment_intersections,
    geometry::{format_mesh_summary, point_in_triangle, validate_mesh},
    reports::failed_stage,
    CaseReport, CorpusCase, CorpusStage, StageReport, StageStatus, SyntheticCase,
};
use ui_tools::{tessellate_general_fill_with_rule, VectorContour, VectorFillRule, VectorPath};

pub fn run_synthetic_case(case: SyntheticCase) -> CaseReport {
    let mut report = CaseReport {
        id: case.id.to_owned(),
        producer: "synthetic/topology".to_owned(),
        selected_stages: CorpusCase::Synthetic(case).selected_stages().to_vec(),
        stages: vec![StageReport {
            stage: CorpusStage::Source,
            status: StageStatus::Ready,
            summary: case.description.to_owned(),
        }],
        diagnostics: Vec::new(),
    };
    let path = synthetic_path(case);
    let intersections = segment_intersections(&path);
    let vector_summary = format!(
        "contours={} points={} finite={} intersections={}",
        path.contours.len(),
        path.contours
            .iter()
            .map(|contour| contour.points.len())
            .sum::<usize>(),
        path.is_finite(),
        intersections.len()
    );
    if case.expected_failure {
        if intersections.is_empty() {
            let message = "expected vector self-intersection was not detected".to_owned();
            report
                .stages
                .push(failed_stage(CorpusStage::Vector, &message));
            report.diagnostics.push(message);
            return report;
        }
        report.stages.push(StageReport {
            stage: CorpusStage::Vector,
            status: StageStatus::ExpectedFailure,
            summary: format!("expected unsupported topology: {vector_summary}"),
        });
        report.stages.push(StageReport {
            stage: CorpusStage::Mesh,
            status: StageStatus::ExpectedFailure,
            summary: "not attempted after expected vector-topology failure".to_owned(),
        });
        return report;
    }
    report.stages.push(StageReport {
        stage: CorpusStage::Vector,
        status: StageStatus::Ready,
        summary: vector_summary,
    });
    match tessellate_general_fill_with_rule(&path, VectorFillRule::EvenOdd) {
        Ok(triangles) => {
            let validation = validate_mesh(&triangles);
            let coverage = validate_coverage(case, &path, &triangles);
            if validation.finite && validation.complete_triangles && coverage.is_empty() {
                report.stages.push(StageReport {
                    stage: CorpusStage::Mesh,
                    status: StageStatus::Ready,
                    summary: format_mesh_summary(&validation),
                });
            } else {
                let message = format!(
                    "mesh validation failed: {}",
                    format_mesh_summary(&validation)
                );
                report
                    .stages
                    .push(failed_stage(CorpusStage::Mesh, &message));
                report.diagnostics.push(message);
            }
            report.diagnostics.extend(coverage);
        }
        Err(error) => {
            report.stages.push(failed_stage(CorpusStage::Mesh, &error));
            report
                .diagnostics
                .push(format!("mesh tessellation failed: {error}"));
        }
    }
    report
}

pub(crate) fn synthetic_path(case: SyntheticCase) -> VectorPath {
    let contour = |points: &[[f32; 2]]| VectorContour::new(points.to_vec(), true);
    match case.id {
        "synthetic/convex-rectangle" => VectorPath::new(vec![contour(&[
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
        ])]),
        "synthetic/concave-notch" => VectorPath::new(vec![contour(&[
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 0.35],
            [0.55, 0.35],
            [0.55, 1.0],
            [0.0, 1.0],
        ])]),
        "synthetic/multi-contour-hole" => VectorPath::new(vec![
            contour(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]),
            contour(&[[0.25, 0.25], [0.25, 0.75], [0.75, 0.75], [0.75, 0.25]]),
        ]),
        "synthetic/near-degenerate" => VectorPath::new(vec![contour(&[
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 0.00001],
            [0.0, 0.00001],
        ])]),
        "synthetic/self-intersection-bowtie" => VectorPath::new(vec![contour(&[
            [0.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
            [1.0, 0.0],
        ])]),
        _ => unreachable!("synthetic case must be declared in SYNTHETIC_CASES"),
    }
}

#[derive(Clone, Copy)]
struct CoverageProbe {
    point: [f32; 2],
    expected_inside: bool,
}

fn coverage_probes(case: SyntheticCase) -> &'static [CoverageProbe] {
    const RECTANGLE: [CoverageProbe; 2] = [
        CoverageProbe {
            point: [0.5, 0.5],
            expected_inside: true,
        },
        CoverageProbe {
            point: [1.25, 0.5],
            expected_inside: false,
        },
    ];
    const NOTCH: [CoverageProbe; 2] = [
        CoverageProbe {
            point: [0.2, 0.2],
            expected_inside: true,
        },
        CoverageProbe {
            point: [0.8, 0.8],
            expected_inside: false,
        },
    ];
    const HOLE: [CoverageProbe; 3] = [
        CoverageProbe {
            point: [0.1, 0.1],
            expected_inside: true,
        },
        CoverageProbe {
            point: [0.5, 0.5],
            expected_inside: false,
        },
        CoverageProbe {
            point: [1.1, 0.5],
            expected_inside: false,
        },
    ];
    match case.id {
        "synthetic/convex-rectangle" => &RECTANGLE,
        "synthetic/concave-notch" => &NOTCH,
        "synthetic/multi-contour-hole" => &HOLE,
        "synthetic/near-degenerate" | "synthetic/self-intersection-bowtie" => &[],
        _ => &[],
    }
}

fn validate_coverage(
    case: SyntheticCase,
    path: &VectorPath,
    triangles: &[[f32; 2]],
) -> Vec<String> {
    coverage_probes(case)
        .iter()
        .filter_map(|probe| {
            let source_inside = point_in_path(probe.point, path);
            let mesh_inside = point_in_mesh(probe.point, triangles);
            if source_inside != probe.expected_inside || mesh_inside != probe.expected_inside {
                Some(format!(
                    "coverage probe {:?}: expected={} source={} mesh={}",
                    probe.point, probe.expected_inside, source_inside, mesh_inside
                ))
            } else {
                None
            }
        })
        .collect()
}

fn point_in_path(point: [f32; 2], path: &VectorPath) -> bool {
    path.contours
        .iter()
        .filter(|contour| contour.closed)
        .fold(false, |inside, contour| {
            if point_in_contour(point, &contour.points) {
                !inside
            } else {
                inside
            }
        })
}

fn point_in_contour(point: [f32; 2], points: &[[f32; 2]]) -> bool {
    let mut inside = false;
    for (a, b) in points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
    {
        if (a[1] > point[1]) != (b[1] > point[1])
            && point[0] < (b[0] - a[0]) * (point[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            inside = !inside;
        }
    }
    inside
}

fn point_in_mesh(point: [f32; 2], triangles: &[[f32; 2]]) -> bool {
    triangles
        .chunks_exact(3)
        .any(|triangle| point_in_triangle(point, [triangle[0], triangle[1], triangle[2]]))
}
