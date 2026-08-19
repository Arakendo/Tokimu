# AR-0024: Renderer Failure Observation And Diagnostic Boundary

| Field | Value |
| --- | --- |
| Status | Accepted |
| Opened | 2026-08-09 |
| Last reviewed | 2026-08-09 |
| Scope | `tokimu-render` validation, WGPU adapter diagnostics, and corpus failure evidence |
| Trigger | The AR-0023 native blend comparison submits and presents eleven accepted draws but produces an empty visible frame; its first-frame diagnostic drain is not yet producing an actionable failure explanation. |
| Related ADRs | ADR-0007, ADR-0008, ADR-0009 |
| Related reviews | AR-0021, AR-0023 |
| Admission exception | None |

## Architectural Question

When Tokimu accepts render commands and reaches `present()` but a corpus visual
assertion fails, what bounded, provider-neutral observation must be available
to distinguish command acceptance, resource preparation, pipeline validation,
GPU execution diagnostics, and visible-frame outcome without turning
`tokimu-render` into a general GPU debugger?

The investigation exposed a second, now-prerequisite question:

> Is Tokimu's public camera projection convention OpenGL `[-1, 1]`, WebGPU
> `[0, 1]`, or a provider-neutral convention which each rendering adapter must
> convert explicitly?

## Trigger And Retained Evidence

`corpus/campaigns/textured-presentation/hello-alpha-policy/src/bin/native_blend_scene.rs` is a corpus-local
Slice 3 fixture. On the native AMD Radeon RX 7900 XTX/Vulkan target it reports:

```text
first frame: 11 draws, 11 material resolutions, 6 pipeline switches
```

The window still presents only the configured black clear color. This persists
after retaining the native window and after adding an opaque reference draw.
The fixture now polls and drains the WGPU diagnostic sink after presentation,
but the observation has not yet yielded an actionable error. Therefore none of
the following are equivalent:

```text
commands accepted
    != resources resolved
    != backend validation reported
    != GPU execution diagnosed
    != expected pixels were visible
```

This is not evidence that alpha blending is broken, and it must not be used to
admit or reject the AR-0023 candidate policies. It is a failure-observation
gap discovered by that fixture.

## Ownership Analysis

- The corpus owns its fixed scene, expected visible distinctions, and retained
  observation record.
- `tokimu-render` owns provider-neutral declaration validation, command
  preparation accounting, and bounded diagnostic records already admitted by
  ADR-0007.
- The WGPU adapter owns asynchronous validation/error callbacks and any
  backend-native diagnostics. It must not leak WGPU-native types across the
  renderer boundary.
- The application/tooling layer may render or archive diagnostics; it does not
  infer a successful frame from a non-error return.
- A framebuffer capture/readback API is explicitly not assumed by this review.

## Alternatives Considered

### A. Retain Current Counters And Treat `present()` As Sufficient

Reject initially. The trigger demonstrates that accepted draw counts and a
successful presentation call do not localize an empty-frame failure.

### B. Add Corpus-Local Instrumentation Only

Useful as immediate investigation work. It may identify a local fixture defect,
but cannot establish whether current renderer diagnostics have a reusable
observation gap.

### C. Admit A Bounded Renderer Failure Observation Contract

Potentially appropriate only if multiple corpus paths need the same facts.
Candidate facts include validated declaration identity, prepared resource
identity/count, backend diagnostic records, and a distinct presentation result.
It must not promise pixel correctness or GPU completion unless the provider can
honestly observe those facts.

### D. Add Generic Framebuffer Readback Or GPU Debugging

Deferred. One failed fixture does not justify capture/readback, a graphics
debugger, or a backend-specific tooling surface.

## Camera Depth Alternatives Exposed By The Investigation

### E. Change Built-In Cameras To WebGPU-Native Depth

Change the built-in camera constructors to produce `[0, 1]` clip depth. This
aligns the current renderer implementation with its only admitted GPU provider,
but changes the meaning of public `Camera::projection` values. Existing CPU
consumers such as CAD unprojection, corpus-local explicit GL projections, and
the retained math-vocabulary conformance study would need deliberate migration.

### F. Preserve GL Camera Meaning And Convert At Provider Upload

Keep `Camera` and CPU-side projection/unprojection in the documented OpenGL
`[-1, 1]` convention, then apply an explicit GL-to-WebGPU depth conversion in
both WGPU camera-upload paths. This makes provider adaptation visible and
preserves current public matrix meaning, but the two upload paths must share
one tested conversion and future providers must declare their own clip-space
adaptation.

### G. Declare The Current Visible Half-Space As Intended

Document that only the part of a GL projection which already maps into WebGPU
`[0, 1]` is visible. Reject as a likely steady-state answer: it silently clips
valid camera-space geometry, wastes part of the declared near/far interval, and
makes ordinary scene depth placement depend on an undocumented adapter detail.

### H. Add A Public Runtime-Selected Clip Convention

Expose the convention on `Camera` and let callers select it. Defer absent
independent provider pressure: this would add public vocabulary and push a
backend compatibility concern onto ordinary scene authors before Tokimu has
demonstrated a second rendering API.

## Initial Findings

1. Current draw, material-resolution, and pipeline-switch counts are useful
   evidence of submitted work, but are insufficient as a visible-frame claim.
2. WGPU error callbacks may be asynchronous; a sink that is merely installed
   is not proof that a fixture will receive every relevant diagnostic at the
   point it checks.
3. Empty-frame investigation must retain causal stages separately and avoid
   collapsing “no reported error” into “correct rendering.”
4. AR-0023 remains blocked only for its blend evidence, not because it has
   selected a renderer architecture.
5. At the time of the trigger, Tokimu's built-in orthographic and perspective cameras constructed
   OpenGL `[-1, 1]` depth projections (`orthographic_rh_gl` and
   `perspective_rh_gl`). WGPU/WebGPU clip depth is `[0, 1]`; no conversion is
   visible between camera upload and shader execution. This turned valid,
   accepted geometry into silently clipped work. Cycle 4 records the accepted
   provider-boundary correction.

## Required Follow-Up

- [x] Reproduce the blend fixture with a known-good opaque control and retain
      the exact prepared command/resource/pipeline identities.
- [x] Determine whether the empty frame is a corpus-local scene defect, a
      renderer validation omission, or backend diagnostics that are not
      observed at the current lifecycle point.
  - [x] The fixture used positive world Z. The current GL projection maps it to
        negative clip Z, outside WebGPU's `[0, 1]` interval. This is valid
        clipping, so no backend validation error is expected.
  - [x] A focused corpus test preserves the mapping and requires the provisional
        negative-Z fixture depths to remain within WebGPU-visible clip depth.
- [x] Test a deliberate backend-invalid draw/pipeline beside the valid fixture
      to establish which diagnostic stages are currently observable.
  - [x] Reused the retained `hello-shader --backend-diagnostic-fixture`
        evidence: invalid WGSL reaches the WGPU diagnostic sink with module and
        entry-point identity and is not submitted. The alpha fixture's validly
        clipped work correctly produces no equivalent backend error.
- [x] Verify recovery: a diagnosed failed frame must not prevent a later valid
      frame from presenting.
  - [x] Not applicable to the retained defect: valid clipping created no
        failed renderer state and therefore no recovery transition. The fixed
        provider mapping is deterministic on every camera-uniform preparation.
- [x] Identify at least one independent corpus pressure before proposing any
      stable renderer diagnostic/capture contract.
  - [x] No diagnostic/capture contract is proposed. The existing evidence
        instead rejects that expansion for valid clipping.
- [x] Audit the cross-cutting consequences of changing the camera convention.
  - [x] Both WGPU camera upload paths forward `projection * view` unchanged.
  - [x] Built-in camera constructors are used by native and browser corpus
        members, including 2D, 3D, stereo, textured, CAD, and Doom consumers.
  - [x] `hello-cad` treats NDC `-1` as the near point during CPU unprojection;
        the Doom TypeScript workbench constructs `perspective_rh_gl`
        explicitly; and AR-0019 math evidence retains GL projection parity.
- [x] Apply ADR-0008 and ADR-0009 before admitting a renderer crossing.
  - [x] The change consumes, rather than redefines, Tokimu camera meaning; adds
        no public API, dependency, allocation, I/O, lock, unsafe code, fallback,
        or new recovery claim; and centralizes both WGPU upload paths on one
        private helper.
  - [x] The added cost is one fixed-size `Mat4` multiplication per prepared
        camera uniform. It is bounded, deterministic, allocation-free, and too
        small for a benchmark to add decision value absent measured camera-
        preparation pressure.
  - [x] Formatting, focused renderer/corpus tests, no-dependency Clippy with
        warnings denied, and browser/WASM compilation pass. Native visual
        confirmation remains a manual hardware observation under AR-0023.

## Reopening Triggers

- another corpus observes accepted/presented-but-visually-wrong work without
  actionable diagnostics;
- a caller requires generic readback, automated visual comparison, or
  GPU/CPU surface handoff;
- a backend diagnostic cannot be translated into bounded Tokimu evidence; or
- fixing the issue requires changing renderer ownership, public validation, or
  error/recovery guarantees.

## Review History

### Cycle 1 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: the AR-0023 blend fixture's empty native frame despite eleven
  accepted draws, eleven material resolutions, six pipeline switches, and a
  successful presentation return.
- Findings: error absence is not a visible-frame guarantee. The next work is
  narrow failure localization, not a new alpha contract or a readback feature.
- Disposition: investigate with corpus-local instrumentation; do not change
  `tokimu-render` public vocabulary yet.
- Resulting ADR or documentation change: opened this record.

### Cycle 2 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: all eleven commands reached material and pipeline resolution,
  including a known-good opaque control, while the frame remained empty. Code
  inspection established that `Camera::orthographic_2d_with_height` uses
  `Mat4::orthographic_rh_gl(-1, 1)` and the WGPU camera upload forwards that
  matrix unchanged. A focused projection test proves world `z = +0.5` maps to
  negative clip depth, while the provisional fixture's negative world depths
  map into WebGPU's visible `[0, 1]` interval.
- Findings: there was no backend error for the diagnostic sink to catch. GPU
  clipping is valid execution, and `present()` correctly reported accepted
  work rather than visible pixels. The deeper issue is a cross-API camera
  depth-convention mismatch or an undocumented restricted half-space, not an
  alpha/blend failure.
- Disposition: retain the corpus-local negative-Z repair to confirm visible
  recovery. Do not change `Camera` or WGPU upload semantics without maintainer
  review because either correction can alter every native and browser scene,
  depth ordering, and retained corpus image.
- Resulting ADR or documentation change: AR-0024 now owns the camera/adapter
  depth-convention question as well as the diagnostic interpretation.

### Cycle 3 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: repository-wide impact audit found two unchanged WGPU camera
  uploads, built-in camera use across native and browser corpus members,
  GL-specific CPU unprojection in `hello-cad`, an explicit GL projection in the
  Doom TypeScript workbench, and retained AR-0019 conformance evidence for GL
  matrices.
- Findings: the existing error system did not miss an error. The renderer and
  GPU executed a valid, fully clipped frame, while first-presentation counters
  correctly described submitted work rather than pixel visibility. Adding
  readback, validation errors, or a generic diagnostic contract would not fix
  the camera mismatch. Changing the camera convention or adapting it at the
  WGPU boundary is cross-cutting and cannot be treated as a fixture-local fix.
- Disposition: retain the tested negative-Z fixture workaround only long enough
  to recover AR-0023 visual evidence. Pause an engine-wide change for maintainer
  selection between WebGPU-native public camera meaning and explicit provider
  conversion. Do not select a public runtime convention without another
  provider.
- Resulting ADR or documentation change: added the camera-depth alternatives
  and impact inventory to this record; no stable renderer contract changed.

### Cycle 4 -- 2026-08-09

- Status entering review: Proposed.
- New evidence: maintainer review selected Alternative F. The WGPU adapter now
  constructs its private camera uniform through one named conversion used by
  both surface presentation and renderer-owned target passes. Its regression
  maps Tokimu clip depths `-1`, `0`, and `1` to WGPU depths `0`, `0.5`, and `1`
  while proving the caller's `Camera` remains unchanged. The alpha fixture has
  removed its provisional negative-Z workaround and again uses positive GL
  depths to pressure the adapter seam.
- Findings: explicit provider conversion preserves CAD unprojection, Doom's
  corpus-local projections, stereo callers, browser/native parity, and AR-0019
  matrix evidence. It adds one bounded matrix multiplication per prepared
  camera, with no allocation or new lifecycle/failure state. The broader test
  run also found and repaired a test-only missing qualification for
  `ShaderVertexSemantic`; production behavior was unaffected.
- Disposition: accept Alternative F. Tokimu cameras retain GL-style `[-1, 1]`
  clip-depth meaning; WGPU converts to `[0, 1]` only when building its private
  GPU uniform. The diagnostic finding is closed: no backend error was missed,
  and successful submission remains distinct from visible-pixel evidence.
- Validation: `cargo fmt --all`; `cargo test -p tokimu-render`; `cargo test -p
  hello-alpha-policy --features native-visual`; `cargo clippy -p tokimu-render
  -p hello-alpha-policy --features hello-alpha-policy/native-visual --no-deps
  -- -D warnings`; and `cargo check -p hello-alpha-policy-web --target
  wasm32-unknown-unknown` all pass. The pinned Ring 0 glam source emits its
  previously retained compiler-warning flood during dependency compilation.
- Resulting ADR or documentation change: updated the SDD with the stable camera
  convention and provider adaptation rule; retained a compact lessons-learned
  reference; no new public renderer vocabulary was admitted.

### Cycle 5 -- 2026-08-09

- Status entering review: Accepted.
- New evidence: the restored positive-depth native alpha fixture visibly
  presented its opaque control and all four blend panels on AMD Radeon RX 7900
  XTX/Vulkan. Its first frame reported eleven draws, eleven material
  resolutions, six pipeline switches, and `diagnostic=none`.
- Findings: the provider conversion recovers visible geometry without changing
  caller-owned camera matrices. The absence of a diagnostic is the correct
  observation for the now-valid frame and remains consistent with the original
  conclusion that valid clipping was never a backend error.
- Disposition: retain Accepted and close the visual recovery checkpoint. Future
  changes to Tokimu camera convention or WGPU conversion reopen this review;
  alpha-policy work continues independently in AR-0023.
- Resulting ADR or documentation change: linked the retained native blend
  observation through the AR-0023 study record.

### Cycle 6 -- 2026-08-11

- Status entering review: Accepted.
- New evidence: E1M1's dynamic-door corpus returned an explicit source geometry
  refresh error through `PlatformEventHandler::on_frame`. `tokimu-platform`
  structurally caught it, recorded it, and exited the native event loop, but
  did not present the error in the window. Without an attached terminal this
  appeared indistinguishable from a crash.
- Findings: the trigger was a missing source texture extent during door-wall
  re-lowering, not a WGPU diagnostic. The corpus now contains that recoverable
  error locally and retains a bounded message in its debug console and stderr.
- Disposition: retain the existing renderer/provider boundary. This is
  corpus-local failure-presentation evidence, not admission of a renderer-wide
  overlay, generic recovery policy, or standard error texture.

### Cycle 7 -- 2026-08-11

- Status entering review: Accepted.
- New evidence: after the dynamic-door corpus correctly created previously
  zero-area `DOORTRAK` spans, those new meshes initially reused numeric handles
  allocated to static cutout meshes. The cutout command path also derived its
  handle base from the mutable opaque-draw count. A normal door activation
  therefore invalidated live presentation-resource identity and closed the
  native observer without an in-window explanation.
- Findings: this is neither a shader error nor a WGPU-visible rendering
  failure. It is an application-side resource-identity/lifetime error whose
  effect becomes visible only at presentation. Fixed disjoint static-opaque,
  static-cutout, and dynamic-door ranges repair the corpus case, but are not a
  general resource-lifetime model.
- Disposition: retain the corpus-local repair and add an open research question:
  whether Tokimu should expose a bounded, explicit resource-identity/allocation
  discipline that keeps live handles stable and makes collision/unresolved
  references observable before presentation. Any such capability must preserve
  application ownership of draw lifetime and recovery policy; it must not
  auto-substitute missing resources or become a general GPU debugger.
- Resulting plan: execute the comparative
  [Renderer Resource Identity And Failure Presentation Test Plan](../Plans/Renderer-Reliability/renderer-resource-identity-and-failure-presentation.md)
  before admitting allocation, lifecycle, containment, or terminal-presentation
  vocabulary.

### Cycle 8 -- 2026-08-11

- Status entering review: Accepted, with the comparative terminal-owner study
  still open.
- New evidence: E1M1's opt-in Doom-sky startup rejected a composed `SKY1`
  raster with 2,048 uncovered source pixels. The native adapter retained that
  startup error and requested exit, but dispatched `on_frame` before shutdown.
  The frame then reported the secondary `Doom sky pipeline missing` error and
  replaced the useful root cause.
- Findings: terminal delivery alone is insufficient if a composition can
  overwrite its first failure during shutdown. The smallest demonstrated
  invariant is composition-local and causal: retain the first terminal
  callback error and stop dispatching frame work once terminal failure is
  pending.
- Disposition: enforce first-failure preservation privately in the native
  adapter and return the original error to the invoking caller. Do not add a
  global mailbox, renderer error overlay, automatic diagnostic material, or
  shared cross-target terminal-record owner. Those ownership questions remain
  open under the comparative test plan.
- Validation: a focused native-platform regression proves a secondary failure
  cannot replace the root error; the exact E1M1 command now reports the
  unresolved `SKY1` coverage instead of the missing-pipeline consequence.

### Cycle 9 -- 2026-08-13

- Status entering review: Accepted, with resource-identity and cross-target
  terminal-owner comparisons still open.
- New evidence: the independent `hello-render-resource-identity-web` fixture
  executed the same B/D/E lifecycle alternatives in browser/WASM, retained
  `ResourceUnresolved` for `MeshHandle(44)`, and presented a real WGPU
  same-handle replacement. The provider reported two uploads, one replacement,
  and one draw; the DOM retained the record after provider return.
- Findings: semantic category and resource identity can agree across native and
  browser targets without assigning their final presentation or lifetime to a
  shared owner. Same-handle replacement is demonstrated provider behavior on
  both paths and therefore cannot be prohibited as an aliasing repair.
- Disposition: retain application-owned lifecycle experiments and
  renderer-owned replacement/validation mechanics as corpus evidence. Do not
  admit B, D, or E, a global diagnostic store, or a cross-target terminal-record
  owner. Browser retention after page disposal remains deliberately unclaimed.

### Cycle 10 -- 2026-08-13

- Status entering review: Accepted; final comparative disposition requested.
- New evidence: maintainer accepted the test plan's recommendation after native
  and browser fixtures agreed on bounded resource identity/failure facts and
  intentional same-handle replacement.
- Findings: application allocation plus current renderer mechanics is the
  smallest demonstrated arrangement. Explicit lifecycle validation and
  generational identity remain useful experiments, not shared meaning. Native
  terminal return and live DOM retention do not require one lifetime owner.
- Disposition: retain Accepted. Preserve application-owned identity allocation,
  recovery and presentation policy; preserve renderer same-handle replacement;
  admit no renderer allocator, kernel resource identity, global diagnostic
  store, or shared terminal-record owner. Reopen only for independent lifecycle
  pressure or a supervisor/page-disposal case that caller ownership cannot
  satisfy. No ADR or SDD change results.

### Cycle 11 -- 2026-08-19

- Status entering review: Accepted; the dormant stronger cross-lifetime
  reopening trigger is now active under the renderer scene-resource lifetime
  plan.
- New evidence: repeated Doom browser map replacement closed Edge only after
  several fresh backend/device/surface lifetimes. The new deterministic
  E1M1-through-E1M9 three-round harness reports logical current/retired
  resource counts, creation counts, replacement timing, and narrowly scoped
  submitted-byte estimates. An independent non-Doom browser fixture now
  performs 27 equivalent whole-backend replacements with 64 meshes, textures,
  and materials per scene.
- Findings: the new pressure concerns physical renderer/provider lifetime, not
  application allocation of logical handles. The Cycle 10 no-allocation
  disposition therefore remains intact. Logical retirement remains distinct
  from WGPU/driver reclamation, which neither harness can observe directly.
- Disposition: reopen only the resource-lifetime question. Complete live
  Alternative-A browser observations and the retained ownership inventory
  before prototyping an adapter-private reset. Admit no reset, arena,
  generation, release, or renderer allocator contract from instrumentation.
- Resulting plan/evidence:
  `docs/Plans/Renderer-Reliability/renderer-scene-resource-lifetime-and-replacement.md`
  and its baseline/inventory evidence record.

### Cycle 12 -- 2026-08-19

- Status entering review: resource-lifetime question reopened; Alternative-A
  browser execution pending.
- New evidence: the automated Doom control completed all 27 replacements in
  19,657.4 ms. The final record retained 27 fresh backend/device/surface
  creations and 26 logically retired sets. The page and Edge window survived;
  the log contains no new GPU-process start during the run and no device-loss,
  OOM, WGPU validation, fatal, or Crashpad record.
- Findings: whole-backend replacement can survive this bounded automated
  workload, so the prior E1M3/E1M5/E1M6 closure is not a deterministic map-
  replacement failure. This does not prove synchronous reclamation or acquit
  movement/timing pressure. The lifetime counters correctly remain logical
  evidence rather than a physical-memory claim.
- Disposition: retain Alternative A as the successful Doom baseline. Run the
  independent non-Doom control and adversarial manual walkabout before the
  B-first gate. No stable renderer lifetime contract is admitted.

### Cycle 13 -- 2026-08-19

- Status entering review: Doom Alternative-A automation passed; independent
  browser control pending.
- New evidence: the non-Doom resource-rich control completed all 27 fresh
  backend/device/surface replacements in 1,644.4 ms, with a 46.92 ms mean and
  no returned diagnostic. The page/window survived.
- Findings: both automated callers tolerate bounded repeated provider-session
  replacement. The earlier movement/map-switch closure is therefore retained
  as timing or interaction-conditioned pressure rather than a deterministic
  E1M3/E1M5/E1M6 or Doom-only failure. OOM and physical reclamation remain
  unproven.
- Disposition: accept Slice 1 and authorize only the feature-gated,
  corpus-private Alternative B prototype. Preserve Cycle 10's rejection of a
  stable renderer allocator/lifetime contract until B passes its sufficiency
  gate.
