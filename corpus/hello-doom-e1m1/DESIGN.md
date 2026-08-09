# Hello Doom E1M1 Static Presentation

## Status

Slice 5B implementation in progress. This consumer owns only the selected
static presentation lowering for E1M1; it does not own WAD parsing, Doom map
semantics, renderer implementation, or runtime state.

## Initial contract

The first lowerer accepts a retained `DoomSurfaceTriangle` and emits an
ordinary `tokimu::Mesh` with checked supplied UVs. It can batch those surfaces
using already-retained `DoomSkySurfaceObservation` records, preserving sky as
an explicit omission rather than inferring a renderer policy from a texture
name. Its explicit map-axis policy is documented in [Classic Doom plane mapping evidence](../../docs/Plans/DOOM/Classic%20Doom%20plane%20mapping%20evidence.md):

```text
u =  x / flat_width
v = -z / flat_height
```

The lowerer retains source subsector, sector, plane, and flat name alongside
the mesh so a later draw list can retain diagnostics without giving those Doom
types to `tokimu-render`.

Wall triangles use the source texels already calculated by
`doom-geometry-provider`, normalized by the selected source texture extent.
The consumer does not reimplement pegging. It excludes only retained
two-sided masked-middle observations; ordinary one-sided middle walls remain
opaque candidates until their selected raster coverage is checked.

The first texture classifier makes palette zero, sRGB upload interpretation,
and point/repeat sampling explicit consumer selections. It yields an opaque
candidate only when every indexed source pixel is covered; otherwise it
returns a source-counted deferred-alpha result for AR-0023. It does not use
alpha bytes as an implicit blend or cutout request.

The consumer now has two executable evidence paths: a headless canonical
preflight and a native static-scene first-frame target. The preflight retains
the ZIP at a Resource Space edge and emits the deterministic source
omissions/texture-material inventory. The native target consumes only its
ordinary mesh/material upload plan, uses explicit opaque depth-writing state,
and owns a bounds-based overview camera because E1M1 exceeds the small-fixture
perspective helper's default far plane. It does not change renderer-wide camera
policy.

Masked-middle classification, general alpha behavior, original Doom plane-span
rendering, sky drawing, and browser/WASM capture remain subsequent increments.

`prepare_e1m1_flats` now accepts caller-owned, already-inspected WAD bytes and
manifest, then selects and decodes E1M1 through the existing providers before
returning the prepared flat/sky assembly. ZIP acquisition remains outside this
consumer function, at the existing Resource Space/archive boundary.

`prepare_e1m1_flat_textures` then decodes only the selected non-sky flat names
with palette zero and returns upload-ready RGBA8 payloads beside their explicit
eligibility result. It still allocates no renderer resource.

The preflight executable accepts the canonical ZIP path and its selected WAD member,
retains the ZIP in an in-memory Resource Space root, reads the WAD as a bounded
derived member, and prints one deterministic E1M1 preflight report. The
workspace intentionally does not retain the extracted WAD or package payload;
running this command against the reviewed local package is the next evidence
step before GPU upload.
