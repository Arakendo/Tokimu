# Doom Authoritative Sky-Coverage Delta Realization

| Field | Value |
| --- | --- |
| Campaign | DOOM |
| Role | Focused exceptional-presentation realization study |
| Status | Candidate 1 composition falsified in E1M1; G2 transport remains proven; relational child study active |
| Parent review | [AR-0030](../../../Architectural%20Reviews/AR-0030-source-owned-presentation-preparation-boundary.md) |
| Controlling plan | [DOOM WAD Checklist](../DOOM%20WAD%20Checklist.md) |
| Source oracle | [Doom Ordered Source-Occurrence Preparation](Doom%20ordered%20source%20occurrence%20preparation.md) |
| Renderer dataflow | [Classic Doom Renderer Dataflow And Tokimu Preparation Seam](../Evidence/Classic%20Doom%20renderer%20dataflow%20and%20Tokimu%20preparation%20seam.md) |
| Slice 1 evidence | [Doom Authoritative Sky-Region Headless Evidence](../Evidence/Doom%20authoritative%20sky-region%20headless%20evidence.md) |
| Slice 2 gate evidence | [Doom Authoritative Sky-Depth Realization Seam Evidence](../Evidence/Doom%20authoritative%20sky-depth%20realization%20seam%20evidence.md) |
| G2 lifetime evidence | [Doom AR-0030 G2 Submission-Local Geometry Evidence](../Evidence/Doom%20AR-0030%20G2%20submission-local%20geometry%20evidence.md) |
| Successor falsifier | [Source-Authorized Relational Contribution Classification](Doom%20source-authorized%20relational%20contribution%20classification.md) |
| Initial corpus target | Canonical E1M1 source-spawn window/hut view and retained distant-sector sky-leak views |
| Generic post-filter | Disabled until the authoritative-sky candidate is independently correct |
| Stable API authority | None; a renderer-contract proposal returns to AR-0030 before admission |

## Purpose

Test the narrow hypothesis that most E1M1 presentation can remain complete,
persistent ordinary geometry rendered with normal GPU depth, while Doom-private
viewer-relative preparation supplies only the exceptional semantic delta that
ordinary depth cannot reproduce: authoritative sky coverage.

The study does **not** reinterpret the healthy ordered Doom ledger as global
world geometry. It uses that ledger as the authority for bounded sky regions
and compares two possible private realizations of the same result.

```text
complete ordinary Doom geometry
        -> ordinary Tokimu draws and depth

Doom source + runtime snapshot + prepared view
        -> healthy ordered Doom ledger
        -> authoritative retained sky regions
        -> experimental exceptional presentation

combined result
        -> tokimu-render presentation
```

The principal question is whether Tokimu already has enough ordinary mechanism
to realize this exceptional coverage, or whether AR-0030 has discovered
pressure for a provider-neutral view-local rendering primitive.

## Ownership Boundary

| Concern | Owner during this study |
| --- | --- |
| `F_SKY1`, sector, SEG, visplane and paired-sky interpretation | Doom provider/corpus |
| Which bounded view-relative regions have authoritative sky meaning | Doom ordered preparation |
| Source provenance, source depth and failure evidence | Doom provider/corpus |
| Persistent sky texture, sampler and pipeline resources | Existing application/renderer boundary |
| Realizing caller-declared view-local coverage and depth | Experimental renderer mechanism under AR-0030 |
| Generic camera/frustum filtering | Disabled downstream experiment |
| Admission of stable render vocabulary | AR-0030 plus independent campaign evidence |

`tokimu-render` must not learn Doom sectors, SEGs, visplanes, sky names, BSP
rules or source clipping policy. Doom owns why and where coverage exists. A
candidate Tokimu mechanism may only know the provider-neutral declaration it is
asked to realize.

## Evidence Carried Forward

The ordered Doom protocol is no longer the leading suspect. At canonical E1M1
source spawn the corrected headless ledger retains:

- 37 admitted SEGs;
- 9 resolved plane instances;
- 17 horizontal spans;
- 1,205 populated columns and 50,679 populated cells;
- zero overlapping writes;
- zero unresolved plane instances.

Two broader realizations then fail in opposite ways:

- fixed-view inverse-projected cells retain raster-shaped coverage but do not
  survive free look;
- continuous occurrence wedges support free look but lose required walls,
  planes and shared boundaries.

Global full submission remains substantially correct under ordinary depth, but
source-invalid distant sectors can appear where Doom presents sky instead of an
ordinary enclosing surface. Earlier world-space sky walls and height-derived
sky tiles were falsified because they assigned coarse world geometry authority
that clipped valid nearby geometry, including the hut.

## Working Hypothesis

For ordinary opaque E1M1 geometry:

```text
near ordinary geometry   -> writes depth -> survives normally
far ordinary geometry    -> loses to nearer ordinary geometry
```

For authoritative Doom sky:

```text
near source-authorized geometry
        depth before the declared sky boundary
        -> remains visible

authoritative sky coverage
        bounded view-local region at a declared relationship
        -> presents sky

unrelated farther geometry
        behind that authority
        -> cannot leak through
```

The first candidate asks whether this can be expressed as continuous,
depth-bearing view-local geometry. Only a demonstrated representational
failure permits investigation of a stronger coverage/compositing mechanism.

## Controls And Candidate Names

Use these names consistently in titles, logs and evidence:

- `global-full-submission` -- unchanged complete E1M1 geometry using ordinary
  renderer depth; known distant-sector sky leak control.
- `global-full-plus-view-local-sky-depth` -- Candidate 1; complete ordinary
  geometry plus Doom-authoritative, continuous view-local depth-bearing sky
  coverage.
- `global-full-plus-bounded-sky-composite` -- Candidate 2; authorized only if
  Candidate 1 is structurally insufficient and AR-0030 records the reason.
- `prepared-frustum-filtered` -- later downstream experiment; prohibited until
  the selected candidate is independently clean.

“Full submission” without one of these qualifiers is not acceptable evidence.

## Binding Invariants

1. The complete ordinary geometry input is identical between the control and
   both candidates.
2. Doom source facts and explicit current runtime state determine authoritative
   sky coverage; renderer heuristics do not infer it from textures or alpha.
3. Sky coverage is view-local and bounded. It never becomes a giant sector or
   world-space invisible occluder.
4. Candidate geometry is derived from continuous/projective ledger boundaries,
   not reconstructed from one quad per `320 x 200` diagnostic cell.
5. Nearer valid geometry survives; farther source-invalid geometry does not.
6. Ordinary walls, floors, ceilings, doors and cutouts are not filtered or
   reconstructed merely to make sky work.
7. The diagnostic raster remains an oracle, never public renderer vocabulary.
8. Persistent materials/textures remain distinct from ephemeral prepared-view
   work. Per-frame view-local work must not masquerade as persistent mesh
   replacement.
9. Missing, ambiguous or invalid source authority fails open with bounded
   evidence. It must not silently hide ordinary geometry.
10. No AABB/frustum pass may repair or participate in the correctness result.

## Canonical Observation Matrix

Every visual candidate must retain the same named observations:

| Observation | Required result |
| --- | --- |
| Source-spawn room continuity | Equal to the global-full control |
| Window view toward hut | Hut remains visible |
| Distant-sector sky leak | Removed only inside authoritative sky coverage |
| Near wall movement | No vanishing surfaces or sudden coverage inversion |
| Free look | Continuous; no fixed view box appears |
| Small yaw/position jitter | No cracks, popping or raster-grid edge |
| Nearby sky ceiling | Does not expose an unrelated room |
| Paired-sky boundary | Preserves the source-authorized nearby contribution |
| Doors/platforms | Current geometry remains driven by runtime snapshots |
| Cutout middle | Transparent texels do not become solid occlusion authority |

Visual observations support, but do not replace, structural and source-ledger
evidence.

## Slice 0 — Freeze Controls And Terminology

- [x] Retain the exact E1M1 package fingerprint, source-spawn pose, hut/window
      pose and known distant-sector leak poses.
- [x] Add explicit launch/report names for `global-full-submission` and the two
      candidate modes; remove ambiguous `FullSubmission` wording from new
      evidence.
- [x] Retain global-full structural counts and native fixed-camera observations
      without changing its geometry.
- [ ] Record which source-ledger sky intervals correspond to each canonical
      visual anomaly.
- [x] Confirm generic AABB/frustum selection is disabled in all Slice 0–5
      comparisons.

### Slice 0 acceptance

- [x] Every retained Candidate 1 observation names an unambiguous pipeline and
      canonical pose. New poses must use the same convention.
- [x] The ordinary full-geometry control remains unchanged.

The retained package fingerprint is
`58146f5aa0e14ef38047a79878307344aec821b9f312da6a9208ec08e399660c`.
The source-spawn control is `(1056, -3616)`, angle `90` degrees. The first
positive authoritative-sky realization pose is the retained
`exterior-hut-east` observation at `(2076, -3560)`, heading `-25.1` degrees.
Its camera is now selected independently with `--exterior-hut-east-view`;
`--candidate1-sky-authority-view` remains only as a temporary compatibility
alias. Because this fixed diagnostic pose does not claim player-sector state,
it requires `--no-walk-collision`.

The direct native A/B commands are:

```text
cargo run -q -p hello-doom-e1m1 --bin static_scene -- \
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD \
  --exterior-hut-east-view --no-walk-collision --measure-two-frames

cargo run -q -p hello-doom-e1m1 --bin static_scene -- \
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD \
  --global-full-plus-view-local-sky-depth \
  --exterior-hut-east-view --no-walk-collision --measure-two-frames
```

## Slice 1 — Headless Authoritative Sky-Region Model

- [x] Extract a Doom-private `AuthoritativeSkyRegion` experimental model from
      the corrected ordered ledger without placing it in a stable Tokimu crate.
- [x] Retain source plane instance, source SEG/boundary provenance, prepared
      view identity and current runtime-snapshot identity.
- [x] Retain continuous/projective horizontal and vertical boundaries plus the
      source depth/order relationship needed by realization.
- [x] Demonstrate that the model is not defined as diagnostic raster cells,
      although raster cells may validate it as an oracle.
- [x] Prove paired-sky, one-sky negative, nearby valid geometry and ambiguous
      authority cases headlessly.
- [x] Add conservation evidence from ledger sky intervals to modeled regions,
      including explicit omission/rejection reasons.

### Slice 1 acceptance

- [x] Every authoritative ledger sky interval has one explained model outcome.
- [x] No non-sky ordinary contribution is removed by this model.
- [x] Invalid or uncertain authority fails open with retained provenance.

## Slice 2 — Candidate 1: Continuous View-Local Depth Surface

Candidate 1 is intentionally smaller than a compositing facility. It gives
ordinary depth testing one final, properly authorized chance.

- [x] Define the smallest corpus-private declaration needed to express
      ephemeral view/clip-local geometry with a declared depth relationship.
- [x] Before changing a stable renderer API, prove whether the declaration can
      be realized using existing private/internal renderer machinery. If not,
      stop and record the minimal experimental seam for AR-0030.
- [x] Generate bounded continuous triangles from authoritative region
      boundaries, not from individual diagnostic cells.
- [x] Reference persistent sky material resources without allocating or
      replacing persistent mesh identity every frame.
- [x] Submit unchanged persistent control geometry plus the view-local sky
      surface through the authorized unstable G2 renderer intake on native
      WGPU. The local payload receives no `MeshHandle`; persistent controls
      retain their existing handle-backed path.
- [x] Retain headless counts for regions, vertices, triangles, rejection
      outcomes and persistent mesh identities.
- [x] Retain native and Browser WebGPU repeated-submission, persistent
      upload/replacement and rejection/recovery counts. The initial browser
      visual used equal-sized persistent controls and hid blue behind orange;
      maintainer observation confirms the corrected nested blue-outer and
      orange-inner controls on Browser WebGPU.
- [x] Add invalid-depth, near-plane and empty-region rejection controls with
      bounded diagnostics.

### Slice 2 gate finding

The corpus-private declaration reduces 66 retained sky intervals and 2,046
diagnostic oracle cells to two continuous declarations containing 12 vertices
and four triangles, with no diagnostic-grid identity and no persistent mesh
identity. Invalid, near-plane, empty, paired-only, one-sky and ordinary-aperture
controls fail open.

The renderer seam audit found no existing private path that can submit these
changing declarations without allocating or replacing a persistent
`MeshHandle`. That conflicts with Binding Invariant 8 and triggers the plan's
stable-contract and lifetime-separation stop conditions. The detailed evidence
is retained in [Doom Authoritative Sky-Depth Realization Seam Evidence](../Evidence/Doom%20authoritative%20sky-depth%20realization%20seam%20evidence.md).

AR-0030 subsequently authorized a private G1--G4 comparison and then the
restricted corpus-only G2 renderer seam without admitting a stable API. The
G2 model places the same two declarations in one
bounded immutable submission: two local payload identities, two ordered draws,
12 vertices, four triangles, one persistent material key, and zero persistent
mesh identities. Local identities cannot resolve across submissions, and
malformed, over-capacity, foreign, missing, or orphaned payloads fail before a
snapshot exists. Native WGPU submissions 41--43 reuse local slots without
changing the two persistent mesh resources; an invalid missing-material batch
is rejected atomically before submission 43 succeeds. Submission 42 changes
the source-view X coordinate by eight units and produces a distinct
geometry-only fingerprint; submission 43 restores the baseline view and the
exact submission-41 fingerprint without persistent upload or replacement
churn. The draw order is background/sky colour, submission-local authority
depth, then the persistent far-wall control, so later depth-only work is not
mistaken for colour erasure. Browser execution now preserves the same counts;
maintainer observation confirms the corrected blue-outer/orange-inner visual
control. This closes the cross-target control-parity gate without claiming
pixel-identical output.

The fixture was then extended for the complete Slice 2 depth relationship with
a third persistent green near-object control. Native submissions 41--43 retain
five draws, three persistent uploads, zero replacements, and the same G2
identity/jitter/recovery facts. Actual Browser WebGPU execution reports the
same five-draw submissions, three persistent uploads and zero replacements.
Maintainer observation confirms the intended relationship: green near geometry
survives, the submission-local authority suppresses farther orange geometry in
its declared region, and blue sky/background remains elsewhere. This is a
semantic cross-target observation, not a pixel-identity claim.

### Slice 2 acceptance

- [x] Near geometry wins, sky presents at its declared boundary, and farther
      geometry loses within the same synthetic fixture.
- [x] Camera jitter changes ephemeral prepared work without persistent mesh
      replacement churn.
- [x] No `320 x 200` grid appears in declaration identity or geometry count.

## Slice 3 — Candidate 1 Synthetic Presentation Matrix

- [x] Extend the paired-sky fixture with a near valid object, authoritative sky
      boundary and farther invalid object in one depth relationship.
- [x] Retain the one-sky negative control: no source authority means no sky
      depth surface.
- [x] Retain vertical partial-aperture and close-wall controls.
- [x] Retain dynamic-door/platform snapshot controls without recreating their
      timing policy.
- [x] Retain cutout non-occluder behavior.
- [x] Run headless structural, native visual, Browser WebGPU and camera-jitter
      observations.
- [x] Preserve semantic comparison claims; do not require pixel-identical
      native/browser output.

The combined headless matrix is retained in
[Doom Candidate 1 Synthetic Presentation Matrix Evidence](../Evidence/Doom%20Candidate%201%20synthetic%20presentation%20matrix%20evidence.md).
It balances all fourteen ordered-reference cases and ten synthetic controls.
The paired-sky positive produces two G2 declarations/four triangles from 66
retained intervals and 2,046 oracle cells. Both negative-authority cases
produce zero declarations and deliberately skip the G2 authority batch rather
than manufacturing an empty submission. All ordinary fixture contributions
remain on their established presentation paths; Candidate 1 neither filters
nor reconstructs them.

### Slice 3 acceptance

- [x] All synthetic fixtures satisfy the binding invariants on both targets.
- [x] Candidate 1 has no unexplained missing or extra contribution.

## Slice 4 — Candidate 1 E1M1 Falsification

- [ ] Run `global-full-submission` and
      `global-full-plus-view-local-sky-depth` at every canonical pose.
- [x] Prove unchanged ordinary geometry counts between the two modes at the
      first positive `exterior-hut-east` pose.
- [x] Retain source-correlated sky-region and boundary-depth evidence for every
      additional candidate declaration.
- [ ] Confirm the hut survives from source spawn and nearby movement.
- [ ] Confirm the known distant-sector leaks disappear only in authoritative
      sky regions.
- [ ] Walk/free-look the exterior, spawn room, first door and known sky-ceiling
      cases without a fixed view window, wall disappearance or seam cracks.
- [ ] Retain native fixed-camera captures and browser/WASM structural and visual
      observations.

The first positive native A/B comparison now conserves the complete ordinary
input exactly: both modes retain `1,922` source contributions, aggregate hash
`30650e57ad9b3c07`, `1,823` opaque draws and the same 14 admitted cutout draws
(12 non-owning-side cutout candidates remain rejected in both controls).
Candidate 1 adds only six submission-local authority declarations containing
264 vertices and 88 triangles. They create six additional draws, no persistent
mesh identity and no persistent mesh replacement. The source model reports six
regions, all realized; ambiguity therefore does not take the fail-open branch.

The declaration fingerprint printed by E1M1 is submission-scoped and is
expected to change between first and warm submissions even when geometry is
unchanged. The G2 conformance fixture separately retains its geometry-only
fingerprint to prove baseline/jitter/baseline recurrence.

The canonical exterior observation then falsified Candidate 1 as a composition
strategy. It clips valid far-left building geometry diagonally, masks valid
outside/hut-adjacent geometry and still permits distant rooms to leak beside
the hut and above the wall. A bounded oracle comparison rules out ordinary
region-to-triangle loss as the cause:

```text
modeled ledger column centers:       320
coverage mismatches:                   0
extra/missing cells:                 0/0
unresolved depth samples:              0
maximum clip-depth error:    0.000000050
mean clip-depth error:       0.000000017
```

Candidate 1 therefore realizes the extracted ledger subset faithfully but
encodes that authority incorrectly as an independent depth surface over an
unchanged global shell. Further triangle tessellation, biasing or manual visual
clipping is prohibited. The child relational-classification study tests
whether authority can instead classify competing source contributions before
ordinary renderer submission.

### Slice 4 acceptance

- [ ] Candidate 1 passes the canonical observation matrix without changing the
      complete ordinary geometry input.
- [x] Any failure is classified as source-authority, representation,
      realization, provider, or observation failure before repair.

## Slice 5 — Candidate 1 Economics And Disposition

- [x] Compare first/warm frame CPU time, draw count, pipeline changes, binding
      allocations, ephemeral payload size and persistent-resource churn against
      global full submission.
- [ ] Test bounded camera motion long enough to expose recurring allocation or
      replacement behavior.
- [x] Record whether the mechanism remains Doom-private or exposes repeated
      pressure for provider-neutral view-local work.
- [x] Record the exact invariant, if any, that depth-bearing view-local geometry
      cannot express.

### Candidate 1 success disposition

If Candidate 1 passes, retain it as Doom corpus/provider mechanism and return
its evidence to AR-0030. Do not automatically admit a stable primitive. Resume
the non-sky falsifier ladder and only then test optional generic filtering.

### Candidate 1 failure disposition

Candidate 1 may advance to Candidate 2 only when evidence states an invariant
it fundamentally cannot represent. “Candidate 2 is easier” is not sufficient.

The E1M1 result does not yet authorize Candidate 2. It first authorizes the
smaller Doom-private relational falsifier: test whether ordered source
authority can classify competing contributions as nearer, beyond, straddling
or unresolved before renderer submission. Candidate 2 is earned only if that
bounded relation requires ordered overlapping coverage/composition that the
classifier cannot express.

## Slice 6 — Candidate 2 Gate: Bounded Coverage/Composition

This slice is parked until Candidate 1 fails structurally and AR-0030 records
the escalation.

- [ ] State the unrepresentable Candidate 1 invariant in provider-neutral terms.
- [ ] Distinguish required coverage, depth, ordering and composition semantics
      from a general render graph or arbitrary pass API.
- [ ] Propose the smallest corpus-private bounded coverage/compositing
      experiment.
- [ ] Re-run the complete Slice 3–5 matrix.
- [ ] Reject any design that embeds Doom source vocabulary or grants the
      renderer authority to infer source visibility.

### Slice 6 acceptance

- [ ] AR-0030 explicitly authorizes the experimental direction before code
      broadens a renderer boundary.
- [ ] Candidate 2 demonstrates an invariant Candidate 1 could not express.

## Slice 7 — Sequential Non-Sky Falsifiers

Only begin after a sky candidate is clean. These cases test whether the Doom
compatibility intervention is actually narrow.

- [ ] Opaque closed geometry: ordinary full submission and depth remain
      sufficient.
- [ ] Two-sided openings and absent tiers: determine whether ordinary geometry
      plus depth remains sufficient.
- [ ] Dynamic doors/platforms: consume explicit current runtime heights.
- [ ] Masked middles: retain caller-declared cutout behavior and non-occluder
      evidence.
- [ ] Partial vertical apertures: run the strongest non-sky falsifier.
- [ ] Add another Doom-private exceptional class only after a fixture proves
      ordinary rendering insufficient.

## Slice 8 — Optional Generic Post-Filter

This slice remains prohibited until the selected prepared presentation is
independently correct.

- [ ] Apply conservative AABB/frustum filtering after the complete ordinary
      geometry plus exceptional-presentation declarations are prepared.
- [ ] Preserve declaration order and all exceptional coverage authority.
- [ ] Prove disabling the filter restores exactly the accepted unfiltered
      candidate.
- [ ] Treat any correctness change as a generic-filter failure, not as a repair
      opportunity.

## Validation Matrix

Run proportionally at each completed slice:

```text
cargo fmt --all -- --check
cargo test -p doom-geometry-provider
cargo test -p hello-doom-visibility-conformance
cargo clippy -p doom-geometry-provider \
  -p hello-doom-visibility-conformance \
  -p hello-doom-e1m1 --all-targets -- -D warnings
```

Retain exact native and browser commands in the evidence generated by the
relevant slice.

## Architectural Stop Conditions

Return to AR-0030 before continuing when:

- Candidate 1 requires a stable/public renderer contract change;
- view-local work cannot be bounded without a renderer-owned scene protocol;
- correct realization requires renderer knowledge of Doom source semantics;
- Candidate 2 begins to resemble a general render graph, pass or mask system;
- persistent resource identity and ephemeral prepared-view work cannot be kept
  separate;
- native/browser behavior diverges materially;
- the sky-only hypothesis is falsified by a non-sky case before its own gate;
- or generic filtering becomes necessary to make the candidate look correct.

## Completion Criteria

This study completes when it can state, with retained evidence:

1. whether global ordinary geometry plus an exceptional Doom sky delta is
   sufficient for canonical E1M1;
2. whether continuous view-local depth-bearing geometry can realize that delta;
3. if not, the precise invariant requiring stronger bounded composition;
4. whether the mechanism remains Doom-private or creates credible pressure for
   provider-neutral Tokimu render vocabulary;
5. the cost and lifecycle of ephemeral prepared-view work;
6. and whether generic AABB/frustum filtering can safely follow correctness.

No result admits portals, mirrors, CAD section views or another hypothetical
consumer as evidence. Those remain future independent falsifiers. Quake remains
dormant unless AR-0030 later requires an independent source-driven campaign.
