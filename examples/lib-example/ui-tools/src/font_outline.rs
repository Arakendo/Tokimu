use ttf_parser::OutlineBuilder;

use crate::{
    tessellate_general_fill_with_rule, UiFontRasterizer, UiRasterTextGlyph, VectorContour,
    VectorFillRule, VectorPath,
};

const POINT_EPSILON: f32 = 1.0e-4;

#[derive(Clone, Debug, PartialEq)]
pub struct UiGlyphOutline {
    pub character: char,
    pub units_per_em: f32,
    pub contours: Vec<UiGlyphContour>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiGlyphContour {
    pub start: [f32; 2],
    pub segments: Vec<UiGlyphOutlineSegment>,
    pub closed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiGlyphOutlineSegment {
    LineTo([f32; 2]),
    QuadTo {
        control: [f32; 2],
        end: [f32; 2],
    },
    CubicTo {
        control1: [f32; 2],
        control2: [f32; 2],
        end: [f32; 2],
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiGlyphOutlineDiagnosticKind {
    MissingOutline,
    InvalidUnitsPerEm,
    NonFiniteCoordinate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiGlyphOutlineDiagnostic {
    pub kind: UiGlyphOutlineDiagnosticKind,
    pub character: char,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiGlyphVectorOptions {
    /// Output-space size of one em.
    pub units_per_em_scale: f32,
    /// Output-space glyph origin, normally the positioned baseline pen.
    pub origin: [f32; 2],
    /// Maximum accepted curve-to-chord deviation in output coordinates.
    pub flatten_tolerance: f32,
    /// Negate native font y coordinates for top-left-origin presentation.
    pub flip_y: bool,
}

impl UiGlyphVectorOptions {
    pub fn new(units_per_em_scale: f32, origin: [f32; 2], flatten_tolerance: f32) -> Self {
        Self {
            units_per_em_scale,
            origin,
            flatten_tolerance,
            flip_y: false,
        }
    }

    pub fn with_flipped_y(mut self, flip_y: bool) -> Self {
        self.flip_y = flip_y;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiGlyphVectorDiagnosticKind {
    InvalidScale,
    InvalidTolerance,
    NonFiniteOrigin,
    InvalidOutline,
    MissingOutline,
    UnsupportedTopology,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiGlyphVectorDiagnostic {
    pub kind: UiGlyphVectorDiagnosticKind,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiGlyphFillTopology {
    SingleConvexContour,
    SingleConcaveContour,
    MultipleContours,
    Invalid,
}

impl UiGlyphOutlineDiagnostic {
    fn new(
        kind: UiGlyphOutlineDiagnosticKind,
        character: char,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            character,
            message: message.into(),
        }
    }
}

impl UiGlyphOutlineSegment {
    pub fn end(&self) -> [f32; 2] {
        match self {
            Self::LineTo(end) | Self::QuadTo { end, .. } | Self::CubicTo { end, .. } => *end,
        }
    }

    pub fn is_finite(&self) -> bool {
        match self {
            Self::LineTo(end) => is_finite(*end),
            Self::QuadTo { control, end } => is_finite(*control) && is_finite(*end),
            Self::CubicTo {
                control1,
                control2,
                end,
            } => is_finite(*control1) && is_finite(*control2) && is_finite(*end),
        }
    }
}

impl UiGlyphContour {
    pub fn is_finite(&self) -> bool {
        is_finite(self.start) && self.segments.iter().all(UiGlyphOutlineSegment::is_finite)
    }
}

impl UiGlyphOutline {
    pub fn is_finite(&self) -> bool {
        self.units_per_em.is_finite()
            && self.units_per_em > 0.0
            && self.contours.iter().all(UiGlyphContour::is_finite)
    }

    /// Lowers this font-provider result into shared vector geometry.
    ///
    /// Text layout supplies `origin`; this adapter never derives placement from
    /// outline bounds. Curves are flattened with the caller's declared
    /// output-space tolerance so the policy remains observable.
    pub fn to_vector_path(
        &self,
        options: UiGlyphVectorOptions,
    ) -> Result<VectorPath, UiGlyphVectorDiagnostic> {
        if !self.is_finite() {
            return Err(vector_diagnostic(
                UiGlyphVectorDiagnosticKind::InvalidOutline,
                "glyph outline is empty, non-finite, or has invalid units per em",
            ));
        }
        if !options.units_per_em_scale.is_finite() || options.units_per_em_scale <= 0.0 {
            return Err(vector_diagnostic(
                UiGlyphVectorDiagnosticKind::InvalidScale,
                "glyph vector scale must be finite and greater than zero",
            ));
        }
        if !options.flatten_tolerance.is_finite() || options.flatten_tolerance <= 0.0 {
            return Err(vector_diagnostic(
                UiGlyphVectorDiagnosticKind::InvalidTolerance,
                "glyph curve tolerance must be finite and greater than zero",
            ));
        }
        if !is_finite(options.origin) {
            return Err(vector_diagnostic(
                UiGlyphVectorDiagnosticKind::NonFiniteOrigin,
                "glyph vector origin must be finite",
            ));
        }

        let scale = options.units_per_em_scale / self.units_per_em;
        let transform = |point| transform_font_point(point, scale, options.origin, options.flip_y);
        let contours = self
            .contours
            .iter()
            .map(|contour| {
                let mut points = vec![transform(contour.start)];
                let mut current = contour.start;
                for segment in &contour.segments {
                    match segment {
                        UiGlyphOutlineSegment::LineTo(end) => points.push(transform(*end)),
                        UiGlyphOutlineSegment::QuadTo { control, end } => flatten_quad(
                            transform(current),
                            transform(*control),
                            transform(*end),
                            options.flatten_tolerance,
                            &mut points,
                            0,
                        ),
                        UiGlyphOutlineSegment::CubicTo {
                            control1,
                            control2,
                            end,
                        } => flatten_cubic(
                            transform(current),
                            transform(*control1),
                            transform(*control2),
                            transform(*end),
                            options.flatten_tolerance,
                            &mut points,
                            0,
                        ),
                    }
                    current = segment.end();
                }
                if contour.closed
                    && points
                        .last()
                        .is_some_and(|end| points_approximately_equal(*end, points[0]))
                {
                    points.pop();
                }
                VectorContour::new(points, contour.closed)
            })
            .collect();

        Ok(VectorPath::new(contours))
    }

    /// Classifies the current bounded fill contract after outline conversion.
    /// This is diagnostic only; it does not claim unsupported glyph topology
    /// can already be tessellated.
    pub fn fill_topology(
        &self,
        options: UiGlyphVectorOptions,
    ) -> Result<UiGlyphFillTopology, UiGlyphVectorDiagnostic> {
        let path = self.to_vector_path(options)?;
        if path.contours.len() != 1 {
            return Ok(UiGlyphFillTopology::MultipleContours);
        }
        match crate::validate_convex_fill(&path) {
            Ok(()) => Ok(UiGlyphFillTopology::SingleConvexContour),
            Err(message) if message.contains("concave") => {
                Ok(UiGlyphFillTopology::SingleConcaveContour)
            }
            Err(_) => Ok(UiGlyphFillTopology::Invalid),
        }
    }
}

impl UiFontRasterizer {
    /// Converts one already-positioned layout glyph into renderer-neutral fill
    /// triangles. The caller owns the baseline origin and output scale; this
    /// method never derives placement from the outline bounds.
    pub fn tessellate_positioned_glyph(
        &self,
        positioned: &UiRasterTextGlyph,
        font_pixels: f32,
        output_units_per_pixel: f32,
        baseline_origin: [f32; 2],
        flatten_tolerance: f32,
    ) -> Result<Vec<[f32; 2]>, UiGlyphVectorDiagnostic> {
        if !font_pixels.is_finite() || font_pixels <= 0.0 {
            return Err(vector_diagnostic(
                UiGlyphVectorDiagnosticKind::InvalidScale,
                "font pixel size must be finite and greater than zero",
            ));
        }
        if !output_units_per_pixel.is_finite() || output_units_per_pixel <= 0.0 {
            return Err(vector_diagnostic(
                UiGlyphVectorDiagnosticKind::InvalidScale,
                "glyph output scale must be finite and greater than zero",
            ));
        }
        let origin = [
            baseline_origin[0] + positioned.pen_x * output_units_per_pixel,
            baseline_origin[1],
        ];
        let outline = self
            .outline(positioned.glyph.character)
            .map_err(|diagnostic| {
                vector_diagnostic(
                    match diagnostic.kind {
                        UiGlyphOutlineDiagnosticKind::MissingOutline => {
                            UiGlyphVectorDiagnosticKind::MissingOutline
                        }
                        _ => UiGlyphVectorDiagnosticKind::InvalidOutline,
                    },
                    diagnostic.message,
                )
            })?;
        let path = outline.to_vector_path(UiGlyphVectorOptions::new(
            output_units_per_pixel * font_pixels,
            origin,
            flatten_tolerance,
        ))?;
        // Flattened font contours can contain shared or crossing edges even
        // when the source outline is visually unambiguous. Even-odd fill is
        // stable for those provider contours and keeps this recovery policy
        // local to fonts; the general vector API remains non-zero by default.
        tessellate_general_fill_with_rule(&path, VectorFillRule::EvenOdd).map_err(|message| {
            vector_diagnostic(UiGlyphVectorDiagnosticKind::UnsupportedTopology, message)
        })
    }

    /// Extracts an unscaled, provider-neutral monochrome outline.
    ///
    /// Despite this type's historical name, outline extraction does not
    /// rasterize or require a renderer. Coordinates remain in font units with
    /// their native y-up orientation; a presentation adapter owns scaling and
    /// coordinate-system conversion.
    pub fn outline(&self, character: char) -> Result<UiGlyphOutline, UiGlyphOutlineDiagnostic> {
        let face = ttf_parser::Face::parse(&self.font_bytes, 0).map_err(|error| {
            UiGlyphOutlineDiagnostic::new(
                UiGlyphOutlineDiagnosticKind::InvalidUnitsPerEm,
                character,
                format!("font provider could not parse outline data: {error:?}"),
            )
        })?;
        let units_per_em = face.units_per_em() as f32;
        if !units_per_em.is_finite() || units_per_em <= 0.0 {
            return Err(UiGlyphOutlineDiagnostic::new(
                UiGlyphOutlineDiagnosticKind::InvalidUnitsPerEm,
                character,
                "font provider reported invalid units per em",
            ));
        }

        let glyph_id = face.glyph_index(character).ok_or_else(|| {
            UiGlyphOutlineDiagnostic::new(
                UiGlyphOutlineDiagnosticKind::MissingOutline,
                character,
                "font provider did not supply a monochrome outline",
            )
        })?;
        let mut builder = GlyphOutlineBuilder::default();
        if face.outline_glyph(glyph_id, &mut builder).is_none() {
            return Err(UiGlyphOutlineDiagnostic::new(
                UiGlyphOutlineDiagnosticKind::MissingOutline,
                character,
                "font provider did not supply a monochrome outline",
            ));
        }
        let contours = builder.finish();
        if contours.iter().any(|contour| !contour.is_finite()) {
            return Err(UiGlyphOutlineDiagnostic::new(
                UiGlyphOutlineDiagnosticKind::NonFiniteCoordinate,
                character,
                "font provider supplied non-finite outline coordinates",
            ));
        }

        Ok(UiGlyphOutline {
            character,
            units_per_em,
            contours,
        })
    }
}

#[derive(Default)]
struct GlyphOutlineBuilder {
    contours: Vec<UiGlyphContour>,
    current: Option<UiGlyphContour>,
}

impl GlyphOutlineBuilder {
    fn finish(mut self) -> Vec<UiGlyphContour> {
        self.finish_current(false);
        self.contours
    }

    fn finish_current(&mut self, closed: bool) {
        let Some(mut contour) = self.current.take() else {
            return;
        };
        contour.closed = closed;
        if !contour.segments.is_empty() {
            self.contours.push(contour);
        }
    }

    fn push_segment(&mut self, segment: UiGlyphOutlineSegment) {
        if let Some(contour) = self.current.as_mut() {
            contour.segments.push(segment);
        }
    }
}

impl OutlineBuilder for GlyphOutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.finish_current(false);
        self.current = Some(UiGlyphContour {
            start: [x, y],
            segments: Vec::new(),
            closed: false,
        });
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.push_segment(UiGlyphOutlineSegment::LineTo([x, y]));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.push_segment(UiGlyphOutlineSegment::QuadTo {
            control: [x1, y1],
            end: [x, y],
        });
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.push_segment(UiGlyphOutlineSegment::CubicTo {
            control1: [x1, y1],
            control2: [x2, y2],
            end: [x, y],
        });
    }

    fn close(&mut self) {
        self.finish_current(true);
    }
}

fn is_finite(point: [f32; 2]) -> bool {
    point[0].is_finite() && point[1].is_finite()
}

fn points_approximately_equal(left: [f32; 2], right: [f32; 2]) -> bool {
    (left[0] - right[0]).abs() <= POINT_EPSILON && (left[1] - right[1]).abs() <= POINT_EPSILON
}

fn vector_diagnostic(
    kind: UiGlyphVectorDiagnosticKind,
    message: impl Into<String>,
) -> UiGlyphVectorDiagnostic {
    UiGlyphVectorDiagnostic {
        kind,
        message: message.into(),
    }
}

fn transform_font_point(point: [f32; 2], scale: f32, origin: [f32; 2], flip_y: bool) -> [f32; 2] {
    let y = if flip_y { -point[1] } else { point[1] };
    [origin[0] + point[0] * scale, origin[1] + y * scale]
}

fn flatten_quad(
    start: [f32; 2],
    control: [f32; 2],
    end: [f32; 2],
    tolerance: f32,
    output: &mut Vec<[f32; 2]>,
    depth: u8,
) {
    if depth >= 16 || point_line_distance(control, start, end) <= tolerance {
        output.push(end);
        return;
    }
    let start_control = midpoint(start, control);
    let control_end = midpoint(control, end);
    let center = midpoint(start_control, control_end);
    flatten_quad(start, start_control, center, tolerance, output, depth + 1);
    flatten_quad(center, control_end, end, tolerance, output, depth + 1);
}

#[allow(clippy::too_many_arguments)]
fn flatten_cubic(
    start: [f32; 2],
    control1: [f32; 2],
    control2: [f32; 2],
    end: [f32; 2],
    tolerance: f32,
    output: &mut Vec<[f32; 2]>,
    depth: u8,
) {
    let deviation =
        point_line_distance(control1, start, end).max(point_line_distance(control2, start, end));
    if depth >= 16 || deviation <= tolerance {
        output.push(end);
        return;
    }
    let p01 = midpoint(start, control1);
    let p12 = midpoint(control1, control2);
    let p23 = midpoint(control2, end);
    let p012 = midpoint(p01, p12);
    let p123 = midpoint(p12, p23);
    let center = midpoint(p012, p123);
    flatten_cubic(start, p01, p012, center, tolerance, output, depth + 1);
    flatten_cubic(center, p123, p23, end, tolerance, output, depth + 1);
}

fn midpoint(left: [f32; 2], right: [f32; 2]) -> [f32; 2] {
    [(left[0] + right[0]) * 0.5, (left[1] + right[1]) * 0.5]
}

fn point_line_distance(point: [f32; 2], start: [f32; 2], end: [f32; 2]) -> f32 {
    let line = [end[0] - start[0], end[1] - start[1]];
    let length_squared = line[0] * line[0] + line[1] * line[1];
    if length_squared <= f32::EPSILON {
        return ((point[0] - start[0]).powi(2) + (point[1] - start[1]).powi(2)).sqrt();
    }
    let area_twice = (line[0] * (start[1] - point[1]) - (start[0] - point[0]) * line[1]).abs();
    area_twice / length_squared.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noto_fixture() -> UiFontRasterizer {
        UiFontRasterizer::from_bytes(include_bytes!("../fixtures/NotoSans-Regular.otf").to_vec())
            .expect("checked-in OTF fixture")
    }

    fn prepared_inter_fixture() -> Option<UiFontRasterizer> {
        let source =
            crate::UiFontSource::from_prepared_corpus("inter", crate::UiFontFormat::Ttf).ok()?;
        UiFontRasterizer::from_bytes(source.bytes).ok()
    }

    fn prepared_provider_fixture(provider: &str, candidates: &[&str]) -> Option<UiFontRasterizer> {
        candidates.iter().find_map(|filename| {
            let path = format!("../../../../target/glyph-corpus/fonts/{provider}/{filename}");
            let bytes = std::fs::read(path).ok()?;
            UiFontRasterizer::from_bytes(bytes).ok()
        })
    }

    #[test]
    fn extracts_provider_neutral_otf_outline() {
        let outline = noto_fixture().outline('A').expect("A outline");

        assert_eq!(outline.character, 'A');
        assert!(outline.units_per_em > 0.0);
        assert!(!outline.contours.is_empty());
        assert!(outline.contours.iter().all(|contour| contour.closed));
        assert!(outline.is_finite());
    }

    #[test]
    fn prepared_ttf_uses_the_same_outline_contract() {
        let Some(font) = prepared_inter_fixture() else {
            return;
        };
        let outline = font.outline('A').expect("Inter A outline");

        assert!(outline.is_finite());
        assert!(!outline.contours.is_empty());
        assert!(outline.contours.iter().all(|contour| contour.closed));
    }

    #[test]
    fn prepared_font_providers_share_the_outline_contract() {
        let providers = [
            (
                "inter",
                ["Inter[opsz,wght].ttf", "Inter-Regular.ttf"].as_slice(),
            ),
            (
                "jetbrains-mono",
                ["JetBrainsMono-Regular.otf", "JetBrainsMono-Regular.ttf"].as_slice(),
            ),
            (
                "noto",
                ["NotoSans-VF.ttf", "NotoSans-Regular.ttf"].as_slice(),
            ),
        ];

        for (provider, candidates) in providers {
            let Some(font) = prepared_provider_fixture(provider, candidates) else {
                continue;
            };
            let outline = font
                .outline('A')
                .unwrap_or_else(|error| panic!("{provider} A outline failed: {error:?}"));

            assert!(outline.is_finite(), "{provider} outline must be finite");
            assert!(!outline.contours.is_empty(), "{provider} needs contours");
            assert!(
                outline.contours.iter().all(|contour| contour.closed),
                "{provider} contours must be closed"
            );
        }
    }

    #[test]
    fn preserves_multiple_contours_for_counter_glyph() {
        let outline = noto_fixture().outline('O').expect("O outline");

        assert!(outline.contours.len() >= 2);
        assert!(outline.contours.iter().all(|contour| contour.closed));
    }

    #[test]
    fn preserves_native_curve_segments() {
        let outline = noto_fixture().outline('S').expect("S outline");

        assert!(outline.contours.iter().any(|contour| {
            contour.segments.iter().any(|segment| {
                matches!(
                    segment,
                    UiGlyphOutlineSegment::QuadTo { .. } | UiGlyphOutlineSegment::CubicTo { .. }
                )
            })
        }));
    }

    #[test]
    fn whitespace_reports_missing_outline() {
        let error = noto_fixture().outline(' ').expect_err("space has no ink");

        assert_eq!(error.kind, UiGlyphOutlineDiagnosticKind::MissingOutline);
        assert_eq!(error.character, ' ');
    }

    #[test]
    fn adapter_scales_and_positions_without_using_outline_bounds() {
        let outline = noto_fixture().outline('A').expect("A outline");
        let unshifted = outline
            .to_vector_path(UiGlyphVectorOptions::new(32.0, [0.0, 0.0], 0.25))
            .expect("unshifted vector path");
        let shifted = outline
            .to_vector_path(UiGlyphVectorOptions::new(32.0, [10.0, 20.0], 0.25))
            .expect("shifted vector path");
        let original_bounds = unshifted.bounds().expect("unshifted bounds");
        let shifted_bounds = shifted.bounds().expect("shifted bounds");

        assert_eq!(shifted.contours.len(), outline.contours.len());
        assert!((shifted_bounds.0[0] - original_bounds.0[0] - 10.0).abs() < 1.0e-3);
        assert!((shifted_bounds.0[1] - original_bounds.0[1] - 20.0).abs() < 1.0e-3);
        assert!(shifted.is_finite());
    }

    #[test]
    fn adapter_preserves_counter_contours_and_closure() {
        let outline = noto_fixture().outline('O').expect("O outline");
        let path = outline
            .to_vector_path(UiGlyphVectorOptions::new(48.0, [0.0, 0.0], 0.2))
            .expect("vector path");

        assert!(path.contours.len() >= 2);
        assert!(path.contours.iter().all(|contour| contour.closed));
        assert!(path
            .contours
            .iter()
            .all(|contour| contour.points.len() >= 3));
    }

    #[test]
    fn smaller_tolerance_produces_at_least_as_many_curve_points() {
        let outline = noto_fixture().outline('S').expect("S outline");
        let coarse = outline
            .to_vector_path(UiGlyphVectorOptions::new(64.0, [0.0, 0.0], 1.0))
            .expect("coarse vector path");
        let fine = outline
            .to_vector_path(UiGlyphVectorOptions::new(64.0, [0.0, 0.0], 0.1))
            .expect("fine vector path");
        let point_count = |path: &VectorPath| {
            path.contours
                .iter()
                .map(|contour| contour.points.len())
                .sum::<usize>()
        };

        assert!(point_count(&fine) >= point_count(&coarse));
    }

    #[test]
    fn adapter_rejects_invalid_presentation_policy() {
        let outline = noto_fixture().outline('A').expect("A outline");
        let error = outline
            .to_vector_path(UiGlyphVectorOptions::new(0.0, [0.0, 0.0], 0.25))
            .expect_err("zero scale must fail");

        assert_eq!(error.kind, UiGlyphVectorDiagnosticKind::InvalidScale);
    }

    #[test]
    fn fill_topology_reports_counter_and_convex_glyphs_before_tessellation() {
        let options = UiGlyphVectorOptions::new(48.0, [0.0, 0.0], 0.2);
        assert_eq!(
            noto_fixture().outline('O').unwrap().fill_topology(options),
            Ok(UiGlyphFillTopology::MultipleContours)
        );
        assert_eq!(
            noto_fixture().outline('A').unwrap().fill_topology(options),
            Ok(UiGlyphFillTopology::MultipleContours)
        );
    }

    #[test]
    fn counter_glyph_can_use_general_fill_adapter() {
        let outline = noto_fixture().outline('O').expect("O outline");
        let path = outline
            .to_vector_path(UiGlyphVectorOptions::new(64.0, [0.0, 0.0], 0.2))
            .expect("O vector path");
        let triangles = crate::tessellate_general_fill(&path).expect("O fill");

        assert!(!triangles.is_empty());
        assert!(triangles
            .iter()
            .all(|point| { point[0].is_finite() && point[1].is_finite() }));

        let (min, max) = path.bounds().expect("O path bounds");
        assert!(triangles.iter().all(|point| {
            point[0] >= min[0] - POINT_EPSILON
                && point[0] <= max[0] + POINT_EPSILON
                && point[1] >= min[1] - POINT_EPSILON
                && point[1] <= max[1] + POINT_EPSILON
        }));
    }

    #[test]
    fn counter_corpus_glyphs_lower_through_general_fill() {
        let font = noto_fixture();
        for character in ['B', 'P', 'Q', 'a', 'e', 'g', '0', '8', '@'] {
            let outline = font.outline(character).expect("counter glyph outline");
            let path = outline
                .to_vector_path(UiGlyphVectorOptions::new(56.0, [0.0, 0.0], 0.2))
                .expect("counter glyph vector path");
            let triangles = crate::tessellate_general_fill(&path)
                .unwrap_or_else(|error| panic!("{character} fill failed: {error}"));
            assert!(
                !triangles.is_empty(),
                "{character} should produce fill geometry"
            );
            assert!(triangles
                .iter()
                .all(|point| point[0].is_finite() && point[1].is_finite()));
        }
    }

    #[test]
    fn prepared_inter_hard_edge_glyphs_preserve_contour_area() {
        let Some(font) = prepared_inter_fixture() else {
            return;
        };
        for character in [
            '%', '&', '2', '4', 'F', 'K', 'M', 'N', 'W', 'X', 'Z', 'h', 'k', 'r',
        ] {
            let path = font
                .outline(character)
                .expect("Inter outline")
                .to_vector_path(UiGlyphVectorOptions::new(96.0, [0.0, 0.0], 0.2))
                .expect("vector path");
            let contour_area = path
                .contours
                .iter()
                .map(|contour| polygon_area(&contour.points))
                .sum::<f32>()
                .abs();
            let triangles = crate::tessellate_general_fill(&path)
                .unwrap_or_else(|error| panic!("{character} fill failed: {error}"));
            let tessellated_area = triangles
                .chunks_exact(3)
                .map(|triangle| triangle_area(triangle[0], triangle[1], triangle[2]).abs())
                .sum::<f32>();
            let winding_signs = triangles
                .chunks_exact(3)
                .map(|triangle| triangle_area(triangle[0], triangle[1], triangle[2]).signum())
                .filter(|sign| *sign != 0.0)
                .collect::<Vec<_>>();
            let relative_area_error = (tessellated_area - contour_area).abs() / contour_area;
            assert!(
                relative_area_error < 0.08,
                "{character} fill area drifted by {relative_area_error}"
            );
            assert!(
                winding_signs.iter().all(|sign| *sign == winding_signs[0]),
                "{character} produced mixed triangle winding: {winding_signs:?}"
            );
            assert_mesh_matches_non_zero_fill(character, &path, &triangles);
        }
    }

    #[test]
    fn prepared_inter_regression_glyphs_preserve_positioned_mesh_coverage() {
        let Some(font) = prepared_inter_fixture() else {
            return;
        };

        for character in ['F', 'K', 'k', 'M', 'e'] {
            let outline = font
                .outline(character)
                .unwrap_or_else(|error| panic!("{character} outline failed: {error:?}"));
            assert!(outline.is_finite(), "{character} outline must be finite");
            assert!(!outline.contours.is_empty(), "{character} needs contours");
            assert!(
                outline.contours.iter().all(|contour| contour.closed),
                "{character} contours must be closed"
            );

            let path = outline
                .to_vector_path(UiGlyphVectorOptions::new(96.0, [0.0, 0.0], 0.2))
                .unwrap_or_else(|error| panic!("{character} vector conversion failed: {error:?}"));
            let layout = font.layout(&character.to_string(), 96.0);
            let positioned = layout
                .glyphs
                .first()
                .unwrap_or_else(|| panic!("{character} produced no positioned glyph"));
            let triangles = font
                .tessellate_positioned_glyph(positioned, 96.0, 1.0, [0.0, 0.0], 0.2)
                .unwrap_or_else(|error| panic!("{character} tessellation failed: {error:?}"));

            assert!(!triangles.is_empty(), "{character} needs fill geometry");
            assert!(triangles
                .iter()
                .all(|point| point[0].is_finite() && point[1].is_finite()));
            assert_mesh_matches_fill_rule(character, &path, &triangles, true);
        }
    }

    #[test]
    fn positioned_glyph_adapter_uses_layout_pen_position() {
        let font = noto_fixture();
        let layout = font.layout("AA", 48.0);
        let first = font
            .tessellate_positioned_glyph(&layout.glyphs[0], 48.0, 0.01, [-1.0, 0.5], 0.2)
            .expect("first positioned glyph");
        let second = font
            .tessellate_positioned_glyph(&layout.glyphs[1], 48.0, 0.01, [-1.0, 0.5], 0.2)
            .expect("second positioned glyph");

        assert!(!first.is_empty());
        assert!(!second.is_empty());
        let first_min_x = first
            .iter()
            .map(|point| point[0])
            .fold(f32::INFINITY, f32::min);
        let second_min_x = second
            .iter()
            .map(|point| point[0])
            .fold(f32::INFINITY, f32::min);
        let expected_delta = (layout.glyphs[1].pen_x - layout.glyphs[0].pen_x) * 0.01;
        assert!(second_min_x > first_min_x);
        assert!((second_min_x - first_min_x - expected_delta).abs() < 0.0005);
        assert!(first
            .iter()
            .chain(second.iter())
            .all(|point| point[0].is_finite() && point[1].is_finite()));
    }

    #[test]
    fn positioned_glyph_scales_across_ui_sizes_without_invalid_geometry() {
        let font = noto_fixture();
        let mut triangle_counts = Vec::new();
        for pixels in [24.0_f32, 56.0, 96.0] {
            let layout = font.layout("O", pixels);
            let triangles = font
                .tessellate_positioned_glyph(&layout.glyphs[0], pixels, 0.01, [0.0, 0.0], 0.2)
                .unwrap_or_else(|error| panic!("{pixels}px glyph failed: {error:?}"));

            assert!(!triangles.is_empty(), "{pixels}px glyph should be visible");
            assert!(triangles
                .iter()
                .all(|point| { point[0].is_finite() && point[1].is_finite() }));
            triangle_counts.push(triangles.len() / 3);
        }

        assert!(triangle_counts[1] >= triangle_counts[0]);
        assert!(triangle_counts[2] >= triangle_counts[1]);
    }

    #[test]
    fn positioned_glyph_output_scale_changes_geometry_not_layout_input() {
        let font = noto_fixture();
        let layout = font.layout("O", 48.0);
        let small = font
            .tessellate_positioned_glyph(&layout.glyphs[0], 48.0, 0.01, [0.0, 0.0], 0.2)
            .expect("small output scale");
        let large = font
            .tessellate_positioned_glyph(&layout.glyphs[0], 48.0, 0.02, [0.0, 0.0], 0.2)
            .expect("large output scale");

        let bounds = |points: &[[f32; 2]]| {
            points.iter().fold(
                ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]),
                |(mut min, mut max), point| {
                    min[0] = min[0].min(point[0]);
                    min[1] = min[1].min(point[1]);
                    max[0] = max[0].max(point[0]);
                    max[1] = max[1].max(point[1]);
                    (min, max)
                },
            )
        };
        let (small_min, small_max) = bounds(&small);
        let (large_min, large_max) = bounds(&large);
        let small_width = small_max[0] - small_min[0];
        let large_width = large_max[0] - large_min[0];

        assert!(small_width > 0.0);
        assert!((large_width / small_width - 2.0).abs() < 0.05);
        assert_eq!(layout.glyphs[0].pen_x, 0.0);
    }

    #[test]
    fn positioned_glyph_tolerance_is_in_output_units() {
        let font = noto_fixture();
        let layout = font.layout("O", 48.0);
        let low_scale = font
            .tessellate_positioned_glyph(&layout.glyphs[0], 48.0, 0.01, [0.0, 0.0], 0.2)
            .expect("low-scale glyph");
        let high_scale = font
            .tessellate_positioned_glyph(&layout.glyphs[0], 48.0, 0.02, [0.0, 0.0], 0.4)
            .expect("high-scale glyph");

        assert!(!low_scale.is_empty());
        assert!(!high_scale.is_empty());
        assert!(high_scale.len() >= low_scale.len());
    }

    #[test]
    fn positioned_glyph_adapter_requires_explicit_font_size() {
        let font = noto_fixture();
        let layout = font.layout("A", 48.0);
        let error = font
            .tessellate_positioned_glyph(&layout.glyphs[0], 0.0, 0.01, [0.0, 0.0], 0.2)
            .expect_err("zero font size must fail");

        assert_eq!(error.kind, UiGlyphVectorDiagnosticKind::InvalidScale);
    }

    #[test]
    fn positioned_glyph_adapter_preserves_missing_outline_diagnostic() {
        let font = noto_fixture();
        let layout = font.layout(" ", 48.0);
        let error = font
            .tessellate_positioned_glyph(&layout.glyphs[0], 48.0, 0.01, [0.0, 0.0], 0.2)
            .expect_err("whitespace has no outline");

        assert_eq!(error.kind, UiGlyphVectorDiagnosticKind::MissingOutline);
    }

    #[test]
    fn fill_topology_keeps_simple_glyphs_on_the_existing_contract() {
        let topology = noto_fixture()
            .outline('-')
            .expect("hyphen outline")
            .fill_topology(UiGlyphVectorOptions::new(48.0, [0.0, 0.0], 0.2))
            .expect("topology classification");

        assert_eq!(topology, UiGlyphFillTopology::SingleConvexContour);
    }

    fn polygon_area(points: &[[f32; 2]]) -> f32 {
        points
            .iter()
            .zip(points.iter().cycle().skip(1))
            .take(points.len())
            .map(|(left, right)| left[0] * right[1] - right[0] * left[1])
            .sum::<f32>()
            * 0.5
    }

    fn triangle_area(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
        ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])) * 0.5
    }

    fn assert_mesh_matches_non_zero_fill(
        character: char,
        path: &VectorPath,
        triangles: &[[f32; 2]],
    ) {
        assert_mesh_matches_fill_rule(character, path, triangles, false);
    }

    fn assert_mesh_matches_fill_rule(
        character: char,
        path: &VectorPath,
        triangles: &[[f32; 2]],
        even_odd: bool,
    ) {
        let (min, max) = path.bounds().expect("glyph bounds");
        let mut mismatches = 0usize;
        for row in 0..32 {
            for column in 0..32 {
                let point = [
                    min[0] + (column as f32 + 0.5) / 32.0 * (max[0] - min[0]),
                    min[1] + (row as f32 + 0.5) / 32.0 * (max[1] - min[1]),
                ];
                let winding = path
                    .contours
                    .iter()
                    .map(|contour| winding_number(point, &contour.points))
                    .sum::<i32>();
                let source_filled = if even_odd {
                    winding.unsigned_abs() % 2 == 1
                } else {
                    winding != 0
                };
                let mesh_filled = triangles.chunks_exact(3).any(|triangle| {
                    point_in_triangle_sample(point, triangle[0], triangle[1], triangle[2])
                });
                mismatches += usize::from(source_filled != mesh_filled);
            }
        }
        assert!(
            mismatches <= 2,
            "{character} mesh disagrees with source fill at {mismatches}/1024 samples"
        );
    }

    fn winding_number(point: [f32; 2], polygon: &[[f32; 2]]) -> i32 {
        polygon
            .iter()
            .zip(polygon.iter().cycle().skip(1))
            .take(polygon.len())
            .fold(0, |winding, (start, end)| {
                let side = triangle_area(*start, *end, point);
                if start[1] <= point[1] && end[1] > point[1] && side > 0.0 {
                    winding + 1
                } else if start[1] > point[1] && end[1] <= point[1] && side < 0.0 {
                    winding - 1
                } else {
                    winding
                }
            })
    }

    fn point_in_triangle_sample(point: [f32; 2], a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> bool {
        let ab = triangle_area(a, b, point);
        let bc = triangle_area(b, c, point);
        let ca = triangle_area(c, a, point);
        (ab >= -1.0e-5 && bc >= -1.0e-5 && ca >= -1.0e-5)
            || (ab <= 1.0e-5 && bc <= 1.0e-5 && ca <= 1.0e-5)
    }
}
