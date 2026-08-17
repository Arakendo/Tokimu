use doom_geometry_provider::DoomSectorRuntimeHeightSnapshot;
use hello_doom_visibility_conformance::{
    dynamic_door_snapshot_fixture, masked_middle_topology_fixture,
    moving_platform_snapshot_fixture, observe_dynamic_ceiling_admission,
    observe_dynamic_floor_admission, observe_topology_admission,
    one_sky_identity_differential_fixture, paired_sky_far_control_fixture,
    projection_near_plane_crossing_fixture, source_terminal_boundary_fixture,
    vertical_aperture_control_fixture, TopologyAdmissionManifest,
};

fn print_manifest(label: &str, manifest: &TopologyAdmissionManifest) {
    println!(
        "fixture={label}; admitted={}; rejected={}; unresolved-fail-open={}; fingerprint={}",
        manifest.admitted, manifest.rejected, manifest.unresolved_fail_open, manifest.fingerprint
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for (label, fixture) in [
        ("open-aperture", source_terminal_boundary_fixture(true)?),
        ("terminal-solid", source_terminal_boundary_fixture(false)?),
        ("paired-sky", paired_sky_far_control_fixture()?),
        ("one-sky-identity", one_sky_identity_differential_fixture()?),
        ("vertical-aperture", vertical_aperture_control_fixture()?),
        ("masked-middle", masked_middle_topology_fixture()?),
        (
            "ambiguous-near-plane",
            projection_near_plane_crossing_fixture()?,
        ),
    ] {
        print_manifest(label, &observe_topology_admission(&fixture)?);
    }

    let door = dynamic_door_snapshot_fixture()?;
    let door_sector = door.map.sectors[1].source;
    for (label, height) in [("door-closed", 0), ("door-open", 128)] {
        print_manifest(
            label,
            &observe_dynamic_ceiling_admission(
                &door,
                DoomSectorRuntimeHeightSnapshot {
                    source_sector: door_sector,
                    floor_height: None,
                    ceiling_height: Some(height),
                },
            )?,
        );
    }

    let platform = moving_platform_snapshot_fixture()?;
    let platform_sector = platform.map.sectors[0].source;
    for (label, height) in [("platform-low", 0), ("platform-raised", 48)] {
        print_manifest(
            label,
            &observe_dynamic_floor_admission(
                &platform,
                DoomSectorRuntimeHeightSnapshot {
                    source_sector: platform_sector,
                    floor_height: Some(height),
                    ceiling_height: None,
                },
            )?,
        );
    }
    Ok(())
}
