# Terminal Surface Provider Study

| Field | Value |
| --- | --- |
| Status | Active research corpus |
| Owner | Tokimu maintainers |
| Related review | `AR-0014-native-terminal-text-surface-and-ratatui-dependency-boundary.md` |
| Related ADRs | ADR-0003, ADR-0004, ADR-0005 |
| Corpus entry | `corpus/focused/observation/hello-terminal-surface` |

## Purpose

Study whether terminal-shaped providers can expose a bounded resolved surface
that Tokimu can present without Tokimu owning terminal layout, shell meaning,
or Ratatui internals.

This is an evidence plan. It does not admit a new crate, a public API, or a
Tokimu-native terminal implementation.

## Research Abstract

Terminal providers and ordinary text layout solve different problems. A
terminal provider resolves a two-dimensional cell grid after deciding width,
wrapping, continuation cells, cursor placement, styles, and damage. Tokimu can
then rasterize the already-resolved surface through its presentation stack
without reinterpreting those terminal decisions.

The corpus tests that handoff as a local candidate model first. Ratatui is one
future producer, not the definition of the model. A deliberately minimal
independent producer is the comparison point that prevents Ratatui types or
behavior from becoming an accidental public contract.

## Ownership Boundary

```text
Shell/session provider
    owns commands, transcript meaning, and session state

Terminal provider
    owns layout, width, wrapping, continuation cells, cursor, and cell styles

Terminal-surface study
    owns local snapshot/delta reconstruction and rejection evidence

Tokimu presentation
    owns font resolution, glyph rasterization, clipping, and renderer commands

Host adapter
    owns normalized input, focus, resize delivery, and pixel presentation
```

Tokimu must not become a terminal emulator merely because it can draw a
terminal-shaped surface. Ratatui must not become a dependency of `tokimu-core`
or `tokimu-runtime`.

## Candidate Research Vocabulary

The following names are corpus-local and intentionally unstable:

```text
SurfaceExtent
SurfaceEpoch
ResolvedCell
CellContent::Grapheme | Continuation | Empty
CursorState
FullFrame
ChangedCells
TerminalSurfaceObservation
```

They are useful only if multiple producers converge on the same semantics.

## Provider Extraction Hypothesis

"Extracting" useful terminal concepts must not mean copying, forking, or
rebranding Ratatui internals. The current hypothesis is narrower: Tokimu may
eventually own a first-party, provider-neutral handoff contract that Ratatui
and a smaller provider can each implement.

| Candidate | Current evidence | Status |
| --- | --- | --- |
| Bounded extent, cells, styles, cursor, and full/delta frames | Independent fixture and Ratatui adapter reconstruct the same retained surface. | Corpus-local candidate |
| Delta validation and retained-surface reconstruction | Both producers exercise epoch, extent, continuation, and invalid-update rules. | Corpus-local candidate |
| CPU raster lowering of resolved cells | The same resolved surface reaches Departure Mono raster evidence without re-running terminal layout. | Corpus-local adapter |
| Ratatui widgets, layout DSL, `Buffer`, and backend traits | These remain Ratatui implementation mechanics. | Explicitly excluded |
| Terminal host, PTY, ANSI parsing, command authority, and transcript meaning | These belong to host or shell/session semantics, not a cell surface. | Explicitly excluded |
| Unicode width, segmentation, shaping, wrapping, and continuation production | Producers decide these before emitting cells; the surface preserves their result. | Provider-owned pending wider evidence |

For browser deployment, consumers must choose and measure the smallest
provider composition they actually need. An island that only displays a
pre-rasterized or independently produced surface must not accidentally link
Ratatui. A browser island that genuinely needs Ratatui layout may opt into a
separate feature and carries its own recorded payload budget. Native consumers
may select richer optional providers, but still retain explicit build and
runtime evidence rather than treating native deployment as budget-free.

Graduation requires more than this local agreement:

- a second non-Ratatui producer with meaningful styled-cell or cursor pressure;
- a web-exported Ratatui path whose linked payload, startup, and warm-frame
  observations are measured separately from the independent fixture;
- evidence that the candidate contract does not leak Ratatui types or duplicate
  provider layout decisions; and
- an AR-0014 decision on continued incubation, capability admission, or
  retirement.

## Lifecycle Hypothesis

```text
producer resize or reset
    -> increment epoch
    -> emit complete frame

producer ordinary update
    -> emit changed cells for the same epoch and extent

consumer
    -> rejects a delta before a complete frame
    -> rejects mismatched epoch or extent
    -> never renders a continuation cell as an independent grapheme
```

This permits a renderer-facing adapter to retain a stable surface without
asking it to reconstruct terminal layout from text runs.

## Non-Goals

- No public terminal-surface crate or stable public API.
- No terminal escape-sequence parser or terminal host.
- No Unicode shaping or width algorithm owned by Tokimu.
- No Ratatui extraction, fork, or copied internals.
- No claim that normal text layout and terminal cell layout are interchangeable.

## Slices

### Slice 1: Local Surface Reconstruction

- [x] Add a headless corpus model with full-frame and delta updates.
- [x] Reject deltas before a baseline or with a mismatched extent or epoch.
- [x] Preserve continuation cells as layout metadata rather than glyphs.

Acceptance criteria:

- [x] Tests prove a baseline plus delta reconstructs deterministic cells.
- [x] Tests prove reset invalidates prior deltas.
- [x] No Ratatui dependency is needed for this slice.

### Slice 2: Resizing And Damage Evidence

- [x] Add deterministic resize, cursor, clipping, and changed-cell fixtures.
- [x] Record full versus delta damage statistics.

Acceptance criteria:

- [x] A resize forces a complete replacement surface.
- [x] Invalid damage is diagnosed at the surface boundary without partially
      mutating the retained surface or incrementing accepted-damage statistics.

### Slice 3: Ratatui Producer Adapter

- [x] Make a corpus-only Ratatui adapter emit the candidate observation.
- [x] Compare its full and incremental observations against the local model.
- [x] Preserve Ratatui-resolved foreground, background, and supported emphasis
      modifiers in the corpus-local cell style.

Acceptance criteria:

- [x] No Ratatui type crosses a Tokimu public boundary.
- [x] Cursor and styled-cell evidence are retained.
- [x] Clipping evidence is retained through a rendered Tokimu surface.

Current evidence: the adapter uses Ratatui's headless `TestBackend` to create
complete local cell frames and derives deltas from its changed cells. The
`TestBackend` is a composition oracle, not a pixel renderer. It carries a
fixture-declared cursor because it is not a terminal host.
Provider-resolved foreground, background, and supported emphasis modifiers now
survive full-frame and style-only delta reconstruction. A bounded CPU raster
fixture now clips cells at its surface edge without reimplementing provider
width or wrapping rules. The shared `tui-tools` cell-to-RGBA seam is exercised
by both the native console corpus and the website Ratatui adapter; native, GPU,
and browser presentation observations remain distinct evidence.

### Slice 4: Independent Producer

- [x] Add a minimal non-Ratatui fixture producer with the same candidate model.
- [x] Compare reconstruction and rejection behavior with the Ratatui adapter.

Acceptance criteria:

- [x] Differences are recorded as provider behavior, candidate-contract gaps,
      or rejected semantics.
- [x] The study does not widen its model silently to accommodate one provider.

The minimal fixture producer deliberately emits only explicit ASCII-like
grapheme cells, default cell style, and a cursor. It does not model Ratatui
styles, width-aware glyph handling, wrapping, or widget layout. Those are
recorded as provider behavior and remaining candidate-contract pressure, not
silently promoted into the shared model. Both producers establish a full frame,
derive a delta, and exercise the same epoch, extent, continuation, and
rejected-update lifecycle.

### Slice 5: Presentation Evidence

- [x] Render a bounded surface through Tokimu text presentation.
- [x] Compile the corpus-local CPU presentation path as a `wasm32-unknown-unknown`
      `cdylib` that exposes only a bounded fixture summary.
- [x] Run the same independent fixture through a local browser/WASM presenter
      that blits opaque RGBA output only.

Acceptance criteria:

- [x] Tokimu rasterizes resolved cells only; it does not calculate terminal
      wrapping or width.
- [x] CPU evidence identifies the fixture producer, Ratatui producer, Departure
      Mono font provider, and the corpus-local CPU raster adapter.
- [x] Native CPU and browser/WASM evidence identify their separate ownership:
      Rust/WASM produces the bounded raster while the browser only displays it.
- [x] Submit the same CPU reference raster to a native renderer texture and
      record the resulting native execution observation.
- [ ] GPU framebuffer readback and pixel-equivalence remain separate from the
      CPU reference artifact, browser canvas presentation, and native texture
      submission.

Current evidence: `hello-terminal-surface` lowers only `SurfaceExtent` and
resolved cells through `ui-tools::UiFontRasterizer` with Departure Mono into a
bounded `288x120` RGBA surface. The independent fixture and the Ratatui adapter
produce deterministic fingerprints through that same adapter. This proves a
provider-neutral CPU presentation handoff, not GPU output, native/WASM parity,
or a terminal host. The same corpus package now builds a local browser presenter
from its WASM `cdylib`. The opaque exports provide only width, height, RGBA
bytes, and a diagnostic summary; browser execution confirmed the same
independent-fixture `288x120` raster and `851900b19a379bcd` CPU fingerprint as
native evidence. The browser blits those bytes through `ImageData` and does not
receive Ratatui, cell, or font-provider types. This is browser display evidence,
not native/GPU renderer equivalence. Cursor state and italic emphasis remain
retained model data; the current CPU adapter does not yet paint a cursor or
synthesize italics.

The native corpus binary now submits that already-authoritative RGBA reference
as one sRGB texture on one centered `Texture2d` quad. Its startup observation
records the CPU fingerprint, dimensions, and one texture upload while stating
`framebuffer_readback=false`. This is native execution evidence only; it does
not compare GPU pixels with the CPU artifact or establish native/WASM parity.

Release `wasm32-unknown-unknown` builds measured `336349` bytes with the
independent configuration and `336346` bytes with the optional
`ratatui-producer` feature. This near equality is intentional: the browser
`cdylib` exports only the independent fixture raster, so the optional Ratatui
producer is not linked into that public boundary. The measurement proves that
the public browser export does not accidentally absorb Ratatui. It does **not**
measure the dependency, startup, or frame cost of an executable Ratatui
producer, which remains required evidence before admission.

The matching native release corpus executables measured `6166016` bytes for
the independent configuration and `6278144` bytes with `ratatui-producer`
enabled: an executable increase of `112128` bytes. Unlike the browser build,
this comparison links the optional producer into a runnable corpus binary. It
is therefore useful dependency-size evidence, but it remains corpus-local: it
does not establish a production budget, startup cost, frame cost, or a public
Tokimu terminal-provider contract.

The corpus also exposes `cargo run -p hello-terminal-surface --features
ratatui-producer -- --measure`. It repeats the existing `READY` to `DONE`
full-frame/delta/reconstruction/CPU-raster path 256 times for each producer,
rejects an unstable raster result, and prints local elapsed and average times.
This is intentionally a comparative CPU-pipeline observation, not a CI timing
budget, native-window startup measurement, sustained GPU frame measurement, or
terminal-host benchmark.

The first local observation on 2026-08-06 measured `956 us` average for the
independent producer and `1786 us` average for the optional Ratatui producer
across 256 repetitions. The values make the optional producer's CPU-path cost
visible without treating one development machine as an architectural budget.

### Slice 6: Admission Review

- [x] Record the browser-export boundary measurement separately from actual
      provider-cost measurement.
- [x] Record a native executable dependency-size comparison where the optional
      producer is actually linked.
- [x] Add a repeatable, headless CPU producer/reconstruction/raster comparison
      for the same bounded extent.
- [x] Record the first local comparative CPU observation with its limitations.
- [x] Instrument native startup readiness and sustained renderer-frame CPU
      observations independently from the CPU pipeline comparison.
- [ ] Record a bounded native-window observation from the current development
      machine, including the startup-ready event and warm-frame resource churn.
- [x] Update AR-0014 with comparative cost and conformance evidence.
- [ ] Decide continued incubation, capability admission, or retirement.

Acceptance criteria:

- [ ] An ADR is opened only if a stable, provider-neutral ownership boundary
      has survived independent use.

## Graduation Trigger

Any permanent capability proposal requires all of the following:

- one Ratatui producer and one independent producer agree on the bounded
  surface semantics;
- conformance covers resize, cursor, clipping, continuations, styles, and
  changed-cell invalidation;
- no public API leaks Ratatui types or terminal-host assumptions;
- a real consumer needs the surface independently of the existing shell corpus;
- the comparative dependency, build, startup, and frame costs are recorded.

### Native Execution Instrumentation

The native corpus viewer now treats the resolved CPU raster as an ordinary
RGBA8 presentation input. It uploads that raster once, renders it through the
existing textured-quad path, and reports a startup-ready event plus periodic
warm-frame renderer observations. The report distinguishes CPU wall-clock
calls for surface acquisition, resource preparation, command encoding, queue
submission, and presentation, along with draw, submit, binding-allocation,
uniform-write, mesh-upload, and texture-allocation counts.

These values are deliberately CPU-side observations. They do not claim GPU
completion, frame-display latency, framebuffer readback, cross-platform
budgets, or standalone terminal-host behavior. A recorded local observation
remains pending.
