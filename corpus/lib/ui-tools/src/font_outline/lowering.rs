use crate::{validate_convex_fill, VectorContour, VectorPath};

use super::{
    types::{point_is_finite, vector_diagnostic},
    UiGlyphFillTopology, UiGlyphOutline, UiGlyphOutlineSegment, UiGlyphVectorDiagnostic,
    UiGlyphVectorDiagnosticKind, UiGlyphVectorOptions, POINT_EPSILON,
};

impl UiGlyphOutline {
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
        if !point_is_finite(options.origin) {
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
        match validate_convex_fill(&path) {
            Ok(()) => Ok(UiGlyphFillTopology::SingleConvexContour),
            Err(message) if message.contains("concave") => {
                Ok(UiGlyphFillTopology::SingleConcaveContour)
            }
            Err(_) => Ok(UiGlyphFillTopology::Invalid),
        }
    }
}

fn points_approximately_equal(left: [f32; 2], right: [f32; 2]) -> bool {
    (left[0] - right[0]).abs() <= POINT_EPSILON && (left[1] - right[1]).abs() <= POINT_EPSILON
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
