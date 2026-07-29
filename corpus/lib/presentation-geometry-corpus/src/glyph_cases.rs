//! Prepared font-outline cases through the structural corpus stages.

use crate::{
    geometry::{format_mesh_summary, validate_mesh},
    reports::failed_stage,
    CaseReport, CorpusCase, CorpusStage, GlyphCase, StageReport, StageStatus,
};
use ui_tools::{
    tessellate_general_fill_with_rule, UiFontFormat, UiFontRasterizer, UiFontSource,
    UiGlyphVectorOptions, VectorFillRule,
};

/// Runs one prepared Inter glyph through the currently observable stages.
pub fn run_glyph_case(case: GlyphCase) -> CaseReport {
    let mut report = CaseReport {
        id: case.id.to_owned(),
        producer: "font-outline/inter".to_owned(),
        selected_stages: CorpusCase::Glyph(case).selected_stages().to_vec(),
        stages: Vec::new(),
        diagnostics: Vec::new(),
    };

    let source = match UiFontSource::from_prepared_corpus("inter", UiFontFormat::Ttf) {
        Ok(source) => {
            report.stages.push(StageReport {
                stage: CorpusStage::Source,
                status: StageStatus::Ready,
                summary: format!(
                    "provider=inter format=ttf file={}",
                    source.identity().source_name
                ),
            });
            source
        }
        Err(error) => {
            report
                .stages
                .push(failed_stage(CorpusStage::Source, &error));
            report.diagnostics.push(error);
            return report;
        }
    };

    let rasterizer = match UiFontRasterizer::from_bytes(source.bytes) {
        Ok(rasterizer) => rasterizer,
        Err(error) => {
            let message = format!("font parse failed: {error}");
            report
                .stages
                .push(failed_stage(CorpusStage::Outline, &message));
            report.diagnostics.push(message);
            return report;
        }
    };
    let outline = match rasterizer.outline(case.character) {
        Ok(outline) => {
            report.stages.push(StageReport {
                stage: CorpusStage::Outline,
                status: StageStatus::Ready,
                summary: format!(
                    "character={:?} contours={} units_per_em={:.0}",
                    case.character,
                    outline.contours.len(),
                    outline.units_per_em
                ),
            });
            outline
        }
        Err(error) => {
            let message = format!("outline extraction failed: {}", error.message);
            report
                .stages
                .push(failed_stage(CorpusStage::Outline, &message));
            report.diagnostics.push(message);
            return report;
        }
    };

    let path = match outline.to_vector_path(UiGlyphVectorOptions::new(1.0, [0.0, 0.0], 0.0005)) {
        Ok(path) => {
            let points: usize = path
                .contours
                .iter()
                .map(|contour| contour.points.len())
                .sum();
            report.stages.push(StageReport {
                stage: CorpusStage::Vector,
                status: StageStatus::Ready,
                summary: format!(
                    "contours={} points={} finite={}",
                    path.contours.len(),
                    points,
                    path.is_finite()
                ),
            });
            path
        }
        Err(error) => {
            let message = format!("vector conversion failed: {}", error.message);
            report
                .stages
                .push(failed_stage(CorpusStage::Vector, &message));
            report.diagnostics.push(message);
            return report;
        }
    };

    match tessellate_general_fill_with_rule(&path, VectorFillRule::EvenOdd) {
        Ok(triangles) => {
            let validation = validate_mesh(&triangles);
            if validation.finite && validation.complete_triangles {
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
