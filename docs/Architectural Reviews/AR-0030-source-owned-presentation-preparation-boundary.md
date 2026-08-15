# AR-0030: Tokimu Render Preparation And Submission Framework

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-08-14 |
| Last reviewed | 2026-08-14 |
| Scope | Stable Tokimu render API / program preparation / renderer boundary |
| Trigger | Doom synthetic and E1M1 evidence falsified both global static-shell rendering with sky depth patches and whole-source Boolean filtering as sufficient source-faithful presentation models. |
| Related ADRs | ADR-0001, ADR-0003, ADR-0008, ADR-0009, ADR-0013 |
| Related evidence | AR-0023, AR-0025, Doom viewer-relative presentation synthetic conformance campaign, planned Quake and independent non-BSP campaigns |
| Admission exception | None |

## Architectural Question

Which Tokimu-render-specific preparation and submission framework should become
the reliable public boundary between programs that decide what to present and
renderers that realize it? The answer must let a program or source provider
select, transform, fragment, synthesize, reorder where semantically permitted,
or otherwise revise presentation data before handoff without making the
renderer own source topology or simulation truth.

This review must settle a strategy. It may admit a new framework, explicitly
retain and strengthen the existing `Renderer` plus ordered `RenderCommand`
contract, or reject a proposed replacement. Continuing indefinite
provider-local incubation is not a completed disposition.

## Context

Tokimu's current renderer contract accepts explicit meshes, materials, cameras,
and ordered draw commands. Full submission remains the correctness fallback.
AR-0025 found that ordinary conservative candidate filters can reduce work but
did not earn a shared culling capability. Its more aggressive Doom experiments
also showed that source-faithful Doom presentation is not equivalent to
rendering one global static Euclidean shell and asking the depth buffer to
resolve everything.

The Doom sky investigation sharpened the missing seam. Classic Doom processes
admitted wall ranges near-to-far, evolves viewer-relative upper/lower coverage,
retains floor and ceiling intervals from that current coverage, and finally
paints sky into retained sky-plane intervals. Sky is downstream presentation;
it is not a world-space visibility wall.

The prospective flow is therefore closer to:

```text
source semantic records + current runtime state + view
        ↓
source-owned viewer-relative presentation preparation
        ↓
surviving source-labelled contributions or fragments
        ↓
ordinary geometry, material, and ordered draw declarations
        ↓
Tokimu renderer realization
        ↓
rasterization, depth, cutout, blending, and presentation
```

This resembles a pre-render preparation phase, but the current evidence does
not support describing it only as a pre-geometry filter. One source SEG can be
partly excluded and survive as two fragments. A Boolean keep/reject filter over
already-final geometry cannot express that result without either retaining a
forbidden overlap or losing required visible intervals.

The framework under review is not a universal visibility algorithm. It is the
stable Tokimu render API that receives the result of application- or
provider-owned preparation. A Doom consumer may reconstruct source-authorized
fragments; a Quake consumer may use BSP/PVS knowledge; a CAD application may
choose LODs or chunks; and an ordinary GLB caller may submit its scene without
special preparation. All must reach one dependable renderer-facing contract
without importing their domain vocabulary into `tokimu-render`.

## Trigger And Evidence

### Corpus observations

- Full static E1M1 submission exposes source geometry that Doom's ordered
  viewer-relative presentation would not retain.
- A hidden world-space paired-sky depth wall suppressed unrelated geometry but
  also clipped valid nearby hut geometry.
- Exact source-height sky tiles and viewer-relative sky cells improved some
  cases but still let unrelated sector geometry survive through terminal sky
  intervals.
- Source-sector and source-flat authority were too broad; a sector or flat
  contributing one visible sky interval does not authorize all of its retained
  geometry for the current view.
- The partial paired-sky fixture retained a far source SEG in `[112,119]` and
  `[201,208]` while excluding its overlap `[120,200]`. Whole-source keep and
  whole-source reject are both incorrect.
- Horizontal-only and insufficiently recomputed Doom controls produced visible
  false negatives when the camera moved, demonstrating that view and current
  runtime state are causal inputs rather than initialization metadata.

### Automated and synthetic evidence

- SEG-granular lowering preserves linedef/sidedef identity and continuous UV
  correspondence across subdivisions.
- Synthetic fixtures separately cover vertical apertures, paired versus
  one-sky boundaries, shared semantic plane keys with distinct presentation
  instances, projection boundaries, stationary dynamic snapshots, camera
  jitter, and masked-middle non-occlusion.
- Native and Browser WebGPU controls demonstrate that provider-prepared ordinary
  declarations can be realized without teaching `tokimu-render` Doom terms.
- Slice 4B of the synthetic campaign now requires an ordered wall/plane/sky
  transition trace and source-labelled partial-fragment realization before
  another E1M1 candidate may run.

### Reference-source evidence

Inspection of the released Doom wall and plane paths and a faithful modern
continuation explained the observed causal ordering. That source is an oracle
for Doom behavior, not authority for Tokimu's architecture. The historic
clip-array, drawseg, visplane, fixed-point, and framebuffer representations
remain replaceable implementation mechanics.

### Independent consumers and commissioned pressure

Doom is the first demonstrated source-driven consumer. It is necessary but not
sufficient to choose the Tokimu render framework. This review deliberately
commissions additional campaigns rather than waiting for accidental callers:

| Campaign | Required pressure | What it must falsify |
| --- | --- | --- |
| Doom / WAD | ordered viewer-relative source preparation, partial SEG survival, sky/plane relationships, cutout surfaces, and moving sector snapshots | a framework that assumes whole-object Boolean visibility, immutable scenes, or generic sky authority |
| Quake / PAK | fully 3D BSP/PVS or leaf/cluster selection, moving brush models, dynamic entities, and source-specific sky/water behavior | a Doom-shaped 2.5D, screen-column, linedef, or plane-key contract |
| Ordinary retained 3D | the existing textured-presentation `Box.glb` path, or an equivalent caller that needs no source visibility reconstruction | a framework that forces every caller to implement a preparation provider or spatial hierarchy |
| Large technical scene | extend the existing `hello-cad` or retained point-cloud pressure with persistent resources, chunk/LOD selection, overlays, and large candidate counts | a framework that requires full per-frame reconstruction, hides resource churn, or assumes game-scene ownership |
| Multi-view or derived-view falsifier | the existing `hello-3d-stereo` control first; later minimap, mirror, portal, or AR-0026 charted views | a framework that assumes one global view, one Euclidean placement, or one presentation occurrence per source object |

Initial admission requires Doom, Quake, one ordinary non-BSP caller, and one
large or multi-view non-Doom caller. Portal/non-Euclidean evidence may follow as
a deliberate reopening pressure unless it is available before admission.
Quake evidence is meritable for this review, but its package provenance and
corpus intake remain separate work; this AR does not authorize importing
`PAK0.PAK` by itself.

### Missing evidence

- A surviving Doom preparation candidate that passes the new ordered synthetic
  guards and the canonical E1M1 path matrix.
- Executable Quake, ordinary retained-3D, and large or multi-view campaign
  evidence against the same candidate framework.
- A comparison against the current `Renderer::submit(&[RenderCommand])`
  contract rather than assuming a new abstraction is necessary.
- Evidence that a shared interface improves replaceability without laundering
  Doom screen-column concepts into Tokimu vocabulary.
- Performance and bounded-work evidence under ADR-0008 after correctness.
- Failure, recovery, native/browser, and invalid-state evidence under ADR-0009
  for any proposed stable seam.

## Ownership Analysis

The current evidence separates three kinds of meaning:

### Source/domain meaning

The Doom provider owns decoded topology, SEGs, sidedefs, wall tiers, subsectors,
near-first traversal, dynamic sector snapshots, paired-sky rules, and the
decision that a source contribution survives wholly or partially for a view.
These facts must not move into `tokimu-render` or the Native Ring merely because
they precede rendering.

### Potential Tokimu composition meaning

Tokimu needs a dependable render-facing seam even when preparation remains
entirely program-owned. The candidate shared meaning is a bounded presentation
submission: explicit views and ordered work, provider-neutral resource
references, declared material/depth/coverage intent, bounded observations, and
an explicit handoff/lifetime boundary.

Before handoff, the program may freely revise the prospective presentation:
filter candidates, generate or clip geometry, choose LOD, duplicate one source
object for multiple views, or preserve full submission. Those transformations
remain the program/provider's responsibility and must be testable independently
of rendering. After handoff, the renderer validates and realizes the accepted
submission; it does not mutate simulation truth, invoke domain algorithms, or
silently reinterpret why an item was included.

The seam must not prescribe AABBs, BSPs, PVS sets, portals, occluders, screen
columns, linedefs, SEGs, visplanes, brush models, CAD assemblies, or any
particular selection algorithm. It may carry opaque correlation identity and
ordinary generated geometry without assigning domain meaning to either.

Each campaign must inventory four distinct identity roles without designing a
single universal ID prematurely:

```text
domain/source identity
    what semantic or source fact produced this work?

presentation occurrence identity
    which prepared occurrence or fragment is this, in which view?

renderer resource identity
    which persistent mesh, material, texture, or other resource is referenced?

submission identity
    which bounded handoff/version owns this ordered work?
```

One source may produce several occurrences, several views may reference one
persistent resource, and one submission must not make an ephemeral occurrence
look like a persistent renderer resource. The study records those distinctions
before deciding whether any require stable named types.

### Renderer meaning

The renderer continues to own realization of ordinary geometry, textures,
samplers, pipelines, depth behavior, admitted cutout behavior, and backend
presentation. It must not infer source visibility, reconstruct missing source
topology, or reorder provider declarations in ways that violate their admitted
material/order semantics.

Rendering must not mutate simulation or source state. Presentation preparation
observes an explicit current snapshot; it does not become the owner of door,
platform, camera, or world truth.

## Dependency Direction

```text
Current experimental direction:

Doom source/runtime snapshot/view
    -> Doom corpus/provider preparation
    -> ordinary Tokimu renderer declarations
    -> tokimu-render/provider realization

Forbidden direction:

tokimu-render
    -> Doom topology, BSP, SEG, sky, or gameplay semantics

Candidate stable direction:

application/provider owns source + current runtime state
    -> application/provider prepares or revises presentation data
    -> stable Tokimu render submission boundary
    -> renderer validates and realizes the submission
```

No current evidence authorizes `tokimu-core` or `tokimu-runtime` to depend on
renderer, platform, Doom, or provider-native objects.

## Alternatives Considered

### Alternative A: Keep Preparation Entirely Provider/Application Local

- Benefits: preserves current dependency boundaries; introduces no speculative
  framework; lets Doom use exactly the granularity its source semantics need.
- Costs: providers may repeat lifecycle, failure, correlation, or declaration
  assembly behavior before Tokimu can recognize shared meaning.
- Failure mode: repeated independent providers converge on an implicit seam but
  Tokimu leaves it inconsistent and difficult to observe or replace.

### Alternative B: Admit A Minimal Provider-Neutral Preparation Lifecycle

- Benefits: could standardize explicit view/snapshot input, bounded output,
  source correlation, failure containment, and ordinary declaration handoff
  while leaving algorithms and topology provider-owned.
- Costs: a trait or framework may freeze before a second consumer proves the
  common meaning; output may be too renderer-shaped or too source-shaped.
- Failure mode: an implementation convenience becomes a universal engine layer
  whose vocabulary quietly reflects Doom.

### Alternative C: Admit A Generic Pre-Geometry Boolean Filter

- Benefits: simple model; composes naturally with already prepared objects and
  conservative frustum/AABB selection.
- Costs: cannot express partial survival without pre-splitting every possible
  contribution or leaking a fragmentation protocol.
- Failure mode: whole-source keep retains forbidden overlap; whole-source
  reject removes required fragments. Current Doom evidence falsifies this as
  the complete solution.

### Alternative D: Let The Renderer Own Scene Visibility And Fragmentation

- Benefits: could centralize scheduling, GPU culling, resource reuse, and
  provider realization.
- Costs: requires renderer knowledge or a generic ontology for source topology,
  dynamic state, portals, sky authority, and presentation order.
- Failure mode: the renderer becomes a hidden owner of simulation/source truth
  and Doom mechanics leak into a supposedly replaceable backend boundary.

### Alternative E: Provider Emits Final Source-Specific Screen Spans

- Benefits: can faithfully express partial and vertically bounded survival;
  closely follows the demonstrated source invariant.
- Costs: risks pixel-resolution coupling, camera-jitter instability, excessive
  churn, and a source-specific raster protocol masquerading as geometry.
- Failure mode: Tokimu accidentally admits screen columns or historic Doom
  raster mechanics as shared presentation vocabulary.

### Alternative F: Hybrid Source Preparation Followed By Ordinary Conservative Selection

- Benefits: source provider first produces correct view-local declarations;
  generic frustum, bounds, batching, and backend work may then operate only
  where they remain conservative and independently useful.
- Costs: multiple stages require explicit ordering, observations, fallback, and
  performance evidence; duplicated work may erase any benefit.
- Failure mode: a later generic selector invalidates source-prepared fragments
  or reorders declarations with alpha/depth consequences.

### Alternative G: Stable Submission Snapshot With Outer-Ring Preparation

- Shape: `tokimu-render` owns a bounded, renderer-specific submission/frame
  contract. Programs and replaceable preparation providers construct and may
  revise it before handoff. Optional generic helpers remain outside the
  renderer and consume/produce the same ordinary presentation vocabulary.
- Benefits: supplies one reliable rendering API without requiring the renderer
  to call domain code; accommodates pass-through GLB, Doom fragments, Quake
  candidates, large-scene LOD, and multiple views; makes validation, order,
  resource reuse, failure, and frame lifetime explicit.
- Costs: requires a migration and may duplicate some meaning already implicit
  in `RenderCommand`; builders can become an accidental scene graph if their
  ownership and lifetime are not bounded.
- Failure mode: the snapshot grows into a universal world model, or pre-submit
  mutation becomes hidden mutable renderer state.

### Alternative H: Renderer-Invoked Preparation Provider

- Shape: the renderer accepts a provider/callback and asks it to prepare each
  view or frame.
- Benefits: centralizes orchestration and could avoid materializing an
  intermediate submission.
- Costs: renderer execution now invokes application/domain code, complicates
  failure containment, determinism, threading, WASM parity, and lifetime
  ownership.
- Failure mode: `tokimu-render` becomes the scheduler and hidden owner of
  source preparation. This remains a negative control unless campaigns show
  that caller-built submissions are inadequate.

## Findings

1. A source-owned preparation stage before final renderer declarations is a
   credible architectural seam and is already the safest description of the
   Doom experiment.
2. Doom alone cannot choose the stable framework. AR-0030 now requires a
   deliberate multi-campaign comparison, including Quake and non-BSP pressure.
3. “Pre-geometry filter” is too narrow as the general description. The stage
   may select, clip, fragment, or synthesize view-local declarations while
   preserving one source identity and continuous semantic correspondence.
4. The renderer remains downstream. It should continue ordinary rasterization,
   depth, cutout, blending, and presentation without knowing why a declaration
   survived.
5. Full submission remains the fallback until a provider-prepared result passes
   zero-false-negative review. Smaller draw counts do not validate semantics.
6. The stable seam should standardize renderer-facing lifecycle, validation,
   ordering, resource, view, failure, observation, and declaration facts—not a
   universal visibility algorithm.
7. Programs must remain able to transform or replace prospective presentation
   data before handoff. That authority does not imply mutable renderer-owned
   scene state after handoff.
8. Portals and charted views are useful future falsifiers because they may also
   produce view-local or fragmented declarations, but they cannot count as a
   second caller before executable evidence exists.
9. The current ordered `RenderCommand` API deserves a baseline trial. A new
   named framework is not earned if explicit handoff, view, validation,
   identity, ordering, lifetime, and failure semantics can strengthen that API
   without making ordinary callers more complicated.

## Disposition

**Under Review; comparative framework admission required.** Keep Doom
viewer-relative preparation Doom-owned while it tests the ordered
wall/plane/sky invariant. In parallel, subject the current command API and
Alternatives B, F, and G to Doom, Quake, ordinary retained-3D, and large or
multi-view pressure. Do not admit a public preparation trait, generic Boolean
filter, screen-span vocabulary, renderer-owned scene graph, or visibility
algorithm merely to complete the review.

The expected leading strategy is Alternative G: a stable Tokimu-render
submission snapshot built outside the renderer, potentially retaining the
current ordered commands as its work vocabulary. This is a hypothesis, not the
accepted result. The final disposition must name the selected Tokimu render
API, its ownership/lifetime/order/failure guarantees, and the rejected
alternatives. Admission then requires an ADR and corresponding SDD/API work.

Conceptually, the leading hypothesis is:

```text
program/domain preparation (mutable while caller-owned)
    Doom fragments | Quake PVS candidates | GLB pass-through | CAD LOD
        ↓
bounded Tokimu render submission (handoff freezes or versions it)
    views + ordered work + resource references + declared render intent
        ↓
tokimu-render realization
    validation + normal raster/depth/cutout/blend + observations
```

“Preparation” may happen before geometry exists, over existing geometry, or by
generating presentation-only geometry. Therefore the admitted concept should
not be named or constrained as a pre-geometry filter unless the campaign matrix
somehow proves that narrower model sufficient.

## Consequences

- The new synthetic Slice 4B is the immediate evidence gate.
- Existing world-space sky depth and global sky-tile experiments remain useful
  negative controls, not candidate architecture.
- Doom may generate view-local fragments before final geometry submission.
- Source identity, order, UV correspondence, runtime snapshot identity, and
  bounded diagnostic reasons must survive preparation.
- Renderer and platform crates gain no Doom dependency or source-state owner.
- AR-0030 cannot close solely on Doom evidence or a smaller draw count.
- A selected framework requires ADR-0008 and ADR-0009 admission evidence, an
  accepted ADR, SDD alignment, and migration evidence before becoming stable.

## Required Follow-Up

- [ ] Complete Slice 4B in the Doom synthetic conformance campaign.
- [ ] Retain native and Browser WebGPU observations for the surviving bounded
      realization.
- [ ] Run the canonical E1M1 falsification matrix only after the new guards
      pass.
- [ ] Record whether final geometry is generated, clipped, or merely selected
      at the provider boundary and retain its source-correlation costs.
- [ ] Compare the surviving preparation path with full submission for
      correctness, churn, bounded work, and performance.
- [ ] Inventory the current `Renderer`, `RenderCommand`, view, resource,
      material, order, observation, and failure guarantees as the baseline
      strategy; identify which guarantees are merely accidental.
- [ ] Write a bounded Quake campaign plan covering 3D BSP/PVS selection, moving
      brush models, dynamic entities, and source-specific presentation without
      importing assets until provenance/intake is approved.
- [ ] Select an ordinary non-BSP retained-3D fixture and prove that pass-through
      submission remains simple; start with the textured-presentation
      `Box.glb` path.
- [ ] Select a large technical or multi-view fixture and retain resource reuse,
      bounded-work, update, and derived-view evidence; start with `hello-cad`
      or the retained point-cloud pressure and `hello-3d-stereo`.
- [ ] Compare Alternatives A, B, F, and G across the campaign matrix. Retain H
      as a negative control unless caller-built submissions are falsified.
- [ ] Define the candidate handoff precisely: mutability before submission,
      immutability or versioning after submission, view/pass ordering, resource
      lifetime, validation, bounded diagnostics, failure, and fallback.
- [ ] Inventory domain/source, presentation-occurrence, renderer-resource, and
      submission identity separately in every campaign. Reject any candidate
      that silently overloads one identity across those lifetimes.
- [ ] Compare three implementation economics without admitting them first:
      complete immutable submission rebuilt each frame; persistent resources
      plus a rebuilt lightweight ordered work list; and a versioned submission
      with bounded changed ranges.
- [ ] Measure whether submissions can remain cheap ephemeral values while
      renderer resources persist. Do not introduce retained renderer scene
      ownership unless the campaign matrix falsifies that simpler model.
- [ ] Exercise failure on both sides of handoff: preparation failure before a
      submission exists; Tokimu boundary rejection of an invalid submission;
      and renderer/provider failure while realizing a valid submission.
- [ ] Demonstrate native and Browser WebGPU parity for the common contract while
      retaining provider-specific preparation differences.
- [ ] Produce a decision matrix and maintainer disposition. If a stable strategy
      is admitted, draft the binding ADR, update the SDD, and migrate at least
      the four admission campaigns.

## Framework Acceptance Criteria

AR-0030 may close with an admitted framework only when all of these hold:

- Doom and Quake both preserve source-faithful behavior without Doom or Quake
  vocabulary appearing in the stable Tokimu render API.
- An ordinary retained-3D caller can submit without implementing a visibility,
  topology, or preparation-provider abstraction.
- A large technical or multi-view caller can reuse persistent resources and
  update bounded presentation data without rebuilding a renderer-owned world.
- A program can inspect and modify presentation data before handoff; after
  handoff, renderer behavior and mutation/lifetime rules are explicit.
- Domain/source identity, presentation-occurrence identity, renderer-resource
  identity, and submission identity remain distinguishable even when a source
  produces fragments, multiple views reuse a resource, or a frame is rebuilt.
- Multiple views and multiple presentation occurrences of one source object
  are representable without pretending they are one global Euclidean object.
- Ordered opaque, cutout, and incubating blend work preserves the responsibilities
  established by AR-0023; the framework does not silently reorder submissions.
- Invalid resources, stale or unsupported declarations, preparation failure,
  renderer failure, and empty submissions produce bounded retained evidence on
  native and browser targets.
- Stale resource references, duplicate occurrence identities, missing views,
  cross-submission ephemeral references, invalid ordering dependencies, empty
  submissions, and provider failure are rejected or observed at the correct
  boundary without being collapsed into one generic render failure.
- Warm-frame static resources do not churn, allocation/work are bounded, and
  performance evidence satisfies ADR-0008 for every admission campaign.
- Full submission remains an explicit correctness fallback where a program's
  preparation result is incomplete or uncertain.
- The chosen public surface is smaller than a scene graph and does not execute
  application/provider callbacks from renderer realization.
- The evidence explains whether submissions are ephemeral, versioned, or
  incrementally updated and demonstrates why that choice is cheaper and safer
  than the rejected lifecycle models.

The review may instead close by explicitly retaining the existing command API
if it passes this matrix and a new framework supplies no material improvement.
That is a real decision, not a failure to decide.

## Reopening Triggers

This review advances toward an admission decision when:

- the Doom candidate passes all applicable synthetic, E1M1, native, and
  browser correctness gates;
- Quake and the selected non-BSP campaigns produce executable evidence against
  the same candidate render boundary;
- provider-local implementations duplicate lifecycle, failure, correlation, or
  observation semantics that Tokimu can own without owning their algorithms;
- a backend cannot realize correct provider-prepared declarations without a
  stable additional contract; or
- portal/charted-view evidence shows that one source object may appear through
  multiple derived views and materially changes the proposed seam.

The review moves away from shared admission when:

- the correct Doom result requires source-specific raster machinery with no
  reusable lifecycle meaning;
- view-local geometry churn or performance makes declaration generation
  impractical; or
- a simpler provider-local decomposition solves the corpus without repeated
  cross-provider pressure.

## Review History

### Cycle 1 -- 2026-08-14

- Status entering review: Proposed.
- New evidence: world-space sky authority and whole-source Boolean filtering
  were falsified; source inspection identified ordered wall/plane coverage as
  the missing Doom invariant; Slice 4B was added to test it synthetically.
- Participants or reviewers: maintainer and Codex.
- Findings: a source-owned stage before renderer realization is credible, but
  the shared framework and its stable vocabulary remain unearned.
- Disposition: Proposed; continue Doom-provider-local evidence gathering.
- Resulting ADR or documentation change: none.

### Cycle 2 -- 2026-08-14

- Status entering review: Proposed.
- New direction: the maintainer requires AR-0030 to select a reliable
  Tokimu-render-specific API rather than merely observe a possible preparation
  seam.
- Campaign gate: Doom and Quake are mandatory, with ordinary retained-3D and
  large or multi-view pressure required to prevent source-specific admission.
- Ownership clarification: programs/providers may modify presentation data
  before handoff; the stable framework owns renderer-facing intake and
  realization guarantees, not the algorithms that produced the data.
- Disposition: Under Review; comparative framework admission required.

### Cycle 3 -- 2026-08-14

- Reviewer feedback: Alternative G remains the leading hypothesis, preferably
  as a modest formalization of the existing ordered command API rather than an
  ornamental replacement framework.
- Added pressure: every campaign must keep source, occurrence, renderer
  resource, and submission identity distinct.
- Added lifecycle comparison: ephemeral rebuilt submissions, lightweight work
  lists over persistent resources, and versioned changed-range submissions must
  be measured before choosing retained state.
- Added failure taxonomy: preparation failure, submission rejection, and
  provider realization failure must remain separately attributable.
- Disposition: no decision change; retain Under Review status and gather the
  expanded comparative evidence.

## References

- `docs/contribution-admission-guide.md`
- `docs/Tokimu Software Design Document.md`
- `docs/Architectural Reviews/AR-0023-textured-surface-alpha-and-depth-policy.md`
- `docs/Architectural Reviews/AR-0025-camera-candidate-selection-and-visibility-culling.md`
- `docs/Architectural Reviews/AR-0026-non-euclidean-spatial-charts-and-authored-angular-topology.md`
- `docs/Plans/DOOM/Studies/Doom viewer-relative presentation synthetic conformance.md`
- `docs/Plans/DOOM/Evidence/Classic Doom visibility clipping evidence.md`
- `docs/lessions/read-reference-source-early.md`
