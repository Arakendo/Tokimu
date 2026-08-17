use hello_doom_visibility_conformance::lower_runtime_snapshots_to_presentation;

fn main() -> Result<(), String> {
    let manifest = lower_runtime_snapshots_to_presentation()?;

    for state in &manifest.door_states {
        println!(
            "door phase={:?}; source={}; floor={}; ceiling={}; occurrence={:?}; resource={:?}; boundaries={}; vertical_range={:?}; lifecycle={:?}",
            state.phase,
            state.source_correlation,
            state.runtime_floor_height,
            state.runtime_ceiling_height,
            state.occurrence_correlation,
            state.renderer_resource_correlation,
            state.prepared_boundaries,
            state.vertical_range,
            state.lifecycle_action,
        );
    }

    for state in &manifest.platform_states {
        println!(
            "platform phase={:?}; source={}; floor={}; ceiling={}; occurrence={:?}; resource={:?}; boundaries={}; vertical_range={:?}; lifecycle={:?}",
            state.phase,
            state.source_correlation,
            state.runtime_floor_height,
            state.runtime_ceiling_height,
            state.occurrence_correlation,
            state.renderer_resource_correlation,
            state.prepared_boundaries,
            state.vertical_range,
            state.lifecycle_action,
        );
    }

    println!(
        "summary door_identity_stable={}; platform_identity_stable={}; current_heights_drive_preparation={}; creates={}; replacements={}; retirements={}; unrelated_resource_reallocations={}; application_movement_policy_present={}; fingerprint={}",
        manifest.door_source_identity_stable,
        manifest.platform_source_identity_stable,
        manifest.current_heights_drive_preparation,
        manifest.affected_creates,
        manifest.affected_replacements,
        manifest.affected_retirements,
        manifest.unrelated_resource_reallocations,
        manifest.application_movement_policy_present,
        manifest.structural_fingerprint,
    );

    Ok(())
}
