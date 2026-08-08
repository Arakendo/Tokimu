# Shared WASM Conformance-Harness Observation

| Field | Value |
| --- | --- |
| Status | Shared A/B/C conformance executed; browser/WGPU evidence remains open |
| Date | 2026-08-08 |
| Target | `wasm32-unknown-unknown` |
| Scope | Full `tokimu-math-study` test target, including A/B/C conformance cases |

## Initial Gap And Resolution

```text
cargo test -p tokimu-math-study --target wasm32-unknown-unknown --locked --offline
```

Cargo initially compiled the test target and then tried to run
`tokimu_math_study-…​.wasm` as a Windows executable (error 193). The study now
pins `wasm-bindgen-test = 0.3.76`, matching the workspace and installed
`wasm-bindgen` runner schema `0.2.126`. Its twelve existing shared
`conformance.rs` tests carry `wasm_bindgen_test` only on `wasm32`; the native
tests retain their ordinary `#[test]` execution.

```text
CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-bindgen-test-runner \
  cargo test -p tokimu-math-study --lib --target wasm32-unknown-unknown --locked
```

The runner used Node `v22.21.0` and executed all 12 shared cases successfully:
A baseline, B provider-backed candidate, C owned candidate, plus the bounded
D Vec3 comparison and the fixed affine/camera differential sweeps.

## Finding

The full shared A/B/C conformance suite now executes on a named WASM engine
using the same assertion bodies as native. The isolated Node probes remain
separate performance/layout evidence; they are not the basis for this result.

This closes the A/B/C native-and-WASM shared-conformance criterion for the
current finite/observed scope. It does not select any stable behavior for
degenerate inputs, establish browser/WGPU behavior, or make D a matrix-capable
candidate.

## Reopening / Completion Conditions

- A new shared conformance case must retain its WASM registration beside the
  native assertion.
- Keep browser/WGPU execution separate: a successful plain-WASM conformance
  run does not satisfy renderer/application evidence.
