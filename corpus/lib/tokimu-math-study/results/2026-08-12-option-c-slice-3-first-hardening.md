# Option C0 Slice 3 First Hardening Result

| Field | Value |
| --- | --- |
| Date | 2026-08-12 |
| Status | Partial Slice 3 evidence; no production admission |
| Candidate | `src/alternative_c.rs` shared with the isolated dependency-free crate |
| Numerical contract | `numerical-contract-c0.md` |
| Toolchain | Rust/Cargo 1.95.0, `x86_64-pc-windows-msvc` |

## Implemented Corpus Surface

The first hardening group adds only mechanics traced by the post-DOOM
manifest or required to make the selected numerical contract testable:

- `Vec3` signed axis values and Euclidean length;
- checked normalization and explicitly zero-tolerant checked normalization;
- deterministic caller-order accumulation without copying `glam`'s `Sum`
  surface;
- explicit nested column-array conversion;
- checked right-handed look-at, GL-depth perspective, and GL-depth
  orthographic construction;
- checked inversion with finite-input validation and a two-sided `1e-3`
  identity residual bound; and
- checked perspective-dividing point projection.

Unchecked operations remain only for the inherited A/C comparison paths.
They retain raw IEEE/provider-observation behavior and are not the recovery
boundary for external data.

## Failure And Differential Evidence

The tests distinguish:

- raw divide-by-zero IEEE propagation;
- valid normalization from zero, NaN, infinity, and overflow rejection;
- exactly-zero tolerant normalization from non-finite rejection;
- valid and degenerate/non-finite view construction;
- valid and invalid perspective/orthographic parameters;
- valid affine inversion from singular/non-finite rejection;
- valid perspective division from zero-`w`/non-finite rejection; and
- A agreement for finite normalization, camera, projection, inversion, and
  projection results versus intentional C0 rejection where A exposes an
  unchecked all-NaN/provider observation.

The selected combined absolute/relative comparator is exercised by the new
differential test. A fixed-seed 128-case property loop exercises unit length,
dot symmetry, cross-product orthogonality, and affine inverse round trips.
Existing conformance sweeps continue to exercise matrix composition and the
inherited caller-shaped paths.

## Boundary And Size Observation

| Observation | Slice 0 C0 | First hardening | Change |
| --- | ---: | ---: | ---: |
| Source lines | 581 | 979 | +398 |
| Nonblank lines | 503 | 860 | +357 |
| Source bytes | 16,047 | 30,196 | +14,149 |
| Source-local tests | 3 | 9 | +6 |
| `unsafe` occurrences | 0 | 0 | 0 |
| Direct `glam::`, `Vec2`, or `Quat` definitions | 0 | 0 | 0 |

Current source SHA-256:

```text
5938890e1a9fe2594cebe2272eba7e0d388c0c22d94f3f8be80c3ebccdba08da
```

This growth is an engineering-cost finding. A Tokimu-owned implementation is
still narrow and auditable, but selected failure semantics and evidence nearly
doubled the source. The study must not describe Option C as cheap merely
because its value types are small.

The isolated manifest remains dependency-free. Ordinary operations use scalar
values and fixed-size stack arrays only; no allocation API, generated source,
foreign type, provider call, or unsafe block is present. A dedicated allocation
observation remains open before the Slice 3 acceptance criterion is claimed.

## Validation

Passed offline:

```text
cargo test --manifest-path corpus/lib/tokimu-math-study/alternative-c-owned-subset/Cargo.toml --locked --offline
  10 tests passed (9 shared implementation tests plus isolated-boundary test)

cargo test -p tokimu-math-study --locked --offline --lib
  50 tests passed

cargo clippy --manifest-path corpus/lib/tokimu-math-study/alternative-c-owned-subset/Cargo.toml --all-targets --locked --offline -- -D warnings
  passed

cargo build --manifest-path corpus/lib/tokimu-math-study/alternative-c-owned-subset/Cargo.toml --target wasm32-unknown-unknown --locked --offline
  passed (compile-only; not browser/engine execution evidence)
```

The main study test still reproduces the already-retained pinned-glam
`unused_attributes` warning flood. The isolated C clippy gate is clean; Windows
may emit non-semantic incremental-cache hard-link fallback warnings during
test builds.

## Remaining Slice 3 Work

- add a bounded reusable generated-case/fuzz entry rather than only an inline
  fixed-seed loop;
- add an independently expressed scalar reference where it can distinguish
  two implementations sharing the same mistake;
- retain a dedicated allocation observation;
- expand conditioning/near-degenerate evidence before claiming every
  operation's boundary domain is complete; and
- decide whether the current `1e-3` inverse residual survives the expanded
  conditioning corpus.
