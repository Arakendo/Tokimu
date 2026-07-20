# UI Tools, Lucide, and Glyph Corpus Audit

Date: 2026-07-20

## Scope

This audit covers the shared `examples/lib-example/ui-tools` project and the
following corpus examples:

- `hello-ui-lucide`
- `hello-ui-lucide2`
- `hello-ui-font`
- `hello-ui-font2`
- `hello-ui-glyph-corpus`

The goal is to keep reusable renderer, rasterization, parsing, and coordinate
policy in `ui-tools`, while leaving corpus selection and example presentation in
the individual examples.

## Findings

### 1. SVG document parsing is duplicated in the examples

`hello-ui-lucide` and `hello-ui-lucide2` each implement their own SVG loading
and extraction logic. `ui-tools` currently owns path tokenization, path parsing,
flattening, and stroke tessellation, but does not own extraction of path data
and primitive elements from an SVG document.

Reusable behavior that should move to `ui-tools`:

- SVG path extraction from a document;
- extraction of `circle`, `rect`, `line`, `polyline`, and `polygon` elements;
- primitive-to-polyline conversion;
- viewBox normalization;
- consistent SVG coordinate conversion.

The examples should select files, call the shared loader, and submit the
resulting meshes.

### 2. `hello-ui-lucide2` contains fallback geometry that weakens the corpus test

The example still contains hand-authored star, heart, activity, circle,
diamond, lightning, triangle, square, hexagon, and arrow geometry. These were
useful while developing the renderer, but they are no longer appropriate as
part of the provider corpus proof.

The fallback shapes should either move to a separate geometry regression
example or remain only as small unit-test fixtures. `hello-ui-lucide2` should
render provider SVGs exclusively.

### 3. SVG coordinate normalization is example-owned

`hello-ui-lucide2` applies the Lucide `24x24` viewBox conversion directly in
the example. That makes the example responsible for provider-specific spatial
policy and will cause duplication when another SVG provider is added.

Normalization should be a shared `ui-tools` operation parameterized by the
source viewBox.

### 4. Font corpus discovery is duplicated

`hello-ui-font` and `hello-ui-font2` independently search parent directories and
the executable path for font files. This is useful application bootstrap code,
but it should not be duplicated across corpus examples.

Preferred direction:

- the preparation script emits a manifest of named font assets;
- `ui-tools` provides a small corpus/provider resolver;
- examples select a named font or manifest-relative path.

### 5. Font bitmap-to-texture conversion remains example-owned

`UiFontRasterizer` and shared line rasterization now live in `ui-tools`, but
the examples still manually convert alpha masks into RGBA texture data and
upload those textures.

`ui-tools` should expose a renderer-neutral RGBA bitmap conversion or texture
payload type. It should not depend on `WgpuBackend`; the example should remain
responsible for the actual renderer upload.

### 6. `hello-ui-glyph-corpus` is a corpus browser, not a raster proof

The example discovers font and icon filenames but renders its rows using the
bitmap text system. It currently proves corpus discovery, paging, and row
selection, not TTF/OTF rasterization or SVG rendering.

This is acceptable if intentional, but the design document and title should
describe it as a corpus browser. Rasterization proofs belong in the dedicated
font and Lucide examples.

### 7. `ui-tools/DESIGN.md` no longer matches the implementation

The design document describes text rendering as a non-goal, while `ui-tools`
now owns:

- `UiFontRasterizer`;
- glyph bearings and font metrics;
- shared text layout;
- complete line bitmap compositing;
- SVG parsing and stroke tessellation.

The document should distinguish three layers:

1. semantic UI text and layout contracts;
2. renderer-neutral rasterization and geometry services;
3. platform/backend texture and mesh submission.

### 8. The legacy Lucide example has its own parser

`hello-ui-lucide` should remain a small five-icon smoke test, but it should
consume the same `ui-tools` SVG loader and tessellator as `hello-ui-lucide2`.
Its custom path parser is an architectural fork and should be removed.

## Ownership Boundary

### `ui-tools` should own

- SVG tokenization and command parsing;
- SVG document extraction;
- primitive conversion;
- viewBox normalization;
- path flattening;
- connected stroke tessellation;
- font metrics and glyph rasterization;
- line layout and baseline placement;
- renderer-neutral bitmap payload conversion;
- reusable coordinate and hit-test math.

### Examples should own

- which corpus files are selected;
- page and grid composition;
- example-specific colors and presentation;
- interaction used to inspect a corpus item;
- renderer upload and draw submission;
- corpus-specific acceptance observations.

## Recommended Refactor Order

1. Add shared SVG document extraction and viewBox normalization to
   `ui-tools/src/svg.rs`.
2. Replace both Lucide example parsers with the shared API.
3. Remove hand-authored fallback shapes from `hello-ui-lucide2`.
4. Add shared font corpus manifest/path resolution.
5. Add a renderer-neutral RGBA bitmap conversion helper for rasterized text.
6. Update `ui-tools/DESIGN.md` to describe the proven raster and SVG services.
7. Add regression tests for SVG primitive extraction, viewBox normalization,
   glyph line compositing, and baseline placement.

## Success Criteria

The boundary is healthy when:

- both Lucide examples use the same SVG loading and tessellation path;
- no example parses SVG path data independently;
- Lucide2 contains provider data rather than hand-authored replacement shapes;
- both font examples use shared font resolution and raster contracts;
- glyph lines are baseline-aligned without example-specific metric formulas;
- `ui-tools` remains renderer-neutral while examples perform backend uploads;
- corpus examples remain focused on proving data and behavior rather than
  reimplementing shared services.
