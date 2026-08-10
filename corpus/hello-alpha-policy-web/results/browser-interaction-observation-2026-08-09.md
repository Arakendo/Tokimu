# Browser Slice 4 Interaction Observation — 2026-08-09

## Capture Identity

| Field | Value |
| --- | --- |
| Target | browser/WASM canvas |
| Backend | browser WebGPU |
| Device kind | `other` |
| Adapter | unavailable from the browser provider (`?`) |
| Viewport | 960 × 600 |
| Host query | `?mode=interaction` |
| Source build identity | `c5473d17c45145c6df378858a79141a02d083f0f` + uncommitted Slice 4 corpus work |
| Interaction manifest | `0a99c714c258bac7f91eb5dd39748651abca8db96bfc1a410d823a18d2c23d93` |
| Binary-mask fixture | `37dc5c494f2394b7c7c99eca6cc800f039975fb6add1e48868fbc965657fa48e` |
| Mixed-alpha fixture | `2d82b95538bf2af33e88a9eb1bd1a2de73e9a1a15d3305f1268c048e7c9fc4dd` |
| Camera convention | Tokimu GL-style `[-1, 1]`, explicitly converted by the WGPU provider to `[0, 1]` |

This is a maintainer-supplied browser visual observation, not a pixel-golden,
browser-vendor, or adapter-vendor guarantee.

## Observed Result

The page reached browser adapter/device preflight and then reported:

```text
ready | interaction first frame presented | 7 draws/7 materials/7 pipelines |
manifest=0a99c714c258bac7f91eb5dd39748651abca8db96bfc1a410d823a18d2c23d93 |
diagnostic=none | backend=browser-webgpu | device=other | adapter=? |
viewport=960x600
```

The upper-left panel visibly showed binary cutout coverage over blue opaque
backing. The upper-right panel visibly showed continuous mixed-alpha Blend
over the same opaque backing. The lower panel visibly showed the binary cutout
and the sloped, depth-writing blended surface exchanging depth across the
panel. The browser observation therefore matches the native fixture's three
semantic cases and locked source manifest; neither image is claimed to be
pixel-identical to the other target.

## Interpretation Boundary

This establishes one browser/WebGPU presentation of the bounded Slice 4 scene.
It does not establish NVIDIA behavior, general intersecting-transparency
handling, renderer-owned sorting, automated framebuffer capture, batching, or
a stable public alpha/depth contract.
