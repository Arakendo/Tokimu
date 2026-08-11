# Coordinate-Frame And Directional Conformance Test Plan

| Field | Value |
| --- | --- |
| Status | Complete — all six slices satisfied; AR-0028 records No Change |
| Opened | 2026-08-10 |
| Related review | AR-0028 |
| Related evidence | AR-0019, AR-0021, AR-0022, AR-0024, AR-0026, E1M1 static observer, Box/PNG corpus |
| Scope | Test whether geometry, source-coordinate conversion, camera basis, normalized input, and supplied texture axes share coherent directional assumptions across native and browser/WASM targets |

## Purpose

The E1M1 corpus exposed three directional failures that initially looked
independent:

- canonical right/front `EXITSIGN` art rendered horizontally mirrored;
- A/D first-person strafe moved in the opposite direction expected by the
  maintainer;
- captured mouse horizontal motion produced opposite yaw.

The local repairs now produce the expected E1M1 observation, but they do not
prove that Tokimu has one coherent spatial convention. This plan tests the
underlying assumptions before AR-0028 admits or rejects a shared coordinate,
camera, or input contract.

The study must distinguish:

```text
source convention
    -> explicit source/provider conversion
    -> Tokimu world/camera/input meaning under test
    -> renderer/backend adaptation
    -> observed presentation
```

A passing screenshot alone is insufficient. Each result must retain the
declared basis and conversion that produced it.

## Constraints

- Keep Doom/WAD sidedef, player-angle, and map-axis meaning inside Doom corpus
  providers and consumers.
- Keep platform pointer acquisition separate from camera yaw meaning.
- Keep renderer UV behavior caller-supplied under ADR-0012.
- Keep WGPU clip-depth conversion at the backend boundary under AR-0024.
- Do not introduce a stable/public coordinate, camera, or input contract while
  executing this plan.
- Do not infer a universal convention from Doom alone. At least one non-Doom
  caller must exercise any proposed shared behavior.
- Treat native and browser/WASM agreement as conformance evidence, not proof
  that the chosen convention is architecturally correct.
- Record false hypotheses and repaired fixtures rather than rewriting the
  study as though the final answer was obvious.

## Required Vocabulary In Test Evidence

Every fixture must state the applicable values rather than relying on words
such as “normal,” “forward,” or “correct” without a frame:

- source axes and handedness, if applicable;
- Tokimu world `+X`, `+Y`, and `+Z` meaning;
- camera forward, up, and right basis vectors;
- positive yaw direction and the view change it produces;
- positive pointer delta and the camera command it produces;
- positive strafe command and the world direction it produces;
- triangle winding and selected front-face/cull mode;
- texture U and V direction on a labeled surface;
- projection/clip-depth convention and backend conversion;
- target, backend, adapter/device metadata, viewport, and build identity.

The retained ledger should use a shape close to this and preserve `?` entries
until evidence resolves them:

| Meaning | Source | After adapter | Tokimu candidate convention | Provider realization |
| --- | --- | --- | --- | --- |
| `+X` | e.g. Doom east | `?` | `?` | N/A |
| `+Y` / up | source-dependent | world up | `?` | WGPU |
| forward | player/camera source | local forward | `?` | N/A |
| right | source side or local basis | local right | `?` | N/A |
| positive yaw | source/control policy | camera yaw | `?` | N/A |
| pointer `+X` | native/browser event | normalized delta | `?` | N/A |
| texture `+U` | source-relative | supplied UV | caller-owned | unchanged |
| front face | source winding | mesh winding | AR-0021 evidence | WGPU |
| clip depth | N/A | GL-style projection | Tokimu camera meaning | `[0,1]` conversion |

## Slice 1: Retained Frame Ledger

### Deliverables

- [x] Create one retained frame-ledger artifact covering:
  - [x] Tokimu math/world assumptions currently exercised by the fixtures;
  - [x] AR-0021 orientation fixture;
  - [x] Box/PNG supplied-UV fixture;
  - [x] native normalized pointer and keyboard input;
  - [x] E1M1 Doom map, wall-side, texture-axis, and player-heading conversion;
  - [x] WGPU/WebGPU clip-depth adaptation.
- [x] For each conversion, identify the owning layer and exact source and
      destination frame.
- [x] Mark every inferred or unspecified value explicitly; do not convert an
      implementation accident into a documented guarantee.
- [x] Record each input path as four separate fields: physical mechanism,
      normalized observation, interaction policy, and camera convention.
- [x] Record the original failed E1M1 hypotheses:
  - [x] the first mirrored-sign diagnosis incorrectly targeted left/back
        sidedefs;
  - [x] canonical-package evidence showed all submitted `EXITSIGN` surfaces
        were right/front;
  - [x] the right/front source-to-world U mapping was then corrected and
        manually observed.

### Validation

- [x] A reviewer can trace positions, normals, camera basis, input deltas, and
      UVs from source/provider input to renderer command without an unnamed
      sign flip.
- [x] The ledger distinguishes current observation, provisional convention,
      accepted contract, and unspecified behavior.

### Acceptance Criteria

- [x] No conversion in the tested path remains hidden behind “looks right.”
- [x] Ownership is assigned without moving Doom or native-window vocabulary
      into `tokimu-core` or `tokimu-render`.

### Retained Evidence

Slice 1 is retained in
[`coordinate-frame-ledger.md`](coordinate-frame-ledger.md). The ledger keeps
accepted contracts, current observations, provisional conventions, explicit
unknowns, and contradiction pressure separate. It also records complete Doom
wall/UV and input/camera traces plus the failed `EXITSIGN` hypothesis.

The E1M1 observer now derives local right through one corpus-local
`observer_right` helper and structurally distinguishes positive yaw from a
screen-right turn. This is evidence about the current observer convention; it
does not admit a public Tokimu camera basis.

## Slice 2: Asymmetric Geometry And Texture-Axis Fixture

### Deliverables

- [x] Extend or companion the AR-0021 fixture with a labeled asymmetric 3D
      surface that makes all of these visually distinguishable:
  - [x] front versus back;
  - [x] world left versus right;
  - [x] texture U minimum versus maximum;
  - [x] texture V minimum versus maximum;
  - [x] positive supplied normal direction.
- [x] Make the fixture deliberately non-symmetric: use readable
      `FRONT`/`BACK`/`LEFT`/`RIGHT`/`UP` labels, distinct corner numbers, and
      explicit `U+`, `U-`, `V+`, and `V-` marks. Do not use a checkerboard,
      cube, or arrow as the sole directional evidence.
- [x] Exercise identity, rotation/translation, and one explicit reflection.
- [x] Exercise `CullMode::None`, `Back`, and `Front` without changing the
      supplied geometry or UV stream.
- [x] Retain a deterministic structural manifest containing positions,
      normals, winding, UVs, transform, cull mode, and expected visible label.
- [x] Preserve the Box/PNG corpus as an independent supplied-UV control.
- [x] Add provider tests proving that generic renderer code does not infer or
      mutate source UV orientation.

### Validation

- [x] Native WGPU and browser/WebGPU present the same labeled face and U/V
      direction for every non-reflected case.
  - [x] Native WGPU labeled matrix manually observed on AMD Radeon RX 7900 XTX.
  - [x] Browser/WebGPU labeled matrix manually observed and compared.
- [x] Reflection behavior is explicit and any compensation is declared by the
      caller rather than inferred from determinant sign inside the renderer.
- [x] Box/PNG continues to render with its existing UV contract.

### Acceptance Criteria

- [x] Geometry facing and texture orientation can be diagnosed independently.
- [x] A UV correction cannot accidentally compensate for an inside-out mesh,
      camera basis error, or culling mismatch.

### Implementation Evidence

The exact shared source structure is retained in the
[`render-orientation-conformance` fixture manifest](../../../corpus/lib/render-orientation-conformance/fixture-manifest.md).
Native and browser consumers now use the same generated labeled atlas,
chamfered panel geometry, UV stream, transform cases, and cull matrix. Tests
assert the panel winding, reflection compensation, aligned UV stream, distinct
atlas corners, and absence of UV mutation during draw-contract validation.

Native and browser targets compile, including `wasm32-unknown-unknown`. The
native and browser labeled matrices were manually compared, and the revised
Box/PNG control was manually observed with one readable image per face. Its
flip-U and swap-UV modes deliberately remain visibly backward or rotated as
caller-owned negative controls; compilation and structural agreement were not
substituted for presented visual evidence.

## Slice 3: Camera Basis And Normalized Input Fixture

### Deliverables

- [x] Build a non-Doom camera-control corpus fixture with labeled world-axis
      landmarks at `+X`, `-X`, `+Y`, `-Y`, `+Z`, and `-Z`.
- [x] Display the current camera position, forward/up/right basis, yaw, pitch,
      pointer capture state, and most recent normalized input command.
- [x] Retain physical device observations separately from normalized commands;
      e.g. `mouse moved right` and `pointer_delta_x > 0` are separate facts.
- [x] Exercise deterministic command cases without requiring live input:
  - [x] positive and negative yaw command;
  - [x] positive and negative pitch command;
  - [x] forward/back movement;
  - [x] left/right strafe;
  - [x] vertical movement where supported.
- [x] Exercise live native input separately:
  - [x] Implement free cursor observation without rotating the camera;
  - [x] Implement click-to-capture;
  - [x] Implement captured raw horizontal motion through the declared policy;
  - [x] Implement Escape-to-release;
  - [x] Implement A/D over the declared local-right basis;
  - [x] Retain a manual native observation of all five behaviors.
- [x] Add browser/WASM input coverage using the same normalized command
      vocabulary where browser pointer-lock support permits it.
- [x] Retain deterministic input-command replays so live-device behavior is
      not the only evidence.

### Validation

- [x] For every command, the resulting basis is orthonormal within a declared
      tolerance and preserves the selected handedness.
- [x] `right`, `up`, and `forward` are derived by one documented cross-product
      order rather than duplicated sign formulas.
- [x] Native and browser command replays produce equivalent basis/pose output.
- [x] Live pointer acquisition differences do not change semantic yaw signs.
- [x] Touch, gamepad, first-person look, and editor orbit remain free to map
      mechanisms into distinct interaction intents without changing the camera
      basis convention under test.

### Acceptance Criteria

- [x] The fixture demonstrates camera and movement direction without importing
      Doom coordinates or renderer-facing source conventions.
- [x] Reversed A/D or pointer motion fails a structural assertion as well as a
      visual observation.

### Implementation Evidence

The shared, corpus-local basis and command model is retained in the
[`camera-direction-manifest.md`](../../../corpus/lib/render-orientation-conformance/camera-direction-manifest.md).
The native `camera_direction_conformance` binary renders six signed-axis
landmarks, displays pose/basis/capture/raw-observation/mapped-command state,
and implements explicit live capture and movement policies. Shared tests cover
orthonormality, command inverses, strafe direction, pointer-policy signs, and
the separation between a raw observation and a camera command.

Native pointer capture and browser pointer lock were manually exercised and
agreed on command/basis results. Their acquisition mechanisms remain separate,
and neither path turns first-person mapping into a universal input policy.

## Slice 4: Doom Source-Conversion Fixture

### Deliverables

- [x] Add a bounded Doom provider fixture with asymmetric labeled source art
      on both right/front and left/back sidedefs.
- [x] Retain source linedef start/end, sidedef identity, source sector,
      source-texture offset, source player angle, and generated world/UV data.
- [x] Add bounded round-trip tests where the conversion is intended to be
      information preserving:
  - [x] source point -> Tokimu point -> source point;
  - [x] source direction -> Tokimu direction -> source direction;
  - [x] source orientation -> Tokimu orientation -> source orientation.
- [x] Assert the provisional E1M1 mapping for:
  - [x] right/front U direction;
  - [x] left/back U direction;
  - [x] wall winding and supplied normal;
  - [x] Doom cardinal player angles converted to world forward;
  - [x] source-spawn camera position and eye height.
- [x] Retain canonical `EXITSIGN` records 342–350 as package integration
      evidence, including sidedef direction and normalized U range.
- [x] Run the source-spawn observer with deterministic command replay for
      forward, strafe, and yaw after conversion.
- [x] Keep the generic renderer input limited to ordinary meshes, UVs,
      materials, camera matrices, and normalized commands.

### Validation

- [x] Labeled right/front and left/back art reads correctly from its owning
      side under the declared Doom-to-world lift.
- [x] Culling, normals, and UV direction agree without renderer-side Doom
      branches or global UV flips.
- [x] Canonical `EXITSIGN` reads correctly in native and browser/WASM E1M1
      observations.
- [x] The fixture fails if the earlier left/back-only false hypothesis is
      reintroduced.
- [x] Round trips preserve the declared value within an explicit tolerance;
      exceptions are named and justified rather than silently approximated.

### Acceptance Criteria

- [x] Doom source conversion is explainable as one declared frame transform
      plus source-side texture semantics, not a collection of screenshot fixes.
- [x] The result does not claim that Doom's selected frame is Tokimu's global
      spatial ontology.

### Structural Evidence

The named conversion, bounded two-sided wall data, exact point/direction
round trips, bounded heading round trips, deterministic source-spawn command
replay, and canonical-package records are retained in
[`doom-coordinate-conversion-evidence.md`](doom-coordinate-conversion-evidence.md).
The final Slice 4 visual uses two spatially separated one-sided Doom walls
with opposed source directions so both owning sides face one camera. The
maintainer confirmed complete `BACK` and `FRONT` panels read correctly from
the left/back and right/front paths respectively. `ViewportRect` was removed
from the fixture after an initial attempt incorrectly treated its documented
pixel scissor behavior as an independent NDC viewport. Canonical face 342 also
has a dedicated browser inspection view and retained maintainer observation;
the distant browser overview was not substituted for that readable sign.

## Slice 5: Projection, Picking, And Cross-Boundary Falsification

### Deliverables

- [x] Project labeled world points through the tested camera and retain their
      expected screen quadrants and depth ordering.
- [x] Unproject or construct picking rays for the same screen points and verify
      that they intersect the corresponding labeled geometry.
- [x] Exercise native and browser/WASM WGPU paths with the explicit
      Tokimu-to-WGPU clip-depth conversion enabled.
- [x] Verify that removing or duplicating the clip-depth conversion fails the
      fixture visibly and structurally.
- [x] Add at least one orbit-control or CAD-style caller to test whether its
      notion of positive yaw/right agrees with or legitimately differs from
      first-person control.
- [x] Record known future falsification pressure from stereo views, reflected
      transforms, portal-derived views, and AR-0026 chart transitions without
      attempting to implement them in this plan.
- [x] Retain a theoretical chart-transition case for both
      orientation-preserving and orientation-reversing transforms. It is a
      vocabulary/falsification artifact only; do not infer transition meaning
      from a raw `Mat4` determinant or introduce a chart API in this plan.

### Validation

- [x] Presented geometry, CPU projection, and picking agree on left/right,
      up/down, forward, and depth.
- [x] Backend clip-depth adaptation does not change world or camera handedness.
- [x] Any first-person/orbit difference is expressed as input policy over a
      shared basis or retained as evidence that no shared policy should exist.

### Acceptance Criteria

- [x] The proposed convention survives a non-render-only consumer such as
      picking or CPU projection.
- [x] No backend or source adapter needs a hidden compensating sign flip.

### Structural Evidence

The shared camera matrix path, signed landmark quadrants, depth ordering,
world-ray reconstruction, WGPU upload-boundary regression, and future
falsification specimens are retained in the
[`projection-picking manifest`](../../../corpus/lib/render-orientation-conformance/projection-picking-manifest.md).
`hello-cad` supplies the independent oblique-view control: it round-trips its
model center through projection and picking, then derives screen-right from
its own inverse view instead of importing the first-person fixture's initial
world `-X`. A future orbit drag remains an application policy, not a missing
global input sign.

## Slice 6: Comparative Decision And AR-0028 Closeout

### Deliverables

- [x] Produce a result table comparing Doom, Box/PNG, orientation, camera/input,
      projection/picking, native, and browser/WASM evidence.
- [x] Classify every directional rule as one of:
  - [x] source/provider conversion;
  - [x] application control policy;
  - [x] candidate Tokimu-owned semantic convention;
  - [x] renderer/backend mechanism;
  - [x] intentionally unspecified behavior.
- [x] Evaluate AR-0028 alternatives using the retained evidence:
  - [x] keep corpus-local conversions;
  - [x] admit a narrow named Tokimu basis;
  - [x] retain provider-owned conventions;
  - [x] reject renderer/platform normalization.
- [x] Identify migration pressure on AR-0019 math vocabulary and AR-0026
      non-Euclidean chart vocabulary.
  - [x] Determine whether evidence needs semantic frame/transform roles above
        `Vec3` and `Mat4`, without making raw math types carry source ownership.
- [x] Update AR-0028 with the supported finding, unresolved limits, and final
      disposition.
- [x] Draft an ADR only if stable Native or public meaning has actually been
      earned.

### Acceptance Criteria

- [x] The selected answer is supported by Doom and at least one independent
      non-Doom caller.
- [x] A maintainer can identify exactly where each source/platform conversion
      belongs and which layer owns the resulting meaning.
- [x] The conclusion does not equate native/WASM agreement with universal
      correctness.
- [x] Any proposed stable contract includes migration, performance,
      diagnostics, failure containment, and recovery evidence required by
      ADR-0008 and ADR-0009.

No stable contract is proposed, so the last criterion is satisfied by an
explicit N/A: the comparative result preserves the existing architecture and
requires any future admission to reopen the complete ADR-0008/0009 gates.

### Comparative Result

The result table, rule classifications, alternative evaluation, AR-0019 and
AR-0026 pressure, and the reason no ADR was drafted are retained in
[`coordinate-frame-comparative-results.md`](coordinate-frame-comparative-results.md).

## Parking And Escalation Rules

Ordinary fixture defects, missing diagnostics, incorrect local signs, and
test/documentation gaps may be repaired within this plan.

Stop and return to architectural review if evidence requires:

- changing an accepted public math, camera, input, UV, facing, or projection
  contract;
- moving source-format meaning into a generic renderer or Native Ring type;
- exposing provider-specific clip or pointer behavior through a stable API;
- choosing a global convention that breaks an independent corpus caller;
- adding durable world/spatial vocabulary that changes AR-0019 or AR-0026;
- accepting contradictory native/browser observations; or
- hiding a conversion solely to preserve compatibility.

## Completion Definition

This plan is complete when all six slices are either satisfied or explicitly
parked with retained reasons, AR-0028 records a decision supported by both Doom
and non-Doom evidence, and no directional conversion in the tested path remains
implicit.
