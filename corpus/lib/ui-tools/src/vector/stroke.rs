use std::f32::consts::TAU;

use super::geometry::normalized_contour_points;
use super::{VectorContour, VectorPath};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VectorStrokeCap {
    Butt,
    Round,
    Square,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VectorStrokeJoin {
    Miter,
    Round,
    Bevel,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VectorStrokeStyle {
    pub half_width: f32,
    pub cap: VectorStrokeCap,
    pub join: VectorStrokeJoin,
    pub miter_limit: f32,
}

/// Tessellates a contour as a centered triangle-list stroke.
///
/// `width` is the half-width in the contour's coordinate system. The contour's
/// explicit `closed` flag controls whether endpoint caps are emitted.
pub fn tessellate_stroke(contour: &VectorContour, width: f32) -> Vec<[f32; 3]> {
    tessellate_stroke_with_style(
        contour,
        VectorStrokeStyle {
            half_width: width,
            cap: VectorStrokeCap::Round,
            join: VectorStrokeJoin::Miter,
            miter_limit: 4.0,
        },
    )
    .unwrap_or_default()
}

/// Tessellates a contour using an explicit stroke style.
///
/// Joins are resolved at shared contour vertices. Miter joins use the existing
/// bounded offset path; bevel and round joins use a segment strip plus an
/// explicit outer-corner join.
pub fn tessellate_stroke_with_style(
    contour: &VectorContour,
    style: VectorStrokeStyle,
) -> Result<Vec<[f32; 3]>, String> {
    if contour.points.len() < 2
        || !contour.is_finite()
        || !style.half_width.is_finite()
        || style.half_width <= 0.0
    {
        return Ok(Vec::new());
    }
    if !style.miter_limit.is_finite() || style.miter_limit <= 0.0 {
        return Err("vector stroke miter limit must be finite and positive".to_string());
    }

    let normalized_points = normalized_contour_points(&contour.points);
    let mut points = normalized_points;
    let count = points.len();
    if contour.closed && count < 3 {
        return Ok(Vec::new());
    }

    if !contour.closed && style.cap == VectorStrokeCap::Square {
        let start_direction = normalize([points[1][0] - points[0][0], points[1][1] - points[0][1]]);
        let end_direction = normalize([
            points[count - 1][0] - points[count - 2][0],
            points[count - 1][1] - points[count - 2][1],
        ]);
        points[0][0] -= start_direction[0] * style.half_width;
        points[0][1] -= start_direction[1] * style.half_width;
        points[count - 1][0] += end_direction[0] * style.half_width;
        points[count - 1][1] += end_direction[1] * style.half_width;
    }

    let points = &points;
    let width = style.half_width;

    if style.join != VectorStrokeJoin::Miter {
        return Ok(tessellate_connected_join_stroke(
            points,
            contour.closed,
            width,
            style.cap,
            style.join,
        ));
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
                scale(
                    sum,
                    (width / denominator)
                        .clamp(-width * style.miter_limit, width * style.miter_limit),
                )
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
    if !contour.closed && style.cap == VectorStrokeCap::Round {
        add_round_cap(&mut positions, points[0], width);
        add_round_cap(&mut positions, points[count - 1], width);
    }
    Ok(positions)
}

fn tessellate_connected_join_stroke(
    points: &[[f32; 2]],
    closed: bool,
    width: f32,
    cap: VectorStrokeCap,
    join: VectorStrokeJoin,
) -> Vec<[f32; 3]> {
    let count = points.len();
    let segments = if closed { count } else { count - 1 };
    let mut normals = Vec::with_capacity(segments);
    for index in 0..segments {
        let next = (index + 1) % count;
        let direction = normalize([
            points[next][0] - points[index][0],
            points[next][1] - points[index][1],
        ]);
        normals.push(perp(direction));
    }

    let mut positions = Vec::with_capacity(segments * 6 + if closed { 0 } else { 72 });
    for index in 0..segments {
        let next = (index + 1) % count;
        let normal = scale(normals[index], width);
        let left_a = [
            points[index][0] + normal[0],
            points[index][1] + normal[1],
            0.0,
        ];
        let right_a = [
            points[index][0] - normal[0],
            points[index][1] - normal[1],
            0.0,
        ];
        let left_b = [
            points[next][0] + normal[0],
            points[next][1] + normal[1],
            0.0,
        ];
        let right_b = [
            points[next][0] - normal[0],
            points[next][1] - normal[1],
            0.0,
        ];
        positions.extend([left_a, right_a, left_b, right_a, right_b, left_b]);
    }

    let first_join = if closed { 0 } else { 1 };
    let join_count = if closed {
        count
    } else {
        count.saturating_sub(2)
    };
    for offset in 0..join_count {
        let index = (first_join + offset) % count;
        let previous_segment = (index + segments - 1) % segments;
        let next_segment = index % segments;
        let incoming = normalize([
            points[index][0] - points[(index + count - 1) % count][0],
            points[index][1] - points[(index + count - 1) % count][1],
        ]);
        let outgoing = normalize([
            points[(index + 1) % count][0] - points[index][0],
            points[(index + 1) % count][1] - points[index][1],
        ]);
        let turn = incoming[0] * outgoing[1] - incoming[1] * outgoing[0];
        if turn.abs() <= f32::EPSILON {
            continue;
        }
        let mut outer_previous = normals[previous_segment];
        let mut outer_next = normals[next_segment];
        if turn < 0.0 {
            outer_previous = scale(outer_previous, -1.0);
            outer_next = scale(outer_next, -1.0);
        }
        let outer_previous = scale(outer_previous, width);
        let outer_next = scale(outer_next, width);
        match join {
            VectorStrokeJoin::Bevel => {
                positions.extend([
                    [
                        points[index][0] + outer_previous[0],
                        points[index][1] + outer_previous[1],
                        0.0,
                    ],
                    [points[index][0], points[index][1], 0.0],
                    [
                        points[index][0] + outer_next[0],
                        points[index][1] + outer_next[1],
                        0.0,
                    ],
                ]);
            }
            VectorStrokeJoin::Round => {
                add_round_join(&mut positions, points[index], outer_previous, outer_next)
            }
            VectorStrokeJoin::Miter => unreachable!("miter joins use the bounded offset path"),
        }
    }

    if !closed && cap == VectorStrokeCap::Round {
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

fn add_round_join(positions: &mut Vec<[f32; 3]>, point: [f32; 2], start: [f32; 2], end: [f32; 2]) {
    let start_angle = start[1].atan2(start[0]);
    let cross = start[0] * end[1] - start[1] * end[0];
    let dot = start[0] * end[0] + start[1] * end[1];
    let sweep = cross.atan2(dot);
    let steps = ((sweep.abs() / (std::f32::consts::PI / 12.0)).ceil() as usize).max(1);
    for index in 0..steps {
        let a = start_angle + sweep * index as f32 / steps as f32;
        let b = start_angle + sweep * (index + 1) as f32 / steps as f32;
        let radius = (start[0] * start[0] + start[1] * start[1]).sqrt();
        positions.extend([
            [point[0], point[1], 0.0],
            [
                point[0] + a.cos() * radius,
                point[1] + a.sin() * radius,
                0.0,
            ],
            [
                point[0] + b.cos() * radius,
                point[1] + b.sin() * radius,
                0.0,
            ],
        ]);
    }
}
