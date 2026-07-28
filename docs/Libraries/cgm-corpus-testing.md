# CGM Presentation Geometry Corpus

## Status

Acquisition and v1 selection setup are complete. WebCGM 2.1 Test Suite Release
1.2 is pinned from the dated OASIS archive with provenance, exact archive and
selected-case hashes, a generated inventory, and a 15-case geometry-first
selection. No CGM parser, importer, or first-party capability has been admitted
yet.

This plan begins after the current SVG presentation-geometry work is stable
enough that a new producer can exercise the shared boundary without obscuring
existing SVG failures.

## Purpose

Build a deliberately selected Computer Graphics Metafile corpus that tests
whether CGM geometry can lower through Tokimu's provider-neutral presentation
geometry contracts:

```text
CGM fixture
    -> CGM decoding and profile interpretation
    -> CGM-owned semantic commands and attributes
    -> provider-neutral vector and paint records
    -> fill or stroke geometry
    -> mesh and diagnostic artifacts
```

The work is an engineering corpus, not a claim of complete CGM, WebCGM, CALS,
or ISO conformance. Its first purpose is to add an independent presentation
geometry producer and expose ownership, decoding, primitive-lowering,
attribute-state, topology, and diagnostic problems.

The key question is:

> Which CGM primitives naturally lower into Tokimu's presentation geometry
> without making CGM semantics part of the vector capability?

## Motivation And Existing Evidence

Tokimu already exercises presentation geometry through independent producers:

- UI surfaces;
- SVG documents and Lucide icons;
- font outlines;
- generated and synthetic geometry.

CGM adds useful pressure because it is a stateful graphics metafile rather than
an XML vector document or font outline. It can test whether the shared boundary
is genuinely producer-neutral while introducing different source semantics:

- ordered metafile elements;
- explicit picture and picture-body lifecycles;
- stateful line, edge, fill, color, and clipping attributes;
- several source encodings;
- geometric primitives that overlap with SVG but arrive through different
  format rules.

The conversation in `docs/Conversations/cgm corpus tests.md` identifies the
WebCGM test suite as the first acquisition candidate and the NIST CGM
conformance-testing report as methodology evidence. Both sources must be
verified and pinned during acquisition rather than treated as implicitly
available.

## Architectural Thesis

CGM is a source format and profile family. It must not become Tokimu's
canonical vector model.

```text
CGM importer
    owns encodings, element classes, metafile state, pictures, profiles,
    source attributes, source diagnostics, and provenance

Presentation geometry
    owns provider-neutral paths, contours, paint intent, clipping contracts,
    fill/stroke expansion, topology, and tessellation

Renderer
    owns GPU execution, uploads, batching, resource lifetime, and caches

Corpus harness
    owns case registration, stage artifacts, structural assertions,
    fingerprints, replay, and reports
```

CGM text, raster data, hyperlinks, DOM behavior, and profile-specific
interaction must not be disguised as vector geometry merely because they can
appear in the same metafile.

## Goals

- Acquire a reproducible, license-reviewed CGM fixture source.
- Preserve upstream fixtures verbatim and record exact provenance.
- Inventory profiles, encodings, case categories, and reference artifacts.
- Select a small geometry-first v1 corpus with a reason for every case.
- Decode one verified source encoding without silent recovery.
- Model enough CGM state to lower selected primitives correctly.
- Reuse the existing presentation-geometry artifact and validation pipeline.
- Distinguish format decoding, semantic interpretation, vector lowering,
  tessellation, and rendering failures.
- Report corpus coverage honestly against pinned denominators.
- Keep unsupported CGM and WebCGM semantics explicit and diagnostic.
- Gather evidence about whether the shared vector boundary survives a new,
  structurally different producer.

## Non-Goals

- Complete ISO 8632 or WebCGM conformance.
- Supporting every CGM encoding in the first milestone.
- Implementing WebCGM DOM APIs, scripting, hyperlinks, or browser behavior.
- Implementing CGM text, font selection, or text layout as vector paths.
- Implementing embedded raster images in the first geometry profile.
- Supporting CALS, ATA, S1000D, or another application profile merely because
  fixtures use CGM.
- Replacing SVG, UI, font, or Lucide providers with CGM abstractions.
- Exposing CGM element names through `VectorPath` or renderer contracts.
- Creating `tokimu-cgm` before an independent consumer and admission review
  justify promotion.
- Treating reference-image similarity as the sole geometry authority.
- Bulk-importing every available fixture before focused cases produce useful
  diagnostics.

## Upstream Candidates

### WebCGM test suite

The first acquisition candidate is the CGM Open WebCGM test suite:

```text
https://www.cgmopen.org/resources/test/
```

The acquisition slice must determine:

- whether the downloadable archive is still reachable;
- the archive's exact version and hash;
- its license or redistribution terms;
- which WebCGM versions and profiles are represented;
- which files are source CGM, reference images, XCF, HTML, scripts, or support
  material;
- which CGM encodings occur in the suite;
- whether individual cases can run independently from the original browser
  harness.

Site availability is not a fixture guarantee. If the original archive cannot
be fetched reproducibly, record the failure and investigate a trustworthy
mirror without representing the mirror as the original source.

### NIST methodology

The NIST report _Detailed Design Specification for Conformance Testing of
Computer Graphics Metafile (CGM) Interpreter Products_ is methodology and
historical evidence:

```text
https://www.nist.gov/publications/detailed-design-specification-conformance-testing-computer-graphics-metafile-cgm
```

It does not count as an executable fixture corpus unless associated test data
is located, its provenance is verified, and its redistribution terms permit
local inclusion.

## Proposed Fixture Layout

```text
third-party/fixtures/webcgm-test-suite/
    provenance.json
    README.md
    upstream/
    selected/
        selection-v1.toml
        feature-matrix.md
```

Rules:

- `upstream/` preserves the acquired source files verbatim.
- `selected/selection-v1.toml` references upstream files instead of copying
  them.
- reduced fixtures, if needed, live under `selected/derived/` and record their
  source case and every deliberate semantic removal.
- license and provenance records remain adjacent to the fixture source.
- downloaded archives and temporary extraction directories do not become
  additional authoritative copies.
- generated artifacts belong under the existing corpus artifact root, not
  beside third-party source fixtures.

## Coverage Accounting

The WebCGM suite may mix geometry, DOM, interaction, hyperlink, profile, and
reference-image cases. Tokimu must therefore report several denominators
separately:

1. Total upstream test cases in the pinned suite.
2. Total upstream source CGM files.
3. Total cases classified as geometry-relevant.
4. Unique upstream geometry cases represented by the selection.
5. Selected encoding variants.
6. Derived or synthetic cases.
7. Cases reaching decode, semantic, vector, mesh, image, or explicit
   unsupported stages.
8. Feature capability status from the matrix.

The primary geometry-selection metric is:

```text
unique upstream geometry cases represented
------------------------------------------ x 100
geometry-relevant cases in the pinned suite
```

Also publish the broader suite percentage:

```text
unique upstream cases represented
--------------------------------- x 100
all cases in the pinned suite
```

Neither percentage is a conformance score. Passing several derived cases from
one source does not increase unique upstream coverage.

The current acquisition and selection baseline is:

```text
955 acquired upstream files
353 acquired CGM source files
341 acquired reference PNG files
232 published static case IDs
15 selected unmodified static cases
0 decoded cases
0 lowered cases
0 conformance claims
```

The 15 selected cases represent 4.2% of the 353 CGM files in the archive.
This is a source-file ratio, not the geometry-selection metric and not a
conformance score. The geometry-relevant denominator remains pending full
classification of the upstream suite.

## First Selection Profile

The first selection should favor independent, bounded geometry evidence.
Candidate case IDs must be verified against the acquired suite before they are
added to the manifest.

| Priority | Capability | Intended evidence |
| --- | --- | --- |
| 1 | Metafile and picture lifecycle | descriptor order, picture boundaries, explicit defaults |
| 1 | VDC extent and scaling | source coordinate normalization without renderer assumptions |
| 1 | Polyline | open path, line attributes, stroke evidence |
| 1 | Polygon | closed contour, fill and edge distinction |
| 1 | Rectangle | primitive lowering and finite bounds |
| 1 | Circle | curved closed geometry through shared flattening |
| 1 | Ellipse | non-circular radii and orientation |
| 2 | Circular and elliptical arcs | endpoint, center, direction, closure, and arc type |
| 2 | Polygon set | multiple contours, visibility flags, and topology pressure |
| 2 | Fill style and color | stateful attribute resolution into paint intent |
| 2 | Line, edge, cap, and join attributes | separation of source state from shared stroke expansion |
| 2 | Clipping rectangle | CGM clip state lowered into the admitted clipping boundary |
| 2 | Transform or mapping pressure | source coordinate mapping and composed presentation geometry |
| 3 | Multiple pictures | lifecycle reset and independent state |
| 3 | Cell array | explicit deferred raster boundary |
| 3 | Text | explicit deferred text/presentation boundary |

The v1 selection should contain roughly 10 to 25 focused cases. More cases are
admitted only after failures remain attributable to a stable stage.

## Candidate Intermediate Model

The first importer should not lower raw element bytes directly into
`VectorPath`. Introduce an example-side CGM semantic record sufficient to make
source ownership and state resolution observable:

```rust
pub struct CgmDocument {
    pub source: CgmSourceInfo,
    pub metafile: CgmMetafileDescriptor,
    pub pictures: Vec<CgmPicture>,
}

pub struct CgmPicture {
    pub name: Option<String>,
    pub descriptor: CgmPictureDescriptor,
    pub records: Vec<CgmPresentationRecord>,
}

pub struct CgmPresentationRecord {
    pub primitive: CgmPrimitive,
    pub resolved_style: CgmResolvedStyle,
    pub clip: Option<CgmClipState>,
    pub source_element: CgmElementIdentity,
}
```

These are candidate shapes, not admitted engine APIs. They should preserve:

- picture lifecycle;
- VDC type, extent, precision, and mapping decisions;
- element identity and source offsets;
- resolved state at each primitive;
- primitive-specific source parameters;
- unsupported elements and profile diagnostics;
- source provenance.

Only after state is resolved should a CGM adapter produce provider-neutral
vector and paint records.

## Encoding Policy

CGM defines binary, character, and clear-text encodings. WebCGM fixture
inventory must establish which encodings are actually present.

The first implementation admits exactly one encoding based on fixture evidence.
Preference should go to the encoding that:

- is used by the selected geometry cases;
- can be identified deterministically;
- has available normative or trustworthy decoding references;
- permits bounded parsing with explicit length and precision checks.

Other encodings must produce an explicit `unsupported-encoding` diagnostic.
Encoding detection must not guess based on partial successful parsing.

## Structural Validation

Every admitted case should validate applicable invariants:

- bounded and deterministic decoding;
- valid element class, ID, length, partition, and parameter boundaries;
- supported precision and coordinate types;
- explicit metafile, picture, and picture-body lifecycle;
- finite source and normalized coordinates;
- finite and ordered VDC extents;
- resolved line, edge, fill, color, and clipping state;
- preserved open and closed topology;
- no silently discarded primitives or trailing parameters;
- finite vector bounds;
- finite mesh vertices and indices;
- indices within range;
- no unexpected zero-area triangles;
- expected contour and connected-component counts;
- output bounded by the resolved clipping region when clipping is admitted;
- repeatable fingerprints for identical input and policy.

State resets at metafile and picture boundaries need dedicated tests. A
stateful interpreter that leaks one fixture's attributes into another is a
corpus failure even when the resulting geometry remains finite.

## Artifact Model

Reuse the stage-aware artifacts already established by
`presentation-geometry-corpus`:

```text
source metadata
decode.json
cgm.json
vector.json
mesh.json
mesh-fingerprint.json
graph.json
report.json
contours.svg
mesh.svg
optional deterministic CPU image
optional separately labeled native screenshot
```

CGM-specific artifacts should add:

- source encoding and profile;
- element counts by class and ID;
- source offsets or record identities;
- metafile and picture descriptors;
- VDC type, precision, extent, and normalization policy;
- resolved attribute-state changes;
- unsupported element inventory;
- reference-image identity when supplied upstream.

Structural artifacts are authoritative for geometry validation. Reference
images are complementary evidence and do not replace stage assertions.

The first stage whose artifact diverges is the owning diagnostic boundary.

## Failure Semantics

Failures must be explicit and stage-owned:

| Boundary | Example failure |
| --- | --- |
| Acquisition | unreachable archive, hash mismatch, unclear redistribution terms |
| Encoding | unsupported encoding, truncated command, invalid partition or length |
| CGM semantics | invalid element order, unsupported precision, missing picture body |
| State resolution | invalid color index, undefined bundle, leaked picture state |
| Primitive lowering | unsupported primitive, malformed polygon set, non-finite arc |
| Vector | lost closure, unsupported topology, invalid clip |
| Mesh | tessellation error, non-finite vertex, degenerate output |
| Reference evidence | missing or mismatched reference image identity |

The importer must not:

- silently skip unknown primitives while claiming a structural pass;
- substitute a different precision, fill style, or encoding;
- flatten CGM text into placeholder rectangles;
- treat raster data as a vector pass;
- use a reference image to infer missing source semantics;
- continue across a malformed element boundary when the next offset is not
  trustworthy.

Unsupported cases should still emit source, provenance, decode, graph, and
report artifacts when those stages are valid.

## Implementation Location

Incubate reusable CGM parsing and lowering under:

```text
examples/lib-example/cgm-corpus/
```

Register CGM presentation cases through:

```text
examples/lib-example/presentation-geometry-corpus/
```

Add a focused visual browser only after structural artifacts are useful:

```text
examples/ui/hello-ui-cgm/
```

Do not create `tokimu-cgm` during this plan.

## Implementation Slices

### Slice 0: Verify Sources And Ownership

Deliverables:

- [x] Verify the CGM Open suite location and archive availability.
- [x] Locate and record version, profile, license, and redistribution terms.
- [x] Record the NIST report as methodology evidence.
- [x] Search the repository for an existing CGM parser, fixture, or overlapping
      geometry importer before adding a new abstraction.
- [x] Record the proposed importer, vector, renderer, and corpus ownership
      boundaries.

Acceptance criteria:

- [x] At least one lawful, reproducible fixture source is identified, or the
      plan records a concrete acquisition blocker.
- [x] No fixture is committed without provenance and redistribution evidence.
- [x] CGM semantics remain importer-owned.
- [x] No first-party crate or new engine boundary is introduced.

### Slice 1: Acquire And Inventory The Upstream Suite

Deliverables:

- [x] Add a bounded preparation script under `scripts/`.
- [x] Download or ingest the exact upstream archive without silently selecting
      a moving latest version.
- [x] Verify and record the archive hash.
- [x] Preserve upstream contents verbatim.
- [x] Inventory file types, case IDs, profiles, encodings, reference images,
      harness files, and support assets.
- [x] Add `provenance.json`, fixture `README.md`, and source license records.
- [x] Keep temporary archives and extraction work outside authoritative fixture
      paths and out of Git.

Acceptance criteria:

- [x] Re-running preparation either reproduces the same fixture identity or
      fails with a useful mismatch diagnostic.
- [ ] The inventory supplies stable denominators for total cases, source CGM
      files, geometry-relevant cases, and encodings.
- [x] Upstream source files are byte-identical to the pinned archive.
- [x] The preparation script does not rewrite selected manifests or reviewed
      artifacts implicitly.

### Slice 2: Classify And Select Corpus V1

Deliverables:

- [ ] Classify upstream cases as geometry, text, raster, DOM, hyperlink,
      profile, interaction, or support material.
- [x] Create `selection-v1.toml`.
- [x] Create `feature-matrix.md`.
- [x] Select 10 to 25 high-return geometry cases.
- [x] Record capability, reason, expected stage, encoding, profile, and explicit
      unsupported boundaries for every case.
- [x] Record reference-image identity without treating it as structural truth.

Acceptance criteria:

- [x] Every selected case has one authoritative upstream source.
- [ ] Selection and coverage counts are reproducible from the inventory.
- [x] Text, raster, DOM, hyperlink, and interaction cases are not reported as
      geometry failures.
- [x] No derived fixture is counted as another unique upstream case.

### Slice 3: Decode One Encoding Safely

Deliverables:

- [ ] Create `examples/lib-example/cgm-corpus`.
- [ ] Detect and admit one verified fixture encoding.
- [ ] Decode bounded element headers, lengths, partitions, parameters, and
      source offsets.
- [ ] Decode the minimal metafile and picture lifecycle.
- [ ] Preserve unsupported elements as structured diagnostics.
- [ ] Add malformed, truncated, oversized, and unsupported-encoding tests.

Acceptance criteria:

- [ ] One selected fixture decodes deterministically into inspectable elements.
- [ ] Invalid lengths and partitions cannot panic or read out of bounds.
- [ ] Unknown and unsupported elements identify class, ID, picture, and source
      offset where available.
- [ ] Other encodings fail explicitly.
- [ ] Decoding performs no rendering and creates no `VectorPath`.

### Slice 4: Resolve CGM State And Coordinates

Deliverables:

- [ ] Decode the required VDC type, precision, extent, scaling, and color
      descriptors for selected cases.
- [ ] Model metafile, picture, and picture-body state transitions.
- [ ] Resolve line, edge, fill, color, and clipping attributes needed by v1.
- [ ] Reset state at documented lifecycle boundaries.
- [ ] Record normalization and source-to-presentation transforms.

Acceptance criteria:

- [ ] Selected primitives carry deterministic resolved state.
- [ ] Two pictures cannot leak style or clipping state into one another.
- [ ] Unsupported precision and color models fail at the CGM semantic stage.
- [ ] Normalization produces finite coordinates and preserves orientation.
- [ ] Renderer dimensions do not influence semantic interpretation.

### Slice 5: Lower Basic Primitives

Deliverables:

- [ ] Lower polyline, polygon, rectangle, circle, and ellipse.
- [ ] Preserve open versus closed source topology.
- [ ] Keep fill and edge intent distinct.
- [ ] Reuse shared vector curves, flattening, fill, and stroke contracts where
      they already exist.
- [ ] Emit `cgm.json` and `vector.json`.

Acceptance criteria:

- [ ] At least five semantically distinct selected cases reach vector evidence.
- [ ] Primitive bounds and contour counts match expected structural evidence.
- [ ] No CGM element names or profile types enter shared vector contracts.
- [ ] Unsupported primitives emit explicit diagnostics without being dropped.
- [ ] Repeated runs produce identical normalized vector fingerprints.

### Slice 6: Add Arcs, Polygon Sets, And Clipping

Deliverables:

- [ ] Lower selected circular and elliptical arc forms.
- [ ] Preserve arc direction and closure semantics.
- [ ] Interpret one bounded polygon-set profile.
- [ ] Lower one admitted clipping-rectangle profile.
- [ ] Add focused synthetic cases for ambiguous or malformed boundaries exposed
      by upstream inputs.

Acceptance criteria:

- [ ] Arc endpoints, direction, closure, and bounds are structurally asserted.
- [ ] Polygon-set topology is preserved or rejected explicitly.
- [ ] Clipped output remains within the resolved clip region.
- [ ] Synthetic diagnostics do not increase upstream coverage.
- [ ] Geometry failures remain attributable to CGM lowering, vector, or mesh.

### Slice 7: Integrate Mesh And Stage Artifacts

Deliverables:

- [ ] Register CGM cases with `presentation-geometry-corpus`.
- [ ] Emit mesh, fingerprint, graph, report, contour, and mesh-view artifacts.
- [ ] Record decoder, normalizer, flattening, stroke, and tessellator algorithm
      identities.
- [ ] Add structural assertions for finite geometry, indices, degenerates,
      bounds, contours, and components.
- [ ] Clear stale downstream artifacts after an earlier-stage failure.

Acceptance criteria:

- [ ] At least one open stroke and one closed fill reach finite mesh evidence.
- [ ] The first divergent stage is visible in `graph.json` and `report.json`.
- [ ] Ordinary corpus runs do not mutate reviewed golden artifacts.
- [ ] Golden updates are explicit and case-scoped.
- [ ] The corpus harness contains no duplicate CGM parser or lowering logic.

### Slice 8: Add Visual And Reference Evidence

Deliverables:

- [ ] Save deterministic CPU images through the existing screenshot helper.
- [ ] Record dimensions, color assumptions, source stage, and image hash.
- [ ] Associate upstream reference images by case identity when available.
- [ ] Add optional manual native-window screenshots as separately labeled
      evidence.
- [ ] Evaluate differential image comparison only after structural stages are
      stable.

Acceptance criteria:

- [ ] Image evidence never replaces source, semantic, vector, or mesh
      assertions.
- [ ] Reference images are never reported as generated Tokimu output.
- [ ] Color-space, antialiasing, and tolerance policy are recorded before
      differential comparison.
- [ ] Headless structural validation remains usable without a window or GPU.

### Slice 9: Expand The Selection By Feature

Deliverables:

- [ ] Add cases in small primitive- or state-oriented batches.
- [ ] Prioritize independent upstream cases over many variations of one source.
- [ ] Recalculate all coverage denominators after every admitted batch.
- [ ] Add explicit expected-invalid and expected-unsupported cases.
- [ ] Record runtime and keep focused local execution practical.

Acceptance criteria:

- [ ] Every batch has a named capability target and likely owning stage.
- [ ] Passing case count is not substituted for upstream coverage.
- [ ] Broad runs can be filtered by case, primitive, profile, encoding, and
      stage.
- [ ] Unsupported cases remain useful evidence rather than silent exclusions.

### Slice 10: Architectural Review

Deliverables:

- [ ] Record whether CGM is a genuinely independent producer of the shared
      presentation-geometry boundary.
- [ ] Record whether native curve ownership remains importer-side or gains new
      pressure for shared curve representation.
- [ ] Record whether stateful paint and clipping resolution expose missing
      provider-neutral contracts.
- [ ] Evaluate whether `cgm-corpus` remains example support or has a real
      non-example consumer.
- [ ] Keep parser/provider promotion separate from presentation-geometry
      findings.
- [ ] Update the SDD, roadmap, Architectural Review, or ADR only when observed
      evidence changes an accepted boundary.

Acceptance criteria:

- [ ] The review distinguishes source-format support from geometry capability
      evidence.
- [ ] At least two independent producers share every proposed promoted contract.
- [ ] No crate is promoted solely because the fixture corpus exists.
- [ ] Deferred text, raster, interaction, encoding, and profile questions remain
      documented.

## First Working Milestone

The first milestone is:

```text
one pinned CGM geometry fixture
    -> one verified encoding
    -> metafile and picture lifecycle
    -> one polyline and one polygon
    -> CGM semantic artifact
    -> provider-neutral vector artifact
    -> finite stroke and fill meshes
    -> stage report and deterministic fingerprints
```

This milestone does not require arcs, text, raster data, WebCGM interaction,
reference-image comparison, all encodings, a visual browser, or a first-party
CGM crate.

## Validation

Focused validation should include:

```powershell
cargo test -p cgm-corpus
cargo test -p presentation-geometry-corpus --lib
cargo fmt --all
```

Before a checkpoint that changes shared public APIs, also run:

```powershell
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Acquisition validation should verify:

- archive and selected-source hashes;
- fixture counts and selection references;
- license and provenance presence;
- no missing support files;
- no implicit network access during ordinary tests.

## Risks And Mitigations

### The Corpus Becomes A General CGM Product

Risk: geometry testing expands into complete profile, DOM, browser, text, and
raster support.

Mitigation: admit cases by current architectural question and record every
other feature as an explicit boundary.

### Stateful Interpretation Leaks Into Vector

Risk: shared geometry types begin carrying CGM bundles, element classes, or
picture state.

Mitigation: resolve source state into provider-neutral records before vector
lowering.

### Multiple Encodings Obscure Semantic Failures

Risk: binary, character, and clear-text decoding are implemented together and
make failures difficult to localize.

Mitigation: admit one verified encoding first and preserve encoding-neutral
semantic tests separately.

### Reference Images Become The Authority

Risk: a visually similar output hides incorrect topology, state, or source
interpretation.

Mitigation: require semantic, vector, and mesh evidence before image
comparison.

### Upstream Availability Or Licensing Is Unclear

Risk: an old suite disappears or cannot lawfully be redistributed.

Mitigation: verify provenance and terms before committing fixtures; retain the
plan and inventory tooling even if acquisition must stop.

### Format Parser Becomes Engine Architecture

Risk: a test importer is promoted because it handles many fixtures.

Mitigation: keep it under example support until a real application consumer
and architectural review justify admission.

### Bulk Coverage Produces Unactionable Failures

Risk: hundreds of cases fail across unrelated semantics.

Mitigation: start with a small selected profile, filter by stage and feature,
and expand only after artifacts localize failures.

## Acceptance Criteria

This plan is complete when:

- a lawful upstream source is pinned with reproducible provenance;
- suite and geometry-relevant denominators are recorded;
- a versioned selection and feature matrix explain every admitted case;
- one encoding decodes with bounded failure semantics;
- metafile, picture, state, and coordinate semantics are inspectable;
- basic primitives, at least one arc, and one clipping case reach the shared
  vector boundary or an explicit unsupported boundary;
- open stroke and closed fill cases produce deterministic structural artifacts;
- CGM, vector, mesh, and optional image failures are distinguishable;
- source-format semantics do not leak into vector or renderer contracts;
- coverage reporting makes no conformance claim;
- architectural review records what CGM establishes about shared presentation
  geometry and what remains importer-specific.

## Graduation Criteria

The CGM importer or corpus support should be considered for promotion beyond
`examples/lib-example` only when:

- at least one non-example consumer needs CGM import;
- public semantic contracts have survived independent fixtures and another
  caller;
- encoding, state, and primitive diagnostics are stable;
- provider implementation details do not leak through public APIs;
- promotion simplifies ownership compared with continued incubation;
- an Architectural Review explicitly recommends admission.

Until then, CGM remains an evidence-producing external format adapter.

## References

- `docs/Conversations/cgm corpus tests.md`
- `docs/Libraries/w3c-svg-corpus-testing.md`
- `docs/Libraries/khronos-gltf-corpus-testing.md`
- `docs/Plans/presentation-geometry-corpus-harness.md`
- `docs/testing-strategy.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/Architectural Reviews/AR-0001-shared-vector-presentation-geometry.md`
- `examples/lib-example/presentation-geometry-corpus`
- CGM Open WebCGM test resources:
  `https://www.cgmopen.org/resources/test/`
- NISTIR 5146:
  `https://www.nist.gov/publications/detailed-design-specification-conformance-testing-computer-graphics-metafile-cgm`
