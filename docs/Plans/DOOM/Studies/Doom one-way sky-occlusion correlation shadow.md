# Doom One-Way Sky-Occlusion Correlation Shadow Study

| Field | Value |
| --- | --- |
| Status | Parked at Slice 1 — eight critical falsifiers reject blanket one-way masking |
| Scope | Test whether a Doom-related sky surface before ordinary geometry predicts absence of exact final source participation |
| Parent review | [AR-0030](../../../Architectural%20Reviews/AR-0030-source-owned-presentation-preparation-boundary.md) |
| Geometry oracle | Complete `global-full-submission` prepared geometry |
| Immediate predecessor | [Doom Oriented Sky-Transition Parity Shadow](Doom%20oriented%20sky-transition%20parity%20shadow.md) |
| Renderer changes authorized | None |
| Stable API authority | None |

## Question

When an actual camera ray intersects a proved Doom-related sky presentation
surface before ordinary complete-world geometry, does that observation
reliably correlate with the farther target having no exact final Doom source
participation?

The hypothesis is deliberately one-way:

```text
actual ray
    ↓
sky-related surface before ordinary target
    ↓
observation only:
farther target may be source-invalid
```

It is not:

```text
sky hit → remove everything farther
```

“Sky occlusion” is the source phenomenon being investigated, not authority
already granted to the geometric hit.

## Predecessor Result

The parity predecessor proved that the available 89 sky-related triangles do
not form a closed World/Sky volume:

```text
proved Enter   0
proved Exit    0
open edges   131
```

It nevertheless found a `10/10` correlation in its first frozen matrix:

- all five known unwanted far-field targets had one sky-related hit first;
- all five required nearby ordinary targets had none.

That result has `authority=correlation-only-not-semantic-transition`. This
study pressure-tests the correlation without reviving parity vocabulary.

## Independent Axes

Every sampled ray records three claims independently:

```text
sky-before-target
    none | one-or-more | ambiguous

exact final source participation
    exact-present | source-partial | absent | unresolved

known historical causal exclusion
    yes | no | not-established
```

`exact-present` requires an ordinary declaration from the frozen-view ordered
preparation with the same source identity to intersect the exact ray.
`source-partial` means matching source declarations exist but none supports
that exact ray. `absent` means there is no matching final declaration.

The ordered result remains a source-protocol observation. It is not rewritten
as a consequence of the sky hit.

## Required Quadrants

| Sky before target | Exact target result | Interpretation |
| --- | --- | --- |
| no | exact-present | expected ordinary control |
| no | partial/absent | the sky correlation cannot explain the exclusion |
| yes | partial/absent | supporting correlation only |
| yes | exact-present | critical falsifier for blanket omission |
| ambiguous | any | unresolved; no authority |

The study searches for `yes + exact-present` first. One honest specimen is
enough to reject “hide everything after the first sky hit.” No ratio or
special case may erase it.

## Adversarial Search

The first broad report samples a deterministic camera grid across poses chosen
from the retained E1M1 trouble areas:

- source spawn;
- near-wall spawn views;
- courtyard and stairs/high platform;
- hut from multiple sides;
- the far-left structure; and
- the retained ceiling/wall causal viewpoints.

The viewport grid includes upward and downward rays. For each ray with a
complete-world ordinary target it records:

```text
pose and source-space ray
ordinary target identity and distance
ordered sky-related hits before the target
nearest sky hit identity, family, provenance and distance
exact ordered-source target result
quadrant and authority disposition
```

Representatives from every non-empty quadrant are retained with replayable
origins and directions. Results must be deterministic.

Doors and platforms remain application-owned runtime facts. This first static
matrix does not claim to cover moving snapshots; dynamic controls are
conditional on the static hypothesis surviving.

## Causal Comparison

The five original unwanted far-field controls retain their independently
proved historical result:

```text
Classic ordered solid coverage
    → target source occurrence absent or partial
```

For those controls the report compares:

```text
sky-before-target?
ordered causal exclusion?
exact target participation?
```

Sky correlation may predict the same outcome. It must never be described as
the Classic cause unless new evidence actually proves that claim.

## Spatial Boundary

Brute-force exact triangle intersection is the initial oracle. A BVH may later
accelerate the identical member set only after exact agreement is demonstrated.
Neither AABB, frustum nor BVH traversal may infer source participation or sky
authority.

## Binding Invariants

1. Global-full geometry remains unchanged and queryable.
2. The shadow removes, recolors and resubmits nothing.
3. A sky-related triangle hit is an observation, not an Enter, Exit, mask or
   omission command.
4. Exact final source participation is measured independently.
5. A `sky-before + exact-present` sample is retained as a falsifier.
6. Partial participation is not collapsed into absence.
7. Unknown or ambiguous evidence never becomes omission authority.
8. Classic causal history remains separately named.
9. Doom vocabulary remains corpus/provider-private.
10. No renderer or provider-neutral spatial contract follows from this study.

## Conservation

For each report:

```text
sampled rays
    = no-ordinary-target + ordinary-target rays

ordinary-target rays
    = sky-before + no-sky-before

ordinary-target rays
    = exact-present + source-partial + absent + unresolved

sky-before rays
    = supporting + critical-falsifier + unresolved
```

Raw triangle hits, grouped semantic observations and duplicate collapses are
retained separately. Deterministic fingerprints cover poses, rays, targets,
sky observations and source dispositions.

## Slice 0 — Contract And Frozen Controls

- [x] Preserve the predecessor's five unwanted and five required controls.
- [x] Rename the hypothesis as a correlation shadow, not a mask or parity.
- [x] Define independent sky, source-participation and causal axes.
- [x] Define the critical falsifier and fail-open rule.

Acceptance: the existing `10/10` is represented as a baseline correlation and
has acquired no presentation authority.

## Slice 1 — Adversarial Static Search

- [x] Sample the deterministic multi-pose upward/neutral/downward grid.
- [x] Populate all observed quadrants without filtering adverse results.
- [x] Retain representative replay rays, including every critical falsifier
      up to a documented bounded output limit.
- [x] Compare the original causal controls independently.
- [x] Verify conservation and repeatability.

Acceptance: the report establishes whether valid exact source geometry occurs
behind a preceding sky-related hit. If any does, blanket one-way masking is
falsified and must be parked.

## Implemented Result — Slices 0–1

The headless command is:

```text
cargo run -q -p hello-doom-e1m1 --bin static_scene -- \
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD \
  --sky-occlusion-correlation-report
```

The first adversarial search sampled eight retained trouble-area poses over a
`32 × 20` viewport grid:

```text
sampled rays          5,120
ordinary targets      4,380
no ordinary target      740
sky before target        36
no sky before target  4,344
```

The four observed quadrants were:

```text
no sky + exact present                 3,414
no sky + partial/absent                  930
sky before + partial/absent               28
sky before + exact present                 8  CRITICAL
```

All eight critical samples are retained in the report. Six occur from the
hut-east pose and preserve exact ordered wall declarations for linedefs 159 or
160 behind paired-sky observations on linedefs 250 or 254. Two occur from the
far-left-structure pose and preserve exact wall 203 behind paired-sky linedef
250.

One representative is:

```text
pose                 hut-east
cell                 (155,105)
ray origin           (2076,-3560,36)
ray direction        (0.893540758,-0.447666839,-0.034341145)
preceding sky        paired-linedef:250 at 196.575
ordinary target      wall:160:BROWN144 at 421.274
ordered declarations 3 matching the exact source
exact ordered hit    421.274
result               sky-before + exact-present
```

The ten predecessor controls remain intact: the five historically excluded
targets occupy `sky-before + absent`, while the five required source-cell holes
occupy `no-sky + partial/absent`. Their historical cause remains ordered solid
coverage and is not rewritten as sky causality.

The report accounts for 46 raw sky hits, 46 semantic groups and zero duplicate
collapses. Conservation is balanced, renderer mutation is false, and two runs
produced fingerprint `b3435e035db5ab1d`.

## Slice 1 Disposition

Blanket “first sky-related hit hides all farther geometry” is falsified. Valid
exact final source geometry occurs behind the same paired-sky surfaces that
correlate with other absent targets. Distance ordering and sky identity alone
cannot distinguish the two outcomes.

Slice 2 is not entered. Expanding the correlation corpus, adding a BVH or
testing live masking cannot repair this falsifier. The weaker use suggested by
the review survives: a sky-before observation may be a diagnostic trigger that
asks the independent ordered Doom evidence to scrutinize a target. It has no
omission authority of its own.

## Slice 2 — Conditional Precision Study

Only if Slice 1 finds no critical falsifier:

- [ ] Expand the pose corpus around windows, adjacent sky sectors and sharply
      pitched views.
- [ ] Add moving door/platform snapshot controls where relationships change.
- [ ] Compare brute-force and BVH queries over identical members.
- [ ] Require zero `sky-before + exact-present` results.

Acceptance: surviving evidence may justify a diagnostic trigger proposal, not
live omission authority.

Not executed: Slice 1 found eight critical falsifiers.

## Slice 3 — Decision

- [x] Return the complete evidence to AR-0030.
- [x] Choose among falsified, diagnostic trigger, larger shadow corpus, or a
      separately reviewed live A/B proposal.

Decision: blanket masking is falsified and parked. Correlation remains useful
only as a possible diagnostic trigger; no live A/B proposal follows.

Even a high-precision result should first be considered as a trigger for more
expensive Doom source-participation scrutiny:

```text
complete geometry query
    ↓
sky-before correlation marks suspicious target
    ↓
ordered Doom evidence decides participation
```

## Stop Conditions

Return to AR-0030 before live work if:

- any critical `sky-before + exact-present` specimen is found;
- useful classification requires treating sky hits as transitions;
- exact source participation cannot be measured independently;
- a renderer depth/compositing contract is required;
- source ambiguity would need to resolve to omission; or
- performance pressure would change the ownership boundary.
