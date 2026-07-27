use super::geometry::{cross, normalized_contour_points, signed_area, subtract};
use super::VectorPath;

mod lyon;

use lyon::{mesh_preserves_contour_bounds, tessellate_lyon_contours};

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

/// Tessellates font outlines through the robust multi-contour path.
///
/// Font contours are provider data and can contain sharp re-entrant joins or
/// borderline topology that is valid but unsuitable for the optimistic
/// single-contour ear-clipping shortcut used by ordinary fills. Keep this
/// policy at the font boundary rather than weakening the general fill fast
/// path for every producer.
pub(crate) fn tessellate_font_fill_with_rule(
    path: &VectorPath,
    fill_rule: VectorFillRule,
) -> Result<Vec<[f32; 2]>, String> {
    if path.contours.is_empty() {
        return Err("font fill requires at least one contour".into());
    }
    if path
        .contours
        .iter()
        .any(|contour| !contour.closed || contour.points.len() < 3)
    {
        return Err("font fill requires closed contours with at least three points".into());
    }
    if !path.is_finite() {
        return Err("font fill received non-finite coordinates".into());
    }

    let contours = path
        .contours
        .iter()
        .map(|contour| sanitized_closed_points(&contour.points))
        .collect::<Vec<_>>();
    if contours.iter().any(|points| points.len() < 3) {
        return Err("font fill contour became degenerate after sanitization".into());
    }

    let triangles = tessellate_font_components(&contours, fill_rule)?;
    if triangles.is_empty() && contours.len() == 1 {
        // Very thin single-contour glyphs such as slash can collapse inside
        // Lyon's fill tolerance at small output scales. The contour is still
        // a valid simple polygon, so preserve it with the local fallback.
        return tessellate_simple_loop(&contours[0]);
    }
    if triangles.iter().flatten().any(|value| !value.is_finite()) {
        return Err("font fill tessellation produced no finite geometry".into());
    }
    Ok(triangles)
}

fn tessellate_font_components(
    contours: &[Vec<[f32; 2]>],
    fill_rule: VectorFillRule,
) -> Result<Vec<[f32; 2]>, String> {
    let mut parents = vec![None; contours.len()];
    for child in 0..contours.len() {
        let mut best_parent = None;
        let mut best_area = f32::INFINITY;
        for candidate in 0..contours.len() {
            if child == candidate
                || !contour_bounds_contain(&contours[candidate], &contours[child])
                || !point_in_polygon(contours[child][0], &contours[candidate])
            {
                continue;
            }
            let area = signed_area(&contours[candidate]).abs();
            if area < best_area {
                best_parent = Some(candidate);
                best_area = area;
            }
        }
        parents[child] = best_parent;
    }

    // Keep ordinary font contours on Lyon's established path. Component
    // partitioning is only needed when the outline contains multiple nested
    // holes/components, as in percent-like glyphs.
    if parents.iter().filter(|parent| parent.is_some()).count() < 2 {
        return tessellate_lyon_contours(contours, fill_rule);
    }

    let mut triangles = Vec::new();
    for root in 0..contours.len() {
        if parents[root].is_some() {
            continue;
        }
        let mut component = vec![root];
        let mut cursor = 0;
        while cursor < component.len() {
            let parent = component[cursor];
            let children = parents
                .iter()
                .enumerate()
                .filter_map(|(index, candidate)| (*candidate == Some(parent)).then_some(index))
                .filter(|index| !component.contains(index))
                .collect::<Vec<_>>();
            component.extend(children);
            cursor += 1;
        }
        let component_contours = component
            .iter()
            .map(|index| contours[*index].clone())
            .collect::<Vec<_>>();
        let component_triangles = tessellate_lyon_contours(&component_contours, fill_rule)?;
        if component_triangles.is_empty() && component_contours.len() == 1 {
            triangles.extend(tessellate_simple_loop(&component_contours[0])?);
        } else {
            triangles.extend(component_triangles);
        }
    }
    Ok(triangles)
}

fn point_in_polygon(point: [f32; 2], polygon: &[[f32; 2]]) -> bool {
    let mut inside = false;
    for (start, end) in polygon
        .iter()
        .zip(polygon.iter().cycle().skip(1))
        .take(polygon.len())
    {
        let crosses = (start[1] > point[1]) != (end[1] > point[1]);
        if crosses {
            let x_at_y =
                start[0] + (point[1] - start[1]) * (end[0] - start[0]) / (end[1] - start[1]);
            if point[0] < x_at_y {
                inside = !inside;
            }
        }
    }
    inside
}

fn contour_bounds_contain(outer: &[[f32; 2]], inner: &[[f32; 2]]) -> bool {
    let outer = contour_bounds(outer);
    let inner = contour_bounds(inner);
    inner[0] > outer[0] && inner[1] > outer[1] && inner[2] < outer[2] && inner[3] < outer[3]
}

fn contour_bounds(points: &[[f32; 2]]) -> [f32; 4] {
    points.iter().fold(
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
    )
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

pub(super) fn tessellate_simple_loop(points: &[[f32; 2]]) -> Result<Vec<[f32; 2]>, String> {
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
