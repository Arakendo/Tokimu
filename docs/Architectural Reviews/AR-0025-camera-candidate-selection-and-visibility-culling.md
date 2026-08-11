# AR-0025: Comparative Camera Candidate-Selection and Visibility Study

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-08-10 |
| Last reviewed | 2026-08-10 |
| Scope | Corpus presentation, source-specific spatial selection, renderer-facing scene preparation, and performance diagnostics |
| Trigger | The interactive E1M1 source-spawn observer now resubmits all 1,861 static draws for every camera update. Startup mesh upload was repaired, but view-dependent candidate selection remains unstudied. |
| Related ADRs | ADR-0003, ADR-0007, ADR-0008, ADR-0009, ADR-0012, ADR-0013 |
| Related reviews | AR-0021, AR-0023, AR-0024, AR-0026 |
| Related evidence | `hello-doom-e1m1` source-spawn observer; E1M1 static presentation evidence; native Vulkan/AMD manual observation |
| Admission exception | None |

## Architectural Question

Which candidate-selection methods materially help the current E1M1 observer,
what facts does each method require, and only then: has any provider-neutral
Tokimu capability earned admission? Which methods must remain source-specific
or provider-specific regardless of measured benefit?

## Admission Clamp

The study may use Doom-specific structures and provider-specific mechanisms as
comparative evidence, but neither may become Tokimu's selected answer merely
because it wins on E1M1. Any proposed Tokimu capability must be usable as a
general provider-neutral service to independently prepared scenes:

- it consumes declared, source-neutral inputs rather than WAD/BSP/portal terms;
- it preserves caller ownership of scene membership, ordering, and source truth;
- it has explicit correctness, diagnostic, and failure-containment behavior;
- it can have more than one provider realization without exposing provider
  handles or implementation-shaped vocabulary; and
- it has independent caller evidence before admission.

A stage may therefore conclude that a technique is valuable only as a
source-specific application optimization, valuable only as provider research,
or unsuitable for Tokimu. Those are successful findings, not failed attempts
to force an engine capability.

The study also permits bounded theoretical trials. Each must declare its
source-neutral hypothesis, inputs, claimed guarantee, counterexample/failure
condition, measurement protocol, and reason it remains corpus-local. An
interesting mechanism without those facts is not a candidate answer.

## Provisional Default and Selection Principle

**Current default: explicit caller-owned full submission.** It is the only
contract presently admitted and remains the fallback whenever a selection
method is unavailable, uncertain, stale, or fails validation.

**Study default candidate: conservative CPU frustum classification over
declared world-space AABBs.** It is the smallest prospective general provider
because it needs only caller-declared bounds and a camera, preserves order, and
can safely fall back to full submission. It is not admitted by this label; the
study must still establish independent utility, performance, diagnostics, and
failure containment.

Tokimu need not select one universal mechanism. A future provider-neutral
surface may support a small, explicit family of conservative filters when each
has a distinct workload case and the default/fallback remains unambiguous. The
selection criterion is not "most modern" or "best on Doom"; it is the smallest
source-neutral method that makes a measurable, correct difference for the
declared workload.

## Context

The E1M1 static scene prepares 1,835 opaque draws plus 26 admitted categorical
cutout draws. The source-spawn observer initially re-uploaded every immutable
mesh on every camera update; that local performance defect is repaired by
uploading static geometry once at startup. Camera motion still submits every
prepared draw because the current renderer consumes an explicit caller-provided
command list and owns no scene graph, map topology, or visibility authority.

The visible black regions in current E1M1 observation remain separate bounded
presentation limits: sky drawing and original Doom plane-span/visibility
reconstruction are not claimed by the static scene. This review must not use
"culling" to hide unsupported source surfaces, missing source semantics, or
renderer defects.

## Trigger and Retained Evidence

- The available native Vulkan/AMD source-spawn observation visibly presents
  1,861 draws under mouse look.
- Static mesh buffers now upload once during startup. Subsequent observer
  frames change the camera and submit draws without replacing static GPU mesh
  resources.
- The scene still supplies all prepared candidates on every frame. No frustum,
  BSP, portal, `REJECT`, `BLOCKMAP`, depth-pyramid, or occlusion-query culling
  is currently claimed.
- E1M1 retains a bounded Doom BSP and source-sector ownership path. That is
  source interpretation evidence, not generic renderer visibility semantics.
- AR-0024 established that accepted commands and successful presentation do not
  by themselves prove expected pixels; any culling experiment must retain
  candidate, rejected, submitted, and visible-observation facts separately.

## Ownership Analysis

- A corpus/application owns scene membership, source-format topology, Doom BSP
  interpretation, source-specific `REJECT` data, and any rule that decides
  which source objects are candidates for a camera.
- A provider-neutral renderer owns realization of explicit mesh/material/camera
  declarations. It must not acquire WAD terms, mutable map truth, or hidden
  application scene ownership merely to skip draws.
- A possible Tokimu capability, if earned, would own only a generic declared
  candidate-selection contract and bounded observability. It must not infer
  visibility from asset names, invent an application scene graph, or promise
  occlusion correctness beyond its stated evidence.
- A WGPU backend may realize provider-native frustum or GPU culling only behind
  an admitted provider-neutral contract. Backend query handles and WGPU types
  must not become caller vocabulary.

## Dependency Direction

```text
Current:
E1M1 source/map preparation -> corpus draw list -> tokimu-render -> WGPU

Possible bounded direction:
application/corpus candidate declaration + camera
    -> admitted Tokimu candidate-selection capability (only if earned)
    -> renderer/provider realization

Not proposed:
WAD BSP/REJECT -> tokimu-render
renderer hidden scene graph -> simulation/source truth
```

## Comparative Study Design

This record is a comparative corpus study, not a commitment to a visibility
subsystem. Every trial receives the same prepared scene, camera poses, target
metadata, timing protocol, and retained visual-observation procedure. A trial
may filter the caller's input sequence but may not reorder it.

The study treats candidate count and frame time as independent facts. A lower
candidate count is useful evidence, but does not establish a performance gain;
an observed visible omission invalidates a conservative trial immediately.

| Stage | Method | Required facts | Ownership | Initial status |
| --- | --- | --- | --- | --- |
| 0 | Explicit full submission | Prepared ordered draw list | Existing caller contract | Baseline |
| 1 | CPU frustum/AABB | Camera and derived world-space AABBs | Corpus-local generic experiment | First implementation |
| 2 | CPU broad phase (BVH or uniform grid) | Stage-1 bounds plus a maintained spatial index | Corpus-local generic experiment | Only if Stage 1 selection cost is material |
| 3 | Doom BSP and, separately, portal/sector traversal | WAD topology and source viewer position | Comparative oracle only; never a direct Tokimu answer | After Stage 1 baseline |
| 4 | GPU depth pyramid, occlusion query, or indirect selection | Provider-native depth/query/resource lifecycle | Provider-specific research; no capability claim | Deferred; requires separate review evidence |
| T | Bounded theoretical trial | Declared hypothesis and falsification protocol | Corpus-local until all admission conditions are earned | As a targeted comparison needs it |

`REJECT` and `BLOCKMAP` are not stages in this sequence. They may be retained
as Doom-specific comparison inputs during Stage 3 only after their actual
historical semantics, completeness, and failure behavior are documented.

### Theory Candidates and General Workload Cases

These are experiments, not pre-approved APIs. They intentionally cover more
than game maps so the study can distinguish a reusable provider from a
Doom-shaped optimization.

| Theory | General, non-game pressure | Expected strength | Required falsification / guardrail |
| --- | --- | --- | --- |
| AABB versus bounding sphere | CAD assemblies, industrial cells, editor scene views | Tests cheap conservative bounds and the cost/precision tradeoff | Never reject a visibly intersecting object; retain bounds-test cost and false-positive count |
| Candidate granularity | Dense CAD part trees, dashboards with repeated markers, tiled simulations | Tests whether per-object, group, or aggregate bounds are the useful unit | Preserve source identity and caller order; do not let batching redefine scene ownership |
| BVH versus uniform grid | Large static scans, robotics environments, spatial editors | Tests whether broad-phase selection reduces CPU work before submission | Static/dynamic invalidation and memory costs must be measured; no hidden global scene graph |
| Temporal coherence / hysteresis frustum | Smooth inspection cameras, map navigation, XR-like head motion | Tests whether prior classifications reduce work or boundary churn | Teleport and abrupt-turn traces must fall back safely; prior-frame results never become truth |
| Doom BSP / portal / sector traversal | Comparison oracle for applications with explicit region topology, such as facilities or rooms | Tests the incremental value of application-owned topology | Remains application/source-owned unless a separate source-neutral region contract earns review |
| Offline potentially-visible sets | Static architectural walkthroughs, factory digital twins, fixed-installation visualizations | Tests memory-for-runtime selection tradeoff | Offline data must be versioned, conservative, and invalidated on scene changes; never infer it from a renderer |
| Occluder eligibility classification | CAD opaque shells, industrial layouts, editor previews | Tests the semantic precondition for later occlusion, including opaque/cutout/blend distinction | No texture-alpha inspection in frustum selection; cutout and Blend cannot be assumed opaque blockers |
| Software occlusion buffer | Deterministic/headless research and constrained devices | Tests coarse occlusion without GPU query lifecycle | Must prove conservative coverage and retain false-negative evidence; not a renderer feature by default |
| GPU HZB/query/indirect selection | Very large visualization or simulation scenes with a capable graphics provider | Tests provider-specific reduction after CPU baselines exist | Requires separate resource/scheduling/latency review; no provider handles in caller vocabulary |

### Trace and Fixture Protocol

The same algorithms must run over retained camera traces rather than only
interactive observation. The initial E1M1 set should include spawn-to-hallway,
courtyard/large-room view, doorway approach, narrow corridor, rapid 180-degree
turn, and a teleport. Each trace records per-frame bounds tests, hierarchy
nodes or source regions visited where applicable, candidates, rejections,
submissions, selection time, renderer submission time, total frame time, and
false-negative findings.

Add one deliberately pathological source-neutral fixture before drawing general
conclusions: many objects behind the camera, outside a single frustum plane,
spatially clustered, and deeply overlapping. E1M1 supplies real-world pressure;
the fixture isolates why a method succeeds or fails.

### Future Falsification Pressure: Portals and Non-Euclidean Presentation

Future corpus work is expected to include portal-transformed, recursive, and
non-Euclidean layouts. They are deliberately out of scope for this review's
initial implementation, but they constrain its conclusions now: a result such
as `camera + world-space AABB` must never be described as universal visibility
truth.

A portal may create a derived view with its own camera transform, clip region,
candidate domain, recursion/dependency identity, and potentially a different
spatial relation to the same source object. A globally culled object can be
visible through that derived view. Any future general provider must therefore
allow callers to declare multiple view instances and candidate domains without
requiring the renderer to own portal topology, non-Euclidean semantics, or
application scene truth.

This is future falsification pressure only. It does not admit portal vocabulary,
recursive rendering, a scene graph, transformed picking, or a multi-view public
contract. It records why the current frustum trial must remain a conservative
filter for one declared view, with full submission as its fallback.

AR-0026 now owns that future spatial-semantics study and may reuse a bounded
E1M1 topology subset. AR-0025 owns only the candidate-selection comparison and
the falsification case: a chart-local object outside the primary global
frustum may still be a valid candidate for a caller-declared derived view.

### Shared Measurement and Correctness Protocol

For every stage, retain at least two fixed camera poses and report:

- candidate, rejected, and submitted draws;
- first and warm frame time, plus mesh uploads/replacements, material
  resolutions, and pipeline switches;
- fixed scene/package/build/adapter metadata and manual native/browser visual
  observations where feasible; and
- source/draw identity and bounded rejection reason for each rejected
  candidate when the method can supply one.

False positives are tolerated and measured. A false negative—visible geometry
rejected by a conservative stage—is a correctness finding, not an acceptable
optimization tradeoff. Current black regions that arise from unsupported sky or
plane reconstruction cannot be classified as culling evidence.

## Methods Considered

### A. Retain Explicit Full Submission

- Benefits: current ownership is simple, transparent, and proven; no new
  renderer contract or hidden culling errors.
- Costs: cost grows with prepared scene size and camera movement submits work
  that is plainly outside the camera frustum.
- Failure mode: applications duplicate ad-hoc selection logic without retained
  diagnostics or target comparison.

### B. CPU Frustum Candidate Selection with World-Space AABBs

- Benefits: tests whether ordinary mesh bounds plus a camera are enough to
  reduce E1M1 submission without changing `tokimu-render` ownership.
- Costs: requires a conservative bounds policy, instrumentation, and proof
  that source-visible surfaces are not incorrectly removed.
- Failure mode: a Doom-specific or one-off helper is prematurely promoted as a
  universal renderer API.

### C. CPU Spatial Broad Phase: BVH or Uniform Grid

- Benefits: tests whether selection itself needs acceleration after ordinary
  frustum/AABB filtering; both approaches remain independent of Doom topology.
- Costs: introduces index construction, invalidation, and diagnostic questions;
  static E1M1 may not provide enough pressure to justify either structure.
- Failure mode: an index turns into an unearned shared scene ownership model.

### D. Doom BSP, Portal, or Sector Candidate Selection

- Benefits: compares generic bounds selection with source topology that can
  describe connected regions and viewer-relative partitions.
- Costs: each method has Doom-specific assumptions and must not become generic
  renderer vocabulary.
- Failure mode: source terms leak across the corpus/renderer boundary or a
  historically specialized structure is overstated as modern visibility truth.

### E. GPU Occlusion Queries, Hierarchical Depth, or Hardware-Specific Culling

- Benefits: potentially stronger reduction for large scenes.
- Costs: asynchronous visibility, temporal behavior, platform variance,
  diagnostic complexity, and likely new resource/scheduling contracts.
- Failure mode: provider mechanics become a premature public capability or
  valid-but-late results are mistaken for authoritative scene truth.

### F. Admit Generic Renderer/Scene Culling

- Benefits: could centralize a repeatedly demonstrated provider-neutral
  candidate/bounds contract and diagnostics across independent consumers.
- Costs: raises public vocabulary, cache/state, performance, and failure
  containment obligations; may imply scene scheduling/ownership that Tokimu has
  not admitted.
- Failure mode: renderer-owned opaque sorting or culling obscures application
  ordering, cutout, and future Blend responsibilities.

Admission requires all conditions in the **Admission Clamp**, not just a
successful E1M1 result. In particular, Doom BSP/portal/sector types and WGPU
occlusion-query/depth-pyramid handles are not eligible public vocabulary.

## Initial Findings

1. Static mesh upload replacement was an ordinary corpus performance defect,
   not evidence for a culling architecture.
2. E1M1 has one real camera-pressure source, but no independent caller yet.
3. Conservative CPU frustum selection is the smallest next experiment because
   it can remain corpus-local and report exact candidate/rejected/submitted
   counts without changing renderer meaning.
4. Doom BSP, `REJECT`, and `BLOCKMAP` are inputs to a Doom-specific experiment,
   not candidates for direct renderer admission.
5. Blend remains incubating under AR-0023; no candidate-selection work may
   reorder or otherwise conceal its caller-owned ordering requirements.
6. For conservative candidate selection, a false positive is measurable
   inefficiency; a false negative is a correctness finding. The experiment must
   optimize only under that asymmetric rule.
7. The first E1M1 Stage-1 observation demonstrates meaningful candidate
   pressure: the fixed source-spawn pose retained 495 of 1,861 draws while the
   full-map overview retained all 1,861. This is one native observation, not an
   admission or universal performance conclusion.

## Disposition

**Under Review.** Keep full explicit submission as the current renderer
contract. Run the staged comparison in order, beginning with a bounded
corpus-local frustum/AABB experiment. The study may compare source-specific
Doom spatial selection after the generic baseline; it may investigate GPU
methods only as provider-specific research. The only admissible outcome is a
source-neutral, provider-neutral capability that satisfies the Admission Clamp
and has independent caller evidence. Do not add a public culling API, scene
graph, Doom visibility coupling, or provider-native occlusion contract from
this record's current evidence.

## Required Follow-Up

- [x] Stage 0: retain first/warm observer-frame measurements for submitted
      draws, mesh uploads/replacements, material resolutions, pipeline
      switches, and frame time without selection.
- [x] Stage 1: define corpus-local world-space AABB evidence from prepared mesh
      positions, distinct from source identity; implement an order-preserving
      CPU frustum filter without changing `tokimu-render` vocabulary.
- [x] Stage 1: retain deterministic candidate/rejected/submitted counts for the
      fixed overview and source-spawn poses, including bounded rejection-plane
      and source-label samples. The overview retains 1,861/1,861; source spawn
      retains 495/1,861 with zero uncertain bounds.
- [x] Stage 1: retain paired native AMD/Vulkan first/warm measurements for full
      submission and frustum/AABB selection without steady-state mesh uploads
      or replacements.
- [x] Stage 1: retain fixed cardinal spawn reports. The yaw-plus-90 pose retains
      1,025 opaque and all 26 cutout draws, providing a deterministic target for
      selected cutout observation.
- [x] Stage 1: retain browser/WASM selected-cutout count/presentation evidence.
      An explicit local package selection reported 1,051 submitted from 1,861
      candidates (1,025 opaque plus all 26 cutouts), exactly matching the
      retained native fixed-pose count. This is manual presentation evidence,
      not timing or pixel-equivalence evidence.
- [ ] Stage 1: retain manual side-by-side native full-submission/selected visual
      observations before claiming that the conservative filter caused no
      visible false negatives.
- [x] Stage 1 theory subtrial: compare per-draw AABB against per-draw enclosing
      sphere across the fixed reports. Spheres were cheaper in one debug run
      but less selective; retain the tradeoff as corpus evidence only.
- [x] Stage 1 theory subtrial: compare per-draw AABBs against contiguous groups
      of 8 and 32 draws. Grouping reduces bound tests/CPU but is less selective;
      it remains corpus-only and does not create batch or scene vocabulary.
- [x] Stage 1: retain a deterministic E1M1 source-spawn in-place turn trace and
      a pathological source-neutral fixture. The turn trace proves fixed-yaw
      closure; the fixture proves mixed contiguous groups safely fail open.
- [x] Stage 1: retain a declared source-relative position trace for the
      per-draw AABB baseline. It is explicitly camera-input evidence, not a
      collision/traversal or player-policy claim.
- [x] Stage 1 theory matrix: repeat bound-shape and granularity trials over the
      declared source-position trace. Every sampled offset preserved the same
      cheaper-but-less-selective ordering; per-draw AABB remains the study
      default for the actual workload.
- [x] Gate Stage 2: retained Stage-1 traces show roughly 2 ms per-draw AABB
      selection in this development-profile workload; open one bounded static
      uniform-grid comparison without admitting a shared scene/index contract.
- [x] Stage 2: vary grid resolution and compare the static grid against the
      Stage-1 per-draw baseline across traces, retaining build cost, index
      storage, cell tests, exact tests, and submitted draws. Every retained
      final count matched the baseline; see the camera evidence record.
- [ ] Stage 2: retain a manual native visual comparison of the fixed medium-grid
      playback mode against the already-retained per-draw/full-submission
      evidence before making any no-visible-omission claim for the grid.
- [x] Temporal subtrial: retain one-frame candidate-carry overlap over smooth
      yaw, abrupt-turn, and declared-teleport poses while keeping fresh AABB
      classification authoritative. The carry is highly over-inclusive after an
      abrupt turn/teleport and is not a useful replacement for fresh selection.
- [x] Expanded-frustum subtrial: compare a 72-degree current-view superset with
      the 60-degree authoritative frustum over the same abrupt-turn/teleport
      trace. The expanded set retained every fresh candidate but adds work and
      still loses temporal overlap at discontinuities; it is not a cache policy.
- [ ] Stage 3: after the ordinary-frustum baseline, compare Doom BSP traversal
      and any separately justified portal/sector method as source-specific
      candidate selection/oracle. Do not place WAD terms in `tokimu-render` or
      propose this source-specific method as Tokimu's answer.
  - [x] Preflight the currently retained source provenance before selection.
        Flat draws retain source subsector identity, while wall and cutout draws
        retain linedef/sidedef identity only; `REJECT` is now decoded only as
        its source-format monster-sight matrix, not as rendering visibility.
  - [x] Establish and test an explicit Doom-only wall-to-subsector attribution
        rule, including one-to-many boundary cases, before any BSP leaf filter.
        Do not infer this from rendered AABBs or leak the attribution into
        `tokimu-render`.
  - [x] Establish and test `REJECT`'s exact LSB-first row-major bit ordering,
        strict too-short rejection, and source-specific claim. The row is a
        monster sector and the column is a player sector; it is not a camera or
        render-visibility oracle.
  - [ ] Stage 3A control: compare a conservative whole-linedef membership-union
        filter with the current full and ordinary-frustum baselines. A wall or
        cutout survives whenever any of its source subsector memberships
        survives; preserve source submission order and retain all false-positive
        cost as measurement rather than tightening the filter.
    - [x] Headless source-spawn/overview count comparison retained; timing and
          visual comparison remain open.
  - [ ] Stage 3B representation experiment: only after Stage 3A, lower a
        corpus-local SEG-granular wall/cutout variant and compare it with the
        whole-linedef control. Preserve the original linedef/sidedef identity,
        side, and continuous texture parameterization across every SEG split.
        Do not modify the established static lowerer, make SEG a renderer
        concept, or create a public fragment/visibility API.
  - [ ] Stage 3 comparison matrix: retain prepared geometry/resource count,
        candidates, submitted draws, selection CPU, command-build CPU, warm
        frame observation, startup preparation, and bounded source identity
        evidence for: full whole-linedef submission; generic frustum on
        whole-linedefs; whole-linedef membership union; SEG-granular generic
        frustum; and SEG-granular Doom BSP selection if the prior experiment
        makes that comparison meaningful.
  - [ ] Stage 3 falsification: reject the SEG-granular direction if it does not
        materially outperform the whole-linedef control, loses source wall
        identity/texture continuity, or creates enough geometry/resource and
        submission cost to erase any selection saving. A favorable E1M1 result
        remains corpus evidence, not a generic Tokimu contract.
- [ ] Stage 4: do not implement GPU occlusion until independent pressure
      justifies a separate provider/resource/scheduling review.
- [ ] Before any occlusion trial, retain an application-owned occluder-eligibility
      classification experiment that keeps opaque, cutout, and Blend semantics
      distinct under ADR-0013 and AR-0023.
- [ ] Theoretical trials: record a source-neutral hypothesis, inputs,
      guarantee, falsifying result, measurement protocol, and corpus-local
      containment before implementation. Do not promote a theory because a
      single E1M1 observation is favorable.
- [ ] Compare against a second independent camera/scene consumer before
      proposing any generic contract, and prove the proposal satisfies every
      Admission Clamp condition.
- [ ] Apply ADR-0008 and ADR-0009 if a shared capability or hot-path contract
      is proposed.

## Reopening Triggers

- a second independent consumer needs the same camera/bounds selection;
- a conservative corpus-local experiment repeatedly preserves visible content
  while materially reducing submission cost;
- a source-specific Doom visibility method conflicts with generic bounds
  selection or reveals an ownership ambiguity;
- an attempted solution requires renderer-owned ordering, scene truth,
  provider-native handles, or a stable public API; or
- culling hides visible geometry, produces nondeterministic results, or masks
  an unsupported presentation behavior.
- a proposed general contract assumes one globally Euclidean camera/world
  relation and cannot express a caller-declared derived view or candidate domain.

## Review History

### Cycle 1 -- 2026-08-10

- Status entering review: Proposed.
- New evidence: source-spawn E1M1 camera interaction made per-frame static
  mesh replacement visible as lag; the local repair uploads static meshes once.
  The camera still submits every prepared draw, while current black regions
  remain explicit sky/plane-presentation limits rather than inferred culling.
- Findings: CPU frustum candidate selection is the smallest non-binding
  experiment; generic culling, Doom visibility data, and GPU occlusion each
  require distinct evidence and must not be conflated.
- Disposition: Under Review; start with corpus-local evidence only.
- Resulting ADR or documentation change: none.

### Cycle 2 -- 2026-08-10

- New review input: the baseline now separates a repaired static-upload defect
  from the remaining 1,861 prepared submissions. The first experiment should
  use derived world-space AABBs and preserve caller order exactly.
- Refined correctness rule: false positives are retained as efficiency evidence;
  false negatives require immediate investigation. Counts alone are insufficient:
  retain at least two fixed poses, first/warm timing, and native/browser visual
  observations where feasible.
- Disposition: proceed with the bounded corpus-local CPU frustum experiment;
  defer Doom BSP/`REJECT` comparison until the ordinary frustum baseline exists.

### Cycle 3 -- 2026-08-10

- Scope refinement: AR-0025 is now a staged comparative corpus study. It will
  compare explicit submission, generic CPU frustum/AABB selection, an optional
  generic CPU broad phase, Doom-specific spatial methods, and only later
  provider-specific GPU occlusion research.
- Guardrail: every stage shares the same pose, timing, order-preservation, and
  false-negative protocol. No stage is evidence for a public capability merely
  because it reduces a Doom draw count.
- Disposition: begin Stage 0/1; Stage 2 and Stage 4 are evidence-gated, and
  Stage 3 remains source-specific.

### Cycle 4 -- 2026-08-10

- Admission clamp: every selected Tokimu answer must be source-neutral,
  provider-neutral, independently usable, and supported by explicit failure
  and diagnostic evidence. Doom and WGPU techniques are comparative evidence,
  not direct candidates for engine vocabulary.
- Study expansion: bounded theoretical trials are welcome when their hypothesis
  and falsification protocol are retained before implementation.
- Disposition: continue the comparison under the clamp; a specialized winner
  is retained as evidence only, never promoted by convenience.

### Cycle 5 -- 2026-08-10

- Default clarified: explicit full submission remains the admitted fallback.
  Conservative CPU frustum/AABB classification is the provisional study
  default because it is the smallest plausible source-neutral provider, not
  because it has been accepted.
- Theory expansion: compare bounds shape, granularity, broad phases, temporal
  coherence, region/PVS oracles, occluder eligibility, software occlusion, and
  provider-specific GPU mechanisms against shared traces and a pathological
  source-neutral fixture.
- Workload rule: a future Tokimu answer may be a small family of explicit
  provider-neutral methods rather than one universal algorithm, but each method
  needs a distinct general workload case and a clear fallback.

### Cycle 6 -- 2026-08-10

- Future falsification pressure recorded: portal-transformed, recursive, and
  non-Euclidean presentation must be able to challenge any eventual general
  candidate-selection contract. A successful ordinary-world frustum result is
  not a claim of universal visibility semantics.
- Disposition: retain this as a boundary constraint only; do not begin portal
  implementation or admit multi-view vocabulary from AR-0025.

### Cycle 7 -- 2026-08-10

- Implemented evidence: the E1M1 static-scene corpus now has opt-in
  `--frustum-aabb`, deterministic `--candidate-report`, and bounded
  `--measure-two-frames` paths. Bounds are derived once from prepared mesh
  positions; uncertain bounds fail open; filtering preserves caller order.
- Native result: source spawn reduced 1,861 candidates to 495 submitted opaque
  draws; the full-map overview rejected none. The paired warm AMD/Vulkan CPU
  observations were 46,281 us for full submission and 28,257 us for selection,
  with about 2,588 us spent in selection. These are single development-profile
  observations, not a benchmark or portable guarantee.
- Validation: six focused selection/observer tests pass and focused strict
  Clippy passes. The existing static-upload repair remains proven by zero frame
  mesh uploads/replacements and 1,861 unreplaced lifetime uploads.
- Additional fixed-pose evidence: yaw-plus-90 retains 1,051 draws, including
  all 26 cutouts. Native visual false-negative review, browser/WASM parity,
  traces, alternate bounds/granularity, and independent caller evidence remain
  open.
- Disposition: Stage 0 and the automated/native portion of Stage 1 are complete.
  Do not open Stage 2 or propose a shared capability yet.

### Cycle 8 -- 2026-08-10

- Implemented evidence: native and browser/WASM now share the corpus-local
  `StaticDrawAabb` and homogeneous frustum classifier rather than maintaining
  independent selection math. Shared observer-heading helpers also keep the
  source-spawn convention aligned.
- Implemented evidence: the TypeScript boundary workbench exposes a separate
  Rust/WASM `render_static_e1m1_selected_cutouts(canvas)` request. It derives
  E1M1 player-one spawn/sector context in Rust, uses the fixed yaw-plus-90
  cutout-survivor pose, and reports candidate/rejected/submitted counts after
  preserving caller order. TypeScript supplies only the canvas and presents the
  result.
- Validation: focused native tests/clippy, WASM `cargo check` and release
  build, generated `wasm-bindgen` web bindings, and the workbench TypeScript
  typecheck pass. Browser visual observation remains intentionally open.
- Limitation: this one-shot browser fixture uploads meshes before filtering its
  command list. It can test selection equivalence/presentation but cannot claim
  a browser warm-frame performance improvement.
- Disposition: request a manual browser selected-cutout observation next; keep
  Stage 2, all theory comparisons, and any shared contract deferred.

### Cycle 9 -- 2026-08-10

- Browser/WASM observation: following explicit local selection of the reviewed
  package, the selected-cutout request presented `1,051` draws from `1,861`
  candidates, rejecting `810`: `1,025/1,835` opaque and `26/26` cutouts. The
  target reported `browser-webgpu`, device kind `other`, and a `960x600` canvas.
- Cross-target result: those counts exactly match the native deterministic
  yaw-plus-90 report. This proves matching selection cardinality and browser
  first presentation for the bounded fixture, not equivalent pixels, adapter
  coverage, or browser performance.
- Disposition: browser Stage-1 count/presentation evidence is complete. Native
  side-by-side visual false-negative review and the broader comparative stages
  remain open; no Tokimu culling capability is proposed.

### Cycle 10 -- 2026-08-10

- Theory trial declaration: compare source-neutral per-draw enclosing spheres
  with the existing derived AABBs under the same fixed camera poses. Both are
  conservative homogeneous-frustum filters; missing/non-finite bounds fail
  open; neither changes command order or renderer vocabulary.
- Result: the sphere retained more E1M1 draws at every useful pose (for example
  525 versus 495 at source heading and 1,092 versus 1,051 at yaw-plus-90), but
  its one debug-run selection cost was lower (about 1.0–1.2 ms versus 2.1–2.5
  ms). The full-map control retained all 1,861 under both shapes.
- Interpretation: this is a real shape/CPU-versus-selectivity tradeoff, not an
  answer. Sphere-specific E1M1 visual review, traces, candidate granularity,
  the pathological fixture, independent callers, and stable-contract evidence
  remain absent.
- Disposition: preserve AABB as the current study default because it is tighter
  on the only real workload; preserve sphere as a viable corpus comparator. Do
  not optimize selection time or promote either shape before the rest of the
  required matrix exists.

### Cycle 11 -- 2026-08-10

- Theory trial declaration: compare per-draw AABBs against contiguous groups of
  8 and 32 caller-ordered draws. A group derives one enclosing AABB; if it
  intersects or contains an uncertain member, every member survives in original
  order. The group has no source, material, batching, or renderer identity.
- Result: group-8 and group-32 reduced the source-heading selection report to
  about 378 us and 135 us, respectively, versus 2,176 us per-draw; they
  submitted 760 and 1,088 draws versus 495. Every non-overview pose showed the
  same cheaper-but-less-selective pattern.
- Interpretation: draw count, bounds-test count, and CPU selection cost are
  separate facts. A lower number of candidate groups is not a useful result if
  the grouping gives the renderer substantially more surviving geometry.
- Disposition: retain per-draw AABB as the study default, and contiguous groups
  as a deliberately blunt comparator. Continue with traces and a pathological
  fixture before considering an index or public contract.

### Cycle 12 -- 2026-08-10

- Trace evidence: a deterministic nine-frame in-place 360-degree source-spawn
  turn retained from 42 to 1,051 of 1,861 candidates per frame, with zero
  uncertain bounds. Its 0 and 360 degree rows both retained 495, confirming
  closure of the declared trace.
- Pathological evidence: a source-neutral fixture interleaved 64 wholly
  rejectable bounds with 64 crossing/overlapping bounds. Per-draw AABBs safely
  rejected 64; contiguous groups of 8 or 32 rejected none and retained all
  128. This is correct conservative behavior, and a direct counterexample to
  treating fewer group tests as an unconditional improvement.
- Disposition: the trace and fixture requirements are materially advanced, but
  only an in-place yaw trace exists so far. Keep a source-position trace,
  visual false-negative review, and independent-scene evidence open; do not
  open Stage 2 or propose a shared capability.

### Cycle 13 -- 2026-08-10

- Source-relative camera evidence: five fixed offsets from the reviewed
  player-one spawn retained 597, 562, 495, 383, and 309 draws from 1,861, with
  zero uncertain bounds. The values are declared camera inputs only; they do
  not claim collision-safe movement, a valid walk path, or player simulation.
- Observation: per-draw AABB selection cost stayed near 2.1–2.2 ms in this
  development-profile report across the offsets. This is additional pressure
  to measure carefully, not yet a release-profile budget or Stage-2 admission.
- Disposition: the real-world baseline now has yaw and position variation.
  Repeat the comparative shapes/granularities over this position trace before
  judging whether an index is justified; retain full submission as fallback.

### Cycle 14 -- 2026-08-10

- Comparative position-trace result: at every declared offset, AABB submitted
  the fewest draws (597→309), enclosing sphere submitted 30–48 more, group-8
  submitted 235–265 more, and group-32 submitted 619–660 more. Sphere/groups
  were less expensive to classify in this development-profile report.
- Interpretation: the added positional evidence does not change the current
  study default. It strengthens the result that CPU selection cost and
  selectivity must remain independently measured rather than collapsed into an
  abstract "culling efficiency" score.
- Disposition: Stage-1 theory comparisons are complete for this one workload.
  Keep the required visual false-negative review, independent caller, and
  release/profile evidence open. Stage 2 remains a maintainer gate, not an
  automatic consequence of the observed ~2 ms per-draw CPU work.

### Cycle 15 -- 2026-08-10

- Stage-2 entry: the maintainer continued the study after Stage-1 retained
  roughly 2 ms per-draw AABB selection across yaw and declared-position traces.
  The bounded source-neutral index is a static uniform grid over derived draw
  AABBs. It filters only candidate bounds, preserves original draw order, and
  still applies the exact per-draw AABB test to grid survivors.

### Cycle 16 -- 2026-08-10

- Resolution and trace comparison: `4x2x4`, `8x4x8`, and `16x4x16` grids were
  built once and queried over the overview, nine-frame 360-degree yaw trace,
  and four additional declared source-relative position offsets. Empty cells
  are not tested. Every final submission count exactly matched the Stage-1
  per-draw AABB baseline across all retained poses, with zero uncertain bounds.
- Trade-off: finer grids reduced exact tests at the source heading from 1,038 to
  722 to 571, but increased occupied-cell tests from 32 to 177 to 620. Their
  estimated storage/build costs were respectively about 35,104 B / 407 us,
  57,280 B / 462 us, and 101,824 B / 599 us. No resolution was uniformly best:
  on the yaw trace the coarse grid selected in roughly 212--1,691 us, the
  medium grid in 371--1,634 us, and the fine grid in 817--2,223 us.
- Interpretation: the grid is a valid conservative broad-phase comparator, not
  an evident default. At this scale, index selectivity, occupied-cell traversal,
  exact-test work, storage, build cost, and camera pose remain separate facts.
  The per-draw AABB baseline remains the study default because it has the
  smallest and simplest retained contract.
- Limitation: this remains one static Doom-derived scene in a development
  profile, without visual false-negative review for grid selection, dynamic
  invalidation, a second independent scene, or an alternate index. It cannot
  justify a Tokimu index capability, BVH research, or a performance claim.
- Disposition: Stage-2's uniform-grid resolution/trace refinement is complete
  for this workload. Preserve the implementation as corpus evidence only; do
  not select a grid resolution or propose a provider-neutral contract. The next
  meaningful comparison, if the maintainer continues, is an independently
  motivated static-scene/index workload or a bounded alternative-index study.

### Cycle 17 -- 2026-08-10

- Playback refinement: `--frustum-grid-8x4x8` renders the retained medium-grid
  experiment with its grid survivors rechecked by the existing exact AABB
  filter. It preserves caller order and explicitly falls back to full
  submission if a grid cannot be constructed. This is corpus-only evidence,
  not a renderer-facing mode.
- Native two-frame check at the yaw-plus-90 pose: the medium grid submitted
  `1,051` of `1,861` candidates (`1,025` opaque plus all `26` cutouts), exactly
  matching the established AABB selection. It made no mesh upload or
  replacement on either frame. Its 2,039/2,266 us selection observations are
  deliberately retained separately from the headless single-index report:
  interactive playback uses two ordered grids to retain opaque-then-cutout
  presentation order.
- Disposition: the playback mode enables the remaining manual visual review;
  it does not select a grid as the default or provide performance evidence in
  favor of one.

### Cycle 18 -- 2026-08-10

- Native maintainer observation: the interactive medium-grid playback again
  retained `1,051` of `1,861` candidates at the fixed yaw-plus-90 pose
  (`1,025` opaque and all `26` cutouts), with zero uncertain bounds and zero
  mesh uploads/replacements after startup. The observed selection times were
  1,855 us first frame and 1,820 us warm frame on Vulkan/AMD.
- Interpretation: this corroborates the bounded playback/count claim only.
  Terminal statistics do not substitute for the still-open side-by-side visual
  false-negative observation, nor establish a portable performance result.

### Cycle 19 -- 2026-08-10

- Temporal theory trial: a corpus-only one-frame candidate carry was observed
  across small yaw changes, an abrupt 190-degree turn, and a declared
  source-relative forward teleport. Every frame first runs fresh conservative
  AABB selection; the carry is reported only as an inclusive comparison, never
  allowed to suppress the fresh result.
- Result: 0-to-5 degrees shared 493 of 495 prior candidates and retained 534
  carried candidates for 532 fresh candidates. At the abrupt turn, overlap fell
  to 12 while the carried set inflated from 38 fresh candidates to 580. The
  teleport shared only four candidates and retained 58 versus 24 fresh.
- Interpretation: temporal overlap exists in smooth movement, but a naive
  carried set becomes materially over-inclusive at discontinuities and reduces
  neither the authoritative ~2 ms AABB classification nor ownership pressure.
  The explicit fresh classification/fallback is correct but exposes no reason
  to stabilize a temporal cache.
- Expanded-frustum refinement: the same trace also tested a 72-degree current
  view against the authoritative 60-degree view. It retained every fresh
  candidate, as asserted by the corpus, but added 55, 46, and 47 candidates at
  the initial/small-yaw poses. Its consecutive expanded overlap improved to 544
  and 576 for the small turns, then fell to 12 at the abrupt turn and 8 after
  the declared teleport.
- Disposition: retain both as negative theoretical results. An expanded view is
  a conservative candidate superset, not a safe temporal cache or CPU-saving
  answer; it still requires fresh classification/fallback. Do not promote
  prior-frame visibility into scene truth.

### Cycle 20 -- 2026-08-10

- Stage-3 preflight: the existing corpus preparation already resolves Doom BSP
  paths and the source spawn leaf. However, its prepared flat evidence retains
  a source subsector, while wall and masked-middle draw evidence retains a
  linedef/sidedef rather than a leaf. A linedef can bound more than one
  subsector, so a BSP leaf filter cannot safely guess a single wall membership.
- Separate `REJECT` finding: `doom-map-provider` validates the lump's required
  byte length and retains its byte observation, but does not yet decode a
  sector-pair visibility matrix. Its bit order, incomplete-data behavior, and
  correct comparison claim are therefore not established in this repository.
- Disposition: do not implement Stage-3 filtering from either fact yet. First
  establish bounded Doom-only attribution and `REJECT` semantics with retained
  source evidence. This is a source-provider question only; it does not justify
  a renderer contract, a generic candidate API, or a change to full submission.

### Cycle 21 -- 2026-08-10

- Doom-source `REJECT` semantics are now retained in `doom-map-provider` as a
  bounded `DoomRejectMatrix`: sector-pair index is
  `monster_sector * sector_count + player_sector`, stored row-major with the
  least-significant bit first. A synthetic three-sector regression proves bit
  placement and out-of-bounds sector queries remain structured failures.
- The decoded API is intentionally named `forbids_monster_sight`, preserving
  the classic source meaning instead of implying camera or rendering
  visibility. Short matrices still fail decoding; no missing/partial fallback
  is invented.
- Canonical E1M1 source observation at player-one sector `38`: `85` sectors,
  `904` REJECT bytes, `9` monster sectors forbidden to sight the player and
  `76` not forbidden. It does not alter the `1,861` prepared-draw set,
  frustum selection, or render submission.
- Disposition: retain `REJECT` only as a Doom-specific comparison/oracle input.
  It has not earned a rendering filter, generic candidate-selection role, or
  public Tokimu contract. Wall-to-subsector one-to-many attribution remains
  the next Stage-3 source-topology prerequisite.

### Cycle 22 -- 2026-08-10

- The remaining source-topology prerequisite is now explicitly modeled in
  `doom-geometry-provider`: every linedef retains the source subsectors that
  contain it through `SEGS`. The relation preserves source subsector order and
  is deliberately one-to-many rather than guessing a leaf from a wall AABB.
  A synthetic two-leaf/one-linedef regression proves that result.
- Canonical E1M1 source report: `475` linedefs; `0` with no subsector
  membership; `269` with one membership; `206` with multiple memberships; and
  a maximum of `6` leaves for one linedef. This is topology evidence only.
- Finding: existing wall and cutout lowering emits whole-linedef meshes. A
  leaf-filter over those meshes can only retain a wall when *any* of its source
  leaf memberships survives, which is conservative but does not provide
  leaf-granular selection. Achieving a more selective BSP comparison would
  require a source-seg wall representation and careful preservation of Doom
  texture-offset/side semantics.
- Disposition: do not silently change the existing static wall lowerer or
  describe whole-linedef membership as BSP visibility. Maintainer judgment is
  required on whether Stage 3 should first compare the conservative
  membership-union filter, or authorize a separate corpus-local seg-granular
  geometry experiment. Neither choice affects renderer ownership or the
  full-submission fallback.

### Cycle 24 -- 2026-08-10

- Manual native comparison surfaced a presentation-evidence limitation: the
  current static E1M1 baseline does not texture/materialize every source
  surface. Prepared mesh bounds and comparative candidate counts remain valid,
  but a full-versus-selected image can only establish no *additional* visible
  omission relative to this same incomplete baseline. It must not be described
  as complete E1M1 rendering evidence.

### Cycle 25 -- 2026-08-10

- Stage-3A is now executable as a headless, source-only membership-union
  control. It derives a conservative 3D bound for each source BSP subsector
  from its retained region and owning sector height; flats follow their source
  subsector and whole-linedef walls/cutouts survive if *any* recorded source
  subsector survives. Missing source data fails open. This operates alongside,
  not inside, renderer submission.
- Canonical E1M1 count result with masked cutouts: overview retains all `237`
  source subsectors and `1,861` draws. At fixed source-spawn yaw plus 90,
  `136/237` leaves survive and the membership union submits `1,115/1,861`
  draws. The already-retained generic per-draw AABB result at that pose is
  `1,051/1,861`; thus this first topology control is less selective by `64`
  draws, but provides the required whole-linedef representation control for a
  later SEG-granular comparison.
- The same development-profile headless report observed `337 us` selection CPU
  at overview and `311 us` at the fixed source-spawn pose. Those numbers cover
  source-leaf AABB classification plus whole-linedef membership union only;
  they are not a render, command-build, or benchmark claim.
- This is structural/count evidence only. It does not yet retain Stage-3A
  visual comparison, and it makes no Doom renderer-visibility claim.

### Cycle 26 -- 2026-08-10

- The Stage-3A control now has a renderable, fixed-pose corpus mode using the
  same original ordered draw arrays and generic renderer commands as the other
  trials. It selects only at the corpus edge; `tokimu-render` receives no Doom
  topology or membership information.
- Native two-frame observation at source-spawn yaw plus 90 with cutouts:
  `1,115` submitted of `1,861` (`1,089` opaque, all `26` cutouts), `746`
  rejected, zero uncertain bounds, zero warm-frame mesh uploads/replacements.
  Selection CPU was `959 us` first / `665 us` warm; command construction was
  `105 us` first / `47 us` warm; full development-profile frame observation was
  `730080 us` first / `34608 us` warm. These are comparative development
  observations, not benchmarks or GPU-completion measurements.
- The control is more conservative than generic AABB (1,115 versus 1,051
  draws) and is currently more expensive in CPU selection. Its value is the
  representation control for Stage 3B, not a claim that Doom topology should
  replace the generic method. Manual visual comparison remains open and is
  bounded by the incomplete baseline-material limitation.

### Cycle 23 -- 2026-08-10

- Stage 3 is refined into a controlled representation-boundary study. Stage 3A
  measures the useful spatial information left after the existing whole-linedef
  lowering, by retaining a wall whenever any of its source subsector memberships
  survives. This is deliberately conservative and provides the control for all
  finer-grained claims.
- Stage 3B tests the counterfactual: preserve BSP-derived `SEG` granularity to
  candidate selection while retaining the original linedef/sidedef identity,
  wall side, and continuous texture parameterization. Its hypothesis is not
  merely that BSP reduces draws, but that current presentation representation
  may coarsen useful source spatial information too early.
- The study must measure total prepared geometry/resources, candidate and draw
  counts, selection/command CPU, startup preparation, and warm-frame behavior.
  A smaller submitted set that costs more due to fragment creation is a
  negative result, not a win. Any resulting distinction between semantic wall
  identity and independently selectable presentation fragments remains
  corpus-local evidence relevant to future CAD and AR-0026 pressure; it is not
  a new Tokimu vocabulary or renderer responsibility.

## References

- `docs/ADR/ADR-0007-kernel-performance-diagnostics.md`
- `docs/ADR/ADR-0008-native-kernel-ring-performance-and-code-quality.md`
- `docs/ADR/ADR-0009-ring-based-verification-failure-containment-and-recovery.md`
- `docs/Architectural Reviews/AR-0023-textured-surface-alpha-and-depth-policy.md`
- `docs/Architectural Reviews/AR-0024-renderer-failure-observation-and-diagnostic-boundary.md`
- `docs/Architectural Reviews/AR-0026-non-euclidean-spatial-charts-and-authored-angular-topology.md`
- `docs/Plans/DOOM/DOOM WAD Checklist.md`
- `docs/Plans/DOOM/E1M1 static presentation evidence.md`
- `docs/Plans/DOOM/E1M1 camera candidate-selection evidence.md`
- `corpus/hello-doom-e1m1/src/bin/static_scene.rs`
- [Doom specifications, REJECT](https://www.gamers.org/docs/FAQ/DOOM.FAQ.Specs.Chapters.4.html)
