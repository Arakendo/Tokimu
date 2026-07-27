//! Artifact cleanup and explicit exclusion evidence for unsupported SVG profiles.

use crate::{
    artifact_io::write_json,
    artifacts::{
        ArtifactAlgorithms, ArtifactEnvelope, GraphArtifact, GraphEdge, GraphNode,
        SvgProfileExclusionArtifact, XmlArtifact,
    },
    evidence::fnv1a64,
    xml_stage::XmlStageEvidence,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Writes durable source/XML evidence for an expected SVG boundary failure.
/// The expectation label keeps malformed input distinct from unsupported
/// profile content without inventing vector or mesh artifacts.
pub(crate) fn write_svg_expected_failure_artifacts(
    case_id: &str,
    producer: &str,
    source_label: String,
    source: String,
    xml: &XmlStageEvidence,
    diagnostic: String,
    expectation: &str,
    artifact_name: &str,
) -> Result<PathBuf, String> {
    let root = PathBuf::from("target/presentation-geometry-corpus").join(case_id);
    fs::create_dir_all(&root).map_err(|error| format!("create artifact directory: {error}"))?;
    for artifact in [
        "vector.json",
        "mesh.json",
        "mesh-fingerprint.json",
        "contours.json",
        "mesh-view.bmp",
    ] {
        remove_stale_artifact(&root, artifact)?;
    }
    let input_hash = format!("fnv1a64:{:016x}", fnv1a64(source.as_bytes(), '\0'));
    let algorithms = ArtifactAlgorithms {
        flatten: format!("not-run:{expectation}"),
        tessellator: format!("not-run:{expectation}"),
        fill_rule: format!("not-run:{expectation}"),
    };
    let envelope = |artifact: &str| ArtifactEnvelope {
        schema: 1,
        artifact: artifact.to_owned(),
        producer: producer.to_owned(),
        case_id: case_id.to_owned(),
        input_hash: input_hash.clone(),
        source: source_label.clone(),
        algorithms: algorithms.clone(),
    };
    let xml_artifact = XmlArtifact {
        metadata: envelope("xml"),
        event_count: xml.event_count,
        start_elements: xml.start_elements,
        end_elements: xml.end_elements,
        text_nodes: xml.text_nodes,
        comments: xml.comments,
        processing_instructions: xml.processing_instructions,
        document_roots: xml.document_roots,
        has_document_element: xml.has_document_element,
    };
    let profile_artifact = SvgProfileExclusionArtifact {
        metadata: envelope(artifact_name),
        expectation: expectation.to_owned(),
        diagnostic,
    };
    let failure_artifact = format!("{artifact_name}.json");
    let graph_artifact = GraphArtifact {
        metadata: envelope("graph"),
        nodes: [
            ("source", "ready", "source.svg"),
            ("xml", "ready", "xml.json"),
            ("vector", "expected-failure", failure_artifact.as_str()),
            ("mesh", "expected-failure", "not-produced"),
        ]
        .into_iter()
        .map(|(stage, status, artifact)| GraphNode {
            id: format!("{case_id}/{stage}"),
            stage: stage.to_owned(),
            status: status.to_owned(),
            artifact: artifact.to_owned(),
        })
        .collect(),
        edges: [("source", "xml"), ("xml", "vector"), ("vector", "mesh")]
            .into_iter()
            .map(|(from, to)| GraphEdge {
                from: format!("{case_id}/{from}"),
                to: format!("{case_id}/{to}"),
            })
            .collect(),
    };
    write_json(&root.join("xml.json"), &xml_artifact)?;
    write_json(
        &root.join(format!("{artifact_name}.json")),
        &profile_artifact,
    )?;
    write_json(&root.join("graph.json"), &graph_artifact)?;
    fs::write(root.join("source.svg"), source)
        .map_err(|error| format!("write source.svg: {error}"))?;
    Ok(root)
}

pub(crate) fn remove_stale_artifact(root: &Path, artifact: &str) -> Result<(), String> {
    let path = root.join(artifact);
    if path.is_file() {
        fs::remove_file(&path)
            .map_err(|error| format!("remove stale artifact {}: {error}", path.display()))?;
    }
    Ok(())
}
