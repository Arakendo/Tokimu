# Isolated A/B/C WASM-Engine Execution

| Field | Value |
| --- | --- |
| Status | Bounded WASM execution evidence; not browser or application conformance |
| Date | 2026-08-08 |
| WASM target | `wasm32-unknown-unknown`, `release` |
| Execution engine | Node.js `v22.21.0` WebAssembly engine |
| Candidates | A — stable direct-provider control; B — provider-backed; C — narrow owned |
| Probe | Composed translation, Y rotation, non-uniform scale, inverse round trip; exported finite checksum |

## Build Commands

```powershell
cargo build --manifest-path corpus/lib/tokimu-math-study/alternative-b-provider-backed/Cargo.toml --target wasm32-unknown-unknown --release --offline
cargo build --manifest-path corpus/lib/tokimu-math-study/alternative-c-owned-subset/Cargo.toml --target wasm32-unknown-unknown --release --offline
```

## Engine Results

| Candidate | `tokimu_math_study_wasm_probe()` result |
| --- | ---: |
| A | `292.00006103515625` |
| B | `292.00006103515625` |
| C | `292` |

The expected caller-shaped checksum is `292`; the B/C difference is within the
study's `3e-5` point-round-trip tolerance after the checksum's weighting. A
and B execute the same pinned provider mechanics and return the same value.

## Interpretation Limits

This executes A, B, and C through minimal isolated `cdylib` probes. It does
not execute the shared conformance suite, a browser canvas, JavaScript input or
lifecycle integration, a renderer, allocation instrumentation, performance
measurement, or an actual Tokimu WASM application. It therefore advances the
specific differential runtime evidence gap but does not satisfy the plan's full
native/WASM behavioral acceptance criteria.
