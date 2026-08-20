# Doom E1M1 Headless Structural Image Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-19 |
| Package | `corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip` |
| Member | `DOOM1.WAD` |
| WAD BLAKE3 | `2a0c5f3c001228980409e483c06c5510e5a1f392d9a3551bc955b55b04aa930b` |
| Map | `E1M1` |
| Artifact | [E1M1 deterministic sector map.bmp](E1M1%20deterministic%20sector%20map.bmp) |
| Dimensions | 1024 by 768 RGBA8 |
| Artifact bytes | 3,145,782 |
| SHA-256 | `4397bf623b6b757bd362591e670b6c368a5b0cd7e8ce1c406cb51e36a20e98e5` |

## Reproduction

```text
cargo run -q -p hello-wad-inspect -- --zip corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD --map-sector-bmp E1M1 "docs/Plans/DOOM/Evidence/E1M1 deterministic sector map.bmp"
```

The exporter decodes the reviewed package and map through the existing bounded
Rust providers, maps source vertices into a fixed 1024 by 768 integer raster,
draws LINEDEFS in source order using deterministic sector colors, and overlays
THING locations with a distinct player-one marker. The common `screenshot`
corpus helper writes a top-down 32-bit BMP.

Two independent executions produced the same SHA-256 value shown above. The
artifact is therefore retained as a deterministic headless structural image.
It is not a native or browser GPU framebuffer capture, a textured 3D scene, a
visibility oracle, or a claim of native/WASM pixel equivalence. Fixed-camera
native and browser image observations remain a separate open checklist item.

