# AR-0001: Shared Vector Presentation Geometry

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-07-21 |
| Last reviewed | 2026-07-23 |
| Scope | Foundational presentation capability / backend boundary |
| Trigger | UI surface duplication and repeated SVG/Lucide stroke pressure |
| Related ADRs | ADR-0003, ADR-0004 |
| Related evidence | `hello-ui-box`, `hello-ui-lucide2`, UI and font-outline vector presentation plans |

## Architectural Question

Should UI surfaces and SVG/icon paths lower into one provider-neutral vector
geometry representation before renderer submission, while keeping vector
geometry incubated inside `ui-tools` until independent consumers justify
promotion?

## Context

`hello-ui-box` currently assembles surface shadows, borders, and fills from
example-local quads. The SVG/Lucide corpus has independently required connected
stroke joins, caps, closure handling, and curve cleanup. These are evidence of a
shared geometry concern, but not yet evidence that a first-party `tokimu-vector`
crate is ready.

The proposed ownership is:

```text
UI semantics -> UI-to-vector lowering -> vector geometry -> renderer
SVG importer -----------------------> vector geometry -> renderer
Font outline adapter ---------------> vector geometry -> renderer
```

UI remains responsible for meaning, themes, layout, and interaction. Vector
geometry owns paths and tessellation. Font and text services remain responsible
for font technology, metrics, shaping contracts, fallback, and positioned glyph
runs. Renderers own GPU execution.

## Emerging Admission Hypothesis

Current corpus pressure suggests that vector geometry may eventually graduate
as a native Tokimu presentation capability, while UI, SVG/font importers, font
providers, and icon providers remain services or adapters that consume it.

```text
Native Tokimu capability
    vector paths, contours, curves, fills, strokes, topology, tessellation

Services and adapters
    UI semantics ---------------------> vector
    SVG importer ---------------------> vector
    Lucide icon provider -------------> vector
    TTF/OTF outline adapter ----------> vector
    diagrams and technical drawing ---> vector

Execution
    vector geometry ------------------> renderer backend
```

Here, *native* does not mean *trusted core*. Vector presentation remains outside
`tokimu-core` and owns no simulation truth. The hypothesis is that paths and
tessellation are shared Tokimu-owned meaning, while UI controls, font formats,
font shaping, icon libraries, and document formats retain their independent
service semantics.

This is not yet the disposition of the review. The font-outline consumer is
planned but unproven, no non-example consumer depends on the contracts, and
broader fill topology and native-curve pressure remain under investigation.

## Trigger And Evidence

- Corpus examples: `hello-ui-box`, `hello-ui-lucide2`.
- Automated tests: existing `ui-tools` SVG parser and stroke tests.
- Repeated implementation friction: local box geometry, stroke repair work,
  and duplicated renderer-facing surface assembly.
- Remaining evidence gaps: general fill topology, semantic clip-to-scissor
  lowering,
  retained caching, and a second non-example consumer of stable vector
  contracts.
- Planned evidence: monochrome TTF/OTF glyph outlines lowering through an
  adapter into shared vector paths, without typography leaking into geometry.

## Ownership Analysis

The proposal concerns presentation geometry, not simulation truth. It is a
foundational presentation capability/backend boundary rather than a trusted
core primitive. The vector model must not own UI semantics, interaction state,
font/icon provider identity, or renderer-native buffers.

## Dependency Direction

```text
Current:
UiSurfaceCommand -> example-local quad assembly -> renderer
SVG commands     -> flattened point lists -> stroke helper -> renderer

Proposed:
UiSurfaceCommand -> UI-to-vector lowering -> VectorPath -> tessellator
SVG importer     -----------------------> VectorPath -> tessellator
Vector geometry  -> renderer-neutral mesh data -> renderer adapter
```

The first implementation remains in `examples/lib-example/ui-tools`. No new
engine crate is admitted by this review.

## Alternatives Considered

### A: Continue With Separate Quad And SVG Paths

- Benefits: no migration cost.
- Costs: duplicated geometry behavior and inconsistent joins/rounding.
- Failure mode: every new consumer repairs its own geometry pipeline.

### B: Create `tokimu-vector` Immediately

- Benefits: clear long-term package name and boundary.
- Costs: freezes contracts before fill, clipping, batching, and independent
  consumers have provided enough evidence.
- Failure mode: premature extraction makes an unstable prototype permanent.

### C: Incubate Shared Geometry In `ui-tools`

- Benefits: small compileable slices, real UI and SVG consumers, reversible
  packaging decision.
- Costs: temporary example-side location and some migration work later.
- Failure mode: the module grows without review if promotion triggers are not
  monitored.

## Findings

The evidence supports a shared provider-neutral geometry direction. UI boxes and
the Lucide corpus now consume the same `VectorPath` and stroke tessellator, and
the implementation has explicit clipping metadata and bounded batching. It does
not yet support a new first-party crate, arbitrary SVG fill support, retained
vector scene graphs, or a renderer-specific mesh/cache contract.

## Disposition

Incubating. The initial UI surface lowering and shared SVG consumption are now
proven in `ui-tools`; continue gathering evidence before extraction.

## Consequences

The implementation can improve UI and SVG together while keeping application
semantics independent of SVG and renderer details. It temporarily places the
prototype in an example support crate and requires explicit diagnostics for
unsupported topology and clipping.

## Required Follow-Up

- [x] Documentation or review record
- [x] Focused implementation slice
- [x] Corpus example or automated test
- [x] Migration and transitional-helper retirement

## Reopening Triggers

- UI and SVG both consume stable vector contracts without semantic leakage;
- font outlines independently consume the same path and tessellation contracts
  without moving metrics, shaping, fallback, or glyph identity into vector;
- a non-example consumer requires the same geometry API;
- unsupported topology blocks a real application consumer;
- renderer batching or clipping requires a boundary redesign;
- duplicate geometry generation remains in corpus applications after migration.

## Review History

### Cycle 1 -- 2026-07-21

- Status entering review: Proposed
- New evidence: UI-box vector plan and current `ui-tools`/Lucide corpus seams.
- Participants or reviewers: Codex working review
- Findings: shared geometry is justified for incubation; crate extraction is
  premature.
- Disposition: Incubating
- Resulting ADR or documentation change: implementation plan created; no ADR
  change required yet.

### Cycle 2 -- 2026-07-21

- Status entering review: Incubating
- New evidence: `hello-ui-box`, `hello-ui-lucide`, and `hello-ui-lucide2`
  now consume shared vector paths and tessellation; SVG path elements and
  primitive elements preserve explicit contour closure; duplicate example
  stroke helpers were removed.
- Automated evidence: 92 `ui-tools` tests pass, including fill, connected
  stroke, clipping metadata, SVG contour, primitive-element, and batch-helper
  coverage.
- Findings: the shared geometry boundary is useful and stable enough for the
  current consumers. Rectangular clipping is represented, and the renderer
  already supports pixel-space scissoring through `ViewportRect`; an explicit
  UI/world-to-pixel lowering step is still absent. Fill topology remains limited to
  convex single-contour paths. The legacy flattened SVG helpers remain only in
  internal regression tests.
- Disposition: remain Incubating; do not extract a first-party vector crate.
- Follow-up: collect upload/tessellation measurements and obtain a non-example
  consumer before proposing promotion.

### Cycle 3 -- 2026-07-21

- Status entering review: Incubating
- New evidence: renderer statistics now expose draw calls and cumulative mesh
  uploads. `hello-ui-box` captured 338 draw calls and 6 cumulative mesh
  uploads on its first frame. `hello-ui-lucide2` captured 100 icons, 100 draw
  calls, and 100 cumulative mesh uploads on its first frame and retained the
  same upload count on its second frame.
- Corpus measurement: the 100-icon Lucide setup reported approximately
  26.100 ms for parsing and tessellation in one native run.
- Findings: the current Lucide consumer retains uploaded geometry across
  frames; its remaining measured submission cost is one draw per icon. This is
  evidence for a future batching investigation, not a reason to expand the
  shared vector contract now.
- Additional renderer evidence: `hello-ui-box` uses stable batch handles and
  an example-local value cache; its second frame retains the initial six mesh
  uploads with zero replacements. Shared retained caching remains deferred and
  is not an admitted vector contract.
- Disposition: remain Incubating; do not extract a first-party vector crate.
- Follow-up: keep batching, clipping execution, broader fill topology, and
  non-example consumption as explicit promotion evidence requirements.

### Cycle 4 -- 2026-07-21

- Status entering review: Incubating
- New evidence: `UiRect::to_pixel_rect` now provides a provider-neutral,
  clamped world/UI-to-pixel conversion with explicit empty and off-screen
  behavior. `hello-ui-box` carries the converted result into renderer
  `ViewportRect` commands and keeps batches separate when their clip regions
  differ.
- Validation: `ui-tools` passes 96 tests; `hello-ui-box` compiles with the
  adapter integration.
- Findings: rectangular clipping is now demonstrated end to end for the
  current orthographic convention. Non-rectangular clipping, broader fill
  topology, batching, and independent consumption remain open evidence gaps.
- Disposition: remain Incubating; do not extract a first-party vector crate.
- Follow-up: use the remaining corpus work to pressure-test supported fill
  topology and draw batching before considering promotion.

### Cycle 5 -- 2026-07-21

- Status entering review: Incubating
- New evidence: UI surface lowering, SVG paths, Lucide icons, and font
  outlines now converge on the same provider-neutral path and tessellation
  boundary. Font outlines are the strongest independent evidence that the
  boundary is presentation geometry rather than an SVG-specific helper.
- Findings:
  - shared vector geometry is becoming a genuine capability candidate, but
    extraction remains deferred until a non-example consumer exists;
  - typography remains outside vector: text layout, baselines, glyph identity,
    fallback, and metrics stop before outline conversion;
  - general fill topology belongs with vector geometry, including contours,
    counters, winding, and concavity rather than with an individual importer;
  - geometry caching remains renderer-facing evidence, not an admitted vector
    responsibility;
  - curve representation remains undecided. Flattened paths are currently
    deterministic and adequate for the corpus, but shared native curves should
    only be considered if SVG and fonts demonstrate repeated pressure for them;
  - the review process itself is producing useful evidence through focused
    corpus tests and measurements rather than intuition alone.
- Disposition: remain Incubating; do not extract a first-party vector crate.
- Follow-up: continue the font-outline plan, compare curve pressure across SVG
  and fonts, and revisit promotion only after the graduation criteria or a
  non-example consumer is present.

### Cycle 6 -- 2026-07-23

- Status entering review: Incubating
- New evidence: `presentation-geometry-corpus` now observes glyph, synthetic,
  Lucide SVG, and UI producers through explicit source/vector/mesh stage
  reports. Eleven reviewed structural snapshots compare read-only, while glyph
  cases also compare normalized mesh fingerprints that do not make triangle
  emission order an early contract.
- Generated evidence: seeded polygon generation is deterministic and replayable
  from `(seed, index)`, with a bounded local count. Generated cases remain
  investigation inputs rather than reviewed golden cases.
- External evidence: the prepared 100-icon Lucide subset records its provider
  revision, selection rule, source path, and count. The SVG runner validates
  that provenance before accepting the source stage.
- Findings:
  - the corpus harness now exposes a repeatable diagnostic boundary across
    independent presentation producers;
  - structural reports, normalized mesh fingerprints, raw geometry, and saved
    CPU images serve different evidence purposes and should remain distinct;
  - generated replay is useful for focused topology investigation without
    expanding the permanent reviewed corpus;
  - Lucide provides pinned external data evidence, but not W3C conformance;
  - W3C subset import, differential rendering, native-curve representation,
    diagnostics ownership, and non-example consumption remain unresolved.
- Disposition: remain Incubating; do not extract a first-party vector crate.
- Follow-up: evaluate whether a small W3C fixture subset or optional
  differential renderer is worth its dependency cost, then revisit the open
  curve and diagnostics questions with evidence rather than assumption.

### Cycle 7 -- 2026-07-23

- Status entering review: Incubating
- New evidence: glyph corpus artifacts now include a deterministic
  `image-fingerprint.json` for the normalized CPU RGBA8 source buffer. The
  reviewed workflow compares image dimensions, source-buffer identity, and a
  pixel hash alongside structural reports and mesh fingerprints.
- Finding: deterministic source-buffer image evidence is useful for corpus
  review, but it remains distinct from native-window screenshots and GPU
  framebuffer capture. This strengthens the case for an eventual presentation
  diagnostics review without moving image capture into vector geometry.
- Disposition: remain Incubating; do not extract a first-party vector crate or
  admit a general image-capture service from this slice alone.
- Follow-up: keep native-window/backend capture separately labeled and revisit
  broader diagnostic ownership only after another corpus consumer needs the
  same artifact contract.

## Future AR Candidate: Deterministic Image Diagnostics

The corpus is beginning to produce a repeated need for saved visual evidence.
`hello-save-image` already demonstrates example-level image export, and the
UI, font, SVG, and vector corpora could use saved artifacts for review,
regression comparison, and AI-assisted inspection.

This should be treated as a separate architectural question rather than folded
into vector geometry:

```text
semantic example or renderer
        -> diagnostic image artifact
        -> review, diff, or corpus archive
```

Potential scope includes deterministic source-buffer export, renderer
framebuffer readback, image metadata, and pixel-diff policy. Source-buffer
export is not equivalent to GPU framebuffer capture, so the two contracts must
not be conflated. The likely future AR should decide whether image capture is
an example utility, a renderer diagnostic capability, or both.

Promotion pressure would include multiple corpus examples needing the same
capture path, repeatable image comparisons across backends, or a need to
inspect native/WASM output without manual screenshots. Until then,
`hello-save-image` remains the reference example and no core dependency is
implied.

## Secondary Finding: Presentation Diagnostics

The image-export discussion suggests a broader inspection concern may emerge
around presentation work:

```text
semantic scene
    -> render
    -> image artifact
    -> SVG or geometry dump
    -> statistics
    -> validation report
```

These outputs inspect presentation without necessarily being part of rendering
or vector geometry. This is only an observed boundary, not an admitted service;
it should be revisited if multiple corpus examples need the same diagnostics.

The current ownership findings can therefore be summarized as:

- presentation geometry owns paths, contours, topology, fill/stroke, and
  tessellation;
- importers and providers own SVG semantics, font technology, and icon
  libraries;
- text owns layout, shaping, metrics, and baselines;
- renderers own GPU execution, batching, uploads, and cache lifetime;
- diagnostics may eventually own inspection artifacts without owning any of
  those semantic truths.

### Cycle 8 -- 2026-07-23

- Status entering review: Incubating
- New evidence: the workspace-level `tests` package consumes the public
  `presentation-geometry-corpus` API and runs synthetic topology cases. It also
  verifies that the public registry distinguishes synthetic and UI producers.
- Finding: the incubating diagnostic boundary is usable by a non-example
  consumer without requiring provider assets or renderer access. This validates
  the current public test surface, but it is not production consumption of a
  vector capability.
- Disposition: remain Incubating; do not extract or promote a first-party
  vector crate from this evidence alone.
- Follow-up: seek a production or independent tool consumer, and decide whether
  W3C/differential coverage is worth its dependency and provenance cost before
  revisiting promotion.

### Cycle 9 -- 2026-07-23

- Status entering review: Incubating
- New evidence: the seeded generator produced 100/100 valid cases for seed 42,
  and the workspace consumer now repeats that batch as a deterministic test.
- Finding: the current generated polygon space does not expose an additional
  reduced failure that needs promotion into the permanent synthetic matrix.
  The generator remains useful as a replayable exploratory tool rather than a
  second unreviewed golden corpus.
- Disposition: remain Incubating; no new synthetic case is added and no vector
  crate is promoted from this result.
- Follow-up: vary seeds or generation strategies when a concrete topology gap
  appears; do not broaden the permanent matrix without a distinct contract.

### Cycle 10 -- 2026-07-23

- Status entering review: Incubating
- New evidence: four deterministic seeds (`42`, `1`, `7`, and `2026`) each
  produced 250/250 valid generated polygon cases. The workspace consumer now
  repeats all 1,000 cases.
- Finding: broader seeded exploration still exposes no distinct reduced
  failure. The current generator is useful for replay and smoke coverage, but
  it does not replace targeted topology fixtures or standards-derived cases.
- Disposition: keep the generated corpus non-golden and leave W3C/resvg
  evaluation deferred; do not add a first-party vector crate.
- Follow-up: obtain a pinned standards-derived fixture only when the project is
  ready to accept its provenance and dependency cost.

### Cycle 11 -- 2026-07-23

- Status entering review: Incubating
- New evidence: the official W3C SVG 1.1 2nd Edition archive is preserved with
  provenance and checksum metadata. Two explicitly admitted cases now run
  through the public corpus runner from source through provider-neutral vector
  paths and CPU mesh validation.
- Structural evidence: each admitted case emits source, vector, mesh,
  fingerprint, contour, mesh-view, and stage-graph artifacts. The artifacts
  record algorithm identity and remain independent of a live renderer or
  browser harness.
- Findings:
  - a pinned standards-derived fixture strengthens the shared geometry
    evidence without changing the vector ownership boundary;
  - structural artifacts are sufficient to localize the current admitted
    cases, so differential rendering is not yet justified;
  - `paths-data-02-t` exposes a real fill-topology gap, while stroke-oriented
    cases remain out of the fill pass until SVG paint and stroke expansion are
    represented explicitly;
  - W3C fixture acquisition and structural execution do not constitute SVG
    conformance.
- Disposition: remain Incubating; do not extract a first-party vector crate or
  add a differential SVG dependency from this evidence alone.
- Follow-up: keep W3C admission incremental, address the fill-topology finding
  with a focused diagnostic slice, and revisit differential rendering only if
  structural artifacts stop localizing a failure.

## References

- `docs/Plans/ui-box-vector-presentation.md`
- `docs/Plans/font-outline-vector-presentation.md`
- `docs/Conversations/Can fonts be vectors.md`
- `.workbench/Notes/UI Boxes moving to SVG Renderer.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0004-foundational-presentation-text-and-icons.md`
