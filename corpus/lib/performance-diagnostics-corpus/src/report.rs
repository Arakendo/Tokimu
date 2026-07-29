use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Serialize;

use crate::PerformanceCaseArtifact;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DiagnosticAttributionArtifact {
    pub scope: String,
    pub identity: String,
    pub cost_scope: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DiagnosticExplanationArtifact {
    pub diagnostic_sequence: u64,
    pub policy_state: String,
    pub attribution: DiagnosticAttributionArtifact,
    pub metric: String,
    pub observed: f64,
    pub budget: f64,
    pub unit: String,
    pub summary: String,
    pub cause_status: String,
    pub next_action: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DiagnosticReportArtifact {
    pub schema: u32,
    pub producer: String,
    pub case_id: String,
    pub explanations: Vec<DiagnosticExplanationArtifact>,
}

pub fn build_diagnostic_report(artifact: &PerformanceCaseArtifact) -> DiagnosticReportArtifact {
    let explanations = artifact
        .diagnostics
        .iter()
        .filter_map(|diagnostic| {
            let metric = diagnostic.metric.as_ref()?;
            let observed = diagnostic.observed?;
            let budget = diagnostic.budget?;
            let unit = diagnostic.unit.as_ref()?;
            let policy_state = match diagnostic.kind.as_str() {
                "performance-budget-exceeded" => "degraded",
                "performance-recovered" => "recovered",
                _ => return None,
            };

            Some(DiagnosticExplanationArtifact {
                diagnostic_sequence: diagnostic.sequence,
                policy_state: policy_state.into(),
                attribution: DiagnosticAttributionArtifact {
                    scope: "subsystem".into(),
                    identity: diagnostic.source.clone(),
                    cost_scope: "collective-subsystem".into(),
                },
                metric: metric.clone(),
                observed,
                budget,
                unit: unit.clone(),
                summary: format!(
                    "{metric} is {policy_state}: {observed:.3} {unit} against {budget:.3} {unit}"
                ),
                cause_status: "not-inferred".into(),
                next_action: format!(
                    "Inspect measurements owned by {}; this report does not infer an individual cause.",
                    diagnostic.source
                ),
            })
        })
        .collect();

    DiagnosticReportArtifact {
        schema: 1,
        producer: "performance-diagnostics-corpus-report".into(),
        case_id: artifact.metadata.case_id.clone(),
        explanations,
    }
}

pub fn write_diagnostic_report(
    output_root: impl AsRef<Path>,
    report: &DiagnosticReportArtifact,
) -> Result<PathBuf, String> {
    let path = output_root
        .as_ref()
        .join("reports")
        .join(format!("{}.json", report.case_id));
    let parent = path
        .parent()
        .ok_or_else(|| format!("report path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    let json = serde_json::to_string_pretty(report)
        .map_err(|error| format!("failed to serialize {}: {error}", path.display()))?;
    fs::write(&path, format!("{json}\n"))
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    Ok(path)
}

pub fn write_all_diagnostic_reports(
    output_root: impl AsRef<Path>,
    artifacts: &[PerformanceCaseArtifact],
) -> Result<Vec<PathBuf>, String> {
    artifacts
        .iter()
        .map(build_diagnostic_report)
        .filter(|report| !report.explanations.is_empty())
        .map(|report| write_diagnostic_report(output_root.as_ref(), &report))
        .collect()
}
