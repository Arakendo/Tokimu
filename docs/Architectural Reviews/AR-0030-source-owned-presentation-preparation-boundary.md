# AR-0030: Tokimu Render Preparation And Submission Framework

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-08-14 |
| Last reviewed | 2026-08-16 |
| Scope | Stable Tokimu render API / program preparation / renderer boundary |
| Trigger | Doom synthetic and E1M1 evidence falsified both global static-shell rendering with sky depth patches and whole-source Boolean filtering as sufficient source-faithful presentation models. |
| Related ADRs | ADR-0001, ADR-0003, ADR-0008, ADR-0009, ADR-0013 |
| Related evidence | AR-0023, AR-0025, Doom viewer-relative presentation synthetic conformance campaign, completed Doom source-topology admission study, successor Doom ordered source-occurrence preparation study, planned Quake and independent non-BSP campaigns |
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
- The first live-observer `prepared-full-submission` realization reran the
  ordered Doom preparation as position and heading changed, then submitted all
  retained declarations without a generic post-filter. It still produced
  cracks where floors and ceilings meet walls, made some walls disappear at
  close range, and exposed the finite prepared view rectangle with sky outside
  it during free look. These are source-preparation/realization failures, not
  renderer full-submission or AABB failures.
- Conservation of retained contributions is therefore necessary but not
  sufficient. Reconstructing fixed-column wall and plane observations as
  approximate world geometry can preserve identities and counts while losing
  continuous boundary coverage and near-view behavior.
- Whole-contribution topology admission is now independently falsified across
  three poses with unchanged source geometry. Retaining the far contribution
  exposes 81–97 source-invalid overlap columns; rejecting it loses 9–15 valid
  side columns; paired-sky supplies no ordinary depth authority that could
  repair the overlap downstream.
- The required Doom operation is therefore not merely `item -> keep/reject`.
  It may map one source contribution to zero, one, or multiple view-local
  presentation occurrences. This strengthens “preparation” over “prefilter”
  without admitting any public occurrence or fragment vocabulary.
- The failed realization exposes three distinct domains that the eventual
  handoff must not collapse: Doom source space, the Doom-owned prepared-view
  domain, and Tokimu presentation world/view space. The failed path discretized
  a prepared view, approximately inverted it into world geometry, and then
  projected it again. That round trip destroyed the coverage continuity the
  preparation had established.

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
- The subsequent complete-geometry admission study independently tested a
  conservative whole-contribution candidate over unchanged source geometry.
  It admitted the far SEG whole at baseline, bounded jitter, and a nearer pose.
  The source-invalid central overlap covered 81, 81, and 97 columns while 15,
  15, and 9 side columns respectively still had to survive. Paired-sky supplied
  no ordinary wall/depth authority in those overlaps, so downstream GPU depth
  could not repair the whole-source result. Rejecting the SEG whole would lose
  the required side intervals. This strengthens the falsification of
  Alternative C and earns investigation of source-owned view-local
  presentation occurrences or fragments; it does not admit screen columns as
  Tokimu vocabulary. Retained fingerprint:
  `e79bb365ef3c1d8bb77dcce721cef1d5a08c1394a1370ffe4a6d35aef8ba94db`.
- The authoritative-sky delta study then reduced 66 retained sky intervals and
  2,046 diagnostic oracle cells to two continuous view-local declarations (12
  vertices, four triangles) with stable material identity, no diagnostic-grid
  identity and no persistent mesh identity. Paired-only, one-sky, ordinary
  aperture, empty, invalid-depth and near-plane controls all fail open.
- The Candidate 1 handoff audit found a concrete renderer-lifetime gap.
  `DrawMeshCommand` can reference geometry only through `MeshHandle`, while
  `WgpuBackend::upload_mesh` stores that geometry as a persistent resource and
  records same-handle uploads as replacement. Camera-dependent declarations
  therefore cannot use the existing seam without misreporting ephemeral
  prepared-view work as persistent resource replacement.
- This result does not admit inline geometry, a transient-mesh command or a
  compositing primitive. It records provider-neutral pressure to distinguish
  bounded ephemeral/view-local work from persistent material, texture,
  pipeline and mesh resources at the eventual handoff.

### Reference-source evidence

Inspection of the released Doom wall and plane paths and a faithful modern
continuation explained the observed causal ordering. That source is an oracle
for Doom behavior, not authority for Tokimu's architecture. The historic
clip-array, drawseg, visplane, fixed-point, and framebuffer representations
remain replaceable implementation mechanics.

The pinned end-to-end trace is retained in
[Classic Doom Renderer Dataflow And Tokimu Preparation Seam](../Plans/DOOM/Evidence/Classic%20Doom%20renderer%20dataflow%20and%20Tokimu%20preparation%20seam.md).
It locates the useful provider boundary around the coupled effects of
`R_StoreWallRange` and `R_RenderSegLoop`: one ordered wall occurrence can emit
wall tiers, mutate the current upper/lower visible window, mark one or more
plane instances, and defer masked work. This falsifies a Boolean whole-mesh
preselection seam, but supports a Doom-private ordered coverage planner whose
complete output is lowered into ordinary Tokimu declarations before renderer
submission.

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
- A Doom realization that can preserve partial wall, floor, ceiling, and sky
  coverage under moving and near-boundary views without exposing a finite
  screen-column observation as a world-space view box.
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

The authoritative-sky gate creates a narrower implementation comparison inside
Alternative G. These are private candidates, not admitted API shapes:

| Candidate | Experimental shape | Principal question |
| --- | --- | --- |
| G1 — inline submission geometry | an ordered draw directly carries or borrows bounded vertices/indices for the submission | can the smallest mechanism remain bounded without bloating commands or obscuring ownership? |
| G2 — submission-local geometry arena | the bounded submission owns geometry payloads and ordered draws reference submission-local identities | can prepared-view work have an explicit lifetime and identity without becoming a persistent asset or renderer-owned scene? |
| G3 — frame-local transient pool | the renderer stages temporary geometry for a frame and exposes no persistent `MeshHandle` | is “frame” sufficiently precise across multiple views, retries, browser presentation, offscreen work and retained observations? |
| G4 — persistent mesh replacement control | existing upload/replacement machinery realizes each changed view-local payload | how much churn and semantic ambiguity result when ephemeral work is represented as asset mutation? |

G2 is the leading hypothesis because the demonstrated lifetime is “this
prepared occurrence belongs to this bounded submission/view,” not merely “one
frame.” It keeps three identities distinct:

```text
persistent renderer resource identity
    survives submissions

submission-local presentation identity
    expires with the bounded handoff/version

domain/source correlation identity
    survives independently for evidence and diagnostics
```

The comparison must not begin by publishing `TransientMesh`, nor may it infer
that all frame-local work shares one lifetime. It should first test private
realizations of the 12-vertex/four-triangle Doom declaration and then apply
independent caller pressure.

A retained precedent survey supports the distinction without selecting an API
by imitation. bgfx demonstrates bounded per-frame transient allocation; wgpu
demonstrates reusable staging with explicit finish/submit/recall; Vulkan makes
safe reuse depend on GPU completion rather than a CPU frame label; and Bevy
demonstrates that extraction/preparation can remain separate from final render
execution. Together these suggest that G3 is a plausible provider-internal
realization of G2, while G2 remains the more precise semantic lifetime. They do
not admit a render graph, render world, frame arena, or `TransientMesh` API.
See
`docs/Plans/Renderer-Reliability/Evidence/AR-0030 transient geometry precedent survey.md`.

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
10. A stable renderer handoff must not assume that every provider-prepared
    presentation can be reconstructed as persistent global world-space
    geometry. View-local contributions are a first-class falsification case,
    even if their eventual renderer realization still uses ordinary triangles.
11. Doom's first E1M1 ordered-occurrence prepared-full candidate balanced every
    destination, source-triangle, fragment, declaration, and renderer-handoff
    count, yet visibly removed required walls, floors, ceilings, and junction
    regions at the canonical source-spawn pose. Exact plane destinations plus
    merged camera-horizontal SEG occurrence domains are therefore insufficient
    authority for complete Doom wall/plane survival. This strengthens the need
    for a richer Doom-private ordered coverage representation while leaving
    the final renderer declarations and their stable API under review.
12. Pinned inspection of released Linux Doom and Chocolate Doom identifies the
    missing authority as one coupled ordered protocol: horizontal solid/pass
    admission, per-column upper/lower window mutation, wall-tier emission,
    plane marking and same-key instance splitting, sky painting, and deferred
    masked work. This gives Doom a viable private frame-planning seam, but
    falsifies the assumption that independently selected complete walls and
    planes can reproduce the source presentation.

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

- [x] Complete Slice 4B in the Doom synthetic conformance campaign.
- [x] Retain native and Browser WebGPU observations for its bounded synthetic
      realization.
- [x] Trace the retained partial-survival falsifier through Classic Doom and
      one faithful port before implementing another E1M1 visual candidate.
      The pinned released-source and Chocolate Doom trace identifies coupled
      horizontal admission, vertical-window mutation, wall-tier emission,
      plane marking/instance splitting, sky paint, and deferred masked work as
      one Doom-owned preparation protocol.
- [ ] Establish and validate the Doom-private `0..N` source-relative
      occurrence representation in the successor study.
- [ ] Run the canonical E1M1 falsification matrix only after the successor's
      source trace, headless, shared-boundary, native, and runtime-snapshot
      guards pass.
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
- [ ] Compare the G1 inline, G2 submission-local, G3 frame-local and G4
      persistent-replacement realization candidates privately. Retain payload
      ownership, allocation/copy/upload cost, view/submission correlation,
      validation failure, cleanup and native/browser evidence.
- [x] Test whether G2 can keep persistent sky material/texture/pipeline
      resources separate from the 12-vertex/four-triangle authoritative-sky
      payload while preserving bounded source correlation and zero persistent
      mesh replacements. The private headless snapshot retains one durable
      material key, two local payload identities, source/view/runtime
      correlation, and zero persistent mesh identities. Native Tokimu/WGPU now
      realizes that snapshot through the restricted experimental seam; actual
      Browser WebGPU now preserves the same bounded identity, recovery and
      persistent-resource observations.
- [x] Authorize the shape of the private G2 GPU handoff experiment before it
      crosses the renderer boundary. Maintainer authorization permits one
      corpus-only feature-gated intake for immutable submission-local geometry:
      no stable public vocabulary, Doom semantics, persistent mesh identity or
      renderer-owned scene state; bounded validation/failure observations are
      required. A direct WGPU fixture alone remains G3 provider evidence, and
      handle-backed replacement remains G4 rather than G2.
- [x] Exercise the authorized G2 seam natively across submissions 41--43.
      Local slots repeat under distinct submission identities, two persistent
      controls remain at two lifetime uploads and zero replacements, a missing
      material rejects submission 900 atomically, and the following valid
      submission presents without diagnostics. Submission 42 rebuilds the
      source-view geometry with an eight-unit source-X jitter and a distinct
      geometry fingerprint; submission 43 restores the baseline view and exact
      submission-41 fingerprint without persistent resource churn. The native
      presentation orders persistent background colour, G2 source-authority
      depth, then the persistent far-wall control. Browser WebGPU repeats the
      same lifetime and recovery observations. Its first visual run exposed
      equal-sized persistent control meshes (orange fully hid blue), an
      ordinary fixture defect now corrected to the native nested bounds;
      maintainer observation confirms the corrected blue outer and orange
      inner controls are both visible in Browser WebGPU. This is semantic
      cross-target visual evidence, not a pixel-identity guarantee.
- [x] Extend the G2 fixture with the complete ordered depth relationship on both
      targets. Submissions 41--43 now retain five draws, three persistent
      uploads and zero replacements. Persistent green near geometry survives,
      submission-local authority suppresses farther orange geometry only in
      its declared region, and persistent blue sky/background remains
      elsewhere. The seam remains feature-gated, corpus-only and unstable;
      this evidence does not admit a stable renderer contract.
- [x] Run the Candidate 1 synthetic conservation matrix. All fourteen ordered
      reference cases and ten focused controls balance with zero unexplained
      contributions. The positive sky case lowers 66 retained intervals and
      2,046 diagnostic oracle cells to two local declarations/four triangles;
      paired-only and one-sky negative authority cases create no G2 batch.
      Ordinary walls, planes, apertures, explicit runtime snapshots and cutout
      behavior remain outside the sky-depth delta rather than being filtered
      or reconstructed by it. Native and Browser WebGPU observations remain
      semantic comparisons, not pixel-identity claims.
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
- Provider-prepared retained regions can be handed off without requiring a
  lossy prepared-view -> approximate-world -> view round trip. The admitted
  contract may realize them as view-local ordinary geometry or through another
  bounded mechanism, but it must preserve continuous coverage and source
  correlation across camera jitter and near-view movement.
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

### Cycle 4 -- 2026-08-16

- New evidence: the source-topology admission study held source geometry fixed
  and falsified whole-contribution Boolean admission at baseline, jittered,
  and nearer camera poses.
- Finding: one far source SEG requires partial participation. Whole admission
  retains 81–97 forbidden columns, while whole rejection loses 9–15 required
  columns, and ordinary depth has no authorized occluder for the overlap.
- Terminology: “prefilter” is now too narrow. Doom preparation may produce
  `0..N` view-local occurrences from one source contribution.
- Representation constraint: the `320 x 200` diagnostic grid is evidence, not
  semantic vocabulary. The next Doom-local experiment must compare continuous
  source-relative/view-relative occurrence representations, starting with
  ordinary view-local triangles before considering a bounded screen-local
  primitive.
- Disposition: no stable API admitted. Alternative C is falsified as the full
  solution; Alternatives F/G remain under review, and Quake/non-BSP evidence
  is still required before shared vocabulary can be selected.

### Cycle 5 -- 2026-08-16

- Primary-source review: pinned Linux Doom and Chocolate Doom revisions were
  traced from frame setup through BSP admission, wall realization, plane/sky
  emission, and deferred masked drawing.
- Finding: Doom constructs presentation incrementally. Horizontal wall
  admission, vertical open-window mutation, wall tiers, and plane marks are
  coupled effects; sky paints retained plane coverage rather than creating it.
- Seam: replace the failed horizontal occurrence candidate with a bounded
  Doom-private ordered coverage reference planner, then lower every retained
  output into ordinary Tokimu declarations for prepared-full-submission.
- Ordering constraint: do not start AABB/frustum post-filtering until the
  prepared-full-submission is independently clean. A generic filter cannot
  restore an occurrence omitted by source preparation.
- Disposition: no stable API admitted. The source trace strengthens
  Alternatives F/G while leaving their provider-neutral vocabulary dependent
  on the commissioned non-Doom campaigns.

### Cycle 6 -- 2026-08-16

- Source-parity repair: audit against pinned released Doom corrected the first
  ceiling-plane row, the no-upper ceiling transition, and the last open row
  below `floorclip` in the provider's unsigned clip representation.
- E1M1 finding: the repaired source protocol reaches canonical E1M1 coherently
  before lowering. Source spawn retains 37 admitted SEGs, 9 resolved plane
  instances, 17 horizontal spans, and 1,205 populated columns with no
  overlapping writes or unresolved plane instances.
- Representation finding: the legacy fixed-view path preserves that
  raster-shaped ledger through 1,205 inverse-projected quads, whereas the
  continuous ordinary-geometry approximation loses required contributions
  despite balancing its internal ledger.
- Disposition: this is pressure for a Doom-private view-local realization, not
  evidence that generic AABB filtering or renderer-owned Doom semantics can
  repair the candidate. No stable API is admitted; return the representation
  question to AR-0030 before broadening the renderer handoff.

### Cycle 7 -- 2026-08-16

- New evidence: the authoritative-sky Candidate 1 model conserves the retained
  Doom sky authority as two continuous declarations containing 12 vertices and
  four triangles, rather than reconstructing the 2,046 diagnostic cells.
- Negative controls: paired-only, one-sky, ordinary-aperture, empty,
  invalid-depth and near-plane inputs create no unauthorized depth declaration
  and preserve bounded rejection evidence.
- Renderer audit: the present `RenderCommand` handoff exposes mesh geometry
  only through persistent `MeshHandle` resources. Existing upload machinery
  necessarily creates or replaces such resources when view-local geometry
  changes.
- Architectural finding: ephemeral prepared-view geometry and persistent
  renderer resources cannot remain honestly distinct through the current
  contract. This is the first concrete Candidate 1 pressure on the shared
  handoff, but Doom alone cannot select its stable vocabulary.
- Disposition: pause the GPU realization at the AR-0030 gate. Do not use
  persistent replacement as a workaround, do not advance to Candidate 2, and
  do not admit a stable renderer API until the commissioned campaign comparison
  can evaluate the same lifetime distinction.
- Reviewer concurrence: Candidate 1 failed at the handoff, not at its depth or
  sky semantics. The next question is how `tokimu-render` accepts bounded
  geometry whose identity and lifetime belong to one submission/view rather
  than to a persistent renderer resource.
- Comparative refinement: privately test inline submission geometry,
  submission-local geometry, a frame-local pool and persistent replacement as
  a negative/control. Submission-local geometry is the leading hypothesis
  because “frame” is not yet proven equivalent to the demonstrated prepared
  submission/view lifetime.

### Cycle 8 -- 2026-08-16

- G2 result: the corpus-only submission-local geometry intake preserves
  submission identity, rejects stale/invalid local references atomically,
  recovers on the next valid submission and executes on native WGPU and Browser
  WebGPU without persistent mesh replacement. G2 remains useful experimental
  lifetime evidence.
- Candidate 1 E1M1 result: an independent Doom-authoritative sky-depth surface
  over unchanged global geometry simultaneously clips valid far-left and
  hut-adjacent geometry while still admitting distant rooms beside the hut and
  above the wall.
- Realization control: all 320 modeled source-column centers match the exact
  extracted ledger interval with zero missing or extra cells. All 320 source
  depths resolve; maximum absolute clip-depth error is `0.000000050` and mean
  error is `0.000000017`.
- Finding: Candidate 1 is not failing because its continuous triangles poorly
  approximate the ledger subset. The extracted authority becomes semantically
  insufficient when detached from the ordered wall/plane protocol and treated
  as a free-standing occluding surface over the global shell.
- Disposition: stop Candidate 1 tuning. Preserve G2 as separate renderer
  lifetime evidence. Before authorizing Candidate 2, run one smaller
  Doom-private falsifier in which finite source-authority occurrences classify
  competing contributions as nearer, beyond, straddling or unresolved before
  ordinary renderer submission.
- Boundary: candidate normals may be retained as facing diagnostics, but they
  do not own occlusion. Infinite supporting planes, texture inference,
  screenshot rules and generic post-filter repair are prohibited. A need for
  ordered overlapping coverage/composition returns to this review before any
  renderer boundary broadens.

### Cycle 9 -- 2026-08-16

- Relational-candidate refinement: comparing a complete world contribution
  only by whether it is nearer than a finite authorizing boundary is
  insufficient. Oversized or overlapping mapper geometry can be nearer while
  extending beyond any source occurrence that authorized its presentation.
- New prerequisite: source support eligibility precedes relational depth.
  Walls are bounded by finite SEG/source-relative occurrences; floors and
  ceilings are bounded by subsector-local plane occurrences unless the ordered
  source protocol proves a broader shared occurrence. Sector identity alone
  does not authorize one global plane contribution.
- Required decision shape: first classify supported, outside-source-support or
  unresolved-support portions; only supported portions may then become nearer,
  beyond, straddling or unresolved relative to an authorizing occurrence.
  Missing support evidence fails open.
- Synthetic falsifier: retain a deliberately oversized floor/ceiling whose
  unsupported excess is geometrically nearer, plus an inverse straddling
  control. Nearness must not admit the excess or let it change the disposition
  of the supported portion.
- E1M1 launch control: the frozen exterior-hut-east composition retained 1,922
  original contributions under global full submission with aggregate inventory
  hash `30650e57ad9b3c07`. All remained unresolved/fail-open because the new
  classifier is not implemented; this is an honest control, not four-case
  relational evidence.
- E1M1 visual confirmation: elevated and fixed exterior inspection exposes
  complete distant rooms and overbroad floor/ceiling regions in the global
  shell. Lazy/overlapping mapper geometry is therefore demonstrated corpus
  pressure, not merely a synthetic precaution. Geometric validity and nearness
  do not establish view-local presentation eligibility.
- Boundary: this remains Doom-private source-occurrence preparation upstream of
  ordinary renderer declarations. If honest support restriction requires
  ordered accumulated coverage rather than bounded occurrences, stop the
  relational study and return that pressure to Candidate 2/AR-0030.

### Cycle 10 -- 2026-08-17

- Diagnostic retention: the E1M1 debug console now mirrors commands and
  responses to the invoking terminal, allowing exact source-ray evidence to
  survive window disposal without adding renderer or kernel diagnostic
  ownership.
- Replay result: five newly copied `LOOK` rays reproduce headlessly with the
  same candidate identity, hit distance, finite sky relationship and classic
  source trace. Together with the prior beside-hut ray, the relational study
  now has six deterministic source inputs rather than screenshot coordinates.
- Important counterexample: one ceiling candidate in subsector `104` is
  reached by the classic source traversal even though a finite sky boundary is
  encountered earlier on the same ray. Other rays are elided by solid ranges.
  Therefore `target reached/not reached` is diagnostic evidence, not a
  sufficient generic keep/reject rule.
- Required ordering remains: candidate occurrence support, finite authority
  overlap, relational depth, then keep/reject/split/unresolved-fail-open.
  Neither an infinite supporting plane, nearest sky hit nor classic traversal
  membership may independently delete a complete contribution.
- The exact rays are retained in the four-case capture ledger, but five remain
  intentionally unassigned to named visual cases because their terminal
  transcript did not include human case labels. This blocks a four-case claim,
  not headless model work.

### Cycle 11 -- 2026-08-17

- Corpus-private partial-contribution evidence now partitions a bounded source
  contribution across source-parameter, horizontal and vertical support and
  then splits a linear depth crossing deterministically. Retained, rejected,
  outside-support and unresolved fragments conserve the original domain while
  preserving source/material/sidedef provenance and UV progress.
- The focused relational gate passes 18 tests. A standalone report retains an
  eight-fragment control with one nearer survivor, one beyond rejection, six
  explicit outside-support slabs and zero unresolved regions. No renderer
  policy or stable contract is introduced.
- Architectural falsifier: two ordered finite authorities can own different
  regions of one contribution. Resolving only the first authority necessarily
  labels the later-owned region as outside the first support, making it
  indistinguishable from genuinely unsupported space. Whole-contribution
  priority and single-authority splitting are therefore insufficient.
- Finding: honest realization requires ordered partitioned composition over
  the remaining contribution domain, with each authority able to classify only
  its finite support. This is concrete Candidate 2 pressure, not permission to
  expose Doom columns, source topology or compositor vocabulary through
  `tokimu-render`.
- Disposition: pause relational Slice 3 before GPU presentation. Decide the
  smallest Doom-private ordered composition experiment under this review; do
  not lower a knowingly incomplete single-authority result and do not repair it
  with AABB/frustum filtering.

### Cycle 12 -- 2026-08-17

- Authorization: one Doom-private, headless ordered partitioned-composition
  experiment is authorized. Each finite authority may refine only the still
  eligible candidate domain in retained source order; terminal retained,
  rejected and unresolved regions cannot be reopened.
- Synthetic result: the two-authority falsifier now conserves one complete
  contribution as one earlier-authority nearer survivor and one
  later-authority beyond rejection. The standalone report retains two ordered
  steps, two terminal fragments, zero unresolved fragments and final
  conservation; the focused library gate passes 141 tests.
- Ordering result: reversing overlapping authority order produces an
  observable semantic difference, while equal-order overlapping solid
  authorities fail open. The experiment does not hide priority behind source
  identity or proximity.
- Source-support result: independently unsupported lazy-map excess remains
  unresolved/fail-open; no authority sequence is permitted to authorize a
  candidate outside its own finite source occurrence. Cutout authority remains
  non-solid.
- Boundary: the experiment introduces no renderer vocabulary, public
  screen-column semantics, general compositor or stable contract. G2
  submission-local geometry remains separate evidence.
- Remaining gate: the six retained E1M1 replay rays contain exact source hits
  and scalar authority relations, but not complete finite occurrence domains.
  E1M1 classification and GPU presentation remain gated until those domains
  are derived from the ordered Doom source protocol. Nearest-hit distance is
  not accepted as a substitute.
- Stop condition: if E1M1 requires reopening finalized regions, arbitrary
  priorities or a global raster lifecycle, return to this review rather than
  enlarging the Doom-private experiment silently.

### Cycle 13 -- 2026-08-17

- Diagnostic refinement: the six retained E1M1 rays now report ordered wall
  and plane occurrence domains alongside their global-shell `LOOK` hits. Wall
  evidence includes SEG-local source parameter, view interval and front/back
  opening; plane evidence includes plane-instance identity, height, retained
  destination view intervals and triangle count.
- Result: five of six suspect global-shell contributions are already absent
  from Doom's ordered source result. The remaining ceiling candidate survives
  only in two narrow view intervals. All six authorizing occurrences are finite
  and source-attributed.
- Finding: most observed geometry leakage is presently caused by global-shell
  realization reintroducing contributions after the source protocol has made a
  terminal rejection, not by the ordered protocol failing to classify them.
  The relational composer must consume only the still-eligible remainder; it
  cannot reopen rejected source contributions.
- Remaining case: the retained ceiling plane and its wall-SEG authority do not
  share the composer's required source parameterization. Plane destination
  intervals and SEG-local source progress are both valid source facts, but an
  invented mapping between them would be an architectural expansion and would
  make nearest-hit evidence authoritative indirectly.
- Disposition: pause the E1M1 relational composer gate at this common-domain
  boundary. Synthetic ordered partition composition remains valid and
  Doom-private. No renderer contract, screen-column API, general compositor or
  priority vocabulary is admitted.
- Next decision: either authorize one bounded source-grounded experiment for a
  plane-versus-SEG comparison domain, or treat the evidence as support for
  realizing the already coherent ordered Doom result directly before any
  relational refinement. GPU presentation and generic AABB/frustum filtering
  remain downstream.

### Cycle 14 -- 2026-08-17

- Maintainer disposition: freeze the relational composer at its successful
  synthetic boundary. Do not invent a plane-to-SEG parameter adapter solely to
  make the abstraction cover the remaining E1M1 ceiling case.
- Interpretation: the relational study did not fail. It established that
  rebuilding already-terminal ordered decisions downstream is redundant, and
  that heterogeneous contribution families need not share one artificial
  source parameterization.
- Authorized next path: literal realization of the coherent ordered Doom
  result. Whole survivors may reuse ordinary geometry; terminal rejects emit
  nothing; partial SEGs use source-relative wall fragments; partial planes are
  isolated as focused representation pressure; unresolved cases fail open with
  bounded evidence.
- Prohibition: the implementation must not return to the global shell after
  preparation, reopen the five source-rejected ray contributions, repair the
  result with AABB/frustum filtering, or promote Doom occurrence vocabulary to
  `tokimu-render`.
- AR-0030 pressure retained: if faithful partial-plane realization cannot be
  expressed through ordinary or G2 submission-local geometry without losing
  the authoritative prepared domain, return to this review before proposing a
  stronger provider-neutral presentation primitive.

## References

- `docs/contribution-admission-guide.md`
- `docs/Tokimu Software Design Document.md`
- `docs/Architectural Reviews/AR-0023-textured-surface-alpha-and-depth-policy.md`
- `docs/Architectural Reviews/AR-0025-camera-candidate-selection-and-visibility-culling.md`
- `docs/Architectural Reviews/AR-0026-non-euclidean-spatial-charts-and-authored-angular-topology.md`
- `docs/Plans/DOOM/Studies/Doom viewer-relative presentation synthetic conformance.md`
- `docs/Plans/DOOM/Studies/Doom source-topology admission over complete geometry.md`
- `docs/Plans/DOOM/Studies/Doom ordered source occurrence preparation.md`
- `docs/Plans/DOOM/Studies/Doom authoritative sky coverage delta realization.md`
- `docs/Plans/DOOM/Studies/Doom source-authorized relational contribution classification.md`
- `docs/Plans/DOOM/Evidence/Classic Doom visibility clipping evidence.md`
- `docs/Plans/DOOM/Evidence/Classic Doom renderer dataflow and Tokimu preparation seam.md`
- `docs/Plans/DOOM/Evidence/Doom authoritative sky-depth realization seam evidence.md`
- `docs/Plans/DOOM/Evidence/Doom relational classifier four-case capture ledger.md`
- `docs/lessions/read-reference-source-early.md`
