//! Deterministic corpus evidence for Tokimu performance diagnostics.
//!
//! Cases use controlled counts rather than machine wall time. This keeps
//! diagnostic transitions reproducible while real applications remain free to
//! feed provider-measured durations into the same kernel contracts.

mod artifacts;
mod cases;
mod report;
mod runner;

pub use artifacts::{
    BudgetArtifact, CaseExpectation, DiagnosticArtifact, DiagnosticTransition, MeasurementSupport,
    NumericSummaryArtifact, ObservationArtifact, PerformanceCaseArtifact,
    PerformanceCorpusMetadata, RenderFrameArtifact, ResourceLifecycleArtifact,
    ResourceLifecycleSummaryArtifact,
};
pub use cases::{all_cases, find_case, PerformanceCase};
pub use report::{
    build_diagnostic_report, write_all_diagnostic_reports, write_diagnostic_report,
    DiagnosticAttributionArtifact, DiagnosticExplanationArtifact, DiagnosticReportArtifact,
};
pub use runner::{
    run_all_cases, run_case, to_pretty_json, write_all_artifacts, write_case_artifact,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_cases_match_their_expected_transitions() {
        for case in all_cases() {
            let first = run_case(case);
            let second = run_case(case);

            assert_eq!(first, second, "case {} was not deterministic", case.id);
            assert_eq!(
                first.actual, first.expected,
                "case {} produced an unexpected outcome",
                case.id
            );
        }
    }

    #[test]
    fn stable_resources_remain_quiet_after_warmup() {
        let artifact = run_case(find_case("renderer/stable-resources").unwrap());

        assert_eq!(artifact.actual, CaseExpectation::Silence);
        assert_eq!(
            artifact
                .render_frames
                .iter()
                .skip(1)
                .map(|frame| frame.binding_allocations)
                .collect::<Vec<_>>(),
            [0, 0, 0]
        );
        assert_eq!(
            artifact
                .render_frames
                .iter()
                .skip(1)
                .map(|frame| frame.mesh_uploads)
                .collect::<Vec<_>>(),
            [0, 0, 0]
        );
    }

    #[test]
    fn repeated_per_frame_allocation_is_detected() {
        let artifact = run_case(find_case("renderer/repeated-binding-allocation").unwrap());

        assert_eq!(artifact.actual, CaseExpectation::Warning);
        assert_eq!(artifact.transitions, [DiagnosticTransition::BudgetExceeded]);
    }

    #[test]
    fn repeated_mesh_upload_is_detected() {
        let artifact = run_case(find_case("renderer/repeated-mesh-upload").unwrap());

        assert_eq!(artifact.actual, CaseExpectation::Warning);
        assert_eq!(artifact.transitions, [DiagnosticTransition::BudgetExceeded]);
    }

    #[test]
    fn bounded_capture_reports_overflow_without_unbounded_history() {
        let artifact = run_case(find_case("diagnostics/bounded-overflow").unwrap());

        assert_eq!(artifact.actual, CaseExpectation::BoundedOverflow);
        assert_eq!(artifact.diagnostic_capacity, 2);
        assert_eq!(artifact.diagnostics.len(), 2);
        assert_eq!(artifact.dropped_records, 3);
    }

    #[test]
    fn artifact_json_records_reproducibility_metadata() {
        let artifact = run_case(find_case("diagnostics/recovery").unwrap());
        let json = to_pretty_json(&artifact).unwrap();

        assert!(json.contains("\"schema\": 2"));
        assert!(json.contains("\"build_profile\""));
        assert!(json.contains("\"target\""));
        assert!(json.contains("\"workload_revision\""));
        assert!(json.contains("\"monitor\""));
    }

    #[test]
    fn asset_lifecycle_preserves_identity_generation_and_missing_measurements() {
        let artifact = run_case(find_case("assets/registered-lifecycle").unwrap());

        assert_eq!(
            artifact
                .resource_lifecycle
                .iter()
                .map(|event| event.transition.as_str())
                .collect::<Vec<_>>(),
            ["allocated", "prepared", "replaced", "prepared", "released"]
        );
        assert!(artifact
            .resource_lifecycle
            .iter()
            .all(|event| event.resource_id == 0));
        assert_eq!(
            artifact
                .resource_lifecycle
                .iter()
                .map(|event| event.generation)
                .collect::<Vec<_>>(),
            [0, 0, 1, 1, 1]
        );
        assert!(artifact
            .resource_lifecycle
            .iter()
            .all(|event| event.measured_bytes.is_none() && event.measured_duration_ms.is_none()));
        assert_eq!(
            artifact.resource_lifecycle_summary,
            Some(ResourceLifecycleSummaryArtifact {
                event_count: 5,
                reset_behavior: "per-case".into(),
                allocated: 1,
                prepared: 2,
                replaced: 1,
                released: 1,
                final_active_resources: 0,
                last_generation: Some(1),
            })
        );
        assert!(artifact.numeric_summary.is_none());
    }

    #[test]
    fn numeric_summary_is_bounded_to_the_case_and_preserves_raw_samples() {
        let artifact = run_case(find_case("diagnostics/recovery").unwrap());
        let summary = artifact.numeric_summary.unwrap();

        assert_eq!(artifact.observations.len(), 3);
        assert_eq!(summary.sample_count, 3);
        assert_eq!(summary.window_size, 3);
        assert_eq!(summary.cadence, "one sample per controlled case step");
        assert_eq!(summary.reset_behavior, "per-case");
        assert_eq!(summary.last, 2.0);
        assert_eq!(summary.total, 11.0);
        assert_eq!(summary.average, 11.0 / 3.0);
        assert_eq!(summary.peak, 5.0);
        assert!(artifact.resource_lifecycle_summary.is_none());
    }

    #[test]
    fn report_consumes_structured_fields_without_parsing_human_messages() {
        let mut artifact = run_case(find_case("diagnostics/recovery").unwrap());
        let original = build_diagnostic_report(&artifact);

        for diagnostic in &mut artifact.diagnostics {
            diagnostic.message = "deliberately unrelated prose".into();
        }
        let changed_message = build_diagnostic_report(&artifact);

        assert_eq!(original, changed_message);
        assert_eq!(original.explanations.len(), 2);
        assert_eq!(
            original
                .explanations
                .iter()
                .map(|explanation| explanation.policy_state.as_str())
                .collect::<Vec<_>>(),
            ["degraded", "recovered"]
        );
        assert!(original.explanations.iter().all(|explanation| {
            explanation.attribution.scope == "subsystem"
                && explanation.attribution.identity == "corpus.synthetic"
                && explanation.attribution.cost_scope == "collective-subsystem"
                && explanation.cause_status == "not-inferred"
        }));
    }
}
