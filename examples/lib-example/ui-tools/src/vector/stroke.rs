use std::f32::consts::TAU;

use super::geometry::normalized_contour_points;
use super::{VectorContour, VectorPath};

/// Tessellates a contour as a centered triangle-list stroke.
///
/// `width` is the half-width in the contour's coordinate system. The contour's
/// explicit `closed` flag controls whether endpoint caps are emitted.
pub fn tessellate_stroke(contour: &VectorContour, width: f32) -> Vec<[f32; 3]> {
    if contour.points.len() < 2 || !contour.is_finite() || !width.is_finite() || width <= 0.0 {
        return Vec::new();
    }

    let normalized_points = normalized_contour_points(&contour.points);
    let points = &normalized_points;
    let count = points.len();
    if contour.closed && count < 3 {
        return Vec::new();
    }
    if !contour.closed && count == 2 {
        let dx = points[1][0] - points[0][0];
        let dy = points[1][1] - points[0][1];
        if (dx * dx + dy * dy).sqrt() < width * 2.0 {
            let center = [
                (points[0][0] + points[1][0]) * 0.5,
                (points[0][1] + points[1][1]) * 0.5,
            ];
            let mut result = Vec::new();
            add_round_cap(&mut result, center, width);
            return result;
        }
    }

    let mut offsets = Vec::with_capacity(count);
    for index in 0..count {
        let point = points[index];
        let previous = if index == 0 {
            if contour.closed {
                points[count - 1]
            } else {
                points[1]
            }
        } else {
            points[index - 1]
        };
        let next = if index + 1 == count {
            if contour.closed {
                points[0]
            } else {
                points[count - 2]
            }
        } else {
            points[index + 1]
        };
        let incoming = normalize([point[0] - previous[0], point[1] - previous[1]]);
        let outgoing = normalize([next[0] - point[0], next[1] - point[1]]);
        let incoming_normal = perp(incoming);
        let outgoing_normal = perp(outgoing);
        let offset = if !contour.closed && index == 0 {
            scale(outgoing_normal, width)
        } else if !contour.closed && index + 1 == count {
            scale(incoming_normal, width)
        } else {
            let sum = normalize([
                incoming_normal[0] + outgoing_normal[0],
                incoming_normal[1] + outgoing_normal[1],
            ]);
            let denominator = sum[0] * outgoing_normal[0] + sum[1] * outgoing_normal[1];
            if denominator.abs() < 0.25 {
                scale(outgoing_normal, width)
            } else {
                scale(sum, (width / denominator).clamp(-width * 4.0, width * 4.0))
            }
        };
        offsets.push(offset);
    }

    let segments = if contour.closed { count } else { count - 1 };
    let mut positions = Vec::with_capacity(segments * 6 + if contour.closed { 0 } else { 72 });
    for index in 0..segments {
        let next = (index + 1) % count;
        let left_a = [
            points[index][0] + offsets[index][0],
            points[index][1] + offsets[index][1],
            0.0,
        ];
        let right_a = [
            points[index][0] - offsets[index][0],
            points[index][1] - offsets[index][1],
            0.0,
        ];
        let left_b = [
            points[next][0] + offsets[next][0],
            points[next][1] + offsets[next][1],
            0.0,
        ];
        let right_b = [
            points[next][0] - offsets[next][0],
            points[next][1] - offsets[next][1],
            0.0,
        ];
        positions.extend([left_a, right_a, left_b, right_a, right_b, left_b]);
    }
    if !contour.closed {
        add_round_cap(&mut positions, points[0], width);
        add_round_cap(&mut positions, points[count - 1], width);
    }
    positions
}

/// Tessellates every contour in a provider-neutral path collection as one
/// renderer-neutral triangle list.
pub fn tessellate_path_strokes(paths: &[VectorPath], width: f32) -> Vec<[f32; 3]> {
    paths
        .iter()
        .flat_map(|path| {
            path.contours
                .iter()
                .flat_map(move |contour| tessellate_stroke(contour, width))
        })
        .collect()
}

fn normalize(vector: [f32; 2]) -> [f32; 2] {
    let length = (vector[0] * vector[0] + vector[1] * vector[1]).sqrt();
    if length <= f32::EPSILON {
        [0.0, 0.0]
    } else {
        [vector[0] / length, vector[1] / length]
    }
}

fn scale(vector: [f32; 2], amount: f32) -> [f32; 2] {
    [vector[0] * amount, vector[1] * amount]
}

fn perp(vector: [f32; 2]) -> [f32; 2] {
    [-vector[1], vector[0]]
}

fn add_round_cap(positions: &mut Vec<[f32; 3]>, point: [f32; 2], width: f32) {
    for index in 0..12 {
        let a = index as f32 * TAU / 12.0;
        let b = (index + 1) as f32 * TAU / 12.0;
        positions.extend([
            [point[0], point[1], 0.0],
            [point[0] + a.cos() * width, point[1] + a.sin() * width, 0.0],
            [point[0] + b.cos() * width, point[1] + b.sin() * width, 0.0],
        ]);
    }
}
