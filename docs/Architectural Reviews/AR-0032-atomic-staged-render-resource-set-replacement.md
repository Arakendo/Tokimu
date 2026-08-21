# AR-0032: Atomic Staged Render Resource-Set Replacement

| Field | Value |
| --- | --- |
| Status | Accepted |
| Opened | 2026-08-19 |
| Last reviewed | 2026-08-20 |
| Scope | Foundational rendering service / provider lifetime boundary |
| Trigger | Alternative B reset falsifiers plus semantic, live WGPU, and repeated-pressure evidence for Alternative C |
| Related ADRs | ADR-0001, ADR-0003, ADR-0005, ADR-0009, ADR-0017, ADR-0018 |
| Related evidence | AR-0024; renderer scene-resource lifetime plan and its Alternative C evidence |
| Admission exception | None |

## Architectural Question

Should Tokimu admit provider-neutral atomic staged replacement semantics for a
complete render resource set, while leaving handle encoding, physical GPU
reclamation, provider synchronization, and final public API shape undecided?

The review admitted the invariant before selecting a transaction API. Cycle 5
later selected an opt-in resource-set session as the stable transaction shape;
individual handle encoding and physical provider policy remain undecided.

The candidate invariant is:

> A candidate render resource set may be constructed alongside the current
> set. Candidate failure leaves the current set authoritative and usable. A
> complete candidate commits as one observable replacement, retires the prior
> set, and prevents identities retained from the retired generation from
> resolving as successor resources merely because local keys were reused.

## Context

Tokimu currently exposes explicit mesh, texture, material, pipeline, camera,
and command resources through `tokimu-render`. Applications decide what to
present; the renderer validates and realizes those declarations. The WGPU
backend owns its device, queue, surface, bindings, and concrete GPU objects but
does not own simulation truth or application scene membership.

Repeated browser Doom map replacement created pressure between two lifetimes:

```text
provider session
    instance + device + queue + surface

composition resource set
    meshes + textures + materials + pipelines + cameras + commands
```

Whole-backend replacement can replace the second lifetime only by also
recreating the first. An adapter-private logical reset retained the provider
session but failed two required invariants: it retired the current set before
successor construction could succeed, and bare reused handles let old commands
alias successor resources.

The resulting Alternative C study separated current and candidate sets and
attached generation meaning to retained identities. It then progressed through
three distinct evidence layers rather than treating one demonstration as the
complete claim.

## Trigger And Evidence

### Semantic evidence

The shared native/WASM corpus model proves:

- candidate construction is isolated from current state;
- injected staging failure preserves and resolves the last-known-good set;
- complete candidate validation precedes one current-state replacement;
- a competing candidate rejects after its expected predecessor changes;
- retained generation-A identity rejects after B commits;
- B may reuse A's local resource key without aliasing the retained A identity;
- missing dependencies and generation exhaustion reject without mutation.

The semantic model is exercised by 24 native tests, generated WASM, an
independent heterogeneous inventory, and actual Doom E1M1/E1M2 inventory
correlation. It does not allocate provider objects.

### Real-provider evidence

The feature-gated WGPU prototype proves on browser WebGPU that:

- one backend/device/surface session can host current A and candidate B;
- 26 real candidate resources can be allocated before a late
  `MissingTexture(9)` rejection;
- A presents the same eight draws before and after failed B;
- a complete B replaces all inventoried logical resource families;
- B presents eight draws after commit;
- the run emits zero provider diagnostics.

The concrete prototype shares provider-session objects while keeping candidate
resource maps and queued commands separate. The live surface remains solely
owned by the current backend.

### Repeated-pressure evidence

One browser WGPU session completed 27 alternating replacements of:

- 64 meshes;
- 64 textures;
- 64 materials;
- one pipeline;
- one camera; and
- 64 commands.

Five cycles staged a complete candidate and then forced
`MissingTexture(65)`. All five retained and presented 64 current draws before
retry. All 27 commits reported exact retirement/installation symmetry,
presented 64 successor draws, returned to one logical live set, retained a
64-entry instance-binding cache, and emitted zero provider diagnostics.
Logical overlap was current plus candidate. Physical GPU reclamation remained
unobserved.

### Important evidence separation

The first provider prototype and the stale-identity semantic model were proven
separately. The first stable-surface candidate then integrated them but exposed
ordinary unscoped submission alongside replacement; retained declarations
could bypass set validation and alias reused successor keys. That candidate was
falsified and retained as Finding 9.

Cycle 5 closes the integration gap without choosing a permanent bit layout for
individual handles. The replacement-enabled session accepts only opaque
set-scoped command batches and rejects a retired batch before resolving any
ordinary local handle. Physical reclamation remains separate and unobserved.

## Ownership Analysis

### Meaning

The proposed meaning is an observable replacement transaction over
presentation resources:

```text
current remains authoritative
    while candidate is incomplete

candidate failure
    -> current remains authoritative

candidate validation + commit
    -> candidate becomes current once
    -> prior set becomes retired
    -> retired identity cannot resolve into current
```

This is not simulation state, source-format preparation, asset loading, or GPU
memory management.

### Owners

- Applications/compositions own which declarations belong to a candidate and
  when replacement is requested.
- `tokimu-render`, as a foundational presentation service, should own the
  provider-neutral current/candidate/commit/retired invariants if admitted.
- Concrete renderer backends own allocation, provider-session reuse, object
  destruction, queue/fence mechanics, and any provider-specific inability to
  stage.
- `tokimu-assets` continues to own asset identity and its separate lifecycle;
  renderer generations must not silently redefine asset generations.
- `tokimu-core` and `tokimu-runtime` must not own render resource sets, GPU
  objects, scene membership, or replacement policy.
- The renderer must not use replacement ownership to become the owner of world
  state, transforms, visibility, or source-domain preparation.

The proposal therefore belongs, if accepted, in `tokimu-render`, not the
kernel and not a new capability crate.

## Dependency Direction

```text
Current experiment:

corpus composition
    -> feature-gated WgpuBackend stage methods
    -> shared WGPU provider session

separate semantic shadow
    -> corpus-local generation model

Proposed semantic direction:

application/composition replacement intent
    -> Tokimu-owned render resource-set transaction semantics
    -> renderer backend realization
    -> WGPU or another provider's allocation/retirement mechanics
```

No WGPU object may cross upward into provider-neutral or author-facing APIs.
Provider errors remain distinguishable from transaction validation errors.
The proposal adds no dependency from core/runtime/assets to WGPU and creates no
new crate.

## Alternatives Considered

### Alternative A: Admit Narrow Set-Level Replacement Semantics

Admit only current/candidate isolation, validate-before-commit, atomic
observable promotion, prior-set retirement, and cross-generation stale
rejection.

- Benefits: directly answers both retained Alternative B falsifiers; matches
  semantic, provider, and pressure evidence; preserves provider replacement;
  keeps applications in control of scene membership.
- Costs: requires a provider-neutral combined identity/transaction proof and a
  migration from the feature-gated WGPU-only seam.
- Failure mode: an underspecified boundary could appear atomic while stale
  bare commands still alias successor resources.

### Alternative B: Retain Staging As Backend-Private WGPU Behavior

Keep the current feature-gated mechanism and admit no shared semantic contract.

- Benefits: smallest immediate API surface; no cross-provider promise.
- Costs: every renderer consumer must reproduce failure containment and stale
  identity policy; other backends may diverge silently.
- Failure mode: application-specific staging shadows become multiple sources
  of truth, recreating the exact reset/aliasing defects the study isolated.

### Alternative C: Admit A General Per-Resource Allocator Or Arena

Expose create/release/reclaim operations or a generalized render-resource
arena.

- Benefits: could support incremental streaming and fine-grained release.
- Costs: introduces substantially more lifetime, synchronization, dependency,
  and handle policy than the evidence requires.
- Failure mode: provider mechanics become the public abstraction while atomic
  whole-set replacement remains optional or incorrectly composed.

No demonstrated caller requires incremental release, so this alternative is
not earned.

### Alternative D: Retain Whole-Backend Replacement

Continue replacing backend/device/surface for each composition set.

- Benefits: simple ownership and complete isolation.
- Costs: couples provider-session and composition lifetimes; cannot retain the
  last valid backend on the same surface while constructing its successor;
  introduces unnecessary browser/device churn.
- Failure mode: replacement pressure and failure handling remain host/provider
  session problems rather than bounded resource-set transitions.

### Alternative E: Admit Final Generational Handle Encoding Now

Choose packed generations, set IDs, arena slots, or another permanent handle
representation as part of this review.

- Benefits: could make one implementation concrete immediately.
- Costs: confuses an earned stale-rejection invariant with one unearned storage
  representation and prematurely couples assets, commands, and providers.
- Failure mode: a convenient WGPU/corpus encoding becomes Tokimu-wide meaning
  before independent implementation pressure tests it.

## Findings

1. Atomic staged set replacement is broader than WGPU mechanics and narrower
   than scene ownership. It is foundational render-service meaning.
2. Alternative B is falsified for both last-known-good preservation and stale
   cross-set identity.
3. Alternative C has semantic, live-provider, and bounded repeated-pressure
   support on native/WASM compositions and browser WGPU.
4. Bounded logical overlap is proven. Bounded physical VRAM overlap, exact
   reclamation timing, and leak freedom are not.
5. The original WGPU prototype proved provider realization but did not itself
   implement cross-generation stale rejection. The selected resource-set
   session now integrates provider staging and set-scoped rejection; the
   historical separation remains relevant to what the earlier evidence proved.
6. The renderer should own transaction invariants, not candidate membership or
   application continuation policy.
7. No evidence requires a per-resource allocator, public arena, reclamation
   heuristic, device-loss recovery contract, or final handle encoding.
8. One concrete provider family, WGPU, has been exercised across native and
   browser targets. A second rendering backend is absent. This limits claims
   about provider implementation universality but does not make the proposed
   semantics WGPU-specific.
9. The stable-surface command-batch candidate has a bypass: the existing
   `Renderer::submit(&[RenderCommand])` accepts retained ordinary A commands
   without set validation. After B reuses A's local keys, that path can resolve
   the retained commands against B. Correct rejection through
   `submit_render_command_set` does not satisfy the accepted invariant while
   the unscoped path remains equally public and usable.
10. Finding 9 is resolved by separating ordinary rendering from replacement
    authority structurally. An ordinary backend supports raw submission but
    cannot replace its resource set. Entering replacement mode consumes it into
    a resource-set session that exposes lifecycle operations and scoped command
    submission, does not implement `Renderer`, and does not expose the
    underlying backend. Retained declarations can be scoped only as an explicit
    authorization for the current set; a retired scoped batch rejects before
    local resource resolution.

## Proposed Admission Boundary

If accepted, the resulting ADR should admit this and no more:

> Tokimu rendering supports atomic staged replacement of a bounded render
> resource set. The current set remains authoritative until a complete
> candidate validates and commits. Candidate failure preserves the current
> set. Commit promotes the candidate as one observable state transition,
> retires the prior set, and requires retained identities from the retired set
> to reject rather than alias resources in the successor set.

Explicit non-decisions:

- exact handle encoding;
- per-resource versus set-level generation storage;
- GPU reclamation timing or memory budgets;
- provider-specific drop, queue, fence, or polling policy;
- device-loss recovery or provider-session recreation;
- incremental resource release;
- final builder, closure, transaction, or trait method shape;
- ownership of application scene membership or replacement timing.

## Disposition

**Accepted -- Alternative A: narrow set-level replacement semantics.**

ADR-0018 records the accepted semantic boundary and the SDD renderer section
now reflects its ownership and lifecycle invariant. The feature-gated
experimental implementation was not promoted unchanged. The stable
resource-set session integrates provider staging with stale-command authority;
native and live browser WGPU conformance pass.

## Consequences

If Alternative A is accepted:

- `tokimu-render` gains a transactional resource-set invariant without gaining
  scene or simulation ownership;
- backends may implement staging differently but must preserve the same
  observable failure and commit behavior;
- applications may retain the last valid presentation while preparing a
  successor;
- cross-set references require enough scoped identity to reject retired sets,
  without prescribing one public handle representation;
- WGPU remains private and physical reclamation claims remain provider-
  specific evidence;
- native and WASM must share contract tests even where provider storage differs;
- the current experimental WGPU methods remain provisional until a combined
  semantic/provider implementation passes the accepted contract.

## Required Follow-Up

- [x] Maintainer accepted Alternative A's proposed admission boundary.
- [x] Created ADR-0018 for the set-level semantics and updated
      the SDD renderer section.
- [x] Designed a provisional provider-neutral set-scoped command batch without
      choosing a permanent handle bit layout.
- [x] Integrated generation/set scope with real provider commands and proved an
      old retained command rejects after successor key reuse.
  - [x] Integrated validation before WGPU command resolution and passed native
        tests plus the release WASM build.
  - [x] Retained the live browser WGPU success record: A remained at eight
        draws after late failure, stale A rejected as set 1 against current set
        3, and scoped B presented eight draws with zero provider diagnostics.
- [x] Prove failed provider staging preserves all current resource families
      through the provider-neutral contract.
- [x] Exercise the accepted contract on native and browser WGPU plus the
      independent resource-rich caller.
  - [x] Native WGPU and release-WASM compilation pass through the selected
        resource-set session.
  - [x] Retained a live browser WGPU record through the selected session: late
        all-family failure preserved eight A draws, stale A rejected before
        resolution, B presented eight draws, and provider diagnostics remained
        zero.
- [ ] Keep physical reclamation, incremental release, and device-loss recovery
      outside implementation acceptance unless separately admitted.
- [x] Replaced the feature-gated staging API in the implementation candidate
      with the narrow `RenderResourceSetLifecycle` surface while retaining the
      reset experiment separately and keeping the old staging feature as a
      compatibility no-op.
- [x] Resolved Finding 9 structurally: replacement consumes the backend into a
      resource-set session with no ordinary `Renderer::submit` surface. A
      compile-fail contract test guards the boundary; this is not caller advice.

## Reopening Triggers

After disposition, reopen or supersede this review if:

- a second renderer backend cannot preserve the admitted observable semantics;
- integrated provider identity cannot reject a stale retained command without
  choosing materially broader identity ownership;
- a real caller requires incremental release rather than whole-set replacement;
- bounded staging overlap causes demonstrated provider allocation failure or
  requires a public memory-budget policy;
- device-loss recovery proves inseparable from ordinary set replacement;
- asset and render generation semantics cannot remain separate;
- provider-specific objects or synchronization leak into the proposed
  provider-neutral boundary; or
- a simpler existing renderer concept can express the full invariant without a
  new stable lifecycle surface.

## Review History

### Cycle 1 -- 2026-08-19

- Status entering review: Proposed
- New evidence: Alternative B atomicity and aliasing falsifiers; Alternative C
  semantic generation proof; heterogeneous Doom and independent inventory
  correlation; live one-shot browser WGPU staging; 27-cycle browser WGPU
  pressure with five contained late failures.
- Participants or reviewers: maintainer and Codex
- Findings: narrow set-level transaction semantics are supported; physical
  reclamation, final API shape, and final handle representation are not. Stale
  rejection remains to be integrated with real provider commands.
- Disposition: Under Review; Alternative A recommended for maintainer decision.
- Resulting ADR or documentation change: none yet.

### Cycle 2 -- 2026-08-19

- Status entering review: Under Review
- New evidence: maintainer review confirmed that the three evidence layers
  support the narrow semantic contract while Finding 5 remains an
  implementation-conformance gate.
- Participants or reviewers: maintainer, Monday, and Codex
- Findings: architecture admission does not require choosing the final handle
  representation or physical reclamation policy; it does require the retained
  command from retired set A to reject after set B reuses A's local keys.
- Disposition: Accepted -- Alternative A, narrow set-level replacement
  semantics.
- Resulting ADR or documentation change: ADR-0018 and the SDD renderer
  lifecycle update.

### Cycle 3 -- 2026-08-20

- Status entering review: Accepted with implementation-conformance Finding 5
  open.
- New evidence: native authority tests, release WASM compilation, and a live
  browser WGPU run retaining A's real command batch across B's commit with
  reused local resource keys.
- Participants or reviewers: maintainer and Codex
- Findings: set-scoped validation rejects retired A before ordinary handle
  resolution; the failed candidate leaves A presentable; current scoped B
  remains usable after commit.
- Disposition: Finding 5 closed for the feature-gated experimental candidate;
  stable contract realization and the existing non-decisions remain open.
- Resulting ADR or documentation change: integrated command conformance
  evidence retained; no change to ADR-0018's admitted boundary.

### Cycle 4 -- 2026-08-20

- Status entering review: Accepted; stable contract realization open.
- New evidence: provider-neutral lifecycle and command-set unit tests, default
  replacement failure/commit exercise, ordinary native compilation, and release
  WASM compilation of the independent resource-rich WGPU caller without the
  staging feature gate.
- Participants or reviewers: maintainer and Codex.
- Findings: the candidate preserves provider-owned population and leaves
  physical reclamation undecided. Native WGPU passed the all-family failure and
  retained-command sequence, but ordinary unscoped `Renderer::submit` bypasses
  the candidate's command-set validation after key reuse.
- Disposition: accepted ADR semantics remain; stable implementation promotion
  is paused on Finding 9. Browser rerun is secondary until submission authority
  is resolved.
- Resulting ADR or documentation change: SDD and ADR-0018 implementation record
  updated; feature-gated staging seam retired; native WGPU conformance evidence
  retained.

### Cycle 5 -- 2026-08-20

- Status entering review: Accepted; stable implementation paused on Finding 9.
- New evidence: an opt-in `WgpuResourceSetSession` consumes the ordinary
  backend, implements the provider-neutral lifecycle, exposes only scoped
  command submission, and does not implement `Renderer` or expose the backend.
  Native WGPU passed the complete failure/commit/stale-rejection sequence with
  reused local keys and zero provider diagnostics. Release WASM compilation
  passed against the same surface.
- Participants or reviewers: maintainer and Codex.
- Findings: raw submission and set replacement need not coexist on one public
  object. Separating them makes the accepted stale-identity invariant
  structural without choosing individual handle encoding or physical
  reclamation policy.
- Disposition: Finding 9 closed. Stable native conformance passes; live browser
  WGPU confirmation remains the implementation acceptance gate.
- Resulting ADR or documentation change: ADR-0018 implementation record, SDD,
  renderer-reliability plan, and native evidence updated to the selected
  resource-set session transaction shape.

### Cycle 6 -- 2026-08-20

- Status entering review: Accepted; live browser implementation conformance
  pending.
- New evidence: Browser WASM/WebGPU completed the stable resource-set session
  sequence with one retained backend/device/surface, a late all-family failure,
  eight A draws before and after failure, atomic B commit, stale A rejection
  before reused-key resolution, eight B draws through current scoped
  submission, an absent raw-submit surface, and zero provider diagnostics. An
  observer outside the page recorded a completed terminal outcome.
- Participants or reviewers: maintainer and Codex.
- Findings: the selected stable transaction surface now has native and browser
  real-provider conformance. Physical reclamation and individual handle
  encoding remain unproven and undecided.
- Disposition: stable cross-target implementation conformance complete for the
  admitted ADR-0018 transaction invariant.
- Resulting ADR or documentation change: stable browser evidence retained and
  the ADR, SDD, review, and renderer-reliability checklist updated.

## References

- `docs/contribution-admission-guide.md`
- `docs/Tokimu Software Design Document.md`
- `docs/ADR/ADR-0001-engine-boundaries.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
- `docs/ADR/ADR-0009-ring-based-verification-failure-containment-and-recovery.md`
- `docs/ADR/ADR-0017-observable-terminal-failure-and-host-crash-conformance.md`
- `docs/ADR/ADR-0018-atomic-staged-render-resource-set-replacement.md`
- `docs/Architectural Reviews/AR-0024-renderer-failure-observation-and-diagnostic-boundary.md`
- `docs/Plans/Renderer-Reliability/renderer-scene-resource-lifetime-and-replacement.md`
- `docs/Plans/Renderer-Reliability/Evidence/renderer-scene-resource-alternative-c-semantic-generation-evidence.md`
- `docs/Plans/Renderer-Reliability/Evidence/renderer-scene-resource-alternative-c-real-provider-staging-evidence.md`
- `docs/Plans/Renderer-Reliability/Evidence/renderer-scene-resource-alternative-c-repeated-provider-pressure-evidence.md`
- `docs/Plans/Renderer-Reliability/Evidence/renderer-scene-resource-adr-0018-integrated-command-conformance.md`
- `docs/Plans/Renderer-Reliability/Evidence/renderer-scene-resource-adr-0018-stable-native-conformance.md`
- `docs/Plans/Renderer-Reliability/Evidence/renderer-scene-resource-adr-0018-stable-browser-conformance.md`
