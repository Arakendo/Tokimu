# Isolated A/B/C WASM Output Observation

| Field | Value |
| --- | --- |
| Status | Target-specific micro-module observation; not application WASM-size evidence |
| Date | 2026-08-08 |
| Target | `wasm32-unknown-unknown` |
| Profile | `release` |
| Rust | `rustc 1.95.0 (59807616e 2026-04-14)` |
| Artifact shape | Uncompressed isolated `cdylib` with one transform/inverse checksum export |

## Output Sizes

| Candidate | WASM bytes |
| --- | ---: |
| A — stable direct-provider control | 2,782 |
| B — provider-backed vocabulary | 3,011 |
| C — narrow owned implementation | 3,953 |

C is 1,171 bytes larger than A and 942 bytes larger than B for this bounded
export. A and B retain provider mechanics; C emits its scalar mechanics.

## Interpretation Limits

These are uncompressed micro-modules, not Tokimu application artifacts. They
exclude renderer/platform glue, JavaScript bindings, browser assets, debugging
or distribution policy, code splitting, LTO policy comparison, compression,
and the wider candidate API surface. The values are a target-specific size
input only and must not select a stable vocabulary or implementation.
