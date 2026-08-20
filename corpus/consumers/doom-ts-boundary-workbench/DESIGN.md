# DOOM TypeScript Boundary Workbench

## Status

Slice 0 and browser-intake Slice 1 are in progress under the DOOM TypeScript
Boundary Stress Plan and AR-0020. The workbench also carries one deliberately
bounded Slice 5B bridge: after explicit local selection, it may ask Rust/WASM
to present the already retained canonical `DOOM1.WAD` member on a supplied
browser canvas. That bridge is consumer evidence, not a browser renderer API
or a TypeScript-owned Doom model.

## Classification

| Field | Declaration |
| --- | --- |
| Package / entry | `corpus/consumers/doom-ts-boundary-workbench` |
| Primary role | Browser/presentation mechanism |
| Reads | Explicit browser file selection/drop events, selected bytes/name/media hint, and returned Rust/WASM observations |
| Emits | Versioned bounded import requests and presentation-only progress/result state |
| Durable state | Rust/WASM Resource Space owns active-session retained bytes and identity; TypeScript owns no semantic or durable game state |
| Semantic authority | None; TypeScript does not parse ZIP/WAD data or select Doom/rendering policy |
| Execution authority | Browser gesture and file-read mechanism; it may invoke only the explicit WASM request |

## Intake Contract

```text
user gesture (TypeScript)
  -> selected bytes + source label + media hint + declared limits
  -> Rust/WASM request (versioned)
  -> Resource Space identity + retained bytes + bounded archive/WAD observation
  -> provider-neutral observation / diagnostic
  -> TypeScript presentation
```

The browser must not fetch, bundle, or publish reviewed Doom data. Local
selection or drag/drop is the first admitted source. Rust/WASM owns byte
limits, empty/oversized rejection, Resource Space identity, replacement, ZIP
and WAD validation, and provider diagnostics.

TypeScript must not inspect ZIP entries, parse a WAD directory, normalize Doom
names, retain a WAD-derived game model, choose rendering policy, or advance
game state. It may display returned observations and pass a browser-owned
canvas to the explicit Rust/WASM first-frame request.

## First Slice Deliverables

- a user-gesture local-file request with visible cancellation;
- a versioned Rust/WASM import request and serializable provider-neutral observation;
- explicit empty, oversized, and unsupported diagnostics;
- a retained authority-delta record for selection, request, disposal, and replacement;
- no network request and no TypeScript format parser.

The intake slice itself did not render E1M1. The later, isolated Slice 5B
bridge consumes the same successful Rust-owned bounded session: TypeScript
does not receive geometry, textures, materials, or renderer state; it only
supplies the canvas and displays the completion or diagnostic string.

## Implemented Rust/WASM Session

`engine/` now exposes `BrowserIntakeSession`. Its schema-v1
`import_selected_package(sourceLabel, mediaHint, bytes)` request accepts no
browser path or ambient authority. It rejects empty and over-limit bytes,
retains exactly one selection in Resource Space, reports its BLAKE3 fingerprint
and retained-byte count, and replaces/disposes previous session bytes through
Rust-owned state. It intentionally does not yet claim archive or WAD success.

`web/src/intake.ts` is the corresponding browser/presentation mechanism. Its
picker binding opens a file input only from a visible button-click gesture; its
drop binding accepts exactly one explicitly dropped `File`. Both forward the
selection to the same bounded Rust/WASM request without interpreting package
contents and return only retained/cancelled/rejected presentation outcomes.
The picker clears the browser input after each request.

## Bounded Browser Work

The corpus build rejects more than 12 MiB of uncompressed emitted startup
payload across the HTML, reachable JavaScript, generated WASM bridge, and WASM
module. Runtime intake and all archive, WAD, map, raster, and texture decoders
retain explicit limits. Before a working-model replacement resets the retained
provider scene or uploads successor resources, Rust also validates bounded
mesh, texture, material, pipeline, camera, command, mesh-vertex-byte, and
source-texture-byte estimates. The exact command count is checked again before
installation.

These limits bound work requested by this corpus. They do not claim exact
physical Edge, WGPU, driver, or GPU residency or synchronous reclamation.

## Static E1M1 Browser Bridge

The generated `render_static_e1m1(canvas)` request is available only in the
WASM build. It reopens `DOOM1.WAD` as a bounded derived ZIP member from the
Rust-owned session, invokes the existing Doom providers and application-local
`hello-doom-e1m1` preparation seam, and submits one fixed-camera opaque frame.
The renderer receives only ordinary meshes, material handles, texture uploads,
and camera data. The canvas is a browser presentation surface, not a TypeScript
rendering abstraction.

The separate `render_e1m1_diagnostic_sky_omissions(canvas)` request is an
AR-0027 comparison control. It explicitly re-lowers only retained Doom sky
omissions and applies the first-party Purple corpus PNG selected by this
application. Its result reports the retained/submitted omission count, asset
path, and reason. Normal E1M1 rendering does not select this material, and the
renderer does not infer fallback policy from missing source data.

This is readiness evidence only until a selected local reviewed package has
visibly presented the frame. It neither makes the browser workbench the WAD
plan's canonical importer nor substitutes for native/WASM conformance capture.

## Experimental Masked-Cutout Browser Bridge

The separate `render_static_e1m1_masked_cutouts(canvas)` WASM request is an
AR-0023/Slice-5 real-caller path. After the same explicit local selection, Rust/WASM
reopens the bounded package and selects E1M1's retained Doom masked-middle
observations. It passes ordinary mesh, texture, material, camera, and ADR-0013's
generic categorical-cutout declaration to WGPU. TypeScript still owns no WAD
parsing, source classification, threshold, or renderer policy; it merely exposes
the distinct request and presents the returned observation.

The original opaque request remains isolated: it does not prepare masked
inputs or register the cutout pipeline. Neither request makes TypeScript a
browser renderer API; continuous Blend remains outside the admitted contract.

## AR-0025 Selected-Cutout Browser Evidence

`render_static_e1m1_selected_cutouts(canvas)` is a third, deliberately
corpus-local request. Rust/WASM resolves the same E1M1 player-one start and
sector context used by the native observer, derives the source heading, applies
the fixed yaw-plus-90 evidence pose, and filters ordinary prepared draws whose
conservative AABBs are wholly outside that camera's homogeneous clip frustum.
TypeScript only exposes the button, forwards the canvas, and displays the
returned count/presentation observation.

This is not a browser scene API, a TypeScript-owned camera, a Doom visibility
rule in the renderer, or an admitted Tokimu culling contract. All meshes are
still uploaded for the one-shot browser fixture before the corpus filter
chooses submitted commands, so this path may prove cross-target selection and
visual behavior but not browser upload or steady-state performance savings.

For local observation after generating `web/pkg/` and `web/app/`, serve the
workbench from its `web/` directory, for example:

```powershell
python -m http.server 4176 --directory corpus/consumers/doom-ts-boundary-workbench/web
```

Select the reviewed ZIP, then use **Render E1M1 selected cutouts**. The
returned observation must retain the candidate/rejected/submitted counts;
the browser should not be treated as a timing benchmark.

## Current Working-Model Browser Test

`render_working_map(canvas, mapName)` is a separate browser/WASM test for the
current native presentation candidate. Rust validates `mapName` as E1M1
through E1M9, reopens the user-selected `DOOM1.WAD`, and owns map decoding,
sector-boundary plane trimming, texture preparation, source-spawn camera
selection, and the grouped sky sequence:

```text
sky panorama
  -> full ordinary-world depth prepass
  -> paired skywall plus source-sky-plane stencil inversion
  -> ordinary world color where parity is even
```

The browser UI exposes **Render working model**, previous/next controls, and
the same `[` / `]` keys as the native walkabout. Every switch prepares and
presents a complete replacement frame before reporting the new map. The
historical fixed E1M1 buttons remain intact as controls; this addition does not
rewrite their retained evidence.

After the initial source-spawn frame, Rust retains the prepared renderer,
commands, and camera for a corpus-private noclip inspection loop. Click the
canvas for pointer-lock mouse look; use W/A/S/D to move, Space/C vertically,
Shift to run, and Escape to release the mouse. TypeScript supplies normalized
input deltas only. Rust owns camera mutation and frame submission, and idle
animation frames do not resubmit the scene. Live input is coalesced behind an
adaptive recovery interval based on the preceding synchronous presentation
cost; this keeps heavier maps from continuously saturating the browser renderer
process. Map replacement disables the loop until a complete replacement has
been prepared and presented. CPU preparation may coexist with the previous
map, but the previous WGPU backend is explicitly released before a replacement
surface is requested for the same canvas; two live backends never overlap on
that canvas. Retained WebGPU provider diagnostics are checked before and after
each inspection presentation and stop the loop visibly.

This is a visual inspection camera, not Doom player simulation. It does not
provide collision, doors/platforms, Things, or audio. The initial observation
reports the selected map, complete stage sequence, ordinary/cutout/sky
contribution counts, boundary-trim audit, adapter, and canvas size.

Build and serve it with:

```powershell
cd corpus/consumers/doom-ts-boundary-workbench
npm install
pwsh -NoProfile -File .\build.ps1
python -m http.server 4176 --bind 127.0.0.1 --directory web
```

Then open `http://127.0.0.1:4176/`, select the reviewed local ZIP, choose
**Render working model**, and use `[` / `]` to rotate maps.

The separate **Run 3x map rotation** control is the Alternative-A baseline for
the renderer scene-resource lifetime plan. It deterministically renders E1M1
through E1M9 three times and retains one bounded record per replacement. Rust
reports logical current/retired resource counts, estimated mesh-vertex and
source-texture payload bytes, and backend/device/surface creation counts.
`physical-gpu-reclamation=unobserved` is deliberate: neither a Rust drop nor a
browser animation-frame yield proves when provider or driver storage is freed.
This automated replacement sequence does not replace the manual walkabout.
