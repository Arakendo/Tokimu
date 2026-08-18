# E1M1 Sky-Leak BVH / Source-Participation Shadow Checkpoint

| Field | Value |
| --- | --- |
| Date | 2026-08-17 |
| Scope | Test whether exact prepared-triangle BVH evidence distinguishes known E1M1 far-field sky leaks from valid presentation |
| Outcome | Architectural finding retained; no presentation mutation |
| Governing reviews | AR-0030 and AR-0031 |

## Question

The global-full E1M1 path renders distant prepared geometry through sky
domains. The Tokimu-first BVH can conservatively answer whether actual
prepared geometry intersects a ray or frustum. This slice asked whether that
spatial evidence also distinguishes source-valid geometry from the known
global-shell leaks.

The experiment was shadow-only:

```text
same retained source ray
    -> exact global prepared triangles
        -> BVH nearest hit
        -> brute-force nearest-hit oracle

    -> Doom-private ordered preparation
        -> final source disposition
        -> final prepared declarations
```

No result changed renderer submission.

## Implementation

`SpatialRayShadow` builds the corpus-local BVH over the complete prepared
triangle inventory, audits containment/conservation, converts retained source
rays through the inventory's current embedding and compares every BVH nearest
hit with the brute-force oracle.

The existing `--ordered-occurrence-six-ray-report` now combines that evidence
with the final ordered handoff. Each case asserts the expected global source
label before checking the source disposition and declaration result.

## Executed Evidence

Command:

```powershell
cargo run -p hello-doom-e1m1 --release --bin static_scene -- corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD --render-strategy=ordered-occurrence-prepared-full --ordered-occurrence-six-ray-report
```

Observed cases:

| Case | BVH nearest global contribution | Doom final handoff |
| --- | --- | --- |
| hut-east-wall-230 | `wall:230:BROWN1`, distance `631.266` | SEGs `415/423` terminally rejected; zero declarations |
| wall-247-east | `wall:247:BROWN96`, distance `1742.658` | SEGs `559/567` terminally rejected; zero declarations |
| ceiling-104-reached | `flat:40:CEIL3_5`, distance `273.102` | partial ceiling; two finite interval groups, two partial sources, eight declarations |
| wall-247-west | `wall:247:BROWN96`, distance `892.484` | SEGs `559/567` terminally rejected; zero declarations |
| ceiling-149-rejected | `flat:7:CEIL3_5`, distance `358.869` | zero associations, destinations, dispositions and declarations |
| ceiling-104-rejected | `flat:40:CEIL3_5`, distance `1921.191` | zero associations, destinations, dispositions and declarations |

All BVH hits matched the brute-force nearest-triangle oracle. Ordered-result
conservation remained balanced.

## Finding

> Actual-geometry spatial relevance and source presentation participation are
> orthogonal facts.

The BVH is correct to retain every tested triangle: each is real prepared
geometry intersected by the ray. Doom is also correct to omit five of those
contributions from the final presentation result and to retain the sixth only
through a bounded partial plane occurrence.

Consequently, a conservative BVH candidate filter cannot fix this sky leak.
It has no evidence with which to distinguish these source-invalid occurrences
from valid far-left, hut-adjacent or opening geometry. Distance, a sky depth
wall, Classic child bounds over larger reconstructed planes, and generic AABB
visibility remain invalid substitutes for the missing source participation
semantics.

## Disposition and Boundary

No BVH submission authority is added. No BSP rebake is justified. No local
relational classifier is resumed merely to rediscover decisions already
present in the ordered source result.

The meaningful implementation direction remains:

```text
complete Doom ordered result
    whole retained     -> ordinary source geometry
    terminal rejected  -> no declaration
    partial SEG        -> source-relative wall realization
    partial plane      -> focused plane-occurrence realization
```

The valid far-left building and hut/outside geometry remain mandatory visual
falsifiers when that realization path resumes. Their exact retained ray
attribution is still incomplete, so this checkpoint does not claim final
visual acceptance or a repaired renderer path.

Continuing from this point would require architectural judgment about how the
complete ordered occurrence result becomes the authoritative live native and
browser preparation unit. That work remains under AR-0030 rather than being
hidden inside the spatial-query study.
