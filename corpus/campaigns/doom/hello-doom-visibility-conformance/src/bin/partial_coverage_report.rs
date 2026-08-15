//! Headless report for the whole-source-candidate expressiveness control.
//!
//! This binary retains bounded corpus evidence only. It does not propose a
//! renderer visibility contract or turn Doom diagnostic columns into a public
//! pixel/span API.

use hello_doom_visibility_conformance::observe_partial_coverage_expressiveness;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let observation = observe_partial_coverage_expressiveness()?;
    println!(
        "status=observed; fixture=partial-paired-sky-far-control; paired-sky-source-seg={}; far-wall-source-seg={}; paired-sky-columns={}; far-wall-columns={}; overlapping-columns={}; far-only-columns={}; overlapping-runs={:?}; surviving-runs={:?}; whole-source-seg-selection={}",
        observation.paired_sky_source_seg,
        observation.far_wall_source_seg,
        observation.paired_sky_columns,
        observation.far_wall_columns,
        observation.overlapping_columns,
        observation.far_only_columns,
        observation.overlapping_runs,
        observation.surviving_runs,
        if observation.requires_source_fragments {
            "insufficient-requires-fragments"
        } else {
            "not-falsified"
        }
    );
    Ok(())
}
