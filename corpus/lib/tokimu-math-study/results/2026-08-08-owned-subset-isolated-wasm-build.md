# Alternative C Isolated WASM Build

| Field | Value |
| --- | --- |
| Status | Build-portability observation; not WASM runtime evidence |
| Date | 2026-08-08 |
| Candidate | C — narrow owned implementation |
| Crate | `alternative-c-owned-subset` with an empty dependency list |
| Target | `wasm32-unknown-unknown` |
| Profile | `dev` |
| Rust | `rustc 1.95.0 (59807616e 2026-04-14)` |

## Command

```powershell
cargo build --manifest-path corpus/lib/tokimu-math-study/alternative-c-owned-subset/Cargo.toml --target wasm32-unknown-unknown --offline
```

## Result

The shared Alternative C source compiled successfully through the dependency-
free isolated crate for `wasm32-unknown-unknown`.

## Interpretation Limits

This confirms neither browser execution nor behavioral conformance on WASM.
It does not measure generated WASM size, performance, allocation behavior,
floating-point edge behavior, JavaScript integration, or a real Tokimu
application. The main study's A/B/C shared library target build remains the
comparison evidence; this result narrows only C's independent compilation
closure claim.
