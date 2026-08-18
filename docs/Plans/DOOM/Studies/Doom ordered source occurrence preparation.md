# Doom Ordered Source-Occurrence Preparation

| Field | Value |
| --- | --- |
| Campaign | DOOM |
| Role | Successor candidate-realization study |
| Status | Parked — structural conservation and live refresh are retained, but source-spawn visual completeness is falsified by large and interior sky leaks |
| Parent review | [AR-0030](../../../Architectural%20Reviews/AR-0030-source-owned-presentation-preparation-boundary.md) |
| Controlling plan | [DOOM WAD Checklist](../DOOM%20WAD%20Checklist.md) |
| Predecessor | [Doom Source-Topology Admission Over Complete Geometry](Doom%20source-topology%20admission%20over%20complete%20geometry.md) |
| Source evidence | [Classic Doom Visibility Clipping Evidence](../Evidence/Classic%20Doom%20visibility%20clipping%20evidence.md) |
| Renderer dataflow | [Classic Doom Renderer Dataflow And Tokimu Preparation Seam](../Evidence/Classic%20Doom%20renderer%20dataflow%20and%20Tokimu%20preparation%20seam.md) |
| Reference trace | [Doom Ordered Source-Occurrence Reference Trace](../Evidence/Doom%20ordered%20source%20occurrence%20reference%20trace.md) |
| Occurrence-model evidence | [Doom Private Source-Occurrence Model Evidence](../Evidence/Doom%20private%20source-occurrence%20model%20evidence.md) |
| Partial-survival evidence | [Doom Headless Partial-Survival Reconstruction Evidence](../Evidence/Doom%20headless%20partial-survival%20reconstruction%20evidence.md) |
| Shared-boundary evidence | [Doom Shared Wall/Plane Boundary Conservation Evidence](../Evidence/Doom%20shared%20wall-plane%20boundary%20conservation%20evidence.md) |
| Presentation-lowering evidence | [Doom Ordinary Occurrence Presentation Lowering Evidence](../Evidence/Doom%20ordinary%20occurrence%20presentation%20lowering%20evidence.md) |
| E1M1 occurrence observation | [Doom E1M1 Continuous Source Occurrence Observation Evidence](../Evidence/Doom%20E1M1%20continuous%20source%20occurrence%20observation%20evidence.md) |
| First prepared-submission evidence | [Doom E1M1 Ordered Fixed-View Prepared Submission Evidence](../Evidence/Doom%20E1M1%20ordered%20fixed-view%20prepared%20submission%20evidence.md) |
| Runtime snapshot correlation | [Doom E1M1 Ordered Runtime Snapshot Correlation Evidence](../Evidence/Doom%20E1M1%20ordered%20runtime%20snapshot%20correlation%20evidence.md) |
| Slice 6B literal handoff | [Doom E1M1 Slice 6B Literal Handoff Evidence](../Evidence/Doom%20E1M1%20Slice%206B%20literal%20handoff%20evidence.md) |
| Reference-planner evidence | [Doom Ordered Reference Planner Synthetic Gate Evidence](../Evidence/Doom%20ordered%20reference%20planner%20synthetic%20gate%20evidence.md) |
| E1M1 source-protocol ledger | [Doom E1M1 Ordered Source Protocol Ledger Evidence](../Evidence/Doom%20E1M1%20ordered%20source%20protocol%20ledger%20evidence.md) |
| Initial target | The retained partial-survival falsifier: one far source contribution with valid left/right survivors and a forbidden middle interval |
| Focused negative studies | [Doom Authoritative Sky-Coverage Delta Realization](Doom%20authoritative%20sky%20coverage%20delta%20realization.md); [Doom Source-Authorized Relational Contribution Classification](Doom%20source-authorized%20relational%20contribution%20classification.md) |
| Next action | Continue through [Doom Render-Subsector Actual-Camera Preparation](Doom%20render-subsector%20actual-camera%20preparation.md). Do not repair this candidate, advance its browser path, or enable the generic post-filter while it is parked. |

## Purpose

Determine whether Doom can fully resolve presentation membership,
multiplicity, and bounded fragmentation before `tokimu-render` receives work,
without copying Doom's historical rasterizer into Tokimu or weakening the
handoff to whole-object Boolean selection.

The predecessor study established that this decision is not always:

```text
submit this source contribution
do not submit this source contribution
```

One far contribution required two surviving source-relative regions separated
by a forbidden middle region. Therefore this successor tests the richer
Doom-owned operation:

```text
source contribution + runtime snapshot + prepared view
        ↓
ordered Doom source preparation
        ↓
0..N presentation occurrences
        ↓
ordinary Tokimu presentation declarations
        ↓
tokimu-render
```

The renderer remains downstream. It does not learn Doom sectors, BSP nodes,
SEGs, visplanes, screen columns, sky rules, or door policy.

## Evidence Carried Forward

The complete released-renderer dataflow and the successor preparation seam are
now recorded in
[Classic Doom Renderer Dataflow And Tokimu Preparation Seam](../Evidence/Classic%20Doom%20renderer%20dataflow%20and%20Tokimu%20preparation%20seam.md).
That analysis supersedes the assumption that horizontal occurrence domains can
authorize complete wall and plane survival. Slice 6 must first establish a
Doom-private ordered coverage reference planner whose wall, vertical-window,
plane-instance, sky, and masked effects are conserved together.

The predecessor's retained falsifier is binding input to this study:

| Pose | Forbidden middle columns | Required surviving columns |
| --- | ---: | ---: |
| Baseline | 81 | 15 |
| Nearer control | 97 | 9 |

Whole admission preserves forbidden participation. Whole rejection destroys
required participation. Camera jitter did not remove the contradiction.

The predecessor remains completed and insufficient. This plan does not reopen
its hypothesis or reinterpret its parked E1M1 slices as unfinished work.

## Working Hypothesis

The useful source-owned operation is preparation rather than selection:

```text
definitely invisible → reject the whole contribution
fully visible        → retain the whole contribution unchanged
partially visible    → emit bounded source-relative occurrences
uncertain            → fail open with retained evidence
```

Whole retain and whole reject remain cheap outcomes. Fragment generation is
paid only when ordered source evidence requires partial participation.

This hypothesis is deliberately Doom-local. AR-0030 may later recognize a
provider-neutral lifecycle or submission handoff only after Quake and non-BSP
campaigns supply independent evidence.

## Terms

- **Source contribution** — one stable Doom-owned wall-role, plane, cutout, or
  other presentation input with source provenance.
- **Presentation occurrence** — one view/snapshot-specific realization of all
  or part of a source contribution. One source may produce zero, one, or more
  occurrences.
- **Source-relative domain** — continuous parameterization over original
  source geometry, such as a SEG interval, retained independently of diagnostic
  screen columns.
- **Shared prepared boundary** — one causal boundary used by both wall and
  plane preparation so independently reconstructed approximations cannot crack.
- **Diagnostic raster** — the bounded `320 x 200` observation grid used as an
  oracle. It is not semantic geometry or renderer API vocabulary.

## Candidate Dataflow

```text
decoded Doom source
    + explicit current runtime snapshot
    + prepared view
        ↓
near-to-far BSP/source traversal
        ↓
ordered solid/pass clipping and shared upper/lower coverage
        ↓
whole retain | whole reject | partial occurrence | unresolved fail-open
        ↓
source-relative endpoints, vertical bounds, UV correspondence, provenance
        ↓
ordinary view-local triangles/declarations
        ↓
prepared-full-submission to tokimu-render
        ↓ optional, later only
generic conservative AABB/frustum filtering
```

The initial realization preference is ordinary view-local triangles derived
from continuous source facts. A screen-local primitive is considered only if
the source trace proves ordinary geometry cannot preserve the required
boundary without inverse projection or raster coupling.

## Hard Invariants

- Doom source and current runtime state remain authoritative for preparation.
- The provider decides membership and multiplicity before renderer handoff.
- Diagnostic columns may verify coverage but never become mesh endpoints,
  scissors, persistent identities, or public renderer vocabulary.
- Generated endpoints and UVs come from original source parameterization, not
  screen-column inverse projection.
- Wall and floor/ceiling preparation consume the same causal prepared
  boundaries where Doom semantics couple them.
- Cutout or masked-middle contributions do not close source coverage merely
  because their supporting quad spans a range.
- Unknown, numerically ambiguous, unsupported, or uncorrelated cases fail open
  and retain bounded reasons.
- Relative order and source correlation survive occurrence generation and
  lowering.
- Static renderer resources remain reusable; camera motion must not cause
  accidental resource replacement or unbounded allocation.
- Full global submission remains the explicit correctness fallback throughout
  the study.
- No change to stable Tokimu renderer, platform, core, or runtime contracts is
  authorized by this plan.

## Slice 0 — Surgical Reference Trace

Do not begin another visual candidate in this slice.

### Deliverables

- [x] Identify the closest reproducible source arrangement corresponding to
      the retained left/middle/right falsifier in Classic Doom source terms.
- [x] Trace that specimen through `R_RenderBSPNode`, `R_AddLine`, solid/pass
      clipping, `R_StoreWallRange`, `R_RenderSegLoop`, `ceilingclip`,
      `floorclip`, and plane marking.
- [x] Repeat the semantic trace in Chocolate Doom or another deliberately
      faithful implementation and record implementation differences separately
      from preserved behavior.
- [x] Retain a stage-by-stage table comparing Doom reference behavior with the
      current Tokimu candidate:

  ```text
  source SEG
  initial projected/source range
  solid ranges before and after clipping
  retained source-relative range(s)
  upper/lower prepared bounds
  floor/ceiling marks
  masked/deferred behavior
  final surviving occurrence(s)
  ```

- [x] Answer explicitly:
  1. What exact source unit enters clipping?
  2. At which stage may it split?
  3. Can one processing unit yield multiple disjoint survivors?
  4. Which continuous/projective values exist before integer-column
     quantization?
  5. Which wall and plane boundaries derive from the same mutable state?
  6. How are masked middles deferred without gaining occlusion authority?
  7. Which facts are necessary to rebuild occurrences from source geometry and
     UVs?
  8. Which facts exist only because Doom ultimately rasterizes columns?
- [x] Record whether every individual emitted occurrence can be constrained to
      one contiguous horizontal source domain with bounded upper/lower domains.
- [x] Update the Classic Doom evidence record with citations to the inspected
      functions/files and clearly separate direct observation from Tokimu
      inference.

### Result

The [retained Slice 0 trace](../Evidence/Doom%20ordered%20source%20occurrence%20reference%20trace.md)
finds that one Classic Doom `seg_t` can cause multiple disjoint
`R_StoreWallRange` calls when accumulated solid ranges divide its projected
range. Every individual call remains one contiguous horizontal interval and
continues to reference the same source SEG. Chocolate Doom preserves that
behavior.

This directly justifies `one source contribution -> 0..N occurrences`, but it
does not justify arbitrary-region occurrences. Slice 1 is constrained to one
contiguous horizontal source-relative interval plus bounded upper/lower domains
per occurrence. Integer columns, `solidsegs`, and clip arrays remain reference
oracles rather than semantic geometry or renderer API vocabulary.

### Acceptance criteria

- The exact divergence stage is named rather than inferred from final images.
- The proposed private representation is justified by retained source evidence.
- No historical Doom storage layout or integer screen resolution is admitted
  merely because the reference implementation uses it.

## Slice 1 — Private Occurrence Model

### Deliverables

- [x] Define a Doom-private prepared occurrence record with distinct source,
      occurrence, view, snapshot, and eventual renderer-resource identities.
- [x] Represent continuous source-relative horizontal domains and bounded
      vertical domains without using diagnostic pixel columns as authority.
- [x] Represent the four bounded outcomes: whole reject, whole retain,
      `0..N` partial occurrences, and unresolved fail-open.
- [x] Retain source provenance sufficient to interpolate geometry, normals,
      wall role, UVs, material identity, and diagnostic attribution.
- [x] Represent shared wall/plane prepared boundaries once, or otherwise prove
      structurally that both consumers use identical causal values.
- [x] Add validation for empty, reversed, overlapping, non-finite, out-of-range,
      and source-identity-mismatched occurrence domains.
- [x] Keep the representation private to the Doom campaign/provider.

### Result

The [retained Slice 1 evidence](../Evidence/Doom%20private%20source-occurrence%20model%20evidence.md)
records one stable source contribution producing two disjoint correlated
occurrences. Source, occurrence, prepared-view, runtime-snapshot,
shared-boundary, and eventual renderer-resource identities remain distinct;
renderer identity is deliberately absent during preparation.

The four outcomes are structurally separated. Zero survivors require a
positively authorized whole reject, whole retain generates no geometry,
partial preparation requires one-or-more validated occurrences, and unresolved
preparation retains the original contribution with evidence. One stored
prepared boundary is shared by wall, floor, ceiling, and sky consumers.

Six focused tests and strict campaign Clippy pass. The branch still contains an
independent legacy two-sided-aperture assertion failure; it is retained as a
crate-wide closeout prerequisite rather than repaired inside this isolated
model slice.

### Acceptance criteria

- One source contribution can produce two disjoint occurrences without being
  duplicated as two unrelated source identities.
- Whole-retain avoids generated geometry where no fragmentation is required.
- Invalid or uncertain preparation cannot silently remove source work.

## Slice 2 — Headless Partial-Survival Reconstruction

### Deliverables

- [x] Re-run the retained baseline and nearer falsifiers through the private
      occurrence model with no renderer involved.
- [x] Prove the forbidden middle is absent while both required survivors remain.
- [x] Repeat under the retained camera-jitter control.
- [x] Retain conservation accounting from source contribution to whole,
      fragmented, rejected, and fail-open outcomes.
- [x] Retain continuous source intervals and compare their projected diagnostic
      coverage with the oracle columns.
- [x] Prove reconstructed endpoints lie on original source geometry and retain
      continuous UV parameterization across split occurrences.
- [x] Add negative controls for near-plane ambiguity, between-column/thin
      projection, unsupported source roles, and empty fragments; all must fail
      open or reject only with positive authority.

### Result

The [retained Slice 2 evidence](../Evidence/Doom%20headless%20partial-survival%20reconstruction%20evidence.md)
replays one stable source SEG through baseline, bounded horizontal jitter, and
nearer-camera poses. Each pose yields two stable occurrence identities over
continuous source-relative intervals. The forbidden middle is absent, every
required diagnostic survivor column is represented, and all reconstructed
endpoints remain on the original source wall with continuous UV width and
source-endpoint correspondence.

Seven evaluated outcomes balance explicitly: three fragmented pose replays,
one thin whole-retain control, one positively authorized whole reject, and two
unresolved fail-open controls. These are seven evaluations, not seven distinct
source identities; all three view replays retain one source identity and the
same two occurrence identities. Diagnostic columns are an after-the-fact
oracle only. No renderer participates and no screen-column inverse projection
constructs geometry.

Ten focused occurrence-model tests and strict campaign Clippy pass. The only
tooling noise is the existing Windows incremental-cache hard-link fallback,
which does not affect the validation result.

### Acceptance criteria

- No required interval is lost and no forbidden interval survives.
- No screen-column-to-world inverse projection is used.
- The same semantic results survive bounded jitter without identity churn.

## Slice 3 — Shared Wall/Plane Coverage

### Deliverables

- [x] Extend the ordered-coverage synthetic fixture so wall processing produces
      the exact shared boundary later consumed by plane preparation.
- [x] Re-run paired-sky, one-sky-negative, vertical-aperture,
      single-sky-plane, and shared-plane-key fixtures through the occurrence
      path.
- [x] Prove upper/lower wall fragments, openings, floor/ceiling marks, and sky
      intervals conserve the same prepared boundary.
- [x] Prove cutout non-occluders remain deferred/visible through transparent
      texels without closing source coverage.
- [x] Retain explicit crack/overlap checks at every shared seam.

### Retained result

Five source fixtures balance against the provider's single ordered transition
stream. Every retained wall interval stays inside both its authored tier and
the then-open boundary; every retained plane interval resolves to the same
source-labelled transition; plane sources were admitted by the same traversal;
and no plane instance reports overlapping writes. Paired-sky events remain
explicit, non-mutating source facts rather than gaining independent occlusion
authority.

The masked-middle control retains 480 middle wall cells while producing no
`OneSidedMiddleClosed` transition. It therefore remains a presentation
contribution without closing source coverage. All fixtures also retain a small,
bounded set of `RaySegmentDepthUnresolved` fail-open observations at exact
projection seams. Those are counted rather than silently discarded and do not
create rejection authority.

### Acceptance criteria

- Contribution accounting and boundary accounting both balance.
- Synthetic wall/plane seams have neither holes nor double-covered authority.
- Sky paints source-authorized retained intervals; it does not become a
  world-space visibility mechanism.

## Slice 4 — Ordinary Presentation Lowering

### Deliverables

- [x] Lower whole and partial occurrences into ordinary Tokimu presentation
      declarations while preserving source order and correlation.
- [x] Interpolate source endpoints and UVs directly from retained continuous
      source domains.
- [x] Keep generated occurrence geometry view-local and bounded; do not install
      it as persistent global world truth.
- [x] Retain structural hashes and allocation/upload/replacement observations
      for first, warm, and jittered frames.
- [x] Present all applicable synthetic controls on native WGPU.
- [x] Compare presentation declarations with headless semantic manifests; images
      remain observations rather than semantic authority.
- [x] Retain maintainer visual confirmation that bounded camera jitter reveals
      neither a finite preparation box nor a crack at the shared seam.

The headless lowering report proves exact `2 retained -> 2 lowered`
conservation, stable source order/correlation, complete UV streams, continuous
source-domain endpoint derivation, view-local generated geometry, and stable
fingerprint
`c707513fb367f3184bf32699a661bf4e71f078d4efa5dd0987e27d1c0e0fc94c`.
The rewritten native fixture consumes those same ordinary `Mesh` declarations.
Native first/warm/jitter frames each submitted four draws with the same
fingerprint and no diagnostic; warm and jitter frames performed zero binding
allocations, mesh uploads, or mesh replacements. Maintainer observation of the
native control confirmed that both bounded source survivors remain visible, the
excluded middle remains absent, and bounded jitter reveals neither a finite
preparation box nor a shared-seam crack. Slice 4 therefore passes.

### Acceptance criteria

- Every retained semantic occurrence has an explainable lowering destination.
- Warm-frame static resources do not churn.
- Camera jitter does not reveal a finite preparation box or crack a shared seam.

## Slice 5 — Explicit Runtime Snapshots

### Deliverables

- [x] Feed declared closed/open/opening/closing door height snapshots through
      the same preparation seam without implementing activation or timing in
      the fixture.
- [x] Feed declared low/raised platform snapshots through the same seam.
- [x] Prove preparation changes causally with current semantic heights rather
      than immutable decoded WAD heights.
- [x] Retain occurrence retirement/replacement evidence without reallocating
      unrelated source or renderer resources.

### Result

The [retained Slice 5 evidence](../Evidence/Doom%20explicit%20runtime%20snapshot%20occurrence%20evidence.md)
passes declared closed, opening, open, and closing door heights plus low and
raised platform heights through the production two-sided-wall preparation
seam. Prepared ranges follow the current snapshots exactly. Stable
source/occurrence/resource correlations yield three bounded creates, two
replacements, and one retirement with zero unrelated reallocations.

The sequence has stable fingerprint
`be0ab8105bbaff9a2976df0b67eb0ca9ad79318c6642185ad2cf9ed56de3785c`
across consecutive runs. The fixture contains no activation, timing, waiting,
or reversal policy. Lifecycle labels describe campaign-local reconciliation;
they do not claim an admitted renderer retirement or allocation API.

### Acceptance criteria

- Application-owned movement policy remains outside preparation.
- Snapshot changes update exactly the affected prepared occurrences and
  boundaries.

## Slice 6 — E1M1 Prepared Full Submission

This is the first E1M1 visual candidate. It may begin only after Slices 0–5
pass.

### Deliverables

- [x] Implement the bounded Doom-private reference planner described by the
      released-renderer dataflow analysis before attempting another optimized
      E1M1 candidate. Retain ordered solid/pass ranges, vertical clip mutations,
      wall tiers, plane marks/instances, sky intervals, and deferred masked
      work in one deterministic manifest.
- [x] Run paired-sky, one-sky-negative, vertical-aperture, shared-plane-key,
      door/platform snapshot, projection-epsilon, and cutout-non-occluder
      controls through that one planner before E1M1 escalation.

The [retained synthetic-gate evidence](../Evidence/Doom%20ordered%20reference%20planner%20synthetic%20gate%20evidence.md)
passes all 14 controlled states through one deterministic planner manifest.
The gate composes ordered BSP admission, vertical coverage mutations, wall
tiers, plane marks/instances, sky intervals, deferred masked-middle work, and
fail-open observations. All cases balance; paired-sky and one-sky authority
remain distinct; aperture wall/plane facts coexist; shared plane keys retain
multiple instances; door/platform snapshots change planner evidence without
movement policy; projection-near ambiguity fails open; and cutout work does
not close source coverage. This authorizes E1M1 escalation but does not itself
prove the current E1M1 prepared-full lowering is coherent.

- [x] Add a clearly named `ordered-occurrence-prepared-full` strategy.
- [ ] Prepare walls, planes, sky, cutouts, doors, and platforms through one
      coherent ordered observation and lower every retained occurrence.
  - [x] Audit the normalized unsigned vertical clip representation against
        released Doom `R_ClearPlanes` / `R_RenderSegLoop`, repair the three
        confirmed inclusive/exclusive row errors, and retain a focused
        source-parity regression before interpreting further E1M1 cracks.
  - [x] Run the repaired ordered source protocol headlessly against canonical
        E1M1 and retain the pre-lowering wall/clip/plane ledger. At source
        spawn it resolves all 9 plane instances with zero overlapping writes
        and zero unresolved instances; the remaining loss is downstream
        representation pressure, not an unobserved source contribution.
  - [x] Correlate retained source occurrences to SEG-granular wall triangles.
  - [x] Preserve opaque versus source-classified masked-middle intent during
        clipping and ordinary mesh lowering.
  - [x] Prove category-specific material identity for every matched source
        triangle before replacing any global wall declaration.
  - [x] Replace global wall declarations only after per-category replacement
        conservation and upload identity are explicit in the prepared list.
  - [x] Correlate retained occurrences with Doom-owned floor/ceiling marks,
        plane identity, sky identity, and paired-sky facts without consuming
        the legacy 320-column reconstruction.
  - [x] Derive one continuous shared vertical boundary for every retained
        occurrence and prove wall/plane consumer conservation.
  - [x] Group retained plane associations by exact source plane identity and
        correlate every instance/subsector reference with its exact source
        surface destination without claiming the whole region is visible.
  - [x] Lower retained floor, ceiling, and sky contributions from those shared
        boundaries into ordinary Tokimu declarations.
    - [x] Lower every exact ordinary source-region destination, retain bounded
          degenerate omissions, and account source sky as background-only.
    - [x] Replace conservative whole source-region plane destinations with
          Doom-owned occurrence-bounded plane survival; whole-region visibility
          remains explicitly unclaimed.
  - [x] Correlate door and platform runtime snapshots to those same prepared
        boundaries.
- [ ] Submit all prepared declarations to `tokimu-render` with no generic
      camera filter.
  - [x] Submit the first balanced fixed-view wall plus ordinary-plane set with
        full submission, while retaining the original global scene solely as
        an independent control.
- [ ] Compare against `global-full-submission` at spawn, hut/window, exterior
      hut, first door, moving platform, green-room cutout, and EXIT poses.
  - [x] Compare the canonical source-spawn pose and retain the decisive visible
        false-negative result.
  - [ ] Remaining poses are parked because the first required pose already
        falsified this plane-survival realization.
- [ ] Retain contribution and boundary conservation manifests at each pose.
- [ ] Verify close-wall motion, free look, camera jitter, and near-view movement
      without disappearing walls, cracks, a finite view box, or stale
      preparation.
- [ ] Verify source-invalid far geometry no longer participates through sky
      while nearby authorized geometry such as the hut survives.
- [ ] Retain bounded failures and fail-open reasons; do not hide an uncertain
      contribution to improve a screenshot.

### Acceptance criteria

- The candidate has no visible false negative in the canonical matrix.
- Every difference from global full submission has positive Doom-owned
  preparation evidence.
- Dynamic state and sky behavior use the same occurrence/boundary model as the
  synthetic fixtures.

### Integration baseline

The strategy name is executable and deliberately distinct from both the
historical screen-column reconstruction and the predecessor topology-admission
study. The original 1,922-contribution scene remains separately available as
the global control; it is not mixed into the ordered candidate.

The predecessor fixed-view candidate replaced that global declaration domain
with 580 opaque draws (309 wall plus 271 conservative whole-region plane draws)
and 12 cutout wall draws. That baseline proved the handoff and resource
conservation, but deliberately did not claim that whole plane regions were
viewer-visible.

An integration fault initially allowed the legacy 320-column runtime refresh to
overwrite the candidate with 51 historical reconstructed draws before the first
frame. The candidate now has no runtime source for that superseded refresh; the
corrected first and warm frames each submitted all 592 predecessor declarations
with zero candidate rejection and zero warm resource churn.

The current bounded-plane candidate clips each exact ordinary source-region
triangle to the disjoint horizontal view intervals carried by the occurrences
that reach that destination. At the same fixed source-spawn pose it reports:

```text
ordinary plane source triangles                 283
  with one or more bounded survivors             72
  fully rejected                                 211
clipped plane triangles                          166
  lowered ordinary plane meshes                  136
  bounded degenerate omissions                    30
sky destination references                         9
sky source triangles                              21
unresolved lowering failures                       0

prepared declarations
  opaque                 445 = 309 wall + 136 plane
  cutout                                            12
  total                                            457
```

Destination, source-triangle, clipped-fragment, and declaration conservation
all balance. The renderer submits the entire 457-declaration prepared list;
there is no generic camera filter and no legacy screen-column reconstruction.
This closes the structural occurrence-bounded lowering item, not the visual
acceptance gate: horizontal occurrence domains may still be insufficient to
describe Doom plane survival, and any canonical missing floor or ceiling is a
false-negative finding rather than permission to patch the result.

### Canonical visual falsification

The required source-spawn comparison falsified the bounded-plane candidate.
Although all structural accounting balanced and the renderer submitted the
entire 457-declaration prepared list, the presented frame omitted multiple
independent required contributions:

- large wall/plane regions at the left and right edges;
- narrow openings around pillar and stair junctions;
- floor/step-edge regions around the central pool; and
- several regions where the sky/background became visible through the room.

The failures are distributed rather than one isolated crack. They demonstrate
that correlating exact plane destinations with merged camera-horizontal SEG
occurrence wedges is not sufficient authority for Doom floor, ceiling, and
wall/plane survival. Balanced bookkeeping proves that the implementation
faithfully lowered its own candidate; it does not make the candidate
source-faithful.

No compensating geometry, generic AABB filter, enlarged epsilon, or screenshot
exception was added. Slice 7 is therefore blocked: a generic conservative
post-filter cannot restore declarations already removed by Doom preparation.
The next step is an AR-0030 decision about the next Doom-private ordered
representation, with authoritative vertical wall/plane coverage as the leading
pressure—not a renderer-owned Doom rule.

### Six-ray reconciliation

The later relational study replayed six retained exterior defects through the
same ordered source protocol. Five suspect global-shell contributions were
already terminally rejected; the sixth ceiling survived only in two narrow
plane-instance view intervals. This changes the leading implementation
question. The ordered preparation is not missing five visibility decisions;
the global-shell realization is discarding them by submitting the original
geometry again.

The relational composer remains valid synthetic evidence but is not used to
reconstruct these decisions. It also exposed that a retained plane occurrence
and a wall SEG authority do not naturally share one source parameterization.
That mismatch is treated as focused representation pressure rather than a
reason to generalize the composer.

## Slice 6B — Literal Ordered-Result Realization

This slice supersedes attempts to repair the global shell with sky depth,
whole-object filtering or downstream relational reclassification. It consumes
one coherent ordered result exactly once.

### Deliverables

- [x] Define a Doom-private exhaustive disposition for each prepared source
      contribution: `whole-retained`, `terminal-rejected`, `partial-seg`,
      `partial-plane` or `unresolved-fail-open`.
- [x] Prove every original contribution reaches exactly one disposition and
      retain per-family conservation for walls, planes, sky and cutouts.
- [x] Reuse existing ordinary geometry only for `whole-retained` contributions.
- [x] Emit no declaration for `terminal-rejected` contributions, and prove the
      five retained bad-ray identities cannot re-enter through a global-shell
      fallback.
- [x] Lower `partial-seg` contributions with the existing source-relative wall
      representation, preserving UV progress, sidedef role and provenance.
- [x] Isolate `partial-plane` contributions behind one Doom-private
      representation experiment. Do not convert plane-instance destination
      intervals into SEG source progress or nearest-hit depth.
- [x] Fail open explicitly for unresolved contributions without silently
      restoring every terminally rejected global-shell contribution.
- [x] Submit every resulting declaration with prepared full submission and no
      AABB/frustum filtering.
- [ ] Re-run the six deterministic rays, source spawn, hut/window, exterior
      hut, first door, moving platform, green-room cutout and EXIT controls.
- [ ] Exercise free look, near-wall movement and camera jitter through the
      native live-refresh seam without a fixed
      view box, disappearing walls or stale prepared state.
- [ ] Retain native and browser structural evidence before any generic
      post-filter experiment.

### Integrated gate finding

The native integration now exposes one Rust-owned, Doom-private preparation
entry point. The ordered-occurrence strategy uses it for every presentation
frame with the current source-camera pose and projected runtime-height
snapshot, replaces the previous declaration vectors only after successful
preparation at the corpus composition edge, resets the matching selection
state, and uploads only the new ordinary declarations. It no longer sets
`fixed_reconstruction_camera=true`. The same entry point is suitable for a
browser host, but that host has not yet supplied E1M1 source/runtime inputs.

This establishes the native lifecycle seam, not its visual closeout. The
headless six-ray replay remains balanced. The same snapshot correlation used
by the door/platform controls now flows into the live refresh, so a stationary
camera cannot retain stale prepared geometry while a sector moves. Free-look,
near-wall movement, camera jitter and safe visual retirement still need
interactive evidence. None of that work may become renderer-owned Doom
visibility or persistent asset identity.

The `--ordered-occurrence-live-refresh-report` control now replays spawn,
bounded yaw changes, a declared forward displacement and return-to-spawn
through the shared entry point. Each pose installs only after complete
conserved preparation; the retained canonical run reports opaque/cutout counts
of `445/12`, `447/12`, `459/12`, `472/12` and `445/12` respectively. This is
composition-local structural evidence, not a substitute for visual free-look
or jitter validation.

### Source-spawn visual falsification

Maintainer inspection of the live `ordered-occurrence-prepared-full` strategy
at source spawn found severe sky-background leakage through large edge regions
and several bounded interior wall/floor seams. The window reported `458 draws`,
which is consistent with the complete `445` opaque plus `12` cutout prepared
declarations and the sky pass. The observation therefore falsifies visual
completeness of the conserved declaration set; it is not evidence of a partial
composition swap or stale refresh.

Treat the marked regions as missing prepared foreground coverage. Do not add a
depth-bearing sky occluder or generic filter to hide them. Before continuing
browser parity, distinguish source-disposition omission from partial-wall or
partial-plane realization, triangle facing/culling, and camera/preparation
projection mismatch. Preserve the source-spawn screenshot as maintainer visual
evidence when an artifact path is available.

Browser parity is also deliberately open. The browser E1M1 consumer currently
has a separate preparation implementation and cannot exercise this native
binary-private seam without either duplicating it or extracting a shared
Doom-private preparation unit. A synthetic browser fixture is not evidence of
the final E1M1 handoff.

### Acceptance criteria

- Every terminal ordered decision survives through renderer submission without
  reopening or contradictory fallback.
- Whole-retained geometry remains complete, including floors and ceilings near
  shared wall boundaries.
- Partial SEG and plane occurrences conserve their own source identities and
  domains without being forced through a common parameterization.
- The five source-rejected six-ray contributions are absent, while the one
  partially retained ceiling is represented only in its authorized domain.
- No correctness result depends on AABB/frustum filtering, renderer-owned Doom
  semantics, screenshot exceptions or a new stable renderer contract.

### Authorized partial-plane occurrence refinement

AR-0030 now treats the complete ordered result as the authoritative live Doom
presentation input. The private semantic vocabulary is a prepared
presentation occurrence: one source contribution may produce zero, one, or
several bounded view/runtime-conditioned occurrences. Absence is authoritative
and cannot be reopened by the global shell.

The first focused partial-plane refinement does not add a renderer primitive.
It observes Doom's exact ordered vertical plane cells, correlates them by plane
kind, sector, subsector, height, texture, light and owning SEG, then intersects
those cells with the inferred source-region triangles. Whole plane triangles
still reuse ordinary geometry; terminally rejected triangles emit nothing.
Only partial plane triangles use the bounded cell intersection.

At the retained ceiling-104 ray, the source protocol supplies `13` exact cells
owned by SEGs `310/311`, both mapping to subsector `104`. They lower to one
ordinary combined declaration rather than the former eight horizontal-wedge
fragments. At source spawn the complete preparation now reports `309` opaque
wall, `12` cutout, `28` floor and `15` ceiling declarations (`352/12` renderer
inputs), while conserving `3,432` lowered plane triangles inside `43` combined
plane meshes. The five terminal six-ray sources remain absent.

Stationary live frames now retain an explicit view/runtime preparation
identity and skip identical rebuild/upload work. A changed camera pose or
door/platform snapshot still prepares a complete conserved result before the
identity and declaration set are replaced.

Open acceptance remains visual: hut, far-left structure, both leak sites,
peripheral coverage, pitch/free-look continuity and absence of a finite view
box must be inspected in the launched native path. Browser consumption of the
same shared Rust unit also remains open.

#### Pitch falsifier and pause gate

A direct remap of the retained Classic Doom plane rows into a pitched Tokimu
camera was tested and rejected. Adding camera pitch to the row-to-world inverse
projection caused the retained partial ceiling at subsector `104` to produce
zero fragments instead of its proven single declaration. The source occurrence
itself remained partial, so this was a representation failure rather than new
source rejection evidence.

The retained rows describe the unpitched source projection that generated
them. They do not gain authority over pitched-camera plane coverage merely
because both use screen coordinates. The implementation therefore returns to
the last conserved unpitched lowering and pauses before choosing among a
pitch-aware source protocol, a conservative provider-local extension, or a new
presentation representation. That choice changes semantic authority and is an
AR-0030 decision, not an ordinary lowering repair.

#### Native spawn-room falsifier

The first native walkabout observation of the focused plane-cell handoff also
failed without relying on pitch. After live refresh the window reported `365`
draws, but large opaque foreground regions partitioned the spawn view and made
roughly half the room disappear. Consequently, balanced source, fragment and
declaration ledgers are necessary but not sufficient acceptance evidence.

The Cycle 31 lowering is retained as diagnostic implementation evidence, not
as an accepted live presentation path. Work remains paused before adding a
local exception or reopening the global shell; the representation question
must return through AR-0030.

#### Hardware-port precedent refinement

Primary-source review confirms that established hardware paths do not retain
Classic visplane rows as arbitrary-pitch geometry. GZDoom prepares world-space
subsector/section plane surfaces, render-sector associations and source-specific
hack/portal relationships, then combines BSP traversal with horizontal and
pitch-aware clipping. Doom iOS/PrBoom-style GL uses the coarser alternative of
triangulated whole-sector planes admitted when an uncovered subsector is
reached, and its source documents the corresponding over-admission risk.

The next representation question is therefore narrower than a custom Doom
renderer: whether one Doom-private render-subsector unit can preserve the E1M1
zero/partial/whole evidence under the actual Tokimu camera while lowering only
ordinary declarations. This remains an AR-0030 decision; no implementation or
stable contract is admitted by the precedent study.

Evidence:
`../Evidence/Hardware Doom arbitrary-pitch plane preparation precedent.md`.

### Released-source clip parity and E1M1 ledger

The production provider's unsigned clip representation has now been audited
against pinned released Doom source and repaired at three inclusive/exclusive
translation points: the first ceiling-plane row, the no-upper ceiling
transition, and the last open row below `floorclip`. A focused regression fixes
the signed-to-unsigned correspondence.

The repaired provider then ran headlessly against E1M1 before either candidate
lowering. At source spawn it retains 37 admitted SEGs, 38 wall-tier
contributions, 9 resolved plane instances, 17 horizontal spans, and 1,205
populated columns with zero overlapping writes and zero unresolved plane
instances. The detailed ledger is retained in
[Doom E1M1 Ordered Source Protocol Ledger Evidence](../Evidence/Doom%20E1M1%20ordered%20source%20protocol%20ledger%20evidence.md).

This narrows the remaining failure. The old fixed-view reconstruction preserves
the ledger as 1,205 inverse-projected quads, while the continuous occurrence
candidate loses required geometry despite balanced internal accounting. The
next step is therefore a representation decision under AR-0030, not another
source-observation patch or an early generic post-filter.

### E1M1 runtime snapshot correlation

The same coherent preparation seam now consumes immutable current-height
snapshots for the first manual door and the tag-2 moving platform. Deterministic
source-boundary-local diagnostic views demonstrate both forms of pressure:

```text
door sector 4 ceiling 0 -> 68
    source occurrences       242 -> 250
    target declarations        4 -> 12

platform sector 70 floor 104 -> -48
    source occurrences       319 -> 319
    target declarations       19 -> 21
```

Both comparisons retain zero downstream lowering failures. Their explicit
near-plane fail-open observations remain bounded and stable across each
baseline/snapshot pair. The door proves current heights can alter ordered
admission itself; the platform proves a stable occurrence set can still lower
different target geometry. Activation, timing, waiting, reversal, input, and
renderer lifecycle policy are absent. The retained evidence is
[Doom E1M1 Ordered Runtime Snapshot Correlation Evidence](../Evidence/Doom%20E1M1%20ordered%20runtime%20snapshot%20correlation%20evidence.md).

### Continuous E1M1 source observation

The first positive E1M1 observation now walks the source BSP near-first and
computes horizontal source-relative occurrences without consuming the legacy
320-column diagnostic reconstruction. At the fixed source-spawn pose it visits
732 source SEGs and reports 171 retained occurrences: 16 whole, 16 partial,
and 137 explicit near-plane fail-open occurrences; 563 contributions are
wholly rejected. The occurrence fingerprint is `69cbc0a1e53db469`.

This observation does not yet replace, remove, or relabel any of the 1,922
original render contributions. Those declarations remain unchanged and
explicitly unresolved/fail-open. Therefore the result establishes positive
Doom-owned occurrence evidence and the integration dataflow, but it is not yet
the prepared-full visual candidate or a conservation proof for walls, planes,
sky, cutouts, doors, and platforms.

### Ordered wall lowering gate

The next real-map gate correlates those 171 occurrences to the existing
SEG-granular textured-wall provider output. Of the occurrences, 135 have wall
geometry and 36 have no visible wall tier. The correlation matches 303 source
triangles, clips them into 331 source-domain fragments, and lowers 321 ordinary
meshes after retaining 10 narrow degenerate omissions. All 303 matched source
triangles resolve through the appropriate existing opaque or cutout material
inventory. It reports zero unresolved failures and fingerprint
`0bcca0e595848b93`.

The existing source-owned material split also survives the path:

```text
matched source triangles    opaque=291  cutout=12
material-resolved triangles opaque=291  cutout=12
clipped triangles           opaque=319  cutout=12
lowered meshes              opaque=309  cutout=12
category conservation       balanced
material conservation       balanced
```

Masked middles are recognized from exact authored Doom identity, not inferred
from alpha bytes. Material lookup is likewise category-specific and fails open
rather than borrowing from the other policy. The original E1M1 declarations
still remain unchanged, so this is a positive wall-destination,
classification, and material-identity gate rather than a prepared-full
presentation claim. The retained evidence is
[Doom E1M1 Ordered Wall Occurrence Lowering Evidence](../Evidence/Doom%20E1M1%20ordered%20wall%20occurrence%20lowering%20evidence.md).

### Ordered plane-association gate

The same 171 retained continuous occurrences now resolve against the
Doom-owned plane marks emitted before legacy plane-span reconstruction. The
fixed source-spawn observation reports:

```text
occurrences                         171
with marked planes                  157
without marked planes                14
plane associations                  259
  floor                             140
  ceiling                           119
  sky ceiling                        12
paired-sky adjustments                8
distinct floor planes                44
distinct ceiling planes              36
distinct sky-ceiling planes            6
unresolved fail-open                   0
```

Occurrence and association accounting both balance. Plane identity includes
source sector, plane kind, current source height, texture, and light level;
sky remains a ceiling-plane classification, and paired-sky remains a
non-authoritative source fact.

### Continuous shared-boundary gate

Every retained occurrence now owns exactly one Doom-private continuous
vertical boundary derived from current sector relationships:

```text
boundaries                          171
  one-sided                        101
  open two-sided                    63
  closed two-sided                   7
wall consumer references           321
plane consumer references          259
unresolved fail-open                  0
boundary conservation         balanced
consumer conservation         balanced
continuous vertical coverage ready true
legacy screen columns used         false
```

For a two-sided boundary, the opening is the intersection of the two current
sector intervals: `max(floor)` through `min(ceiling)`. One-sided and closed
two-sided boundaries remain solid. Paired-sky classification is retained as
metadata and does not mutate that opening. Reversed or missing source heights
fail open rather than fabricating coverage.

All 321 lowered wall declarations and all 259 plane associations resolve to
these same boundaries. This closes the causal-boundary prerequisite without
claiming plane presentation: no plane mesh is emitted, no renderer contract is
changed, and the original E1M1 plane declarations remain unchanged. The next
gate is source-owned plane-instance preparation and lowering from this shared
boundary model, followed by explicit runtime-height snapshot correlation.

### Exact plane-instance destination gate

The 259 retained associations group into 80 exact source plane instances and
146 distinct instance/subsector references. Every reference resolves to an
exact source-region geometry destination containing 304 source triangles in
total; no destination is unresolved. Instance and destination conservation
both balance, and equal numerical plane values in different source sectors
remain distinct.

This is deliberately a destination proof rather than a visibility claim. It
establishes where every retained plane fact can lower while leaving whole-region
versus partial-region survival to the next Doom-owned preparation step.

## Slice 7 — Optional Generic Conservative Post-Filter

This slice is downstream optimization evidence, never a repair for source
preparation.

### Deliverables

- [ ] Feed the complete Slice 6 prepared declaration list to the existing
      conservative AABB/frustum selector.
- [ ] Preserve occurrence order, source provenance, and all fail-open work.
- [ ] Attribute every removal to the generic stage separately from Doom
      preparation.
- [ ] Repeat the canonical visual and structural matrix under camera motion and
      dynamic snapshot changes.
- [ ] Retain CPU cost, draw reduction, resource churn, and false-positive/
      false-negative observations.

### Acceptance criteria

- The generic stage removes only work already safe to remove conservatively.
- Disabling it restores exactly the Slice 6 prepared-full result.
- Any correctness difference parks the generic stage; it does not trigger a
  workaround in Doom preparation.

## Slice 8 — Browser/WASM Parity

### Deliverables

- [ ] Run all synthetic semantic manifests in DOM/WASM.
- [ ] Present the applicable occurrence fixtures through Browser WebGPU.
- [ ] Run the E1M1 prepared-full candidate at retained canonical poses.
- [ ] Retain equivalent source/occurrence outcome counts, reason categories,
      ordering, snapshot identity, and structural hashes.
- [ ] Record backend, adapter, build, viewport, and timing metadata without
      claiming native/browser pixel identity.
- [ ] Confirm first/warm/jitter frames do not introduce unexpected uploads,
      replacements, or occurrence identity churn.

## Slice 9 — Decision And Handoff

### Deliverables

- [ ] Produce a matrix comparing global full submission, ordered-occurrence
      prepared full submission, and the optional generic post-filter.
- [ ] State whether ordinary view-local triangles are sufficient or whether a
      bounded screen-local primitive remains independently justified.
- [ ] State the measured frequency and cost of whole-retain, whole-reject,
      fragmentation, and fail-open outcomes.
- [ ] Update AR-0030 with Doom's surviving provider-local dataflow and clearly
      distinguish it from any proposed shared Tokimu render handoff.
- [ ] Update the DOOM WAD Checklist and campaign index with the accepted,
      parked, or falsified next action.
- [ ] Retain or remove experimental paths only after their evidence and
      invocations are reproducible.

## Stop And Escalate Conditions

Return to the maintainer and AR-0030 before continuing if:

- correct occurrences require a new stable/public Tokimu renderer primitive;
- source-relative ordinary triangles cannot preserve a demonstrated required
  interval without raster-resolution authority;
- the reference trace requires copying implementation-specific Doom storage
  rather than preserving an independently testable invariant;
- wall and plane semantics cannot share prepared boundaries without changing
  engine ownership;
- correct preparation requires the renderer to invoke Doom/provider code;
- generated view-local geometry causes material performance, memory, or
  resource-lifetime pressure;
- native and browser cannot express equivalent semantic occurrences; or
- the experiment would broaden from Doom evidence into a generic framework
  before Quake/non-BSP pressure exists.

## Validation Expectations

Use focused campaign validation during implementation, followed by workspace
gates before a closeout claim:

```text
cargo fmt --all -- --check
cargo test -p hello-doom-visibility-conformance
cargo test -p hello-doom-e1m1
cargo clippy -p hello-doom-visibility-conformance --all-targets -- -D warnings
cargo clippy -p hello-doom-e1m1 --all-targets -- -D warnings
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Browser compilation is not browser execution. Retain actual DOM/WASM and
Browser WebGPU observations where required by the slices.

## Completion Criteria

This plan completes only when it reaches one of these honest dispositions:

1. **Sufficient Doom-local preparation** — source-informed `0..N` occurrences
   pass the synthetic, E1M1, native, and browser gates; or
2. **Representation falsified** — retained evidence demonstrates why
   source-relative ordinary occurrences cannot preserve required Doom
   presentation; or
3. **Architectural escalation** — a stable renderer/framework decision is
   required and the evidence is returned to AR-0030 without silently admitting
   it locally.

Completion does not admit a provider-neutral preparation trait, Doom visibility
algorithm, screen-column primitive, renderer-owned scene graph, or stable
submission framework.

## Related Records

- [AR-0023 — Textured Surface Alpha And Depth Policy](../../../Architectural%20Reviews/AR-0023-textured-surface-alpha-and-depth-policy.md)
- [AR-0025 — Camera Candidate Selection And Visibility Culling](../../../Architectural%20Reviews/AR-0025-camera-candidate-selection-and-visibility-culling.md)
- [AR-0030 — Source-Owned Presentation Preparation Boundary](../../../Architectural%20Reviews/AR-0030-source-owned-presentation-preparation-boundary.md)
- [Doom Viewer-Relative Presentation Synthetic Conformance](Doom%20viewer-relative%20presentation%20synthetic%20conformance.md)
- [Doom Source-Topology Admission Over Complete Geometry](Doom%20source-topology%20admission%20over%20complete%20geometry.md)
- [Classic Doom Visibility Clipping Evidence](../Evidence/Classic%20Doom%20visibility%20clipping%20evidence.md)
- [Read Reference Source Early](../../../lessions/read-reference-source-early.md)
