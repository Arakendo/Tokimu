//! Stable structural fingerprints and topology diagnostics.

use crate::SegmentIntersectionArtifact;
use std::cmp::Ordering;
use ui_tools::VectorPath;

/// Produces an order-independent fingerprint for a flat triangle list.
///
/// Tessellators are free to emit equivalent triangles in different orders, so
/// this evidence normalizes vertex order within each triangle and triangle
/// order across the mesh. Coordinates are quantized only for the fingerprint;
/// the raw mesh artifact remains available for detailed diagnostics.
pub(crate) fn canonical_triangle_hash(triangles: &[[f32; 2]]) -> String {
    let mut canonical = triangles
        .chunks_exact(3)
        .map(|triangle| {
            let mut points = [
                canonical_point(triangle[0]),
                canonical_point(triangle[1]),
                canonical_point(triangle[2]),
            ];
            points.sort_by(compare_points);
            points
        })
        .collect::<Vec<_>>();
    canonical.sort_by(compare_triangles);

    let mut hash = 0xcbf29ce484222325;
    for point in canonical.into_iter().flatten() {
        for byte in point[0]
            .to_bits()
            .to_le_bytes()
            .into_iter()
            .chain(point[1].to_bits().to_le_bytes())
        {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    format!("fnv1a64:{hash:016x}")
}

/// Produces a stable, order-preserving fingerprint for source-to-vector
/// lowering. Contour order and open/closed topology are retained deliberately:
/// they are semantic evidence before any mesh implementation is selected.
pub(crate) fn canonical_path_hash(paths: &[&VectorPath]) -> String {
    let mut hash = 0xcbf29ce484222325;
    for path in paths {
        for contour in &path.contours {
            hash_byte(&mut hash, u8::from(contour.closed));
            hash_bytes(&mut hash, &(contour.points.len() as u64).to_le_bytes());
            for point in &contour.points {
                let point = canonical_point(*point);
                hash_bytes(&mut hash, &point[0].to_bits().to_le_bytes());
                hash_bytes(&mut hash, &point[1].to_bits().to_le_bytes());
            }
        }
        hash_byte(&mut hash, 0xff);
    }
    format!("fnv1a64:{hash:016x}")
}

fn hash_byte(hash: &mut u64, byte: u8) {
    *hash ^= u64::from(byte);
    *hash = hash.wrapping_mul(0x100000001b3);
}

fn hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        hash_byte(hash, *byte);
    }
}

fn canonical_point(point: [f32; 2]) -> [f32; 2] {
    [quantize_coordinate(point[0]), quantize_coordinate(point[1])]
}

fn quantize_coordinate(value: f32) -> f32 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

fn compare_points(left: &[f32; 2], right: &[f32; 2]) -> Ordering {
    left[0]
        .partial_cmp(&right[0])
        .unwrap_or(Ordering::Equal)
        .then_with(|| left[1].partial_cmp(&right[1]).unwrap_or(Ordering::Equal))
}

fn compare_triangles(left: &[[f32; 2]; 3], right: &[[f32; 2]; 3]) -> Ordering {
    left.iter()
        .zip(right.iter())
        .map(|(left, right)| compare_points(left, right))
        .find(|ordering| *ordering != Ordering::Equal)
        .unwrap_or(Ordering::Equal)
}

pub(crate) fn segment_intersections(path: &VectorPath) -> Vec<SegmentIntersectionArtifact> {
    let mut intersections = Vec::new();
    for (contour_index, contour) in path.contours.iter().enumerate() {
        let segment_count = if contour.closed {
            contour.points.len()
        } else {
            contour.points.len().saturating_sub(1)
        };
        for first in 0..segment_count {
            let first_start = contour.points[first];
            let first_end = contour.points[(first + 1) % contour.points.len()];
            for second in (first + 1)..segment_count {
                if second == first + 1
                    || (contour.closed && first == 0 && second + 1 == segment_count)
                {
                    continue;
                }
                let second_start = contour.points[second];
                let second_end = contour.points[(second + 1) % contour.points.len()];
                if let Some(point) =
                    line_segment_intersection(first_start, first_end, second_start, second_end)
                {
                    intersections.push(SegmentIntersectionArtifact {
                        first_contour: contour_index,
                        first_segment: first,
                        second_contour: contour_index,
                        second_segment: second,
                        point,
                    });
                }
            }
        }
    }
    intersections
}

fn line_segment_intersection(
    first_start: [f32; 2],
    first_end: [f32; 2],
    second_start: [f32; 2],
    second_end: [f32; 2],
) -> Option<[f32; 2]> {
    let first_direction = [first_end[0] - first_start[0], first_end[1] - first_start[1]];
    let second_direction = [
        second_end[0] - second_start[0],
        second_end[1] - second_start[1],
    ];
    let denominator = cross(first_direction, second_direction);
    if denominator.abs() <= f32::EPSILON {
        return None;
    }
    let offset = [
        second_start[0] - first_start[0],
        second_start[1] - first_start[1],
    ];
    let first_factor = cross(offset, second_direction) / denominator;
    let second_factor = cross(offset, first_direction) / denominator;
    if (0.000001..=0.999999).contains(&first_factor)
        && (0.000001..=0.999999).contains(&second_factor)
    {
        Some([
            first_start[0] + first_factor * first_direction[0],
            first_start[1] + first_factor * first_direction[1],
        ])
    } else {
        None
    }
}

fn cross(first: [f32; 2], second: [f32; 2]) -> f32 {
    first[0] * second[1] - first[1] * second[0]
}

pub(crate) fn fnv1a64(bytes: &[u8], character: char) -> u64 {
    let mut hash = 0xcbf29ce484222325;
    for byte in bytes
        .iter()
        .copied()
        .chain((character as u32).to_le_bytes())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub(crate) fn fnv1a64_bytes(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
