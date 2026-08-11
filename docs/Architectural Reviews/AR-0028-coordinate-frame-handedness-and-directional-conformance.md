# AR-0028: Coordinate-Frame Handedness And Directional Conformance

| Field | Value |
| --- | --- |
| Status | No Change |
| Opened | 2026-08-10 |
| Last reviewed | 2026-08-10 |
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

**No Change; Alternative A is the supported operating rule.** The review found
no missing universal Tokimu basis. It found several differently owned
directional rules that had been implicit: Doom source conversion, application
camera-control policy, caller-supplied renderer inputs, and WGPU clip-depth
adaptation. Those rules are now explicit and tested at their owning boundaries.

Do not globally flip renderer UVs or platform input, and do not introduce a
public camera-basis API from this evidence. Provider conversions may remain
local only when named, testable, and lowered into existing explicit Tokimu
inputs; Alternative C's implicit provider autonomy remains rejected.

The comparative result and complete ownership table are retained in
[`coordinate-frame-comparative-results.md`](../Plans/Tests/coordinate-frame-comparative-results.md).
No ADR is produced because no Native or stable public meaning changed.

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

## Review History

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
  `docs/Plans/Tests/coordinate-frame-directional-conformance.md`.

## References

- `docs/Architectural Reviews/AR-0021-geometry-orientation-and-facing-conformance.md`
- `docs/Architectural Reviews/AR-0022-textured-mesh-coordinate-and-sampling-boundary.md`
- `docs/Architectural Reviews/AR-0024-renderer-failure-observation-and-diagnostic-boundary.md`
- `docs/Architectural Reviews/AR-0026-non-euclidean-spatial-charts-and-authored-angular-topology.md`
- `docs/ADR/ADR-0012-supplied-mesh-texture-coordinates-and-sampling-policy.md`
- `corpus/lib/doom-geometry-provider/src/lib.rs`
- `corpus/hello-doom-e1m1/src/bin/static_scene.rs`
