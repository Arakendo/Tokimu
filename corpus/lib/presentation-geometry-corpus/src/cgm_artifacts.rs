//! Structural artifacts for CGM source and vector evidence.

use std::{fs, path::PathBuf};

use cgm_corpus::{inspect_binary_cgm_file, lower_picture_primitives, DecodeLimits};
use ui_tools::VectorPath;

use crate::{
    artifact_io::write_json,
    artifacts::{
        ArtifactAlgorithms, ArtifactEnvelope, CgmArtifact, CgmPrimitiveSourceArtifact,
        GraphArtifact, GraphEdge, GraphNode, VectorArtifact, VectorContourArtifact,
        VectorFingerprint,
    },
    cases::CgmExpectation,
    evidence::{canonical_path_hash, fnv1a64, segment_intersections},
    geometry::signed_area,
    CgmCase,
};

/// Writes source-semantic and provider-neutral vector artifacts for one CGM
/// case. Mesh artifacts are intentionally omitted until the source adapter can
/// resolve CGM fill and edge intent without guessing standard defaults.
pub fn write_cgm_artifacts(case: CgmCase) -> Result<PathBuf, String> {
    let source_path = fixture_path(case.file_name);
    let source = fs::read(&source_path)
        .map_err(|error| format!("read CGM fixture {}: {error}", source_path.display()))?;
    let inspection = inspect_binary_cgm_file(&source_path, DecodeLimits::default())
        .map_err(|error| format!("inspect CGM fixture {}: {error}", case.file_name))?;
    let picture = inspection
        .pictures
        .first()
        .ok_or_else(|| format!("CGM fixture {} contains no picture", case.file_name))?;
    let root = PathBuf::from("target/presentation-geometry-corpus").join(case.id);
    fs::create_dir_all(&root).map_err(|error| format!("create artifact directory: {error}"))?;
    let input_hash = format!("fnv1a64:{:016x}", fnv1a64(&source, '\0'));
    let algorithms = ArtifactAlgorithms {
        flatten: "cgm-basic-primitives-v1:circle-ellipse-arc-segments=32".to_owned(),
        tessellator: "not-produced:CGM-paint-resolution-pending".to_owned(),
        fill_rule: "not-produced:CGM-fill-and-edge-intent-pending".to_owned(),
    };
    let envelope = |artifact: &str| ArtifactEnvelope {
        schema: 1,
        artifact: artifact.to_owned(),
        producer: "cgm/webcgm".to_owned(),
        case_id: case.id.to_owned(),
        input_hash: input_hash.clone(),
        source: format!("WebCGM static10/{}", case.file_name),
        algorithms: algorithms.clone(),
    };

    let cgm = CgmArtifact {
        metadata: envelope("cgm"),
        metafile_name: inspection.metafile_name,
        picture_name: picture.name.clone(),
        source_bytes: inspection.source_bytes,
        element_count: inspection.elements.len(),
        primitive_count: picture.primitives.len(),
        attribute_count: picture.attributes.len(),
        primitives: picture
            .primitives
            .iter()
            .map(|primitive| CgmPrimitiveSourceArtifact {
                source_element: primitive.source_element,
                source_offset: primitive.source_offset,
                attribute_count: primitive.attribute_count,
                kind: primitive.kind.clone(),
                state: primitive.state.clone(),
                controls: primitive.controls.clone(),
            })
            .collect(),
        clip_rectangle: picture.controls.clip_rectangle,
        clip_indicator: picture.controls.clip_indicator,
        diagnostic_count: inspection.diagnostics.len(),
    };

    if !matches!(case.expectation, CgmExpectation::VectorPass) {
        clear_stale_vector_artifacts(&root)?;
        let (vector_status, vector_artifact) = match case.expectation {
            CgmExpectation::SourceOnly => (None, None),
            CgmExpectation::ExpectedUnsupportedLowering { .. } => {
                (Some("expected-failure"), Some("not-produced"))
            }
            CgmExpectation::VectorPass => unreachable!("covered by the outer guard"),
        };
        let mut nodes = vec![
            graph_node(case, "source", "ready", "source.cgm"),
            graph_node(case, "cgm", "ready", "cgm.json"),
        ];
        let mut edges = vec![graph_edge(case, "source", "cgm")];
        if let (Some(status), Some(artifact)) = (vector_status, vector_artifact) {
            nodes.push(graph_node(case, "vector", status, artifact));
            edges.push(graph_edge(case, "cgm", "vector"));
        }
        let graph = GraphArtifact {
            metadata: envelope("graph"),
            nodes,
            edges,
        };

        write_json(&root.join("cgm.json"), &cgm)?;
        write_json(&root.join("graph.json"), &graph)?;
        fs::write(root.join("source.cgm"), source)
            .map_err(|error| format!("write CGM source: {error}"))?;
        return Ok(root);
    }

    let primitives = lower_picture_primitives(picture)
        .map_err(|error| format!("lower CGM fixture {}: {error}", case.file_name))?;
    if primitives.is_empty() {
        return Err(format!(
            "CGM fixture {} lowered no primitives",
            case.file_name
        ));
    }
    let paths = primitives
        .iter()
        .map(|primitive| &primitive.path)
        .collect::<Vec<_>>();
    let vector = VectorArtifact {
        metadata: envelope("vector"),
        source_bounds: None,
        transformed_bounds: None,
        bounds: union_path_bounds(&paths),
        contours: paths
            .iter()
            .flat_map(|path| path.contours.iter())
            .enumerate()
            .map(|(index, contour)| VectorContourArtifact {
                index,
                closed: contour.closed,
                points: contour.points.clone(),
                signed_area: signed_area(&contour.points),
            })
            .collect(),
        // CGM source state remains in cgm.json until fill/edge intent can be
        // resolved without claiming CGM default or bundle behavior.
        paint_records: Vec::new(),
        intersections: paths
            .iter()
            .flat_map(|path| segment_intersections(path))
            .collect(),
        clips: Vec::new(),
    };
    let vector_fingerprint = VectorFingerprint {
        metadata: envelope("vector-fingerprint"),
        path_count: paths.len(),
        contour_count: paths.iter().map(|path| path.contours.len()).sum(),
        point_count: paths
            .iter()
            .flat_map(|path| &path.contours)
            .map(|contour| contour.points.len())
            .sum(),
        canonical_path_hash: canonical_path_hash(&paths),
    };
    let graph = GraphArtifact {
        metadata: envelope("graph"),
        nodes: [
            graph_node(case, "source", "ready", "source.cgm"),
            graph_node(case, "cgm", "ready", "cgm.json"),
            graph_node(case, "vector", "ready", "vector.json"),
            graph_node(case, "mesh", "expected-failure", "not-produced"),
        ]
        .into(),
        edges: [
            graph_edge(case, "source", "cgm"),
            graph_edge(case, "cgm", "vector"),
            graph_edge(case, "vector", "mesh"),
        ]
        .into(),
    };

    write_json(&root.join("cgm.json"), &cgm)?;
    write_json(&root.join("vector.json"), &vector)?;
    write_json(&root.join("vector-fingerprint.json"), &vector_fingerprint)?;
    write_json(&root.join("graph.json"), &graph)?;
    fs::write(root.join("source.cgm"), source)
        .map_err(|error| format!("write CGM source: {error}"))?;
    Ok(root)
}

/// A source-only or expected-boundary case must not retain artifacts emitted
/// by an earlier vector-capable revision of the same case ID.
fn clear_stale_vector_artifacts(root: &std::path::Path) -> Result<(), String> {
    for file_name in ["vector.json", "vector-fingerprint.json"] {
        let path = root.join(file_name);
        if path.exists() {
            fs::remove_file(&path).map_err(|error| {
                format!("remove stale CGM artifact {}: {error}", path.display())
            })?;
        }
    }
    Ok(())
}

fn graph_node(case: CgmCase, stage: &str, status: &str, artifact: &str) -> GraphNode {
    GraphNode {
        id: format!("{}/{stage}", case.id),
        stage: stage.to_owned(),
        status: status.to_owned(),
        artifact: artifact.to_owned(),
    }
}

fn graph_edge(case: CgmCase, from: &str, to: &str) -> GraphEdge {
    GraphEdge {
        from: format!("{}/{from}", case.id),
        to: format!("{}/{to}", case.id),
    }
}

fn fixture_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../third-party/fixtures/webcgm-test-suite/upstream/static10")
        .join(file_name)
}

fn union_path_bounds(paths: &[&VectorPath]) -> Option<([f32; 2], [f32; 2])> {
    paths
        .iter()
        .filter_map(|path| path.bounds())
        .reduce(|(min, max), (next_min, next_max)| {
            (
                [min[0].min(next_min[0]), min[1].min(next_min[1])],
                [max[0].max(next_max[0]), max[1].max(next_max[1])],
            )
        })
}
