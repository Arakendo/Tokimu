use std::{
    fs,
    path::{Path, PathBuf},
};

use tokimu_assets::{AssetLifecycleKind, AssetLifecycleObservation, AssetStore};
use tokimu_core::{
    DiagnosticKind, DiagnosticRecord, DiagnosticSeverity, Diagnostics, PerformanceBudget,
    PerformanceMonitor, PerformanceUnit,
};
use tokimu_render::RenderStats;

use crate::cases::PerformanceCaseMeasurement;
use crate::{
    BudgetArtifact, CaseExpectation, DiagnosticArtifact, DiagnosticTransition,
    NumericSummaryArtifact, ObservationArtifact, PerformanceCase, PerformanceCaseArtifact,
    PerformanceCorpusMetadata, RenderFrameArtifact, ResourceLifecycleArtifact,
    ResourceLifecycleSummaryArtifact,
};

const MONITOR_ALGORITHM: &str = "tokimu-core-performance-monitor-v1";
const RENDER_COUNTER_POLICY: &str = "render-frame-binding-allocation-pressure-v1";
const MESH_UPLOAD_POLICY: &str = "render-frame-mesh-upload-pressure-v1";

pub fn run_all_cases() -> Vec<PerformanceCaseArtifact> {
    crate::all_cases().iter().map(run_case).collect()
}

pub fn run_case(case: &PerformanceCase) -> PerformanceCaseArtifact {
    let mut diagnostics = Diagnostics::with_capacity(case.diagnostic_capacity);
    let mut observations = Vec::new();
    let mut render_frames = Vec::new();
    let mut resource_lifecycle = Vec::new();
    let mut budget = None;
    let mut renderer_counter_policy = None;

    match case.measurement {
        PerformanceCaseMeasurement::Observations {
            source,
            metric,
            limit,
            unit,
            required_consecutive_violations,
            values,
        } => {
            let performance_budget = PerformanceBudget::new(source, metric, limit, unit)
                .with_required_consecutive_violations(required_consecutive_violations);
            budget = Some(budget_artifact(&performance_budget));
            let mut monitor = PerformanceMonitor::new(performance_budget);

            for (sequence, value) in values.iter().copied().enumerate() {
                observations.push(ObservationArtifact { sequence, value });
                monitor.observe(value, &mut diagnostics);
            }
        }
        PerformanceCaseMeasurement::RenderBindingAllocations {
            required_consecutive_violations,
            frames,
        } => {
            let performance_budget = PerformanceBudget::new(
                "corpus.renderer",
                "frame binding allocations after warm-up",
                0.0,
                PerformanceUnit::Count,
            )
            .with_required_consecutive_violations(required_consecutive_violations);
            budget = Some(budget_artifact(&performance_budget));
            renderer_counter_policy = Some(RENDER_COUNTER_POLICY.into());
            let mut monitor = PerformanceMonitor::new(performance_budget);

            for (sequence, stats) in frames.iter().copied().enumerate() {
                render_frames.push(render_frame_artifact(sequence, stats));
                if sequence > 0 {
                    let value = f64::from(stats.frame.binding_allocations);
                    observations.push(ObservationArtifact { sequence, value });
                    monitor.observe(value, &mut diagnostics);
                }
            }
        }
        PerformanceCaseMeasurement::RenderMeshUploads {
            required_consecutive_violations,
            frames,
        } => {
            let performance_budget = PerformanceBudget::new(
                "corpus.renderer",
                "frame mesh uploads after warm-up",
                0.0,
                PerformanceUnit::Count,
            )
            .with_required_consecutive_violations(required_consecutive_violations);
            budget = Some(budget_artifact(&performance_budget));
            renderer_counter_policy = Some(MESH_UPLOAD_POLICY.into());
            let mut monitor = PerformanceMonitor::new(performance_budget);

            for (sequence, stats) in frames.iter().copied().enumerate() {
                render_frames.push(render_frame_artifact(sequence, stats));
                if sequence > 0 {
                    let value = f64::from(stats.frame.mesh_uploads);
                    observations.push(ObservationArtifact { sequence, value });
                    monitor.observe(value, &mut diagnostics);
                }
            }
        }
        PerformanceCaseMeasurement::AssetLifecycle => {
            let mut store = AssetStore::default();
            let (handle, allocated) =
                store.allocate_with_source_observed::<Vec<u8>, _>("corpus/generated.asset");
            let prepared = store
                .mark_prepared(handle)
                .expect("allocated corpus asset must prepare");
            let replaced = store
                .mark_replaced(handle)
                .expect("allocated corpus asset must replace");
            let prepared_again = store
                .mark_prepared(handle)
                .expect("replacement corpus asset must prepare");
            let released = store
                .release(handle)
                .expect("allocated corpus asset must release");
            resource_lifecycle.extend(
                [allocated, prepared, replaced, prepared_again, released]
                    .iter()
                    .map(resource_lifecycle_artifact),
            );
        }
        PerformanceCaseMeasurement::Unsupported { .. } => {}
    }

    let diagnostic_artifacts = diagnostics
        .records()
        .iter()
        .map(diagnostic_artifact)
        .collect::<Vec<_>>();
    let transitions = diagnostics
        .records()
        .iter()
        .filter_map(|record| match record.kind {
            DiagnosticKind::PerformanceBudgetExceeded => Some(DiagnosticTransition::BudgetExceeded),
            DiagnosticKind::PerformanceRecovered => Some(DiagnosticTransition::Recovered),
            DiagnosticKind::Message | DiagnosticKind::BackendError => None,
        })
        .collect::<Vec<_>>();
    let actual = actual_outcome(case, &transitions, diagnostics.dropped_records());
    let numeric_summary = numeric_summary(&observations);
    let resource_lifecycle_summary = resource_lifecycle_summary(&resource_lifecycle);

    PerformanceCaseArtifact {
        metadata: PerformanceCorpusMetadata {
            schema: 2,
            producer: "performance-diagnostics-corpus".into(),
            case_id: case.id.into(),
            build_profile: env!("TOKIMU_CORPUS_PROFILE").into(),
            target: env!("TOKIMU_CORPUS_TARGET").into(),
            workload_revision: case.workload_revision.into(),
            monitor: MONITOR_ALGORITHM.into(),
            renderer_counter_policy,
        },
        description: case.description.into(),
        measurement: case.support(),
        expected: case.expected,
        actual,
        budget,
        observations,
        render_frames,
        resource_lifecycle,
        numeric_summary,
        resource_lifecycle_summary,
        transitions,
        diagnostics: diagnostic_artifacts,
        diagnostic_capacity: case.diagnostic_capacity,
        dropped_records: diagnostics.dropped_records(),
    }
}

pub fn to_pretty_json(artifact: &PerformanceCaseArtifact) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(artifact)
}

pub fn write_case_artifact(
    output_root: impl AsRef<Path>,
    artifact: &PerformanceCaseArtifact,
) -> Result<PathBuf, String> {
    let path = output_root
        .as_ref()
        .join(format!("{}.json", artifact.metadata.case_id));
    let parent = path
        .parent()
        .ok_or_else(|| format!("artifact path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    let json = to_pretty_json(artifact)
        .map_err(|error| format!("failed to serialize {}: {error}", path.display()))?;
    fs::write(&path, format!("{json}\n"))
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    Ok(path)
}

pub fn write_all_artifacts(output_root: impl AsRef<Path>) -> Result<Vec<PathBuf>, String> {
    run_all_cases()
        .iter()
        .map(|artifact| write_case_artifact(output_root.as_ref(), artifact))
        .collect()
}

fn budget_artifact(budget: &PerformanceBudget) -> BudgetArtifact {
    BudgetArtifact {
        source: budget.source.clone(),
        metric: budget.metric.clone(),
        limit: budget.limit,
        unit: unit_name(budget.unit).into(),
        required_consecutive_violations: budget.required_consecutive_violations,
    }
}

fn diagnostic_artifact(record: &DiagnosticRecord) -> DiagnosticArtifact {
    DiagnosticArtifact {
        sequence: record.sequence(),
        severity: match record.severity {
            DiagnosticSeverity::Info => "info",
            DiagnosticSeverity::Warning => "warning",
            DiagnosticSeverity::Error => "error",
        }
        .into(),
        kind: match record.kind {
            DiagnosticKind::Message => "message",
            DiagnosticKind::BackendError => "backend-error",
            DiagnosticKind::PerformanceBudgetExceeded => "performance-budget-exceeded",
            DiagnosticKind::PerformanceRecovered => "performance-recovered",
        }
        .into(),
        source: record.source.clone(),
        message: record.message.clone(),
        metric: record
            .performance
            .as_ref()
            .map(|performance| performance.metric.clone()),
        observed: record
            .performance
            .as_ref()
            .map(|performance| performance.observed),
        budget: record
            .performance
            .as_ref()
            .map(|performance| performance.budget),
        unit: record
            .performance
            .as_ref()
            .map(|performance| unit_name(performance.unit).into()),
    }
}

fn render_frame_artifact(sequence: usize, stats: RenderStats) -> RenderFrameArtifact {
    RenderFrameArtifact {
        sequence,
        draw_calls: stats.frame.draw_calls,
        submit_calls: stats.frame.submit_calls,
        binding_allocations: stats.frame.binding_allocations,
        uniform_buffer_writes: stats.frame.uniform_buffer_writes,
        mesh_uploads: stats.frame.mesh_uploads,
        mesh_replacements: stats.frame.mesh_replacements,
        texture_allocations: stats.frame.texture_allocations,
        texture_replacements: stats.frame.texture_replacements,
        texture_writes: stats.frame.texture_writes,
        lifetime_binding_allocations: stats.lifetime.binding_allocations,
        lifetime_uniform_buffer_writes: stats.lifetime.uniform_buffer_writes,
        lifetime_mesh_uploads: stats.lifetime.mesh_uploads,
        lifetime_mesh_replacements: stats.lifetime.mesh_replacements,
        lifetime_texture_allocations: stats.lifetime.texture_allocations,
        lifetime_texture_replacements: stats.lifetime.texture_replacements,
        lifetime_texture_writes: stats.lifetime.texture_writes,
    }
}

fn resource_lifecycle_artifact(
    observation: &AssetLifecycleObservation,
) -> ResourceLifecycleArtifact {
    ResourceLifecycleArtifact {
        sequence: observation.sequence,
        resource_kind: "asset".into(),
        resource_id: observation.asset_id.0,
        generation: observation.generation,
        transition: match observation.kind {
            AssetLifecycleKind::Allocated => "allocated",
            AssetLifecycleKind::Prepared => "prepared",
            AssetLifecycleKind::Replaced => "replaced",
            AssetLifecycleKind::Released => "released",
        }
        .into(),
        source: observation.source.clone(),
        measured_bytes: None,
        measured_duration_ms: None,
    }
}

fn numeric_summary(observations: &[ObservationArtifact]) -> Option<NumericSummaryArtifact> {
    let first = observations.first()?;
    let total = observations
        .iter()
        .map(|observation| observation.value)
        .sum::<f64>();
    let peak = observations
        .iter()
        .skip(1)
        .fold(first.value, |peak, observation| peak.max(observation.value));

    Some(NumericSummaryArtifact {
        sample_count: observations.len(),
        window_size: observations.len(),
        cadence: "one sample per controlled case step".into(),
        reset_behavior: "per-case".into(),
        last: observations.last().expect("first observation exists").value,
        total,
        average: total / observations.len() as f64,
        peak,
    })
}

fn resource_lifecycle_summary(
    observations: &[ResourceLifecycleArtifact],
) -> Option<ResourceLifecycleSummaryArtifact> {
    if observations.is_empty() {
        return None;
    }

    let mut allocated = 0;
    let mut prepared = 0;
    let mut replaced = 0;
    let mut released = 0;
    for observation in observations {
        match observation.transition.as_str() {
            "allocated" => allocated += 1,
            "prepared" => prepared += 1,
            "replaced" => replaced += 1,
            "released" => released += 1,
            _ => {}
        }
    }

    Some(ResourceLifecycleSummaryArtifact {
        event_count: observations.len(),
        reset_behavior: "per-case".into(),
        allocated,
        prepared,
        replaced,
        released,
        final_active_resources: allocated.saturating_sub(released),
        last_generation: observations
            .last()
            .map(|observation| observation.generation),
    })
}

fn actual_outcome(
    case: &PerformanceCase,
    transitions: &[DiagnosticTransition],
    dropped_records: u64,
) -> CaseExpectation {
    if matches!(
        case.measurement,
        PerformanceCaseMeasurement::Unsupported { .. }
    ) {
        return CaseExpectation::Unsupported;
    }
    if dropped_records > 0 {
        return CaseExpectation::BoundedOverflow;
    }
    match transitions {
        [] => CaseExpectation::Silence,
        [DiagnosticTransition::BudgetExceeded] => CaseExpectation::Warning,
        [DiagnosticTransition::BudgetExceeded, DiagnosticTransition::Recovered] => {
            CaseExpectation::WarningThenRecovery
        }
        _ => CaseExpectation::Warning,
    }
}

fn unit_name(unit: PerformanceUnit) -> &'static str {
    match unit {
        PerformanceUnit::Seconds => "seconds",
        PerformanceUnit::Milliseconds => "milliseconds",
        PerformanceUnit::Count => "count",
    }
}
