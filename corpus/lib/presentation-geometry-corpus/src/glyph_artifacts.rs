//! Durable diagnostic artifacts emitted from font-outline corpus cases.

use crate::{
    artifact_io::write_json,
    artifacts::{
        ArtifactAlgorithms, ArtifactEnvelope, GraphArtifact, GraphEdge, GraphNode,
        ImageFingerprint, MeshArtifact, MeshFingerprint, OutlineArtifact, OutlineContourArtifact,
        OutlineSegmentArtifact, VectorArtifact, VectorContourArtifact,
    },
    evidence::{canonical_triangle_hash, fnv1a64, fnv1a64_bytes, segment_intersections},
    geometry::{
        bounds_of_points, contours_svg, mesh_svg, rasterize_mesh, signed_area, validate_mesh,
    },
    GlyphCase,
};
use screenshot::{write_bmp, write_manifest, Rgba8Image};
use std::{fs, path::PathBuf};
use ui_tools::{
    tessellate_general_fill_with_rule, UiFontFormat, UiFontRasterizer, UiFontSource,
    UiGlyphVectorOptions, VectorFillRule,
};

/// Writes normalized diagnostics for one glyph case under the generated target
/// tree. Generated evidence never mutates reviewed golden expectations.
pub fn write_glyph_artifacts(case: GlyphCase) -> Result<PathBuf, String> {
    let source = UiFontSource::from_prepared_corpus("inter", UiFontFormat::Ttf)?;
    let rasterizer = UiFontRasterizer::from_bytes(source.bytes.clone())
        .map_err(|error| format!("font parse failed: {error}"))?;
    let outline = rasterizer
        .outline(case.character)
        .map_err(|error| format!("outline extraction failed: {}", error.message))?;
    let path = outline
        .to_vector_path(UiGlyphVectorOptions::new(1.0, [0.0, 0.0], 0.0005))
        .map_err(|error| format!("vector conversion failed: {}", error.message))?;
    let triangles = tessellate_general_fill_with_rule(&path, VectorFillRule::EvenOdd)
        .map_err(|error| format!("mesh tessellation failed: {error}"))?;

    let root = PathBuf::from("target/presentation-geometry-corpus").join(case.id);
    fs::create_dir_all(&root).map_err(|error| format!("create artifact directory: {error}"))?;
    let input_hash = format!("fnv1a64:{:016x}", fnv1a64(&source.bytes, case.character));
    let algorithms = ArtifactAlgorithms {
        flatten: "ui-glyph-outline-flatten-v1:tolerance=0.0005".to_owned(),
        tessellator: "ui-tools-general-fill".to_owned(),
        fill_rule: "even-odd".to_owned(),
    };
    let source_name = source.identity().source_name;
    let envelope = |artifact: &str| ArtifactEnvelope {
        schema: 1,
        artifact: artifact.to_owned(),
        producer: "font-outline/inter".to_owned(),
        case_id: case.id.to_owned(),
        input_hash: input_hash.clone(),
        source: source_name.clone(),
        algorithms: algorithms.clone(),
    };

    let outline_artifact = OutlineArtifact {
        metadata: envelope("outline"),
        character: outline.character,
        units_per_em: outline.units_per_em,
        contours: outline
            .contours
            .iter()
            .map(|contour| OutlineContourArtifact {
                start: contour.start,
                closed: contour.closed,
                segments: contour
                    .segments
                    .iter()
                    .map(|segment| match segment {
                        ui_tools::UiGlyphOutlineSegment::LineTo(end) => {
                            OutlineSegmentArtifact::Line { end: *end }
                        }
                        ui_tools::UiGlyphOutlineSegment::QuadTo { control, end } => {
                            OutlineSegmentArtifact::Quadratic {
                                control: *control,
                                end: *end,
                            }
                        }
                        ui_tools::UiGlyphOutlineSegment::CubicTo {
                            control1,
                            control2,
                            end,
                        } => OutlineSegmentArtifact::Cubic {
                            control1: *control1,
                            control2: *control2,
                            end: *end,
                        },
                    })
                    .collect(),
            })
            .collect(),
    };
    let vector_artifact = VectorArtifact {
        metadata: envelope("vector"),
        source_bounds: None,
        transformed_bounds: None,
        bounds: path.bounds(),
        contours: path
            .contours
            .iter()
            .enumerate()
            .map(|(index, contour)| VectorContourArtifact {
                index,
                closed: contour.closed,
                signed_area: signed_area(&contour.points),
                points: contour.points.clone(),
            })
            .collect(),
        paint_records: Vec::new(),
        intersections: segment_intersections(&path),
        clips: Vec::new(),
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
        nodes: ["source", "outline", "vector", "mesh"]
            .into_iter()
            .map(|stage| GraphNode {
                id: format!("{}/{}", case.id, stage),
                stage: stage.to_owned(),
                status: "ready".to_owned(),
                artifact: match stage {
                    "source" => "source.txt",
                    "outline" => "outline.json",
                    "vector" => "vector.json",
                    "mesh" => "mesh.json",
                    _ => unreachable!(),
                }
                .to_owned(),
            })
            .collect(),
        edges: ["source", "outline", "vector"]
            .into_iter()
            .map(|stage| GraphEdge {
                from: format!("{}/{}", case.id, stage),
                to: format!(
                    "{}/{}",
                    case.id,
                    match stage {
                        "source" => "outline",
                        "outline" => "vector",
                        "vector" => "mesh",
                        _ => unreachable!(),
                    }
                ),
            })
            .collect(),
    };

    write_json(&root.join("outline.json"), &outline_artifact)?;
    write_json(&root.join("vector.json"), &vector_artifact)?;
    write_json(&root.join("mesh.json"), &mesh_artifact)?;
    write_json(&root.join("mesh-fingerprint.json"), &mesh_fingerprint)?;
    write_json(&root.join("graph.json"), &graph_artifact)?;
    fs::write(root.join("contours.svg"), contours_svg(&path))
        .map_err(|error| format!("write contours.svg: {error}"))?;
    fs::write(root.join("mesh.svg"), mesh_svg(&mesh_artifact.triangles))
        .map_err(|error| format!("write mesh.svg: {error}"))?;
    let image = rasterize_mesh(&mesh_artifact.triangles, 256, 256)?;
    let image_fingerprint = ImageFingerprint {
        metadata: envelope("image-fingerprint"),
        width: 256,
        height: 256,
        format: "rgba8".to_owned(),
        source_buffer: "mesh-cpu".to_owned(),
        pixel_hash: format!("fnv1a64:{:016x}", fnv1a64_bytes(&image)),
    };
    write_json(&root.join("image-fingerprint.json"), &image_fingerprint)?;
    write_bmp(
        root.join("mesh-cpu.bmp"),
        Rgba8Image {
            width: 256,
            height: 256,
            pixels: &image,
        },
    )
    .map_err(|error| format!("write mesh-cpu.bmp: {error}"))?;
    write_manifest(
        root.join("mesh-cpu.manifest"),
        &[
            ("artifact", "mesh-cpu.bmp"),
            ("format", "bmp"),
            ("buffer", "cpu-rgba8"),
            ("gpu_readback", "false"),
            ("source_stage", "mesh"),
            ("dimensions", "256x256"),
            ("background", "12,15,21,255"),
            ("foreground", "165,210,245,255"),
        ],
    )
    .map_err(|error| format!("write mesh-cpu.manifest: {error}"))?;
    Ok(root)
}
