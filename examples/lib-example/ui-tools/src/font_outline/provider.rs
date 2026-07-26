use ttf_parser::OutlineBuilder;

use crate::{
    tessellate_general_fill_with_rule, UiFontRasterizer, UiRasterTextGlyph, VectorFillRule,
};

use super::{
    types::vector_diagnostic, UiGlyphContour, UiGlyphOutline, UiGlyphOutlineDiagnostic,
    UiGlyphOutlineDiagnosticKind, UiGlyphOutlineSegment, UiGlyphVectorDiagnostic,
    UiGlyphVectorDiagnosticKind, UiGlyphVectorOptions,
};

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
