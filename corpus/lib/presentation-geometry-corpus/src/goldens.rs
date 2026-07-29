//! Reviewed structural-report representation and comparison helpers.
//!
//! Golden fixtures deliberately capture the stable corpus contract rather than
//! producer-specific implementation data. Artifact files add the detailed
//! geometry evidence separately.

use crate::CaseReport;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct GoldenSnapshot {
    schema: u32,
    case_id: String,
    producer: String,
    selected_stages: Vec<String>,
    stages: Vec<GoldenStage>,
    diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
struct GoldenStage {
    stage: String,
    status: String,
    summary: String,
}

pub(crate) fn snapshot(report: &CaseReport) -> GoldenSnapshot {
    GoldenSnapshot {
        schema: 1,
        case_id: report.id.clone(),
        producer: report.producer.clone(),
        selected_stages: report
            .selected_stages
            .iter()
            .map(|stage| stage.name().to_owned())
            .collect(),
        stages: report
            .stages
            .iter()
            .map(|stage| GoldenStage {
                stage: stage.stage.name().to_owned(),
                status: stage.status.name().to_owned(),
                summary: stage.summary.clone(),
            })
            .collect(),
        diagnostics: report.diagnostics.clone(),
    }
}

pub(crate) fn first_difference(expected: &str, actual: &str) -> String {
    let expected_lines = expected.lines().collect::<Vec<_>>();
    let actual_lines = actual.lines().collect::<Vec<_>>();
    let line_count = expected_lines.len().max(actual_lines.len());
    for index in 0..line_count {
        let expected_line = expected_lines.get(index).copied().unwrap_or("<missing>");
        let actual_line = actual_lines.get(index).copied().unwrap_or("<missing>");
        if expected_line != actual_line {
            return format!(
                "first difference at line {}\n  expected: {}\n  actual:   {}",
                index + 1,
                expected_line,
                actual_line
            );
        }
    }
    "content differs despite matching lines".to_owned()
}
