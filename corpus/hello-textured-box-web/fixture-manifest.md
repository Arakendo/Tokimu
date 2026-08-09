# Hello Textured Box Browser - Structural Manifest

This is a browser-consumer structural manifest, not a screenshot, a claim of
browser/native pixel equality, or an imported GLB-material record.

## Shared Sources

The browser consumer embeds the following bytes directly in its WASM module:

| Input | Source / checksum authority | Browser use |
| --- | --- | --- |
| Geometry | Pinned Khronos `Box.glb`, SHA-256 `ed52f7192b8311d700ac0ce80644e3852cd01537e4d62241b9acba023da3d54e` | Decode 24 positions/24 normals/36 indices; expand to 36 triangle-list vertices. |
| Grid | `corpus/assets/PNG/Dark/texture_01.png`, SHA-256 `07e28d7a86396fa4f1d7c43040f8e57e2374f6fa35b02bc48e70f9ccd4041c1a` | Initial texture, plus `M` cycle entry. |
| Dark door | `corpus/assets/PNG/Dark/texture_11.png`, SHA-256 `a1bdb8059b5939367cad693c4aef3043d744600fb3d155e45884d7f7ec634924` | `M` cycle entry for UV-orientation observation. |
| Green door | `corpus/assets/PNG/Green/texture_11.png`, SHA-256 `62fdf8d029989ac0436e35c242a498fe10287975bfbb8ff14feff63db8e3d409` | `M` cycle entry for texture-identity comparison. |

The native fixture manifest contains source provenance and PNG profile details:
[`../hello-textured-box/fixture-manifest.md`](../hello-textured-box/fixture-manifest.md).

## Declared Browser Scene

| Concern | Declared value |
| --- | --- |
| Geometry/material relation | Independent sources; the selected PNG is not a `Box.glb` material. |
| UV generation | Corpus-owned planar mapping selected from position/normal, then scaled by `3.25` so clamp/repeat is observable. |
| UV modes | Identity, U flip, U/V swap. |
| Sampler modes | Point/clamp, point/repeat, linear/clamp, linear/repeat. |
| Pipeline | `Textured3d`, back-face culling, depth-writing 3D state. |
| Camera | Fixed `(2.8, 1.8, 2.8)` looking at origin. |
| Texture interpretation | Normalized RGBA8 sRGB color texture. |
| Alpha profile | Opaque initial profile; no cutout or general transparency claim. |
| Startup | Browser adapter/device preflight, then asynchronous Tokimu WGPU construction; ready/unsupported/failed are distinct DOM states. |

## Validation Boundary

```text
cargo test -p hello-textured-box-web
cargo check -p hello-textured-box-web --target wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown -p hello-textured-box-web
wasm-bindgen target/wasm32-unknown-unknown/debug/hello-textured-box-web.wasm --out-dir corpus/hello-textured-box-web/web/pkg --target web
```

The focused Rust tests verify the deterministic mode declarations, but do not
establish a browser GPU frame. The retained first-frame observation is in
[`results/browser-manual-observation.md`](results/browser-manual-observation.md).
