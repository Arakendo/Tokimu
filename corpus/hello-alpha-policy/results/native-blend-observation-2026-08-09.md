# Native Slice 3 Blend Observation — 2026-08-09

## Capture Identity

| Field | Value |
| --- | --- |
| Target | native WGPU window |
| Backend | Vulkan |
| Device kind | discrete GPU |
| Adapter | AMD Radeon RX 7900 XTX |
| Viewport | 960 × 600 |
| Corpus command | `cargo run -p hello-alpha-policy --features native-visual --bin native_blend_scene` |
| Fixture source | manifest-locked `mixed-alpha` exact RGBA8 array |
| Blend equation | straight source-alpha `AlphaBlend` |
| Depth test | `LessEqual` |
| Camera convention | Tokimu GL-style `[-1, 1]`, converted by WGPU to `[0, 1]` |

This is a maintainer-supplied visual observation, not a pixel-golden artifact
or a cross-vendor renderer guarantee.

> Historical scope: the original four-panel capture preceded the added
> `continuous-gradient` control and first-versus-warm resource-reuse
> instrumentation. The expanded observation below closes those additional
> questions while preserving the original evidence.

## Observed Result

The first frame visibly presented all four comparison panels plus the opaque
blue control. The window reported:

```text
11 draws, 11 material resolutions, 6 pipeline switches, diagnostic=none
```

The upper far-then-near and near-then-far panels produced visibly different
color bands while retaining the same texture, geometry, UVs, transforms,
camera, and blend equation. This demonstrates order dependence for the
experimental blend profile on this target.

The lower depth-write-disabled and depth-write-enabled panels also produced
visibly different results under the fixture's identical near-to-far caller
order. This demonstrates that depth-write state is independently observable
from blend equation and caller order. The opaque control confirms that the
positive GL-space fixture depths remain visible after the AR-0024 WGPU upload
conversion.

## Expanded Fixture And Reuse Observation

The expanded fixture visibly presented the continuous-gradient-over-opaque
control as well as the four original comparison panels. Its native AMD/Vulkan
terminal observations were:

```text
first frame: 12 draws, 12 material resolutions, 6 pipeline switches,
13 binding allocations, 0 uniform writes, 0 mesh uploads, diagnostic=none

warm frame: 12 draws, 12 material resolutions, 6 pipeline switches,
0 binding allocations, 0 uniform writes, 0 mesh uploads, diagnostic=none
```

The first frame allocated the camera plus twelve per-draw instance bindings.
The unchanged warm frame reused them and did not upload static mesh resources.
Material resolution and pipeline selection remain per-draw current-renderer
work. This is a native provider observation, not a batching, render-order, or
bind-group contract.

## Interpretation Boundary

This observation establishes native AMD/Vulkan execution for the bounded
comparison and closes AR-0024's visible-recovery check. It does not establish
browser parity, NVIDIA behavior, a correct general transparency order, a
renderer-owned sorting service, or a stable public blend contract. Those
questions remain in AR-0023 and the comparative corpus plan.
