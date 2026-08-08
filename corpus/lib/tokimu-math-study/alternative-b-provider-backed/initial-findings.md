# Alternative B Initial Findings

| Field | Value |
| --- | --- |
| Status | Interim evidence; no migration recommendation |
| Candidate | Provider-backed Tokimu vocabulary over the pinned local `glam` provider |
| Scope | Five retained type names; behavior implemented only where the 0.1 manifest requires it |
| Source observation | 450 nonblank Rust lines / 13,719 bytes in `src/alternative_b.rs`, including tests (2026-08-07) |

## What The Probe Demonstrates

- `Vec2`, `Vec3`, `Vec4`, `Quat`, and `Mat4` can be presented as candidate
  Tokimu types while retaining `glam` mechanics in private fields.
- The transform-heavy A/B workload has identical checksums for the retained
  run, and its allocation-counter run reported zero ordinary-operation
  allocations for both alternatives.
- Native tests and a `wasm32-unknown-unknown` build pass for the candidate.
- A source audit found no `glam` type in an Alternative B `pub` signature.
  Provider construction and conversion methods are crate-private evidence
  helpers, not candidate public API.
- An integration test compiles as an external consumer using B vectors,
  matrices, and the candidate camera fixture without importing a provider type
  or reaching the crate-private upload conversion.
- The bounded `hello-cad` cursor-ray fixture exercises the previously
  untested homogeneous `Mat4 * Vec4` and perspective-divide path against the
  A baseline. It adds a real migration seam: B exposes `Vec4::w()` rather than
  the provider's public field shape.
- The bounded `hello-hole-punch` node-resolution fixture exercises the
  `set_w_axis(...)` compatibility helper against a real decoded glTF node and
  parent-child composition. It confirms that the helper is an attributable
  migration seam, not an untested API substitution.
- The bounded FPS motion fixture adds `distance` and in-place vector update.
  It also makes B's private representation visible at a real caller seam:
  baseline component mutation becomes getter-based reconstruction.

## Costs And Leaks Still Visible

- The candidate implementation contains 38 direct provider references. That
  is expected for a delegation experiment, but means the provider remains a
  strong internal representation assumption.
- Candidate size and alignment match the provider in current builds, but no
  `repr` promise, POD/FFI promise, serialization promise, or ABI compatibility
  claim is made. The equality is observed evidence, not a stable contract.
- Current writable `glam::Mat4::w_axis` access becomes `w_axis()` plus
  `set_w_axis(...)`. This is a real ergonomic and migration difference, not a
  cosmetic rename.
- `Vec2` and `Quat` remain intentionally minimal because no current direct
  caller earned more operations. Therefore B is not source-compatible with the
  full `glam` API and cannot be judged by type names alone.
- No real caller has been migrated. Conversion counts at renderer, asset, GPU,
  serialization, FFI, and authoring boundaries remain unknown. A corpus-local
  renderer-shaped fixture now confirms one explicit conversion at a
  provider-specific matrix upload boundary; it is illustrative, not a count
  for the real renderer.
- The equivalent C fixture must reconstruct a provider matrix from its owned
  column array. This makes B's private-provider representation a meaningful
  migration tradeoff to measure, rather than proof that all wrappers are free.
- The single timing observation is not statistically useful; host identity was
  unavailable to the sandbox and repeated native/WASM measurement is pending.
- The retained source-surface observation records 41 direct provider references
  in B and must not be mistaken for a binary-size or long-term maintenance
  comparison.

## Interim Disposition

**Conditionally viable for further study.** The experiment demonstrates a
private provider seam for the currently pressured vector and transform subset,
but it does not yet demonstrate that the seam is worth its compatibility,
ergonomics, and maintenance cost. Complete migration, measurement, and the
deferred DOOM pressure revisit remain required before AR-0019 can choose retain,
wrap, replace, or reject.
