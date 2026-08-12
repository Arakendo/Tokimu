# Renderer Resource Identity And Failure Presentation Test Plan

| Field | Value |
| --- | --- |
| Status | Active — Slices 1–2 and 4 complete; Slice 3 terminal-owner question left open; Slice 5 in progress |
| Opened | 2026-08-11 |
| Related reviews | AR-0024, AR-0027 |
| Related ADRs | ADR-0007, ADR-0008, ADR-0009, ADR-0010, ADR-0011, ADR-0013 |
| First pressure source | E1M1 dynamic `DOORTRAK` wall spans |
| Scope | Test stable renderer-resource identity, explicit lifecycle intent, bounded failure observation, containment, recovery, and opt-in diagnostic presentation without transferring application semantics into the renderer or kernel |

## Purpose

The E1M1 manual-door corpus exposed two failures which appeared to be native
crashes but originated above WGPU:

1. dynamic wall re-lowering returned a source-geometry preparation error;
   `tokimu-platform` retained the returned error and exited its event loop, but
   a detached native window presented no explanation; and
2. newly materialized `DOORTRAK` meshes reused numeric handles already assigned
   to cutout meshes because the cutout base was derived from a mutable draw
   count. The renderer's intentional replace-on-upload behavior made this
   accidental alias look like a valid replacement until presentation failed.

The corpus repair uses disjoint local handle ranges and a local console error
path. Those repairs are evidence, not a reusable Tokimu contract. This plan
tests whether any smaller kernel, renderer, platform, or application contract
is earned.

The study must keep these concerns separate:

```text
resource identity and lifecycle intent
    create | replace | retire | reference

failure observation
    declaration | resource resolution | provider | presentation | platform

failure containment and recovery
    continue | reject operation | end composition | restart provider

diagnostic presentation
    text/record | explicit stand-in | fatal surface
```

A single `ErrorSystem`, automatic fallback texture, or general exception layer
is not an acceptable starting abstraction.

## Questions Under Test

1. Which layer should allocate or validate stable live renderer-resource
   identity?
2. How does a caller distinguish deliberate replacement from accidental handle
   aliasing without making every ordinary frame allocation-heavy?
3. Which failures are recoverable operations, which end only the active
   composition, and which are provider/platform-fatal?
4. What bounded record survives when a native window cannot continue?
5. Can the same observation shape work on native and browser/WASM without
   leaking WGPU, windowing, JavaScript, or Doom vocabulary?
6. When is an opt-in error texture useful, and when would it misrepresent a
   geometry, lifecycle, shader, or provider failure?
7. Is any finding Native Ring meaning, or is it renderer/platform mechanism
   consumed through an existing kernel diagnostic contract?

## Constraints

- Preserve application ownership of scene membership, draw lifetime, source
  identity, recovery policy, and whether diagnostic presentation is requested.
- Preserve renderer ownership of provider-neutral resource validation and
  prepared-command resolution.
- Preserve provider ownership of WGPU/device/surface failure details and map
  them into bounded provider-neutral observations where possible.
- Do not silently substitute a mesh, texture, material, shader, or pipeline.
- Do not treat successful `present()` as proof that expected pixels appeared.
- Do not catch an arbitrary panic and continue through potentially corrupted
  state. Panic/fatal containment is a separate claim from recoverable `Result`
  handling.
- Keep `tokimu-core` free of GPU, window, browser, asset-format, and source-game
  vocabulary.
- Any Native Ring candidate must pass ADR-0008, ADR-0009, and ADR-0011 evidence
  gates. Prefer an Outer Ring mechanism if kernel ownership is not proven.
- Maintain native/WASM feasibility. Browser presentation may use a different
  final reporting surface while retaining equivalent semantic records.
- Do not stabilize a public API while executing this plan. An admission choice
  requires a recorded maintainer decision and an ADR where applicable.

## Failure And Lifecycle Matrix

Every fixture must retain the operation, expected ownership, observed layer,
continuation result, and diagnostic identity. At minimum test:

| Case | Required distinction |
| --- | --- |
| Create a fresh mesh identity | successful creation is not replacement |
| Replace an existing mesh intentionally | stable identity and explicit replacement remain supported |
| Accidentally allocate an already-live mesh identity | must not masquerade as unrelated intentional replacement |
| Reference an unresolved mesh/material/pipeline | rejected command is distinct from provider failure |
| Retire then reference an identity | stale reference is distinct from never-created identity if the tested model claims that distinction |
| Dynamic draw addition | does not renumber or invalidate existing live resources |
| Source geometry preparation failure | remains distinct from renderer-resource failure |
| Renderer declaration rejection | remains distinct from WGPU/backend validation |
| Provider/surface failure | remains distinct from application command failure |
| Returned frame-handler error | retained before native/browser composition ends |
| Fatal panic or abort fixture | recorded only as fatal-process evidence; no unsafe continuation claim |
| Explicit diagnostic stand-in | original identity and reason survive; normal rendering never opts in implicitly |

## Alternatives To Compare

### A. Application-Owned Disjoint Handle Ranges

Retain the E1M1 repair as the baseline. It is cheap and explicit, but callers
must coordinate ranges and can reproduce the same bug independently.

### B. Application-Owned Allocator/Registry Helper

Test a non-public helper which issues stable typed handles and records
create/replace/retire intent. This may remain application/tooling vocabulary if
the renderer only needs the resulting handles and commands.

### C. Renderer-Owned Stable Identity Allocation

Test whether the renderer should allocate resource identities or return opaque
creation tokens. This can prevent collisions but may entangle semantic scene
preparation with a concrete renderer lifecycle and complicate replay/WASM.

### D. Generational Typed Handles

Compare an index-plus-generation model which distinguishes stale from current
references. Measure representation, lookup, churn, replacement semantics, and
native/WASM behavior before considering admission.

### E. Explicit Create/Replace/Retire Operations Over Existing Handles

Keep caller-selected typed handles, but make lifecycle intent explicit and
validate it. This may preserve deterministic caller identity while detecting
accidental aliasing; it also adds state and validation work.

### F. Validation And Observation Only

Retain current identity ownership but add bounded checks/records around
resource replacement and command resolution. This may solve the observed
diagnostic gap without admitting allocation policy.

The final disposition may compose alternatives—for example an application
allocator with renderer-side validation—but each ownership claim must be
evidenced independently.

## Slice 1: Reproduce And Retain The Baseline

### Deliverables

- [x] Add a deterministic corpus fixture which reproduces the original mutable
      offset/handle-alias failure without depending on Doom input.
- [x] Retain the E1M1 `DOORTRAK` sequence as the real-caller case:
  - [x] closed zero-area source spans omitted;
  - [x] opening creates the four ordinary textured `DOORTRAK` triangles
        attributable to linedefs 155 and 156;
  - [x] existing cutout resources retain their original identities;
  - [x] closing suppresses the dynamic-only spans;
  - [x] reopening reuses the intended dynamic identities without collision.
- [x] Record current renderer behavior for intentional same-handle mesh upload
      and accidental unrelated same-handle upload.
- [x] Retain native target, adapter, build, command counts, resource identities,
      and termination/continuation outcome.

### Validation

- [x] The synthetic fixture fails for the same identity/lifetime reason as the
      historical E1M1 bug, not because of missing geometry or texture input.
- [x] The repaired E1M1 path remains visually correct and produces no silent
      event-loop exit.

### Acceptance Criteria

- [x] The original defect is reproducible independently of Doom semantics.
- [x] Deliberate replacement remains visibly distinct from accidental aliasing.

### Retained Evidence

The initial fixture, current renderer behavior, deterministic output, and
native E1M1 observation are retained in
[`renderer-resource-identity-baseline-evidence.md`](renderer-resource-identity-baseline-evidence.md).
The deterministic E1M1 close/suppress/reopen identity replay is retained below;
Slice 1 is complete.

## Slice 2: Resource Identity Alternatives

### Deliverables

- [x] Implement corpus-local prototypes for Alternatives B, D, E, and F; retain
      A as the baseline. Prototype C only if the preceding results leave a real
      renderer-owned allocation question.
- [x] Exercise every operation claimed by each prototype. The comparison
      retains that F is observation-only and does not claim retire or reference
      validation semantics.
- [x] Preserve deterministic caller-selected identity where each alternative
      claims to support replay or offline preparation.
- [x] Retain a second, non-Doom dynamic replacement caller (`hello-glb`) whose
      two stable mesh identities create on its first native frame and replace
      intentionally on its second.
- [ ] Measure:
  - [x] per-resource representation size;
  - [x] steady-state lookup/validation work as a preliminary native timing
        observation, not a performance gate;
  - [x] allocation/lifecycle and replacement counts;
  - [x] churn under repeated create/retire cycles;
  - [x] failure-record bounding;
  - [x] native and WASM compile feasibility; browser execution remains open.

### Validation

- [x] No alternative renumbers a live resource when an unrelated resource is
      added.
- [x] Intentional replacement updates the intended resource only.
- [x] Collision and stale/unresolved cases produce stable bounded identities.
- [x] An independent native WGPU caller confirms that repeated same-handle
      upload remains a useful intentional replacement operation.
- [x] No Native Ring candidate is proposed from this slice, so ADR-0008's full
      admission gate is not yet applicable. Target/profile/workload-labelled
      corpus measurements are retained rather than treating fixture timing as a
      shared performance guarantee.

### Acceptance Criteria

- [x] At least two viable alternatives survive the bounded correctness and
      target/profile-labelled performance pressure: B, D, and E remain viable
      corpus candidates; F remains observation-only.
- [x] No prototype crosses into a public/stable contract.

### Retained Evidence

The implementation matrix, native churn observation, bounded diagnostic-ring
test, and WASM compile feasibility are retained in
[`renderer-resource-identity-alternatives-evidence.md`](renderer-resource-identity-alternatives-evidence.md).
The comparisons include a second native renderer caller and a release-profile
bounded churn measurement. No identity alternative is admitted from fixture
timing alone; Slice 3 must determine whether any validation/observation shape
is shared across failure layers.

## Slice 3: Failure Observation Boundary

### Deliverables

- [ ] Inject and retain one failure at each applicable layer:
  - [x] source preparation;
  - [x] renderer declaration/resource resolution;
  - [x] backend/provider validation;
  - [x] surface acquisition/presentation;
  - [x] application frame-handler return;
  - [x] platform event-loop termination.
- [x] Define a provisional bounded observation envelope containing only facts
      demonstrated useful by the fixtures, such as phase, operation, typed
      resource identity, caller correlation, provider-neutral category, and
      continuation status.
- [x] Prove that WGPU-native details can remain provider diagnostics without
      leaking their types into the provisional envelope.
- [ ] Retain exactly where each observation remains accessible after the active
      composition ends on native and browser/WASM.
  - [ ] Maintainer decision: retain the current caller-owned native terminal
        result plus fixture-owned browser status surfaces, or authorize a
        separate investigation into a shared cross-target terminal-record
        owner. **Disposition (2026-08-11): leave the shared owner open.**
        Retain caller/fixture-owned terminal delivery and continue comparing
        only bounded semantic failure facts until a lifetime or independent
        caller case proves caller ownership insufficient.

### Validation

- [x] A reviewer can determine whether failure occurred before command
      preparation, during resource resolution, in the provider, or during
      platform shutdown from the retained cross-layer matrix.
- [x] Normal successful frames do not allocate unbounded diagnostic history.
- [x] Returned recoverable errors and fatal panics are not reported as the same
      recovery class.

### Acceptance Criteria

- [x] No tested recoverable failure appears only as a disappearing window or
      stalled browser surface: native delivery reaches the terminal caller;
      browser fixtures retain their DOM-owned status surfaces; E1M1 retains its
      corpus-local console record. This is not yet a shared cross-target
      terminal-record guarantee.
- [ ] Diagnostic records are bounded, source-correlatable, and honest about
      what the provider could not observe.

### Retained Evidence

The provisional envelope, six-layer evidence matrix, live native provider
fixture, and remaining terminal/cross-target gaps are retained in
[`renderer-failure-observation-boundary-evidence.md`](renderer-failure-observation-boundary-evidence.md).

## Slice 4: Containment And Recovery Policies

### Deliverables

- [ ] Classify each injected failure as one of:
  - [x] reject operation and continue the composition;
  - [x] retain the last known-good resource/frame and continue;
  - [x] end only the active composition with a retained terminal record;
  - [x] provider/platform fatal with no continuation claim.
- [x] Test that an application may choose among authorized recoveries without
      mutating renderer-owned state behind the renderer's contract.
- [x] Exercise a last-known-good intentional mesh replacement failure: the old
      mesh remains identifiable and no partial replacement is claimed.
- [x] Test bounded repeat failures in the retained record; console and browser
      bridge rate limiting remain unclaimed until separately exercised.
- [x] Record whether any recovery requires a new capability or can compose
      existing diagnostics, resource upload, and application lifecycle seams.

### Validation

- [x] Recovery is explicit and deterministic for the same command sequence.
- [x] No recovery silently binds a fallback asset or converts failure into
      success.
- [x] Fatal-state tests stop safely and retain only the terminal/process
      evidence the platform can honestly guarantee; no continuation is claimed.

### Acceptance Criteria

- [x] Application policy remains separate from failure detection.
- [x] The study identifies the smallest demonstrated containment arrangement:
      caller validation plus existing renderer upload and platform-result
      seams; no new shared mechanism is admitted.

### Retained Evidence

The caller-staged last-known-good comparison, bounded-repeat result, failure
classification, and remaining provider/fatal gap are retained in
[`renderer-containment-and-recovery-evidence.md`](renderer-containment-and-recovery-evidence.md).

## Slice 5: Diagnostic Presentation Comparison

### Deliverables

- [ ] Compare at least these presentation choices using the same retained
      failure records:
  - [ ] structured record only;
  - [ ] text/console overlay;
  - [ ] application-supplied conspicuous diagnostic texture/material:
    - [x] native E1M1 opt-in Purple PNG stand-in for retained sky omissions;
    - [x] native visual observation and bounded record transcript;
    - [ ] browser/WASM equivalent observation;
  - [ ] terminal composition error surface where continued scene rendering is
        unsafe.
- [ ] Exercise at least four distinct meanings: intentional source omission,
      source geometry failure, missing renderer resource, and provider failure.
- [ ] Prove that the application-supplied stand-in remains opt-in and retains
      original identity plus reason beside the draw.
- [ ] Exercise native and browser/WASM presentation without claiming pixel
      identity.
- [ ] Use a second independent non-Doom caller before proposing shared
      diagnostic-presentation vocabulary.

### Validation

- [ ] A reviewer cannot mistake diagnostic presentation for successful source
      or renderer resource resolution.
- [ ] Cases for which a texture would lie use text/terminal presentation or no
      replacement instead.
- [ ] Bundled or generated diagnostic assets retain provenance, color-space,
      sampler, packaging, and offline evidence required by AR-0027.

### Slice 5 Evidence

The first Alternative A implementation and its intentionally narrow meaning
matrix are retained in
[`diagnostic-presentation-comparison-evidence.md`](diagnostic-presentation-comparison-evidence.md).

### Acceptance Criteria

- [ ] AR-0027 has enough comparative evidence to retain corpus-local behavior,
      admit a narrow intent, or reject a standard error texture.
- [ ] No automatic renderer fallback is introduced.

## Slice 6: Independent Caller And Cross-Target Pressure

### Deliverables

- [ ] Re-run the selected resource-identity and observation candidates through:
  - [ ] E1M1 dynamic doors;
  - [ ] one non-Doom dynamic mesh/resource caller;
  - [ ] native WGPU;
  - [ ] browser/WebGPU WASM.
- [ ] Include one resource-rich scene and one small synthetic fixture so scale
      and clarity do not depend on the same caller.
- [ ] Retain unsupported GPU/vendor coverage explicitly; do not infer NVIDIA
      behavior from AMD/Vulkan or Apple/Metal observations.
- [ ] Test shutdown/terminal-record delivery under both an attached diagnostic
      console and a detached presentation surface.

### Validation

- [ ] Equivalent semantic failures retain equivalent categories and resource
      identities across targets.
- [ ] Target-specific provider details remain available without becoming the
      provider-neutral contract.

### Acceptance Criteria

- [ ] At least two independent callers and both execution targets support any
      proposed shared contract.
- [ ] Remaining adapter/vendor gaps are explicit and do not block an honest
      scoped disposition.

## Slice 7: Ownership And Admission Decision

### Deliverables

- [ ] Compare final evidence against these possible dispositions:
  - [ ] application/tooling helper only;
  - [ ] renderer-owned validation with application-owned allocation;
  - [ ] renderer-owned stable allocation/lifetime capability;
  - [ ] kernel-native resource identity or failure-observation meaning with
        Outer Ring realization;
  - [ ] no shared admission; retain corpus-local patterns.
- [ ] Identify separately which findings update AR-0024 and AR-0027.
- [ ] Run ADR-0008 performance/code-quality, ADR-0009 verification/recovery,
      ADR-0010 provenance where applicable, and ADR-0011 security gates for any
      Native Ring candidate.
- [ ] Update or create an ADR only after the ownership choice has maintainer
      authorization.
- [ ] Update SDD, lessons, examples, and validation guidance if a stable
      contract is admitted.

### Validation

- [ ] The proposed owner is the smallest layer capable of maintaining the
      invariant across independent callers.
- [ ] The decision does not make `tokimu-core` own renderer resources, WGPU
      mechanisms, windows, browser surfaces, or application recovery policy.
- [ ] Rejected and deferred alternatives retain the evidence that rejected
      them.

### Acceptance Criteria

- [ ] AR-0024 and AR-0027 receive explicit evidence-backed dispositions.
- [ ] Any accepted capability has tests, retained failure evidence, bounded
      diagnostics, performance evidence, and native/WASM coverage proportional
      to its ring.

## Overall Completion Gate

This plan is complete only when:

- stable live resource identity survives unrelated dynamic additions;
- deliberate replacement remains supported and explicit;
- accidental aliasing, stale references, and unresolved resources are either
  structurally prevented or produce bounded actionable observations;
- tested recoverable failures do not present only as a disappearing native
  window or stalled browser surface;
- recovery and diagnostic-presentation policy remain application-owned unless
  independent evidence earns a narrower shared semantic;
- no automatic fallback hides a failed source, resource, material, shader, or
  provider operation;
- AR-0024 and AR-0027 can make separate decisions from the retained evidence;
  and
- any kernel/Native Ring proposal passes the accepted performance,
  verification, provenance, and security gates before admission.

## Initial Disposition

Begin with Slices 1 through 3 using corpus-local prototypes. The strongest
initial hypothesis is application-owned allocation plus renderer-owned
validation and bounded observation, but this is not a decision. Test
generational identity and explicit lifecycle operations before deciding
whether kernel-native meaning exists. Do not add public renderer or kernel
vocabulary merely to replace E1M1's local numeric ranges.
