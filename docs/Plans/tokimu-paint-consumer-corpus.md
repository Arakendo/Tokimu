# Tokimu Paint Consumer Corpus

## Status

| Field | Value |
| --- | --- |
| Status | Public v1 proof deployed; corpus hardening and admission review remain |
| Consumer tier | Tier 2: incubating consumer |
| Target | Rust/WASM application embedded as an optional `tokimuengine.org` island |
| Public evidence | `https://tokimuengine.org/lab/paint/` |
| Last reviewed | 2026-07-31 |
| Source discussion | `docs/Conversations/Tokimu Paint.md` |
| Related plans | `consumer-corpora.md`, `tokimu-website.md`, `typescript-shader-material-presentation-control.md` |
| Related corpus | `corpus/lib/raster-image-corpus`, `corpus/hello-raster-image` |
| Related review | `AR-0006-raster-image-requirement-pipeline.md` |

### Public Proof Checkpoint

The first public Paint proof is deployed at
`https://tokimuengine.org/lab/paint/`. The deployed consumer demonstrates:

- a Rust/WASM-owned editable image, command history, preview, and PNG export;
- TypeScript-owned pointer, keyboard, file, toolbar, zoom, and download
  mechanisms;
- Canvas as a copied-pixel presentation target rather than the authoritative
  document or save source;
- pencil, eraser, bounded brush size, exact-match fill, sampling, new/open,
  undo/redo, reset, zoom/fit, and deterministic PNG export;
- state-aware undo and redo controls and a visually bounded document canvas;
- Lucide-backed toolbar presentation without making icon identity part of the
  Paint engine contract;
- a durable static page, bounded payload checks, automated website publication,
  and teardown paths for the website island and standalone workbench.

This checkpoint is public consumer evidence, not capability graduation. Native
and WASM parity artifacts, structured session diagnostics, formal browser
coverage, separately stored screenshots, and the admission review remain open.

## Purpose

Tokimu Paint is a small, deliberately ordinary raster editor used to pressure
Tokimu's image, asset, presentation, diagnostics, WASM, and website-consumer
boundaries through a useful application.

The first version is not a material painter and does not attempt to reproduce
every feature of Microsoft Paint. It proves a narrower lifecycle:

```text
encoded image bytes
        ↓
provider-neutral DecodedImage
        ↓
editable raster document
        ↓
deterministic edit commands
        ↓
bounded undo history
        ↓
preview and export
```

The central architectural question is:

> Can a browser application open, edit, undo, inspect, and save raster content
> through bounded Tokimu-owned semantics without making TypeScript, Canvas, an
> encoded image format, or the renderer the owner of image meaning?

Tokimu Paint should also become a practical repository tool. Maintainers should
eventually be able to use it to inspect alpha, make small fixture edits, and
exercise the same image contracts consumed by textures and shaders.

## Primary Composition Claim

```text
file selection / pointer / keyboard
                ↓
       TypeScript browser adapter
                ↓
        Rust/WASM PaintSession
                ↓
     editable raster document + history
                ↓
 provider-neutral preview observation
                ↓
      Canvas presentation + download
```

At no point does:

- TypeScript decode PNG, JPEG, or BMP;
- Canvas become the authoritative pixel store;
- browser image decoding silently replace Tokimu decoding;
- an encoded source format define editing semantics;
- the renderer mutate editable document truth; or
- undo restore presentation pixels instead of document state.

## Why This Is A Consumer Corpus

Focused raster corpus tests already ask whether bounded providers can decode
representative PNG, JPEG, and BMP inputs into a normalized `DecodedImage`.

Tokimu Paint asks a different question:

> Can a downstream application compose decoding, asset identity, mutable image
> state, commands, history, presentation, diagnostics, export, and browser
> lifecycle into a coherent tool?

The application-shaped pressure is intentional. It can expose:

- whether `DecodedImage` is being asked to own mutation it should not own;
- whether edit commands are deterministic and independently testable;
- whether undo history captures application meaning or renderer artifacts;
- whether preview and export agree about orientation, color, and alpha;
- whether large images or histories are bounded before allocation;
- whether WASM copies and serialized observations remain practical;
- whether browser mechanisms leak upward into image semantics; and
- whether the website island contract is sufficient for a stateful tool.

This repository-owned consumer remains architectural evidence. It is not an
independent production consumer and does not promote an image-editing capability
by itself.

## Consumer Classification

The first implementation is Tier 2 because it may consume:

- `corpus/lib/raster-image-corpus` for incubating provider-neutral decoding;
- existing website island infrastructure;
- public Tokimu asset, diagnostic, and WASM boundaries where available.

Every dependency on `corpus/lib` must remain named in the consumer's
`DESIGN.md`. The application must not present those APIs as stable Tokimu
contracts.

## Ownership

### The Paint application owns

- document lifecycle and dirty state;
- current tool, foreground color, and bounded tool options;
- edit command meaning;
- undo and redo transaction boundaries;
- selection meaning if selection is admitted later;
- save intent and unsaved-change warnings;
- application-specific status and workflow.

### Rust/WASM owns

- the authoritative editable pixel document;
- deterministic rasterization of admitted edit commands;
- pixel sampling and flood-fill semantics;
- bounded history application and reversal;
- normalized image dimensions, stride, orientation, color interpretation, and
  alpha metadata;
- provider-neutral snapshots and diagnostics;
- export preparation and encoding invocation below the application request.

### Raster providers own

- validation and decoding of encoded PNG, JPEG, and BMP bytes;
- format-specific metadata inspection;
- encoding only for explicitly admitted output providers;
- format-specific diagnostics below the provider-neutral boundary.

Raster providers do not own tools, document history, dirty state, or browser
download behavior.

### TypeScript owns

- island mounting and release;
- browser file selection, drag/drop, pointer capture, keyboard shortcuts, and
  resize observation;
- translating browser input into bounded semantic commands;
- toolbar, palette, status, and diagnostic presentation;
- requesting preview observations from Rust/WASM;
- creating a browser download from explicitly returned export bytes;
- accessible focus and lifecycle reporting.

TypeScript does not own pixel mutation, flood fill, undo, source decoding, or
encoded output semantics.

### Canvas owns

- pixels presented to the visitor, not editable image truth.

Canvas may scale or checkerboard the preview. It must not become the source used
for undo, save, color sampling, or subsequent editing.

### The website owns

- durable static explanation and screenshots;
- explicit activation of the Paint island;
- maturity and limitation labels;
- generated WASM and JavaScript assets;
- island failure, reset, and release behavior.

## Candidate Editable Image Boundary

`DecodedImage` is evidence produced by a decoder. Editing introduces different
semantics:

```text
DecodedImage
    immutable normalized decoder result

EditableRasterDocument
    mutable application document
    dimensions and pixel format
    authoritative pixel storage
    dirty revision
    bounded history
```

`EditableRasterDocument` is a provisional name, not an admitted Tokimu type.
The first implementation should keep it local to the Paint consumer or a
clearly labeled incubating corpus library.

The distinction must remain visible even if both types initially hold top-down
RGBA8 pixels. Shared storage shape does not imply shared ownership.

The initial editable profile is:

- top-down rows;
- tightly packed RGBA8 pixels;
- explicit `ColorSrgb` interpretation;
- straight alpha;
- checked non-zero dimensions;
- checked byte size;
- deterministic integer pixel coordinates.

Linear/data images, indexed palette preservation, higher precision, HDR, and
color-profile conversion remain explicit non-goals for the first profile.

## Edit Command Model

Browser pointer events must lower into semantic edit commands. They must not
directly mutate a shared pixel buffer.

The initial candidate commands are:

```text
PencilStroke
    points
    color
    diameter

EraseStroke
    points
    diameter

DrawLine
    start
    end
    color
    diameter

DrawRectangle
    bounds
    stroke
    optional fill

FloodFill
    origin
    replacement color
    match policy

SampleColor
    point
```

The first implementation may begin with pencil, eraser, flood fill, and color
sampling. Line and rectangle should follow once the command/history boundary is
proven.

Rules:

- coordinates are validated before mutation;
- non-finite or out-of-range input is rejected;
- stroke interpolation is deterministic and owned by Rust;
- one pointer gesture becomes one undoable transaction;
- flood fill has an explicit connectivity policy;
- replacement with an identical color is a deterministic no-op;
- commands either commit completely or leave the document unchanged;
- command diagnostics identify the rejected command and reason.

The first flood-fill profile should use exact RGBA matching and four-connected
neighbors. Tolerance, perceptual matching, and alpha-special behavior remain
future policy rather than implicit heuristics.

## History Model

Undo history is application state, not a stack of screenshots owned by Canvas.

The first proof should compare two implementation strategies with evidence:

- reversible pixel patches storing changed spans and their prior values; and
- bounded whole-document snapshots for deliberately small documents.

The selected strategy must expose:

- current document revision;
- undo and redo depth;
- bytes retained by history;
- configured history budget;
- whether the oldest transaction was evicted;
- deterministic behavior after undo followed by a new edit.

History must be bounded by bytes and transaction count. A single edit that
cannot fit within policy must fail explicitly or commit without history only if
the application requested that behavior. It must never silently allocate an
unbounded copy.

## Import Semantics

The first browser proof should admit PNG, baseline JPEG, and bounded BMP through
the existing raster corpus providers.

All admitted inputs lower into the same editable document profile:

```text
PNG  ─┐
JPEG ─┼─▶ DecodedImage ─▶ EditableRasterDocument
BMP  ─┘
```

Format-specific behavior remains diagnostic:

- JPEG input is opaque and lossy; the document does not pretend to retain JPEG
  coefficients or exact recompression identity.
- Indexed PNG or BMP input expands to RGBA8; palette identity is not preserved
  in the first version.
- BMP row direction is normalized before editing.
- PNG alpha remains straight alpha under the first admitted profile.
- source metadata may be displayed but does not become editable document truth
  unless separately admitted.

Browser-native image decoding may be used only as separately labeled
differential evidence. It must not become fallback execution.

## Export And Save Semantics

Browser download is a mechanism. Save format, color policy, alpha policy, and
encoding diagnostics belong below that mechanism.

The initial export policy should be:

1. admit one deterministic lossless output provider;
2. prove round-trip dimensions, orientation, and pixels;
3. add other formats only with explicit semantics.

PNG is the preferred first public output because it preserves the editable
RGBA8 profile. A bounded BMP encoder may be used first as a smaller mechanical
proof if PNG encoding is not yet available, but the website must label the
format honestly.

The first implementation is an application-local deterministic RGBA8 PNG
encoder. It uses explicit filter-none scanlines, a bounded output policy, and
the existing raster PNG decoder for round-trip evidence. It is an admitted
Paint export provider, not a promoted general-purpose image encoding contract.

JPEG export is deferred until the application can request and report:

- quality;
- alpha flattening behavior and background color;
- color interpretation;
- lossy-save warning;
- independent tolerant decode comparison.

Save must never claim to preserve the original encoding. Opening a JPEG and
saving a PNG is a document export, not a source-file round trip.

## WASM Contract

The first bounded API should remain small and data-oriented:

```text
PaintSession(config)

open(bytes, format_hint?) -> DocumentObservation
new_document(width, height, color) -> DocumentObservation
apply(command) -> EditObservation
undo() -> EditObservation
redo() -> EditObservation
sample(point) -> ColorObservation
preview() -> PixelObservation
export(request) -> ExportObservation
reset()
dispose()
```

The exact serialization strategy remains implementation evidence. Large pixel
buffers should not be encoded as JSON. Candidate transfer paths include:

- a bounded copied byte view exposed by WASM;
- a browser-owned `ImageData` populated from an explicit snapshot;
- a renderer texture path after a second consumer proves that need.

No public contract should expose decoder objects, DOM objects, `ImageBitmap`,
Canvas contexts, WGPU handles, or raw pointers with lifetime assumptions the
browser cannot enforce.

## Website Island

Tokimu Paint should be embedded as an explicitly activated island, like the
Asteroids consumer:

```html
<section
  data-tokimu-island="tokimu-paint"
  data-state="idle"
>
  <!-- Durable static fallback and limitations remain readable. -->
</section>
```

The island configuration should bound at least:

- maximum source bytes;
- maximum width and height;
- maximum decoded pixels;
- maximum history bytes;
- maximum transaction count;
- permitted input formats;
- permitted output formats.

Lifecycle requirements:

- activation initializes one session;
- reset releases the session before returning to idle;
- navigation and failed startup release listeners, pointer capture, object
  URLs, animation frames, and WASM resources;
- unsaved document state is never uploaded or transmitted;
- static fallback remains available if WASM cannot start.

Unlike Asteroids, Paint does not require a continuous animation loop while
idle. Presentation should redraw only after document, viewport, tool-preview,
or lifecycle changes.

## User Experience Direction

The interface should feel like a compact technical raster workbench rather than
a browser clone of a desktop ribbon:

- bounded canvas with checkerboard transparency;
- small tool rail;
- foreground color and compact palette;
- document dimensions, zoom, revision, dirty state, and history depth;
- visible diagnostics without allowing them to resize the canvas;
- explicit Open, New, Undo, Redo, Export, Reset, and Run/Stop controls;
- crisp nearest-neighbor zoom for pixel inspection;
- optional fitted preview distinct from document scale.

The first release should support:

- mouse, pen, and touch through Pointer Events;
- keyboard shortcuts for undo, redo, save/export, and tool selection;
- visible focus;
- textual status for screen readers;
- zoom without changing document pixels;
- high-contrast tool selection;
- reduced-motion behavior for any animated feedback.

## Determinism

- Pixel coordinates and command inputs are integer or deterministically
  quantized.
- Stroke interpolation and shape rasterization are Rust-owned.
- Flood fill has fixed connectivity and match policy.
- One fixture plus one command sequence produces the same document hash.
- Undo and redo restore exact document hashes.
- Export of a lossless format is deterministic where the selected provider
  promises deterministic bytes; otherwise decoded pixel equivalence is the
  authoritative comparison.
- Browser refresh timing does not alter document state.

## Diagnostics

Diagnostics should identify the owning stage:

```text
file acquisition
    browser mechanism

decode
    raster provider

document creation
    editable image boundary

command validation
    Paint semantics

history
    transaction and budget policy

preview transfer
    WASM/browser boundary

export preparation
    image semantics

encode
    output provider

download
    browser mechanism
```

The first stage whose observation diverges is the owning diagnostic boundary.
The UI should summarize repeated diagnostics and keep detailed occurrences
available without allowing verbose output to expand the preview region.

## Performance And Resource Budgets

Initial values are corpus policy, not universal engine guarantees.

The first implementation should record:

- decode time;
- document creation time;
- command application time;
- flood-fill visited pixels;
- changed pixels per transaction;
- history bytes and eviction count;
- preview transfer bytes and duration;
- Canvas upload/presentation duration;
- export preparation and encoding duration;
- WASM and JavaScript payload sizes.

Required bounds:

- no unchecked dimension multiplication;
- no unbounded flood-fill queue;
- no unbounded history;
- no per-frame full-document copy while idle;
- no repeated decoder initialization for ordinary edits;
- no leaked object URLs or browser listeners;
- sustained performance misses emit bounded Tokimu diagnostics where the
  producer can measure them honestly.

## Security And Privacy

- Files remain local to the visitor's browser.
- The application performs no upload or telemetry.
- Encoded input is treated as untrusted.
- Source bytes, dimensions, decoded bytes, command counts, and history are
  bounded before allocation.
- Malformed input returns a visible diagnostic without panic.
- Export filenames are sanitized by the browser adapter.
- Clipboard support is deferred until permission, format, and ownership policy
  are explicit.

## Proposed Repository Shape

```text
corpus/
  consumers/
    tokimu-website-paint/
      DESIGN.md
      README.md
      build.ps1
      package.json
      tsconfig.json
      engine/
        Cargo.toml
        src/lib.rs
      web/
        index.html
        paint.ts
        styles.css

website/
  docs/
    lab/
      paint.md
```

Generated `dist/`, WASM build output, and temporary edited images remain
ignored. Website deployment copies only reviewed generated island assets.

## Implementation Slices

### Slice 0: Boundary Record And Fixtures

Deliverables:

- [x] Create the consumer `DESIGN.md` with its Tier 2 dependencies.
- [x] Select one small RGBA PNG, one opaque baseline JPEG, and one bounded BMP
      already admitted by the raster corpus.
- [x] Add one Tokimu-authored transparent editing fixture.
- [x] Define document, command, history, preview, and export observation
      ownership; only the document observation is implemented in this slice.
- [x] Record unsupported formats and editing semantics.

Acceptance criteria:

- [x] Every state and operation has a named owner.
- [x] Fixture provenance remains traceable to the raster corpus.
- [x] `DecodedImage` and editable document state remain distinct.
- [x] The plan does not promote a shared editable-image capability.

### Slice 1: Headless Editable Document

Deliverables:

- [x] Implement a bounded application-owned editable RGBA8 document.
- [x] Create a document from normalized `DecodedImage`.
- [x] Create a blank document without an encoded provider.
- [x] Add revision, dirty state, exact pixel hash, and structural observation.
- [x] Add checked coordinate and allocation validation.

Acceptance criteria:

- [x] Document tests require no window, GPU, DOM, or live renderer.
- [x] Equivalent PNG, JPEG, and BMP decoded observations enter the same
      editable profile.
- [x] Source `DecodedImage` remains unchanged after edits.
- [x] Invalid dimensions and coordinates fail deterministically.

### Slice 2: Pencil, Eraser, Fill, And Sampling

Deliverables:

- [x] Add deterministic pencil and eraser strokes.
- [x] Add exact-match four-connected flood fill.
- [x] Add provider-neutral color sampling.
- [x] Define one pointer gesture as one transaction.
- [x] Emit changed bounds and changed-pixel counts.

Acceptance criteria:

- [x] Identical command sequences produce identical document hashes.
- [x] No-op commands preserve revision and dirty state.
- [x] Fill is bounded and cannot recurse through the native call stack.
- [x] Tool behavior is independent of browser event frequency after input is
      lowered into the same semantic command.

### Slice 3: Undo And Redo

Deliverables:

- [x] Implement bounded transaction history.
- [x] Add undo, redo, branch-after-undo, and reset behavior.
- [x] Record retained bytes, depth, and evictions.
- [x] Add deterministic history fixtures for stroke and flood fill.

Acceptance criteria:

- [x] Undo restores the exact prior document hash.
- [x] Redo restores the exact committed document hash.
- [x] A new edit after undo clears the obsolete redo branch.
- [x] History never exceeds configured byte or transaction policy.

### Slice 4: Shape Tools And Command Hardening

Deliverables:

- [x] Add line and rectangle commands.
- [x] Add bounded stroke diameter.
- [x] Add clipping for commands crossing document bounds.
- [x] Add malformed, excessive, and adversarial command fixtures.

Acceptance criteria:

- [x] Shape output is deterministic across native tests; WASM parity remains a
      later session-boundary check.
- [x] Clipping never writes outside the document.
- [x] Invalid values fail before mutation.
- [x] A failed command leaves revision, pixels, and history unchanged.

### Slice 5: Lossless Export

Deliverables:

- [x] Select and document the first lossless output provider.
- [x] Export the editable document without Canvas readback.
- [x] Decode the result through an independent or existing admitted path.
- [x] Compare dimensions, orientation, alpha, and exact pixels.
- [x] Return bounded bytes and export diagnostics through the session API.

Acceptance criteria:

- [x] Exported lossless pixels equal the authoritative document.
- [x] Transparent pixels survive round trip.
- [x] Original source encoding is never claimed to be preserved.
- [x] Encoding failure does not clear dirty state or mutate the document.

### Slice 6: Rust/WASM Session

Deliverables:

- [x] Add the `cdylib`/`rlib` consumer package.
- [x] Expose bounded open, new, apply, undo, redo, sample, preview, export,
      reset, and dispose operations.
- [x] Avoid JSON for large pixel and export buffers.
- [x] Add native and `wasm32-unknown-unknown` contract validation.
- [x] Add malformed boundary-input tests.

Acceptance criteria:

- [ ] Native and WASM command sequences produce equivalent observations.
- [x] Provider-native and browser-native objects do not cross the API.
- [x] Large buffers have explicit copied-buffer ownership and session lifetime.
- [x] Disposed sessions reject further operations predictably.

### Slice 7: Standalone Browser Workbench

Deliverables:

- [x] Build the bounded Canvas preview, tool rail, palette, and status region.
- [x] Add file selection and drag/drop for admitted image formats through the
      same bounded Rust/WASM decode path.
- [x] Add pointer capture.
- [x] Add keyboard shortcuts for tools, undo, redo, and export.
- [x] Add zoom, fit, and checkerboard transparency without changing pixels.
- [x] Add browser download for admitted export bytes.
- [x] Keep bounded browser observations scrollable and independent from canvas
      layout.
- [ ] Present structured Rust/WASM diagnostics when the session admits them.

Acceptance criteria:

- [x] The TypeScript adapter typechecks against the generated WASM contract.
- [x] Canvas is fed from copied Rust/WASM preview bytes and never becomes the
      authoritative edit or save source.
- [x] A public browser run proves a user can open, draw, fill, sample, undo, redo,
      and export.
- [ ] Pointer, keyboard, touch, and resize mechanisms release on disposal.
- [x] Verbose browser observations cannot resize or cover the editing viewport.
- [ ] Structured Rust/WASM diagnostics remain independently bounded when they
      are admitted by the session contract.

### Slice 8: Website Island Integration

Deliverables:

- [x] Register `tokimu-paint` with the shared island loader.
- [x] Add a durable static Paint page with capability and limitation text.
- [x] Require explicit activation.
- [x] Publish reviewed generated WASM and TypeScript assets through the website
      build script.
- [x] Record deterministic payload limits and diagnostic startup observations.
- [x] Add reset and navigation teardown evidence through the shared controller
      and standalone pagehide disposal.

Acceptance criteria:

- [x] The page remains useful without JavaScript or WASM.
- [x] Activation creates one bounded session.
- [x] Reset and navigation release all owned resources.
- [ ] No image bytes leave the browser.
- [x] The website labels Paint as experimental consumer evidence.

### Slice 9: Corpus Evidence And Hardening

Deliverables:

- [x] Add serializable deterministic blank-document command replays with terminal
      document and history observations.
- [ ] Emit document, history, export, and performance artifacts.
- [ ] Capture separately labeled browser screenshots.
- [x] Add malformed files, oversized documents, fill stress, alpha, and odd
      dimensions at the headless document boundary.
- [x] Compare preview pixels and fingerprints with authoritative document snapshots.
- [ ] Record native/WASM parity and supported-browser observations.

Acceptance criteria:

- [ ] Regressions localize to decode, document, command, history, preview,
      encode, or browser ownership.
- [x] Structural assertions remain authoritative over screenshots.
- [x] Correctness tests remain headless and deterministic where meaningful.
- [x] Performance and memory policies fail visibly rather than degrading
      without diagnosis.

### Slice 10: Admission Review

Deliverables:

- [ ] Compare Paint evidence with raster, shader texture, asset, screenshot,
      and other image consumers.
- [ ] Decide whether editable image, edit command, history, or encoder
      semantics have independent consumers.
- [ ] Record application-local glue and repeated ownership friction.
- [ ] Update AR-0006 or open a focused review if evidence changes the raster
      requirement boundary.
- [ ] Update the SDD or ADRs only for accepted architectural decisions.

Acceptance criteria:

- [ ] No capability graduates solely because Paint needs it.
- [ ] Accepted, deferred, and rejected findings are explicit.
- [ ] Application workflow remains separate from provider-neutral image
      semantics.
- [ ] Future material-painting work can reuse admitted image contracts without
      inheriting Paint-specific tools or UI.

## Initial Release Definition

The first useful public Paint island is complete when a visitor can:

- explicitly activate the island;
- create a small blank image or open an admitted PNG, JPEG, or BMP;
- draw with a pencil and eraser;
- use exact-match flood fill and color sampling;
- undo and redo bounded transactions;
- inspect dimensions, revision, dirty state, history, and diagnostics;
- export one lossless image format;
- reset and release the application cleanly.

This public interaction checklist was demonstrated on the deployed website on
2026-07-31. It records an observed consumer proof, not a cross-browser guarantee.

The release must also prove:

- Rust/WASM owns authoritative pixels and edit semantics;
- TypeScript owns browser interaction only;
- Canvas owns presentation only;
- input and history remain bounded;
- static website content survives interactive failure;
- structural replay evidence can reproduce the edited result.

## Deferred Growth

Later corpus cycles may consider:

- selection and clipboard;
- copy, cut, paste, crop, resize, and rotate;
- tolerance-based fill;
- indexed and palette-preserving documents;
- alpha-aware compositing tools;
- layers;
- text;
- image transforms and filters;
- JPEG export;
- material slots and live shader preview;
- procedural textures;
- normal, mask, and height-map semantics;
- native desktop hosting;
- shared editing sessions;
- baking and material-painting workflows.

Each addition must identify the semantic boundary it pressures. The application
must not become a feature backlog whose only justification is resemblance to a
desktop paint program.

## Non-Goals

The first implementation does not provide:

- a general document framework;
- a promoted image-editing crate;
- layers;
- arbitrary filters;
- vector drawing semantics;
- text layout;
- color-managed professional editing;
- source palette or JPEG coefficient preservation;
- runtime JavaScript image processing;
- browser-native decoder fallback;
- GPU compute editing;
- collaborative editing;
- cloud storage;
- telemetry or upload;
- a material painter.

## Risks

### `DecodedImage` Becomes Mutable Application State

Mitigation: require an explicit document creation boundary and prove source
decoded evidence remains unchanged.

### Canvas Becomes The Hidden Document

Mitigation: sample, undo, save, and hash only Rust-owned document pixels.

### Pointer Frequency Changes The Image

Mitigation: lower gestures to bounded semantic point sequences and keep stroke
rasterization deterministic in Rust.

### History Copies Exhaust Memory

Mitigation: enforce byte and transaction budgets, report retained memory, and
compare snapshots with changed-span patches before stabilizing a model.

### Browser Mechanisms Define Semantics

Mitigation: keep file selection, pointer events, object URLs, and downloads in
TypeScript while Rust owns decoding, editing, and export intent.

### Paint Prematurely Promotes A Universal Document Model

Mitigation: keep the first document and history implementation application
owned; seek independent pressure before extraction.

### Public Demo Overstates Format Fidelity

Mitigation: publish explicit maturity labels for decode, editable
normalization, metadata loss, and output support.

## Graduation Evidence

Paint may support admission only when:

- another independent consumer needs the same provider-neutral editable image
  semantics;
- raster providers, shader textures, screenshots, and Paint agree on image
  interpretation without provider leakage;
- headless edit commands and history remain usable without renderer startup;
- native and WASM consumers produce equivalent document observations;
- repeated application glue has a clear Tokimu owner;
- an Architectural Review records accepted ownership and dependency direction.

Until then, Paint remains a valuable application and a source of evidence, not
proof that every local abstraction belongs in Tokimu.

## References

- [`Tokimu Paint` conversation](../Conversations/Tokimu%20Paint.md)
- [`Consumer Corpora`](consumer-corpora.md)
- [`Tokimu Website`](tokimu-website.md)
- [`Raster Image Corpus Testing`](../Libraries/raster-image-corpus-testing.md)
- [`AR-0006: Raster Image Requirement Pipeline`](../Architectural%20Reviews/AR-0006-raster-image-requirement-pipeline.md)
- [`Website Asteroids Consumer`](../../corpus/consumers/tokimu-website-asteroids/DESIGN.md)
- [`Interactive Island Contract`](../../website/docs/lab/island-contract.md)
- [`ADR-0001: Engine Boundaries`](../ADR/ADR-0001-engine-boundaries.md)
- [`ADR-0003: Capability Ownership Boundary`](../ADR/ADR-0003-capability-ownership-boundary.md)
- [`ADR-0007: Kernel Performance Diagnostics`](../ADR/ADR-0007-kernel-performance-diagnostics.md)
