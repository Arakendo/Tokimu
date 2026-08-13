//! Producer-neutral geometry diagnostics used by corpus runners and artifacts.

use crate::MeshValidation;
use ui_tools::VectorPath;

pub(crate) fn contours_svg(path: &VectorPath) -> String {
    let mut data = String::new();
    for contour in &path.contours {
        if let Some(start) = contour.points.first() {
            data.push_str(&format!("M {} {} ", start[0], start[1]));
            for point in &contour.points[1..] {
                data.push_str(&format!("L {} {} ", point[0], point[1]));
            }
            if contour.closed {
                data.push('Z');
            }
        }
    }
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1 1\"><path d=\"{data}\" fill=\"none\" stroke=\"black\"/></svg>\n"
    )
}

pub(crate) fn mesh_svg(triangles: &[[f32; 2]]) -> String {
    let mut polygons = String::new();
    for triangle in triangles.chunks_exact(3) {
        polygons.push_str(&format!(
            "<polygon points=\"{},{} {},{} {},{}\"/>",
            triangle[0][0],
            triangle[0][1],
            triangle[1][0],
            triangle[1][1],
            triangle[2][0],
            triangle[2][1]
        ));
    }
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1 1\"><g fill=\"none\" stroke=\"black\">{polygons}</g></svg>\n"
    )
}

pub(crate) fn rasterize_mesh(
    triangles: &[[f32; 2]],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, String> {
    let (min, max) =
        bounds_of_points(triangles).ok_or_else(|| "mesh has no vertices".to_owned())?;
    let span_x = (max[0] - min[0]).max(f32::EPSILON);
    let span_y = (max[1] - min[1]).max(f32::EPSILON);
    let scale = ((width - 48) as f32 / span_x).min((height - 48) as f32 / span_y);
    let offset_x = (width as f32 - span_x * scale) * 0.5;
    let offset_y = (height as f32 - span_y * scale) * 0.5;
    let map = |point: [f32; 2]| {
        [
            offset_x + (point[0] - min[0]) * scale,
            height as f32 - offset_y - (point[1] - min[1]) * scale,
        ]
    };
    let mut pixels = [12_u8, 15, 21, 255].repeat((width * height) as usize);
    for triangle in triangles.chunks_exact(3) {
        let points = [map(triangle[0]), map(triangle[1]), map(triangle[2])];
        let min_x = points
            .iter()
            .map(|point| point[0])
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.0) as u32;
        let max_x = points
            .iter()
            .map(|point| point[0])
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min(width as f32 - 1.0) as u32;
        let min_y = points
            .iter()
            .map(|point| point[1])
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.0) as u32;
        let max_y = points
            .iter()
            .map(|point| point[1])
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min(height as f32 - 1.0) as u32;
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if point_in_triangle([x as f32 + 0.5, y as f32 + 0.5], points) {
                    let offset = ((y * width + x) * 4) as usize;
                    pixels[offset..offset + 4].copy_from_slice(&[165, 210, 245, 255]);
                }
            }
        }
    }
    Ok(pixels)
}

pub(crate) fn point_in_triangle(point: [f32; 2], triangle: [[f32; 2]; 3]) -> bool {
    let edge = |a: [f32; 2], b: [f32; 2], p: [f32; 2]| {
        (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])
    };
    let a = edge(triangle[0], triangle[1], point);
    let b = edge(triangle[1], triangle[2], point);
    let c = edge(triangle[2], triangle[0], point);
    (a >= 0.0 && b >= 0.0 && c >= 0.0) || (a <= 0.0 && b <= 0.0 && c <= 0.0)
}

pub(crate) fn bounds_of_points(points: &[[f32; 2]]) -> Option<([f32; 2], [f32; 2])> {
    let first = *points.first()?;
    let mut min = first;
    let mut max = first;
    for point in &points[1..] {
        min[0] = min[0].min(point[0]);
        min[1] = min[1].min(point[1]);
        max[0] = max[0].max(point[0]);
        max[1] = max[1].max(point[1]);
    }
    Some((min, max))
}

pub(crate) fn union_bounds(
    first: ([f32; 2], [f32; 2]),
    second: ([f32; 2], [f32; 2]),
) -> ([f32; 2], [f32; 2]) {
    (
        [first.0[0].min(second.0[0]), first.0[1].min(second.0[1])],
        [first.1[0].max(second.1[0]), first.1[1].max(second.1[1])],
    )
}

pub(crate) fn signed_area(points: &[[f32; 2]]) -> f32 {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .map(|(a, b)| a[0] * b[1] - b[0] * a[1])
        .sum::<f32>()
        * 0.5
}

pub(crate) fn triangle_area(triangle: &[[f32; 2]]) -> f32 {
    if triangle.len() < 3 {
        return 0.0;
    }
    ((triangle[1][0] - triangle[0][0]) * (triangle[2][1] - triangle[0][1])
        - (triangle[1][1] - triangle[0][1]) * (triangle[2][0] - triangle[0][0]))
        .abs()
        * 0.5
}

pub(crate) fn validate_mesh(triangles: &[[f32; 2]]) -> MeshValidation {
    let complete_triangles = triangles.len().is_multiple_of(3);
    let finite = triangles
        .iter()
        .all(|point| point[0].is_finite() && point[1].is_finite());
    let degenerate_triangles = triangles
        .chunks_exact(3)
        .filter(|triangle| triangle_area(triangle) <= f32::EPSILON)
        .count();
    let total_area = triangles.chunks_exact(3).map(triangle_area).sum();
    MeshValidation {
        finite,
        complete_triangles,
        triangle_count: triangles.len() / 3,
        degenerate_triangles,
        total_area,
    }
}

pub(crate) fn format_mesh_summary(validation: &MeshValidation) -> String {
    format!(
        "triangles={} vertices={} finite={} complete={} degenerate={} area={:.6}",
        validation.triangle_count,
        validation.triangle_count * 3,
        validation.finite,
        validation.complete_triangles,
        validation.degenerate_triangles,
        validation.total_area
    )
}
