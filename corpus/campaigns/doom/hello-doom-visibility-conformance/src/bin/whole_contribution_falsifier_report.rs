//! Slice 3 report for whole-source-contribution admission.
//!
//! This binary compares alternatives A and B over identical source geometry.
//! It deliberately does not invoke source-fragment reconstruction.

use hello_doom_visibility_conformance::observe_whole_contribution_falsifier;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let observation = observe_whole_contribution_falsifier()?;
    println!(
        "status={}; fixture={}; original-geometry-unchanged={}; alternative-a=retain-whole-far-contribution; negative-control-requires-partial={}; fingerprint={}",
        if observation.negative_control_requires_partial_survival {
            "falsified"
        } else {
            "not-falsified"
        },
        observation.fixture,
        observation.original_geometry_unchanged,
        observation.negative_control_requires_partial_survival,
        observation.fingerprint
    );
    for pose in observation.poses {
        println!(
            "pose={}; viewer={:?}; alternative-b-far={:?}:{:?}; topology={}/{}/{}; unrelated-rejections={:?}; overlap-columns={}; survivor-columns={}; ordinary-depth-authority-in-overlap={}; visible-source-invalid-columns-if-whole={}; requires-partial={}; topology-fingerprint={}",
            pose.label,
            pose.viewer_position,
            pose.far_result,
            pose.far_reason,
            pose.topology_admitted,
            pose.topology_rejected,
            pose.topology_unresolved_fail_open,
            pose.unrelated_rejections,
            pose.overlapping_columns,
            pose.surviving_columns,
            pose.ordinary_depth_authority_in_overlap,
            pose.visible_source_invalid_columns_if_retained_whole,
            pose.requires_partial_survival,
            pose.topology_fingerprint
        );
    }
    Ok(())
}
