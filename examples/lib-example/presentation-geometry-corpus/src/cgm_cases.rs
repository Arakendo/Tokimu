//! WebCGM producer cases backed by the incubating `cgm-corpus` adapter.
//!
//! The corpus harness deliberately consumes only the adapter's inspection and
//! provider-neutral vector results. Binary parsing, CGM descriptors, and
//! source-state semantics remain in `cgm-corpus`.

use std::path::PathBuf;

use cgm_corpus::{
    inspect_binary_cgm_file, lower_picture_primitives, CgmClipIndicator, CgmError, CgmInspection,
    CgmScalingMode, DecodeLimits,
};

use crate::{
    cases::CgmExpectation, reports::failed_stage, CaseReport, CgmCase, CorpusCase, CorpusStage,
    StageReport, StageStatus,
};

pub fn run_cgm_case(case: CgmCase) -> CaseReport {
    let mut report = CaseReport {
        id: case.id.to_owned(),
        producer: "cgm/webcgm".to_owned(),
        selected_stages: CorpusCase::Cgm(case).selected_stages().to_vec(),
        stages: vec![StageReport {
            stage: CorpusStage::Source,
            status: StageStatus::Ready,
            summary: format!("{} ({})", case.description, case.file_name),
        }],
        diagnostics: Vec::new(),
    };

    let inspection =
        match inspect_binary_cgm_file(fixture_path(case.file_name), DecodeLimits::default()) {
            Ok(inspection) => inspection,
            Err(error) => {
                let message = format!("CGM decode failed: {error}");
                report
                    .stages
                    .push(failed_stage(CorpusStage::Vector, &message));
                report.diagnostics.push(message);
                return report;
            }
        };
    report.stages[0].summary = source_summary(case, &inspection);
    if case.expectation == CgmExpectation::SourceOnly {
        return report;
    }
    let Some(picture) = inspection.pictures.first() else {
        let message = "CGM source contains no picture to lower".to_owned();
        report
            .stages
            .push(failed_stage(CorpusStage::Vector, &message));
        report.diagnostics.push(message);
        return report;
    };
    match lower_picture_primitives(picture) {
        Ok(primitives) if !primitives.is_empty() => {
            let contour_count = primitives
                .iter()
                .map(|primitive| primitive.path.contours.len())
                .sum::<usize>();
            let point_count = primitives
                .iter()
                .flat_map(|primitive| &primitive.path.contours)
                .map(|contour| contour.points.len())
                .sum::<usize>();
            let finite = primitives
                .iter()
                .all(|primitive| primitive.path.is_finite());
            if finite {
                report.stages.push(StageReport {
                    stage: CorpusStage::Vector,
                    status: StageStatus::Ready,
                    summary: format!(
                        "primitives={} contours={} points={} finite=true",
                        primitives.len(),
                        contour_count,
                        point_count
                    ),
                });
            } else {
                let message = "CGM lowering produced non-finite vector geometry".to_owned();
                report
                    .stages
                    .push(failed_stage(CorpusStage::Vector, &message));
                report.diagnostics.push(message);
            }
        }
        Ok(_) => {
            let message = "CGM source lowered no primitives".to_owned();
            report
                .stages
                .push(failed_stage(CorpusStage::Vector, &message));
            report.diagnostics.push(message);
        }
        Err(CgmError::UnsupportedPrimitiveLowering { kind, .. })
            if matches!(
                case.expectation,
                CgmExpectation::ExpectedUnsupportedLowering {
                    kind: expected_kind
                } if kind == expected_kind
            ) =>
        {
            report.stages.push(StageReport {
                stage: CorpusStage::Vector,
                status: StageStatus::ExpectedFailure,
                summary: format!("expected unsupported CGM primitive lowering: {kind}"),
            });
        }
        Err(error) => {
            let message = format!("CGM vector lowering failed: {error}");
            report
                .stages
                .push(failed_stage(CorpusStage::Vector, &message));
            report.diagnostics.push(message);
        }
    }
    report
}

/// Summarizes source-format observations without interpreting them as
/// provider-neutral paint or clipping semantics.
fn source_summary(case: CgmCase, inspection: &CgmInspection) -> String {
    let primitive_count = inspection
        .pictures
        .iter()
        .map(|picture| picture.primitives.len())
        .sum::<usize>();
    let attribute_count = inspection
        .pictures
        .iter()
        .map(|picture| picture.attributes.len())
        .sum::<usize>();
    let stateful_primitive_count = inspection
        .pictures
        .iter()
        .flat_map(|picture| &picture.primitives)
        .filter(|primitive| !primitive.state.is_default() || !primitive.controls.is_default())
        .count();
    let clip_on_count = inspection
        .pictures
        .iter()
        .flat_map(|picture| &picture.primitives)
        .filter(|primitive| primitive.controls.clip_indicator == Some(CgmClipIndicator::On))
        .count();
    let clip_rectangle_count = inspection
        .pictures
        .iter()
        .flat_map(|picture| &picture.primitives)
        .filter(|primitive| primitive.controls.clip_rectangle.is_some())
        .count();
    let vdc_extent_picture_count = inspection
        .pictures
        .iter()
        .filter(|picture| picture.descriptor.vdc_extent.is_some())
        .count();
    let metric_scaling_picture_count = inspection
        .pictures
        .iter()
        .filter(|picture| picture.descriptor.scaling_mode == CgmScalingMode::Metric)
        .count();

    format!(
        "{} ({}) elements={} pictures={} primitives={} attributes={} stateful-primitives={} clip-on-primitives={} clip-rectangle-primitives={} vdc-extent-pictures={} metric-scaling-pictures={}",
        case.description,
        case.file_name,
        inspection.elements.len(),
        inspection.pictures.len(),
        primitive_count,
        attribute_count,
        stateful_primitive_count,
        clip_on_count,
        clip_rectangle_count,
        vdc_extent_picture_count,
        metric_scaling_picture_count,
    )
}

fn fixture_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../third-party/fixtures/webcgm-test-suite/upstream/static10")
        .join(file_name)
}
