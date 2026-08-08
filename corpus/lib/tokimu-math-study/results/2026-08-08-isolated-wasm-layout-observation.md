# Isolated A/B/C WASM Layout Observation

| Field | Value |
| --- | --- |
| Status | WASM compiler/engine observation; not a stable ABI claim |
| Date | 2026-08-08 |
| Target | `wasm32-unknown-unknown`, `release` |
| Execution engine | Node.js `v22.21.0` WebAssembly engine |
| Method | Corpus-only `align_of` exports from isolated A/B/C modules |

## Engine Output

| Candidate | `Vec4` alignment | `Mat4` alignment |
| --- | ---: | ---: |
| A — stable direct-provider control | 16 | 16 |
| B — provider-backed vocabulary | 16 | 16 |
| C — narrow owned implementation | 4 | 4 |

## Finding

The native layout split is also present on this WASM target: B retains A's
observed alignment while C's scalar representation does not. Mathematical
conformance and bounded engine execution therefore do not imply direct
representation compatibility across either tested target.

## Interpretation Limits

The exports report only alignment for two types. They do not establish a Rust
or C ABI, field order, serialization format, browser GPU-buffer acceptance,
SIMD strategy, or a stable Tokimu layout policy. A future representation change
needs a named boundary, target-specific evidence, and an explicit decision.
