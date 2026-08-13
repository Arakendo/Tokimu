//! Native observation runner for the AR-0026/AR-0019 chart control.

use tokimu_math_study::chart_junction::{
    trace_fingerprint, trace_with_a, trace_with_b, trace_with_c,
};

fn main() {
    let baseline = trace_with_a();
    let provider_backed = trace_with_b();
    let owned = trace_with_c();
    println!(
        "status=completed; workload=ar-0026-chart-control; alternative=B; endpoint={:?}; restored={:?}; direction={:?}; composed_orientation={:?}; reflected_orientation={:?}; fingerprint={:08x}",
        provider_backed.endpoint,
        provider_backed.restored_point,
        provider_backed.transported_direction,
        provider_backed.composed_orientation,
        provider_backed.reflected_orientation,
        trace_fingerprint(provider_backed),
    );
    println!(
        "status=completed; workload=ar-0026-chart-control; alternative=A; endpoint={:?}; restored={:?}; direction={:?}; composed_orientation={:?}; reflected_orientation={:?}; fingerprint={:08x}",
        baseline.endpoint,
        baseline.restored_point,
        baseline.transported_direction,
        baseline.composed_orientation,
        baseline.reflected_orientation,
        trace_fingerprint(baseline),
    );
    println!(
        "status=completed; workload=ar-0026-chart-control; alternative=C0; endpoint={:?}; restored={:?}; direction={:?}; composed_orientation={:?}; reflected_orientation={:?}; fingerprint={:08x}",
        owned.endpoint,
        owned.restored_point,
        owned.transported_direction,
        owned.composed_orientation,
        owned.reflected_orientation,
        trace_fingerprint(owned),
    );
}
