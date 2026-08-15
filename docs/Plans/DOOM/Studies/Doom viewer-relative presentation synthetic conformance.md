# Doom Viewer-Relative Presentation Synthetic Conformance

| Field | Value |
| --- | --- |
| Campaign | DOOM |
| Role | Supporting test-campaign plan |
| Status | Active |
| Parent review | AR-0025 (closed; post-close Doom evidence continues) |
| Controlling plan | [DOOM WAD Checklist](../DOOM%20WAD%20Checklist.md) |
| Proposed corpus target | `corpus/campaigns/doom/hello-doom-visibility-conformance/` |
| Next action | Implement Slice 4B's source-informed ordered wall/plane coverage trace, realize the retained partial-coverage runs as Doom-owned presentation fragments, and pass its negative controls before returning to E1M1. |

## Problem

E1M1 has been an excellent integration falsifier, but it is too large and too
visually entangled to remain the first test for every Doom presentation change.
The current live classic-BSP control mixed at least three independently useful
findings in one manual run:

- changing BSP subsectors removed portions of a continuous spawn-room floor;
- an opening door exposed the sky enclosure until the observer crossed it; and
- the hut sky aperture improved while still presenting unrelated distant
  geometry.

Those failures respectively pressure plane continuity, live source topology,
and exact wall/plane/sky coverage. Re-running all of E1M1 after each local
change is slow and makes it too easy to repair a screenshot without proving the
source rule.

This plan creates a small synthetic conformance campaign. It proves bounded
Doom source semantics headlessly first, then presents small native and browser
fixtures, and only then escalates a candidate to E1M1.

## Governing Hypothesis

Synthetic Doom-like source records can isolate viewer-relative presentation
invariants quickly, provided the fixtures exercise the same decoded types,
provider functions, and presentation preparation code used by E1M1.

The campaign must not create a second mock Doom renderer whose tests pass while
the production corpus follows different code.

The likely payoff is not a fundamentally new way to render Doom. A valid final
result may be that faithful Doom presentation needs substantially Doom's own
viewer-relative concepts. The architectural question is which source
invariants require those mechanisms, which responsibilities remain Doom-owned,
and whether any boundary survives independent future callers.

```text
synthetic Doom source records
        ↓
shared Doom provider/preparation functions
        ↓
headless semantic observations
        ↓ pass
small rendered native fixture
        ↓ pass
browser/WASM parity fixture
        ↓ pass
canonical E1M1 falsification
```

## Goals

- Separate plane-continuity, dynamic-topology, wall admission, and sky-aperture
  failures into independently reproducible fixtures.
- Make most source-protocol iteration possible through ordinary fast tests.
- Retain exact source identities and reasons for every admitted, clipped,
  rejected, or unresolved span.
- Exercise the same behavior through a small rendered fixture before paying
  the E1M1 integration cost.
- Preserve native/browser claim separation.
- Establish an explicit escalation gate: synthetic success permits E1M1
  testing but never implies E1M1 success.
- Distinguish reusable invariants from Doom's particular implementation
  mechanics.
- Determine whether provider-neutral architecture can be achieved by keeping
  visibility/presentation preparation source-owned, without inventing a
  provider-neutral visibility algorithm.
- Test the invariant extracted from the released Doom renderer: ordered wall
  processing evolves viewer-relative coverage first, retained plane intervals
  consume that coverage second, and sky presentation paints only the retained
  intervals rather than establishing visibility through world-space geometry.

## Non-Goals

- Historic Doom pixel parity.
- A generic Tokimu visibility, portal, occlusion, scene-graph, or culling API.
- A second Doom decoder or parallel presentation algorithm used only by tests.
- Replacing E1M1 as the canonical integration and visual falsification corpus.
- Treating screenshot similarity as source-semantic proof.
- Admitting visplanes, screen columns, sky names, BSP nodes, SEGs, or Doom
  sector state into `tokimu-render` or the Native Ring.
- Solving sprites, Things, lighting parity, or general gameplay.
- Reproducing binary-angle tables, original fixed-point rounding, framebuffer
  column ordering, or other 1993 raster details solely for pixel agreement.
- Copying Doom's `ceilingclip`, `floorclip`, drawseg, or visplane data
  structures merely because source inspection revealed the behavior they
  implement. The campaign reproduces the bounded invariant, not the original
  renderer's storage layout.

## Expected Payoff And Decision Outputs

The campaign is successful if it produces evidence for any of these outcomes:

### Outcome A — Bounded Doom Presentation Preparation

Doom retains BSP/SEG traversal, openings, plane spans, sky authority, and live
sector-state projection, then emits ordinary Tokimu presentation declarations.
The renderer does not learn Doom topology.

### Outcome B — Transferable Invariants, Source-Specific Mechanisms

The tests identify rules that may transfer conceptually even when the algorithm
does not. For example:

> A near source boundary may close only the portion of the viewer-relative
> domain for which it has source-authorized coverage.

Such an invariant may later inform another provider without admitting Doom's
screen-column implementation as universal machinery.

### Outcome C — Explicit Negative Generalization Result

The evidence may show that visibility and presentation semantics belong to the
source/domain provider more often than a conventional renderer-centric design
assumes:

```text
Doom source topology
        ↓
Doom viewer-relative presentation preparation
        ↓
ordinary Tokimu meshes/materials/views/draws/identities
        ↓
renderer realization
```

That is a provider-neutral architecture without a provider-neutral visibility
algorithm. It is an acceptable and potentially important result.

### Outcome D — No Candidate Survives Current Scope

If every bounded candidate fails and the remaining work requires historical
raster parity, the plan parks with exact failed invariants. It does not expand
quietly into a source port.

The campaign should therefore answer these questions at disposition:

- What semantic truth remains in decoded Doom source records?
- What temporary runtime truth must presentation preparation consume?
- Which viewer-relative facts must a Doom provider own?
- What ordinary declaration and diagnostic facts can Tokimu own without
  knowing Doom?
- Which tempting algorithms or contracts did the evidence tell us not to
  generalize?

## Ownership And Boundaries

- Synthetic map construction, source traversal, span classification, sky
  rules, and dynamic-sector snapshots remain Doom corpus/provider concerns.
- `tokimu-render` receives ordinary meshes, materials, cameras, pipelines, and
  draw commands only.
- Renderer command order remains caller-owned. A selector may filter survivors
  without reordering them.
- Decoded source records remain immutable. Dynamic doors/platforms produce an
  explicit temporary semantic snapshot or equivalent bounded input.
- Unknown source identities and unsupported fixture states reject explicitly
  or fail open according to a named test case; they never disappear silently.
- Any pressure for a stable/public or provider-neutral contract stops this plan
  and returns to Architectural Review.

## Fixture Construction Rule

Every fixture must enter below the WAD-byte decoder but above geometry and
presentation preparation:

```text
small hand-authored DoomMapCore/source records
        ↓
the same Doom geometry/provider functions used by E1M1
        ↓
the same source-labelled candidate/span result shape
```

It is acceptable to use a test builder to remove record-construction noise.
The builder may create vertices, linedefs, sidedefs, sectors, SEGs, subsectors,
nodes, and Things. It may not decide visibility, invent expected triangles, or
perform an alternate lowering.

Each fixture retains:

- source record identities;
- viewer source position, height, and direction;
- sector floor/ceiling heights and flat identities;
- wall tier, side, and texture identity;
- admitted/rejected/clipped result and reason;
- deterministic structural fingerprint; and
- rendered target metadata when presentation is exercised.

Fixture construction should remain explicit for the first several cases. A
slightly tedious builder is preferable to an early convenience API that
quietly encodes expected visibility or presentation behavior. Builder
conveniences may be extracted only after repeated authored fixtures reveal a
genuinely structural pattern.

## Campaign Correctness Laws

The following rules apply across every slice:

```text
false-negative budget = zero
false-positive budget = measured and explained
```

- A required visible contribution may not be discarded for a lower draw count.
- An uncertain classification fails open and retains the uncertainty reason.
- Candidate-count reduction becomes optimization evidence only after the
  applicable semantic and rendered fixtures retain every required contribution.
- Every Level-1 observation emits both a compact deterministic fingerprint and
  an expanded explainable trace.
- Fingerprints cover source identities, presentation-instance identities,
  admitted/clipped intervals, rejection reasons, and runtime state. Final
  triangle or draw counts alone are insufficient fingerprints.
- Historic fixed-point or pixel-raster parity remains out of scope. Evidence
  that cannot be resolved without binary-angle or fixed-point behavior becomes
  an explicit reopening pressure rather than an implicit scope expansion.

The campaign stops for maintainer review if progress begins to require exact
binary-angle tables, fixed-point raster rounding, visplane implementation
quirks, or framebuffer ordering solely to match historical pixels. Continuing
from that point would be source-port archaeology and needs a separate value
decision.

## Initial Fixture Matrix

| Fixture | Source shape | Required invariant | Primary current failure |
| --- | --- | --- | --- |
| Continuous plane across leaves | One floor split across two or more subsectors | Moving across a leaf boundary does not remove visible floor coverage | Spawn floor disappears around pillars |
| Pillar and surrounding plane | Solid pillar inside one continuous surrounding sector/plane | Pillar occludes locally; the surrounding floor and ceiling remain continuous | Pillar-position false negatives |
| Same-side and crossing wall bearings | Walls outside one FOV side plus a wall crossing both sides | Same-side exterior rejects; view-crossing segment survives | Previously repaired FOV endpoint assumption |
| Viewer-plane wall | Solid wall crossing the viewer plane | Wall fails open without closing an unsafe horizontal range | Close-wall disappearance |
| Dynamic door aperture | Two rooms and a ceiling-driven door sector | Closed, opening, open, closing, and closed snapshots change source admission immediately | Sky visible until observer crosses door |
| Moving platform boundary | Adjacent sectors with a changing floor height | Traversal and visible spans use the current runtime floor height | Stale dynamic topology risk |
| Sky aperture with distant wall | Near paired-sky boundary plus unrelated far sector geometry | Presented sky coverage excludes the far wall without hiding valid near geometry | Hut/outdoor leak |
| Single sky-plane coverage | A closed subsector has a nearer `F_SKY1` ceiling plane while an ordinary candidate lies farther along the same view interval | The named source plane may exclude the far candidate only across its bounded projection; it must not grant authority to every one-sky boundary | Remaining lower hut/main-building leak |
| Unequal paired-sky ceilings | Two adjacent `F_SKY1` ceilings at different heights | Color omission and owning-side boundary role remain distinct | Linedef-252 control |
| One-sky negative control | Only one adjacent ceiling uses `F_SKY1` | Ordinary wall band remains; paired-sky authority is not broadened | Over-broad sky repair risk |
| Exact wall/plane edge | Wall tier and floor/ceiling span share a projected boundary | No visible crack and no overlap patch is required | Reconstructed-plane edge gaps |
| Partial horizontal occlusion | Near wall covers only part of a far span | Only covered columns reject; survivor identity and interval remain explicit | Whole-SEG over/under selection |
| Vertical partial occlusion | One column contains upper wall, opening, lower wall, floor, and ceiling contributions at different depths | Near coverage clips only the correct upper/lower interval; the opening and surviving plane spans remain | Horizontal-only solid-range limitation |
| Shared plane key, disjoint instances | Two spatially distinct contributions share `(kind, height, flat, light)` but occupy disjoint/conflicting screen regions | Semantic identity may be shared while presentation instances remain separate; no coverage is fabricated between them | Plane-instance merge finding |
| Projection epsilon neighborhood | FOV-edge, viewer-plane, barely-behind, nearly-zero-width, and extremely-close valid SEGs | Classification remains conservative and explainable around each boundary | Close-wall and endpoint-bearing defects |
| Stationary dynamic transition | Fixed camera observes `closed → opening → open → closing → closed` door/platform state | Results respond to explicit runtime heights without camera movement or observer crossing | Dynamic-state causality |
| Equivalent topology, perturbed record order | Same source relationships built in irrelevant record orders | Structural result/fingerprint remains equivalent after source identity normalization | Accidental `Vec` iteration authority |
| Invalid far-first traversal control | Equivalent map deliberately traversed far before near | Protocol rejects the invalid order or produces the explicitly expected changed coverage | Hidden near-first dependency |
| Thin/pathological topology | Thin sector, zero-height sector, collinear consecutive SEGs, repeated linedef membership, tiny opening, and identical-height distinct sectors | Each case is accepted, rejected, or fails open under a named source rule | Unclassified topology pressure |
| Camera jitter trace | Tiny deterministic position/yaw steps around FOV, viewer-plane, and wall/plane boundaries | Fingerprint changes only for an explained boundary crossing; required surfaces do not flicker | Floating classification instability |
| Cross-representation wall | Whole-linedef, SEG-granular, and reconstructed-span forms describe one simple source wall | Source identity, UV progression, visible interval, and admission reason agree even when draw shape differs | Representation/subdivision drift |
| Masked middle negative control | Cutout wall before opaque geometry | Cutout does not become an ordinary solid occluder | AR-0023 interaction |
| Ordered wall-to-plane coverage | Near one-sided and two-sided wall tiers precede farther wall and plane contributions in the same bounded columns | Near-first wall processing updates upper/lower coverage before floor, ceiling, or sky intervals are retained; closed intervals cannot be reopened by later contributions | Source inspection showed sky is downstream of wall/plane coverage |
| Paired-sky protocol differential | Identical two-sided source boundary run once with one sky ceiling and once with paired sky ceilings | Paired sky changes the source upper-bound/plane-mark result without creating an invisible world-space occluder | Existing paired-sky depth-wall fixture was falsified as authoritative |
| Terminal sky with valid near control | A hut-like near contribution precedes a retained sky interval while unrelated geometry lies farther behind it | The valid near contribution survives, the farther source-unreachable contribution does not enter the terminal interval, and authority remains bounded by column and vertical range | E1M1 sky wall clipped the hut while sky tiles leaked other sectors |
| Fragmented far-source survival | One far SEG overlaps a nearer closed interval but extends on both sides | One source identity survives as two ordered presentation fragments with continuous UV/source correspondence; Boolean keep/reject is rejected | Partial paired-sky control retained two required eight-column runs |

The fixture inventory may expand when E1M1 produces a new irreducible failure,
but every addition must name the source invariant it isolates.

## Evidence Levels

### Level 1 — Headless Semantic Evidence

The test asserts source identities, intervals, plane/span ownership, dynamic
state, and rejection reasons. No GPU or screenshot is involved.

Passing Level 1 means only that the source-semantic result is internally
consistent.

Every Level-1 run retains a compact fingerprint for cheap native/WASM
comparison and an expanded trace containing the exact facts that produced it.

### Level 2 — Small Rendered Evidence

The same fixture is rendered with deliberately legible materials and fixed
cameras. Native and browser observations are retained separately. The fixture
should use tens of draws, not E1M1-scale thousands.

Passing Level 2 means the prepared result survives lowering, upload, depth,
clipping, and presentation on the observed target. It is not pixel parity.

### Level 3 — E1M1 Integration Evidence

Only candidates passing the applicable Level-1 and Level-2 fixtures are
enabled in E1M1. The canonical observation set includes:

- source spawn and pillar movement;
- close-wall turns;
- the first dynamic door;
- stairs and moving platforms;
- hut/outdoor sky aperture;
- courtyard and toxic-pit approaches; and
- exit/secret-catwalk areas.

Any E1M1 visible false negative rejects the candidate regardless of synthetic
results or candidate-count improvement.

## Slice 0 — Baseline And Shared Seam Audit

- [x] Inventory the exact E1M1 functions currently used for source BSP
      traversal, SEG wall lowering, sky adjacency, plane-span reconstruction,
      dynamic-sector snapshots, and source-labelled diagnostics.
- [x] Identify the smallest existing seams the synthetic builder can call
      without copying those algorithms.
- [x] Treat any entanglement discovered in `static_scene.rs` as evidence:
      extract only private Doom-provider/campaign helpers that let E1M1 and the
      synthetic target call the same preparation path.
- [x] Record the current falsified E1M1 observations as baseline expectations,
      including the exact wall-249 replay.
- [x] Define deterministic source-record numbering conventions for fixtures.
- [x] Confirm no renderer or Native Ring API change is required.

### Deliverables

- A short shared-seam inventory in the campaign README or evidence ledger.
- A fixture-schema sketch with bounded record counts.
- A list of any function visibility/refactoring changes needed to reuse real
  code.

### Acceptance Criteria

- Every proposed fixture reaches production Doom preparation code.
- E1M1 and synthetic callers visibly converge on the same extracted helper;
  convenience is not accepted as evidence when the algorithms remain separate.
- No visibility or lowering algorithm exists only in the test crate.
- Any required extraction is private or Doom-provider-local.

Slice 0's audit and extraction ledger is retained in
[Viewer-relative presentation shared-seam audit](../Evidence/Doom%20viewer-relative%20presentation%20shared-seam%20audit.md).
The inventory, schema, baselines, ownership check, and first extracted seam
are complete: both E1M1 and Slice 1 now call the provider-local classic BSP
observation. Slice 2 now also shares source-only vertical clip and plane-span
observation; flat-cell reconstruction and renderer realization remain local to
the E1M1 presentation consumer.

## Slice 1 — Synthetic Source Builder And Structural Oracle

- [x] Create the proposed `hello-doom-visibility-conformance` corpus target or
      an equivalently scoped Doom-campaign test module approved during review.
- [x] Add a small builder for explicit vertices, sectors, sidedefs, linedefs,
      SEGs, subsectors, BSP nodes, and viewer facts.
- [x] Validate all references and reject malformed fixture topology before a
      test can claim presentation behavior.
- [x] Retain canonical structural manifests and fingerprints.
- [x] Standardize the compact fingerprint on normalized source identities,
      presentation-instance identities, intervals, reasons, and runtime state;
      retain an expanded human-readable trace beside it.
- [x] Add negative builder tests for invalid indices, missing sides, empty
      subsectors, and contradictory sector ownership.

### Deliverables

- Reusable Doom-only fixture construction support.
- Deterministic manifest output suitable for native and WASM comparison.

### Retained Slice-1 evidence

- `hello-doom-visibility-conformance` owns only explicit `DoomMapCore` source
  construction, bounded reference validation, and structural manifests. Its
  `observe_classic_bsp` method delegates directly to
  `doom_geometry_provider::observe_doom_classic_bsp`, the same production
  helper now used by E1M1.
- Five deterministic tests cover stable manifest construction, production BSP
  invocation, invalid vertex references, missing linedef sides, empty
  subsectors, and contradictory sidedef/sector ownership.
- The first source control has eight vertices, eight one-sided linedefs, eight
  SEGs, two subsectors, one BSP node, and a watched leaf. It proves the shared
  control can traverse an explicit fixture without inventing a test-only
  selector.
- `compact_evidence_fingerprint` now retains five named normalized buckets:
  source identities, presentation instances, intervals, classifier reasons,
  and runtime state. The source-only BSP seam explicitly records empty
  presentation instances and `runtime:static-source`; later slices extend the
  values rather than changing the format. A bucket-order control proves the
  compact fingerprint is insensitive to irrelevant collection order while the
  human-readable trace preserves every bucket.

### Acceptance Criteria

- The builder contains no candidate-selection or visibility decisions.
- Malformed fixtures fail with bounded source-labelled diagnostics.
- Repeated construction produces identical manifests.

## Slice 2 — Plane Continuity And Near-Wall Semantics

- [x] Add static source-plane controls showing that equal adjacent sectors add
      no plane mark, a changed floor adds only the source floor mark, and
      source-view height suppresses only the impossible plane side.
- [x] Add heading-independent source-plane-mark and exact viewer-plane/edge-on
      controls; those retain source identity without pretending that the
      current horizontal BSP control has reconstructed a plane.
- [x] Extract the source-only plane-span and vertical-clip observation from
      E1M1's presentation preparation. Keep flat-mesh resolution and render
      draw construction in the E1M1/corpus presentation layer.
- [x] Implement the continuous-plane, pillar, FOV-bearing, and viewer-plane
      fixtures.
- [x] Add the shared-plane-key/disjoint-presentation-instance fixture; prove
      that merging semantic peers cannot fabricate coverage between them.
- [x] Rotate the two-leaf control through eight headings; preserve deterministic
      source admission and retain empty plane spans whenever no SEG is admitted.
- [x] Add an away-from-boundary micro-jitter control; prove semantic admission
      remains stable while the pose-specific fingerprint changes deliberately.
- [x] Move the viewer across the two-leaf partition and prove the provider
      recomputes a different source SEG admission rather than retaining the
      previous pose's candidate set.
- [x] Exercise exact and epsilon-neighborhood projection inputs: left/right FOV
      boundary, viewer plane, one endpoint barely behind, nearly zero projected
      width, and extremely close valid geometry.
- [x] Extend the directional matrix from the two-leaf control to each current
      synthetic subsector-boundary fixture.
- [x] Retain a deterministic micro-jitter trace around each current
      classification boundary, including the reason for every fingerprint
      transition.
- [x] Assert that plane coverage is not inferred from reached leaves alone.
- [x] Preserve the existing same-side FOV rejection and crossing-wall
      admissibility rules as regressions.
- [x] Record source-required missing contributions as correctness-failing
      false negatives, and retain extra source-only candidates separately
      until a presentation oracle can classify them as false positives.
### Deliverables

- Headless semantic tests for static plane and wall continuity.
- A deterministic pose/result trace for each fixture.
- Presentation-instance and projection-boundary evidence that is independent of
  final triangle count.

### Acceptance Criteria

- No visible-required plane span disappears solely because the viewer changes
  BSP leaf.
- Near/viewer-plane walls do not vanish or acquire fabricated closure power.
- Equal semantic plane keys remain separate when their presentation coverage is
  disjoint or conflicting.
- Tiny camera changes do not cause unexplained required-surface flicker.
- All rejection reasons retain source identities.

### Retained Slice-2 evidence (static prerequisite controls)

- The synthetic target now calls the production `observe_doom_seg_plane_marks`
  path for equal and unequal two-sided-sector controls. Equal sector facts
  produce neither mark; a changed neighboring floor produces only the front
  floor mark. Source view height disables a floor mark at/below that floor and
  a ceiling mark at/above that ceiling without inventing the other mark.
- Changing the synthetic viewer heading leaves those source plane facts
  identical. This deliberately separates immutable source eligibility from
  later viewer-relative SEG admission.
- `DoomVisibilityFixture::classic_bsp_manifest` now fingerprints a bounded
  pose/result trace containing admitted SEG record order, reached leaves,
  backface/FOV/edge-on/fail-open counts, solid coverage count, watched
  elisions, and bounded samples. Repeating a pose is stable; reversing the
  heading changes the fingerprint with its retained rejection evidence.
- A deliberately authored viewer-on-wall case reaches the production classic
  traversal as `edge_on`, rather than acquiring ordinary solid-range authority.
  It is a bounded source-projection control only, not historic column parity.
- `DoomVisibilityFixture::observe_classic_vertical_clips` now lowers its
  explicit source walls and delegates to the same
  `doom_geometry_provider::observe_doom_classic_vertical_clip_state` used by
  E1M1. The deterministic fixture proves repeated tier, clip, and plane-span
  evidence is stable without creating a material, mesh resource, or renderer
  draw.
- These controls do **not** yet prove continuous presented floor/ceiling
  coverage across leaves. The next interaction fixtures must apply the shared
  span result to partial wall/opening/sky conditions before the campaign can
  claim that stronger invariant.
- The two-leaf continuous-plane control proves one source floor instance can
  collect admitted SEG evidence from more than one BSP leaf. The pillar
  control confirms that a nearer obstacle may reduce the contributors at a
  pose without creating a second floor identity. A heading reversal empties
  projected spans when no SEG is admitted but leaves the source plane marks
  unchanged; an exact viewer-on-wall control inherits the traversal's
  fail-open result and claims no vertical coverage.
- The shared-key collision control gives two source sectors equal floor
  semantics (height, flat, and light) while their projected writes conflict.
  The provider retains one semantic key, two presentation instances, and one
  explicit collision split; it does not merge those writes merely because the
  source plane values match.
- Eight deterministic headings over the two-leaf control retain the same
  source result for a repeated pose. When the current source protocol admits
  no SEG, the corresponding plane-span result is empty rather than inferred
  from reached leaves.
- An away-from-boundary heading jitter of `±1e-7` radians preserves admitted
  SEG records and all retained classification counts. Its fingerprints differ
  intentionally because the trace includes the exact pose; fingerprint
  equality is therefore not used as a false proxy for semantic invariance.
- Moving the two-leaf viewer from the west to east side of its partition
  changes the provider-admitted SEG set. The fixture proves re-evaluation at a
  new source pose; it does not yet claim a rendered plane-continuity result.
- A crossing-bearing source pose retains a non-empty admitted SEG set, while a
  same-side exterior heading retains an explicit FOV rejection. This guards
  the current source-FOV distinction without claiming original Doom column
  parity.
- The same control records three `±1e-7` heading samples at the FOV bearing.
  Each exact pose repeats identically and retains its expanded FOV rejection
  trace. This is the first boundary-jitter control; near-plane, tiny-width,
  and endpoint-behind cases remain deliberately open.
- Moving the viewer one source unit across the viewer-plane control keeps the
  exact position edge-on/fail-open, then changes to an ordinary forward
  source classification on one side and a backface/FOV rejection on the
  other. The control prevents a close-wall boundary from silently becoming
  generic solid-range coverage.
- The eight-heading matrix now covers both current subsector-boundary fixtures:
  the two-leaf control and the independent pillar topology. Each exact pose
  repeats through the shared vertical-clip seam; no-admission poses retain no
  inferred plane span.
- The viewer-plane boundary now retains deterministic `-1`, `0`, and `+1`
  source-position observations. Each fingerprint transition carries named
  `edge_on`, backface, FOV, and near-fail-open counters in its trace. This
  records source classification changes without pretending that nearby poses
  should have equal results.
- A one-endpoint-behind SEG fails open without any solid-range closure. A
  one-unit-wide valid SEG is admitted with a bounded three-column conservative
  interval, and an extremely-close valid SEG is admitted normally. These are
  source-projection facts only; the campaign claims neither pixel parity nor
  renderer-owned clipping semantics.
- A reverse-heading control reaches both synthetic BSP leaves while admitting
  no SEG and produces no plane-span key. Reached source topology is therefore
  not permitted to fabricate plane coverage.
- The forward two-leaf control now names source SEG `0` as a required
  contribution. Its absence fails the test as a correctness-failing synthetic
  false negative. Other admitted SEG identities are retained separately as
  **unresolved extra candidates**, not labelled false positives: a source-only
  fixture has no authoritative rendered visibility oracle. Slice 5/real
  source-presentation controls may classify those candidates later; this
  restriction prevents draw-count reduction from laundering an unproven
  visibility claim.

## Slice 3 — Dynamic Topology Snapshots

- [x] Extract a shared Doom-local height-snapshot projection seam. It accepts
      declared temporary height facts but owns neither a door/platform state
      machine nor renderer state.
- [x] Implement closed, opening, open, closing, and reclosed door snapshots.
- [x] Implement a moving-platform height sequence.
- [x] Run the complete door and platform sequence from a stationary camera
      before adding any observer-motion control.
- [x] Prove decoded source records remain unchanged while the temporary
      semantic view reflects current runtime heights.
- [x] Exercise traversal before and after the observer crosses the dynamic
      boundary; results must depend on state, not which side currently owns the
      observer.
- [x] Retain stale-source negative controls demonstrating the prior failure.

### Deliverables

- Headless dynamic-door and moving-platform traces.
- Source immutability and runtime-snapshot tests.

### Slice-3 retained evidence

- The same immutable `DoomMapCore` is projected through declared door ceiling
  states `128 → 96 → 64 → 96 → 128`; lowered wall maxima follow the supplied
  state exactly while horizontal source-BSP admission remains unchanged.
- The same fixture is projected through declared platform floor states
  `0 → 16 → 32 → 16 → 0`; lowered wall minima follow the supplied state
  exactly.
- A stationary two-leaf observer retains its source location and sees its
  floor-plane identity change with `0 → 24 → 0`; the first and reclosed
  observations match.
- A snapshot naming an unavailable sector source record is rejected as
  `RuntimeSnapshotSectorUnavailable`; stale state cannot disappear as a
  silent no-op.
- The two-sided dynamic-boundary control retains a closed height band and an
  open empty band for identical decoded source. West/east viewers take
  different source traversal paths, while the prepared open doorway remains
  empty from both sides.

### Slice-3 seam finding

The first dynamic-fixture audit initially treated reuse of E1M1's runtime
controllers as a prerequisite. Review corrected that overreach: this slice
tests whether preparation consumes **current declared heights**, not whether a
synthetic fixture can reproduce E1M1 activation, timing, waiting, or reversal
policy. The ownership boundary remains:

```text
E1M1 application
    specials.rs
        manual door / platform runtime state machines

synthetic conformance campaign
    supplies declared current height facts
    must not duplicate the state machines

doom-map-provider
    decode-only by documented boundary

doom-geometry-provider
    geometry preparation, not moving-sector runtime meaning
```

The campaign can begin with a narrower shared seam:
`project_doom_sector_runtime_heights` in `doom-geometry-provider`. It accepts
explicit caller-owned snapshot facts and projects them over a cloned
`DoomMapCore`; it does not decide timing, activation, or a door/platform
policy. E1M1's live runtime and the synthetic fixture now use that same
projection. This is not evidence for a Tokimu-wide dynamic-topology contract,
and it does not authorize moving the state machines into the renderer or
Native Ring.

### Acceptance Criteria

- An open doorway has the same admission result from either side when all
  other facts match.
- A stationary observer sees every admission change caused by runtime semantic
  state alone; no camera update or boundary crossing is required.
- Every state transition is attributable to explicit runtime height input.
- No presentation selector mutates decoded WAD records.

### Slice-4 initial evidence

- Adjacent `F_SKY1` ceilings at unequal heights retain two non-colored paired
  sky-boundary triangles while ordinary upper wall bands are absent; the SEG
  remains source-classified `Open`, not a generic solid occluder.
- One `F_SKY1` ceiling plus one ordinary ceiling retains the ordinary upper
  wall band and produces no paired-sky depth boundary.
- An authored two-sided `MASKED` middle is retained as source texture identity
  while the underlying opening remains source-classified `Open`. Texture or
  alpha data has not been permitted to invent solid occluder authority.
- A two-leaf near-solid/far-wall control retains near SEG `0` and rejects only
  the watched far subsector `1` when the near source interval fully covers the
  far child's projected interval. Its otherwise identical two-sided/open
  control retains both source SEGs and has no far-child prune. This is
  evidence for source-labelled range authority, not generic mesh occlusion.
- A two-sided aperture with front heights `0..128` and back heights `24..96`
  retains independent upper and lower source wall tiers plus floor/ceiling
  mark facts. The upper tier may legitimately consume that aperture's visible
  ceiling span; source marking and resulting visible plane span are therefore
  retained as distinct observations.
- Heading perturbations of +/- `1e-7` radians around the aperture control are
  deterministic and retain both tier contributions. This is a bounded edge
  control, not a claim of original Doom pixel parity.
- The vertical observation now retains bounded per-column final clip facts:
  source SEG IDs for upper/lower/middle tiers plus the remaining opening
  interval. These diagnostic cells remain Doom-provider evidence and are not
  renderer pixels, a scissor contract, or a public visplane API.
- Paired-sky candidate identity is now retained per projected source column,
  separately from upper/lower/middle wall tiers. The paired-sky/far-wall
  control retains the near paired-sky SEG and the far wall as separate source
  candidates: the far wall contributes the sole ordinary solid range while
  paired sky contributes none. This proves the Level-1 separation, not final
  depth/color suppression.
- The same control found and repaired an ordinary vertical-observer defect:
  a valid SEG crossing both horizontal FOV edges was discarded solely because
  both endpoints lay outside the FOV. The vertical observer now uses the same
  crossing-aware source-FOV test as classic BSP admission.

## Slice 4 — Sky, Wall, And Plane Boundary Semantics

- [x] Implement paired-sky, one-sky, distant-wall, and exact edge fixtures.
  - [x] Paired-sky and one-sky source controls distinguish omitted ordinary
        upper-wall presentation from retained paired-sky depth-boundary facts.
  - [x] Masked-middle source control proves authored middle texture identity
        does not change an open two-sided SEG into a solid occluder.
  - [x] Add a near-aperture/distant-control fixture with explicit surviving
        near identity and excluded far identity. The paired open control
        confirms that proximity alone cannot grant solid-range authority.
  - [x] Add exact horizontal and vertical edge controls to the same aperture.
        The aperture's upper/lower/opening intervals and +/- `1e-7` heading
        trace are now retained headlessly.
- [x] Keep visible wall color, depth/coverage authority, and sky-span identity
      separate in the observations. Per-column traces retain tier sources and
      clip bounds, while source plane marks remain distinct from a resulting
      visible plane span.
- [x] Prove that a near sky aperture can exclude unrelated far geometry without
      hiding the near control geometry. The fixed native paired-sky control
      retains the blue sky presentation and the lower orange far-wall span,
      while the source-owned paired-sky depth boundary excludes only the
      overlapping upper far-wall span.
- [x] Exercise partial horizontal coverage rather than only full-screen closure.
      The solid/open near-control pair retains explicit covered and surviving
      far identities; the vertical aperture covers only a bounded column set.
- [x] Exercise vertical partial coverage in the same columns: upper wall,
      opening, lower wall, floor, and ceiling must retain independent clip
      intervals at different depths. The vertical-aperture control retains
      distinct upper/lower source tiers, a non-empty opening, and independent
      floor/ceiling source marks and projected-span evidence.
- [x] Include the masked-middle negative control so alpha bytes do not confer
      solid occluder authority.
- [x] Retain source-labelled paired-sky candidate columns separately from
      ordinary upper-wall and horizontal-solid authority. The inverse one-sky
      control has no paired-sky candidate columns.
- [x] Present the paired-sky/far-wall candidate with an explicit Doom-local
      depth/color order before claiming that the far wall is excluded. This is
      Level-2 presentation evidence, not a reason to grant generic occlusion
      authority to paired sky. The native control submits sky colour, then the
      non-coloured source boundary, then the unrelated far wall.
- [x] Reject overlap epsilons and broad hidden depth walls as acceptance tools.
      The native paired-sky boundary is lowered from the exact source SEG and
      authored ceiling-height difference; its partial-screen result requires
      neither screenshot-sized geometry nor an overlap tolerance.
- [x] Refine the sky aperture family after the E1M1 lower-hut captures showed a
      different source relationship: a nearer lone `F_SKY1` ceiling plane can
      precede an ordinary static-shell hit even when `sky-boundary=none`.
      `single_sky_plane_far_control_fixture` retains a closed named sky-plane
      source loop and explicitly proves that it has no paired-sky boundary
      authority. The presentation control is intentionally bounded to the
      declared source-plane interval; it does not reinterpret the existing
      one-sky upper-wall negative as a universal sky occluder.
- [x] Falsify global source-flat authority in E1M1. Drawing every retained
      `F_SKY1` flat depth-only removed the reported sky leaks but also masked
      valid nearby hut geometry. The experiment proves that exact source
      identity without current viewer/span admission is still too broad.
- [x] Test source-sector admission for retained sky flats. It failed at the
      predicted granularity boundary: the hut remained visible nearby but was
      masked after the observer backed away. A sector that contributes one sky
      span does not grant every retained subsector flat presentation authority.
- [x] Test exact viewer-relative sky screen-cell depth coverage reconstructed
      from the current shared BSP/vertical-span observation. This bounded
      dynamic corpus mesh removed the source-sector granularity error, but
      E1M1 still exposed two independent failures: the older unconditional
      paired-sky boundary hid the hut from the spawn-room window, while a sky
      ceiling cell still allowed unrelated nearby-room geometry through. The
      exact-cell control therefore improves identity granularity but does not
      establish sufficient depth/terminal-coverage semantics.
- [ ] Run E1M1 with the unconditional paired-sky boundary presentation removed
      while retaining its source records for inspection. Determine whether
      the remaining ceiling leak is caused by ordinary Euclidean depth order
      disagreeing with Doom source reachability rather than by missing sky
      cell identity.

The existing paired-sky depth boundary, global source-flat depth, admitted
source-sector flat, and exact world-space sky-cell experiments are retained as
falsified or bounded mechanism controls. None may satisfy this slice's source
authority claim merely by producing a desirable image. The source-informed
candidate is specified separately in Slice 4B.

### Deliverables

- Headless exact interval/span evidence.
- An explicit explanation of how wall and plane boundaries share coverage.
- A per-column vertical contribution trace that explains every retained or
  clipped interval.

### Acceptance Criteria

- Far geometry is excluded only where the source protocol establishes coverage.
- Near valid geometry remains visible from both authored sides.
- Wall/plane joins require no screenshot-specific patch.
- Upper/lower wall, opening, floor, and ceiling contributions coexist without
  a horizontal-only shortcut deleting required vertical coverage.
- The result remains Doom-owned and provider-local.

## Slice 4A — Ordering, Pathology, And Representation Controls

- [x] Separate irrelevant evidence-container order from source-record identity.
      The compact evidence fingerprint test proves that source, interval, and
      classifier buckets normalize iteration order without merging their
      authority. Review refined the original "source record construction
      order" wording: raw WAD record indices are retained source identity, so
      permuting them is not an irrelevant mutation and must not be laundered
      into an equivalent topology claim. A future importer may add a named
      source-correspondence normalizer if it needs to compare differently
      ordered source containers.
- [x] Deliberately violate near-first traversal and require an explicit
      rejection or a precisely explained coverage difference. A provider-local,
      test-only far-first control on the same near-solid/far-wall source map
      admits `[1, 0]` and cannot prune the watched far leaf; production
      near-first admits only `[0]` and prunes that leaf through its retained
      solid range. Traversal order remains structurally fixed inside the Doom
      provider; no caller-visible renderer or application selector was added.
- [x] Add Level-1 topology controls for thin sectors, zero-height closed
      sectors, collinear consecutive SEGs, repeated linedef membership, tiny
      openings, and identical-height sectors with distinct source identity.
      The controls retain nearly-zero-width valid SEGs conservatively,
      distinguish a zero-height back sector from a one-unit opening, accept a
      five-edge loop with two collinear SEGs sharing one linedef, and split
      conflicting equal-key plane coverage by source sector identity.
- [x] Compare whole-linedef, SEG-granular, and reconstructed-span
      representations of one simple wall. The split-wall control compares two
      whole-linedef triangles, six SEG-clipped triangles, and a bounded
      reconstructed `0.25..0.75` source interval.
- [x] Require equivalent source identity, UV progression, visible interval,
      and admission reason without requiring identical draw representation.
      Every representation retains linedef, sidedef, sector, side, and wall
      role; the full split spans source U `0..128` and the reconstructed span
      retains U `32..96`. Triangle count is explicitly non-semantic.

### Deliverables

- Ordering-invariance and ordering-significance traces.
- A bounded topology/pathology ledger.
- Cross-representation equivalence evidence.

### Acceptance Criteria

- Irrelevant container order has no semantic authority.
- Required near-first ordering is explicit and tested rather than inherited
  accidentally from collection iteration.
- Pathological inputs cannot silently disappear or fabricate geometry.
- Subdivision changes representation granularity without changing source
  semantics or continuous texture progression.

## Slice 4B — Source-Informed Ordered Coverage Reconstruction (New)

Source inspection changed the tested causal model. This slice does not attempt
to reproduce Doom pixels or copy its renderer. It tests the smaller invariant
that wall traversal, vertical clip evolution, retained plane intervals, and sky
painting form one ordered Doom-owned preparation protocol.

The source-informed state is diagnostic and provider-local:

```text
near-to-far admitted SEG/tier
        ↓
per-column upper/lower coverage evolves
        ↓
floor/ceiling plane intervals are retained from current coverage
        ↓
sky identity paints only retained sky-plane intervals
        ↓
surviving source-labelled fragments become ordinary presentation declarations
```

It is not a public pixel-span API, renderer scissor contract, or generic
occlusion service.

- [ ] Add a bounded provider-local ordered-coverage trace whose input is the
      already admitted near-to-far source SEG/tier sequence and whose output
      retains, per diagnostic column, the upper bound, lower bound, plane-mark
      intervals, source identity, and reason for every state transition.
  - [ ] Prove a one-sided middle wall closes both applicable bounds and that a
        later far wall or plane cannot re-enter the closed interval.
  - [ ] Prove two-sided upper and lower tiers update their respective bounds
        independently while the opening remains available to farther source
        contributions.
  - [ ] Fail open with an explicit reason when projection, vertical ordering,
        or source ownership is unresolved; never convert uncertainty into a
        closed interval.
- [ ] Add a paired-sky protocol differential using identical geometry and
      traversal order for one-sky and paired-sky cases. Assert the changed
      upper-bound/plane-mark facts directly and assert that neither result
      invents a world-space hidden depth wall.
- [ ] Add a terminal-sky fixture with three ordered roles: a valid near
      hut-like contribution, a bounded retained sky-plane interval, and an
      unrelated farther contribution.
  - [ ] Require the near contribution to survive where source order admits it.
  - [ ] Require the farther contribution not to enter the terminal sky
        interval after the ordered source protocol has closed it.
  - [ ] Repeat with a small deterministic camera jitter and retain every
        explained boundary transition.
- [ ] Realize the existing `partial-paired-sky-far-control` result as two
      Doom-owned presentation fragments for the same far source SEG:
      `[112,119]` and `[201,208]`, with `[120,200]` excluded by the nearer
      source interval.
  - [ ] Preserve linedef, sidedef, SEG, wall-tier, and continuous source-UV
        correspondence across both surviving fragments.
  - [ ] Retain a negative Boolean selector result proving that whole-source
        keep loses the excluded overlap and whole-source reject loses both
        required survivor runs.
- [ ] Re-run the deliberate far-first traversal control against the new trace.
      It must reject the invalid protocol order or retain the precisely named
      changed result; collection order may not silently substitute for Doom's
      required near-first traversal.
- [ ] Retain a short reference-evidence mapping from the extracted invariant to
      the inspected classic Doom and faithful-port wall/plane paths. Record
      behavior and ordering only; do not claim exact fixed-point, visplane, or
      framebuffer parity.
- [ ] Add small native presentation only after the corresponding headless
      trace passes. Use legible source-role colors and ordinary Tokimu draws;
      do not revive the falsified world-space sky-depth boundary as the
      authority.
- [ ] Add Browser WebGPU observation only after native semantic and
      presentation evidence pass, retaining semantic rather than pixel parity.

### Deliverables

- A deterministic ordered wall/plane/sky transition trace.
- A source-labelled partial-fragment manifest with continuous correspondence.
- Paired-sky, terminal-sky, and far-first negative controls.
- Native and browser observations for the surviving bounded realization.

### Acceptance Criteria

- Sky presentation consumes retained source-plane coverage; it does not create
  global visibility authority through world-space geometry.
- A valid near hut-like contribution is not clipped merely because a later sky
  interval occupies overlapping screen columns.
- Source-unreachable far geometry cannot re-enter a column/vertical interval
  closed by earlier ordered Doom source processing.
- Partial survival is represented without converting one source contribution
  into an all-or-nothing candidate and without losing source or UV identity.
- One-sided, two-sided upper/lower, opening, ordinary plane, and paired-sky
  transitions have independently explainable effects.
- The realization remains Doom-owned and provider-local. `tokimu-render`
  receives only ordinary declarations and learns no Doom coverage vocabulary.
- Exact original arrays, fixed-point rounding, or framebuffer pixel order are
  not required for the retained semantic claim.

## Slice 5 — Small Native Presentation Fixture

- [x] Build the first fixed-camera paired-sky/far-wall native control from the
      same source fixture as Level 1. It explicitly submits blue sky colour,
      then a non-coloured paired-sky depth boundary, then the far wall; it uses
      no E1M1 assets or generic occlusion API.
- [x] Retain the first native visual observation of that paired-sky control
      before claiming that its declared depth order excludes the far wall only
      in the source-owned interval. Manual observation retained on 2026-08-14:
      blue sky remained visible, the lower orange far-wall span remained
      visible, and only its upper overlap was clipped. The presented frame
      reported `draws=3`, `materials=3`, `pipelines=3`, and
      `diagnostic=none`; this is an observation of the declared fixture, not a
      pixel-determinism or original-Doom parity claim.
- [x] Present each applicable synthetic fixture through ordinary Tokimu render
      commands and the same Doom-prepared result used by headless tests.
  - [x] Add a one-sky negative mode to the paired-sky native executable. It
        changes the actual source height/texture relationship, retains no
        paired-sky boundary triangles, and presents the authored upper wall as
        visible green geometry through the same three-draw path. This prevents
        a texture-name-only pseudo-control from laundering absent geometry.
  - [x] Retain the one-sky native visual observation: the green ordinary upper
        wall remains visible, with no invisible paired-sky authority. Native
        observation retained on 2026-08-14: `source-boundary-triangles=0`,
        `control-mesh-vertices=6`, and `far-wall-triangles=2`; first and warm
        frames each reported three draws, materials, and pipelines with
        `diagnostic=none`. The warm frame retained zero uploads and
        replacements, three lifetime uploads, and zero lifetime replacements.
  - [x] Add a vertical-aperture native control driven by the shared structural
        fixture and production SEG wall lowering. Green upper and yellow lower
        tiers retain separate source roles around opening `24..96`; an orange
        far surface is submitted behind them through ordinary Tokimu commands.
  - [x] Retain the vertical-aperture native visual observation: orange survives
        in the opening while green/yellow cover only their respective source
        intervals. Native observation retained on 2026-08-14: two upper and
        two lower source triangles, four draws/material resolutions, two
        pipeline switches, and `diagnostic=none`; the unchanged warm frame
        retained zero mesh uploads/replacements, four lifetime uploads, and
        zero lifetime replacements.
- [x] Use legible asymmetric textures/colors for front/back, floor/ceiling,
      near/far, sky, cutout, and unresolved surfaces. The controls deliberately
      use blue sky/background, orange far geometry, green ordinary/retained
      near geometry, yellow lower tiers, and the explicit Purple diagnostic
      asset only where the E1M1 source case declares an omission. Colour is
      fixture-local evidence, not an admitted renderer semantic.
- [x] Include rendered controls for shared-key/disjoint plane instances,
      vertical partial occlusion, stationary dynamics, and the projection
      epsilon neighborhood before escalating those cases to E1M1.
  - [x] Add the shared-key/disjoint-plane native control. It lowers actual
        bounded subsector floor surfaces from two different source sectors
        with equal floor-key facts, then retains them as separate green/orange
        source instances. Native observation on 2026-08-14: three draws,
        materials, and two pipelines with `diagnostic=none`; the unchanged
        warm frame retained zero mesh uploads/replacements.
  - [x] Retain the matching Browser WebGPU visual observation. On 2026-08-14
        `shared-key-plane` presented separate green/orange source regions with
        three first/warm/jitter draws and zero warm/jitter mesh churn.
  - [x] Add the native stationary dynamic-door snapshot control. It lowers the
        closed two-sided wall band from an explicit ceiling snapshot and proves
        that the corresponding open snapshot yields no source wall band; it
        owns no E1M1 activation, timing, waiting, or reversal policy.
  - [x] Retain the native dynamic closed-band/open-aperture observation.
        On 2026-08-14 it presented three draws/materials, two pipelines, and
        `diagnostic=none`; the actual closed source band was green with six
        lowered vertices while the explicit open snapshot had zero bands and
        retained the orange far control. The control projects this specific
        doorway's varying source `Z` axis horizontally; it does not relabel
        the map or claim a general world-axis rule. The warm frame had zero mesh
        uploads/replacements.
  - [x] Retain the matching Browser WebGPU visual observation for dynamic
        closed-band/open-aperture snapshots. On 2026-08-14
        `dynamic-door-snapshot` presented the closed two-triangle green band
        and open zero-band orange control with three first/warm/jitter draws
        and zero warm/jitter mesh churn.
  - [x] Add the native stationary moving-platform snapshot control. It lowers
        two independent wall meshes from the same immutable source fixture with
        declared floor heights `0` and `48`; the side-by-side presentation is
        diagnostic only and owns no platform timing or activation policy.
        On 2026-08-14 the native control visibly retained the taller green
        `floor=0` wall and shorter yellow `floor=48` wall, with three draws,
        three materials, three pipelines, and `diagnostic=none`.
  - [x] Retain the matching Browser WebGPU `Platform snapshots` observation.
        On 2026-08-14 `platform-snapshot` rendered the same immutable source
        under declared floors `0` and `48` as the visibly distinct green and
        yellow/orange source controls. First, warm, and bounded-jitter frames
        each reported three draws and zero warm/jitter mesh uploads or
        replacements, with `diagnostic=none`. The browser host owns only button
        selection and bounded status presentation.
  - [x] Add and retain the native projection-epsilon presentation control. On
        2026-08-14 the behind-viewer source SEG failed open with no solid
        admission, while the thin and extremely-close valid source SEGs
        presented green/orange with three first/warm draws, two pipelines,
        `diagnostic=none`, and zero warm mesh churn. Its per-case horizontal
        magnification is explicit diagnostic presentation, not pixel parity.
  - [x] Retain matching Browser WebGPU projection-epsilon observation. On
        2026-08-14 `projection-epsilon` retained the behind-viewer
        fail-open case with zero solid admission, the valid thin control with
        three covered columns, and the extremely-close valid control with 320
        covered columns. Its legible green/orange presentation matched the
        named source cases; first, warm, and bounded-jitter frames each
        reported three draws, zero warm/jitter mesh churn, and
        `diagnostic=none`.
  - [x] Retain a native masked-middle negative control using the admitted
        categorical-cutout mechanism: a checkerboard cutout is in front of an
        opaque far wall, so transparent texels must retain the far-wall
        contribution. The control does not infer source visibility from alpha;
        it proves that the rendered mechanism cannot be mistaken for ordinary
        solid source authority. On 2026-08-14, the native fixture visibly
        retained a green checkerboard only at declared opaque texels and an
        orange far wall through transparent texels, with three draws and no
        backend diagnostic. On 2026-08-14 the matching Browser WebGPU control
        retained the same green categorical coverage over the orange far wall
        while exposing the orange wall through checkerboard holes. Its first,
        warm, and bounded-jitter frames each retained three draws with zero
        warm/jitter mesh uploads or replacements and `diagnostic=none`.
- [x] Provide fixed named cameras and decide the observer boundary. Each native
      and browser control uses its named fixed screen-plane/orthographic
      diagnostic camera, with the paired-sky control additionally applying one
      retained `offset_x=0.08` camera-update control. A synthetic interactive
      observer is deliberately not added: it would duplicate the E1M1
      application's input, movement, and source-navigation policy without
      proving another presentation invariant. E1M1 remains the bounded
      interactive integration observer; these controls remain fixed-camera
      source fixtures.
- [x] Retain first/warm-frame resource observations and prove camera movement
      causes no static mesh replacement churn.
  - [x] Make the paired-sky control reject any unchanged warm frame with a
        static mesh upload or replacement and print frame/lifetime counts.
        Native observation retained on 2026-08-14: first and warm frames each
        presented three draws, three material resolutions, and three pipeline
        switches with `diagnostic=none`; the warm frame reported zero mesh
        uploads, zero replacements, three lifetime uploads, and zero lifetime
        replacements.
  - [x] Add the bounded camera-movement control before closing the parent item.
        Native observation retained on 2026-08-14: the third frame applied an
        actual camera offset of `0.08` while retaining three draws/materials/
        pipelines, zero frame mesh uploads/replacements, three lifetime mesh
        uploads, zero lifetime replacements, and `diagnostic=none`.
- [x] Capture manual visual observations without claiming pixel determinism.
      The first paired-sky observation is retained above as a qualitative
      source-interval check with explicit frame metadata.

### Slice-5 initial evidence

- The first run presented a black frame because the colour-only sky pipeline
  declared no depth attachment while the WGPU surface pass used
  `Depth32Float`. The backend validation callback retained the incompatibility,
  but `present()` itself returned success. The fixture now promotes any drained
  backend diagnostic to a terminal fixture error and uses a depth-compatible,
  non-writing sky pipeline. This is retained AR-0024/AR-0027 diagnostic-boundary
  evidence, not an engine-wide error-delivery admission.
- The repaired native control presents three deliberately distinct roles:
  colour-only sky, a non-coloured paired-sky depth boundary, and an unrelated
  far wall. The observation demonstrates that source coverage authority can be
  presented independently from visible wall colour without adding a generic
  occlusion contract to `tokimu-render`.
- The first one-sky negative implementation initially changed only a ceiling
  texture name on the paired fixture. Its regression correctly found that the
  original low-to-high sector relationship produced no viewer-facing authored
  upper wall. The fixture now changes the source height relationship too:
  viewer-side sky ceiling `128`, opposite ordinary ceiling `96`, and an
  authored upper texture. The resulting control proves real ordinary-wall
  presentation rather than treating missing geometry as negative evidence.
- Manual observation of the corrected one-sky control retained a visible green
  upper wall across the source interval, blue background outside it, and the
  independent lower orange far wall. This is the expected inverse of the
  paired-sky depth-only result and demonstrates that one sky ceiling alone
  cannot acquire paired-sky coverage authority.

### Deliverables

- A small native executable or mode with fixture and camera selectors.
- Native visual and structural evidence ledger.

### Acceptance Criteria

- Each semantic failure has a visually unambiguous small reproduction.
- All required surfaces are visible and no forbidden surface survives.
- No fixture needs E1M1 assets to explain its result.

## Slice 6 — Browser/WASM Parity

- [x] Add a dedicated browser/WASM companion that preserves Rust-owned source
      fixture construction, Doom lowering, pipeline state, and bounded
      diagnostics; TypeScript/DOM owns only named-fixture selection and status
      display. `cargo check -p hello-doom-visibility-conformance-web --target
      wasm32-unknown-unknown` passes and `wasm-bindgen` produced the local web
      package.
- [x] Run the applicable structural fixtures through DOM/WASM. Browser
      observations retain the same paired boundary (two triangles), one-sky
      ordinary wall control (six source-control vertices), and vertical
      aperture upper/lower tiers (two triangles each) as native.
- [x] Present paired-sky, one-sky-negative, and vertical-aperture fixtures
      through Browser WebGPU. Manual observations retained on 2026-08-14:
      paired sky showed blue sky with the far control fully excluded;
      one-sky showed a green ordinary upper wall with the red far control;
      vertical aperture showed the green upper and yellow lower source tiers
      around the red opening control.
- [x] Retain browser adapter/device/viewport metadata and bounded failures.
      Each observation reported `backend=browser-webgpu`, `device=other`,
      `adapter=` (unreported by this browser), and `canvas=960x600`; all three
      reported `status=presented` with no provider diagnostic.
- [x] Compare semantic manifests across native and browser without requiring
      pixel-identical captures. Native and browser retain the same source
      counts and ordered source-control meaning; screenshots are visual
      observations only.
- [x] Run a deterministic bounded camera update after first and unchanged warm
      browser frames. The `offset_x=0.08` third frame must retain zero mesh
      uploads/replacements; it is a static-resource stability control, not a
      claim of historical Doom camera behavior. On 2026-08-14, paired-sky,
      one-sky-negative, and vertical-aperture each retained zero jitter-frame
      mesh uploads and replacements at three, three, and four draws
      respectively.
- [x] Present the cutout non-occluder through Browser WebGPU. On 2026-08-14,
      the green checkerboard remained in front only at retained categorical
      texels while the orange far wall remained visible through its transparent
      texels. The first, warm, and `offset_x=0.08` jitter frames retained three
      draws, zero warm/jitter mesh uploads or replacements, and
      `diagnostic=none`; this is renderer-mechanism evidence only and grants
      no source visibility authority to alpha.
- [x] Keep unsupported browser mechanisms explicit rather than substituting a
      CPU result silently. Browser WebGPU was available for these observations;
      a future unsupported result must remain a returned bounded status.

### Slice 6 implementation note

`cargo clippy -p hello-doom-visibility-conformance-web --all-targets --target
wasm32-unknown-unknown -- -D warnings` reaches pre-existing strict-WASM
baseline failures in `tokimu-platform` (`let_unit_value`) and `tokimu-render`
(`Arc` over non-`Send`/`Sync` browser WebGPU resources). The new browser crate
itself compiles under the WASM target; this campaign does not suppress or
charge those existing core target-lint findings to the source fixtures.

### Deliverables

- Browser fixture host and retained manifest comparison.
- Native/browser presentation observations.

### Acceptance Criteria

- Structural fingerprints agree for equivalent fixture inputs.
- Boundary transitions may differ in timing cost but not in semantic reason or
  required-surface survival.
- Browser first-frame presentation is observed for the applicable fixtures.
- Target-specific presentation mechanisms do not change Doom source meaning.

## Slice 7 — E1M1 Escalation Gate

### Targeted classic-source checkpoint

The 2026-08-14 E1M1 reruns falsified both ordinary world-depth variants of
sky authority: an unconditional paired-sky vertical boundary clipped valid
hut geometry, while an exact viewer-relative sky cell placed at its owning
source ceiling still allowed unrelated static-shell geometry with a nearer
Euclidean depth to survive. A targeted reread of the released renderer and
the historically faithful Chocolate Doom continuation explains why neither
variant matches the source presentation model:

- `R_RenderSegLoop` derives ceiling and floor visplane cells from the current
  per-column `ceilingclip` and `floorclip` bounds while visiting admitted wall
  ranges;
- one-sided middle walls terminate both clip bounds, while two-sided upper and
  lower tiers update the corresponding bound independently;
- the paired-sky height rule changes `worldtop` before those bounds and plane
  marks are derived; it does not create an invisible world-space wall; and
- `R_DrawPlanes` paints a sky texture only into the retained sky-visplane
  column intervals. It does not submit a source-height sky surface to compete
  with an already submitted global shell in a depth buffer.

The durable source invariant is therefore viewer-relative, ordered screen
coverage shared by wall and plane preparation. The historic arrays and column
loops are reference mechanisms, not Tokimu contracts. The next E1M1 candidate
must prevent source-unreachable geometry from entering terminal sky intervals
or present the authoritative source spans directly; another world-space sky
depth mesh is not an authorized refinement.

**Falsified:** world-space sky geometry or hidden sky depth surfaces as the
authoritative solution to Doom sky occlusion. Classic Doom sky presentation is
downstream of viewer-relative ordered wall/plane coverage; paired-sky handling
modifies that source clipping process rather than introducing an invisible
world-space occluder.

- [x] Test whether Boolean filtering of whole source contributions is expressive
      enough before choosing the next realization. The
      `partial-paired-sky-far-control` makes one far source SEG wider than a
      nearer paired-sky interval. Its shared provider observation retains both
      overlapping columns and far-only columns for the same far source SEG.
      The headless result is `81` paired-sky columns and `97` far-wall columns:
      overlap `[120,200]` is one `81`-column run, while required survivors
      `[112,119]` and `[201,208]` are two eight-column runs. Keeping that SEG
      whole cannot encode the excluded overlap; rejecting it loses the two
      required survivor runs. Whole-SEG candidate selection is therefore
      falsified at this boundary. A subsequent experiment must retain
      source-derived fragments/intervals, while leaving their realization as
      ordinary Doom-consumer meshes or draws unresolved. These diagnostic runs
      are corpus evidence, not renderer scissors or a public pixel-span API.

- [x] Define which synthetic fixtures guard each canonical E1M1 observation in
      [`Doom synthetic-to-E1M1 coverage matrix.md`](../Evidence/Doom%20synthetic-to-E1M1%20coverage%20matrix.md).
      All currently named native/browser synthetic target controls are now
      green. The matrix remains partial where its named E1M1 composition
      falsifier has not yet survived; green synthetic evidence permits a
      labelled experiment, never a normal replacement mode.
- [x] Prevent E1M1 candidate modes from running as proposed solutions when an
      applicable synthetic guard is red, except under an explicit negative
      control flag. Full submission remains the default executable mode.
      Every source-presentation candidate requires its own explicit
      `--doom-seg-*` experimental flag and reports its `candidate_selection`
      identity in first-frame metadata; no flag or UI control silently selects
      a candidate as the normal E1M1 presentation.
- [ ] Re-run the canonical E1M1 pose/path matrix only after all applicable
      guards pass.
- [ ] Retain source identities for any remaining E1M1-only failure and decide
      whether it requires a new synthetic fixture.
- [ ] Compare full submission and the candidate without accepting visual loss
      for lower draw counts.
- [x] Require the five frontier guards—presentation-instance identity,
      vertical partial occlusion, stationary dynamics, projection epsilon, and
      camera jitter—to pass before returning to the current hut/door/spawn
      failures. All five now have passing Level-1 tests and separate native
      and Browser WebGPU first/warm/jitter observations. This gate authorizes
      the labelled E1M1 falsification run that exposed the world-space sky
      model's failure; it does not authorize another E1M1 candidate by itself.
- [ ] Require Slice 4B's ordered wall/plane coverage trace, paired-sky
      differential, terminal-sky near/far control, and partial-fragment
      realization to pass before the next E1M1 presentation candidate runs.
      The previously green frontier guards remain necessary but are no longer
      sufficient after source inspection changed the causal model.

### Deliverables

- Synthetic-to-E1M1 coverage matrix.
- E1M1 native and browser integration evidence for the surviving candidate.

### Acceptance Criteria

- Every known E1M1 failure class has a preceding synthetic guard or a recorded
  reason it cannot be isolated.
- E1M1 remains the final falsifier.
- No candidate becomes the normal E1M1 mode solely because synthetic tests pass.

## Slice 8 — Disposition And Reuse Review

- [ ] Record which source-protocol concepts survived independent fixtures.
- [ ] Separate transferable invariants from Doom-specific mechanisms; do not
      promote one merely because the other proved necessary.
- [ ] Separate reusable Doom-provider helpers from corpus-only presentation
      machinery.
- [ ] Decide whether AR-0025 needs only appended evidence or a genuinely new
      architectural review due to independent non-Doom pressure.
- [ ] Update the WAD checklist and relevant evidence ledgers.
- [ ] Park rejected alternatives with exact reopening triggers.
- [ ] Record the strongest negative-generalization result: which visibility,
      topology, or presentation concepts Tokimu and its renderer should not
      own based on this campaign alone.
- [ ] Name future independent pressure, such as a Quake, CAD, or charted-space
      corpus, without adding that work to this Doom campaign.

### Deliverables

- Final test-campaign disposition.
- A responsibility map covering source truth, runtime semantic state,
  provider-owned preparation, Tokimu declarations/diagnostics, and renderer
  realization.
- Updated DOOM campaign dashboard and checklist links.

### Acceptance Criteria

- The campaign either advances one evidence-backed Doom presentation path to
  E1M1 or rejects all tested paths honestly.
- A successful result does not require a shared visibility capability; a
  justified provider-owned preparation boundary satisfies this plan.
- No public or shared Tokimu capability is admitted implicitly.
- Remaining gaps identify the smallest next experiment rather than a vague
  “visibility still broken” task.

## Validation Matrix

At minimum, implementation work should run:

```powershell
cargo fmt --all
cargo test -p doom-geometry-provider
cargo test -p hello-doom-e1m1
cargo test -p hello-doom-visibility-conformance
cargo clippy --workspace --all-targets -- -D warnings
```

The proposed package command is illustrative until Slice 1 chooses the final
crate/module shape. Native and browser fixture commands must be documented once
they exist.

## Failure And Escalation Rules

- A synthetic false negative stops presentation work before E1M1.
- A synthetic false positive is retained and measured; it may be acceptable if
  correctness is preserved.
- A small rendered failure returns to the corresponding headless semantic or
  lowering seam before E1M1.
- An E1M1-only failure adds a fixture only when its invariant can be isolated
  without copying E1M1 wholesale.
- Pressure to expose Doom source types to `tokimu-render`, mutate decoded WAD
  truth, reorder caller draws, or add a stable visibility contract is an
  architectural finding and stops implementation.
- Performance evidence is considered only after correctness. Smaller draw
  counts never compensate for visible omissions.

## Completion Criteria

This plan is complete when:

- the initial fixture matrix has headless semantic coverage;
- applicable fixtures have native and browser presentation evidence;
- the synthetic-to-E1M1 escalation gate is documented and exercised;
- E1M1 either validates a surviving candidate or supplies retained evidence
  rejecting it;
- all shared/public capability questions are explicitly deferred or sent to
  review; and
- the WAD checklist identifies the next meaningful Doom slice without relying
  on repeated manual discovery of already-isolated failures.

## Parking Criteria

The campaign may park before completion when:

- exact Doom screen-span reconstruction is deliberately out of current scope;
- all current candidates are rejected and no smaller experiment is justified;
  or
- browser execution is unavailable after native semantics are retained.
- the remaining discrepancy requires historical fixed-point/pixel-raster
  replication with no demonstrated Tokimu architectural payoff.

A parked plan must name the failed fixture, retained evidence, and reopening
trigger. It must not mark E1M1 presentation complete.

## Related Records

- [DOOM WAD Checklist](../DOOM%20WAD%20Checklist.md)
- [E1M1 Hut Sky-Boundary Evidence](../Evidence/E1M1%20hut%20sky-boundary%20evidence.md)
- [E1M1 Camera Candidate-Selection Evidence](../Evidence/E1M1%20camera%20candidate-selection%20evidence.md)
- [Classic Doom Visibility Clipping Evidence](../Evidence/Classic%20Doom%20visibility%20clipping%20evidence.md)
- [Lesson: Read Available Reference Source Earlier](../../../lessions/read-reference-source-early.md)
- [AR-0025 Camera Candidate Selection And Visibility Culling](../../../Architectural%20Reviews/AR-0025-camera-candidate-selection-and-visibility-culling.md)
- [AR-0023 Textured Surface Alpha And Depth Policy](../../../Architectural%20Reviews/AR-0023-textured-surface-alpha-and-depth-policy.md)
- [AR-0030 Tokimu Render Preparation And Submission Framework](../../../Architectural%20Reviews/AR-0030-source-owned-presentation-preparation-boundary.md)

This campaign is AR-0030's Doom admission pressure, not authority to choose the
Tokimu render framework by itself. The eventual strategy must also survive the
AR's Quake, ordinary retained-3D, and large or multi-view campaign gates.
