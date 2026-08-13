# AR-0019 Option C Second-Stage Control

| Field | Value |
| --- | --- |
| Status | Slice 0 retained control; no production migration authorized |
| Date | 2026-08-12 |
| Repository revision | `c84108cd2eabe2dbe13b658f4f493f996ca33d74` |
| Rust | `rustc 1.95.0 (59807616e 2026-04-14)` |
| Cargo | `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` |
| Host | `x86_64-pc-windows-msvc` |
| Plan | `docs/Plans/Native-Math/Studies/ar-0019-option-c-owned-math-and-bulk-compute.md` |

## Alternative A Control

The production control remains the path-pinned `glam` package at:

```text
version:  0.29.3
revision: d36e7eeff05338c56c4aa8d59fc2615e7963b1b7
path:     third-party/ring-0/glam
features: default-features = false; std = enabled
status:   clean submodule worktree
```

`cargo tree -p tokimu-core -e features --offline` reports only the selected
`std` feature for this dependency path. This describes the current feature
selection; it is not a claim that unused source inside the checked-out package
cannot affect audit or maintenance work.

The current compiler emits repeated `unused_attributes` warnings because
0.29.3 places `#[must_use]` on trait methods in implementation blocks. AR-0019
Cycle 53 records the upstream repair commit
`d5d92e48d628f2232295770c4a7b909e4b81c150`, first released in 0.30.2. The
comparison from the audited revision to release commit
`73e8582703ea1790dd41d0faca3df8beda4730a3` changes approximately 170 files,
with 29,090 insertions and 10,156 deletions. Those counts describe lifecycle
review surface; they do not imply that every changed line is security-sensitive
or that 0.29.3 has a demonstrated numerical defect.

The isolated A control is `baseline-a`. It imports the actual stable
`tokimu_core::math` vocabulary rather than declaring a second provider alias.
Its source is 75 lines / 2,798 bytes with one local test.

## Alternative C0 Control

C0 continues to reuse the existing original implementation:

```text
shared source:
  corpus/lib/tokimu-math-study/src/alternative_c.rs

isolated boundary:
  corpus/lib/tokimu-math-study/alternative-c-owned-subset/src/lib.rs

implemented values:
  Vec3, Vec4, Mat4

intentionally absent:
  Vec2, Quat
```

The isolated crate includes the shared source with a `#[path]` module. It does
not copy the implementation and its manifest has no dependencies. This is the
only Option C implementation authorized by the second-stage plan.

Current source observations:

| Surface | Lines | Nonblank | Bytes | Local tests | Unsafe mentions |
| --- | ---: | ---: | ---: | ---: | ---: |
| Shared C0 implementation | 581 | 503 | 16,047 | 3 | 0 |
| Isolated boundary/probes | 83 | 73 | 2,873 | 1 | 0 |

C0 currently covers the bounded operation manifest retained in
`operation-inventory.md`: vector construction/arithmetic and geometric
mechanics, camera/projection constructors, affine transforms, matrix
composition, transpose/inverse, column-array handoff, and explicit final-column
mutation. Singular inverse and degenerate/non-finite behavior remain
observations, not selected Tokimu contracts.

## Reproduction And Hashes

The following commands succeeded with network resolution disabled:

```powershell
cargo test --manifest-path corpus/lib/tokimu-math-study/baseline-a/Cargo.toml --locked --offline
cargo test --manifest-path corpus/lib/tokimu-math-study/alternative-c-owned-subset/Cargo.toml --locked --offline
cargo build --manifest-path corpus/lib/tokimu-math-study/alternative-c-owned-subset/Cargo.toml --target wasm32-unknown-unknown --locked --offline
```

Observed results:

- A: one isolated control test passed; the known provider warning flood was
  reproduced.
- C0 native: four tests passed, including the three tests in the reused shared
  implementation and the isolated-boundary test.
- C0 WASM: `wasm32-unknown-unknown` development build succeeded. This is
  compile-only evidence; the inherited 2026-08-08 records retain actual WASM
  engine execution evidence.
- The only additional C0 messages concerned the Windows filesystem declining
  hard links in Cargo's incremental cache and are not candidate source
  warnings.

Control hashes:

```text
f8e1b6f7b9b2e411cc037b4d97c06cda062a6dc5bdd80dc1768e877634e26cd2  Cargo.toml
3bd0e1792dd0bc3ecd1b0b9152d191b8f61040c842e4edafefac87639ca0d042  Cargo.lock
963e2bb98e682816d3a5777da72a0a0843a9c9a833a580bc311f606a8f1a4ff0  baseline-a/Cargo.toml
6c737c8ee7294adca4e9528d4e7519e53fdb38f9544f8d7f27f94fb7d8eb2ca3  baseline-a/Cargo.lock
787810587e43fdcbd9bbb649eee9c7e1d7421a14f6a4f32417e512cad3fdf189  src/alternative_c.rs
aae17d2b21e8c8ec7552da3b864ed4e34b925cd9bf02ad0e397f52cb5b1e15f1  alternative-c-owned-subset/src/lib.rs
cfd09db9cc05da679d831d10592fa36cad600d4cbb46651b7abb9567fedb02f9  alternative-c-owned-subset/Cargo.toml
2ab064e0189643f07685532ff388435c68ff5a22f9afbc632ec36f15c0c227a8  alternative-c-owned-subset/Cargo.lock
```

Paths after the study root are abbreviated in the hash list. Hashes identify
this control snapshot; they are not signatures or provenance proof.

## Control Disposition

- A remains the production and oracle control.
- C0 remains corpus-local and conditionally viable.
- No dependency, stable public type, ADR disposition, or production source was
  changed to establish this control.
- The next authorized work is the post-DOOM operation and boundary rescan. It
  may shrink or refine C0; it does not automatically expand it.
