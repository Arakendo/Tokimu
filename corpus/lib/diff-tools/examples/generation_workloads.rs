//! Reproducible, non-golden workload observations for Diff Tools generation.
//!
//! Run with:
//! `cargo run -p diff-tools --example generation_workloads --release`
//!
//! The output is diagnostic evidence. It is not a cross-machine performance
//! contract and intentionally avoids pass/fail timing thresholds.

use std::time::Instant;

use diff_tools::{diff_text, DiffGenerationConfig, DiffLimits, TextDocument};

fn main() {
    run_workload("interactive", 160, 19);
    run_workload("artifact", 1_600, 37);
}

fn run_workload(name: &str, line_count: usize, change_interval: usize) {
    let old_source = source(line_count, None);
    let new_source = source(line_count, Some(change_interval));
    let limits = DiffLimits::default();
    let old = TextDocument::parse(&old_source, limits).expect("generated input must be valid");
    let new = TextDocument::parse(&new_source, limits).expect("generated input must be valid");

    let started = Instant::now();
    let document = diff_text(
        format!("{name}-before.txt"),
        &old,
        format!("{name}-after.txt"),
        &new,
        DiffGenerationConfig::default(),
        limits,
    )
    .expect("generated workload must remain within the default bounds");
    let elapsed = started.elapsed();
    let hunk_count = document
        .files()
        .iter()
        .map(|file| file.hunks().len())
        .sum::<usize>();

    println!(
        "diff-tools workload={name} lines={line_count} matrix_cells={} hunks={hunk_count} algorithm={:?} elapsed_ms={:.3}",
        (line_count + 1) * (line_count + 1),
        document.algorithm(),
        elapsed.as_secs_f64() * 1_000.0,
    );
}

fn source(line_count: usize, change_interval: Option<usize>) -> String {
    (0..line_count)
        .map(|line| {
            if change_interval.is_some_and(|interval| line % interval == 0) {
                format!("record {line:04}: changed\n")
            } else {
                format!("record {line:04}: stable\n")
            }
        })
        .collect()
}
