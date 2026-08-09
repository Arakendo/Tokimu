# Hello Textured Box - Slice 0 Fixture Manifest

This is a source and decoded-structure manifest, not a GPU capture or a claim
of visual equivalence. SHA-256 values were recorded on 2026-08-09.

## Geometry

| Field | Value |
| --- | --- |
| Fixture | `third-party/fixtures/khronos-gltf-sample-assets/upstream/Models/Box/glTF-Binary/Box.glb` |
| Upstream source | Khronos glTF Sample Assets, revision `2bac6f8c57bf471df0d2a1e8a8ec023c7801dddf` (vendored selected fixture, not a Git submodule) |
| Size / SHA-256 | 1,664 bytes / `ed52f7192b8311d700ac0ce80644e3852cd01537e4d62241b9acba023da3d54e` |
| Structural decode | GLB v2; one scene, two nodes, one mesh, one primitive, 24 positions, 24 normals, 36 indices |
| Bounds | `[-0.5, -0.5, -0.5]` to `[0.5, 0.5, 0.5]` |
| Source UV evidence | This `Box.glb` primitive has no `TEXCOORD_0`; the corpus supplies its own explicit planar UV data through the reviewed candidate generic contract. |

The geometry facts are already covered by `gltf-corpus`'s
`box_positions_normals_and_indices_decode` test. Expanding the indexed source
for the current renderer yields 36 triangle-list vertices. The fixture's
source geometry still does not prove a texture-coordinate stream; its
corpus-owned planar stream is a separate declared input.

## Initial PNG Selection

| ID | Fixture | SHA-256 | Profile | Reason |
| --- | --- | --- | --- | --- |
| `grid` | `corpus/assets/PNG/Dark/texture_01.png` | `07e28d7a86396fa4f1d7c43040f8e57e2374f6fa35b02bc48e70f9ccd4041c1a` | 1024x1024, indexed palette, 4-bit | Coarse grid and crosshair make UV orientation, out-of-range addressing, and point/linear changes legible. |
| `door-dark` | `corpus/assets/PNG/Dark/texture_11.png` | `a1bdb8059b5939367cad693c4aef3043d744600fb3d155e45884d7f7ec634924` | 1024x1024, indexed palette, 8-bit | Labelled `DOOR` and centered frame reveal rotation, U/V inversion, and per-face orientation. |
| `door-green` | `corpus/assets/PNG/Green/texture_11.png` | `62fdf8d029989ac0436e35c242a498fe10287975bfbb8ff14feff63db8e3d409` | 1024x1024, indexed palette, 8-bit | Same presentation construction in a distinct palette; changes texture identity without changing geometry or supplied UVs. |

All three are first-party reference assets. They have no `tRNS` chunk and use
PNG indexed color (`color_type=3`), so none is an alpha-policy fixture. The
corpus must either add a separately documented first-party alpha source or
retain alpha as unsupported/deferred.

## Opening Baseline And Current Candidate Contract

| Concern | Opening evidence | Current candidate contract / retained limit |
| --- | --- | --- |
| Mesh streams | Positions and normals only | Optional checked caller UV stream; `Textured3d` requires it. |
| Textured shader | `PipelineKind::Texture2d` derives UV from 2D position | Separate `Textured3d` consumes supplied UVs; the 2D path is unchanged. |
| Sampler | WGPU source sampler: nearest filter and clamp-to-edge | Material declares point/linear and per-axis clamp/repeat without exposing WGPU objects. |
| Texture input | Renderer accepts explicit normalized RGBA8 sRGB/linear upload | PNG decoder output remains outside the renderer. |
| Alpha | No selected source has alpha | Initial profile is opaque. Blended/cutout texture admission remains unresolved; see AR-0023. |

## Reproduction

```text
cargo test -p gltf-corpus --test khronos_selection box_positions_normals_and_indices_decode
```

Focused current validation:

```text
cargo test -p hello-textured-box
cargo test -p tokimu-render
```

Neither command is rendering evidence; native and browser observations are
retained separately beside their respective corpus consumers.
