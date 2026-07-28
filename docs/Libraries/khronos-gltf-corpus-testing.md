# Khronos glTF Corpus Testing

## Purpose

Tokimu intends to use a deliberately selected subset of the Khronos glTF
Sample Assets as a model-import and mesh-geometry corpus. The corpus will
exercise the 3D asset pipeline at stable ownership boundaries:

```text
Khronos glTF or GLB fixture
    -> glTF parsing and format lowering
    -> Tokimu-owned model and mesh data
    -> structural validation and diagnostic artifacts
    -> format-agnostic renderer submission
```

This will be an engineering corpus, not a claim of complete glTF conformance.
Its purpose is to expose container, accessor, scene-graph, transform, mesh,
attribute, material, texture, animation, and renderer-boundary problems using
independent, standards-based input.

glTF and GLB are source formats. They must not become Tokimu's canonical model
representation.

## Current Status

As of 2026-07-27, the first source-structural slice is acquired and pinned to
Khronos `glTF-Sample-Assets` revision
`2bac6f8c57bf471df0d2a1e8a8ec023c7801dddf`.

| Measure | Count | Percentage | Meaning |
| --- | ---: | ---: | --- |
| Pinned Khronos sample-asset revision | 1 / 1 | **100%** | The v1 source revision is recorded locally |
| Logical upstream models represented | 6 | Denominator inventory pending | `Triangle`, `Box`, `BoxTextured`, `MeshPrimitiveModes`, `MultipleScenes`, and `SimpleMeshes` are registered |
| Source variants structurally inspected | 6 / 6 selected | **100% of v1 selection** | JSON/external-buffer and GLB framing are validated; MeshPrimitiveModes records every core primitive mode |
| Variants with geometry accessors decoded | 5 / 6 selected | **83% of v1 selection** | Triangle geometry decodes for five cases; MeshPrimitiveModes stops at a verified unsupported-topology boundary rather than silently dropping modes |
| Variants lowered into Tokimu model/mesh data | 0 / 6 | **0%** | Canonical model and mesh lowering remain unimplemented |
| Focused GLB boundary examples | 1 | Not a corpus-coverage metric | `hello-glb` proves source identity and renderer ownership boundaries |

The official importer progress number is therefore:

```text
6 source variants structurally inspected
5 source variants decoded into corpus-owned primitive evidence
1 source variant stops at an explicit unsupported-topology boundary
0 source variants lowered into Tokimu model or mesh data
```

A repository-wide percentage cannot be calculated honestly until the logical
model and source-variant inventories are recorded for the pinned revision.
Once recorded, those coverage denominators must remain tied to that revision.

## Existing Evidence

`examples/hello-glb` is the current executable GLB boundary proof. It:

- records the pinned Khronos `Box.glb` source as Tokimu-owned asset identity;
- decodes its triangle primitive through the format-specific `gltf-corpus`
  helper;
- expands its indexed positions and normals into the renderer's current
  non-indexed Tokimu-owned `Mesh` contract;
- submits that mesh without exposing GLB meaning to the renderer;
- keeps simulation and rendering ownership separate.

It does **not** admit canonical glTF/GLB importer semantics, model resources,
materials, textures, or a general asset pipeline. The renderer still receives
only Tokimu `Mesh` geometry. This example is evidence for the importer boundary,
not an importer-capability admission.

This distinction is intentional:

```text
GLB bytes -> glTF importer -> Tokimu model/mesh -> renderer
```

## Upstream Fixtures

The upstream reference is the current Khronos glTF Sample Assets repository.
The earlier `glTF-Sample-Models` repository is archived. The selected revision
is preserved under:

```text
third-party/fixtures/khronos-gltf-sample-assets/
    provenance.json
    upstream/
    selected/
        selection-v1.toml
        feature-matrix.md
```

`upstream/` contains verbatim selected model subtrees rather than the complete
repository. The selection manifest references files in that tree instead of
duplicating them. This keeps the first proof small while retaining exact source
and model-level license records.

Every selected case should record:

- logical model ID;
- source path and encoding variant;
- pinned upstream revision;
- license and provenance;
- capability under test;
- reason for admission;
- expected stage or explicit unsupported boundary;
- required external buffers, images, and extensions.

## Coverage Accounting

glTF Sample Assets commonly provide more than one encoding of the same logical
model. For example, one model may have separate `.gltf`, embedded, binary
`.glb`, or extension-specific variants. These are useful independent pipeline
cases, but they are not independent model designs.

Report these measures separately:

1. Total logical models in the pinned upstream revision.
2. Unique logical models represented by the selection.
3. Selected source variants.
4. Variants that reached parse, lowering, validation, and render stages.
5. Local synthetic or reduced fixtures.
6. Explicitly unsupported cases.
7. Feature capability status from the feature matrix.

The primary coverage calculation should be:

```text
unique logical upstream models represented
------------------------------------------ x 100
logical models in the pinned revision
```

Variant coverage should be reported independently:

```text
selected source variants exercised
---------------------------------- x 100
source variants in the pinned revision
```

Do not report a procedural cube, several encodings of one Box model, or local
malformed fixtures as several unique upstream models.

## First Selection

The first selection should be small and diagnostic. Candidate model names must
be verified against the pinned upstream revision before admission.

| Capability | Candidate pressure | Intended evidence |
| --- | --- | --- |
| Minimal geometry | Triangle or equivalent | positions, indices, primitive mode, bounds |
| Indexed mesh | Box or equivalent | shared vertices, triangle winding, normals |
| Binary container | GLB variant of the minimal box | GLB chunks, offsets, embedded buffer |
| Texture coordinates | BoxTextured or equivalent | UV attributes and image references |
| Multiple primitives | MeshPrimitiveModes or equivalent | primitive separation and explicit unsupported-topology evidence |
| Shared mesh instances | SimpleMeshes | two nodes referencing one mesh and TRS transforms |
| Node hierarchy | a small hierarchical model | local/world transforms and traversal |
| Materials | a small PBR sample | provider-neutral material lowering |
| Animation | AnimatedCube or equivalent | channels, samplers, time bounds |
| Skinning | SimpleSkin or equivalent | joints, inverse bind matrices, weights |
| Morph targets | a focused morph sample | target deltas and weight semantics |

Initial implementation should stop after geometry, indexing, attributes,
transforms, and explicit diagnostics are stable. Materials, textures,
animation, skinning, and morph targets should enter in separate batches so a
failure has one likely owner.

## Synthetic Mesh Fixtures

Khronos models should be complemented by small Tokimu-owned fixtures that
isolate structural failure classes:

```text
primitives/
    single-triangle
    quad
    cube
    tetrahedron
    sphere

topology/
    open-cube
    duplicate-face
    reversed-face
    non-manifold-edge
    zero-area-triangle

attributes/
    hard-normals
    smooth-normals
    missing-normal
    mirrored-uv
    tangent-seam

transforms/
    negative-scale
    non-uniform-scale
    rotated-parent
    distant-origin
```

Synthetic fixtures do not increase Khronos coverage. They provide focused
regression evidence when an upstream model reveals a problem.

## Canonical Import Result

The importer should lower format data into Tokimu-owned meaning before
rendering:

```text
ImportedModel
    scene hierarchy
    named nodes and transforms
    meshes and primitives
    vertex attribute streams
    index data
    bounds
    material references
    texture resource references
    optional skeletons
    optional animations
    optional morph targets
    metadata and provenance
```

This is an evolving candidate model, not an admitted public contract. The
corpus should discover which fields remain stable across a building, crate,
vehicle, person, animal, and future non-glTF importer.

Physics, clothing, navigation, and attachment semantics are Tokimu capability
questions. They must not be presented as glTF guarantees merely because a
source file contains related metadata or extensions.

## Structural Validation

Every imported primitive should produce bounded diagnostics for at least:

- finite positions, normals, tangents, UVs, colors, and weights;
- indices within the referenced vertex range;
- triangle-compatible index counts where triangle topology is expected;
- finite local and world transforms;
- finite bounds containing every position;
- zero-area triangles;
- approximately unit-length normals where supplied;
- valid joint and weight references where skinning is admitted;
- external resource resolution;
- declared, required, supported, and unsupported extensions.

Topology diagnostics should leave room for:

- boundary edge count;
- non-manifold edge count;
- connected component count;
- duplicate vertices and triangles;
- winding consistency;
- Euler characteristic.

Not every valid model is closed or manifold. Such findings should usually be
reported, not universally rejected.

## Diagnostic Artifacts

For each selected case, preserve stage-specific evidence:

```text
reports/<case-id>/
    input.json
    source-manifest.json
    model.json
    mesh.json
    topology.json
    attributes.json
    transforms.json
    bounds.json
    report.json
    wireframe.png
    normals.png
    uv.png
    depth.png
    final.png
```

Structural artifacts are authoritative for importer and geometry validation.
Saved CPU images, GPU captures, and native-window screenshots are complementary
evidence and must be labeled by source. A visually plausible render does not
prove correct accessors, transforms, topology, normals, or resource ownership.

The first stage whose artifact diverges is the owning diagnostic boundary.

## Validation Commands

The current boundary example can be built with:

```powershell
cargo check -p hello-glb
```

It can be inspected manually with:

```powershell
cargo run -p hello-glb
```

The initial format-specific corpus harness validates glTF JSON, external
buffers, GLB headers, chunk bounds, JSON-first ordering, structural inventory,
and the first geometry accessors:

```powershell
pwsh -NoProfile -File .\scripts\verify-khronos-gltf-corpus.ps1
cargo test -p gltf-corpus
```

The v1 decoder accepts triangle primitives with `FLOAT VEC3` positions,
optional `FLOAT VEC3` normals, optional `FLOAT VEC2` `TEXCOORD_0` streams,
and unsigned scalar indices. It validates buffer-view bounds, byte offsets and
strides, finite values, attribute counts, index ranges, triangle counts, and
computed bounds. `BoxTextured` also verifies that a required external PNG can
be pinned and retained as source evidence. The inspector records PBR
base-color factors and material-to-texture-to-image references, validates those
references, and stops before image, sampler, material, or renderer lowering.
Source-level scenes decode root nodes, child links,
finite local transforms, and deterministic world-transform traversals.
`MeshPrimitiveModes` additionally records all seven core primitive modes at
the source-inspection boundary and proves that the first unsupported mode
(`POINTS`) emits an explicit diagnostic rather than silently disappearing. The
decoder still does not produce Tokimu model, scene, or mesh data. The bounded
`hole_punch1.glb` animation corpus additionally proves a narrow
`EXT_meshopt_compression` importer profile: `ATTRIBUTES` and `TRIANGLES`
views, plus the `EXPONENTIAL` post-filter, reconstruct logical GLB buffers
before ordinary accessor decoding. It also admits finite, strictly ordered
linear translation tracks as decoded importer evidence. Meshopt remains an
external importer mechanism; the renderer and runtime do not depend on it.

## Current Exclusions

The following are currently unimplemented or explicitly deferred:

- canonical `.gltf` and `.glb` importer semantics beyond structural corpus
  inspection;
- sparse accessors and general accessor/component combinations;
- canonical indexed-mesh import beyond corpus-owned decoded evidence;
- tangent, color, joint, and weight streams in the imported path;
- canonical scene and node hierarchy import;
- material, texture, sampler, and image import;
- rotation, scale, weights, cubic interpolation, skeleton, skinning, and
  morph-target import beyond the admitted linear translation-track evidence;
- cameras and lights from glTF;
- compression and mesh extensions beyond the admitted
  `EXT_meshopt_compression` profile, including Draco;
- extension negotiation and required-extension policy;
- reference-viewer image comparison;
- complete glTF conformance.

Unsupported features should stop at a structured diagnostic boundary. They
must not silently disappear while the case is reported as passing.

## How To Expand The Corpus

Add cases in small feature-oriented batches:

1. Pin one upstream revision and record its inventory.
2. Select a minimal logical model and one source variant.
3. Record provenance, capability, reason, dependencies, and expected boundary.
4. Parse and emit source/container diagnostics before lowering.
5. Lower into Tokimu-owned model and mesh artifacts.
6. Validate structural invariants before rendering.
7. Add fixed diagnostic views only after structural output is stable.
8. Register local reduced fixtures for isolated regressions.
9. Update the feature matrix and both coverage calculations.

The highest-return order is:

1. GLB container and glTF JSON structure.
2. Buffers, buffer views, accessors, positions, and indices.
3. Primitive topology, winding, bounds, and normals.
4. Nodes, hierarchy, and transform composition.
5. UVs, textures, and simple material references.
6. Multiple meshes, primitives, and material slots.
7. Animation, skinning, and morph targets.
8. Optional extensions and malformed-input hardening.

## Architectural Boundary

The intended ownership remains:

```text
glTF importer
    owns glTF/GLB syntax, containers, accessors, and extension semantics

Tokimu assets/model capability
    owns stable asset identity, canonical model meaning, resource references,
    diagnostics, and lifecycle contracts

Tokimu mesh/geometry capability
    owns renderer-neutral vertex/index attributes, bounds, and topology

Renderer
    owns GPU execution, uploads, batching, pipelines, and cache lifetime
```

The renderer must never parse glTF, retain importer-native objects, or treat a
GLB file path as renderable truth. Importers must not define physics,
animation, or other Tokimu capability semantics solely according to one source
format.

Repeated pressure from glTF, OBJ, CAD, procedural geometry, and future physics
meshes may justify shared first-party model or mesh contracts. That decision
requires architectural review; the corpus itself does not admit a capability.

## References

- `docs/Libraries/w3c-svg-corpus-testing.md`
- `docs/testing-strategy.md`
- `docs/roadmap.md`
- `docs/Conversations/GLB Model Data.md`
- `docs/Conversations/3D vector reusability.md`
- `docs/Conversations/3d formats.md`
- `examples/hello-glb/DESIGN.md`
- `examples/hello-glb/src/main.rs`
