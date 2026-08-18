# Doom Source-Ordered Non-Presentation Causality Study

| Field | Value |
| --- | --- |
| Status | Experimental realization addendum complete and falsified — hut presentation improved, but exact plane false-retention and false-omission captures reject Boolean reached-domain authority |
| Scope | Explain the exact causal chain by which Doom produces no presentation occurrence for selected E1M1 source contributions |
| Parent review | [AR-0030](../../../Architectural%20Reviews/AR-0030-source-owned-presentation-preparation-boundary.md) |
| Authority input | [Doom Ordered Source-Occurrence Preparation](Doom%20ordered%20source%20occurrence%20preparation.md) |
| Spatial falsifier | [Doom Custom BVH View-Cell And Aperture Study](Doom%20custom%20BVH%20view-cell%20aperture%20study.md) |
| Existing source evidence | [Classic Doom Visibility Clipping Evidence](../Evidence/Classic%20Doom%20visibility%20clipping%20evidence.md) |
| Existing protocol ledger | [Doom E1M1 Ordered Source Protocol Ledger Evidence](../Evidence/Doom%20E1M1%20Ordered%20Source%20Protocol%20Ledger%20Evidence.md) |
| Stable API authority | None |
| Renderer changes authorized | No stable renderer/API change; one explicit corpus-private presentation strategy is authorized for walkabout evaluation |
| Result checkpoint | [2026-08-18 Doom Non-Presentation Causality Slice 1](../../../Checkpoints/2026-08-18-doom-non-presentation-causality-slice1.md) |
| Counterfactual checkpoint | [2026-08-18 Doom Non-Presentation Exact Counterfactuals](../../../Checkpoints/2026-08-18-doom-non-presentation-exact-counterfactuals.md) |
| Walkabout checkpoint | [2026-08-18 Doom Source-Covered Walkabout Experiment](../../../Checkpoints/2026-08-18-doom-source-covered-walkabout-experiment.md) |
| Falsifier checkpoint | [2026-08-18 Doom Source-Covered Walkabout Falsifiers](../../../Checkpoints/2026-08-18-doom-source-covered-walkabout-falsifiers.md) |
| Next action | Return to AR-0030; correlate wall 241 with final wall-occurrence output, then decide whether to authorize a source-plane occurrence-support realization study |

## Question

For each E1M1 contribution that the complete prepared shell contains but Doom
does not present from a particular view, what exact ordered source event caused
that absence?

The study asks for a mechanistic answer in Doom renderer terms:

```text
not merely
    target absent
    target subsector not reached
    solid range covered it

but
    target occurrence
        ↓ projected to this source interval
    earlier source event X
        ↓ established this exact covering interval / clip boundary
    later target stage Y
        ↓ had no surviving interval, row range or plane mark
    therefore
        no presentation occurrence was produced
```

The intended result is a causal explanation, not another classifier that tries
to reproduce Doom from geometry after the fact.

## Terminology Correction

The user-visible symptom looks like a sector was rejected, but Classic Doom
does not generally submit or reject a complete sector as one render object.

The relevant source units are:

- BSP children and subsectors visited in near-to-far order;
- SEGs admitted, clipped or never visited;
- contiguous wall ranges passed to `R_StoreWallRange`;
- upper, lower and middle wall tiers bounded per column;
- floor and ceiling marks produced from the same clip state;
- source-keyed plane instances and their surviving spans; and
- deferred masked contributions.

Consequently the study will use this formulation:

> Why did this exact source-correlated wall or plane occurrence fail to be
> produced for this prepared view?

It may still report sector identity as provenance, but `sector rejected` is
not an accepted causal reason.

## Why Existing Evidence Is Not Enough

The retained six-ray report already establishes final outcomes:

| Case | Exact prepared geometry | Ordered result |
| --- | --- | --- |
| `hut-east-wall-230` | `wall:230:BROWN1` | rejected; SEGs 415/423 produce no declarations |
| `wall-247-east` | `wall:247:BROWN96` | rejected; SEGs 559/567 produce no declarations |
| `ceiling-104-reached` | `flat:40:CEIL3_5` | retained as a partial plane occurrence |
| `wall-247-west` | `wall:247:BROWN96` | rejected; SEGs 559/567 produce no declarations |
| `ceiling-149-rejected` | `flat:7:CEIL3_5` | no association, destination or declaration |
| `ceiling-104-rejected` | `flat:40:CEIL3_5` | no association, destination or declaration |

Those results answer **what** Doom's ordered protocol did. They do not yet
answer all of the following:

- Which exact earlier SEG or boundary supplied the covering authority?
- Was the target BSP child skipped, or was the target SEG reached and clipped?
- If a child was skipped, what was its projected interval and which accumulated
  solid ranges covered it?
- Did a target wall have zero horizontal survivors, or did its particular
  upper/lower/middle tier collapse later?
- Was a plane absent because its subsector was not visited, because it was not
  eligible, because no interval was marked, or because its per-column vertical
  range was already empty?
- What differs in the retained and rejected subsector 104 ceiling views?
- What role, if any, did paired sky actually play?

`terminally rejected`, `source-covered`, `not reached` and `zero associations`
remain summary dispositions. This study must expand them into source-event
chains.

## Working Causal Model

Classic Doom constructs presentation incrementally rather than selecting from
a completed global mesh:

```text
near-first BSP traversal
    ↓
source-facing and horizontal-FOV SEG admission
    ↓
solid/pass classification
    ↓
accumulated horizontal solid-range clipping
    ↓
R_StoreWallRange for each surviving interval
    ↓
shared ceilingclip / floorclip mutation
    ├── bounded wall-tier columns
    └── floor / ceiling plane marks
            ↓
        source-keyed plane instances and spans
```

A farther contribution often does not undergo an explicit object-level
rejection. The renderer reaches a point where no source-authorized presentation
range remains, so no later occurrence is produced.

The initial causal categories are deliberately stage-specific:

1. `outside-horizontal-view` — the source projection lies outside the Classic
   horizontal view domain.
2. `backface-or-zero-projection` — `R_AddLine` admits no projected SEG range.
3. `far-child-solid-range-covered` — accumulated nearer solid ranges prevent
   BSP recursion into the target child.
4. `target-seg-solid-range-covered` — the target SEG is reached, but its entire
   projected interval is already horizontally covered.
5. `wall-tier-vertically-empty` — a surviving horizontal wall range exists,
   but the requested tier has no open vertical interval.
6. `plane-not-eligible` — the current front/back sector relationship does not
   mark that floor or ceiling.
7. `plane-mark-clipped-empty` — the plane is eligible, but shared per-column
   clip state leaves no cells.
8. `plane-instance-not-emitted` — marks exist but no final source-keyed span
   survives.
9. `masked-deferred` — the occurrence is deferred and explicitly did not
   create solid occlusion authority.
10. `unresolved` — the current replay cannot prove one of the preceding causes.

These names are hypotheses for the diagnostic ledger, not new public enums or
prejudged outcomes for the retained cases.

## What Counts As A Causal Explanation

Each target report must identify two related events:

### First decisive target event

The earliest target-specific stage after which the target can no longer
produce the queried presentation occurrence in the unmodified replay.

Examples:

```text
target far child was not traversed
target SEG projected range had no uncovered columns
target ceiling mark was empty in every relevant column
```

### Covering provenance

The earlier source event or events whose accumulated state authorized that
decision.

Examples:

```text
near SEG 270 inserted solid range [179,221]
    ↓
target child bbox projected to [186,208]
    ↓
range fully covered
    ↓
target subsector not visited
```

or:

```text
prior upper/lower wall tiers moved ceilingclip/floorclip
    ↓
target ceiling was eligible
    ↓
top/bottom interval empty at all target columns
    ↓
no plane cells or spans emitted
```

A report that cannot name covering provenance must say `unresolved`; it may not
promote correlation, spatial distance, a BVH candidate or a sky texture into a
cause.

## Private Causal Ledger

The initial implementation should extend the existing Doom-private reference
planner with observation-only provenance. It must not change traversal or clip
decisions.

Each event should retain enough information to print:

```text
event identity and predecessor
prepared-view and runtime-snapshot identity
renderer stage
source node / child / subsector / SEG / linedef / sidedef / sector
target correlation, if any
input projected interval
solid-range or vertical-clip state before the event
operation and exact reason
state after the event
source events responsible for newly covered ranges
wall-tier and plane-mark consequences
downstream target occurrences produced or prevented
```

Accumulated `solidsegs` need a diagnostic provenance sidecar so a covered range
can name which nearer source events contributed its coverage. The historical
renderer does not need such provenance to draw; this study needs it to answer
the causal question. The sidecar is evidence only and must reproduce the same
coverage union and decisions as the existing replay.

Plane diagnostics must similarly preserve the wall-range event responsible for
each `ceilingclip` or `floorclip` mutation. A final empty mark should be
traceable to the ordered boundary changes that made it empty.

For every target-relevant column mutation, the retained record should be
inspectable in this form:

```text
column
clip value before
responsible SEG / linedef / sidedef and wall tier
clip value after
plane marks affected by the mutation
```

This is causal provenance added to the diagnostic replay. It is not a claim
that Classic Doom retained historical ownership for each clip-array value.

## Reference And Replay Discipline

The causal explanation has three evidence levels:

1. **Direct Classic behavior** — stage semantics verified against Linux Doom
   1.10 and a faithful control such as Chocolate Doom.
2. **Tokimu faithful replay observation** — exact E1M1 source identities,
   intervals and state mutations produced by the existing reference planner.
3. **Tokimu inference** — correlation from the source occurrence to persistent
   render-subsector geometry or a possible modern realization.

Reports must label these levels. A convenient Rust-side provenance structure
does not become a claim that Classic Doom stored that structure.

The fixed `320 x 200` diagnostic domain remains valid as an oracle for what the
released source protocol did. It does not become world geometry, camera API or
Tokimu renderer vocabulary.

## Paired-Sky Question

The study must answer sky's role rather than assuming it:

- Did a paired-sky comparison suppress an upper wall tier?
- Did it alter ceiling marking or plane identity?
- Was the target already excluded by an ordinary solid range before sky was
  relevant?
- Did sky merely paint a source-authorized opening after the real participation
  decision had already occurred?

The permitted conclusion may be `sky was not causal`.

`F_SKY1`, a paired-sky boundary or a visible sky pixel cannot independently be
reported as the reason distant world geometry was rejected unless the exact
source trace establishes the corresponding ordered transition.

## Counterfactual Shadow

After the unmodified causal ledger is deterministic, a bounded counterfactual
replay may suppress one named covering event while keeping source input and
camera fixed:

```text
normal replay
    covering event X present
    target occurrence absent

counterfactual shadow
    suppress only X's coverage mutation
    target interval / mark / occurrence appears or remains absent
```

This can distinguish a necessary covering event from a merely nearby event.
It remains diagnostic intervention, not a proposed renderer rule. Cascading
changes must be reported, and an intervention that changes unrelated ordering
too broadly is inconclusive rather than positive proof.

## Proposed Command And Human Workflow

The first headless interface should be:

```powershell
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --ordered-non-presentation-causality-report
```

It should print all six retained cases and a deterministic summary. A later
interactive diagnostic may add:

```text
WHY
WHY PIXEL x y
WHY SOURCE wall:230
```

That interactive command is optional and must reuse the same frozen-view
causal ledger. It must not infer causality from the nearest prepared triangle
alone.

## Slices

### Slice 0 — Freeze Targets And Causal Contract

- [x] Reuse the six exact retained BVH/source rays and their expected ordered
      outcomes without changing their identities.
- [x] Define target wall-tier and plane-occurrence correlation explicitly.
- [x] Define the private causal event, covering-provenance and unresolved
      ledgers.
- [x] Prove that enabling observation changes no ordered outcome, declaration
      fingerprint or renderer submission.

Acceptance: all six final outcomes and existing fingerprints are unchanged;
every event is observation-only and source-correlated.

### Slice 1 — Horizontal BSP And SEG Causality

- [x] Record near/far BSP child traversal with projected bbox intervals and
      the exact accumulated solid ranges considered at each decision.
- [x] Add provenance to diagnostic solid-range union entries without changing
      their coverage semantics.
- [x] Record `R_AddLine` admission, projection, solid/pass classification and
      every surviving `R_StoreWallRange` interval for target and covering SEGs.
- [x] Explain `hut-east-wall-230`, `wall-247-east` and `wall-247-west` through
      their first decisive events and covering source chains.
- [x] Retain one nearby wall-positive control whose interval survives the same
      stages.

Acceptance: each absent wall has an exact target interval, exact covering
intervals, exact covering source identities and a named divergence from the
positive control. `behind sky` is not an accepted substitute.

### Slice 2 — Plane Non-Presentation Causality

- [x] Trace target subsector visitation and plane eligibility.
- [x] Attribute each relevant `ceilingclip` and `floorclip` mutation to its
      ordered wall-range event.
- [x] Trace target plane marks through source-keyed instance and span emission.
- [x] Explain `ceiling-149-rejected` and `ceiling-104-rejected` without treating
      a whole sector or reconstructed plane mesh as the unit of authority.
- [x] Compare both with `ceiling-104-reached` and identify the earliest causal
      difference.
- [x] Produce a synchronized retained/rejected subsector 104 trace aligned by
      source identity and renderer stage where possible, then identify the
      first material divergence in traversal, wall range, clip state, plane
      marking or span emission.

Acceptance: the paired subsector 104 cases identify exactly why one view emits
a partial occurrence and the other emits none. Every empty plane result is
distinguished as not visited, not eligible, clipped empty or not emitted. The
answer must show the earliest differing source fact or mutation rather than
only comparing final ledgers.

### Slice 3 — Paired-Sky Causal Audit

- [x] Identify every paired-sky event on the retained causal chains.
- [x] State whether it changes a wall tier, plane mark, plane key, terminal
      coverage or only final sky presentation.
- [x] Prove which rejected cases would remain rejected without treating sky as
      physical geometry.
- [ ] Retain hut, far-left and single-sky positive controls against an overly
      broad `sky caused it` explanation.

Acceptance: sky's role is described by exact ordered source mutations. Cases
where sky is merely correlated are labelled non-causal.

The first six-ray ledger finds no paired-sky SEG in any of the five exact
covering chains. The two rejected ceiling targets never enter plane eligibility
or vertical clipping, so no target-relevant clip-array mutation exists to
attribute in those views; their first decisive event is earlier BSP child
pruning. This closes the target-specific clip question without claiming that
generic vertical-clip provenance is complete. Hut, far-left and single-sky
positive controls remain open against over-generalization.

### Slice 4 — Bounded Counterfactuals

- [x] Suppress one named covering mutation per rejected class in a shadow-only
      replay.
- [x] Report whether the target becomes visited, gains a wall interval, gains
      plane marks or remains absent for another reason.
- [x] Record cascading changes and classify broad interventions as
      inconclusive.
- [x] Make no presentation-affecting submission change.

Acceptance: counterfactuals corroborate or narrow the normal causal chains;
they are never used as the desired rendering policy.

The broad control disables all solid-range BSP pruning and reaches all five
absent targets. The exact control additionally suppresses each of the 20
focused covering SEG mutations independently. Twelve interventions reopen at
least part of the target domain, and every absent case has at least one such
event. Cascades are reported as newly/lost visited subsectors, newly/lost
admitted SEGs and the before/after far-child prune count. The result proves
event necessity for the original target-child decision where reopening occurs;
it does not turn those SEGs into permanent occluder objects or a desired
renderer policy.

### Slice 5 — Explanation And Realization Handoff

- [x] Produce one concise causal table for the six rays.
- [x] Separate direct Classic facts, replay observations and Tokimu inference.
- [x] Record which source-ordered invariants a free-look plane realization must
      preserve.
- [x] If sky is non-causal for the retained leak cases, decide whether the
      evidence supports renaming the defect class from `skybox leak` to
      `source-invalid far-field resurrection` or another source-faithful term.
- [x] Return to AR-0030 before changing presentation authority or implementing
      a new realization.

Acceptance: a maintainer can answer “why does Doom not present this?” by
naming the first decisive source event and its covering provenance, not by
pointing at a final screenshot or spatial proxy.

### Experimental Realization Addendum — Source-Covered Global Shell

The maintainer authorized one presentation-affecting E1M1 test after the
causal ledger established that the five retained leak specimens disappear
before their source domains are traversed. This is an explicit, non-default
corpus strategy named `source-covered-global-shell`; it does not alter
`tokimu-render` or claim a stable Tokimu visibility contract.

- [x] Begin from the original complete prepared E1M1 shell.
- [x] Replay ordinary near-first Doom BSP coverage for the actual horizontal
      camera position and heading.
- [x] Retain each complete floor/ceiling draw whose owning subsector was
      reached; never apply a SEG or child proxy bbox to the reconstructed
      plane.
- [x] Retain a wall draw when any resolved owning subsector was reached.
- [x] Fail open when source ownership cannot be resolved.
- [x] Prepare the complete next draw set before replacing the active set.
- [x] Add exact headless controls for the five absent specimens, the reached
      subsector 104 ceiling and the nearby SUPPORT2 wall.
- [x] Smoke-test two native frames at the source spawn.
- [x] Complete visual walkabout review far enough to decide the candidate:
      the hut improves materially, but exact spawn-window and hut captures
      falsify reached-subsector authority over complete planes.
- [ ] Exercise door/platform runtime changes before treating the strategy as a
      lifecycle-complete candidate.

The plane policy is deliberately whole-source-owner geometry. The experiment
uses ordered source traversal to decide whether a domain participates, while
Tokimu continues to realize that participating domain as ordinary complete
geometry. It does not attempt Classic visplane or screen-column reconstruction.

Acceptance for the visual experiment is asymmetric: the five known
source-covered far-field specimens must disappear, while the reached ceiling,
nearby wall and complete local room remain present without stale or partial
refresh frames. A visual failure is evidence about this candidate, not license
to move Doom vocabulary into the renderer.

The walkabout did fail that acceptance. Three reached subsectors retain whole
ceilings for which the exact source plane key is absent, while one nearby floor
is omitted because its subsector proxy is skipped even though the source plane
key is present. A nearby alternate pose also shows that key presence cannot
authorize every point of the complete correlated mesh. The next realization
question is therefore view-local source-plane occurrence support over actual
reconstructed support, not another Boolean whole-plane predicate.

## Required Summary Table

The completed study must produce at least:

| Target | Final outcome | First decisive event | Covering provenance | Downstream consequence | Counterfactual | Evidence level |
| --- | --- | --- | --- | --- | --- | --- |
| wall/plane identity | absent/whole/partial | exact stage and interval | exact source event chain | no range/mark/span | target appears/remains absent/inconclusive | Classic direct / replay / inference |

The positive `ceiling-104-reached` row is mandatory. Without it, the study has
not shown why the negative ceiling is different.

The completed evidence must also include a side-by-side event comparison for
the retained and rejected subsector 104 ceiling views:

```text
retained view                 rejected view
source/stage A                source/stage A
source/stage B                source/stage B
clip state                    clip state
        \                    /
         first material divergence
                    ↓
      partial occurrence vs absence
```

Sequence numbers need not align between views. Source identity and renderer
stage are the comparison keys; any unmatched event remains explicit.

## Conservation And Determinism

Every replay must retain:

- source contribution counts;
- BSP/subsector/SEG visitation counts;
- wall-range and plane-mark counts;
- emitted whole, partial and absent occurrence counts;
- unresolved causal chains;
- event and final-result fingerprints; and
- unchanged ordinary declaration and renderer-submission fingerprints.

Adding provenance may increase diagnostic memory and report time. It must not
change source ordering, clipping arithmetic or presentation outcomes.

## Architectural Boundary

This study assumes only that the existing ordered protocol is the current
source-participation oracle under investigation. It does not yet change a
stable Tokimu contract.

```text
Doom source + runtime snapshot + prepared view
        ↓
source-ordered causal replay and provenance
        ↓
why an occurrence was whole / partial / absent
        ↓
future Doom-private realization research
```

The exact-geometry BVH may identify and verify the queried prepared triangle.
It cannot explain its source non-presentation. Render-subsector connectivity
and aperture chains may be printed as comparative diagnostics, but they cannot
replace the ordered causal chain.

Any proposal to expose BSP, `solidsegs`, screen columns, visplanes, portals or
Doom source roles through `tokimu-render` is an architectural finding and
returns to AR-0030 before implementation.

## Non-Goals

- No new spatial rejection proxy.
- No WAD BSP rebake.
- No claim that a complete sector is the presentation unit.
- No sky depth wall, invisible occluder or generic portal terminal.
- No repair of the parked 365-draw or render-subsector candidates.
- No arbitrary-pitch realization in the causal-ledger slices.
- No source-column or visplane vocabulary in stable Tokimu APIs.
- No renderer implementation, stable API or default-strategy change; the
  named corpus-private walkabout strategy is the sole presentation-affecting
  addendum.
- No assumption that the first nearby geometric object caused the exclusion.

## Parking And Escalation Criteria

Ordinary missing provenance, incorrect correlation, nondeterminism or replay
instrumentation is local implementation work.

Return to AR-0030 if:

- Classic source behavior and the current ordered oracle disagree materially;
- a retained final outcome cannot be explained without changing the accepted
  source unit or authority;
- arbitrary-pitch input is required to establish the historical neutral-view
  cause;
- causal observation would require renderer-owned Doom semantics; or
- the result contradicts the conclusion that the ordered protocol contains the
  distinguishing participation information.

Park the study if exact causal provenance cannot be recovered without changing
the decisions being observed. Do not fill missing causal evidence with BVH,
distance, cell reachability or sky correlation.

## Expected Value

The study should turn the current broad statement:

> Doom's ordered protocol rejects distant geometry behind the visible scene.

into exact statements such as:

> Near source event X established solid interval A. The target child projected
> wholly inside A, so `R_RenderBSPNode` did not visit target subsector Y. Its
> target SEG and plane marks were therefore never produced.

or:

> The target subsector was visited and its ceiling was eligible, but earlier
> wall-range events X and Z closed the shared vertical interval in every target
> column. No source-keyed ceiling span survived.

That evidence will identify the invariants a modern realization must preserve
without mistaking Classic Doom's raster storage for Tokimu geometry.
