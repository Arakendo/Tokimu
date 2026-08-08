# Alternative C Initial Findings

| Field | Value |
| --- | --- |
| Status | Interim evidence; no stable migration recommendation |
| Candidate source | 494 nonblank Rust lines / 15,807 bytes including tests (2026-08-07) |
| Provider references in candidate source | 0 |
| Unsafe blocks | 0 |
| Implemented candidate types | `Vec3`, `Vec4`, `Mat4` |

## What The Probe Demonstrates

- The currently exercised vector and transform subset can be implemented in
  original scalar Rust with no provider dependency, macro dependency, generated
  source, or unsafe code.
- The same shared C source passes through a dependency-free isolated corpus
  crate with no Cargo dependencies. This validates the current compilation
  boundary independently of the wider A/B study crate, which necessarily links
  the `glam` control.
- That isolated crate also builds for `wasm32-unknown-unknown`, demonstrating
  only target compilation for the dependency-free subset—not browser runtime
  conformance or measurement.
- Shared vector cases and non-singular transform cases agree with the current
  baseline observations; native and `wasm32-unknown-unknown` builds pass.
- The shared transform workload returns the same checksum as A and B, and the
  bounded transform workload observes zero allocations for C.
- A provider-specific renderer upload can be isolated to an explicit adapter
  that reconstructs a provider matrix from C's owned column array.
- The bounded `hello-cad` cursor-ray fixture now exercises homogeneous
  `Mat4 * Vec4`, perspective divide, and degenerate-ray rejection against the
  A baseline, confirming the owned matrix covers that named caller pressure.
- The bounded `hello-hole-punch` node-resolution fixture exercises the owned
  final-column setter, imported column-array input, and parent-child
  composition against a real decoded glTF node.
- The bounded FPS motion fixture adds repeated direction, movement, and
  distance pressure. C retains the baseline's direct component mutation shape
  for this corpus-local comparison.

## Costs And Open Questions

- `Vec2` and `Quat` are absent because no current direct caller has earned
  them. C therefore does not represent a complete replacement for the current
  five-type public re-export.
- `Mat4::inverse` uses bounded stack-only pivoted Gauss-Jordan elimination. Its
  all-NaN singular result is provisional experiment behavior, not a selected
  Tokimu contract.
- C's degenerate-view and singular-inverse non-finite output masks match the
  current baseline observation. That parity remains an observation, not a
  chosen validation, diagnostic, or recovery policy.
- The owned representation's adapter reconstruction is a real migration cost
  while provider-specific renderer boundaries remain. Its measured impact is
  not yet known beyond the bounded fixture.
- Scalar mechanics have not been compared against provider SIMD lowering,
  binary size, compile time, fuzz/property evidence, or browser/WASM timing.
- C has no real renderer, scene, serialization, FFI, or authoring migration.

## Interim Disposition

**Conditionally viable for further study.** C has cleared the first independence
test—Tokimu can own the exercised mechanics—but it has not earned a public API
migration. Real caller pressure, target measurements, and the deferred DOOM
revisit must determine whether that independence is worth the maintenance and
adapter cost.
