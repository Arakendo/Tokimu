# Option C0 Slice 4 Native/WASM Parity Observation

| Field | Value |
| --- | --- |
| Date | 2026-08-12 |
| Status | Bounded native/Node-WASM parity evidence; not browser or stable ABI evidence |
| Native target | `x86_64-pc-windows-msvc` |
| WASM target | `wasm32-unknown-unknown`, Cargo `--release --locked --offline` |
| WASM engine | Node.js `v22.21.0` WebAssembly engine |
| Candidate | Dependency-free `alternative-c-owned-subset` C0 |

## Checked Semantic Parity

The corpus-only export `tokimu_math_study_wasm_checked_probe()` returns six
bits for the currently selected C0 outcomes:

1. finite nonzero normalization produces unit length;
2. zero and non-finite checked normalization reject;
3. a caller-shaped affine inverse round trip succeeds;
4. singular inversion rejects;
5. valid checked camera/projection point projection succeeds while direct
   projection of zero has zero homogeneous `w` and rejects; and
6. degenerate view and invalid projection construction reject.

Native unit execution and the release WASM module instantiated by Node both
observed `63` (`0b11_1111`). This is a bounded semantic equivalence check, not
a claim that every C0 test, browser behavior, or renderer path has executed in
WASM.

## Scalar Representation Observation

| Target | `Vec4` size/alignment | `Mat4` size/alignment |
| --- | --- | --- |
| Native | `16 / 4` | `64 / 4` |
| Node WASM | `16 / 4` | `64 / 4` |

The retained nested-column test and the existing explicit provider handoff test
exercise copying, field access, and scalar column conversion. The current
renderer migration still needs one candidate-to-provider reconstruction for a
representative upload and two private reconstructions for its public camera
view/projection handoff; the 2026-08-12 shared `migration_c` test filter passed
all three corresponding tests.

These are observations only. They do not promise Rust/C ABI, POD, field order,
serialization, GPU buffer layout, or permanent alignment. The inherited
provider comparison still observes A's `Vec4`/`Mat4` alignment as 16 while C0
is 4 on both recorded targets; scalar matrix-array upload remains the explicit
boundary.

## Target Scope And Limits

The installed Rust targets on the recorded host are only
`x86_64-pc-windows-msvc` and `wasm32-unknown-unknown`. No additional target
could be validated without toolchain installation, so none is claimed. A scan
of `src/alternative_c.rs` found no target-specific candidate path; its sole
`cfg` is the test module. The Node runner itself is host test machinery and
does not enter the candidate.

## Reproduction

```powershell
cargo test --manifest-path corpus/lib/tokimu-math-study/alternative-c-owned-subset/Cargo.toml --locked --offline
cargo build --manifest-path corpus/lib/tokimu-math-study/alternative-c-owned-subset/Cargo.toml --target wasm32-unknown-unknown --release --locked --offline
node corpus/lib/tokimu-math-study/alternative-c-owned-subset/run_wasm_checked_probe.mjs corpus/lib/tokimu-math-study/alternative-c-owned-subset/target/wasm32-unknown-unknown/release/tokimu_math_study_owned_subset.wasm
cargo test -p tokimu-math-study --locked --offline migration_c --lib
```

The known pinned-`glam` warning flood and Cargo hard-link fallback messages are
retained separately. Neither is a C0 numerical or target-specific diagnostic.

## Disposition

The selected scalar C0 behaviors agree on the two available targets, and no
target-only candidate branch was necessary. Slice 4 does not authorize a
production migration or settle the provider-alignment difference. Slice 5 must
measure complete caller-shaped performance/cost before any optimization or
admission conclusion.
