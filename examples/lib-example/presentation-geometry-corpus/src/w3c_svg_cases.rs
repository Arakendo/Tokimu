//! W3C SVG fixture execution and explicit profile-limit classification.

use crate::{
    fixture_paths::{find_w3c_fixture_root, w3c_source_label, w3c_svg_source_path},
    geometry::{format_mesh_summary, validate_mesh},
    reports::failed_stage,
    svg_support::{summarize_paths, tessellate_svg_fills, tessellate_svg_strokes},
    xml_stage::inspect_xml_stage,
    CaseReport, CorpusCase, CorpusStage, StageReport, StageStatus, W3cSvgCase, W3cSvgExpectation,
};
use std::fs;
use ui_tools::{parse_svg_document_vector_records_from_xml_events, SvgViewportSource};
use xml_tools::XmlSourceId;

/// Runs one W3C fixture through the admitted SVG profile and mesh stages.
pub fn run_w3c_svg_case(case: W3cSvgCase) -> CaseReport {
    let mut report = CaseReport {
        id: case.id.to_owned(),
        producer: case.producer().to_owned(),
        selected_stages: CorpusCase::W3cSvg(case).selected_stages().to_vec(),
        stages: Vec::new(),
        diagnostics: Vec::new(),
    };
    let Some(fixture_root) = find_w3c_fixture_root() else {
        let message =
            "W3C SVG fixture not found; run verify-w3c-svg-fixtures.ps1 after acquiring it"
                .to_owned();
        report
            .stages
            .push(failed_stage(CorpusStage::Source, &message));
        report.diagnostics.push(message);
        return report;
    };
    let source_path = w3c_svg_source_path(&fixture_root, case);
    let svg = match fs::read_to_string(&source_path) {
        Ok(svg) => {
            report.stages.push(StageReport {
                stage: CorpusStage::Source,
                status: StageStatus::Ready,
                summary: format!(
                    "file={} bytes={} source={}",
                    case.file_name,
                    svg.len(),
                    w3c_source_label(case)
                ),
            });
            svg
        }
        Err(error) => {
            let message = format!("W3C SVG source read failed: {error}");
            report
                .stages
                .push(failed_stage(CorpusStage::Source, &message));
            report.diagnostics.push(message);
            return report;
        }
    };
    let xml = match inspect_xml_stage(&svg, XmlSourceId::new(2)) {
        Ok(xml) => {
            report.stages.push(StageReport {
                stage: CorpusStage::Xml,
                status: StageStatus::Ready,
                summary: xml.evidence.summary(),
            });
            xml
        }
        Err(message) => {
            report.stages.push(failed_stage(CorpusStage::Xml, &message));
            report.diagnostics.push(message);
            return report;
        }
    };
    debug_assert!(xml.evidence.has_document_element);
    let records = match parse_svg_document_vector_records_from_xml_events(
        &xml.events,
        12,
        // W3C cases are documents, not embedded icon fragments. Their root
        // viewBox is the authored coordinate contract under test.
        SvgViewportSource::DocumentViewBox,
    ) {
        Ok(records) if !records.is_empty() => {
            if case.expectation == W3cSvgExpectation::UnsupportedProfile {
                let message = "W3C SVG fixture was expected to stop at the admitted SVG profile, but it lowered successfully; review the selected-fixture expectation";
                report
                    .stages
                    .push(failed_stage(CorpusStage::Vector, message));
                report.diagnostics.push(message.to_owned());
                return report;
            }
            records
        }
        Ok(_) => {
            let message = "W3C SVG parser produced no vector paths".to_owned();
            report
                .stages
                .push(failed_stage(CorpusStage::Vector, &message));
            report.diagnostics.push(message);
            return report;
        }
        Err(error)
            if matches!(
                case.expectation,
                W3cSvgExpectation::UnsupportedProfile | W3cSvgExpectation::ExpectedInvalidInput
            ) =>
        {
            let label = match case.expectation {
                W3cSvgExpectation::UnsupportedProfile => "expected SVG-profile exclusion",
                W3cSvgExpectation::ExpectedInvalidInput => "expected invalid SVG input",
                W3cSvgExpectation::StructuralPass => unreachable!(),
            };
            report.stages.push(StageReport {
                stage: CorpusStage::Vector,
                status: StageStatus::ExpectedFailure,
                summary: format!("{label}: {error}"),
            });
            report.stages.push(StageReport {
                stage: CorpusStage::Mesh,
                status: StageStatus::ExpectedFailure,
                summary: "not attempted after expected vector-boundary failure".to_owned(),
            });
            return report;
        }
        Err(error) => {
            let message = format!("W3C SVG vector conversion failed: {error}");
            report
                .stages
                .push(failed_stage(CorpusStage::Vector, &message));
            report.diagnostics.push(message);
            return report;
        }
    };
    let paths = records
        .iter()
        .map(|record| record.path.clone())
        .collect::<Vec<_>>();
    report.stages.push(StageReport {
        stage: CorpusStage::Vector,
        status: StageStatus::Ready,
        summary: summarize_paths(case.description, &paths),
    });
    let mut fill_meshes = tessellate_svg_fills(&records, "W3C SVG");
    let stroke_meshes = tessellate_svg_strokes(&records, 1.0 / 480.0, "W3C SVG");
    fill_meshes
        .triangles
        .extend(stroke_meshes.triangles.iter().copied());
    fill_meshes.stroke_paths = stroke_meshes.stroke_paths;
    fill_meshes.diagnostics.extend(stroke_meshes.diagnostics);
    let validation = validate_mesh(&fill_meshes.triangles);
    report.diagnostics.extend(fill_meshes.diagnostics);
    if fill_meshes.fill_paths == 0 && fill_meshes.stroke_paths == 0 {
        report.stages.push(StageReport {
            stage: CorpusStage::Mesh,
            status: StageStatus::ExpectedFailure,
            summary: "no admitted fill or stroke paths produced mesh geometry".to_owned(),
        });
    } else if !validation.finite || !validation.complete_triangles {
        let message = format!(
            "W3C SVG fill mesh validation failed: {}",
            format_mesh_summary(&validation)
        );
        report
            .stages
            .push(failed_stage(CorpusStage::Mesh, &message));
        report.diagnostics.push(message);
    } else {
        report.stages.push(StageReport {
            stage: CorpusStage::Mesh,
            status: StageStatus::Ready,
            summary: format!(
                "closed_paths={} stroke_paths={} open_paths={} {}",
                fill_meshes.fill_paths,
                fill_meshes.stroke_paths,
                paths.len() - fill_meshes.fill_paths,
                format_mesh_summary(&validation)
            ),
        });
    }
    report
}
