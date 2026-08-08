# Alternative D Bounded Status Report

| Field | Value |
| --- | --- |
| Status | Rejected for expansion; retain only the `Vec3` provenance case study |
| Scope | Derived scalar `Vec3`; no matrix, quaternion, Vec2, Vec4, SIMD, or generated slice |
| Date | 2026-08-08 |

## Evidence By Category

| Category | Current evidence | Result / limit |
| --- | --- | --- |
| Provenance | `COPY-MANIFEST.md` now maps every local line range to corpus scaffolding or named pinned `glam` anchors. | Reviewable bounded derivation; it must never be presented as original Tokimu source. |
| Correctness | The shared native and Node-WASM conformance suite exercises D's admitted Vec3 arithmetic/normalization case successfully. | Valid only for D's Vec3 slice; it cannot satisfy matrix caller cases. |
| Runtime performance | No D-specific workload exists because retained transform, stereo, upload, and renderer workloads require `Mat4`. | Unmeasured, not assumed equivalent to C or A. |
| Layout | Native `size_of::<Vec3>()` / `align_of::<Vec3>()` observation is 12 / 4 bytes, matching A/B/C Vec3 on the observed host. | Not an ABI, FFI, SIMD, GPU, or serialization promise. |
| Size / source | The retained source-surface observation records one Rust file, 139 nonblank lines, and 4,064 bytes at its measurement point. | Small code size excludes required provenance and future update-review work. |
| Compile / target | D compiles in the full native and `wasm32-unknown-unknown` study targets; its shared Vec3 case executes in Node WASM. | Does not demonstrate isolated-provider independence beyond its no-dependency source shape. |
| Unsafe / SIMD | The source, manifest, and notice retain no unsafe block, architecture intrinsic, generated source, or SIMD backend. | Any future addition needs individual invariant and target review. |
| Update burden | Every expansion or relevant upstream fix requires a new pinned source selection, line ledger, attribution/license review, local-change record, and target validation. | Higher lineage burden than C without a demonstrated advantage. |

## Disposition

Alternative D is a useful provenance-control specimen, but is **rejected for
expansion at this time**. It is neither a maintainable general fork nor a
one-time extraction for the current matrix-driven callers: C tests the same
ownership hypothesis without creating upstream-diff and incorporation work.

Retain D's small Vec3 derivation and documents as a case study. Reopen only
for a measured C deficit or a concrete upstream-compatibility requirement that
outweighs D's provenance/update burden.
