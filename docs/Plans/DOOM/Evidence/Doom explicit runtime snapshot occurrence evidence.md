# Doom Explicit Runtime-Snapshot Occurrence Evidence

## Scope

This record retains Slice 5 evidence for
[Doom Ordered Source-Occurrence Preparation](../Studies/Doom%20ordered%20source%20occurrence%20preparation.md).
It asks whether the same source-preparation seam responds to explicit current
sector heights without taking ownership of door or platform movement policy.

The fixture supplies immutable snapshots only:

```text
decoded source map
    + declared current floor/ceiling heights
        ↓
Doom two-sided wall preparation
        ↓
present occurrence / changed occurrence / absent occurrence
        ↓
bounded create / replace / retire reconciliation
```

There is no activation event, clock, velocity, wait period, reversal rule, or
input handling in this path.

## Reproduction

```powershell
cargo run -p hello-doom-visibility-conformance --bin runtime_snapshot_occurrence_report
cargo test -p hello-doom-visibility-conformance source_occurrence::tests --lib
cargo clippy -p hello-doom-visibility-conformance --all-targets -- -D warnings
```

## Retained Results

The door source correlation remains `sector:1:ceiling-boundary` through four
declared states:

| State | Current ceiling | Prepared vertical range | Reconciliation |
| --- | ---: | --- | --- |
| Closed | 0 | `[0, 128]` | create occurrence/resource `501/601` |
| Opening | 48 | `[48, 128]` | replace `501/601` |
| Open | 128 | absent | retire `501/601` |
| Closing | 64 | `[64, 128]` | create `501/601` |

The platform source correlation remains `sector:0:floor-boundary`:

| State | Current floor | Prepared vertical range | Reconciliation |
| --- | ---: | --- | --- |
| Low | 0 | `[0, 128]` | create occurrence/resource `502/602` |
| Raised | 48 | `[48, 128]` | replace `502/602` |

The complete sequence reports:

```text
door identity stable=true
platform identity stable=true
current heights drive preparation=true
creates=3
replacements=2
retirements=1
unrelated resource reallocations=0
application movement policy present=false
fingerprint=be0ab8105bbaff9a2976df0b67eb0ca9ad79318c6642185ad2cf9ed56de3785c
```

Two consecutive executions produced the same fingerprint. The focused
occurrence suite passed 19 tests and strict campaign-wide Clippy passed. The
Windows incremental compiler cache again
fell back from hard links to copies; that is environment noise rather than a
semantic or validation failure.

The full library run passed 90 of 91 tests. Its sole failure remains the
independent, previously retained
`two_sided_aperture_retains_independent_upper_lower_opening_and_plane_intervals`
baseline assertion; Slice 5 neither changes nor weakens that control.

## Interpretation

The changed vertical ranges are derived from the declared current heights, not
the immutable heights originally decoded into the fixture map. Only the one
correlated door or platform occurrence changes. Source identity remains stable
while presentation presence and geometry change.

`Create`, `Replace`, and `Retire` describe bounded reconciliation required by
the prepared result. The numeric resource correlations are campaign-local
evidence; this study does not claim that `tokimu-render` owns allocation or has
admitted a public retirement API. It also does not claim that these snapshots
exercise a live Doom controller. E1M1 remains the later integration control for
application-produced runtime heights.

## Disposition

Slice 5 passes. Explicit semantic height snapshots causally update exactly the
affected prepared occurrence and boundary while application-owned movement
policy remains outside preparation. No shared renderer or engine contract is
admitted by this result.
