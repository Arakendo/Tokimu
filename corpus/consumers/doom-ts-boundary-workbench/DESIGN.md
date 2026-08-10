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
picker binding opens a file input only from a visible button-click gesture,
forwards one selected `File` to the Rust/WASM request, clears the browser input
afterward, and returns only retained/cancelled/rejected presentation outcomes.

## Static E1M1 Browser Bridge

The generated `render_static_e1m1(canvas)` request is available only in the
WASM build. It reopens `DOOM1.WAD` as a bounded derived ZIP member from the
Rust-owned session, invokes the existing Doom providers and application-local
`hello-doom-e1m1` preparation seam, and submits one fixed-camera opaque frame.
The renderer receives only ordinary meshes, material handles, texture uploads,
and camera data. The canvas is a browser presentation surface, not a TypeScript
rendering abstraction.

This is readiness evidence only until a selected local reviewed package has
visibly presented the frame. It neither makes the browser workbench the WAD
plan's canonical importer nor substitutes for native/WASM conformance capture.

## Experimental Masked-Cutout Browser Bridge

The separate `render_static_e1m1_masked_cutouts(canvas)` WASM request is an
AR-0023/Slice-5 experiment. After the same explicit local selection, Rust/WASM
reopens the bounded package and selects E1M1's retained Doom masked-middle
observations. It passes only ordinary mesh, texture, material, camera, and a
corpus-local custom pipeline to WGPU. TypeScript still owns no WAD parsing,
source classification, threshold, or renderer policy; it merely exposes the
distinct request and presents the returned observation.

The original opaque request remains isolated: it does not prepare masked
inputs or register the experimental pipeline. Neither request is a browser
renderer API or a stable `tokimu-render` alpha contract.
