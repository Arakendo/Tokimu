# Tokimu Math Vocabulary Study

This corpus area compares three experimental alternatives to Tokimu's current
direct `glam` type re-export:

- a Tokimu-owned vocabulary backed by `glam`;
- a narrow original Tokimu implementation; and
- a bounded, provenance-preserving fork/copy candidate.

Alternative A, the current `tokimu_core::math` re-export, remains the control.
No candidate in this folder is a stable Tokimu API or an admitted Native Ring
implementation.

The experiment is governed by
`docs/Plans/native-math-vocabulary-foreign-type-case-study.md` and AR-0019.
Candidate source and Cargo targets will be added only in the slices that define
their shared operation manifest and measurement boundary.

The current operation inventory is deliberately provisional. Once the DOOM WAD
plan completes, this corpus must be revisited: new object, transform,
animation, imported-data, collision, or rendering pressure may require new
manifest entries and conformance cases before this experiment can recommend a
stable vocabulary boundary.

## Initial Measurement Entry Point

The shared transform workload compares Alternative A with Alternative B and
checks their checksum before reporting elapsed time. Run it in the intended
profile and retain the command, target, toolchain, host metadata, iteration
count, and output with any result:

```powershell
cargo run -p tokimu-math-study --release --bin measure_transform_workload -- 1000000 15
```

This is a repeatable workload harness, not a benchmark conclusion. It warms
each candidate, rotates A/B/C measurement order across the requested sample
count, and reports min/median/max elapsed time. Its first use is to expose
wrapper and conversion cost; the final study must add allocation, binary-size,
compile-time, and target-specific evidence.

The first informational A/B invocation is retained in
`results/2026-08-07-initial-a-b-transform-run.md`.

The companion allocation observation uses a separate executable so argument
parsing and reporting occur outside each counted workload:

```powershell
cargo run -p tokimu-math-study --release --bin measure_transform_allocations -- 1000000
```

It fails if any currently wired A, B, or C transform workload allocates after
its counter is reset. This is narrow workload evidence, not a statement about
every future candidate API or application integration path.

The B/C adapter-boundary comparison is separately runnable:

```powershell
cargo run -p tokimu-math-study --release --bin measure_upload_boundary -- 1000000
```

It measures only the representative provider-upload conversion shape and must
not be generalized to a full renderer migration. The workload uses optimization
barriers around each conversion; a prior constant-checksum form was discarded
because the compiler eliminated the provider-backed loop.

The matching upload-allocation observation is available separately:

```powershell
cargo run -p tokimu-math-study --release --bin measure_upload_allocations -- 1000000
```

## Candidate-Isolated Link Outputs

These three minimal release executables each invoke exactly one shared
transform workload. They exist only to compare the linked output shape for the
same bounded workload; they do not represent a full Tokimu application or
renderer dependency closure.

```powershell
cargo build -p tokimu-math-study --release --locked --offline --bin measure_transform_binary_a
cargo build -p tokimu-math-study --release --locked --offline --bin measure_transform_binary_b
cargo build -p tokimu-math-study --release --locked --offline --bin measure_transform_binary_c
```

Current evidence is summarized, without a selection recommendation, in
`interim-comparison.md`.

The next real-caller migration evidence is constrained by
`representative-migration-protocol.md`; original callers and stable crates must
remain unchanged until an ADR explicitly selects a migration.

## Native Layout Observation

The current A/B/C size and alignment facts can be observed on the active native
target with:

```powershell
cargo run -p tokimu-math-study --locked --offline --bin observe_layouts
```

This is representation evidence only. It does not declare a stable Tokimu ABI,
FFI, SIMD, serialization, or GPU-upload layout.
