use hello_render_resource_identity::{
    observe_caller_staged_recovery, observe_churn, observe_failure_boundary_fixture,
    observe_fixed_disjoint_ranges, representation_evidence, reproduce_mutable_offset_alias,
};

fn main() {
    let failure = reproduce_mutable_offset_alias();
    let fixed = observe_fixed_disjoint_ranges();
    println!(
        "AR-0024/0027 mutable-offset baseline: cutout={} dynamic={} recomputed-cutout={} dynamic-upload={:?} original-resolves={:?} recomputed-resolves={:?}",
        failure.original_cutout_handle.0,
        failure.dynamic_handle.0,
        failure.recomputed_cutout_handle.0,
        failure.dynamic_upload,
        failure.original_cutout_now_resolves_to,
        failure.recomputed_cutout_resolves_to,
    );
    println!(
        "AR-0024/0027 fixed-range baseline: cutout={} dynamic={} cutout-resolves={:?} dynamic-resolves={:?}",
        fixed.cutout_handle.0,
        fixed.dynamic_handle.0,
        fixed.cutout_resolves_to,
        fixed.dynamic_resolves_to,
    );
    println!(
        "AR-0024/0027 representation baseline: {:?}",
        representation_evidence()
    );
    println!(
        "AR-0024/0027 native churn observation: {:?}",
        observe_churn(10_000)
    );
    let failure_boundary = observe_failure_boundary_fixture();
    println!(
        "AR-0024/0027 failure-boundary fixture: total={}; retained={:?}",
        failure_boundary.total_recorded(),
        failure_boundary.retained().collect::<Vec<_>>(),
    );
    println!(
        "AR-0024/0027 caller-staged recovery: {:?}",
        observe_caller_staged_recovery()
    );
}
