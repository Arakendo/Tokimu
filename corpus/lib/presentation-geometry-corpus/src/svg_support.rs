//! Shared SVG record analysis used by provider-specific corpus runners.

use ui_tools::{
    clip_path_to_convex_polygon, is_convex_polygon_clip, tessellate_general_fill_with_rule,
    tessellate_stroke_with_style, SvgFillRule, SvgStrokeLinecap, SvgStrokeLinejoin,
    SvgVectorRecord, VectorFillRule, VectorPath, VectorStrokeCap, VectorStrokeJoin,
    VectorStrokeStyle,
};

#[derive(Debug, Default)]
pub(crate) struct SvgFillMeshes {
    pub triangles: Vec<[f32; 2]>,
    pub fill_paths: usize,
    pub stroke_paths: usize,
    pub diagnostics: Vec<String>,
}

pub(crate) fn summarize_paths(description: &str, paths: &[VectorPath]) -> String {
    let contour_count = paths.iter().map(|path| path.contours.len()).sum::<usize>();
    let point_count = paths
        .iter()
        .flat_map(|path| path.contours.iter())
        .map(|contour| contour.points.len())
        .sum::<usize>();
    let closed_contours = paths
        .iter()
        .flat_map(|path| path.contours.iter())
        .filter(|contour| contour.closed)
        .count();
    format!(
        "{description} paths={} contours={} points={} closed_contours={closed_contours}",
        paths.len(),
        contour_count,
        point_count,
    )
}

/// Tessellates SVG fill paint using SVG's implicit subpath closure rule.
///
/// Source paths remain unmodified for stroke lowering. `SvgVectorRecord`
/// supplies a fill-only view so the shared vector layer never needs to know
/// SVG paint semantics.
pub(crate) fn tessellate_svg_fills(
    records: &[SvgVectorRecord],
    diagnostic_context: &str,
) -> SvgFillMeshes {
    let mut result = SvgFillMeshes {
        triangles: Vec::new(),
        fill_paths: 0,
        stroke_paths: 0,
        diagnostics: Vec::new(),
    };

    for record in records {
        if !record.fill {
            continue;
        }
        let fill_path = record.path_for_fill();
        if fill_path.contours.is_empty() {
            continue;
        }
        let fill_path = match record.clip_path.as_ref() {
            Some(clip)
                if fill_path.contours.iter().all(|contour| contour.closed)
                    && is_convex_polygon_clip(clip) =>
            {
                match clip_path_to_convex_polygon(&fill_path, clip) {
                    Ok(path) => path,
                    Err(error) => {
                        result.diagnostics.push(format!(
                            "{diagnostic_context} fill clip intersection failed: {error}"
                        ));
                        continue;
                    }
                }
            }
            Some(_) => {
                result.diagnostics.push(format!(
                    "{diagnostic_context} fill clip requires one convex closed clip and closed fill contours"
                ));
                continue;
            }
            None => fill_path,
        };
        let fill_rule = match record.fill_rule {
            SvgFillRule::NonZero => VectorFillRule::NonZero,
            SvgFillRule::EvenOdd => VectorFillRule::EvenOdd,
        };
        match tessellate_general_fill_with_rule(&fill_path, fill_rule) {
            Ok(mut triangles) => {
                result.triangles.append(&mut triangles);
                result.fill_paths += 1;
            }
            Err(error) => result.diagnostics.push(format!(
                "{diagnostic_context} path fill tessellation failed: {error}"
            )),
        }
    }
    result
}

/// Expands admitted SVG strokes into normalized vector geometry.
///
/// `stroke_scale` converts one SVG user-space unit into the uniform normalized
/// vector-space unit used by the current corpus viewport policy. Non-uniform
/// transforms and `preserveAspectRatio` are intentionally not hidden here;
/// callers must choose the policy explicitly until the importer records the
/// complete stroke transform.
pub(crate) fn tessellate_svg_strokes(
    records: &[SvgVectorRecord],
    stroke_scale: f32,
    diagnostic_context: &str,
) -> SvgFillMeshes {
    let mut result = SvgFillMeshes {
        triangles: Vec::new(),
        fill_paths: 0,
        stroke_paths: 0,
        diagnostics: Vec::new(),
    };

    for record in records {
        if !record.stroke {
            continue;
        }
        if record.clip_path.is_some() {
            result.diagnostics.push(format!(
                "{diagnostic_context} stroke clipping is outside the current structural profile"
            ));
            continue;
        }
        let cap = match record.stroke_linecap {
            SvgStrokeLinecap::Butt => VectorStrokeCap::Butt,
            SvgStrokeLinecap::Round => VectorStrokeCap::Round,
            SvgStrokeLinecap::Square => VectorStrokeCap::Square,
        };
        let join = match record.stroke_linejoin {
            SvgStrokeLinejoin::Miter => VectorStrokeJoin::Miter,
            SvgStrokeLinejoin::Round => VectorStrokeJoin::Round,
            SvgStrokeLinejoin::Bevel => VectorStrokeJoin::Bevel,
        };
        let style = VectorStrokeStyle {
            half_width: record.stroke_width * stroke_scale * 0.5,
            cap,
            join,
            miter_limit: record.stroke_miterlimit,
        };
        let dash_contours = record
            .path
            .contours
            .iter()
            .map(|contour| {
                record
                    .stroke_dasharray
                    .as_deref()
                    .map(|pattern| dash_contour(contour, pattern, record.stroke_dashoffset))
                    .unwrap_or_else(|| Ok(vec![contour.clone()]))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|contours| contours.into_iter().flatten().collect::<Vec<_>>());
        match dash_contours.and_then(|contours| {
            contours
                .iter()
                .map(|contour| tessellate_stroke_with_style(contour, style))
                .collect::<Result<Vec<_>, _>>()
        }) {
            Ok(meshes) => {
                for mesh in meshes {
                    result
                        .triangles
                        .extend(mesh.into_iter().map(|vertex| [vertex[0], vertex[1]]));
                }
                result.stroke_paths += 1;
            }
            Err(error) => result.diagnostics.push(format!(
                "{diagnostic_context} path stroke expansion failed: {error}"
            )),
        }
    }
    result
}

fn dash_contour(
    contour: &ui_tools::VectorContour,
    pattern: &[f32],
    dashoffset: f32,
) -> Result<Vec<ui_tools::VectorContour>, String> {
    if pattern.is_empty() || pattern.iter().all(|value| *value == 0.0) {
        return Err("SVG dash pattern must contain a positive length".into());
    }
    if !dashoffset.is_finite()
        || pattern
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
    {
        return Err("SVG dash pattern must contain finite non-negative lengths".into());
    }
    if contour.points.len() < 2 {
        return Ok(Vec::new());
    }

    let mut normalized_pattern = pattern
        .iter()
        .copied()
        .filter(|value| *value > 0.0)
        .collect::<Vec<_>>();
    if normalized_pattern.is_empty() {
        return Err("SVG dash pattern must contain a positive length".into());
    }
    if normalized_pattern.len() % 2 == 1 {
        let repeated = normalized_pattern.clone();
        normalized_pattern.extend(repeated);
    }
    let pattern_total: f32 = normalized_pattern.iter().sum();
    if !pattern_total.is_finite() || pattern_total <= 0.0 {
        return Err("SVG dash pattern total must be positive".into());
    }

    let mut pattern_position = (-dashoffset).rem_euclid(pattern_total);
    let mut pattern_index = 0usize;
    while pattern_position >= normalized_pattern[pattern_index]
        && normalized_pattern[pattern_index] > 0.0
    {
        pattern_position -= normalized_pattern[pattern_index];
        pattern_index = (pattern_index + 1) % normalized_pattern.len();
    }
    let mut pattern_remaining = normalized_pattern[pattern_index] - pattern_position;
    if pattern_remaining <= 0.0 {
        pattern_remaining = normalized_pattern[pattern_index];
    }
    let mut on = pattern_index.is_multiple_of(2);
    let mut active = Vec::<[f32; 2]>::new();
    let mut result = Vec::new();
    let segment_count = if contour.closed {
        contour.points.len()
    } else {
        contour.points.len() - 1
    };

    for index in 0..segment_count {
        let start = contour.points[index];
        let end = contour.points[(index + 1) % contour.points.len()];
        let delta = [end[0] - start[0], end[1] - start[1]];
        let length = (delta[0] * delta[0] + delta[1] * delta[1]).sqrt();
        if length <= f32::EPSILON {
            continue;
        }
        let mut distance = 0.0;
        while distance < length {
            let step = pattern_remaining.min(length - distance);
            let a = distance / length;
            let b = (distance + step) / length;
            let first = [start[0] + delta[0] * a, start[1] + delta[1] * a];
            let last = [start[0] + delta[0] * b, start[1] + delta[1] * b];
            if on {
                if active.last().copied() != Some(first) {
                    active.push(first);
                }
                active.push(last);
            } else if active.len() >= 2 {
                result.push(ui_tools::VectorContour::new(
                    std::mem::take(&mut active),
                    false,
                ));
            }
            distance += step;
            pattern_remaining -= step;
            if pattern_remaining <= f32::EPSILON {
                if on && active.len() >= 2 {
                    result.push(ui_tools::VectorContour::new(
                        std::mem::take(&mut active),
                        false,
                    ));
                }
                pattern_index = (pattern_index + 1) % normalized_pattern.len();
                on = pattern_index.is_multiple_of(2);
                pattern_remaining = normalized_pattern[pattern_index];
            }
        }
    }
    if active.len() >= 2 {
        result.push(ui_tools::VectorContour::new(active, false));
    }
    Ok(result)
}
