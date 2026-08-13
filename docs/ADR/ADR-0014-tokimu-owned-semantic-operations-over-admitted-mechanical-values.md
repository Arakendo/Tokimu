# ADR-0014: Tokimu-Owned Semantic Operations Over Admitted Mechanical Values

## Status

Accepted — 2026-08-13, with ADR-0005 browser-evidence substitution

AR-0029's earlier No Change disposition was reopened after the production
`glam` 0.33.3 admission attempt turned the retained Alternative-A constructor
vocabulary into strict-Clippy compatibility debt. Maintainer review selected
Narrow B rather than suppressing those deprecations or exposing
`glam::camera` publicly.

## Context

ADR-0003 distinguishes Tokimu-owned meaning from replaceable implementation,
while ADR-0010 permits audited, pinned third-party source to execute in Ring 0.
AR-0019 then separated three ownership questions that had previously collapsed
into the statement that Tokimu uses `glam`:

```text
Who owns the semantics?
Who owns the public vocabulary?
Who owns the executing implementation?
```

Tokimu currently admits five audited `glam` value types as public ordinary-math
vocabulary. The provider also supplies their numerical implementation. That
does not establish that every operation reachable through those types is
provider-owned meaning.

The `glam` 0.29.3 to 0.33.3 update exposed this distinction. Tokimu callers use
three constructor families for meaning already constrained by the SDD and
AR-0024:

- a right-handed, Y-up look-at view with `-Z` forward;
- a right-handed perspective projection with GL-style `[-1, 1]` clip depth;
  and
- a right-handed orthographic projection with the same clip-depth convention.

The provider reorganized those constructors under a new `glam::camera`
vocabulary. Following that organization directly would move 86 dated caller
sites and expand foreign public vocabulary even though Tokimu's meaning did
not change.

The Option B corpus compared two responses:

- **Narrow B** retains admitted provider value types but places a
  provider-neutral Tokimu operation over demonstrated semantic construction;
  and
- **Full B** wraps the five ordinary value types and delegates their mechanics
  privately.

Narrow B absorbed the observed constructor update with zero post-adoption
caller changes and three private adapter changes. Full B provided no additional
insulation for that shock while adding broader migration, documentation,
compatibility, semantic-drift, and performance costs. AR-0026 and AR-0028 also
show that chart identity, qualified location, transition intent, source
embedding, and orientation meaning belong above raw numerical mechanics.

The architectural question is therefore broader than three camera functions
but narrower than owning all math:

> When may Tokimu own a provider-neutral semantic operation over already
> admitted mechanical values without adopting the provider's semantic
> vocabulary or claiming ownership of ordinary mathematics?

## Decision

Tokimu owns a semantic operation when independent evidence establishes both:

1. **Tokimu or an identified Tokimu domain owns the meaning**; and
2. **direct mechanical-provider exposure causes demonstrated semantic
   ambiguity, update shock, or ownership leakage**.

When those conditions and the admission gates below are satisfied, Tokimu may
place a bounded provider-neutral semantic seam over already admitted mechanical
values:

```text
caller
    -> Tokimu-owned semantic operation
    -> private provider adapter
    -> admitted numerical mechanics
    -> admitted value carrier
```

Tokimu owns the semantic operation's name, inputs, units, handedness or frame
assumptions, validation, bounded failure categories, required determinism, and
native/WASM conformance. The selected provider may continue to own ordinary
value representation and numerical mechanics.

This decision does not make all operations performed by Tokimu code into
Tokimu-owned semantics. It does not authorize wrappers solely to hide a
dependency, imitate a provider API, or make foreign vocabulary look local.

### Initial application

The first eligible seam is limited to the three AR-0029 construction families:

```text
view_look_at_rh
projection_perspective_rh_gl
projection_orthographic_rh_gl
```

The operation names above are descriptive rather than final API spelling. The
contract must retain right-handedness, Y-up, `-Z` forward for the view, and
GL-style `[-1, 1]` projection depth. It returns the currently admitted matrix
value carrier and keeps `glam::camera` private.

The initial application is admitted in `tokimu_core::math` as three checked,
provider-neutral constructors. The implementation may use the pinned provider
privately; callers receive the already admitted matrix value carrier.

### Layer ownership

The intended separation is:

```text
Ordinary numerical values and mechanics
    Vec3, Mat4, addition, dot, multiply, inverse
        -> currently the admitted glam provider

Semantic construction or query
    view from eye/target/up
    projection with Tokimu clip convention
    future independently earned operations
        -> Tokimu or the identified domain owner

Domain primitives
    Camera, Bounds, Ray, ChartLocation, Transition
        -> Tokimu or the identified domain owner when admitted

Provider realization
    WGPU clip adaptation and device-specific transport
        -> provider adapter
```

The WGPU `[0, 1]` depth conversion remains private to the WGPU upload boundary.
Camera lifecycle, active selection, viewport, uniform layout, and presentation
submission remain outside the pure construction seam.

## Admission Decision Procedure

Every proposed semantic seam must answer these questions in order:

1. **Is this ordinary mathematics or mechanical manipulation?**
   - If yes, retain provider mechanics and reject the semantic seam.
2. **Who owns the meaning?**
   - If Tokimu ownership or a specific domain owner is not established, resolve
     that ownership first.
3. **Is direct provider exposure causing demonstrated harm?**
   - Without semantic ambiguity, update shock, or ownership leakage, do not
     abstract preemptively.
4. **Can a bounded provider-neutral contract express the meaning?**
   - If not, return to architectural review rather than leaking provider types
     or policy.
5. **Is the seam smaller than the provider surface or churn it contains?**
   - If not, reconsider the decomposition, a higher-level primitive, Full B,
     Option C, or outward movement.
6. **Does corpus evidence satisfy the caller, target, failure, performance,
   security, and documentation gates?**
   - If not, keep the seam incubating.

Passing this procedure makes a seam eligible for explicit admission. It does
not allow implementation without the normal ADR-0005, ADR-0008, ADR-0009,
ADR-0010, and ADR-0011 gates appropriate to its ring and authority.

## Admission Criteria

A new seam may be proposed only when all applicable criteria pass:

- an ADR, the SDD, or retained corpus invariants already establish the semantic
  claim and its owner;
- at least one real caller demonstrates the operation, and a second independent
  caller or documented ADR-0005 substitution supports stability;
- direct provider vocabulary has caused recorded ambiguity, update shock, or
  ownership leakage;
- the contract uses provider-neutral primitives plus only already admitted
  value carriers;
- inputs, outputs, units, frames, orientation, failure behavior, and
  intentionally unspecified behavior are precise;
- native and WASM behavior agrees where those targets are supported;
- malformed, degenerate, non-finite, and overflow behavior is bounded where
  applicable;
- the operation adds no hidden global state, allocation, I/O, threading,
  lifecycle, callback, or provider authority;
- performance evidence passes the applicable ADR-0008 gate under caller-shaped
  workloads;
- the seam is smaller and more coherent than the provider surface or update
  churn it contains;
- normal documentation, examples, migration, rollback, and provider-pin update
  handling are retained; and
- reviewers search existing Tokimu vocabulary and higher-level primitives
  before adding another operation.

An `N/A` response requires a local reason. Repeated `N/A` answers that never
influence admission should be removed during later checklist review rather
than preserved as ceremony.

## Rejection Criteria

Reject or return a proposed seam to incubation when:

- it merely renames a provider function;
- it wraps unqualified addition, multiplication, dot, cross, inverse,
  normalization, or similar ordinary mechanics without owned semantic meaning;
- its owner is a source adapter, application, or other domain rather than
  Tokimu generally;
- no demonstrated harm exists at the direct provider boundary;
- it forwards provider behavior that Tokimu has not specified;
- provider modules, errors, traits, lifecycle values, or mutable state cross
  the seam;
- it mirrors adjacent provider functionality for completeness;
- equivalent Tokimu meaning or a suitable higher-level primitive already
  exists;
- the contract hides an unresolved ownership disagreement; or
- target, failure, performance, provenance, security, or documentation evidence
  is incomplete.

Examples normally rejected by this decision include `tokimu_dot`,
`tokimu_add`, `tokimu_matrix_multiply`, and an unqualified
`tokimu_normalize`. Their mechanics are useful but not distinct Tokimu meaning.

## Domain And Primitive Rule

For a proposed Tokimu primitive such as `Bounds`, `Ray`, `Transform`, or
`ChartLocation`, review must answer independently:

```text
Who owns the primitive's semantics?
Who owns its public vocabulary?
Who owns its executing implementation?
```

A Tokimu-owned primitive may legitimately contain admitted provider-backed
values while owning its own invariants and operations. For example, a future
`Bounds` could own containment and intersection meaning while using admitted
vector carriers. That does not imply Tokimu invented vectors or needs to wrap
all vector mechanics.

If the primitive must prevent callers from seeing the provider value type, the
question has crossed beyond Narrow B into public-vocabulary ownership and must
return to AR-0019.

Repeated semantic helper pressure may instead prove that a domain primitive is
missing. A growing family of ray helpers should trigger review of whether
`Ray` is the owned concept; bounds helpers should trigger the same question for
`Bounds`. Do not preserve a helper collection merely because each helper was
individually defensible.

### Domain ownership is not Tokimu ownership

An operation's semantic richness does not establish Native Tokimu ownership.
The Doom source-to-world embedding, for example, owns source frame,
orientation, and direction correspondence but remains Doom-provider meaning.
A future Tokimu chart transition may be Tokimu spatial meaning if AR-0026 earns
it. Both can use identical matrix mechanics while belonging to different
owners.

Narrow B must not become a semantic junk drawer for source formats,
applications, platform input, rendering providers, or experiments.

## Expansion And Reopening Flags

Return to architectural review before extending a seam when:

- a new operation cannot be explained by the already admitted semantic
  invariant without broadening ownership;
- the seam accumulates adjacent provider functions rather than independently
  required Tokimu semantics;
- camera identity, active selection, viewport, renderer resources, chart
  identity, source embedding, input policy, or lifecycle begins moving into
  ordinary math;
- callers demand provider-free public values, representation, layout, indexing,
  public fields, serialization, reflection, POD, or ABI guarantees;
- conversions multiply across module or ring boundaries;
- provider-specific escape hatches become necessary for correctness or
  performance;
- validation or terminology is duplicated between semantic seams;
- an upstream change still requires public caller changes that the admitted
  seam claimed to contain;
- a provider semantic difference forces Tokimu to choose new numerical policy;
- many narrow seams collectively become harder to understand or maintain than
  an owned primitive or vocabulary; or
- Ring 0 foreign execution or vocabulary ownership is challenged on grounds
  not addressed by the narrow seam.

Operation count is evidence to inspect, not an architectural threshold. A
coherent seven-operation contract may remain narrow; an unrelated second
operation may already indicate the wrong owner.

The number of admitted semantic seams is not by itself evidence for Full B.
Full B requires a distinct recurring problem solved by owning the underlying
value vocabulary. Existing wrapper investment is not such evidence.

## Failure Condition And Periodic Review

Narrow semantic seams have failed as a strategy if the repository accumulates
many overlapping adapters, widespread conversions, special representation
rules, duplicated validation, or provider-specific bypasses. That state must
reopen AR-0019 rather than being described as successful incremental adoption.

Maintainers should periodically review admitted seams and remove or consolidate
items that no longer affect decisions. A seam that becomes ordinary mechanics,
loses its caller, or is superseded by an owned primitive should not remain
permanent merely because it once passed admission.

## Consequences

### Benefits

- Tokimu-owned meaning survives provider API reorganization.
- Mature audited numerical implementation remains reusable.
- Public growth follows demonstrated semantics rather than provider breadth.
- Domain primitives can own invariants without requiring Tokimu-owned ordinary
  vectors and matrices.
- A later provider or owned implementation can preserve the same semantic seam.

### Costs

- Tokimu owns each admitted seam's validation, documentation, compatibility,
  and failure behavior indefinitely.
- Foreign public value coupling remains where provider types are admitted.
- ADR-0010 provenance, source, unsafe/SIMD, target, security, license, and update
  obligations remain unchanged while the provider executes in Ring 0.
- Poor review discipline could recreate Full B one semantic-looking helper at a
  time.
- Provider behavior outside the explicitly owned seam remains provider behavior.

## Non-Decisions

This decision does not:

- admit any camera/projection semantic operation beyond the three named above;
- admit `glam::camera` as public vocabulary;
- hide or replace the five currently admitted provider value types;
- select Full B or Option C;
- create a camera framework, frame type, chart system, ray, bounds, transform,
  portal, or recursive-view API;
- move source-provider or input policy into Tokimu math;
- change WGPU clip adaptation; or
- weaken any ADR-0010 or AR-0015 provenance obligation.

## Acceptance And Flagging Gates

- [x] The Narrow B corpus must demonstrate unchanged callers across both exact
      reviewed provider pins.
- [x] Representative camera, renderer, CAD, Doom, orientation, stereo, GLB
      compile, and chart controls must remain inside the proposed ownership
      boundary.
- [x] Native and Node-WASM default/`simd128` checked contracts must agree.
- [x] ARM64 must compile for both candidate pins.
- [x] Checked malformed and degenerate inputs must produce bounded
      provider-neutral failures without native unwind or WASM trap.
- [x] Authority and selected dependency closure must remain unchanged from the
      audited provider model.
- [x] Caller-shaped performance evidence must retain the measured cost and show
      no unresolved architectural regression.
- [x] WGPU clip conversion, chart identity, source embedding, camera lifecycle,
      viewport, and input policy must remain outside the seam.
- [x] Actual-browser camera behavior is retained on the prior exact provider;
      dual-provider contract/Node-WASM evidence and a fresh 0.33.3 WASM build
      substitute under ADR-0005 because no attachable browser was available.
      Fresh 0.33.3 actual-browser replay remains required follow-up.
- [x] GLB runtime and browser-oriented camera construction evidence is retained
      without expanding the seam. Fresh 0.33.3 browser observation shares the
      same explicit ADR-0005 substitution above.
- [x] Maintainers judged the measured checked-construction cost against a
      named real workload budget rather than only a constructor stress loop.
- [x] Stable names, engine-neutral placement, public documentation, semver,
      migration, rollout, rollback, and provider-pin handling must be reviewed.
- [x] Maintainers explicitly accepted this decision as ADR-0014.

If any open gate exposes a broader camera/view contract, provider-specific
state, target divergence, material performance defect, or need to hide the
underlying value types, return to AR-0029 or AR-0019 rather than weakening this
proposal.

## Verification After Acceptance

Every admitted seam must retain:

- named real and independent callers;
- a provider-neutral semantic contract and reference cases;
- malformed, degenerate, and failure evidence;
- native/WASM and supported actual-browser evidence;
- caller-shaped performance evidence;
- exact provider identities and update-shock replay;
- authority and dependency-closure review;
- public documentation and explicit non-claims; and
- a rollback or outward-movement path.

The initial rollout is bounded to the three operations above. Rollback restores
the previously admitted direct constructors and exact 0.29.3 pin; doing so
would also restore the compatibility debt that triggered this decision.

## References

- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
- `docs/ADR/ADR-0008-native-kernel-ring-performance-and-code-quality.md`
- `docs/ADR/ADR-0009-ring-based-verification-failure-containment-and-recovery.md`
- `docs/ADR/ADR-0010-ring-zero-third-party-source-admission.md`
- `docs/ADR/ADR-0011-ring-based-security-authority-and-trust-boundaries.md`
- `docs/Architectural Reviews/AR-0019-native-math-vocabulary-and-foreign-type-boundary.md`
- `docs/Architectural Reviews/AR-0029-camera-view-and-projection-construction-ownership.md`
- `docs/Architectural Reviews/AR-0026-non-euclidean-spatial-charts-and-authored-angular-topology.md`
- `docs/Architectural Reviews/AR-0028-coordinate-frame-handedness-and-directional-conformance.md`
- `docs/Plans/Native-Math/Studies/ar-0019-option-b-provider-backed-vocabulary-and-semantic-seams.md`
- `corpus/lib/tokimu-math-study/results/2026-08-12-option-b-decision-matrix.md`
- `corpus/lib/tokimu-math-study/results/2026-08-12-option-b-spatial-cross-review.md`
- `docs/Tokimu Software Design Document.md`
