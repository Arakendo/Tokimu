# Alternative C: Narrow Owned Subset

This area tests an original Tokimu implementation limited to operations earned
by real callers and the shared conformance manifest.

The first slice is `src/alternative_c.rs::{Vec3, Vec4}`. It intentionally
contains no provider reference and implements only the frozen vector
operations, including the `extend` / `truncate` boundary. It is not a complete
Alternative C result, stable API, or authorization to recreate a general math
library.

## Independent Compilation Boundary

`Cargo.toml` and `src/lib.rs` form a dependency-free corpus crate that imports
the shared candidate source by path rather than copying it. It is intentionally
outside the parent workspace, so the command below validates that the current
owned subset has no `glam`, build-script, macro, or runtime dependency merely
because the wider A/B comparison crate needs the provider control:

```powershell
cargo test --manifest-path corpus/lib/tokimu-math-study/alternative-c-owned-subset/Cargo.toml --offline
cargo build --manifest-path corpus/lib/tokimu-math-study/alternative-c-owned-subset/Cargo.toml --target wasm32-unknown-unknown --offline
```

This is a compilation-boundary observation only. It does not make C stable,
prove numerical completeness, or establish that a future Tokimu implementation
will remain this small.

The same crate also exposes a corpus-only plain-WASM checksum probe for
isolated Node WebAssembly-engine execution. It is not a browser/API contract.

`src/migration_c.rs` also records the current renderer-boundary cost: until a
renderer uses the owned representation directly, a provider upload reconstructs
`glam::Mat4` from the candidate column array. That explicit conversion is
evidence to measure, not a hidden compatibility promise.
