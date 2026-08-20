# Doom E1M2 Native/WASM Structural Parity Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-19 |
| Package | `corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip` |
| Member | `DOOM1.WAD` |
| Map | `E1M2` |
| Scope | Importer and prepared source-geometry structure |

## Compared Paths

Both consumers read the explicitly supplied ZIP through
`doom-wad-package`, decode the selected map through `doom-map-provider`, and
derive walls, sky boundaries, and sector-bounded subsector planes through
`doom-geometry-provider` plus the application-local `hello-doom-e1m1`
lowering. TypeScript selects a map and presents the returned observation; it
does not parse or reconstruct WAD geometry.

The native half is reproducible without opening a renderer window:

```powershell
cargo run -q -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --map=E1M2 --skywall-parity-full --sector-boundary-trim `
  --topology-inventory-report
```

The retained real browser-WebGPU observation was:

```text
browser working-model frame presented: map=E1M2; strategy=global-full-plus-grouped-sky-parity; stages=sky-panorama>full-world-depth-prepass>paired-skywall-and-source-sky-plane-stencil-inversion>even-parity-world-color; sector-boundary-trim=true; opaque=1921; cutouts=0; skywalls=20; sky-planes=242; surface-triangles=3635; edge-conformance-insertions=642; camera=source-spawn; embedding=preserve-north; backend=browser-webgpu; canvas=960x600
```

The native headless structural record was:

```text
E1M2 native/browser structural comparison: importer=shared-rust-wad-package+doom-map-provider; geometry=shared-rust-doom-geometry-provider+hello-doom-e1m1-lowering; opaque=1921; cutouts=0; skywalls=20; sky-planes=242; source-boundary-triangles=3635; edge-conformance-insertions=642; inventory-records=2163; inventory-hash=719d905e4371ff94; authority=native-headless-structure-for-comparison-with-real-browser-webgpu-observation-not-rendered-pixel-parity
```

## Result

| Structural field | Native | Browser/WASM | Result |
| --- | ---: | ---: | --- |
| Opaque declarations | 1,921 | 1,921 | exact |
| Cutout declarations | 0 | 0 | exact |
| Paired-skywall triangles | 20 | 20 | exact |
| Source-sky-plane triangles | 242 | 242 | exact |
| Source-boundary surface triangles | 3,635 | 3,635 | exact |
| Edge-conformance insertions | 642 | 642 | exact |

The native inventory independently conserves the same declaration families:
447 floor, 395 ceiling, 155 upper-wall, 208 lower-wall, 716 middle-wall, and
242 sky-plane records. The five ordinary families sum to 1,921; including sky
planes yields all 2,163 inventory records. Its aggregate structural hash is
`719d905e4371ff94`.

This establishes exact importer/prepared-geometry structure for the retained
E1M2 comparison. It does not claim rendered-pixel identity, shader arithmetic
identity, adapter equivalence, or browser GPU-memory observability. Browser
execution remains necessary evidence; WASM compilation alone is not parity.
