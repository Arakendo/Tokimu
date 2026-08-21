# AR-0033: Scoped In-Set Presentation Resource Updates

| Field | Value |
| --- | --- |
| Status | Accepted — ADR-0019 |
| Opened | 2026-08-20 |
| Last reviewed | 2026-08-20 |
| Scope | Foundational rendering service / presentation-resource authority boundary |
| Trigger | The persistent browser Doom console needs repeated small text texture or mesh changes while one ADR-0018 resource set remains authoritative |
| Related ADRs | ADR-0001, ADR-0003, ADR-0005, ADR-0009, ADR-0017, ADR-0018, ADR-0019 |
| Related evidence | AR-0007, AR-0024, AR-0030, AR-0032, Doom D.1 debug-console evidence |
| Admission exception | None |

## Architectural Question

What bounded update, if any, may replace presentation data inside one current
render resource set without weakening set identity, atomic whole-set
replacement, scoped-command validation, or retired-set stale rejection?

The first pressure is deliberately narrower than a general mutable-resource
system:

```text
authoritative resource set remains current
    -> one existing presentation resource receives new realization data
    -> already scoped composition continues against that same set
```

The review must determine whether the shared meaning is replacement of the
realization behind an existing set-scoped identity, an explicitly dynamic
resource class, submission-local data, or no new stable contract.

## Context

ADR-0018 admits an atomic transaction for replacing a complete bounded render
resource set. Its selected WGPU shape consumes a backend into a resource-set
session that exposes set-scoped command submission and a deliberate live-camera
upload. It structurally withholds ordinary raw submission and does not expose
the backend. This prevents retired commands from resolving reused successor
keys.

Ordinary per-frame commands and camera data do not answer a different need:
small persistent presentation data may change while the logical composition and
its authoritative set remain the same. The browser Doom console exposes that
gap. Prompt edits and transcript appends change a raster texture and possibly
its presentation mesh on nearly every input event. Replacing the complete map
resource set per keystroke is a correctness control, but it may be the wrong
semantic and operational unit.

The review does not assume that a broad `DynamicResource` category is needed.
A narrower operation such as replacing the provider realization behind one
existing, set-scoped resource identity may be sufficient.

## Trigger And Evidence

### Current corpus evidence

- The native Doom D.1 console updates a bounded transcript and prompt through
  Tokimu's ordinary textured-2D presentation path.
- The browser Doom working model now retains one input/frame loop and one
  ADR-0018 resource-set session through movement and map replacement.
- The replacement-enabled session exposes only scoped command submission and
  live camera upload. It has no texture or mesh update surface.
- The console raster changes repeatedly without representing a new map,
  composition, provider session, or simulation state.
- A DOM overlay could display the text, but would not prove D.1's intended
  Tokimu presentation/render composition.

### Existing guarantees that must survive

- Current-set authority remains unambiguous.
- Retired or foreign command sets reject before local resource lookup.
- Raw submission cannot bypass replacement-session authority.
- Failed update preparation leaves the prior presentation resource usable.
- Whole-set replacement remains atomic and does not race an in-set update.
- The renderer does not become owner of console, UI, map, or simulation truth.
- Native and browser provider failures remain observable under ADR-0017.

### Missing evidence

- no provider-neutral prototype of an in-set texture or mesh update;
- no failure-injection proof that the old realization survives a failed update;
- no ordering rule between update visibility, frame submission, and whole-set
  commit;
- no repeated-pressure measurement for small updates;
- no independent non-console caller;
- no evidence that texture and mesh changes require the same contract;
- no decision on whether descriptor/topology changes are allowed or only
  same-shape content replacement.

## Ownership Analysis

The application or composition owns the intent to update a presentation
resource and owns the semantic source data. `tokimu-render` would own only the
provider-neutral authority and failure invariant if a stable operation is
admitted. A backend would continue to own upload, allocation, synchronization,
and concrete replacement mechanics.

The proposed meaning is not kernel-native and must not enter `tokimu-core` or
`tokimu-runtime`. It belongs, if earned, beside existing render resource and
resource-set semantics in `tokimu-render`. It must not own UI layout, text,
glyph selection, Doom diagnostics, scene membership, simulation state, or
physical reclamation policy.

## Dependency Direction

```text
Current:

application/composition update intent
    -> ordinary backend same-handle mechanics OR no replacement-session path

replacement-enabled composition
    -> ADR-0018 resource-set session
    -> scoped commands + camera update only

Proposed candidate:

application/composition update intent
    -> provider-neutral set-scoped update authority
    -> replaceable backend realization
```

No candidate may expose `WgpuBackend`, accept WGPU objects, restore unscoped
submission, or make a browser/DOM mechanism the semantic contract.

## Required Candidate Invariants

Any candidate stronger than the whole-set control must prove:

1. The current resource-set identity does not change merely because an
   authorized member's realization changes.
2. The target identity belongs to the current renderer session and current
   resource set; retired and foreign identities reject before local lookup.
3. Preparation or provider failure leaves the previous realization
   authoritative and presentable.
4. Success becomes visible at one defined frame/update boundary; partial
   texture bytes, mesh vertices, bindings, or dependent state are not visible.
5. Existing scoped commands either remain valid by contract or reject
   explicitly; they cannot silently bind an unintended resource.
6. Whole-set commit and in-set update ordering is deterministic. An update
   cannot land in a retired or candidate set accidentally.
7. Repeated updates remain bounded diagnostically and do not imply physical
   GPU reclamation guarantees that were not observed.
8. Simulation and semantic source truth remain outside the renderer.

## Alternatives Considered

### Alternative A: Whole-Set Staged Replacement Per Edit

- Benefits: uses the already accepted ADR-0018 invariant; supplies the
  correctness control; adds no new stable mutation authority.
- Costs: reconstructs and stages unrelated map resources for tiny console
  changes; turns stable composition identity over on every edit; may create
  unnecessary overlap and command regeneration.
- Failure mode: semantically safe but operationally disproportionate, masking
  a missing smaller unit behind repeated scene transactions.

### Alternative B: Scoped Replacement Behind An Existing Identity

- Meaning: replace the realization or content associated with one existing
  resource identity in the current set; do not change set identity.
- Benefits: narrowest likely shared contract; existing scoped commands can
  continue if dependency and visibility rules are precise.
- Costs: requires atomic failure semantics and ordering with whole-set commit;
  texture, mesh, material, and pipeline replacement may not share one honest
  rule.
- Failure mode: a bare same-handle overwrite exposes partial state, lets an
  update target a retired set, or makes old commands observe unintended data.

### Alternative C: Explicit Scoped Dynamic Resource Class

- Meaning: resources declared dynamic at creation receive a bounded update
  vocabulary distinct from immutable set members.
- Benefits: makes mutability and possible provider allocation policy explicit;
  can reject updates to resources that did not opt in.
- Costs: introduces a new resource category and lifecycle vocabulary before
  evidence proves that mutability class, rather than scoped identity, is the
  shared semantic.
- Failure mode: provider usage flags or buffer strategies leak into public
  meaning, producing a large mutable-resource subsystem from one console.

### Alternative D: Submission-Local Or Transient Presentation Data

- Meaning: express changing console geometry or pixels as data owned by one
  scoped submission rather than as persistent set members.
- Benefits: avoids persistent replacement identity where persistence is not
  required; may fit per-frame text or procedural geometry.
- Costs: current submission-local geometry does not establish text textures,
  bounded upload lifetime, or compatibility with the replacement session;
  rebuilding complete transient pixels every frame may still be wasteful.
- Failure mode: transient data becomes an unscoped authority bypass or grows a
  second hidden resource store outside ADR-0018 validation.

### Alternative E: External Browser DOM Overlay

- Benefits: simple browser implementation; no GPU resource mutation.
- Costs: bypasses the Tokimu presentation proof, is browser-specific, and
  produces no native/WASM render-path parity evidence.
- Failure mode: a platform workaround is mistaken for engine evidence while
  other dynamic presentation callers remain unsolved.
- Role in review: negative architectural control, not the preferred D.1
  implementation.

### Alternative F: Continue Incubation Without Browser Console

- Benefits: preserves current structural authority and avoids premature API.
- Costs: leaves the now-reproducible persistent-browser caller incomplete.
- Failure mode: later callers invent independent update paths before shared
  invariants are studied.

## Study Plan

### Slice 0: Freeze The Pressure And Correctness Control

- [x] Inventory the exact console resources that change per edit and those that
      remain stable.
- [x] Implement or model Alternative A privately to establish that whole-set
      replacement preserves correctness and to measure its amplification.
- [x] Retain command/set identities, CPU work, logical overlap, and provider
      diagnostics without claiming physical reclamation.
  - [x] Retain current set identity, command regeneration, and modeled logical
        overlap/amplification.
  - [x] Measure the actual whole-set control's CPU interval and provider
        diagnostics alongside the first real-provider candidate. The accounting
        model deliberately does not manufacture timing evidence.

Slice 0 found one changing fixed-size RGBA8 texture. The console quad,
material dependency, pipeline, camera, and open-console command topology remain
stable. Against the retained E1M1 inventory, Alternative A would stage 1,241
logical persistent resources and regenerate 2,069 commands for that one
changing texture. The modeled source-byte amplification is approximately
3.13x before repeated WAD/map preparation or provider-private work. See
[AR-0033 Slice 0 console whole-set control evidence](../Plans/Renderer-Reliability/Evidence/AR-0033%20Slice%200%20console%20whole-set%20control.md).

### Slice 1: Provider-Neutral Semantic Shadows

- [x] Prototype Alternatives B, C, and D behind corpus-private types.
- [x] Inject failure after candidate data preparation and after partial provider
      allocation; prove the prior realization remains usable.
- [x] Prove retired-set and foreign-session targets reject before lookup.
- [x] Define and test ordering against frame submission and whole-set commit.

Alternative B is the smallest surviving persistent-resource candidate. C adds
a resource-class eligibility gate but still requires B's scoped transaction
machinery. D survives abstractly without persistent identity, but no current
transient texture payload proves it can present the console raster. Advancing B
to real-provider evidence now requires an explicitly authorized experimental
authority shape because the ADR-0018 session correctly exposes no in-set
texture mutation. See
[AR-0033 Slice 1 semantic-shadow evidence](../Plans/Renderer-Reliability/Evidence/AR-0033%20Slice%201%20semantic%20shadows.md).

### Slice 2: Real Provider And Repeated Pressure

- [x] Exercise the smallest surviving candidate on native WGPU and browser
      WebGPU without exposing raw backend submission.
- [x] Run repeated prompt/transcript updates with bounded logical counts and
      externally observed terminal outcomes.
- [x] Distinguish logical update completion from physical allocation reuse or
      reclamation.

The feature-gated provider experiment preserves the existing session authority
shape: it exposes a texture-only candidate/commit operation on the scoped
session, while provider-internal access remains private and raw submission
remains structurally absent. Native WGPU and browser WebGPU both proved dropped
candidate preservation, an atomic fixed-descriptor realization swap, unchanged
set identity, continued presentation through the same command batch, and stale
candidate rejection after a whole-set commit. Browser pressure then completed
27 console-sized updates with five prepared drops under the external terminal
observer. See
[AR-0033 Slice 2 provider and pressure evidence](../Plans/Renderer-Reliability/Evidence/AR-0033%20Slice%202%20provider%20and%20pressure.md).

### Slice 3: Independent Caller

- [x] Add one non-console, resource-changing caller such as a procedural
      texture, brush preview, streaming text atlas, or dynamic mesh.
- [x] Determine whether texture and mesh pressure converge on one semantic or
      require separate narrow contracts.

The renderer-resource-identity corpus is the independent procedural-texture
caller. It exercises generated 16x16 RGBA8 content independently of Doom and
the console-sized pressure path. Both callers converge on fixed-descriptor
texture-content replacement. Neither exercises mesh content or topology, so
mesh mutation remains outside the candidate rather than being generalized by
analogy.

### Slice 4: Admission Review

- [x] Compare surviving candidates against Alternative A and the independent
      caller.
- [x] Admit only the smallest provider-neutral invariant supported by both
      callers, or retain the behavior as corpus/provider-local.
- [x] Revise ADR-0018 only if its whole-set invariant changes; otherwise create
      a separate ADR for the orthogonal in-set update contract.

## Findings

1. The browser lifecycle prerequisite for D.1 is now satisfied.
2. The remaining blocker is update authority, not keyboard transport, command
   parsing, text rasterization, or map preparation.
3. ADR-0018 deliberately does not admit incremental release or change
   same-handle replacement semantics within an authoritative set. Its
   structural no-bypass property must remain intact.
4. Whole-set replacement is the known-safe control, not yet the selected
   implementation.
5. The console proves one real caller but does not justify a broad dynamic
   resource class.
6. A second independent caller is required before stable admission.
7. Physical GPU allocation reuse and reclamation remain provider observations,
   not required semantic guarantees.
8. Slice 0 confirms that the concrete first edit changes texture content only;
   it does not yet justify shared texture/mesh or descriptor/topology mutation.
9. Whole-set replacement remains correct but is a disproportionate unit for
   the E1M1 control: 1,241 staged logical resources and 2,069 regenerated
   commands for one changing raster texture.
10. B, C, and D all survive the corpus-private failure/scope/ordering shadow.
    B is smallest for a persistent texture; C adds classification without
    removing transaction requirements; D lacks a real transient-texture path.
11. A real B experiment cannot use the accepted replacement session without
    adding deliberately bounded experimental authority. That provider-test
    surface must not be smuggled in as a stable session method or backend
    escape hatch.
12. A feature-gated texture-only session transaction passed native WGPU and
    browser WebGPU failure, commit, unchanged-command, and whole-set ordering
    checks without exposing raw submission.
13. Twenty-seven console-sized browser updates, including five fully prepared
    candidate drops, retained one resource set and fixed logical inventory with
    zero provider diagnostics. The external observer classified the run
    `completed`.
14. Logical completion and provider-object replacement were observed; physical
    allocation reuse and reclamation were not.
15. The independent procedural-texture caller converges with the console on
    fixed-descriptor texture-content replacement only. Mesh and descriptor
    mutation remain unsupported and unadmitted.

## Disposition

**Accepted — Alternative B, narrow texture-content form.** ADR-0019 admits
atomic replacement of content behind one existing texture identity in the
current authoritative set when descriptor and semantic role remain unchanged.
Alternative A remains the correctness control; C's dynamic classification and
D's unproven transient texture path are not admitted. Meshes, descriptor
changes, material-semantic rebinding, pipelines, raw backend access, unscoped
submission, and physical reclamation guarantees remain outside the decision.

## Consequences

- Browser D.1 remains paused without weakening its acceptance criteria.
- ADR-0018 remains binding and unchanged.
- The provider-neutral candidate/commit lifecycle is stable; provider
  candidates and allocation mechanics remain provider-owned.
- Any provider-backed experiment must retain scoped authority and terminal
  outcome observation.
- Texture and mesh update semantics may be split if evidence does not support a
  single honest contract.

## Required Follow-Up

- [x] Record the D.1 browser finding and alternatives.
- [x] Implement Slice 0 correctness-control evidence.
- [x] Prototype surviving provider-neutral semantic shadows.
- [x] Exercise native and browser provider failure behavior.
- [x] Add one independent non-console caller.
- [x] Decide whether an ADR, ADR-0018 revision, or no stable admission follows.

Decision: create separate ADR-0019. ADR-0018 remains unchanged because
whole-set authority turnover and current-set texture-content replacement are
orthogonal transactions with an explicit ordering rule.

## Reopening Triggers

This review is already open. If deferred or rejected, reopen when:

- a second independent caller needs repeated in-set presentation updates;
- whole-set replacement demonstrates material bounded-performance or overlap
  pressure under the console workload;
- a provider cannot preserve the required failure/visibility invariant;
- an existing accepted transient/presentation seam satisfies the need without
  new authority; or
- implementation evidence shows that updates cannot remain orthogonal to
  ADR-0018 whole-set replacement.

## Review History

### Cycle 1 -- 2026-08-20

- Status entering review: Proposed
- New evidence: persistent browser Doom input/frame lifecycle; stable ADR-0018
  resource-set session; native changing-console presentation; missing scoped
  texture/mesh update authority.
- Participants or reviewers: maintainer and Codex
- Findings: the console exposes an architectural update category between
  whole-set replacement and ordinary per-frame command/camera changes.
- Disposition: Proposed; authorize corpus-private comparative study, withhold
  stable admission.
- Resulting ADR or documentation change: none; ADR-0018 remains unchanged.

### Cycle 2 -- 2026-08-20

- Status entering review: Proposed; admission gate reached
- New evidence: native and browser provider execution; externally closed
  27-update browser pressure; prepared-candidate failure preservation; stale
  ordering against whole-set commit; independent procedural-texture caller.
- Participants or reviewers: maintainer and Codex
- Findings: fixed-descriptor texture-content replacement is the smallest shared
  semantic. Dynamic-resource classification adds no value, and no evidence
  generalizes the operation to meshes or descriptor mutation.
- Disposition: Accepted — Alternative B, narrow texture-content form.
- Resulting ADR or documentation change: ADR-0019 and stable
  `RenderTextureContentUpdateLifecycle`; ADR-0018 remains unchanged.

## References

- `docs/ADR/ADR-0018-atomic-staged-render-resource-set-replacement.md`
- `docs/ADR/ADR-0019-fixed-descriptor-set-scoped-texture-content-replacement.md`
- `docs/Architectural Reviews/AR-0007-semantic-ui-composition-boundary.md`
- `docs/Architectural Reviews/AR-0024-renderer-failure-observation-and-diagnostic-boundary.md`
- `docs/Architectural Reviews/AR-0030-source-owned-presentation-preparation-boundary.md`
- `docs/Architectural Reviews/AR-0032-atomic-staged-render-resource-set-replacement.md`
- `docs/Plans/DOOM/DOOM WAD Checklist.md`
- `docs/Plans/DOOM/Evidence/D1 debug console evidence.md`
- `docs/Plans/Renderer-Reliability/Evidence/AR-0033 Slice 0 console whole-set control.md`
- `docs/Plans/Renderer-Reliability/Evidence/AR-0033 Slice 1 semantic shadows.md`
