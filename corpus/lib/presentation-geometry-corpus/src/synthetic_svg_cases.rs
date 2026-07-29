//! Compact inline SVG fixtures for parser and namespace behavior.

use crate::{
    geometry::{format_mesh_summary, validate_mesh},
    reports::failed_stage,
    xml_stage::inspect_xml_stage,
    CaseReport, CorpusCase, CorpusStage, StageReport, StageStatus, SyntheticSvgCase,
};
use ui_tools::{
    parse_svg_document_vector_records_from_xml_events, tessellate_general_fill_with_rule,
    SvgFillRule, SvgViewportSource, VectorFillRule,
};
use xml_tools::XmlSourceId;

/// Runs a compact inline SVG document through XML, SVG semantic lowering, and
/// structural fill tessellation. These fixtures make namespace/profile
/// behavior observable without importing unrelated provider assets.
pub fn run_synthetic_svg_case(case: SyntheticSvgCase) -> CaseReport {
    let mut report = CaseReport {
        id: case.id.to_owned(),
        producer: "svg/synthetic".to_owned(),
        selected_stages: CorpusCase::SyntheticSvg(case).selected_stages().to_vec(),
        stages: vec![StageReport {
            stage: CorpusStage::Source,
            status: StageStatus::Ready,
            summary: format!("inline fixture bytes={}", case.source.len()),
        }],
        diagnostics: Vec::new(),
    };
    let xml = match inspect_xml_stage(case.source, XmlSourceId::new(3)) {
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
    let records = match parse_svg_document_vector_records_from_xml_events(
        &xml.events,
        12,
        SvgViewportSource::Caller([0.0, 0.0, 24.0, 24.0]),
    ) {
        Ok(records) if records.len() == 1 => records,
        Ok(records) => {
            let message = format!(
                "namespace fixture expected exactly one SVG path after ignoring foreign geometry, found {}",
                records.len()
            );
            report
                .stages
                .push(failed_stage(CorpusStage::Vector, &message));
            report.diagnostics.push(message);
            return report;
        }
        Err(error) => {
            let message = format!("synthetic SVG vector conversion failed: {error}");
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
        summary: format!(
            "{} paths=1 contours={} points={}",
            case.description,
            paths[0].contours.len(),
            paths[0]
                .contours
                .iter()
                .map(|contour| contour.points.len())
                .sum::<usize>()
        ),
    });
    let triangles = match tessellate_general_fill_with_rule(
        &paths[0],
        match records[0].fill_rule {
            SvgFillRule::NonZero => VectorFillRule::NonZero,
            SvgFillRule::EvenOdd => VectorFillRule::EvenOdd,
        },
    ) {
        Ok(triangles) => triangles,
        Err(error) => {
            let message = format!("synthetic SVG fill tessellation failed: {error}");
            report
                .stages
                .push(failed_stage(CorpusStage::Mesh, &message));
            report.diagnostics.push(message);
            return report;
        }
    };
    let validation = validate_mesh(&triangles);
    if validation.finite && validation.complete_triangles && validation.triangle_count > 0 {
        report.stages.push(StageReport {
            stage: CorpusStage::Mesh,
            status: StageStatus::Ready,
            summary: format_mesh_summary(&validation),
        });
    } else {
        let message = format!(
            "synthetic SVG mesh validation failed: {}",
            format_mesh_summary(&validation)
        );
        report
            .stages
            .push(failed_stage(CorpusStage::Mesh, &message));
        report.diagnostics.push(message);
    }
    report
}
