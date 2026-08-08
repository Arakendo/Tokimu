# Stereo Camera Boundary Observation

## Scope

This is a release-mode native-host observation of complete construction of the
bounded `hello-3d-stereo` camera pair. Each iteration constructs two orbit
views and two half-width projections, then forms the current provider-valued
`tokimu::Camera` values. It includes B/C's explicit renderer boundary work.

It is not a portable benchmark, a frame-time budget, or a selection result. It
does not measure WGPU upload, draw submission, browser execution, or the
post-DOOM caller set.

## Environment

| Field | Value |
| --- | --- |
| Date | 2026-08-08 |
| Host target | `x86_64-pc-windows-msvc` |
| Compiler | `rustc 1.95.0 (59807616e 2026-04-14)` |
| Profile | Cargo `--release` |
| Iterations per sample | 100,000 complete stereo camera pairs |
| Samples per alternative | 9, rotated A/B/C order |

## Commands

```text
cargo run -p tokimu-math-study --bin measure_stereo_camera_allocations --release --locked --offline -- 100000
cargo run -p tokimu-math-study --bin measure_stereo_camera_workload --release --locked --offline -- 100000 9
```

## Retained Results

| Alternative | Minimum | Median | Maximum | Allocation count |
| --- | ---: | ---: | ---: | ---: |
| A: direct provider | 6,663,500 ns | 7,323,000 ns | 8,045,500 ns | 0 |
| B: private provider-backed vocabulary | 6,439,500 ns | 6,662,900 ns | 7,773,300 ns | 0 |
| C: owned scalar subset | 9,160,800 ns | 9,246,100 ns | 9,789,000 ns | 0 |

All candidates retained checksum `172113`. C's median was approximately 26%
higher than A on this host; B's median was approximately 9% lower. These
differences are observations of this tiny workload and toolchain, not claims
about general transform, renderer, or WASM performance.

## Interpretation

- The B/C renderer crossings are allocation-free in this representative path.
- The C result is a performance finding that must remain visible in any
  owned-implementation decision; its source independence does not make the
  cost irrelevant.
- The result requires target-specific native/WASM repetition before any gate
  conclusion under ADR-0008. The DOOM-plan revisit remains a selection blocker.
