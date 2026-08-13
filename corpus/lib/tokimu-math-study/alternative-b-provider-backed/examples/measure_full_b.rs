//! Corpus-only repeated-cost observations for Full B under either private pin.

use std::time::Instant;
use tokimu_math_study_provider_backed::{tokimu_math_study_wasm_stereo_camera_probe, Mat4, Vec3};

fn transform(iterations: u32) -> f64 {
    let mut checksum = 0.0_f64;
    for frame in 0..iterations {
        let phase = frame as f32 * 0.000_031;
        let matrix = Mat4::from_translation(Vec3::new(phase.sin(), 2.0, phase.cos()))
            * Mat4::from_rotation_y(phase)
            * Mat4::from_scale(Vec3::new(1.25, 0.75, 2.0));
        let point = matrix.transform_point3(Vec3::new(3.0, -1.0, 2.0));
        checksum += f64::from(core::hint::black_box(point.x() + point.y() + point.z()));
    }
    checksum
}

fn inverse(iterations: u32) -> f64 {
    let matrix = Mat4::from_translation(Vec3::new(4.0, -3.0, 9.0))
        * Mat4::from_rotation_y(0.73)
        * Mat4::from_scale(Vec3::new(1.25, 0.75, 2.0));
    let mut checksum = 0.0_f64;
    for _ in 0..iterations {
        let columns = core::hint::black_box(matrix.inverse()).to_cols_array();
        checksum += f64::from(columns[12] + columns[9]);
    }
    checksum
}

fn summary(samples: &mut [u128]) -> (u128, u128, u128) {
    samples.sort_unstable();
    (
        samples[0],
        samples[samples.len() / 2],
        samples[samples.len() - 1],
    )
}

fn measure(iterations: u32, sample_count: usize, run: impl Fn(u32) -> f64) -> (u128, u128, u128) {
    let expected = run(iterations);
    let mut samples = Vec::with_capacity(sample_count);
    for _ in 0..sample_count {
        let started = Instant::now();
        assert_eq!(run(iterations), expected);
        samples.push(started.elapsed().as_nanos());
    }
    summary(&mut samples)
}

fn main() {
    let iterations = std::env::args()
        .nth(1)
        .map(|value| value.parse::<u32>())
        .transpose()
        .expect("iterations must be an unsigned integer")
        .unwrap_or(1_000_000);
    let sample_count = std::env::args()
        .nth(2)
        .map(|value| value.parse::<usize>())
        .transpose()
        .expect("samples must be an unsigned integer")
        .unwrap_or(15);
    assert!(sample_count > 0, "samples must be positive");

    let transform = measure(iterations, sample_count, transform);
    let inverse = measure(iterations, sample_count, inverse);
    let stereo = measure(iterations, sample_count, |iterations| {
        f64::from(tokimu_math_study_wasm_stereo_camera_probe(iterations))
    });
    println!("iterations={iterations},samples={sample_count}");
    println!(
        "transform_elapsed_ns=min:{},median:{},max:{}",
        transform.0, transform.1, transform.2
    );
    println!(
        "inverse_elapsed_ns=min:{},median:{},max:{}",
        inverse.0, inverse.1, inverse.2
    );
    println!(
        "stereo_elapsed_ns=min:{},median:{},max:{}",
        stereo.0, stereo.1, stereo.2
    );
}
