# E1M1 Static Presentation Evidence

## Scope

This records the fixed-scene evidence and native/browser first-frame
observations for Slice 5B. It is evidence for a bounded static presentation
projection only. It does not claim original Doom visibility, dynamic sector
behavior, sky drawing, masked-middle behavior, alpha policy, gameplay, or
pixel-equivalent browser rendering.

## Evidence Model

| Kind | What is retained | Claim |
| --- | --- | --- |
| Deterministic scene evidence | reviewed package identity, fixed camera policy, draw/texture-handle inventory, source omission counts, and structural reports | equivalent inputs produce the same described static scene request; this is not a framebuffer-byte claim |
| Visual observation evidence | fixed-camera native and browser images plus target, adapter, build, package, and camera provenance | the requested scene visibly presented on that target; images are not expected to be pixel-identical across providers |

## Canonical invocation

From the repository root:

```powershell
cargo run -p hello-doom-e1m1 --bin hello-doom-e1m1 -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD

cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD
```

## Retained preparation result

The canonical preflight reports:

| Item | Result |
| --- | ---: |
| Opaque flat triangles | 853 |
| Opaque wall triangles | 982 |
| Submitted floor / ceiling triangles | 463 / 390 |
| Submitted middle / upper / lower wall triangles | 588 / 184 / 210 |
| Submitted static draws | 1,835 |
| Flat texture/material uploads | 21 |
| Wall texture/material uploads | 29 |
| Deferred-alpha selected rasters | 0 |
| Source-classified sky omissions | 74 |
| Source-classified masked-middle omissions | 13 |
| Degenerate flat omissions | 43 across 21 subsectors; no complete subsector omission |
| Degenerate wall omissions | 16 across 8 zero-height source spans |

The preflight executable emits the complete, deterministic source-kind/name to
texture/material-handle inventory. It assigns handles only to the selected
fully covered, palette-zero sRGB opaque profile and makes no renderer decision
for deferred-alpha sources.

## Native observation

The native `static_scene` target presented the full 1,835-draw overview with
visible floor, ceiling, and wall textures. Its initial blank presentation was
resolved as camera clipping: `Camera::perspective_3d` has a deliberate 100-unit
far plane for small fixtures, while E1M1 spans thousands of source units. The
corpus target now derives a fixed observer and explicit near/far range from the
ordinary submitted mesh bounds. This remains a corpus-local camera policy.

## Known omissions

- Sky source records are retained but not drawn.
- Masked middles remain source-classified omissions pending AR-0023.
- The eight fully omitted wall linedefs are `DOORTRAK`/`DOORSTOP` middle spans
  with identical minimum and maximum heights; no facing normal is fabricated.
- Browser/WASM first-frame evidence remains separate and incomplete. Browser
  package intake must follow the AR-0020/TTSDD TypeScript-boundary plan rather
  than silently acquiring reviewed Doom bytes from the site.

## Browser/WASM Bootstrap Build Evidence

The `doom-ts-boundary-workbench` now exposes a consumer-local,
promise-returning `render_static_e1m1(canvas)` request after explicit local ZIP
selection. Rust/WASM reopens the bounded `DOOM1.WAD` member from its retained
Resource Space session, runs the same application-local static preparation
seam, and passes only a browser canvas to the WGPU provider. TypeScript gets no
map, texture, material, or renderer-policy object.

The following build checks passed on 2026-08-09:

```powershell
cargo check -p doom-ts-boundary-workbench-engine --target wasm32-unknown-unknown
cargo build -p doom-ts-boundary-workbench-engine --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/doom_ts_boundary_workbench_engine.wasm `
  --target web --out-dir corpus/consumers/doom-ts-boundary-workbench/web/pkg `
  --out-name doom_ts_boundary_workbench_engine
node corpus/consumers/aspnet-wasm-asset-workbench/node_modules/typescript/bin/tsc `
  -p corpus/consumers/doom-ts-boundary-workbench/tsconfig.json
```

## Browser/WASM Observation

On 2026-08-09, an explicit local selection of
`doom-shareware-corpus-v1.zip` completed the browser request and returned:

```text
browser first frame presented: 1835 draws
```

The supplied canvas visibly contained the fixed-camera textured E1M1 overview.
This establishes browser readiness and first-frame execution for the same
bounded package path as the native consumer. It is a manual visual observation,
not a committed image artifact or a pixel-equivalence claim. Future browser
captures include the returned backend, device-kind, adapter, and canvas-size
metadata; native startup emits analogous metadata beside its fixed-camera
invocation.

## Masked-Cutout Browser Request

For AR-0023 Slice 5, the same bounded session now also exposes
`render_static_e1m1_masked_cutouts(canvas)`. The request is deliberately
separate from `render_static_e1m1(canvas)`: opaque presentation does not create
or validate the cutout pipeline. The request reuses the Rust-owned
package/member path, selects retained E1M1 source classifications, and draws
26 masked-middle candidates alongside the 1,835 opaque draws through ADR-0013's
generic categorical-cutout declaration. TypeScript receives only the completion
string and canvas presentation.

The request compiles for `wasm32-unknown-unknown`; bindings and the browser
presentation script regenerate successfully. On 2026-08-10, the rebuilt
admitted path visibly presented the reviewed package at `960x600`: opaque
reported 1,835 draws and masked cutout reported 1,861 draws. This is a manual
cross-target observation with browser device `other` and an unavailable adapter
name, not a committed image artifact or pixel-equivalence claim.
