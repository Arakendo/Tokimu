//! Corpus-local AR-0026 chart-transition control for AR-0019.
//!
//! Chart identity, adjacency, angular measures, and orientation declarations
//! are deliberately local data in this module. The A/B/C comparison below asks a
//! narrower question: does the already inventoried ordinary math surface carry
//! the same local-point, local-direction, composition, inverse, and
//! orientation-sign facts? It does not propose a Tokimu chart API.

use crate::{
    alternative_b::{Mat4 as BMat4, Vec3 as BVec3},
    alternative_c::{Mat4 as CMat4, Vec3 as CVec3},
};
use tokimu_core::math::{Mat4 as AMat4, Vec3 as AVec3};

/// Corpus-only identity for a local coordinate chart.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChartId {
    Entry,
    Junction,
    Exit,
}

/// Authored meaning retained separately from a numerically invertible matrix.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrientationBehavior {
    Preserving,
    Reversing,
}

/// One deterministic observation from a locally Euclidean chart transition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartTrace {
    pub path: [ChartId; 3],
    pub endpoint: [f32; 3],
    pub restored_point: [f32; 3],
    pub transported_direction: [f32; 3],
    pub composed_orientation: OrientationBehavior,
    pub reflected_orientation: OrientationBehavior,
}

fn orientation_from_a(transform: AMat4) -> OrientationBehavior {
    let source = AVec3::X.cross(AVec3::Z).dot(AVec3::Y);
    let transformed = transform
        .transform_vector3(AVec3::X)
        .cross(transform.transform_vector3(AVec3::Z))
        .dot(AVec3::Y);
    if source * transformed > 0.0 {
        OrientationBehavior::Preserving
    } else {
        OrientationBehavior::Reversing
    }
}

fn orientation_from_c(transform: CMat4) -> OrientationBehavior {
    let source = CVec3::X.cross(CVec3::Z).dot(CVec3::Y);
    let transformed = transform
        .transform_vector3(CVec3::X)
        .cross(transform.transform_vector3(CVec3::Z))
        .dot(CVec3::Y);
    if source * transformed > 0.0 {
        OrientationBehavior::Preserving
    } else {
        OrientationBehavior::Reversing
    }
}

fn orientation_from_b(transform: BMat4) -> OrientationBehavior {
    // Keep these directions explicit: AR-0026 pressure does not require Full B
    // to grow public X/Z constants merely to express chart semantics.
    let x = BVec3::new(1.0, 0.0, 0.0);
    let y = BVec3::new(0.0, 1.0, 0.0);
    let z = BVec3::new(0.0, 0.0, 1.0);
    let source = x.cross(z).dot(y);
    let transformed = transform
        .transform_vector3(x)
        .cross(transform.transform_vector3(z))
        .dot(y);
    if source * transformed > 0.0 {
        OrientationBehavior::Preserving
    } else {
        OrientationBehavior::Reversing
    }
}

/// Runs an authored three-chart control using current Alternative A mechanics.
#[must_use]
pub fn trace_with_a() -> ChartTrace {
    let entry_to_junction = AMat4::from_translation(AVec3::new(12.0, 0.0, -3.0))
        * AMat4::from_rotation_y(core::f32::consts::FRAC_PI_2);
    let junction_to_exit = AMat4::from_translation(AVec3::new(-4.0, 0.0, 9.0))
        * AMat4::from_rotation_y(-core::f32::consts::FRAC_PI_2);
    let composed = junction_to_exit * entry_to_junction;
    let entry_point = AVec3::new(2.0, 0.0, -1.0);
    let endpoint = composed.transform_point3(entry_point);
    let restored_point = composed.inverse().transform_point3(endpoint);
    let transported_direction = composed.transform_vector3(AVec3::Z).normalize();
    let reflected = AMat4::from_scale(AVec3::new(-1.0, 1.0, 1.0));
    ChartTrace {
        path: [ChartId::Entry, ChartId::Junction, ChartId::Exit],
        endpoint: endpoint.to_array(),
        restored_point: restored_point.to_array(),
        transported_direction: transported_direction.to_array(),
        composed_orientation: orientation_from_a(composed),
        reflected_orientation: orientation_from_a(reflected),
    }
}

/// Runs the identical authored control using the provider-backed Full B seam.
#[must_use]
pub fn trace_with_b() -> ChartTrace {
    let entry_to_junction = BMat4::from_translation(BVec3::new(12.0, 0.0, -3.0))
        * BMat4::from_rotation_y(core::f32::consts::FRAC_PI_2);
    let junction_to_exit = BMat4::from_translation(BVec3::new(-4.0, 0.0, 9.0))
        * BMat4::from_rotation_y(-core::f32::consts::FRAC_PI_2);
    let composed = junction_to_exit * entry_to_junction;
    let entry_point = BVec3::new(2.0, 0.0, -1.0);
    let endpoint = composed.transform_point3(entry_point);
    let restored_point = composed.inverse().transform_point3(endpoint);
    let transported_direction = composed
        .transform_vector3(BVec3::new(0.0, 0.0, 1.0))
        .normalize();
    let reflected = BMat4::from_scale(BVec3::new(-1.0, 1.0, 1.0));
    ChartTrace {
        path: [ChartId::Entry, ChartId::Junction, ChartId::Exit],
        endpoint: endpoint.to_array(),
        restored_point: restored_point.to_array(),
        transported_direction: transported_direction.to_array(),
        composed_orientation: orientation_from_b(composed),
        reflected_orientation: orientation_from_b(reflected),
    }
}

/// Runs the identical authored control using the owned C0/C1 candidate.
#[must_use]
pub fn trace_with_c() -> ChartTrace {
    let entry_to_junction = CMat4::from_translation(CVec3::new(12.0, 0.0, -3.0))
        * CMat4::from_rotation_y(core::f32::consts::FRAC_PI_2);
    let junction_to_exit = CMat4::from_translation(CVec3::new(-4.0, 0.0, 9.0))
        * CMat4::from_rotation_y(-core::f32::consts::FRAC_PI_2);
    let composed = junction_to_exit * entry_to_junction;
    let entry_point = CVec3::new(2.0, 0.0, -1.0);
    let endpoint = composed.transform_point3(entry_point);
    let restored_point = composed.inverse().transform_point3(endpoint);
    let transported_direction = composed.transform_vector3(CVec3::Z).normalize();
    let reflected = CMat4::from_scale(CVec3::new(-1.0, 1.0, 1.0));
    ChartTrace {
        path: [ChartId::Entry, ChartId::Junction, ChartId::Exit],
        endpoint: endpoint.to_array(),
        restored_point: restored_point.to_array(),
        transported_direction: transported_direction.to_array(),
        composed_orientation: orientation_from_c(composed),
        reflected_orientation: orientation_from_c(reflected),
    }
}

/// Compact cross-target observation of the fixed chart control.
///
/// This is deliberately a corpus fingerprint, not a serialization format or
/// chart protocol. It lets a browser/WASM host prove that it evaluated the
/// same A/B/C semantic control that native tests exercise.
#[must_use]
pub fn trace_fingerprint(trace: ChartTrace) -> u32 {
    let mut fingerprint = 0x26C0_0019_u32;
    for component in trace
        .endpoint
        .into_iter()
        .chain(trace.restored_point)
        .chain(trace.transported_direction)
    {
        fingerprint = fingerprint.rotate_left(5) ^ component.to_bits();
    }
    for behavior in [trace.composed_orientation, trace.reflected_orientation] {
        let value = match behavior {
            OrientationBehavior::Preserving => 1,
            OrientationBehavior::Reversing => 2,
        };
        fingerprint = fingerprint.rotate_left(3) ^ value;
    }
    fingerprint
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_near(left: [f32; 3], right: [f32; 3]) {
        for (left, right) in left.into_iter().zip(right) {
            assert!((left - right).abs() <= 1.0e-4, "{left} != {right}");
        }
    }

    #[test]
    fn identical_chart_meaning_has_equal_a_b_and_c_traces() {
        let baseline = trace_with_a();
        let provider_backed = trace_with_b();
        let owned = trace_with_c();
        for candidate in [provider_backed, owned] {
            assert_eq!(baseline.path, candidate.path);
            assert_near(baseline.endpoint, candidate.endpoint);
            assert_near(baseline.restored_point, candidate.restored_point);
            assert_near(
                baseline.transported_direction,
                candidate.transported_direction,
            );
            assert_eq!(
                baseline.composed_orientation,
                candidate.composed_orientation
            );
            assert_eq!(
                baseline.reflected_orientation,
                candidate.reflected_orientation
            );
        }
    }

    #[test]
    fn invertibility_and_orientation_are_independent_facts() {
        for trace in [trace_with_a(), trace_with_b(), trace_with_c()] {
            assert_near(trace.restored_point, [2.0, 0.0, -1.0]);
            assert_eq!(trace.composed_orientation, OrientationBehavior::Preserving);
            assert_eq!(trace.reflected_orientation, OrientationBehavior::Reversing);
        }
    }

    #[test]
    fn a_b_and_c_chart_fingerprints_match() {
        let baseline = trace_fingerprint(trace_with_a());
        assert_eq!(baseline, trace_fingerprint(trace_with_b()));
        assert_eq!(baseline, trace_fingerprint(trace_with_c()));
    }
}
