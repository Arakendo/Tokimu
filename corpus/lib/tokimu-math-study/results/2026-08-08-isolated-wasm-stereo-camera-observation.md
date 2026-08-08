# Isolated WASM Stereo-Camera Math Observation

## Scope

This is a Node WebAssembly-engine observation of the isolated A/B/C candidate
crates. Each exported function repeatedly performs the bounded
`hello-3d-stereo` math path and extracts the two composed matrices' columns.

It exercises candidate math and column-array representation work. It does not
construct `tokimu::Camera`, run WGPU, upload bytes, or measure browser frame
behavior; the native stereo-camera observation remains the evidence for the
current public renderer facade.

## Environment and Command

| Field | Value |
| --- | --- |
| Date | 2026-08-08 |
| Build target | `wasm32-unknown-unknown` |
| Build profile | Cargo `--release` / `--locked` / `--offline` |
| Execution engine | Node.js `v22.21.0` WebAssembly engine |
| Iterations per sample | 100,000 |
| Samples per alternative | 9, rotated A/B/C order |

The crates were built individually for `wasm32-unknown-unknown`, then their
`tokimu_math_study_wasm_stereo_camera_probe` exports were instantiated and
timed with `process.hrtime.bigint()` in one Node process.

## Retained Results

| Alternative | Minimum | Median | Maximum | Raw WASM bytes | Checksum |
| --- | ---: | ---: | ---: | ---: | ---: |
| A: direct provider | 9,799,300 ns | 9,838,900 ns | 9,882,300 ns | 11,738 | 172113 |
| B: provider-backed vocabulary | 9,309,500 ns | 9,355,600 ns | 9,958,000 ns | 11,878 | 172113 |
| C: owned scalar subset | 9,317,100 ns | 9,351,600 ns | 9,655,300 ns | 12,843 | 172113 |

For this isolated WASM probe, B and C medians were approximately 5% below A
and effectively equal to each other. C's raw output was 1,105 bytes above A;
these are uncompressed micros-only artifacts, not application binary sizes.

## Interpretation

- This target observation does not reproduce the native host's C slowdown in
  the full public-camera workload. The two measurements have different
  execution engines and boundary scope, so neither can be generalized to the
  other.
- C's current scalar implementation has a larger isolated raw WASM output in
  this probe; output size and runtime remain separate tradeoffs.
- A browser/WGPU and post-DOOM representative workload are still required
  before an ADR-0008 performance disposition or alternative selection.
