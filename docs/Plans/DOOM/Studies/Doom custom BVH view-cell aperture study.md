# Doom Custom BVH View-Cell And Aperture Study

| Field | Value |
| --- | --- |
| Status | Parked after Slices 0–3 shadow falsifier — physical aperture transfer cannot safely own Doom presentation participation |
| Scope | Test whether a Doom-private hybrid of exact-geometry BVH queries and camera-relative aperture traversal explains E1M1 presentation participation |
| Parent review | [AR-0030](../../../Architectural%20Reviews/AR-0030-source-owned-presentation-preparation-boundary.md) |
| Spatial review | [AR-0031](../../../Architectural%20Reviews/AR-0031-conservative-spatial-query-capability.md) |
| Long-horizon comparison | [AR-0026](../../../Architectural%20Reviews/AR-0026-non-euclidean-spatial-charts-and-authored-angular-topology.md) — conceptual pressure only; no shared implementation admitted |
| Immediate predecessor | [Doom Render-Subsector Actual-Camera Preparation](Doom%20render-subsector%20actual-camera%20preparation.md), especially negative Slice 2B |
| Result checkpoint | [2026-08-18 Doom Custom BVH View-Transfer Shadow](../../../Checkpoints/2026-08-18-doom-custom-bvh-view-transfer-shadow.md) |
| Stable API authority | None |
| Renderer changes authorized | None |

## Question

Can the existing exact-geometry BVH be combined with a Doom-private view-cell
and directed-aperture model so that actual-camera preparation carries bounded
view regions rather than a single Boolean `reachable` result?

The study does **not** assume that adding azimuth metadata to a BVH fixes Doom
sky presentation. It separates four meanings that the earlier experiments
show must not be collapsed:

```text
BVH
    where finite geometry may intersect a query

cell connectivity
    which finite regions share boundaries

aperture view transfer
    which bounded part of this camera view can enter the next region

Doom presentation closure
    whether farther source contributions may participate in that bounded view
```

The first two are persistent world facts. View transfer is camera-relative.
Presentation closure remains Doom-owned policy and evidence.

The study uses the following careful characterization:

> Doom's map geometry is locally Euclidean, while its presentation semantics
> are not always faithfully described by one global 3D embedding.

Canonical E1M1 is not reclassified as authored non-Euclidean space. Its
ordinary render-subsector transitions remain in one coordinate domain with
identity transfer. The resemblance to charted space is useful because both
problems require path-qualified view evidence; it is not evidence that their
world semantics or implementations are the same.

## Why This Study Exists

The retained E1M1 BVH correctly reports all six exact ray targets as geometric
hits. Doom's ordered source oracle retains one and rejects five. The BVH is not
wrong; geometric relevance is deliberately weaker than presentation
participation.

Slice 2B then built a finite render-subsector connectivity graph:

- 237 cells;
- 607 shared-boundary relationships;
- no isolated cells;
- conservative traversal reaching 236 cells; and
- paired-sky-terminal traversal still reaching 233 cells.

Both policies reached all six target cell sets and disagreed with the ordered
source oracle on the five rejected targets. Only one rejected target's
shortest path crossed a paired-sky edge. Plain reachability and global
paired-sky terminality are therefore falsified as presentation resolvers.

The remaining hypothesis is narrower:

> The useful missing state may be the bounded portion of the actual view that
> survives each directed aperture, not another global property of a BVH node
> or a cell.

## Implemented Result

Slices 0–3 were authorized and executed as a corpus-private, shadow-only
experiment. Renderer submission remained unchanged.

The directed-aperture inventory retained the predecessor graph fingerprint
`13500e039c076c04` and added a separate aperture fingerprint
`3447a97c840c5a0f`:

```text
cells                         237
shared relationships          607
directed edges              1,214
source-correlated              305
traversable apertures          457
non-traversable boundaries     150
zero-clearance relationships    33
aperture containment failures    0
```

Actual-camera transfer preserved distinct path-qualified view states. The six
retained rays produced 632 total states, a peak of 306 for one view, and no
unbounded-growth failure. That made the state mechanics viable enough to test
the semantic hypothesis.

The hypothesis nevertheless failed on the positive control. The exact ray to
the required subsector 104 ceiling crossed these finite boundaries before its
BVH hit:

```text
boundary 39<>48   paired sky   ray z=30.96   opening=[-56,24]
boundary 48<>49   closed       ray z=32.68   opening=[-56,24]
boundary 49<>104  implicit     ray z=47.82   opening=[0,24]
```

The ray lies outside every physical opening, yet the exact BVH and brute-force
oracle hit the ceiling and Doom's ordered source protocol correctly retains
its presentation occurrence. This is not a graph-decoding, BVH-containment or
view-window clipping defect. A valid Doom presentation occurrence is not
necessarily describable as physical cell-to-cell view transfer in the inferred
global 3D embedding.

Across the six observations, aperture transfer reached 782 of 2,175 relevant
surfaces. Of the 1,393 surfaces outside its domain, 74 retained surfaces had to
be rescued by the complete ordered-source oracle. Conversely, 290 surfaces
inside reached cells were source-covered, so physical reachability cannot
safely grant participation either. Variant C matched all six controls only by
using the predecessor ordered-source oracle both outside and inside the
transferred domain.

```text
Variant A Boolean connectivity disagreements       5
Variant B bounded-transfer disagreements           1
Variant D paired-sky bounded disagreements         1
Variant C full ordered-fallback disagreements      0
matrix fingerprint                  d7071068f8b6b571
```

Disposition: retain the aperture inventory and view-transfer machinery as
diagnostic evidence only. Do not install it as Doom presentation authority or
extract it into Tokimu. Calling the extra admission region a "presentation
aperture" would merely rename the source-ordered protocol the study was meant
to simplify.

## Azimuth Finding

Azimuth is not normally a persistent property of world geometry. It depends on
camera position and orientation. A static azimuth interval attached to a BVH
member would be valid only for one observer or one explicitly defined local
chart.

The experiment may calculate camera-relative evidence such as:

```text
horizontal angular interval
vertical angular interval
near/far depth interval
facing and sidedness
projected aperture polygon
clipped child frustum or beam
```

Angular intervals are useful diagnostics and may be a compact conservative
query representation. They do not become persistent member identity or
presentation authority.

## Proposed Private Representation

The first experiment remains concrete and Doom-private. It does not introduce
a public `CustomBvh`, `ViewCell`, `Portal`, `Aperture` or `VisibilityProvider`
trait.

### Exact-geometry BVH

Reuse the current conservative BVH meaning:

```text
member
    stable prepared-geometry identity
    exact triangle or finite member geometry
    actual AABB
    immutable artifact/revision identity
    source contribution correlation
    render-subsector membership

node
    conservative member bounds
    child/member range
    optional conservative cell-membership summary
```

The BVH may reject geometry definitely outside an actual query. It cannot
declare geometry visible, source-authorized or occluded.

Candidate metadata worth testing as a sidecar or leaf correlation includes:

- owning render-subsector identity;
- source linedef, SEG, sidedef, sector and plane identity;
- contribution family and sidedness;
- finite surface normal or conservative normal cone;
- runtime geometry revision;
- touching directed-aperture identities; and
- ordinary/sky source role.

Mixed semantic metadata should not be aggregated into node-level rejection
authority. For example, a node containing both ordinary and sky-related
members cannot become a `sky node`.

### View-cell sidecar

Each cell is the finite render-subsector domain already established by the
active study:

```text
view cell
    cell identity
    finite boundary
    floor/ceiling height snapshot
    source/render-sector correlation
    BVH member identities
    outgoing directed aperture identities
    provenance and revision
```

Cell membership permits restricted BVH candidate queries. It does not mean
that every member in a reached cell participates.

### Directed aperture sidecar

An aperture represents a finite transferable boundary observation, not merely
an undirected graph edge:

```text
directed aperture
    source cell
    destination cell
    finite boundary segment or polygon
    bottom/top opening extent from the current runtime snapshot
    source-facing orientation
    source linedef/SEG/sidedef provenance
    boundary role
    runtime open/closed/unresolved state
    paired-sky annotation
    masked-middle annotation
```

The two directions may have different source evidence. A symmetric shared
boundary does not imply symmetric presentation behavior.

The initial boundary roles remain diagnostic:

- implicit partition;
- closed solid;
- positive opening;
- masked-middle opening;
- paired-sky opening; and
- unresolved fail-open.

They are inputs to view transfer. No role alone grants global rejection.

## Per-View Traversal State

The key change from Slice 2B is that traversal carries a bounded view region:

```text
ViewTransferState
    current cell
    clipped 3D view volume or conservative beam
    optional horizontal/vertical angular intervals
    minimum and maximum depth
    predecessor aperture
    boundary-chain identity
    accumulated source-closure evidence
    unresolved reasons
```

Multiple states may reach the same cell through different apertures. They must
not be merged merely because their cell identity matches. A merge is valid only
when the union remains conservative and preserves the closure evidence needed
to explain every later decision.

The general finding under observation is:

> Destination identity is insufficient to identify a view occurrence.

For Doom, the same render subsector reached through two aperture chains may
carry different bounded view regions and different presentation lineage. For a
future charted space, the same destination chart reached through different
transition paths may also carry different transformed cameras, orientations
and clipped views.

## AR-0026 Comparison Lens

The custom-BVH study may produce mechanics relevant to AR-0026, but code reuse
must be earned by independent corpus evidence. The following vocabulary is
tracked as a design lens rather than an API proposal:

| Candidate concept | Doom-private observation | Future AR-0026 observation |
| --- | --- | --- |
| Spatial domain or chart | One Euclidean E1M1 coordinate domain containing finite render subsectors | Explicit chart identity with local coordinates and local metric |
| View occurrence | Actual-camera view of one render subsector qualified by aperture chain and bounded region | View of a chart qualified by transition lineage and transformed local state |
| Directed transition | Directional finite aperture with identity coordinate transfer | Authored boundary with explicit source-to-destination transform |
| Transferred view | Clipped view volume passed into an adjacent subsector | Clipped view transformed into destination-chart coordinates |
| Presentation terminal | Bounded Doom-owned result that produces sky/background and no further world traversal | A possible non-spatial presentation result, distinct from an authored chart transition |

This comparison suggests a domain-of-validity rule broader than BVH bounds:

> Spatial, query and presentation evidence is authoritative only inside the
> representation and coordinate/presentation domain in which it was
> established.

For example, a future ray in chart A cannot be queried directly against a BVH
whose members and bounds are expressed in chart B. An explicit transition must
first produce the B-local ray and view occurrence:

```text
ray_A + view occurrence A
        ↓ transition T(A→B)
ray_B + transformed occurrence B
        ↓
BVH_B query in B coordinates
```

No such transform is needed for ordinary canonical E1M1 adjacency. Adding a
generic transition matrix to Doom merely because it may be useful later would
be speculative and would make the identity control look like evidence it does
not provide.

### Sky As A Presentation-Domain Transition

Sky may be usefully described as a transition in presentation without calling
it a spatial portal:

```text
ordinary Doom aperture
    → adjacent render-subsector view occurrence

future non-Euclidean transition
    → transformed destination-chart view occurrence

Doom sky presentation terminal
    → bounded sky/background occurrence
    → no implied destination spatial cell or chart
```

This model explains why sky is neither ordinary world geometry nor a portal to
another E1M1 room. It does not grant termination authority. A presentation
terminal must still be proved for the particular bounded view occurrence by
Doom-owned source evidence. `F_SKY1` or paired-sky identity alone remains
insufficient.

### Concrete-First Naming Fence

Any implementation authorized under this study should retain concrete names
such as:

```text
DoomRenderSubsector
DoomDirectedAperture
DoomViewTransferState
DoomPresentationTerminalObservation
```

A later AR-0026 corpus should independently discover its own concrete chart,
transition and view-occurrence representation. Similar shape in two documents
is not extraction evidence. Reusable vocabulary is considered only after both
implementations exist, their invariants are compared, and a smaller shared
meaning is demonstrated without importing Doom sky or source-coverage policy.

## Candidate Query Flow

```text
actual camera + viewport + runtime-height snapshot
        ↓
locate starting render subsector
        ↓
seed state with actual camera frustum
        ↓
intersect outgoing finite aperture with current clipped view volume
        ↓
empty intersection?
    yes → do not traverse that state
    no  → produce bounded child view state
        ↓
query exact-geometry BVH using child volume and destination-cell membership
        ↓
correlate candidates to exact source contributions
        ↓
apply Doom-private ordered/closure evidence inside the bounded view region
        ↓
retain, source-cover, outside-query or unresolved-fail-open
        ↓
ordinary render declarations + conservation evidence
```

The BVH and aperture traversal can reduce the set on which Doom-private
preparation operates. They do not replace that preparation.

## Sky Boundary

This experiment retains the accepted rule:

> Sky is source presentation meaning, not depth-bearing world closure
> geometry.

Consequently:

- paired sky annotates an aperture or closure observation;
- it does not create a physical occluder;
- it does not automatically stop all traversal through that cell boundary;
- it cannot reject geometry reached through an unrelated aperture; and
- any terminal decision must name the exact bounded view region and source
  evidence it closes.

The study must distinguish these possible outcomes:

```text
aperture transfer alone distinguishes the six rays
    → retain as evidence, then test broader positive controls

aperture transfer narrows candidates but source closure remains necessary
    → likely healthy division of labor

only a special paired-sky terminal rule distinguishes them
    → test for clipped hut/far-left controls before granting authority

all variants still reach the five rejected targets
    → park the hybrid as a presentation resolver
```

## Experimental Variants

### Variant A — BVH plus cell membership

Restrict exact BVH candidates to cells reached by the conservative graph.
This is a control and is expected to reproduce the Slice 2B failure.

### Variant B — Geometric aperture transfer

Carry an actual clipped view volume through positive, masked, paired-sky and
implicit apertures. Closed boundaries stop transfer. Unresolved boundaries
fail open and retain their reason.

No Doom source-coverage or sky-terminal decision is permitted in this variant.
If this variant still reaches all five rejected targets, aperture transfer is
falsified only as an **independent presentation resolver**. Continue to Variant
C: the transfer may still establish a smaller, bounded domain in which Doom
closure can operate.

### Variant C — Bounded Doom closure shadow

Apply existing Doom-private ordered source evidence only within each surviving
view-transfer region. Report exactly which source contribution and bounded
region closes later participation.

This variant is still shadow-only. It must not reuse Classic screen-row cells
as final pitched-camera geometry.

Variant C must report whether aperture transfer materially reduces or
localizes the ordered work compared with the complete predecessor protocol:

- source occurrences considered before and after transfer;
- bounded view regions receiving ordered closure;
- maximum and total ordered coverage state;
- whether exact terminal authority is simpler than full-screen replay; and
- which predecessor machinery remains necessary.

Rejecting the five rays is insufficient if Variant C merely reconstructs the
entire Classic protocol behind different names.

### Variant D — Paired-sky falsifier

Treat paired-sky closure as terminal only inside the exact transferred view
region that encountered it. Compare this to Variant C and to positive hut,
far-left and partial-plane controls. This exists to falsify overly broad sky
authority, not to encode the desired answer.

## Required Diagnostics

Every observed contribution should report:

```text
camera/view fingerprint
runtime-height revision
BVH artifact/revision
source and target cell
aperture chain
per-aperture source identity and role
input and output clipped-view fingerprints
BVH candidate/hit identity
ordered source disposition
terminal or fail-open reason
final shadow disposition
```

Required summary ledgers:

- input geometry and source-contribution conservation;
- states created, merged, terminated and unresolved;
- per-variant total/peak live states, distinct path lineages, repeated
  destination cells and maximum occurrences of one cell;
- BVH candidates before and after cell/aperture restriction;
- aperture role counts and maximum traversal depth;
- exact reasons for every rejected contribution;
- disagreements against brute-force geometry and ordered source oracles; and
- deterministic fingerprints over graph, BVH, runtime snapshot and view
  results.

## Corpus Matrix

The first matrix must include:

- the six exact retained BVH/source rays;
- source spawn at neutral pitch;
- bounded pitch up/down and yaw movement;
- the known complete hut and far-left geometry controls;
- beside-hut and above-wall leak views;
- the retained partial subsector 104 ceiling;
- green-room masked-middle cutout;
- first door and moving-platform runtime snapshots;
- near-wall and off-axis SCAN/LOOK samples; and
- return-to-identical-pose determinism.

An implementation is not successful merely because it rejects five named
rays. It must retain required nearby and partial geometry while explaining the
distinction with bounded evidence.

## Proposed Slices

### Slice 0 — Freeze The Shadow Contract

- [x] Reuse the existing BVH, render-subsector and six-ray identities.
- [x] Define a private directed-aperture inventory and conservation ledger.
- [x] Define clipped-view state and deterministic fingerprinting.
- [x] Keep renderer submission unchanged.

Acceptance: the new report can reproduce the Slice 2B graph and BVH controls
without changing their fingerprints.

### Slice 1 — Directed Aperture Inventory

- [x] Correlate shared finite boundaries with exact source identities or an
      explicit implicit/unresolved reason.
- [x] Calculate runtime bottom/top opening extents and directionality.
- [x] Enumerate implicit, multiple-linedef and malformed cases explicitly.
- [x] Prove that aperture geometry lies on both incident cell boundaries.

Acceptance: every traversable edge is supported or unresolved fail-open; no
source role is inferred from texture appearance alone.

### Slice 2 — Actual-Camera View Transfer

- [x] Propagate conservative clipped view windows through directed apertures.
- [x] Preserve multiple distinct states reaching the same cell.
- [x] Correlate reached cells and windows with prepared-member identities.
- [x] Compare the exact six-ray spatial results with unrestricted BVH and
      brute force.

Acceptance: zero false-negative geometric candidates and deterministic state
and chain fingerprints across retained poses. State growth is reported before
any merging optimization is proposed; an embarrassing count is evidence, not
permission for an unexplained lossy merge.

Result: failed. State growth was bounded, but the required subsector 104
ceiling occurrence is outside the physically transferred aperture domain.

### Slice 3 — Doom Closure Shadow

- [x] Measure source-ordered outcomes inside and outside bounded transferred
      regions.
- [x] Compare Variants B, C and D on the decisive six-ray matrix.
- [x] Retain exact terminal and unresolved reasons.
- [x] Make no presentation-affecting omission.

Acceptance: the shadow either distinguishes the retained/rejected controls
without geometric false negatives or provides a precise falsifier explaining
why it cannot.

Result: the precise falsifier was obtained. The larger movement, runtime and
visual matrix was not run because continuing could not recover presentation
authority without replaying the complete ordered-source protocol.

### Slice 4 — Architectural Review Gate

- [x] Decide whether the useful result is Doom-private preparation machinery,
      a reusable view-cell/aperture capability, or no useful resolver.
- [x] Return to AR-0031 before moving portal, aperture, clipped-view or
      occlusion vocabulary into an engine crate or public contract.
- [x] Return to AR-0030 before changing source-presentation authority or live
      renderer submission.

### Optional Post-Slice-3 AR-0026 Comparison — Separately Authorized

This comparison is not part of the currently proposed Doom implementation.
After the Doom shadow has produced evidence, a separately authorized AR-0026
fixture may:

- [ ] reuse a small E1M1 room/doorway/corridor subset only as local chart
      content, following AR-0026's one-way topology-donor rule;
- [ ] compare an identity chart transition with an explicitly authored
      translated/rotated transition over otherwise identical local geometry;
- [ ] query a destination-local BVH only after transforming the camera, ray and
      clipped view into that chart;
- [ ] reach one destination chart through two paths and prove distinct view
      occurrence identities and results;
- [ ] compare occurrence lineage, state growth, cycle handling and domain
      authority with the Doom aperture report; and
- [ ] keep Doom presentation terminals out of the chart-transition model
      unless an independent non-Doom caller demonstrates the same meaning.

Useful outcomes include shared diagnostic notation, a common conservation
pattern or a falsifier for global-camera/global-BVH assumptions. This optional
comparison does not authorize a shared crate, public trait, renderer portal
contract or canonical-WAD reinterpretation.

## Falsification And Parking Criteria

Park Variant B as an independent presentation resolver if:

- bounded aperture transfer still reaches all five rejected contributions.

Continue through Variant C after that result. Park the complete hybrid as a
presentation resolver if:

- bounded aperture transfer provides no meaningful candidate reduction,
  authority localization or diagnostic simplification for Doom closure;
- Variant C effectively replays the complete Classic ordered protocol rather
  than applying a smaller bounded closure problem;
- distinguishing them requires global cell or paired-sky rejection;
- required hut, far-left, spawn-room or partial-plane geometry is clipped;
- correctness depends on treating sky as physical depth geometry;
- state growth is unbounded or requires unexplained lossy merging;
- runtime doors/platforms cannot revise aperture state without stale results;
- the BVH must own Doom-specific presentation roles; or
- renderer submission must understand cells, portals, BSP or Doom semantics.

Ordinary implementation defects in clipping, source correlation,
fingerprinting or conservation remain local work. A need for a stable generic
aperture contract, new engine ownership or cross-cutting runtime authority is
an architectural finding and returns to AR-0031.

## Explicit Non-Goals

- No WAD BSP rebake.
- No replacement of the decoded Classic BSP as source evidence.
- No generic portal renderer.
- No precomputed potentially-visible-set contract.
- No software occlusion rasterizer unless separately authorized by evidence.
- No sky depth mask or invisible wall.
- No public trait, new engine crate or facade export.
- No assumption that Doom and Quake mean the same thing by cell, leaf or
  portal.
- No claim that canonical Doom is authored non-Euclidean space.
- No generic chart/transition abstraction extracted from the Doom experiment
  before independent AR-0026 corpus evidence exists.

## Expected Value Even If Falsified

A negative result still answers a useful question: whether actual finite
aperture geometry and camera-relative view transfer add semantic information
beyond conservative BVH queries and Boolean connectivity. The retained chains,
clipped-view fingerprints and source disagreements would prevent later engine
work from promoting spatial relevance into presentation authority under a new
name.
