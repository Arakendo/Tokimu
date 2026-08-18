# Doom Source-Occurrence Support Over Reconstructed Geometry Study

| Field | Value |
| --- | --- |
| Status | Live A/B falsified by adversarial E1M1 walkabout — retained as diagnostic only |
| Scope | Determine whether Doom's source-keyed, view-local wall and plane cells can authorize ordinary reconstructed geometry without whole-subsector or whole-key Boolean promotion |
| Parent review | [AR-0030](../../../Architectural%20Reviews/AR-0030-source-owned-presentation-preparation-boundary.md) |
| Immediate falsifiers | [2026-08-18 Doom Source-Covered Walkabout Falsifiers](../../../Checkpoints/2026-08-18-doom-source-covered-walkabout-falsifiers.md) |
| Prior causal evidence | [Doom Source-Ordered Non-Presentation Causality Study](Doom%20source-ordered%20non-presentation%20causality%20study.md) |
| Stable API authority | None |
| Renderer changes authorized | None during shadow slices |
| Presentation install authorized | Opt-in corpus-private `source-occurrence-supported` strategy only; no stable/default installation |

## Question

Can the Doom-private ordered replay relate its exact retained wall/plane cells
to Tokimu's actual reconstructed geometry closely enough to distinguish all of
the following without exposing source spans to the renderer?

```text
source cell does not cover reconstructed hit
        → source-valid non-presentation

source cell covers reconstructed hit
        → ordinary geometry may participate there

source proxy does not contain reconstructed support
        → proxy cannot reject it

mapping unresolved
        → retain fail-open and diagnose
```

The study is not another whole-object filter. It tests a relation between two
different representations:

```text
Doom ordered source occurrence       reconstructed plane/wall support
    key + column/row interval    ×       finite source geometry
                         ↓
              ordinary clipped geometry
```

## Starting Evidence

The `source-covered-global-shell` experiment is useful but insufficient:

- subsectors 54, 55 and 117 are reached while their exact ceiling keys are
  absent, producing false retention;
- subsector 130 is skipped by an `outside-fov` child proxy while its sector 5
  floor key has populated spans and the complete shell has a local hit only
  132.480 source units away, producing false omission;
- a nearby pose shows that key presence alone cannot authorize every point of
  a complete correlated plane mesh; and
- wall 241 is horizontally admitted behind an earlier sky boundary, but its
  final retained wall-cell support at the exact hit remains unproven.

This means the study must preserve both source occurrence and actual geometry.
Neither may silently borrow rejection authority from the other.

## Slice 0 — Immutable Capture Matrix

- [x] Freeze the five walkabout rays with exact source coordinates and expected
      complete-shell hit identity.
- [x] Re-run each ray against the complete shell and the source-covered
      candidate.
- [x] Preserve the prior five negative and two positive controls unchanged.
- [x] Add deterministic capture and result fingerprints.

Acceptance: the matrix detects the three known false retentions, the local
subsector 130 false omission and the unresolved wall 241 case without relying
on screenshots.

## Slice 1 — Exact Source-Cell Correlation

- [x] Map each source ray to the bounded Classic diagnostic column and row
      using the replay's fixed horizontal/vertical projection.
- [x] For a plane hit, correlate exact kind, runtime height, texture, light,
      sector and retained instance row interval at that column.
- [x] For a wall hit, correlate exact SEG/linedef/tier and retained ordered wall
      interval at that column.
- [x] Report `supported`, `unsupported` or `unresolved-fail-open` separately
      from complete-shell intersection.
- [x] Record the earlier sky boundary only as comparative provenance.

Acceptance: wall 241 receives final wall-cell evidence, and every plane capture
has an exact cell-support disposition. Horizontal BSP admission or plane-key
existence alone is not accepted as final support.

The 320x200 grid is bounded source evidence, not renderer pixel parity. The
test maps a world/source ray into that source projection; it does not equate a
Tokimu viewport pixel with one Classic pixel.

## Slice 2 — Plane Cell To Geometry Shadow

- [x] Reuse the existing source-plane domain-cell reconstruction.
- [x] Compare the current `same source subsector` cell/geometry association
      against an exact plane-key plus source-sector association.
- [x] Clip all matching reconstructed source-plane triangles against the
      finite projected cell support in a shadow result.
- [x] Bound duplicate/overlapping fragments and report amplification,
      degenerates, unresolved mappings and containment failures.
- [x] Keep sky instances as background authority, not depth-writing plane
      geometry.

Acceptance: the shadow contains the supported subsector 130 floor point without
reintroducing the unsupported sector 24/41 ceiling points. No source child or
SEG bbox rejects a reconstructed plane.

The key hypothesis is deliberately narrow: the existing partial-plane path
associates a cell with the subsector owning the cell's source SEG. The
subsector 130 capture suggests that the cell instead supplies support to
matching reconstructed geometry by exact plane key and source sector. This
must be tested, not assumed.

## Slice 3 — Wall Fragment Comparison

- [x] Compare existing horizontally clipped ordered wall declarations with the
      exact retained vertical wall-cell intervals.
- [x] Determine whether wall 241 is already absent from exact retained cells or
      whether current wall lowering over-promotes horizontal admission.
- [x] If needed, construct shadow-only ordinary wall fragments from the exact
      retained cells and report fragment amplification.
- [x] Retain nearby wall 135 / SUPPORT2 as the mandatory positive wall control.

Acceptance: the wall policy explains both wall 241 and SUPPORT2 using final
retained source cells. A sky boundary is not converted into an occluder.

## Slice 4 — Corpus Matrix And Decision

- [x] Run the five new captures and prior seven controls.
- [ ] Add spawn, hut, far-left, pitch, near-wall and return-pose scan controls.
- [x] Verify deterministic source-cell, clipped-geometry and result
      fingerprints.
- [ ] Measure fragment/draw amplification and preparation time separately from
      correctness.
- [ ] Return results to AR-0030 before installing a presentation strategy.

Acceptance: all exact negative and positive controls agree; unresolved cases
remain visible and explicit; source-cell geometry remains Doom-private; the
renderer would receive only ordinary finite declarations.

## Conservation

Every shadow result must account for:

- complete-shell input triangles and source identities;
- source cells considered, supported and unresolved;
- reconstructed triangles considered;
- fully unsupported triangles;
- whole, clipped and degenerate output fragments;
- duplicate/overlap findings;
- sky background destinations; and
- exact-ray agreements and disagreements.

No output fragment may exist without both source-cell support and matching
reconstructed source identity. No input may disappear because a proxy bound
failed to contain derived geometry.

## Architectural Boundary

- All Doom keys, columns, rows, SEGs, tiers and spans remain in the Doom corpus
  or provider-private preparation side.
- `tokimu-render` receives no Doom, BSP, portal, sky-boundary or span concept.
- The exact-geometry BVH supplies intersection and containment evidence only.
- The study does not define a generic Tokimu visibility capability.
- Application/runtime policy continues to own current door/platform state;
  the study consumes only a current snapshot.
- No presentation-affecting strategy is installed during shadow slices.

## Parking And Escalation

Ordinary projection defects, missing diagnostics, deterministic replay issues
and local clipping defects remain implementation work.

Return to AR-0030 before presentation installation, or earlier if:

- exact source-cell support cannot be related to reconstructed geometry without
  renderer-owned Doom semantics;
- the required relation needs a new stable/public contract;
- arbitrary-pitch support contradicts the retained source authority rather
  than extending its realization; or
- fragment amplification becomes a material architectural constraint rather
  than an implementation parameter.

## Paused Finding — 2026-08-18

The five new captures agree with exact final cells (`5/5`) and all four plane
captures agree with the exact-key-plus-source-sector geometry shadow (`4/4`).
Wall 241 has one correlated middle-tier interval at the sampled column, but
that interval does not retain the sampled row; the ordered preparation emits
no declaration. Wall 135 remains the positive wall control.

The prior seven-control set cannot be promoted unchanged into an exact-cell
gate. The ray historically named `ceiling-104-reached` proves that subsector
104 contributes a partial ceiling occurrence somewhere in the frozen view; it
does not prove that the exact BVH hit point is source-present. The exact ray
maps to source cell `(160,62)`, where the matching plane-key instance has no
interval. Neither the current ordered declarations nor the new cell/geometry
shadow intersect that ray.

This is not a projection arithmetic defect: the diagnostic inverse uses the
same `320x200`, 45-degree horizontal field and aspect-derived vertical field
as the provider. Broadening cells from exact key plus source sector to plane
key alone did not restore the ray and materially increased fragments, so that
authority expansion was rejected.

The shadow is therefore parked before presentation installation. AR-0030 must
decide whether the historical subsector-104 control remains only an
object-occurrence control, whether a new independently visible exact positive
plane control is required, and how arbitrary pitch may extend source
realization without claiming pixel parity. Result checkpoint:
`docs/Checkpoints/2026-08-18-doom-source-occurrence-support-shadow.md`.

## Authorized Follow-Up — Neutral Exact Positives And Pitch Lift

Maintainer review retained the historical `ceiling-104-reached` failure as an
object/exact-ray granularity warning rather than weakening it into a passing
pixel oracle. The follow-up searched four neutral-pitch poses on a bounded
`32x20` sampling of the Classic projection. A candidate was admitted only when
all four independent paths agreed:

```text
complete reconstructed shell exact hit
    + final retained source cell
    + existing ordered declaration exact hit
    + exact-key/source-sector cell geometry exact hit
```

The search found `481` agreements across `12` distinct
pose/plane/sector/subsector identities. Three diverse controls were frozen:

| Control | Cell | Exact source | Distance |
| --- | --- | --- | --- |
| spawn ceiling | `(155,65)` | sector 38 / subsector 97 / `CEIL3_5` | `170.858` |
| spawn floor | `(155,125)` | sector 39 / subsector 105 / `FLAT14` | `330.520` |
| near-wall floor | `(165,145)` | sector 38 / subsector 102 / `FLOOR4_8` | `131.685` |

All three retain matching source cells, intersect existing ordered
declarations, and intersect the reconstructed cell shadow at the same distance
within `0.01`. The deterministic discovery fingerprint is
`129127dbc170cb2b`; the frozen-control fingerprint is `e5cb5acbddd8406d`.

The pitch shadow keeps the neutral-source-authorized finite world fragments
fixed. It projects each exact world ray through cameras pitched `-15`, `0` and
`+15` degrees, reconstructs the actual-camera ray, and intersects the same
ordinary fragments. All nine queries preserve their hit and distance. Classic
rows are not reinterpreted as world boundaries after preparation.

This resolves the immediate positive-oracle and bounded pitch-lift questions
in shadow form. It does not install a presentation strategy. The next decision
belongs to AR-0030: authorize a corpus-private live A/B realization using the
neutral-authorized finite world regions, or require a broader pitch/pose
matrix first.

## Live A/B Realization — 2026-08-18

Maintainer review authorized the corpus-private live candidate after the
neutral exact-positive and pitch-lift gates passed. The candidate is selected
explicitly with:

```text
--render-strategy=source-occurrence-supported
```

It combines final ordered wall declarations with reconstructed floor and
ceiling fragments clipped to exact plane key, source sector and retained
source cells. Preparation consumes the current runtime height snapshot,
completes and verifies both wall and plane conservation before replacing the
composition's current prepared set, and gives the renderer only ordinary
opaque/cutout declarations. Camera pitch does not rebuild Classic rows; it
reprojects the already-prepared finite world geometry normally. Position,
yaw, eye height and moving-sector height changes trigger a new complete
preparation.

The explicit control remains:

```text
--render-strategy=global-full-submission
```

The headless acceptance report is:

```text
--source-occurrence-live-report
```

It passes `15/15` twice with deterministic fingerprint
`b578061ac0312dce`. The matrix includes the five walkabout falsifiers, the six
historical exact negative rays, wall 135 as a positive wall, and the three
independently justified positive planes. The historical subsector-104 ceiling
remains explicitly object-positive but exact-ray-negative; its old `6/7`
shadow result is not rewritten.

A native Vulkan two-frame smoke run completed at spawn. The runtime-camera
preparation produced `360` opaque and `12` cutout candidate declarations from
the complete E1M1 shell, with balanced conservation, `0` unresolved wall or
plane contributions, and `7,702` output plane triangles (`7.940x` plane
triangle amplification). The first frame completed in approximately `84.9`
ms CPU time and the warm frame in `16.1` ms. These are smoke measurements, not
accepted performance budgets.

The strategy remains opt-in and corpus-local. The next gate is an adversarial
walkabout at the hut, far-left structure, near-spawn windows/stairs, retained
sector 24/41 captures, floor 130, walls 241/135, the first moving door and
platform, green-room cutout, and representative `-15`/`+15` degree pitch.
Visual failures must be replayed through `LOOK` and the complete-shell → final
cell → ordered declaration → prepared world support → candidate chain before
any policy expansion.

## Live Walkabout Falsification — 2026-08-18

The first maintainer walkabout reports that the opt-in candidate is worse than
the control and contains many visible holes. This fails the live coherence
gate even though all `15/15` hand-selected exact rays remain green. The
candidate must not become the default and its exact support must not be
broadened merely to conceal the result.

The headless report now also measures nearest-hit deltas against the complete
shell over four `32x20` neutral-view grids (`2,560` rays). It records seven
complete-shell hits with no candidate hit and two more whose nearest candidate
is a different, farther contribution. The affected samples include floors in
sectors 37, 38 and 39 and a sector 38 ceiling. The deterministic broad-grid
fingerprint is `d049a8b79f7404c8`.

That bounded grid under-represents the visual failure because it covers only
the earlier retained neutral poses, not the maintainer's complete walk path,
but it demonstrates the important methodological failure: a sparse exact-ray
matrix can validate local source-occurrence claims without establishing a
watertight live presentation. Balanced declaration conservation likewise
proves accounting, not screen coverage.

The precise cause of each observed hole remains unresolved until one or more
actual failing camera/pixel replays are retained. Plausible local causes
include incomplete source-cell-to-world support reconstruction, cracks or
coverage loss between finite cell fragments, or valid Classic occurrences
that the current lowering fails to realize at modern raster resolution. Those
possibilities must be distinguished with exact evidence; none authorizes a
new renderer contract or generic BSP/BVH visibility policy.
