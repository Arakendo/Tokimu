# Doom Relational Classifier Four-Case Capture Ledger

| Field | Retained value |
| --- | --- |
| Study | [Source-Authorized Relational Contribution Classification](../Studies/Doom%20source-authorized%20relational%20contribution%20classification.md) |
| Package | `doom-shareware-corpus-v1.zip` |
| Package browser-intake BLAKE3 | `58146f5aa0e14ef38047a79878307344aec821b9f312da6a9208ec08e399660c` |
| Member | `DOOM1.WAD` |
| Map | `E1M1` |
| Embedding | `PreserveNorth` |
| Fixed view | `exterior-hut-east` |
| Source position | `(2076, -3560)` |
| Source heading | `-25.1 degrees` |
| Source eye height | `36` |
| Runtime snapshot | immutable decoded source heights; no active door or platform mutation |
| Generic filter | disabled |
| Renderer control | global full submission |

## Fixed-Pose Launch Control

The first retained launch at this pose established that the reviewed package,
member, map, embedding and global-full control load coherently. It is a launch
and inventory control, not a substitute for the four exact replay rays below.

```text
strategy=implicit-global-full
stages=original-complete-geometry>renderer-full-submission
records=1922
presentation_global=1
runtime_related=174
families=floor:463,ceiling:390,wall-upper:172,wall-lower:210,wall-middle:588,cutout-middle:26,sky-plane:73
aggregate_hash=30650e57ad9b3c07
opaque_draws=1823
cutout_draws=26
candidates=1849
rejected=12
submitted=1837
```

All twelve bounded rejections were non-owning-side masked-middle controls. The
inventory reported every one of the 1,922 original contributions as
`unresolved-fail-open`, which is expected before the relational classifier is
implemented. It proves that this launch did not silently use the new study to
delete global geometry.

Visual inspection of this same global-full control also confirms real E1M1
lazy-map pressure: complete distant rooms and overbroad floor/ceiling regions
are present in the global shell even where the fixed view's ordered source
protocol would not authorize their simultaneous presentation. This is retained
as qualitative corpus evidence for the source-support prerequisite. Exact
candidate support ranges remain pending the replayable rays and headless
source-occurrence reports; the screenshots do not define those ranges.

## Capture Rule

Each row remains open until a `LOOK` observation supplies a copyable
`--look-ray-report=source-x,source-y,source-z,dx,dy,dz` token and the headless
replay returns the same candidate source identity. A screenshot, approximate
screen coordinate, material name or inferred landmark is not sufficient.

The authorizing occurrence is resolved later from ordered preparation. It must
not be guessed from proximity to the candidate or from Candidate 1's generated
depth geometry.

Interactive console output is mirrored to the invoking terminal with the
`[doom-console]` prefix. This keeps exact replay evidence recoverable even when
the presentation window closes or its overlay is difficult to transcribe.

Start the unchanged global-full control at the frozen pose from the repository
root with:

```powershell
cargo run -q -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --exterior-hut-east-view --no-walk-collision
```

Do not add `--measure-two-frames` while collecting interactive rays; that flag
intentionally presents two bounded frames and exits. Add it only for retained
first/warm-frame measurements.

Open the debug console with backquote/tilde, aim the center ray at one named
case, and run `LOOK`. Copy its emitted replay token. Verify it headlessly with:

```powershell
cargo run -q -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --look-ray-report=<copied-source-ray>
```

| Case | Required result | Replay token | Candidate source identity | Authorizing occurrence | State |
| --- | --- | --- | --- | --- | --- |
| Far-left building, including diagonal loss | `Nearer` | pending | pending | pending | open |
| Valid outside wall / hut-adjacent structure | `Nearer` | pending | pending | pending | open |
| Far-room leak beside hut | `Beyond` | `--look-ray-report=2076.000000000,-3560.000000000,36.000000000,0.905568898,-0.424199343,0.000000000` | `wall:230:BROWN1`; linedef `230`; sidedef `327`; sector `49`; hit `(2647.655,-3827.783,36.000)` | earlier sky boundary: linedef `250`; sidedef `349`; sector `5`; distance `207.450` | candidate captured; support/relation pending |
| Far-room leak above wall | `Beyond` | pending | pending | pending | open |

### Captured far-room/beside-hut replay

The headless replay deterministically reports the ordinary wall hit at distance
`631.266`. On the same source ray it reports a finite sky boundary at distance
`207.450`, before the ordinary hit. The classic source trace identifies target
subsectors `{139, 142}` and target SEGs `[415, 423]`, reaches neither, and
records their elision by a solid horizontal range at node `141`.

```text
candidate=wall:230:BROWN1
candidate_source=linedef:230,sidedef:327,sector:49
candidate_hit=(2647.655,-3827.783,36.000)
candidate_distance=631.266
authority_source=linedef:250,sidedef:349,sector:5
authority_distance=207.450
authority_relation=before-ordinary-hit
classic_target_subsectors={139,142}
classic_target_segs=[415,423]
classic_reached=[]
classic_admitted_target_segs=[]
classic_elision=node:141,reason:solid-range,interval:[0,180],covering-range:[0,303]
```

These distances are strong preliminary `Beyond` evidence, but the row remains
open until Slice 1 derives the candidate's finite source-support overlap and
compares both occurrences in the declared prepared-view source-ray domain.
Neither the earlier boundary nor the classic elision may bypass that support
gate.

## Additional Exact Replay Set

Five further interactive `LOOK` observations were copied from terminal output
and replayed headlessly against the unchanged reviewed package. Every replay
returned the same candidate identity, hit distance, sky relation, viewer
subsector and classic-source trace as its interactive observation. They are
therefore retained as exact classifier inputs, but are not silently assigned
to one of the four named visual rows: the human visual-to-case attribution was
not recorded with the terminal transcript.

| Ray | Replay token | Candidate | Earlier source authority | Classic-source result |
| --- | --- | --- | --- | --- |
| R1 | `--look-ray-report=1306.508666992,-3272.168457031,21.432840347,0.939651787,-0.338751376,0.047981590` | wall `247`, sidedef `344`, sector `56`, `BROWN96`, distance `1742.658` | sky boundary linedef `250`, sidedef `349`, sector `5`, distance `1109.461` | target subsectors `{190,192}` not reached; target SEGs `[559,567]` elided by node `197` solid range |
| R2 | `--look-ray-report=1477.330444336,-3594.213134766,8.994521141,-0.792175531,-0.565008104,0.230702817` | ceiling flat, subsector `104`, sector `40`, `CEIL3_5`, distance `273.102` | sky boundary linedef `252`, sidedef `353`, sector `5`, distance `95.197` | target subsector `104` reached; no target SEG or elision |
| R3 | `--look-ray-report=2115.047851562,-3569.925048828,8.994521141,0.928815067,-0.358562857,0.093463443` | wall `247`, sidedef `344`, sector `56`, `BROWN96`, distance `892.484` | sky boundary linedef `250`, sidedef `349`, sector `5`, distance `217.744` | target subsectors `{190,192}` not reached; target SEGs `[559,567]` elided by node `197` solid range |
| R4 | `--look-ray-report=2139.683349609,-3196.036376953,8.994521141,0.180356100,0.780082107,0.599119186` | ceiling flat, subsector `149`, sector `7`, `CEIL3_5`, distance `358.869` | sky-plane occurrence at subsector `130`, sector `5`, distance `345.516` | target subsector `149` not reached; node `152` solid-range elision |
| R5 | `--look-ray-report=2902.150878906,-3206.857421875,8.994521141,-0.952072978,-0.304107845,0.032795019` | ceiling flat, subsector `104`, sector `40`, `CEIL3_5`, distance `1921.191` | sky boundary linedef `252`, sidedef `353`, sector `5`, distance `1450.612`; a different sky plane lies behind the candidate | target subsector `104` not reached; node `101` solid-range elision |

This set falsifies a tempting shortcut. `classic target not reached` cannot be
the classifier by itself: R2 reaches its target while still placing a finite
sky boundary before the ordinary hit. Conversely, a nearer boundary cannot
reject by distance alone until candidate support and the authority's finite
horizontal and vertical domains overlap. The retained result therefore
supports the study's ordered decision shape:

```text
candidate source support
    -> finite authority overlap
    -> relational depth
    -> keep / reject / split / unresolved-fail-open
```

The five rays are sufficient to begin headless support/relationship work. They
do not close Slice 0's four named rows until each replay is explicitly tied to
the visual case that motivated it.

## Corpus-Private Slice 1 Gate

The first bounded classifier implementation now lives only in the Doom
visibility corpus:

```text
corpus/campaigns/doom/hello-doom-visibility-conformance/
    src/relational_classifier.rs
    src/bin/relational_classifier_report.rs
```

It admits no Tokimu renderer vocabulary. The model makes three decisions in
order:

1. intersect a SEG- or plane-occurrence-local candidate with the authority's
   finite source-parameter, horizontal and vertical intervals;
2. choose an authority by retained Doom order, failing open on equal-order
   ambiguity rather than choosing by proximity or material;
3. compare deterministic positive source-ray parameters and report `Nearer`,
   `Beyond`, `Straddling` or `Unresolved`.

Partial finite overlap retains the excluded source, horizontal and vertical
ranges explicitly. Empty overlap produces `OutsideSourceSupport` and never
reaches relational depth. Missing samples, mixed center/edge conventions,
parallel or behind-view relations, near-plane ambiguity, non-finite values,
negative ray parameters, unresolved support and ambiguous authority order all
fail open. Candidate normal/facing is retained beside the result but cannot
change it.

Run the bounded report with:

```powershell
cargo run -p hello-doom-visibility-conformance --bin relational_classifier_report
```

The focused unit gate currently passes nine classifier tests. The broader
crate gate passes 126 tests. These results prove the decision mechanics, not
the four E1M1 classifications: the exact E1M1 rays still need occurrence-local
finite support derived from the ordered source ledger and named visual-case
attribution.

## Corpus-Private Slice 2 Gate

The partial-contribution model now partitions one complete source contribution
across source-parameter, horizontal and vertical support before applying a
piecewise-linear depth relationship. Every resulting region is explicitly one
of retained-nearer, rejected-beyond, outside-source-support or
unresolved/fail-open. The model preserves source identity, sidedef role,
material identity and UV progress.

The bounded report retains this deterministic control:

```text
fragments=8
retained=1
rejected=1
outside-support=6
unresolved=0
conserved=true
renderer-policy=none
stable-contract=none
```

The focused classifier gate passes 18 tests after adding the ordered-authority
falsifier. The falsifier supplies two finite solid authorities over distinct
parts of one candidate. The current first-authority resolver correctly selects
the earlier occurrence, but necessarily labels the later authority's region as
outside that first support even though the later occurrence independently owns
it. Consequently, single-authority keep/reject/split cannot represent the full
ordered result. The next honest model is ordered partitioned composition over
the unresolved remainder; ad hoc priority or whole-object filtering is not an
acceptable repair.

This closes Slice 2 at its explicit architectural stop and leaves synthetic
presentation paused pending AR-0030 judgment. It does not classify the named
E1M1 visual cases.

## Comparison Record Shape

Every completed row must retain one domain-consistent comparison:

```text
comparison_domain=prepared-view-source-ray-t
candidate_t=...
authority_t=...
finite_source_parameter=[...]
candidate_support_identity=...
candidate_support_range=[...]
outside_support_range=[...]
authorized_horizontal_interval=[...]
authorized_vertical_interval=[...]
sample_convention=column-center|column-edge|continuous-ray
classification=Nearer|Beyond|Straddling|Unresolved
reason=...
```

Candidate facing/normal may be retained beside the result, but it is never the
authorizing fact. `Unresolved` fails open.

## Current Disposition

The shared package, pose and runtime snapshot are frozen. Six exact headless
replays are retained: the named beside-hut leak plus the five-ray set above.
The four case rows are deliberately not all claimed yet because the five-ray
terminal transcript did not retain their human visual-case labels. Slice 0
remains active until those exact observations are attributed to the named
cases (or replacement rays are captured with labels). The fixed-pose launch
control and reproducible rays prove the capture composition and diagnostic
path; they do not substitute for that attribution.
