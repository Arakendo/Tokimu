//! Incubating, stage-aware presentation geometry corpus runner.
//!
//! The runner deliberately reports structural evidence before it writes any
//! artifacts. This keeps the first contract small while the cases are still
//! teaching us which representations need to become stable.

mod artifact_io;
mod artifacts;
mod cases;
mod catalog;
mod evidence;
mod fixture_paths;
mod generated_cases;
mod geometry;
mod glyph_artifacts;
mod glyph_cases;
mod golden_workflow;
mod goldens;
mod lucide_svg_cases;
mod reports;
mod svg_artifact_cases;
mod svg_artifact_cleanup;
mod svg_record_artifacts;
mod svg_support;
mod synthetic_cases;
mod synthetic_svg_cases;
mod ui_case;
mod w3c_svg_cases;
mod xml_stage;

pub use artifacts::{
    ArtifactAlgorithms, ArtifactEnvelope, GraphArtifact, GraphEdge, GraphNode, ImageFingerprint,
    MeshArtifact, MeshFingerprint, MeshValidation, OutlineArtifact, OutlineContourArtifact,
    OutlineSegmentArtifact, SegmentIntersectionArtifact, SvgProfileExclusionArtifact,
    VectorArtifact, VectorContourArtifact, XmlArtifact,
};
pub use cases::{
    CorpusCase, GlyphCase, SvgCase, SyntheticCase, SyntheticSvgCase, UiCase, W3cSvgCase,
    W3cSvgExpectation, W3cSvgSource,
};
pub use catalog::{
    all_cases, find_case, find_glyph_case, glyph_cases, run_case, svg_cases, synthetic_cases,
    synthetic_svg_cases, ui_cases, w3c_svg_cases,
};
pub use generated_cases::run_generated_case;
pub use glyph_artifacts::write_glyph_artifacts;
pub use glyph_cases::run_glyph_case;
pub use golden_workflow::{bless_case, compare_case, golden_snapshot_path};
pub use lucide_svg_cases::run_svg_case;
pub use reports::{CaseReport, CorpusStage, StageReport, StageStatus};
pub use svg_artifact_cases::{
    write_svg_artifacts, write_synthetic_svg_artifacts, write_w3c_artifacts,
};
use svg_record_artifacts::write_svg_record_artifacts;
pub use synthetic_cases::run_synthetic_case;
pub use synthetic_svg_cases::run_synthetic_svg_case;
pub use ui_case::run_ui_case;
pub use w3c_svg_cases::run_w3c_svg_case;

#[cfg(test)]
use synthetic_cases::synthetic_path;

#[cfg(test)]
use cases::{GLYPH_STAGES, SVG_STAGES};
#[cfg(test)]
use evidence::canonical_triangle_hash;
#[cfg(test)]
use goldens::first_difference as golden_diff;
#[cfg(test)]
use xml_stage::inspect_xml_stage;
#[cfg(test)]
use xml_tools::XmlSourceId;

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

const SYNTHETIC_SVG_CASES: [SyntheticSvgCase; 1] = [SyntheticSvgCase::new(
    "svg/synthetic/prefixed-namespace",
    "prefixed SVG namespace with a foreign local-name collision",
    r#"<s:svg xmlns:s="http://www.w3.org/2000/svg" xmlns:foreign="urn:tokimu:foreign" viewBox="0 0 24 24"><foreign:path d="M 0 0 H 24 V 24 H 0 Z"/><s:path d="M 2 2 H 22 V 22 H 2 Z"/></s:svg>"#,
)];

const W3C_SVG_CASES: [W3cSvgCase; 50] = [
    W3cSvgCase::expected_unsupported_profile(
        "svg/w3c/painting-fill-03-t",
        "painting-fill-03-t.svg",
        "W3C even-odd and non-zero fill-rule fixture with unadmitted defs/text",
    ),
    W3cSvgCase::expected_unsupported_profile(
        "svg/w3c/paths-data-16-t",
        "paths-data-16-t.svg",
        "W3C implicit line-to and relative path fixture with unadmitted defs/text",
    ),
    W3cSvgCase::expected_unsupported_profile(
        "svg/w3c/struct-group-01-t",
        "struct-group-01-t.svg",
        "W3C nested group and inherited-presentation fixture with unadmitted defs/text",
    ),
    W3cSvgCase::expected_unsupported_profile(
        "svg/w3c/coords-trans-02-t",
        "coords-trans-02-t.svg",
        "W3C transform composition fixture with unadmitted defs/text",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/paths-data-16-geometry",
        "paths-data-16-geometry.svg",
        "W3C-derived implicit line-to and relative path geometry",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/painting-fill-03-geometry",
        "painting-fill-03-geometry.svg",
        "W3C-derived even-odd and non-zero fill-rule geometry",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/paths-data-01-curves-geometry",
        "paths-data-01-curves-geometry.svg",
        "W3C-derived cubic and smooth-cubic fill geometry",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/paths-data-02-quadratics-geometry",
        "paths-data-02-quadratics-geometry.svg",
        "W3C-derived quadratic and smooth-quadratic fill geometry",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/coords-trans-02-group-geometry",
        "coords-trans-02-group-geometry.svg",
        "W3C-derived nested group transforms and inherited fill geometry",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/paths-data-03-arcs-geometry",
        "paths-data-03-arcs-geometry.svg",
        "W3C-derived closed elliptical-arc diagnostic geometry",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/shapes-polygon-01-geometry",
        "shapes-polygon-01-geometry.svg",
        "W3C-derived filled polygon geometry",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/shapes-polyline-01-geometry",
        "shapes-polyline-01-geometry.svg",
        "W3C-derived open polyline geometry",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/shapes-rect-01-geometry",
        "shapes-rect-01-geometry.svg",
        "W3C-derived rectangle and rounded-rectangle fill geometry",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/shapes-circle-01-geometry",
        "shapes-circle-01-geometry.svg",
        "W3C-derived circle fill geometry",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/shapes-ellipse-01-geometry",
        "shapes-ellipse-01-geometry.svg",
        "W3C-derived ellipse fill geometry",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/shapes-line-01-geometry",
        "shapes-line-01-geometry.svg",
        "W3C-derived open line geometry",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/shapes-line-02-stroke-geometry",
        "shapes-line-02-stroke-geometry.svg",
        "W3C-derived line, polyline, and open-path stroke intent",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/paths-data-10-stroke-geometry",
        "paths-data-10-stroke-geometry.svg",
        "W3C-derived open and closed path cap and join geometry",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/shapes-polyline-02-geometry",
        "shapes-polyline-02-geometry.svg",
        "W3C-derived polyline and path-equivalence geometry",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/coords-trans-03-elementary-geometry",
        "coords-trans-03-elementary-geometry.svg",
        "W3C-derived elementary and nested transform composition",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/coords-trans-04-stroke-geometry",
        "coords-trans-04-stroke-geometry.svg",
        "W3C-derived transformed open stroke geometry",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/coords-trans-05-reflection-geometry",
        "coords-trans-05-reflection-geometry.svg",
        "W3C-derived reflected geometry with a non-zero root viewBox origin",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/shapes-line-03-dash-geometry",
        "shapes-line-03-dash-geometry.svg",
        "W3C-derived dashed stroke geometry and dash phase",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/clip-rect-geometry",
        "clip-rect-geometry.svg",
        "W3C-derived rectangular clip intersection and fill mesh",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/clip-polygon-geometry",
        "clip-polygon-geometry.svg",
        "W3C-derived convex polygon clip intersection and fill mesh",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/clip-transformed-polygon-geometry",
        "clip-transformed-polygon-geometry.svg",
        "W3C-derived transformed convex polygon clip intersection and fill mesh",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/struct-group-01-inheritance-geometry",
        "struct-group-01-inheritance-geometry.svg",
        "W3C-derived group paint inheritance and child overrides",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/painting-fill-04-inheritance-geometry",
        "painting-fill-04-inheritance-geometry.svg",
        "W3C-derived nested fill, stroke, and stroke-width inheritance",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/painting-fill-02-current-color-geometry",
        "painting-fill-02-current-color-geometry.svg",
        "W3C-derived currentColor fill resolution through inherited and local color",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/painting-stroke-08-opacity-geometry",
        "painting-stroke-08-opacity-geometry.svg",
        "W3C-derived in-range stroke-opacity paint intent",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/paths-data-04-geometry",
        "paths-data-04-geometry.svg",
        "W3C-derived explicit line and close path geometry",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/paths-data-05-geometry",
        "paths-data-05-geometry.svg",
        "W3C-derived relative line and nested contour geometry",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/paths-data-06-geometry",
        "paths-data-06-geometry.svg",
        "W3C-derived absolute and relative horizontal/vertical path geometry",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/paths-data-07-geometry",
        "paths-data-07-geometry.svg",
        "W3C-derived relative horizontal/vertical stepped geometry",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/paths-data-08-geometry",
        "paths-data-08-geometry.svg",
        "W3C-derived implicit line pairs after move commands",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/paths-data-09-geometry",
        "paths-data-09-geometry.svg",
        "W3C-derived relative implicit line pairs and nested contours",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/paths-data-13-geometry",
        "paths-data-13-geometry.svg",
        "W3C-derived repeated horizontal and vertical command arguments",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/paths-data-12-geometry",
        "paths-data-12-geometry.svg",
        "W3C-derived repeated cubic and smooth-cubic geometry",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/paths-data-14-geometry",
        "paths-data-14-geometry.svg",
        "W3C-derived relative implicit line pairs and subpaths",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/paths-data-15-geometry",
        "paths-data-15-geometry.svg",
        "W3C-derived repeated quadratic and smooth-quadratic geometry",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/shapes-rect-02-geometry",
        "shapes-rect-02-geometry.svg",
        "W3C-derived rectangle corner-radius coupling",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/shapes-rect-03-geometry",
        "shapes-rect-03-geometry.svg",
        "W3C-derived rectangle corner-radius clamping",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/shapes-circle-02-geometry",
        "shapes-circle-02-geometry.svg",
        "W3C-derived circle default-coordinate behavior",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/shapes-polygon-02-geometry",
        "shapes-polygon-02-geometry.svg",
        "W3C-derived concave and star polygon fill geometry",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/shapes-ellipse-02-geometry",
        "shapes-ellipse-02-geometry.svg",
        "W3C-derived ellipse default-coordinate behavior",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/struct-use-01-geometry",
        "struct-use-01-geometry.svg",
        "W3C-derived bounded local href and xlink:href geometry reuse",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/struct-use-01-placement-geometry",
        "struct-use-01-placement-geometry.svg",
        "W3C-derived local href and xlink:href placement through x and y",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/masking-path-01-circle-clip-geometry",
        "masking-path-01-circle-clip-geometry.svg",
        "W3C-derived user-space circular clip intersection for a rectangular target",
    ),
    W3cSvgCase::derived_profile_fixture(
        "svg/w3c-derived/masking-path-02-curve-clip-geometry",
        "masking-path-02-curve-clip-geometry.svg",
        "W3C-derived cubic closed fill clipped by a user-space rectangle",
    ),
    W3cSvgCase::expected_invalid_input(
        "svg/w3c-derived/shapes-polygon-03-geometry",
        "shapes-polygon-03-geometry.svg",
        "W3C-derived rejection of odd polyline coordinate lists",
    ),
];

const UI_CASES: [UiCase; 1] = [UiCase::new("ui/panel-surface", "default panel surface")];

const ALL_CASES: [CorpusCase; 62] = [
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
    CorpusCase::SyntheticSvg(SYNTHETIC_SVG_CASES[0]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[0]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[1]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[2]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[3]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[4]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[5]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[6]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[7]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[8]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[9]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[10]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[11]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[12]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[13]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[14]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[15]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[16]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[17]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[18]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[19]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[20]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[21]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[22]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[23]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[24]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[25]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[26]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[27]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[28]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[29]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[30]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[31]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[32]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[33]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[34]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[35]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[36]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[37]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[38]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[39]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[40]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[41]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[42]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[43]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[44]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[45]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[46]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[47]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[48]),
    CorpusCase::W3cSvg(W3C_SVG_CASES[49]),
    CorpusCase::Ui(UI_CASES[0]),
];

#[cfg(test)]
mod tests;
