//! Incubating, stage-aware presentation geometry corpus runner.
//!
//! The runner deliberately reports structural evidence before it writes any
//! artifacts. This keeps the first contract small while the cases are still
//! teaching us which representations need to become stable.

use screenshot::{write_bmp, write_manifest, Rgba8Image};
use serde::Serialize;
use std::{
    cmp::Ordering,
    fs,
    path::{Path, PathBuf},
};
use ui_tools::{
    lower_surface_to_vector, tessellate_general_fill_with_rule, SvgFillRule, UiFontFormat,
    UiFontRasterizer, UiFontSource, UiGlyphVectorOptions, UiRect, UiSurfaceCommand, UiSurfaceRole,
    UiTheme, VectorContour, VectorFillRule, VectorPath,
};

const GLYPH_CASES: [GlyphCase; 4] = [
    GlyphCase::new("glyph/inter/K", 'K'),
    GlyphCase::new("glyph/inter/k", 'k'),
    GlyphCase::new("glyph/inter/M", 'M'),
    GlyphCase::new("glyph/inter/e", 'e'),
];

const SYNTHETIC_CASES: [SyntheticCase; 5] = [
    SyntheticCase::new("synthetic/convex-rectangle", "convex rectangle"),
    SyntheticCase::new("synthetic/concave-notch", "concave notch"),
    SyntheticCase::new("synthetic/multi-contour-hole", "multi-contour hole"),
    SyntheticCase::new("synthetic/near-degenerate", "near-degenerate rectangle"),
    SyntheticCase::expected_failure(
        "synthetic/self-intersection-bowtie",
        "self-intersecting bow-tie",
    ),
];

const SVG_CASES: [SvgCase; 1] = [SvgCase::new(
    "svg/lucide/archive",
    "archive.svg",
    "Lucide archive SVG",
)];

const W3C_SVG_CASES: [W3cSvgCase; 2] = [
    W3cSvgCase::new(
        "svg/w3c/painting-fill-03-t",
        "painting-fill-03-t.svg",
        "W3C even-odd and non-zero fill-rule fixture",
    ),
    W3cSvgCase::new(
        "svg/w3c/paths-data-16-t",
        "paths-data-16-t.svg",
        "W3C implicit line-to and relative path fixture",
    ),
];

const UI_CASES: [UiCase; 1] = [UiCase::new("ui/panel-surface", "default panel surface")];

const ALL_CASES: [CorpusCase; 11] = [
    CorpusCase::Glyph(GLYPH_CASES[0]),
    CorpusCase::Glyph(GLYPH_CASES[1]),
    CorpusCase::Glyph(GLYPH_CASES[2]),
    CorpusCase::Glyph(GLYPH_CASES[3]),
    CorpusCase::Synthetic(SYNTHETIC_CASES[0]),
    CorpusCase::Synthetic(SYNTHETIC_CASES[1]),
    CorpusCase::Synthetic(SYNTHETIC_CASES[2]),
    CorpusCase::Synthetic(SYNTHETIC_CASES[3]),
    CorpusCase::Synthetic(SYNTHETIC_CASES[4]),
    CorpusCase::Svg(SVG_CASES[0]),
    CorpusCase::Ui(UI_CASES[0]),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GlyphCase {
    pub id: &'static str,
    pub character: char,
}

impl GlyphCase {
    pub const fn new(id: &'static str, character: char) -> Self {
        Self { id, character }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyntheticCase {
    pub id: &'static str,
    pub description: &'static str,
    pub expected_failure: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SvgCase {
    pub id: &'static str,
    pub file_name: &'static str,
    pub description: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct W3cSvgCase {
    pub id: &'static str,
    pub file_name: &'static str,
    pub description: &'static str,
}

impl W3cSvgCase {
    pub const fn new(id: &'static str, file_name: &'static str, description: &'static str) -> Self {
        Self {
            id,
            file_name,
            description,
        }
    }
}

impl SvgCase {
    pub const fn new(id: &'static str, file_name: &'static str, description: &'static str) -> Self {
        Self {
            id,
            file_name,
            description,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiCase {
    pub id: &'static str,
    pub description: &'static str,
}

impl UiCase {
    pub const fn new(id: &'static str, description: &'static str) -> Self {
        Self { id, description }
    }
}

impl SyntheticCase {
    pub const fn new(id: &'static str, description: &'static str) -> Self {
        Self {
            id,
            description,
            expected_failure: false,
        }
    }

    pub const fn expected_failure(id: &'static str, description: &'static str) -> Self {
        Self {
            id,
            description,
            expected_failure: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorpusCase {
    Glyph(GlyphCase),
    Synthetic(SyntheticCase),
    Svg(SvgCase),
    W3cSvg(W3cSvgCase),
    Ui(UiCase),
}

impl CorpusCase {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Glyph(case) => case.id,
            Self::Synthetic(case) => case.id,
            Self::Svg(case) => case.id,
            Self::W3cSvg(case) => case.id,
            Self::Ui(case) => case.id,
        }
    }

    pub const fn selected_stages(self) -> &'static [CorpusStage] {
        match self {
            Self::Glyph(_) => &GLYPH_STAGES,
            Self::Synthetic(_) | Self::Svg(_) | Self::W3cSvg(_) | Self::Ui(_) => &PATH_STAGES,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorpusStage {
    Source,
    Outline,
    Vector,
    Mesh,
}

const GLYPH_STAGES: [CorpusStage; 4] = [
    CorpusStage::Source,
    CorpusStage::Outline,
    CorpusStage::Vector,
    CorpusStage::Mesh,
];
const PATH_STAGES: [CorpusStage; 3] = [CorpusStage::Source, CorpusStage::Vector, CorpusStage::Mesh];

impl CorpusStage {
    pub const ALL: [Self; 4] = [Self::Source, Self::Outline, Self::Vector, Self::Mesh];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Outline => "outline",
            Self::Vector => "vector",
            Self::Mesh => "mesh",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StageStatus {
    Ready,
    Failed,
    ExpectedFailure,
}

impl StageStatus {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Failed => "failed",
            Self::ExpectedFailure => "expected-failure",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StageReport {
    pub stage: CorpusStage,
    pub status: StageStatus,
    pub summary: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseReport {
    pub id: String,
    pub producer: String,
    pub selected_stages: Vec<CorpusStage>,
    pub stages: Vec<StageReport>,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
struct GoldenSnapshot {
    schema: u32,
    case_id: String,
    producer: String,
    selected_stages: Vec<String>,
    stages: Vec<GoldenStage>,
    diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
struct GoldenStage {
    stage: String,
    status: String,
    summary: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ArtifactEnvelope {
    pub schema: u32,
    pub artifact: String,
    pub producer: String,
    pub case_id: String,
    pub input_hash: String,
    pub source: String,
    pub algorithms: ArtifactAlgorithms,
}

#[derive(Clone, Debug, Serialize)]
pub struct ArtifactAlgorithms {
    pub flatten: String,
    pub tessellator: String,
    pub fill_rule: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct OutlineArtifact {
    pub metadata: ArtifactEnvelope,
    pub character: char,
    pub units_per_em: f32,
    pub contours: Vec<OutlineContourArtifact>,
}

#[derive(Clone, Debug, Serialize)]
pub struct OutlineContourArtifact {
    pub start: [f32; 2],
    pub closed: bool,
    pub segments: Vec<OutlineSegmentArtifact>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind")]
pub enum OutlineSegmentArtifact {
    Line {
        end: [f32; 2],
    },
    Quadratic {
        control: [f32; 2],
        end: [f32; 2],
    },
    Cubic {
        control1: [f32; 2],
        control2: [f32; 2],
        end: [f32; 2],
    },
}

#[derive(Clone, Debug, Serialize)]
pub struct VectorArtifact {
    pub metadata: ArtifactEnvelope,
    pub bounds: Option<([f32; 2], [f32; 2])>,
    pub contours: Vec<VectorContourArtifact>,
    pub intersections: Vec<SegmentIntersectionArtifact>,
}

#[derive(Clone, Debug, Serialize)]
pub struct VectorContourArtifact {
    pub index: usize,
    pub closed: bool,
    pub points: Vec<[f32; 2]>,
    pub signed_area: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct SegmentIntersectionArtifact {
    pub first_contour: usize,
    pub first_segment: usize,
    pub second_contour: usize,
    pub second_segment: usize,
    pub point: [f32; 2],
}

#[derive(Clone, Debug, Serialize)]
pub struct MeshArtifact {
    pub metadata: ArtifactEnvelope,
    pub bounds: Option<([f32; 2], [f32; 2])>,
    pub triangles: Vec<[f32; 2]>,
    pub validation: MeshValidation,
}

#[derive(Clone, Debug, Serialize)]
pub struct MeshValidation {
    pub finite: bool,
    pub complete_triangles: bool,
    pub triangle_count: usize,
    pub degenerate_triangles: usize,
    pub total_area: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct MeshFingerprint {
    pub metadata: ArtifactEnvelope,
    pub bounds: Option<([f32; 2], [f32; 2])>,
    pub triangle_count: usize,
    pub degenerate_triangles: usize,
    pub total_area: f32,
    pub canonical_triangle_hash: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ImageFingerprint {
    pub metadata: ArtifactEnvelope,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub source_buffer: String,
    pub pixel_hash: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct GraphArtifact {
    pub metadata: ArtifactEnvelope,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Clone, Debug, Serialize)]
pub struct GraphNode {
    pub id: String,
    pub stage: String,
    pub status: String,
    pub artifact: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
}

impl CaseReport {
    pub fn passed(&self) -> bool {
        self.diagnostics.is_empty()
            && self.stages.iter().all(|stage| {
                matches!(
                    stage.status,
                    StageStatus::Ready | StageStatus::ExpectedFailure
                )
            })
    }
}

pub fn glyph_cases() -> &'static [GlyphCase] {
    &GLYPH_CASES
}

pub fn find_glyph_case(id: &str) -> Option<GlyphCase> {
    glyph_cases().iter().copied().find(|case| case.id == id)
}

pub fn synthetic_cases() -> &'static [SyntheticCase] {
    &SYNTHETIC_CASES
}

pub fn svg_cases() -> &'static [SvgCase] {
    &SVG_CASES
}

pub fn w3c_svg_cases() -> &'static [W3cSvgCase] {
    &W3C_SVG_CASES
}

pub fn ui_cases() -> &'static [UiCase] {
    &UI_CASES
}

pub fn all_cases() -> &'static [CorpusCase] {
    &ALL_CASES
}

pub fn find_case(id: &str) -> Option<CorpusCase> {
    all_cases()
        .iter()
        .copied()
        .find(|case| case.id() == id)
        .or_else(|| {
            w3c_svg_cases()
                .iter()
                .copied()
                .find(|case| case.id == id)
                .map(CorpusCase::W3cSvg)
        })
}

pub fn run_case(case: CorpusCase) -> CaseReport {
    match case {
        CorpusCase::Glyph(case) => run_glyph_case(case),
        CorpusCase::Synthetic(case) => run_synthetic_case(case),
        CorpusCase::Svg(case) => run_svg_case(case),
        CorpusCase::W3cSvg(case) => run_w3c_svg_case(case),
        CorpusCase::Ui(case) => run_ui_case(case),
    }
}

/// Returns the reviewed fixture location for a case report.
pub fn golden_snapshot_path(case_id: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/golden/presentation-geometry")
        .join(golden_case_key(case_id))
        .join("report.json")
}

fn golden_mesh_fingerprint_path(case_id: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/golden/presentation-geometry")
        .join(golden_case_key(case_id))
        .join("mesh-fingerprint.json")
}

fn golden_image_fingerprint_path(case_id: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/golden/presentation-geometry")
        .join(golden_case_key(case_id))
        .join("image-fingerprint.json")
}

fn golden_case_key(case_id: &str) -> String {
    format!(
        "{}--{:016x}",
        case_id.replace('/', "__"),
        fnv1a64(case_id.as_bytes(), '\0')
    )
}

/// Writes one reviewed structural snapshot. This is intentionally an explicit
/// operation; ordinary corpus runs never mutate fixtures.
pub fn bless_case(case: CorpusCase) -> Result<PathBuf, String> {
    let report = run_case(case);
    if !report.passed() {
        return Err(format!("cannot bless failed case {}", report.id));
    }
    let generated_root = match case {
        CorpusCase::Glyph(glyph) => Some(write_glyph_artifacts(glyph)?),
        CorpusCase::Synthetic(_)
        | CorpusCase::Svg(_)
        | CorpusCase::W3cSvg(_)
        | CorpusCase::Ui(_) => None,
    };
    let path = golden_snapshot_path(&report.id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("create golden directory: {error}"))?;
    }
    let snapshot = golden_snapshot(&report);
    let json = serde_json::to_string_pretty(&snapshot)
        .map_err(|error| format!("serialize golden snapshot: {error}"))?;
    fs::write(&path, format!("{json}\n"))
        .map_err(|error| format!("write golden snapshot: {error}"))?;
    if let Some(root) = generated_root {
        let source = root.join("mesh-fingerprint.json");
        let target = golden_mesh_fingerprint_path(&report.id);
        fs::copy(&source, &target)
            .map_err(|error| format!("write golden mesh fingerprint: {error}"))?;
        let source = root.join("image-fingerprint.json");
        let target = golden_image_fingerprint_path(&report.id);
        fs::copy(&source, &target)
            .map_err(|error| format!("write golden image fingerprint: {error}"))?;
    }
    Ok(path)
}

/// Compares one case with its reviewed structural snapshot without mutating it.
pub fn compare_case(case: CorpusCase) -> Result<(), String> {
    let report = run_case(case);
    if !report.passed() {
        return Err(format!(
            "case {} failed before golden comparison",
            report.id
        ));
    }
    let path = golden_snapshot_path(&report.id);
    let expected = fs::read_to_string(&path)
        .map_err(|error| format!("read golden {}: {error}", path.display()))?;
    let actual = serde_json::to_string_pretty(&golden_snapshot(&report))
        .map_err(|error| format!("serialize golden snapshot: {error}"))?
        + "\n";
    if expected != actual {
        return Err(format!(
            "golden mismatch: {}\n{}",
            path.display(),
            golden_diff(&expected, &actual)
        ));
    }
    if let CorpusCase::Glyph(glyph) = case {
        let root = write_glyph_artifacts(glyph)?;
        let mesh_path = golden_mesh_fingerprint_path(&report.id);
        let expected_mesh = fs::read_to_string(&mesh_path)
            .map_err(|error| format!("read golden {}: {error}", mesh_path.display()))?;
        let actual_mesh = fs::read_to_string(root.join("mesh-fingerprint.json"))
            .map_err(|error| format!("read generated mesh fingerprint: {error}"))?;
        if expected_mesh != actual_mesh {
            return Err(format!(
                "golden mesh mismatch: {}\n{}",
                mesh_path.display(),
                golden_diff(&expected_mesh, &actual_mesh)
            ));
        }
        let image_path = golden_image_fingerprint_path(&report.id);
        let expected_image = fs::read_to_string(&image_path)
            .map_err(|error| format!("read golden {}: {error}", image_path.display()))?;
        let actual_image = fs::read_to_string(root.join("image-fingerprint.json"))
            .map_err(|error| format!("read generated image fingerprint: {error}"))?;
        if expected_image != actual_image {
            return Err(format!(
                "golden image mismatch: {}\n{}",
                image_path.display(),
                golden_diff(&expected_image, &actual_image)
            ));
        }
    }
    Ok(())
}

fn golden_diff(expected: &str, actual: &str) -> String {
    let expected_lines = expected.lines().collect::<Vec<_>>();
    let actual_lines = actual.lines().collect::<Vec<_>>();
    let line_count = expected_lines.len().max(actual_lines.len());
    for index in 0..line_count {
        let expected_line = expected_lines.get(index).copied().unwrap_or("<missing>");
        let actual_line = actual_lines.get(index).copied().unwrap_or("<missing>");
        if expected_line != actual_line {
            return format!(
                "first difference at line {}\n  expected: {}\n  actual:   {}",
                index + 1,
                expected_line,
                actual_line
            );
        }
    }
    "content differs despite matching lines".to_owned()
}

fn golden_snapshot(report: &CaseReport) -> GoldenSnapshot {
    GoldenSnapshot {
        schema: 1,
        case_id: report.id.clone(),
        producer: report.producer.clone(),
        selected_stages: report
            .selected_stages
            .iter()
            .map(|stage| stage.name().to_owned())
            .collect(),
        stages: report
            .stages
            .iter()
            .map(|stage| GoldenStage {
                stage: stage.stage.name().to_owned(),
                status: stage.status.name().to_owned(),
                summary: stage.summary.clone(),
            })
            .collect(),
        diagnostics: report.diagnostics.clone(),
    }
}

/// Runs a semantic UI surface through its public vector-lowering adapter.
pub fn run_ui_case(case: UiCase) -> CaseReport {
    let mut report = CaseReport {
        id: case.id.to_owned(),
        producer: "ui/surface".to_owned(),
        selected_stages: CorpusCase::Ui(case).selected_stages().to_vec(),
        stages: vec![StageReport {
            stage: CorpusStage::Source,
            status: StageStatus::Ready,
            summary: case.description.to_owned(),
        }],
        diagnostics: Vec::new(),
    };
    let theme = UiTheme::default();
    let command = UiSurfaceCommand {
        rect: UiRect::new([0.0, 0.0], [0.72, 0.48]),
        style: theme.surface(UiSurfaceRole::Panel),
        clip: None,
    };
    let layers = lower_surface_to_vector(&command);
    report.stages.push(StageReport {
        stage: CorpusStage::Vector,
        status: StageStatus::Ready,
        summary: format!(
            "layers={} contours={} points={}",
            layers.len(),
            layers
                .iter()
                .map(|layer| layer.path.contours.len())
                .sum::<usize>(),
            layers
                .iter()
                .flat_map(|layer| layer.path.contours.iter())
                .map(|contour| contour.points.len())
                .sum::<usize>()
        ),
    });
    let mut triangles = Vec::new();
    for layer in &layers {
        match tessellate_general_fill_with_rule(&layer.path, VectorFillRule::EvenOdd) {
            Ok(mut layer_triangles) => triangles.append(&mut layer_triangles),
            Err(error) => report
                .diagnostics
                .push(format!("UI layer tessellation failed: {error}")),
        }
    }
    let validation = validate_mesh(&triangles);
    if validation.finite && validation.complete_triangles && report.diagnostics.is_empty() {
        report.stages.push(StageReport {
            stage: CorpusStage::Mesh,
            status: StageStatus::Ready,
            summary: format_mesh_summary(&validation),
        });
    } else {
        let message = format!(
            "UI mesh validation failed: {}",
            format_mesh_summary(&validation)
        );
        report
            .stages
            .push(failed_stage(CorpusStage::Mesh, &message));
        report.diagnostics.push(message);
    }
    report
}

/// Runs a real SVG producer through the shared vector and fill-mesh stages.
///
/// Lucide icons are stroke-oriented. The current fill tessellator therefore
/// consumes only closed contours, while open paths remain visible evidence at
/// the vector stage until stroke expansion is admitted separately.
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
    let paths = match ui_tools::parse_svg_document_vector_paths(&svg, 12, [0.0, 0.0, 24.0, 24.0]) {
        Ok(paths) if !paths.is_empty() => paths,
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
    let contour_count = paths.iter().map(|path| path.contours.len()).sum::<usize>();
    let point_count = paths
        .iter()
        .flat_map(|path| path.contours.iter())
        .map(|contour| contour.points.len())
        .sum::<usize>();
    report.stages.push(StageReport {
        stage: CorpusStage::Vector,
        status: StageStatus::Ready,
        summary: format!(
            "{} paths={} contours={} points={} closed_contours={}",
            case.description,
            paths.len(),
            contour_count,
            point_count,
            paths
                .iter()
                .flat_map(|path| path.contours.iter())
                .filter(|contour| contour.closed)
                .count()
        ),
    });

    let mut triangles = Vec::new();
    let mut fill_paths = 0;
    for path in &paths {
        if path.contours.iter().all(|contour| contour.closed) {
            match tessellate_general_fill_with_rule(path, VectorFillRule::EvenOdd) {
                Ok(mut path_triangles) => {
                    triangles.append(&mut path_triangles);
                    fill_paths += 1;
                }
                Err(error) => {
                    report
                        .diagnostics
                        .push(format!("closed SVG path fill tessellation failed: {error}"));
                }
            }
        }
    }
    let validation = validate_mesh(&triangles);
    if fill_paths == 0 || !validation.finite || !validation.complete_triangles {
        let message = format!(
            "SVG fill mesh validation failed: closed_paths={} {}",
            fill_paths,
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
                fill_paths,
                paths.len() - fill_paths,
                format_mesh_summary(&validation)
            ),
        });
    }
    report
}

/// Runs one explicitly admitted W3C path fixture without invoking the W3C
/// browser harness. Open stroke-only paths remain expected limitations until
/// stroke expansion is admitted as a separate geometry capability.
pub fn run_w3c_svg_case(case: W3cSvgCase) -> CaseReport {
    let mut report = CaseReport {
        id: case.id.to_owned(),
        producer: "svg/w3c".to_owned(),
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
    let source_path = fixture_root.join("upstream/svg").join(case.file_name);
    let svg = match fs::read_to_string(&source_path) {
        Ok(svg) => {
            report.stages.push(StageReport {
                stage: CorpusStage::Source,
                status: StageStatus::Ready,
                summary: format!(
                    "file={} bytes={} source=W3C SVG 1.1 2nd Edition",
                    case.file_name,
                    svg.len()
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
    let records =
        match ui_tools::parse_svg_document_vector_records(&svg, 12, [0.0, 0.0, 480.0, 360.0]) {
            Ok(records) if !records.is_empty() => records,
            Ok(_) => {
                let message = "W3C SVG parser produced no vector paths".to_owned();
                report
                    .stages
                    .push(failed_stage(CorpusStage::Vector, &message));
                report.diagnostics.push(message);
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
    let contour_count = paths.iter().map(|path| path.contours.len()).sum::<usize>();
    let point_count = paths
        .iter()
        .flat_map(|path| path.contours.iter())
        .map(|contour| contour.points.len())
        .sum::<usize>();
    report.stages.push(StageReport {
        stage: CorpusStage::Vector,
        status: StageStatus::Ready,
        summary: format!(
            "{} paths={} contours={} points={} closed_contours={}",
            case.description,
            paths.len(),
            contour_count,
            point_count,
            paths
                .iter()
                .flat_map(|path| path.contours.iter())
                .filter(|contour| contour.closed)
                .count()
        ),
    });

    let mut triangles = Vec::new();
    let mut fill_paths = 0;
    for record in &records {
        if record.fill && record.path.contours.iter().all(|contour| contour.closed) {
            let fill_rule = match record.fill_rule {
                SvgFillRule::NonZero => VectorFillRule::NonZero,
                SvgFillRule::EvenOdd => VectorFillRule::EvenOdd,
            };
            match tessellate_general_fill_with_rule(&record.path, fill_rule) {
                Ok(mut path_triangles) => {
                    triangles.append(&mut path_triangles);
                    fill_paths += 1;
                }
                Err(error) => report.diagnostics.push(format!(
                    "closed W3C SVG path fill tessellation failed: {error}"
                )),
            }
        }
    }
    let validation = validate_mesh(&triangles);
    if fill_paths == 0 {
        report.stages.push(StageReport {
            stage: CorpusStage::Mesh,
            status: StageStatus::ExpectedFailure,
            summary: "no closed fill paths; stroke-only geometry is outside W3C v1 mesh scope"
                .to_owned(),
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
                "closed_paths={} open_paths={} {}",
                fill_paths,
                paths.len() - fill_paths,
                format_mesh_summary(&validation)
            ),
        });
    }
    report
}

fn find_w3c_fixture_root() -> Option<PathBuf> {
    let mut directory = std::env::current_dir().ok()?;
    loop {
        let candidate = directory.join("third-party/fixtures/w3c-svg-1.1-2nd-edition");
        if candidate.join("provenance.json").is_file()
            && candidate.join("selected/selection-v1.toml").is_file()
        {
            return Some(candidate);
        }
        if !directory.pop() {
            return None;
        }
    }
}

fn find_lucide_corpus_root() -> Option<PathBuf> {
    let mut directory = std::env::current_dir().ok()?;
    loop {
        let candidate = directory.join("target/lucide-corpus-100");
        if candidate.join("provenance.json").is_file() {
            return Some(candidate);
        }
        if !directory.pop() {
            return None;
        }
    }
}

pub fn run_synthetic_case(case: SyntheticCase) -> CaseReport {
    let mut report = CaseReport {
        id: case.id.to_owned(),
        producer: "synthetic/topology".to_owned(),
        selected_stages: CorpusCase::Synthetic(case).selected_stages().to_vec(),
        stages: vec![StageReport {
            stage: CorpusStage::Source,
            status: StageStatus::Ready,
            summary: case.description.to_owned(),
        }],
        diagnostics: Vec::new(),
    };
    let path = synthetic_path(case);
    let intersections = segment_intersections(&path);
    let vector_summary = format!(
        "contours={} points={} finite={} intersections={}",
        path.contours.len(),
        path.contours
            .iter()
            .map(|contour| contour.points.len())
            .sum::<usize>(),
        path.is_finite(),
        intersections.len()
    );
    if case.expected_failure {
        if intersections.is_empty() {
            let message = "expected vector self-intersection was not detected".to_owned();
            report
                .stages
                .push(failed_stage(CorpusStage::Vector, &message));
            report.diagnostics.push(message);
            return report;
        }
        report.stages.push(StageReport {
            stage: CorpusStage::Vector,
            status: StageStatus::ExpectedFailure,
            summary: format!("expected unsupported topology: {vector_summary}"),
        });
        report.stages.push(StageReport {
            stage: CorpusStage::Mesh,
            status: StageStatus::ExpectedFailure,
            summary: "not attempted after expected vector-topology failure".to_owned(),
        });
        return report;
    }
    report.stages.push(StageReport {
        stage: CorpusStage::Vector,
        status: StageStatus::Ready,
        summary: vector_summary,
    });
    match tessellate_general_fill_with_rule(&path, VectorFillRule::EvenOdd) {
        Ok(triangles) => {
            let validation = validate_mesh(&triangles);
            let coverage = validate_coverage(case, &path, &triangles);
            if validation.finite && validation.complete_triangles && coverage.is_empty() {
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
            report.diagnostics.extend(coverage);
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

/// Runs one deterministic generated polygon without adding it to the reviewed
/// case list. Generated cases are investigation inputs, not golden contracts.
pub fn run_generated_case(seed: u64, index: usize) -> CaseReport {
    let path = generated_path(seed, index);
    let id = format!("generated/{seed}/{index}");
    let intersections = segment_intersections(&path);
    let mut report = CaseReport {
        id,
        producer: "generated/seeded-polygon".to_owned(),
        selected_stages: PATH_STAGES.to_vec(),
        stages: vec![StageReport {
            stage: CorpusStage::Source,
            status: StageStatus::Ready,
            summary: format!(
                "seed={seed} index={index} contours={} points={}",
                path.contours.len(),
                path.contours
                    .iter()
                    .map(|contour| contour.points.len())
                    .sum::<usize>()
            ),
        }],
        diagnostics: Vec::new(),
    };

    if !intersections.is_empty() {
        let message = format!(
            "generated polygon has {} self-intersection(s)",
            intersections.len()
        );
        report
            .stages
            .push(failed_stage(CorpusStage::Vector, &message));
        report.diagnostics.push(message);
        return report;
    }

    report.stages.push(StageReport {
        stage: CorpusStage::Vector,
        status: StageStatus::Ready,
        summary: format!(
            "finite={} intersections={}",
            path.is_finite(),
            intersections.len()
        ),
    });
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
                    "generated mesh validation failed: {}",
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
                .push(format!("generated mesh tessellation failed: {error}"));
        }
    }
    report
}

fn generated_path(seed: u64, index: usize) -> VectorPath {
    let mut state = seed
        .wrapping_add((index as u64).wrapping_mul(0x9e3779b97f4a7c15))
        .max(1);
    let point_count = 5 + (next_random(&mut state) % 4) as usize;
    let mut points = Vec::with_capacity(point_count);
    for point_index in 0..point_count {
        let angle = std::f32::consts::TAU * point_index as f32 / point_count as f32;
        let radius = 0.28 + next_unit(&mut state) * 0.16;
        points.push([0.5 + angle.cos() * radius, 0.5 + angle.sin() * radius]);
    }
    VectorPath::new(vec![VectorContour::new(points, true)])
}

fn next_random(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn next_unit(state: &mut u64) -> f32 {
    (next_random(state) as f64 / u64::MAX as f64) as f32
}

fn synthetic_path(case: SyntheticCase) -> VectorPath {
    let contour = |points: &[[f32; 2]]| VectorContour::new(points.to_vec(), true);
    match case.id {
        "synthetic/convex-rectangle" => VectorPath::new(vec![contour(&[
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
        ])]),
        "synthetic/concave-notch" => VectorPath::new(vec![contour(&[
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 0.35],
            [0.55, 0.35],
            [0.55, 1.0],
            [0.0, 1.0],
        ])]),
        "synthetic/multi-contour-hole" => VectorPath::new(vec![
            contour(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]),
            contour(&[[0.25, 0.25], [0.25, 0.75], [0.75, 0.75], [0.75, 0.25]]),
        ]),
        "synthetic/near-degenerate" => VectorPath::new(vec![contour(&[
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 0.00001],
            [0.0, 0.00001],
        ])]),
        "synthetic/self-intersection-bowtie" => VectorPath::new(vec![contour(&[
            [0.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
            [1.0, 0.0],
        ])]),
        _ => unreachable!("synthetic case must be declared in SYNTHETIC_CASES"),
    }
}

#[derive(Clone, Copy)]
struct CoverageProbe {
    point: [f32; 2],
    expected_inside: bool,
}

fn coverage_probes(case: SyntheticCase) -> &'static [CoverageProbe] {
    const RECTANGLE: [CoverageProbe; 2] = [
        CoverageProbe {
            point: [0.5, 0.5],
            expected_inside: true,
        },
        CoverageProbe {
            point: [1.25, 0.5],
            expected_inside: false,
        },
    ];
    const NOTCH: [CoverageProbe; 2] = [
        CoverageProbe {
            point: [0.2, 0.2],
            expected_inside: true,
        },
        CoverageProbe {
            point: [0.8, 0.8],
            expected_inside: false,
        },
    ];
    const HOLE: [CoverageProbe; 3] = [
        CoverageProbe {
            point: [0.1, 0.1],
            expected_inside: true,
        },
        CoverageProbe {
            point: [0.5, 0.5],
            expected_inside: false,
        },
        CoverageProbe {
            point: [1.1, 0.5],
            expected_inside: false,
        },
    ];
    match case.id {
        "synthetic/convex-rectangle" => &RECTANGLE,
        "synthetic/concave-notch" => &NOTCH,
        "synthetic/multi-contour-hole" => &HOLE,
        "synthetic/near-degenerate" | "synthetic/self-intersection-bowtie" => &[],
        _ => &[],
    }
}

fn validate_coverage(
    case: SyntheticCase,
    path: &VectorPath,
    triangles: &[[f32; 2]],
) -> Vec<String> {
    coverage_probes(case)
        .iter()
        .filter_map(|probe| {
            let source_inside = point_in_path(probe.point, path);
            let mesh_inside = point_in_mesh(probe.point, triangles);
            if source_inside != probe.expected_inside || mesh_inside != probe.expected_inside {
                Some(format!(
                    "coverage probe {:?}: expected={} source={} mesh={}",
                    probe.point, probe.expected_inside, source_inside, mesh_inside
                ))
            } else {
                None
            }
        })
        .collect()
}

fn point_in_path(point: [f32; 2], path: &VectorPath) -> bool {
    path.contours
        .iter()
        .filter(|contour| contour.closed)
        .fold(false, |inside, contour| {
            if point_in_contour(point, &contour.points) {
                !inside
            } else {
                inside
            }
        })
}

fn point_in_contour(point: [f32; 2], points: &[[f32; 2]]) -> bool {
    let mut inside = false;
    for (a, b) in points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
    {
        if (a[1] > point[1]) != (b[1] > point[1])
            && point[0] < (b[0] - a[0]) * (point[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            inside = !inside;
        }
    }
    inside
}

fn point_in_mesh(point: [f32; 2], triangles: &[[f32; 2]]) -> bool {
    triangles
        .chunks_exact(3)
        .any(|triangle| point_in_triangle(point, [triangle[0], triangle[1], triangle[2]]))
}

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

/// Writes normalized diagnostics for one case under the generated target tree.
///
/// This is intentionally separate from reviewed fixtures: ordinary runs may
/// refresh generated evidence, but never mutate golden expectations.
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
        intersections: segment_intersections(&path),
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

/// Writes structural artifacts for one admitted W3C SVG case. This deliberately
/// stops at source, vector, and CPU mesh evidence; it does not invoke the W3C
/// browser harness or capture a backend framebuffer.
pub fn write_w3c_artifacts(case: W3cSvgCase) -> Result<PathBuf, String> {
    let fixture_root = find_w3c_fixture_root()
        .ok_or_else(|| "W3C SVG fixture not found; run verify-w3c-svg-fixtures.ps1".to_owned())?;
    let source_path = fixture_root.join("upstream/svg").join(case.file_name);
    let source = fs::read_to_string(&source_path)
        .map_err(|error| format!("read W3C source {}: {error}", source_path.display()))?;
    let paths = ui_tools::parse_svg_document_vector_paths(&source, 12, [0.0, 0.0, 480.0, 360.0])
        .map_err(|error| format!("W3C vector conversion failed: {error}"))?;
    let mut triangles = Vec::new();
    for path in &paths {
        if path.contours.iter().all(|contour| contour.closed) {
            let mut path_triangles =
                tessellate_general_fill_with_rule(path, VectorFillRule::EvenOdd)
                    .map_err(|error| format!("W3C mesh tessellation failed: {error}"))?;
            triangles.append(&mut path_triangles);
        }
    }

    let root = PathBuf::from("target/presentation-geometry-corpus").join(case.id);
    fs::create_dir_all(&root).map_err(|error| format!("create artifact directory: {error}"))?;
    let input_hash = format!("fnv1a64:{:016x}", fnv1a64(source.as_bytes(), '\0'));
    let algorithms = ArtifactAlgorithms {
        flatten: "svg-path-flatten-v1:subdivisions=12".to_owned(),
        tessellator: "ui-tools-general-fill".to_owned(),
        fill_rule: "even-odd".to_owned(),
    };
    let envelope = |artifact: &str| ArtifactEnvelope {
        schema: 1,
        artifact: artifact.to_owned(),
        producer: "svg/w3c".to_owned(),
        case_id: case.id.to_owned(),
        input_hash: input_hash.clone(),
        source: format!("W3C SVG 1.1 2nd Edition/{}", case.file_name),
        algorithms: algorithms.clone(),
    };
    let vector_artifact = VectorArtifact {
        metadata: envelope("vector"),
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
        intersections: paths.iter().flat_map(segment_intersections).collect(),
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
        nodes: ["source", "vector", "mesh"]
            .into_iter()
            .map(|stage| GraphNode {
                id: format!("{}/{}", case.id, stage),
                stage: stage.to_owned(),
                status: "ready".to_owned(),
                artifact: match stage {
                    "source" => "source.svg",
                    "vector" => "vector.json",
                    "mesh" => "mesh.json",
                    _ => unreachable!(),
                }
                .to_owned(),
            })
            .collect(),
        edges: [("source", "vector"), ("vector", "mesh")]
            .into_iter()
            .map(|(from, to)| GraphEdge {
                from: format!("{}/{}", case.id, from),
                to: format!("{}/{}", case.id, to),
            })
            .collect(),
    };

    write_json(&root.join("vector.json"), &vector_artifact)?;
    write_json(&root.join("mesh.json"), &mesh_artifact)?;
    write_json(&root.join("mesh-fingerprint.json"), &mesh_fingerprint)?;
    write_json(&root.join("graph.json"), &graph_artifact)?;
    fs::write(root.join("source.svg"), source)
        .map_err(|error| format!("write W3C source artifact: {error}"))?;
    fs::write(
        root.join("contours.svg"),
        paths.iter().map(contours_svg).collect::<String>(),
    )
    .map_err(|error| format!("write W3C contour artifact: {error}"))?;
    fs::write(root.join("mesh.svg"), mesh_svg(&mesh_artifact.triangles))
        .map_err(|error| format!("write W3C mesh artifact: {error}"))?;
    Ok(root)
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|error| format!("serialize {}: {error}", path.display()))?;
    fs::write(path, format!("{json}\n"))
        .map_err(|error| format!("write {}: {error}", path.display()))
}

/// Produces an order-independent fingerprint for a flat triangle list.
///
/// Tessellators are free to emit equivalent triangles in different orders, so
/// this evidence normalizes vertex order within each triangle and triangle
/// order across the mesh. Coordinates are quantized only for the fingerprint;
/// the raw mesh artifact remains available for detailed diagnostics.
fn canonical_triangle_hash(triangles: &[[f32; 2]]) -> String {
    let mut canonical = triangles
        .chunks_exact(3)
        .map(|triangle| {
            let mut points = [
                canonical_point(triangle[0]),
                canonical_point(triangle[1]),
                canonical_point(triangle[2]),
            ];
            points.sort_by(compare_points);
            points
        })
        .collect::<Vec<_>>();
    canonical.sort_by(compare_triangles);

    let mut hash = 0xcbf29ce484222325;
    for point in canonical.into_iter().flatten() {
        for byte in point[0]
            .to_bits()
            .to_le_bytes()
            .into_iter()
            .chain(point[1].to_bits().to_le_bytes())
        {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    format!("fnv1a64:{hash:016x}")
}

fn canonical_point(point: [f32; 2]) -> [f32; 2] {
    [quantize_coordinate(point[0]), quantize_coordinate(point[1])]
}

fn quantize_coordinate(value: f32) -> f32 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

fn compare_points(left: &[f32; 2], right: &[f32; 2]) -> Ordering {
    left[0]
        .partial_cmp(&right[0])
        .unwrap_or(Ordering::Equal)
        .then_with(|| left[1].partial_cmp(&right[1]).unwrap_or(Ordering::Equal))
}

fn compare_triangles(left: &[[f32; 2]; 3], right: &[[f32; 2]; 3]) -> Ordering {
    left.iter()
        .zip(right.iter())
        .map(|(left, right)| compare_points(left, right))
        .find(|ordering| *ordering != Ordering::Equal)
        .unwrap_or(Ordering::Equal)
}

fn contours_svg(path: &ui_tools::VectorPath) -> String {
    let mut data = String::new();
    for contour in &path.contours {
        if let Some(start) = contour.points.first() {
            data.push_str(&format!("M {} {} ", start[0], start[1]));
            for point in &contour.points[1..] {
                data.push_str(&format!("L {} {} ", point[0], point[1]));
            }
            if contour.closed {
                data.push('Z');
            }
        }
    }
    format!("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1 1\"><path d=\"{data}\" fill=\"none\" stroke=\"black\"/></svg>\n")
}

fn mesh_svg(triangles: &[[f32; 2]]) -> String {
    let mut polygons = String::new();
    for triangle in triangles.chunks_exact(3) {
        polygons.push_str(&format!(
            "<polygon points=\"{},{} {},{} {},{}\"/>",
            triangle[0][0],
            triangle[0][1],
            triangle[1][0],
            triangle[1][1],
            triangle[2][0],
            triangle[2][1]
        ));
    }
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1 1\"><g fill=\"none\" stroke=\"black\">{polygons}</g></svg>\n"
    )
}

fn rasterize_mesh(triangles: &[[f32; 2]], width: u32, height: u32) -> Result<Vec<u8>, String> {
    let (min, max) =
        bounds_of_points(triangles).ok_or_else(|| "mesh has no vertices".to_owned())?;
    let span_x = (max[0] - min[0]).max(f32::EPSILON);
    let span_y = (max[1] - min[1]).max(f32::EPSILON);
    let scale = ((width - 48) as f32 / span_x).min((height - 48) as f32 / span_y);
    let offset_x = (width as f32 - span_x * scale) * 0.5;
    let offset_y = (height as f32 - span_y * scale) * 0.5;
    let map = |point: [f32; 2]| {
        [
            offset_x + (point[0] - min[0]) * scale,
            height as f32 - offset_y - (point[1] - min[1]) * scale,
        ]
    };
    let mut pixels = vec![12_u8, 15, 21, 255].repeat((width * height) as usize);
    for triangle in triangles.chunks_exact(3) {
        let points = [map(triangle[0]), map(triangle[1]), map(triangle[2])];
        let min_x = points
            .iter()
            .map(|point| point[0])
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.0) as u32;
        let max_x = points
            .iter()
            .map(|point| point[0])
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min(width as f32 - 1.0) as u32;
        let min_y = points
            .iter()
            .map(|point| point[1])
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.0) as u32;
        let max_y = points
            .iter()
            .map(|point| point[1])
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min(height as f32 - 1.0) as u32;
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if point_in_triangle([x as f32 + 0.5, y as f32 + 0.5], points) {
                    let offset = ((y * width + x) * 4) as usize;
                    pixels[offset..offset + 4].copy_from_slice(&[165, 210, 245, 255]);
                }
            }
        }
    }
    Ok(pixels)
}

fn point_in_triangle(point: [f32; 2], triangle: [[f32; 2]; 3]) -> bool {
    let edge = |a: [f32; 2], b: [f32; 2], p: [f32; 2]| {
        (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])
    };
    let a = edge(triangle[0], triangle[1], point);
    let b = edge(triangle[1], triangle[2], point);
    let c = edge(triangle[2], triangle[0], point);
    (a >= 0.0 && b >= 0.0 && c >= 0.0) || (a <= 0.0 && b <= 0.0 && c <= 0.0)
}

fn bounds_of_points(points: &[[f32; 2]]) -> Option<([f32; 2], [f32; 2])> {
    let first = *points.first()?;
    let mut min = first;
    let mut max = first;
    for point in &points[1..] {
        min[0] = min[0].min(point[0]);
        min[1] = min[1].min(point[1]);
        max[0] = max[0].max(point[0]);
        max[1] = max[1].max(point[1]);
    }
    Some((min, max))
}

fn union_bounds(first: ([f32; 2], [f32; 2]), second: ([f32; 2], [f32; 2])) -> ([f32; 2], [f32; 2]) {
    (
        [first.0[0].min(second.0[0]), first.0[1].min(second.0[1])],
        [first.1[0].max(second.1[0]), first.1[1].max(second.1[1])],
    )
}

fn signed_area(points: &[[f32; 2]]) -> f32 {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .map(|(a, b)| a[0] * b[1] - b[0] * a[1])
        .sum::<f32>()
        * 0.5
}

fn segment_intersections(path: &VectorPath) -> Vec<SegmentIntersectionArtifact> {
    let mut intersections = Vec::new();
    for (contour_index, contour) in path.contours.iter().enumerate() {
        let segment_count = if contour.closed {
            contour.points.len()
        } else {
            contour.points.len().saturating_sub(1)
        };
        for first in 0..segment_count {
            let first_start = contour.points[first];
            let first_end = contour.points[(first + 1) % contour.points.len()];
            for second in (first + 1)..segment_count {
                if second == first + 1
                    || (contour.closed && first == 0 && second + 1 == segment_count)
                {
                    continue;
                }
                let second_start = contour.points[second];
                let second_end = contour.points[(second + 1) % contour.points.len()];
                if let Some(point) =
                    line_segment_intersection(first_start, first_end, second_start, second_end)
                {
                    intersections.push(SegmentIntersectionArtifact {
                        first_contour: contour_index,
                        first_segment: first,
                        second_contour: contour_index,
                        second_segment: second,
                        point,
                    });
                }
            }
        }
    }
    intersections
}

fn line_segment_intersection(
    first_start: [f32; 2],
    first_end: [f32; 2],
    second_start: [f32; 2],
    second_end: [f32; 2],
) -> Option<[f32; 2]> {
    let first_direction = [first_end[0] - first_start[0], first_end[1] - first_start[1]];
    let second_direction = [
        second_end[0] - second_start[0],
        second_end[1] - second_start[1],
    ];
    let denominator = cross(first_direction, second_direction);
    if denominator.abs() <= f32::EPSILON {
        return None;
    }
    let offset = [
        second_start[0] - first_start[0],
        second_start[1] - first_start[1],
    ];
    let first_factor = cross(offset, second_direction) / denominator;
    let second_factor = cross(offset, first_direction) / denominator;
    if (0.000001..=0.999999).contains(&first_factor)
        && (0.000001..=0.999999).contains(&second_factor)
    {
        Some([
            first_start[0] + first_factor * first_direction[0],
            first_start[1] + first_factor * first_direction[1],
        ])
    } else {
        None
    }
}

fn cross(first: [f32; 2], second: [f32; 2]) -> f32 {
    first[0] * second[1] - first[1] * second[0]
}

fn triangle_area(triangle: &[[f32; 2]]) -> f32 {
    if triangle.len() < 3 {
        return 0.0;
    }
    ((triangle[1][0] - triangle[0][0]) * (triangle[2][1] - triangle[0][1])
        - (triangle[1][1] - triangle[0][1]) * (triangle[2][0] - triangle[0][0]))
        .abs()
        * 0.5
}

fn validate_mesh(triangles: &[[f32; 2]]) -> MeshValidation {
    let complete_triangles = triangles.len().is_multiple_of(3);
    let finite = triangles
        .iter()
        .all(|point| point[0].is_finite() && point[1].is_finite());
    let degenerate_triangles = triangles
        .chunks_exact(3)
        .filter(|triangle| triangle_area(triangle) <= f32::EPSILON)
        .count();
    let total_area = triangles.chunks_exact(3).map(triangle_area).sum();
    MeshValidation {
        finite,
        complete_triangles,
        triangle_count: triangles.len() / 3,
        degenerate_triangles,
        total_area,
    }
}

fn format_mesh_summary(validation: &MeshValidation) -> String {
    format!(
        "triangles={} vertices={} finite={} complete={} degenerate={} area={:.6}",
        validation.triangle_count,
        validation.triangle_count * 3,
        validation.finite,
        validation.complete_triangles,
        validation.degenerate_triangles,
        validation.total_area
    )
}

fn fnv1a64(bytes: &[u8], character: char) -> u64 {
    let mut hash = 0xcbf29ce484222325;
    for byte in bytes
        .iter()
        .copied()
        .chain((character as u32).to_le_bytes())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn fnv1a64_bytes(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn failed_stage(stage: CorpusStage, message: &str) -> StageReport {
    StageReport {
        stage,
        status: StageStatus::Failed,
        summary: message.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_cases_are_deterministic_and_stage_order_is_stable() {
        assert_eq!(glyph_cases()[0], GlyphCase::new("glyph/inter/K", 'K'));
        assert_eq!(
            CorpusStage::ALL.map(CorpusStage::name),
            ["source", "outline", "vector", "mesh"]
        );
    }

    #[test]
    fn mesh_fingerprint_ignores_triangle_and_vertex_order() {
        let first = [
            [0.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
        ];
        let reordered = [
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0],
            [0.0, 1.0],
            [0.0, 0.0],
            [1.0, 0.0],
        ];
        assert_eq!(
            canonical_triangle_hash(&first),
            canonical_triangle_hash(&reordered)
        );
    }

    #[test]
    fn generated_cases_replay_from_seed_and_index() {
        let first = run_generated_case(42, 7);
        let replay = run_generated_case(42, 7);
        assert_eq!(first.id, "generated/42/7");
        assert_eq!(first.producer, replay.producer);
        assert_eq!(first.stages, replay.stages);
        assert_eq!(first.diagnostics, replay.diagnostics);
        assert!(first.passed());
    }

    #[test]
    fn case_lookup_does_not_accept_unknown_cases() {
        assert!(find_glyph_case("glyph/inter/k").is_some());
        assert!(find_glyph_case("glyph/inter/unknown").is_none());
    }

    #[test]
    fn synthetic_cases_have_stable_ids_and_valid_input_paths() {
        assert_eq!(synthetic_cases().len(), 5);
        for case in synthetic_cases() {
            assert!(synthetic_path(*case).is_finite());
        }
    }

    #[test]
    fn self_intersection_is_classified_at_the_vector_boundary() {
        let report = run_synthetic_case(synthetic_cases()[4]);
        assert!(report.passed());
        assert_eq!(report.stages[1].stage, CorpusStage::Vector);
        assert_eq!(report.stages[1].status, StageStatus::ExpectedFailure);
        assert_eq!(report.stages[2].stage, CorpusStage::Mesh);
        assert_eq!(report.stages[2].status, StageStatus::ExpectedFailure);
    }

    #[test]
    fn golden_diff_reports_the_first_changed_line() {
        let diff = golden_diff("one\ntwo\n", "one\nchanged\n");
        assert!(diff.contains("line 2"));
        assert!(diff.contains("expected: two"));
        assert!(diff.contains("actual:   changed"));
    }

    #[test]
    fn svg_cases_have_stable_ids() {
        assert_eq!(svg_cases().len(), 1);
        assert_eq!(svg_cases()[0].id, "svg/lucide/archive");
        assert_eq!(
            find_case("svg/lucide/archive"),
            Some(CorpusCase::Svg(svg_cases()[0]))
        );
    }

    #[test]
    fn ui_cases_have_stable_ids() {
        assert_eq!(ui_cases().len(), 1);
        assert_eq!(
            find_case("ui/panel-surface"),
            Some(CorpusCase::Ui(ui_cases()[0]))
        );
    }

    #[test]
    fn producer_stage_selection_is_explicit() {
        assert_eq!(
            CorpusCase::Glyph(glyph_cases()[0]).selected_stages(),
            &GLYPH_STAGES
        );
        assert_eq!(
            CorpusCase::Svg(svg_cases()[0]).selected_stages(),
            &PATH_STAGES
        );
    }
}
