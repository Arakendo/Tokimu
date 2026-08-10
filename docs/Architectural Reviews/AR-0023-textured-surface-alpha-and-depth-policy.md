# AR-0023: Textured Surface Alpha And Depth Policy

| Field | Value |
| --- | --- |
| Status | Accepted in part — Cutout admitted for implementation; Blend incubating |
| Opened | 2026-08-09 |
| Last reviewed | 2026-08-09 |
| Scope | Cross-cutting renderer/material and backend boundary |
| Trigger | AR-0022 established a narrow opaque textured-mesh path, while its alpha audit found that `Textured3d` combines source-alpha blending with depth writes and has no cutout threshold or ordering policy. |
| Related ADRs | ADR-0001, ADR-0003, ADR-0008, ADR-0009, ADR-0012, ADR-0013 |
| Related evidence | AR-0006, AR-0022 Cycle 10, `tokimu-render` pipeline/shader/backend code, DOOM Slice 5B source coverage facts, `docs/Plans/textured-surface-alpha-policy-comparative-corpus.md` |
| Admission exception | None |

## Architectural Question

What is the smallest provider-neutral alpha and depth policy, if any, that
`tokimu-render` should admit for textured 3D surfaces without treating encoded
image alpha or Doom masked-middle coverage as renderer-owned semantics?

## Context

The first textured Box corpus profile is intentionally opaque. Its PNG inputs
contain no transparency, and AR-0022 correctly proved supplied UV and sampler
behavior without making an alpha claim.

The renderer already carries two relevant mechanisms: textured shaders preserve
the sampled alpha channel, and `BlendMode::AlphaBlend` maps to conventional
WGPU source-alpha blending. The default `Textured3d` state, however, is
`depth_writing_3d()`, which retains depth writes while using `AlphaBlend`.
That does not define ordering or depth semantics for general transparent
surfaces. Cutout/alpha test has no threshold vocabulary at all.

DOOM adds real future pressure: raster decoding can retain covered pixels as
straight alpha, but a masked middle texture is a source/application fact, not a
license for the renderer to infer discard, blend ordering, or a magic threshold.

## Trigger And Evidence

- Corpus examples:
  - `hello-textured-box` and `hello-textured-box-web` use an opaque initial
    profile and deliberately defer alpha.
  - DOOM Slice 5B needs to classify masked middle behavior before static E1M1
    presentation, but has not selected its policy.
- Automated tests:
  - RGBA8 validation preserves payload bytes; material/pipeline tests cover
    declared blend state and WGPU maps `AlphaBlend` to `ALPHA_BLENDING`.
  - No corpus test proves correct transparent ordering, depth-write behavior,
    or cutout threshold behavior.
- Audits or diagnostics:
  - AR-0022 Cycle 10 identifies the blend-plus-depth-write combination as an
    explicit blocker, not a failure of PNG decoding.
- Missing evidence:
  - A real transparent/cutout source fixture with documented intent.
  - A caller-defined rule for cutout threshold or transparent draw ordering.
  - Native and browser presentation evidence for any selected policy.

## Ownership Analysis

- Asset/format providers own decoded straight-alpha pixels and source coverage
  observations. They do not choose GPU discard or ordering.
- A corpus application owns whether its selected material requests an admitted
  generic alpha policy and owns render ordering where the policy requires it.
- `tokimu-render` may own only a bounded provider-neutral pipeline/material
  declaration and validation for an admitted policy, plus backend realization.
- The WGPU backend owns blend/depth state realization, not alpha semantics.
- Doom owns classification of masked middles, palette selection, and any
  original-behavior claim. The renderer must not receive WAD terms or infer
  Doom policy from alpha bytes.
- This boundary must not become a general material graph, order-independent
  transparency system, source-format alpha heuristic, or renderer-owned scene
  sorting service.

## Dependency Direction

```text
Decoded source alpha / source coverage
        ↓ observed fact
Corpus application selects an admitted generic policy and draw order
        ↓ provider-neutral declaration
tokimu-render validates policy → WGPU realizes blend/depth state
```

No PNG, GLB, WAD, palette, or coverage-provider type crosses into
`tokimu-render`.

## Alternatives Considered

### Alternative A: Keep Textured3d Opaque Only

- Benefits: preserves current demonstrated semantics; no ordering or threshold
  contract is implied.
- Costs: transparent and masked surfaces remain unavailable to future callers.
- Failure mode: callers create ad hoc shaders or silently misuse alpha blend.

### Alternative B: Admit A Bounded Cutout Policy

- Benefits: potentially fits masked/categorical surfaces without transparent
  ordering; can retain depth writes.
- Costs: requires a precise, provider-neutral threshold and source fixture;
  threshold selection can accidentally encode source-format or Doom behavior.
- Failure mode: a universal-looking threshold becomes a hidden asset policy.

### Alternative C: Admit A Bounded Blended-Surface Policy

- Benefits: preserves continuous source alpha for a real transparent caller.
- Costs: must explicitly define depth-write and draw-order responsibility;
  needs native/browser evidence.
- Failure mode: a convenient pipeline default is mistaken for correct
  transparency ordering.

### Alternative D: Admit General Transparency/Material Infrastructure Now

- Benefits: broad future flexibility.
- Costs: no current evidence supports sorting services, OIT, material graphs,
  texture arrays, or source-format rules.
- Failure mode: solving an unspecified future renderer instead of one caller.

## Findings

1. Straight alpha is an input fact, not a rendering-policy decision.
2. The existing `AlphaBlend` backend mapping is implementation capability, not
   sufficient semantic proof for transparent 3D surfaces.
3. Opaque textured rendering is independently useful and must remain usable
   without this review choosing alpha behavior.
4. Cutout and blended surfaces are separate alternatives: one need not admit
   the other.
5. If cutout is admitted, its discard threshold must be a caller-declared
   generic input or be justified by the caller's explicit contract. The
   renderer must not silently canonize a conventional value such as `0.5`.
6. If blending is admitted, the policy must state whether the application
   orders blended draws or the renderer owns an explicit sorting contract.
   There is no meaningful implicit middle ground.
7. The current evidence does not justify changing the `Textured3d` default or
   adding a source fixture yet.

## Disposition

Accepted in part. ADR-0013 admits a narrow caller-declared categorical-cutout
capability for implementation. It requires an explicit finite threshold and
explicit comparison, uses ordinary opaque depth behavior for retained
fragments, and must not infer intent from source alpha. It is not a member of a
shared `AlphaPolicy` type.

Blend remains an existing renderer mechanism and valuable corpus evidence, not
an admitted general blended-3D capability. Its ordering and depth ownership
remain incubating; no stable blend API, sorting service, or material system is
authorized by this review.

## Consequences

- AR-0022 can progress and receive a UV/sampler decision independently of
  alpha.
- DOOM Slice 5B must keep masked-middle behavior visibly deferred until this
  review records a decision that actually fits it.
- Any future policy must provide typed validation and retain native/browser
  presentation evidence; a decoder result alone is insufficient.

## Required Follow-Up

- [x] Design a bounded comparative corpus that holds RGBA8 input constant while
      varying opaque, cutout, blend, depth-write, and caller-order declarations.
- [x] Execute the shared synthetic fixture and interaction matrix.
- [x] Exercise Doom masked middles as independent real cutout pressure.
- [x] Identify a separate first-party continuous-alpha caller and retain Blend
      under incubation rather than admitting it.
- [x] State and test threshold comparison, draw-order, and depth-write
      responsibility before proposing stable vocabulary.
- [x] Apply ADR-0008 and ADR-0009 pre-admission gates; reopen them for the
      concrete Cutout implementation.
- [x] Create ADR-0013 for the accepted Cutout contract.

## Reopening Triggers

- a corpus consumer needs visible transparent or categorical masked pixels;
- DOOM Slice 5B selects a concrete masked-middle requirement;
- a native/WASM backend cannot preserve a chosen bounded policy;
- source alpha starts being used as an implicit blend/cutout selector; or
- a cutout threshold is silently assumed rather than declared by a caller; or
- blended draws rely on unspecified ordering or depth-write behavior; or
- a simpler opaque-only decomposition satisfies the caller.

## Related Failure Observation

The native Slice 3 blend fixture currently submits and presents accepted work
but initially displayed an empty frame. AR-0024 localized the cause to the
camera/WebGPU clip-depth convention rather than alpha policy. AR-0024 accepted
explicit conversion at the WGPU upload boundary, and the fixture again uses
positive GL depths as regression pressure. Visual recovery evidence remains
part of this alpha study; the camera decision is preserved separately in
[AR-0024: Renderer Failure Observation And Diagnostic Boundary](AR-0024-renderer-failure-observation-and-diagnostic-boundary.md).

## Review History

### Cycle 1 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: AR-0022 Cycle 10's alpha implementation audit; current
  `Textured3d` shader and render-state implementation.
- Participants or reviewers: project maintainer and Codex implementation
  review.
- Findings: the alpha data path is explicit, but transparent/cutout semantics
  are not selected; opaque presentation remains the honest current profile.
- Disposition: Proposed; no renderer contract is admitted.
- Resulting ADR or documentation change: opened this record and linked it from
  AR-0022 and the DOOM Slice 5B plan.

### Cycle 2 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: independent architectural review of the initial record.
- Findings: cutout and blending are not only different implementation paths;
  they create different ownership questions. A cutout threshold cannot become
  canonical merely through convention, and blending must explicitly allocate
  draw-order responsibility to either the application or a renderer sorting
  contract.
- Disposition: retain Proposed and the opaque-only current profile. The review
  now records threshold ownership and blended-draw ordering as mandatory
  admission questions rather than downstream implementation details.
- Resulting ADR or documentation change: none.

### Cycle 3 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: maintainer discussion found that choosing cutout first could
  stabilize two contracts without ever testing their interaction, shared
  vocabulary, depth semantics, or ordering ownership against the same inputs.
- Findings: a comparative corpus should hold image data constant while varying
  declared opaque, cutout, and blend policy; Doom can supply real cutout
  pressure, while blending requires a separate continuous-alpha caller.
- Disposition: retain opaque-only admission while the comparative study runs.
  Do not design the public enum before the fixtures expose whether the
  capabilities are genuinely related.
- Resulting ADR or documentation change: added the Textured Surface
  Alpha-Policy Comparative Corpus plan.

### Cycle 4 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: Slice 0 and the headless portion of Slice 1 now provide six
  manifest-locked exact RGBA8 fixtures, seven fixed scene identities, explicit
  `<` and `<=` threshold candidates, caller-order observations, conventional
  straight-alpha reference contribution, and typed negative evidence.
- Findings: the exact `128/255` fixture makes threshold equality observable;
  opaque, cutout, and blend produce distinct semantics from identical bytes.
  Initial scene generation incorrectly tied transforms to submission index;
  the retained regression now proves reversed blend cases change only caller
  order. The headless evidence still does not justify public renderer
  vocabulary.
- Disposition: continue the corpus-local study. Retain opaque-only admission
  and return to this review before introducing a stable renderer crossing.
- Resulting ADR or documentation change: added `hello-alpha-policy` and its
  browser boundary record; no ADR change.

### Cycle 5 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: the corpus now has an optional native Slice 2 visual target.
  It realizes the frozen mixed-alpha source under an explicit opaque control
  and two labelled cutout candidates (`discard below 128/255` and `discard at
  or below 128/255`). A second panel places the binary mask over an opaque
  background with declared depth writes. The target uses existing
  `Pipeline::custom_wgsl` support, supplied UVs, and explicit opaque depth
  state; no `tokimu-render` public type, default, or shader was changed.
- Findings: the equality decision is visible in a real backend realization
  without allowing a conventional threshold to masquerade as a public
  default. A redundant private WGPU mesh cache was exposed by the focused
  strict lint check and removed; adding the optional target also exposed an
  ambiguous default corpus executable, repaired by declaring the headless
  report as the crate's `default-run`. Mesh/shader compatibility continues to
  be checked from the source mesh before upload.
- Disposition: retain Proposed and opaque-only admission. Native visual
  observation, zero/one threshold cases, negative backend validation, blend
  comparison, and browser/WASM evidence remain required before any contract
  decision.
- Resulting ADR or documentation change: none; this is a corpus-local
  experimental realization only.

### Cycle 6 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: the native Vulkan target (AMD Radeon RX 7900 XTX) presented
  the Slice 2 scene at 960 × 600. The opaque control displayed all five
  `mixed-alpha` texels; `discard below 128/255` retained the exact 128-alpha
  texel; and `discard at or below 128/255` omitted it. The binary-mask depth
  panel exposed its opaque blue backing through discarded pixels.
- Findings: this matches the frozen headless classification and makes the
  comparison boundary observable on a real GPU. It is one native observation,
  not a cross-target or cross-vendor guarantee or a basis for selecting a
  default threshold. Current direct hardware access covers AMD/Vulkan and
  Apple/Metal, but not NVIDIA; NVIDIA alpha/depth behavior remains explicitly
  unverified and may contain gaps.
- Disposition: continue Slice 2 with the same shared source on browser/WASM;
  retain opaque-only admission and do not promote the custom-WGSL candidates.
- Resulting ADR or documentation change: retained the native visual record in
  `corpus/hello-alpha-policy/results/`.

### Cycle 7 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: `hello-alpha-policy-web` now compiles for
  `wasm32-unknown-unknown`, generates browser bindings, and serves its host
  page locally. Native and browser targets import one shared cutout WGSL
  generator plus the exact fixture, threshold, viewport, depth, and visual
  layout values from `hello-alpha-policy`.
- Findings: sharing only the headless classification was insufficient to
  prevent target-specific visual layout or shader drift, so those executable
  study inputs were centralized before browser execution. Browser compilation,
  generated bindings, and HTTP availability have been established separately;
  adapter/device readiness and first presentation have not yet been observed.
- Disposition: continue Slice 2 and retain opaque-only admission. A browser
  observation is required before claiming native/WebGPU categorical agreement.
- Resulting ADR or documentation change: promoted the browser boundary record
  into an executable corpus member without changing `tokimu-render` contracts.

### Cycle 8 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: the browser/WASM alpha fixture reached browser adapter/device
  preflight and Tokimu first presentation at 960 × 600. Its opaque, `<`, and
  `<=` panels agree with the native AMD/Vulkan observation: the exact
  128-alpha texel remains under `< 128/255` and is discarded under
  `<= 128/255`; binary-mask holes expose the opaque depth backing.
- Findings: the shared fixture, threshold, shader, layout, and declared depth
  state produce the same categorical outcome on the currently tested native
  and browser paths. Browser WebGPU supplies no useful adapter name in this
  capture, so the retained record leaves it unavailable. This remains limited
  vendor coverage; it is not NVIDIA evidence.
- Disposition: Slice 2 cross-target interior-threshold evidence is satisfied.
  Continue the zero/one threshold and negative backend-validation cases before
  revisiting a renderer contract; retain opaque-only admission.
- Resulting ADR or documentation change: retained the browser visual record in
  `corpus/hello-alpha-policy-web/results/`.

### Cycle 9 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: `hello-alpha-policy` now has explicit 0.0 and 1.0 threshold
  boundary tests in addition to its interior-threshold cases. A missing custom
  candidate shader is retained as a typed pipeline-validation rejection. The
  established `hello-shader --backend-diagnostic-fixture` additionally passed
  on the native backend: deliberate invalid WGSL was reported through the WGPU
  diagnostic sink with its module and entry-point identities preserved, and was
  never submitted.
- Findings: alpha policy can rely on the existing generic malformed-WGSL
  diagnostic corpus rather than duplicating an intentionally invalid shader
  path. The alpha-specific evidence remains focused on whether valid declared
  policies differ in fragment/depth behavior.
- Disposition: Slice 2 validation/failure evidence is satisfied. Keep its
  remaining 0/1 GPU visual observations open; do not treat headless boundary
  arithmetic as a public threshold decision.
- Resulting ADR or documentation change: none.

### Cycle 10 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: the corpus now contains `native_blend_scene`, a separate
  Slice 3 executable that uses the same first-party `mixed-alpha` RGBA8 source
  for four panels. It retains two caller submission sequences (far-then-near
  and near-then-far) and two explicitly chosen straight-alpha depth-write
  states. It uses corpus-local custom WGSL which preserves sampled alpha; the
  alpha equation remains the explicit `AlphaBlend` pipeline state. Focused
  tests and no-dependency Clippy pass.
- Findings: implementation can hold source data, UVs, transforms, camera, and
  blend equation fixed while varying only caller order or declared depth-write
  state. No renderer ordering service or public alpha-policy vocabulary was
  introduced. Native presentation and browser realization are still required;
  executable construction alone is not blend conformance evidence.
- Disposition: continue Slice 3 with native visual observation, then mirror the
  same frozen case on browser/WASM. Opaque-only remains the admitted profile.
- Resulting ADR or documentation change: none.

### Cycle 11 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: after AR-0024's accepted provider-boundary depth conversion,
  the native AMD Radeon RX 7900 XTX/Vulkan fixture visibly presented all four
  blend comparison panels and its opaque control at 960 × 600. The first frame
  retained eleven draws, eleven material resolutions, six pipeline switches,
  and no backend diagnostic. Reversed caller order produced visibly different
  color bands; explicit depth writes on/off also produced distinct results.
- Findings: the native fixture now demonstrates that caller order and depth-
  write state are independently observable while source RGBA8, UVs, geometry,
  camera, transforms, and blend equation remain fixed. The successful recovery
  also confirms AR-0024 was a provider mapping defect rather than an alpha
  failure. This is one manual AMD/Vulkan observation, not browser, NVIDIA, or
  general transparency-order evidence.
- Disposition: close the Slice 3 native-presentation checkpoint and continue
  the continuous-gradient, diagnosable caller-intent, recovery, and browser
  comparison work. Retain opaque-only admission until the comparative study
  reaches its acceptance criteria.
- Resulting ADR or documentation change: retained
  `corpus/hello-alpha-policy/results/native-blend-observation-2026-08-09.md`.

### Cycle 12 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: the shared corpus crate now owns the Slice 3 blend layout,
  depths, offsets, and control location, which native and browser fixtures
  import unchanged. Native adds the 256-byte continuous-gradient-over-opaque
  control beside the `mixed-alpha` order/depth panels. Both targets deliberately
  reject an invalid `depth_write` plus `DepthTest::Disabled` declaration before
  constructing the valid fixed scene. The browser module now exposes
  `?mode=blend` and generated bindings; native focused tests, browser/WASM
  compilation, and affected no-dependency Clippy pass.
- Findings: the corpus can make caller order visible through its typed
  `CallerOrdering` records and frozen scene manifest without admitting an
  ordering service to `tokimu-render`. The malformed state is rejected before
  backend registration and does not prevent later valid setup. These are source
  and compilation facts, not a substitute for a post-change native or browser
  first-presentation observation.
- Disposition: continue with the two manual visual observations. Do not infer
  browser presentation, general transparency correctness, or a stable blend
  contract from generated bindings or native source construction.
- Resulting ADR or documentation change: expanded the Slice 3 corpus only; no
  renderer contract changed.

### Cycle 13 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: the browser/WASM `?mode=blend` fixture reached first
  presentation at 960 × 600 with twelve draws, twelve material resolutions,
  six pipeline switches, and no backend diagnostic. The browser identified its
  backend as browser WebGPU, device kind as `other`, and adapter as unavailable.
  The continuous-gradient control, both caller-order variants, and both
  depth-write variants were visibly distinct.
- Findings: the exact native/browser shared comparison reaches a second target
  without a renderer-owned ordering service. A valid scene has no ordering
  diagnostic to capture; instead the page and retained observation make caller
  ordering explicit corpus input. This remains one browser path and must not be
  generalized to browser vendors, adapters, NVIDIA, or a stable blend API.
- Disposition: close the browser Slice 3 presentation checkpoint and retain the
  browser observation. Native re-observation of the newly added continuous
  gradient is still required. Slice 3 remains Proposed while the study decides
  whether corpus-only ordering evidence is sufficient for any future public
  contract discussion.
- Resulting ADR or documentation change: retained
  `corpus/hello-alpha-policy-web/results/browser-blend-observation-2026-08-09.md`.

### Cycle 14 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: before considering render-order, shader-resource, batching, or
  PBR vocabulary, the Slice 3 fixtures now retain the existing renderer's
  per-frame material resolutions, pipeline switches, binding allocations,
  uniform writes, and mesh uploads. The native fixture moved fixed mesh and
  initial-camera uploads outside its frame loop. The browser fixture presents
  its unchanged command array once for setup and once for a warm-frame
  observation.
- Findings: the present implementation submits queued draws in caller order;
  it selects pipelines as that order requires and resolves material/camera/
  instance bindings per draw. The retained counters measure current provider
  work without asserting that WGPU bind groups, batching, or a render queue are
  provider-neutral semantics. This instrumentation exposes present behavior;
  it does not establish a need for renderer reordering or a public shader
  resource model.
- Disposition: retain AR-0023 scope. Collect the updated native and browser
  first-versus-warm observations and the order-invariance evidence for opaque
  and cutout before opening a separate review. Escalate only if callers need a
  stable ordering declaration, the renderer must schedule/reorder submissions,
  or shader resources cannot remain corpus-local.
- Resulting ADR or documentation change: refined the comparative corpus plan
  and fixture instrumentation only; no renderer contract changed.

### Cycle 15 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: browser/WASM presented the unchanged Slice 3 command array on
  both its first and warm frames. The first frame reported twelve draws,
  twelve material resolutions, six pipeline switches, thirteen binding
  allocations, and zero mesh uploads. The warm frame reported the same draws,
  material resolutions, and pipeline switches, but zero binding allocations
  and zero mesh uploads; the backend diagnostic remained `none`.
- Findings: current browser/WGPU setup allocates its camera plus per-draw
  instance bindings once, then reuses them across the identical warm frame.
  Material resolution and pipeline selection remain submission-time work in
  the current implementation. This is useful performance evidence, not proof
  that binding groups are a provider-neutral concept or that batching or draw
  reordering is required.
- Disposition: retain the browser first-versus-warm observation and continue
  Slice 3. Native observation of the same static-fixture refinement remains
  required. Do not open a render-order, scheduler, shader-resource, or PBR
  review from this one corpus result.
- Resulting ADR or documentation change: updated
  `corpus/hello-alpha-policy-web/results/browser-blend-observation-2026-08-09.md`
  and the comparative corpus plan.

### Cycle 16 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: the corpus-only fixed `LessEqual` depth oracle now composes
  the same background/far/near layers in both far-then-near and near-then-far
  submission orders. Its test proves that opaque and retained-cutout profiles
  with depth writes produce the same result, while no-depth-write Blend
  produces different results.
- Findings: under this deliberately bounded depth model, order sensitivity is
  pressure specific to continuous blending rather than a generic property of
  opaque or categorical coverage. The result does not prove backend raster
  behavior, solve intersecting transparent geometry, or justify renderer-owned
  draw reordering.
- Disposition: retain caller ordering as corpus input for Blend and continue
  the visual interaction matrix. No separate render-order or shader-resource
  review is warranted by the current evidence.
- Resulting ADR or documentation change: added the corpus-local
  `opaque_and_cutout_are_depth_order_invariant_while_blend_is_not` regression
  test and refined the comparative plan.

### Cycle 17 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: native AMD/Vulkan presented the expanded continuous-gradient
  plus order/depth fixture. First frame reported twelve draws, twelve material
  resolutions, six pipeline switches, thirteen binding allocations, zero
  uniform writes, zero mesh uploads, and no diagnostic. Its identical warm
  frame retained the same draw/material/pipeline work with zero binding
  allocations, uniform writes, and mesh uploads. Both browser/WASM and native
  intentionally rejected invalid depth-write-without-depth-test state before
  later valid presentation.
- Findings: the two observed WGPU targets agree on the fixed scene's current
  first-versus-warm reuse behavior. Exact caller input and typed absent/empty/
  invalid caller-order rejections make responsibility visible in corpus
  evidence. A valid renderer frame correctly has no error diagnostic; the
  evidence does not require a public Blend-order diagnostic.
- Disposition: Slice 3 is complete. Retain opaque-only admission while Slice 4
  exercises cutout/blend interaction. Do not open a separate ordering,
  batching, shader-resource, or PBR review from the present evidence.
- Resulting ADR or documentation change: updated
  `native-blend-observation-2026-08-09.md` and closed Slice 3 acceptance
  criteria in the comparative plan.

### Cycle 18 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: Slice 4 now has one fixed corpus-local interaction manifest,
  fingerprinted as
  `0a99c714c258bac7f91eb5dd39748651abca8db96bfc1a410d823a18d2c23d93`.
  It locks the binary-mask and mixed-alpha fixture identities, viewport,
  panel transforms, backing/fixed/sloped depths, and seven submissions:
  cutout over opaque, Blend over opaque, then cutout and a depth-writing
  sloped Blend crossing over opaque. Native AMD/Vulkan visibly presented that
  scene with seven draws and no diagnostic. Browser/WASM now imports the same
  constants and realization under `?mode=interaction`; the wasm target build
  and generated binding export succeed.
- Findings: this makes source and structural drift observable before visual
  comparison. It does not establish browser presentation, target parity,
  general intersecting-transparency correctness, renderer-owned ordering, or
  a stable alpha contract.
- Disposition: retain opaque-only admission and continue Slice 4 with a
  browser first-presentation and manual fixed-camera observation. Compare the
  browser result to the retained native observation only after both targets
  report the same locked manifest.
- Resulting ADR or documentation change: expanded the comparative corpus plan
  and browser design note; no renderer contract changed.

### Cycle 19 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: browser/WASM `?mode=interaction` reached first presentation
  at 960 × 600 after adapter/device preflight. It reported the same locked
  interaction manifest as native,
  `0a99c714c258bac7f91eb5dd39748651abca8db96bfc1a410d823a18d2c23d93`,
  with seven draws, seven material resolutions, seven pipeline switches, and
  `diagnostic=none`. Its visible cutout-over-opaque, Blend-over-opaque, and
  depth-crossing cutout/Blend panels match the bounded native AMD/Vulkan
  observation. Browser identity is retained as browser WebGPU, device `other`,
  adapter unavailable; NVIDIA remains an explicit coverage gap.
- Findings: the available targets agree on the same scene input, manifest,
  policy/depth declarations, first-presentation status, and three visual
  distinctions. That is cross-target evidence for the corpus fixture, not
  proof of pixel equality, arbitrary transparency intersections, renderer
  sorting, or public alpha vocabulary.
- Disposition: complete Slice 4 for available targets. Retain opaque-only
  admission and proceed to independent real-caller pressure in Slice 5; do
  not let this successful synthetic comparison settle AR-0023's cutout or
  Blend disposition.
- Resulting ADR or documentation change: retained
  `browser-interaction-observation-2026-08-09.md` and closed available-target
  Slice 4 checklist evidence.

### Cycle 20 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: the reviewed shareware E1M1 package (BLAKE3
  `58146f5aa0e14ef38047a79878307344aec821b9f312da6a9208ec08e399660c`)
  has 13 retained source-classified two-sided masked middles. Their four
  texture definitions are `BRNBIGC`, `BRNBIGL`, `BRNBIGR`, and `BROWNGRN`.
  The first three have 4,546, 213, and 303 uncovered pixels respectively;
  `BROWNGRN` is fully covered. The Doom consumer now lowers the retained
  classifications into 26 non-degenerate corpus-local cutout candidates with
  a declared RGBA8 binary-coverage rule: discard at or below alpha zero and
  write depth. No candidate is uploaded to or drawn by the opaque E1M1 scene.
- Findings: source classification is independently meaningful from raster
  coverage: the fully covered `BROWNGRN` observation still becomes a masked
  middle candidate. This is real cutout pressure, not source-alpha inference.
  The renderer sees no WAD terms and no new public alpha vocabulary. Confirmed
  degenerate candidates remain separately retained; unrelated lowering errors
  still fail preparation.
- Disposition: complete the real cutout-pressure portion of Slice 5 while
  retaining opaque-only admission. The selected declaration is corpus-local;
  native/browser presentation and a separate continuous-alpha caller remain
  required before a cutout or Blend contract can be proposed.
- Resulting ADR or documentation change: retained
  `docs/Plans/DOOM/E1M1 masked-middle cutout intake evidence.md`; no ADR or
  renderer contract changed.

### Cycle 21 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: `hello-glb` is an independent first-party continuous-alpha
  consumer. Its application-owned presentation override changes the retained
  Khronos Box's resolved opacity to `0.35` for interactive inspection without
  mutating the source material. That state selects an explicit alpha-blend,
  depth-tested, depth-write-disabled, two-sided diagnostic pipeline and
  retains the limitation that intersecting transparent geometry is not
  guaranteed beyond submission order. The focused `hello-glb` policy tests
  pass.
- Findings: this satisfies the need for a non-Doom caller with continuous
  alpha, but it does not add a new renderer interface: it uses the already
  explicit pipeline state locally. Its structural mapping matches the shared
  Blend study profile with caller ordering and no depth writes. Native visual
  evidence for the opaque Box path exists; a fixed translucent observation and
  browser/WASM availability remain outstanding before treating this caller as
  cross-target blend evidence.
- Disposition: retain opaque-only admission. Continue Slice 5 by comparing the
  two real callers to the shared fixture with retained observations; do not
  interpret existing local state selection as a stable Blend API.
- Resulting ADR or documentation change: recorded `hello-glb` as the selected
  independent continuous-alpha caller in the comparative corpus plan and
  retained `docs/Plans/alpha-policy-real-caller-comparison.md`.

### Cycle 22 -- 2026-08-09

- Evidence: the E1M1 experimental masked-middle path now reaches native WGPU
  first-frame initialization on AMD Radeon RX 7900 XTX/Vulkan with 1,835
  unchanged opaque draws plus 26 source-selected cutout candidates. Its custom
  corpus WGSL declares binary discard at alpha zero and ordinary depth writes;
  no Doom type or alpha inference reaches `tokimu-render`.
- Ordinary finding and repair: `BROWNGRN` is a source-classified masked middle
  with fully covered current pixels. The initial experimental uploader selected
  only deferred-alpha textures and therefore failed with a missing material.
  The repaired candidate selector follows retained source classification and
  includes both fully covered and uncovered selected textures. A focused test
  prevents coverage bytes from becoming the hidden selection policy.
- Limits: successful native initialization is not a visual cutout observation,
  browser/WASM comparison, or capability admission. Fixed-camera visual and
  browser evidence remain required. `hello-glb` remains the independent
  continuous-alpha caller.
- Disposition: retain opaque-only admission and continue Slice 5. The finding
  is local consumer integration evidence, not grounds for a renderer API or a
  new architectural review.

### Cycle 23 -- 2026-08-09

- Evidence: the browser workbench visibly presented the same explicitly local
  reviewed E1M1 package through both fixed-camera actions: 1,835 opaque draws
  and 1,861 masked-cutout draws. The returned browser metadata was
  `backend=browser-webgpu`, `device=other`, blank adapter name, and a `960x600`
  canvas. The normal opaque request and experimental cutout request are
  separate Rust/WASM entries; TypeScript supplies only local bytes, a canvas,
  and presentation controls.
- Findings: this is the first real-caller browser observation for categorical
  cutout. It agrees with the retained native structural delta and the shared
  fixture's explicit opaque/depth-writing profile. It establishes neither
  pixel equality, arbitrary alpha behavior, automatic alpha classification,
  renderer sorting, nor a public cutout capability.
- Disposition: retain opaque-only admission. The E1M1 cutout caller has enough
  bounded native/browser integration evidence to proceed toward Slice 6 review;
  Slice 5 still needs the separately retained GLB continuous-alpha visual
  comparison before the two caller pressures are considered together.

### Cycle 24 -- 2026-08-09

- Evidence: the `hello-glb -- --transparent` capture entry now freezes the
  ordinary orbit/mesh transforms while selecting its existing application-owned
  `0.35` opacity override. On AMD Radeon RX 7900 XTX/Vulkan it initializes the
  explicit alpha-blend, depth-tested, depth-write-disabled pipeline and
  presents two-draw warm frames with zero new binding allocations, uniform
  writes, or mesh uploads.
- Findings: a fixed capture can be structurally stable without inventing a
  renderer capture/readback feature. The GLB source material remains untouched;
  the application override and its admitted limitation—submission order is the
  only ordering guarantee for intersecting transparent geometry—remain visible
  in the retained artifact.
- Limits: this is not a human visual observation, browser/WASM blend
  conformance, a general sorting result, or a public Blend admission.
- Disposition: retain opaque-only admission. Continue Slice 5 with the two
  remaining fixed-camera visual observations before comparing both real callers
  as a complete evidence set.

### Cycle 25 -- 2026-08-09

- Evidence: a reviewer visibly observed the frozen native `hello-glb
  -- --transparent` capture: the retained Box is continuously translucent over
  its opaque floor, and the title reports `presentation=transparent`. This
  joins E1M1's browser/WebGPU categorical-cutout observation as the completed
  real-caller comparison set.
- Findings: the callers place distinct pressure on the shared fixture results.
  Doom needs categorical visibility with ordinary depth writes; GLB inspection
  needs continuous contribution with explicit no-depth-write and only
  submission-order intersection limits. Neither requires a format-aware
  renderer API, shader-resource contract, PBR system, or renderer-owned
  transparent scheduling service. The evidence does not force a common public
  alpha-policy enum.
- Coverage limits: E1M1 is observed on browser/WebGPU and native startup;
  GLB is observed on native AMD/Vulkan only. The shared fixture—not a browser
  GLB application—supplies browser/WASM Blend evidence. NVIDIA remains the
  repository-wide uncovered adapter target.
- Disposition: Slice 5 is complete. Retain opaque-only admission while Slice 6
  applies the ADR-0008/0009 gates and compares separate-versus-common candidate
  contract pressure before AR-0023 chooses an admission outcome.

### Cycle 26 -- 2026-08-09

- Evidence: Slice 6 retained an ADR-0008/0009 pre-admission ledger covering
  ownership, hot-path shape, bounded inputs, target coverage, warm-frame
  observations, rejection/recovery, diagnostics, and containment. The corpus
  added no stable alpha API to `tokimu-render`; its custom shaders and existing
  pipeline state remain experimental machinery.
- Findings: the full gate is not honestly completable as an implementation
  gate without a chosen public contract. That is recorded as `N/A` with a local
  reason, not as a waiver. The evidence rules out a shared alpha-policy type:
  cutout owns threshold/comparison and categorical discard, while Blend owns
  continuous contribution plus caller-visible depth/order responsibility.
- Disposition: complete the Slice 6 pre-admission gate review. A maintainer
  decision is now required before implementation continues: retain/defer,
  propose cutout, propose Blend, or move either concern outward. Any chosen
  stable crossing must reopen the ledger with measurements and full concrete
  ADR-0008/0009 answers.

### Cycle 27 -- 2026-08-09

- Evidence: focused validation passed: 18 headless alpha-oracle tests, 25
  feature-enabled alpha corpus tests, 56 `tokimu-render` tests, two relevant
  WASM checks, and strict focused Clippy for the alpha corpus and renderer.
- Ordinary finding and repair: headless `cargo test -p hello-alpha-policy`
  initially auto-discovered visual binaries whose optional `tokimu` dependency
  was unavailable. Explicit `[[bin]]` declarations now bind all three visual
  binaries to `native-visual`. Strict Clippy then identified three literal
  depth-order assertions; they are now compile-time invariants, leaving runtime
  tests to exercise actual scene behavior.
- Findings: the final Slice 6 validation improves corpus hygiene and does not
  change alpha ownership or manufacture a stable API. The remaining decision
  is architectural, not an implementation or evidence blocker.
- Disposition: retain Proposed/opaque-only status pending maintainer choice.

### Cycle 28 -- 2026-08-09

- Decision: the maintainer accepted the comparative result rather than the
  earlier plausible shared enum. ADR-0013 authorizes implementation of only a
  narrow caller-declared categorical-cutout capability: finite inclusive
  threshold, explicit below/at-or-below comparison, categorical discard, and
  ordinary opaque depth behavior for retained fragments.
- Blend: continuous Blend remains a renderer mechanism and corpus-study
  profile. It has useful evidence, including a separate translucent GLB caller,
  but its public ordering and depth contract is not admitted. No sorting,
  material-system, PBR, source-format, or shared alpha-policy work follows
  from this decision.
- Required next work: implement Cutout through the focused admission plan,
  reopen the concrete ADR-0008/0009 gates, replace the corpus-local cutout WGSL
  path, and migrate E1M1 while retaining source classification outside the
  renderer.
- Disposition: accepted in part — Cutout implementation is authorized; Blend
  remains incubating.

### Cycle 29 -- 2026-08-09

- Implementation: `tokimu-render` now has dedicated `CategoricalCutout`,
  checked `CutoutThreshold`, and `CutoutComparison` vocabulary plus
  `Pipeline::textured_3d_cutout`. The constructor specializes renderer-owned
  textured WGSL, fixes opaque color output and ordinary depth write/test state,
  and cannot be confused with `BlendMode`.
- Failure behavior: non-finite/out-of-range thresholds, a cutout kind without
  a declaration, and attempts to relax Cutout into blend/no-depth state reject
  before backend submission. The focused renderer suite contains 61 passing
  tests. Native alpha corpus tests pass with its visual targets compiled, and
  the browser alpha consumer compiles for `wasm32-unknown-unknown`.
- Migration: the shared alpha fixture's native and browser Cutout profiles now
  call the admitted renderer capability. Native/browser visual observations of
  that replacement, E1M1 migration, and full warm-frame measurements remain
  open; the earlier custom-WGSL observations are retained as pre-admission
  evidence rather than relabeled as proof of this implementation.

## References

- `docs/Architectural Reviews/AR-0006-raster-image-requirement-pipeline.md`
- `docs/Architectural Reviews/AR-0022-textured-mesh-coordinate-and-sampling-boundary.md`
- `docs/Plans/textured-box-glb-png-corpus.md`
- `docs/Plans/textured-surface-alpha-policy-comparative-corpus.md`
- `docs/Plans/categorical-cutout-capability-admission.md`
- `docs/ADR/ADR-0013-caller-declared-categorical-cutout-surfaces.md`
- `docs/Plans/DOOM/DOOM WAD Checklist.md`
- `crates/tokimu-render/src/pipeline.rs`
- `crates/tokimu-render/src/wgpu_backend/pipeline_support.rs`
