//! Lucide SVG fixture execution with provenance validation.

use crate::{
    fixture_paths::find_lucide_corpus_root,
    geometry::{format_mesh_summary, validate_mesh},
    reports::failed_stage,
    svg_support::{summarize_paths, tessellate_closed_fills},
    xml_stage::inspect_xml_stage,
    CaseReport, CorpusCase, CorpusStage, StageReport, StageStatus, SvgCase,
};
use std::fs;
use ui_tools::{parse_svg_document_vector_records_from_xml_events, SvgViewportSource};
use xml_tools::XmlSourceId;

/// Runs a Lucide SVG through the shared vector and fill-mesh stages.
///
/// Lucide icons are stroke-oriented. Open paths remain vector evidence until
/// stroke expansion is admitted separately.
pub fn run_svg_case(case: SvgCase) -> CaseReport {
    let mut report = CaseReport {
        id: case.id.to_owned(),
        producer: "svg/lucide".to_owned(),
        selected_stages: CorpusCase::Svg(case).selected_stages().to_vec(),
        stages: Vec::new(),
        diagnostics: Vec::new(),
    };
    let corpus_root = match find_lucide_corpus_root() {
        Some(path) => path,
        None => {
            let message = "Lucide corpus not found; run prepare-lucide-sample.ps1".to_owned();
            report
                .stages
                .push(failed_stage(CorpusStage::Source, &message));
            report.diagnostics.push(message);
            return report;
        }
    };
    let provenance_path = corpus_root.join("provenance.json");
    let provenance = match fs::read_to_string(&provenance_path)
        .ok()
        .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
    {
        Some(value)
            if value.get("provider").and_then(serde_json::Value::as_str) == Some("lucide")
                && value
                    .get("revision")
                    .and_then(serde_json::Value::as_str)
                    .is_some()
                && value
                    .get("count")
                    .and_then(serde_json::Value::as_u64)
                    .is_some() =>
        {
            value
        }
        _ => {
            let message = format!(
                "Lucide provenance missing or invalid: {}",
                provenance_path.display()
            );
            report
                .stages
                .push(failed_stage(CorpusStage::Source, &message));
            report.diagnostics.push(message);
            return report;
        }
    };
    let source_path = corpus_root.join(case.file_name);
    if !source_path.is_file() {
        let message = format!("Lucide asset not found: {}", case.file_name);
        report
            .stages
            .push(failed_stage(CorpusStage::Source, &message));
        report.diagnostics.push(message);
        return report;
    }
    let svg = match fs::read_to_string(&source_path) {
        Ok(svg) => {
            report.stages.push(StageReport {
                stage: CorpusStage::Source,
                status: StageStatus::Ready,
                summary: format!(
                    "file={} bytes={} provider={} revision={} count={}",
                    case.file_name,
                    svg.len(),
                    provenance["provider"].as_str().unwrap_or("unknown"),
                    provenance["revision"].as_str().unwrap_or("unknown"),
                    provenance["count"].as_u64().unwrap_or_default()
                ),
            });
            svg
        }
        Err(error) => {
            let message = format!("SVG source read failed: {error}");
            report
                .stages
                .push(failed_stage(CorpusStage::Source, &message));
            report.diagnostics.push(message);
            return report;
        }
    };
    let xml = match inspect_xml_stage(&svg, XmlSourceId::new(1)) {
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
        SvgViewportSource::Caller([0.0, 0.0, 24.0, 24.0]),
    ) {
        Ok(records) if !records.is_empty() => records,
        Ok(_) => {
            let message = "SVG parser produced no vector paths".to_owned();
            report
                .stages
                .push(failed_stage(CorpusStage::Vector, &message));
            report.diagnostics.push(message);
            return report;
        }
        Err(error) => {
            let message = format!("SVG vector conversion failed: {error}");
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
    let fill_meshes = tessellate_closed_fills(&records, "SVG");
    let validation = validate_mesh(&fill_meshes.triangles);
    report.diagnostics.extend(fill_meshes.diagnostics);
    if fill_meshes.fill_paths == 0 {
        report.stages.push(StageReport {
            stage: CorpusStage::Mesh,
            status: StageStatus::ExpectedFailure,
            summary:
                "no closed fill paths; stroke-only SVG geometry is outside the current mesh scope"
                    .to_owned(),
        });
    } else if !validation.finite || !validation.complete_triangles {
        let message = format!(
            "SVG fill mesh validation failed: closed_paths={} {}",
            fill_meshes.fill_paths,
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
                "closed_paths={} open_paths={} {}",
                fill_meshes.fill_paths,
                paths.len() - fill_meshes.fill_paths,
                format_mesh_summary(&validation)
            ),
        });
    }
    report
}
