# Presentation Geometry Corpus Harness

## Status

Incubating. Slices 1 through 8 have partial executable evidence in the
example-side runner. Stage selection and glyph lineage artifacts are now
implemented. Synthetic topology probes now classify self-intersection at the
vector boundary instead of assuming the mesh tessellator will reject it. The
immediate trigger is unresolved font-outline geometry loss in the
`hello-ui-text-vectors` corpus, including malformed `K`, `k`, `M`, `e`, and
other hard-edge or mixed-curve glyphs.

## Purpose

Build a focused corpus harness that captures and validates every stage between
a semantic presentation producer and renderer-neutral geometry.

The harness should turn a visual report such as:

> The `k` is missing an arm.

into stage-specific evidence such as:

> Glyph contour 0 is intact after outline extraction, but triangle coverage is
> lost during general-fill tessellation.

This work supports the investigation in AR-0001. It does not itself admit a
new vector capability or settle the representation of native curves.

The same observe-record-compare pattern may later apply to layout, scenes,
physics, or other staged transformations. This plan remains scoped to
presentation geometry until an independent domain proves that a more general
diagnostics framework is warranted.

## Motivation And Current Evidence

Tokimu now has several independent producers of presentation geometry:

```text
UI surfaces
SVG documents
Lucide icons
font outlines
synthetic geometry
        |
        v
provider-neutral vector paths
        |
        v
tessellated geometry
        |
        v
renderer execution
```

The existing examples prove broad behavior, but diagnosing a failure still
requires inspecting the final screen and adding temporary logging to one
example. That is no longer sufficient for the topology problems exposed by
font outlines.

Recent failures have included:

- missing branches or diagonals in `K`, `k`, `M`, `N`, and `Z`;
- malformed mixed line/curve glyphs such as `%`, `&`, `2`, `4`, and `e`;
- valid counter glyphs whose final fill differs from their source contours;
- fixes that improve one glyph while regressing another;
- unit assertions that pass while the rendered glyph remains visibly wrong.

These failures may originate in outline extraction, curve flattening, contour
cleanup, fill-rule handling, tessellation, coordinate transforms, or renderer
submission. A stage-aware corpus is needed to distinguish those causes.

## Governing Principles

### Test The Shared Boundary

Producer-specific import remains separate, but all producers should be
inspectable through the same vector, mesh, and artifact contracts.

### Preserve Stage Evidence

The harness must retain enough information to identify the first stage that
diverged. A final image alone is useful review evidence but is not a diagnosis.

The first stage whose artifact diverges from its reviewed contract is the
owning diagnostic boundary. A downstream rendering defect must not be assigned
to the renderer when the vector or mesh artifact had already diverged.

### Observe Before Concluding

The corpus follows one debugging discipline:

```text
observe
    -> record
    -> compare
    -> conclude
```

It exists to make architectural boundaries observable. It should replace
cross-layer guessing with evidence while leaving ownership decisions to the
responsible implementation and Architectural Review.

### Keep Ordinary Runs Read-Only

Normal corpus execution must never rewrite reviewed expectations. Updating
golden artifacts must be a separate, explicit, noisy action.

### Normalize Before Comparing

Floating-point geometry, independent contour ordering, and triangle ordering
must not create meaningless diffs. Snapshot policy must state what is
canonicalized and what remains semantically significant.

### Incubate Before Promotion

The harness begins under `examples/lib-example/`. It must not create a
first-party `tokimu-corpus` or `tokimu-vector` crate until repeated independent
use establishes stable ownership and public contracts.

## Architectural Position

This is corpus and diagnostic infrastructure. It does not own presentation
semantics, provider technology, vector meaning, or rendering.

```text
Producer adapter
    owns SVG, font, icon, UI, or synthetic input semantics

Vector implementation
    owns paths, contours, fill/stroke policy, and tessellation

Corpus harness
    owns case execution, structural validation, normalized artifacts,
    comparisons, and reports

Screenshot helper
    owns deterministic CPU RGBA8 artifact encoding

Renderer
    owns GPU execution and any future framebuffer capture
```

The harness may observe each layer. It must not become the implementation of
those layers or expose their private parser/backend objects as corpus
contracts.

## Initial Location

Incubate shared runner and artifact code at:

```text
examples/lib-example/presentation-geometry-corpus/
```

Keep compact source cases and reviewed expected artifacts under the workspace
fixture structure established by `docs/testing-strategy.md`:

```text
tests/fixtures/
    presentation-geometry/
        synthetic/
        glyph/
        svg/
        lucide/
        ui/
    golden/
        presentation-geometry/

examples/lib-example/presentation-geometry-corpus/
    DESIGN.md
    src/
```

Write generated diagnostic runs under:

```text
target/presentation-geometry-corpus/
    <case-id>/
```

Large external corpora and prepared provider assets should remain pinned under
`third-party/` or generated under `target/` according to the existing testing
strategy. They should not be copied into the repository without provenance and
license review.

## Pipeline Model

Every case selects the stages it can meaningfully produce:

```text
source input
    |
    v
provider outline or document geometry
    |
    v
VectorPath
    |
    v
tessellated mesh
    |
    v
CPU source image or backend capture
```

The common produced record should remain deliberately plain:

```rust
pub struct ProducedCase {
    pub source: Option<SourceArtifact>,
    pub outline: Option<OutlineArtifact>,
    pub vector: Option<VectorArtifact>,
    pub mesh: Option<MeshArtifact>,
    pub image: Option<ImageArtifact>,
    pub diagnostics: Vec<CorpusDiagnostic>,
}
```

An unavailable stage is not automatically a failure. The case manifest defines
which stages and assertions are required.

Each executed stage also contributes one node to a diagnostic pipeline graph.
The graph records lineage, status, algorithm identity, input/output artifact
IDs, and optional timing without becoming a scheduler or scene graph.

## Case Model

The first implementation should use Rust-authored case declarations. A TOML
manifest may follow once the fields survive several producer types. Avoid
stabilizing a data format before the runner semantics are known.

```rust
pub struct CorpusCase {
    pub id: CorpusCaseId,
    pub input: CorpusInput,
    pub stages: StageSelection,
    pub assertions: Vec<CorpusAssertion>,
    pub comparisons: Vec<CorpusComparison>,
}

pub enum CorpusInput {
    Synthetic(SyntheticCase),
    Glyph(GlyphCase),
    Svg(SvgCase),
    Lucide(LucideCase),
    Ui(UiCase),
}
```

Provider-specific adapters may carry the source information required to lower
their input, but all cases converge on provider-neutral vector and mesh
artifacts.

If repeated serializers establish a stable common shape, the implementation may
introduce an internal generic envelope such as `Artifact<T>`. That type should
be earned by outline, vector, mesh, and image consumers rather than assumed in
the first slice.

## Artifact Envelope

Every machine-readable artifact carries common metadata automatically:

```json
{
  "schema": 1,
  "artifact_id": "glyph/inter/U+004B/vector",
  "artifact_kind": "vector",
  "producer": "glyph",
  "generator": "presentation-geometry-corpus",
  "input_hash": "...",
  "source_revision": "...",
  "algorithms": {
    "outline": "ttf-parser-0.25",
    "flatten": "ui-tools-adaptive-v1",
    "fill": "ui-tools-general-fill-v1"
  },
  "policy": {
    "tolerance": 0.001,
    "fill_rule": "even-odd"
  },
  "artifact": {}
}
```

Required metadata includes:

- schema and artifact kind;
- stable case and artifact identity;
- producer kind and generator version;
- input hash and fixture/source revision;
- relevant algorithm identities and versions;
- tolerance, fill rule, coordinate convention, and normalization policy;
- source or capture type for images.

Wall-clock generation time and elapsed timings belong in the run report and
`graph.json`, not deterministic golden content. They may be recorded for
diagnostics, but normal golden comparison ignores them.

## Diagnostic Artifacts

Each inspected case should be able to emit:

```text
target/presentation-geometry-corpus/<case-id>/
    source.txt or source.svg
    outline.json
    vector.json
    contours.svg
    mesh.json
    mesh.svg
    render.bmp
    graph.json
    report.json
```

Only meaningful artifacts are written. A synthetic case may have no outline;
a headless geometry case may have no image.

### `outline.json`

Records provider-neutral source contours before vector lowering:

- contour index and closure;
- native line, quadratic, and cubic segments;
- point/control-point coordinates;
- bounds and signed area;
- provider identity and source fixture revision where relevant.

### `vector.json`

Records the geometry accepted by the vector boundary:

- contours and explicit closure;
- native or flattened segments according to the current contract;
- fill rule and stroke policy;
- bounds, signed area, and winding classification;
- normalization and flattening tolerance.

### `contours.svg`

Provides a human-readable diagnostic view with:

- unfilled contour polylines;
- contour-specific colors;
- point markers and optional point indices;
- contour direction indicators;
- source and vector bounds;
- self-intersection markers;
- no renderer-specific correction.

This artifact is the first diagnostic target for the current `K` and `k`
failures.

### `mesh.json` And `mesh.svg`

Record and visualize:

- triangle vertices and indices or normalized triangle triples;
- mesh bounds and total signed/absolute area;
- degenerate triangle count;
- connected components where deterministically available;
- source/vector sample points that should be filled;
- uncovered or unexpectedly covered samples.

Leave the mesh diagnostic schema extensible for later topology measures such as
Euler characteristic, boundary-edge count, non-manifold-edge count, connected
components, and hole count. The first 2D milestone need only implement metrics
that localize current fill failures. Future 3D use remains a separate admission
question rather than an implied responsibility of this plan.

### `render.bmp`

Use the existing `examples/lib-example/screenshot` helper for deterministic CPU
RGBA8 artifacts. Label these as source-buffer artifacts. They do not establish
GPU framebuffer equivalence.

A future backend capture path may produce a separately identified artifact. It
must not silently replace source-buffer output.

### `graph.json`

Records pipeline lineage rather than scene structure:

- producer and selected stage order;
- input and output artifact IDs for every stage;
- stage status and diagnostics;
- algorithm identity and version;
- optional elapsed time;
- skipped or unavailable stages and their reasons.

Timing is observational evidence, not a correctness assertion. Future execution
systems may attach task or executor identity to a stage without changing the
meaning of the artifact pipeline.

## Snapshot Normalization

Snapshots must record a schema version and normalization policy.

Initial policy:

- reject non-finite coordinates before serialization;
- round coordinates only for textual comparison, never before validation;
- preserve segment order within a contour;
- preserve contour closure and fill rule;
- normalize repeated terminal points without changing logical closure;
- canonicalize independent contour ordering only when ownership and nesting are
  retained explicitly;
- do not canonicalize triangle order until tests prove order is irrelevant;
- record both original and normalized bounds.

Raw floating-point debug dumps are useful temporary evidence but are not stable
goldens.

## Structural Assertions

Implement deterministic structural assertions before pixel comparison:

- all source, vector, and mesh coordinates are finite;
- contours contain the required number of distinct points;
- logical closure is preserved;
- expected contour and native-segment counts are preserved;
- mesh triangle streams are complete;
- mesh indices, if used, are in range;
- triangles above the configured epsilon are non-degenerate;
- output is deterministic across repeated runs;
- vector and mesh bounds agree within a documented tolerance;
- source fill samples remain covered by the mesh;
- known holes remain uncovered;
- expected connected components and holes are preserved where measurable.

Assertions must state the stage they validate. A mesh coverage assertion must
not be reported as proof that outline extraction is correct.

## Initial Corpus

Begin with cases that isolate known topology rather than importing hundreds of
external documents immediately.

### Synthetic

- triangle;
- rectangle;
- concave arrow;
- donut;
- nested holes;
- self-intersecting bow-tie;
- repeated vertex;
- nearly coincident edge;
- long thin diagonal;
- mixed line and quadratic contour.

### Font Glyphs

Use the prepared Inter fixture first, then repeat representative cases with
JetBrains Mono and Noto where useful:

- `O`, `8`, `@` for counters and nested contours;
- `A`, `M`, `N`, `W`, `X`, `Z`, `K`, `k` for hard edges and branches;
- `%`, `&`, `2`, `4`, `e`, `g`, `r` for mixed curves and compact topology.

Each glyph case records the font fixture, character/code point, font size,
flattening tolerance, expected stage counts, and reason for admission.

### SVG And Lucide

Start with compact checked-in regressions already represented by the Lucide
corpus:

- open and closed paths;
- hard joins and round caps;
- arcs and cubic curves;
- counters and multiple contours;
- the specific icons that previously exposed stroke or fill defects.

### UI

Add one box/surface case only after the synthetic path proves the shared runner.
UI semantics should be lowered by `ui-tools`; the corpus receives the resulting
vector geometry.

## External Corpus Strategy

### W3C SVG Tests

Use a deliberately selected, pinned subset after the internal path/fill harness
is stable. Initially target paths, fills, transforms, and clipping. Record the
upstream revision, source URL, license/provenance, and unsupported SVG features.

Passing a selected W3C SVG case validates Tokimu's SVG adapter and vector path
for that case. It does not claim full SVG conformance.

### resvg/usvg Differential Reference

Use `resvg` or `usvg` as an optional external reference renderer for selected
SVG cases. Keep this outside the authoritative vector contract:

- Tokimu output and reference output are separately labeled;
- reference version and settings are recorded;
- pixel tolerances are explicit;
- disagreement is evidence to inspect, not automatic proof that either side is
  architecturally correct.

Do not add this dependency in the first implementation slice.

### Property-Generated Cases

Add constrained generated paths only after deterministic case replay and
artifact emission exist. Every failure must print or save a seed and reduce to
a persistent regression case before being considered fixed.

Useful invariants include finite output, deterministic output, valid triangle
streams, bounded geometry, and no unexpected loss of sampled fill coverage.

### Future Inspect Mode

A later interactive inspector may present synchronized source, outline, vector,
mesh, coverage, render, and statistics views. Selecting a contour or triangle
could highlight its upstream and downstream lineage through `graph.json`.

This is a possible consumer of stable corpus artifacts, not part of the first
harness milestone. The artifact format must remain useful without an inspector,
window, or GPU.

## Implementation Slices

### Slice 1: Freeze The Current Glyph Regressions

- [x] Add focused cases for `K`, `k`, `M`, and `e`.
- [x] Capture provider-neutral outline and current vector data.
- [x] Record expected contour and native-segment counts.
- [x] Add fill-coverage probes that fail on the currently missing branches.
- [x] Keep the existing character-map example as the visual entry point.

Validation:

- at least one assertion reproduces each visible defect before a fix;
- tests identify outline, vector, or mesh as the failing stage;
- no glyph-specific geometry correction is introduced.

Implementation evidence:

- `ui-tools` now has a focused Inter regression covering `K`, `k`, `M`, and
  `e` through `tessellate_positioned_glyph`, the same positioned adapter used
  by `hello-ui-text-vectors`;
- the regression checks finite provider-neutral outlines, explicit contour
  closure, non-empty mesh output, and parity-based source-versus-mesh coverage;
- the coverage probe is intentionally independent of final raster output, so a
  failure identifies the shared fill/tessellation boundary before a renderer
  screenshot is involved;
- outline/vector artifact serialization and exact segment-count records are now
  emitted by the corpus runner and remain generated evidence until goldens are
  admitted.

### Slice 2: Add The Incubating Runner

- [x] Create `examples/lib-example/presentation-geometry-corpus`.
- [x] Implement initial case IDs, glyph inputs, diagnostics, and reports.
- [x] Run selected cases sequentially and deterministically.
- [x] Allow selecting one case for rapid investigation.
- [x] Return all selected case failures in one corpus report rather than aborting at the
  first unsupported input.
- [x] Add explicit producer stage selection to the case/report contract.
- [x] Record stage lineage and status in `graph.json`.

Implementation evidence:

- the initial runner covers the focused Inter cases `K`, `k`, `M`, and `e`;
- `list`, `run <case-id>`, and default all-case execution are available;
- the current structural report records `source`, `outline`, `vector`, and
  `mesh` status in a stable order;
- a successful `glyph/inter/K` run reports the provider, contour count,
  flattened point count, and triangle count without opening a renderer window;
- runtime stage skipping remains open; the current report records each case's
  explicit producer stage selection and successful glyph artifacts include
  deterministic stage lineage in `graph.json`.

Validation:

- repeated runs produce the same report ordering and structural results;
- a failed case does not prevent unrelated cases from running;
- the support library depends on public/incubating capability interfaces, not
  parser or renderer internals.

### Slice 3: Emit Outline And Vector Artifacts

- [x] Define versioned normalized outline and vector records.
- [x] Add the common metadata envelope with input hash and algorithm identity.
- [x] Emit `outline.json`, `vector.json`, `mesh.json`, and `contours.svg`.
- [x] Mark points, contour indices, winding, and bounds.
- [x] Record explicit segment intersections for flattened glyph contours.
- [x] Include fixture identity, flatten tolerance, tessellator identity, and
  fill rule in the artifact metadata.

Implementation evidence:

- successful runs write generated evidence under
  `target/presentation-geometry-corpus/<case-id>/`;
- JSON records use schema version `1` and a shared metadata envelope;
- the input hash is deterministic FNV-1a over the prepared source bytes and
  case character, excluding timestamps and machine-specific paths;
- outline records preserve native line, quadratic, and cubic segment identity;
- vector records preserve contour order, closed state, flattened points,
  signed area, bounds, and non-adjacent segment intersections;
- mesh records preserve triangle vertices and count degenerate triangles;
- `contours.svg` is a human-viewable flattened contour diagnostic, not a
  replacement for the structured records.

Validation:

- `K`, `k`, `M`, and `e` can be inspected without running a GPU window;
- artifacts identify the first missing or changed segment;
- normalized output is stable across repeated runs.

### Slice 4: Emit And Validate Mesh Artifacts

- [x] Emit `mesh.json` and `mesh.svg`.
- [x] Validate finite coordinates and triangle completeness.
- [x] Record bounds, area, and degenerates.
- [x] Add initial coverage probes for synthetic topology cases.
- [x] Compare source/vector expected fill against mesh coverage for those
      probes.

Implementation evidence:

- mesh reports now distinguish tessellation success from structural mesh
  validity;
- finite vertices, complete triangle groups, degenerate triangle count, and
  total absolute triangle area are recorded in `mesh.json` and the console
  report;
- non-finite or incomplete mesh structure fails the mesh stage instead of being
  reported as a successful case;
- degenerate triangles remain visible diagnostic evidence without being treated
  as an automatic failure until corpus expectations establish an appropriate
  tolerance;
- `mesh.svg` provides a simple human-viewable triangle diagnostic while the
  structured JSON remains authoritative.
- synthetic rectangle, concavity, and hole cases now exercise explicit
  inside/outside probes against both the source contours and generated mesh;
  glyph coverage remains open until stable probe points are defined for font
  outlines.

Validation:

- a glyph with intact contours but missing rendered branches fails at the mesh
  stage;
- holes and counters have explicit uncovered probes;
- structural checks catch a regression even when coarse bounds still pass.

### Slice 5: Add Synthetic Geometry Cases

- [x] Add the initial synthetic topology matrix.
- [x] Keep each case small enough to reason about manually.
- [x] Record structural mesh invariants and observed area/counts.
- [x] Exercise a seeded generated batch and promote each reduced failure into
  this set; the reviewed seed produced no new failures.
- [x] Add explicit self-intersection expectations.
- [x] Add a valid near-degenerate expectation.

Implementation evidence:

- the runner now includes `synthetic/convex-rectangle`,
  `synthetic/concave-notch`, `synthetic/multi-contour-hole`, and
  `synthetic/near-degenerate`;
- synthetic cases share the same vector and mesh validation as font cases;
- the current cases produce finite, complete meshes with observed areas of
  `1.0`, `0.7075`, `0.75`, and approximately `0.00001` respectively;
- no font parser, SVG importer, or renderer is involved in these cases.

Validation:

- fill-rule, concavity, hole, and near-degenerate behavior are tested
  independently of fonts and SVG;
- a self-intersecting bow-tie is reported as an expected unsupported-topology
  vector result, and mesh is explicitly marked not attempted rather than
  treated as an unexplained runner failure;
- unsupported topology produces an explicit diagnostic.
- the reviewed seeds `42`, `1`, `7`, and `2026` each produce 250/250 valid
  generated cases, so there is no additional reduced failure to promote at
  this time;
- the workspace consumer repeats those four generated batches as deterministic
  regression checks.

### Slice 6: Integrate Saved Visual Evidence

- [x] Use the existing screenshot helper for deterministic CPU artifacts.
- [x] Emit artifact manifests with dimensions, format, source stage, and color
  assumptions.
- [x] Compare a deterministic CPU image fingerprint without claiming GPU
      framebuffer equivalence.
- [x] Document native-window screenshots as separately labeled manual evidence.
- [x] Add deterministic CPU image comparison after the source-buffer path
      stabilized.

Implementation evidence:

- glyph mesh artifacts now include `mesh-cpu.bmp` and
  `mesh-cpu.manifest`;
- the image is produced from the normalized CPU triangle buffer, not a GPU
  framebuffer or native window capture;
- the manifest records dimensions, format, source stage, buffer type, GPU
  readback status, and explicit colors;
- glyph cases now emit `image-fingerprint.json`, and reviewed glyph goldens
  compare its RGBA8 dimensions, source-buffer identity, and deterministic
  pixel hash alongside the mesh fingerprint;
- native-window capture remains an external/manual workflow and is still
  separate from CPU image evidence; its labeling convention is documented in
  `examples/lib-example/screenshot/manual/`.

Validation:

- an image artifact states whether it came from a CPU source buffer or backend
  capture;
- ordinary runs do not rewrite reviewed images;
- image evidence complements rather than replaces structural checks.

### Slice 7: Add Additional Producers

- [x] Add a selected Lucide/SVG case through the same runner.
- [x] Add one UI surface lowering case through `lower_surface_to_vector`.
- [x] Confirm the SVG adapter stops at `VectorPath` before shared mesh work.
- [x] Record that open SVG stroke paths remain vector-stage evidence until
      stroke expansion is tested separately.
- [x] Audit examples for duplicate corpus diagnostic serialization after the
      shared contract proved stable.

Evidence so far:

```text
svg/lucide/archive
  source: archive.svg, 340 bytes
  vector: 3 paths, 3 contours, 50 points, 1 closed contour
  mesh: 1 closed fill path, 2 open stroke paths, 18 triangles
```

This is convergence evidence for SVG and font producers at `VectorPath`, but
not yet evidence that SVG stroke semantics belong in the shared fill
tessellator.

Validation:

- font, SVG/Lucide, synthetic, and UI cases share vector and mesh validation;
- producer semantics do not leak into the vector artifact schema;
- examples remain focused visual corpus applications.

Cleanup evidence:

- the geometry corpus runner is the only workspace code emitting the shared
  `outline.json`, `vector.json`, `mesh.json`, `graph.json`, SVG diagnostic, and
  mesh/image fingerprint artifacts;
- visual examples may still write their own presentation-specific manifests or
  screenshots, but they do not duplicate the corpus artifact schema;
- no extraction or shared serializer expansion is justified by this audit.

### Slice 8: Add Golden Workflow

- [x] Place reviewed expectations under
  `tests/fixtures/golden/presentation-geometry/`.
- [x] Implement compare without mutation.
- [x] Add an explicit update/bless command only after comparison is stable.
- [x] Print the first changed line for a structural mismatch and require
      deliberate review before blessing.

Implementation evidence:

- `compare <case-id>` reads a canonical structural report and never writes it;
- `bless <case-id>` is the only command that writes a reviewed report fixture;
- fixture directory keys include a stable case-ID hash so case-sensitive IDs
  such as `glyph/inter/K` and `glyph/inter/k` remain distinct on Windows;
- current reviewed snapshots cover all eleven registered cases;
- snapshots preserve stage selection, stage status, summaries, and diagnostics,
  while leaving raw mesh and image artifacts available for later specialized
  comparisons.
- glyph cases also emit an order-independent `mesh-fingerprint.json` containing
  normalized bounds, validation metrics, and a quantized canonical triangle
  hash; this protects mesh evidence without making triangle emission order a
  premature contract.
- mismatch output includes the fixture path and first differing line with
  expected and actual content.
- `compare-all` validates every registered case in one read-only command;
  `bless-all` provides the corresponding explicit update operation.

Validation:

- normal tests cannot bless regressions;
- schema or normalization changes fail with an actionable diagnostic;
- equivalent glyph meshes remain comparable when triangle or vertex order
  changes, while meaningful geometry changes alter the fingerprint;
- goldens follow `docs/testing-strategy.md` placement and provenance rules.

### Slice 9: Evaluate External And Generated Corpora

- [x] Select and pin a small W3C path/fill subset.
- [ ] Evaluate optional resvg differential rendering.
- [x] Add constrained seeded generation and replay.
- [x] Record generation count, seed, stage summaries, and failures in the CLI
      output before expanding corpus size.

Validation:

- external provenance and versions are pinned;
- unsupported SVG semantics remain explicit;
- generated failures are reproducible from a recorded seed;
- corpus runtime remains practical for focused local investigation.

Current evidence:

- `generate <seed> <count>` produces deterministic convex-ish polygon inputs;
- generated inputs are intentionally non-golden and are not added to the
  reviewed case registry;
- generation is capped at 1000 cases per invocation to keep local diagnostics
  bounded;
- The pinned W3C fixture now supplies two admitted source/vector/mesh cases;
  the remaining manifest entries stay selection candidates until their SVG
  semantics and topology are independently supported.
- Optional differential rendering remains open because it adds dependency
  cost and is not needed to localize the current structural failures.
- the prepared Lucide external subset now records provider revision, source
  selection rule, and selected count in `provenance.json`; it remains a data
  corpus, not a W3C conformance claim.

Deferral decision:

- do not add `resvg` or `usvg` to the workspace yet; neither is currently an
  established dependency and the differential boundary is not needed to
  localize the present glyph/SVG failures;
- do not label the Lucide subset as W3C evidence; a W3C subset requires checked
  fixture provenance, an explicit feature scope, and a pinned upstream source;
- reopen this work when at least one current failure cannot be localized by
  outline/vector/mesh artifacts, or when a non-example consumer needs a
  backend-independent SVG acceptance signal.

Current W3C evidence:

- `svg/w3c/painting-fill-03-t` reaches source, vector, and mesh stages;
- `svg/w3c/paths-data-16-t` reaches source, vector, and mesh stages;
- SVG records now preserve importer-owned `fill` and `stroke` intent while the
  shared `VectorPath` contract remains paint-neutral;
- the W3C runner sends only closed records marked for fill into the shared
  fill tessellator, leaving stroke-only geometry as explicit non-fill evidence;
- both cases emit source, vector, mesh, fingerprint, contour, mesh-view, and
  stage-graph artifacts under the corpus target directory;
- `paths-data-02-t` remains a reduced fill-topology failure;
- stroke-oriented cases remain vector-stage evidence until SVG paint and
  stroke-expansion semantics are represented explicitly.

### Slice 10: Architectural Review

- [x] Record whether the harness exposes one stable presentation-geometry
  boundary across independent producers.
- [x] Record whether native curves are still importer-owned or need a shared
  representation.
- [x] Evaluate whether diagnostics remain example support or justify a broader
  presentation-diagnostics review.
- [x] Add a non-example consumer and evaluate whether it justifies vector
  promotion.
- [x] Keep packaging changes separate from evidence collection.

Validation:

- update AR-0001 with observed findings and unresolved questions;
- update ADRs only if an accepted ownership decision changes;
- do not promote a crate solely because the corpus runner exists.

Current review result:

- AR-0001 Cycle 11 records the current harness, golden, replay, image,
  workspace-consumer, and W3C structural evidence;
- `tests` consumes the runner through its public API and exercises synthetic
  geometry cases without depending on prepared external assets;
- vector remains incubating and no new first-party crate is admitted;
- W3C/resvg differential coverage, a production consumer, and the decision
  whether the runner itself should graduate remain promotion-relevant gaps;
  W3C structural evidence is no longer wholly deferred.

## First Working Milestone

The first milestone is intentionally narrower than the full conversation
proposal:

```text
Rust-authored glyph case
    -> provider-neutral outline
    -> VectorPath
    -> mesh
    -> outline/vector/mesh reports
    -> contours.svg and mesh.svg
```

It is complete when `K`, `k`, `M`, and `e` produce inspectable artifacts and
coverage assertions that distinguish contour loss from tessellation loss.

This milestone does not require TOML manifests, PNG encoding, W3C imports,
resvg, an HTML report, random generation, or a new first-party engine crate.

## Failure Semantics

The runner must diagnose and continue when practical:

- source fixture is missing or has changed identity;
- provider cannot produce a monochrome outline;
- source contains malformed or non-finite geometry;
- vector lowering loses closure or rejects topology;
- tessellation fails or produces invalid geometry;
- expected fill coverage is lost;
- an artifact schema is unsupported;
- an external reference tool is unavailable;
- an optional render stage cannot execute headlessly.

Fallback must never change provider, fill rule, tolerance, or algorithm without
recording that choice in the case report.

## Risks And Mitigations

### Harness Becomes A Second Vector Implementation

Risk: diagnostic code reconstructs geometry and accidentally becomes another
source of truth.

Mitigation: the harness observes public/incubating results. Independent math is
limited to assertions and visualization, and is labeled as diagnostic.

### Snapshot Noise Hides Regressions

Risk: harmless ordering or floating-point changes produce large diffs.

Mitigation: version normalization policy, preserve semantic ordering, and favor
structural invariants over raw dumps.

### Golden Updates Bless Bugs

Risk: broad snapshot replacement accepts broken output.

Mitigation: separate compare and update commands; make updates explicit and
review individual artifacts.

### External SVG Scope Expands Too Early

Risk: W3C/resvg cases turn a vector investigation into full SVG conformance.

Mitigation: select only path/fill cases tied to current contracts and diagnose
unsupported document semantics.

### Pixel Differences Become The Only Authority

Risk: backend/color-space noise obscures the responsible geometry stage.

Mitigation: require outline, vector, and mesh evidence before image comparison.

### Corpus Runtime Becomes Slow

Risk: thousands of icons, fonts, images, and generated cases make local work
impractical.

Mitigation: support focused case selection, keep a compact regression tier, and
run broad external corpora separately.

## Acceptance Criteria

This plan is complete when:

- synthetic, glyph, SVG/Lucide, and UI producers can converge on one stage-aware
  corpus record;
- known glyph regressions have deterministic stage-specific assertions;
- outline, vector, mesh, and optional image artifacts are versioned and
  inspectable;
- ordinary runs never mutate reviewed expectations;
- structural validation catches geometry loss that bounds-only tests miss;
- generated artifacts identify their fixture, tolerance, fill rule, algorithm,
  and source/capture type;
- `graph.json` identifies stage lineage, status, and algorithm versions without
  making timings part of deterministic correctness;
- external cases are pinned and make no unsupported conformance claim;
- examples no longer duplicate diagnostic serialization claimed by the shared
  harness;
- AR-0001 records what the corpus establishes about presentation geometry,
  curves, diagnostics, and promotion.

## Graduation Criteria

The corpus harness should be considered for promotion beyond
`examples/lib-example` only when:

- multiple independent corpus examples depend on stable case and artifact
  contracts;
- at least one workspace or non-example test consumer needs the same runner;
- snapshot normalization and update policy have survived real regressions;
- the harness contains no producer or renderer implementation semantics;
- extraction simplifies ownership compared with continued incubation.

Until then, it remains evidence-producing example support.

## References

- `docs/Conversations/SCG Corpus Tests.md`
- `docs/Plans/font-outline-vector-presentation.md`
- `docs/Plans/ui-box-vector-presentation.md`
- `docs/Architectural Reviews/AR-0001-shared-vector-presentation-geometry.md`
- `docs/testing-strategy.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0004-foundational-presentation-text-and-icons.md`
- `examples/lib-example/ui-tools`
- `examples/lib-example/screenshot`
- `examples/ui/hello-ui-text-vectors`
- `examples/ui/hello-ui-font2`
- `examples/ui/hello-ui-lucide2`
