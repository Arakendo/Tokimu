use hello_doom_visibility_conformance::{
    observe_authoritative_sky_regions, prepare_authoritative_sky_depth_declarations,
    prepare_authoritative_sky_submission_local_geometry, terminal_sky_ordered_fixture,
    SubmissionIdentity, SubmissionLocalGeometryLimits,
};

fn main() -> Result<(), String> {
    let fixture = terminal_sky_ordered_fixture().map_err(|error| error.to_string())?;
    let regions = observe_authoritative_sky_regions(&fixture, 41, "static-source-fixture")?;
    let depth = prepare_authoritative_sky_depth_declarations(&regions, 0.25, "doom-sky:SKY1");
    let snapshot = prepare_authoritative_sky_submission_local_geometry(
        &depth,
        SubmissionIdentity(1),
        SubmissionLocalGeometryLimits::default(),
    )
    .map_err(|error| error.to_string())?;
    println!(
        "ar-0030-g2; submission={}; local-payloads={}; ordered-draws={}; vertices={}; triangles={}; persistent-materials={:?}; persistent-mesh-identities={}; source-correlations={:?}; fingerprint={}",
        snapshot.submission.0,
        snapshot.payloads.len(),
        snapshot.draws.len(),
        snapshot.total_vertices,
        snapshot.total_triangles,
        snapshot.persistent_material_keys,
        snapshot.persistent_mesh_identities,
        snapshot
            .draws
            .iter()
            .map(|draw| draw.source_correlation.as_str())
            .collect::<Vec<_>>(),
        snapshot.structural_fingerprint,
    );
    Ok(())
}
