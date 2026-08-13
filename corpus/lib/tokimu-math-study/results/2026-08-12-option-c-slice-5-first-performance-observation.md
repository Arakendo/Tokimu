# Option C0 Slice 5 First Caller-Shaped Performance Observation

| Field | Value |
| --- | --- |
| Date | 2026-08-12 |
| Status | First native-host performance/build observation; Slice 5 remains open |
| Target | `x86_64-pc-windows-msvc` |
| Profile | Cargo `--release --locked --offline` |
| Toolchain | Rust/Cargo 1.95.0; LLVM version not retained by this invocation |
| Host identity | Not retained; do not compare these timings to another host as a benchmark |
| Candidates | A direct provider, B private provider-backed vocabulary, C0 owned scalar candidate |

## Workload Separation

The measurements intentionally distinguish complete caller-shaped work from a
small conversion loop:

- **Repeated transform**: builds one rotation/translation transform, then
  transforms and accumulates one point per iteration.  It is a caller-shaped
  transform loop, but not a whole Doom, collision, import, or renderer frame.
- **Stereo camera**: reproduces `hello-3d-stereo`'s two view/projection
  constructions and the existing private renderer-camera handoff per
  iteration.  It is the most complete current A/B/C caller comparison.
- **Upload conversion**: converts a pre-built candidate matrix to the current
  provider value repeatedly.  It measures the explicit representation boundary
  only; it is not a renderer upload or draw submission.

Every timed A/B/C caller-shaped runner first checked an equal visible checksum
and rotated candidate order between samples.  These are descriptive host
observations, not universal performance claims.

## Repeated Transform

Command:

```powershell
cargo run -p tokimu-math-study --release --bin measure_transform_workload --locked --offline -- 1000000 15
cargo run -p tokimu-math-study --release --bin measure_transform_allocations --locked --offline -- 1000000
```

| Candidate | Minimum | Median | Maximum | Allocation calls |
| --- | ---: | ---: | ---: | ---: |
| A direct provider | 3,201,400 ns | 3,972,200 ns | 4,082,200 ns | 0 |
| B private provider-backed | 3,326,800 ns | 3,980,900 ns | 4,045,900 ns | 0 |
| C0 owned scalar | 2,862,800 ns | 3,512,700 ns | 3,623,100 ns | 0 |

All candidates retained `[299659.72, -996994300.0, -101670.57]`.  On this
specific process C0's median was about 11.6% below A; the result does not show
that this will hold for vectorized provider builds, another CPU, or a complete
engine workload.

## Stereo Camera And Renderer Boundary

Command:

```powershell
cargo run -p tokimu-math-study --release --bin measure_stereo_camera_workload --locked --offline -- 100000 15
cargo run -p tokimu-math-study --release --bin measure_stereo_camera_allocations --locked --offline -- 100000
```

| Candidate | Minimum | Median | Maximum | Allocation calls |
| --- | ---: | ---: | ---: | ---: |
| A direct provider | 6,655,000 ns | 6,971,900 ns | 8,372,500 ns | 0 |
| B private provider-backed | 8,798,200 ns | 9,054,900 ns | 11,035,700 ns | 0 |
| C0 owned scalar | 8,778,600 ns | 8,966,700 ns | 11,072,100 ns | 0 |

All candidates retained checksum `172113`.  C0 is about 28.6% above A on this
complete retained path, but about 1.0% below B.  The result therefore isolates
the current cost pressure as the migration/boundary-shaped path, not evidence
that C0 itself needs SIMD, `unsafe`, or a provider delegation.  All three
paths remained allocation-free under their host-only counters.

The small boundary-only control also retained zero allocation calls for one
million conversions.  Its single sequential timing was B `1,000,100 ns` and
C0 `1,007,700 ns`; it is retained only to rule out an obvious order-of-
magnitude conversion cost, not as a throughput comparison.

## Build And Output Observations

Fresh and then immediately repeated isolated builds used distinct target
directories.  Their dependency closures are deliberately different, so the
timings describe the current isolated controls rather than an intrinsic math
compile-time advantage.

| Control | Fresh build | Immediate repeated build | Release DLL | Plain WASM module |
| --- | ---: | ---: | ---: | ---: |
| A (`baseline-a`) | 4,970.632 ms | 351.161 ms | 109,056 B | 11,738 B |
| C0 (`owned-subset`) | 1,003.202 ms | 141.553 ms | 118,272 B | 18,921 B |

The A module exports its historic transform/stereo/layout probes, while C0
exports a newer checked semantic and layout probe family; the WASM sizes are
not like-for-like and are **not** an admission-size conclusion.  The current
shared transform-only executable pair is closer in purpose: A `123,904 B`, C0
`124,416 B` (+512 B).  It still includes different link closures and cannot
predict full-engine output size.

## What This Does And Does Not Decide

- No material C0-specific deficit has been established.  C1 optimization,
  target SIMD, `unsafe`, and provider delegation remain unjustified.
- C0's scalar implementation remains the required portability/reference
  control; no optimization replaces it.
- Doom observer, collision, projection/picking, and imported-scene A/C
  comparisons have **not** been measured.  The production callers still use A,
  and silently switching them for a timing run would be a production migration,
  not a valid study measurement.  Slice 5 retains a separate corpus-replay
  refinement for those caller paths.
- No timing here changes ADR-0019's retained disposition or admits C0 to Ring
  0.
