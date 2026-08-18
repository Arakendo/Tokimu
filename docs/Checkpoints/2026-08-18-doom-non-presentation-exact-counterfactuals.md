# 2026-08-18 Doom Non-Presentation Exact Counterfactuals

## Disposition

The bounded counterfactual slice confirms that the five retained absent target
domains are not merely correlated with nearby solid geometry. Suppressing one
named solid-range mutation while retaining its SEG traversal and admission can
reopen the target BSP child in every absent case.

The result is deliberately narrower than a rendering proposal:

```text
ordinary replay
    exact ordered solid-range union covers target child
    → child skipped

counterfactual replay
    suppress one named source SEG's coverage mutation
    → target child may become reachable
    → downstream traversal may cascade
```

The causal unit remains the accumulated ordered coverage state. Individual
SEGs are event provenance, not context-free occluder objects.

## Reproduction

```powershell
cargo run -q -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --ordered-non-presentation-causality-report
```

Summary:

```text
cases=6
absent-covering-provenance-resolved=5/5
absent-targets-reached-without-solid-pruning=5/5
exact-covering-event-counterfactuals-tested=20
exact-counterfactuals-reopening-target=12/20
absent-cases-with-individually-necessary-covering-event=5/5
absent-cases-with-paired-sky-causal-source=0/5
positive-wall-control=retained
case-inventory-fingerprint=ff713d1723103e40
matrix-fingerprint=f5c9e7577570bd75
replay-identical=true
conservation=balanced
submission-changes=none
```

## Exact Interventions

The report tests every focused source event. The following table retains one
compact all-target reopening witness per absent case; other tested events and
partial/none outcomes remain in the full headless report.

| Case | Suppressed mutation | Original contribution | Target result | Recorded cascade |
| --- | --- | --- | --- | --- |
| `hut-east-wall-230` | SEG 405 / linedef 159 | `[162,210]` | both target subsectors 139/142 reached | +12 subsectors, +19 admitted SEGs, far prunes `9→9` |
| `wall-247-east` | SEG 405 / linedef 159 | `[154,174]` | both target subsectors 190/192 reached | +12 subsectors, +20 admitted SEGs, far prunes `19→19` |
| `wall-247-west` | SEG 405 / linedef 159 | `[149,199]` | both target subsectors 190/192 reached | +12 subsectors, +19 admitted SEGs, far prunes `8→8` |
| `ceiling-149-rejected` | SEG 398 / linedef 272 | `[0,95]` | target subsector 149 reached | +7 subsectors, +10 admitted SEGs, far prunes `6→6` |
| `ceiling-104-rejected` | SEG 125 / linedef 37 | `[126,155]` | target subsector 104 reached | +3 subsectors, +7 admitted SEGs, far prunes `33→32` |

No intervention loses a previously visited subsector or admitted SEG in these
five selected witnesses. The new work is expected downstream traversal after
removing an earlier coverage mutation; it is reported rather than mistaken for
a local renderer toggle.

The remaining eight tested mutations do not reopen the target domain because
later or overlapping accumulated coverage still supports the prune. Some
mutations reopen only one of a paired wall target's two subsectors. Therefore:

> A source SEG can be individually necessary for one execution's target-child
> prune, but the general authority belongs to the ordered coverage union.

## Positive Wall Control

The nearby retained wall control is the exact prepared hit
`wall:135:SUPPORT2`, SEG 270 / linedef 135:

```text
target subsector={88}
target reached=true
SEG admitted=true
projected interval=[150,184]
fully covered before admission=false
prepared declarations=2
retained view interval=[-0.0576914224,0.1515589034]
```

This establishes the requested positive comparison through the same stages:
an incompletely covered target child is traversed, its SEG is admitted and an
ordinary prepared wall declaration survives.

## Sky And Problem Separation

The exact covering chains still contain zero paired-sky source SEGs. Existing
synthetic paired-sky, one-sky and single-sky-plane controls remain valid and
continue to distinguish final sky presentation behavior. They do not acquire
causal authority over these five exclusions.

The evidence now separates two problems:

```text
source-covered far-field resurrection
    ordered horizontal coverage skips a source domain
    complete reconstructed geometry must not silently resurrect it

free-look plane realization
    source domain participates
    arbitrary-pitch world-space floor/ceiling realization remains separate
```

This separation is evidence for a future Doom-private realization. It is not
yet permission to filter declarations, expose Doom columns through Tokimu, or
make BSP concepts renderer-owned.

## Implementation And Validation Boundary

- The provider counterfactual suppresses exactly one named source SEG's range
  mutation while retaining ordinary traversal, facing, projection and SEG
  admission.
- Suppressed source identity, linedef and projected interval are recorded.
- Each replay reports target reach and downstream traversal/admission deltas.
- The ordinary replay remains unchanged and continues to pass its deterministic
  fingerprints and conservation checks.
- The exact prepared BVH is target identity evidence only.

## Remaining Work

- Correlate explicit E1M1 hut and far-left positive views with the causal
  report; retain the existing synthetic single-sky controls alongside them.
- Cross-reference the exact stages with Linux Doom / Chocolate Doom source
  before upgrading faithful replay evidence to direct Classic evidence.
- Return the completed evidence table to AR-0030 before implementing any
  source-domain exclusion in live free-look preparation.
