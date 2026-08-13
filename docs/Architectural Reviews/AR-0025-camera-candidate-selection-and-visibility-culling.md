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

### Future Hypothesis: Composable Candidate-Selection Stages

The study may eventually establish that a view uses more than one selection
mechanism. Do not call every mechanism an occlusion provider: frustum tests,
spatial indices, source-topology traversal, and screen-span clipping make
different claims. Until independent evidence earns a capability, they remain
corpus/application mechanisms rather than caller vocabulary.

The candidate model to test is:

```text
prepared ordered candidates
    -> cheap conservative selector(s)
    -> specialized selector(s)
    -> submitted ordered candidates
```

For ordinary conservative selectors, each stage must be monotonic:

```text
output identities ⊆ input identities
```

and must preserve the relative order of survivors. A rejection is valid only
when that stage can prove irrelevance within its declared view/domain. A stage
that produces a source-authoritative visible set—such as the ongoing Doom BSP
and screen-span experiment—has different semantics and must declare them
explicitly; it cannot silently masquerade as generic conservative culling.

Future evaluation must distinguish these roles:

| Role | Authority | Initial use |
| --- | --- | --- |
| Candidate-domain producer | Declares a view-local candidate domain; may introduce a derived domain | Portal/chart/topology research, not a selector |
| Conservative selector | Filters its supplied candidates only; failure falls back to its input | Frustum/AABB and spatial broad phase trials |
| Source-authoritative selector | Applies source-specific presentation semantics with retained reasons | Doom BSP/screen-span comparison only |
| Shadow selector | Produces observations only and cannot alter submission | Parallel corpus comparison and safe rollout evidence |

Multiple mechanisms can help when cheap broad rejection reduces the work seen
by expensive specialized stages. Their value is not assumed from draw counts:
the study must compare composition cost, preserved ordering, identities
retained/rejected by each stage, and visible omissions. A failed selector must
either return its input unchanged or make continuation explicitly unsafe; it
must never silently substitute an empty candidate list.

Parallel/shadow operation is specifically encouraged for corpus evidence:

```text
same prepared candidates
    -> full baseline
    -> generic frustum shadow result
    -> Doom source-presentation shadow result
```

Retain set disagreements (`A - B`, `B - A`, and intersection) with bounded
source identities before any selector is allowed to affect presentation. This
is more informative than comparing aggregate draw counts alone.

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

### Classic Doom Visibility Is a Distinct Oracle

Classic Doom does not require a watertight exterior world that can safely be
viewed from every free-flight position. Its software renderer traverses source
BSP/subsector structure and maintains screen-space solid-wall clipping while
processing visible wall segments. The original source's `R_ClipSolidWallSegment`
is explicitly the solid-wall screen-range operation; see the
[id Software Doom source release](https://github.com/id-software/doom).

Therefore, a full submitted 3D mesh scene can expose source-valid wall spans
that the classic presentation would never submit to the screen. This is not
evidence that a source wall is malformed, nor justification to hide it in
`tokimu-render`. It is concrete Stage-3 pressure for a Doom-owned comparative
visibility experiment.

The SKY1 panorama made this distinction observable in E1M1. Native `LOOK`
identified a suspected exterior span as linedef 247 / sidedef 344: a source
valid `BROWN96` upper span from -24 through 176 between sectors 56 and 68.
It remains ordinary foreground geometry under the current full-mesh corpus
presentation. Any later compatibility comparison must retain the source span,
camera pose, BSP traversal/clipping decision, and whether the classic oracle
would submit it; it must not introduce an image-driven "outside" suppression
rule.

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
| Composed selector pipeline | CAD/editor scenes combining broad spatial rejection with local topology | Tests whether declared filtering stages compose safely and reduce total work | Stages must state conservative/source-authoritative/heuristic guarantees, preserve survivor order, and retain failure fallback semantics |
| Shadow selector comparison | Corpus regression, editor diagnostics, provider rollout | Tests selector disagreements without granting presentation authority | Shadow result cannot alter candidate submission; retain bounded identity differences rather than only totals |
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

Every comparative selector must state its guarantee before it can participate
in any composed experiment. A selector with observed false negatives is a
counterexample fixture only: it must never silently narrow a production-like
candidate set by intersection with conservative selectors.

| Selector | Current guarantee | Composition status |
| --- | --- | --- |
| Full submission | no visibility rejection | valid baseline/fallback |
| Frustum/AABB | conservative geometric rejection | comparative candidate control only |
| Stage 3B per-column SEG grid | falsified; visible false negatives observed | failure reproducer only; never compose as a rejecting provider |
| Source-faithful Doom visibility | not yet established | requires a separate source-owned experiment |

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
  - [ ] Retain one fixed E1M1 sky-boundary pose and source-valid span control
        (including linedef 247 / sidedef 344) through a Doom BSP/front-to-back
        screen-clipping comparison. Establish whether each span is submitted by
        the classic oracle before treating it as a lowering defect or a
        visibility candidate.
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
    - [x] Retain a separate provider-lowered `DoomSegTexturedWallTriangle`
          representation and a synthetic split-wall regression proving that
          both fragments retain the original linedef/sidedef identity and the
          same source-texel U coordinate at their shared seam.
    - [x] Retain the E1M1 headless representation report: `519` source SEGs
          produced `1,256` SEG-wall triangles across `454` linedefs. The
          current whole-wall control has `1,823` opaque wall draws; neither
          number is yet a submitted-draw comparison because SEG geometry is
          not uploaded or selected in this checkpoint.
    - [x] Apply the existing viewer-pose subsector control independently to
          SEG-owned candidates, retaining source SEG order. At the fixed
          spawn-yaw-plus-90 control pose, `136/237` source subsectors retained
          `780/1,256` SEG-wall triangles. This is still a frustum-filtered
          source-subsector control, not a classic Doom screen-clip oracle.
    - [x] Establish a source-faithful near-first BSP traversal observation.
          At the canonical source spawn `(1056,-3616)`, E1M1 visits all `237`
          subsectors beginning `103,104,97,96,...`; this ordering controls no
          coverage yet.
    - [x] Separate Doom occluder authority from screen-interval mechanics.
          E1M1 observes `343` one-sided, `8` closed-back-sector, and `10`
          closed-opening SEGs, alongside `371` open SEGs. This classification
          records only source sector-height evidence and cannot itself hide a
          projected span.
    - [ ] Add bounded screen-span clipping, with a separate Doom-owned
          occluder-authority classification, before interpreting the surviving
          SEG set as Doom presentation visibility.
      - [x] Fixed source-spawn control records `320` horizontal diagnostic
            columns: among `732` source SEGs, `553` are outside, `135` are
            fully covered, `35` partially visible, and `9` fully visible.
            It retains SEG/linedef, traversal rank, interval, authority, and
            coverage contribution without claiming historic projection parity.
      - [x] Retain visible subintervals and lower only those bounded,
            source-labelled portions into a separate corpus presentation
            representation. The fixed source-spawn control produces `47`
            visible source intervals / `154` lowered meshes; the source-derived
            hut control produces `4` intervals / `11` meshes. Neither path has
            uploaded the representation or claimed a visual comparison.
      - [x] Upload the bounded, source-labelled representation in a separate
            corpus mode. The fixed source-spawn control uploads `154` ordinary
            wall meshes from `47` retained source intervals, with no masked
            middles; it retains separate resource/command/warm-frame evidence.
            The diagnostic screen projection is not historic-Doom parity.
      - [ ] Retain a manual fixed-pose comparison of that representation with
            the static shell, including the hut-wall observation. Do not turn
            the diagnostic column projection into a presentation-correctness
            claim.
        - [x] Falsify the one-dimensional coverage control before treating it
              as a viable presentation filter. The native source-spawn mode
              retained only `154` walls and visibly removed substantial
              spawn-room surfaces, so its horizontal-only coverage state is
              unsound for E1M1 presentation.
        - [ ] If Stage 3B continues, replace the one-dimensional control only
              with a separately documented Doom-owned projected vertical-span
              experiment. It must preserve non-occluding openings and retain
              explicit false-negative inspection; do not repair this by
              weakening generic renderer culling or hiding source geometry.
          - [x] Retain a headless two-dimensional source-grid control before
                any replacement presentation mode. It records vertical source
                spans independently of horizontal interval coverage, but is
                not uploaded or claimed correct.
          - [x] Refine that grid to per-column source-ray/SEG intersections,
                rather than using one enclosing rectangle for a sloped wall.
                Retain the change in candidates before considering visual work.
          - [ ] Do not upload the per-column result until a fixed-pose
                false-negative protocol, including the spawn room and hut
                control, is specified. The lower count is not visual evidence.
          - [x] Establish that the per-column upload is fixed-pose comparison
                geometry, not interactive visibility. Turning the native
                camera after source-spawn preparation leaves the selected SEG
                set unchanged and exposes missing geometry.
          - [x] Before any dynamic experiment, retain a headless source-pose
                trace of per-column selection change. Do not rebuild static
                GPU meshes or make camera motion own Doom BSP state merely to
                make the comparison mode look interactive.
            - [x] Retain a four-heading source-spawn trace. Selection changes
                  materially across the declared poses, confirming that a
                  fixed prepared wall set cannot represent interactive views.
            - [x] Retain a separate dynamic corpus control which uploads the
                  lowerable SEG wall set once and varies only a source-owned
                  draw-enable mask after lowering the current observer pose
                  through the selected comparative embedding. Preserve zero
                  warm-frame mesh upload/replacement evidence.
            - [x] Manually inspect turns and movement for visible false
                  negatives; retain exact source poses for any omissions.
                  Current material coverage omissions remain explicit and are
                  not misclassified as visibility rejection.
              - [x] Falsified at close camera range: ordinary nearby walls
                    vanish while the per-column source-grid control updates.
                    This is a visible false negative, so the dynamic control
                    is not usable presentation/culling behavior.
          - [x] Retain three exact false-negative source-pose replays.
                    They select only 4, 7, and 11 SEGs while visually dropping
                    ordinary nearby/courtyard walls; close this per-column
                    coverage branch as unsound rather than tuning thresholds.
          - [x] Audit the retained leaf/source ordering against local source-ray
                depth before attempting richer vertical clipping. All three
                counterexample poses contain later, nearer closing SEGs after
                an earlier SEG has already claimed the same diagnostic cell
                (`134`, `147`, and `928` attempted depth inversions). Therefore
                near-first *subsector* traversal is not sufficient as direct
                per-SEG/per-cell occluder order in this control.
          - [x] Reject upper/lower diagnostic-grid state as the immediate
                successor. Primary classic-Doom source evidence shows that
                `solidsegs` is a horizontal solid-range list; vertical
                `ceilingclip`/`floorclip` handling occurs later during admitted
                wall-tier/plane drawing. Do not generalize the failed 2D grid.
            - [x] Compare the retained leaf/source order against a deliberately
                  non-authoritative global nearest-SEG ordering control. It
                  reduced the three replay depth inversions from `134/147/928`
                  to `0/0/36`, but retained essentially the same tiny selected
                  sets (`4/7/10` versus `4/7/11`). Ordering is therefore a
                  real defect in the old control, but correcting it cannot
                  rescue boolean screen-cell closure.
          - [x] Establish a separate headless, Doom-owned source-protocol
                control before another visual mode: viewer-side BSP recursion,
                backface/FOV SEG admission, solid versus pass range authority,
                horizontal solid-range union, and far-child bbox rejection.
                Compare it with the three retained false-negative poses before
                interpreting any resulting SEG set as presentation evidence.
            - [x] Retain the first `R_AddLine`-style admission checkpoint over
                  all three false-negative poses. Directed source SEG facing
                  rejects `355/350/356` backfaces from `732` source SEGs;
                  after bounded FOV rejection, the remaining admissions are
                  `106/119/126` solid and `73/84/94` pass SEGs. This is not
                  yet far-child pruning or visible-draw evidence.
            - [x] Add the bounded horizontal solid-range union to the same
                  near-first leaf control. The three retained poses close all
                  `320/320` diagnostic columns after only `3/22/2` solid
                  contributor intervals; the remaining `103/97/124` admitted
                  solid SEGs are already covered. Pass/opening SEGs do not
                  mutate this union. This exposes why far-child bbox pruning,
                  rather than a flat all-leaves list, is the next necessary
                  source-protocol question.
            - [x] Add a headless near-first recursive BSP control with
                  source-`checkcoord` child-bbox projection. It visits
                  `94/118/83` leaves and `278/331/241` source SEGs in the
                  three retained poses. The control prunes `4/19/7` far
                  children only after solid-range closure, rejects a separate
                  `6/3/5` child bboxes as definitely outside the source FOV,
                  and deliberately fails open for the remaining `93/75/82`
                  ambiguous/behind/containing-viewer bboxes. The resulting
                  `3/24/3` solid and `2/37/2` pass admissions are
                  source-protocol measurements only, not a submitted mesh set
                  or visual correctness claim.
            - [x] Trace the retained exterior suspect separately through the
                  recursive control. At the three interior counterexample
                  poses and at the source-derived hut control, linedef `247`
                  belongs to source subsectors `190` and `192`, neither of
                  which is reached (`0` visited / `0` admitted). The trace
                  identifies a solid-range far-child rejection at node `235`
                  for near-wall A, courtyard, and hut-control, and at node
                  `197` for near-wall B. This directly contrasts with the
                  earlier all-leaves grid, where its two SEGs had reached a
                  fully-covered interval. It is evidence for continuing a
                  Doom-owned traversal/clip study, not proof that the source
                  wall is malformed or that the current approximation is
                  visual parity. The retained projected intervals are
                  `66..319`, `101..153`, `0..319`, and `36..319`, each
                  covered by the then-complete `0..319` solid range; these
                  are diagnostic columns, not pixels.
            - [x] Map each admitted source SEG back to already provider-lowered
                  opaque SEG-wall triangles without uploading or selecting
                  them. Near-wall A, near-wall B, courtyard, and hut-control
                  respectively retain `5/61/5/4` admitted source SEGs and
                  `6/91/4/8` lowerable SEG-wall triangles. Linedef `247`
                  contributes `0` lowerable triangles in every control. This
                  maps the traversal observation to existing source geometry;
                  it is not a visual draw set because planes, vertical tier
                  clipping, and exact source projection remain unmodeled.
            - [x] Inventory the retained static flat mesh portion separately.
                  The recursively visited subsectors own `184/230/164/150`
                  existing floor draws and `149/157/136/120` existing ceiling
                  draws at near-wall A, near-wall B, courtyard, and hut-control.
                  These are source-labelled mesh counts only, not Doom plane
                  spans or selected draws. They make the remaining wall-tier
                  and plane-span reconstruction boundary explicit before any
                  future presentation comparison is considered.
            - [x] Split the admitted wall-triangle inventory by source tier.
                  The same four poses retain upper/lower/middle counts of
                  `0/0/6`, `4/34/53`, `0/0/4`, and `0/0/8`. In particular,
                  most of near-wall B's 91 lowerable wall triangles are source
                  middle tiers; their eventual opaque/cutout/presentation
                  treatment cannot be inferred from horizontal solid ranges.
                  This is source-tier evidence, not a material-policy change.
          - [x] Retain the first source plane-mark checkpoint at the
                  source eye height (`36`). After horizontal SEG admission,
                  source wall/sector facts mark floor/ceiling eligibility of
                  `4/3`, `53/33`, `5/3`, and `4/4` respectively; the same
                  controls observe `2/15/0/0` paired-`F_SKY1` ceiling
                  adjustments. These are `R_StoreWallRange`-stage facts only:
                  no per-column clip arrays, visplane spans, flat draw
                  selection, or renderer state are produced.
          - [x] Continue the recursive source protocol into a bounded,
                headless wall-tier vertical-clip trace. At the source-spawn
                control it records `37` admitted SEGs with `8/7/23`
                upper/lower/middle tier spans, `36/37` floor/ceiling source
                marks, and `823/875` ceiling/floor clip-boundary updates.
                Near-wall B retains `2/17/25` tier spans and `355/706`
                updates. Marked planes can advance a boundary without an
                upper/lower tier; one-sided middles are terminal while
                two-sided masked middles remain open. This proves that wall
                tier, opening, and plane-span reconstruction must
                remain separate from horizontal solid-range admission; the
                trace creates no visplanes, flat selection, renderer state, or
                new presentation/culling mode.
          - [x] Inventory the decoded source plane grouping keys before any
                span construction. The recursive controls retain floor/ceiling
                contributor counts and distinct `(height, flat, light)` keys,
                normalizing `F_SKY1` ceilings to a common sky identity. Source
                spawn records `36/37` contributors and `6/7` keys; near-wall B
                records `53/33` contributors and `10/7` keys. This establishes
                that plane identity is independent of sector identity and clip
                updates; it creates no visplane, span, flat selection, or
                presentation/culling mode.
          - [x] Reconstruct bounded source-plane instances from the clip state
                before each admitted wall range mutates it. Source spawn
                retains `5/1` floor/ceiling keys as `8` instances after `2`
                collision splits; near-wall B retains `4/1` keys as `8`
                instances after `3` splits. Near-wall A, courtyard-loss, and
                hut-control require no split. All five finish with zero
                overlapping writes. This proves that semantic plane key and
                screen-plane instance identity differ; it does not claim
                visplane parity, select flats, lower geometry, or establish a
                presentation/culling mode.
      - [x] Retain a source-derived hut control: player-one looking at the
            LINEDEFS #208 midpoint. Both SEG records for linedef `247` project
            to columns `166..180` and are fully covered before they contribute
            coverage (`SEG 567` is `OpeningClosed`; `SEG 559` is
            `BackSectorClosed`). This supports a presentation-model mismatch,
            not deletion of the source wall, within the bounded control.
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

### Cycle 24 -- 2026-08-11

- Stage 3B has begun at the representation boundary. The new
  `lower_doom_seg_textured_wall_triangles` path clips the already admitted
  whole-linedef wall triangles to each source SEG while interpolating their
  existing source-texel UVs. It is deliberately separate from the static
  lowerer and generic renderer; no public fragment, `SEG`, or visibility
  abstraction was added.
- A synthetic two-SEG source line verifies the crucial seam property: both
  fragments carry the original wall identity and evaluate U=`57` at the shared
  source midpoint after a seven-texel sidedef offset. The E1M1 headless report
  retained `1,256` triangles from `519` source SEGs and `454` source linedefs.
  Shared world positions with multiple U values are reported only as a
  diagnostic: overlapping sides, roles, and materials make that whole-map
  count unsuitable as a continuity verdict.
- Next: establish a source-faithful front-to-back BSP traversal and bounded
  screen-span clipping control. No conclusion about Doom's historical screen
  clipping, generic occlusion, or a renderer-owned visibility contract follows
  from the current frustum-filtered SEG checkpoint.
- The viewer-pose control now retains `780/1,256` SEG triangles at the fixed
  spawn-yaw-plus-90 pose (`136/237` source subsectors), compared with the
  established whole-linedef membership-union control's `1,115/1,861` draws.
  The counts are useful representation evidence only: Stage 3B has not yet
  uploaded SEG geometry, measured command-build/warm frames, or reproduced
  classic Doom's front-to-back screen clipping.
- The first near-first traversal observation is also retained independently:
  source spawn `(1056,-3616)` walks all `237` leaves beginning
  `103,104,97,96,99,98,102,100`. It is deliberately only ordering evidence;
  no SEG closes a screen span and no visibility conclusion follows yet.
- The first source authority control reports `343` one-sided, `8` closed-back,
  `10` closed-opening, and `371` open E1M1 SEGs. It follows the retained
  source-sector conditions rather than alpha/material inference; projection,
  partial clipping, and coverage mutation remain unimplemented.
- A fixed `320`-column source-space coverage control now combines near-first
  BSP order with that authority input. It observes `553` outside, `135` fully
  covered, `35` partial, and `9` fully visible source SEGs. It is a bounded
  diagnostic control only: it emits no clipped meshes and does not claim exact
  classic-Doom projection, masked/sky behavior, or presentation parity.
- The source-derived hut control supplies the first suspect-specific result.
  From player-one toward the retained linedef-208 midpoint, linedef `247`'s
  two SEGs both project to `166..180` and both are fully covered before their
  own source authority could mutate coverage. The result is evidence that the
  static shell can expose source-valid geometry outside Doom-like presentation;
  it is not yet proof of full historic-renderer equivalence.

### Cycle 25 -- 2026-08-11

- Stage 3B now retains visible source subintervals as a separate lowered
  representation. A provider-local helper clips an already SEG-granular wall
  triangle to a declared owning-linedef interval while preserving SEG,
  linedef, sidedef, role, source texture coordinates, and texture phase. Its
  regression verifies bounded positions, retained identity, and finite
  in-range texture coordinates after subinterval clipping.
- The fixed source-spawn diagnostic control lowers `47` visible intervals into
  `154` ordinary supplied-UV meshes; the source-derived hut control lowers
  `4` intervals into `11` meshes. Linedef `247` remains excluded at the hut
  pose because both of its source SEGs were fully covered before their own
  source authority could contribute coverage.
- This validates a separate source-labelled comparison representation, not
  presentation correctness. The `320` diagnostic columns use a bounded
  source-space approximation; the meshes are deliberately not uploaded yet.
  The next checkpoint is a separate corpus render mode with explicit resource,
  command, warm-frame, and fixed-pose visual evidence.

### Cycle 26 -- 2026-08-11

- A separate native `--doom-seg-clip-presentation` mode now uploads only the
  bounded, source-labelled Stage 3B representation before the ordinary
  comparative embedding is applied. It retains the normal source-labelled wall
  material path but intentionally omits flats and masked middles, so it cannot
  be mistaken for a replacement E1M1 renderer or a generic visibility feature.
- Its source-spawn two-frame control retains `47` visible intervals, `154`
  source-wall meshes, `154` submitted opaque draws, `0` cutout draws, three
  pipelines (ordinary wall, sky, and debug), zero warm-frame mesh uploads or
  replacements, and an `8.036 ms` development-profile warm frame on the
  retained AMD/Vulkan workstation. The uploaded mode is evidence that the
  representation can reach the existing renderer without expanding its
  vocabulary; it is not visual or historic-Doom parity evidence.
- Next: retain the manual fixed-pose shell-versus-visible-SEG observation,
  especially for the hut/linedef-247 control. A prettier screenshot alone
  cannot decide whether the diagnostic coverage model is source-faithful.

### Cycle 27 -- 2026-08-11

- The required native comparison falsified the uploaded one-dimensional
  screen-span control. The source-spawn `--doom-seg-clip-presentation` scene
  showed large missing portions of the spawn room, not merely the target hut
  wall. It therefore has visible false negatives and cannot serve as a Doom
  presentation filter, despite its favorable `154`-draw count and clean
  resource behavior.
- Diagnosis: horizontal source-column coverage loses the vertical opening/span
  information that Doom's viewer-relative wall presentation needs. A near SEG
  may legitimately cover part of a column without authorizing rejection of
  every farther wall fragment at that x-coordinate. The normal static E1M1
  scene remains the only usable presentation path.
- Next: either stop Stage 3B with this negative result or conduct a new,
  explicitly bounded Doom-owned projected vertical-span control. No renderer
  contract, generic occlusion claim, or source-wall deletion follows from this
  failure.

### Cycle 28 -- 2026-08-11

- The first replacement remains headless: a fixed `320 x 200` source-space
  grid projects each source SEG's horizontal interval and enclosing
  front/back-sector vertical range before applying only the existing Doom
  occluder-authority classification. At source spawn it observes `553`
  outside, `117` fully covered, `53` partial, and `9` fully visible SEGs,
  compared with the rejected one-dimensional control's `135` fully covered and
  `35` partial SEGs.
- This demonstrates that vertical span information materially changes the
  candidate result, but it has not yet proven sufficient: each projected wall
  is still conservatively represented by a rectangular grid extent rather than
  exact per-column projected source geometry. The two-dimensional result is
  therefore retained as a diagnostic comparison only and is deliberately not
  uploaded for another visual claim.

### Cycle 29 -- 2026-08-11

- The source-grid control now has a still-bounded per-column refinement. Each
  horizontal diagnostic ray intersects the finite source SEG before deriving
  that column's vertical floor/ceiling span; only a finite forward intersection
  is accepted, with a deterministic endpoint-depth fallback retained for a
  grid edge. A focused regression covers forward versus behind-camera source
  ray/SEG intersection.
- At source spawn, per-column spans retain `111` fully covered and `59`
  partial SEGs (`15,379` covered cells), compared with the enclosing-rectangle
  control's `117` fully covered and `53` partial (`17,968` covered cells).
  The directional result is expected: less invented coverage preserves more
  source candidates. It still does not establish sufficient Doom clipping
  behavior, so no upload or renderer behavior changed.

### Cycle 30 -- 2026-08-11

- A separate manual comparison upload retained normal flats and cutouts while
  replacing the static shell's walls with `150` whole-SEG draws selected from
  `68` per-column source candidates. Its source-spawn frame was visually
  plausible enough to inspect, but turning the native camera immediately
  revealed missing geometry because the selection was calculated once before
  static resource upload.
- That is a useful boundary result, not an implementation defect: the mode was
  deliberately a fixed-pose corpus comparison. It must not be described as
  camera culling or repaired by per-frame static mesh replacement. The next
  evidence is a headless source-pose change trace, followed only if justified
  by an explicit source-owned dynamic selection/resource-lifetime design.

### Cycle 31 -- 2026-08-11

- The per-column headless source-spawn trace establishes that this is a truly
  viewer-dependent source selection experiment: headings `90`, `180`, `270`,
  and `0` retain `68`, `44`, `6`, and `21` source SEGs respectively. The large
  variation rules out treating the source-spawn prepared set as a general
  camera culling result.
- A dynamic continuation would require a separate Doom-owned policy for pose
  observation, candidate recomputation, mesh/resource reuse or retirement,
  stale-selection behavior, and failure reporting. The current corpus app
  deliberately does none of that. No static mesh-reupload path, renderer
  state, or generic camera-occlusion contract has been added.

### Cycle 32 -- 2026-08-11

- The bounded dynamic continuation now exists as a separate native corpus
  control, `--doom-seg-per-column-dynamic`. It uploads every currently
  lowerable SEG wall once (with normal flats and cutouts retained), then each
  observer frame lowers the current world pose through the explicit AR-0028
  comparative embedding and updates only a caller-owned draw-enable mask from
  the source-owned per-column grid. No Doom data reaches `tokimu-render`; the
  renderer receives the same ordinary, already-uploaded mesh handles and draw
  commands.
- Its source-spawn PreserveNorth two-frame control retained `2,095` candidate
  draws, rejected `1,068`, and submitted `1,027` (`1,001` opaque + `26`
  cutout). Selection took approximately `11.9 ms` in this development build;
  the warm frame had zero mesh uploads and zero mesh replacements. This proves
  resource stability, not acceptable interactive performance or source-faithful
  presentation.
- The dynamic control explicitly retained missing current wall-material
  coverage rather than failing or synthesizing a material: `BRNBIGC`,
  `BRNBIGL`, and `BRNBIGR` remain unsupported source textures. They are a
  separate material-coverage limitation, not a reason to alter visibility
  semantics. Visual false-negative testing under turning/movement remains open.

### Cycle 33 -- 2026-08-11

- Native interactive inspection immediately falsified the dynamic per-column
  control as a presentation selector: approaching ordinary walls causes them
  to disappear. The source-grid approximation therefore has visible
  close-range false negatives even though its submitted set updates after a
  turn and its mesh-resource lifetime is stable. It must remain diagnostic
  evidence only and must not be enabled in a normal E1M1 invocation.
- The same PreserveNorth run reports `E`/`USE` as unavailable by design. The
  corpus's manual-door path has not yet migrated its source-correspondence,
  dynamic-height lowering, collision, and picking controls through the
  AR-0028 comparative embedding. This is an explicit capability boundary, not
  an input-event loss or renderer failure. Do not bypass it with a local
  coordinate compensation merely to make this visibility experiment more
  convenient.
- Stage 3B now has a useful negative result: static resource reuse can support
  changing caller submission masks, but the current source-grid coverage model
  is insufficient to decide those masks safely. Any continuation needs a more
  faithful Doom presentation reconstruction and its own false-negative
  protocol; it is not a candidate for generic Tokimu camera culling.

### Cycle 34 -- 2026-08-11

- A declared source-position trace makes the failed control reproducible
  without relying on free navigation. At the unchanged source heading, offsets
  `[0,0]`, `[64,0]`, `[-64,0]`, `[0,64]`, `[0,-64]`, and `[128,64]` from
  player one selected `68`, `62`, `63`, `54`, `76`, and `54` source SEGs.
  Coverage varied from `14,362` to `20,160` grid cells. This confirms that the
  control's aggressive candidate changes occur under small declared pose
  changes; it provides no counter-evidence to the observed close-wall false
  negatives.
- The native debug-console `CAMERA` command now reports the current world pose
  plus its exact AR-0028-lowered Doom source `(x,y,heading)` pose. Any future
  visual anomaly can therefore be replayed as a bounded source observation
  instead of guessed from navigation.

### Cycle 35 -- 2026-08-11

- Free-navigation inspection supplied three retained source-pose replays for
  the dynamic control: `(1202,-3502;-24.0°)`, `(1296,-3427;-0.4°)`, and
  `(1514,-2481;-29.2°)`. The per-column grid selected only `4`, `7`, and `11`
  source SEGs respectively, while marking `344`, `393`, and `425` fully
  covered. Each pose visibly omitted ordinary nearby or courtyard geometry.
- This is decisive Stage 3B negative evidence. The failure is systemic: a
  bounded source-grid coverage approximation is not conservative enough to
  act as a Doom presentation selector. Do not tune its column count, relax
  thresholds, or add generic renderer fallback to make the screenshots look
  better. The static full shell and the existing conservative generic frustum
  experiments remain the usable evidence paths.
- The `--doom-seg-per-column-dynamic` executable remains only an intentionally
  labelled failure reproducer and resource-lifetime control. Any future Doom
  presentation reconstruction must start with a new, source-faithful clipping
  model and independently specified false-negative protocol.

### Cycle 36 -- 2026-08-11

- The negative result is narrowed precisely. Stage 3B has **not** falsified
  SEG-granular representation, retained source identity/continuous UVs,
  near-first BSP order, or source occluder classification. It falsified the
  downstream interpretation that a near SEG may permanently close every
  vertical portion of a touched diagnostic screen column.
- The three replayed false-negative poses are permanent successor
  counterexamples: close-wall A `(1202,-3502;-24.0°)`, close-wall B
  `(1296,-3427;-0.4°)`, and courtyard `(1514,-2481;-29.2°)`. A future
  source-faithful Doom visibility study must retain ordinary visible geometry
  at all three, plus the fixed spawn, hut, turn, and movement controls.
- The likely next hypothesis is richer per-column clip state, with upper and
  lower screen bounds and explicit openings, rather than one boolean
  `covered` state. That remains a Doom-owned source-presentation experiment;
  no claim is made yet that it reconstructs historic Doom clipping or that the
  state is reusable renderer vocabulary.

### Cycle 37 -- 2026-08-11

- Before pursuing that richer state, the existing boolean grid gained a
  non-mutating local-depth audit. It records when a later source SEG attempts
  to close a diagnostic cell at a closer finite ray depth than the first SEG
  that the current near-first-subsector/source-record order had already allowed
  to close it. The audit does not alter coverage or selection.
- The three retained false-negative replays contain `134`, `147`, and `928`
  such attempted inversions respectively. Representative close-wall A evidence
  is source SEG `302` at depth `92.859` preceding source SEG `307` at depth
  `13.706` for the same projected cells. This is direct evidence that the
  current near-first **subsector** order is not itself a sufficient per-SEG or
  per-cell depth order.
- Consequently, Stage 3B has narrowed again: SEG granularity and source
  occluder classification remain useful evidence, but a vertical clip-state
  successor must not be built on the current leaf-rank/source-record order.
  The next bounded question is source-owned SEG/span ordering—not renderer
  sorting, generic occlusion, or another column-resolution adjustment.

### Cycle 38 -- 2026-08-11

- A second headless ordering control sorts source SEGs by their nearest finite
  source-space point to the viewer before applying the unchanged per-column
  grid. This is deliberately **not** a claim about Doom traversal or an
  admissible visibility algorithm: one long SEG can have different local order
  on different rays.
- It is nevertheless a useful discriminator. For close-wall A, close-wall B,
  and courtyard, the local-depth audit changes from `134/147/928` inversions
  under the BSP-leaf/source-record order to `0/0/36` under the nearest-SEG
  control. Thus coarse leaf ordering is genuinely one cause of the previous
  false rejection evidence.
- The selected SEG counts remain `4/7/10` (versus `4/7/11`), however, and the
  fully-covered counts remain `344/393/426`. The catastrophic selection does
  not materially improve. This independently falsifies the idea that ordering
  correction alone can salvage the boolean-grid selector; it must remain a
  failure reproducer.

### Cycle 39 -- 2026-08-11

- Primary source review corrected the successor hypothesis. Classic Doom’s
  `R_ClipSolidWallSegment` maintains `solidsegs` as horizontal ranges, while
  `R_ClipPassWallSegment` deliberately does not close those ranges for
  windows/portals. `R_AddLine` performs backface/FOV admission before this
  classification, and `R_RenderBSPNode` uses `R_CheckBBox` plus accumulated
  solid ranges to decide whether to visit a far subtree. Per-column vertical
  clip arrays appear later in wall-tier/plane drawing.
- Therefore the failed 2D boolean grid is now a deliberately retired
  approximation, not the foundation for an upper/lower-bound successor. The
  next bounded Stage 3B task is a Doom-owned horizontal source-protocol
  control—BSP recursion, SEG admission, solid/pass authority, solid-range
  union, and far-child bbox test—tested headlessly against the three retained
  counterexample poses before any presentation work. See
  `docs/Plans/DOOM/Evidence/Classic Doom visibility clipping evidence.md`.

### Cycle 40 -- 2026-08-12

- The first source-protocol control now applies directed SEG backface and
  bounded-FOV admission in the existing near-first BSP leaf order, retaining
  separate solid and pass authority. Across close-wall A, close-wall B, and
  courtyard, it rejects `355/350/356` backfaces and then admits
  `106/119/126` solid plus `73/84/94` pass SEGs.
- Adding a diagnostic horizontal union of solid intervals reaches all 320
  columns with only `3/22/2` contributing solid intervals. The other
  `103/97/124` admitted solid intervals are already closed; pass intervals do
  not close the union. These counts are neither screen pixels nor selection
  results. They establish that the missing protocol component is viewer-side
  far-child bbox rejection against the accumulated union—not another generic
  grid or a renderer culling policy.

### Cycle 41 -- 2026-08-12

- The bounded source-protocol control now recurses viewer-side BSP children,
  processes the near child first, and projects the decoded far-child bbox only
  to ask whether its horizontal range is already solid. Ambiguous/behind/
  containing-viewer bboxes fail open; no renderer or generic visibility state
  was introduced.
- On the three retained false-negative poses it visits `114/127/92` leaves and
  `341/361/267` source SEGs, with only `4/19/7` far-child prunes and
  `113/84/91` explicit fail-open far checks. The control has not yet been
  lowered, uploaded, or visually compared. Its sole conclusion is that the
  original source protocol can produce a far-subtree candidate reduction which
  the previous all-leaves boolean grid could not represent.
- The current control now selects its two bbox silhouette corners with the
  source `checkcoord` table and maps angles through a perspective plane rather
  than an all-corner linear-angle approximation. It remains short of Doom's
  exact binary-angle/FOV lookup arithmetic and must not become a visual mode
  until a separate regression validates the remaining approximation or it is
  replaced with a better source-faithful equivalent.

### Cycle 42 -- 2026-08-12

- The source-bbox preflight now mirrors classic Doom's `checkcoord`
  silhouette-corner selection, while interval mapping uses a perspective-plane
  tangent relation—the structural role of `viewangletox`. Unit controls retain
  solid-range union, bbox fail-open, and perspective interval behavior.
- The correction leaves close-wall A and courtyard unchanged but changes
  close-wall B slightly: `129 → 127` leaves, `364 → 361` source SEGs, and
  `26 → 24` solid admissions. Bbox/screen mapping is therefore material
  evidence, while the remaining fixed-point-table gap keeps the study
  headless.

### Cycle 43 -- 2026-08-12

- The recursive source-bbox control now distinguishes a definitely
  out-of-FOV far child from an ambiguous bbox. Only the latter fails open;
  the former is a separately retained source-FOV rejection. On near-wall A,
  near-wall B, and courtyard-loss respectively, the resulting waterfall is:
  `94/118/83` leaves, `278/331/241` source SEGs, `4/19/7` solid-range
  far-child prunes, `6/3/5` definite-FOV rejections, and `93/75/82`
  fail-open checks. All three still close their bounded 320-column horizontal
  solid union.
- The trace now includes a source-derived hut control and records the known
  exterior suspect, linedef `247`, separately. Its two SEGs belong to source
  subsectors `190` and `192`; neither leaf is reached or admitted in any of
  the three retained failure poses or the hut control. The responsible far
  child is solid-range rejected at node `235` for near-wall A, courtyard, and
  hut-control, and at node `197` for near-wall B. This is a meaningful
  difference from the falsified all-leaves screen-grid, which had reached both
  SEG records and covered them later. The result is limited to source-traversal
  evidence: it does not establish exact classic-Doom visual parity or authorize
  a presentation filter.
- The retained rejection records now also show the exact bounded source-column
  interval and covering solid range: `66..319` at node `235` for near-wall A,
  `101..153` at node `197` for near-wall B, `0..319` at node `235` for
  courtyard, and `36..319` at node `235` for hut-control. Each is covered by
  the diagnostic `0..319` solid range before the watched subtree is skipped.
  This is inspectable evidence for the control’s causal chain, not screen-pixel
  or historic-Doom parity evidence.
- The same headless trace now maps admitted SEG identity back to the existing
  provider-lowered opaque wall triangles. Near-wall A, near-wall B, courtyard,
  and hut-control retain `5/61/5/4` admitted SEGs and `6/91/4/8` lowerable
  opaque triangles. The same visited subsectors still own `184/230/164/150`
  pre-existing floor draws and `149/157/136/120` pre-existing ceiling draws.
  Linedef `247` contributes none. These are deliberately not submitted draws:
  the flat counts are source-labelled static mesh inventory, not Doom plane
  spans, and source wall-tier/plane clipping has not yet been reconstructed.
  The result therefore only establishes that the recursive protocol can select
  existing source-labelled geometry without changing renderer state.
- The inventory is now split by existing source wall tier. Near-wall A,
  near-wall B, courtyard, and hut-control respectively retain upper/lower/middle
  triangle counts of `0/0/6`, `4/34/53`, `0/0/4`, and `0/0/8`. The near-wall B
  result demonstrates that a substantial selected portion is source `Middle`,
  so horizontal solid-range admission alone cannot decide its later
  opaque/cutout/tier presentation treatment. No material contract changes.
- The first source plane checkpoint now follows the original wall-stage logic
  only as far as floor/ceiling eligibility at source eye height `36`. Across
  near-wall A, near-wall B, courtyard, and hut-control it records floor/ceiling
  marks of `4/3`, `53/33`, `5/3`, and `4/4`, plus paired-sky ceiling adjustments
  of `2/15/0/0`. This precedes classic per-column clipping and visplane span
  construction, so it remains provider evidence rather than flat selection or
  presentation evidence.

### Cycle 44 -- 2026-08-12

- The plane-mark checkpoint makes the next source boundary explicit. A single
  admitted horizontal column can contain an upper wall, an opening into farther
  geometry, a lower wall, and floor/ceiling participation simultaneously.
  Therefore the falsified boolean occupancy grid was missing source semantics,
  not merely column resolution.
- Any continuation must remain headless and retain a small number of columns in
  event detail: initial upper/lower clip bounds, each admitted SEG/tier update,
  surviving opening, floor/ceiling marks, and final visible intervals. The
  three retained false-negative poses, source-spawn doorway pressure, and hut
  control remain mandatory falsifiers before anything is lowered into
  presentation geometry.
- Wall/plane source reconstruction and presentation lowering remain separate:
  first prove visible source wall fragments and plane spans, then separately
  decide whether a Doom presentation adapter can lower them into ordinary
  Tokimu geometry. No renderer visibility, visplane, or Doom-specific contract
  is admitted by this checkpoint.

### Cycle 45 -- 2026-08-12

- The recursive control now retains a bounded per-column wall-tier clip trace,
  using the already-admitted source SEG order and source heights. At the source
  spawn it observes 37 admitted SEGs, 8 upper/7 lower/23 middle tier spans,
  36 floor and 37 ceiling marks, and 823 ceiling plus 875 floor boundary
  updates. A bounded center-column trace retains prior and post clip limits.
- The four counterexample controls demonstrate why this state cannot collapse
  back into a boolean occlusion grid: near-wall B contains 2 upper, 17 lower,
  and 25 middle spans, while the other close/courtyard/hut controls are often
  middle-only. The trace distinguishes terminal one-sided middles from
  two-sided/masked middles, which remain open presentation facts; marked planes
  can advance boundaries even where no upper/lower tier exists.
- This is a source-protocol checkpoint only. It does not construct visplanes,
  select existing flat meshes, upload new geometry, or establish historic Doom
  parity. The next open question is whether a bounded source plane-span
  reconstruction can be proven against the mandatory fixed-pose false-negative
  controls before any presentation-lowering experiment is proposed.

### Cycle 46 -- 2026-08-12

- A separate recursive source-key trace now records the plane grouping facts
  that precede span construction: `(height, flat identity, light)` for marked
  floor/ceiling contributors, with `F_SKY1` ceilings normalized to a common
  sky identity. Source spawn has `36/37` floor/ceiling contributors and `6/7`
  keys; near-wall B has `53/33` contributors and `10/7` keys, including `17`
  sky contributors.
- This confirms that a later plane-span reconstruction cannot treat a sector,
  a wall tier, or a clip-boundary update as plane identity. The trace is still
  source-only and constructs no visplane/span, flat selection, renderer state,
  or presentation result.

### Cycle 47 -- 2026-08-12

- Cross-review evidence: the Option C bulk study keeps E1M1-scale AABB
  selection as a small CPU-favouring control, while synthetic ordered AABB and
  point classification establish only bounded numerical filters. WGPU has a
  measured warm advantage at large synthetic scale but no named deficit that
  earns a specialized provider. A corpus-local AR-0026 chart trace likewise
  shows that derived local transforms can remain caller-owned semantic input.
- Findings: none of this turns Doom BSP, SEG, portal, screen-clip, or
  chart-derived view semantics into generic renderer vocabulary. Generic
  candidate identity/order, query domain, and rejection evidence remain useful
  cross-domain facts; source-specific selection protocols remain providers.
- Disposition: retain the existing source-protocol study and the separation
  between numerical filtering mechanisms and any future admitted capability.

### Cycle 48 -- 2026-08-13

- The recursive Doom-only control now reconstructs bounded source-plane cells
  from the ceiling/floor clip limits that exist immediately before each
  admitted wall range changes them. Plane keys retain kind, height, flat, and
  light; `F_SKY1` ceilings retain the established normalized sky key.
- A first one-instance-per-key prototype exposed `235` repeated column writes
  at source spawn and `151` at near-wall B. Merging those writes by bounding
  union would have fabricated plane coverage. The retained implementation now
  splits a colliding horizontal range into a separate diagnostic plane
  instance. Source spawn resolves `5/1` floor/ceiling keys into `8` instances
  after `2` splits; near-wall B resolves `4/1` keys into `8` instances after
  `3` splits. Near-wall A, courtyard-loss, and hut-control require no splits,
  and all five controls finish with zero overlapping writes.
- This is positive representation evidence and a useful negative abstraction
  result: semantic plane key is not sufficient screen-plane instance identity.
  The trace still stops before classic projection parity, flat lookup,
  triangulation, upload, or presentation selection. The next bounded question
  is whether these retained instances can select and lower source flat spans
  without reintroducing any of the mandatory false negatives; no renderer or
  public visibility contract follows from this cycle.

## References

- `docs/ADR/ADR-0007-kernel-performance-diagnostics.md`
- `docs/ADR/ADR-0008-native-kernel-ring-performance-and-code-quality.md`
- `docs/ADR/ADR-0009-ring-based-verification-failure-containment-and-recovery.md`
- `docs/Architectural Reviews/AR-0023-textured-surface-alpha-and-depth-policy.md`
- `docs/Architectural Reviews/AR-0024-renderer-failure-observation-and-diagnostic-boundary.md`
- `docs/Architectural Reviews/AR-0026-non-euclidean-spatial-charts-and-authored-angular-topology.md`
- `docs/Plans/DOOM/DOOM WAD Checklist.md`
- `docs/Plans/DOOM/Evidence/E1M1 static presentation evidence.md`
- `docs/Plans/DOOM/Evidence/E1M1 camera candidate-selection evidence.md`
- `corpus/campaigns/doom/hello-doom-e1m1/src/bin/static_scene.rs`
- [Doom specifications, REJECT](https://www.gamers.org/docs/FAQ/DOOM.FAQ.Specs.Chapters.4.html)
