# Tokimu Conservative Spatial-Query Capability Study

> Historical filename: this study began as the Tokimu BSP capability setup
> plan. BSP is now a candidate provider/structure, not the architectural
> candidate named by this document.

| Field | Value |
| --- | --- |
| Status | Historical study complete through portable extraction; prospective capability work continues in AR-0031 |
| Scope | Determine whether Tokimu should own an optional conservative spatial-query capability and use Doom as its first demanding corpus |
| Leading ownership hypothesis | Optional Ring 2 conservative spatial-query meaning, with BVH/BSP/grid/brute-force as possible providers; Ring 1 remains limited to lightweight spatial primitives |
| Design order | Define and falsify a BSP **for Tokimu first**; only afterward map, adapt, rebake or retain source BSPs such as Classic Doom and Quake |
| Predecessor | [Doom BSP Presentation-Domain Resolver Study](Studies/Doom%20BSP%20presentation-domain%20resolver%20study.md) — parked after source-bound/derived-geometry authority mismatch was demonstrated |
| Governing decisions | ADR-0001, ADR-0003, AR-0025 and AR-0030 |
| Non-goal | Rebuild `DOOM1.WAD` nodes and assume a different classic node builder fixes Tokimu presentation |

## Purpose

### Current architectural question

Slice 2 changed the question supported by the evidence:

> Should Tokimu own an optional conservative spatial-query capability?

The demonstrated meaning is finite members, immutable artifact/revision,
frustum and ray queries, retained/rejected/unresolved outcomes, reasons and
conservation. It promises no tree topology. BVH, BSP, grid and brute force are
provider or control choices unless later callers prove that a structure's
distinctive topology is itself semantic.

References to “Tokimu BSP” below describe the historical hypothesis or a
specific experimental provider. They no longer name the leading architectural
candidate.

The current Doom investigation has reached a representation mismatch rather
than a loader defect:

```text
Classic Doom BSP
    bounds and traverses source SEGs/subtrees

Tokimu presentation
    reconstructs larger floor/ceiling meshes
    uses an actual pitched 3D camera and frustum

therefore

valid Classic subtree decision
    != automatically valid Tokimu geometry decision
```

The canonical E1M1 audit found exact raw/decoded agreement for all `236` NODES
records and containment of descendant SEG endpoints by all `472` child boxes.
At the same time, `323/472` inferred plane-region envelopes extend beyond those
boxes. The bake is internally consistent for the representation it bounds; it
is not a spatial index over Tokimu's reconstructed presentation geometry.

This plan explores a different direction: bake and query a BSP whose bounded
members are the geometry Tokimu actually intends to prepare or present. Doom
remains the first corpus and source of falsifiers, but Classic Doom's BSP data
does not define the new capability contract.

The primary design rule is:

> Define BSP for Tokimu first. Source-engine BSPs are later mappings, adapters,
> comparative evidence or deliberately coexisting source-private structures.

Tokimu must not define `BSP` by taking the union of whatever Doom and Quake
happen to store. Those engines use related partition structures in service of
different world, visibility, collision and presentation models. The Tokimu
meaning must arise from Tokimu's own representation, ownership, revision,
query, failure and portability requirements.

The plan must answer two different questions without collapsing them:

1. Can a Tokimu-owned BSP safely partition and query actual Tokimu geometry?
2. Does that spatial result provide enough evidence for source-faithful Doom
   presentation participation, or is further Doom-owned occurrence/portal
   policy still required?

A positive answer to the first question does not imply a positive answer to
the second.

## Provisional Tokimu-First Meaning

The smallest useful definition currently under study is:

> A deterministic binary spatial partition over explicitly identified finite
> members, with exact split-member correlation, proven node/leaf containment,
> immutable bake and revision identity, conservative spatial queries,
> contribution conservation and bounded failure semantics.

This definition is intentionally missing:

- visibility and occlusion policy;
- portals and potentially visible sets;
- renderer submission or draw order;
- materials and pipelines;
- collision and content classification;
- simulation or world ownership;
- Doom subsectors, SEGs and visplanes;
- Quake brushes, contents and PVS.

Those may use, annotate or coexist with a Tokimu BSP. They do not become what
Tokimu means by BSP unless later evidence proves that a smaller spatial meaning
is not useful.

The definition is also representation-parametric in the semantic sense. It
must always say which exact finite representation was baked:

```text
finite members with stable identity
    -> partition and, where necessary, split
    -> exact fragments correlated to original members
    -> fragments assigned to leaves
    -> node and leaf bounds proven to contain those fragments
```

This does not pre-authorize a Rust generic `Bsp<T>` or any other public type.
The first corpus implementation should keep the member representation concrete
until the required identity and ownership shape is observed.

## Architectural Hypothesis

ADR-0003 defines three ownership rings. The leading placement is:

```text
Ring 1 — engine kernel
    Vec/Mat/Transform
    Plane/Ray/AABB/frustum primitives
    stable IDs and diagnostics substrate

Ring 2 — optional Tokimu spatial capability
    BSP semantic model
    bake/query requests and results
    bounded-representation and conservation rules
    provider descriptors and diagnostics
    optional portable reference implementation while it remains lightweight

Ring 3 — specialized provider backends
    optional imported/source BSP adapter
    optional accelerated or external-library implementation
```

The provider contract belongs with Ring 2 meaning. A small portable reference
builder/traverser may also live with that capability if evidence earns it and
it remains dependency-light. Ring 3 is for replaceable specialized execution,
not a mandatory extra layer around Tokimu's own basic implementation.

This is a hypothesis, not an accepted crate graph. In particular:

- do not add BSP vocabulary to `tokimu-core`, `tokimu-runtime`,
  `tokimu-render` or `tokimu-platform` during this study;
- do not create `tokimu-spatial`, `tokimu-bsp` or a provider trait until the
  corpus-local experiment has established the smallest useful semantic model;
- do not make all applications carry a BSP provider;
- do not make the renderer request or traverse BSP data;
- do not let a provider own simulation state, movement policy or runtime
  activation/timing.

If admitted, the application/runtime composition root would own provider
selection and lifecycle under the existing capability rules. Simulation or
source state would supply immutable geometry/runtime snapshots. Presentation
preparation could consume query results and hand ordinary declarations to the
renderer.

## Critical Distinctions

### Tokimu BSP is not an imported engine's opinion

A source BSP may eventually be handled in four legitimate ways:

```text
direct mapping
    only when its members, bounds and guarantees satisfy Tokimu's contract

adapter with proof
    preserve source structure while proving Tokimu member correlation and
    containment

rebake
    construct a Tokimu artifact from the actual representation Tokimu queries

coexistence
    retain source BSP for source semantics and Tokimu BSP for Tokimu spatial
    queries
```

For Doom, coexistence is the leading expectation. The original BSP remains
useful for source correspondence, compatibility diagnostics and Doom-owned
presentation evidence. A Tokimu bake covers actual reconstructed geometry and
answers actual-camera spatial queries. Neither tree impersonates the other.

For Quake, direct mapping may be closer, but it must still be proven. PVS,
brush contents and collision classifications remain Quake-owned metadata or
domain semantics unless independently admitted; they must not be smuggled into
a generic `BspLeaf` merely because the source format stores them nearby.

### Spatial partition is not presentation authority

A BSP can safely answer questions about members it actually bounds:

```text
actual Tokimu geometry
    -> baked partitions and proven containing bounds
    -> point/ray/frustum candidate query
```

It does not automatically answer:

- which side of a Doom two-sided boundary produces an upper/lower tier;
- which plane occurrence Classic marks after wall-column clipping;
- how paired sky participates;
- whether an otherwise spatially present surface should be omitted by a
  source-specific presentation protocol;
- renderer ordering, blending, material or depth policy.

The first experiment therefore grants the BSP only conservative spatial
candidate authority. Occlusion or presentation rejection must be earned
separately and remain attributable to the evidence that supports it.

### Bake representation must equal rejection representation

The governing invariant is:

> A spatial bound has rejection authority only over the representation it
> actually bounds.

Every baked node and leaf must name the exact member representation it bounds.
If the provider partitions triangles, its bounds govern those triangles. If it
partitions draw fragments, its bounds govern those fragments. A bound over
source records must not be promoted to authority over generated geometry
without a retained containment proof.

### BSP is not necessarily a binary visibility oracle

Queries need at least three outcomes while the capability is being earned:

```text
retained
    positive evidence says the member participates in the query domain

rejected
    sufficient evidence proves non-participation for the exact member

unresolved
    evidence is absent, stale, ambiguous or outside provider capability
    -> caller retains/fails open
```

The original input inventory must be conserved across these outcomes.

### Static topology is not runtime geometry

Doors and platforms are application-owned runtime state. A Tokimu BSP design
must explicitly choose and measure one or more of:

- immutable static bake plus a conservative dynamic sidecar;
- bounded leaf/node refit after a current-height snapshot changes;
- removal/reinsertion of affected dynamic members;
- bounded rebake of an affected region;
- full rebake, retained only if evidence shows it is sufficiently cheap.

The provider receives current geometry or height-derived fragments. It never
owns activation, timing, collision or movement policy.

## Candidate Semantic Inputs And Outputs

These are study vocabulary, not proposed Rust APIs.

### Bake input

- immutable bake identity and revision;
- dimensionality and coordinate convention;
- finite geometry members with stable caller-owned identity;
- the exact positions/fragments each bound is expected to contain;
- static/dynamic classification;
- numeric tolerances and build limits;
- optional source correlation retained as opaque caller identity, not as Doom
  vocabulary;
- explicit failure and resource budgets.

### Baked result

- immutable artifact identity and input revision;
- partition planes and child relationships;
- leaves containing exact member or fragment identities;
- retained bounds for every node, leaf and member representation;
- split-member correlation back to the original member;
- conservation totals and a structural fingerprint;
- build diagnostics, unresolved members and limit failures;
- serialization/version facts only if a later slice earns persistence.

### Query input

- baked artifact identity/revision;
- immutable view or spatial query identity;
- actual Tokimu frustum, ray, point, volume or region;
- current dynamic overlay/refit revision;
- explicit query limits and requested evidence level.

### Query result

- retained/rejected/unresolved member or fragment identities;
- conservation evidence against the bake inventory;
- reason per decision;
- artifact, view and dynamic-snapshot identities;
- traversal and budget diagnostics;
- no renderer mesh handles, pipelines, materials, Doom SEGs, subsectors or
  gameplay state.

## Provider Shapes To Compare

The study should compare these shapes rather than assuming one prematurely.

### A. Geometry BSP provider

Builds and queries a spatial partition over actual finite geometry. It can
support frustum, ray and region candidates and may become broadly reusable.
It does not claim occlusion or source presentation semantics.

### B. Visibility BSP provider

Adds front-to-back traversal and occlusion evidence over bounded occluder and
occludee representations. This is more useful for Doom presentation, but the
contract may be too presentation-specific or may require portal/coverage
concepts that a generic BSP does not honestly own.

### C. Provider-private BSP inside presentation preparation

Defines no generic BSP capability. Doom's preparation provider bakes Tokimu
geometry internally and returns ordinary declarations plus conservation
evidence under AR-0030. This is the fallback if only Doom benefits or if the
useful semantics remain inseparable from source presentation rules.

The study should prefer A until evidence earns B. C remains a successful
outcome, not a failure to generalize.

### D. Smaller spatial-query capability with BSP as an implementation

Tokimu may need conservative ray/frustum/region queries but not partition-plane
topology or split fragments as shared meaning. In that case the Ring 2 contract
could be a smaller spatial-query capability, while BSP, BVH or another index is
a replaceable implementation detail. This alternative must remain viable until
callers prove that explicit front/back partition topology is itself semantic.

## Required Adversary — BSP Versus BVH

The first slices must not let the name of the study select the winner. The
generic requirements currently listed—finite members, containment, ray and
frustum candidates, revisions, conservative failure and dynamic refit—are also
natural BVH responsibilities.

```text
BSP may earn its place through
    explicit partition planes
    front/back ordering
    partition-aligned fragment splitting
    space/portal/occlusion reasoning that needs that topology

BVH may be preferable for
    conservative bounds over actual geometry
    ray/frustum candidate queries
    limited or no geometry splitting
    dynamic refit
    simpler containment and update costs
```

The study must answer:

> Which required Tokimu caller consumes partition topology or split-member
> meaning that cannot honestly be supplied by a BVH or a smaller spatial-query
> contract?

Possible outcomes include admitting BSP, admitting BVH, admitting both,
admitting a smaller common spatial-query vocabulary with both as providers, or
keeping all of them provider-private.

## Work Plan

### Slice 0 — Preserve And Park Current Evidence

- [x] Preserve the Classic BSP raw/decoded/SEG-envelope audit.
- [x] Preserve plane-region overruns and the subsector `64`, `97`, `99` and
      `113` counterexamples.
- [x] Preserve full-submission BSP tinting, `LOOK`, `SCAN` and headless replay.
- [x] Park the Classic-BSP-as-Tokimu-plane-prefilter direction without deleting
      its implementation or evidence.
- [x] Record this plan as the proposed successor in AR-0030; do not change
      AR-0030's accepted/proposed disposition merely by linking it.

Acceptance: the old approach remains replayable and cannot silently become the
new provider's correctness baseline.

### Slice 1 — Corpus-Local Tokimu Geometry Bake

- [x] Write the corpus-local Tokimu-first semantic contract in terms of finite
      member identity, partitions, fragments, containment, revisions,
      conservation and bounded failure; do not begin from Doom NODES fields.
- [x] Select the exact initial member representation: prepared triangles or
      explicitly bounded draw fragments. Record why it is the thing queries
      are allowed to reject.
- [x] Build a deterministic, corpus-local BSP over the complete E1M1 ordinary
      geometry inventory; do not create a shared crate or public trait.
- [x] Retain original contribution identity across every split fragment.
- [x] Prove every leaf contains its assigned fragments and every node bound
      contains the complete descendant fragment envelope.
- [x] Conserve every original member as unsplit, split, dynamic-sidecar or
      unresolved; no silent loss or duplication.
- [x] Retain build time, node/leaf depth, split amplification, memory estimate,
      structural fingerprint and bounded diagnostic samples.
- [x] Build or reuse a comparable corpus-local BVH/control hierarchy over the
      same members and retain the semantic and cost differences.

Acceptance: identical input produces an identical structural fingerprint; all
containment and conservation checks pass; malformed/degenerate geometry fails
explicitly and within limits.

#### Slice 1 first evidence

The corpus-local command is:

```text
--tokimu-spatial-bake-report
```

It consumes the unchanged complete prepared-triangle inventory, not Doom NODES
or subsectors:

```text
members                 1,849
source draw labels         624
floor triangles             463
ceiling triangles           390
wall triangles              970
cutout triangles             26
dynamic sidecar               0
```

The bounded median-plane reference BSP reports:

```text
nodes / leaves                 14,231 / 7,116
maximum depth                  20
final fragments                334,345
amplification                  180.824770x
generated-fragment budget      500,000 (reached)
depth-limited leaves           3,876
budget-limited leaves          12
fragment payload lower bound   16,048,560 bytes
containment failures           0
missing original members       0
area delta / tolerance         0.595776 / 280.524399
fingerprint                    78b7e9300f148c33
observed build time            406–412 ms
```

Amplification by family is:

```text
floor      463 -> 117,716    254.246220x
ceiling    390 ->  67,572    173.261538x
wall       970 -> 146,116    150.635052x
cutout      26 ->   2,941    113.115385x
```

The same-member BVH control reports:

```text
nodes / leaves              255 / 128
maximum depth               7
members                     1,849
amplification               1.0x
containment failures        0
missing / duplicate         0 / 0
fingerprint                 599d8ca7411ffd11
observed build time         4.9–5.2 ms
```

The fingerprints repeat across independent runs. Two focused tests retain
split identity/area conservation, containment, inventory conservation and
deterministic synthetic fingerprints.

An initial pre-correction diagnostic exposed an ordinary limit-enforcement
defect: the intended `500,000` fragment budget was checked per node rather than
globally. That run reached `650,337` final fragments (`351.723634x`) after
`974,955` generated split fragments. The corrected bake globally stops further
partitioning and retains the affected node as a conservative leaf; the bounded
result above replaces it as the implementation control while the first result
is retained as fragmentation evidence.

This is a material finding, not yet a verdict on every BSP strategy. It shows
that naive median-axis triangle splitting is not a viable default Tokimu index
for this representation, and that the BVH adversary is presently much smaller
and faster while satisfying the same Slice 1 containment/conservation needs.
Proceeding requires deciding whether the next experiment should change BSP
split-plane/member policy, test a non-splitting partition, or advance the BVH
control to actual-camera queries first. Do not tune the BSP merely to improve
its headline numbers.

### Slice 2 — Actual Tokimu Frustum And Ray Queries

- [x] Record the selected disposition: advance the unsplit BVH control first
      and park further BSP construction until a caller requires partition
      topology or split-fragment meaning that the BVH cannot supply.
- [x] Query the BVH with the actual Tokimu view/projection used by the native
      camera, including pitch and off-axis rays.
- [x] Compare BVH candidates with the existing conservative per-member
      AABB/frustum classifier and brute-force exact triangle/ray controls.
- [x] Replay source spawn, bounded yaw, bounded movement, neutral/up/down pitch,
      near-wall controls and the retained marked rays.
- [x] Require every rejected member to have a geometric proof over the exact
      baked representation.
- [x] Add explicit stale-revision behavior to the extracted immutable
      corpus-local artifact. Unsupported-query and query-budget vocabulary
      remain unearned because no provider boundary was admitted.
- [x] Execute the same deterministic fixture natively and through the
      `wasm32-unknown-unknown` test runner from the authorized corpus-library
      home. The native `static_scene` binary remains correctly native-only.
- [x] Compare the Slice 1 BSP and same-inventory BVH evidence. No current
      frustum/ray caller consumes partition planes or split fragments; the BSP
      therefore has no earned semantic value for this slice.

Acceptance: no brute-force-visible sampled member is rejected; every result is
bound to one bake, view and dynamic revision; candidate reduction is measured
but is secondary to correctness.

#### Slice 2 first evidence

The corpus-local command is:

```text
--tokimu-spatial-query-report
```

It rebuilds the deterministic unsplit BVH over the same `1,849` exact prepared
triangles, identifies the immutable bake by fingerprint `599d8ca7411ffd11`,
and labels the unchanged inventory `dynamic-revision=static-0`. Nine headless
queries use Tokimu's checked right-handed view/projection math at `1280x800`
and vertical field of view `60` degrees:

```text
source spawn: neutral, yaw -45, yaw +45, pitch up, pitch down
retained movement/near-wall controls: subsector 97 and subsector 64
retained off-axis rays: wall 101 and wall 107
```

For every pose, BVH frustum candidates exactly equal a brute-force pass of the
same conservative per-triangle AABB/frustum classifier. Exact BVH nearest-ray
hits also equal brute-force triangle tests:

```text
poses                         9
frustum false negatives      0
frustum false positives      0
nearest-ray mismatches        0
matrix fingerprint            3c80342bb2cfcdf4

BVH frustum members tested    287 .. 1,257 of 1,849
BVH ray triangles tested       43 ..   361 of 1,849
observed debug query time
    frustum                   0.46 .. 2.22 ms
    ray                       0.009 .. 0.046 ms
```

The retained rays resolve to the same source-correlated members previously
identified by `LOOK`: sector-38 `FLOOR4_8`, sector-29 `FLOOR4_8`, linedef 101
`STARTAN3`, and linedef 107 `DOORSTOP`. The query reports retained, rejected,
unresolved and conservation totals per view while changing no renderer
submission.

These timings are debug observations, not performance guarantees. The exact
candidate equality is deliberately against the existing conservative AABB
classifier, not rendered-pixel visibility. The result establishes a useful
spatial-query control without granting occlusion or presentation authority.

The direct `wasm32-unknown-unknown` check of `static_scene` remains inapplicable:
that binary owns a native window lifecycle and fails on its existing native
`run_window_with_app`/window-construction path before this diagnostic is a
portable consumer. This is retained as a placement constraint rather than
worked around by publishing an engine API prematurely.

Portable extraction is now complete in
`corpus/lib/tokimu-spatial-query-study`. Fixed cross-target fixture evidence is
recorded in
`docs/Checkpoints/2026-08-17-spatial-query-portable-extraction.md`. AR-0031 now
owns the prospective capability question; this historical plan should not be
expanded into a public API design document.

### Slice 3 — Doom Presentation Correlation In Shadow Mode

- [x] Park this BSP-specific slice without implementation: Slice 2 found no
      current spatial-query result that requires BSP partition topology, and
      repeating the BVH's conservative candidates through a BSP would not add
      semantic evidence.
- [ ] If BSP-specific pressure appears, feed conservative BSP candidates into
      Doom-private preparation as evidence only, while submitting the unchanged
      complete inventory.
- [ ] Correlate retained/rejected/unresolved candidates with existing Doom
      source occurrence, plane-span, sky and wall-tier diagnostics.
- [ ] Keep spatial reasons distinct from Doom presentation reasons.
- [ ] Paint disagreements without allowing either side to remove geometry.
- [ ] Determine whether the Tokimu BSP removes the camera-domain and
      bound-authority disagreements while leaving genuine source-occurrence
      questions visible.

Acceptance: the diagnostic can state separately, for any inspected surface,
what the Tokimu geometry BSP knows and what Doom's source protocol knows.

This acceptance criterion is conditional on BSP being resumed. Existing
`LOOK`, `SCAN` and BSP diagnostic tinting already preserve the Doom source side;
the new BVH result is deliberately only geometric candidate evidence and does
not impersonate source occurrence knowledge.

### Slice 4 — Runtime Geometry Strategy

- [x] Replay manual door sector `4` and down/wait/up platform sector `70`
      endpoint snapshots through the bake/query boundary. Turbo-floor and
      intermediate/reversal/wait phases remain refinements.
- [x] Compare dynamic sidecar, topology refit and immutable rebuild approaches
      at the retained endpoints.
- [x] Verify baseline and current geometry revisions differ and reject use of
      the stale baseline identity for each current snapshot.
- [x] Measure update cost and candidate correctness at the retained endpoint
      snapshots.
- [x] Extend door and platform matrices through closed/high, 25/50/75 percent
      motion, open/low, closing/ascending and waiting/repeated-geometry states.
- [ ] Add the turbo-floor sequence as a final Doom runtime refinement if it
      supplies geometry behavior not already exercised by the platform.
- [x] Keep activation, timing, collision and carried-observer policy outside the
      provider.

Acceptance: current geometry changes are visible atomically to queries, stale
artifacts fail explicitly, and the chosen strategy has bounded measured cost.

#### Slice 4 endpoint evidence

The corpus-local command is:

```text
--tokimu-spatial-runtime-report
```

It reconstructs runtime geometry from immutable current-height map snapshots
and first proves exact geometry-multiset equality with the ordinary prepared
baseline: `1,849` triangles and geometry fingerprint `9f394a35516f5567`.
Zero-area authored bands omitted by ordinary preparation are also omitted from
the runtime inventory.

Observed debug endpoint results are:

```text
                                     door open       platform low
current members                      1,853           1,849
immutable rebuild correctness        match           match
topology refit supported             no              no
dynamic sidecar correctness          match           match
stale baseline rejected              yes             yes

snapshot geometry preparation        ~10–11 ms       ~10–11 ms
immutable BVH rebuild                ~7.5–8.5 ms      ~7.5–8.4 ms
sidecar membership update            ~0.5–0.8 ms      ~0.5–0.6 ms
```

One reusable sidecar static BVH contains `1,831` members; the door snapshot
queries `22` current dynamic members and the platform snapshot `18`. Its
frustum sets and exact nearest rays match each snapshot's complete brute-force
current-geometry oracle.

Refit is not merely slower or unimplemented: the current lowering changes
stable member identity. The door grows from `1,849` to `1,853` triangles, and
the platform replaces identities even though its total count remains `1,849`.
A bounds-only refit therefore cannot honestly represent either endpoint under
the current member contract. Refit may return only if a future representation
proves membership stability.

The endpoint evidence leaves both immutable rebuild and dynamic sidecar viable.
Rebuild has the simpler artifact/revision lifecycle. Sidecar avoids rebuilding
the static hierarchy but requires an explicit stable/dynamic classification.
This endpoint-only disposition is superseded by the sequence and release
evidence below.

#### Slice 4 sequence and release evidence

The endpoint report now covers `19` immutable application revisions:

```text
door
    closed
    opening 25 / 50 / 75
    open
    closing 75 / 50 / 25
    closed again

platform
    high
    descending 25 / 50 / 75
    low
    waiting low
    ascending 75 / 50 / 25
    high again
```

Every immutable rebuild and static-plus-dynamic query matches the complete
current-geometry frustum/ray oracle. All `19` current revisions reject the
baseline revision, including repeated geometric states. Geometry structure
fingerprint and application snapshot revision are bound separately so closing
and opening at the same height cannot alias lifecycle identity.

The sidecar remains bounded across motion:

```text
static members         1,831
door dynamic members      18 closed / 22 moving
platform dynamic members  18 endpoints / 20 moving
```

Bounds-only refit is eligible only when geometry has returned exactly to the
baseline member identities. It is ineligible at every genuinely moving door
or platform snapshot because lowering adds or replaces members.

One release run observed full geometry preparation around `2.5–3.6 ms`, BVH
rebuild around `0.38–0.54 ms`, and sidecar extraction around `0.12–0.20 ms`.
A subsequent `20`-replay aggregate retained `380` snapshot samples:

```text
full geometry preparation mean          2.719 ms
immutable BVH rebuild mean              0.430 ms
immutable total update mean             3.149 ms
sidecar extraction mean                 0.133 ms
sidecar total update mean               2.852 ms
immutable query mean                    0.0568 ms
composite sidecar query mean            0.0602 ms
```

The sidecar saves approximately `0.297 ms` (`9.4%`) of the measured total
update while both paths still reconstruct the complete current geometry. It
also adds composite revision, conservation and query-union semantics and has a
slightly higher observed query mean.

Corpus disposition: use immutable replacement as the reference lifecycle for
the next portability experiment. Keep the sidecar as a measured optimization
candidate, but do not admit its composite lifecycle until a caller can produce
changed fragments directly or a performance budget makes the saved rebuild
material. This is not a shared Ring 2 decision.

### Slice 5 — Occlusion Experiment, Only If Earned

- [ ] Specify the exact occluder and occludee representations and prove that
      their bounds have authority over them.
- [ ] Compare front-to-back BSP traversal with a retained full-scene oracle.
- [ ] Treat planes, walls, cutouts, sky boundaries and dynamic geometry as
      separate evidence families where their semantics differ.
- [ ] Require a retained falsifier for every promoted rejection rule.
- [ ] Keep ambiguous coverage submitted.

Acceptance: no presentation-affecting experiment begins until Slices 1–4 show
zero unsafe geometric rejections and AR-0030 records the authorized authority
boundary.

### Slice 6 — Ring And Contract Decision

- [ ] Run a Quake comparison over native BSP/PVS data and actual prepared
      Tokimu geometry.
- [ ] Select one ordinary non-BSP caller and prove it is not forced to implement
      or understand BSP concepts.
- [ ] Compare provider-private duplication against a Ring 2 semantic contract.
- [ ] Answer why BSP rather than BVH; decide whether partition topology is
      Tokimu-owned meaning or only one provider's implementation technique.
- [ ] Decide whether the earned abstraction is geometry partitioning,
      visibility preparation, a smaller spatial-query primitive with BSP/BVH
      providers, or source-local machinery.
- [ ] If Ring 2 is earned, open/update the architectural decision before adding
      a crate, stable trait, capability descriptor or facade export.
- [ ] If it is not earned, retain the corpus-local implementation and record the
      falsification without weakening Ring 1 boundaries.

Acceptance: ownership is decided from Doom, Quake and non-BSP evidence—not from
the existence of one successful E1M1 implementation.

## Validation Matrix

| Axis | Required controls |
| --- | --- |
| Representation | whole draw, split fragment, triangle, degenerate input |
| Structure | BSP reference bake, BVH/control hierarchy, brute-force inventory |
| Camera | spawn, yaw sweep, movement path, neutral/up/down pitch, near wall |
| Query | frustum, ray, point/leaf location, unsupported/ambiguous input |
| Runtime | static, door closed/open/intermediate, floor/platform endpoints |
| Target | native interactive, native headless, browser/WASM feasibility |
| Correctness | brute-force geometry, global-full presentation, conservation |
| Cost | bake, memory, split amplification, refit/update, query latency |

Correctness evidence precedes optimization. A smaller candidate set is not a
success if it omits required presentation.

## Diagnostics And Evidence Requirements

Every bake should report at least:

```text
input members / triangles
output nodes / leaves / fragments
split amplification
maximum depth
contained / underbounded / unresolved
conservation balance
structural fingerprint
elapsed time and bounded memory estimate
```

Every query should report at least:

```text
bake revision
view/query identity
dynamic revision
visited nodes / leaves
retained / rejected / unresolved
reason counts
conservation balance
elapsed time
```

`LOOK` should be able to identify the original contribution, baked fragments,
leaf/path, geometric query result, Doom source result and any disagreement.

## Falsifiers And Return-Early Conditions

Return to architectural review before widening implementation if:

- safe useful queries require renderer materials, pipelines or submission
  ordering in the BSP contract;
- Doom SEG, subsector, visplane, sky or gameplay vocabulary appears necessary
  in a supposedly generic provider API;
- the provider would need to own or mutate world/runtime state;
- dynamic correctness requires an unbounded rebuild or silent stale fallback;
- split amplification or query cost materially exceeds the retained controls;
- native/WASM determinism requires incompatible semantic contracts;
- Quake and Doom need fundamentally different provider meanings;
- BSP supplies no required semantic value beyond a simpler BVH or conservative
  spatial-query provider;
- an ordinary non-BSP caller would be forced through the capability;
- presentation correctness still requires the coupled Doom wall/plane protocol
  after geometric candidate disagreements are removed.

Any of these may indicate that the right result is a provider-private BSP, a
smaller geometry capability, or no shared BSP capability at all.

## Explicit Non-Goals

- Exact Classic Doom pixel parity.
- Treating the new bake as a replacement WAD NODES lump.
- Rebaking canonical source assets as a repair.
- Making BSP the universal Tokimu world representation.
- Defining Tokimu BSP as a normalized copy of Doom or Quake file structures.
- Moving collision, navigation, physics or simulation truth into the BSP.
- Renderer-owned culling or source semantics.
- A general portal/PVS/occlusion framework before corpus evidence earns it.
- A stable serialization format or authoring API in the first slices.
- Performance optimization before containment and conservation are proven.

## Initial Decision Gate

Before Slice 1 implementation, update AR-0030 with the proposed pivot and
authorize one corpus-local Tokimu-first reference bake over actual E1M1
geometry plus a comparable BVH/control hierarchy. That authorization must not
pre-approve Ring 2, a new crate, a provider trait, BSP over BVH or
presentation-affecting rejection.

The first concrete question is intentionally narrow:

> Can a deterministic BSP over the exact prepared E1M1 geometry provide
> conservative actual-camera candidate queries with complete containment and
> contribution conservation?

If yes, the study proceeds to source correlation and runtime geometry. If no,
Tokimu has learned that a BSP is not the right generic spatial structure for
this representation before committing the concept to an engine ring.

## References

- `docs/ADR/ADR-0001-engine-boundaries.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/capability-backends.md`
- `docs/Tokimu Software Design Document.md`
- `docs/Architectural Reviews/AR-0025-camera-candidate-selection-and-visibility-culling.md`
- `docs/Architectural Reviews/AR-0030-source-owned-presentation-preparation-boundary.md`
- `docs/Plans/DOOM/Studies/Doom BSP presentation-domain resolver study.md`
- `docs/lessions/bounds-authority-follows-bounded-representation.md`
