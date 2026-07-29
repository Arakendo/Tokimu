//! CGM-owned primitive lowering into provider-neutral vector paths.
//!
//! This module is intentionally narrow. It lowers only source forms whose
//! topology and coordinate meaning are already demonstrated by the selected
//! corpus. CGM attributes and provenance remain attached to the adapter record;
//! `VectorPath` itself stays free of CGM concepts.

use std::f64::consts::TAU;

use ui_tools::{VectorContour, VectorPath};

use crate::{
    CgmError, CgmPicture, CgmPictureControlState, CgmPresentationState, CgmPrimitive,
    CgmPrimitiveKind, CgmResult, CgmVdcExtent,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CgmPrimitiveTopology {
    Open,
    Closed,
}

const CURVE_FLATTENING_SEGMENTS: usize = 32;

/// A provider-neutral path with the CGM-only evidence that produced it.
#[derive(Clone, Debug, PartialEq)]
pub struct CgmVectorPrimitive {
    pub source_element: usize,
    pub source_offset: usize,
    pub attribute_count: usize,
    pub state: CgmPresentationState,
    pub controls: CgmPictureControlState,
    pub topology: CgmPrimitiveTopology,
    pub path: VectorPath,
}

/// Lowers every currently admitted primitive in a single picture.
pub fn lower_picture_primitives(picture: &CgmPicture) -> CgmResult<Vec<CgmVectorPrimitive>> {
    let extent = picture
        .descriptor
        .vdc_extent
        .ok_or_else(|| CgmError::MissingVdcExtent {
            picture: picture.name.clone(),
        })?;

    picture
        .primitives
        .iter()
        .map(|primitive| lower_primitive(primitive, extent))
        .collect()
}

/// Lowers one CGM primitive through the currently admitted geometry profile.
pub fn lower_primitive(
    primitive: &CgmPrimitive,
    extent: CgmVdcExtent,
) -> CgmResult<CgmVectorPrimitive> {
    let (points, closed, topology) = match &primitive.kind {
        CgmPrimitiveKind::Polyline { points } => (
            normalize_points(points, extent, primitive.source_offset)?,
            false,
            CgmPrimitiveTopology::Open,
        ),
        CgmPrimitiveKind::Polygon { points } => (
            normalize_points(points, extent, primitive.source_offset)?,
            true,
            CgmPrimitiveTopology::Closed,
        ),
        CgmPrimitiveKind::PolygonSet { .. } => {
            return Err(CgmError::UnsupportedPrimitiveLowering {
                offset: primitive.source_offset,
                kind: "polygon-set point/flag topology",
            })
        }
        CgmPrimitiveKind::Rectangle { first, second } => (
            normalize_points(
                &[
                    *first,
                    [second[0], first[1]],
                    *second,
                    [first[0], second[1]],
                ],
                extent,
                primitive.source_offset,
            )?,
            true,
            CgmPrimitiveTopology::Closed,
        ),
        CgmPrimitiveKind::Circle { center, radius } => (
            normalize_circle(*center, *radius, extent, primitive.source_offset)?,
            true,
            CgmPrimitiveTopology::Closed,
        ),
        CgmPrimitiveKind::Ellipse {
            center,
            first_axis,
            second_axis,
        } => (
            normalize_ellipse(
                *center,
                *first_axis,
                *second_axis,
                extent,
                primitive.source_offset,
            )?,
            true,
            CgmPrimitiveTopology::Closed,
        ),
        CgmPrimitiveKind::CircularArc {
            center,
            start_vector,
            end_vector,
            radius,
        } => (
            normalize_circular_arc(
                *center,
                *start_vector,
                *end_vector,
                *radius,
                extent,
                primitive.source_offset,
            )?,
            false,
            CgmPrimitiveTopology::Open,
        ),
        CgmPrimitiveKind::EllipticalArc {
            center,
            first_axis,
            second_axis,
            start_vector,
            end_vector,
        } => (
            normalize_elliptical_arc(
                *center,
                *first_axis,
                *second_axis,
                *start_vector,
                *end_vector,
                extent,
                primitive.source_offset,
            )?,
            false,
            CgmPrimitiveTopology::Open,
        ),
    };

    Ok(CgmVectorPrimitive {
        source_element: primitive.source_element,
        source_offset: primitive.source_offset,
        attribute_count: primitive.attribute_count,
        state: primitive.state.clone(),
        controls: primitive.controls.clone(),
        topology,
        path: VectorPath::new(vec![VectorContour::new(points, closed)]),
    })
}

/// Flattens a CGM circle in source VDC space before normalizing each sample.
///
/// This preserves a potentially non-square source extent: an admitted circle
/// can become an ellipse in normalized presentation coordinates, which is a
/// coordinate mapping outcome rather than an implicit renderer correction.
fn normalize_circle(
    center: [i32; 2],
    radius: i32,
    extent: CgmVdcExtent,
    source_offset: usize,
) -> CgmResult<Vec<[f32; 2]>> {
    if radius <= 0 {
        return Err(CgmError::InvalidPrimitive {
            offset: source_offset,
            reason: "circle radius must be positive before flattening".to_owned(),
        });
    }
    (0..CURVE_FLATTENING_SEGMENTS)
        .map(|step| {
            let angle = step as f64 * TAU / CURVE_FLATTENING_SEGMENTS as f64;
            normalize_source_point(
                [
                    f64::from(center[0]) + f64::from(radius) * angle.cos(),
                    f64::from(center[1]) + f64::from(radius) * angle.sin(),
                ],
                extent,
                source_offset,
            )
        })
        .collect()
}

/// Flattens a CGM ellipse using its two source conjugate-diameter endpoints.
///
/// CGM stores the endpoints rather than pre-subtracted axis vectors. The
/// source-space vectors are therefore `first_axis - center` and
/// `second_axis - center`; samples are normalized only after this source
/// semantic is resolved.
fn normalize_ellipse(
    center: [i32; 2],
    first_axis: [i32; 2],
    second_axis: [i32; 2],
    extent: CgmVdcExtent,
    source_offset: usize,
) -> CgmResult<Vec<[f32; 2]>> {
    let first = source_vector(first_axis, center);
    let second = source_vector(second_axis, center);
    if first == [0.0, 0.0] || second == [0.0, 0.0] {
        return Err(CgmError::InvalidPrimitive {
            offset: source_offset,
            reason: "ellipse conjugate diameters must not collapse at the center".to_owned(),
        });
    }
    (0..CURVE_FLATTENING_SEGMENTS)
        .map(|step| {
            let angle = step as f64 * TAU / CURVE_FLATTENING_SEGMENTS as f64;
            normalize_source_point(
                [
                    f64::from(center[0]) + first[0] * angle.cos() + second[0] * angle.sin(),
                    f64::from(center[1]) + first[1] * angle.cos() + second[1] * angle.sin(),
                ],
                extent,
                source_offset,
            )
        })
        .collect()
}

/// Flattens an admitted open CGM circular arc. The input vectors are source
/// VDC offsets from `center`; CGM's circular-arc primitive sweeps
/// counter-clockwise from start to end.
fn normalize_circular_arc(
    center: [i32; 2],
    start: [i32; 2],
    end: [i32; 2],
    radius: i32,
    extent: CgmVdcExtent,
    source_offset: usize,
) -> CgmResult<Vec<[f32; 2]>> {
    if radius <= 0 {
        return Err(CgmError::InvalidPrimitive {
            offset: source_offset,
            reason: "circular arc radius must be positive before flattening".to_owned(),
        });
    }
    let start_angle = vector_angle(start, source_offset, "circular arc start vector")?;
    let end_angle = vector_angle(end, source_offset, "circular arc end vector")?;
    let sweep = counter_clockwise_sweep(start_angle, end_angle);

    normalize_arc_samples(extent, source_offset, |fraction| {
        let angle = start_angle + sweep * fraction;
        [
            f64::from(center[0]) + f64::from(radius) * angle.cos(),
            f64::from(center[1]) + f64::from(radius) * angle.sin(),
        ]
    })
}

/// Flattens an admitted open CGM elliptical arc. Start and end vectors are
/// expressed in the ellipse's conjugate-diameter basis, not as a renderer
/// transform. Resolving that source meaning remains in this adapter.
fn normalize_elliptical_arc(
    center: [i32; 2],
    first_axis: [i32; 2],
    second_axis: [i32; 2],
    start: [i32; 2],
    end: [i32; 2],
    extent: CgmVdcExtent,
    source_offset: usize,
) -> CgmResult<Vec<[f32; 2]>> {
    let first = source_vector(first_axis, center);
    let second = source_vector(second_axis, center);
    let start_coefficients = solve_conjugate_coefficients(
        first,
        second,
        [f64::from(start[0]), f64::from(start[1])],
        source_offset,
    )?;
    let end_coefficients = solve_conjugate_coefficients(
        first,
        second,
        [f64::from(end[0]), f64::from(end[1])],
        source_offset,
    )?;
    let start_angle = start_coefficients[1].atan2(start_coefficients[0]);
    let end_angle = end_coefficients[1].atan2(end_coefficients[0]);
    let sweep = counter_clockwise_sweep(start_angle, end_angle);

    normalize_arc_samples(extent, source_offset, |fraction| {
        let angle = start_angle + sweep * fraction;
        [
            f64::from(center[0]) + first[0] * angle.cos() + second[0] * angle.sin(),
            f64::from(center[1]) + first[1] * angle.cos() + second[1] * angle.sin(),
        ]
    })
}

fn normalize_arc_samples(
    extent: CgmVdcExtent,
    source_offset: usize,
    sample: impl Fn(f64) -> [f64; 2],
) -> CgmResult<Vec<[f32; 2]>> {
    (0..=CURVE_FLATTENING_SEGMENTS)
        .map(|step| {
            let source = sample(step as f64 / CURVE_FLATTENING_SEGMENTS as f64);
            normalize_source_point(source, extent, source_offset)
        })
        .collect()
}

fn vector_angle(vector: [i32; 2], source_offset: usize, name: &str) -> CgmResult<f64> {
    if vector == [0, 0] {
        return Err(CgmError::InvalidPrimitive {
            offset: source_offset,
            reason: format!("{name} must not collapse at the center"),
        });
    }
    Ok(f64::from(vector[1]).atan2(f64::from(vector[0])))
}

fn solve_conjugate_coefficients(
    first: [f64; 2],
    second: [f64; 2],
    vector: [f64; 2],
    source_offset: usize,
) -> CgmResult<[f64; 2]> {
    let determinant = first[0] * second[1] - first[1] * second[0];
    if determinant.abs() <= f64::EPSILON {
        return Err(CgmError::InvalidPrimitive {
            offset: source_offset,
            reason: "elliptical arc conjugate diameters must not be collinear".to_owned(),
        });
    }
    let x = vector[0];
    let y = vector[1];
    Ok([
        (x * second[1] - y * second[0]) / determinant,
        (first[0] * y - first[1] * x) / determinant,
    ])
}

fn counter_clockwise_sweep(start: f64, end: f64) -> f64 {
    let sweep = (end - start).rem_euclid(TAU);
    if sweep <= f64::EPSILON {
        TAU
    } else {
        sweep
    }
}

fn source_vector(endpoint: [i32; 2], center: [i32; 2]) -> [f64; 2] {
    [
        f64::from(endpoint[0]) - f64::from(center[0]),
        f64::from(endpoint[1]) - f64::from(center[1]),
    ]
}

fn normalize_source_point(
    point: [f64; 2],
    extent: CgmVdcExtent,
    source_offset: usize,
) -> CgmResult<[f32; 2]> {
    let delta_x = f64::from(extent.second[0]) - f64::from(extent.first[0]);
    let delta_y = f64::from(extent.second[1]) - f64::from(extent.first[1]);
    if delta_x == 0.0 || delta_y == 0.0 {
        return Err(CgmError::InvalidVdcExtent {
            offset: source_offset,
            reason: "cannot normalize against a degenerate VDC extent".to_owned(),
        });
    }
    let normalized = [
        ((point[0] - f64::from(extent.first[0])) / delta_x) as f32,
        ((point[1] - f64::from(extent.first[1])) / delta_y) as f32,
    ];
    if normalized.iter().all(|value| value.is_finite()) {
        Ok(normalized)
    } else {
        Err(CgmError::InvalidPrimitive {
            offset: source_offset,
            reason: "curve normalization produced non-finite coordinates".to_owned(),
        })
    }
}

fn normalize_points(
    points: &[[i32; 2]],
    extent: CgmVdcExtent,
    source_offset: usize,
) -> CgmResult<Vec<[f32; 2]>> {
    points
        .iter()
        .copied()
        .map(|point| {
            extent
                .normalize(point)
                .ok_or_else(|| CgmError::InvalidVdcExtent {
                    offset: source_offset,
                    reason: "cannot normalize against a degenerate VDC extent".to_owned(),
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CgmPrimitiveKind, CgmVdcExtent};

    fn primitive(kind: CgmPrimitiveKind) -> CgmPrimitive {
        CgmPrimitive {
            source_element: 3,
            source_offset: 12,
            attribute_count: 0,
            state: CgmPresentationState::default(),
            controls: CgmPictureControlState::default(),
            kind,
        }
    }

    #[test]
    fn lowers_admitted_primitives_without_cgm_in_vector_path() {
        let extent = CgmVdcExtent {
            first: [0, 100],
            second: [200, 0],
        };
        let polyline = lower_primitive(
            &primitive(CgmPrimitiveKind::Polyline {
                points: vec![[0, 100], [200, 0]],
            }),
            extent,
        )
        .expect("polyline should lower");
        assert_eq!(polyline.topology, CgmPrimitiveTopology::Open);
        assert!(!polyline.path.contours[0].closed);
        assert_eq!(
            polyline.path.contours[0].points,
            vec![[0.0, 0.0], [1.0, 1.0]]
        );

        let rectangle = lower_primitive(
            &primitive(CgmPrimitiveKind::Rectangle {
                first: [0, 100],
                second: [200, 0],
            }),
            extent,
        )
        .expect("rectangle should lower");
        assert_eq!(rectangle.topology, CgmPrimitiveTopology::Closed);
        assert!(rectangle.path.contours[0].closed);
        assert_eq!(rectangle.path.contours[0].points.len(), 4);

        let circle = lower_primitive(
            &primitive(CgmPrimitiveKind::Circle {
                center: [100, 50],
                radius: 25,
            }),
            extent,
        )
        .expect("circle should lower");
        assert!(circle.path.contours[0].closed);
        assert_eq!(
            circle.path.contours[0].points.len(),
            CURVE_FLATTENING_SEGMENTS
        );
        assert!(circle.path.is_finite());

        let ellipse = lower_primitive(
            &primitive(CgmPrimitiveKind::Ellipse {
                center: [100, 50],
                first_axis: [150, 50],
                second_axis: [100, 75],
            }),
            extent,
        )
        .expect("ellipse should lower");
        assert!(ellipse.path.contours[0].closed);
        assert_eq!(
            ellipse.path.contours[0].points.len(),
            CURVE_FLATTENING_SEGMENTS
        );
        assert_eq!(ellipse.path.contours[0].points[0], [0.75, 0.5]);
        assert!(ellipse.path.is_finite());
    }

    #[test]
    fn curve_lowering_uses_wide_intermediate_vdc_arithmetic() {
        let extent = CgmVdcExtent {
            first: [i32::MIN, i32::MIN],
            second: [i32::MAX, i32::MAX],
        };
        let ellipse = lower_primitive(
            &primitive(CgmPrimitiveKind::Ellipse {
                center: [0, 0],
                first_axis: [i32::MAX, 0],
                second_axis: [0, i32::MAX],
            }),
            extent,
        )
        .expect("full-range ellipse should lower without signed subtraction overflow");

        assert!(ellipse.path.is_finite());
        assert!(ellipse.path.bounds().is_some());
    }
}
