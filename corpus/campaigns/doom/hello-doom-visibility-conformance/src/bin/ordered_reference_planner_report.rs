use hello_doom_visibility_conformance::observe_ordered_reference_planner;

fn main() -> Result<(), String> {
    let manifest = observe_ordered_reference_planner()?;
    println!(
        "ordered-reference planner: cases={}; balanced={}; solid-pass={}; vertical={}; walls={}; planes={}; sky={}; masked={}; fail-open={}; movement-policy={}; fingerprint={}",
        manifest.evaluated_cases,
        manifest.balanced_cases,
        manifest.has_solid_and_pass_ranges,
        manifest.has_vertical_clip_mutations,
        manifest.has_wall_tiers,
        manifest.has_plane_instances,
        manifest.has_sky_intervals,
        manifest.has_deferred_masked_work,
        manifest.fail_open_retained,
        manifest.application_movement_policy_present,
        manifest.structural_fingerprint,
    );
    for case in &manifest.cases {
        println!(
            "case={}; snapshot={}; order={:?}; solid={}; pass={}; covered={}; transitions={}; wall-cells={}/{}; plane-marks={}; plane-instances={}; sky-instances={}; paired-sky={}; masked={}; fail-open={}; balanced={}; fingerprint={}",
            case.case,
            case.runtime_snapshot,
            case.admitted_seg_order,
            case.solid_admitted,
            case.pass_admitted,
            case.covered_columns,
            case.coverage_transitions,
            case.retained_wall_tier_cells,
            case.wall_tier_cells,
            case.plane_marks,
            case.plane_instances,
            case.sky_plane_instances,
            case.paired_sky_intervals,
            case.deferred_masked_work,
            case.fail_open,
            case.balanced,
            case.structural_fingerprint,
        );
    }
    if manifest.balanced_cases != manifest.evaluated_cases {
        return Err("one or more reference-planner controls did not balance".to_owned());
    }
    Ok(())
}
