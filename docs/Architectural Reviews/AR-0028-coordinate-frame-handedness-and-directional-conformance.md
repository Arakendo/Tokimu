# AR-0028: Coordinate-Frame Handedness And Directional Conformance

| Field | Value |
| --- | --- |
| Status | No Change — Doom-local orientation repair retained |
| Opened | 2026-08-10 |
| Last reviewed | 2026-08-13 |
| Scope | Cross-cutting coordinate, camera, geometry, input, and source-provider conformance |
| Trigger | The E1M1 corpus exposed reversed right/front wall art, A/D strafe, and mouse yaw while moving Doom coordinates into Tokimu's first-person presentation. |
| Related ADRs | ADR-0003, ADR-0008, ADR-0009, ADR-0012, ADR-0013 |
| Related reviews | AR-0019, AR-0021, AR-0022, AR-0024, AR-0025, AR-0026 |
| Admission exception | None |

## Architectural Question

What explicit coordinate-frame, handedness, and directional conventions must
Tokimu own—and what source/provider adapters must declare—so that geometry
facing, camera yaw, movement axes, and supplied texture coordinates remain
coherent across native and WASM without baking Doom conventions into the
renderer or core math vocabulary?

## Context

Tokimu already has focused evidence for geometry facing (AR-0021), supplied
mesh UVs (ADR-0012 / AR-0022), and WebGPU clip-depth adaptation (AR-0024).
Those records intentionally did not establish one project-wide spatial-frame
contract.

The E1M1 observer now joins three directional observations that initially
appeared unrelated:

```text
source Doom 2D coordinates -> Tokimu world X/Z lift
right/front wall -> horizontally mirrored EXITSIGN
first-person observer -> A/D strafe direction felt reversed
captured mouse delta -> yaw direction felt reversed
```

The individual repairs are useful corpus evidence, but they are not yet proof
that Tokimu's global assumptions are complete or mutually consistent. In
particular, a local texture fix must not become a disguised renderer UV rule,
and an input-sign change must not redefine a stable camera convention without
cross-target and independent-consumer evidence.

The similar visual symptom does not imply common ownership. The review must
keep the following meanings separate even when each can manifest as “backwards”:

```text
Doom texture direction     -> source conversion
Doom player heading        -> source conversion
normalized movement/input  -> interaction convention
camera basis and yaw       -> candidate shared semantics
mesh winding and UVs       -> already bounded caller contracts
WGPU clip depth            -> provider adaptation
```

## Trigger And Retained Evidence

- The E1M1 native observer initially displayed canonical `EXITSIGN` source art
  horizontally mirrored. Canonical-package preflight identifies all eight
  submitted `EXITSIGN` triangles as `Right`/front-sided upper-wall records
  (linedefs 342–350), disproving the first left/back-only hypothesis.
- The repaired Doom source lowerer makes right/front texture U decrease along
  the stored linedef and left/back texture U increase. This is a provisional
  source-to-world mapping; `tokimu-render` still passes caller-supplied UVs
  unchanged.
- A focused provider regression covers both right/front and left/back endpoint
  U directions; the native observer now reads the exit sign correctly.
- The same observer exposed reversed local strafe and captured mouse-yaw signs.
  Its input handling was repaired at the corpus/platform boundary, including
  raw captured `DeviceEvent::MouseMotion` forwarding.
- AR-0021 independently established that native and browser/WASM agree on the
  renderer's front/back-facing fixture. AR-0022 independently established
  supplied UV stream and sampler behavior with non-Doom Box/PNG evidence.
- No current evidence establishes a stable, public world-coordinate, camera
  basis, or input-axis contract for every Tokimu consumer.
- A raw `Vec3` or `Mat4` can carry the required numbers but cannot identify
  whether they are a source position, world direction, object-to-world map,
  world-to-view map, chart transition, projection, or provider adaptation.
  This is evidence for AR-0019's distinction between semantic spatial
  vocabulary and ordinary math vocabulary, not evidence that those math types
  themselves are defective.

## Ownership Analysis

- **Tokimu math** owns ordinary vector, matrix, transform, and projection
  mechanics, but not a source-format's compass, texture orientation, or
  editor-axis vocabulary.
- **Tokimu camera/input semantics**, if admitted, should own a named local
  basis and pointer-delta meaning independent of WGPU, native windowing, or a
  particular map format.
- **Doom geometry and presentation providers** own the Doom 2D-to-3D lift,
  Doom sidedef direction, source texture axes, player-angle conversion, and
  any source-specific vertical convention.
- **Renderer adapters** own only their documented clip/depth conversion and
  realization of caller-provided positions, normals, and UVs. They must not
  infer Doom right/front orientation or flip texture U.
- This review must not make `tokimu-core` own window pointer behavior, WAD
  terms, GPU coordinate conventions, or application/player state.

Input evidence must also distinguish four authorities:

```text
physical mechanism     mouse moved right
normalized observation pointer_delta_x > 0
interaction policy     positive delta requests a chosen look/orbit direction
camera convention      positive yaw rotates around a declared axis/direction
```

Native mouse capture, browser pointer lock, touch drag, gamepad look, and
editor orbit controls may legitimately map mechanisms to intent differently.
AR-0028 must not promote one native mouse gesture into a platform-wide camera
contract by accident.

## Dependency Direction

```text
Current evidence path:

Doom map/source direction + native pointer events
        -> corpus provider/application conversions
        -> ordinary Tokimu meshes, camera, normalized input
        -> WGPU adapter realization

Candidate clarified path:

source or platform convention
        -> explicit adapter conversion and retained declaration
        -> Tokimu-owned world/camera/input convention (only if earned)
        -> renderer provider-specific clip/depth adaptation
```

Neither path permits `tokimu-render` to import source-map types or lets a
window provider silently choose world-space meaning.

## Alternatives Considered

### Alternative A: Corpus-Local Conformance Matrix First

Retain the current E1M1 repairs as source/application behavior and build a
small, cross-target conformance matrix before changing a public Tokimu spatial
contract.

- Benefits: separates actual shared convention pressure from Doom and native
  observer behavior; lowest compatibility risk.
- Costs: temporary corpus-local conversions remain visible.
- Failure mode: duplicated local conventions can drift if the matrix and
  retained declarations are not completed promptly.

### Alternative B: Admit A Named Tokimu Spatial And Camera Basis Now

Define one public world handedness, forward/right/up basis, yaw sign, and
pointer-delta convention immediately, then migrate source adapters to it.

- Benefits: a simple long-term vocabulary and a direct basis for editors,
  simulation, CAD, portals, and non-Euclidean chart studies.
- Costs: broad migration across camera, picking, input, imported geometry,
  existing corpus fixtures, and potentially CPU-side projection logic.
- Failure mode: a convention chosen from one first-person Doom observer leaks
  into unrelated applications and must later be unwound.

### Alternative C: Provider-Owned Conventions Without A Shared Contract

Let each source/platform/provider select and document its own orientation,
with conversions only where individual consumers need them.

- Benefits: avoids premature Native admission.
- Costs: makes cross-provider composition, generic camera controls, editor
  tooling, and future spatial charts harder to reason about.
- Failure mode: implicit sign conversions proliferate and bugs reappear as
  “almost correct” presentation defects.

### Alternative D: Renderer-Normalized Texture Or Camera Directions

Have `tokimu-render` or `tokimu-platform` globally flip U, yaw, or movement
directions to make the current E1M1 presentation look correct.

- Benefits: smallest apparent code change.
- Costs: violates caller-supplied UV ownership and provider-neutral input
  normalization; conflicts with AR-0021/AR-0022 evidence.
- Failure mode: fixes Doom while mirroring Box/PNG, CAD, editor, or WASM
  callers. Reject unless new evidence overturns existing ADRs.

## Initial Conformance Plan

1. Record a compact frame ledger for each corpus fixture: source axes,
   world axes, handedness, forward/up/right vectors, winding/front-face,
   texture U/V direction, pointer delta sign, and movement directions.
   Preserve unknown values as `?` rather than filling them from intuition.
2. Extend the AR-0021 orientation fixture with labeled world basis and
   camera-facing observations; repeat native and browser/WASM.
3. Add a non-Doom first-person or orbit-control fixture using the same
   normalized input events, without source-map conversion.
4. Retain a Doom directional fixture covering front/right and back/left wall
   texture text or asymmetric markers, player-heading conversion, strafe, and
   mouse yaw. Keep it source-provider local.
5. For coordinate and direction conversions that are intended to preserve
   information, prove a bounded round trip (`source -> Tokimu -> source`) for
   points, directions, and orientation. Record explicitly where reversibility
   does not apply, such as lossy normalized device observations.
6. Compare the results against Box/PNG supplied-UV evidence and the existing
   WebGPU clip-depth adapter. Do not introduce a public global convention until
   at least one independent non-Doom caller needs it.
7. If the matrix reveals a contradiction between accepted contracts, stop and
   revise the affected ADR rather than compensating in an adapter.

The asymmetric fixture should be intentionally difficult to misread: labeled
front/back/left/right/up directions plus distinct `U+`, `U-`, `V+`, and `V-`
texture markers. Cubes, arrows, and checkerboards that remain plausible after
reflection are inadequate evidence for this review.

## Acceptance Clamp

No shared coordinate/camera admission is justified unless evidence shows:

- a named convention is coherent across geometry, camera, input, and texture
  axes, rather than merely making one screenshot look correct;
- native and browser/WASM preserve the relevant observable directions;
- at least one independent non-Doom consumer benefits from the same convention;
- source/provider conversions remain explicit, tested, and reversible;
- input evidence names physical mechanism, normalized observation,
  interaction policy, and camera convention separately;
- renderer UV and facing contracts remain caller-supplied/provider-neutral;
- CPU projection, picking, and any AR-0026 chart work are considered before
  changing stable math or world vocabulary; and
- ADR-0008/0009 evidence establishes performance, diagnostics, failure
  containment, and migration behavior for any Native Ring change.

## Current Disposition

**No Change; Alternative A completed the investigation without earning a
Tokimu-wide coordinate contract.** The old direct Doom lift was classified as
H1 + H2: it reversed the canonical source landmark determinant before camera
construction, and its lifted source-right opposed camera-right. H3 explained
why the first free-camera screenshots were insufficient, but the fixed source
spawn fixture reproduced the signed mismatch.

The Doom corpus consumer now selects the orientation-preserving Preserve North
adapter explicitly. Preserve East remains an equally coherent comparison
control related by a 180-degree world-Y rotation. Doom-relative evidence cannot
select between those absolute alignments, so Preserve North is a Doom consumer
convention rather than a universal Tokimu cardinal-axis decision.

Do not globally flip renderer UVs or platform input, and do not introduce a
public camera-basis API from this evidence. Provider conversions may remain
local only when named, testable, and lowered into existing explicit Tokimu
inputs; Alternative C's implicit provider autonomy remains rejected.

The comparative result and complete ownership table are retained in
[`coordinate-frame-comparative-results.md`](../Plans/Coordinate-Conformance/Evidence/coordinate-frame-comparative-results.md).
No ADR is produced because no Native or stable public meaning changed.

The final headless result and reproduction command are retained in
[`doom-source-world-spatial-orientation-evidence.md`](../Plans/Coordinate-Conformance/Studies/doom-source-world-spatial-orientation-evidence.md).

### Conformance Progress — 2026-08-10

The test plan now retains five distinct evidence layers without collapsing
their ownership:

- a frame ledger names the Doom, camera/input, renderer, and WGPU conversions;
- a shared asymmetric labeled fixture agrees across native and browser/WASM;
- a non-Doom live camera fixture agrees across native pointer capture and
  browser pointer lock while keeping acquisition separate from policy;
- the Doom provider owns named point/direction lifts, exact inverses, bounded
  heading round trips, opposed sidedef U axes, wall-facing tests, and a
  deterministic source-spawn command replay; a native asymmetric-art fixture
  now also presents complete readable `FRONT` and `BACK` panels through the
  right/front and left/back lowering paths;
- CPU projection and picking return to the same six labeled landmarks, while
  the WGPU regression proves that GL-style depth is converted exactly once at
  upload. `hello-cad` independently derives screen-right from its oblique view
  and round-trips its model center through picking.

Canonical package preflight confirms that both E1M1 `EXITSIGN` housings are
right/front upper-wall surfaces. An initial browser inspection camera averaged
opposed housings and correctly rejected their zero summed normal. A second
grouping exposed that linedefs 342–345 are themselves the four `+Z`, `-Z`,
`+X`, and `-X` faces of one rectangular housing. The repaired camera selects
single face 342. The browser presented that view successfully and the
maintainer confirmed that its `EXIT` lettering reads correctly.

No evidence so far requires a renderer/global UV flip, a platform-wide pointer
sign, or a stable Native spatial type. The evidence instead supports explicit
source conversion, application-owned interaction policy, ordinary
caller-supplied geometry/UVs, and one provider-owned clip-depth adaptation.

## Consequences

- The E1M1 corpus keeps its directional repairs and gains retained source-side
  evidence rather than silently treating them as visual polish.
- Existing renderer behavior remains stable: `Textured3d` consumes supplied UV
  coordinates, and WGPU keeps only its explicit clip-depth adaptation.
- Future camera, editor, picking, CAD, and non-Euclidean work must declare
  coordinate conversions rather than assuming that Doom's frame is Tokimu's.
- AR-0026 chart transitions may eventually make orientation-preserving versus
  orientation-reversing mappings intentional semantic data. The renderer must
  not infer that meaning from a matrix determinant, and this review does not
  yet propose a public chart-transition API.
- A later accepted result will require an ADR and targeted migration plan if it
  changes any public math, camera, input, or source-adapter contract.

## Required Follow-Up

- [x] Create the cross-target frame ledger and make it a retained corpus artifact.
- [x] Extend the AR-0021 native/browser orientation fixture with labeled basis observations.
- [x] Add an independent normalized-input camera fixture.
- [x] Add a Doom asymmetric texture-axis and observer-direction fixture.
- [x] Add bounded source/world point, direction, and orientation round-trip tests.
- [x] Separate mechanism, normalized observation, interaction policy, and camera convention in input evidence.
- [x] Compare both fixture families before selecting any Tokimu-wide convention.
- [x] Update AR-0019 and AR-0026 if a proposed spatial vocabulary changes their assumptions. No vocabulary was proposed; both reviews retain the observed future pressure without a contract change.

## Reopening Triggers

- native and browser/WASM disagree on a labeled direction or facing fixture;
- a non-Doom corpus needs the same camera/input basis;
- a renderer/provider change requires a second hidden sign conversion;
- CPU picking or projection disagrees with presented camera geometry;
- portal, chart, CAD, stereo, or editor work cannot express its frame without
  changing a public contract; or
- current E1M1 repairs fail an asymmetric source-direction fixture.

### Outside-Landmark Falsification Pressure — 2026-08-11

Interactive comparison of the E1M1 exterior with a canonical Doom capture
raised a plausible spatial concern: the small exterior hut appeared on the
opposite screen side from an informal recollection of the source view. The
same Tokimu observation was made after `--spawn-yaw-plus-90` and free movement,
so it is **not** source-faithful positional or heading evidence and does not
justify an axis flip.

The current adapter lift is numerically direct and exactly invertible:

```text
Doom map (x, y) -> corpus world (x, height, y)
```

and retained point/direction inverse tests continue to cover that exact rule.
That does **not** prove orientation preservation relative to Tokimu world-up.
For source right/east `(1, 0)` and forward/north `(0, 1)`, the new headless
observation records:

```text
source cross2(right, forward)               = +1
lifted right                                = +X
lifted forward                              = +Z
dot(cross(lifted right, lifted forward), +Y) = -1
observer camera-right for forward +Z         = -X
dot(lifted source-right, camera-right)        = -1
```

The old round-trip evidence and this handedness evidence can both be true.
The former proves reversibility; the latter proves that the current lift and
right-handed observer basis disagree about which lifted direction represents
source screen-right. This is a genuine reopening trigger, but it does not yet
identify whether the correct repair is the map embedding, heading/camera
conversion, or a more narrowly source-owned composition rule.

The investigation now separates:

- **H1 — world embedding reflection:** source landmark handedness reverses
  before a camera exists;
- **H2 — camera-basis reflection:** landmarks remain coherent, but source-right
  and presented camera-right oppose each other; and
- **H3 — uncontrolled comparison:** yaw offset, walked pose, FOV, or landmark
  identity explains the screenshot difference.

The purple sky experiment also established that neighbouring black regions can
be absent prepared geometry rather than a world-frame error. Before reopening
the disposition, a controlled landmark fixture must hold all of the following
constant: source position, source heading, yaw offset, pitch, field of view,
and landmark/source identity. It must compare the canonical source view with
the unmodified `--spawn-observer` path (not `--spawn-yaw-plus-90` and not a
walked pose). Only a mismatched identified landmark under that controlled
comparison is directional-conformance evidence.

No conversion sign is changed in this review cycle. The next evidence must use
identified non-collinear E1M1 records and a fixed unmodified source-spawn view;
renderer UVs, WGPU adaptation, and normalized platform input are explicitly
outside the repair surface.

The first source-record fixture now uses `THINGS #0` `(1056,-3616)`, the
midpoint of start-door `LINEDEFS #0` `(1056,-3680)`, and the midpoint of the
interactively identified exterior `BROWN1` hut wall `LINEDEFS #208`
`(2176,-3824)`. Its ordered source determinant is `+71,680`; after the current
lift, the corresponding cross product dotted with world `+Y` is `-71,680`.
This upgrades H1 from a unit-basis suspicion to canonical-package structural
evidence. The same fixture places the hut `+1,120` along Doom source-right but
`-1,120` along the current observer camera-right, giving H2 canonical landmark
support as well. The later fixed-pose native and browser observations close the
visual control without changing that source-record classification.

### Comparative Embedding Checkpoint

Three corpus-local embeddings now consume identical decoded Doom directions:

```text
CurrentReflected: east -> +X, north -> +Z
PreserveEast:     east -> +X, north -> -Z
PreserveNorth:    east -> -X, north -> +Z
```

All three are exactly invertible. Both candidates restore source orientation
and source-right/camera-right alignment to `+1`. Preserve East and Preserve
North differ by a 180-degree world-Y rotation when applied coherently, so a
Doom-relative screenshot cannot select between them. Selection requires an
independent axis relationship or an explicit Doom adapter policy.

Repository inspection also found that the existing Doom wall texture-axis
documentation explicitly names the current lift's right/front screen
reflection and reverses right/front U direction to keep source art readable.
The corrected `EXITSIGN` is therefore a known source-owned compensation for
the reflected embedding, not independent evidence that the embedding is
correct. Any candidate migration must reconsider that compensation alongside
wall winding and normals without changing ADR-0012's supplied-UV renderer
contract.

The running matrix is retained in
[`doom-orientation-embedding-comparison.md`](../Plans/Coordinate-Conformance/Studies/doom-orientation-embedding-comparison.md).

The first migration probe now runs the real source-derived sidedef fixture
under both candidates. Because either candidate reflects the current prepared
geometry, each must rebuild winding and normals; preserving renderer culling
cannot repair the source relationship. The fixture also explicitly removes
the existing reflected U compensation. Both candidates retain source-side
facing and readable camera-right U progression. A five-heading headless replay
likewise preserves transformed source-forward movement, source-right strafe,
and screen-right pointer look without introducing candidate-specific input
signs. These results narrow the affected seam to Doom source-to-world geometry,
UV, heading, and related source correspondence, but still cannot choose
Preserve East over Preserve North because the candidates remain related by a
coherent 180-degree world-Y rotation.

The maintainer then inspected canonical native E1M1 at the unchanged source
spawn under each candidate. Preserve East and Preserve North both place the
identified exterior hut on source-right and both retain readable `EXITSIGN`
art. This closes the fixed-spawn native visual control and confirms that the
wall-U/winding migration is coherent. It also supplies direct negative
selection evidence: Doom-relative presentation cannot decide which world
cardinal relationship Tokimu should preserve. Collision, Doom-membership
selection, dynamic doors, and browser parity were initially outside this
observation and were closed by the later source-correspondence controls.

The provisional architectural result is therefore narrower and stronger than
an axis choice: **source embeddings must not reverse orientation accidentally;
the remaining coherent 180-degree world-Y alignment is conventional and must
be owned explicitly outside Doom-relative evidence.** E1M1 has falsified the
reflection but cannot choose a global Tokimu cardinal convention.

The next source-correspondence probes preserve that conclusion. Exact picking
under both candidates retains the same hit distance after transforming the ray
and prepared mesh together. A source-owned collision wrapper lowers candidate
positions/deltas into the unchanged Doom blockmap and lifts the resolved
position back; both candidates retain contacted linedef identity, broad-phase
evidence, and the resolved source position. Interactive floor transitions now
use the same explicit conversion.

The maintainer subsequently walked both candidate E1M1 compositions with
collision enabled and observed coherent movement, wall blocking, and floor
transitions. After transforming the conservative subsector AABBs, both
candidates also produce the same source-membership observations: the overview
retains `237/237` subsectors and `1861` draws, while the fixed
source-spawn-yaw-plus-90 pose retains `61/237` and `474`. Flat facing is likewise
identical at `463` floor-up and `390` ceiling-down with zero inverted cases.
These results remove collision, floors, flat winding, and conservative source
membership as possible selectors between the two cardinal alignments. Dynamic
door re-lowering and browser parity were the final open controls.

An asymmetric diagnostic texture then exposed one more coupled migration
surface that ordinary Doom flats could not reveal. The retained purple
`texture_01.png` source reads `WALL`, but sky-omission ceiling meshes presented
the label right-to-left under both Preserve East and Preserve North. Reversing
only the continuous source-spatial flat U coordinate made `WALL` readable
under both candidates. This result is independent of the candidates' 180-degree
cardinal alignment and does not implicate PNG decoding or the generic
caller-supplied renderer UV contract; the independent directional and Box/PNG
fixtures retain those controls. The candidate migration now reverses flat U
about the source origin while retaining the separate wall-side U policy. It
must not reflect each triangle around its local UV extent, because that would
make triangulation boundaries change texture phase.

## Review History

### Cycle 1 -- 2026-08-10

- Status entering review: Proposed
- New evidence: E1M1 `EXITSIGN` is right/front-sided and initially mirrored;
  source-side U direction, local strafe, and mouse yaw each needed correction.
  The corrected sign was manually observed in the native E1M1 observer.
- Participants or reviewers: maintainer, Codex
- Findings: the separate symptoms may be manifestations of one unrecorded
  frame/basis assumption, but current evidence does not establish a global
  Tokimu convention.
- Disposition: Incubating under Alternative A.
- Resulting ADR or documentation change: None; this record preserves the
  investigation before any global architecture change.

### Cycle 2 -- 2026-08-10

- Status entering review: Incubating
- New evidence: independent review identified the frame ledger as the
  highest-value artifact, requested deliberately asymmetric direction/UV
  labels and reversible coordinate fixtures, and separated physical input,
  normalized observation, interaction policy, and camera convention.
- Participants or reviewers: maintainer, Monday, Codex
- Findings: similar inversions can belong to different layers. AR-0019 may
  need semantic frame vocabulary above raw math types, while AR-0026 may later
  require explicit orientation-preserving or orientation-reversing chart
  transitions.
- Disposition: remain Incubating under Alternative A; add the proposed
  falsification work without selecting a public convention or API.
- Resulting ADR or documentation change: expanded AR-0028 conformance plan and
  `docs/Plans/Coordinate-Conformance/coordinate-frame-directional-conformance.md`.

### Cycle 3 -- 2026-08-10

- Status entering review: Incubating under Alternative A.
- New evidence: complete native right/front and left/back labeled Doom art;
  canonical native/browser `EXITSIGN`; cross-target orientation and camera
  controls; CPU projection/picking; independent CAD and Box/PNG controls.
- Participants or reviewers: maintainer, Codex.
- Findings: every exercised sign/conversion has an identifiable source,
  application, renderer-input, or backend owner. Native/browser agreement is
  provider parity evidence, not proof of a universal basis. Future semantic
  frame/chart roles may sit above raw math but are not yet earned.
- Disposition: No Change. Retain explicit boundary conversions under
  Alternative A, reject global renderer/platform normalization, and produce no
  ADR.

### Cycle 4 -- 2026-08-11

- Status entering review: No Change.
- New evidence: canonical-versus-Tokimu exterior comparison placed a recognizable
  hut on the opposite screen side; a headless basis observation then proved
  that the direct Doom X/Y-to-world X/Z lift reverses signed orientation about
  world `+Y`, and that lifted source-right opposes observer camera-right.
- Participants or reviewers: maintainer, Monday, Codex.
- Findings: invertibility did not establish handedness preservation. H1 and H2
  are now independently testable; the uncontrolled screenshot remains H3
  pressure rather than sufficient repair evidence. Both orientation-preserving
  candidates pass the first structural basis checks, but are rotationally
  equivalent for Doom-relative presentation. Existing right/front U behavior
  is coupled compensation for the reflected lift.
- Disposition: Reopened under Alternative A. Retain the current implementation
  while identifying canonical source landmarks and the precise conversion
  boundary. Do not compensate in renderer, UV, platform input, or WGPU code.

### Cycle 5 -- 2026-08-12

- New cross-review evidence: the AR-0019 corpus-local chart control classifies
  a composed rigid transition as orientation-preserving and an independently
  invertible negative-X reflection as orientation-reversing under both pinned
  A and owned C0.
- Findings: the classification is derived by the framed semantic layer from
  transported basis directions. It confirms that raw matrix invertibility does
  not decide intended orientation behavior, without making `Mat4` a source- or
  chart-aware type.
- Disposition: retain the existing Doom source-adapter investigation and no
  global convention. The chart result is supporting evidence only.

### Cycle 6 -- 2026-08-13

- Status entering review: Reopened under Alternative A.
- New evidence: Preserve East and Preserve North retain exact picking,
  collision, floor transitions, flat facing, source-membership selection, and
  dynamic-door open/close/reopen correspondence. The final Browser WebGPU
  Preserve North fixture presented `1823/1823` opaque draws with
  `camera=canonical-exitsign`; the observation named
  `embedding=preserve-north`, and the maintainer confirmed readable `EXIT` art.
- Participants or reviewers: maintainer, Codex.
- Findings: H1 + H2 classify the old reflection; H3 was an observation-control
  concern rather than the cause. Preserve North is a coherent Doom consumer
  convention, while Preserve East demonstrates that Doom cannot choose a
  universal world-axis alignment. Dynamic geometry must pass through the same
  explicit adapter as static geometry.
- Disposition: No Change. Retain the Doom-local Preserve North repair and the
  explicit comparison controls. Admit no renderer UV flip, platform input
  normalization, public camera basis, or Tokimu-wide cardinal convention; no
  ADR is produced.

## References

- `docs/Architectural Reviews/AR-0021-geometry-orientation-and-facing-conformance.md`
- `docs/Architectural Reviews/AR-0022-textured-mesh-coordinate-and-sampling-boundary.md`
- `docs/Architectural Reviews/AR-0024-renderer-failure-observation-and-diagnostic-boundary.md`
- `docs/Architectural Reviews/AR-0026-non-euclidean-spatial-charts-and-authored-angular-topology.md`
- `docs/ADR/ADR-0012-supplied-mesh-texture-coordinates-and-sampling-policy.md`
- `corpus/lib/doom-geometry-provider/src/lib.rs`
- `corpus/campaigns/doom/hello-doom-e1m1/src/bin/static_scene.rs`
