//! Operator-isolation control for the Slice 5 caller-path regression.
//!
//! This is not an engine benchmark. It exists only because the retained CAD
//! and GLB caller ports both regress at their repeated `Mat4::inverse()`
//! boundary. The input is one finite, well-conditioned affine matrix; each
//! candidate repeats the same inversion and retains its own deterministic
//! checksum.

use std::time::Instant;

use tokimu_core::math::{Mat4 as AMat4, Vec3 as AVec3};
use tokimu_math_study::{
    alternative_b::{Mat4 as BMat4, Vec3 as BVec3},
    alternative_c::{Mat4 as CMat4, Vec3 as CVec3},
};

const DEFAULT_ITERATIONS: u32 = 1_000_000;
const DEFAULT_SAMPLES: usize = 15;

#[derive(Clone, Copy)]
enum Candidate {
    Baseline,
    ProviderBacked,
    Owned,
}

impl Candidate {
    fn inverse_checksum(self, iterations: u32) -> f64 {
        let mut checksum = 0.0_f64;
        match self {
            Self::Baseline => {
                let matrix = AMat4::from_translation(AVec3::new(4.0, -3.0, 9.0))
                    * AMat4::from_rotation_y(0.73)
                    * AMat4::from_scale(AVec3::new(1.25, 0.75, 2.0));
                for _ in 0..iterations {
                    let inverse = core::hint::black_box(matrix.inverse());
                    checksum += f64::from(inverse.w_axis.x + inverse.z_axis.y);
                }
            }
            Self::ProviderBacked => {
                let matrix = BMat4::from_translation(BVec3::new(4.0, -3.0, 9.0))
                    * BMat4::from_rotation_y(0.73)
                    * BMat4::from_scale(BVec3::new(1.25, 0.75, 2.0));
                for _ in 0..iterations {
                    let inverse = core::hint::black_box(matrix.inverse());
                    let columns = inverse.to_cols_array();
                    checksum += f64::from(columns[12] + columns[9]);
                }
            }
            Self::Owned => {
                let matrix = CMat4::from_translation(CVec3::new(4.0, -3.0, 9.0))
                    * CMat4::from_rotation_y(0.73)
                    * CMat4::from_scale(CVec3::new(1.25, 0.75, 2.0));
                for _ in 0..iterations {
                    let inverse = core::hint::black_box(matrix.inverse());
                    let columns = inverse.to_cols_array();
                    checksum += f64::from(columns[12] + columns[9]);
                }
            }
        }
        checksum
    }
}

fn parse<T: std::str::FromStr>(value: Option<String>, default: T, label: &str) -> T {
    value
        .map(|value| {
            value
                .parse()
                .unwrap_or_else(|_| panic!("{label} must be an integer"))
        })
        .unwrap_or(default)
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
    let mut arguments = std::env::args().skip(1);
    let iterations = parse(arguments.next(), DEFAULT_ITERATIONS, "iteration count");
    let samples = parse(arguments.next(), DEFAULT_SAMPLES, "sample count");
    assert!(samples > 0, "sample count must be greater than zero");
    assert!(
        arguments.next().is_none(),
        "usage: measure_inverse_workload [iterations] [samples]"
    );

    let candidates = [
        Candidate::Baseline,
        Candidate::ProviderBacked,
        Candidate::Owned,
    ];
    let expected = candidates.map(|candidate| candidate.inverse_checksum(iterations));
    let mut elapsed = [
        Vec::with_capacity(samples),
        Vec::with_capacity(samples),
        Vec::with_capacity(samples),
    ];

    for sample in 0..samples {
        for offset in 0..candidates.len() {
            let index = (sample + offset) % candidates.len();
            let started = Instant::now();
            let checksum = candidates[index].inverse_checksum(iterations);
            assert_eq!(checksum, expected[index]);
            elapsed[index].push(started.elapsed().as_nanos());
        }
    }

    println!("iterations={iterations}");
    println!("samples={samples}");
    for (label, samples) in ["baseline", "provider_backed", "owned"]
        .into_iter()
        .zip(elapsed.iter_mut())
    {
        let (minimum, median, maximum) = summary(samples);
        println!("{label}_inverse_elapsed_ns=min:{minimum},median:{median},max:{maximum}");
    }
    println!("checksums={expected:?}");
}
