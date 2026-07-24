# UI Boxes Through Vector Presentation

## Status

Implementation in progress; vector, UI-box, SVG migration, and review slices
are complete for the current bounded scope. Promotion remains deferred.

## Purpose

Replace example-local quad assembly for UI surfaces with a shared,
provider-neutral vector geometry path that can also serve SVG, icons,
diagrams, and future presentation clients.

This plan does not make UI depend on SVG. SVG is an importer. UI surfaces and
SVG documents should independently lower into the same vector geometry model.

```text
UI surface semantics                  SVG document
        |                                  |
        v                                  v
UI-to-vector lowering                 SVG importer
        |                                  |
        +------------> VectorPath <--------+
                            |
                            v
                   fill/stroke tessellation
                            |
                            v
                  renderer-neutral geometry
                            |
                            v
                     render submission
```

The governing boundary is:

> UI owns meaning and interaction. Vector presentation owns paths and
> tessellation. Render backends own GPU execution.

## Motivation And Evidence

The UI corpus currently repeats surface drawing behavior in example code. In
particular, `hello-ui-box` expands one `UiSurfaceCommand` into separate shadow,
border, and fill quads and maps semantic roles to renderer materials locally.
This works for square boxes, but it makes rounded corners, consistent borders,
shared stroke behavior, and future vector-backed controls application
responsibilities.

The Lucide corpus has already pressured the SVG stroke implementation into a
connected path tessellator with joins and caps. UI surfaces are another real
consumer of the same underlying geometric concepts. Reusing a provider-neutral
path pipeline would let improvements apply to both consumers without making UI
aware of SVG syntax or icon providers.

Current constraints include:

- `UiSurfaceCommand` contains a rectangle and resolved semantic style;
- `UiDrawer` owns semantic command generation and clipping decisions;
- the SVG parser currently flattens directly into point lists and loses some
  contour, closure, and source-style information;
- `ui-tools` currently has stroke tessellation but no general fill contract;
- `hello-ui-box` owns shadow, border, fill ordering and material resolution;
- `WgpuBackend::upload_mesh` creates a GPU buffer for each upload, so arbitrary
  per-surface uploads would create avoidable churn;
- rectangular clipping is now carried as metadata on surface commands and
  lowered vector layers; `UiRect::to_pixel_rect` provides the provider-neutral
  world-to-pixel conversion, while a later adapter can copy that result into
  the renderer's existing pixel-space scissor command.

## Architectural Position

This work incubates in `examples/lib-example/ui-tools` until the geometry
semantics have at least two independent consumers and survive corpus pressure.
It does not immediately create `tokimu-vector` or change the accepted engine
crate graph.

Before promoting vector geometry into a first-party capability, record an
Architectural Review covering ownership, dependency direction, provider
neutrality, and the graduation trigger. The implementation may gather evidence
inside `ui-tools` while that review remains Incubating.

The existing foundational presentation decision remains in force:

- applications communicate semantic intent;
- providers and importers communicate implementation data;
- renderer-native buffers and objects do not leak into semantic commands;
- presentation geometry must not own simulation truth or interaction state.

## Goals

- Preserve `UiSurfaceCommand` as the semantic UI boundary.
- Introduce a reusable, renderer-neutral vector path representation.
- Support square and rounded rectangular UI surfaces through one path builder.
- Tessellate fills and strokes without UI-specific logic in the tessellator.
- Lower surface shadow, border, and fill into deterministic ordered geometry.
- Batch generated triangles so the initial proof does not upload one mesh per
  surface every frame.
- Migrate `hello-ui-box` as the first UI corpus proof.
- Adapt SVG importing to the shared path representation after the UI proof is
  stable.
- Add focused geometry tests and preserve visual corpus evidence.

## Non-Goals

- Creating a complete SVG implementation.
- Supporting every SVG fill rule, mask, filter, gradient, transform, or text
  feature in the first slice.
- Creating a full retained-mode vector scene graph.
- Moving vector code into a new engine crate before admission evidence exists.
- Moving layout, themes, focus, hit testing, or interaction into the vector
  layer.
- Solving GPU atlas allocation, retained mesh caching, or general render graph
  batching in the first proof.
- Changing UI hit regions to follow decorative curves.
- Claiming arbitrary concave fill, holes, or winding support before tests prove
  those contracts.

## Proposed Internal Model

The initial provider-neutral model should be small and concrete:

```rust
pub struct VectorPath {
    pub contours: Vec<VectorContour>,
}

pub struct VectorContour {
    pub points: Vec<[f32; 2]>,
    pub closed: bool,
}

pub enum FillRule {
    NonZero,
    EvenOdd,
}

pub enum LineJoin {
    Miter,
    Bevel,
    Round,
}

pub enum LineCap {
    Butt,
    Square,
    Round,
}

pub struct StrokeStyle {
    pub width: f32,
    pub join: LineJoin,
    pub cap: LineCap,
    pub miter_limit: f32,
}
```

Exact names may change during implementation, but the model must preserve:

- contour boundaries;
- explicit open or closed state;
- provider-neutral coordinates;
- fill and stroke policy independent of UI themes and renderer materials;
- enough information for importers to avoid guessing path closure later.

`PathBuilder` should initially provide only proven constructors and operations:

- `move_to`;
- `line_to`;
- `close`;
- `rect`;
- `rounded_rect`.

Curve commands may remain importer-side flattening initially, provided the
result retains contour and closure metadata. A later review may admit native
curve segments if flattening evidence proves inadequate.

## Surface Lowering Contract

`UiSurfaceCommand` remains the input. A UI-specific lowering step resolves its
semantic style into ordered vector draw layers:

```text
1. shadow
2. border
3. fill
```

The vector tessellator must not know what a panel, button, card, hover state, or
theme role means. The lowering adapter owns:

- choosing the path constructor and corner radius;
- resolving border width and elevation into geometry requests;
- preserving semantic paint/material roles;
- deterministic layer ordering.

The vector layer owns:

- contour construction;
- fill tessellation;
- stroke expansion;
- joins, caps, and miter limits;
- validation of degenerate geometry.

The renderer adapter owns:

- mapping resolved paint roles to pipelines or materials;
- batching triangles by compatible render state;
- mesh upload and draw submission;
- backend-specific resource lifetime.

## Implementation Slices

### Slice 0: Record The Review Boundary

- [x] Create an Architectural Review for shared vector presentation geometry.
- [x] Record UI surfaces and SVG/Lucide as the initial independent consumers.
- [x] Set the disposition to Incubating unless review evidence supports more.
- [x] Link this plan, the UI-box note, relevant examples, and SVG tests.
- [x] State that no new engine crate is admitted by this implementation plan.

Validation:

- the ownership and dependency direction are reviewable before extraction;
- reopening and graduation triggers are explicit.

### Slice 1: Add Provider-Neutral Path Types

- [x] Add a focused vector module under `ui-tools`.
- [x] Add `VectorPath`, `VectorContour`, closure state, and style enums.
- [x] Add a small `PathBuilder` with rectangle and rounded-rectangle support.
- [x] Reject or safely discard non-finite points and degenerate contours.
- [x] Add unit tests for bounds, contour count, closure, winding consistency,
  square rectangles, and rounded rectangles.

Validation:

- `cargo test -p ui-tools` passes;
- constructing a UI box requires no SVG string or parser object;
- no renderer or GPU type appears in the vector model.

### Slice 2: Add The Smallest Honest Fill Tessellator

- [x] Implement convex closed-contour fill tessellation.
- [x] Return renderer-neutral triangle positions or a small geometry result.
- [x] Define winding and front-face expectations explicitly.
- [x] Diagnose unsupported concave contours, holes, or multiple-contour fills
  rather than silently producing corrupt geometry.
- [x] Add tests for rectangles, rounded rectangles after flattening, reversed
  winding, duplicates, zero-area contours, and non-finite input.

Validation:

- generated triangles remain within path bounds;
- total triangle area approximately matches source contour area;
- no NaNs, inverted triangles, or open fill contours are emitted;
- unsupported topology fails explicitly.

### Slice 3: Reuse And Normalize Stroke Tessellation

- [x] Adapt the connected stroke builder to consume `VectorContour`.
- [x] Preserve explicit open/closed behavior.
- [x] Keep joins and caps style-driven rather than importer-driven.
- [x] Retain regression tests derived from the Lucide failures: `@`, archive,
  anvil, asteroid, angry, arrows, and curved sparkle paths.
- [x] Remove point-list assumptions that infer closure from coincident points.

Validation:

- existing Lucide corpus output does not regress;
- 90-degree box corners join without gaps;
- open paths receive caps only at true endpoints;
- closed paths do not receive endpoint caps.

### Slice 4: Lower UI Surfaces Into Vector Draw Layers

- [x] Add a UI-specific surface-to-vector lowering function in `ui-tools`.
- [x] Resolve shadow, border, and fill in deterministic order.
- [x] Build borders as a stroke or explicit nested fill according to whichever
  produces stable uniform width under the current tessellator.
- [x] Keep material or paint roles attached to geometry batches without
  exposing renderer-native handles.
- [x] Add tests for layer order, border width, shadow offset, corner radius,
  elevation behavior, and theme-only paint changes.

Validation:

- changing a theme color does not change path geometry;
- changing corner radius does not change UI semantics or hit regions;
- border thickness remains uniform on all sides;
- shadow remains behind border and fill.

### Slice 5: Add A Bounded Submission Strategy

- [x] Group generated triangles by compatible paint/material role for a frame.
- [x] Upload at most one generated mesh per compatible batch in the initial
  proof, not one mesh per UI surface.
- [x] Use stable handles or a documented rotating handle set for dynamic batch
  uploads.
- [x] Record buffer churn and draw count in diagnostics or test-visible stats.
- [x] Defer retained mesh caching until repeated frame evidence demonstrates a
  real need. The example has an explicit local cache; shared retained caching
  remains deferred.

Validation:

- adding more boxes does not produce one upload per box;
- submission order preserves shadow, border, and fill layering;
- the implementation does not leak `wgpu` objects into `ui-tools`.

### Slice 6: Migrate `hello-ui-box`

- [x] Replace local shadow, border, and fill quad assembly with shared lowering.
- [x] Preserve the existing semantic `UiDrawer` and `UiSurfaceCommand` flow.
- [x] Add square and rounded box variants to the corpus screen.
- [x] Include flat, raised, bordered, nested, and clipped cases.
- [x] Remove local surface tessellation after parity is demonstrated.
- [x] Capture implementation observations in the example design document.

Validation:

- existing square-box appearance remains materially equivalent;
- rounded corners are visibly continuous;
- nested bounds and title/text alignment do not regress;
- one example-local call submits the shared surface batches;
- `hello-ui-box` contains no local border/shadow/fill tessellation code.

### Slice 7: Define Clipping Behavior

- [x] Separate semantic/layout clipping from geometric path construction.
- [x] Document that the first implementation carries rectangular scissor
  metadata; renderer application remains a follow-up adapter step.
- [x] Do not create rounded corners at newly clipped edges.
- [x] Add tests for unclipped, fully clipped, and partially clipped surfaces.
- [x] Add a provider-neutral world/UI-to-pixel clip conversion with viewport
  clamping and explicit empty/off-screen behavior.
- [x] Wire the `hello-ui-box` adapter to carry converted clips into renderer
  `ViewportRect` commands without merging batches across different clips.

Validation:

- clipping does not mutate the source semantic command;
- partially clipped rounded surfaces do not acquire invented corner radii;
- the current limitation is explicit: non-rectangular clipping is unsupported;
  renderer command wiring is demonstrated in `hello-ui-box` and remains an
  adapter responsibility for other consumers.

### Slice 8: Adapt SVG Importing To `VectorPath`

- [x] Add direct SVG path extraction that preserves contours and closure in
  `VectorPath`/`VectorContour` values.
- [x] Migrate primitive-element extraction for circles, rectangles, lines,
  polylines, and polygons.
- [x] Retire the legacy flattened path API after confirming there are no
  downstream workspace callers; retain its logic only inside SVG regression
  tests during the transition.
- [x] Keep SVG styling and parsing concerns in the importer.
- [x] Feed imported contours into the same stroke tessellator used by the
  Lucide corpus examples.
- [x] Route only topology supported by the fill tessellator through shared fill;
  diagnose the rest. `parse_svg_document_convex_fill_meshes` provides this
  bounded importer path, while `validate_convex_fill` exposes the decision
  without allocating geometry.
- [x] Preserve the 100-icon interactive Lucide corpus as a regression surface.

Validation:

- importer changes do not alter UI semantic APIs;
- UI path construction does not depend on SVG parsing;
- the Lucide corpus remains usable for identifying individual failures;
- closure is explicit rather than inferred from duplicate endpoints.

Current evidence: `parse_svg_document_vector_paths` now parses path-element
data directly and groups multiple subpaths into one `VectorPath`. Both
`hello-ui-lucide` and `hello-ui-lucide2` tessellate its contours through
`tessellate_stroke`. Primitive elements use the same contour model, and the
repeated path-collection stroke helper now lives in `ui-tools`. The old
flattened parser and stroke adapter are test-only compatibility fixtures; no
workspace consumer depends on them. SVG styling remains importer-owned, while
fill consumers can call `validate_convex_fill` before using the bounded shared
fill tessellator.

### Slice 9: Admission Review And Cleanup

- [x] Review duplicate geometry helpers and retire the duplicated Lucide
  path-collection stroke helpers.
- [x] Add renderer-visible upload and draw statistics without coupling the
  renderer to UI or vector semantics.
- [x] Record measured tessellation, upload, and draw-batch results from a
  representative corpus run.
- [x] Decide whether fills, native curves, retained caching, and geometry
  clipping require follow-up plans.
- [x] Revisit the Architectural Review with implementation findings.
- [ ] Promote to a first-party vector capability only if graduation criteria
  are met; otherwise keep the implementation incubating in `ui-tools`.

Validation:

- public and example APIs describe their actual guarantees;
- no deprecated point-list path remains without a migration note;
- docs, tests, examples, and review disposition agree with the code.

## Test Strategy

## Current Evidence Snapshot

As of the third review cycle:

- `cargo test -p ui-tools` passes 96 tests.
- `hello-ui-box` submits surface geometry in grouped shadow, border, and fill
  batches rather than one surface mesh per command.
- `hello-ui-lucide` and `hello-ui-lucide2` consume `VectorPath` contours and
  the shared stroke tessellator.
- SVG path elements and primitive elements have explicit open/closed contour
  tests.
- Rectangular clipping metadata survives semantic-to-vector lowering without
  changing rounded source geometry. `UiRect::to_pixel_rect` converts the
  orthographic UI/world rectangle to a clamped top-left-origin pixel rectangle,
  and `hello-ui-box` carries that result into renderer `ViewportRect` commands.
- `RenderStats` now exposes frame draw calls and cumulative mesh uploads. This
  makes renderer activity observable without exposing GPU objects or UI/vector
  concepts to the renderer contract.
- `hello-ui-box` now reports its first-frame draw-call count and cumulative
  mesh-upload count, providing a repeatable capture point for the next corpus
  run.
- A first native capture at revision `979826f` reported:

  ```text
  consumer: hello-ui-box
  draw_calls: 338
  cumulative_mesh_uploads: 6
  mesh_replacements: 0
  ```

  This is an observed baseline, not a performance threshold. Tessellation
  timing, Lucide corpus measurements, and buffer churn remain unmeasured.
- A first native capture of the 100-icon Lucide consumer at the same revision
  reported:

  ```text
  consumer: hello-ui-lucide2
  icons: 100
  draw_calls: 100
  cumulative_mesh_uploads: 100
  ```

  The one-mesh-per-icon relationship is now observable. Whether retained
  geometry or draw batching should be promoted remains an open architectural
  question rather than an assumed optimization requirement.
- A second-frame capture of `hello-ui-lucide2` reported the same `100` draw
  calls and the same cumulative `100` mesh uploads. This confirms that the
  current sample retains uploaded icon meshes across frames; the remaining
  cost is submission granularity, not per-frame upload churn.
- A second-frame capture of `hello-ui-box` reported `draw_calls: 338`,
  `cumulative_mesh_uploads: 6`, and `mesh_replacements: 0`, matching the
  first-frame values. An example-local value cache now avoids replacing
  unchanged batch meshes while preserving stable handles.
- This resolves the current example-level upload churn without promoting a
  retained mesh cache into `ui-tools` or the renderer contract. A shared cache
  remains deferred until a second consumer demonstrates the same ownership and
  invalidation requirements.
- The same run reported `geometry_ms: 26.100` for parsing and tessellating the
  100-icon sample during setup. This is a rough native example measurement,
  not a benchmark contract, but it gives future tessellation changes a
  repeatable comparison point.

The current measurement slice is complete for this review cycle. A future
benchmark can make the capture procedure repeatable across revisions, but the
examples already expose the evidence needed to decide whether the next work is
batching, topology, or provider integration.

### Measurement Capture

The first capture should use the existing example instrumentation:

```text
cargo run -p hello-ui-box
```

Record the first `hello-ui-box first-frame stats` line and the revision used
for the run. The minimum evidence record is:

```text
consumer: hello-ui-box
revision: <git revision>
draw_calls: <reported value>
cumulative_mesh_uploads: <reported value>
```

This is intentionally a capture procedure, not a performance threshold. The
first run establishes observed behavior; repeated runs and a second consumer
are required before turning the numbers into an architectural guarantee.

For the Lucide comparison, use:

```text
cargo run -p hello-ui-lucide2
```

Record the `corpus build`, `first-frame`, and `second-frame` lines. The second
frame is important: unchanged cumulative uploads demonstrate retained meshes,
while unchanged draw calls expose submission granularity without conflating it
with upload churn.

### Follow-up Disposition

Current evidence supports these bounded conclusions:

- UI-box lowering benefits from semantic batching and shared vector geometry.
- Lucide geometry is retained across frames after setup.
- The Lucide sample still submits one draw per icon; batching is a possible
  future optimization, not yet a required vector-capability contract.
- Clipping metadata is preserved, and renderer scissor execution already exists
  through `ViewportRect`. The semantic conversion is covered in `ui-tools`, and
  the UI-box consumer now demonstrates adapter-level integration.
- Tessellation timing is observable in the corpus consumer, but no threshold
  or allocation guarantee is established.

Keep retained caching, draw batching, clipping execution, native curve support,
and broader fill topology as explicit follow-up investigations. Do not promote
them into the shared vector boundary solely because the current examples make
them convenient.

### Unit Tests

- path construction and contour closure;
- path bounds and winding;
- convex fill triangulation and area preservation;
- open and closed stroke behavior;
- line joins, caps, and miter fallback;
- surface lowering order and resolved geometry;
- rejection of degenerate or non-finite input.

### Corpus Tests

- `hello-ui-box` proves semantic UI surfaces lower through vector geometry;
- `hello-ui-button`, `hello-ui-panel`, and `hello-ui-card` remain regression
  consumers after the box proof stabilizes;
- `hello-ui-lucide2` pressures stroke topology and SVG importing;
- focused expanded-icon mode remains the preferred way to inspect failures.

### Backend Tests

- batch count and upload count are bounded by compatible render state rather
  than surface count;
- triangle ordering is stable;
- resize and DPI changes preserve geometry scale policy;
- native rendering remains free of validation errors.

### Visual Validation

Capture fixed-size reference screenshots for:

- square and rounded boxes;
- thin and thick borders;
- flat, raised, and floating elevations;
- nested surfaces;
- partial clipping;
- representative Lucide hard corners and curves.

Golden images should be introduced only when output stability and comparison
tolerance are defined. Until then, screenshots are review evidence rather than
automated pixel-perfect guarantees.

## Acceptance Criteria

The plan is complete when:

- UI surface semantics remain renderer- and SVG-independent;
- square and rounded boxes use the shared vector path pipeline;
- fill and stroke tessellation have focused automated tests;
- `hello-ui-box` no longer assembles its own shadow, border, and fill quads;
- shadow, border, and fill ordering is deterministic;
- hit regions continue to derive from UI layout geometry;
- submission avoids one mesh upload per surface;
- SVG importing can target the same vector path representation without making
  UI depend on SVG;
- unsupported topology and clipping behavior fail explicitly;
- the Lucide corpus does not regress from the connected-stroke baseline;
- corpus applications no longer duplicate geometry-generation behavior owned
  by the shared vector implementation;
- the Architectural Review records whether vector geometry remains incubating
  or has earned promotion.

## Risks And Mitigations

### Premature Generalization

Risk: the vector model grows into a complete SVG DOM or retained scene graph.

Mitigation: admit operations only when `hello-ui-box`, SVG/Lucide, or another
concrete consumer needs them. Keep the initial path model flattened and small.

### Tessellation Scope Creep

Risk: arbitrary fills, holes, winding, clipping, anti-aliasing, and native
curves are attempted together.

Mitigation: ship convex fills first, reject unsupported topology explicitly,
and add one follow-up plan per independently justified capability.

### GPU Upload Churn

Risk: dynamic vector geometry creates one allocation per surface per frame.

Mitigation: batch by compatible render state in the proof and gather evidence
before designing retained caches or allocators.

### Semantic Leakage

Risk: the vector layer learns about buttons, panels, theme roles, or focus.

Mitigation: keep semantic-to-vector lowering in the UI adapter and test the
vector module using geometry-only inputs.

### Clipping Artifacts

Risk: intersecting rounded surface bounds creates false corners or malformed
contours.

Mitigation: prefer renderer scissoring for the initial rectangular clip proof,
or diagnose unsupported geometry clipping explicitly.

### SVG Regression

Risk: changing path representation regresses already repaired Lucide strokes.

Mitigation: preserve focused icon fixtures and run the interactive 100-icon
corpus throughout migration.

## Graduation Criteria

A first-party vector presentation capability or crate may be proposed when:

- UI surfaces and SVG/icons independently consume the same geometry contracts;
- neither consumer leaks its semantic vocabulary into the vector model;
- corpus applications no longer contain duplicate geometry-generation logic
  for behavior claimed by the shared vector implementation;
- fill and stroke guarantees are documented and tested;
- at least one non-example consumer needs the same contracts;
- native and planned WASM dependency direction remains clean;
- renderer-native resource types remain outside the semantic geometry API;
- the Architectural Review finds extraction simpler than continued
  incubation.

Until those criteria are met, shared vector presentation remains an
evidence-backed implementation inside `ui-tools`, not an admitted engine crate.

## References

- `.workbench/Notes/UI Boxes moving to SVG Renderer.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0004-foundational-presentation-text-and-icons.md`
- `docs/Architectural Reviews/README.md`
- `docs/testing-strategy.md`
- `examples/lib-example/ui-tools/src/draw.rs`
- `examples/lib-example/ui-tools/src/svg.rs`
- `examples/ui/hello-ui-box`
- `examples/ui/hello-ui-lucide2`
- `crates/tokimu-render/src/wgpu_backend.rs`
