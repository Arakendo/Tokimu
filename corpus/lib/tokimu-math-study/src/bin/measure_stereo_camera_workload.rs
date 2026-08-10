//! Repeated-sample host observation for complete A/B/C stereo-camera creation.
//!
//! This is a descriptive corpus tool, not a portable benchmark. It rotates
//! sample order and verifies the same retained checksum before reporting
//! timings; host, target, profile, and toolchain belong in its result record.

use std::time::Instant;

use tokimu_math_study::workloads::{
    baseline_stereo_camera_workload, owned_stereo_camera_workload,
    provider_backed_stereo_camera_workload,
};

const DEFAULT_ITERATIONS: u32 = 100_000;
const DEFAULT_SAMPLES: usize = 15;

#[derive(Clone, Copy)]
enum Candidate {
    Baseline,
    ProviderBacked,
    Owned,
}

impl Candidate {
    fn run(self, iterations: u32) -> f32 {
        match self {
            Self::Baseline => baseline_stereo_camera_workload(iterations),
            Self::ProviderBacked => provider_backed_stereo_camera_workload(iterations),
            Self::Owned => owned_stereo_camera_workload(iterations),
        }
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
        "usage: measure_stereo_camera_workload [iterations] [samples]"
    );

    let candidates = [
        Candidate::Baseline,
        Candidate::ProviderBacked,
        Candidate::Owned,
    ];
    let expected = candidates[0].run(iterations);
    for candidate in candidates {
        assert!((candidate.run(iterations) - expected).abs() <= 1.0e-3);
    }

    let mut baseline = Vec::with_capacity(samples);
    let mut provider_backed = Vec::with_capacity(samples);
    let mut owned = Vec::with_capacity(samples);
    for sample in 0..samples {
        for offset in 0..candidates.len() {
            let candidate = candidates[(sample + offset) % candidates.len()];
            let started = Instant::now();
            let checksum = candidate.run(iterations);
            let elapsed = started.elapsed().as_nanos();
            assert!((checksum - expected).abs() <= 1.0e-3);
            match candidate {
                Candidate::Baseline => baseline.push(elapsed),
                Candidate::ProviderBacked => provider_backed.push(elapsed),
                Candidate::Owned => owned.push(elapsed),
            }
        }
    }

    let baseline = summary(&mut baseline);
    let provider_backed = summary(&mut provider_backed);
    let owned = summary(&mut owned);
    println!("iterations={iterations}");
    println!("samples={samples}");
    println!(
        "baseline_elapsed_ns=min:{},median:{},max:{}",
        baseline.0, baseline.1, baseline.2
    );
    println!(
        "provider_backed_elapsed_ns=min:{},median:{},max:{}",
        provider_backed.0, provider_backed.1, provider_backed.2
    );
    println!(
        "owned_elapsed_ns=min:{},median:{},max:{}",
        owned.0, owned.1, owned.2
    );
    println!("checksum={expected}");
}
