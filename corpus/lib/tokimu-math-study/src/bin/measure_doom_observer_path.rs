//! Repeated-sample E1M1 source-observer camera preparation control.

use std::time::Instant;

use tokimu_math_study::migration_hello_doom_observer::{
    observer_camera_with_a, observer_camera_with_b, observer_camera_with_c,
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
    fn run(self, iterations: u32) -> f64 {
        let mut checksum = 0.0_f64;
        for frame in 0..iterations {
            let yaw = frame as f32 * 0.000_73;
            let pitch = ((frame % 97) as f32 - 48.0) * 0.001;
            let camera = match self {
                Self::Baseline => observer_camera_with_a([1056.0, -3616.0], yaw, pitch),
                Self::ProviderBacked => observer_camera_with_b([1056.0, -3616.0], yaw, pitch),
                Self::Owned => observer_camera_with_c([1056.0, -3616.0], yaw, pitch),
            };
            checksum += f64::from(core::hint::black_box(
                camera.position[0] + camera.forward[2] + camera.view_projection_columns[10],
            ));
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

fn main() {
    let mut arguments = std::env::args().skip(1);
    let iterations = parse(arguments.next(), DEFAULT_ITERATIONS, "iteration count");
    let samples = parse(arguments.next(), DEFAULT_SAMPLES, "sample count");
    assert!(samples > 0, "sample count must be greater than zero");
    assert!(
        arguments.next().is_none(),
        "usage: measure_doom_observer_path [iterations] [samples]"
    );
    let candidates = [
        Candidate::Baseline,
        Candidate::ProviderBacked,
        Candidate::Owned,
    ];
    let expected = candidates.map(|candidate| candidate.run(iterations));
    let mut elapsed = [
        Vec::with_capacity(samples),
        Vec::with_capacity(samples),
        Vec::with_capacity(samples),
    ];
    for sample in 0..samples {
        for offset in 0..candidates.len() {
            let index = (sample + offset) % candidates.len();
            let started = Instant::now();
            let checksum = candidates[index].run(iterations);
            assert_eq!(checksum, expected[index]);
            elapsed[index].push(started.elapsed().as_nanos());
        }
    }
    println!("iterations={iterations}");
    println!("samples={samples}");
    for (label, values) in ["baseline", "provider_backed", "owned"]
        .into_iter()
        .zip(elapsed.iter_mut())
    {
        values.sort_unstable();
        println!(
            "{label}_doom_observer_elapsed_ns=min:{},median:{},max:{}",
            values[0],
            values[values.len() / 2],
            values[values.len() - 1]
        );
    }
    println!("checksums={expected:?}");
}
