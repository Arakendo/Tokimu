# FBX Corpus Testing

## Status

As of 2026-07-28, corpus acquisition and selection v1 are complete. Tokimu now
contains 23 checksum-pinned FBX cases representing 14 logical scenes from the
`ufbx` repository at tag `v0.23.0`, with 13 verified external dependencies.
Tokimu does not yet contain an FBX decoder, importer, or admitted FBX
capability.

Unlike SVG and glTF, FBX does not appear to have a public standards-owned
conformance suite with stable expected results. The first work is therefore to
verify and assemble a lawful, reproducible engineering corpus from independent
sources without presenting it as official Autodesk conformance.

## Purpose

Build a deliberately selected FBX corpus that exercises the 3D asset pipeline
at stable ownership boundaries:

```text
FBX fixture
    -> FBX decoding and source-scene interpretation
    -> Tokimu-owned imported model evidence
    -> structural validation and diagnostic artifacts
    -> format-agnostic renderer submission
```

The corpus exists to discover and validate Tokimu's scene, transform, mesh,
material, animation, skeleton, skinning, and morph-target boundaries. FBX is a
source format. It must not become Tokimu's canonical scene or model
representation.

The primary architectural question is:

> Which model and scene concepts remain stable when FBX and glTF independently
> lower into Tokimu-owned meaning?

## Motivation

glTF and FBX provide different architectural pressure.

glTF strongly exercises:

- JSON and GLB containers;
- buffers, buffer views, and accessors;
- explicit extension negotiation;
- PBR material interchange;
- a compact runtime-delivery format.

FBX strongly exercises:

- connected object graphs;
- transform inheritance and pivots;
- animation curves and layered animation;
- skeletons and skinning;
- blend shapes;
- exporter-specific interpretation and normalization;
- binary and ASCII encodings across format versions.

The shared pressure is useful only when neither importer defines the canonical
model:

```text
FBX importer
       \
        -> Tokimu imported model -> renderer/runtime capabilities
       /
glTF importer
```

If both importers naturally lower into the same provider-neutral contracts,
that is evidence for Tokimu model, mesh, animation, and skeleton capabilities.
The existence of an FBX corpus alone does not admit those capabilities.

## Architectural Ownership

```text
FBX importer
    owns FBX encodings, records, object and connection graphs, source
    transforms, exporter quirks, source diagnostics, and provenance

Tokimu model and scene semantics
    own provider-neutral nodes, hierarchy, transforms, mesh references,
    materials, animations, skeletons, morph targets, and diagnostics

Tokimu mesh and animation capabilities
    own renderer- and format-neutral geometry and evaluated motion contracts

Renderer
    owns GPU execution, uploads, batching, pipelines, and cache lifetime

Corpus harness
    owns case registration, stage artifacts, comparisons, fingerprints,
    reports, and reviewed evidence
```

`ufbx`, the Autodesk SDK, Blender, Maya, 3ds Max, or another exporter or loader
must not escape the importer or corpus boundary. Their types and behavior may
provide evidence, but they do not define Tokimu's public model.

## Goals

- Identify a lawful, reproducible public FBX fixture source.
- Preserve source provenance, versions, checksums, and per-case licenses.
- Build a small exporter-diverse v1 selection with a reason for every case.
- Exercise binary FBX first while keeping ASCII FBX an explicit independent
  encoding target.
- Decode source records with bounded failure semantics.
- Preserve an inspectable FBX object and connection graph before lowering.
- Lower one static mesh and node transform into Tokimu-owned evidence.
- Add hierarchy, materials, animation, skinning, and blend shapes in separate
  feature batches.
- Compare selected structural results against an independent implementation
  where lawful and practical.
- Reuse the glTF corpus vocabulary when the semantics genuinely match.
- Distinguish decode, source interpretation, canonical lowering, geometry,
  animation, and rendering failures.
- Report coverage against pinned denominators rather than pass counts.

## Non-Goals

- Claiming complete Autodesk FBX compatibility or conformance.
- Porting `ufbx` line by line before corpus evidence identifies Tokimu's needed
  subset.
- Making the Autodesk FBX SDK a runtime or engine dependency.
- Treating `ufbx` output as Tokimu's canonical scene model.
- Supporting every historical FBX version in the first milestone.
- Implementing binary and ASCII decoders simultaneously.
- Admitting NURBS, subdivision surfaces, geometry caches, constraints, or every
  material model in v1.
- Normalizing away source transforms or exporter behavior without recording
  the decision.
- Rendering directly from FBX-native objects.
- Bulk-committing a multi-gigabyte dataset without a reviewed selection and
  repository-size policy.
- Promoting an FBX crate solely because corpus fixtures exist.

## Corpus Layers

FBX lacks one authoritative standards corpus, so coverage should combine three
separately reported layers.

### Community compatibility corpus

The `ufbx` project is the primary acquisition candidate because its public test
infrastructure is designed around broad FBX compatibility, malformed inputs,
multiple exporters, binary and ASCII encodings, and semantic checks.

Acquisition must verify:

- the exact repository and dataset revision;
- fixture and test-code licenses;
- whether the large public dataset may be redistributed;
- whether test data is stored in Git, Git LFS, archives, or external hosting;
- whether expected results can be consumed independently from `ufbx`;
- whether selected cases can be pinned without importing the whole dataset;
- whether exporter-generated assets carry additional restrictions.

`ufbx` is an independent implementation and potential differential oracle. It
is not an official Autodesk conformance authority.

### Vendor and exporter samples

Candidate sources include redistributable samples from:

- Autodesk FBX SDK distributions;
- Blender;
- Maya;
- 3ds Max;
- MotionBuilder;
- other tools that explicitly permit redistribution of exported fixtures.

Every candidate must pass a per-source license review. An installed SDK,
application, or locally exported file is not automatically redistributable.

### Tokimu production and synthetic corpus

Tokimu-owned fixtures should isolate behaviors discovered in public and
production files:

```text
static-mesh/
    triangle
    indexed-cube
    split-normals
    multiple-material-slots

transforms/
    parent-child
    rotated-parent
    negative-scale
    non-uniform-scale
    pivots-and-pre-rotation

animation/
    translation
    rotation
    scale
    stepped-curve
    layered-clips

deformation/
    simple-skin
    multiple-influences
    blend-shape

invalid/
    truncated-header
    invalid-offset
    missing-connection
    cyclic-hierarchy
    non-finite-value
```

Synthetic and production cases provide focused regression evidence. They do
not increase public-corpus coverage.

## Proposed Fixture Layout

```text
third-party/fixtures/fbx-corpus/
    README.md
    provenance.json
    inventory.json
    upstream/
    selected/
        selection-v1.toml
        feature-matrix.md
```

Rules:

- `upstream/` contains only reviewed, pinned source fixtures.
- Selection manifests reference upstream paths instead of copying files.
- Large optional datasets remain acquired through a reproducible script unless
  repository policy explicitly admits them.
- Download archives and temporary extraction directories belong under
  `target/`, not beside authoritative fixtures.
- Every selected case records its own license, source revision, checksum,
  exporter where known, encoding, FBX version, and expected diagnostic stage.
- Reduced fixtures record their parent case and every deliberate modification.
- Generated reports belong under the corpus artifact root, not with source
  fixtures.
- Local proprietary production files remain outside Git and cannot contribute
  to public coverage totals.

## Coverage Accounting

FBX coverage has no honest single denominator. Report these measures
independently:

1. Public source datasets and revisions pinned.
2. Total fixture files in each pinned dataset.
3. Unique logical scenes represented by the selected profile.
4. Exporters represented.
5. FBX versions represented.
6. Binary and ASCII encoding cases.
7. Selected cases reaching decode, source graph, canonical model, mesh,
   animation, deformation, and render stages.
8. Synthetic and private production cases.
9. Expected-invalid and expected-unsupported cases.
10. Feature status from the matrix.

The primary public-dataset metric is per source:

```text
unique upstream scenes represented
---------------------------------- x 100
scenes in the pinned upstream dataset
```

Exporter, version, and feature coverage must be reported as matrices rather
than rolled into that percentage. Several exports of one logical scene are
valuable compatibility cases but are not several independent model designs.

Passing case count must never be described as FBX conformance.

## First Selection

The v1 profile should be small enough that each failure has a likely owner.

| Capability | Candidate pressure | Intended evidence |
| --- | --- | --- |
| Binary container | minimal binary FBX | header, version, records, offsets, bounds |
| Source graph | one mesh node | objects, connections, stable IDs |
| Static geometry | triangle or cube | positions, indices, polygon boundaries |
| Vertex mapping | split normals and UVs | control points versus polygon vertices |
| Hierarchy | parent and child nodes | local and world transforms |
| Transform semantics | pivoted or pre-rotated node | source transform evaluation and normalization |
| Material binding | two material slots | polygon material assignment and references |
| ASCII encoding | equivalent minimal scene | independent syntax, same semantic result |
| Animation | one translation or rotation track | stacks, layers, curves, time bounds |
| Skinning | one small skeleton | joints, clusters, inverse bind data, weights |
| Blend shape | one target | shape deltas, channels, weights |
| Malformed input | truncated or invalid record | bounded deterministic diagnostics |

The first working batch should stop at binary static geometry and hierarchy.
Materials, ASCII, animation, skinning, and blend shapes should enter as
separate batches.

## Acquisition Result

Selection v1 uses tracked fixtures from the `ufbx` repository at revision
`fcc5d6ba444cfd3eb80677dba5e37e493941abe5` (`v0.23.0`). The repository-level
license offers MIT or Unlicense terms; Tokimu retains the notice and selects
the MIT alternative.

The pinned repository contains 1,456 FBX files totaling 45,918,954 bytes.
Selection v1 commits 23 FBX cases and 13 OBJ reference artifacts totaling
approximately 1.26 MB with the retained license. The complete clone remains an
optional cache under `target/`.

The separate public `ufbx` dataset described upstream was not admitted because
it is approximately 4.7 GB and requires a separate redistribution and
repository-size review. Autodesk SDK samples and other vendor samples were not
admitted because their per-file redistribution terms have not yet been
reviewed. Neither omission reduces the selected `ufbx` denominator or creates
an unsupported compatibility claim.

## Canonical Import Evidence

The corpus should lower into an evolving Tokimu-owned candidate rather than a
publicly stabilized FBX-shaped API:

```text
ImportedModel
    source identity and provenance
    scenes
    named nodes and hierarchy
    local transforms
    mesh and primitive references
    vertex attribute streams
    polygon or triangle topology
    material and texture references
    optional skeletons and skins
    optional animation clips and channels
    optional morph targets
    source diagnostics
```

This candidate must be compared with the glTF corpus model. Shared concepts
should be promoted only when both importers use them without source-format
details leaking into the contract.

FBX-specific concepts may remain in source evidence when no honest
provider-neutral equivalent exists. Examples include source object IDs,
connection types, pivots, pre/post rotations, geometric transforms, animation
stacks and layers, and exporter metadata.

## Differential Validation

`ufbx` may serve as an optional structural oracle during importer development:

```text
FBX fixture
    -> ufbx adapter -> normalized comparison artifact

FBX fixture
    -> Tokimu importer -> normalized comparison artifact

normalized artifacts
    -> structural diff
```

The comparison artifact should include only agreed observations:

- source version and encoding;
- object and connection counts;
- node hierarchy and names;
- local and world transforms;
- mesh, polygon, and vertex counts;
- bounds;
- material assignments;
- animation stack, layer, curve, and key counts;
- skeleton, skin, and blend-shape summaries.

Differential agreement is evidence, not proof. A mismatch does not establish
which implementation is correct, and an agreement may reproduce a shared
interpretation error. Tokimu-owned assertions and reviewed fixtures remain
necessary.

If `ufbx` is integrated:

- it stays behind an example-side or tooling-only adapter;
- its license notice is retained;
- no `ufbx` type enters Tokimu contracts;
- ordinary engine builds do not compile or link it;
- replacing it does not change corpus case identity.

## Diagnostic Artifacts

Each selected case should produce stage-specific evidence:

```text
reports/<case-id>/
    input.json
    source-records.json
    objects.json
    connections.json
    source-scene.json
    model.json
    meshes.json
    transforms.json
    materials.json
    animations.json
    skins.json
    morphs.json
    bounds.json
    topology.json
    comparison.json
    graph.json
    report.json
    wireframe.png
    normals.png
    final.png
```

Artifacts should record:

- schema and algorithm versions;
- input and dependency hashes;
- importer implementation identity;
- source version and encoding;
- normalization policy;
- coordinate-system and unit conversion;
- elapsed stage timings;
- warnings, unsupported features, and failure ownership.

The first stage whose artifact diverges is the owning diagnostic boundary.
Structural artifacts are authoritative for importer validation. Images provide
complementary evidence and do not prove transform, topology, animation, or
deformation correctness.

## Structural Validation

At minimum, selected cases should validate:

- bounded record offsets and property lengths;
- finite numeric properties;
- stable object IDs and valid connection targets;
- acyclic scene traversal or explicit cycle diagnostics;
- finite local and world transforms;
- declared unit and axis conversion;
- polygon indices within the control-point range;
- layer-element mapping and reference modes;
- finite positions, normals, tangents, UVs, colors, and weights;
- material-slot references;
- finite bounds containing all geometry;
- animation key ordering, interpolation, and time bounds;
- skin joint and weight references;
- morph target vertex correspondence;
- deterministic repeated output.

Not every FBX scene is triangulated, manifold, consistently wound, or expressed
in Tokimu's coordinate system. Those properties should be interpreted and
reported at the correct stage rather than universally rejected.

## Implementation Slices

### Slice 0: Acquire And License The Corpus

Deliverables:

- [x] Verify the `ufbx` repository, test-data source, revision, and licenses.
- [x] Inventory dataset size, storage mechanism, fixture count, and expected
      result formats.
- [x] Determine whether selected fixtures may be committed or must remain
      reproducibly downloaded.
- [ ] Review at least one independent exporter or vendor sample source.
- [x] Record rejected sources and the reason for rejection.

Acceptance criteria:

- [x] Every admitted byte has recorded provenance and redistribution terms.
- [x] Acquisition is reproducible from a pinned revision or archive hash.
- [x] Ordinary tests perform no implicit network access.
- [x] Temporary archives and extraction trees remain outside authoritative
      fixtures.
- [x] No compatibility or conformance claim is made from source acquisition.

### Slice 1: Build Inventory And Selection

Deliverables:

- [x] Create `provenance.json`, `inventory.json`, `README.md`,
      `selection-v1.toml`, and `feature-matrix.md`.
- [x] Inventory logical scenes, encodings, versions, exporters, features, and
      dependencies where discoverable.
- [x] Select 10 to 25 high-return v1 cases.
- [x] Record case hashes, capabilities, reasons, licenses, expected stages, and
      unsupported boundaries.
- [x] Add a verifier for fixture and manifest integrity.

Acceptance criteria:

- [x] Selection and coverage counts reproduce from the inventory.
- [x] Every selected case has one authoritative source identity.
- [x] Binary, ASCII, exporter, and feature counts remain distinct.
- [x] Derived and synthetic fixtures do not increase upstream coverage.
- [x] Missing dependencies fail before parsing begins.

### Slice 2: Decode A Minimal Binary FBX

Status: complete on 2026-07-29 for the selected binary syntax profile.

Deliverables:

- [x] Create `examples/lib-example/fbx-corpus`.
- [x] Validate the binary signature and supported version.
- [x] Decode bounded node records, property arrays, strings, and raw values.
- [x] Preserve source offsets and record hierarchy.
- [x] Add truncated, oversized, invalid-offset, and unsupported-version tests.

Acceptance criteria:

- [x] One minimal fixture decodes into a deterministic source-record artifact.
- [x] Invalid lengths, offsets, and arrays cannot panic or read out of bounds.
- [x] Unsupported versions stop with a structured diagnostic.
- [x] Decoding performs no rendering and creates no Tokimu model objects.
- [x] Repeated runs produce identical source fingerprints.

Evidence:

- The selected Maya 6100 and 7500 binary cube fixtures exercise both 32-bit
  and 64-bit node headers.
- The decoder preserves record names, offsets, declared ends, properties, and
  child hierarchy in serializable source evidence.
- Compressed property arrays are decoded through bounded zlib input and checked
  against their declared element width.
- Unit coverage rejects invalid signatures, unsupported versions, truncated
  headers, out-of-range record ends, oversized inputs, and oversized arrays.

### Slice 3: Resolve Objects And Connections

Deliverables:

- [ ] Decode the minimal `Objects` and `Connections` profile used by the v1
      static-mesh case.
- [ ] Preserve stable source IDs, object classes, names, and connection types.
- [ ] Build an inspectable source scene graph.
- [ ] Detect missing targets, duplicate IDs, and hierarchy cycles.
- [ ] Emit `objects.json`, `connections.json`, and `source-scene.json`.

Acceptance criteria:

- [ ] One mesh node and its geometry connection are reconstructed.
- [ ] Source graph errors identify IDs, classes, and source offsets.
- [ ] Connection order does not silently redefine semantic ownership.
- [ ] No FBX-native object escapes the importer boundary.

### Slice 4: Lower Static Geometry

Deliverables:

- [ ] Decode control points and polygon vertex indices.
- [ ] Preserve polygon boundaries before triangulation.
- [ ] Decode one normal and one UV mapping/reference profile.
- [ ] Lower geometry into corpus-owned imported-model evidence.
- [ ] Emit mesh, attribute, topology, and bounds artifacts.

Acceptance criteria:

- [ ] One selected mesh reaches finite indexed geometry.
- [ ] Polygon and attribute indices are in range.
- [ ] Mapping and reference modes are preserved or rejected explicitly.
- [ ] Bounds contain every decoded position.
- [ ] The renderer receives no FBX path, object, or connection type.

### Slice 5: Resolve Hierarchy And Transforms

Deliverables:

- [ ] Decode local translation, rotation, and scale.
- [ ] Compose parent-child world transforms.
- [ ] Record source axes, handedness, units, and normalization.
- [ ] Add pivot, pre/post rotation, and geometric-transform cases
      incrementally.
- [ ] Compare a shared hierarchy case with glTF canonical evidence.

Acceptance criteria:

- [ ] Local and world transforms are finite and deterministic.
- [ ] Axis and unit conversion is explicit and testable.
- [ ] Source transform details do not leak into renderer contracts.
- [ ] Unsupported transform semantics stop before producing misleading world
      transforms.
- [ ] Equivalent FBX and glTF cases can be compared without format-native
      fields.

### Slice 6: Add Materials And Textures

Deliverables:

- [ ] Decode one bounded material and texture-reference profile.
- [ ] Preserve polygon material assignments.
- [ ] Resolve external texture dependencies through Tokimu asset identity.
- [ ] Record unsupported shading models and layered textures.
- [ ] Emit material and texture-reference artifacts.

Acceptance criteria:

- [ ] One multi-material mesh preserves its slot assignments.
- [ ] Missing textures fail at asset resolution, not rendering.
- [ ] FBX material classes do not become Tokimu material semantics.
- [ ] Renderer-native resources are absent from importer artifacts.

### Slice 7: Add Animation

Deliverables:

- [ ] Decode one animation stack, layer, curve node, curve, and key profile.
- [ ] Preserve time bounds and interpolation intent.
- [ ] Lower one translation or rotation channel into Tokimu-owned animation
      evidence.
- [ ] Add layered and held-position cases separately.
- [ ] Emit animation summaries and sampled validation artifacts.

Acceptance criteria:

- [ ] Keys are finite, ordered, and attached to a valid target property.
- [ ] Clip identity and source layer structure remain inspectable.
- [ ] Playback does not silently reset state between source clips.
- [ ] Runtime animation code depends on Tokimu contracts, not FBX objects.

### Slice 8: Add Skinning And Blend Shapes

Deliverables:

- [ ] Decode one skeleton and skin cluster profile.
- [ ] Resolve joint hierarchy, inverse bind data, and vertex influences.
- [ ] Decode one blend-shape target and channel.
- [ ] Validate weight normalization and target correspondence.
- [ ] Emit skin, skeleton, and morph artifacts.

Acceptance criteria:

- [ ] Joint and control-point references are valid and deterministic.
- [ ] Unsupported link modes and deformation profiles are explicit.
- [ ] Morph deltas match the base geometry domain.
- [ ] FBX deformation objects do not enter renderer contracts.

### Slice 9: Add ASCII FBX

Deliverables:

- [ ] Decode a minimal ASCII fixture into the same source-record model.
- [ ] Preserve textual source spans and diagnostics.
- [ ] Compare binary and ASCII exports of one logical scene.
- [ ] Add malformed token, nesting, number, string, and array cases.
- [ ] Keep syntax decoding separate from semantic graph interpretation.

Acceptance criteria:

- [ ] Equivalent binary and ASCII cases reach comparable source semantics.
- [ ] Syntax-specific details stop before canonical model lowering.
- [ ] Invalid text input cannot cause unbounded allocation or recursion.
- [ ] ASCII support does not weaken binary decoder diagnostics.

### Slice 10: Differential And Visual Evidence

Deliverables:

- [ ] Add an optional `ufbx` comparison adapter if licensing and build
      boundaries remain acceptable.
- [ ] Define a normalized comparison schema.
- [ ] Record implementation and algorithm identities.
- [ ] Save deterministic structural and visual artifacts.
- [ ] Keep native-window screenshots separately labeled as manual evidence.

Acceptance criteria:

- [ ] Differential results identify the first differing observation.
- [ ] `ufbx` is absent from ordinary engine dependency graphs.
- [ ] Comparison disagreement is not automatically classified as a Tokimu bug.
- [ ] Images never replace source, model, transform, or mesh assertions.
- [ ] Headless structural validation remains available without a GPU.

### Slice 11: Expand And Harden

Deliverables:

- [ ] Add exporter, version, and feature batches in small increments.
- [ ] Add expected-invalid and expected-unsupported cases.
- [ ] Fuzz bounded decoding surfaces.
- [ ] Track runtime, allocations, recursion depth, and artifact size.
- [ ] Recalculate all denominators after each admitted batch.

Acceptance criteria:

- [ ] Every batch has a named capability target and likely owning stage.
- [ ] Broad runs filter by case, exporter, version, encoding, feature, and
      stage.
- [ ] Unsupported cases remain useful evidence rather than silent exclusions.
- [ ] Large-file validation remains bounded and suitable for automation.
- [ ] Passing case count is not substituted for compatibility coverage.

### Slice 12: Architectural Review

Deliverables:

- [ ] Record which model, mesh, transform, animation, skeleton, and morph
      concepts survived both FBX and glTF pressure.
- [ ] Record which concepts remain source-format-specific.
- [ ] Decide whether `fbx-corpus` remains example support or has a real
      non-example consumer.
- [ ] Evaluate provider use, a native Rust importer, or a staged replacement
      without treating implementation choice as canonical semantics.
- [ ] Update the SDD, roadmap, Architectural Review, or ADR only when evidence
      changes an accepted boundary.

Acceptance criteria:

- [ ] The review distinguishes importer compatibility from engine capability
      admission.
- [ ] At least two independent producers share every proposed promoted
      contract.
- [ ] No crate is promoted solely because a large corpus exists.
- [ ] Deferred versions, exporters, encodings, and advanced features remain
      documented.

## First Working Milestone

```text
one pinned binary FBX fixture
    -> verified signature and version
    -> bounded source records
    -> Objects and Connections evidence
    -> one node and one static mesh
    -> explicit axes, units, and transform policy
    -> Tokimu-owned model and mesh artifacts
    -> finite bounds and topology report
    -> optional format-agnostic render
```

This milestone does not require ASCII FBX, materials, textures, animation,
skinning, blend shapes, NURBS, subdivision, differential `ufbx` integration,
or a first-party FBX crate.

## Highest-Return Priority

1. Reproducible acquisition, licensing, and inventory.
2. Binary record safety and source diagnostics.
3. Objects, connections, scene hierarchy, and stable IDs.
4. Control points, polygons, normals, and UV mapping modes.
5. Axis, unit, pivot, and transform evaluation.
6. Material slots and texture references.
7. Animation stacks, layers, curves, and clips.
8. Skeletons, skinning, and blend shapes.
9. ASCII encoding and exporter diversity.
10. Advanced surfaces, caches, constraints, and uncommon versions.

## Edge-Case Backlog

After the highest-return profile is stable, investigate:

- 32-bit versus 64-bit binary node headers;
- compressed and uncompressed property arrays;
- very large arrays and deep record nesting;
- duplicate object IDs and dangling connections;
- unusual names, namespaces, and Unicode strings;
- negative and non-uniform scales;
- transform inheritance modes;
- rotation order, pivots, offsets, and pre/post rotations;
- geometric transforms distinct from node transforms;
- multiple scenes or disconnected roots;
- n-gons, concave polygons, holes, and degenerate polygons;
- layer-element mapping and reference combinations;
- smoothing groups and hard-edge reconstruction;
- generated versus supplied normals and tangents;
- embedded media and relative path resolution;
- layered textures and legacy material models;
- animation layers, additive blending, and extrapolation;
- constraints and driven properties;
- multiple skins and influence link modes;
- blend-shape channels and in-between targets;
- NURBS, patches, subdivision surfaces, and geometry caches;
- cameras, lights, metadata, and application-specific properties;
- malformed, truncated, adversarial, and fuzz-generated inputs.

## Validation

Initial commands should become:

```powershell
pwsh -NoProfile -File .\scripts\verify-ufbx-fbx-corpus.ps1
cargo test -p fbx-corpus
cargo fmt --all
```

Before a checkpoint that changes shared public APIs, also run:

```powershell
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Acquisition validation must verify:

- source revision or archive hashes;
- fixture and dependency hashes;
- selection references and coverage counts;
- adjacent license and provenance records;
- no missing external textures or media;
- no implicit network access during ordinary tests.

## Risks And Mitigations

### FBX Defines The Canonical Model

Risk: Tokimu adopts FBX object, connection, transform, material, or animation
concepts as engine truth.

Mitigation: compare every candidate contract against glTF and keep source
artifacts distinct from canonical evidence.

### A Large Corpus Creates False Confidence

Risk: hundreds of files parse while important exporters or semantic features
remain unsupported.

Mitigation: report exporter, version, encoding, stage, and feature matrices
separately from pass counts.

### The Dataset Is Too Large For Git

Risk: a multi-gigabyte corpus burdens clones and history.

Mitigation: commit a reviewed small selection and use pinned acquisition for
optional broad datasets.

### Differential Output Becomes The Authority

Risk: Tokimu copies an independent implementation's behavior or architecture
without understanding the source semantics.

Mitigation: compare normalized observations, retain Tokimu-owned assertions,
and investigate mismatches rather than blindly matching them.

### A Literal Port Imports Another Architecture

Risk: a C implementation is translated type-for-type into Rust and becomes an
accidental public model.

Mitigation: implement the corpus-required semantic slices naturally in Rust,
with source decoding and canonical lowering kept separate.

### Exporter Quirks Become Universal Semantics

Risk: behavior observed in one Blender, Maya, or 3ds Max export becomes an
engine guarantee.

Mitigation: label exporter evidence and require independent pressure before
promoting normalization rules.

### Transform Normalization Hides Source Meaning

Risk: axis, unit, pivot, or inheritance conversions produce plausible renders
while losing inspectable source behavior.

Mitigation: preserve source transforms, record every normalization step, and
validate local and world results structurally.

### Parser Work Expands Without A Consumer

Risk: the corpus becomes a complete FBX product before Tokimu needs the
features.

Mitigation: admit cases by current engine question and stop at explicit
unsupported boundaries.

## Completion Criteria

This plan is complete when:

- a lawful public source and a reproducible acquisition path are pinned;
- corpus, exporter, encoding, version, and feature denominators are recorded;
- a versioned selection explains every admitted case;
- one binary FBX profile decodes with bounded failure semantics;
- source objects, connections, hierarchy, and transforms are inspectable;
- one static mesh reaches Tokimu-owned model and mesh evidence;
- materials, one animation case, one skin case, and one blend-shape case reach
  canonical evidence or explicit unsupported boundaries;
- binary and ASCII syntax can be distinguished from shared semantics;
- differential evidence, if used, remains optional and importer-local;
- importer, model, mesh, animation, and renderer failures are distinguishable;
- coverage reporting makes no Autodesk conformance claim;
- architectural review records what FBX and glTF jointly establish about
  Tokimu's canonical model.

## Graduation Criteria

FBX importer support should be considered for promotion beyond
`examples/lib-example` only when:

- at least one non-example application needs FBX import;
- public semantic contracts have survived both FBX and another importer;
- decoding and source diagnostics are stable;
- implementation-provider details do not leak through public APIs;
- promotion simplifies ownership compared with continued incubation;
- an Architectural Review explicitly recommends admission.

Until then, FBX remains an evidence-producing external format adapter.

## References

- `docs/Libraries/README.md`
- `docs/Conversations/FBX Corpus Tests.md`
- `docs/Libraries/khronos-gltf-corpus-testing.md`
- `docs/Libraries/cgm-corpus-testing.md`
- `docs/testing-strategy.md`
- `docs/roadmap.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/Architectural Reviews/AR-0001-shared-vector-presentation-geometry.md`
- `examples/lib-example/gltf-corpus`
- `ufbx`: `https://github.com/ufbx/ufbx`
