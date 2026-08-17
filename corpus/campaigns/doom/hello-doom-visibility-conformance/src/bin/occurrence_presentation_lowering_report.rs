use hello_doom_visibility_conformance::lower_occurrences_to_presentation;

fn main() -> Result<(), String> {
    let manifest = lower_occurrences_to_presentation()?;

    println!(
        "whole: order={}; source={}; occurrence={:?}; interval={:?}; vertices={}; uv={}; generated-view-local={}",
        manifest.whole_control.source_order,
        manifest.whole_control.source_correlation,
        manifest.whole_control.occurrence_correlation,
        manifest.whole_control.source_interval,
        manifest.whole_control.mesh.vertex_count(),
        manifest.whole_control.mesh.has_texture_coordinates(),
        manifest.whole_control.generated_view_local_geometry,
    );
    for declaration in &manifest.partial_declarations {
        println!(
            "partial: order={}; source={}; occurrence={:?}; interval={:?}; material={}; vertices={}; uv={}; attribution={}; generated-view-local={}",
            declaration.source_order,
            declaration.source_correlation,
            declaration.occurrence_correlation,
            declaration.source_interval,
            declaration.material_identity,
            declaration.mesh.vertex_count(),
            declaration.mesh.has_texture_coordinates(),
            declaration.diagnostic_attribution,
            declaration.generated_view_local_geometry,
        );
    }

    let valid = manifest.retained_semantic_occurrences == manifest.lowered_semantic_occurrences
        && manifest.source_order_preserved
        && manifest.source_correlation_preserved
        && manifest.endpoints_from_continuous_source_domains
        && manifest.uv_streams_complete
        && manifest.generated_geometry_is_view_local;
    if !valid {
        return Err(format!("ordinary occurrence lowering failed: {manifest:?}"));
    }

    println!(
        "status=validated; retained-occurrences={}; lowered-occurrences={}; source-order-preserved={}; source-correlation-preserved={}; endpoints-from-continuous-domains={}; uv-streams-complete={}; generated-geometry-view-local={}; fingerprint={}",
        manifest.retained_semantic_occurrences,
        manifest.lowered_semantic_occurrences,
        manifest.source_order_preserved,
        manifest.source_correlation_preserved,
        manifest.endpoints_from_continuous_source_domains,
        manifest.uv_streams_complete,
        manifest.generated_geometry_is_view_local,
        manifest.structural_fingerprint,
    );
    Ok(())
}
