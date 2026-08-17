use hello_doom_visibility_conformance::observe_candidate1_synthetic_matrix;

fn main() -> Result<(), String> {
    let manifest = observe_candidate1_synthetic_matrix()?;
    println!(
        "candidate1 synthetic matrix: ordered={}/{}; controls={}; sky={}/{} intervals,{}/{} cells; declarations={}; local={}/{}/{} triangles; negative-authority={}:declarations={}; removed-non-sky={}; persistent-mesh-identities={}; runtime-snapshots={}; cutout-deferred={}; continuous={}; no-generic-filter={}; unexplained={}; semantic-comparison-only={}; fingerprint={}",
        manifest.balanced_cases,
        manifest.ordered_cases,
        manifest.controls.len(),
        manifest.positive_sky_modeled_intervals,
        manifest.positive_sky_input_intervals,
        manifest.positive_sky_modeled_cells,
        manifest.positive_sky_input_cells,
        manifest.positive_sky_declarations,
        manifest.positive_sky_local_payloads,
        manifest.positive_sky_local_draws,
        manifest.positive_sky_local_triangles,
        manifest.negative_authority_controls,
        manifest.negative_authority_declarations,
        manifest.removed_non_sky_contributions,
        manifest.persistent_mesh_identities,
        manifest.runtime_snapshots_are_declared_inputs,
        manifest.cutout_remains_deferred,
        manifest.no_diagnostic_grid_identity,
        manifest.no_generic_filter_used,
        manifest.unexplained_contributions,
        manifest.semantic_comparison_only,
        manifest.structural_fingerprint,
    );
    for control in &manifest.controls {
        println!(
            "control={}; snapshots={:?}; balanced={}; sky-depth-batches={}; deferred-cutout={}; fail-open={}; fingerprints={:?}",
            control.case,
            control.snapshots,
            control.balanced,
            control.sky_depth_batches,
            control.deferred_cutout_work,
            control.fail_open,
            control.structural_fingerprints,
        );
    }
    Ok(())
}
