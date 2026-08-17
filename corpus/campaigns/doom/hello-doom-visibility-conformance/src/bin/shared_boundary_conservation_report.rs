use hello_doom_visibility_conformance::observe_shared_boundary_conservation;

fn main() -> Result<(), String> {
    let observation = observe_shared_boundary_conservation()?;
    println!(
        "shared-boundary conservation: cases={}/{}; sky-authorized={}; seams-balanced={}; fingerprint={}",
        observation.balanced_cases,
        observation.evaluated_cases,
        observation.sky_paints_source_authorized_intervals,
        observation.no_cracks_or_double_authority,
        observation.fingerprint
    );
    for case in &observation.cases {
        println!(
            "fixture={}; admitted-segs={}; transitions={}; wall-cells={}/{}; planes=floor:{},ceiling:{},sky:{}; paired-sky-events={}; fail-open={}({:?}); checks=chain:{},wall-domain:{},plane-boundary:{},plane-source:{},paired-sky-non-mutating:{},no-overlap:{}",
            case.fixture,
            case.admitted_source_segs,
            case.ordered_transitions,
            case.retained_wall_cells,
            case.retained_wall_cells + case.omitted_wall_cells,
            case.floor_plane_instances,
            case.ceiling_plane_instances,
            case.sky_plane_instances,
            case.paired_sky_events,
            case.unresolved_fail_open,
            case.fail_open_reasons,
            case.transition_chain_contiguous,
            case.wall_intervals_inside_shared_opening,
            case.plane_intervals_match_shared_boundary,
            case.plane_sources_were_admitted,
            case.paired_sky_events_are_non_mutating,
            case.no_plane_overlap_writes,
        );
    }
    println!(
        "cutout: admitted={}; retained-middle-cells={}; closed-source-coverage={}; fail-open={}({:?}); bounded-ray-depth-only={}",
        observation.cutout_source_admitted,
        observation.cutout_retained_wall_cells,
        observation.cutout_closed_source_coverage,
        observation.cutout_unresolved_fail_open,
        observation.cutout_fail_open_reasons,
        observation.cutout_fail_open_is_only_bounded_ray_depth,
    );
    if observation.balanced_cases != observation.evaluated_cases
        || !observation.sky_paints_source_authorized_intervals
        || !observation.no_cracks_or_double_authority
    {
        return Err("shared wall/plane boundary conservation failed".to_owned());
    }
    Ok(())
}
