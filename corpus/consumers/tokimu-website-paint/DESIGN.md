# Tokimu Website Paint Consumer

## Classification

Tokimu Paint is a Tier 2 consumer corpus entry. Its first implementation uses
the incubating `corpus/lib/raster-image-corpus` decoder boundary and does not
present that boundary as a stable Tokimu API.

## Composition Claim

```text
encoded bytes
    -> raster provider DecodedImage
    -> Paint-owned EditableRasterDocument
    -> semantic edit commands and bounded history
    -> provider-neutral observations
    -> TypeScript and Canvas mechanisms
```

Application state owns editing meaning. Rust/WASM owns authoritative pixels and
deterministic edits. Raster providers own encoded formats. TypeScript owns
browser interaction. Canvas owns displayed pixels only.

`DecodedImage` and `EditableRasterDocument` are intentionally different types.
Paint copies normalized decoder evidence into application-owned mutable storage;
the source observation is never mutated by editing.

## Initial Fixtures

The first provider-composition test uses existing provenance-tracked fixtures:

- PNG: `third-party/fixtures/w3c-svg-1.1-2nd-edition/upstream/images/PngSuite/basn6a08.png`
- JPEG: `third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages/testorig.jpg`
- BMP: `third-party/fixtures/raster-images/upstream/libjpeg-turbo/testimages/shira_bird8.bmp`
- transparent editing fixture: a Tokimu-authored 2x2 transparent RGBA8 document
  created directly through the blank-document boundary

## Initial Observations

Document observations report dimensions, stride, storage size, pixel format,
color interpretation, alpha mode, orientation, revision, dirty state, and a
deterministic exact-pixel fingerprint. Command, history, preview, and export
observations remain separate records owned by their respective stages.

## Current WASM Boundary

`WasmPaintSession` owns one local `PaintSession`. It accepts only admitted
PNG, JPEG, or BMP bytes and lowercases only the small source-format label.
Small commands and observations use JSON control records. Preview RGBA pixels
and PNG export data use explicit copied byte buffers instead of JSON. Disposal
clears the Rust-owned initial document and workspace; later operations then
fail predictably.

## Website Island Boundary

The same built Paint workbench may be published as an explicitly activated
`tokimuengine.org` iframe island. The website adapter owns only activation,
iframe lifetime, fallback visibility, and release. It does not receive or
interpret paint commands, document pixels, history, export bytes, or provider
observations. Those remain inside the consumer's Rust/WASM and TypeScript
workbench boundary.

The standalone workbench also disposes its local WASM session on `pagehide`.
This is lifecycle evidence for the consumer, not a browser-wide Paint service.

## Explicitly Unsupported

- indexed or compressed editable storage
- linear, HDR, or higher-precision documents
- browser-native decode fallback
- Canvas-owned undo or save state
- palette preservation
- layers, selection, filters, text, material painting, and collaboration
- promotion of a shared editable-image capability from this consumer alone
