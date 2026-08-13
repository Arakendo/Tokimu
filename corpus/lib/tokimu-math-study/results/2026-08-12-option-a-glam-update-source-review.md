# Option A `glam` 0.33.3 Source Review Intake

| Field | Value |
| --- | --- |
| Status | Selected closure, package mechanisms, and unsafe/SIMD change history reviewed; broader semantic/target evidence remains open |
| Parent | 0.29.3 at `d36e7eeff05338c56c4aa8d59fc2615e7963b1b7` |
| Candidate | 0.33.3 at `9928729066db87d97fa779e129469721a289beae` |
| Candidate tree | `1f593dbd530fddf83d501b946be3e67e3837246f` |
| Candidate parent | `17e32a32cd4abe30df4488b05d9feabc2d73fc1a` |
| Annotated tag object | `fae5594033edc8cf0f385a1bcbbab5205eabe7df` |
| Crate SHA-256 | `7360BD2CD76E0CD9032D42CF2922155CECEA2685B0CFA4630C3246DF030BCFD6` |
| License | MIT OR Apache-2.0 |
| Declared MSRV | Rust 1.68.2 |

## Identity reconciliation

The live crates.io query reported 0.33.3 as current on 2026-08-12. The GitHub
annotated release tag peels to the candidate commit, and the registry package's
`.cargo_vcs_info.json` names that same commit. The release commit is titled
`Prepare 0.33.3 release (#764)` and is dated 2026-08-03.

All 202 Git blobs below `src/` were compared to the expanded crates.io payload
using Git blob hashing:

```text
checked:   202
missing:     0
mismatch:    0
```

This proves the packaged Rust source corresponds byte-for-byte to the candidate
commit for `src/`. Package metadata and non-source inclusions remain separately
reviewable; the crate archive checksum above identifies the exact downloaded
payload.

## Delta scale

The raw upstream commit range contains:

```text
290 files changed
115,332 insertions
31,834 deletions
176 changed entries under src/
```

This is not a small patch review. Much of the volume is generated swizzle code,
file moves, newly optional type families, tests, code generation, tools, and
documentation, but generated source remains part of the Ring 0 review burden.

Initial lexical inventories over Rust source found:

| Inventory | 0.29.3 | 0.33.3 | Intake interpretation |
| --- | ---: | ---: | --- |
| Lines containing `unsafe` | 971 | 903 | count decreased, but paths and implementations changed |
| Files containing `unsafe` | 84 | 91 | increased, partly due to new type families/optional integration and WASM path moves |
| Intrinsic/target-feature/arch lines | 81 | 100 | increased and requires selected-path review |
| Intrinsic/target-feature/arch files | 37 | 39 | increased |

The `wasm32` source hierarchy was substantially renamed to `wasm`; raw path
counts therefore overstate implementation novelty. New unsafe-bearing paths
also include optional `encase`, `isize`, and `usize` families which Tokimu must
prove are not selected by `std` alone. These facts narrow manual review; they do
not complete it.

## Release-note findings relevant to Tokimu

- 0.30.0 changed `look_to_lh`/`look_to_rh` to require normalized direction and
  up inputs. Repository search found no production `look_to_*` call sites;
  current callers predominantly use `look_at_rh`.
- 0.30.2 changed scalar vector `min`/`max` behavior to agree with SIMD paths,
  including different NaN propagation from primitive scalar `min`/`max`.
  Differential and degenerate controls must cover this before admission.
- 0.30.4 fixed unsound core-SIMD `Vec3A` conversions. Core-SIMD is optional,
  but the affected source still belongs in the upstream delta review.
- 0.31.0 changed `Quat::from_affine3`, removed `Vec2`/`DVec2::angle_between`,
  and changed some receiver forms. No current Tokimu use of the named breaking
  APIs was found in the first call-site search.
- 0.31.1 added wasm64 support and changed SIMD constant naming.
- 0.32.1 fixed scalar/matrix division behavior.
- 0.33.0 made non-f32 and boolean type families optional while retaining them
  in upstream defaults. Tokimu disables defaults and selects only `std`; the
  five f32 public types must remain available without enabling `all-types`.
- 0.33.2 added camera/projection modules. Their presence does not admit those
  APIs or semantics into Tokimu.

## Resolved selected closure

The isolated candidate manifest retains `default-features = false, features =
["std"]`. Cargo metadata, the lockfile, dependency-tree inspection, and the
Ring 0 audit agree on one local package node:

```text
tokimu-core
└── glam 0.33.3 (local path/submodule)
    └── feature: std
```

The upstream default changed to `std + all-types`, but Tokimu disables it.
`std` remains an empty feature. The new f64, integer, size, serialization,
random, archive, interoperability, and core-SIMD dependency edges are optional
and absent from the resolved Tokimu closure. The always-available f32 and
boolean families retain the five types Tokimu re-exports.

## Categorized source manifest

The 176 changed `src/` entries were classified by path and their relationship
to the exact `std`-only build:

| Category | Changed entries | Review disposition |
| --- | ---: | --- |
| selected or unconditional f32/bool/camera/support source | 116 name-status entries; 67 numstat entries after rename pairing | inside manual review |
| unselected non-f32 type families and double-camera source | 49 | closure exclusion proved; retained in whole-delta trust burden |
| unselected optional feature integrations | 10 | closure exclusion proved; manifests and cfg gates inspected |
| other support source | 1 | inspect with unconditional support |

The selected/unconditional category covers `src/f32`, `src/bool`, the new
`src/camera`, selected swizzles/WASM paths, and root support modules used by the
f32 build. The larger name-status count reflects upstream path moves, notably
`wasm32` to `wasm`; it is not treated as 116 independent implementations.

The selected-path semantic changes identified so far are:

- vector `min`/`max` intentionally align scalar NaN behavior with SIMD;
- scalar/matrix division is corrected and reciprocal helpers are added;
- quaternion small-angle interpolation returns the target rather than silently
  omitting the requested rotation;
- new inverse alternatives and angle/rotation helpers add APIs without
  changing Tokimu's admitted vocabulary;
- camera/view/projection construction moves into the new module reviewed by
  AR-0029; and
- wasm64 support generalizes and renames existing WASM implementations.

## Build and host behavior triage

The candidate tree contains no `build.rs`, proc-macro target, native/prebuilt
library, object, or WASM binary. The exact selected source contains no matches
for filesystem, network, process, environment, threading, heap-collection,
lock, mutable-global, FFI, or inline-assembly entry points. Code generation is
an upstream development activity represented by checked-in templates and
generated Rust; it does not execute during Tokimu's Cargo build. The repository
does contain development-only templates, tests, benches, and tools, so archive
and non-source layout review remains part of the package audit rather than
being mistaken for runtime closure.

The checked-in generated source was reproduced with the candidate repository's
exact nested `tools/codegen` revision
`673ed2c712d0c2db35fed00a21da7f132ab3cd7f`. Running
`cargo run --release -p codegen -- --check '**'` completed without changing the
candidate tree. Reproduction acquired and built a separate 175-package
development-tool closure; that closure is review/tooling burden and is not part
of Tokimu's selected Ring 0 runtime closure.

## Unsafe and target-path triage

Across selected/unconditional paths, the raw diff contains 53 added and 49
removed lines mentioning `unsafe`, spread across 19 f32 implementation files.
The additions inspected so far are representation casts/dereference helpers,
SIMD lane conversions, and generated WASM/affine equivalents; the wasm64 path
move accounts for several apparent delete/add pairs. No new FFI or inline
assembly boundary was found. This is triage, not a completed proof: each
selected representation and SIMD implementation remains on the manual review
checklist.

`git log -G unsafe` narrows the selected-path change history to ten commits.
Two explicitly remove known soundness hazards: core-SIMD `Vec3A` conversion and
tuple-layout conversion assumptions. Four are predominantly mechanical
receiver/reference, release-generation, or path-move changes. The remaining
material cases are:

- a new `Affine3` uses `repr(C)` and dereferences its `Mat3 + Vec3` storage as
  four `Vec3` columns; it is not part of Tokimu's public five-type vocabulary,
  but it is compiled source and remains a manual layout-proof item;
- SIMD quaternion `to_array` uses a transparent-layout pointer cast to become
  `const`; scalar code remains field-based;
- `is_negative_mask` adds architecture-specific sign-bit SIMD operations; and
- scalar/matrix reciprocal work adds or moves architecture-specific intrinsics
  while fixing the operand order.

This classification is evidence against treating all 102 unsafe-diff lines as
new authority, but it does not waive the novel `Affine3`, const-cast, or SIMD
paths.

Manual review of the material selected-path cases found:

- `Affine3`'s dereference cast is between two four-column layouts whose
  components are all `repr(C)` `Vec3` values. `Mat3` contributes the first
  three columns and `translation` the fourth; the source and target are both
  48 bytes with 4-byte alignment on the selected native build. The type is
  compiled but is not re-exported by Tokimu.
- SIMD `Quat::to_array` copies from either a `repr(transparent)` 128-bit SIMD
  value or the scalar `repr(C)` four-float representation into `[f32; 4]`.
  The cast does not create a longer-lived reference. The native layout
  observer continues to report 16-byte size and alignment for Tokimu's `Quat`.
- `is_negative_mask` compares the integer interpretation of each SIMD lane
  with zero. It performs no memory access and deliberately classifies `-0.0`,
  negative infinity, and negative-sign NaNs by their sign bit, matching the
  scalar implementation.
- the scalar/matrix division repair delegates to per-column scalar/vector
  division; its SSE2, NEON, WASM, and scalar variants add no new raw memory
  operation. A Tokimu regression now checks operand order over all 16 `Mat4`
  elements.
- the SSE2 quaternion interpolation change moves the existing intrinsic
  sequence into a shared helper before adding `slerp_long`; it does not add a
  new memory or host boundary.
- the core-SIMD fix replaces a pointer into a temporary array with a pointer
  into the live SIMD value, and the tuple-conversion fix removes assumptions
  about Rust tuple layout across core-SIMD, SSE2, NEON, and WASM paths.

Five other `unsafe`-matching commits are confined to unselected `rkyv`,
`bytemuck`, `spirv`, or `encase` integration. Receiver/reference and release
regeneration commits account for the remaining mechanical churn. This closes
the change-history/source inspection, but not target execution: NEON and
SIMD-enabled WASM remain explicit validation gaps rather than being inferred
from the AMD/x86-64 host.

## Panic, cfg, and packaged-content review

The added selected-source panic-like calls are overwhelmingly
feature-controlled `glam_assert!` preconditions for normalized directions,
non-zero vectors and determinants, and legal projection depth ranges. Tokimu
does not select `glam-assert`; these do not create an unconditional production
panic path. The only new unconditional `assert!` calls found are bounds checks
on generated boolean-vector indexing. AR-0029's checked constructors validate
the camera/projection cases before entering the private provider boundary.

Target/feature changes separate type-family selection, add wasm64 support, and
rename the prior `wasm32` implementation hierarchy to `wasm`. The selected
feature remains empty `std`; `core-simd`, SPIR-V, serialization/interoperation,
and non-f32 families remain outside the resolved closure. Native x86-64 SSE2
and compile-only wasm32 evidence exist. NEON, wasm64, and SIMD-enabled browser
WASM execution remain named target gaps.

`cargo package --list --allow-dirty` reports 287 packaged paths: 202 `src`, 27
tests, 18 templates, 14 benches, 8 GitHub workflow/template files, and 18 root
metadata/documentation files. The package carries both license texts,
attribution, changelog, lock/manifest files, and checked-in templates. It does
not package the `tools/codegen` executable or nested generator source, so exact
generation requires the separately pinned upstream repository checkout rather
than the crates.io payload alone. None of the non-source package content is
selected by Tokimu's Cargo runtime build.

## Security intake

On 2026-08-12, a refreshed search of the official RustSec advisory database for
the `glam` crate returned no named advisory result, and the upstream release
notes instead identify the two soundness fixes above. A workspace-local
`cargo-audit` 0.22.2 installation then scanned the production and candidate
lockfiles against the same freshly fetched 1,216-advisory database snapshot.

Both scans produced the same non-zero result:

- `RUSTSEC-2026-0194` and `RUSTSEC-2026-0195` affect `quick-xml` 0.39.4;
- `RUSTSEC-2024-0436` reports unmaintained `paste` 1.0.15;
- `RUSTSEC-2026-0192` reports unmaintained `ttf-parser` 0.25.1; and
- `RUSTSEC-2026-0253` and `RUSTSEC-2026-0002` report unsoundness warnings for
  `lru` 0.12.5.

No finding names `glam`, and the advisory sets are identical across the two
locks. Dependency inversion locates `quick-xml` beneath corpus `xml-tools`,
`paste` and `lru` beneath the Ratatui corpus provider, and `ttf-parser` beneath
`ui-tools`/font parsing. None enters the audited Ring 0 closure. The scan is
therefore clean with respect to the `glam` revision, while the broader
workspace security baseline remains explicitly non-green and requires
separate remediation rather than an Option A suppression.

## License and attribution reconciliation

The candidate continues to declare `MIT OR Apache-2.0`. Its `LICENSE-MIT` and
`LICENSE-APACHE` files are byte-identical to the audited 0.29.3 parent; Git
reports no license-file change across the revision range. Both files remain in
the exact 287-path package payload. The candidate changes `Cargo.toml` and
`README.md`, but no new license, notice, patent, attribution, redistribution,
or bundled-binary obligation was identified. Existing Tokimu attribution and
source-initialization obligations therefore remain applicable without
expansion, subject to the final revision-specific audit.

## Open review work

- complete direct old/new edge-semantic and representative real-caller
  observations beyond the passing retained conformance and plain-WASM controls;
- execute the browser-ready candidate fixture on an actual browser; automated
  control is presently blocked by the host's Node 22.21 runtime while the
  installed browser harness requires at least 22.22;
- determine whether the warning cleanup and other benefits justify the full
  recurring review burden.

## Integration finding

An isolated candidate compile proved the `std`-only closure and cleared the old
generated-swizzle warnings, but 0.33.3 deprecates the `Mat4` camera/projection
constructors used throughout Tokimu. Upstream recommends new
`glam::camera::*` functions. Adopting that advice would exceed this plan's
five-type foreign-vocabulary scope, while suppressing the warnings is expressly
disallowed. See `2026-08-12-option-a-glam-update-validation.md`.
