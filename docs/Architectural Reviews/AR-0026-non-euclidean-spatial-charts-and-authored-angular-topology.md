# AR-0026: Non-Euclidean Spatial Charts and Authored Angular Topology

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-08-10 |
| Last reviewed | 2026-08-11 |
| Scope | Long-horizon spatial semantics, presentation, traversal, and authoring evidence |
| Trigger | Maintainer interest in authored maps or objects whose spatial closure is not globally Euclidean, including a junction where a complete circuit has 520 standard angular degrees rather than 360. |
| Related ADRs | ADR-0001, ADR-0003, ADR-0008, ADR-0009 |
| Related reviews | AR-0019, AR-0021, AR-0025 |
| Related evidence | Future `hello-non-euclidean-junction` corpus; `hello-doom-e1m1` source-traceable sector/subsector geometry; AR-0025 portal/non-Euclidean falsification pressure |
| Admission exception | None |

## Architectural Question

Can Tokimu eventually support authored spaces assembled from locally Euclidean
charts with explicit adjacency and transition maps, including angular
deficit/excess junctions, without making a global Euclidean transform hierarchy,
one-world-position model, or provider-specific rendering mechanism the source
of truth? If so, what smallest provider-neutral semantic boundary is actually
useful across games, CAD/topology tools, simulation, and visualization?

## First-Class Capability Hypothesis

The maintainer's long-term ambition is stronger than a portal renderer or a
special-case corpus: **non-Euclidean simulation may become first-class Tokimu
meaning.** If corpus evidence earns that direction, Tokimu would own the
provider-neutral semantics of spatial charts, adjacency, transition maps,
angular topology, entity location, traversal, and query continuity. Rendering,
physics/query providers, picking, audio, navigation, editors, and replication
would consume that shared spatial truth rather than inventing separate portal
rules.

Conceptually, this pressures the ordinary assumption:

```text
position = Vec3
```

toward a semantically qualified location:

```text
location = chart identity + local coordinates
```

and explicit transitions for locations, directions, orientations, and queries.
This notation is a research hypothesis, not a proposed public type or layout.

"First class" means that Tokimu owns the meaning and cross-system invariants.
It does not yet mean the entire feature belongs in Ring 0, that every mechanism
must be Tokimu-authored, or that all applications must use charted space. Ring
placement and provider boundaries require separate ADR-0003 admission evidence,
and the ordinary Euclidean path must remain straightforward and efficient.

## Terminology and Necessary Distinction

"A circle is 520 degrees" can describe two different things. This review must
not confuse them:

1. **A unit convention.** If one ordinary Euclidean revolution is merely
   labelled `520` rather than `360`, only the angular unit changed. This is a
   coordinate/display convention and does not require non-Euclidean semantics.
2. **Authored angular excess.** If locally ordinary sheets whose standard
   Euclidean angles total 520 degrees are cyclically identified around one
   junction, a circuit closes only after those 520 degrees. This is an
   angular-excess, piecewise-flat spatial junction; it is the non-Euclidean
   case this record studies. A 240-degree closure would similarly express
   angular deficit.

The semantic fact is therefore not a mutable `Circle { degrees: 520 }`. It is
the local metric/topological neighbourhood: incident regions, their cyclic
order, angular measures, and the boundary-identification rules that close the
junction.

## Context and Why This Is an Architectural Question

Current Tokimu corpus rendering relies on ordinary camera transforms and
locally Euclidean mesh positions. That is appropriate for current evidence.
It is not evidence that every future Tokimu world has one global Euclidean
coordinate system.

Non-Euclidean authoring challenges otherwise easy assumptions:

- one world coordinate has one globally sufficient presentation location;
- one camera has one globally sufficient frustum traversal;
- global AABBs establish all candidate selection;
- shortest paths, rays, and picking remain in one homogeneous space;
- a transform hierarchy fully owns spatial adjacency; and
- an opaque-looking surface is necessarily an ordinary global occluder.

AR-0025 already records portal-transformed and recursive views as falsification
pressure for a global camera/AABB visibility contract. This record owns the
larger spatial-semantics question rather than allowing portals, unusual angular
closure, and renderer tricks to collapse into one undocumented feature.

AR-0019 separately studies ownership of Tokimu's public vector, quaternion,
and matrix vocabulary. AR-0026 may eventually pressure that decision because
local coordinates and chart transitions need semantic qualification that a
bare global `Vec3` or `Mat4` cannot supply. It does not currently justify
expanding or replacing the five-type math vocabulary: chart identity and
transition meaning may belong above the ordinary math mechanics.

## Ownership and Dependency Direction

- An authored spatial model owns charts/regions, local coordinates, adjacency,
  transition maps, angular neighbourhoods, and any declared closure rule.
- Simulation owns entity state, traversal decisions, collision policy, time,
  and gameplay/tool truth. Presentation must not mutate that truth to make an
  impossible layout look plausible.
- A view/presentation adapter may realize caller-declared local views and
  transitions. It must not infer topology from mesh coincidence, asset names,
  or a graphics provider's coordinate conventions.
- A graphics provider realizes explicit draw/view declarations. WGPU/WebGPU,
  native windowing, and shader implementation details cannot become spatial
  vocabulary.
- An editor may visualize, inspect, and author spatial facts, but its flattened
  embedding is diagnostic/presentation evidence, not semantic truth.

```text
authored charts + transitions + local metric/topology
    -> simulation traversal / caller-declared view instances
    -> presentation candidate domains and explicit draws
    -> renderer/provider realization

not proposed:
global transform hierarchy -> hidden topology truth
graphics provider -> spatial semantics
flattened editor embedding -> authoritative world geometry
```

## Initial Corpus Ladder

The first corpus must stay deliberately small. It is not a general portal,
physics, or renderer program.

| Stage | Corpus evidence | What it can establish | Explicit non-claim |
| --- | --- | --- | --- |
| 0 | Ordinary 360-degree locally Euclidean control | Existing local chart, render, and traversal baseline | No new spatial capability |
| 1 | 240-degree deficit and 520-degree excess junction | Explicit angular closure and cyclic region ordering are representable as authored data | Global curvature, general physics, or arbitrary manifolds |
| 2 | Deterministic walk trace across identified boundaries | Traversal uses adjacency/transition rules rather than a guessed global position | General navigation/pathfinding |
| 3 | Local ray/picking trace crossing a boundary | A caller can transform a query across an explicit transition | A universal physics raycast API |
| 4 | One portal-style derived local view | Visibility is local to a caller-declared view/candidate domain | Recursive rendering or renderer-owned portal semantics |
| 5 | Editor/diagnostic flattened view of sheets and junctions | Editor embedding can explain authored topology without owning it | WYSIWYG global Euclidean truth |

Each stage must retain an ordinary control, a deficit case, and an excess case;
fixed traces; source identity; transition identity; expected local region; and
observed presentation. A 520-degree label alone is insufficient evidence.

## Doom Corpus as a Topology Donor

The existing Doom corpus can supply bounded local geometry and source topology
without making Doom itself non-Euclidean. The reusable evidence already
includes source-traceable walls and flats, linedef/sidedef/sector/subsector
identity, BSP paths and regions, a resolved player spawn, ordinary local camera
observation, texture/material preparation, and native/browser presentation.

The first AR-0026 integration should extract a small E1M1 subset—one room, a
doorway/boundary, a corridor, and a second room—as locally Euclidean chart
content. It must not reinterpret the canonical WAD or claim new Doom semantics.
The causal comparison keeps local geometry, texture identities, and observer
inputs fixed while changing only chart identity, adjacency, or an explicit
boundary transition:

```text
canonical control:
room A -> identity transition -> room B

experimental case:
same room A -> declared rotation/translation -> same local room B
```

The experimental fixture may duplicate or re-identify extracted topology so
two charts overlap in a flattened diagnostic embedding while remaining
semantically distinct. Later variants may build a nontrivial transition loop
and a 520-degree angular-excess junction from locally ordinary regions.

This is one-way reuse:

- the Doom/WAD providers remain authoritative only for decoded source records;
- an AR-0026 corpus adapter explicitly creates chart-local experimental data;
- chart/transition semantics do not enter `doom-map-provider`,
  `doom-geometry-provider`, or the canonical E1M1 preparation path; and
- the renderer continues to receive generic view, mesh, material, and draw
  declarations rather than Doom or non-Euclidean topology.

The same fixture can later falsify a universal reading of AR-0025: place chart
B outside the primary view's global Euclidean frustum while presenting it
through a caller-declared transition-derived view. Global AABB rejection then
remains correct only for the primary candidate domain, not for all derived
views.

## Candidate Semantic Directions

### A. Retain Globally Euclidean Worlds Only

- Benefits: small, familiar transform/camera/spatial-index model.
- Costs: cannot express the maintainer's intended angular-excess or chart
  topology without presentation deception.
- Failure mode: portal-like features accumulate as hidden exceptions.

### B. Coordinate-Unit Convention Only

- Benefits: supports alternate author-facing angle labels at negligible cost.
- Costs: does not change geometry; cannot model a true 520-standard-degree
  closure.
- Failure mode: a display unit is mistaken for non-Euclidean semantics.

### C. Application-Owned Local Charts and Explicit Transition Maps

- Benefits: a source-neutral way to represent local coordinate domains and
  declared mappings; can inform games, topology/CAD tools, simulation spaces,
  inspectors, portals, minimaps, and multi-view visualization.
- Costs: forces explicit ownership choices for traversal, views, picking,
  collision, and serialization.
- Failure mode: charts become a premature universal scene graph or expose
  provider-specific matrices/resources as application truth.

### D. Renderer-Owned Portal/Non-Euclidean Mechanism

- Benefits: potentially convenient for one visual effect.
- Costs: conflates topology, simulation, and rendering; cannot establish
  traversal/collision/picking truth.
- Failure mode: presentation-only tricks silently become world semantics.

### E. General Differential-Geometry or Manifold Subsystem

- Benefits: broad mathematical reach.
- Costs: vastly exceeds current callers and makes ownership, performance, and
  verification obligations enormous.
- Failure mode: speculative mathematics becomes an unmaintainable Ring 0 or
  rendering dependency.

## Initial Findings

1. The maintainer's intended 520-degree case is meaningful only if it denotes
   authored angular excess, not a renamed angular unit.
2. Locally Euclidean charts plus explicit adjacency and transition rules are a
   more promising future seam than a global non-Euclidean renderer feature.
3. The potential general value is not limited to games: it could support
   topology-aware CAD/editor diagnostics, spatial simulation, multi-view
   visualization, and applications with disconnected or transformed local
   coordinate domains.
4. No existing corpus demonstrates the required semantic ownership, validation,
   performance, serialization, or failure-containment evidence. Nothing is
   admitted by this record.
5. A coherent first-class result would be broader than rendering: simulation
   location, traversal, collision/query, picking, visibility, navigation,
   audio propagation, serialization, editing, and replication would need to
   agree on the same chart/transition truth or explicitly declare their limits.

## Disposition

**Incubating.** Retain the long-horizon hypothesis and build a narrowly scoped
future corpus before proposing any Tokimu spatial capability. Current Euclidean
camera, mesh, and renderer paths remain valid for their declared workloads.
Do not add a chart, portal, manifold, alternate-angle, renderer recursion, or
physics API from this record.

## Required Follow-Up

- [ ] Write a `hello-non-euclidean-junction` plan that distinguishes coordinate
      relabelling from actual angular deficit/excess topology.
- [ ] Add a bounded E1M1 topology-donor intake to that plan: select and retain
      one room/boundary/corridor/room subset with source identities and prove an
      identity-transition chart representation reproduces the ordinary control.
- [ ] Keep the canonical WAD packages, Doom providers, and E1M1 preparation
      semantics unchanged; create experimental chart topology only in an
      AR-0026 corpus adapter.
- [ ] Change exactly one declared boundary transition while retaining local
      geometry, materials, and observer input; demonstrate displaced/rotated
      emergence and a self-overlapping-but-chart-distinct layout.
- [ ] Establish a first-party semantic data model only inside the corpus:
      local region identity, local coordinates, cyclic adjacency, transition
      identity, and declared local angular measures.
- [ ] Require each experimental chart transition to declare and test whether
      it is orientation-preserving or orientation-reversing independently of
      invertibility and round-trip success. Do not infer authored transition
      meaning solely from raw matrix mechanics.
- [ ] Implement 360-degree, 240-degree, and 520-degree junction controls with
      deterministic traversal traces and explicit expected closure evidence.
- [ ] Add boundary-crossing query/picking evidence without changing generic
      physics or renderer contracts.
- [ ] Add one caller-declared derived local view and prove AR-0025's global
      AABB/frustum experiment is not overclaimed as universal visibility.
- [ ] Retain diagnostics for invalid/non-invertible transition declarations,
      recursion/cycle limits, missing target charts, and ambiguous boundary
      selection. Do not silently choose a transition.
- [ ] Establish a second independent non-game use case before proposing a
      provider-neutral capability. Candidate pressure includes topology/CAD
      inspection, spatial simulation, or multi-view technical visualization.
- [ ] Inventory cross-system invariants for entity location, traversal,
      collision/query, visibility, picking, navigation, audio, serialization,
      editing, and replication; distinguish required first-class semantics from
      mechanisms that can remain replaceable providers.
- [ ] Demonstrate that an ordinary single-chart Euclidean application remains a
      simple, bounded path rather than paying unavoidable chart-recursion or
      transition costs.
- [ ] Feed only measured local-coordinate and transition operations back into
      AR-0019. Determine whether they pressure ordinary math mechanics, require
      spatial semantic wrappers above math, or both; do not add speculative
      manifold operations to the five-type study.
- [ ] Apply ADR-0008, ADR-0009, ADR-0010, and ADR-0011 before any Native Ring,
      security-sensitive, hot-path, or stable cross-provider contract proposal.

## Reopening Triggers

- a working junction corpus demonstrates that a corpus-local chart/transition
  model is coherent and useful;
- an independent non-game caller needs the same semantic facts;
- current global coordinate, camera, picking, spatial-index, or visibility
  assumptions block a valid bounded experiment;
- an implementation requires a public transform, serialization, runtime,
  renderer, physics, or provider contract; or
- evidence shows that the desired effect is only a coordinate convention or
  presentation trick, not a durable spatial semantic need.

## Review History

### Cycle 1 -- 2026-08-10

- Status entering review: Proposed.
- New evidence: maintainer identified a desired authored angular-excess case in
  which a circuit around a junction closes at 520 standard degrees, together
  with future portal/non-Euclidean corpus interest.
- Findings: distinguish alternate angle units from true angular topology;
  retain local charts and explicit transitions as a hypothesis, not an API.
- Disposition: Incubating; a tiny controlled corpus is required before any
  shared capability or subsystem discussion.
- Resulting ADR or documentation change: none.

### Cycle 2 -- 2026-08-10

- New relationship: AR-0026 may pressure AR-0019's public math-vocabulary
  decision because local coordinates and chart transitions need qualified
  spatial meaning beyond unadorned global vectors and matrices.
- Findings: that pressure currently argues for keeping semantic spatial types
  separable from ordinary math mechanics; it does not authorize new math types,
  a `glam` replacement, or a public chart representation.
- Disposition: retain Incubating status and feed only operations demonstrated
  by a future junction corpus into AR-0019.

### Cycle 3 -- 2026-08-10

- Maintainer direction: explore non-Euclidean simulation as a distinctive
  first-class Tokimu capability rather than merely a renderer portal effect.
- Findings: first-class status would require one Tokimu-owned spatial truth
  consumed consistently across simulation and presentation systems. It does
  not yet determine Ring placement, implementation ownership, or API shape.
- Disposition: retain Incubating status; treat first-class semantic ownership
  as the north-star hypothesis for the corpus ladder, with ordinary Euclidean
  use preserved as the default bounded case.

### Cycle 4 -- 2026-08-10

- New corpus opportunity: existing E1M1 evidence can donate a small,
  source-traceable room/boundary/corridor/room topology, observer spawn, and
  presentation path to the first AR-0026 experiment.
- Guardrail: Doom remains an ordinary Euclidean source. An AR-0026 adapter owns
  all chart extraction, re-identification, and transition mutation; canonical
  WAD providers and E1M1 semantics remain unchanged.
- Disposition: use the Doom subset as a realistic control after the minimal
  synthetic junction fixture, then compare identity, transformed, overlapping,
  loop-closure, picking, and derived-view cases.

### Cycle 5 -- 2026-08-11

- New evidence: AR-0028 produced a direct, invertible Doom-to-world lift that
  reverses both a canonical landmark determinant and the source-right versus
  camera-right side relation.
- Findings: future chart transitions must distinguish invertibility from
  orientation behavior. An orientation reversal may be a provider defect in a
  Euclidean import or intentional authored meaning in a charted space; the
  transition declaration and its caller context must make that distinction
  explicit.
- Disposition: retain Incubating status. Add orientation-preserving versus
  orientation-reversing evidence to the future corpus without proposing a
  chart API or changing ordinary Euclidean math.

## References

- `docs/ADR/ADR-0001-engine-boundaries.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/Architectural Reviews/AR-0019-native-math-vocabulary-and-foreign-type-boundary.md`
- `docs/Architectural Reviews/AR-0025-camera-candidate-selection-and-visibility-culling.md`
- `docs/Plans/DOOM/DOOM WAD Checklist.md`
- `corpus/hello-doom-e1m1/src/lib.rs`
- `docs/Tokimu Software Design Document.md`
