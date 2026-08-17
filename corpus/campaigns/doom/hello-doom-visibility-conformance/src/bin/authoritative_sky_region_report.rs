use hello_doom_visibility_conformance::{
    observe_authoritative_sky_regions, one_sky_far_control_fixture, paired_sky_far_control_fixture,
    prepare_authoritative_sky_depth_declarations, terminal_sky_ordered_fixture,
    vertical_aperture_control_fixture, AuthoritativeSkyRegionManifest, DoomVisibilityFixture,
};

fn report_case(
    case: &str,
    fixture: DoomVisibilityFixture,
    expected_intervals: Option<bool>,
) -> Result<(), String> {
    let manifest = observe_authoritative_sky_regions(&fixture, 41, "static-source-fixture")?;
    print_manifest(case, &manifest);
    print_depth_manifest(case, &manifest, 0.25);

    if manifest.input_sky_intervals
        != manifest.modeled_sky_intervals + manifest.omitted_sky_intervals
        || manifest.input_sky_cells != manifest.modeled_sky_cells + manifest.omitted_sky_cells
    {
        return Err(format!("{case}: authoritative sky ledger did not conserve"));
    }
    if manifest.removed_non_sky_contributions != 0 {
        return Err(format!(
            "{case}: sky-region modeling removed an ordinary contribution"
        ));
    }
    if let Some(expected) = expected_intervals {
        let observed = manifest.input_sky_intervals > 0;
        if observed != expected {
            return Err(format!(
                "{case}: expected retained-sky-intervals={expected}, observed={observed}"
            ));
        }
    }
    Ok(())
}

fn print_depth_manifest(case: &str, regions: &AuthoritativeSkyRegionManifest, clip_depth: f32) {
    let depth = prepare_authoritative_sky_depth_declarations(regions, clip_depth, "doom-sky:SKY1");
    let vertices = depth
        .declarations
        .iter()
        .map(|declaration| declaration.positions.len())
        .sum::<usize>();
    let triangles = depth
        .declarations
        .iter()
        .map(|declaration| declaration.triangle_count)
        .sum::<usize>();
    let rejected = depth
        .outcomes
        .iter()
        .filter(|outcome| outcome.rejection.is_some())
        .count();
    println!(
        "authoritative-sky-depth case={case}; clip-depth={clip_depth}; declarations={}; vertices={vertices}; triangles={triangles}; rejected={rejected}; persistent-material={}; persistent-mesh-identities={}; diagnostic-grid-identities=0; fingerprint={}",
        depth.declarations.len(),
        depth.persistent_material_key,
        depth.persistent_mesh_identities,
        depth.structural_fingerprint,
    );
}

fn print_manifest(case: &str, manifest: &AuthoritativeSkyRegionManifest) {
    println!(
        "authoritative-sky case={case}; fixture={}; snapshot={}; input-intervals={}; input-cells={}; modeled-regions={}; modeled-intervals={}; modeled-cells={}; omitted-intervals={}; omitted-cells={}; paired-columns-observed={}; paired-columns-claimed={}; removed-non-sky={}; fail-open={}; fingerprint={}",
        manifest.prepared_view.fixture,
        manifest.runtime_snapshot,
        manifest.input_sky_intervals,
        manifest.input_sky_cells,
        manifest.regions.len(),
        manifest.modeled_sky_intervals,
        manifest.modeled_sky_cells,
        manifest.omitted_sky_intervals,
        manifest.omitted_sky_cells,
        manifest.paired_boundary_columns_observed,
        manifest.paired_boundary_columns_claimed,
        manifest.removed_non_sky_contributions,
        manifest.fail_open,
        manifest.structural_fingerprint,
    );
    for (index, region) in manifest.regions.iter().enumerate() {
        println!(
            "authoritative-sky-region case={case}; region={index}; plane={:?}:{}:{}:{}; instance={}; authority={}:{}; source-order={}; source-sectors={:?}; source-segs={:?}; paired-segs={:?}; horizontal-ndc={:.9}..{:.9}; boundary-knots={}/{}; oracle-columns={}..{}; oracle-intervals={}; oracle-cells={}",
            region.source_plane.kind,
            region.source_plane.height,
            region.source_plane.texture,
            region.source_plane.light,
            region.source_plane_instance,
            region.source_sector,
            region.source_seg,
            region.source_order,
            region.source_sectors,
            region.source_segs,
            region.paired_sky_boundary_source_segs,
            region.horizontal_ndc[0],
            region.horizontal_ndc[1],
            region.upper_boundary.len(),
            region.lower_boundary.len(),
            region.oracle_columns[0],
            region.oracle_columns[1],
            region.oracle_intervals,
            region.oracle_cells,
        );
    }
}

fn main() -> Result<(), String> {
    let positive = terminal_sky_ordered_fixture().map_err(|error| error.to_string())?;
    let positive_regions =
        observe_authoritative_sky_regions(&positive, 41, "static-source-fixture")?;
    report_case("retained-terminal-sky-positive", positive, Some(true))?;
    for (control, depth) in [("invalid-depth", f32::NAN), ("near-plane", -1.0)] {
        let manifest =
            prepare_authoritative_sky_depth_declarations(&positive_regions, depth, "doom-sky:SKY1");
        println!(
            "authoritative-sky-depth-control control={control}; declarations={}; outcomes={:?}; persistent-mesh-identities={}; fingerprint={}",
            manifest.declarations.len(),
            manifest.outcomes,
            manifest.persistent_mesh_identities,
            manifest.structural_fingerprint,
        );
    }
    report_case(
        "paired-boundary-without-retained-plane",
        paired_sky_far_control_fixture().map_err(|error| error.to_string())?,
        Some(false),
    )?;
    report_case(
        "one-sky-negative",
        one_sky_far_control_fixture().map_err(|error| error.to_string())?,
        Some(false),
    )?;
    report_case(
        "nearby-ordinary-aperture",
        vertical_aperture_control_fixture().map_err(|error| error.to_string())?,
        Some(false),
    )?;
    Ok(())
}
