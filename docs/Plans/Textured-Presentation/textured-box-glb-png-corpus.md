# Textured Box GLB And PNG Corpus

## Status

Evidence collection completed on 2026-08-09. This bounded renderer and
asset-boundary corpus experiment is now closed as a plan. It is deliberately
separate from the DOOM WAD plan: the Box
geometry and first-party PNGs provide generic texture-coordinate and sampling
pressure without importing WAD, palette, pegging, plane, or game semantics.

It supplies implementation evidence to
[AR-0022: Textured Mesh Coordinate And Sampling Boundary](../../Architectural%20Reviews/AR-0022-textured-mesh-coordinate-and-sampling-boundary.md).
AR-0022 remains the authority for any renderer-contract decision.
ADR-0012 accepts the UV/sampler contract proven by AR-0022. AR-0023 owns the
deliberately deferred alpha/depth question; it is not an unfinished
implementation item in this completed corpus plan.

## Purpose

Tokimu already has two independently useful inputs:

- the pinned Khronos `Models/Box/glTF-Binary/Box.glb` fixture, decoded by
  `corpus/focused/data-interchange/hello-glb` into Tokimu-owned positions and normals; and
- 78 first-party PNG texture fixtures in `corpus/assets/PNG`, which exercise
  grids, center lines, diagonals, checkers, low contrast, and palette variants.

The renderer can upload normalized RGBA8 data and bind a sampled material, but
its current mesh vertex contract has no caller-supplied texture coordinates.
Its `Texture2d` path derives coordinates from 2D position and uses one implicit
point-filtered, clamp-to-edge sampler. That is not a truthful 3D textured-mesh
contract.

This corpus will make that missing boundary concrete with a small, inspectable
scene: real decoded Box geometry, explicitly selected first-party PNG input,
Tokimu-owned render data, and native/browser observations.

## Goals

- Prove or reject a minimal generic contract for supplied per-vertex UVs on a
  3D mesh.
- Prove that a normalized RGBA8 texture can be selected, uploaded, and sampled
  on the Box without renderer knowledge of PNG or GLB semantics.
- Exercise declared sampler choices (at minimum point/linear and
  clamp/repeat) with fixtures whose visual result makes each choice obvious.
- Make opaque, blended, and cutout alpha behavior explicit; do not silently
  treat PNG alpha as a rendering policy.
- Retain deterministic structural evidence and separate native and browser/WASM
  execution observations.
- Produce evidence reusable by Slice 5B of the DOOM WAD plan after the generic
  boundary earns admission.

## Non-Goals

- A general glTF importer, glTF material importer, model viewer, or automatic
  use of images embedded/referenced by a GLB.
- Renderer-owned PNG decoding, asset paths, source names, or decoder objects.
- Doom texture composition, palette/`COLORMAP`, pegging, flats, sprites, sky,
  or perspective-correct Doom plane mapping.
- Lighting, normal maps, mip generation, anisotropic filtering, texture arrays,
  atlases, texture streaming, or a universal material graph.
- Pixel-identical GPU screenshots across drivers or browsers.

## Ownership And Evidence Boundary

```text
pinned Box.glb ----> corpus GLB decoder ----> Tokimu-owned mesh streams
                                                     |
first-party PNG ---> raster-image decoder ---> normalized RGBA8 texture input
                                                     |
                                                     v
                                  generic renderer mesh/material/sampler contract
                                                     |
                                                     v
                                            native or browser WGPU backend
```

- The GLB helper owns format-specific decode and retained fixture provenance.
- The raster-image corpus/provider owns PNG decode and turns bytes into
  provider-neutral normalized RGBA8 evidence.
- The corpus application selects a texture, transform, UV set, sampler, and
  alpha mode.
- `tokimu-render` may own only generic mesh streams, texture handles, material
  binding, declared sampling/alpha policy, backend realization, and diagnostics.
- The backend owns WGPU resources and browser acquisition mechanics; it does
  not decide source-asset or scene meaning.

Neither source format may leak into a public renderer API. The Box's geometry
and a PNG selected by the corpus are intentionally independent inputs; this
does not claim that `Box.glb` supplies, owns, or binds the PNG material.

## Fixture Set And Selection Rules

Slice 0 will record exact file sizes and SHA-256 values in the corpus manifest.
The initial minimum selection should include one PNG each for:

| Purpose | Preferred fixture characteristic | Required observation |
| --- | --- | --- |
| UV orientation | center line / labelled directional pattern | detects U/V flip and face rotation |
| Addressing | checker or grid | repeat versus clamp is distinguishable |
| Filtering | high-contrast fine grid or diagonal | point versus linear is distinguishable |
| Alpha | an asset with meaningful transparent/covered pixels, if available | cutout/blend outcome is explicit |
| Color variation | one matching pattern from a second palette | texture identity changes without mesh change |

The selected PNGs remain first-party reference assets, not decoder goldens or
colorimetric standards. If the existing set lacks a useful alpha case, Slice 0
must record that fact and either create a small first-party alpha fixture with
documented intent or defer alpha execution rather than fabricating a result.

## Implementation Slices

### Slice 0: Freeze Sources, Baseline, And Observations

Deliverables:

- [x] Record the exact Box fixture path, source revision/vendor state,
  checksum, decoded primitive count, and its current positions/normals result
  in [`corpus/campaigns/textured-presentation/hello-textured-box/fixture-manifest.md`](../../../corpus/campaigns/textured-presentation/hello-textured-box/fixture-manifest.md).
- [x] Select the initial PNG matrix and record each path, checksum, dimensions,
  alpha absence, and diagnostic purpose in the fixture manifest. The first-party
  set has no alpha source, so the alpha row remains an explicit later decision.
- [x] Record the current renderer limitation: no UV stream, derived 2D UV,
  implicit point/clamp sampling, and no alpha evidence from the selected inputs.
- [x] Name the future `hello-textured-box` corpus consumer and retain its
  boundary/design assertions in [`corpus/campaigns/textured-presentation/hello-textured-box/DESIGN.md`](../../../corpus/campaigns/textured-presentation/hello-textured-box/DESIGN.md).

Acceptance criteria:

- [x] A reviewer can reproduce every selected source input without searching
  for an unrecorded asset.
- [x] The baseline says exactly what is missing; it does not call the current
  2D texture path a 3D textured-mesh proof.

### Slice 1: Prove The Generic Mesh UV Contract

Deliverables:

- [x] Record AR-0022's incubating minimal UV/sampler/alpha proposal before
  implementing a public renderer seam. It remains reviewable and non-stable
  until the conformance evidence is complete.
- [x] Add an optional mesh UV stream aligned one-to-one with positions, with
  checked construction and `MeshValidationError` diagnostics for mismatch.
- [x] Extend the GPU vertex representation and `Textured3d` pipeline to
  consume supplied coordinates; it does not derive 3D UV from position.
- [x] Preserve the existing untextured mesh path; empty UV streams remain
  valid until a shader explicitly requires `TextureCoordinate2`.
- [x] Add unit tests for valid UVs, length mismatch, missing UV on a textured
  draw, and the empty-stream untextured behavior.

Acceptance criteria:

- [x] The renderer accepts caller UVs only through a provider-neutral mesh
  contract.
- [x] No GLB, PNG, WAD, or Doom type appears in `tokimu-render`.
- [x] A failed textured draw reports the missing/malformed mesh input through
  typed construction or draw-contract validation, without a backend panic or
  silent fallback.

### Slice 2: Make Sampling And Alpha Policy Visible

Deliverables:

- [x] Implement AR-0022's smallest sampler vocabulary: point/linear filtering
  and independent clamp/repeat U/V addressing.
- [x] Bind sampler policy as declared material input rather than as a global
  WGPU default.
- [x] Define the first alpha profile: the existing pipeline chooses opaque,
  alpha blend, or additive behavior; cutout/alpha test is explicitly
  unsupported until a source fixture and a reviewed threshold policy exist.
- [x] Add renderer tests that verify the chosen policy reaches backend resource
  creation/binding without exposing backend sampler objects.

Acceptance criteria:

- [x] The corpus requests sampler policy in generic terms and retains the
  selected policy in the native window title and manual observation record.
- [x] Alpha cannot accidentally change rendering behavior merely because a
  decoded PNG contains alpha: no source-alpha profile is implied, and cutout
  remains unsupported.

### Slice 3: Build The Textured-Box Corpus Consumer

Deliverables:

- [x] Add the focused `hello-textured-box` corpus entry.
- [x] Decode the pinned Box through the existing format helper, convert it to
  Tokimu-owned geometry, and provide explicit planar UV data owned by the
  corpus conversion boundary.
- [x] Decode the selected grid/dark-door/green-door PNG bytes through the
  existing raster boundary, then create explicitly labelled normalized RGBA8
  renderer textures.
- [x] Present a fixed camera and transform set that exposes at least three Box
  faces; retain a small mode/status readout with mesh, texture, sampler, and
  alpha declarations.
- [x] Provide deterministic modes for UV-orientation, addressing, filtering,
  palette variation, and alpha (when admitted).
  - [x] `M` cycles the independently uploaded grid/dark-door/green-door inputs.
  - [x] `R` cycles point-clamp, point-repeat, linear-clamp, and linear-repeat.
  - [x] `X` cycles identity, U-flip, and U/V-swap mappings without changing
        the decoded mesh positions/normals or selected texture.
- Deferred to [AR-0023](../../Architectural%20Reviews/AR-0023-textured-surface-alpha-and-depth-policy.md):
  do not add an alpha mode to the initial `Textured3d` fixture. The current 3D
  default combines `BlendMode::AlphaBlend` with depth writes, which does not
  establish correct transparent ordering; cutout has no admitted threshold
  policy.

Acceptance criteria:

- [x] Changing a texture or sampler changes only the declared material input;
  Box source geometry and UV input remain unchanged.
- [x] The orientation fixture can expose a U/V flip, a face rotation, clamp
  edge smear, repeat tiling, and filtering differences without ambiguous scene
  motion.
- [x] The consumer makes no claim that the selected PNG is a Box GLB material.

### Slice 4: Native And Browser Evidence

Deliverables:

- [x] Run the consumer on native WGPU and retain the environment, adapter,
  texture, sampler, alpha, and structural mesh evidence.
- [x] Add the narrow browser/WASM entry using the proven asynchronous WebGPU
  readiness pattern documented in `docs/lessions`.
- [x] Capture each required interactive mode only after the browser reports a presented
  frame; retain unsupported/timeout/failure states as distinct diagnostics.
- [x] Re-run the AR-0021 orientation posture as necessary so UV evidence is not
  confused with culling or camera-control defects.

Acceptance criteria:

- [x] Native and browser each execute at least the UV-orientation and one
  addressing/filtering mode, or retain a precise platform-specific blocker.
- [x] Successful module compilation is not presented as GPU initialization or
  rendering evidence.
- [x] Browser initialization retains the ready/presented state rather than a
  fixed time assumption.

Implementation-only validation retained before browser observation:

- [x] `cargo test -p hello-textured-box-web` completed successfully.
  The three focused tests cover the UV transform cycle, the complete generic
  sampler cycle, and the intentional out-of-range coordinate scale.
- [x] `cargo check -p hello-textured-box-web --target wasm32-unknown-unknown`
  completed successfully.
- [x] `wasm-bindgen` generated the browser package from the WASM artifact.
- [x] The project maintainer observed the browser's first presented textured
  frame; the retained record deliberately leaves interactive sampling and UV
  comparison open until those controls are observed.

These checks establish an executable browser artifact, not adapter acquisition,
surface configuration, or a presented frame.

### Slice 5: Negative Cases, Review, And DOOM Handoff

Deliverables:

- [x] Add negative corpus cases for malformed/missing UV data, invalid RGBA8
  payload/texture identity, unsupported sampler/alpha declaration, and
  unavailable browser presentation.
  - [x] Mesh construction rejects a non-empty UV stream whose count differs
        from positions; `Textured3d` draw-contract validation separately
        rejects a mesh with no supplied stream.
  - [x] RGBA8 descriptor and backend admission tests reject zero dimensions,
        wrong payload length, and duplicate texture identity before allocation.
  - [x] The sampler declaration is a closed provider-neutral enum rather than
        free-form backend input; the first corpus profile explicitly does not
        admit source alpha/cutout.
  - [x] The browser harness exposes distinct no-WebGPU, preflight timeout, and
        runtime-failure states. A real unsupported-browser observation remains
        optional evidence rather than a condition for claiming a working
        browser's presentation result.
- [x] Record structural manifests beside visual captures; label visual evidence
  as observation rather than cross-platform pixel equivalence.
- [x] Update AR-0022 with what the fixture proves, remaining gaps, and whether
  the generic contract should be retained, narrowed, or revised.
- [x] Update the DOOM WAD Slice 5B checklist only with the generic capability
  actually proven here; Doom-specific mapping remains a separate follow-up.

Acceptance criteria:

- [x] Every unsupported case fails explicitly at its owning boundary or is
  explicitly deferred to its owning review rather than silently approximated.
- [x] The final review separates mesh UV, sampling, alpha, GLB decode, PNG
  decode, native execution, and browser presentation claims.
- [x] The work produces a clear next decision for AR-0022 and no hidden Doom
  semantics in the renderer.

## Validation

Each implemented slice should run, as applicable:

```text
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The corpus must additionally retain:

- source/decoded/renderer-input manifests with fixture checksums;
- focused unit and integration-test results;
- native adapter and presented-frame evidence; and
- browser readiness, adapter/device, and first-present evidence or a precise
  failure diagnostic.

## Risks And Stop Conditions

- If UV, sampler, or alpha needs a broad material graph, stop and return to
  AR-0022; this plan admits only the narrowest demonstrated contract.
- If Box conversion requires importing glTF material behavior, keep that work
  outside this corpus and retain the current independent-source design.
- If a selected PNG does not distinguish the intended sampling behavior,
  replace it with another recorded first-party fixture rather than infer a
  visual conclusion.
- If browser execution exposes a general backend lifecycle defect, record it
  as a separate finding; do not report an absent frame as texture evidence.
- If a requested Doom behavior needs source-texel semantics or plane mapping,
  return it to the Doom provider/application boundary.
- If an alpha fixture would rely on `Textured3d` source-alpha blending while
  depth writes remain enabled, stop at AR-0022. A transparent rendering policy
  needs a separate ordering/depth decision; it must not be smuggled in as PNG
  fixture work.

## Plan Completion And Architecture Handoff

The corpus evidence collection is complete: it has a compact repeatable Box
scene, explicit UV/sampler composition, structural manifests, focused tests,
native and browser observations, and negative-boundary evidence. This plan is
therefore closed.

Architecture admission remains outside the completed plan:

- ADR-0012 governs the generic UV/sampler contract that DOOM Slice 5B may
  reuse; and
- AR-0023 owns any future alpha/cutout/depth decision.

## Current Closure Table

| Area | State | Evidence / next action |
| --- | --- | --- |
| Supplied UV contract | Accepted | ADR-0012 binds checked stream construction and `Textured3d` consumption. |
| Sampler vocabulary | Accepted | ADR-0012 binds point/linear and clamp/repeat material declarations; backend mapping and corpus evidence are retained. |
| Browser first presentation | Complete | Maintainer-observed first frame and explicit ready/unsupported/failed states are retained. |
| Browser interactive visual comparison | Complete | Maintainer confirmed the `M`/`R`/`X` controls and matching buttons update declared state and redraw after the first frame. |
| Alpha / cutout | Deferred by evidence | A transparent-source profile needs a separate depth/ordering or threshold decision; AR-0022 Cycle 10 prevents accidental admission through PNG bytes. |
| AR-0022 disposition | Accepted | ADR-0012 admits the narrow UV/sampler portion only. Alpha/depth remains AR-0023. |
| DOOM Slice 5B | Follow-on | It may reuse no more than an accepted generic UV/sampler contract. Indexed palette, masked middle, and plane mapping remain Doom decisions. |

## References

- [AR-0022: Textured Mesh Coordinate And Sampling Boundary](../../Architectural%20Reviews/AR-0022-textured-mesh-coordinate-and-sampling-boundary.md)
- [ADR-0012: Supplied Mesh Texture Coordinates And Sampling Policy](../../ADR/ADR-0012-supplied-mesh-texture-coordinates-and-sampling-policy.md)
- [AR-0021: Geometry Orientation And Facing Conformance](../../Architectural%20Reviews/AR-0021-geometry-orientation-and-facing-conformance.md)
- [Raster Image Corpus Testing](../../Libraries/raster-image-corpus-testing.md)
- [Hello GLB Design](../../../corpus/focused/data-interchange/hello-glb/DESIGN.md)
- [Streaming RGBA8 Texture Updates](../Standalone/streaming-rgba8-texture-updates.md)
- [DOOM WAD Checklist](../DOOM/DOOM%20WAD%20Checklist.md)
