# Renderer Scene-Resource Lifetime And Replacement Plan

| Field | Value |
| --- | --- |
| Status | In progress -- Slices 1 and 2 accepted; Alternative B rejected by retained falsifiers; corpus-private Alternative C semantic prototype implemented |
| Opened | 2026-08-19 |
| Related reviews | AR-0024 and AR-0030 |
| Related ADRs | ADR-0001, ADR-0003, ADR-0007, ADR-0009, ADR-0011, ADR-0017 |
| First pressure source | Repeated browser/WASM Doom working-map replacement |
| Scope | Determine the smallest honest lifetime model for replacing a bounded set of renderer resources while retaining a provider session, device, and presentation surface |

## Purpose

The browser Doom walkabout can replace E1M1 through E1M9 from one page. An
initial defect allowed the old and replacement WGPU surfaces to overlap on the
same canvas. Dropping the old backend before creating its replacement allowed
E1M3 to survive, but a later walkabout still closed Edge around E1M5 or E1M6.
The retained browser log contains no explicit WGPU validation error, device
loss, or out-of-memory report, so memory exhaustion is plausible rather than
proven.

One concrete source of cumulative pressure remains:

```text
map replacement
    -> create a new WGPU backend
    -> create a new surface and device
    -> upload a complete replacement resource set
    -> drop the preceding backend
```

Rust ownership ends the preceding backend's observable lifetime, but does not
prove when the browser, WGPU implementation, driver, or GPU reclaims retired
work. Reusing the existing backend is not currently a local consumer change:
sampleable texture creation rejects an already-live handle, materials retain
concrete texture views, and the renderer exposes no individual release or
bounded scene-resource reset operation.

This plan studies that missing lifetime seam. It does **not** assume that a
scene arena, explicit release API, renderer allocator, or device reuse policy
is already the right contract.

This pressure is adjacent to, but distinct from, AR-0030's comparison between
persistent renderer resources and submission-local/view-local preparation:

```text
AR-0030 axis
    persistent renderer resource
        vs submission-local/view-local work

this plan's axis
    persistent provider session
        vs replaceable composition-resource lifetime
```

The study must not merge those axes into one owner merely because both are
observed near the renderer.

## Architectural Question

What is the smallest Tokimu-owned or adapter-private mechanism that can replace
one complete presentation-resource set with another while:

- preserving application ownership of scene membership and replacement policy;
- preserving deterministic application-owned logical resource identity;
- keeping provider device, queue, surface, and synchronization details behind
  the renderer adapter;
- preventing stale commands or materials from observing retired resources;
- retaining the last known-good composition when replacement preparation
  fails; and
- exposing bounded evidence about logical retirement without falsely claiming
  immediate physical GPU reclamation?

## Required Distinctions

The study must keep these concepts separate:

```text
logical resource identity
    != physical GPU allocation

scene membership
    != backend/session lifetime

same-handle replacement
    != retirement

retirement requested
    != provider work completed
    != physical memory reclaimed

composition swap completed
    != old resources synchronously destroyed

ordinary resource reset
    != device-loss recovery
    != surface recreation
```

`scene-resource set`, `arena`, and `generation` are provisional study terms.
They do not become stable API vocabulary merely by appearing in this plan.

## Ownership Constraints

- The application/composition owns which scene is current, when replacement is
  requested, source/runtime snapshots, logical handles, and whether failure
  retains the previous composition or ends it.
- `tokimu-render` may own provider-neutral validation of resource references,
  replacement boundaries, and bounded lifecycle observations if independent
  evidence earns those meanings.
- The WGPU backend owns concrete buffers, textures, views, bind groups,
  pipelines, device/queue work, surface state, and backend-specific retirement
  mechanics.
- `tokimu-platform` owns the native/browser host lifetime and surface creation
  mechanics; it must not become the owner of scene contents.
- `tokimu-core` and `tokimu-runtime` must not acquire GPU-resource or renderer
  session vocabulary.
- Doom preparation, grouped-sky parity, sector-boundary trimming, map movement,
  and WAD semantics remain outside this plan.
- No public or stable API is authorized during comparative implementation.
  Admission requires an explicit disposition and an ADR/SDD update when the
  accepted boundary changes.

## Alternatives To Compare

### A. Whole-Backend Replacement

Retain the current implementation as the baseline: build a new backend for
every map and drop the old backend. It gives simple Rust ownership but creates
new devices and surfaces, depends on external reclamation timing, and cannot
distinguish scene replacement from provider-session replacement.

### B. Adapter-Private Scene-Resource Reset

Keep one device, queue, and surface, then clear all scene-owned resource maps at
an explicit quiescent replacement point. This is the smallest likely repair,
but it must prove dependency safety, stale-command rejection, same-handle reuse,
and honest reclamation observations. It must not reset provider-global objects
that are intentionally reusable.

### C. Replaceable Scene-Resource Arena

Build resources inside a provisional arena/generation, validate a complete
candidate, atomically make it current, and retire the previous arena. This
supports last-known-good replacement and deterministic stale-reference
detection, but may temporarily double resource residency and may introduce an
unearned renderer lifecycle abstraction.

### D. Explicit Dependency-Aware Resource Release

Expose release intent for individual mesh, texture, material, pipeline, and
camera handles. This may support incremental scenes, but callers or the
renderer must honor dependencies such as material-to-texture views and
command-to-resource references. It is broader and more error-prone than the
observed whole-composition replacement requirement.

### E. Backend-Internal Reclamation Policy

Keep current provider-neutral declarations and make the WGPU adapter infer
unreferenced resources or retire them after an internal frame boundary. This
minimizes shared vocabulary, but implicit liveness can hide application intent,
make deterministic replay harder, and retain resources indefinitely when
references are ambiguous.

### F. Whole-Session Or Page Replacement

Treat each selected map as a new browser session/page and rely on host teardown
to reclaim the previous provider. This is a useful containment control and may
remain an emergency fallback, but it sacrifices seamless map rotation and does
not answer resource-rich composition replacement in long-lived applications.

The final disposition may combine mechanisms, such as an adapter-private
resource arena plus provider-neutral bounded observations. Each ownership claim
must be justified separately.

## Slice 1: Reproduce And Bound The Lifetime Pressure

Initial implementation and the current ownership inventory are retained in
[Renderer Scene-Resource Lifetime Baseline And Inventory Evidence](Evidence/renderer-scene-resource-lifetime-baseline-and-inventory-evidence.md).

### Deliverables

- [x] Add a repeatable browser rotation harness that cycles E1M1 through E1M9
      for at least three rounds without requiring manual button timing.
- [x] Retain manual walkabout as a separate visual/interaction test; automated
      rotation must not claim to reproduce every movement-triggered failure.
- [ ] Record per replacement:
  - [x] map and replacement sequence number;
  - [x] backend, device, and surface creation counts;
  - [x] logical uploads and same-handle replacements by resource family;
  - [x] active and retired logical resource counts;
  - [x] estimated CPU-side and provider-submitted resource bytes where those
        estimates are honest;
  - [x] frame/replacement timings and bounded provider diagnostics;
  - [x] page, renderer-process, and GPU-process survival where observable for
        the first successful Doom Alternative-A run; retain the same evidence
        for the independent control.
- [x] Label browser process memory or GPU-memory observations by source and
      availability. Absence of a measurement must not be reported as zero.
- [x] Add a smaller non-Doom resource-rich replacement fixture so any proposed
      shared contract has an independent caller.
- [x] Preserve the current whole-backend replacement behavior as Alternative A.

### Validation

- [x] The harness distinguishes a returned Rust/WASM error, device loss,
      renderer-process termination, GPU-process restart, and whole-window exit
      when the host exposes those facts.
- [x] The earlier whole-window disappearance is classified as an unresolved
      terminal outcome and immediate conformance failure under ADR-0017; later
      successful runs narrow reproduction but do not erase it.
- [x] A successful cycle does not become proof of synchronous reclamation.
- [x] The E1M3 overlap repair remains in force: no replacement creates two live
      WGPU surfaces for the same canvas.

### Acceptance Criteria

- [x] The current lifecycle and its observable limits are reproducible without
      relying on the remembered E1M5/E1M6 failure location.
- [x] Evidence can distinguish cumulative replacement pressure from a
      deterministic map-specific preparation defect.

## Slice 2: Inventory Resource Ownership And Dependencies

### Deliverables

- [x] Inventory every persistent renderer/WGPU resource collection and identify
      its owner, creation operation, replacement behavior, reference edges, and
      current drop boundary.
- [x] Retain an explicit dependency graph covering at least:

  ```text
  command -> mesh / material / pipeline / camera
  material -> texture view / sampler
  pipeline -> layouts / shader modules
  surface -> device configuration
  ```

- [x] Separate scene-replaceable resources from provider/session resources and
      immutable/shared caches demonstrated by current callers.
- [x] Identify which handles may be reused after retirement, which references
      become stale, and how a stale command can be diagnosed deterministically.
- [x] Record what WGPU/Rust drop establishes and what it cannot establish about
      submitted work or physical reclamation.

### Validation

- [x] Every resource removed by a prototype is reachable from the inventory.
- [x] No material, bind group, command, or cached provider object retains an
      undocumented reference into a retired resource set.

### Acceptance Criteria

- [x] A reviewer can identify the exact safe reset boundary without treating
      `HashMap::clear`, Rust `Drop`, queue completion, and GPU-memory release as
      synonyms.

## Slice 3: Corpus-Private Lifetime Prototypes

### Deliverables

- [x] Prototype Alternative B first behind a private/experimental seam.
- [x] Apply the B-first sufficiency gate before implementing C. Alternative B
      is rejected, not promoted:
  - [x] Atomic last-known-good replacement was evaluated and falsified because
        reset retires the current scene before successor staging can succeed.
  - [x] Deterministic stale-handle rejection was evaluated and falsified
        because an old bare handle aliases a successor resource with the same
        numeric value.
  - [x] Logical-handle reuse without retired-set aliasing was evaluated and
        falsified by that same live browser probe.
  - [x] No new device or surface during ordinary replacement was demonstrated.
  - [x] Bounded logical-retirement evidence was demonstrated without claiming
        physical reclamation.
- [x] Prototype Alternative C only after the two retained B falsifiers earned
      it. The first implementation is a pure corpus Rust model: it stages an
      E1M2 candidate beside committed E1M1, injects a material-stage failure,
      proves E1M1 remains current, commits a complete E1M2 candidate, and then
      rejects the retained E1M1 handle as stale while the same local key
      resolves E1M2. Native and WASM execute the same model. It does not yet
      stage renderer/provider resources; see
      [Alternative C Semantic Generation Prototype Evidence](Evidence/renderer-scene-resource-alternative-c-semantic-generation-evidence.md).
- [x] Correlate C with heterogeneous real-caller inventories before considering
      provider staging. The independent 64-mesh/texture/material caller now
      executes the same semantic shadow in generated WASM, and the Doom browser
      workbench projects each successfully presented map's actual prepared
      mesh/texture/material/pipeline/camera/command counts into that shadow.
      Native tests, both WASM builds, and a live E1M1-to-E1M2 browser transition
      pass. E1M1 contributed 1,236 logical resources and E1M2 contributed
      2,353; failed E1M2 staging retained E1M1, successful commit retired E1M1,
      the generation-0 handle rejected stale, and generation 1 resolved reused
      mesh key 1. The shadow is explicitly not evidence that the current
      provider replacement is atomic.
- [x] Execute the separately authorized corpus-private real-provider staging
      probe in a live browser. One browser WGPU backend/device/surface presented
      eight draws from A, staged 26 B resources before an explicit
      `MissingTexture(9)` failure, then presented the same eight A draws again.
      A complete second B retired and replaced eight meshes, textures,
      materials, commands, one pipeline, and one camera; B presented eight
      draws with zero provider diagnostics. Physical overlap bytes and
      reclamation remain unobserved. See
      [Alternative C Real-Provider Staging Evidence](Evidence/renderer-scene-resource-alternative-c-real-provider-staging-evidence.md).
- [ ] Execute the authorized repeated-provider pressure harness. The fixed
      staging mechanism now has a 27-cycle browser workload alternating two
      64-resource-family sets with a late complete-candidate failure every
      fifth cycle. Each cycle enforces one post-commit logical set, two sets
      only during staging, complete inventory symmetry, 64 presented draws,
      one provider session, and zero provider diagnostics. Live execution is
      pending; see
      [Alternative C Repeated Provider Pressure Evidence](Evidence/renderer-scene-resource-alternative-c-repeated-provider-pressure-evidence.md).
- [ ] Prototype D only if the inventory or independent caller demonstrates a
      real incremental-release requirement that whole-set replacement cannot
      satisfy.
- [ ] Retain E and F as comparison controls unless evidence justifies code.
- [ ] For each implemented alternative, exercise:
  - [x] complete resource-set creation in the semantic C model, including draw
        reference validation before candidate construction succeeds;
  - [ ] intentional same-handle mesh replacement within the current set;
  - [x] candidate construction followed by successful atomic semantic
        installation;
  - [x] candidate construction failure with previous composition retained and
        resolvable;
  - [x] stale resource reference after generation retirement;
  - [x] reuse of the same local resource key in a later generation without
        aliasing the retired handle;
  - [ ] bounded repeated replacement;
  - [ ] shutdown during or immediately after replacement.
- [ ] Keep Doom source preparation and render declarations identical across the
      A/B comparison; only resource/provider lifetime may differ.

### Validation

- [x] In the semantic C model, at most one committed scene-resource set is
      addressable; staged candidates are not resolved through current state.
- [x] In the semantic C model, temporary dual logical residency is explicit
      and bounded to current plus candidate. Failure drops the candidate;
      commit replaces current in one state mutation. Provider residency is not
      exercised or claimed.
- [x] In the real-provider C prototype, temporary A/B allocation overlap is
      explicit and candidate resources cannot enter live resolution before
      validation and commit. Physical byte overlap and reclamation remain
      unobserved.
- [ ] Commands cannot resolve a resource from the wrong set/generation.
- [ ] Intentional same-handle replacement remains supported and distinct from
      cross-set stale identity.
- [x] The real-provider staging probe rejects its missing texture explicitly,
      preserves A, and performs no missing-resource substitution.

### Acceptance Criteria

- [ ] At least one alternative reuses a single provider session/device/surface
      over repeated resource-set replacement while preserving correctness.
- [ ] If B survives its sufficiency gate, C remains unimplemented unless a
      retained requirement demonstrates additional semantic value.
- [x] C's temporary staging answers B's atomicity failure, and its generation
      distinction answers B's stale-aliasing failure. No other semantic value
      is claimed yet.
- [x] No prototype exposes WGPU objects or Doom vocabulary through a
      provider-neutral boundary.

## Slice 4: Atomicity, Failure, And Recovery

### Deliverables

- [x] Define and test the candidate lifecycle without making the vocabulary
      stable:

  ```text
  prepare CPU declarations
      -> stage candidate resources
      -> validate complete candidate
      -> install atomically
      -> retire preceding logical set
      -> observe provider retirement only as far as supported
  ```

- [ ] Inject failure during mesh, texture, material, pipeline, and camera
      staging where applicable.
- [ ] Inject surface acquisition/presentation failure independently of
      resource-set replacement.
- [x] Preserve the first causal failure and the last known-good composition
      whenever continued presentation is safe.
- [ ] Define the escalation boundary at which device loss ends the provider
      session rather than masquerading as an ordinary scene reset.
- [ ] Bound lifecycle observations; repeated replacement must not accumulate an
      unbounded retirement log.

### Validation

- [ ] A failed candidate is never partially current.
- [ ] Retiring the previous set occurs only after successful installation of
      the candidate, unless application policy explicitly ends the composition.
- [ ] Provider/session fatal states do not claim successful scene recovery.

### Acceptance Criteria

- [ ] Replacement atomicity and failure containment are deterministic for the
      same input sequence.
- [ ] Application recovery policy remains separate from renderer/provider
      detection and resource teardown mechanics.

## Slice 5: Native, Browser, And Independent-Caller Pressure

### Deliverables

- [x] Add a corpus-private external browser supervisor that owns an isolated
      browser process, correlates a unique run and page-subject identity, and
      classifies completion, structured rejection, owned-process exit, page
      heartbeat loss, and page replacement/reload without guessing a cause.
- [x] Instrument both the Doom workbench and independent resource-lifetime
      fixture with bounded operation, heartbeat, page-error, and terminal
      records. Keep the observer outside the page failure domain.
- [ ] Exercise the surviving alternative on native WGPU and browser WebGPU.
- [ ] Run the automated E1M1-through-E1M9 rotation for at least three complete
      rounds, followed by the adversarial manual walkabout/map-switch test.
- [ ] Observe browser/page/renderer/GPU-process liveness from outside the page
      failure domain; an in-page returned record alone does not satisfy
      ADR-0017.
  - [x] External browser-process and page-subject observation implemented and
        controlled subject termination classified end to end.
  - [ ] Live Edge/WGPU rotation and adversarial walkabout observed with the new
        harness.
    - [x] Retained-session E1M1-through-E1M9 rotation completed under external
          supervision (`run-id=46044-1787188803903131100`, 576,189.115 ms,
          130 acknowledged events).
    - [x] Manual adversarial E1M1-to-E1M3 switching and sustained movement,
          including `W+C` forward-plus-descend input, passed user visual
          inspection after descend moved away from Edge's `Ctrl+W` shortcut.
          This is interaction/presentation evidence, not external terminal
          closure evidence.
    - [ ] Adversarial manual walkabout/map switching completed under external
          supervision.
  - [ ] Renderer- and GPU-process identity remain unsupported by the current
        corpus-private harness and must not be claimed from browser-log text
        alone.
- [ ] Exercise the independent non-Doom fixture with equivalent resource-set
      replacement and stale-reference cases.
- [ ] Include dynamic replacement within one scene so arena reset does not
      accidentally prohibit the existing same-handle replacement behavior.
- [ ] Retain adapter, device, browser version, target/profile, corpus revision,
      resource counts, high-water estimates, and unsupported observability.
- [ ] Compare replacement latency and peak logical/provider-submitted residency
      against Alternative A; treat external GPU-memory readings as observations,
      not portable guarantees.

### Validation

- [ ] No test creates a new device or surface during ordinary scene-resource
      replacement under the retained-session alternative.
- [ ] Native and browser retain equivalent logical lifecycle outcomes even when
      provider details differ.
- [ ] Doom visual output, grouped-sky behavior, movement, and map selection are
      unchanged by the lifetime alternative.

### Acceptance Criteria

- [ ] Repeated browser rotation and manual movement complete without a closed
      window, lost interaction, unexplained blank frame, or unbounded logical
      resource growth on the tested target.
- [ ] Every started browser run closes as success, structured rejection/fatal,
      or independently observed external termination. Any unresolved
      disappearance fails acceptance immediately.
- [ ] At least one independent caller demonstrates that the surviving lifetime
      meaning is not merely a Doom workaround.

## Slice 6: Ownership And Admission Decision

### Deliverables

- [ ] Compare the final evidence against these dispositions:
  - [ ] adapter-private reset only;
  - [ ] renderer-owned replaceable resource-set boundary with
        application-owned logical identity;
  - [ ] explicit renderer release operations;
  - [ ] application/corpus helper only;
  - [ ] whole-session/page replacement;
  - [ ] no shared admission.
- [ ] Reopen AR-0024 for renderer resource-lifetime/identity implications and
      update AR-0030 with the Doom/browser evidence.
- [ ] Decide whether the accepted mechanism changes a stable/public renderer
      contract. If it does, record the decision in an ADR and update the SDD.
- [ ] Run the applicable performance, verification/recovery, provenance, and
      security gates for any Native Ring or stable shared candidate. Mark a
      gate not applicable explicitly rather than implying it passed.
- [ ] Retain rejected alternatives and the evidence that rejected them.

### Validation

- [ ] The proposed owner is the smallest layer capable of maintaining the
      invariant across both callers and both targets.
- [ ] The decision preserves application-owned scene membership and recovery
      policy while keeping provider reclamation details private.
- [ ] No accepted wording promises immediate GPU-memory reclamation unless a
      provider can actually prove it.

### Acceptance Criteria

- [ ] AR-0024 and AR-0030 have an explicit evidence-backed disposition.
- [ ] README, SDD, ADRs, examples, and validation guidance agree with any
      admitted contract.
- [ ] If no shared contract is earned, the tested containment mechanism and
      its reopening triggers remain documented without stabilizing vocabulary.

## Overall Completion Gate

This plan is complete only when:

- cumulative replacement is reproducible and distinguished from map-specific
  failure;
- the persistent renderer-resource dependency graph is documented;
- one retained-session alternative survives native, browser, Doom, and an
  independent caller, or the evidence explicitly rejects retained sessions;
- logical retirement, stale references, atomic installation, and provider
  fatal states remain distinguishable;
- same-handle replacement within a live resource set remains supported;
- repeated replacement does not create new devices/surfaces or unbounded
  logical residency under the surviving alternative;
- no observation overclaims physical GPU reclamation;
- application, renderer, platform, and provider ownership remain separated;
  and
- the final disposition updates the controlling reviews and any binding design
  documents it actually changes.

## Initial Disposition

Begin with Slices 1 and 2. The strongest current hypothesis is a retained WGPU
session with a bounded, adapter-private replaceable scene-resource set, but
that is not yet a decision. Do not expose reset/release publicly until the
dependency inventory, independent caller, atomic replacement failures, and
native/browser evidence show which semantic boundary is genuinely shared.
