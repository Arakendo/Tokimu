//! Manual repeated-sample measurement entry point for the shared A/B/C
//! transform workload.
//!
//! This is deliberately a small, dependency-free harness rather than a
//! benchmark framework. It warms every candidate, rotates their measurement
//! order, and reports the retained samples so a result does not accidentally
//! compare three different positions in one process. Host and target metadata
//! remain the responsibility of the retained result artifact.

use std::time::Instant;

use tokimu_math_study::workloads::{
    baseline_transform_workload, owned_transform_workload, provider_backed_transform_workload,
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
    fn run(self, iterations: u32) -> [f32; 3] {
        match self {
            Self::Baseline => baseline_transform_workload(iterations),
            Self::ProviderBacked => provider_backed_transform_workload(iterations),
            Self::Owned => owned_transform_workload(iterations),
        }
    }
}

fn parse_argument<T: std::str::FromStr>(argument: Option<String>, default: T, label: &str) -> T {
    argument
        .map(|value| {
            value
                .parse::<T>()
                .unwrap_or_else(|_| panic!("{label} must be an unsigned integer"))
        })
        .unwrap_or(default)
}

fn measure(candidate: Candidate, iterations: u32) -> (u128, [f32; 3]) {
    let started = Instant::now();
    let checksum = candidate.run(iterations);
    (started.elapsed().as_nanos(), checksum)
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
    let iterations = parse_argument(arguments.next(), DEFAULT_ITERATIONS, "iteration count");
    let samples = parse_argument(arguments.next(), DEFAULT_SAMPLES, "sample count");
    assert!(samples > 0, "sample count must be greater than zero");
    assert!(
        arguments.next().is_none(),
        "usage: measure_transform_workload [iterations] [samples]"
    );

    let candidates = [
        Candidate::Baseline,
        Candidate::ProviderBacked,
        Candidate::Owned,
    ];
    let expected_checksum = candidates[0].run(iterations);
    for candidate in candidates {
        assert_eq!(candidate.run(iterations), expected_checksum);
    }

    let mut baseline_samples = Vec::with_capacity(samples);
    let mut provider_backed_samples = Vec::with_capacity(samples);
    let mut owned_samples = Vec::with_capacity(samples);

    for sample in 0..samples {
        for offset in 0..candidates.len() {
            let candidate = candidates[(sample + offset) % candidates.len()];
            let (elapsed, checksum) = measure(candidate, iterations);
            assert_eq!(checksum, expected_checksum);

            match candidate {
                Candidate::Baseline => baseline_samples.push(elapsed),
                Candidate::ProviderBacked => provider_backed_samples.push(elapsed),
                Candidate::Owned => owned_samples.push(elapsed),
            }
        }
    }

    let (baseline_min, baseline_median, baseline_max) = summary(&mut baseline_samples);
    let (provider_backed_min, provider_backed_median, provider_backed_max) =
        summary(&mut provider_backed_samples);
    let (owned_min, owned_median, owned_max) = summary(&mut owned_samples);

    println!("iterations={iterations}");
    println!("samples={samples}");
    println!("baseline_elapsed_ns=min:{baseline_min},median:{baseline_median},max:{baseline_max}");
    println!("provider_backed_elapsed_ns=min:{provider_backed_min},median:{provider_backed_median},max:{provider_backed_max}");
    println!("owned_elapsed_ns=min:{owned_min},median:{owned_median},max:{owned_max}");
    println!("checksum={expected_checksum:?}");
}
