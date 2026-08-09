# AR-0022: Textured Mesh Coordinate And Sampling Boundary

| Field | Value |
| --- | --- |
| Status | Accepted |
| Opened | 2026-08-08 |
| Last reviewed | 2026-08-09 |
| Scope | Cross-cutting renderer/material boundary |
| Trigger | At opening, Slice 5B needed to present source-traceable Doom wall and plane textures while the renderer could not consume supplied UV coordinates or a declared sampler/alpha policy. |
| Related ADRs | ADR-0001, ADR-0003, ADR-0008, ADR-0009, ADR-0012 |
| Related evidence | Slice 5 Doom geometry/raster providers, AR-0006, AR-0021, `tokimu-render` mesh/texture/pipeline tests |
| Admission exception | None |

## Architectural Question

What minimal provider-neutral textured-mesh, sampler, and alpha-test contract
must `tokimu-render` expose so that a corpus consumer can present ordinary 3D
geometry with caller-supplied texture coordinates, without admitting Doom WAD,
palette, pegging, flat, sprite, or game-state semantics into the renderer?

## Context

Slice 5 completed a headless Doom geometry seam. It emits ordinary triangle
positions, source identity, texture names, and source-texel coordinates for
walls. It emits source-traceable floor and ceiling triangles separately, while
deliberately deferring plane mapping. `doom-raster-provider` can compose a Doom
texture or decode a flat, select a palette explicitly, and lower covered pixels
to provider-neutral RGBA8. Coverage is retained as straight alpha.

The current renderer already owns generic RGBA8 texture allocation and a
material texture handle, but its mesh contract contains only positions and
normals. `GpuVertex` mirrors those two streams. `PipelineKind::Texture2d`
derives UV from `position.xy`, and its default transform is 2D painter-order
rather than a 3D surface contract. The WGPU backend uses a point-filtered,
clamp-to-edge sampler for every source texture.

That is adequate for the present 2D proof but cannot truthfully consume Doom's
existing wall texel coordinates or a plane mapping. A corpus-local shader that
interprets Doom coordinates behind a generic `Mesh` would hide the missing
contract; changing the renderer to understand Doom names or pegging would be
an ownership violation.

## Trigger And Evidence

- Corpus examples:
  - `doom-geometry-provider` retains every admitted wall triangle's source
    texel coordinates and supports source-traceable floors and ceilings.
  - `doom-raster-provider` lowers a selected palette and coverage into RGBA8
    without choosing renderer color space, sampler, lighting, or alpha policy.
  - Slice 5B requires static E1M1 walls, floors, ceilings, sky classification,
    and material requirements through an ordinary renderer boundary.
- Automated tests:
  - Current renderer tests prove RGBA8 payload validation, point filtering,
    clamp addressing, and material texture binding.
  - Current Doom geometry/raster tests prove texel-coordinate retention and
    indexed coverage lowering.
- Audits or diagnostics:
  - `PipelineKind::Texture2d` derives its UV coordinates from 2D position.
  - The WGPU vertex buffer has no texel-coordinate attribute.
  - AR-0021's native/WASM fixture proves front/back classification and
    reflection handling; it does not prove texture coordinates or alpha test.
- Repeated implementation friction:
  - Both arbitrary mesh texture placement and Doom's documented wall mapping
    need caller-supplied UVs. The missing field is not Doom-specific.
- Missing evidence at opening: no native/WASM textured 3D fixture exercised
  supplied UV, repeat versus clamp addressing, point versus linear filtering,
  or cutout alpha behavior.
- Current remaining evidence: native execution visibly distinguishes supplied
  UVs and clamp/repeat; browser/WASM has a presented first frame and prepared
  equivalent controls awaiting retained interactive comparison. Cutout alpha
  remains unsupported, and plane mapping remains undecided; original Doom
  view-dependent spans and a conventional 3D plane mapping are not equivalent
  claims.

## Ownership Analysis

The renderer should own only the provider-neutral execution vocabulary:

- optional per-vertex texture coordinates aligned with mesh positions;
- a declared source-texture sampling policy;
- explicit opaque, blended, and cutout/candidate-alpha behavior in pipeline or
  material policy; and
- backend realization of those choices.

The Doom corpus owns WAD bytes, patch composition, palette selection,
`COLORMAP`, texture/flat names, pegging, source texel axes, sky classification,
and the eventual Doom-specific plane mapping decision. A corpus application
owns which generic material/pipeline declaration it chooses from those inputs.

This is a foundational rendering boundary, not Ring 0 simulation truth. It
must not own map sectors, source asset names, mutable game state, software
renderer behavior, or a full universal material graph.

## Dependency Direction

```text
Current:

Doom WAD -> Doom raster/geometry providers -> source texel requirements
                                                X
Tokimu Mesh positions + normals -> renderer-derived 2D UV -> WGPU sampler

Proposed evidence direction:

Doom WAD -> Doom raster/geometry providers -> ordinary RGBA8 pixels,
                                             positions, normals, UVs,
                                             generic material declaration
                                                        |
                                                        v
tokimu-render textured mesh/material contract -> WGPU/native and WASM backend
```

No Doom type or WAD byte crosses into `tokimu-render`. The generic contract
must not depend on corpus crates.

## Alternatives Considered

### Alternative A: Preserve Derived 2D UVs

- Benefits: no public mesh change.
- Costs: cannot represent arbitrary 3D texture placement or Doom's retained
  texel axes; planes and walls would be visually misleading.
- Failure mode: a known 2D convenience behavior is mistaken for a general
  textured-mesh contract.

### Alternative B: Add A Doom-Only Presentation Adapter Or Shader

- Benefits: could produce an early E1M1 screenshot quickly.
- Costs: duplicates renderer behavior and creates a second texture contract in
  the corpus.
- Failure mode: Doom-specific source semantics leak toward the renderer or a
  corpus workaround becomes the de facto generic API.

### Alternative C: Incubate A Narrow Generic Textured-3D Contract

- Benefits: supplied UVs, sampler choice, and alpha behavior are expressed
  once as provider-neutral rendering meaning; Doom becomes a real caller.
- Costs: requires native/WASM tests, migration of existing mesh callers, and a
  clear boundary between source alpha facts and renderer discard/blend policy.
- Failure mode: the contract expands prematurely into a material graph or
  bakes in Doom plane behavior.

### Alternative D: Keep Slice 5B Deferred

- Benefits: preserves current boundaries without new API work.
- Costs: no truthful textured static E1M1 presentation; Slice 5 evidence stays
  headless.
- Failure mode: indefinite deferment hides that the current `Texture2d` name
  overstates the supported use case.

## Findings

1. The input evidence is already decomposed correctly: Doom providers expose
   ordinary pixels and geometry requirements, not renderer objects.
2. At opening, the generic renderer seam was insufficient: it lacked supplied
   UVs and generic sampler selection. The incubating implementation now fills
   that narrowly scoped gap; its final admission remains this review's decision.
3. Point filtering is a useful candidate for the first Doom presentation, but
   clamp-to-edge conflicts with repeated wall/flat texel coordinates. Selecting
   repeat must be a generic sampler policy, not a Doom exception.
4. Doom indexed coverage is an alpha fact. Whether a material blends, discards
   below a threshold, or remains unsupported is a separate renderer policy.
5. AR-0021 permits explicit back-face culling for the corrected Doom winding
   under its provisional contract and native/WASM fixture. It does not settle
   UV, sampler, alpha, or plane behavior.

## Disposition

Accepted. ADR-0012 admits the narrow generic supplied-UV and sampler contract.
Slice 5B may use only that demonstrated generic vocabulary; it must not infer a
Doom alpha, texture color-space, or plane-mapping policy. AR-0023 owns alpha
and depth policy, while Doom-specific material and plane policies remain
separate work.

## Candidate Incubating Contract

Slice 0 exposes a compact candidate that reuses the current mesh, semantic
shader validation, material, and backend seams instead of creating a second
textured-mesh API. It is a proposal for corpus implementation, not an accepted
stable public contract.

### Mesh and shader input

`Mesh` gains an optional `texture_coordinates: Vec<[f32; 2]>` stream. An empty
stream means the mesh supplies no UVs; a present stream must have exactly one
entry for every position. Construction and validation must reject a mismatched
stream before GPU allocation. Existing untextured mesh constructors retain an
empty stream.

`ShaderVertexSemantic` gains `TextureCoordinate2`. Existing
`ShaderModuleDefinition::validate_mesh` then supplies the rejection boundary:
a shader that declares UV at location 2 cannot accept a mesh without a valid
UV stream. This generalizes the renderer's existing position/normal validation
instead of hiding missing coordinates in a corpus shader.

### Pipeline

Add one built-in `PipelineKind::Textured3d`:

- vertex locations: position 0, normal 1, texture coordinate 2;
- uses the existing material color, sampled texture, sampler, instance, and
  camera binding schema;
- transforms 3D position through the camera and passes caller UV unchanged;
- has `depth_writing_3d()` as its default render state; and
- has no GLB, PNG, Doom, palette, source-coordinate, or plane behavior.

The existing `Texture2d` pipeline retains its derived 2D coordinates and is
not silently reinterpreted as `Textured3d`.

### Sampling

Add a bounded provider-neutral `TextureSampler` declaration to `Material`:

```text
filter: point | linear
address_u: clamp | repeat
address_v: clamp | repeat
```

Its default remains the current point/clamp behavior. The backend maps only
these declared values to its sampler descriptor. It does not expose `wgpu`
objects, admit mip policy, or let source image metadata choose sampling.

### Alpha

The first `Textured3d` profile admits only the existing pipeline blend choice
(`opaque`, `alphaBlend`, or `additive`). It does not admit alpha test/cutout,
discard threshold, premultiplied-alpha conversion, or source-format-derived
behavior. The initial selected PNGs have no alpha and will use an opaque
pipeline. A later cutout decision requires a separate source fixture and an
AR-0022 update.

### Rejection and migration behavior

- a UV length mismatch is an explicit mesh-validation error;
- a `Textured3d` shader/mesh compatibility check rejects absent UVs;
- existing untextured draws and `Texture2d` retain their behavior;
- material updates preserve the established default point/clamp sampler unless
  a caller explicitly requests another declared policy.

This candidate is bounded: no indices, tangents, multiple UV sets, per-texture
samplers, material graph, mip policy, or source-coordinate transform is
admitted by this work.

## Consequences

- Any eventual `Mesh`/pipeline API change crosses a general renderer boundary
  and must satisfy the applicable ADR-0008/ADR-0009 gate, including native and
  WASM evidence.
- Existing 2D callers need an explicit migration/default story; no caller may
  silently inherit arbitrary supplied UV semantics.
- The first admitted Doom material must state palette, texture color-space
  interpretation, sampling/address mode, and alpha behavior explicitly.
- Doom plane mapping stays a separate Slice 5B decision after generic UV
  capability exists.

## Required Follow-Up

- [ ] Propose the smallest generic mesh UV representation and validate stream
      length, absent-UV behavior, and 2D compatibility.
- [ ] Propose a bounded provider-neutral sampler policy with point filtering
      and clamp/repeat modes; do not expose WGPU objects.
- [ ] Decide whether cutout alpha is a narrow generic pipeline policy or stays
      explicitly unsupported in the first static E1M1 submission.
- [ ] Add a native/WASM textured-3D fixture covering supplied UV, back-face
      culling, point sampling, clamp/repeat, and an alpha case.
- [ ] Select a Doom palette/color-space/material mapping only after the generic
      fixture has evidence.
- [ ] Select original view-dependent plane spans or explicitly non-equivalent
      plane mapping as a separate Doom decision.
- [ ] Revise Slice 5B checklist statuses and create an ADR only if the generic
      contract earns stable public admission.

## Reopening Triggers

- a second non-Doom consumer needs supplied mesh UVs or sampler policy;
- a native/WASM backend disagrees on the conformance fixture;
- the smallest contract cannot represent existing 2D texture callers without a
  silent behavior change;
- a proposed material/alpha policy requires Doom source concepts in the
  renderer; or
- original Doom plane spans require semantics beyond the generic contract.

## Review History

### Cycle 2 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: `hello-textured-box` Slice 0 fixture and boundary inventory.
- Findings: the selected Khronos `Box.glb` is a small 24-position/24-normal,
  36-index primitive with no `TEXCOORD_0`; its indexed expansion demonstrates
  a real caller for generic UV input without importing a glTF material. Three
  independently selected first-party PNGs provide grid, labelled-orientation,
  and palette-variation pressure. The current 78-file PNG set contains no
  `tRNS` transparency chunk, so it cannot honestly exercise alpha policy.
- Disposition: retain Proposed. The evidence strengthens the case for a narrow
  generic UV/sampler seam, but does not itself admit a public API or select an
  alpha policy.
- Resulting documentation: added
  `docs/Plans/textured-box-glb-png-corpus.md` and
  `corpus/hello-textured-box/{DESIGN.md,fixture-manifest.md}`.

### Cycle 3 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: an incubating generic `tokimu-render` implementation of the
  candidate contract: optional validated mesh UV stream, `TextureCoordinate2`
  shader semantic, `Textured3d` built-in pipeline, and a bounded material
  sampler declaration.
- Findings: existing `Texture2d` behavior remains separate. Mesh construction
  rejects a non-empty UV stream with the wrong length; semantic draw-contract
  validation rejects `Textured3d` without UVs before backend submission;
  default sampler behavior stays point/clamp; linear/repeat maps only from the
  declared provider-neutral material policy. The initial scope keeps cutout
  alpha unsupported.
- Validation: `cargo fmt --all`; `cargo test -p tokimu-render` (56 tests
  passed). The workspace emits existing `glam` `unused_attributes` warnings;
  this change adds no warning.
- Disposition: retain Proposed. This is unit and backend-mapping evidence, not
  yet a native or browser textured-3D presentation proof.

### Cycle 4 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: `corpus/hello-textured-box` native corpus entry. It decodes
  the pinned Box, expands its 36 indexed triangle vertices, supplies
  corpus-owned planar UVs, decodes the independent first-party grid PNG into
  normalized RGBA8, and selects `Textured3d` with the explicit default
  point/clamp sampler.
- Validation: `cargo test -p hello-textured-box` (one structural conversion
  test passed); `cargo fmt --all` and `git diff --check` passed. Existing
  third-party `glam` warnings remain external to this change.
- Disposition: retain Proposed. This establishes a runnable native composition
  path, but no visual capture, sampler-mode comparison, alpha source, browser
  execution, or DOOM handoff evidence has yet been retained.

### Cycle 5 -- 2026-08-09

- New correction: the first sampler comparison used only unit-interval UVs,
  which makes clamp and repeat observationally equivalent. The corpus now
  deliberately scales its planar UVs to `3.25`; clamp edge behavior and repeat
  tiling can therefore be compared honestly. Point-versus-linear remains a
  separate sampling observation, not an assumed visible difference.

### Cycle 6 -- 2026-08-09

- New evidence: project maintainer manually observed the native textured Box.
  Supplied UVs visibly sample the independent PNG, and deliberately
  out-of-range coordinates distinguish clamp edge behavior from repeat tiling.
  The observation and controls are retained in
  `corpus/hello-textured-box/results/native-manual-observation.md`.
- Alpha disposition: `Textured3d` uses the existing explicit pipeline blend
  policy. Its first corpus profile is opaque; source alpha/cutout remains
  unsupported because the selected PNG fixtures contain no transparency.

### Cycle 7 -- 2026-08-09

- New evidence: `hello-textured-box-web` is a separate browser/WASM consumer.
  It embeds the same pinned Box bytes and first-party grid PNG as the native
  corpus entry, performs the same format-boundary conversions, supplies
  corpus-owned out-of-range planar UVs, and uses the generic `Textured3d`
  contract. Its HTML harness uses the AR-0021 browser adapter/device preflight,
  an asynchronous Tokimu renderer construction path, and explicit ready,
  unsupported, and failure states.
- Validation: `cargo test -p hello-textured-box-web`; `cargo check -p
  hello-textured-box-web --target wasm32-unknown-unknown`; and `wasm-bindgen
  target/wasm32-unknown-unknown/debug/hello-textured-box-web.wasm --out-dir
  corpus/hello-textured-box-web/web/pkg --target web` completed successfully.
- Limitation: no browser was connected to this implementation session. These
  are build/package facts only, not adapter, surface, first-present, sampling,
  or native/browser equivalence evidence.
- Disposition: retain Proposed and leave Slice 4 browser presentation evidence
  open.

### Cycle 8 -- 2026-08-09

- New evidence: the project maintainer observed a first browser/WASM presented
  frame from `hello-textured-box-web`. The result is retained in
  `corpus/hello-textured-box-web/results/browser-manual-observation.md`.
- Findings: the browser consumer composes the scoped GLB geometry decode,
  corpus-owned UVs, PNG normalization, texture upload, material binding, and
  browser surface presentation. It also now prepares native-equivalent `M`,
  `R`, and `X` controls without moving browser-input ownership into the
  renderer.
- Remaining evidence: the first browser observation does not yet establish a
  browser-side U/V transformation or sampler-mode comparison. Those visual
  controls remain deliberately open rather than inferred from their code path.
- Disposition: retain Proposed. The generic seam has native and browser first
  presentation evidence; browser interactive conformance, alpha, negative
  corpus cases, and DOOM handoff remain open.

### Cycle 9 -- 2026-08-09

- Negative-boundary review: malformed UV counts fail in `Mesh` construction;
  absent UVs fail in `Textured3d` draw-contract validation; invalid dimensions,
  malformed RGBA8 payloads, and duplicate source texture handles fail before
  WGPU allocation. The sampler vocabulary is closed and provider-neutral, and
  the initial corpus profile continues to reject source-alpha/cutout admission
  by scope rather than silently choosing an alpha-test threshold.
- Browser failure posture: the fixture surfaces no-WebGPU, adapter/device
  preflight timeout, and Rust runtime errors as distinct visible states. The
  browser first-frame result does not erase those diagnostics.
- Disposition: retain Proposed. Negative renderer and browser-harness paths
  are explicit; a dedicated negative visual corpus capture is not yet needed
  unless a real caller exposes a missing diagnostic.

### Cycle 10 -- 2026-08-09

- Alpha implementation audit: the textured fragment shader preserves sampled
  straight alpha by multiplying `textureSample(...)` by the material color;
  the backend maps the declared `AlphaBlend` policy to conventional WGPU
  source-alpha blending. The source data path therefore does not discard or
  silently reinterpret alpha.
- Blocking finding: `Textured3d` currently takes `depth_writing_3d()` as its
  default state, which combines `AlphaBlend` with depth writes. That remains
  harmless for the initial opaque corpus profile, but it is not sufficient
  evidence for a general blended transparent-surface contract: draw ordering,
  depth-write behavior, and any cutout threshold are still unspecified.
- Consequence: do not create a decorative alpha PNG merely to claim coverage.
  A later alpha slice must first choose either a bounded opaque/cutout policy,
  or a transparent-material ordering/depth policy, then exercise it with a
  source fixture. Neither result may be inferred from PNG alpha alone.
- Follow-up: AR-0023 now owns this separate alpha/depth policy question.
- Disposition: retain Proposed. The generic UV/sampler path is ready for final
  review; alpha is an intentionally separate unresolved admission question.

### Cycle 11 -- 2026-08-09

- Review recommendation: retain the narrow generic UV and sampler contract as
  the candidate outcome of this AR: optional checked per-vertex UVs, a
  `Textured3d` pipeline that requires them, and declared point/linear plus
  clamp/repeat material sampling. The evidence does not support a material
  graph, GLB-material import, PNG-aware renderer API, Doom coordinate concept,
  or alpha-test feature.
- Promotion condition: retain one browser-side interactive observation showing
  a UV transformation and a sampler/addressing change after the first rendered
  frame. This is a deliberately small confirmation that the same contract
  composes in both consumers; it is not a cross-platform pixel-equality test.
- Separate follow-up: treat transparency/cutout as a new bounded review once a
  real caller can state its required depth, ordering, and threshold semantics.
  It must not block a final decision on supplied UV and sampler vocabulary.
- Decision now requested from maintainers: accept the UV/sampler portion as a
  minimal generic renderer contract, retain it incubating for another caller,
  or revise it. Alpha remains outside this choice.

### Cycle 12 -- 2026-08-09

- New evidence: after the browser interaction controls were added, the project
  maintainer confirmed that the fixture worked correctly. The retained browser
  record now covers its `M`, `R`, and `X` control-driven texture, sampler, and
  UV state changes after a presented frame.
- Finding: native and browser consumers now both exercise the same declared
  supplied-UV and sampler vocabulary. This remains composition evidence, not
  a pixel-equivalence or universal visible-filtering claim.
- Disposition: retain Proposed pending the Cycle 11 maintainer architecture
  decision. The textured-Box evidence-collection plan is complete; AR-0023
  owns the separate alpha/depth policy question.

### Cycle 13 -- 2026-08-09

- New evidence: project maintainer and independent architectural reviewer
  agree with the Cycle 11 narrow-contract recommendation.
- Findings: the Box+PNG corpus demonstrates an independently useful generic
  seam outside Doom. The retained contract admits only checked supplied UVs,
  `Textured3d` UV consumption, and declared point/linear plus clamp/repeat
  sampling; legacy position-derived `Texture2d` remains separate.
- Disposition: Accepted. ADR-0012 binds this UV/sampler contract. AR-0023
  remains the separate proposed review for alpha/depth policy.
- Resulting ADR or documentation change: ADR-0012; SDD and DOOM Slice 5B
  handoff updated.

## Closure Conditions

The browser interactive-observation condition was completed in Cycle 12. The
review can close its UV/sampler portion once a maintainer selects the Cycle 11
disposition. A later alpha/cutout decision is intentionally not a closure
condition for this narrower question; AR-0023 owns it when a caller supplies
the required ordering/depth or threshold semantics.

### Cycle 1 -- 2026-08-08

- Status entering review: Proposed.
- New evidence: Slice 5B capability audit of the Doom geometry/raster outputs
  and current `tokimu-render` mesh, texture, sampler, and texture shader seam.
- Participants or reviewers: project maintainer and Codex implementation
  review.
- Findings: supplied 3D texture coordinates and sampling policy are missing
  from the renderer boundary; a Doom-local workaround would hide the gap.
- Disposition: Proposed.
- Resulting ADR or documentation change: created this AR and retained the
  Slice 5B blocker; no generic API was admitted.

## References

- `docs/ADR/ADR-0001-engine-boundaries.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0008-native-kernel-ring-performance-and-code-quality.md`
- `docs/ADR/ADR-0009-ring-based-verification-failure-containment-and-recovery.md`
- `docs/Architectural Reviews/AR-0006-raster-image-requirement-pipeline.md`
- `docs/Architectural Reviews/AR-0021-geometry-orientation-and-facing-conformance.md`
- `docs/Architectural Reviews/AR-0023-textured-surface-alpha-and-depth-policy.md`
- `docs/Plans/DOOM/DOOM WAD Checklist.md`
- `corpus/lib/doom-geometry-provider/src/lib.rs`
- `corpus/lib/doom-raster-provider/src/lib.rs`
- `crates/tokimu-render/src/mesh.rs`
- `crates/tokimu-render/src/texture.rs`
- `crates/tokimu-render/src/pipeline.rs`
- `docs/Plans/textured-box-glb-png-corpus.md`
- `corpus/hello-textured-box/DESIGN.md`
- `corpus/hello-textured-box/fixture-manifest.md`
