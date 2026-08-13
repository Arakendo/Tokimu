//! Corpus-only cost isolation for the Narrow-B checked construction seam.

use std::time::Instant;
use tokimu_math_study_narrow_b::{
    projection_orthographic_rh_gl, projection_perspective_rh_gl, view_look_at_rh, Mat4, Vec3,
};

#[cfg(feature = "provider-029")]
fn direct_view(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
    Mat4::look_at_rh(eye, target, up)
}

#[cfg(feature = "provider-033")]
fn direct_view(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
    glam_033::camera::rh::view::look_at_mat4(eye, target, up)
}

#[cfg(feature = "provider-029")]
fn direct_perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    Mat4::perspective_rh_gl(fov, aspect, near, far)
}

#[cfg(feature = "provider-033")]
fn direct_perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    glam_033::camera::rh::proj::opengl::perspective(fov, aspect, near, far)
}

#[cfg(feature = "provider-029")]
fn direct_orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4 {
    Mat4::orthographic_rh_gl(left, right, bottom, top, near, far)
}

#[cfg(feature = "provider-033")]
fn direct_orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4 {
    glam_033::camera::rh::proj::opengl::orthographic(left, right, bottom, top, near, far)
}

fn run_direct(iterations: u32) -> f64 {
    let mut checksum = 0.0_f64;
    for frame in 0..iterations {
        let phase = frame as f32 * 0.000_031;
        let eye = Vec3::new(phase.sin() * 3.0, 2.0, 6.0 + phase.cos());
        let view = direct_view(eye, Vec3::ZERO, Vec3::Y);
        let perspective = direct_perspective(core::f32::consts::FRAC_PI_3, 16.0 / 9.0, 0.1, 100.0);
        let orthographic = direct_orthographic(-8.0, 8.0, -4.5, 4.5, -10.0, 10.0);
        checksum += f64::from(core::hint::black_box(
            view.to_cols_array()[0]
                + perspective.to_cols_array()[5]
                + orthographic.to_cols_array()[10],
        ));
    }
    checksum
}

fn run_checked(iterations: u32) -> f64 {
    let mut checksum = 0.0_f64;
    for frame in 0..iterations {
        let phase = frame as f32 * 0.000_031;
        let eye = Vec3::new(phase.sin() * 3.0, 2.0, 6.0 + phase.cos());
        let view = view_look_at_rh(eye, Vec3::ZERO, Vec3::Y).expect("valid view");
        let perspective =
            projection_perspective_rh_gl(core::f32::consts::FRAC_PI_3, 16.0 / 9.0, 0.1, 100.0)
                .expect("valid perspective");
        let orthographic = projection_orthographic_rh_gl(-8.0, 8.0, -4.5, 4.5, -10.0, 10.0)
            .expect("valid orthographic");
        checksum += f64::from(core::hint::black_box(
            view.to_cols_array()[0]
                + perspective.to_cols_array()[5]
                + orthographic.to_cols_array()[10],
        ));
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

    let expected_direct = run_direct(iterations);
    let expected_checked = run_checked(iterations);
    assert_eq!(expected_direct, expected_checked);
    let mut direct = Vec::with_capacity(sample_count);
    let mut checked = Vec::with_capacity(sample_count);
    for sample in 0..sample_count {
        if sample % 2 == 0 {
            let started = Instant::now();
            assert_eq!(run_direct(iterations), expected_direct);
            direct.push(started.elapsed().as_nanos());
            let started = Instant::now();
            assert_eq!(run_checked(iterations), expected_checked);
            checked.push(started.elapsed().as_nanos());
        } else {
            let started = Instant::now();
            assert_eq!(run_checked(iterations), expected_checked);
            checked.push(started.elapsed().as_nanos());
            let started = Instant::now();
            assert_eq!(run_direct(iterations), expected_direct);
            direct.push(started.elapsed().as_nanos());
        }
    }
    let direct = summary(&mut direct);
    let checked = summary(&mut checked);
    println!("iterations={iterations},samples={sample_count}");
    println!(
        "direct_elapsed_ns=min:{},median:{},max:{}",
        direct.0, direct.1, direct.2
    );
    println!(
        "checked_elapsed_ns=min:{},median:{},max:{}",
        checked.0, checked.1, checked.2
    );
    println!("checksum={expected_direct}");
}
