# Browser Slice 3 Blend Observation — 2026-08-09

## Capture Identity

| Field | Value |
| --- | --- |
| Target | browser/WASM canvas |
| Backend | browser WebGPU |
| Device kind | `other` |
| Adapter | unavailable from the browser provider |
| Viewport | 960 × 600 |
| Host query | `?mode=blend` |
| Fixture sources | manifest-locked `continuous-gradient` and `mixed-alpha` exact RGBA8 arrays |
| Blend equation | straight source-alpha `AlphaBlend` |
| Declared depth state | `LessEqual`; comparison panels explicitly select depth write on or off |
| Camera convention | Tokimu GL-style `[-1, 1]`, converted by the WGPU provider to `[0, 1]` |

This is a maintainer-supplied browser visual observation, not a pixel-golden,
browser-vendor, or adapter-vendor guarantee.

## Observed Result

After the static-fixture refinement, the browser page reported:

```text
ready | blend first + warm frame presented |
first=12 draws/12 materials/6 pipelines/13 binding allocations/0 mesh uploads |
warm=12 draws/12 materials/6 pipelines/0 binding allocations/0 mesh uploads |
diagnostic=none | backend=browser-webgpu | device=other | adapter=? |
viewport=960x600
```

The upper-left continuous-gradient-over-opaque control visibly progressed
through the retained source alpha range. The upper `far → near` and `near →
far` mixed-alpha panels visibly differed, while retaining the same declared
source, geometry, UVs, transforms, camera, and blend equation. The lower
depth-write-off and depth-write-on panels also visibly differed under the
frozen near-to-far sequence.

The page labels retained the caller-order and depth-write intent beside the
image. `diagnostic=none` is expected for this valid comparison; caller order
is corpus input evidence, not a WGPU validation diagnostic. The unchanged warm
frame reused renderer-owned bindings and static mesh resources; material
resolution and pipeline selection remain per-draw work for the current
submission implementation.

## Interpretation Boundary

This establishes browser/WASM presentation for the bounded Slice 3 comparison
on one browser WebGPU path. It does not establish NVIDIA behavior, a general
transparent sorting policy, renderer-owned ordering, an automated pixel
comparison, batching, a WGPU bind-group contract, or a stable public blend
contract.
