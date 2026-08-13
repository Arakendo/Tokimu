//! Slice 8 CPU scaling controls for the two bounded bulk-reference candidates.
//!
//! This is deliberately not a GPU benchmark. It distinguishes one-shot input
//! construction, resident-input/fresh-result classification, and resident-input
//! / reused-result classification so a later provider experiment cannot label
//! allocation or transfer work as pure dispatch work.

use std::time::Instant;

use tokimu_math_study::bulk_reference::{
    candidate_count, classification_checksum, classify_aabbs, classify_aabbs_into, classify_points,
    classify_points_into, generated_aabbs, generated_points, unit_cube_planes,
};

// `1_861` is the retained E1M1 full-submission control. It is intentionally
// between the dispatch-hostile 1K case and the general scaling series; it is
// not evidence that a game-map draw count earns GPU compute.
const SIZES: [usize; 5] = [1_000, 1_861, 10_000, 100_000, 1_000_000];
const SAMPLES: usize = 5;

#[derive(Clone, Copy)]
enum Workload {
    Bounds,
    Points,
}

impl Workload {
    const ALL: [Self; 2] = [Self::Bounds, Self::Points];

    const fn label(self) -> &'static str {
        match self {
            Self::Bounds => "ordered_aabb",
            Self::Points => "ordered_point",
        }
    }
}

fn median(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[samples.len() / 2]
}

fn report(label: &str, samples: &mut [u128], candidates: usize, checksum: u64) {
    println!(
        "{label}=median_ns:{}; candidates:{candidates}; checksum:{checksum:016x}",
        median(samples)
    );
}

fn main() {
    assert!(
        std::env::args().nth(1).is_none(),
        "usage: measure_bulk_references"
    );
    let planes = unit_cube_planes();
    println!("samples={SAMPLES}; semantics=ordered-id-preserving; gpu=not-involved");

    for size in SIZES {
        for workload in Workload::ALL {
            let mut one_shot_elapsed = Vec::with_capacity(SAMPLES);
            let mut one_shot_checksum = None;
            let mut one_shot_candidates = 0;
            for _ in 0..SAMPLES {
                let started = Instant::now();
                let records = match workload {
                    Workload::Bounds => classify_aabbs(&planes, &generated_aabbs(size)),
                    Workload::Points => classify_points(&planes, &generated_points(size)),
                };
                one_shot_elapsed.push(started.elapsed().as_nanos());
                let checksum = classification_checksum(&records);
                assert!(one_shot_checksum.is_none_or(|expected| expected == checksum));
                one_shot_checksum = Some(checksum);
                one_shot_candidates = candidate_count(&records);
            }
            report(
                &format!("size={size}; workload={}; mode=one_shot", workload.label()),
                &mut one_shot_elapsed,
                one_shot_candidates,
                one_shot_checksum.expect("samples are nonzero"),
            );

            match workload {
                Workload::Bounds => {
                    let input = generated_aabbs(size);
                    let reference = classify_aabbs(&planes, &input);
                    let checksum = classification_checksum(&reference);
                    let candidates = candidate_count(&reference);
                    let mut fresh_elapsed = Vec::with_capacity(SAMPLES);
                    for _ in 0..SAMPLES {
                        let started = Instant::now();
                        let records = classify_aabbs(&planes, &input);
                        fresh_elapsed.push(started.elapsed().as_nanos());
                        assert_eq!(classification_checksum(&records), checksum);
                    }
                    report(
                        &format!(
                            "size={size}; workload={}; mode=resident_input_fresh_result",
                            workload.label()
                        ),
                        &mut fresh_elapsed,
                        candidates,
                        checksum,
                    );
                    let mut reused = Vec::with_capacity(size);
                    let mut reused_elapsed = Vec::with_capacity(SAMPLES);
                    for _ in 0..SAMPLES {
                        let started = Instant::now();
                        classify_aabbs_into(&planes, &input, &mut reused);
                        reused_elapsed.push(started.elapsed().as_nanos());
                        assert_eq!(classification_checksum(&reused), checksum);
                    }
                    report(
                        &format!(
                            "size={size}; workload={}; mode=resident_input_reused_result",
                            workload.label()
                        ),
                        &mut reused_elapsed,
                        candidates,
                        checksum,
                    );
                }
                Workload::Points => {
                    let input = generated_points(size);
                    let reference = classify_points(&planes, &input);
                    let checksum = classification_checksum(&reference);
                    let candidates = candidate_count(&reference);
                    let mut fresh_elapsed = Vec::with_capacity(SAMPLES);
                    for _ in 0..SAMPLES {
                        let started = Instant::now();
                        let records = classify_points(&planes, &input);
                        fresh_elapsed.push(started.elapsed().as_nanos());
                        assert_eq!(classification_checksum(&records), checksum);
                    }
                    report(
                        &format!(
                            "size={size}; workload={}; mode=resident_input_fresh_result",
                            workload.label()
                        ),
                        &mut fresh_elapsed,
                        candidates,
                        checksum,
                    );
                    let mut reused = Vec::with_capacity(size);
                    let mut reused_elapsed = Vec::with_capacity(SAMPLES);
                    for _ in 0..SAMPLES {
                        let started = Instant::now();
                        classify_points_into(&planes, &input, &mut reused);
                        reused_elapsed.push(started.elapsed().as_nanos());
                        assert_eq!(classification_checksum(&reused), checksum);
                    }
                    report(
                        &format!(
                            "size={size}; workload={}; mode=resident_input_reused_result",
                            workload.label()
                        ),
                        &mut reused_elapsed,
                        candidates,
                        checksum,
                    );
                }
            }
        }
    }
}
