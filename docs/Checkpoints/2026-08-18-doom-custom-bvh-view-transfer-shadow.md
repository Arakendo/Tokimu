# Doom Custom BVH View-Transfer Shadow Checkpoint

Date: 2026-08-18

## Disposition

The Doom-private view-cell/aperture hypothesis is parked as a presentation
resolver. Slices 0–3 produced a decisive shadow falsifier without changing
renderer submission, stable contracts, engine ownership or AR-0026 code.

Physical aperture transfer is useful diagnostic topology, but canonical Doom
presentation occurrences are not restricted to the physical openings inferred
between Tokimu's reconstructed render-subsector cells. The existing
source-ordered protocol remains necessary both outside and inside the physical
transfer domain.

## Authorized Fence

The experiment was limited to:

- corpus-private E1M1 diagnostics;
- the existing exact prepared-triangle BVH;
- the existing finite render-subsector graph;
- runtime-height aperture observations;
- actual-camera, path-qualified clipped view windows;
- the six retained BVH/source-oracle rays; and
- shadow comparisons with unchanged renderer submission.

It did not authorize a public portal/BVH contract, renderer vocabulary, a WAD
BSP rebake, physical sky geometry or non-Euclidean chart implementation.

## Directed-Aperture Inventory

The predecessor graph identity was conserved while aperture facts received a
separate deterministic fingerprint:

```text
cells                         237
shared relationships          607
directed edges              1,214
source-correlated              305
traversable apertures          457
non-traversable boundaries     150
zero-clearance relationships    33
aperture containment failures    0
graph fingerprint             13500e039c076c04
aperture fingerprint          3447a97c840c5a0f
```

The 33 zero-clearance relationships are valid shared-boundary correlations,
not traversable openings and not malformed geometry. Treating them as
inventory failures was an ordinary implementation defect and was repaired by
separating graph adjacency from runtime traversability.

## View-Transfer Mechanics

The shadow projects finite vertical aperture quads through the actual Tokimu
camera, intersects their conservative NDC windows with the parent view, and
retains multiple nondominated states per destination cell. Each state preserves
its aperture-chain identity, depth interval, runtime revision and deterministic
fingerprint. Near-plane ambiguity fails open.

The six retained observations created 632 states in total. Peak state count was
306, with a maximum observed path depth of 22 and maximum 16 occurrences of one
cell in one observation. State growth was material but bounded; no unexplained
lossy merge was needed.

## Decisive Positive Falsifier

The exact positive control targets the retained ceiling occurrence in render
subsector 104:

```text
origin     [1477.3304, -3594.2131, 8.9945]
direction  [-0.79217, -0.56500, 0.23070]
BVH hit    flat:40:CEIL3_5 at distance 273.102
source     ordered-oracle=retained
```

Before reaching that exact triangle, the ray crosses:

| Distance | Boundary | Role | Ray height | Physical opening | Inside |
| ---: | --- | --- | ---: | --- | --- |
| 95.20 | `39<>48` | paired-sky opening | 30.96 | `[-56,24]` | no |
| 102.65 | `48<>49` | closed solid | 32.68 | `[-56,24]` | no |
| 168.31 | `49<>104` | implicit partition | 47.82 | `[0,24]` | no |

The exact BVH and brute-force oracle both hit the ceiling, and the existing
Doom ordered-source oracle correctly retains it. Physical aperture transfer
therefore produces a false negative for required presentation. This is not a
BSP decode, BVH containment or window-clipping failure: the valid source
presentation occurrence lies outside the inferred physical portal domain.

## Variant Comparison

```text
cases                                             6
Variant A Boolean connectivity disagreements      5
Variant B bounded transfer disagreements          1
Variant D paired-sky terminal disagreements        1
Variant C complete ordered fallback disagreements 0

relevant surfaces                              2,175
inside transferred cells/windows                 782
outside transfer                               1,393
retained outside transfer needing source rescue    74
source-covered despite reached cells              290

view states total                                 632
view states peak                                  306
matrix fingerprint                   d7071068f8b6b571
submission changes                                none
```

Variant C agrees only because it retains the complete predecessor ordered
source oracle as a fallback for contributions outside physical transfer and as
coverage authority within reached cells. This is not meaningful authority
localization: all six exact target samples require the fallback decision.

## Architectural Finding

For this corpus, physical view-cell reachability is neither necessary nor
sufficient for Doom presentation participation:

```text
outside physical aperture domain
    can still be a retained Doom presentation occurrence

inside a reached cell
    can still be covered by Doom's ordered source protocol
```

The BVH remains an exact conservative spatial-query mechanism. View transfer
remains useful diagnostic evidence. Neither may accept or reject Doom
presentation declarations. Inventing a wider "presentation aperture" would
have to encode Doom's ordered source semantics and would merely rename the
existing authority.

This negative result also fences AR-0026: path-qualified view state may be a
useful mechanical comparison, but canonical E1M1 does not provide evidence for
a reusable chart/portal capability.

## Validation

```text
cargo fmt --all
cargo test -q -p hello-doom-e1m1 --bin static_scene
cargo clippy -q -p hello-doom-e1m1 --bin static_scene
cargo run -q -p hello-doom-e1m1 --bin static_scene -- \
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD \
  --render-subsector-connectivity-report
```

The targeted binary passed 87 tests. Clippy completed with pre-existing
dead-code and workspace warnings; no new error was introduced. The report
asserts aperture containment, graph identity, exact BVH/brute-force agreement,
the six-ray source outcomes and unchanged submission.

## Remaining Decision

Do not continue the larger pitch/movement/runtime visual matrix for this
resolver. Its semantic acceptance condition has already failed. Any next Doom
presentation slice must remain honest about the source-ordered occurrence
protocol rather than granting physical cells, apertures, BVH nodes or sky
geometry authority they did not demonstrate.
