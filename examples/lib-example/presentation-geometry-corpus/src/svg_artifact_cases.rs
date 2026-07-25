//! Source acquisition for SVG artifact producers.
//!
//! The shared record serializer remains separate because Lucide, synthetic,
//! and W3C fixtures all converge on the same XML/vector/mesh evidence shape.

use crate::{
    fixture_paths::{
        find_lucide_corpus_root, find_w3c_fixture_root, w3c_source_label, w3c_svg_source_path,
    },
    write_svg_record_artifacts, SvgCase, SyntheticSvgCase, W3cSvgCase, W3cSvgExpectation,
};
use std::{fs, path::PathBuf};
use ui_tools::{parse_svg_document_vector_records_from_xml_events, SvgViewportSource};
use xml_tools::XmlSourceId;

/// Writes structural artifacts for a prepared Lucide SVG case.
pub fn write_svg_artifacts(case: SvgCase) -> Result<PathBuf, String> {
    let corpus_root = find_lucide_corpus_root()
        .ok_or_else(|| "Lucide corpus not found; run prepare-lucide-sample.ps1".to_owned())?;
    let source_path = corpus_root.join(case.file_name);
    let source = fs::read_to_string(&source_path)
        .map_err(|error| format!("read Lucide source {}: {error}", source_path.display()))?;
    let xml = crate::xml_stage::inspect_xml_stage(&source, XmlSourceId::new(1))
        .map_err(|error| format!("Lucide XML inspection failed: {error}"))?;
    let records = parse_svg_document_vector_records_from_xml_events(
        &xml.events,
        12,
        SvgViewportSource::Caller([0.0, 0.0, 24.0, 24.0]),
    )
    .map_err(|error| format!("Lucide vector conversion failed: {error}"))?;
    write_svg_record_artifacts(
        case.id,
        "svg/lucide",
        format!("Lucide/{}", case.file_name),
        source,
        xml,
        records,
    )
}

/// Writes structural artifacts for an inline SVG semantic fixture.
pub fn write_synthetic_svg_artifacts(case: SyntheticSvgCase) -> Result<PathBuf, String> {
    let source = case.source.to_owned();
    let xml = crate::xml_stage::inspect_xml_stage(&source, XmlSourceId::new(3))
        .map_err(|error| format!("synthetic SVG XML inspection failed: {error}"))?;
    let records = parse_svg_document_vector_records_from_xml_events(
        &xml.events,
        12,
        SvgViewportSource::Caller([0.0, 0.0, 24.0, 24.0]),
    )
    .map_err(|error| format!("synthetic SVG vector conversion failed: {error}"))?;
    write_svg_record_artifacts(
        case.id,
        "svg/synthetic",
        "inline namespace fixture".to_owned(),
        source,
        xml,
        records,
    )
}

/// Writes structural artifacts for one admitted W3C SVG case. This deliberately
/// stops at source, XML, vector, and CPU mesh evidence; it does not invoke the
/// W3C browser harness or capture a backend framebuffer.
pub fn write_w3c_artifacts(case: W3cSvgCase) -> Result<PathBuf, String> {
    let fixture_root = find_w3c_fixture_root()
        .ok_or_else(|| "W3C SVG fixture not found; run verify-w3c-svg-fixtures.ps1".to_owned())?;
    let source_path = w3c_svg_source_path(&fixture_root, case);
    let source = fs::read_to_string(&source_path)
        .map_err(|error| format!("read W3C source {}: {error}", source_path.display()))?;
    let xml = crate::xml_stage::inspect_xml_stage(&source, XmlSourceId::new(2))
        .map_err(|error| format!("W3C XML inspection failed: {error}"))?;
    let records = match parse_svg_document_vector_records_from_xml_events(
        &xml.events,
        12,
        SvgViewportSource::Caller([0.0, 0.0, 480.0, 360.0]),
    ) {
        Ok(records) => records,
        Err(error) if case.expectation == W3cSvgExpectation::UnsupportedProfile => {
            return crate::svg_artifact_cleanup::write_svg_profile_exclusion_artifacts(
                case.id,
                case.producer(),
                w3c_source_label(case),
                source,
                &xml.evidence,
                error.to_string(),
            );
        }
        Err(error) => return Err(format!("W3C vector conversion failed: {error}")),
    };
    if case.expectation == W3cSvgExpectation::UnsupportedProfile {
        return Err(
            "W3C SVG fixture unexpectedly lowered despite an unsupported-profile expectation"
                .to_owned(),
        );
    }
    write_svg_record_artifacts(
        case.id,
        case.producer(),
        w3c_source_label(case),
        source,
        xml,
        records,
    )
}
