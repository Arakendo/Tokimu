# Textured Surface Alpha-Policy Comparative Corpus

## Status

In progress since 2026-08-09. This plan supplies comparative evidence to
[AR-0023: Textured Surface Alpha And Depth Policy](../../Architectural%20Reviews/AR-0023-textured-surface-alpha-and-depth-policy.md).
It does not admit a renderer contract. AR-0023 owns the review disposition,
and an accepted or revised ADR is required before an experimental profile
becomes stable public `tokimu-render` vocabulary.

## Purpose

Determine whether opaque, categorical cutout, and continuous blending are:

- variants of one provider-neutral material policy;
- separate rendering capabilities that merely consume the same alpha bytes;
  or
- responsibilities that should be split between renderer mechanism and
  caller-owned scene orchestration.

The corpus holds image data, geometry, camera, and transforms constant while
changing only declared alpha, depth, and ordering intent. This makes the
central ownership rule observable:

> Alpha bytes are source data. Visibility, discard, depth, and ordering are
> declared caller policy.

## Architectural Questions

The study must answer these questions independently rather than assuming an
`AlphaPolicy` enum in advance:

1. Does cutout need only a caller-owned threshold plus ordinary depth writes,
   or does it expose additional coverage semantics?
2. Is a threshold comparison `<`, `<=`, or another explicitly named rule, and
   how are zero, one, NaN, infinity, and out-of-range values diagnosed?
3. Can blending remain a bounded material/pipeline mechanism while the caller
   visibly owns submission order?
4. Does blended depth writing have any safe default, or must it always be
   explicit?
5. Can missing or contradictory ordering intent be diagnosed before a visually
   plausible but incorrect frame is presented?
6. Do cutout and blend share meaningful public vocabulary beyond consuming an
   RGBA texture?
7. Which responsibility belongs in `tokimu-render`, which belongs in the
   application/corpus scene, and which should remain unsupported?

## Goals

- Compare opaque, cutout, and blend profiles against the same fixtures.
- Exercise categorical, continuous, overlapping, and intersecting alpha cases.
- Separate fragment, depth, ordering, and failure semantics in retained
  evidence.
- Prove that one RGBA8 texture can produce different results solely through
  caller-declared policy.
- Use Doom masked middles as independent real cutout pressure.
- Use a separate first-party translucent scene as independent blend pressure.
- Collect native WGPU and browser/WASM observations without claiming
  pixel-identical output.
- Produce enough evidence for AR-0023 to admit, defer, reject, or split the
  candidate capabilities.

## Non-Goals

- A general material graph, renderer-owned scene graph, or shader-authoring
  system.
- Renderer-owned transparent-object sorting, order-independent transparency,
  depth peeling, weighted blending, multisample alpha-to-coverage, or
  physically based transmission.
- Inferring policy from PNG, GLB, WAD, palette, or decoded alpha values.
- Treating Doom masked middles as evidence for continuous blending.
- Stabilizing a public enum before the corpus has compared separate candidate
  shapes.
- Pixel-golden conformance across adapters, drivers, operating systems, or
  browsers.

## Ownership And Dependency Boundary

```text
first-party RGBA8 fixtures ----> decoded pixels + retained coverage facts
                                          |
Doom masked middle ------------> Doom-owned source classification
                                          |
                                          v
corpus scene declares alpha/depth/order intent
                                          |
                                          v
experimental provider-neutral candidate seam
                                          |
                                          v
native WGPU / browser WebGPU realization
```

- Fixture and format providers own decoded RGBA8 values and source identity.
- Doom owns the statement that a selected middle texture is categorically
  masked; no WAD term crosses into `tokimu-render`.
- The corpus application owns profile choice, cutout threshold, draw sequence,
  and any claimed ordering intent.
- `tokimu-render` may eventually own only validated provider-neutral
  declarations and backend realization admitted by AR-0023 and an ADR.
- The WGPU backend owns blend, depth, discard, and pipeline realization. It
  does not infer policy from pixel contents.
- Renderer-owned sorting is outside this plan. If evidence requires it, stop
  and return the ownership question to AR-0023.

## Proposed Corpus Shape

```text
corpus/campaigns/textured-presentation/hello-alpha-policy/
    Cargo.toml
    DESIGN.md
    fixture-manifest.md
    src/
    results/

corpus/campaigns/textured-presentation/hello-alpha-policy-web/
    Cargo.toml
    DESIGN.md
    src/
    web/
    results/
```

The native and browser entries should share scene descriptions and structural
evidence helpers. They must not duplicate alpha meaning in target-specific
code. A small corpus-local helper may name experimental profiles, but those
names are not public renderer precedent.

## Fixture Matrix

All alpha fixtures are first-party, tiny, generated or hand-authored, and
recorded with dimensions, exact RGBA8 bytes or lossless source, checksum,
color-space interpretation, and intended diagnostic purpose.

| Fixture | Alpha population | Purpose |
| --- | --- | --- |
| Opaque control | alpha 255 only | proves unchanged opaque baseline |
| Binary mask | alpha 0 and 255 | isolates categorical keep/discard behavior |
| Threshold boundary | values immediately below, equal to, and above each tested threshold | exposes comparison and quantization rules |
| Continuous gradient | alpha 0 through 255 | exposes continuous contribution and banding |
| Mixed alpha | transparent, partial, and opaque texels in one image | renders identical RGBA8 under every profile |
| Colored transparent texels | nonzero RGB with alpha 0 | proves policy is not inferred from RGB or coverage heuristics |

Minimum threshold cases are `0.0`, a caller-selected interior value, and
`1.0`. Invalid cases include negative, greater-than-one, NaN, and positive or
negative infinity. The fixture must preserve exact 8-bit boundary values so a
comparison cannot be explained away as image-decoder variation.

## Scene Matrix

The fixed scene uses ordinary supplied-UV quads, one camera, fixed transforms,
and an opaque reference background. Image data remains unchanged across policy
comparisons.

| Case | Geometry / submission | Required comparison |
| --- | --- | --- |
| Same texture, three profiles | three isolated quads | opaque ignores alpha for visibility; cutout categorizes; blend contributes continuously |
| Cutout over opaque | foreground masked quad, opaque background | discarded fragments do not occlude; kept fragments use declared depth behavior |
| Blend over opaque | gradient foreground, opaque background | continuous contribution and depth-write choice remain visible |
| Overlapping blend pair | near and far translucent quads | front-to-back versus back-to-front submission differs visibly |
| Cutout/blend intersection | intersecting categorical and continuous quads | capabilities do not silently share depth or ordering assumptions |
| Identical-depth overlap | coplanar or near-coplanar controlled pair | depth comparison/failure remains explicit rather than called alpha behavior |
| Reversed input order | same scene with only draw order reversed | caller-owned ordering is retained and diagnosable |

Every case records the exact draw sequence, transforms, depth-test state,
depth-write choice, texture identity, threshold when applicable, and expected
semantic distinction. The corpus does not label an image “correct” merely
because it looks plausible.

## Evidence Dimensions

Each profile/case produces one structured observation with four separate
sections:

| Dimension | Required evidence |
| --- | --- |
| Fragment semantics | opaque keep, cutout keep/discard comparison, or blend factors/contribution intent |
| Depth semantics | depth test, depth-write choice, and whether a discarded fragment can write depth |
| Ordering responsibility | irrelevant, explicitly caller-owned with retained sequence, or unresolved and rejected |
| Failure and validation | invalid threshold, unsupported state, missing ordering declaration, backend failure, and recovery result |

Deterministic evidence consists of fixture hashes, scene descriptions, draw
order, policy declarations, validation outcomes, and structural fingerprints.
Native/browser images are retained visual observations with adapter, target,
build, viewport, and case metadata; their PNG bytes are not a rendering
specification.

Current hands-on native hardware coverage is AMD Radeon/Vulkan and Apple/Metal.
NVIDIA execution is presently unverified and may contain backend-specific gaps;
passing observations on the available adapters must not be reported as NVIDIA
conformance. See
[`GPU Adapter Validation Coverage`](../../lessions/gpu-adapter-validation-coverage.md).

## Implementation Slices

### Slice 0: Freeze Questions, Baseline, And Fixtures

Deliverables:

- [x] Create the native and browser corpus design records and classify their
      ownership, inputs, outputs, state, and authority.
- [x] Record current behavior: opaque is demonstrated; `AlphaBlend` exists as
      mechanism; current `Textured3d` blend-plus-depth-write behavior is not an
      admitted transparency contract; cutout has no threshold vocabulary.
- [x] Create the six first-party alpha fixtures and retain exact hashes and
      byte-level alpha distributions.
- [x] Freeze the scene matrix, fixed camera, transforms, viewport, and draw
      identities before implementing candidate policy seams.
- [x] Define structured observation and failure schemas without choosing a
      stable renderer API.

Acceptance criteria:

- [x] One reviewer can reproduce every fixture and scene without an unrecorded
      asset or manual layout choice.
- [x] The baseline distinguishes existing mechanism from admitted semantics.
- [x] No fixture name or decoder behavior implicitly selects an alpha policy.

### Slice 1: Build A Headless Semantic Oracle

Deliverables:

- [x] Implement corpus-local reference evaluation for opaque, threshold
      comparison, straight-alpha contribution, and expected depth eligibility.
- [x] Record the exact threshold comparison candidates rather than silently
      choosing `<` or `<=`.
- [x] Produce deterministic expected fragment-classification tables for every
      alpha byte in the boundary and mixed fixtures.
- [x] Model draw order as explicit input and prove reversed sequences remain
      distinct observations.
- [ ] Add negative tests for non-finite/out-of-range thresholds, missing blend
      ordering intent, contradictory depth declarations, and malformed RGBA8.
  - [x] Reject non-finite/out-of-range thresholds, missing/empty/duplicate
        caller ordering, empty draw identities, malformed RGBA8 lengths, and
        overflowing dimensions with typed retained failures.
  - [ ] Define a candidate renderer request before deciding what constitutes a
        contradictory depth declaration; the headless oracle does not invent
        that contract merely to complete this item.

Acceptance criteria:

- [x] Headless evidence explains what each visual case is intended to expose.
- [x] The oracle owns no GPU objects and makes no provider-specific rounding
      guarantee beyond its declared reference arithmetic.
- [ ] Unsupported or ambiguous input produces a structured rejection, not a
      fallback profile.
  - [x] All currently defined invalid inputs produce typed failure records.
  - [ ] Contradictory depth intent remains undefined until a candidate GPU
        request is available; no fallback is implemented.

### Slice 2: Compare Opaque And Cutout Candidates

Deliverables:

- [x] Preserve the current explicit opaque profile as the control.
  - [x] The native corpus realizes the frozen opaque control with an explicit
        `Opaque` render state, rather than inheriting `Textured3d`'s existing
        blend mechanism.
- [x] Implement cutout as an experimental candidate without stabilizing a
      public enum or default threshold.
  - [x] The native corpus realizes `< 128/255` and `<= 128/255` through two
        labelled corpus-local WGSL shaders, using the existing custom-WGSL
        mechanism without changing `tokimu-render` pipeline vocabulary.
- [ ] Exercise zero, interior, and one thresholds plus byte values below,
      equal to, and above the boundary.
  - [x] The headless oracle covers `0.0`, `128/255`, and `1.0`, including the
        distinct `<`/`<=` behavior at transparent, exact-boundary, and opaque
        source alpha.
  - [ ] Native/browser visual observations currently cover the interior
        `128/255` threshold only; zero/one visual variants remain open.
- [x] Prove discarded fragments do not write color or depth and retained
      fragments follow the explicitly declared depth state.
  - [x] The headless oracle asserts discarded fragments are ineligible for
        depth writes, and the retained native depth panel exposes its opaque
        backing through discarded binary-mask texels.
- [x] Run the same mixed-alpha texture under opaque and cutout while changing
      no source bytes, UVs, geometry, camera, or transforms.
  - [x] Native and browser executables import the same exact fixture,
        corpus-local WGSL generator, threshold, viewport, UV geometry, and
        visual layout constants from `hello-alpha-policy`.
- [x] Retain shader/pipeline validation failures and backend diagnostics.
  - [x] The corpus retains a typed provider-neutral rejection for a missing
        custom candidate shader; it cannot fall back to an opaque/default
        pipeline.
  - [x] The existing generic `hello-shader --backend-diagnostic-fixture` path
        registered deliberately invalid WGSL on the native backend, drained the
        renderer diagnostic sink, preserved its module and entry-point identity,
        and exited without submitting the invalid pipeline. Alpha policy reuses
        that backend evidence rather than duplicating a malformed-shader
        mechanism.

Acceptance criteria:

- [ ] Cutout behavior can be described without PNG, WAD, coverage-provider,
      or Doom vocabulary.
- [ ] Threshold ownership and comparison semantics are explicit and tested.
- [ ] Native and browser observations agree on categorical outcomes, or the
      study retains a precise cross-target blocker.
  - [x] Native AMD/Vulkan observation agrees with the headless oracle.
  - [x] Browser/WASM source compiles, bindings generate, the local host serves,
        adapter/device preflight succeeds, and first presentation is observed.
  - [x] Browser/WebGPU observation matches the native categorical interior
        threshold result. The browser does not expose a useful adapter name;
        that absence is retained in the observation rather than inferred.

### Slice 3: Compare Blending, Depth, And Caller Ordering

Deliverables:

- [x] Implement a native corpus-local comparison with fixed RGBA8 source
      data, explicit `AlphaBlend`, caller submission order, and separately
      declared depth-write states.
  - [x] `native_blend_scene` holds source texture, UVs, camera, blend equation,
        and transforms constant across its far-then-near, near-then-far,
        depth-write-off, and depth-write-on panels.
  - [x] Retain native first-presentation and visual observations before treating
        the candidate execution as demonstrated.
    - [x] AR-0024 localized the first empty frame to positive world Z mapping
          outside WebGPU's clip-depth interval under Tokimu's current GL-style
          projection. Maintainers selected explicit WGPU-boundary conversion;
          both upload paths now share the tested mapping and the corpus has
          restored positive GL depths. AMD/Vulkan visibly presented all panels,
          the opaque control, and `diagnostic=none`; the retained observation
          is `native-blend-observation-2026-08-09.md`.
- [x] Exercise the continuous-gradient and mixed-alpha fixtures with straight
      source-alpha blending.
  - [x] Native `native_blend_scene` uploads both immutable sources. Its
        upper-left control blends all 256 continuous-alpha bytes over opaque
        blue; its four comparison panels retain `mixed-alpha` for ordering and
        depth-write differences.
  - [x] Browser/WASM `?mode=blend` imports the same gradient and mixed-alpha
        source identities, blend WGSL, layout, camera depths, and declared
        pipeline states from `hello-alpha-policy`.
  - [x] Native AMD/Vulkan observation retained
        `native-blend-observation-2026-08-09.md`: the added gradient and all
        four comparison panels visibly presented; first=12 draws/12 material
        resolutions/6 pipeline switches/13 binding allocations and warm=the
        same draws/material/pipeline work with zero binding allocations,
        uniform writes, and mesh uploads.
  - [x] Browser/WASM observation retained
        `browser-blend-observation-2026-08-09.md`: the gradient, both caller
        orders, and both depth-write states visibly presented with 12 draws and
        `diagnostic=none`; its identical warm frame retained the same
        draw/material/pipeline work with zero binding allocations and mesh
        uploads.
- [x] Compare explicit depth writes on and off; do not inherit a default and
      call it correct.
- [x] Submit overlapping translucent quads front-to-back and back-to-front,
      retaining the exact caller sequence in every observation.
  - [x] The corpus-only fixed `LessEqual` depth oracle proves that the same
        far/near reversal preserves opaque and retained-cutout output when
        those profiles write depth, while no-depth-write Blend changes output.
        This is a bounded reference result, not a renderer sorting contract.
- [x] Instrument the existing renderer behavior before proposing submission,
      shader-resource, batching, or ordering vocabulary.
  - [x] The fixtures retain the current submission sequence; WGPU reports
        per-frame material resolutions, pipeline switches, binding allocations,
        uniform writes, and mesh uploads without introducing a render queue or
        a public shader-resource contract.
  - [x] The native fixture uploads fixed meshes and its initial camera before
        its frame boundary, so its second frame reports renderer reuse rather
        than fixture-authored static-mesh churn. The browser fixture presents
        the identical command array twice to retain first-versus-warm counters.
  - [x] Retain the updated native and browser first-versus-warm observations.
    - [x] Browser/WASM retained first=12 draws/12 material resolutions/6
          pipeline switches/13 binding allocations/0 mesh uploads and warm=12
          draws/12 material resolutions/6 pipeline switches/0 binding
          allocations/0 mesh uploads, with `diagnostic=none`.
    - [x] Native AMD/Vulkan retained the same setup-to-warm reuse result as
          browser/WASM: thirteen first-frame binding allocations and zero
          warm-frame binding allocations, uniform writes, or mesh uploads.
        A binding allocation is a provider-observable resource allocation, not
        a claim that all providers expose WGPU bind groups.
- [x] Define an experimental way for the caller to state ordering intent or
      record that no adequate diagnosable declaration was found.
  - [x] The corpus-local `StudyProfile::Blend` requires `CallerOrdering`, and
        records typed missing, empty, empty-identity, and duplicate-identity
        rejections. The frozen scene manifest and both executable fixtures
        retain far-then-near, near-then-far, and near-then-far-with-depth-write
        sequences; no renderer ordering contract is proposed.
- [x] Reject any case that claims correct general blending while ordering
      responsibility is absent or ambiguous.
  - [x] Missing, empty, and duplicate caller-order declarations are typed corpus
        rejections; valid observations retain the exact source order. No
        renderer ordering or general-correctness claim is made.
- [x] Exercise recovery after invalid/unsupported state without losing the
      next valid frame.
  - [x] Both native and browser Slice 3 setup deliberately reject a depth-write
        request with `DepthTest::Disabled` before preparing their valid fixed
        scene. Browser/WASM then presented valid first and warm frames; native
        AMD/Vulkan likewise visibly presented the expanded first and warm
        frames. Both targets retain rejection before valid setup and later
        successful presentation.

Acceptance criteria:

- [x] Evidence separates blend factors, depth writes, and draw ordering rather
      than treating them as one switch.
- [x] “Caller-owned ordering” is visible in retained input and diagnostics.
  - [x] The corpus records exact caller submission input and provider-observable
        pipeline/material/resource-churn counters.
  - [x] Missing, empty, empty-identity, and duplicate ordering declarations
        have typed corpus rejections. Valid renderer frames correctly report
        `diagnostic=none`; no public Blend ordering diagnostic is admitted.
- [x] No renderer-owned sorting service is introduced by implication.

### Slice 4: Exercise Interaction And Cross-Target Behavior

Deliverables:

- [x] Run cutout in front of opaque geometry, blend in front of opaque
      geometry, and intersecting cutout/blend geometry.
  - [x] Add one fixed native/browser interaction scene with three panels:
        categorical cutout over opaque, continuous Blend over opaque, and a
        cutout plane crossing a sloped blended plane over opaque backing.
    - [x] Native `native_interaction_scene` uses the fixed shared layouts and
          exact binary-mask/mixed-alpha fixture identities. Its sloped mesh
          test proves vertices occur on both sides of the cutout depth.
    - [x] Native AMD/Vulkan visual observation retained in
          `native-interaction-observation-2026-08-09.md`: seven draws with no
          diagnostic; categorical cutout, continuous Blend, and the
          depth-crossing interaction are separately visible. It records the
          source build identity and locked interaction manifest
          `0a99c714c258bac7f91eb5dd39748651abca8db96bfc1a410d823a18d2c23d93`.
    - [x] Browser/WASM `?mode=interaction` realizes the same three fixed
          panels, shared texture identities, UV-bearing meshes, camera,
          transforms, submission order, and declared depth states. The
          `wasm32-unknown-unknown` build and binding generation succeed; the
          browser adapter/device/first-presentation observation is retained in
          `browser-interaction-observation-2026-08-09.md`.
  - [x] Keep source fixtures, UVs, camera, transforms, declared depth states,
        and submission order manifest-locked for each panel. The crossing case
        must use geometry that actually meets in depth, not only screen-space
        overlap. Shared constants define the panel layout and depths; native
        and browser issue the same seven draws: opaque backing then cutout;
        opaque backing then no-depth-write Blend; opaque backing then
        depth-writing sloped Blend then cutout.
- [x] Render identical RGBA8, geometry, UV, camera, and transforms under every
      candidate profile. Slice 2 supplies the same-texture profile comparison;
      the Slice 4 panels retain the independently meaningful binary-mask and
      mixed-alpha consumers without changing their per-panel inputs.
- [x] Execute the full minimum matrix on native WGPU and browser/WebGPU using
      the established asynchronous browser readiness/presentation pattern.
- [x] Retain adapter, backend, device kind, viewport, build identity, fixture
      hash, scene hash, and presented-frame status.
  - [ ] Retain NVIDIA execution evidence when suitable hardware becomes
        available; until then, report NVIDIA as an explicit coverage gap rather
        than silently reducing the adapter matrix.
- [x] Capture fixed-camera visual observations without pixel-golden claims.
- [x] Compare structural observations across targets before interpreting image
      differences. Both targets retain the same manifest, viewport, source
      fixture hashes, seven submissions, first-presentation status, and
      `diagnostic=none`; visual agreement is interpreted only at that bounded
      scene level.

Acceptance criteria:

- [x] A reviewer can identify whether a difference came from policy, depth,
      order, input, or target.
- [x] Successful compilation, adapter acquisition, device readiness, and first
      presentation remain separate states.
- [x] Cross-target failure is retained as evidence and does not silently reduce
      the matrix. No available-target failure occurred; NVIDIA remains an
      explicit uncovered target rather than an implied pass.

Slice 4 is complete for the available native AMD/Vulkan and browser/WebGPU
paths. NVIDIA execution remains parked as the repository-wide explicit
coverage gap, not as a blocker to interpreting this bounded cross-target
comparison.

### Slice 5: Independent Real-Caller Pressure

Deliverables:

- [x] Select at least one source-traceable Doom masked-middle case and lower
      its classification into the experimental generic cutout candidate.
      E1M1 retains 13 source classifications across four texture names and
      lowers 26 non-degenerate wall triangles; see
      [E1M1 masked-middle cutout intake evidence](../DOOM/Evidence/E1M1%20masked-middle%20cutout%20intake%20evidence.md).
- [x] Keep Doom threshold choice and original-behavior claims at the Doom
      consumer boundary; pass only generic declared intent onward. The local
      binary-coverage experiment declares discard at or below RGBA8 alpha zero
      with depth write enabled. It changes neither the opaque E1M1 draw plan
      nor any renderer contract.
- [x] Select a separate first-party continuous-alpha consumer that genuinely
      needs blending and does not borrow Doom as a synthetic justification.
      `hello-glb`'s application-owned inspection state makes the retained Box
      mesh continuously translucent at opacity `0.35`, retaining alpha blend,
      disabled depth writes, and explicitly limited submission-order behavior.
      Its opacity override is independent of its GLB source material and of
      Doom; its focused policy tests pass.
- [x] Compare each real caller against the matching shared fixture behavior.
  - [x] The E1M1 opt-in cutout path reached native AMD/Vulkan first-frame
        initialization with the shared fixture's corresponding explicit
        opaque/depth-writing state and 26 additional source-selected draws.
        This is structural integration evidence only; a fixed-camera native
        visual capture remains open.
  - [x] The existing Rust/WASM browser workbench now builds a separate local
        `render_static_e1m1_masked_cutouts(canvas)` request and regenerates its
        bindings/TypeScript surface. Its normal opaque request remains isolated
        from candidate preparation and custom-pipeline validation.
  - [x] Browser/WebGPU visibly presented the selected canonical package through
        both fixed-camera actions: opaque `1835` draws and masked-cutout `1861`
        draws. The workbench returned `cutouts=false`/`true` respectively with
        `backend=browser-webgpu`, `device=other`, blank adapter identity, and
        `canvas=960x600`; this is retained bounded visual evidence, not
        pixel-golden or stable-contract evidence.
- [ ] Record migration/API pressure, diagnostics quality, native/WASM behavior,
      and any duplicated or awkward vocabulary.
  - [x] Retain the initial caller-to-fixture mapping and boundary/API-pressure
        comparison in
        [Alpha-policy real-caller comparison](Evidence/alpha-policy-real-caller-comparison.md).
        The E1M1 browser observation is retained; native E1M1 and transparent
        GLB visual comparisons remain open and are not represented as completed
        two-caller evidence.
  - [x] The GLB capture entry now freezes its normal orbit/mesh transforms at
        the initial frame. Its valid AMD/Vulkan first frame retained two draws,
        four setup binding allocations, and no uniform or mesh uploads; warm
        frames retained the same two draws with zero new bindings, uniform
        writes, or mesh uploads. This is diagnostics/performance evidence, not
        a visual or browser comparison.
  - [x] A reviewer visibly observed the frozen native GLB capture with the
        translucent Box over its opaque floor and title
        `presentation=transparent`. The GLB entry intentionally remains native
        only; its shared-fixture counterpart, rather than a new browser GLB
        application, supplies the browser/WASM Blend observation.

Acceptance criteria:

- [x] Cutout has independent real pressure beyond the synthetic fixture.
- [x] Blending is not admitted without an independent continuous-alpha caller.
      `hello-glb` supplies the required independent caller, but AR-0023 still
      admits neither Blend nor cutout as a stable renderer capability.
- [x] Neither caller passes format-specific types or policy inference into the
      renderer. The retained boundary table separates source ownership from
      renderer-facing generic intent.
- [x] The real callers map to separate shared-fixture roles without forcing a
      shared public alpha vocabulary, renderer-owned ordering service, shader
      resource interface, PBR contract, or source-format renderer term.

### Slice 6: Performance, Failure Containment, And Review

Deliverables:

- [x] Apply ADR-0008's full performance gate to any proposed stable renderer
      crossing: pipeline variants, shader branching, allocation, batching,
      draw-order preparation, native behavior, and the sequential WASM path.
  - [x] Retain the pre-admission gate ledger in
        [Alpha-policy Slice 6 gate review](Evidence/alpha-policy-slice6-gate-review.md).
        No stable crossing is proposed, so the implementation-specific full
        gate is explicitly pending rather than ritualistically marked passed.
- [x] Apply ADR-0009's verification/failure-containment gate: unit tests,
      corpus tests, malformed inputs, backend failures, error capture, recovery,
      and retained evidence.
  - [x] The same ledger records the existing typed invalid-input, shader/
        state rejection, valid-frame recovery, diagnostic, and containment
        evidence—and its deliberate recovery limits.
- [x] Verify no source-alpha scan or heuristic occurs on the steady-state draw
      path to select policy.
  - [x] Candidate selection and source decoding happen in bounded preparation;
        fixed frame submission consumes prebuilt meshes/materials/pipelines.
- [x] Compare API pressure for separate cutout/blend declarations against a
      common policy shape without making either public prematurely.
  - [x] The ledger rejects a common public policy shape because cutout and
        Blend differ in threshold, depth, and ordering ownership.
- [x] Update AR-0023 with findings, rejected shapes, unresolved questions, and
      a disposition.
  - [x] Cycle 26 retains the gate ledger and completes the pre-admission review;
        it deliberately leaves the stable disposition to maintainer judgment.
- [x] Create or revise an ADR only for semantics the review accepts as a
      stable renderer contract.
  - [x] ADR-0013 admits categorical Cutout only; it expressly keeps Blend
        incubating and rejects a common alpha-policy shape.

Acceptance criteria:

- [x] Every proposed admission has an independent caller, native/browser
      evidence, negative tests, diagnostics, and proportional performance
      evidence.
  - [x] The review proposes no stable admission; the retained ledger records
        what the corpus evidence proves and what future implementation evidence
        would still be required for each candidate.
- [x] “N/A” gate answers carry a local reason.
  - [x] The Slice 6 ledger records why no implementation-specific full gate,
        benchmark, recovery service, or ADR can exist before a stable crossing
        is selected.
- [x] The review can choose among the dispositions below without treating a
      successful demo as automatic admission.
  - [x] The maintainer selected bounded Cutout after the evidence split it from
        Blend; `docs/Plans/Textured-Presentation/Studies/categorical-cutout-capability-admission.md` carries
        the remaining concrete implementation and full-gate work.

## Candidate Review Outcomes

AR-0023 may conclude:

1. retain opaque only and defer both alpha capabilities;
2. admit bounded cutout while deferring or rejecting blending;
3. admit cutout and blend as separate capabilities;
4. admit a shared alpha-policy vocabulary because evidence shows genuinely
   common validation and ownership;
5. admit only renderer blend mechanism while keeping ordering/depth
   orchestration in an explicit outer-ring scene contract;
6. move one or both concerns outward because the proposed semantics do not
   belong in `tokimu-render`.

The study must report why the rejected outcomes failed. It must not prefer a
common enum merely because it is syntactically compact.

## Validation Matrix

Minimum automated validation:

- fixture checksum and exact alpha-distribution tests;
- threshold boundary and invalid-value tests;
- opaque/cutout/blend structural observation tests;
- draw-order reversal and missing-order declaration tests;
- depth-write on/off and discarded-depth tests;
- malformed RGBA8 and missing-resource tests;
- backend pipeline/shader validation diagnostics;
- recovery followed by a valid presented frame;
- native compilation and focused tests;
- `wasm32-unknown-unknown` compilation and browser binding generation;
- browser readiness/device/first-presentation state separation;
- `cargo fmt --all`, relevant Clippy checks, and focused/workspace tests as
  proportional to the touched boundary.

## Parking And Escalation Rules

Continue ordinary fixture, consumer, test, diagnostic, and evidence work
within this plan. Return to AR-0023 before continuing when evidence requires:

- a stable/public renderer contract;
- renderer-owned scene sorting or new scene authority;
- a dependency-direction change;
- source-format policy in a generic renderer API;
- a provider-specific behavior presented as universal semantics;
- a material performance or WASM-path finding;
- contradictory native/browser semantics; or
- weakening failure validation merely to complete the matrix.

If the blend caller never materializes, park blend rather than manufacturing
admission pressure. If Doom cutout can be satisfied outside the renderer with
a cleaner existing boundary, record that alternative rather than presuming
Native ownership.

## Completion Criteria

The plan is complete when:

- the shared fixture and scene matrices have reproducible structural evidence;
- native and browser observations cover the minimum comparison matrix or retain
  precise blockers;
- Doom supplies real cutout pressure and blending has an independent caller or
  is explicitly parked;
- threshold, depth, ordering, validation, recovery, and performance findings
  are retained separately;
- AR-0023 records whether the capabilities are shared, separate, outward, or
  deferred; and
- any accepted stable contract is recorded in an ADR rather than inferred from
  corpus code.

## References

- `docs/Architectural Reviews/AR-0023-textured-surface-alpha-and-depth-policy.md`
- `docs/Architectural Reviews/AR-0022-textured-mesh-coordinate-and-sampling-boundary.md`
- `docs/ADR/ADR-0008-native-kernel-ring-performance-and-code-quality.md`
- `docs/ADR/ADR-0009-ring-based-verification-failure-containment-and-recovery.md`
- `docs/ADR/ADR-0012-supplied-mesh-texture-coordinates-and-sampling-policy.md`
- `docs/Plans/Textured-Presentation/textured-box-glb-png-corpus.md`
- `docs/Plans/DOOM/DOOM WAD Checklist.md`
- `docs/lessions/webgpu-wasm-quick-reference.md`
