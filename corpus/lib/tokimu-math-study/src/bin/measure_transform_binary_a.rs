//! Candidate-isolated release link target for Alternative A size observation.

use std::hint::black_box;

use tokimu_math_study::workloads::baseline_transform_workload;

fn main() {
    let iterations = std::env::args()
        .nth(1)
        .map(|argument| argument.parse::<u32>())
        .transpose()
        .expect("iteration count must be an unsigned integer")
        .unwrap_or(1_000_000);

    black_box(baseline_transform_workload(iterations));
}
