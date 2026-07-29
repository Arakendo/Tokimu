# Performance Diagnostics Corpus

This example-side support library validates Tokimu's provider-neutral
performance diagnostic contracts with deterministic inputs.

It covers:

- healthy observations;
- one transient spike;
- sustained pressure and warning latching;
- recovery;
- bounded diagnostic overflow;
- stable renderer resources after warm-up;
- deliberately repeated binding allocation;
- deliberately repeated mesh upload;
- provider-neutral asset allocation, preparation, replacement, and release;
- explicitly unsupported GPU completion timing.

The renderer cases use controlled `RenderStats` snapshots. They do not claim to
measure a GPU or establish cross-machine timing goldens.

Call `run_all_cases()` to collect in-memory artifacts, or
`write_all_artifacts(output_root)` to write one JSON artifact per case. The
artifacts include build and target identity, workload revision, algorithm
identity, budgets, observations, diagnostics, transition order, and dropped
record counts. Resource lifecycle artifacts preserve stable identity and
generation while leaving bytes and durations absent when the producer cannot
measure them honestly.

Schema 2 also includes bounded per-case summaries. Numeric cases retain their
raw samples and report count, last, total, average, and peak. Resource cases
retain their raw events and report transition counts, final active resources,
and last generation. Both reset per case; percentiles and unbounded history are
intentionally absent.

`build_diagnostic_report()` is the first non-console structured consumer. It
produces author-facing JSON explanations from diagnostic kind, source, metric,
observation, budget, and unit without parsing the human-readable message.
Reports preserve the diagnostic sequence, attribute current evidence to a
collective subsystem, and explicitly decline to infer an individual cause.
`write_all_diagnostic_reports()` persists reports containing explanations under
`reports/<case-id>.json`.
