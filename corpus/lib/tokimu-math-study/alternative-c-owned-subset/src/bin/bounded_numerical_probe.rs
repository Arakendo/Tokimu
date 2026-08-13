//! Bounded, dependency-free generated pressure for the owned math candidate.
//!
//! This executable is corpus evidence, not a stable diagnostic or fuzzing API.
//! Arguments are optional `<seed> <case-count>` values; the case count is
//! deliberately clamped so untrusted invocation cannot turn it into an
//! unbounded workload.

use tokimu_math_study_owned_subset::{Mat4, Vec3};

const DEFAULT_SEED: u32 = 0xC0DE_C0DE;
const DEFAULT_CASES: u32 = 256;
const MAX_CASES: u32 = 4_096;

fn main() {
    let mut arguments = std::env::args().skip(1);
    let seed = arguments
        .next()
        .map(|value| value.parse::<u32>().expect("seed must be an unsigned integer"))
        .unwrap_or(DEFAULT_SEED);
    let cases = arguments
        .next()
        .map(|value| value.parse::<u32>().expect("case count must be an unsigned integer"))
        .unwrap_or(DEFAULT_CASES)
        .min(MAX_CASES);
    assert!(arguments.next().is_none(), "expected at most a seed and case count");

    let checksum = run(seed, cases);
    println!("option-c bounded numerical probe: seed={seed} cases={cases} checksum={checksum:.6}");
}

fn run(mut seed: u32, cases: u32) -> f32 {
    let mut checksum = 0.0;
    for _ in 0..cases {
        let vector = Vec3::new(
            next_range(&mut seed, -1_000.0, 1_000.0),
            next_range(&mut seed, -1_000.0, 1_000.0),
            next_range(&mut seed, -1_000.0, 1_000.0),
        );
        let direction = vector.try_normalize().expect("bounded vector is nonzero");
        let translation = Vec3::new(
            next_range(&mut seed, -100.0, 100.0),
            next_range(&mut seed, -100.0, 100.0),
            next_range(&mut seed, -100.0, 100.0),
        );
        let transform = Mat4::from_translation(translation)
            * Mat4::from_rotation_y(next_range(&mut seed, -3.0, 3.0))
            * Mat4::from_scale(Vec3::new(
                next_range(&mut seed, 0.25, 3.25),
                next_range(&mut seed, 0.25, 3.25),
                next_range(&mut seed, 0.25, 3.25),
            ));
        let inverse = transform
            .try_inverse()
            .expect("bounded affine transform is conditioned");
        let restored = inverse.transform_point3(transform.transform_point3(vector));
        assert!(near(restored.x, vector.x));
        assert!(near(restored.y, vector.y));
        assert!(near(restored.z, vector.z));
        checksum += direction.x + restored.y * 0.01 + restored.z * 0.001;
    }
    checksum
}

fn next_range(seed: &mut u32, minimum: f32, maximum: f32) -> f32 {
    *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    minimum + (maximum - minimum) * ((*seed >> 8) as f32 / ((u32::MAX >> 8) as f32))
}

fn near(actual: f32, expected: f32) -> bool {
    let tolerance = 1.0e-3_f32.max(1.0e-5 * actual.abs().max(expected.abs()));
    actual.is_finite() && expected.is_finite() && (actual - expected).abs() <= tolerance
}
