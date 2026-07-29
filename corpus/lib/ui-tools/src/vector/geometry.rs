pub(super) fn normalized_contour_points(points: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let mut normalized = points.to_vec();
    if normalized.len() > 1 && normalized.first() == normalized.last() {
        normalized.pop();
    }
    normalized
}

pub(super) fn signed_area(points: &[[f32; 2]]) -> f32 {
    points
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let next = points[(index + 1) % points.len()];
            point[0] * next[1] - next[0] * point[1]
        })
        .sum::<f32>()
        * 0.5
}

pub(super) fn subtract(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

pub(super) fn cross(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[1] - a[1] * b[0]
}

/// Clips closed contours against an axis-aligned rectangle.
///
/// This is intentionally a small polygon-clipping primitive. It preserves
/// contour order and fill-rule responsibility, but does not perform boolean
/// operations between contours or clip open strokes.
pub fn clip_path_to_axis_aligned_rect(
    path: &VectorPath,
    min: [f32; 2],
    max: [f32; 2],
) -> Result<VectorPath, String> {
    if !min[0].is_finite()
        || !min[1].is_finite()
        || !max[0].is_finite()
        || !max[1].is_finite()
        || min[0] >= max[0]
        || min[1] >= max[1]
    {
        return Err("axis-aligned clip rectangle must have finite positive bounds".into());
    }

    let clip = VectorPath::new(vec![VectorContour::new(
        vec![
            [min[0], min[1]],
            [max[0], min[1]],
            [max[0], max[1]],
            [min[0], max[1]],
        ],
        true,
    )]);
    clip_path_to_convex_polygon(path, &clip)
}

/// Clips closed contours against one convex closed polygon.
///
/// The clip polygon is deliberately limited to one contour. This supports
/// common rectangular, polygonal, and flattened circular clip paths without
/// pretending to implement arbitrary polygon boolean operations.
pub fn clip_path_to_convex_polygon(
    path: &VectorPath,
    clip: &VectorPath,
) -> Result<VectorPath, String> {
    let clip_points = clip
        .contours
        .first()
        .filter(|contour| contour.closed)
        .map(|contour| normalized_contour_points(&contour.points))
        .filter(|points| points.len() >= 3)
        .ok_or_else(|| "convex clipping requires one closed polygon contour".to_owned())?;
    if clip.contours.len() != 1 || !is_convex_polygon(&clip_points) {
        return Err("convex clipping requires one finite convex polygon contour".into());
    }

    let orientation = signed_area(&clip_points).signum();
    if orientation == 0.0 || !clip_points.iter().flatten().all(|value| value.is_finite()) {
        return Err("convex clipping requires finite non-degenerate geometry".into());
    }

    let mut contours = Vec::new();
    for contour in &path.contours {
        if !contour.closed || contour.points.len() < 3 {
            return Err("convex clipping requires closed polygon contours".into());
        }

        let mut points = normalized_contour_points(&contour.points);
        for edge_index in 0..clip_points.len() {
            let from = clip_points[edge_index];
            let to = clip_points[(edge_index + 1) % clip_points.len()];
            points = clip_polygon_edge(&points, from, to, orientation);
            if points.len() < 3 {
                break;
            }
        }
        if points.len() >= 3 {
            contours.push(VectorContour::new(points, true));
        }
    }

    Ok(VectorPath::new(contours))
}

/// Reports whether a path is the single closed finite convex contour required
/// by [`clip_path_to_convex_polygon`].
pub fn is_convex_polygon_clip(path: &VectorPath) -> bool {
    let Some(contour) = path.contours.first() else {
        return false;
    };
    let points = normalized_contour_points(&contour.points);
    path.contours.len() == 1
        && contour.closed
        && points.len() >= 3
        && points.iter().flatten().all(|value| value.is_finite())
        && is_convex_polygon(&points)
}

fn is_convex_polygon(points: &[[f32; 2]]) -> bool {
    let mut turn = 0.0;
    for index in 0..points.len() {
        let a = subtract(points[(index + 1) % points.len()], points[index]);
        let b = subtract(
            points[(index + 2) % points.len()],
            points[(index + 1) % points.len()],
        );
        let cross_product = cross(a, b);
        if !cross_product.is_finite() {
            return false;
        }
        if cross_product.abs() > 1.0e-6 {
            if turn == 0.0 {
                turn = cross_product.signum();
            } else if turn != cross_product.signum() {
                return false;
            }
        }
    }
    turn != 0.0
}

fn clip_polygon_edge(
    points: &[[f32; 2]],
    clip_start: [f32; 2],
    clip_end: [f32; 2],
    orientation: f32,
) -> Vec<[f32; 2]> {
    if points.is_empty() {
        return Vec::new();
    }

    let edge = subtract(clip_end, clip_start);
    let inside =
        |point: [f32; 2]| orientation * cross(edge, subtract(point, clip_start)) >= -1.0e-6;
    let intersection = |segment_start: [f32; 2], segment_end: [f32; 2]| {
        let segment = subtract(segment_end, segment_start);
        let denominator = cross(edge, segment);
        if denominator.abs() <= f32::EPSILON {
            return segment_start;
        }
        let t = cross(edge, subtract(clip_start, segment_start)) / denominator;
        [
            segment_start[0] + segment[0] * t,
            segment_start[1] + segment[1] * t,
        ]
    };

    let mut result = Vec::new();
    let mut previous = *points.last().expect("non-empty polygon");
    let mut previous_inside = inside(previous);
    for &current in points {
        let current_inside = inside(current);
        match (previous_inside, current_inside) {
            (true, true) => result.push(current),
            (true, false) => result.push(intersection(previous, current)),
            (false, true) => {
                result.push(intersection(previous, current));
                result.push(current);
            }
            (false, false) => {}
        }
        previous = current;
        previous_inside = current_inside;
    }
    result
}
use super::types::{VectorContour, VectorPath};
