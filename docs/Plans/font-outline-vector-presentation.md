# Font Outlines Through Vector Presentation

## Status

Implementation in progress. The experiment boundary, provider-neutral outline
contract, font-to-vector adapter, bounded general-fill path, and positioned
glyph tessellation helper are implemented. A rendered raster-versus-vector
corpus proof and cost/readability measurements remain open.

## Purpose

Evaluate whether monochrome TTF/OTF glyph outlines can lower into Tokimu's
incubating provider-neutral vector geometry path without moving font meaning,
text layout, shaping, or provider ownership into the vector subsystem.

The governing boundary is:

> Text owns semantic requests and positioned glyph contracts. Font providers
> own font technology and outline extraction. Vector presentation may own the
> geometry used to draw outline glyphs. Render backends own execution.

This plan treats vector glyph rendering as one optional presentation strategy,
not as the definition of Tokimu's font architecture.

```text
TextSpec
    |
    v
font resolution and optional shaping
    |
    v
positioned glyph run
    |
    v
font outline adapter
    |
    v
VectorPath
    |
    v
fill tessellation
    |
    v
renderer-neutral geometry
    |
    v
render backend
```

Alternative execution paths remain valid:

```text
positioned glyph run
    |-- outline tessellation
    |-- CPU-rasterized glyph atlas
    |-- signed-distance-field atlas
    |-- native/platform text
    `-- color or bitmap glyph presentation
```

## Motivation And Evidence

Tokimu already has independent corpus pressure from UI surfaces, SVG documents,
Lucide icons, TTF/OTF font examples, and provider-neutral text contracts. UI
surfaces and SVG/icons currently converge on the vector implementation
incubating in `examples/lib-example/ui-tools`.

Ordinary monochrome TTF/OTF glyphs commonly contain quadratic or cubic outline
contours. Those contours are another plausible consumer of provider-neutral
paths and fill tessellation. If glyph outlines can consume the same geometry
contracts without leaking typography into them, they provide useful evidence
for the eventual admission review of shared vector presentation.

The existing text corpus also gives this experiment a comparison path. The same
positioned text can be rendered through the current raster implementation and
through outline tessellation, allowing Tokimu to compare correctness, metrics,
quality, geometry size, caching behavior, and small-size readability.

The experiment must not begin from the assumption that vector rendering is
better for all text. Raster atlases may remain preferable for ordinary UI text,
while outline geometry may be useful for large text, zoomable technical views,
transformed glyphs, image export, and future document-oriented presentation.

## Architectural Position

ADR-0004 remains authoritative:

- text and icon semantics are foundational presentation capabilities;
- TTF/OTF parsing, shaping, rasterization, atlases, and render submission remain
  replaceable providers or backends;
- measurement and layout remain usable without a window, GPU, or live renderer;
- parser-native font objects and renderer-native resources do not leak through
  author-facing contracts.

This work initially incubates in `examples/lib-example/ui-tools` and the font
corpus examples. It does not immediately create `tokimu-vector`, change the
accepted crate graph, or make the base font provider depend on a renderer.

The preferred dependency shape is:

```text
text semantic contracts
        |
        v
font provider contracts
        |
        v
font-outline-to-vector adapter
        |
        v
vector geometry contracts
        |
        v
renderer adapter
```

The font provider may expose provider-neutral outline data. A dedicated adapter
then converts that data into `VectorPath`. This avoids requiring metrics-only,
raster-only, platform, or headless providers to depend on vector geometry.

## Goals

- Define a provider-neutral monochrome glyph-outline result.
- Preserve contour boundaries, closure, winding, and glyph-space coordinates.
- Convert glyph outlines into the existing vector path model through an
  explicit adapter.
- Render a positioned glyph run using shared fill tessellation.
- Preserve text measurement, advances, baselines, and layout from the text
  capability rather than recomputing them from path bounds.
- Compare raster and vector execution of the same semantic text scene.
- Diagnose unsupported outline topology and font presentation formats.
- Gather evidence about curve quality, point counts, tessellation cost, caching,
  draw submission, and readability at multiple sizes.
- Determine whether glyph pressure justifies native path curves or broader fill
  topology in the vector model.

## Non-Goals

- Moving Unicode, shaping, bidi, ligatures, kerning, fallback, line breaking,
  baselines, or caret behavior into vector geometry.
- Replacing every text renderer with vector triangles.
- Supporting color fonts, bitmap strikes, SVG glyph documents, gradients, or
  layered compositing in the first proof.
- Selecting a final font parser, shaping engine, rasterizer, or atlas strategy.
- Creating a general retained vector scene graph.
- Promoting a first-party vector crate before admission criteria are met.
- Treating path bounds as text metrics or vertically centering each glyph by
  its individual outline bounds.

## Proposed Internal Contracts

The font-facing result should describe outlines without exposing parser-native
objects:

```rust
pub struct GlyphOutline {
    pub contours: Vec<GlyphContour>,
    pub units_per_em: u16,
}

pub struct GlyphContour {
    pub start: [f32; 2],
    pub segments: Vec<GlyphOutlineSegment>,
    pub closed: bool,
}

pub enum GlyphOutlineSegment {
    LineTo([f32; 2]),
    QuadTo {
        control: [f32; 2],
        end: [f32; 2],
    },
    CubicTo {
        control1: [f32; 2],
        control2: [f32; 2],
        end: [f32; 2],
    },
}
```

Exact names may change during implementation. The contract must preserve:

- native contour boundaries and closure;
- line, quadratic, and cubic segment identity;
- font-unit coordinates and units-per-em scaling information;
- no parser-native face, table, or glyph object;
- no text role, Unicode scalar, layout cell, material, mesh, or GPU handle.

The first adapter may flatten curves with an explicit tolerance:

```text
GlyphOutline
    |
    v
font-outline-to-vector adapter
    |
    +-- scale from font units
    +-- apply glyph placement transform
    +-- flatten curves with declared tolerance
    +-- preserve contour closure and winding
    v
VectorPath
```

Flattening policy must be observable. It must not silently use a tolerance that
changes unpredictably with window size or backend behavior.

## Ownership Boundaries

### Text Capability Owns

- semantic text requests and roles;
- provider-neutral font identity and handles;
- fallback and diagnostics contracts;
- measurement, advances, baselines, ascent, descent, and line gap;
- line layout, wrapping, clipping, and alignment;
- positioned glyph runs and renderer-neutral draw intent;
- shaping input and output contracts when shaping is present.

### Font And Shaping Providers Own

- TTF/OTF/system-font decoding;
- face, variation-axis, and glyph resolution;
- metrics extraction;
- glyph substitution and positioning implementation;
- glyph outline extraction;
- format-specific unsupported-feature diagnostics.

### Font-To-Vector Adapter Owns

- conversion from provider-neutral glyph outlines to vector paths;
- font-unit scaling and positioned-glyph transforms;
- curve flattening policy for the initial proof;
- preservation of contour and fill-rule information;
- mapping outline failures into structured presentation diagnostics.

### Vector Presentation Owns

- provider-neutral paths and contours;
- curve flattening if later admitted as a shared vector responsibility;
- fill topology validation;
- winding and fill rules;
- fill tessellation;
- renderer-neutral triangle geometry.

### Renderer Adapter Owns

- geometry caching and resource lifetime;
- mapping paint to materials or pipelines;
- batching, upload, and draw submission;
- backend-specific antialiasing and sampling behavior.

## Implementation Slices

### Slice 0: Record The Experiment Boundary

- [x] Link this plan from the font/vector conversation and relevant review.
- [x] Record that vector glyph rendering is an optional presentation strategy.
- [x] Record that no first-party crate is admitted by this plan.
- [x] Identify the raster path as the comparison baseline.

Validation:

- the dependency direction is explicit before implementation;
- font meaning cannot flow downward into vector geometry;
- the experiment has a documented stopping point.

### Slice 1: Define Provider-Neutral Glyph Outlines

- [x] Add glyph outline, contour, and segment contracts under `ui-tools`.
- [x] Adapt the current TTF and OTF provider path to emit outlines.
- [x] Preserve lines, quadratic curves, cubic curves, and closed contours.
- [x] Return explicit diagnostics for missing outlines and unsupported formats.
- [x] Add focused tests using known glyphs with multiple contours and holes.

Validation:

- no parser-native object appears in the outline contract;
- metrics-only use remains independent from vector geometry;
- TTF and OTF fixtures satisfy the same outline contract.

### Slice 2: Add The Font-To-Vector Adapter

- [x] Convert glyph contours into `VectorPath` contours.
- [x] Apply units-per-em scaling and positioned-glyph transforms explicitly.
- [x] Flatten quadratic and cubic curves with a declared tolerance.
- [x] Preserve contour closure and winding.
- [x] Add tests for bounds, scaling, transforms, and deterministic output.

Validation:

- changing glyph placement moves geometry without changing text metrics;
- repeated conversion produces stable point and contour ordering;
- no layout cell or text role enters the vector model.

Implementation evidence:

- `UiGlyphOutline`, `UiGlyphContour`, and `UiGlyphOutlineSegment` preserve
  parser-neutral line, quadratic, and cubic outline data in font units;
- `UiFontRasterizer::outline` currently exposes the provider result without
  rasterizing it. The type name is historical and should be revisited when the
  provider boundary graduates;
- `UiGlyphOutline::to_vector_path` owns explicit units-per-em scaling, baseline
  origin, optional y-axis inversion, and output-space flattening tolerance;
- the checked-in Noto OTF fixture proves native curves, multiple closed
  contours, missing whitespace outlines, translation, scaling, and tolerance
  pressure;
- `ui-tools` passes 108 tests after this slice;
- unsupported color, bitmap-only, and SVG glyph presentation cannot yet be
  distinguished through the current `ab_glyph` provider and remains the open
  Slice 1 diagnostic item.

### Slice 3: Prove Honest Glyph Fill Topology

- [x] Identify which corpus glyphs exceed the current convex-fill contract.
- [x] Add bounded general-fill support for multiple contours and holes using
  the private `ui-tools` tessellation adapter.
- [x] Add a pre-tessellation topology classification for simple convex,
  concave, multiple-contour, and invalid glyph paths.
- [x] Define and test non-zero and even-odd fill behavior as required.
- [x] Test ordinary glyphs with counters such as `A`, `B`, `O`, `P`, `Q`, `a`,
  `e`, `g`, `0`, `8`, and `@`.
- [x] Test concave outlines and reversed contour winding.

Validation:

- counters remain empty rather than being filled accidentally;
- concave glyphs do not silently produce corrupt triangles;
- unsupported topology returns a structured diagnostic;
- generated triangles remain finite and within expected outline bounds.

Current evidence:

- `A` and `O` are classified as multiple-contour glyphs before tessellation;
- a simple hyphen remains classified as a single convex contour;
- the original convex-fill contract still rejects multi-contour and concave
  paths, while `tessellate_general_fill` now handles those cases through a
  private lyon adapter;
- the general-fill proof currently relies on the tessellator's non-zero
  default fill behavior. Alternate fill rules remain an explicit contract gap;
- `O` now proves a real font counter can lower through the outline adapter and
  general-fill tessellator without requiring font or text semantics in vector.

Implementation evidence:

- `lyon_path` and `lyon_tessellation` are private `ui-tools` dependencies;
- the public API returns renderer-neutral triangle positions and does not
  expose lyon path, vertex, or tessellator types;
- concave single-contour and multi-contour geometry have focused tests;
- font-specific counter rendering is covered by the checked-in Noto `O`
  outline test.
- the counter proof also checks that generated vertices remain within the
  flattened outline bounds, guarding against the class of runaway geometry
  that motivated this slice.

### Slice 4: Render A Positioned Glyph Run

- [x] Add a vector-outline text execution adapter.
- [x] Consume an existing positioned glyph run from text layout.
- [x] Place each glyph from baseline, bearing, advance, and provider metrics.
- [x] Keep path bounds out of line-layout decisions.
- [x] Add a bounded renderer-neutral tessellation helper for one positioned glyph.
- [x] Cache repeated glyph meshes sufficiently for a bounded corpus proof.

Validation:

- [x] the adapter consumes the existing layout pen position rather than
  deriving placement from outline bounds;
- [x] raster and vector paths use the same measured line and glyph positions;
- baseline alignment does not depend on individual glyph bounds;
- repeated glyphs can reuse outline or tessellated geometry;
- headless measurement still requires no renderer.

Implementation evidence:

- `UiFontRasterizer::tessellate_positioned_glyph` accepts an existing
  `UiRasterTextGlyph`, explicit font size, output scale, and baseline origin;
- it returns renderer-neutral triangle positions and keeps `Mesh`, material,
  pipeline, and GPU handles out of `ui-tools`' font contract;
- repeated `A` glyphs are covered by a test proving the second glyph follows
  the layout pen advance rather than the first glyph's outline bounds; the
  assertion now checks the expected output-space displacement numerically;
- tracking tests now distinguish provider glyph metrics from presentation
  placement: tracking changes the second pen position without changing either
  glyph advance, giving raster and vector consumers one shared placement
  contract;
- `hello-ui-font2` caches local glyph meshes by provider row and character,
  while instances retain their individual pen positions;
- `ui-tools` passes 124 tests after the current outline, topology, placement,
  provider, and scale-validation slices.

### Slice 5: Add A Side-By-Side Corpus Example

- [x] Extend an existing font corpus example or add a focused vector-font
  example.
- [x] Render identical semantic scenes through raster and vector strategies.
- [x] Include Inter, JetBrains Mono, and Noto fixtures where available.
- [x] Include normal prose, metric torture strings, punctuation, digits,
  ascenders, descenders, counters, and repeated glyphs.
- [x] Show multiple sizes, including ordinary UI text and large display text.

Validation:

- [ ] visual differences are attributable to execution strategy rather than
  layout;
- missing glyphs and unsupported presentation formats are reported visibly;
- the example remains a corpus proof rather than a production font viewer.

Implementation evidence:

- `hello-ui-font2` now retains its existing raster comparison and adds vector
  glyph meshes for the same three corpus lines and font rows;
- vector glyphs consume `UiFontRasterizer::layout` and
  `tessellate_positioned_glyph`, so the example does not independently place
  glyphs from outline bounds;
- the adapter receives the requested font size explicitly, preventing provider
  ascent differences from silently changing comparison scale;
- the provider matrix test applies the same outline contract to the prepared
  Inter, JetBrains Mono, and Noto fixtures when those corpus assets are
  available, while keeping the checked-in Noto fixture independent of corpus
  preparation;
- missing outline failures remain distinguishable from malformed outlines at
  the positioned-glyph boundary;
- `hello-ui-font2` now reports a vector diagnostic for an individual failed
  glyph and continues the corpus render instead of aborting the entire
  provider comparison;
- the example now uses one shared camera/world-to-pixel convention for raster
  and vector presentation, preventing the vector column from rendering at
  roughly twice the raster size or escaping its comparison region;
- the comparison no longer applies a raster-only horizontal squash; raster
  textures and vector glyphs now use the same pixel-to-camera scale on both
  axes, making remaining differences attributable to font execution rather
  than example-local aspect compensation;
- the comparison uses the established example convention of one pixel-scale
  unit over the viewport height for both raster quads and vector glyph meshes;
  the camera's world-height factor remains an implementation detail of the
  renderer rather than a second scale applied by the corpus example;
- raster heading placement no longer uses viewport width as an independent
  horizontal scale; horizontal placement now follows the camera aspect ratio
  through the same height-derived pixel convention as vector placement;
- the font comparison now reports viewport size, shared pixel scale, and the
  raster/vector column anchors at startup, making screenshot evidence tied to
  an explicit presentation configuration rather than an undocumented window
  state;
- each provider comparison now reports the resolved fixture path, format, and
  byte length, making the measurement record identify the actual font asset
  rather than only its logical provider name;
- the comparison manifest now records the expected fixture set and an
  optional `TOKIMU_CORPUS_REVISION` value, so a captured measurement can be
  tied to an explicit source revision when the runner supplies one;
- the multi-size corpus report now counts native line, quadratic, and cubic
  outline segments before flattening, giving Slice 7 direct evidence about
  whether font providers contribute meaningful native curve pressure;
- the vector side is intentionally a first proof only: it still needs visual
  review, provider coverage confirmation, and cost measurements before this
  slice is considered complete.

### Slice 6: Measure Cost And Quality

- [x] Record curve-flattened point counts at several output sizes.
- [x] Record tessellation time and generated triangle counts.
- [x] Record first-frame and retained-frame uploads and draw calls.
- [ ] Compare small-size legibility against the raster baseline.
- [ ] Compare scaling quality at large sizes and under transforms.
- [x] Record whether flattening tolerance must be transform-aware.

Validation:

- [ ] measurements are repeatable and tied to a revision and fixture set;
- observations are not promoted into guarantees prematurely;
- the evidence distinguishes setup cost, retained cost, and submission cost.

Implementation evidence:

- `hello-ui-font2` reports vector glyph mesh count, generated triangle count,
  and initial outline/tessellation build time;
- the vector proof reports both glyph instances and cached mesh count, keeping
  reuse visible rather than implying that every drawn occurrence is uploaded;
- the corpus now reports point and triangle counts for the same sample at 24,
  56, and 96 pixels, providing initial scale-growth evidence;
- the font comparison now measures representative metric-torture samples in
  addition to prose: narrow `I`, wide `W`, digits, punctuation, and `AV`
  pairs;
- a native startup run completed for Inter, JetBrains Mono, and Noto with 365
  vector glyph instances, 123 cached meshes, 4,767 generated triangles, and
  approximately 788 ms of initial outline/tessellation build time on the
  current workstation;
- the example reports advance-based layout width beside raster visible-ink
  width for every provider and corpus line, keeping expected metric differences
  inspectable during visual review;
- the latest runtime parity check kept advance and raster visible-ink widths
  within approximately one pixel for Inter, JetBrains Mono, and Noto across
  all three corpus lines;
- `hello-ui-font2` now reports the first two renderer frames, including draw
  calls, mesh uploads, and mesh replacements, so initial setup cost can be
  compared directly with retained-frame submission;
- point-count and multi-size quality measurements remain intentionally open.
- current headless scale tests establish finite geometry, monotonic tessellation
  detail, predictable output scaling, and transform-aware flattening tolerance;
  screenshot-based legibility and native-curve quality remain open.
- a headless Noto `O` proof now tessellates at 24, 56, and 96 pixels and
  verifies non-empty finite geometry at each size; visual small-size legibility
  and transform-specific quality remain open for corpus review. The same proof
  also checks that adaptive flattening does not reduce triangle detail as the
  requested output size increases; a separate headless proof confirms output
  scale changes geometry bounds predictably while leaving the layout pen
  unchanged. The tolerance proof also confirms that flattening tolerance is
  specified in output units and must be scaled with the output transform.

### Slice 7: Review Native Curve Pressure

- [ ] Evaluate whether flattened `VectorPath` contours remain adequate.
- [x] Identify duplicate curve-flattening logic between SVG and fonts.
- [ ] Compare geometry growth and visual quality across zoom levels.
- [ ] Propose native vector path segments only if at least SVG and fonts need
  the same contract.

Validation:

- native curves are admitted only from independent consumer evidence;
- a rejected native-curve proposal leaves an explicit rationale;
- importer-specific curve knowledge does not leak into tessellation silently.

Current evidence:

- font outlines preserve native quadratic and cubic segments before using an
  adaptive, tolerance-based flattening pass;
- SVG path parsing currently uses importer-local sampled curve expansion for
  its legacy path representation;
- both paths terminate in provider-neutral flattened contours, but their
  flattening policies are not yet identical enough to justify extracting a
  shared native-curve or shared flattening service;
- this is evidence of a possible future convergence point, not evidence that
  `VectorPath` should own font metrics, SVG command state, or importer policy.

### Slice 8: Admission Review And Cleanup

- [x] Record findings in the vector Architectural Review.
- [ ] Decide whether font outlines count as a stable independent vector
  consumer.
- [x] Remove duplicate example-local outline conversion and tessellation logic.
- [x] Keep vector glyph rendering incubating if contracts remain unstable.
- [ ] Update ADR-0004 only if the accepted ownership decision changes.

Validation:

- the review records evidence rather than assuming promotion;
- implementation and documentation agree on ownership;
- unsupported font presentations remain explicit future work.

Current disposition:

- AR-0001 Cycle 5 records font outlines as strong independent evidence for a
  shared presentation-geometry capability;
- the font-outline adapter and corpus consumer no longer duplicate conversion
  or tessellation logic locally;
- promotion remains open because the consumer is still an example and native
  curve quality, visual parity, and a non-example consumer remain unresolved;
- shared corpus evidence now has an incubating artifact helper with independent
  `hello-save-image` and `hello-ui-font2` consumers; it writes CPU source
  artifacts and manifests but does not imply GPU readback;
- no ADR-0004 change is required because text/provider ownership is unchanged.

## Corpus Matrix

### Semantic Scenes

- title, heading, body, caption, button label, status label;
- `The quick brown fox jumps over the lazy dog.`;
- `Hamburgefontsiv`;
- `0123456789`;
- `0O1Il|{}[]()<>`;
- `AVAVAVAV`, `ToToToTo`, and `WaWaWaWaWa`;
- repeated narrow and wide glyphs such as `IIIIIIII` and `WWWWWWWW`;
- ascenders and descenders such as `bdfhklt` and `gjpqy`;
- counter-bearing glyphs such as `ABOPQRadegopq0689@`.

### Providers

- Inter for proportional UI text;
- JetBrains Mono for fixed-advance and punctuation pressure;
- Noto for fallback, broader coverage, and contrasting metrics.

### Sizes And Transforms

- ordinary UI size;
- medium heading size;
- large display size;
- non-uniform scaling as an explicit stress case, not a supported typography
  guarantee;
- zoomed technical-view presentation.

## Failure Semantics

The proof must diagnose rather than silently substitute when:

- a font provider cannot produce an outline;
- a glyph is missing;
- a glyph uses bitmap-only, color-layer, or SVG presentation;
- contour topology is unsupported;
- curve data or coordinates are malformed or non-finite;
- tessellation fails;
- fallback changes provider identity.

A configured fallback may continue rendering, but the resulting provider and
diagnostic must remain inspectable.

## Acceptance Criteria

The plan is complete when:

- TTF and OTF fixtures expose provider-neutral monochrome outlines;
- one adapter lowers those outlines into the shared vector path model;
- glyphs with concavity, multiple contours, and counters render correctly or
  fail with explicit diagnostics;
- positioned glyph runs preserve text metrics and baseline layout;
- raster and vector strategies render the same semantic corpus side by side;
- point counts, triangle counts, setup cost, retained cost, and draw submission
  are observable;
- examples do not own duplicate conversion behavior claimed by shared code;
- an Architectural Review records whether glyph outlines strengthen vector
  promotion evidence;
- no Unicode, shaping, fallback, font-provider, UI, or renderer-specific meaning
  leaks into vector geometry.

## Risks And Mitigations

### Font Semantics Leak Into Vector Geometry

Risk: vector paths begin carrying Unicode, glyph IDs, baselines, or text roles.

Mitigation: keep all typography in the text and font layers. Vector geometry
receives only contours, transforms, fill policy, and paint-neutral geometry.

### Font Providers Become Vector-Dependent

Risk: every provider must depend on vector geometry even when it only supplies
metrics, raster glyphs, or platform text.

Mitigation: expose provider-neutral outline data and place conversion in a
separate adapter that depends on both contracts.

### Topology Scope Creep

Risk: the experiment attempts arbitrary vector fills, color fonts, compositing,
native curves, and GPU caching at once.

Mitigation: begin with monochrome outlines, add only topology required by the
fixed corpus, and diagnose everything else explicitly.

### Small Text Regresses

Risk: mathematically accurate outlines look worse than hinted raster glyphs at
ordinary UI sizes.

Mitigation: retain the raster path, compare side by side, and treat vector text
as optional unless quality evidence supports broader use.

### Geometry And Submission Cost

Risk: tessellating every glyph creates excessive vertices, uploads, and draws.

Mitigation: measure first, reuse repeated glyph geometry, and avoid promoting a
cache or batching contract until repeated consumers establish ownership needs.

### Flattening Becomes Scale-Dependent

Risk: one flattened outline is either wasteful at small sizes or visibly coarse
when enlarged.

Mitigation: record tolerance and transform assumptions, compare multiple sizes,
and use the evidence to decide whether native curves or tiered caches are
required.

## Visual Evidence Artifacts

Visual parity remains a corpus-review question, not a semantic contract. When
the example is run with image export available, retain deterministic artifacts
for inspection alongside the numeric reports:

```text
target/corpus-images/hello-ui-font2/
    raster.png or raster.bmp
    vector.png or vector.bmp
    comparison-notes.md
```

The existing `hello-save-image` example is the reference for source-buffer
export. It does not yet establish GPU framebuffer readback, so saved source
images and backend screenshots must remain labeled as different evidence. A
future presentation-diagnostics AR may define a shared capture and diff
contract if multiple corpus examples need it.

`examples/ui/hello-ui-font2/comparison-notes.md` is the review template for
recording one run without promoting image capture into a shared service.
The incubating `examples/lib-example/screenshot` helper now centralizes CPU
RGBA8 validation, deterministic BMP writing, and review manifests for future
corpus consumers; it deliberately does not capture GPU surfaces.
Its focused tests also verify the BMP signature, top-down row orientation, and
RGBA-to-BGRA conversion so saved artifacts have a stable source-buffer format.
`hello-save-image` now consumes the helper instead of carrying a private BMP
encoder, proving the artifact contract has an independent non-font consumer.
It also writes a metadata manifest beside the BMP, preserving dimensions and
the distinction between CPU source-buffer export and GPU readback.
`hello-ui-font2` now writes a companion comparison manifest with its viewport,
scale, anchors, and raster/vector strategies, giving the font corpus a second
consumer of the shared review-metadata path.

### Required Visual Review Procedure

For each visual review, record:

1. the exact `hello-ui-font2` command and prepared fixture paths;
2. the startup viewport, pixel scale, and column-anchor diagnostics;
3. one native-window screenshot or saved source image;
4. the checklist result below, including the provider and sample that exposed
   any failure.

Do not mark a visual item complete from headless geometry tests alone. The
headless tests establish finite geometry, placement, and metric invariants;
the saved image establishes legibility, baseline appearance, counters, and
curve quality.

### Visual Review Checklist

- [ ] raster and vector headings share the same baseline;
- [ ] repeated glyphs preserve advance spacing rather than outline-bound
  spacing;
- [ ] counters remain open in `B`, `O`, `P`, `Q`, `R`, `a`, `e`, `g`, `0`, `8`,
  and `@`;
- [ ] curve-heavy glyphs do not show spikes, gaps, or excessive faceting;
- [ ] small UI text remains legible beside the raster baseline;
- [ ] large display text remains smooth without changing semantic placement;
- [ ] missing outlines and unsupported presentations remain visible in the
  diagnostic output.

## Next Evidence Slice

1. Run `hello-ui-font2` with prepared Inter, JetBrains Mono, and Noto
   fixtures, retaining `comparison.txt` and a matching screenshot.
2. Complete the visual checklist for baseline, advance spacing, counters,
   curve quality, and small/large text legibility.
3. Compare the native-segment reports with the SVG/Lucide corpus before
   deciding whether flattened contours remain adequate.
4. Reopen AR-0001 only if those results change the ownership or promotion
   decision.

## Graduation Criteria

Font outlines strengthen the case for a first-party vector presentation
capability when:

- UI surfaces, SVG/icons, and font outlines independently consume the same path
  and tessellation contracts;
- none of those consumers leaks its semantic vocabulary into vector geometry;
- glyph topology guarantees are documented and tested;
- duplicate conversion and tessellation behavior has left corpus applications;
- native and planned WASM dependency directions remain clean;
- at least one non-example consumer needs the same vector contracts;
- extraction is simpler than continued incubation according to Architectural
  Review.

Until those criteria are met, glyph-outline vector rendering remains an
evidence-producing presentation adapter in `ui-tools`, not an admitted engine
crate or the mandatory Tokimu text path.

## References

- `docs/Conversations/Can fonts be vectors.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0004-foundational-presentation-text-and-icons.md`
- `docs/Plans/ui-box-vector-presentation.md`
- `docs/Notes/text-corpus-v1-validation.md`
- `docs/testing-strategy.md`
- `examples/lib-example/ui-tools`
- `examples/ui/hello-ui-font`
- `examples/ui/hello-ui-font2`
- `examples/ui/hello-ui-font2/DESIGN.md`
- `examples/ui/hello-ui-lucide2`
