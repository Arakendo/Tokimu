# Alternative A Baseline

This area will retain measurements and conformance evidence for the current
direct `glam` re-export. It is a control, not a new implementation.

## Independent WASM Control

`Cargo.toml` and `src/lib.rs` compile the actual stable
`tokimu_core::math::{Mat4, Vec3}` re-exports as a small `cdylib`. Its
corpus-only checksum export runs the same bounded transform/inverse path as
the isolated B and C probes in a Node WebAssembly engine.

This is a narrow A/B/C runtime control, not a second stable API, browser
integration test, or full Tokimu application.
