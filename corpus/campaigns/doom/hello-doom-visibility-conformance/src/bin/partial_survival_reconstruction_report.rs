use hello_doom_visibility_conformance::observe_partial_survival_reconstruction;

fn main() -> Result<(), String> {
    let observation = observe_partial_survival_reconstruction()?;
    for pose in &observation.poses {
        println!(
            "pose={}; viewer={:?}; source={}; occurrences={:?}; retained={:?}; excluded={:?}; survivor-columns={}/{}; forbidden-columns={}; endpoint-checks={}; endpoints-on-source={}; uv-continuous={}",
            pose.label,
            pose.viewer_position,
            pose.source_identity,
            pose.occurrence_identities,
            pose.retained_intervals,
            pose.excluded_interval,
            pose.represented_survivor_columns,
            pose.required_survivor_columns,
            pose.forbidden_columns,
            pose.endpoint_checks,
            pose.endpoints_on_source_geometry,
            pose.uv_parameterization_continuous,
        );
    }
    println!(
        "status=validated; evaluated={}; partial-pose-replays={}; distinct-replayed-sources={}; whole-retained={}; fragmented={}; whole-rejected={}; failed-open={}; near-plane-fail-open={}; unsupported-role-fail-open={}; empty-positive-reject={}; thin-retained={}; stable-source-under-jitter={}; stable-occurrences-under-jitter={}; screen-column-inverse-projection={}; fingerprint={}",
        observation.evaluated_contributions,
        observation.partial_pose_replays,
        observation.distinct_replayed_source_identities,
        observation.whole_retained,
        observation.fragmented,
        observation.whole_rejected,
        observation.failed_open,
        observation.near_plane_failed_open,
        observation.unsupported_role_failed_open,
        observation.empty_fragment_rejected_with_authority,
        observation.thin_projection_retained,
        observation.stable_source_identity_under_jitter,
        observation.stable_occurrence_identity_under_jitter,
        !observation.no_screen_column_inverse_projection,
        observation.fingerprint,
    );
    Ok(())
}
