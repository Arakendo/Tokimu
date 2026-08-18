# 2026-08-18 Doom Non-Presentation Causality Slice 1

## Disposition

The first causal slice is complete enough to answer the retained six-ray
question mechanistically:

> In all five absent cases, nearer ordinary solid SEGs accumulated complete
> horizontal coverage of the target BSP child's projected interval. Classic
> near-first traversal therefore skipped that child before the target wall or
> plane occurrence could be produced.

This is a Doom-private diagnostic result. It changes no renderer submission,
stable contract or presentation authority. The pre-study workspace was
checkpointed as commit `671fd1d` (`Checkpoint Doom spatial and presentation
studies`).

## Reproduction

```powershell
cargo run -q -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --ordered-non-presentation-causality-report
```

Deterministic summary:

```text
cases=6
absent-covering-provenance-resolved=5/5
absent-targets-reached-without-solid-pruning=5/5
absent-cases-with-paired-sky-causal-source=0/5
case-inventory-fingerprint=ff713d1723103e40
matrix-fingerprint=1c3a4c05c008552b
replay-identical=true
conservation=balanced
submission-changes=none
```

## Six-Ray Causal Matrix

| Target | Final outcome | First decisive event | Focused covering provenance | Downstream consequence | Counterfactual | Evidence level |
| --- | --- | --- | --- | --- | --- | --- |
| `wall:230:BROWN1`, SEGs 415/423 | absent | node 141 skips target subsectors 139/142; target interval `[0,180]` is covered | SEGs 152, 161, 173, 174, 405, 406, 413; linedefs 159, 160, 204, 207, 208, 446, 447 | target subsectors are not reached; no declaration | target subsectors are reached when all solid-range pruning is disabled | exact-geometry control + faithful replay |
| `wall:247:BROWN96`, east, SEGs 559/567 | absent | node 197 skips target subsectors 190/192; target interval `[143,198]` is covered | SEGs 173, 403, 405, 406, 413; linedefs 159, 160, 179, 204, 446 | target subsectors are not reached; no declaration | target subsectors are reached when all solid-range pruning is disabled | exact-geometry control + faithful replay |
| subsector 104 ceiling | partial | target child is traversed | not applicable | 3 source associations produce 1 destination | remains reached in the broad control | exact-geometry control + faithful replay |
| `wall:247:BROWN96`, west, SEGs 559/567 | absent | node 197 skips target subsectors 190/192; target interval `[129,228]` is covered | SEGs 403, 405, 406; linedefs 159, 160, 179 | target subsectors are not reached; no declaration | target subsectors are reached when all solid-range pruning is disabled | exact-geometry control + faithful replay |
| subsector 149 ceiling | absent | node 152 skips target subsector 149; target interval `[10,191]` is covered | SEGs 101, 394, 395, 398; linedefs 163, 271, 272 | plane eligibility and vertical clip stages are not entered; 0 associations and 0 destinations | target subsector is reached when all solid-range pruning is disabled | exact-geometry control + faithful replay |
| subsector 104 ceiling | absent | node 101 skips target subsector 104; target interval `[136,155]` is covered | SEG 125 / linedef 37 contributes the decisive `[136,155]` coverage from input `[126,155]` | plane eligibility and vertical clip stages are not entered; 0 associations and 0 destinations | target subsector is reached when all solid-range pruning is disabled | exact-geometry control + faithful replay |

The counterfactual column is a deliberately broad class-level control. It is
not yet the study's required one-named-event intervention.

## Paired Subsector 104 Ceiling

```text
retained view                       rejected view
subsector 104 reached               node 101 target child considered
3 plane associations                target interval [136,155]
1 plane destination                 accumulated coverage [120,319]
                                    SEG 125 / linedef 37 supplies [136,155]
                                    child skipped
                                    plane eligibility not entered
                                    0 associations, 0 destinations
            \                      /
             first material divergence
                         ↓
          target child traversal vs solid-range prune
                         ↓
             partial occurrence vs absence
```

This resolves the earlier concern about excluding a known-good subsector 104
ceiling: the ceiling is a valid positive control. Its retained view reaches the
subsector and produces one partial destination. Only the rejected view has the
covering state that prevents traversal.

## Sky Audit

No focused covering chain contains a SEG whose plane-mark observation reports
a paired-sky ceiling adjustment. In particular, SEG 125 / linedef 37 is an
ordinary solid covering event in the rejected subsector 104 view. The two
rejected ceiling cases disappear before plane eligibility, so sky cannot be
their target-stage cause.

For these specimens, `source-invalid far-field resurrection` is a more precise
defect description than `skybox leak`: complete prepared geometry resurrects
source occurrences that the ordered renderer never produced. The older visual
name remains useful as a symptom, not as a causal diagnosis.

## Implementation Evidence

- The six retained ray definitions now have one shared identity source used by
  the existing source report and the causal report.
- `DoomClassicBspObservation` retains structured watched-child elisions and an
  ordered solid-range provenance sidecar.
- The sidecar mirrors the existing range union; a debug assertion requires its
  intervals to equal the decision union after every mutation.
- Focused provenance replays the solid events in order and selects only events
  that add previously uncovered target columns.
- The report prepares the ordered result twice and requires equality before
  reporting `replay-identical=true`.
- The exact prepared BVH remains only the target-identity control. It supplies
  no non-presentation authority.
- The no-solid-pruning traversal is a shadow-only broad counterfactual and is
  explicitly labelled as such.

## Ordinary Findings Resolved

- The first provider unit test expected one global event ordinal, but the
  solid-range and watched-elision ledgers intentionally have independent local
  ordinals. The test now verifies each ledger's own deterministic order.
- The rejected plane cases initially looked as though they required
  `ceilingclip` provenance. The causal trace showed that they never reach that
  stage; recording invented clip causes would have been wrong.

## Remaining Work

- Add a nearby wall-positive control that survives the same horizontal stages.
- Add hut, far-left and single-sky positive controls to bound the sky audit.
- Suppress one named covering mutation per rejected class and report cascades.
- If a later target is reached but disappears vertically, add per-column
  `ceilingclip` / `floorclip` ownership provenance for that actual case.
- Complete the direct Linux Doom / Chocolate Doom source cross-reference
  before labelling these E1M1 identities as direct Classic observations rather
  than faithful Tokimu replay evidence.

## Architectural Result

No architectural finding was encountered. The result reinforces the existing
AR-0030 boundary: the distinguishing participation information lives in the
source-ordered preparation protocol. Neither a BVH, a reconstructed plane
bound nor sky geometry explains these exclusions after the fact.
