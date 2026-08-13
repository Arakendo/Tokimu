# Option B Frozen Control

| Field | Value |
| --- | --- |
| Status | Slice 0 complete; no production migration authorized |
| Observation date | 2026-08-12 |
| Repository revision | `c84108cd2eabe2dbe13b658f4f493f996ca33d74` |
| Production A | `glam` 0.29.3 at `d36e7eeff05338c56c4aa8d59fc2615e7963b1b7` |
| Reviewed update control | `glam` 0.33.3 at `9928729066db87d97fa779e129469721a289beae`; not admitted |
| Selected provider features | local path, `default-features = false`, `std` only |
| Host | Windows x86-64 MSVC |
| Rust | 1.95.0 (`59807616e`, LLVM 22.1.2) |
| Cargo | 1.95.0 |
| Node | 22.23.2 |
| AR-0019 | Retain A; C0/C1 remains incubating corpus evidence |
| AR-0029 | Under Review |

This freezes the identities used by the Option B study. It does not alter the
production gitlink, dependency requirement, lockfile resolution, public math
exports, or renderer camera contract.

## Candidate Identities

### Narrow B

Narrow B is the isolated AR-0029 semantic-construction candidate. It owns only
three checked construction families:

- right-handed look-at;
- right-handed GL-depth perspective; and
- right-handed GL-depth orthographic projection.

The inherited 0.33.3 candidate observation retained six constructor-contract
tests and representative native/WASM compilation. It deliberately kept the
five existing public `glam` value types. Actual browser execution and the
AR-0029 maintainer decision remained open. Those limitations remain attached
to the inherited observation.

### Full B

The existing Full-B implementation remains isolated under
`alternative-b-provider-backed` and delegates privately to production A.

| Measure | Frozen value |
| --- | ---: |
| Shared wrapper source | 553 lines; 459 nonblank; 13,956 bytes |
| Literal private `glam::` references | 39 |
| Isolated facade source | 82 lines; 72 nonblank; 2,855 bytes |
| Isolated tests | 10 |
| Public wrapper names | `Vec2`, `Vec3`, `Vec4`, `Quat`, `Mat4` |
| Explicit representative matrix crossings | 9 |

The wrapper promises no C ABI, POD contract, transparent representation, or
provider layout equivalence. Its current mechanics are bounded by real caller
fixtures, except that `Vec2` and `Quat` remain intentionally minimal public-name
controls.

## Reproduction Result

The isolated Full-B suite passes all ten tests against the exact 0.29.3
provider. Compiling the private provider also reproduces the known
generated `#[must_use]` warning flood.

This is an early discriminating result:

- wrapping can hide provider vocabulary and constructor churn from callers;
- wrapping does not remove provider compilation, source audit, generated-code,
  unsafe/SIMD, target, or remediation work; and
- Full B must not claim supply-chain independence while `glam` executes inside
  the wrapper.

## Inherited Versus Rerun Evidence

| Evidence | Date retained | Current use | Limitation |
| --- | --- | --- | --- |
| Initial B correctness/layout/allocation observations | 2026-08-07 to 2026-08-08 | Starting oracle only | Predates later Doom and chart pressure |
| Nine explicit representative matrix crossings | 2026-08-08 | Migration control | Bounded corpus set, not production migration |
| Renderer camera public-vocabulary scan | 2026-08-08 | Stable-boundary control | Must be refreshed after later callers |
| Option A 0.33.3 candidate validation | 2026-08-12 | Update-shock control | Actual browser observation remains open |
| Option C second-stage evidence | 2026-08-12 | Owned-mechanics comparison | C remains incubating and unadmitted |
| Full-B isolated test suite | 2026-08-12 rerun | 10/10 current native test control | Native host only at this slice |

All caller-shaped performance, native/WASM/browser parity, update-shock,
ergonomics, binary-size, and representative-migration claims must be rerun by
their later Option B slices. Passing this control does not satisfy them.
