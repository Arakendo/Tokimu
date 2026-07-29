//! Semantic UI surface corpus execution.

use crate::{
    geometry::{format_mesh_summary, validate_mesh},
    reports::failed_stage,
    CaseReport, CorpusCase, CorpusStage, StageReport, StageStatus, UiCase,
};
use ui_tools::{
    lower_surface_to_vector, tessellate_general_fill_with_rule, UiRect, UiSurfaceCommand,
    UiSurfaceRole, UiTheme, VectorFillRule,
};

/// Runs a semantic UI surface through its public vector-lowering adapter.
pub fn run_ui_case(case: UiCase) -> CaseReport {
    let mut report = CaseReport {
        id: case.id.to_owned(),
        producer: "ui/surface".to_owned(),
        selected_stages: CorpusCase::Ui(case).selected_stages().to_vec(),
        stages: vec![StageReport {
            stage: CorpusStage::Source,
            status: StageStatus::Ready,
            summary: case.description.to_owned(),
        }],
        diagnostics: Vec::new(),
    };
    let theme = UiTheme::default();
    let command = UiSurfaceCommand {
        rect: UiRect::new([0.0, 0.0], [0.72, 0.48]),
        style: theme.surface(UiSurfaceRole::Panel),
        clip: None,
    };
    let layers = lower_surface_to_vector(&command);
    report.stages.push(StageReport {
        stage: CorpusStage::Vector,
        status: StageStatus::Ready,
        summary: format!(
            "layers={} contours={} points={}",
            layers.len(),
            layers
                .iter()
                .map(|layer| layer.path.contours.len())
                .sum::<usize>(),
            layers
                .iter()
                .flat_map(|layer| layer.path.contours.iter())
                .map(|contour| contour.points.len())
                .sum::<usize>()
        ),
    });
    let mut triangles = Vec::new();
    for layer in &layers {
        match tessellate_general_fill_with_rule(&layer.path, VectorFillRule::EvenOdd) {
            Ok(mut layer_triangles) => triangles.append(&mut layer_triangles),
            Err(error) => report
                .diagnostics
                .push(format!("UI layer tessellation failed: {error}")),
        }
    }
    let validation = validate_mesh(&triangles);
    if validation.finite && validation.complete_triangles && report.diagnostics.is_empty() {
        report.stages.push(StageReport {
            stage: CorpusStage::Mesh,
            status: StageStatus::Ready,
            summary: format_mesh_summary(&validation),
        });
    } else {
        let message = format!(
            "UI mesh validation failed: {}",
            format_mesh_summary(&validation)
        );
        report
            .stages
            .push(failed_stage(CorpusStage::Mesh, &message));
        report.diagnostics.push(message);
    }
    report
}
