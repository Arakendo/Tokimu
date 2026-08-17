use hello_doom_visibility_conformance::observe_private_occurrence_model;

fn main() {
    let observation = observe_private_occurrence_model();
    println!(
        "status=validated; source-contributions={}; partial-occurrences={}; distinct-source-identities={}; whole-retain-generated-geometry={}; unresolved-retains-original={}; shared-boundary-consumers={}; rejected-invalid-controls={}; fingerprint={}",
        observation.source_contributions,
        observation.partial_occurrences,
        observation.distinct_source_identities,
        observation.whole_retain_generated_geometry,
        observation.unresolved_retains_original,
        observation.shared_boundary_consumers,
        observation.rejected_invalid_controls,
        observation.fingerprint,
    );
}
