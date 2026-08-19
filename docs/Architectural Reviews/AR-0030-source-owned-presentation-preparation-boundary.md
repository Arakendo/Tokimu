# AR-0030: Tokimu Render Preparation And Submission Framework

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-08-14 |
| Last reviewed | 2026-08-17 |
| Scope | Stable Tokimu render API / program preparation / renderer boundary |
| Trigger | Doom synthetic and E1M1 evidence falsified both global static-shell rendering with sky depth patches and whole-source Boolean filtering as sufficient source-faithful presentation models. |
| Related ADRs | ADR-0001, ADR-0003, ADR-0008, ADR-0009, ADR-0013, ADR-0014 |
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

### Cycle 15 -- 2026-08-17

- Native lifecycle result: the literal ordered-result implementation gained a
  shared Rust-owned preparation entry, live camera refresh and current
  door/platform height snapshots without admitting Doom semantics to the
  renderer.
- Structural result: source-spawn, bounded yaw/movement, six-ray and runtime
  snapshot controls conserve successfully through complete ordinary
  declaration replacement.
- Visual falsification: the complete source-spawn frame reports 458 draws but
  exposes severe sky-background leakage through large peripheral and smaller
  interior regions. Conservation proved complete handling of the candidate's
  records, not semantic correctness of those records.
- Disposition: park the literal ordered-result study. Do not advance browser
  parity, generic post-filtering or sky-depth repair from the visually invalid
  result.
- Authorized R&D: open a Doom-private BSP presentation-domain resolver study
  in headless shadow mode over the original complete contribution inventory.
  First attribute four marked source-spawn rays through exact camera domain,
  BSP path, subtree decision, source disposition, lowering and handoff.
- Boundary unchanged: BSP/source participation remains Doom-owned; no stable
  renderer/runtime contract or shared visibility capability is admitted. If a
  safe prefilter cannot be separated from the coupled wall/vertical-plane
  protocol, retain that as a falsification and return here.

### Cycle 16 -- 2026-08-17

- Diagnostic refinement: the BSP study now has a full-submission visual
  instrument over the original global contribution inventory. Shadow BSP
  classification selects only corpus-local PNG materials and transient tint
  overrides; it cannot remove geometry.
- Source taxonomy: diagnostic families are floor plane, ceiling plane,
  one-sided wall, source code-1 door, two-sided boundary, masked middle and
  presentation-global skybox. Unequal current floor/ceiling heights refine a
  two-sided boundary as a height-transition boundary. “Window” and “stair” are
  deliberately not promoted from human visual interpretations to source
  semantics.
- Conservative outcomes: admitted SEGs and visited source planes are bright;
  only positive terminal solid-range coverage is shown as rejected. Projection
  or traversal absence remains an explicit unresolved/fail-open blue state.
- Inspection loop: all, accepted, rejected and unresolved focus modes retain
  identical draw membership and dim only nonmatching records. `LOOK` replay
  reports family, disposition, exact reason and source identity for the hit.
- Runtime scope: free look and movement recompute the shadow observation from
  the current camera and application-owned current-height snapshot. No
  activation or timing policy moves into preparation.
- Boundary unchanged: the existing provider-neutral draw-local material
  override realizes the visualization. No Doom term, BSP traversal, source
  identity or new contract enters `tokimu-render`; the resolver remains
  appearance-only until the study's presentation gate is earned.

### Cycle 17 -- 2026-08-17

- Live diagnostic finding: a purple floor hit in source subsector `113`
  exposed two distinct BSP-derived domains. Its one SEG has endpoint bounds
  `x=896..928, y=-3072`, while its nine-step root-to-leaf partition path
  infers a valid convex plane region `x=896..928, y=-3104..-2992`.
- Exact evidence: the retained ray hits the reconstructed floor at
  `(925.819,-3026.415)`, inside the inferred plane region but `45.585` source
  units outside the explicit SEG bounds. Node `107` projects the far-child SEG
  box outside the view while the floor hit is near the view center.
- Interpretation: child bounding boxes over wall/SEG evidence cannot be
  promoted directly to whole-subsector plane rejection authority. The flat
  itself is not thereby shown oversized; it is lowered from the complete
  retained BSP partition path, whose implicit boundaries extend beyond the
  leaf's explicit SEG endpoints.
- Diagnostic refinement: `LOOK` now reports SEG bounds, inferred plane-region
  bounds and hit membership in both. The shadow reason
  `source-plane-child-seg-bounds-outside-fov` remains purple,
  unresolved/fail-open and fully submitted.
- Architectural pressure retained: a future Doom-private resolver may need
  distinct wall/SEG and plane participation rules, or may prove inseparable
  from the coupled wall/vertical-plane protocol. No presentation authority,
  renderer contract or engine-wide visibility abstraction is admitted by this
  finding.

### Cycle 18 -- 2026-08-17

- Viewport-scan finding: a pitched/free-look view exposed two ordinary wall
  hits at the extreme right edge that the frozen classic-width BSP shadow
  manifest left unresolved. Both hits lie inside their explicit SEG bounds
  and inferred leaf regions; their source SEGs are admitted when the off-axis
  rays are traced as centered views.
- Camera-domain evidence: the frozen source heading is `-154.944°`; the two
  wall sample rays are offset by `-48.688°` and `-45.416°`. They are therefore
  outside the resolver's fixed classic `±45°` horizontal traversal domain
  even though the pitched Tokimu perspective presents them in the viewport.
- Diagnostic refinement: these cases now report
  `source-wall-seg-bounds-outside-fov` rather than generic traversal ambiguity,
  and sampled LOOK reports the normalized sample-minus-view heading.
- Architectural finding: unrestricted Tokimu pitch cannot be assumed to share
  one classic fixed-width horizontal Doom source view. A presentation-
  affecting resolver would need an explicit choice among the actual Tokimu
  frustum, multiple source views, or a constrained classic presentation
  domain. That choice is not made by this diagnostic refinement.
- Boundary unchanged: no generic camera, renderer or runtime API is changed;
  unresolved edge walls remain submitted. Do not widen the Doom-private FOV
  or reject these contributions without a deliberate AR-0030 decision.

### Cycle 19 -- 2026-08-17

- Maintainer direction: use actual prepared geometry and the actual Tokimu
  frustum as a conservative false-negative guard around source-specific BSP
  evidence. A BSP proxy disagreement must not omit geometry that intersects
  the actual view; retain and diagnose it instead.
- Family authority: exact finite wall SEG/projected-range evidence may support
  strong wall rejection. Whole floors and ceilings require plane-appropriate
  support and may not inherit rejection authority from one child SEG endpoint
  box or covered horizontal range alone.
- Shadow implementation: a solid-range-pruned plane is classified rejected
  only when its prepared draw AABB is also definitely outside the actual
  Tokimu frustum. An intersecting plane becomes unresolved/fail-open with
  `prepared-geometry-frustum-vetoed-plane-rejection`; unavailable bounds or
  camera evidence remains explicit ambiguity. Wall classification is
  unchanged.
- Scope: the prepared AABB/frustum result is a veto against unsafe negative
  evidence, not a generic visibility oracle, occlusion system or positive BSP
  acceptance rule. Full diagnostic submission remains conserved and no
  renderer/runtime/public contract changes.
- Validation: a focused three-outcome test fixes rejected, vetoed and
  unavailable plane behavior. The retained `16x10` subsector `64` replay still
  produces one unresolved floor group, zero rejected samples and the same
  inferred-region/SEG-box disagreement. A two-frame source-spawn GPU control
  reclassifies exactly `158` plane draws from rejected to unresolved, retains
  all `1,849` declarations and reports zero renderer removals.
- Live falsifier: a later `32x20` walk/free-look scan reports `120` visible
  prepared-frustum veto samples and zero rejected nearest hits across the
  floors and ceilings of subsectors `97` and `99`. The center subsector `97`
  floor hit is `46.273` units outside its sole SEG box but inside the inferred
  plane region; its BSP leaf is nevertheless solid-range covered. This
  directly falsifies promoting horizontal solid coverage of the child proxy
  to whole-plane rejection authority and corroborates the family split. An
  exact headless replay reproduces the four groups and expands representative
  hits `10.699`, `53.255`, `6.538` and `123.321` units outside their SEG boxes
  but inside their inferred plane regions.
- Remaining gate: exercise source-spawn, movement, pitch and runtime-height
  snapshots before deciding whether any family-specific negative evidence is
  trustworthy enough for a presentation-affecting experiment.

### Cycle 20 -- 2026-08-17

- Canonical baked-data audit: all `236` raw E1M1 NODES records exactly match
  the decoded partition, bounding-box and child fields. All `472` child boxes
  contain their complete recursively collected descendant SEG-endpoint
  envelopes; no source-proxy underbound was found.
- Representation result: only `149/472` child boxes contain the corresponding
  inferred convex plane-region envelope. `323/472` inferred plane domains
  extend beyond the boxes, including the retained node `95`/`96`, subsector
  `97`/`99` falsifiers.
- Finding: the decoder and canonical BSP bake are consistent for the source
  structures the boxes actually bound. The defect is downstream authority:
  a valid source-subtree prune cannot be promoted to rejection of a larger
  reconstructed whole-plane contribution.
- Cross-domain rule: a bound has negative authority only over the
  representation it actually bounds. This applies to future hierarchical
  source imports without making source BSP boxes a generic engine visibility
  concept.
- Disposition: keep canonical `DOOM1.WAD` unchanged. A later alternate-node-
  builder comparison may perturb the evidence diagnostically, but rebaking is
  not an authorized repair or runtime prerequisite.
- Boundary unchanged: the audit is a corpus-local headless observation over
  import bytes and existing decoded/inferred structures. It adds no provider,
  renderer, runtime or stable/public contract.

### Cycle 21 -- 2026-08-17

- Positive-authority correction: reaching a source subsector selects possible
  Classic Doom floor/ceiling destinations but does not prove occurrence of an
  entire reconstructed Tokimu convex plane mesh. The earlier
  `visited-source-plane=accepted` diagnostic overclaimed authority.
- Shadow refinement: reached intersecting planes now remain unresolved/fail-
  open with `source-plane-subsector-reached-occurrence-unproven`. Prepared
  planes definitely outside the actual Tokimu frustum use the distinct
  `rejected-outside-frustum` disposition and
  `prepared-geometry-outside-frustum` reason. Solid-range rejection remains a
  wall/SEG result.
- Source-spawn control: all `1,849` declarations remain submitted with zero
  renderer removals. Plane outcomes are `584` actually outside-frustum and
  `269` unresolved; accepted outcomes are now the `70` exact admitted wall-
  family draws rather than `407` reached plane draws plus those walls.
- Viewport control: the retained `32x20` walk/free-look replay reports `631`
  hits, `203` accepted wall-family hits, zero rejected hits and `428`
  unresolved plane hits across fourteen groups. No visible sampled plane is
  rejected or mislabeled accepted.
- Remaining authority question: plane acceptance or source-specific rejection
  requires wall-range/per-column plane-occurrence evidence. Neither subsector
  reachability nor child-box pruning supplies it alone.
- Boundary unchanged: this remains appearance-only corpus shadow evidence over
  unchanged full submission. No renderer, runtime or stable/public contract is
  added.

### Cycle 22 -- 2026-08-17

- Plane-evidence refinement: interactive `LOOK`, headless single-ray replay
  and automatic headless-scan representatives now match flat hits against the
  existing Classic `320x200` plane-span observation by exact plane key and
  source sector. Reports retain matching instance, populated-column,
  populated-cell and source-SEG counts.
- Authority remains deliberately narrow: a matching span proves that the
  source plane identity participates somewhere in the frozen Classic view. It
  does not prove that an entire reconstructed Tokimu subsector mesh or an
  arbitrary pitched Tokimu pixel participates. Absence is diagnostic and does
  not authorize whole-plane rejection.
- Retained falsifier: subsector `97`'s exact visible floor occurrence remains
  source-protocol rejected, while the same sector-38 `FLOOR4_8` plane key is
  present elsewhere in one Classic instance with `249` populated columns,
  `17,381` populated cells and eight source SEGs. Plane identity participation
  therefore cannot be substituted for exact occurrence authority.
- Validation: the focused static-scene suite passes `74/74`; the retained
  headless ray reproduces the expected exact hit, BSP waterfall and new span
  evidence without renderer startup.
- Runtime freshness control: immutable current-height snapshot replays still
  change twelve door-sector and nineteen platform-sector target declarations
  through the same shared preparation seam with zero unresolved lowering. The
  platform's horizontal occurrence fingerprint remains stable while vertical
  realization changes, as expected; activation and timing policy remain
  application-owned and absent from preparation.
- Boundary unchanged: the implementation is corpus-local diagnostic work over
  an existing provider observation. It changes no submission, renderer,
  runtime ownership or stable/public contract.

### Cycle 23 -- 2026-08-17

- Proposed direction: park the attempt to use Classic Doom's BSP directly as
  rejection authority over Tokimu's larger reconstructed presentation meshes.
  Retain its complete diagnostic and audit evidence as a falsifier.
- Successor plan: `docs/Plans/DOOM/Tokimu BSP capability setup plan.md`
  proposes a deterministic BSP bake over the exact geometry Tokimu intends to
  query, beginning corpus-locally with E1M1.
- Leading ownership hypothesis under ADR-0003: Ring 1 retains only lightweight
  spatial primitives; an earned BSP semantic model/provider contract belongs
  in an optional Ring 2 spatial capability; specialized or external providers
  belong in Ring 3. A small portable reference implementation may remain with
  Ring 2 if it is dependency-light and corpus evidence earns it.
- Required distinction: a geometry BSP may earn conservative spatial candidate
  authority without earning Doom presentation, occlusion, sky, wall-tier or
  plane-occurrence authority. Doom-private preparation remains responsible for
  source semantics and the renderer still receives ordinary declarations.
- Authorization state: documentation and review only. No new crate, stable
  trait, facade export, engine dependency or presentation-affecting rejection
  is authorized. The first possible implementation gate is one corpus-local
  deterministic bake with containment, conservation and actual-camera query
  evidence.
- Reopening evidence: Ring 2 admission requires Doom, Quake and ordinary
  non-BSP pressure. Failure to separate useful spatial meaning from Doom's
  coupled presentation protocol favors a provider-private BSP rather than a
  generic capability.

### Cycle 24 -- 2026-08-17

- Tokimu-first clarification: the proposed capability must define BSP from
  Tokimu's own finite-member identity, partition, split correlation,
  containment, revision, conservative-query, conservation and bounded-failure
  requirements. Doom and Quake structures are later mappings, adapters,
  rebake inputs or coexisting source-private structures; they do not define the
  Tokimu semantic model.
- Provisional smallest meaning deliberately excludes visibility, occlusion,
  portals, PVS, renderer ordering, materials, collision, simulation, Doom
  subsectors/SEGs/visplanes and Quake brushes/contents. These may consume or
  annotate spatial results without becoming BSP semantics.
- Representation requirement: every artifact must name the exact finite member
  representation it partitions and preserve exact fragment-to-original
  correlation. No public Rust generic or stable type is implied by this
  semantic requirement.
- Required adversary: compare the corpus-local BSP experiment with a BVH or
  equivalent containing hierarchy over the same members. If ray/frustum
  candidates, containment and dynamic refit are the only earned needs, BSP may
  be an implementation behind a smaller spatial-query capability rather than
  Ring 2 meaning of its own.
- Source relationship: Classic Doom's BSP and a Tokimu BSP may legitimately
  coexist because they answer different questions. Direct source mapping is
  permitted only with proof that member identity, bounds and guarantees satisfy
  the Tokimu contract.
- Authorization remains documentation-only. A first implementation review must
  authorize both the Tokimu-first corpus bake and its BVH/control adversary;
  it must not select BSP merely because this investigation began with Doom.

### Cycle 25 -- 2026-08-17

- Authorized corpus result: the first Tokimu-first bake uses the exact `1,849`
  prepared E1M1 triangles as finite members, retaining draw/triangle identity
  through splitting. It is corpus-local and does not read Doom BSP topology,
  affect submission or introduce a shared API/crate.
- Correctness: the bounded BSP and same-inventory BVH both report zero
  containment failures and complete member conservation. BSP fragment area is
  conserved within the declared tolerance. Independent runs repeat BSP
  fingerprint `78b7e9300f148c33` and BVH fingerprint `599d8ca7411ffd11`.
- Material BSP result: a deterministic median-axis splitter reaches its global
  `500,000` generated-fragment budget, leaves `334,345` final fragments from
  `1,849` triangles (`180.824770x`), creates `14,231` nodes and reaches depth
  `20`. Fragment payload alone has a `16,048,560`-byte lower bound. Family
  amplification is floor `254.25x`, ceiling `173.26x`, wall `150.64x` and
  cutout `113.12x`.
- BVH adversary: the unsplit control retains `1,849` members in `255` nodes at
  depth `7`, with `1.0x` amplification. Observed debug-build construction is
  approximately `4.9–5.2 ms`, versus `406–412 ms` for the bounded BSP.
- Ordinary failure corrected: the first report enforced the fragment limit per
  node and reached `650,337` final fragments (`351.72x`). The limit is now a
  global generated-work budget; when reached, remaining geometry stays in a
  conservative leaf. The first result remains retained evidence rather than a
  valid bounded artifact.
- Architectural finding: naive median-axis triangle splitting is not a viable
  default Tokimu spatial index for this representation, while the BVH currently
  satisfies the same Slice 1 containment/conservation requirements much more
  cheaply. This does not falsify all BSP policies, but continuing requires an
  explicit choice among a different split/member policy, a non-splitting
  partition experiment, or advancing BVH queries first. Do not optimize away
  the adversarial evidence implicitly.
- Authorization state: stop before Slice 2 actual-camera queries pending that
  choice. No Ring 2 admission, provider contract or presentation authority is
  earned.

### Cycle 26 -- 2026-08-17

- Selected disposition: advance the unsplit BVH control through actual-camera
  queries and park further BSP construction until a caller requires explicit
  partition topology or split-fragment semantics unavailable from a BVH.
- Corpus implementation: `--tokimu-spatial-query-report` rebuilds the same
  deterministic `1,849`-member BVH and queries it with Tokimu's checked native
  camera view/projection math at nine spawn, yaw, pitch, movement, near-wall
  and retained off-axis poses.
- Correctness: every BVH frustum result exactly equals the same-member
  brute-force conservative AABB/frustum oracle; all exact nearest-triangle ray
  results equal brute force. Totals are zero false negatives, zero false
  positives and zero ray mismatches. Matrix fingerprint is
  `3c80342bb2cfcdf4`.
- Work reduction: observed frustum leaf testing is `287..1,257` members and ray
  testing is `43..361` triangles instead of `1,849` brute-force tests. These
  are debug diagnostic observations and do not establish a performance budget.
- Retained falsifiers: the subsector 97/64 floor rays and linedef 101/107 wall
  rays resolve to the same exact source-correlated members as prior `LOOK`
  evidence without BSP child-bound authority or triangle fragmentation.
- Architectural finding: no current caller consumes BSP partition planes or
  split fragments. The evidence now favors investigating a smaller optional
  conservative spatial-query capability rather than admitting BSP as Tokimu
  Ring 2 meaning. No capability, crate, trait or public API is admitted yet.
- Portability limit: the native `static_scene` binary's existing window/runtime
  path is not a WASM target, so direct target checking fails before reaching
  this diagnostic. Browser parity requires an authorized portable consumer;
  this is not worked around by prematurely moving the corpus algorithm into an
  engine crate.
- Boundary unchanged: queries remain corpus-local, immutable and diagnostic.
  They do not alter renderer submission, Doom preparation authority, runtime
  ownership or presentation membership.

### Cycle 27 -- 2026-08-17

- Naming disposition: the leading architectural candidate is now an optional
  conservative spatial-query capability, not “Tokimu BSP.” BSP remains a
  historical study name and possible provider pending a caller that requires
  partition-specific semantics.
- Runtime endpoint diagnostic: `--tokimu-spatial-runtime-report` reconstructs
  current geometry for door sector 4 (`ceiling 0->68`) and platform sector 70
  (`floor 104->-48`) from immutable height snapshots. Activation, timing,
  collision and observer policy are absent.
- Representation proof: after applying ordinary preparation's zero-area
  omission rule, the reconstructed baseline exactly equals the complete
  `1,849`-triangle prepared geometry multiset, fingerprint
  `9f394a35516f5567`.
- Correctness: immutable rebuild and one reusable static-BVH/dynamic-sidecar
  union both exactly match complete current-geometry brute-force frustum and
  nearest-ray oracles at the retained boundary-local views. No renderer
  submission changes.
- Refit finding: bounds-only topology refit is unsupported for both endpoints
  because member identity changes. Door-open adds four triangles; platform-low
  replaces identities despite retaining the same total count. Refit cannot be
  treated as a mere bounds update under the exact-member contract.
- Cost observation: debug current-geometry preparation is approximately
  `10–11 ms`; immutable BVH rebuild is `7.5–8.5 ms`; sidecar current-member
  selection is `0.5–0.8 ms` after a reusable `1,831`-member static build.
  These are observations, not budgets.
- Lifecycle remains undecided: immutable rebuild provides simpler artifact
  identity; sidecar avoids static rebuild but requires explicit dynamic
  classification. Intermediate phases, release measurements and portable
  execution remain required before selection or Ring 2 admission.

### Cycle 28 -- 2026-08-17

- Runtime sequence: the corpus diagnostic now covers nineteen immutable door
  and platform revisions through closed/high, 25/50/75-percent motion,
  open/low, closing/ascending and repeated/waiting states. Both immutable BVH
  replacement and reusable static-BVH/dynamic-sidecar union match complete
  current-geometry frustum/ray brute force at every revision.
- Revision rule: application snapshot revision is bound separately from the
  geometry structure fingerprint. Opening and closing at the same height may
  reproduce geometry but cannot alias artifact lifecycle identity. All
  nineteen current revisions reject the baseline artifact identity.
- Dynamic bounds: the sidecar remains `18..22` members over a reusable
  `1,831`-member static artifact. Refit is eligible only at exact
  baseline-equivalent closed/high states and remains ineligible during actual
  motion because exact member identity changes.
- Release economics: twenty complete release replays provide `380` snapshot
  samples. Mean complete geometry preparation is `2.719 ms`; immutable BVH
  rebuild `0.430 ms`; sidecar extraction `0.133 ms`. Total update means are
  `3.149 ms` immutable and `2.852 ms` sidecar. Query means are `0.0568 ms`
  rebuilt and `0.0602 ms` composite sidecar.
- Corpus disposition: immutable replacement becomes the reference lifecycle.
  The sidecar's `0.297 ms` (`9.4%`) total saving does not yet earn composite
  revision/conservation/query-union semantics because both paths first rebuild
  the complete geometry. Sidecar remains a measured optimization candidate.
- Architectural gate: portable CPU/WASM evidence is next, but choosing its
  repository location and moving reusable machinery out of the native corpus
  binary requires placement review. No Ring 2 contract, shared crate, trait or
  facade export is admitted by this result.

### Cycle 29 -- 2026-08-17

- Placement review extracted only immutable BVH, conservative frustum/ray,
  audit, fingerprint, refit and revision mechanics into the corpus-local
  `tokimu-spatial-query-study` crate.
- E1M1 source conversion, runtime movement policy, renderer submission and
  presentation interpretation remain outside. No Ring 2 contract or facade
  export was added.
- One fixed fixture executes natively and through the
  `wasm32-unknown-unknown` test runner with identical fingerprint, candidates,
  nearest hit and stale/revised lifecycle assertions.
- Full E1M1 bake, nine-pose query and nineteen-snapshot runtime fingerprints
  and conservation remain unchanged after extraction.
- AR-0031 now owns prospective spatial-capability admission. This does not
  alter AR-0030's preparation question or grant queries presentation authority.

### Cycle 30 -- 2026-08-17

- The retained six-ray E1M1 handoff replay now queries the exact global
  prepared-triangle inventory through the corpus BVH before inspecting the
  final Doom ordered result. The BVH result is checked against the brute-force
  triangle oracle and remains shadow-only.
- All six suspect global-shell contributions are nearest BVH hits. Five have
  no final ordered declaration: walls `230` and `247` terminally reject their
  source SEGs, while ceiling planes for subsectors `149` and `104` have no
  association, destination, disposition or declaration at their rejected
  poses.
- The sixth BVH hit, ceiling subsector `104` at the reached pose, survives only
  as a partial plane occurrence with two finite view intervals and eight
  prepared declarations.
- Finding: actual-geometry spatial relevance and source presentation
  participation are orthogonal. A conservative BVH correctly retains all six
  triangles and therefore cannot repair the sky leak without being granted
  unsupported source-presentation authority.
- Disposition: no BVH submission filter is admitted. The leak fix remains in
  the Doom-private complete ordered-result handoff: whole retained geometry,
  terminal omission, partial SEG realization and focused partial-plane
  realization. Work stops before inventing a local classifier that would
  reconstruct coupled ordered coverage.
- Detailed evidence:
  `docs/Checkpoints/2026-08-17-sky-leak-bvh-source-shadow.md`.

### Cycle 31 -- 2026-08-17

- Maintainer authorization: treat the complete Doom ordered result as the
  authoritative live presentation input and solve partial planes from their
  own bounded plane-domain evidence. Do not broaden this into a generic plane
  compositor or renderer visibility contract.
- Vocabulary: the Doom-private semantic unit is a prepared presentation
  occurrence, conditioned by view and runtime snapshot. A source contribution
  may produce zero, one or several bounded occurrences; absence is
  authoritative.
- Focused implementation: partial source-plane triangles now intersect the
  exact ordered vertical plane cells matching kind, sector, subsector, height,
  texture, light and source SEG. Whole planes reuse ordinary geometry and
  terminal decisions still emit nothing. Resulting fragments consolidate into
  one ordinary mesh per surviving source triangle.
- Retained ceiling evidence: subsector `104`, sector `40`, `CEIL3_5` has `13`
  exact cells owned by SEGs `310/311`. Its retained ray now has one combined
  ordinary declaration; the five rejected rays still have zero declarations.
- Source-spawn structure: `3,432` lowered partial-plane triangles consolidate
  into `43` plane meshes. With `309` opaque walls and `12` cutouts, the live
  handoff is `352` opaque plus `12` cutout declarations and remains balanced.
- Lifecycle refinement: an explicit camera/runtime preparation identity skips
  identical stationary rebuilds. Identity is installed only after successful
  complete prepare-then-replace, preserving atomic refresh semantics.
- Boundary unchanged: no `tokimu-render`, Ring 2, BVH authority or stable
  public contract was added. Native visual falsifiers and browser parity
  remain open.
- Detailed evidence:
  `docs/Checkpoints/2026-08-17-doom-prepared-occurrence-partial-plane.md`.

### Cycle 32 -- 2026-08-17

- A direct pitch adaptation was tested before visual acceptance: camera pitch
  was added to the inverse projection of Classic Doom's retained 320x200 plane
  rows, and pitch was included in the preparation identity.
- This falsified the retained six-ray control. The previously proven partial
  ceiling at subsector `104` changed from one declaration to zero even though
  its source occurrence remained partial and the global BVH still proved the
  triangle geometrically relevant.
- Finding: Classic Doom plane rows are evidence for the unpitched source
  projection that produced them. Reinterpreting those row coordinates as
  coverage in a pitched Tokimu camera changes their represented world domain;
  camera pitch does not transfer that authority.
- The attempted adaptation was removed and the last conserved horizontal
  preparation restored. True pitched free-look remains an architectural
  question: it needs an explicit rule for deriving additional plane coverage
  outside the source protocol, not an arithmetic remap or silent fail-open.
- Per the study stop conditions, implementation pauses before copying the
  source vertical clipper, broadening provider contracts, submitting whole
  partial planes, or adding a renderer primitive.

### Cycle 33 -- 2026-08-17

- Native visual acceptance falsified the Cycle 31 prepared handoff at E1M1
  spawn after live movement. The active `365`-draw result presented large
  opaque foreground regions and effectively removed roughly half of the spawn
  room from the view.
- This is stronger than the earlier pitch-only finding: the unpitched exact
  plane-cell lowering can balance every structural ledger and still produce an
  invalid presentation under ordinary camera refresh.
- Disposition: Cycle 31 remains useful representation evidence but is not an
  accepted live rendering path. Its exact Classic row cells cannot yet serve
  as sufficient world-space prepared-plane geometry for Tokimu walkabout.
- No screenshot-specific omission, global-shell fallback, renderer exception,
  or relaxed conservation rule is authorized. The next work is the focused
  hardware-port precedent study already identified in Cycle 32, followed by an
  explicit AR-0030 representation decision.

### Cycle 34 -- 2026-08-17

- Focused primary-source precedent compared GZDoom with the smaller Doom
  iOS/PrBoom-style hardware path for floors, ceilings, sky and free-look.
- Both replace Classic visplane rows with persistent world-space plane
  geometry. GZDoom uses subsector/section surfaces plus render-sector, hack and
  portal preparation; Doom iOS admits coarser triangulated whole-sector planes
  when an uncovered subsector is reached.
- GZDoom retains source traversal but combines horizontal angular coverage with
  pitch-aware subsector tests over actual plane endpoint heights. Doom iOS
  retains horizontal occlusion and explicitly acknowledges over-admission risk
  from whole-sector granularity.
- Finding: the next candidate, if authorized, should test a Doom-private render
  subsector as the persistent plane unit, followed by actual-camera traversal.
  It must not use Classic row cells as final geometry or grant a reached leaf
  authority over an entire sector plane.
- No implementation is authorized by this evidence alone. Render-sector
  association, pitch-aware participation and sky role form a new
  representation decision for AR-0030.
- Detailed evidence:
  `docs/Plans/DOOM/Evidence/Hardware Doom arbitrary-pitch plane preparation precedent.md`.

### Cycle 35 -- 2026-08-17

- Maintainer authorization: proceed with one corpus-local Doom-private
  render-subsector actual-camera preparation experiment under the Cycle 34
  representation finding.
- The persistent unit owns a finite ordered subsector boundary, source and
  render-sector association, current plane facts, ordinary world-space
  surfaces, wall-tier correlation, sky role and explicit unsupported
  provenance. It is not a stable Tokimu BSP or renderer concept.
- The experiment replaces Classic row cells as final plane geometry. Classic
  ordered observations remain diagnostic oracles; they may not fill the new
  representation's holes or acquire authority under arbitrary pitch.
- Per-view preparation may use Doom BSP near-first ordering, source-appropriate
  horizontal coverage and pitch-aware tests over actual geometry. Child bounds
  retain authority only over the source representation they bound, and sky
  remains non-world presentation meaning.
- The first proof fence is geometry completeness, correct treatment of the five
  retained omissions plus partial ceiling, and continuity across neutral
  pitch, bounded pitch, yaw and movement. Neutral-pitch results must agree with
  retained Classic evidence wherever that evidence has exact authority.
- A deterministic per-render-subsector inventory is required before the first
  presentation-affecting candidate. The existing BVH may establish where
  geometry is, but Doom-private preparation alone decides whether its source
  presentation belongs.
- Implementation is authorized through corpus-local construction, shadow
  traversal, ordinary-declaration lowering, runtime/camera lifecycle and native
  visual acceptance. Browser parity starts only after the native visual gate.
- No new crate, stable/public trait, renderer semantic, generic portal
  primitive, sky-depth geometry or Ring 2 presentation authority is authorized.
  Evidence requiring one of those returns to this review.
- Controlling study:
  `docs/Plans/DOOM/Studies/Doom render-subsector actual-camera preparation.md`.

### Cycle 36 -- 2026-08-18

- Slices 0–2 of the authorized render-subsector experiment are complete as a
  corpus-private headless shadow; renderer submission remains unchanged.
- E1M1 construction conserves all 237 subsectors, 474 plane units, 732 source
  SEGs and 1,256 wall-tier triangles with zero unresolved boundaries,
  degenerates, containment failures or winding failures.
- Nine actual-camera poses, including bounded pitch, yaw, movement, return,
  near-wall and off-axis controls, produced zero geometric false negatives.
- All six retained participation rays agree. In particular, subsector 104's
  ceiling is retained from the reached pose and source-covered from the
  rejected pose, demonstrating view-local rather than whole-sector authority.
- An initial near-plane defect was repaired: a source solid SEG cannot close a
  range unless it is source-facing and both finite endpoints are in front of
  the camera. Near-plane ambiguity fails open.
- Repeated reports were deterministic: matrix fingerprint
  `20042a967aaec227`, six-ray fingerprint `64b7e8b802b10d14`.
- No stable contract, new crate, generic portal concept, sky-depth geometry or
  Doom renderer vocabulary was introduced. Slice 3 ordinary-declaration
  preparation is the next authorized step.
- Detailed evidence:
  `docs/Checkpoints/2026-08-18-doom-render-subsector-shadow.md`.

### Cycle 37 -- 2026-08-18

- Slice 3 now lowers the conserved render-subsector shadow into complete
  ordinary opaque/cutout declarations plus terminal sky, frustum and
  source-coverage evidence. It remains headless and is not installed.
- Seven prepared poses conserve all 2,182 input triangles. Spawn/return
  declaration identity is stable; the prepared matrix fingerprint is
  `d46154cd27ec89a9`.
- Ordinary finding repaired: texture-name membership was insufficient cutout
  provenance and initially yielded zero cutout declarations. Exact existing
  linedef/sidedef source classification correlates all 26 masked-middle source
  triangles; the owning-side green-room pose retains two cutout declarations.
- No renderer or stable contract changed. Atomic composition-local install is
  intentionally left pending so the alternative preparation theory can be
  reviewed at a clean shadow/presentation boundary.

### Cycle 38 -- 2026-08-18

- Authorized Slice 2B tested render-subsector connectivity as a shadow-only
  discriminator before the prepared view acquired renderer authority.
- E1M1's finite graph contains 237 cells, 607 shared-boundary relationships and
  no isolated cell. It retains explicit closed, positive, masked, paired-sky,
  implicit and unresolved-fail-open edge reasons.
- Conservative reachability visits 236 cells. The paired-sky-terminal falsifier
  visits 233, but both reach every target in the six-ray matrix and disagree
  with the ordered source oracle on all five rejected far-field specimens.
- The exact BVH independently confirms that each target is geometrically
  present along its ray. This is compatible with source-level rejection and
  does not turn geometry or topological reachability into participation
  authority.
- Only the hut-east wall's shortest conservative chain crosses paired sky.
  Wall 247 and the rejected ceilings remain reachable through ordinary or
  implicit paths, so paired-sky terminality is not a sufficient resolver.
- The negative result is deterministic at graph fingerprint
  `13500e039c076c04` and matrix fingerprint `1d5228cf89a8478b`. No declaration,
  renderer submission, stable contract or ownership boundary changed.
- Disposition: retain connectivity as diagnostic topology evidence only. It
  does not repair the open source-participation problem; live atomic install
  remains pending review of that problem.

### Cycle 39 -- 2026-08-18

- The authorized custom-BVH view-cell/aperture follow-up completed Slices 0–3
  as a corpus-private shadow. Renderer submission and stable contracts remain
  unchanged.
- The predecessor graph fingerprint remains `13500e039c076c04`. Its directed
  aperture sidecar contains 457 traversable and 150 non-traversable
  relationships, 33 zero-clearance relationships and zero boundary
  containment failures at fingerprint `3447a97c840c5a0f`.
- Actual-camera transfer retained multiple path-qualified clipped-view states.
  The six-ray matrix produced 632 states total and a peak of 306; state growth
  was bounded enough to test the semantic hypothesis.
- The required positive subsector 104 ceiling is an exact BVH/brute-force hit
  and is retained by the ordered source oracle, yet its ray crosses all three
  inferred physical boundaries above their runtime openings. Physical aperture
  transfer therefore omits a required Doom presentation occurrence.
- Across 2,175 relevant surfaces, transfer reached 782 and left 1,393 outside.
  Seventy-four retained surfaces outside transfer require source-ordered
  rescue, while 290 source-covered surfaces are inside reached cells.
  Reachability is consequently neither necessary nor sufficient.
- Variant C agrees with all six controls only by retaining the complete
  predecessor ordered-source oracle outside and inside the transferred domain.
  It does not materially localize presentation authority.
- Disposition: park the physical aperture hybrid as a Doom presentation
  resolver. Retain its topology and path-state mechanics only as diagnostics.
  Do not invent wider presentation apertures, sky geometry, renderer portal
  semantics or Tokimu ownership to work around the falsifier.
- Detailed evidence:
  `docs/Checkpoints/2026-08-18-doom-custom-bvh-view-transfer-shadow.md`.

### Cycle 40 -- 2026-08-18

- Maintainer direction: open a diagnostic study explaining why Doom's ordered
  protocol does not produce the retained far-field E1M1 contributions.
- The question is refined from whole-sector rejection to exact source
  occurrence causality. The study must name the first decisive BSP, SEG,
  wall-tier or plane event and the earlier source events that supplied its
  covering state.
- Existing `terminally rejected`, `source-covered`, `not reached` and `zero
  associations` results are treated as outcomes, not sufficient causal
  explanations.
- The paired subsector 104 ceiling views form the mandatory positive/negative
  comparison. Sky's role must be proved from exact ordered mutations and may
  validly be non-causal.
- Authorization is diagnostic-only: observation provenance and bounded
  counterfactual replay may be added inside the Doom corpus, but source
  decisions, renderer submission, stable contracts and presentation authority
  remain unchanged.
- Controlling study:
  `docs/Plans/DOOM/Studies/Doom source-ordered non-presentation causality study.md`.

### Cycle 41 -- 2026-08-18

- The first source-ordered non-presentation causal slice now reports the six
  retained exact rays with deterministic target and covering provenance.
- All five absent targets first disappear at near-first BSP child pruning:
  accumulated ordinary solid SEG ranges fully cover the target child's
  projected horizontal interval before the target wall or plane occurrence is
  eligible to be produced.
- Focused provenance resolves all five covering chains. A broad shadow replay
  with solid-range child pruning disabled reaches all five targets; this is
  class-level corroboration, not yet a one-event necessity proof.
- The paired subsector 104 control is decisive. The retained view reaches the
  subsector and produces three associations and one partial destination. In
  the rejected view, SEG 125 / linedef 37 supplies the final `[136,155]`
  coverage needed to skip node 101's target child, so plane eligibility and
  vertical clipping are never entered.
- None of the five focused covering chains contains a paired-sky SEG. Sky is
  therefore non-causal for these exact exclusions; `skybox leak` describes the
  visual symptom, while `source-invalid far-field resurrection` describes the
  evidenced defect class more precisely.
- The diagnostic provenance sidecar is observation-only, mirrors the existing
  solid-range union and leaves ordered results, conservation and renderer
  submission unchanged. No stable contract or presentation authority changed.
- Result checkpoint:
  `docs/Checkpoints/2026-08-18-doom-non-presentation-causality-slice1.md`.

### Cycle 42 -- 2026-08-18

- The exact counterfactual follow-up independently suppressed all 20 focused
  covering SEG mutations across the five absent E1M1 cases. Twelve
  interventions reopen at least part of the target domain, and every absent
  case has at least one individually necessary event for its original
  target-child prune.
- The result is not a permanent occluder classification. Eight focused events
  can be removed without reopening the target because later or overlapping
  coverage remains. The reusable source fact is the accumulated ordered solid
  coverage union; SEG identity is its causal provenance.
- A nearby positive wall control now traverses the same stages:
  `wall:135:SUPPORT2`, SEG 270 / linedef 135, reaches subsector 88, is admitted
  over projected interval `[150,184]` and produces two ordinary declarations.
- The evidence separates source-covered far-field resurrection from free-look
  plane realization. The former is decided before the target domain is
  traversed; the latter begins only after a source domain participates.
- Paired sky remains absent from all five focused covering chains. Existing
  positive sky controls remain final-presentation evidence, not authority for
  these exclusions.
- No renderer submission, stable contract or ownership boundary changed.
  Live source-domain exclusion remains gated on AR-0030 after E1M1 positive
  control correlation and direct reference-source cross-checking.
- Result checkpoint:
  `docs/Checkpoints/2026-08-18-doom-non-presentation-exact-counterfactuals.md`.

### Cycle 43 -- 2026-08-18

- Maintainer authorization added one presentation-affecting, corpus-private
  E1M1 strategy for visual evaluation: `source-covered-global-shell`.
- The candidate starts from the complete original prepared shell and replays
  Doom's ordinary near-first solid coverage at the actual horizontal camera
  pose. It suppresses only contributions whose resolved source ownership is
  exclusively in unreached subsectors.
- Participating floors and ceilings retain their whole original reconstructed
  geometry. Child/SEG proxy boxes and Classic screen spans have no rejection
  authority over those larger plane meshes. Walls are retained if any resolved
  owning subsector participates; unresolved ownership fails open.
- Live movement uses composition-local prepare-then-replace refresh. The
  renderer continues to receive ordinary declarations and owns no Doom, BSP,
  sky, portal or source-domain vocabulary.
- Seven exact headless controls pass: all five source-covered far-field
  specimens are absent, while the reached subsector 104 ceiling and nearby
  wall 135 / SUPPORT2 remain. Conservation is balanced for every preparation.
- A native two-frame spawn smoke run completed with 967 opaque and 24 cutout
  draws after the runtime-camera refresh, from 1,823 opaque and 26 cutout input
  draws. The reduction is experimental evidence, not a performance or
  correctness claim.
- The candidate remains non-default pending visual walkabout and dynamic
  door/platform freshness review. A failure returns evidence to this review;
  it does not grant Doom traversal concepts to the stable renderer boundary.
- Result checkpoint:
  `docs/Checkpoints/2026-08-18-doom-source-covered-walkabout-experiment.md`.

### Cycle 44 -- 2026-08-18

- Maintainer walkabout reports that `source-covered-global-shell` materially
  improves the hut area, but exact captures falsify it as a sufficient
  presentation policy.
- Three reconstructed ceilings are retained because their subsectors are
  reached even though the exact source sector/plane key has no occurrence in
  the frozen-view source replay. These are reached-domain false positives.
- A nearby sector 5 floor provides the complementary false negative. The
  filtered scene has no ray hit, while the complete shell hits the floor only
  132.480 source units away. Its subsector is skipped by an `outside-fov`
  source child proxy even though the exact sector/plane key has populated
  source spans.
- An alternate hut-area pose reports the suspect ceiling key present while
  suspicious whole-plane geometry remains. Replacing “subsector reached” with
  “plane key exists” would therefore be another invalid Boolean promotion:
  plane-key occurrence does not authorize every point of every correlated
  reconstructed polygon.
- Wall 241 is separately retained after its target subsector and SEG are
  reached/admitted, but the captured hit lies behind an earlier sky boundary.
  Horizontal admission is not final wall-fragment proof; exact final wall
  occurrence correlation remains required before changing wall policy.
- Architectural disposition: retain the strategy as a corpus A/B diagnostic,
  do not make it default, and do not silently tighten it. A successor must be
  reviewed as a Doom-private source-occurrence realization over actual
  reconstructed support, with no Doom, sky, BSP or span vocabulary entering
  the renderer.
- Falsifier checkpoint:
  `docs/Checkpoints/2026-08-18-doom-source-covered-walkabout-falsifiers.md`.

### Cycle 45 -- 2026-08-18

- Maintainer authorization opens a diagnostic-first successor study relating
  Doom's final source-keyed wall/plane cells to actual reconstructed geometry.
- The study explicitly tests exact plane-key plus source-sector cell support
  against the current same-subsector association. It does not grant a complete
  source key Boolean authority over a complete mesh.
- The five walkabout captures and prior seven controls are mandatory
  falsifiers. Wall 241 must be correlated with final vertical wall-cell
  support rather than horizontal BSP admission alone.
- Shadow slices may add Doom-private ray/cell diagnostics and ordinary clipped
  geometry observations. Renderer changes, stable API changes and live
  presentation installation remain unauthorized until the shadow result
  returns to this review.
- Controlling study:
  `docs/Plans/DOOM/Studies/Doom source-occurrence support over reconstructed geometry.md`.

### Cycle 46 -- 2026-08-18

- The source-occurrence support shadow maps frozen exact rays into final Doom
  wall/plane cells and clips reconstructed planes by exact plane key, source
  sector and finite cell support. The five new capture expectations pass
  `5/5`; all four plane capture rays agree with clipped geometry.
- Wall 241 is absent from the final sampled middle-tier cell and has zero
  ordered declarations. Wall 135 remains supported. This resolves the wall
  comparison without making sky a generic occluder.
- The historical `ceiling-104-reached` specimen was found to be positive only
  at whole-object occurrence granularity. Its exact BVH ray maps to source
  cell `(160,62)`, where the matching plane-key instance has no interval. The
  existing ordered declarations and the new geometry shadow both miss that
  exact ray.
- This corrects, rather than overturns, the earlier causal result: subsector
  104 is reached and contributes a partial ceiling somewhere in the view, but
  the selected BVH point was never proved source-present.
- Broadening association from exact key plus source sector to plane key alone
  did not restore the ray and increased representative fragment output from
  `3,966` to `5,497` and `3,325` to `5,528`; the broadening was removed.
- The shadow remains non-presentational and the exact-cell gate deliberately
  reports `6/7`. Presentation installation is paused pending an independently
  justified positive plane oracle and a decision on fixed Classic rows under
  arbitrary pitch.
- Result checkpoint:
  `docs/Checkpoints/2026-08-18-doom-source-occurrence-support-shadow.md`.

### Cycle 47 -- 2026-08-18

- Maintainer review preserves the `ceiling-104-reached` exact-cell failure and
  clarifies the next gate: independently justified neutral-pitch positive
  planes must agree across complete geometry, final cells, ordered
  declarations and reconstructed support before pitch lifting is considered.
- A bounded four-pose `32x20` neutral search found `481` four-way agreements
  across `12` distinct pose/plane/sector/subsector identities. Three frozen
  controls cover one ceiling, two floors, two poses, two sectors and three
  subsectors.
- Every frozen control has a complete-shell hit, a retained exact source cell,
  an ordered-declaration exact hit and an exact-key/source-sector geometry
  shadow hit at the same distance within `0.01`.
- The neutral-authorized finite world fragments were then queried through
  cameras pitched `-15`, `0` and `+15` degrees. All nine pitched projections
  reconstruct and intersect the same fragments at the same distance. Classic
  rows participate only in neutral source preparation; they are not treated
  as persistent world-space boundaries.
- Discovery fingerprint: `129127dbc170cb2b`. Frozen-control fingerprint:
  `e5cb5acbddd8406d`. No renderer mutation or stable contract change occurred.
- Architectural disposition: the immediate neutral-positive and bounded
  pitch-lift gates pass in shadow form. Return before installing a live
  presentation strategy; the next decision is whether this evidence is enough
  for another corpus-private A/B walkabout or whether a broader pose/pitch
  matrix is required first.

### Cycle 48 -- 2026-08-18

- Maintainer review authorizes an opt-in corpus-private live A/B realization
  after the `481` neutral four-way agreements, three frozen exact positive
  planes, and nine successful fixed-world-geometry pitch queries. It does not
  authorize a default strategy, stable contract, or renderer vocabulary.
- `--render-strategy=source-occurrence-supported` now combines final ordered
  wall declarations with reconstructed plane fragments clipped by exact plane
  key, source sector and retained source cells. The complete current-runtime
  preparation must pass wall, plane and declaration conservation before the
  composition replaces its prepared declarations.
- Actual Tokimu pitch reprojects those finite ordinary world fragments and is
  not part of the source-cell preparation identity. Position, yaw, eye height,
  door ceiling and moving-floor snapshots remain preparation inputs owned by
  the corpus application.
- The dedicated headless acceptance report passes `15/15` twice with
  fingerprint `b578061ac0312dce`. It retains the old subsector-104 result as an
  object-occurrence positive but exact-ray negative rather than weakening the
  historical `6/7` gate.
- A native Vulkan two-frame spawn smoke completed with `360` opaque and `12`
  cutout candidate declarations after the runtime-camera refresh. Plane
  preparation emitted `7,702` triangles (`7.940x` amplification), all
  conservation checks passed, and unresolved counts remained zero. First and
  warm CPU frame times were approximately `84.9` and `16.1` ms respectively;
  they are observations, not budgets.
- Architectural disposition: begin the adversarial E1M1 walkabout with
  `global-full-submission` retained as the explicit control. Visual holes or
  leaks return exact replay evidence to this review; they do not authorize
  source-cell, BSP, sky, or portal semantics in `tokimu-render`.

### Cycle 49 -- 2026-08-18

- The first maintainer walkabout falsifies the
  `source-occurrence-supported` candidate as a coherent live presentation:
  it is visually worse than the global control and exposes many holes.
- The `15/15` exact matrix remains valid for its local claims, but is now
  explicitly insufficient as a whole-view acceptance gate. Conservation also
  remains balanced and therefore demonstrates accounting rather than
  watertight screen coverage.
- A new non-authoritative four-pose `32x20` complete-shell comparison records
  seven complete-shell nearest hits with no candidate hit and two nearest-hit
  displacements over `2,560` rays. Broad-grid fingerprint:
  `d049a8b79f7404c8`. This bounded matrix does not claim to reproduce every
  hole seen on the maintainer's wider walk path.
- Architectural disposition: the strategy remains opt-in diagnostic evidence
  and is rejected for default promotion. Do not broaden source-cell admission
  to make the image whole. Retain exact failing camera/pixel replays next and
  distinguish lowering/fragment coverage defects from limits of lifting
  Classic view-conditioned occurrences into ordinary world geometry before
  deciding whether this line of study continues.
- Five retained live `LOOK` rays subsequently prove that the visible panorama
  is showing through omitted ordinary geometry rather than an authorized sky
  aperture. Global-full hits one ceiling and four floors at distances from
  `56.203` through `138.783`; all five rays have neither a source sky plane nor
  a sky boundary.
- Two spawn rays lie near `+/-40` degrees elevation, outside the Classic
  source projection's roughly `+/-32` degree vertical window. Finite neutral
  support cannot supply their nearby floor/ceiling under free pitch. Three
  additional missing floors remain within the source window, proving that
  pitch range alone does not explain the candidate's omissions.
- The live `LOOK` diagnostic now retains and prints the global-full nearest
  hit when this candidate misses, then uses it as the source trace/plane
  occurrence target. Candidate removal can no longer make the diagnostic
  misleadingly report an empty target set.

### Cycle 50 -- 2026-08-18

- Maintainer review separates Classic source participation from complete
  arbitrary-camera world presentation and recommends retiring source cells as
  the latter's authority. Global-full remains the geometry-completeness oracle
  while ordered evidence may later supply only narrowly scoped, positively
  proven exclusions.
- The final bounded audit covers the three moderate-pitch floor holes. All
  three exact pixels have populated matching plane-key cells, yet none has a
  prepared hit. Sector 38/subsector 114 has two matching ordered declarations;
  sector 2/subsector 116 and sector 12/subsector 29 have zero.
- This result exposes an authority limit, not merely a missing Boolean. Doom's
  retained plane cell key contains kind, height, texture and light but not
  source sector. Cell presence proves merged visplane support at a source
  pixel, not ownership by one exact reconstructed sector/subsector surface.
- Architectural disposition: abandon Classic source-cell support as the
  complete free-look representation. Preserve it as Classic-view, positive
  merged-plane, partial-occurrence and diagnostic evidence. Cell absence may
  not reject arbitrary-camera geometry; cell presence requires independent
  source identity before it can authorize an exact world contribution.
- Any successor must begin with persistent Doom-private geometry complete for
  arbitrary Tokimu cameras, then shadow narrowly scoped positive causal
  exclusions against global-full. The five original far-field exclusions are
  retained falsifiers, not permission to install that policy. Hardware-port
  render-subsector/render-sector and explicit compatibility-hack practice may
  inform a new study, but no stable or live strategy is authorized here.

### Cycle 51 -- 2026-08-18

- Maintainer discussion proposes a new free-look realization hypothesis over
  complete global-full geometry: ordered, oriented semantic sky transitions
  may place each actual camera ray in alternating World/Sky state before its
  nearest ordinary target.
- The proposal is stricter than raw intersection parity. Doom does not provide
  a guaranteed watertight skybox, so only source-proven `Enter` and `Exit`
  events may change state. Duplicate triangles, tangencies, malformed
  sequences, unknown initial state and unproved closure remain explicit and
  fail open to World.
- AABB/frustum/BVH mechanisms may accelerate conservative candidate and exact
  intersection queries only. They do not infer transition roles or acquire
  presentation authority.
- The frozen first gate contrasts the five original far-field resurrection
  rays, expected to be Sky before the unwanted hit, with the five newly
  retained ordinary floor/ceiling holes, expected to remain World. Classic
  ordered solid coverage remains the historical cause; parity is tested only
  as an implementation-equivalence hypothesis.
- The proposed study requires a genuine Enter/Exit specimen before parity can
  claim value beyond a one-way sky mask. It authorizes no implementation,
  live candidate, renderer mutation or stable contract by itself.
- Monday review recommends shadow-only Slices 0–2 and emphasizes that the
  boundary/closure audit may terminate the study before parity execution.
  Slice 3 remains conditional on a strict `10/10` result and is not
  pre-authorized by that recommendation. A maintainer start instruction is
  still required.
- Proposed study:
  `docs/Plans/DOOM/Studies/Doom oriented sky-transition parity shadow.md`.

### Cycle 52 -- 2026-08-18

- Maintainer authorization opened shadow-only Slices 0–2 with the binding
  instruction to stop before parity if Slice 1 could not prove semantic
  `Enter` and `Exit` events.
- Slice 0 freezes five far-field expected-Sky rays and five newly retained
  required-World floor/ceiling rays. Global-full targets remain unchanged and
  renderer mutation is false.
- The Slice 1 inventory contains 16 paired-sky triangles in eight linedef
  groups and 73 source-sky ceiling triangles in 35 source groups. All eight
  paired groups have `F_SKY1` ceilings on both sides; zero separates World from
  Sky.
- Combined topology has 199 unique edges, of which 131 are open and 68 are
  manifold; no non-manifold edge was observed. Source-sky ceiling planes are
  locally oriented open caps but establish neither closed-domain initial state
  nor an Exit.
- The semantic inventory consequently proves zero Enter and zero Exit events.
  Slice 2 parity was not executed, exactly as required by the gate. The
  conservative World result matches the five required-world controls only.
- Every far-field ray nevertheless has one raw sky-related hit before its
  unwanted target, while every required-world control has none. Raw any-hit
  correlation is `10/10`, but is retained explicitly as correlation-only
  one-way-mask evidence, not parity, transition authority, or Classic causal
  explanation.
- Conservation is balanced and two runs produce fingerprint
  `864b11fc73f28f2c`. Real oriented parity is parked. A bounded one-way sky-hit
  mask would require a new review and adversarial controls before any live or
  renderer-affecting work.

### Cycle 53 -- 2026-08-18

- Maintainer authorization opened a bounded **one-way sky-occlusion
  correlation shadow**, explicitly not a live mask, parity model or omission
  rule. The first adversary was valid exact source geometry behind a preceding
  sky-related hit.
- The report sampled 5,120 rays across eight retained E1M1 trouble-area poses;
  4,380 rays hit complete ordinary geometry and 36 had at least one
  sky-related surface first.
- Of those 36, 28 ended at a partial/absent ordered source target, but eight
  ended at exact final ordered source geometry. Six retain walls 159/160 from
  the hut-east pose behind paired-sky linedefs 250/254; two retain wall 203
  from the far-left pose behind paired-sky linedef 250.
- These are the binding `sky-before + exact-present` falsifiers. A first
  sky-related hit cannot hide all farther geometry: the same kind of
  observation precedes both source-absent and source-present targets.
- The predecessor's ten controls remain stable, historical exclusion remains
  attributed to ordered solid coverage, conservation is balanced, and two
  runs produce fingerprint `b3435e035db5ab1d`.
- Blanket one-way masking is parked before live work, BVH work or dynamic
  expansion. Sky-before correlation may remain a diagnostic trigger for
  independent ordered-source scrutiny, but has no omission authority.

### Cycle 54 -- 2026-08-18

- After closing sky-derived rejection, the next corpus-private A/B isolates
  the contribution family with exact final provenance: ordered wall fragments
  over untouched global-full planes.
- The preparer consumes the actual view/runtime map snapshot, lowers only
  final ordered wall declarations, and retains all 853 global-full plane
  declarations identically. Any unresolved wall preparation fails open to the
  complete global opaque/cutout sets before atomic replacement.
- The exact headless gate passes `5/5`: wall 241 remains absent; wall 135
  remains present; and walls 159, 160 and 203 from the sky-correlation
  falsifiers remain present at their exact rays.
- Two runs produce fingerprint `cb3ed7d517e1b942`; family conservation is
  balanced and the renderer continues to receive ordinary declarations only.
- The opt-in `final-wall-occurrence-global-planes` strategy is ready for human
  walkabout. It is evidence about wall-family preparation only and grants no
  plane, sky or provider-neutral visibility decision.

### Cycle 55 -- 2026-08-18

- Review of the one-way report corrected its scope: the eight
  `sky-before + exact-present` cases each had two grouped hits, so they did not
  falsify a separate grouped-crossing parity hypothesis.
- A renderer-neutral reclassification reproduced the exact retained 36-ray
  corpus and preserved every ordered group, family, distance, raw winding and
  locally provable source-ceiling side.
- The matrix is strongly correlated but not complete: `26` odd/absent, `8`
  even/exact-present, `0` odd/exact-present, and `2` even/absent.
- Both adverse rays use the same paired-boundary then source-sky-plane family
  sequence as six successful hut rays. All eight source-plane hits are from
  the ceiling backside; paired-sky orientation remains unresolved. Parity,
  family sequence and locally proved ceiling side therefore do not distinguish
  the results.
- Two runs produced fingerprint `26afe710bce75ebc`; conservation is balanced
  and renderer mutation is false. The full 5,120-ray expansion and live A/B
  are parked before inventing a new semantic discriminator.

### Cycle 56 -- 2026-08-18

- The maintainer authorized a reversible live visual falsification despite the
  failed grouped-parity/source-occurrence correlation gate. This authorization
  does not promote parity to Doom source truth.
- The experiment begins with the complete ordinary world, writes its nearest
  opaque/cutout coverage into depth, inverts one stencil bit for every
  double-sided paired-skywall fragment before that depth, and renders ordinary
  color only where parity remains even. Source sky ceilings do not toggle it.
- The required renderer mechanism is admitted generically by ADR-0014 as
  `Disabled`, `InvertOnDepthPass`, and `RequireZero`; no Doom, sky, portal, BSP,
  or volume vocabulary enters `tokimu-render`.
- A native two-frame Vulkan proof completed with 1,823 opaque draws, 14 facing
  cutout draws, 16 paired-skywall triangles, and 3,693 total draw calls. Warm
  command construction was `224 µs`; warm frame CPU time was `104,483 µs`.
- The opt-in control is `--skywall-parity-full`. The ordinary no-flag
  global-full path remains the comparison. Human walkabout remains the
  acceptance/falsification step.

### Cycle 57 -- 2026-08-18

- The first live skywall-only walkabout produced an exact required-world
  falsifier: admitted wall 205 was masked after one paired-skywall crossing,
  while an earlier source `F_SKY1` ceiling crossing was retained in the same
  ray evidence but excluded from the live parity count.
- The ordered ray is source sky ceiling subsector 48 at distance `139.203`,
  paired skywall linedef 253 at `170.985`, then ordinary wall 205 at `201.929`.
  Classic source tracing reaches and admits the wall.
- Maintainer authorization expands the reversible experiment to toggle the
  same low stencil bit for either source family. The corrected replay has two
  crossings, even parity, and retains wall 205.
- The renderer capability remains unchanged and provider-neutral. Source sky
  identity, grouping, and the decision to submit 73 plane triangles remain in
  the Doom corpus composition.
- A native two-frame Vulkan proof completed with 3,766 draw calls. Warm
  command construction was `325 µs`; warm frame CPU time was `87,929 µs`.
  Human walkabout remains the acceptance/falsification step, and the prior
  two even/absent grouped-correlation counterexamples remain unresolved.

## References

- `docs/contribution-admission-guide.md`
- `docs/Tokimu Software Design Document.md`
- `docs/Architectural Reviews/AR-0023-textured-surface-alpha-and-depth-policy.md`
- `docs/Architectural Reviews/AR-0025-camera-candidate-selection-and-visibility-culling.md`
- `docs/Architectural Reviews/AR-0026-non-euclidean-spatial-charts-and-authored-angular-topology.md`
- `docs/Plans/DOOM/Studies/Doom viewer-relative presentation synthetic conformance.md`
- `docs/Plans/DOOM/Studies/Doom source-topology admission over complete geometry.md`
- `docs/Plans/DOOM/Studies/Doom ordered source occurrence preparation.md`
- `docs/Plans/DOOM/Tokimu BSP capability setup plan.md`
- `docs/Plans/DOOM/Studies/Doom authoritative sky coverage delta realization.md`
- `docs/Plans/DOOM/Studies/Doom source-authorized relational contribution classification.md`
- `docs/Plans/DOOM/Studies/Doom oriented sky-transition parity shadow.md`
- `docs/Plans/DOOM/Studies/Doom grouped sky-crossing parity shadow.md`
- `docs/ADR/ADR-0014-single-bit-stencil-mask-pipeline-state.md`
- `docs/Plans/DOOM/Evidence/Classic Doom visibility clipping evidence.md`
- `docs/Plans/DOOM/Evidence/Classic Doom renderer dataflow and Tokimu preparation seam.md`
- `docs/Plans/DOOM/Evidence/Doom authoritative sky-depth realization seam evidence.md`
- `docs/Plans/DOOM/Evidence/Doom relational classifier four-case capture ledger.md`
- `docs/lessions/read-reference-source-early.md`
- `docs/lessions/bounds-authority-follows-bounded-representation.md`
