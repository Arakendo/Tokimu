# Option C0 Inverse Isolation And C1 Affine Prototype

| Field | Value |
| --- | --- |
| Date | 2026-08-12 |
| Status | Slice 5 evidence; C1 is corpus-only and partial |
| Target | `x86_64-pc-windows-msvc` |
| Profile | Cargo `--release --locked --offline` |
| Toolchain | Rust/Cargo 1.95.0 |
| Host identity | Not retained; timings are not cross-host benchmarks |
| Candidates | A direct provider, B private provider-backed vocabulary, C0 scalar reference, C1 scalar affine fast path |

## Why This Control Exists

The first retained caller-shaped observation found a material C0 deficit in two
real corpus paths:

| Path | A median | C0 median | C0 disposition |
| --- | ---: | ---: | --- |
| CAD cursor ray, 100,000 calls | 2,977,700 ns | 11,883,500 ns | material regression |
| Pinned Khronos Box model + floor transforms, 100,000 calls | 34,688,700 ns | 48,514,200 ns | material regression |

Both repeat `Mat4::inverse()`. `measure_inverse_workload` therefore holds one
finite, well-conditioned affine matrix constant and measures only its inverse.
It is an operator isolate, not an engine benchmark.

```powershell
cargo run -p tokimu-math-study --release --bin measure_inverse_workload --locked --offline -- 1000000 15
```

## C0 Operator Isolation

| Candidate | Median, 1,000,000 affine inverses |
| --- | ---: |
| A direct provider | 8,986,500 ns |
| B private provider-backed | 10,598,700 ns |
| C0 generic Gauss--Jordan | 93,176,400 ns |

C0 is about 10.4 times A on this bounded control. The regression is therefore
not an upload conversion or an allocation effect. C0's general pivoting
Gauss--Jordan algorithm remains useful as a portable checked/reference route,
but it is not an acceptable repeated affine inverse mechanism for the observed
caller pressure.

## C1: Small Safe Affine Fast Path

C1 changes only the corpus candidate's unchecked `Mat4::inverse()` route:

1. Exact affine matrices produced by the existing constructors use a direct
   inverse of their 3x3 linear block and transformed translation.
2. Singular/non-finite affine inputs do not use the fast result.
3. All non-affine matrices retain the C0 Gauss--Jordan route.
4. `try_inverse()` remains on the checked C0 reference route.

No `unsafe`, target feature, SIMD intrinsic, provider call, new public type, or
production caller was introduced. The implementation is still scalar safe Rust.

The candidate has 128 deterministic affine comparisons against its retained
scalar reference plus a non-affine projection fallback regression. The complete
library suite has 55 passing tests.

After C1:

| Candidate | Median, 1,000,000 affine inverses |
| --- | ---: |
| A direct provider | 9,074,500 ns |
| B private provider-backed | 10,584,900 ns |
| C1 owned scalar | 14,182,200 ns |

C1 remains about 56% above A on this narrow operator control, but removes the
order-of-magnitude C0 deficit without adding foreign execution or unsafe code.

## Replayed Caller Paths After C1

```powershell
cargo run -p tokimu-math-study --release --bin measure_caller_paths --locked --offline -- 100000 15
```

The GLB control decodes the pinned Khronos Box once before timing and executes
the real retained model/floor transformations across its 24 positions/normals.
It excludes decoding and renderer submission. CAD retains the real cursor-ray
path.

| Path | A median | B median | C1 median | C1 result |
| --- | ---: | ---: | ---: | --- |
| CAD cursor ray | 2,970,300 ns | 2,957,500 ns | 12,620,800 ns | still material regression |
| GLB model + floor | 34,429,700 ns | 37,217,900 ns | 33,707,600 ns | bounded control recovered |

The CAD matrix is projection × view and is non-affine, so C1 deliberately uses
the retained general reference route there. This is a useful boundary: C1 is a
validated repair for ordinary affine import/scene transforms, not a claim that
generic projection inversion is solved.

## E1M1 Source-Observer Camera Preparation

`migration_hello_doom_observer` ports the current PreserveNorth source lift,
observer-direction construction, `look_at_rh`, and perspective preparation.
The source embedding remains caller semantics; the comparison does not claim
that the math candidate owns Doom's heading or frame meaning.

```powershell
cargo run -p tokimu-math-study --release --bin measure_doom_observer_path --locked --offline -- 1000000 15
```

| Candidate | Median, 1,000,000 observer preparations |
| --- | ---: |
| A direct provider | 45,339,200 ns |
| B private provider-backed | 44,884,200 ns |
| C1 owned scalar | 51,902,600 ns |

All candidates retained the same f64 checksum. C1 is about 14.5% above A on
this one host/control. The path does not invert the combined projection-view
matrix, so the remaining C1 CAD deficit is not being hidden by this result.
It is a real Doom camera-mechanics observation, not a complete E1M1 frame,
visibility, collision, or renderer benchmark.

The repeated f64 checksums were deterministic for each candidate. Their small
cross-candidate differences remain ordinary repeated-f32 accumulation effects;
the migration conformance suite continues to compare each visible result under
the selected per-operation tolerance.

## Current Disposition

- A repeated affine inverse is a decision-relevant C0 regression.
- C1 safely recovers the real imported-scene control, while retaining C0 as the
  non-affine/checked scalar reference.
- CAD/picking remains open: a general non-affine inverse strategy requires its
  own numerical and maintenance evidence before C1 can be described as a
  complete Option C performance response.
- E1M1 observer-camera preparation is bounded and moderately slower than A;
  collision and complete observer/renderer replay remain separate open work.
- Nothing here changes ADR-0019, admits C to Ring 0, or authorizes migration of
  production callers.
