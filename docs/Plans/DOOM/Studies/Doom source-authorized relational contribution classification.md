# Doom Source-Authorized Relational Contribution Classification

| Field | Value |
| --- | --- |
| Campaign | DOOM |
| Role | Focused pre-render contribution-classification study |
| Status | Parked after successful synthetic study and decisive E1M1 negative-admission result |
| Parent study | [Authoritative Sky-Coverage Delta Realization](Doom%20authoritative%20sky%20coverage%20delta%20realization.md) |
| Parent review | [AR-0030](../../../Architectural%20Reviews/AR-0030-source-owned-presentation-preparation-boundary.md) |
| Source oracle | [Ordered Source-Occurrence Preparation](Doom%20ordered%20source%20occurrence%20preparation.md) |
| Capture ledger | [Relational Classifier Four-Case Capture Ledger](../Evidence/Doom%20relational%20classifier%20four-case%20capture%20ledger.md) |
| Renderer authority | None; surviving ordinary declarations only |
| Generic post-filter | Disabled until this candidate is independently correct |

## Purpose

Test whether Doom's ordered source authority can classify competing source
contributions *relationally* before renderer submission, without turning that
authority into a free-standing depth surface.

Candidate 1 represented authoritative sky coverage as additional view-local
depth geometry over an unchanged global shell. Its continuous triangles match
the extracted 320-column ledger at every sampled column center, yet the E1M1
exterior observation simultaneously shows valid geometry being clipped and
invalid distant geometry leaking through. That falsifies the composition
hypothesis, not the triangle approximation.

This study tests a different hypothesis:

```text
Doom source + runtime snapshot + prepared view
        -> ordered source-authority occurrences

candidate source contribution
        x its own source-authorized support occurrence
        x finite authorizing occurrence
        x shared bounded view interval
        -> outside support / ineligible
        -> keep
        -> reject
        -> partial survivor(s)
        -> unresolved / fail open

surviving ordinary contributions/fragments
        -> optional later generic conservative filter
        -> tokimu-render
```

The candidate's own source occurrence first limits where that contribution is
eligible to exist. Only then does the authorizing occurrence supply relational
depth evidence about the eligible portion. The authorizing occurrence does not
become a new occluding object in the scene.

## Question Under Test

After restricting a candidate to its own demonstrated source-occurrence support,
can the supported portion be classified within the finite horizontal and
vertical domain where Doom's ordered protocol grants source authority by its
depth relationship to the authorizing source boundary?

The expected relation at a shared camera ray or bounded interval is:

```text
candidate before authority
    -> nearer / allowed

candidate beyond authority
    -> reject that bounded portion

candidate crosses authority
    -> retain one or more partial survivors

relation unavailable or ambiguous
    -> fail open and retain bounded evidence
```

This is deliberately a two-stage decision:

```text
candidate geometry
    x candidate source-occurrence support
        -> supported portion
        -> outside-support portion
        -> unresolved support / fail open

supported portion
    x finite authorizing occurrence
    x prepared view
        -> nearer | beyond | straddling | unresolved
```

`OutsideSourceSupport` is not a depth classification. It records that global
geometry exceeded the source occurrence from which presentation eligibility
can be demonstrated. If support itself cannot be resolved, the candidate fails
open; the study must not turn missing source evidence into permission to delete
geometry.

The candidate's own normal is diagnostic input only. A normal can classify
facing, but it cannot prove occlusion. No contribution may be rejected merely
because its own normal faces away from the camera during this study.

## Ownership Boundary

| Concern | Owner during this study |
| --- | --- |
| SEG, sidedef, sector, `F_SKY1` and source-occurrence meaning | Doom provider/corpus |
| Which occurrence grants authority and over what bounded domain | Doom ordered preparation |
| Candidate's own eligible source-occurrence/support domain | Doom provider/corpus |
| Candidate-to-authority depth comparison | Doom-private classifier |
| Splitting and source/UV provenance of partial survivors | Doom-private lowering |
| Ordinary draw, material, depth and cutout realization | Existing Tokimu renderer contract |
| Optional AABB/frustum rejection after correctness | Later generic experiment |
| Stable render/preparation vocabulary | AR-0030 plus independent campaign evidence |

`tokimu-render` must not receive sectors, SEGs, sky names, screen columns,
source-boundary planes or Doom classification outcomes. It receives only the
ordinary surviving declarations produced upstream.

## Hard Prohibitions

1. Do not use an authorizing SEG's infinite supporting plane as global
   authority. The finite source occurrence, horizontal interval and vertical
   interval must all apply.
2. Do not infer source authority from a texture name, normal, alpha value,
   material, proximity or screenshot.
3. Do not reject a whole mesh when only a bounded portion is classified.
4. Do not silently drop unresolved or numerically unstable relations. They
   fail open with source-correlated evidence.
5. Do not tune Candidate 1's depth triangles under this plan. Candidate 1 is a
   negative composition control.
6. Do not use AABB/frustum selection to repair classification correctness.
7. Do not broaden a stable renderer API under this plan.
8. Do not let geometric nearness make geometry eligible outside the
   candidate's own demonstrated source-occurrence support.
9. Do not treat sector identity as one globally valid floor or ceiling
   occurrence. Plane support is subsector/source-occurrence-local unless the
   source protocol proves a broader shared occurrence.

## Controls And Candidate Names

- `global-full-submission` -- unchanged complete E1M1 geometry and ordinary
  depth; visual correctness control with known distant-sector sky leaks.
- `global-full-plus-view-local-sky-depth` -- falsified Candidate 1 composition
  control.
- `source-authority-relational-full` -- all ordinary contributions surviving
  the Doom-private relational classifier, submitted without a generic
  post-filter.
- `source-authority-relational-frustum` -- later optional control applying a
  conservative generic filter to the already-correct relational result.

## Observed E1M1 Lazy-Map Pressure

The oversized-support problem is present in canonical E1M1, not merely a
defensive synthetic possibility. Under unchanged global-full submission, the
fixed exterior view exposes complete floors, ceilings and rooms beyond the
bounded source occurrences the original ordered presentation reaches for that
view. Elevated inspection makes the overbroad global shell especially clear:
adjacent and distant rooms coexist as ordinary world geometry even though the
source protocol does not authorize them as one simultaneous presentation.

This evidence changes the experiment's burden:

```text
complete E1M1 world contribution
    != source-authorized presentation occurrence

geometrically nearer
    != eligible for presentation
```

The synthetic lazy-mapper controls therefore reproduce observed corpus
pressure. They are not speculative robustness cases. A successful classifier
must restrict candidate support before relational depth, while retaining
source identity and failing open when that support cannot be established.

## Canonical Four-Case Falsifier

The first gate is intentionally smaller than another complete E1M1 renderer
walkthrough. Capture exact source identities and compare both the candidate's
own facing and its relation to the finite authorizing occurrence.

| Case | Required classification |
| --- | --- |
| Far-left valid building, including the diagonally clipped portion | Nearer/allowed |
| Valid outside wall and hut-adjacent structure currently masked by Candidate 1 | Nearer/allowed |
| Far-room geometry leaking beside the hut | Beyond authority/rejected in the bounded domain |
| Far-room geometry leaking above the wall | Beyond authority/rejected in the bounded domain |

For every case retain:

```text
camera/view identity
candidate source identity and presentation role
candidate source-occurrence/support identity
supported, outside-support and unresolved-support source ranges
candidate normal/facing result (diagnostic only)
authorizing occurrence identity
finite source parameter range
authorized horizontal and vertical view interval
comparison domain and depth parameterization
candidate depth range
authority depth range
nearer | beyond | straddling | unresolved
classification reason
survivor source ranges, if split
```

Depth values are comparable only when they use the same declared domain. The
initial study uses the prepared-view source ray parameter `t`; projected clip
depth, world distance, source-line parameter and screen-column position may be
retained as diagnostics, but must not be compared as though they were the same
quantity. Every observation therefore records:

```text
comparison_domain=prepared-view-source-ray-t
candidate_t=...
authority_t=...
finite_source_parameter=...
authorized_horizontal_interval=...
authorized_vertical_interval=...
```

Column centers and column edges are also distinct samples. A result derived at
one must not be silently reused at the other.

## Slice 0 — Freeze Evidence And Terminology

- [x] Separate object-facing classification from visibility/occlusion.
- [x] State that the useful relation belongs to the authorizing source
      occurrence, not the candidate's own normal.
- [x] State that candidate eligibility belongs to its own source-occurrence
      support and precedes relational depth classification.
- [x] Preserve Candidate 1 as a negative composition control rather than a
      repair target.
- [x] Name the four canonical E1M1 observations and their required outcomes.
- [x] Freeze the reviewed package fingerprint, fixed exterior view pose and
      immutable runtime-height snapshot shared by the first four-case capture.
- [ ] Capture replayable look-ray/source reports for all four observations.
  - [x] Mirror interactive `LOOK` commands and observations to the invoking
        terminal so exact tokens survive window disposal and overlay wrapping.
  - [x] Headlessly reproduce the named beside-hut ray and five additional
        exact rays with matching candidate, boundary and source-trace facts.
  - [ ] Attribute the five-ray terminal set to the named visual cases, or
        capture replacement rays while recording each case label.
- [x] Record the exact package fingerprint, view pose and runtime snapshot used
      by the four-case matrix in the retained evidence ledger.

### Slice 0 acceptance

- [ ] Every visual observation has an exact candidate source identity and a
      reproducible view ray or bounded interval.

## Slice 1 — Headless Finite-Authority Relation

- [x] Introduce a corpus-private observation model for `Nearer`, `Beyond`,
      `Straddling` and `Unresolved`; do not publish it from Tokimu crates.
- [x] Introduce a separate corpus-private support observation for `Supported`,
      `OutsideSourceSupport` and `UnresolvedSupport`; do not disguise support
      eligibility as a depth result.
- [x] Resolve wall support from finite SEG/source-relative intervals and plane
      support from subsector/source-occurrence-local contributions rather than
      from whole linedef or sector-wide world meshes.
- [x] Restrict each candidate to its own support before consulting an
      authorizing boundary; retain outside-support ranges explicitly.
- [x] Resolve the authorizing source occurrence from the ordered ledger rather
      than from proximity or material inspection.
- [x] Require overlap with the occurrence's finite source parameter range,
      authorized horizontal interval and authorized vertical interval.
- [x] Compare source-ray depth at deterministic sample positions and retain
      numerical tolerances explicitly, including the declared ray parameter,
      column-center/edge convention and comparison domain.
- [x] Prove that an otherwise identical candidate outside the finite interval
      is not classified by the boundary's infinite supporting plane.
- [x] Prove ambiguous, missing, parallel, behind-view and near-plane relations
      fail open.
- [x] Retain candidate facing/normal results alongside, but never as authority.
- [x] Prove unresolved candidate support fails open rather than silently
      deleting the complete contribution.
  - [x] Retain the first shortcut falsifier: an unreached target is not
        required for a nearer finite sky boundary (exact replay R2 reaches
        subsector `104`). Source traversal outcome may support diagnosis but
        cannot replace finite-support and relational classification.

### Slice 1 acceptance

- [ ] The four canonical observations separate as required without renderer
      execution or screenshot-dependent rules.
- [x] No infinite-plane false rejection is possible in the retained controls.
- [x] No nearer-but-unsupported geometry is admitted by relational depth.

The completed Slice 1 mechanics are corpus-private synthetic evidence in
`relational_classifier.rs`; they do not claim that the six retained E1M1 rays
have yet supplied complete finite support intervals. The canonical four-case
acceptance therefore remains open rather than allowing correct arithmetic to
stand in for missing source attribution.

## Slice 2 — Partial-Contribution Falsifier

- [x] Add a synthetic contribution entirely nearer than authority.
- [x] Add a synthetic contribution entirely beyond authority.
- [x] Add a contribution crossing the authority depth within one bounded
      horizontal interval.
- [x] Add a contribution entering/leaving the authority's horizontal domain.
- [x] Add a contribution crossing the authorized vertical interval.
- [x] Add a lazy-mapper floor/ceiling control whose world geometry extends
      beyond its source-authorized subsector occurrence while the entire mesh
      remains nearer than authority; only the supported portion is eligible.
- [x] Add the inverse oversized control where one world mesh spans supported
      nearer and unsupported/beyond regions; neither portion may lend its
      classification to the other.
- [x] Add adjacent subsectors sharing sector/plane identity but different
      effective presentation support, proving sector identity alone does not
      authorize a global plane occurrence.
- [x] Split straddling contributions into deterministic survivor ranges while
      preserving source identity, sidedef role, material identity and UV
      parameterization.
- [x] Prove survivor conservation: every classified source range is retained,
      rejected with a reason, excluded as outside source support, or marked
      unresolved/fail-open.
- [x] Prove cutout/masked-middle contributions do not become solid authority.

### Slice 2 acceptance

- [x] The classifier is demonstrably richer than `object -> bool` and does not
      lose valid partial contributions.
- [x] If ordered overlapping authorities cannot be represented by this model,
      stop and record concrete Candidate 2 pressure instead of adding ad hoc
      priority rules.

The corpus-private partition model conserves the complete three-axis domain as
disjoint retained, rejected, outside-support or unresolved fragments. A
bounded report is available through:

```powershell
cargo run -p hello-doom-visibility-conformance --bin relational_partial_contribution_report
```

The ordered-authority falsifier also establishes the planned stop condition.
When two finite authorities own different portions of one contribution, the
current ordered resolver selects the first authority and can only label the
second authority's portion `OutsideSourceSupport`. That portion is independently
supported by the later authority. Distinguishing it from genuinely unsupported
space therefore requires ordered partitioned composition over the remaining
domain. Another whole-contribution priority rule would be incorrect.

This was concrete Candidate 2 pressure. AR-0030 subsequently authorized one
bounded Doom-private ordered-partition experiment. The experiment may refine
only the still-eligible remainder; terminal retained, rejected and unresolved
regions cannot be reopened.

## Slice 2B — Ordered Partitioned Composition

- [x] Add a Doom-private ordered composer in which each finite authority
      classifies only its overlap with the remaining eligible contribution.
- [x] Preserve terminal retained, rejected and unresolved regions so later
      authorities cannot reopen them.
- [x] Prove per-authority and final conservation.
- [x] Prove a later authority can classify a region outside the first
      authority without admitting genuinely unsupported lazy-map geometry.
- [x] Reverse overlapping authority order and prove either an observable
      semantic difference or commutativity rather than hiding order behind an
      identity sort.
- [x] Prove cutout authority is skipped and equal-order overlapping solid
      authority fails open.
- [x] Retain a standalone structural report with no renderer vocabulary,
      screen-column contract or stable API claim.
- [x] Replay and attribute all six retained E1M1 rays through the ordered
      source protocol, retaining candidate and authority occurrence domains
      rather than relying on candidate/nearest-authority hit distances.
- [x] Prove that terminal source-protocol rejection is upstream of relational
      composition: five suspect global-shell contributions are already absent
      from the ordered result and must not be reopened by a later classifier.
- [ ] Derive one honest common comparison domain for the remaining partially
      retained ceiling contribution and its wall authority. The plane instance
      has destination view intervals while the boundary has SEG-local source
      parameter and opening intervals; no shared source parameter is currently
      demonstrated.
- [ ] Run the ordered composer only over E1M1 contributions that remain eligible
      after the ordered source protocol and have a demonstrated common finite
      comparison domain. Do not feed terminally rejected global-shell geometry
      into it.

### Slice 2B acceptance

- [x] The synthetic two-authority falsifier conserves one complete contribution
      as one nearer survivor and one later-authority rejection.
- [x] Ordering is explicit, monotonic and observable under overlapping
      authority.
- [x] Unsupported excess remains unresolved/fail-open.
- [x] All six E1M1 rays carry attributed ordered-protocol outcomes without
      screenshot, infinite-plane or nearest-hit rules: five candidates are
      terminally rejected and one ceiling survives only in two narrow view
      intervals.
- [ ] Every still-eligible candidate/authority pair sent to the composer has a
      common finite source-parameter, horizontal and vertical comparison
      domain. The remaining plane-versus-SEG case does not yet satisfy this.

Run the bounded synthetic gate with:

```powershell
cargo run -p hello-doom-visibility-conformance --bin ordered_partition_composition_report
```

The current report retains two ordered steps, one retained fragment, one
rejected fragment, zero unresolved fragments and final conservation. The
focused library gate passes 142 tests. The E1M1 replay now clears attribution
but exposes a stricter boundary: five global-shell candidates were already
terminally rejected by Doom's ordered protocol, and the one surviving ceiling
uses a plane-instance parameterization that is not interchangeable with the
authorizing wall SEG's source parameter. E1M1 composition therefore remains
paused on a real common-domain question rather than missing source identity.

## E1M1 Disposition

Freeze the relational work here. It remains valid synthetic evidence that
ordered finite authorities can monotonically partition a contribution, but it
is not the next E1M1 realization path.

The six-ray replay establishes a simpler upstream rule:

```text
ordered Doom result
    whole retained     -> reuse ordinary source geometry
    terminal rejected  -> emit nothing
    partial SEG        -> source-relative wall fragment
    partial plane      -> focused plane-occurrence realization
```

Five of the six suspect global-shell contributions are terminally rejected by
the ordered protocol. A later relational stage must not rediscover or reopen
those decisions. The sixth is already a partial plane occurrence; forcing it
through SEG source parameterization would invent a relationship merely to make
the composer appear universal.

This is useful negative evidence for AR-0030: relational classification can
explain source-authority relationships, but reconstructing decisions already
present in the ordered protocol is redundant and creates false common-domain
pressure across heterogeneous source families. Slices 3–5 below are parked
unless new evidence produces an eligible same-domain contribution that the
ordered protocol has not already classified.

## Slice 3 — Synthetic Presentation (Parked)

- [ ] Lower all retained whole and partial survivors into ordinary Tokimu
      declarations using only private corpus code.
- [ ] Submit every survivor (`source-authority-relational-full`) without AABB
      or frustum filtering.
- [ ] Visually and structurally prove nearer geometry survives, beyond geometry
      disappears only within authorized coverage and outside-domain geometry
      remains unchanged.
- [ ] Repeat under small camera yaw/position jitter.
- [ ] Run native WGPU and Browser WebGPU observations.
- [ ] Retain structural fingerprints, survivor counts, rejection reasons,
      resource churn and bounded failure/recovery evidence.

### Slice 3 acceptance

- [ ] Native and browser observations agree semantically.
- [ ] The renderer remains unaware of Doom authority and classification.

## Slice 4 — E1M1 Four-Case Admission (Parked)

- [ ] Run `global-full-submission`, Candidate 1 and
      `source-authority-relational-full` at the exact four-case pose(s).
- [ ] Confirm the complete ordinary input is unchanged before the relational
      classification stage.
- [ ] Confirm the far-left building remains complete without diagonal loss.
- [ ] Confirm the hut and valid outside wall remain visible.
- [ ] Confirm both beside-hut and above-wall distant leaks are removed only
      within the finite authoritative domain.
- [ ] Retain contribution conservation from classifier output through ordinary
      renderer submission.
- [ ] Walk/free-look the exterior and repeat the cases from near and far views.
- [ ] Re-run source-spawn room, first-door, sky-ceiling, runtime door/platform
      and cutout controls for collateral loss.

### Slice 4 acceptance

- [ ] All four required relations produce the required E1M1 presentation with
      no new missing geometry, fixed view box, seam crack or stale-view result.
- [ ] No correctness claim depends on generic post-filtering.

## Slice 5 — Economics And Optional Generic Filter (Parked)

- [ ] Compare preparation time, survivor count, split-fragment count, draw
      count, payload size, allocations and persistent-resource churn against
      global full submission and Candidate 1.
- [ ] Test bounded motion long enough to expose recurring rebuild or identity
      pressure.
- [ ] Only after Slice 4 passes, apply ordinary conservative AABB/frustum
      selection to the relational survivors.
- [ ] Prove the generic post-filter only removes work and does not change the
      accepted semantic result.

### Slice 5 acceptance

- [ ] The Doom-private classifier is practical enough for continued corpus use,
      or its measured cost is retained as a material negative result.
- [ ] Any generic filter remains downstream, conservative and independently
      disableable.

## Decision Ladder

### Relational candidate succeeds

Retain it as Doom-private preparation. Return only provider-neutral pressure to
AR-0030; do not automatically admit its private model or a stable render
primitive.

### Partial contributions are required but sufficient

Retain source-fragment splitting as Doom-private lowering. This is evidence
that semantic contribution identity and presentation granularity may differ;
it is not evidence for renderer-owned clipping.

### Ordered overlapping authority is required

Use only the authorized Doom-private monotonic ordered partition experiment.
If correctness requires reopening a finalized region, arbitrary priorities or
a global raster lifecycle, stop and return to AR-0030. Do not grow the
classifier into an unnamed screen-space compositor.

### The four cases do not separate headlessly

Reject the relational hypothesis before renderer work. Preserve the data as
evidence that authorizing-boundary depth alone is insufficient.

### Candidate source support cannot be bounded honestly

Stop before renderer work. Record whether the missing invariant requires
ordered accumulated occurrence/coverage state and return that pressure to
AR-0030 Candidate 2. Do not approximate plane support with a sector-wide mesh
or promote global geometry extent to source authority.

## Architectural Stop Conditions

Return to AR-0030 before continuing if:

- the renderer would need Doom source vocabulary;
- correctness requires a stable/public renderer contract change;
- ordered overlapping authority requires a general composition model;
- resolution-dependent screen columns would become public API vocabulary;
- source-authorized depth cannot distinguish the canonical keep/reject cases;
- candidate source-occurrence support cannot be established without importing
  ordered accumulated coverage semantics;
- failure containment would require silently hiding unresolved contributions;
- another campaign exposes contradictory meaning for the proposed shared
  mechanism.

## Completion Criteria

This study completes when it either:

1. demonstrates a Doom-private relational pre-render classifier across the
   synthetic and E1M1 matrices, including partial survivors and cross-target
   evidence; or
2. rejects the hypothesis with a precise invariant that justifies Candidate 2
   or another explicitly reviewed direction.

Either outcome must update AR-0030 and the controlling Doom checklist before a
broader renderer framework is proposed.
