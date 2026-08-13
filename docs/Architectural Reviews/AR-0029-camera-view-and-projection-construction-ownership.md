# AR-0029: Camera, View, And Projection Construction Ownership

| Field | Value |
| --- | --- |
| Status | Accepted — Narrow B admitted by ADR-0014 |
| Opened | 2026-08-12 |
| Last reviewed | 2026-08-13 |
| Scope | Foundational pure-math construction at the camera/presentation boundary |
| Trigger | `glam` 0.33.3 moves 86 repository uses of three deprecated matrix constructors toward a new foreign `glam::camera` vocabulary |
| Related ADRs | ADR-0003, ADR-0005, ADR-0008, ADR-0009, ADR-0010, ADR-0014 |
| Related evidence | AR-0019, AR-0024, AR-0026, AR-0028, SDD camera contract, Option A update plan |
| Admission exception | None |

## Architectural Question

Does Tokimu own the narrow, provider-neutral semantics for constructing its
right-handed view matrices and GL-depth perspective/orthographic projection
matrices while keeping the selected numerical implementation private?

## Context

Tokimu currently exposes five audited `glam` types through
`tokimu_core::math`. Callers construct views and projections through associated
`Mat4` functions. The 0.33.3 update candidate deprecates the three functions
Tokimu uses and redirects callers into `glam::camera::*`.

Following that migration literally would expand Tokimu's foreign public
vocabulary in response to upstream API organization. Suppressing the warnings
would violate the update study. Remaining on 0.29.3 would avoid the immediate
decision but preserve the porous semantic boundary.

Independent Tokimu evidence already assigns meaning here:

- the SDD and AR-0024 retain right-handed, Y-up camera construction and
  GL-style `[-1, 1]` clip depth as Tokimu meaning;
- the WGPU provider converts that depth convention privately to `[0, 1]`;
- AR-0028 distinguishes source orientation, view basis, screen direction, yaw,
  and input policy; and
- AR-0026 may require qualified views without changing ordinary matrix
  mechanics.

This review is not an Alternative C migration and does not propose Tokimu-owned
`Mat4`. It asks whether three already-repeated semantic constructors need a
Tokimu boundary above Alternative A's implementation.

## Trigger And Evidence

- Corpus examples: camera, CAD, GLB, DOOM, orientation, and chart controls use
  the constructor families.
- Automated tests: the 0.33.3 candidate passes strict core Clippy, 29 core
  tests, WASM core build, the retained math study, and representative compile
  checks.
- Audits or diagnostics: strict representative Clippy fails only because
  `look_at_rh`, `perspective_rh_gl`, and `orthographic_rh_gl` are deprecated.
- Independent consumers: repository search currently finds 86 textual uses
  across engine and corpus Rust source.
- Repeated implementation friction: camera meaning has already required
  AR-0024 depth adaptation and AR-0028 orientation controls.
- Missing evidence: an owned seam has not yet passed full caller migration,
  numerical, invalid-input, native/WASM, performance, or public-API review.

## Ownership Analysis

The meaning is limited to:

- right-handed, Y-up look-at view construction with `-Z` forward;
- right-handed perspective and orthographic projection;
- GL-style `[-1, 1]` clip depth and Y-up output;
- explicit finite parameters and defined invalid/degenerate behavior; and
- separation from provider-specific clip-depth conversion.

Tokimu already owns these camera/projection claims in its design and renderer
contract, while `glam` currently owns both the implementation and the caller
vocabulary used to express them. Under the candidate alternative, Tokimu owns
the narrow constructor contract and delegates numerical mechanics privately.

The contract must not own camera lifecycle, active-camera selection, GPU
uniform layout, backend clip-depth policy, world state, input policy, scene
topology, or chart identity.

Placement remains part of the experiment. Pure matrix construction may live in
the engine-neutral math boundary if it introduces no presentation mechanism;
camera resource lifecycle remains in `tokimu-render`.

## Dependency Direction

```text
Current:

Tokimu caller
    -> foreign Mat4 associated constructor
    -> glam camera/projection semantics and implementation

Candidate:

Tokimu caller
    -> narrow Tokimu pure-math constructor
    -> private selected math implementation
    -> glam::camera in Alternative A today

WGPU provider
    -> private Tokimu [-1,1] to WebGPU [0,1] upload conversion
```

No caller may import or re-export `glam::camera` through this investigation.

## Alternatives Considered

### Alternative A: Retain The Deprecated Foreign Constructors

- Benefits: no Tokimu API or migration work.
- Costs: warnings require suppression or an old pin; semantic callers remain
  coupled to retired upstream vocabulary.
- Failure mode: maintenance policy fossilizes around 0.29.3 or hides warnings.

### Alternative B: Tokimu-Owned Narrow Constructors, Private `glam`

- Benefits: preserves earned Tokimu semantics; contains upstream organization;
  remains compatible with Alternative A or a future owned implementation.
- Costs: a small public contract, migration, invalid-input policy, tests, and
  long-term compatibility responsibility.
- Failure mode: the seam grows into a speculative camera framework or merely
  renames all of `glam::camera`.

### Alternative C: Publicly Adopt `glam::camera`

- Benefits: direct upstream migration and documentation alignment.
- Costs: expands foreign vocabulary beyond AR-0019's admitted five types and
  lets upstream organization move Tokimu's semantic boundary.
- Failure mode: future upstream churn repeatedly becomes Tokimu architecture.

### Alternative D: Defer The Update

- Benefits: preserves the audited production state while evidence is gathered.
- Costs: retains the warning flood and recurring audit/security distance from
  current upstream.
- Failure mode: a temporary pause becomes indefinite avoidance.

## Findings

The dependency update is numerically and mechanically healthy enough that
freezing 0.29.3 is not yet justified. The update nevertheless proves that
camera/projection construction is expressed through foreign vocabulary at a
scale and semantic importance not captured by the original five-type decision.

AR-0024 and the SDD already constrain the result more narrowly than the full
upstream camera module. This supports a bounded Alternative B experiment. It
does not yet admit that experiment as stable API, settle its crate placement,
or authorize production pin movement.

## Prior Disposition — 2026-08-13

**No Change.** Retain Alternative A as the production math vocabulary and keep
the isolated Narrow B prototype as executable incubation evidence. This review
does not admit stable Tokimu-owned camera/view/projection constructors, expose
`glam::camera`, authorize a production migration, or change the selected
provider solely to resolve this ownership question.

The study demonstrated that a three-family provider-neutral seam is feasible,
bounded, provider-private, and capable of insulating representative callers
from the measured provider update. It did not complete actual-browser Narrow B
execution, actual GLB runtime/browser observation, stable API placement and
documentation, migration/rollback planning, or explicit maintainer selection
of a stable/public change. Those missing gates do not prevent a No Change
disposition; they prevent admission.

This disposition was superseded later on 2026-08-13 by Cycle 9 after the exact
0.33.3 production admission attempt supplied the named reopening pressure.

## Superseding Disposition — Cycle 9, 2026-08-13

**Admit Alternative B (Narrow B) under ADR-0014.** Keep the five already
admitted `glam` value carriers and numerical implementation, but make the
three demonstrated right-handed view/GL-depth projection constructors
Tokimu-owned checked operations in `tokimu_core::math`. Keep `glam::camera`
private.

The production 0.33.3 attempt passed the workspace tests but strict Clippy
rejected the 86-site Alternative-A vocabulary as deprecated. The honest A
choices were permanent targeted compatibility debt or public adoption of the
new provider camera vocabulary. The completed B study had already shown that
three provider-private adapter calls contain this exact update shock while
Full B adds broader costs without proportional benefit. Maintainer quorum
selected Narrow B instead of either A workaround.

Fresh 0.33.3 browser attachment was unavailable during admission. ADR-0005
therefore permits a documented substitution consisting of retained
actual-browser camera behavior on the prior exact pin, dual-provider Narrow B
contract evidence, and fresh native/Node-WASM/wasm32 validation. A fresh
0.33.3 actual-browser replay remains follow-up and must not be described as
completed evidence.

## Consequences

- The Option A update may continue in its isolated worktree.
- The prototype must expose only Tokimu types and primitives.
- Existing provider clip-depth adaptation remains unchanged.
- Direct `glam::camera` use by callers is not an accepted migration.
- Full migration cost becomes measured Option A lifecycle evidence.
- If the seam cannot remain three-family and engine-neutral, the update pauses
  rather than widening this review silently.

## Required Follow-Up

- [x] Record the dependency-update trigger and alternatives.
- [x] Prototype the three constructor families in the isolated 0.33.3 worktree.
- [x] Retain numerical and degenerate-input contracts without comparing only
      against the deprecated functions.
- [x] Migrate representative camera, renderer, CAD, DOOM, and orientation
      callers without exposing `glam::camera`.
- [x] Migrate and compile representative GLB and textured-box callers without
      exposing `glam::camera`.
- [x] Retain GLB runtime and browser-oriented construction evidence without
      expanding the seam; apply the documented ADR-0005 substitution for the
      unavailable fresh 0.33.3 browser replay.
- [x] Resolve the measured checked-construction cost before admission: retain
      unconditional validation while avoiding duplicate normalization and
      construct each caller's intended projection/view exactly once.
- [x] Place the pure checked constructors in `tokimu_core::math` and admit them
      through ADR-0014; camera lifecycle and provider clip conversion remain
      elsewhere.
- [ ] Replay the admitted camera-consuming fixture in an actual browser on the
      exact 0.33.3 tree when an attachable browser is available.

## Reopening Triggers

- the seam requires foreign camera types or provider-specific state;
- invalid-input behavior differs materially across native/WASM;
- migration requires semantics beyond the three demonstrated families;
- another math provider cannot preserve the contract;
- qualified/charted views require a different decomposition; or
- the current `glam` candidate cannot pass performance or correctness gates.

## Review History

### Cycle 9 -- 2026-08-13

- Reopening evidence: production admission of exact `glam` 0.33.3 makes the
  retained direct constructors strict-Clippy deprecations at repository scale.
- Rejected shortcuts: no global/blanket deprecation suppression and no public
  `glam::camera` vocabulary expansion.
- Quorum: maintainer acceptance plus the retained comparative review supports
  Narrow B; Full B and C remain unselected.
- Binding result: ADR-0014 admits exactly three checked semantic constructors;
  the exact 0.33.3 provider remains private implementation.
- Evidence exception: fresh actual-browser attachment was unavailable, so the
  explicitly bounded ADR-0005 substitution is retained with replay follow-up.

### Cycle 8 -- 2026-08-13

- Maintainer disposition: retain Alternative A in production and continue
  Narrow B only as incubation evidence; no stable/public B change is selected.
- Closure basis: the Option B study completed its comparative work and produced
  a bounded recommendation. The remaining browser/GLB, placement,
  documentation, migration, rollback, and acceptance work is admission work,
  not evidence that the existing architecture must change now.
- Binding result at that time: No Change. Cycle 9 later superseded this result.
- Reopening condition: independent caller pressure plus completion of the
  deferred admission gates, or provider evolution that makes the current
  direct-construction boundary materially untenable.

### Cycle 4 -- 2026-08-12

- Status entering review: Incubating; production remains on the audited
  0.29.3 provider and no stable seam is admitted.
- New evidence: the separate Option B study froze a provider-neutral contract
  and rebuilt Narrow B as an independent dual-provider candidate. The same
  public integration test passes four contract cases against exact 0.29.3 and
  0.33.3 without caller edits. Three private construction calls are the only
  provider-revision differences; bounded error categories retain operation
  identity and no public provider camera module or error leaks.
- Ordinary finding: the first perspective oracle incorrectly treated affine
  `transform_point3` as projective division. The test was corrected to derive
  NDC Z from independent clip Z/W scalars. This does not change the candidate
  contract and reinforces keeping affine and projective operations distinct.
- Placement finding: if admitted later, the smallest existing location is
  `tokimu-core::math`; pure construction can live beside the current value
  vocabulary while camera lifecycle remains in `tokimu-render` and WGPU clip
  conversion remains provider-private.
- Disposition: retain Incubating. The narrow seam now demonstrates native
  update-shock absorption, but representative runtime migration, WASM/browser,
  performance, security, and stable-admission gates remain open.

### Cycle 5 -- 2026-08-12

- Cross-review evidence: Narrow B's stereo and Doom observer callers pass
  unchanged under exact 0.29.3 and 0.33.3 providers. Existing stereo and CAD
  callers create multiple camera/view instances without asking the three-family
  constructor seam to own camera identity, viewports, or submission.
- Renderer boundary: the focused WGPU test still maps Tokimu GL-style
  `[-1, 1]` depth to `[0, 1]` only in the private camera uniform and leaves the
  source camera unchanged.
- Scope result: chart identity, qualified location, source embedding, input
  policy, portals, and recursive view orchestration do not belong in Narrow B.
  A future qualified-view caller must reopen this review rather than enlarge
  the constructor seam by convenience.
- Disposition: retain Under Review. The cross-review introduces no broader
  camera/view contract and no stable admission; the Option B comparative
  maintainer gate remains next.

### Cycle 6 -- 2026-08-12

- Option B recommendation: continue Narrow B incubation while retaining A in
  production; park Full B because it provides no additional insulation for the
  demonstrated constructor shock and carries substantially broader costs.
- Remaining evidence: actual-browser Narrow-B execution, the retained GLB
  runtime/browser gate, real-workload judgment of checked-construction cost,
  stable documentation and placement, and explicit maintainer selection.
- Binding status: remain Under Review as review guidance. Do not create an ADR
  or production migration plan unless maintainers explicitly select the narrow
  seam after the missing gates.
- Disposition: pause at maintainer judgment with production and provider pin
  unchanged.

### Cycle 7 -- 2026-08-12

- Documentation result: the then-unnumbered Proposed ADR that later became
  ADR-0014 frames Narrow B as a
  general admission procedure for Tokimu-owned semantic operations over already
  admitted mechanical values. It does not treat the initial function count as
  architecture, authorize ordinary-math wrappers, or promote domain-owned Doom
  source semantics into Tokimu ownership.
- Initial application: the three AR-0029 constructor families remain the only
  proposed seam. Broader primitive use requires independent ownership plus
  demonstrated pressure, and repeated helpers must trigger review for a
  higher-level primitive.
- Open gates: actual-browser Narrow-B execution, GLB runtime/browser evidence,
  named real-workload performance judgment, stable API/placement/documentation,
  migration/rollback, and explicit maintainer acceptance remain unchecked in
  the Proposed ADR.
- Disposition at that time: retain Under Review. The draft is
  non-authoritative and intentionally unnumbered; production and the provider
  pin remain unchanged. Cycle 9 later admits it as ADR-0014.

### Cycle 3 -- 2026-08-12

- Status entering review: Incubating.
- New evidence: the dependency-isolated plain-WASM Alternative A control
  matches exactly across 0.29.3 and 0.33.3, and all 12 focused retained
  conformance cases pass. An initial same-host release workload measured the
  checked Tokimu prototype at 14.6-14.7 ms median for 100,000 stereo-camera
  iterations. Investigation found two independent duplicated-work defects:
  the corpus caller constructed a default projection/view and then replaced
  the view, while the wrapper normalized the same basis before the private
  provider normalized it again. After repairing both without weakening
  validation, three checked-path medians were 7.907, 8.034, and 7.932 ms;
  direct provider-backed controls in the same candidate process were 8.959,
  8.960, and 9.089 ms. The separately built 0.29.3 baseline was 7.384 ms.
- Participants or reviewers: maintainer, Monday review, Codex investigation.
- Findings: the initial result was real but did not establish an inherent cost
  for Tokimu-owned camera semantics. Fallible construction prevented the
  optimizer from discarding the caller's dead default construction, exposing
  a previously hidden corpus defect; pre-normalization then duplicated work
  already performed by the provider. Finite-input and finite-result validation
  remains in force after both repairs. The final within-candidate control does
  not show a checked-boundary throughput penalty. The approximately seven
  percent difference from the separately built 0.29.3 observation is retained
  as noise-sensitive cross-build evidence, not attributed to the semantic seam.
- Disposition: resolve the checked-construction performance blocker. Retain the
  caller-shape regression and three-run measurements as evidence; stable
  admission and engine-neutral placement remain separate open decisions.

### Cycle 2 -- 2026-08-12

- Status entering review: Incubating.
- New evidence: Monday's independent review agreed that the update provider is
  healthy while the 86-call camera/projection vocabulary boundary is not. The
  review explicitly separates a Tokimu-owned semantic seam from AR-0019
  Alternative C and rejects public `glam::camera` adoption as the default.
- Participants or reviewers: maintainer, Monday review, Codex investigation.
- Findings: Alternative B is the preferred bounded direction; the contract
  must retain handedness, forward/up, GL clip depth, invalid-input behavior,
  perspective/orthographic parameters, cross-target equivalence, and separate
  provider depth adaptation.
- Disposition: continue the isolated prototype and source audit; do not move
  the production pin before the remaining evidence and stable-admission gate.

### Cycle 1 -- 2026-08-12

- Status entering review: Proposed.
- New evidence: the audited 0.33.3 candidate clears the generated warning flood
  and passes narrow validation, but redirects 86 existing uses into new
  foreign camera vocabulary.
- Participants or reviewers: maintainer, Monday review, Codex investigation.
- Findings: the implementation provider remains viable; the exposed semantic
  constructor boundary does not.
- Disposition: Incubating; authorize Alternative B only as an isolated bounded
  prototype.
- Resulting ADR or documentation change: none yet.

## References

- `docs/Architectural Reviews/AR-0019-native-math-vocabulary-and-foreign-type-boundary.md`
- `docs/Architectural Reviews/AR-0024-renderer-failure-observation-and-diagnostic-boundary.md`
- `docs/Architectural Reviews/AR-0026-non-euclidean-spatial-charts-and-authored-angular-topology.md`
- `docs/Architectural Reviews/AR-0028-coordinate-frame-handedness-and-directional-conformance.md`
- `docs/ADR/ADR-0014-tokimu-owned-semantic-operations-over-admitted-mechanical-values.md`
- `docs/Plans/Native-Math/Studies/ar-0019-option-a-glam-current-release-update.md`
- `docs/Tokimu Software Design Document.md`
- `crates/tokimu-core/src/math.rs`
- `crates/tokimu-render/src/camera.rs`
