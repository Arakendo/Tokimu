# Doom E1M1 Ordered Runtime Snapshot Correlation Evidence

## Claim

Explicit immutable current-height snapshots for the E1M1 first door and moving
platform change declarations produced by the same ordered source-occurrence
preparation seam as the Slice 6 fixed-view candidate. The fixture does not
recreate activation, timing, waiting, reversal, or input policy.

This establishes runtime-state causality through the occurrence-bounded plane
candidate. It does not yet prove live animated integration or visual
source-faithfulness at the canonical matrix.

## Method

For each moving sector, the report derives deterministic source-boundary-local
views from linedefs touching that sector. It prepares the unchanged map and an
immutable projected-height snapshot through
`prepare_ordered_occurrence_submission`, then compares declarations whose
source sector or adjacent source linedef belongs to the target boundary.

```text
same decoded E1M1 source
+ same deterministic local view
        + current-height snapshot A/B
            -> ordered source occurrences
            -> shared wall/plane boundaries
            -> ordinary Tokimu declarations
```

The local views are diagnostic inputs only. They do not become camera policy or
runtime movement behavior.

## Retained observations

### First manual door

```text
target sector                         4
ceiling snapshot                   0 -> 68
view                         linedef 151/right
source occurrences             242 -> 250
bounded source fail-open        241 -> 241
all declarations                442 -> 503
target declarations                 4 -> 12
target declaration changed               true
downstream lowering unresolved              0
```

Opening the door changes the viewer-relative occurrence set. Runtime motion is
therefore not merely an edit to a permanently fixed mesh inventory.

### Down/wait/up platform

```text
target sector                        70
floor snapshot                   104 -> -48
view                         linedef 471/right
source occurrences             319 -> 319
bounded source fail-open        316 -> 316
all declarations                571 -> 573
target declarations               19 -> 21
target declaration changed               true
downstream lowering unresolved              0
```

Here the ordered occurrence identity remains stable while target wall/plane
declarations change. The same preparation seam therefore supports both a
topology-visible change and a geometry-only current-height change.

`source-fail-open` counts refer to explicit near-plane ambiguity retained by
the ordered observer. They are not lowering failures and remain unchanged
between each baseline/snapshot pair. The separately reported downstream
wall/plane lowering failure count is zero.

## Command

```powershell
cargo run -q -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --ordered-occurrence-runtime-snapshot-report
```

## Boundaries retained

- Application-owned activation and timing policy is absent.
- No imported `DoomMapCore` is mutated.
- No renderer resource is created, replaced, or retired by the report.
- No generic camera filter participates.
- Source-boundary-local view selection is diagnostic-only.
- Live door/platform animation remains a later integration control.
