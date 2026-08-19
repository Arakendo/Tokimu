# Doom Oriented Sky-Transition Parity Shadow Study

| Field | Value |
| --- | --- |
| Status | Parked at Slice 1 semantic gate — Doom evidence supplies open one-way sky surfaces, not a closed transition volume |
| Scope | Test whether ordered, source-proven sky-transition events can reproduce the desired E1M1 sky/world presentation over complete persistent geometry |
| Parent review | [AR-0030](../../../Architectural%20Reviews/AR-0030-source-owned-presentation-preparation-boundary.md) |
| Geometry oracle | `global-full-submission` |
| Immediate predecessor | [Doom Source-Occurrence Support Over Reconstructed Geometry](Doom%20source-occurrence%20support%20over%20reconstructed%20geometry.md) |
| Causal evidence | [Doom Source-Ordered Non-Presentation Causality Study](Doom%20source-ordered%20non-presentation%20causality%20study.md) |
| Prior sky experiment | [Doom Authoritative Sky-Coverage Delta Realization](Doom%20authoritative%20sky%20coverage%20delta%20realization.md) |
| Stable API authority | None |
| Renderer changes authorized | None; shadow diagnostics only if separately authorized |

## Question

Can Doom-private preparation classify the state along an actual Tokimu camera
ray by ordered, oriented semantic sky transitions while retaining the complete
ordinary world geometry as its geometry oracle?

The proposed state machine is:

```text
camera ray begins in proved initial state
        ↓
ordered semantic transition events

World + Enter → Sky
Sky   + Exit  → World

0 proved transitions before ordinary hit → World
1 proved transition                      → Sky
2 proved transitions                     → World
3 proved transitions                     → Sky
```

This is a parity hypothesis only after semantic entry/exit events have been
proved. It is explicitly **not** raw `sky triangle intersections % 2`.

## Review Disposition

Monday review endorses the hard falsifiers, ambiguity policy, complete-world
baseline and historical-causality fence, and recommends authorizing shadow-only
Slices 0–2 exactly as written. This review does not itself start implementation.

The operational decision tree is binding:

```text
Slice 1: can semantic Enter and Exit be proved?
    no  → park parity or rename the surviving one-way hypothesis
    yes ↓

Slice 2: do all ten exact controls agree?
    no  → falsified; do not add per-ray exceptions
    yes ↓

Slice 3: can a genuine Exit and adversarial controls survive?
    no  → bounded one-way mask at best
    yes → return complete evidence to AR-0030
```

Slice 3 is conditional work, not pre-authorized by the recommendation. Slice
1 may terminate the study before the ten-ray parity gate if Doom cannot supply
honest oriented transitions. Unknown initial state remains `Ambiguous → World`;
it cannot be selected merely to improve a screenshot.

## Why This Study Exists

The source-cell live candidate established two useful facts before it was
parked:

1. global-full remains the only current representation that consistently
   supplies nearby ordinary geometry under arbitrary free look; and
2. the blue panorama seen through its holes had no sky authority. Five exact
   rays had `sky_boundary=none`, `source_sky_plane=none`, and a nearby
   global-full floor or ceiling at distances from `56.203` through `138.783`.

The original far-field resurrection specimens differ. They contain complete
ordinary geometry that Classic Doom did not finally present, and several have
a nearer source sky boundary before the unwanted hit. Classic's actual causal
exclusion was ordered solid coverage, not sky. Nevertheless, a positively
proved sky transition might be a smaller free-look realization of the desired
presentation result.

The study therefore tests an implementation-equivalence hypothesis:

> Tokimu may be able to reproduce the desired free-look presentation using
> Doom-private sky-transition state even though sky did not cause Classic
> Doom's original ordered rejection.

It must never rewrite the retained causal history as “Doom rejected the room
because the ray entered sky.”

## Working Representation

The shadow may use a corpus-private observation shaped conceptually like:

```text
SkyTransitionObservation {
    ray_distance
    orientation
    source_boundary_identity
    source_provenance
    role: Enter | Exit | Ambiguous | NonTransition
}
```

This is study vocabulary, not a proposed public Rust type or renderer command.

Only `Enter` and `Exit` modify state. `Ambiguous` and `NonTransition` are
retained in the report but fail open to ordinary world presentation.

## Semantic Boundary Requirement

Doom does not supply a guaranteed watertight sky volume. Its evidence may
include:

- `F_SKY1` ceiling regions;
- intentionally suppressed upper walls between paired sky ceilings;
- disconnected sky sectors;
- open presentation apertures with no physical closure; and
- source relationships whose triangles are only diagnostic realizations.

Consequently a geometric intersection is only a candidate observation. It
becomes a transition event only when Doom source semantics establish all of:

1. a stable semantic boundary identity;
2. which side is World and which side is Sky;
3. a usable orientation at the exact intersection;
4. whether the crossing is an entry or exit; and
5. provenance sufficient to reproduce and audit the decision.

For an oriented proved boundary:

```text
ray · boundary_normal < 0 → candidate Enter
ray · boundary_normal > 0 → candidate Exit
approximately tangent     → NonTransition or Ambiguous
```

The sign alone does not grant semantic authority. Source meaning must first
prove that the surface represents a transition.

## Duplicate And Degeneracy Policy

Raw triangle intersections cannot toggle parity directly. A conceptual quad
may be triangulated twice, adjacent sky regions may overlap, and a ray may hit
an edge shared by several triangles.

Candidate hits are grouped using at least:

```text
same semantic boundary identity
+ approximately equal ray distance
+ compatible orientation
```

One group may produce at most one transition. The shadow records:

- raw triangle hits;
- grouped boundary hits;
- collapsed duplicates;
- tangent/non-transition groups;
- conflicting orientations;
- overlapping distinct boundary identities; and
- unresolved groups.

Malformed sequences do not silently toggle:

```text
Exit while in World
Enter while already in Sky
two incompatible events at one distance
unknown initial state
```

Each becomes `Ambiguous` and the presentation decision fails open to World.

## Initial State

The default initial state is World only when the camera is not proved to begin
inside a semantic sky domain. A camera under, within, or beyond an open sky
region requires explicit evidence. If the initial state cannot be proved, the
ray result is `Ambiguous → World` during this study.

## Spatial Query Boundary

AABB, frustum and BVH mechanisms have a deliberately limited role:

```text
AABB / frustum
    conservative broad-phase candidate selection

BVH
    accelerated exact ray candidate queries

Doom-private transition resolver
    semantic identity, orientation and state
```

The brute-force exact intersection result remains the oracle for the BVH.
Neither an AABB nor a BVH node may:

- infer Enter or Exit;
- decide that something is “behind sky”;
- reject ordinary geometry;
- repair an unresolved transition; or
- acquire Doom sky vocabulary in a stable Tokimu spatial contract.

## Frozen First-Gate Matrix

The first report uses ten retained exact rays.

### Far-field resurrection candidates

These are the five original exact negative specimens from the ordered causal
study:

- `hut-east-wall-230`;
- `wall-247-east`;
- `wall-247-west`;
- `ceiling-149-rejected`; and
- `ceiling-104-rejected`.

The hypothesis expects state `Sky` immediately before each unwanted
global-full hit. A ray without a proved odd transition is a hypothesis failure,
not permission to synthesize one.

### Required ordinary geometry controls

These are the five newly retained holes from the source-cell walkabout:

- spawn sector 38/subsector 103 floor, distance `56.365`;
- spawn sector 38/subsector 103 ceiling, distance `56.203`;
- sector 38/subsector 114 floor, distance `114.443`;
- sector 2/subsector 116 floor, distance `88.605`; and
- sector 12/subsector 29 floor, distance `138.783`.

All five currently report neither a sky boundary nor a source sky plane. The
hypothesis expects `World` before the required ordinary hit. Any event that
makes one of these rays `Sky` falsifies the candidate.

## Required First Report

For every frozen ray, report:

```text
case
global-full nearest ordinary target and distance
raw candidate sky intersections
grouped semantic boundary observations
ordered proved events:
    [distance, identity, orientation, role, provenance]
initial state and its authority
state immediately before ordinary target:
    World | Sky | Ambiguous
expected state
match
```

The report must include a deterministic fingerprint and prove BVH/brute-force
agreement over the exact same semantic-boundary members.

## Binding Invariants

1. Global-full ordinary geometry is unchanged and remains available throughout
   every shadow slice.
2. No geometry is removed, recolored, depth-patched or resubmitted by the
   shadow.
3. Only positively proved semantic `Enter` and `Exit` events change state.
4. Missing or ambiguous evidence resolves to World, never Sky.
5. Events are ordered by exact ray distance, not BSP traversal order, draw
   order, AABB order or material order.
6. Duplicate triangles cannot toggle state twice.
7. A sky-looking material, paired-sky relationship or `F_SKY1` identity alone
   does not establish a transition.
8. An event behind the nearest ordinary target cannot affect that target.
9. Runtime doors/platforms remain application-owned snapshots; transition
   preparation consumes current facts without acquiring movement policy.
10. Classic causal exclusion and parity realization remain separately named
    claims.
11. Doom/BSP/sky vocabulary remains corpus/provider-private.
12. No stable renderer or spatial-query contract follows from a passing E1M1
    shadow.

## Conservation Ledger

Every ray must account for:

```text
raw semantic-boundary triangles considered
    = exact misses
    + exact intersections

exact intersections
    = grouped members

groups
    = Enter
    + Exit
    + Ambiguous
    + NonTransition

proved state changes
    = Enter + Exit events actually consumed
```

Additionally retain duplicate-collapse counts, events after the ordinary
target, initial-state disposition, final state, and all fail-open reasons.
Nothing may disappear because an acceleration proxy omitted or reordered it.

## Slice 0 — Freeze Vocabulary And Exact Controls

- [x] Freeze all ten replay rays, expected targets, distances and expected
      states in one corpus-private table.
- [x] Name semantic boundary identities independently of triangle identity.
- [x] Inventory existing paired-sky boundary and source-sky-plane diagnostics
      that may supply candidate observations.
- [x] Define initial-state, orientation, duplicate-collapse and ambiguity
      rules before executing parity.
- [x] Preserve global-full and the source-cell live candidate only as controls;
      neither is mutated.

Acceptance: all inputs and expected outcomes are deterministic, and no raw
triangle hit has acquired transition authority merely by being sky-related.

## Slice 1 — Boundary And Closure Audit

- [x] Build a shadow-only semantic boundary inventory with source provenance.
- [x] Audit whether each boundary has a proved World side, Sky side and usable
      orientation.
- [x] Report the decisive open-edge and non-manifold closure counts. Broader
      disconnected/overlap classification stopped when the semantic-side gate
      had already failed.
- [ ] Compare brute-force and BVH exact intersections over identical semantic
      members. Not executed: the audit admitted zero semantic transition
      members, so there is no transition BVH to query.
- [x] Determine whether conventional closed-volume parity is actually
      supported or whether Doom only supplies one-way presentation apertures.

Acceptance: every candidate event has an explicit authority disposition. If
no exit can be proved, say so and do not describe the result as geometric
parity.

## Slice 2 — Ten-Ray Ordered Transition Shadow

- [ ] Emit the required per-ray event/state report.
- [ ] Require all five required ordinary controls to remain World.
- [ ] Test whether all five far-field resurrection controls become Sky through
      positively proved events.
- [ ] Verify duplicate, ordering and conservation ledgers.
- [ ] Run twice and require stable fingerprints.

Acceptance: the first gate is `10/10`, with no ambiguous event used as Sky and
no renderer mutation. A lower score falsifies the hypothesis in its current
form; it does not authorize exceptions per ray.

Slice 2 was not executed. Slice 1 proved neither an `Enter` nor an `Exit`, so
running a parity state machine would violate the study's primary semantic
fence.

## Slice 3 — Adversarial Exit And Initial-State Search

Only after Slice 2 passes:

- [ ] Search specifically for a genuine `Enter → Exit → ordinary geometry`
      specimen.
- [ ] Test camera-under/inside-sky initial states.
- [ ] Test adjacent paired-sky ceilings and disconnected sky regions.
- [ ] Test grazing edges, shared triangle edges and coplanar overlaps.
- [ ] Revisit hut, far-left building, spawn windows/stairs, green-room cutout,
      and sharply pitched views.
- [ ] Include moving door/platform snapshots near any sky relationship.

Acceptance: parity demonstrates value beyond “any sky boundary hides everything
farther.” If no valid exit specimen exists, park parity and characterize the
surviving hypothesis as a bounded one-way sky mask instead.

## Slice 4 — Decision Gate

- [ ] Return the complete evidence to AR-0030.
- [ ] Decide whether the hypothesis is falsified, remains diagnostic, warrants
      a larger corpus, or justifies a separate live A/B proposal.
- [ ] Keep all live composition and renderer work unauthorized until that
      decision is recorded.

Passing this Doom shadow alone cannot admit a provider-neutral renderer
primitive. Independent corpus pressure would still be required.

## Implemented Result — Slices 0–1

The headless command is:

```text
cargo run -q -p hello-doom-e1m1 --bin static_scene -- \
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD \
  --sky-transition-parity-report
```

The frozen inventory contains `89` candidate triangles:

```text
paired-sky height-discontinuity triangles   16
paired semantic linedef groups               8
source-sky open-plane triangles             73
source-sky plane groups                     35
```

All eight paired groups have `F_SKY1` ceilings on both source sides. None
separates a World side from a Sky side. They are valid paired-sky height
discontinuity observations, but not semantic transitions.

The combined surface topology reports:

```text
unique edges       199
manifold edges      68
open edges         131
non-manifold edges   0
```

The source-sky ceiling planes have a local below/above orientation, but their
finite open caps do not prove a closed sky domain or any corresponding Exit.
The resulting semantic inventory therefore has:

```text
proved Enter   0
proved Exit    0
```

The exact ten-ray observations are still informative. Each of the five
far-field resurrection rays has exactly one sky-related raw hit before its
unwanted ordinary target. Four hit paired-sky height discontinuities (linedef
250 for three rays and linedef 252 for one); one hits a source-sky open plane.
All five required nearby floor/ceiling controls have zero raw hits.

Consequently a non-semantic rule of “any raw sky-related hit before the target”
correlates `10/10` with this frozen matrix. That result has
`authority=correlation-only-not-semantic-transition`. It cannot be called
parity, Enter/Exit, a closed skybox, or a Classic causal explanation.

The conservative semantic result remains World for all rays and therefore
matches only the five required-world controls (`5/10`). Slice 2 was correctly
not executed. Conservation is balanced, renderer mutation is false, and two
runs produced fingerprint `864b11fc73f28f2c`.

## Disposition

Real oriented parity is parked. Doom's retained source evidence describes open
sky-presentation surfaces and paired-sky discontinuities, not a closed
World/Sky volume. Inventing missing lateral closure or promoting both-sky
boundaries into World/Sky transitions would violate the study.

The surviving observation may justify a separately reviewed **bounded one-way
sky-hit mask shadow** over complete global-full geometry. Such a successor
would have to admit honestly that it is testing positive one-way presentation
evidence, aggressively search for valid ordinary geometry behind one raw hit,
and retain the ten rays only as its first controls. No such successor or live
candidate is authorized by this result.

## Parking And Escalation

Ordinary implementation findings include deterministic sorting, duplicate
clustering, incorrect normals, missing provenance, BVH/brute disagreement and
diagnostic formatting. Fix those locally and continue within an authorized
shadow slice.

Return early to AR-0030 if:

- defining Enter/Exit requires inventing closure not present in Doom evidence;
- the initial state cannot be proved for ordinary cameras;
- semantic boundary identity requires renderer-owned Doom concepts;
- a live solution requires a new compositing/depth contract;
- parity hides any required ordinary control;
- all useful evidence is one-way and no genuine Exit exists; or
- a material performance requirement would change the architectural boundary.

## References

- `docs/Architectural Reviews/AR-0023-textured-surface-alpha-and-depth-policy.md`
- `docs/Architectural Reviews/AR-0025-camera-candidate-selection-and-visibility-culling.md`
- `docs/Architectural Reviews/AR-0030-source-owned-presentation-preparation-boundary.md`
- `docs/Architectural Reviews/AR-0031-conservative-spatial-query-capability.md`
- `docs/Plans/DOOM/Studies/Doom authoritative sky coverage delta realization.md`
- `docs/Plans/DOOM/Studies/Doom source-authorized relational contribution classification.md`
- `docs/Plans/DOOM/Studies/Doom source-ordered non-presentation causality study.md`
- `docs/Plans/DOOM/Studies/Doom source-occurrence support over reconstructed geometry.md`
- `docs/Plans/DOOM/Evidence/Classic Doom renderer dataflow and Tokimu preparation seam.md`
- `docs/Plans/DOOM/Evidence/Doom authoritative sky-depth realization seam evidence.md`
