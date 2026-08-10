//! Manual comparison of the B and C provider-upload conversion boundaries.

use std::time::Instant;

use tokimu_math_study::{migration_b, migration_c};

fn main() {
    let iterations = std::env::args()
        .nth(1)
        .map(|argument| argument.parse::<u32>())
        .transpose()
        .expect("iteration count must be an unsigned integer")
        .unwrap_or(1_000_000);

    let b_started = Instant::now();
    let b_checksum = migration_b::provider_upload_workload(iterations);
    let b_elapsed = b_started.elapsed();

    let c_started = Instant::now();
    let c_checksum = migration_c::provider_upload_workload(iterations);
    let c_elapsed = c_started.elapsed();

    assert_eq!(b_checksum, c_checksum);
    println!("iterations={iterations}");
    println!("provider_backed_upload_elapsed_ns={}", b_elapsed.as_nanos());
    println!("owned_upload_elapsed_ns={}", c_elapsed.as_nanos());
    println!("checksum={b_checksum}");
}
