//! Shared XML/vector/mesh evidence serialization for SVG producers.

use crate::{
    artifact_io::write_json,
    artifacts::{
        ArtifactAlgorithms, ArtifactEnvelope, ClipPathArtifact, GraphArtifact, GraphEdge,
        GraphNode, MeshArtifact, MeshFingerprint, PaintArtifact, VectorArtifact,
        VectorContourArtifact, XmlArtifact,
    },
    evidence::{canonical_triangle_hash, fnv1a64, segment_intersections},
    geometry::{
        bounds_of_points, contours_svg, mesh_svg, signed_area, union_bounds, validate_mesh,
    },
    svg_artifact_cleanup::remove_stale_artifact,
    svg_support::{tessellate_svg_fills, tessellate_svg_strokes},
    xml_stage::XmlStageInspection,
};
use std::{fs, path::PathBuf};
use ui_tools::{is_convex_polygon_clip, SvgColor, SvgVectorRecord, VectorPath};

pub(crate) fn write_svg_record_artifacts(
    case_id: &str,
    producer: &str,
    source_label: String,
    source: String,
    xml: XmlStageInspection,
    records: Vec<SvgVectorRecord>,
    stroke_scale: f32,
) -> Result<PathBuf, String> {
    let paths = records
        .iter()
        .map(|record| record.path.clone())
        .collect::<Vec<_>>();
    // Only convex clips on SVG fill views have a structural intersection path
    // today. Do not emit an unclipped mesh for other clip combinations.
    let has_unresolved_clips = records.iter().any(|record| {
        record.clip_path.as_ref().is_some_and(|clip| {
            !record.fill
                || record.stroke
                || !record
                    .path_for_fill()
                    .contours
                    .iter()
                    .all(|contour| contour.closed)
                || !is_convex_polygon_clip(clip)
        })
    });
    let fill_meshes = (!has_unresolved_clips)
        .then(|| tessellate_svg_fills(&records, "SVG artifact"))
        .unwrap_or_default();
    if !fill_meshes.diagnostics.is_empty() {
        return Err(fill_meshes.diagnostics.join("; "));
    }
    let stroke_meshes = if has_unresolved_clips {
        Default::default()
    } else {
        tessellate_svg_strokes(&records, stroke_scale, "SVG artifact")
    };
    if !stroke_meshes.diagnostics.is_empty() {
        return Err(stroke_meshes.diagnostics.join("; "));
    }
    let fill_paths = fill_meshes.fill_paths;
    let stroke_paths = stroke_meshes.stroke_paths;
    let mut triangles = fill_meshes.triangles;
    triangles.extend(stroke_meshes.triangles);
    let root = PathBuf::from("target/presentation-geometry-corpus").join(case_id);
    fs::create_dir_all(&root).map_err(|error| format!("create artifact directory: {error}"))?;
    let input_hash = format!("fnv1a64:{:016x}", fnv1a64(source.as_bytes(), '\0'));
    let algorithms = ArtifactAlgorithms {
        flatten: "svg-path-flatten-v1:subdivisions=12".to_owned(),
        tessellator: "ui-tools-general-fill+stroke-v1".to_owned(),
        fill_rule: "per-record SVG fill-rule".to_owned(),
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
    let vector_artifact = VectorArtifact {
        metadata: envelope("vector"),
        source_bounds: records
            .iter()
            .filter_map(|record| record.source_bounds)
            .reduce(union_bounds),
        transformed_bounds: records
            .iter()
            .filter_map(|record| record.transformed_bounds)
            .reduce(union_bounds),
        bounds: paths
            .iter()
            .filter_map(VectorPath::bounds)
            .reduce(union_bounds),
        contours: paths
            .iter()
            .flat_map(|path| path.contours.iter())
            .enumerate()
            .map(|(index, contour)| VectorContourArtifact {
                index,
                closed: contour.closed,
                signed_area: signed_area(&contour.points),
                points: contour.points.clone(),
            })
            .collect(),
        paint_records: records
            .iter()
            .enumerate()
            .map(|(record_index, record)| PaintArtifact {
                record_index,
                fill: record.fill,
                stroke: record.stroke,
                fill_color: svg_color_components(record.fill_color),
                stroke_color: svg_color_components(record.stroke_color),
                fill_opacity: record.fill_opacity,
                stroke_opacity: record.stroke_opacity,
                opacity: record.opacity,
                stroke_width: record.stroke_width,
            })
            .collect(),
        intersections: paths.iter().flat_map(segment_intersections).collect(),
        clips: records
            .iter()
            .enumerate()
            .filter_map(|(target_record, record)| {
                record.clip_path.as_ref().map(|clip| ClipPathArtifact {
                    target_record,
                    bounds: clip.bounds(),
                    contour_count: clip.contours.len(),
                    point_count: clip
                        .contours
                        .iter()
                        .map(|contour| contour.points.len())
                        .sum(),
                })
            })
            .collect(),
    };
    let xml_artifact = XmlArtifact {
        metadata: envelope("xml"),
        event_count: xml.evidence.event_count,
        start_elements: xml.evidence.start_elements,
        end_elements: xml.evidence.end_elements,
        text_nodes: xml.evidence.text_nodes,
        comments: xml.evidence.comments,
        processing_instructions: xml.evidence.processing_instructions,
        document_roots: xml.evidence.document_roots,
        has_document_element: xml.evidence.has_document_element,
    };
    let mesh_artifact = MeshArtifact {
        metadata: envelope("mesh"),
        bounds: bounds_of_points(&triangles),
        validation: validate_mesh(&triangles),
        triangles,
    };
    let mesh_fingerprint = MeshFingerprint {
        metadata: mesh_artifact.metadata.clone(),
        bounds: mesh_artifact.bounds,
        triangle_count: mesh_artifact.validation.triangle_count,
        degenerate_triangles: mesh_artifact.validation.degenerate_triangles,
        total_area: mesh_artifact.validation.total_area,
        canonical_triangle_hash: canonical_triangle_hash(&mesh_artifact.triangles),
    };
    let graph_artifact = GraphArtifact {
        metadata: envelope("graph"),
        nodes: ["source", "xml", "vector", "mesh"]
            .into_iter()
            .map(|stage| GraphNode {
                id: format!("{case_id}/{stage}"),
                stage: stage.to_owned(),
                status: if stage == "mesh"
                    && (has_unresolved_clips || (fill_paths == 0 && stroke_paths == 0))
                {
                    "expected-failure".to_owned()
                } else {
                    "ready".to_owned()
                },
                artifact: match stage {
                    "source" => "source.svg",
                    "xml" => "xml.json",
                    "vector" => "vector.json",
                    "mesh" if has_unresolved_clips || (fill_paths == 0 && stroke_paths == 0) => {
                        "not-produced"
                    }
                    "mesh" => "mesh.json",
                    _ => unreachable!(),
                }
                .to_owned(),
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
    write_json(&root.join("vector.json"), &vector_artifact)?;
    if has_unresolved_clips || (fill_paths == 0 && stroke_paths == 0) {
        for artifact in ["mesh.json", "mesh-fingerprint.json", "mesh.svg"] {
            remove_stale_artifact(&root, artifact)?;
        }
    } else {
        write_json(&root.join("mesh.json"), &mesh_artifact)?;
        write_json(&root.join("mesh-fingerprint.json"), &mesh_fingerprint)?;
        fs::write(root.join("mesh.svg"), mesh_svg(&mesh_artifact.triangles))
            .map_err(|error| format!("write SVG mesh artifact: {error}"))?;
    }
    write_json(&root.join("graph.json"), &graph_artifact)?;
    fs::write(root.join("source.svg"), source)
        .map_err(|error| format!("write SVG source artifact: {error}"))?;
    fs::write(
        root.join("contours.svg"),
        paths.iter().map(contours_svg).collect::<String>(),
    )
    .map_err(|error| format!("write SVG contour artifact: {error}"))?;
    Ok(root)
}

fn svg_color_components(color: Option<SvgColor>) -> Option<[f32; 4]> {
    color.map(|SvgColor::Rgba(components)| components)
}
