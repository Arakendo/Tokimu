//! Provider-neutral vector paths used by presentation geometry.
//!
//! This module intentionally stops before tessellation. It defines the small
//! path contract shared by UI lowering and future SVG importing.

use std::f32::consts::TAU;

use lyon_path::math::point;
use lyon_path::Path;
use lyon_tessellation::{FillOptions, FillRule, FillTessellator, FillVertex};

#[derive(Clone, Debug, PartialEq)]
pub struct VectorPath {
    pub contours: Vec<VectorContour>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VectorContour {
    pub points: Vec<[f32; 2]>,
    pub closed: bool,
}

impl VectorPath {
    pub fn new(contours: Vec<VectorContour>) -> Self {
        Self { contours }
    }

    pub fn is_finite(&self) -> bool {
        self.contours.iter().all(VectorContour::is_finite)
    }

    pub fn bounds(&self) -> Option<([f32; 2], [f32; 2])> {
        let mut points = self
            .contours
            .iter()
            .flat_map(|contour| contour.points.iter());
        let first = *points.next()?;
        let mut min = first;
        let mut max = first;

        for point in points {
            min[0] = min[0].min(point[0]);
            min[1] = min[1].min(point[1]);
            max[0] = max[0].max(point[0]);
            max[1] = max[1].max(point[1]);
        }

        Some((min, max))
    }
}

impl VectorContour {
    pub fn new(points: Vec<[f32; 2]>, closed: bool) -> Self {
        Self { points, closed }
    }

    pub fn is_finite(&self) -> bool {
        self.points
            .iter()
            .all(|point| point[0].is_finite() && point[1].is_finite())
    }
}

/// Tessellates one convex closed contour into a triangle list.
///
/// The returned vertices are grouped as independent triangles. Concave,
/// multi-contour, open, and degenerate paths are rejected until their fill
/// contracts have their own evidence and tests.
pub fn tessellate_convex_fill(path: &VectorPath) -> Result<Vec<[f32; 2]>, String> {
    validate_convex_fill(path)?;

    let contour = &path.contours[0];
    let points = normalized_contour_points(&contour.points);
    let winding = signed_area(&points).signum();

    let mut triangles = Vec::with_capacity((points.len() - 2) * 3);
    for index in 1..points.len() - 1 {
        if winding > 0.0 {
            triangles.extend([points[0], points[index], points[index + 1]]);
        } else {
            triangles.extend([points[0], points[index + 1], points[index]]);
        }
    }
    Ok(triangles)
}

/// Tessellates closed provider-neutral contours, including concave contours
/// and multiple contours such as glyph counters using the default non-zero rule.
pub fn tessellate_general_fill(path: &VectorPath) -> Result<Vec<[f32; 2]>, String> {
    tessellate_general_fill_with_rule(path, VectorFillRule::NonZero)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VectorFillRule {
    NonZero,
    EvenOdd,
}

/// Tessellates closed contours with an explicit provider-neutral fill rule.
pub fn tessellate_general_fill_with_rule(
    path: &VectorPath,
    fill_rule: VectorFillRule,
) -> Result<Vec<[f32; 2]>, String> {
    if path.contours.is_empty() {
        return Err("general fill requires at least one contour".into());
    }
    if path
        .contours
        .iter()
        .any(|contour| !contour.closed || contour.points.len() < 3)
    {
        return Err("general fill requires closed contours with at least three points".into());
    }
    if !path.is_finite() {
        return Err("general fill received non-finite coordinates".into());
    }

    let contours = path
        .contours
        .iter()
        .map(|contour| sanitized_closed_points(&contour.points))
        .collect::<Vec<_>>();
    for points in &contours {
        if points.len() < 3 {
            return Err("general fill contour became degenerate after sanitization".into());
        }
    }

    // A single simple contour is a complete polygon. Ear clipping preserves
    // its concave notches directly and avoids asking the multi-contour fill
    // path to infer topology it does not need for ordinary glyphs. A crossing
    // contour first becomes a planar collection of bounded faces; merely
    // inserting crossing points still leaves one invalid polygon.
    if contours.len() == 1 {
        let split_points = split_self_intersections(&contours[0]);
        if split_points.len() != contours[0].len() {
            return tessellate_intersecting_single_contour(&contours[0], fill_rule);
        }
        let simple_loops = split_repeated_vertex_loops(split_points);
        if simple_loops.len() == 1 && simple_loops[0] == contours[0] {
            // Ear clipping is a useful fast path for ordinary concave
            // contours, but a flattened provider outline can still be a
            // valid fill even when its local ear search is numerically
            // inconclusive. Let the general tessellator own that fallback
            // instead of turning a recoverable geometry condition into a
            // provider failure.
            if let Ok(triangles) = tessellate_simple_loop(&contours[0]) {
                return Ok(triangles);
            }
        }
    }

    let triangles = tessellate_lyon_contours(&contours, fill_rule)?;
    if mesh_preserves_contour_bounds(&contours, &triangles) {
        return Ok(triangles);
    }

    let mut repaired_triangles = Vec::new();
    let mut regular_contours = Vec::new();
    let mut repair_needed = false;
    for points in contours {
        let split_points = split_self_intersections(&points);
        let simple_loops = split_repeated_vertex_loops(split_points);
        if simple_loops.len() > 1 {
            repair_needed = true;
            for simple_loop in simple_loops {
                repaired_triangles.extend(tessellate_simple_loop(&simple_loop)?);
            }
            continue;
        }
        if simple_loops
            .first()
            .is_some_and(|loop_points| loop_points != &points)
        {
            repair_needed = true;
        }
        regular_contours.extend(simple_loops);
    }
    if !repair_needed {
        // A bounds discrepancy alone is not evidence of bad topology. Keep
        // Lyon's fill result for ordinary concave outlines; the local repair
        // path is intentionally reserved for contours that actually split.
        return Ok(triangles);
    }
    if !regular_contours.is_empty() {
        repaired_triangles.extend(tessellate_lyon_contours(&regular_contours, fill_rule)?);
    }
    Ok(repaired_triangles)
}

fn tessellate_lyon_contours(
    contours: &[Vec<[f32; 2]>],
    fill_rule: VectorFillRule,
) -> Result<Vec<[f32; 2]>, String> {
    let mut builder = Path::builder();
    for points in contours {
        builder.begin(point(points[0][0], points[0][1]));
        for vertex in &points[1..] {
            builder.line_to(point(vertex[0], vertex[1]));
        }
        builder.close();
    }
    let lyon_path = builder.build();
    let mut buffers = lyon_tessellation::VertexBuffers::<[f32; 2], u32>::new();
    let mut output = lyon_tessellation::BuffersBuilder::new(&mut buffers, |vertex: FillVertex| {
        vertex.position().to_array()
    });
    let lyon_fill_rule = match fill_rule {
        VectorFillRule::NonZero => FillRule::NonZero,
        VectorFillRule::EvenOdd => FillRule::EvenOdd,
    };
    FillTessellator::new()
        .tessellate_path(
            &lyon_path,
            &FillOptions::default().with_fill_rule(lyon_fill_rule),
            &mut output,
        )
        .map_err(|error| format!("general fill tessellation failed: {error:?}"))?;
    Ok(buffers
        .indices
        .chunks_exact(3)
        .flat_map(|triangle| {
            [
                buffers.vertices[triangle[0] as usize],
                buffers.vertices[triangle[1] as usize],
                buffers.vertices[triangle[2] as usize],
            ]
        })
        .collect())
}

fn mesh_preserves_contour_bounds(contours: &[Vec<[f32; 2]>], triangles: &[[f32; 2]]) -> bool {
    if triangles.is_empty() {
        return false;
    }
    let source = contours.iter().flatten().fold(
        [
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ],
        |bounds, point| {
            [
                bounds[0].min(point[0]),
                bounds[1].min(point[1]),
                bounds[2].max(point[0]),
                bounds[3].max(point[1]),
            ]
        },
    );
    let mesh = triangles.iter().fold(
        [
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ],
        |bounds, point| {
            [
                bounds[0].min(point[0]),
                bounds[1].min(point[1]),
                bounds[2].max(point[0]),
                bounds[3].max(point[1]),
            ]
        },
    );
    // Lyon may move a boundary vertex by a few floating-point units while
    // tessellating a valid contour. Do not route an otherwise valid glyph
    // through the conservative self-intersection repair path for that noise;
    // reserve repair for meaningful missing extents.
    let extent = (source[2] - source[0])
        .abs()
        .max((source[3] - source[1]).abs());
    let tolerance = (extent * 1.0e-4).max(1.0e-6);
    source
        .iter()
        .zip(mesh.iter())
        .all(|(source, mesh)| (source - mesh).abs() <= tolerance)
}

fn sanitized_closed_points(points: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let mut sanitized = Vec::with_capacity(points.len());
    for &point in points {
        if sanitized
            .last()
            .is_none_or(|previous| !points_approximately_equal(*previous, point))
        {
            sanitized.push(point);
        }
    }
    if sanitized.len() > 1 && points_approximately_equal(sanitized[0], *sanitized.last().unwrap()) {
        sanitized.pop();
    }
    sanitized
}

/// Removes samples that lie on the straight segment between their neighbors.
///
/// Curve flatteners commonly produce several points along a straight or nearly
/// straight run. They are valid boundary samples, but retaining every one of
/// them makes the small ear-clipping fallback needlessly fragile: a candidate
/// ear can contain another sample exactly on its edge. Keep genuine corners and
/// only remove points that are both nearly collinear and between their neighbors.
fn simplified_loop(points: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let mut simplified = points.to_vec();
    if simplified.len() < 4 {
        return simplified;
    }

    loop {
        let mut removed = false;
        for index in 0..simplified.len() {
            let previous = simplified[(index + simplified.len() - 1) % simplified.len()];
            let current = simplified[index];
            let next = simplified[(index + 1) % simplified.len()];
            let to_previous = subtract(previous, current);
            let to_next = subtract(next, current);
            let scale = (to_previous[0] * to_previous[0] + to_previous[1] * to_previous[1]).sqrt()
                * (to_next[0] * to_next[0] + to_next[1] * to_next[1]).sqrt();
            let collinear = cross(to_previous, to_next).abs() <= (scale * 1.0e-5).max(1.0e-10);
            let between = to_previous[0] * to_next[0] + to_previous[1] * to_next[1] <= 1.0e-8;
            if collinear && between {
                simplified.remove(index);
                removed = true;
                break;
            }
        }
        if !removed || simplified.len() < 3 {
            break;
        }
    }
    simplified
}

fn split_self_intersections(points: &[[f32; 2]]) -> Vec<[f32; 2]> {
    if points.len() < 4 {
        return points.to_vec();
    }

    let mut insertions = vec![Vec::<(f32, [f32; 2])>::new(); points.len()];
    for left_index in 0..points.len() {
        let left_next = (left_index + 1) % points.len();
        for right_index in left_index + 1..points.len() {
            let right_next = (right_index + 1) % points.len();
            if left_next == right_index || right_next == left_index {
                continue;
            }
            let Some((left_t, right_t, intersection)) = segment_intersection(
                points[left_index],
                points[left_next],
                points[right_index],
                points[right_next],
            ) else {
                continue;
            };
            insertions[left_index].push((left_t, intersection));
            insertions[right_index].push((right_t, intersection));
        }
    }

    let mut split =
        Vec::with_capacity(points.len() + insertions.iter().map(Vec::len).sum::<usize>());
    for (index, &point) in points.iter().enumerate() {
        split.push(point);
        insertions[index].sort_by(|left, right| left.0.total_cmp(&right.0));
        for &(_, intersection) in &insertions[index] {
            if split
                .last()
                .is_none_or(|previous| !points_approximately_equal(*previous, intersection))
            {
                split.push(intersection);
            }
        }
    }
    split
}

fn split_repeated_vertex_loops(points: Vec<[f32; 2]>) -> Vec<Vec<[f32; 2]>> {
    for left_index in 0..points.len() {
        for right_index in left_index + 2..points.len() {
            if left_index == 0 && right_index + 1 == points.len() {
                continue;
            }
            if !points_approximately_equal(points[left_index], points[right_index]) {
                continue;
            }

            let first_loop = points[left_index..right_index].to_vec();
            let mut second_loop = points[right_index..].to_vec();
            second_loop.extend_from_slice(&points[..left_index]);

            let mut loops = Vec::new();
            if first_loop.len() >= 3 {
                loops.extend(split_repeated_vertex_loops(first_loop));
            }
            if second_loop.len() >= 3 {
                loops.extend(split_repeated_vertex_loops(second_loop));
            }
            return loops;
        }
    }
    vec![points]
}

/// Fonts can contain a single re-entrant outline that a multi-contour
/// tessellator interprets as separate fill regions. Ear clipping preserves
/// the original ordered boundary for this narrow fallback.
fn tessellate_intersecting_single_contour(
    source: &[[f32; 2]],
    fill_rule: VectorFillRule,
) -> Result<Vec<[f32; 2]>, String> {
    let mut scanlines = split_self_intersections(source)
        .into_iter()
        .map(|point| point[1])
        .collect::<Vec<_>>();
    scanlines.sort_by(f32::total_cmp);
    scanlines.dedup_by(|left, right| (*left - *right).abs() <= 1.0e-6);

    let mut triangles = Vec::new();
    for band in scanlines.windows(2) {
        let bottom = band[0];
        let top = band[1];
        if top - bottom <= 1.0e-6 {
            continue;
        }
        let middle = (bottom + top) * 0.5;
        let mut crossings = Vec::new();
        for index in 0..source.len() {
            let start = source[index];
            let end = source[(index + 1) % source.len()];
            if (end[1] - start[1]).abs() <= 1.0e-8
                || middle <= start[1].min(end[1])
                || middle >= start[1].max(end[1])
            {
                continue;
            }
            crossings.push(ScanlineCrossing {
                start,
                end,
                direction: if end[1] > start[1] { 1 } else { -1 },
                middle_x: x_at_y(start, end, middle),
            });
        }
        crossings.sort_by(|left, right| left.middle_x.total_cmp(&right.middle_x));

        let mut winding = 0_i32;
        let mut left = None;
        for crossing in crossings {
            let filled_before = fill_rule.contains(winding);
            winding += crossing.direction;
            let filled_after = fill_rule.contains(winding);
            match (filled_before, filled_after) {
                (false, true) => left = Some(crossing),
                (true, false) => {
                    let Some(left) = left.take() else {
                        return Err("scanline fill ended without a left boundary".into());
                    };
                    let lower_left = x_at_y(left.start, left.end, bottom);
                    let lower_right = x_at_y(crossing.start, crossing.end, bottom);
                    let upper_left = x_at_y(left.start, left.end, top);
                    let upper_right = x_at_y(crossing.start, crossing.end, top);
                    triangles.extend([
                        [lower_left, bottom],
                        [lower_right, bottom],
                        [upper_right, top],
                        [lower_left, bottom],
                        [upper_right, top],
                        [upper_left, top],
                    ]);
                }
                _ => {}
            }
        }
        if left.is_some() || winding != 0 {
            return Err("scanline fill did not close its winding interval".into());
        }
    }
    if triangles.is_empty() {
        return Err("self-intersecting contour produced no scanline geometry".into());
    }
    Ok(triangles)
}

#[derive(Clone, Copy)]
struct ScanlineCrossing {
    start: [f32; 2],
    end: [f32; 2],
    direction: i32,
    middle_x: f32,
}

impl VectorFillRule {
    fn contains(self, winding: i32) -> bool {
        match self {
            Self::NonZero => winding != 0,
            Self::EvenOdd => winding.unsigned_abs() % 2 == 1,
        }
    }
}

fn x_at_y(start: [f32; 2], end: [f32; 2], y: f32) -> f32 {
    start[0] + (y - start[1]) * (end[0] - start[0]) / (end[1] - start[1])
}

fn tessellate_simple_loop(points: &[[f32; 2]]) -> Result<Vec<[f32; 2]>, String> {
    let points = simplified_loop(points);
    if points.len() < 3 {
        return Err("simple fill loop requires at least three points".into());
    }
    let winding = signed_area(&points).signum();
    if winding == 0.0 {
        return Err("simple fill loop has zero signed area".into());
    }

    let mut remaining = (0..points.len()).collect::<Vec<_>>();
    let mut triangles = Vec::with_capacity((points.len() - 2) * 3);
    while remaining.len() > 3 {
        let mut ear = None;
        for current in 0..remaining.len() {
            let previous = remaining[(current + remaining.len() - 1) % remaining.len()];
            let vertex = remaining[current];
            let next = remaining[(current + 1) % remaining.len()];
            let turn = cross(
                subtract(points[vertex], points[previous]),
                subtract(points[next], points[vertex]),
            );
            if turn * winding <= 1.0e-8 {
                continue;
            }
            if remaining.iter().copied().any(|candidate| {
                candidate != previous
                    && candidate != vertex
                    && candidate != next
                    && point_in_triangle(
                        points[candidate],
                        points[previous],
                        points[vertex],
                        points[next],
                    )
            }) {
                continue;
            }
            ear = Some((current, previous, vertex, next));
            break;
        }

        let Some((current, previous, vertex, next)) = ear else {
            return Err("simple fill loop could not find a valid ear".into());
        };
        triangles.extend([points[previous], points[vertex], points[next]]);
        remaining.remove(current);
    }
    triangles.extend([
        points[remaining[0]],
        points[remaining[1]],
        points[remaining[2]],
    ]);
    Ok(triangles)
}

fn point_in_triangle(point: [f32; 2], a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> bool {
    let ab = cross(subtract(b, a), subtract(point, a));
    let bc = cross(subtract(c, b), subtract(point, b));
    let ca = cross(subtract(a, c), subtract(point, c));
    const EPSILON: f32 = 1.0e-8;
    // Boundary points do not block an ear. Flattened curves and intentional
    // collinear samples often place a vertex exactly on a candidate edge.
    (ab > EPSILON && bc > EPSILON && ca > EPSILON)
        || (ab < -EPSILON && bc < -EPSILON && ca < -EPSILON)
}

fn segment_intersection(
    left_start: [f32; 2],
    left_end: [f32; 2],
    right_start: [f32; 2],
    right_end: [f32; 2],
) -> Option<(f32, f32, [f32; 2])> {
    let left = [left_end[0] - left_start[0], left_end[1] - left_start[1]];
    let right = [right_end[0] - right_start[0], right_end[1] - right_start[1]];
    let denominator = cross_2d(left, right);
    if denominator.abs() <= 1.0e-8 {
        return None;
    }

    let delta = [
        right_start[0] - left_start[0],
        right_start[1] - left_start[1],
    ];
    let left_t = cross_2d(delta, right) / denominator;
    let right_t = cross_2d(delta, left) / denominator;
    const ENDPOINT_EPSILON: f32 = 1.0e-5;
    if !(ENDPOINT_EPSILON..=1.0 - ENDPOINT_EPSILON).contains(&left_t)
        || !(ENDPOINT_EPSILON..=1.0 - ENDPOINT_EPSILON).contains(&right_t)
    {
        return None;
    }

    Some((
        left_t,
        right_t,
        [
            left_start[0] + left[0] * left_t,
            left_start[1] + left[1] * left_t,
        ],
    ))
}

fn cross_2d(left: [f32; 2], right: [f32; 2]) -> f32 {
    left[0] * right[1] - left[1] * right[0]
}

fn points_approximately_equal(left: [f32; 2], right: [f32; 2]) -> bool {
    const EPSILON: f32 = 1.0e-6;
    (left[0] - right[0]).abs() <= EPSILON && (left[1] - right[1]).abs() <= EPSILON
}

/// Validates whether a path is currently eligible for the bounded convex-fill
/// tessellator without allocating output geometry.
///
/// Importers can use this to diagnose unsupported topology before choosing a
/// fill path. It intentionally does not claim support for holes, multiple
/// contours, or arbitrary concave SVG fills.
pub fn validate_convex_fill(path: &VectorPath) -> Result<(), String> {
    if path.contours.len() != 1 {
        return Err("convex fill requires exactly one contour".into());
    }

    let contour = &path.contours[0];
    if !contour.closed {
        return Err("convex fill requires a closed contour".into());
    }
    if !contour.is_finite() {
        return Err("convex fill received non-finite coordinates".into());
    }

    let points = normalized_contour_points(&contour.points);
    if points.len() < 3 {
        return Err("convex fill requires at least three points".into());
    }

    let area = signed_area(&points);
    if area.abs() <= f32::EPSILON {
        return Err("convex fill received a zero-area contour".into());
    }

    let winding = area.signum();
    let mut turn_sign = 0.0;
    for index in 0..points.len() {
        let a = points[index];
        let b = points[(index + 1) % points.len()];
        let c = points[(index + 2) % points.len()];
        let turn = cross(subtract(b, a), subtract(c, b));
        if turn.abs() <= f32::EPSILON {
            continue;
        }
        if turn_sign == 0.0 {
            turn_sign = turn.signum();
        } else if turn.signum() != turn_sign {
            return Err("convex fill received a concave contour".into());
        }
    }

    if turn_sign != winding {
        return Err("convex fill contour winding is inconsistent".into());
    }

    Ok(())
}

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

fn normalized_contour_points(points: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let mut normalized = points.to_vec();
    if normalized.len() > 1 && normalized.first() == normalized.last() {
        normalized.pop();
    }
    normalized
}

fn signed_area(points: &[[f32; 2]]) -> f32 {
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

fn subtract(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn cross(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[1] - a[1] * b[0]
}

#[derive(Clone, Debug, Default)]
pub struct PathBuilder {
    contours: Vec<VectorContour>,
    current: Vec<[f32; 2]>,
}

impl PathBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn move_to(mut self, point: [f32; 2]) -> Self {
        self.finish_open_contour();
        self.current.push(point);
        self
    }

    pub fn line_to(mut self, point: [f32; 2]) -> Self {
        self.current.push(point);
        self
    }

    pub fn close(mut self) -> Self {
        if !self.current.is_empty() {
            self.contours
                .push(VectorContour::new(std::mem::take(&mut self.current), true));
        }
        self
    }

    pub fn rect(mut self, min: [f32; 2], size: [f32; 2]) -> Self {
        self.finish_open_contour();
        let max = [min[0] + size[0], min[1] + size[1]];
        self.contours.push(VectorContour::new(
            vec![min, [max[0], min[1]], max, [min[0], max[1]]],
            true,
        ));
        self
    }

    pub fn rounded_rect(mut self, min: [f32; 2], size: [f32; 2], radius: f32) -> Self {
        self.finish_open_contour();
        let max = [min[0] + size[0], min[1] + size[1]];
        let radius = radius
            .max(0.0)
            .min(size[0].abs() * 0.5)
            .min(size[1].abs() * 0.5);
        let segments = 8;
        let centers = [
            [max[0] - radius, min[1] + radius],
            [max[0] - radius, max[1] - radius],
            [min[0] + radius, max[1] - radius],
            [min[0] + radius, min[1] + radius],
        ];
        let start_angles = [-TAU / 4.0, 0.0, TAU / 4.0, TAU / 2.0];
        let mut points = Vec::with_capacity(segments * 4);

        for (center, start) in centers.into_iter().zip(start_angles) {
            for step in 0..segments {
                let angle = start + (step as f32 / segments as f32) * TAU / 4.0;
                points.push([
                    center[0] + radius * angle.cos(),
                    center[1] + radius * angle.sin(),
                ]);
            }
        }

        self.contours.push(VectorContour::new(points, true));
        self
    }

    pub fn build(mut self) -> VectorPath {
        self.finish_open_contour();
        VectorPath::new(self.contours)
    }

    fn finish_open_contour(&mut self) {
        if !self.current.is_empty() {
            self.contours
                .push(VectorContour::new(std::mem::take(&mut self.current), false));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rectangle_is_one_closed_contour_with_expected_bounds() {
        let path = PathBuilder::new().rect([-2.0, -1.0], [4.0, 3.0]).build();

        assert_eq!(path.contours.len(), 1);
        assert!(path.contours[0].closed);
        assert_eq!(path.contours[0].points.len(), 4);
        assert_eq!(path.bounds(), Some(([-2.0, -1.0], [2.0, 2.0])));
    }

    #[test]
    fn rounded_rectangle_is_finite_and_closed() {
        let path = PathBuilder::new()
            .rounded_rect([-1.0, -0.5], [2.0, 1.0], 0.25)
            .build();

        assert_eq!(path.contours.len(), 1);
        assert!(path.contours[0].closed);
        assert!(path.is_finite());
        assert_eq!(path.contours[0].points.len(), 32);
    }

    #[test]
    fn builder_preserves_open_contours() {
        let path = PathBuilder::new()
            .move_to([0.0, 0.0])
            .line_to([1.0, 0.0])
            .build();

        assert_eq!(
            path.contours,
            vec![VectorContour::new(vec![[0.0, 0.0], [1.0, 0.0]], false)]
        );
    }

    #[test]
    fn close_marks_only_the_current_contour_closed() {
        let path = PathBuilder::new()
            .move_to([0.0, 0.0])
            .line_to([1.0, 0.0])
            .close()
            .move_to([2.0, 0.0])
            .line_to([3.0, 0.0])
            .build();

        assert_eq!(path.contours.len(), 2);
        assert!(path.contours[0].closed);
        assert!(!path.contours[1].closed);
    }

    #[test]
    fn convex_fill_tessellates_a_rectangle_with_consistent_winding() {
        let path = PathBuilder::new().rect([0.0, 0.0], [2.0, 1.0]).build();
        let triangles = tessellate_convex_fill(&path).unwrap();

        assert_eq!(triangles.len(), 6);
        assert!(triangles.chunks_exact(3).all(|triangle| {
            cross(
                subtract(triangle[1], triangle[0]),
                subtract(triangle[2], triangle[0]),
            ) > 0.0
        }));
    }

    #[test]
    fn convex_fill_accepts_a_rounded_rectangle() {
        let path = PathBuilder::new()
            .rounded_rect([0.0, 0.0], [2.0, 1.0], 0.2)
            .build();

        let triangles = tessellate_convex_fill(&path).unwrap();

        assert_eq!(triangles.len(), (path.contours[0].points.len() - 2) * 3);
        assert!(triangles
            .iter()
            .all(|point| { point[0].is_finite() && point[1].is_finite() }));
    }

    #[test]
    fn convex_fill_normalizes_reversed_winding() {
        let path = VectorPath::new(vec![VectorContour::new(
            vec![[0.0, 0.0], [0.0, 1.0], [2.0, 1.0], [2.0, 0.0]],
            true,
        )]);

        let triangles = tessellate_convex_fill(&path).unwrap();

        assert_eq!(triangles.len(), 6);
        assert!(triangles.chunks_exact(3).all(|triangle| {
            cross(
                subtract(triangle[1], triangle[0]),
                subtract(triangle[2], triangle[0]),
            ) > 0.0
        }));
    }

    #[test]
    fn convex_fill_accepts_a_repeated_closing_point() {
        let path = VectorPath::new(vec![VectorContour::new(
            vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [0.0, 0.0]],
            true,
        )]);

        assert_eq!(tessellate_convex_fill(&path).unwrap().len(), 3);
    }

    #[test]
    fn convex_fill_rejects_unsupported_topology() {
        let open = PathBuilder::new()
            .move_to([0.0, 0.0])
            .line_to([1.0, 0.0])
            .line_to([0.0, 1.0])
            .build();
        let concave = VectorPath::new(vec![VectorContour::new(
            vec![[0.0, 0.0], [2.0, 0.0], [1.0, 0.5], [2.0, 1.0], [0.0, 1.0]],
            true,
        )]);

        assert!(tessellate_convex_fill(&open).is_err());
        assert!(tessellate_convex_fill(&concave).is_err());
    }

    #[test]
    fn convex_fill_rejects_degenerate_and_multi_contour_paths() {
        let degenerate = PathBuilder::new().rect([0.0, 0.0], [0.0, 1.0]).build();
        let multi = VectorPath::new(vec![
            VectorContour::new(vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]], true),
            VectorContour::new(vec![[2.0, 2.0], [3.0, 2.0], [2.0, 3.0]], true),
        ]);
        let non_finite = VectorPath::new(vec![VectorContour::new(
            vec![[0.0, 0.0], [f32::NAN, 0.0], [0.0, 1.0]],
            true,
        )]);

        assert!(tessellate_convex_fill(&degenerate).is_err());
        assert!(tessellate_convex_fill(&multi).is_err());
        assert!(tessellate_convex_fill(&non_finite).is_err());
    }

    #[test]
    fn convex_fill_validation_reports_support_without_geometry() {
        let supported = VectorPath::new(vec![VectorContour::new(
            vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            true,
        )]);
        let unsupported = VectorPath::new(vec![VectorContour::new(
            vec![[0.0, 0.0], [1.0, 0.0], [0.5, 0.25], [0.0, 1.0]],
            true,
        )]);

        assert!(validate_convex_fill(&supported).is_ok());
        assert!(validate_convex_fill(&unsupported).is_err());
    }

    #[test]
    fn general_fill_tessellates_a_concave_contour() {
        let path = VectorPath::new(vec![VectorContour::new(
            vec![
                [0.0, 0.0],
                [3.0, 0.0],
                [3.0, 1.0],
                [1.0, 1.0],
                [1.0, 3.0],
                [0.0, 3.0],
            ],
            true,
        )]);

        let triangles = tessellate_general_fill(&path).expect("concave fill");

        assert!(!triangles.is_empty());
        assert!(triangles
            .iter()
            .all(|point| { point[0].is_finite() && point[1].is_finite() }));
    }

    #[test]
    fn general_fill_ignores_duplicate_outline_points() {
        let path = VectorPath::new(vec![VectorContour::new(
            vec![
                [-1.0, -1.0],
                [1.0, -1.0],
                [1.0, -1.0],
                [1.0, 1.0],
                [-1.0, 1.0],
                [-1.0, -1.0],
            ],
            true,
        )]);
        let triangles = tessellate_general_fill(&path).expect("duplicate points are harmless");
        assert_eq!(triangles.len(), 6);
    }

    #[test]
    fn general_fill_ignores_collinear_flattening_samples() {
        let path = VectorPath::new(vec![VectorContour::new(
            vec![
                [0.0, 0.0],
                [0.5, 0.0],
                [1.0, 0.0],
                [1.0, 0.5],
                [1.0, 1.0],
                [0.0, 1.0],
            ],
            true,
        )]);

        let triangles = tessellate_general_fill(&path)
            .expect("collinear samples from a flattened contour are harmless");

        assert_eq!(triangles.len(), 6);
    }

    #[test]
    fn simple_fill_allows_vertices_on_candidate_ear_edges() {
        let points = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [2.0, 1.0], [0.0, 1.0]];

        let triangles = tessellate_simple_loop(&points)
            .expect("a boundary vertex should not block a valid ear");

        assert_eq!(triangles.len(), 6);
    }

    #[test]
    fn general_fill_preserves_self_intersecting_font_outline_extents() {
        let points = vec![
            [0.0050781253, 0.0],
            [0.057910156, 0.14550781],
            [0.079394534, 0.14550781],
            [0.13291016, 0.0],
            [0.113378905, 0.0],
            [0.08251953, 0.08632813],
            [0.075927734, 0.10629883],
            [0.06679688, 0.13681641],
            [0.0703125, 0.13681641],
            [0.061181642, 0.10590821],
            [0.05478516, 0.08632813],
            [0.024804687, 0.0],
            [0.0050781253, 0.0],
        ];
        let expected_min_x = points
            .iter()
            .map(|point| point[0])
            .fold(f32::INFINITY, f32::min);
        let path = VectorPath::new(vec![VectorContour::new(points, true)]);

        let triangles = tessellate_general_fill(&path).expect("self-intersecting font fill");
        let actual_min_x = triangles
            .iter()
            .map(|point| point[0])
            .fold(f32::INFINITY, f32::min);

        assert!(
            (actual_min_x - expected_min_x).abs() <= 1.0e-6,
            "expected min x {expected_min_x}, got {actual_min_x}"
        );
    }

    #[test]
    fn general_fill_tessellates_a_counter_with_multiple_contours() {
        let path = VectorPath::new(vec![
            VectorContour::new(vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]], true),
            VectorContour::new(vec![[1.0, 1.0], [1.0, 3.0], [3.0, 3.0], [3.0, 1.0]], true),
        ]);

        let triangles = tessellate_general_fill(&path).expect("counter fill");

        assert!(!triangles.is_empty());
        assert!(triangles
            .iter()
            .all(|point| { point[0].is_finite() && point[1].is_finite() }));
    }

    #[test]
    fn general_fill_supports_explicit_fill_rules() {
        let path = VectorPath::new(vec![
            VectorContour::new(vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]], true),
            VectorContour::new(vec![[1.0, 1.0], [1.0, 3.0], [3.0, 3.0], [3.0, 1.0]], true),
        ]);

        let non_zero = tessellate_general_fill_with_rule(&path, VectorFillRule::NonZero)
            .expect("non-zero counter fill");
        let even_odd = tessellate_general_fill_with_rule(&path, VectorFillRule::EvenOdd)
            .expect("even-odd counter fill");

        assert!(!non_zero.is_empty());
        assert!(!even_odd.is_empty());
        assert!(non_zero
            .iter()
            .chain(even_odd.iter())
            .all(|point| point[0].is_finite() && point[1].is_finite()));
    }

    #[test]
    fn general_fill_accepts_reversed_inner_contour_winding() {
        let path = VectorPath::new(vec![
            VectorContour::new(vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]], true),
            VectorContour::new(vec![[1.0, 1.0], [3.0, 1.0], [3.0, 3.0], [1.0, 3.0]], true),
        ]);

        let triangles = tessellate_general_fill_with_rule(&path, VectorFillRule::NonZero)
            .expect("reversed inner contour fill");

        assert!(!triangles.is_empty());
        assert!(triangles
            .iter()
            .all(|point| point[0].is_finite() && point[1].is_finite()));
    }

    #[test]
    fn stroke_uses_explicit_open_and_closed_contour_state() {
        let open = VectorContour::new(vec![[0.0, 0.0], [1.0, 0.0]], false);
        let closed = VectorContour::new(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]], true);

        assert_eq!(tessellate_stroke(&open, 0.1).len(), 78);
        assert_eq!(tessellate_stroke(&closed, 0.1).len(), 24);
    }

    #[test]
    fn closed_stroke_normalizes_a_repeated_endpoint_without_inference() {
        let repeated = VectorContour::new(
            vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0]],
            true,
        );
        let explicit =
            VectorContour::new(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]], true);

        assert_eq!(
            tessellate_stroke(&repeated, 0.1).len(),
            tessellate_stroke(&explicit, 0.1).len()
        );
    }

    #[test]
    fn path_collection_strokes_all_contours_in_order() {
        let paths = vec![
            VectorPath::new(vec![VectorContour::new(
                vec![[0.0, 0.0], [1.0, 0.0]],
                false,
            )]),
            VectorPath::new(vec![VectorContour::new(
                vec![[0.0, 1.0], [1.0, 1.0]],
                false,
            )]),
        ];

        let mesh = tessellate_path_strokes(&paths, 0.1);

        assert_eq!(
            mesh.len(),
            tessellate_stroke(&paths[0].contours[0], 0.1).len() * 2
        );
        assert!(mesh
            .iter()
            .all(|vertex| vertex.iter().all(|value| value.is_finite())));
    }
}
