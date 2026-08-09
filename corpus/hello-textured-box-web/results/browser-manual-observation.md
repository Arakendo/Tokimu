# Textured Box Browser Manual Observation

| Field | Value |
| --- | --- |
| Date | 2026-08-09 |
| Target | Browser/WASM WebGPU fixture |
| Observer | Project maintainer |
| Consumer | `hello-textured-box-web` |
| Result | First textured frame rendered; interactive comparison controls confirmed |

## Observed

The browser fixture rendered the pinned Khronos Box with the independent
first-party grid PNG through the browser/WASM `Textured3d` path. This is
browser presentation evidence for the scoped composition: GLB geometry decode,
corpus-owned supplied UVs, PNG normalization, texture upload, material binding,
and WGPU surface presentation.

It is not evidence of browser/native pixel equivalence, imported GLB material
semantics, PNG decoder conformance, cutout alpha, or a complete browser input
contract.

## Interactive Observation

After the browser controls were added, the project maintainer reported that the
fixture worked correctly. The `M`, `R`, and `X` controls (and matching buttons)
change the declared texture, sampler, and UV state and redraw the presented
scene. This completes the browser-side interaction observation required by the
textured-Box study.

This observation confirms control-driven composition, not pixel-identical
native/browser output or a claim that every filtering difference is visibly
distinct on every adapter.
