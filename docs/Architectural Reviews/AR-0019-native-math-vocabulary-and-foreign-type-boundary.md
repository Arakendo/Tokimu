# AR-0019: Native Math Vocabulary And Foreign-Type Boundary

| Field | Value |
| --- | --- |
| Status | Retain A; C0/C1 remains incubating corpus evidence |
| Opened | 2026-08-07 |
| Last reviewed | 2026-08-12 |
| Scope | Native Ring / third-party public vocabulary / cross-cutting |
| Trigger | ADR-0010 audit retained `glam` as the current Ring 0 math implementation while exposing its types through Tokimu's public API |
| Related ADRs | ADR-0003, ADR-0005, ADR-0008, ADR-0009, ADR-0010, ADR-0011 |
| Related evidence | `docs/Dependency Audits/Ring 0/glam-d36e7eeff05338c56c4aa8d59fc2615e7963b1b7.md`; AR-0015; AR-0026; `crates/tokimu-core/src/math.rs`; 3D corpus consumers |
| Related plans | `docs/Plans/Native-Math/native-math-vocabulary-foreign-type-case-study.md`; `docs/Plans/Native-Math/Studies/ar-0019-option-c-owned-math-and-bulk-compute.md`; `docs/Plans/Native-Math/Studies/ar-0019-option-a-glam-current-release-update.md` |
| Admission exception | None |

## Architectural Question

Should Tokimu replace its public re-export of `glam` math types with a
Tokimu-owned math vocabulary while retaining `glam` as the current private
implementation where that remains beneficial—and what repeatable evidence
should govern equivalent future foreign-type findings in the Native Ring?

## Context

ADR-0003 identifies lightweight spatial and mathematical primitives as Native
Tokimu meaning. ADR-0010 separately identifies a public foreign type as an
admission cost: using an implementation privately is different from making its
names, representation, and upstream compatibility policy part of Tokimu's
stable vocabulary.

The current implementation intentionally re-exports `glam::{Mat4, Quat, Vec2,
Vec3, Vec4}` from `tokimu_core::math`. The `glam` dependency is source-pinned,
locally selected, feature-constrained, and audited. That establishes current
source provenance. It does **not** decide that `glam` is the permanent semantic
owner of Tokimu math.

This review preserves the current provider while examining a narrower boundary:
whether Tokimu should own the public type names and contracts independently of
the implementation. It is a case study for future Native Ring dependencies
whose foreign types become public vocabulary; it is not authorization to copy
an upstream library or redesign math APIs speculatively.

## Trigger And Evidence

- **Current public surface:** `crates/tokimu-core/src/math.rs` has one direct
  foreign re-export: `Mat4`, `Quat`, `Vec2`, `Vec3`, and `Vec4`.
- **Real consumers:** `tokimu-render::Camera`, the `tokimu` facade, and current
  3D, CAD, GLB, shader, hole-punch, FPS, stereo, and audio-visualizer corpus
  consumers use this vocabulary. The re-export is therefore an existing
  compatibility commitment, not unused convenience.
- **Current provider evidence:** the retained `glam` audit records the exact
  pinned revision, selected `std` feature, no selected transitive dependency,
  native/WASM validation, unsafe SIMD surface, and compiler-warning reopening
  trigger.
- **Ownership pressure:** ADR-0003 calls math primitives Native meaning, while
  ADR-0010 warns that foreign public types create upstream representation,
  serialization, FFI, WASM, authoring, and migration cost.
- **Missing evidence:** no Tokimu-owned wrapper or narrow implementation has
  been measured against actual callers. There is no comparative data for API
  ergonomics, conversions, ABI/layout, SIMD lowering, binary size, compile
  time, allocation, native/WASM behavior, or migration cost.

## Ownership Analysis

Tokimu should own the semantic contracts it presents: coordinate and transform
meaning, operation behavior promised to callers, diagnostics and validation at
external boundaries, and any stable type names it chooses to expose.

`glam` currently owns the implementation and public representation of those
values. It must not become the owner of Tokimu world state, scene authority,
serialization policy, renderer state, or application meaning.

A Tokimu-owned vocabulary would not automatically justify a Tokimu-owned math
implementation. A wrapper can make the public contract Tokimu-owned while a
pinned provider continues to supply mechanics. Conversely, copying only five
named types without a bounded operation inventory could silently create a
second, incomplete math library with more maintenance risk than the original
dependency.

## Emerging Implementation-Ownership Hypothesis

Maintainer discussion has identified a stronger, still non-binding concern:
if Ring 0 defines Tokimu's irreducible trusted meaning, foreign code executing
inside that ring can leave an ambiguity between semantic ownership and
implementation responsibility. This is an architectural smell to test, not a
conclusion that audited dependencies are inherently unsafe or unacceptable.

The models under consideration are:

| Model | Rule for foreign implementation in Ring 0 |
| --- | --- |
| 1 — current ADR-0010 model | Allow it when source is pinned, inspectable, audited, and bounded. |
| 2 — ownership-default model | Prefer Tokimu implementation; require exception-level evidence that the foreign implementation materially reduces risk without compromising semantic ownership. |
| 3 — exclusive implementation model | Ring 0 executes only Tokimu-owned implementation; third-party code is an oracle, reference, or Outer Ring provider. |

This review does not change ADR-0010's current Model-1 policy. The math study
is useful precisely because it can test the practical tradeoff: Alternative C
may support stronger implementation ownership if it remains correct,
performant, portable, and maintainable under real pressure; if it grows beyond
those bounds, it may demonstrate that a tightly controlled foreign provider is
the lower-risk choice. `glam` therefore remains both the present provider and
a correctness/performance comparison oracle during this experiment.

The clearest form of Model 3 is a further hypothesis: **Ring 0 executes only
Tokimu-owned code and contains no provider boundary.** Providers, third-party
libraries, platform mechanisms, and other replaceable implementations begin
outside Ring 0 behind explicit contracts. Under that model, a Ring 0 defect,
behavior, performance decision, or update is unambiguously Tokimu's direct
responsibility rather than a property inherited from a provider.

The concern is **trust-boundary ambiguity**, not a claim that third-party code
is categorically unsafe. A conventional kernel may execute foreign code. The
stronger Tokimu hypothesis is different: Ring 0 is the smallest body of
meaning and execution that Tokimu itself guarantees. An audited provider can
make its evolution tightly controlled, but it cannot remove the split between
Tokimu's semantic responsibility and upstream implementation responsibility.
Self-implemented Ring 0 would collapse semantic, implementation, failure,
performance, evolution, and security ownership into Tokimu.

If evidence supports Model 3, ADR-0010 would become a transition and
remediation policy rather than the desired steady state: a foreign Ring 0
implementation may remain while justified, but each occurrence stays visible,
audited, and under pressure toward Tokimu implementation, outward movement,
or removal. The eventual supply-chain boundary would exclude foreign runtime
source, provider boundaries, dependency build scripts, proc macros, and
network resolution from Ring 0. Mature third-party systems would still remain
valuable as pinned, external correctness, behavior, and performance oracles.

This cleanliness is not free. Direct implementation transfers correctness,
performance, portability, security, testing, and maintenance responsibility to
Tokimu. Its intended pressure is therefore useful: if a concept is too
expensive to own indefinitely, that is evidence to reconsider whether it is
truly irreducible Native meaning or should instead remain an Outer Ring
capability. The hypothesis is viable only if Ring 0 remains deliberately small
and each admitted implementation earns its cost through evidence. Tokimu
authorship is not itself a safety property: replacing a mature provider with a
smaller but numerically incorrect implementation would improve provenance while
making the engine worse.

## Dependency Direction

```text
Current:

Tokimu public callers
    |
    v
tokimu_core::math re-exports glam types
    |
    v
pinned Ring 0 glam implementation

Study target:

Tokimu public callers
    |
    v
Tokimu-owned math contracts and type names
    |
    +--> pinned glam implementation adapter (measured candidate)
    |
    +--> narrow Tokimu implementation (only if earned)
```

The study must preserve one-way dependency direction. No renderer, platform,
corpus, or provider object may enter the math vocabulary. The candidate public
types must not make a target-specific SIMD layout, GPU object, or foreign
library handle part of a Tokimu guarantee without explicit evidence.

## Alternatives Considered

### Alternative A: Retain The Current `glam` Re-Exports

- Benefits: no migration, conversion, representation, or new-maintenance cost;
  current source provenance and caller pressure are already documented.
- Costs: Tokimu's public vocabulary remains coupled to `glam` names and its
  upstream API and representation evolution.
- Failure mode: a later provider replacement becomes an ecosystem-wide caller
  migration rather than an internal implementation change.

### Alternative B: Introduce A Tokimu-Owned Public Vocabulary Over `glam`

- Benefits: separates Tokimu semantic names and contracts from the current
  provider; preserves a potential future implementation seam.
- Costs: wrappers can add conversion, layout, ABI, trait, serialization, and
  ergonomics friction; an `inner: glam::Vec3` field may still leak provider
  assumptions through methods or representations.
- Failure mode: a nominal wrapper becomes a constant impedance mismatch or
  duplicates `glam`'s API without reducing coupling.

### Alternative C: Implement Only The Measured Tokimu Math Subset

- Benefits: Tokimu owns both vocabulary and mechanics; removes the retained
  foreign unsafe and future-toolchain surface if the provider can be removed.
- Costs: correctness, SIMD, portability, and maintenance obligations move to
  Tokimu; the apparently small five-type surface can require many operations,
  traits, tests, and target-specific details.
- Failure mode: an incomplete or slower parallel math implementation grows by
  reacting to callers instead of a bounded, measured contract.

### Alternative D: Copy Or Fork Arbitrary `glam` Source Immediately

- Benefits: apparent immediate control of source and representation.
- Costs: imports a broad source and generated-code maintenance surface without
  proving that Tokimu needs it; upstream fixes and audits become Tokimu work.
- Failure mode: “five types” expands into a partial unreviewed fork, including
  swizzle generation and unsafe implementation details, without a deliberate
  compatibility or performance decision.

## Findings

- The current `glam` audit supports retention as the current provider; it does
  not settle permanent semantic ownership of Tokimu math.
- The existing public re-export means any replacement is a compatibility and
  performance migration, not a local cleanup.
- A Tokimu-owned public vocabulary is architecturally plausible because math
  primitives are Native meaning under ADR-0003, but no evidence yet establishes
  that its migration benefit exceeds its cost.
- The first experiment must be bounded by real caller operations and must
  compare a provider-backed candidate with the direct-re-export baseline before
  a copied or forked implementation is considered.
- The initial provider-backed candidate proves a private implementation seam
  for the currently pressured vector and transform subset, but it also exposes
  an ergonomic `w_axis` migration difference and leaves real-caller conversion
  cost unmeasured. It is therefore conditionally viable evidence, not a
  recommendation to migrate.
- The initial bounded-fork slice proves that provenance can be retained, but
  its maintenance obligations begin immediately and no current evidence shows
  an advantage over the original owned candidate. Expand it only for a named,
  measured reason.
- This review's reusable method is: distinguish implementation admission from
  public-vocabulary admission; inventory real callers; define a narrow contract;
  measure compatibility, performance, targets, and maintenance; then choose
  retain, wrap, replace, or reject explicitly.

## Disposition

**Retain A; C incubation continues.** Retain the audited `glam` re-export as
Tokimu's one stable production vocabulary. C0/C1 remains a focused, reversible
corpus alternative, not a migration track. No ADR revision, source fork,
public type change, or provider replacement is authorized by this record.

## Consequences

- New public `glam` re-exports remain prohibited unless this record and the
  dependency audit are updated with equivalent caller and migration evidence.
- A candidate must begin outside the stable `tokimu_core::math` surface, such
  as a corpus study or explicitly experimental crate, so failure does not
  create a second permanent vocabulary.
- Any representation claim (`repr`, alignment, POD/FFI, serialization,
  bytemuck compatibility, SIMD behavior) requires direct native and WASM
  evidence; familiar field layouts are not sufficient proof.
- The final method may apply to future foreign Native types, but each case must
  still prove its own ownership, caller, target, and performance facts.

## Required Follow-Up

- [x] Create a focused corpus study with only the currently public concepts:
      `Vec2`, `Vec3`, `Vec4`, `Quat`, and `Mat4`.
- [x] Inventory the operations, traits, conversions, layouts, and error or
      validation behavior that real renderer and corpus callers require.
- [x] Implement provider-backed and owned bounded candidates without changing
      `tokimu_core::math` or adding a second stable public API.
- [x] Compare the candidate with direct `glam` use for correctness, caller
      migration, native/WASM behavior, allocation, binary size, compile time,
      and measured hot paths.
- [x] Record unsafe, SIMD, ABI/layout, serialization, reflection, and
      authoring-frontend consequences separately from API-name ownership.
- [x] Decide retain, wrap, replace, or reject from retained evidence; create or
      revise an ADR only if the public ownership boundary changes.
- [ ] After `docs/Plans/DOOM/DOOM WAD Checklist.md` is complete, rescan its
      resulting object, transform, animation, scene-import, and rendering
      callers; revise the operation inventory and rerun affected candidates
      before AR-0019 receives a final disposition.

## Deferred DOOM Pressure Revisit

The current study is intentionally based on the callers present on 2026-08-07.
The DOOM WAD plan is expected to introduce or materially exercise object and
scene pressure that can change which math operations, representations, and
boundary conversions are actually important. Current results remain valid
evidence for the present caller set, but they are not sufficient to select a
permanent vocabulary boundary until that plan completes.

Completion of the DOOM WAD plan requires a new retained source scan and a
review of whether its object lifecycle, transform hierarchy, animation,
collision, rendering, or imported-data paths add pressure not represented by
the frozen 0.1 inventory. New pressure must be named in the inventory and
shared conformance suite rather than added opportunistically to a candidate.

## Deferred Non-Euclidean Spatial Pressure

AR-0026 introduces long-horizon pressure that the current five-type inventory
does not represent. Locally Euclidean charts, angular deficit/excess junctions,
and explicit transition maps may need typed local coordinates, chart/region
identity, transition composition, transformed directions or queries, and
validation that cannot be expressed honestly as one global `Mat4` plus
unqualified `Vec3` values.

This does not prove that those concepts belong in `tokimu_core::math`, that the
current `glam` provider is unsuitable, or that Tokimu should invent manifold
math now. It does prove that AR-0019 must not select a permanent public math
vocabulary solely from today's globally Euclidean renderer callers. A future
AR-0026 corpus may reveal that some required values are spatial semantic types
above ordinary vectors and matrices rather than additional math primitives.

AR-0019 therefore retains ordinary vector/matrix mechanics as one separable
question and chart/topology meaning as another. Any later selection must test
whether the chosen ownership boundary can support strongly distinguished local
coordinates and transitions without leaking a foreign provider's type system
into the spatial semantic contract.

## Reopening Triggers

- A second foreign Native dependency exposes a public type or trait and needs
  the same vocabulary-admission analysis.
- A `glam` update, warning escalation, advisory, target regression, or public
  representation change increases the current provider's cost.
- A real caller requires an operation or representation outside the five
  retained types.
- The DOOM WAD plan completes, because its object and scene workload may alter
  the real caller pressure that this review uses as its decision basis.
- A bounded experiment demonstrates a compatible Tokimu-owned contract with
  acceptable measured costs—or demonstrates that the seam is not worthwhile.
- A future FFI, serialization, reflection, or TypeScript-facing boundary makes
  `glam` representation part of an externally visible contract.
- AR-0026 produces an executable chart/transition corpus whose local-coordinate
  or transition-composition needs cannot be represented without changing the
  current five-type public vocabulary or its ownership boundary.

## Review History

### Cycle 1 -- 2026-08-07

- Status entering review: Proposed.
- New evidence: the ADR-0010 audit pinned and retained `glam` as the current
  Ring 0 provider; source scan confirmed five direct public math re-exports and
  multiple real renderer and corpus consumers; maintainers identified the
  distinction between provider admission and semantic vocabulary ownership.
- Participants or reviewers: project maintainer, Monday architectural review,
  and Codex implementation review.
- Findings: source provenance justifies current retention but does not answer
  whether public foreign vocabulary is the desired permanent boundary.
- Disposition: incubating against a bounded provider-backed vocabulary study.
- Resulting ADR or documentation change: no ADR change; this record supplies
  the case-study method and preserves current provider evidence.

### Cycle 2 -- 2026-08-07

- Status entering review: Incubating.
- New evidence: corpus-local Alternative B defines all five candidate names
  without a foreign public signature; shared vector and transform conformance,
  native/WASM compilation, layout observations, a checksum workload, and a
  zero-allocation workload observation pass.
- Findings: the seam is technically viable for the tested subset, but it is
  not transparent—`Mat4::w_axis` changes shape, `Vec2`/`Quat` are intentionally
  incomplete, provider representation assumptions remain internal, and no real
  caller migration or statistically useful performance result exists.
- Disposition: retain Incubating status; continue evidence gathering and defer
  a final choice until migration evidence and the DOOM pressure revisit exist.
- Resulting ADR or documentation change: no ADR change; retained
  `alternative-b-provider-backed/initial-findings.md`.

### Cycle 3 -- 2026-08-07

- Status entering review: Incubating.
- New evidence: Alternative C now has an original scalar/vector/matrix subset,
  including bounded stack-only inversion and shared workload/allocation cases;
  Alternative D has a provenance-preserving scalar `Vec3` slice with a copied-
  source manifest and explicit update policy. Representative B/C renderer
  fixtures make their provider upload conversions visible.
- Findings: C demonstrates that the currently exercised mechanics can be owned
  without a provider dependency or allocation in the bounded workload, but it
  has not proved a full API, real migration cost, or selected singular-matrix
  contract. D's provenance can be retained, but its maintenance burden starts
  immediately and no evidence establishes an advantage over C.
- Disposition: retain Incubating status; continue only with real-caller and
  target-specific evidence. D remains paused absent a measured C deficit or
  concrete upstream-compatibility need.
- Resulting ADR or documentation change: no ADR change; retained the interim
  comparison, C implementation evidence, and D copy manifest.

### Cycle 4 -- 2026-08-07

- Status entering review: Incubating.
- New evidence: maintainer and Monday architectural discussion distinguished
  semantic ownership from direct implementation ownership. It proposed three
  models: the current audited-foreign-provider policy, a Tokimu-implementation
  default with evidence-based exceptions, and an exclusively Tokimu-implemented
  Ring 0. The B/C copied `hello-3d-mono` cases also now provide native compile
  evidence. The platform's moved-closure error was repaired; the shared WASM
  app check now shows the same native-window application-shape limitation in A,
  B, and C, rather than candidate math.
- Findings: implementation and semantic ownership coinciding could simplify
  the trust model, but treating self-authorship as a sufficient security or
  quality property would transfer numerical correctness, SIMD, portability,
  optimization, and maintenance risk to Tokimu without proof of a net benefit.
  The stronger no-provider Ring 0 model makes that responsibility boundary
  especially clear, but remains an open hypothesis to test through this and
  later case studies, including post-DOOM caller pressure.
- Disposition: retain ADR-0010 unchanged and retain this review as Incubating.
  Do not promote exclusive Tokimu implementation to an ADR rule until retained
  evidence compares its risk and maintenance cost with the audited-provider
  alternative.
- Resulting ADR or documentation change: no ADR change; this record now
  preserves the hypothesis and its evaluation models.

### Cycle 5 -- 2026-08-07

- Status entering review: Incubating.
- New evidence: the bounded `hello-glb` transform fixture now decodes the
  pinned Khronos `Box.glb` asset through the existing corpus decoder and runs
  the model and floor paths against its actual positions and normals. A, B, and
  C agree within the study's retained floating-point tolerance. The study's
  local lockfile evidence was refreshed offline and the locked study suite
  passes.
- Findings: the provider-backed and original-owned candidates now have
  imported-geometry evidence in addition to synthetic transform inputs. This
  exercises real decoded data, non-uniform scaling, inverse-transpose normal
  transformation, and zero-safe normalization, but it does not establish a
  full `hello-glb` application, loader, renderer, or public API migration.
- Disposition: retain Incubating status. Treat this as a stronger bounded
  caller-operation check, not a replacement for app-level migration, repeated
  target measurements, or the post-DOOM pressure revisit.
- Resulting ADR or documentation change: no ADR change; retained the pinned
  asset comparison and updated the migration protocol and interim comparison.

### Cycle 6 -- 2026-08-07

- Status entering review: Incubating.
- New evidence: maintainer and Monday discussion refined the no-provider
  hypothesis. The stated concern is trust-boundary ambiguity: Tokimu can own
  an invariant while a distinct upstream project remains responsible for the
  implementation that executes it. The discussion also identifies audited
  foreign Ring 0 admission as a possible transitional/remediation policy and
  third-party systems as retained external oracles.
- Findings: exclusive Tokimu implementation would make the final execution
  boundary unusually clear, but provenance alone cannot justify it. Every
  candidate must demonstrate that Tokimu can competently own its deliberately
  small contract across correctness, performance, portability, security, and
  maintenance. If it cannot, the architectural question becomes whether the
  irreducible Ring 0 contract is smaller than the proposed implementation, or
  whether the difficult mechanism belongs outside Ring 0.
- Disposition: retain ADR-0010 and this review's Incubating status unchanged.
  Continue treating Model 3 as a demanding hypothesis, with `glam` retained as
  a differential correctness/performance oracle while the math study gathers
  evidence.
- Resulting ADR or documentation change: no ADR change; retained the refined
  trust-boundary and transitional-policy rationale in this review.

### Cycle 7 -- 2026-08-07

- Status entering review: Incubating.
- New evidence: the source inventory's CAD `Vec4` pressure is now exercised by
  a corpus-local port of `hello-cad`'s cursor-to-world ray path. It admits only
  homogeneous `Mat4 * Vec4`, perspective division, and `Vec3::length_squared`
  for zero-length rejection; A, B, and C agree within the retained floating-
  point tolerance.
- Findings: the owned C candidate covers a further real caller path without a
  provider. B covers the same path but makes a concrete API-shape migration
  visible: `Vec4::w()` replaces the baseline public field. This remains a
  migration cost observation, not a decision that either field shape belongs
  in a stable Tokimu contract. D remains unable to participate because its
  deliberately paused slice has no matrix type.
- Disposition: retain Incubating status. The extra caller path strengthens the
  bounded study but does not replace app-level migration, target-specific
  measurements, numerical edge-case evidence, or the post-DOOM revisit.
- Resulting ADR or documentation change: no ADR change; retain the narrowed
  operation-manifest extension and CAD comparison fixture.

### Cycle 8 -- 2026-08-07

- Status entering review: Incubating.
- New evidence: a bounded `hello-hole-punch` node-resolution fixture exercises
  column-major glTF node input, an animation translation override of the final
  matrix column, and parent-child composition. A/B/C agree on a two-node path;
  the independent comparison also supplies a decoded node from pinned Khronos
  `Box.glb` input.
- Findings: the previously identified writable-column difference is now real
  caller evidence. B's `set_w_axis(...)` makes the mutation explicit, while C
  owns the same mechanism directly. Neither result selects a stable matrix API;
  it only makes the migration and ownership cost observable. D remains outside
  the comparison because it has no admitted matrix type.
- Disposition: retain Incubating status. The bounded evidence does not replace
  scene traversal, animation scheduling, renderer integration, target-specific
  measurements, or the post-DOOM caller-pressure review.
- Resulting ADR or documentation change: no ADR change; retain the bounded
  animated-node fixture and evidence record.

### Cycle 9 -- 2026-08-07

- Status entering review: Incubating.
- New evidence: following the CAD and animated-node additions, the complete
  study library passes fresh native and `wasm32-unknown-unknown` compile
  checks using the locked offline dependency graph.
- Findings: the current B/C candidate source remains compile-portable for the
  target under study. A successful build is not behavioral or performance
  evidence: no browser execution, WASM allocation observation, binary-size
  comparison, or target-identified timing has been retained.
- Disposition: retain Incubating status and preserve the distinction between
  build portability and runtime conformance.
- Resulting ADR or documentation change: no ADR change; updated retained
  target-evidence wording.

### Cycle 10 -- 2026-08-07

- Status entering review: Incubating.
- New evidence: a reproducible source-surface observation records the current
  A/B/C/D candidate files and pinned `glam/src` tree. The B candidate is 450
  nonblank lines with 41 direct private-provider references; C is 494 lines
  with no provider reference; D remains a 139-line `Vec3`-only derivation.
- Findings: the present owned subset is compact enough to remain worth
  studying, while the full provider source tree and D's provenance obligations
  make clear that source-line counts cannot settle maintenance, quality, or
  safety. The observation strengthens the need for later binary-size,
  performance, correctness, and target-specific evidence rather than replacing
  those gates.
- Disposition: retain Incubating status. No candidate is selected from source
  size; preserve the observation as one maintenance/provenance input only.
- Resulting ADR or documentation change: no ADR change; retained the source-
  surface observation.

### Cycle 11 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: the case study now has a side-by-side decision matrix that
  separates public-vocabulary independence, implementation independence,
  provenance, bounded caller correctness, target builds, runtime evidence,
  provider-boundary shape, and unresolved maintenance risk for A/B/C/D.
- Findings: C is the leading experiment for the self-implemented Ring 0
  hypothesis because it owns the current mechanics without provider code and
  has cleared multiple bounded caller paths. A remains the only stable,
  production-compatible control. B is a valid comparison and possible
  transition seam, not an assumed destination; D remains paused. These are
  directions for evidence gathering, not a stable selection or an ADR change.
- Disposition: retain Incubating status and stable A behavior. Selection remains
  blocked by post-DOOM caller pressure, repeated target measurements, runtime
  WASM execution, numeric edge evidence, and real migration evidence.
- Resulting ADR or documentation change: no ADR change; retained the
  provisional decision matrix.

### Cycle 12 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: the math study now supplies a reusable foreign-public-type
  review checklist and applies it retrospectively to `glam`. It distinguishes
  ownership, public-boundary cost, supply-chain evidence, correctness and
  performance evidence, and final disposition rather than treating source
  admission as sufficient public-vocabulary justification.
- Findings: the `glam` case has sufficient evidence to remain an audited
  implementation control, but it is only partially complete for a public
  vocabulary decision: full downstream/FFI/serialization migration, repeated
  target measurements, WASM execution, and post-DOOM caller pressure remain
  open. The checklist makes those gaps reusable rather than math-specific.
- Disposition: retain Incubating status. Use the checklist for any future
  Native Ring foreign public-type proposal; it supplements and does not weaken
  ADR-0008 through ADR-0011.
- Resulting ADR or documentation change: no ADR change; retained the reusable
  checklist and its `glam` application.

### Cycle 13 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: shared conformance now compares the A/B/C non-finite output
  masks for degenerate `look_at_rh` inputs and singular matrix inversion.
- Findings: B and C match the observed baseline failure shape for these two
  cases. This is useful differential evidence but does not choose whether a
  future Tokimu contract should reject, diagnose, return a typed failure, or
  retain non-finite values. Such a choice needs real boundary and ADR-0009
  recovery evidence rather than implicit provider compatibility.
- Disposition: retain Incubating status and retain these outcomes as observed
  behavior only.
- Resulting ADR or documentation change: no ADR change; retained the labelled
  degenerate-matrix conformance case.

### Cycle 14 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: a refreshed direct-import scan finds no production or corpus
  caller importing `Vec2` or `Quat` from `tokimu_core::math`. Several corpus
  applications define local 2D vectors, but those are independent application
  concepts rather than evidence for the current public re-export.
- Findings: B's minimal probes remain useful to expose present vocabulary
  coupling; C correctly omits both unpressured types. The absence of an
  in-repository caller does not authorize their removal from the stable public
  API, which requires downstream compatibility and release evidence.
- Disposition: retain Incubating status and do not widen C. Revisit when DOOM,
  another caller, or downstream evidence creates named pressure.
- Resulting ADR or documentation change: no ADR change; retained the
  unpressured-public-type finding and reopening triggers.

### Cycle 15 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: a bounded `hello-fps-web` port exercises direction construction,
  zero-safe normalization, repeated in-place motion, component mutation, and
  distance observation. It adds only `Vec3::distance` and add-assign to the
  reviewed manifest; A/B/C agree within the retained tolerance.
- Findings: C continues to cover real caller pressure with its owned scalar
  implementation. B covers the behavior but converts the baseline mutable
  component assignment into getter-based reconstruction, a concrete wrapper
  migration cost that must remain visible. The fixture is not a full FPS app,
  input, renderer, or browser migration.
- Disposition: retain Incubating status; use this as caller evidence, not a
  stable public field/API decision.
- Resulting ADR or documentation change: no ADR change; retained the FPS
  fixture and narrow operation-manifest extension.

### Cycle 16 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: Monday's review of the reusable foreign-public-type checklist
  confirms its central distinction: an auditable foreign implementation is not
  automatically acceptable Tokimu public vocabulary. The review adds one
  further ownership question—what irreducible semantic value Tokimu would lose
  if the type disappeared entirely.
- Findings: current caller count is not enough to establish Native meaning.
  The `Vec2`/`Quat` result demonstrates the distinction: public availability
  and a source re-export do not yet prove irreducible Tokimu value. The
  checklist remains useful review guidance, not an ADR, until multiple future
  cases demonstrate that its method consistently determines outcomes.
- Disposition: retain Incubating status and retain the checklist as a
  case-study-derived procedure. Do not elevate it into admission policy yet.
- Resulting ADR or documentation change: no ADR change; added the irreducible-
  value question and review-method note to the checklist.

### Cycle 17 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: the shared A/B/C transform runner now performs a checked
  warm-up, collects repeated samples, rotates the A/B/C timing order, and
  reports min/median/max. A nine-sample release run retained matching checksums
  for every invocation.
- Findings: the owned C median is lower in this one process, but the ranges
  overlap and sandbox policy still prevents retaining host CPU/OS metadata.
  The result establishes a less order-biased, repeatable harness; it does not
  establish a performance advantage or change the candidate disposition.
- Disposition: retain Incubating status. Use the result as descriptive workload
  evidence only; require identified-host and target-specific measurements,
  representative caller workloads, and the post-DOOM revisit before using
  timing as a selection input.
- Resulting ADR or documentation change: no ADR change; retained the repeated
  transform observation and documented the runner protocol.

### Cycle 18 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: candidate-isolated release executables each link exactly one
  shared transform workload. On the retained target, A and B are 123,904 bytes
  and C is 124,416 bytes.
- Findings: the tiny 512-byte C difference demonstrates that an original
  implementation is not automatically the smaller linked output, even for the
  study's minimal path. The harness deliberately excludes a renderer,
  application closure, packaging, LTO comparison, and WASM output, so the
  values are descriptive only.
- Disposition: retain Incubating status. Do not use isolated executable size as
  a candidate-selection criterion; collect representative application and
  target outputs only after the caller boundary is better defined.
- Resulting ADR or documentation change: no ADR change; retained the
  candidate-isolated link-output observation.

### Cycle 19 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: A/B/C now share four deterministic non-singular affine inverse
  round trips combining translation, X/Y rotation, non-uniform scale, and
  varied point ranges. B and C match A within `3e-5`; C also returns each input
  point within that stated tolerance.
- Findings: the sweep materially broadens the scalar C inverse evidence beyond
  a translation-only path while keeping its scope honest. It does not establish
  randomized numeric robustness, an exhaustive matrix contract, or a recovery
  choice for singular/non-finite input.
- Disposition: retain Incubating status. Keep the `3e-5` tolerance as
  case-study conformance evidence only; add property/fuzz, target-runtime, and
  caller-pressure evidence before any stable implementation decision.
- Resulting ADR or documentation change: no ADR change; retained the affine
  inverse sweep as a labelled shared conformance case.

### Cycle 20 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: direct source scans of `hello-shader` and
  `hello-audio-visualizer` show only `Mat4::from_translation` with a zero
  `Vec3` for orthographic camera views. Neither caller has inverse,
  composition, vector-four, projection, component-access, or adapter-boundary
  math pressure.
- Findings: a new bounded migration would duplicate already stronger transform
  evidence without testing an additional contract. Retaining negative evidence
  is more useful than inflating the corpus fixture count.
- Disposition: retain Incubating status and no new fixture. Reopen if either
  caller develops a distinct operation or an observable renderer boundary.
- Resulting ADR or documentation change: no ADR change; retained the
  presentation caller-pressure scan.

### Cycle 21 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: the exact shared Alternative C source compiles and passes four
  tests through a separate corpus crate with an empty dependency list. The
  target is intentionally outside the parent workspace so the A/B control's
  `glam` dependency cannot be inherited accidentally.
- Findings: C's present source boundary is provider-free in compilation as
  well as text. This satisfies only the narrow current subset's provenance and
  closure claim; it does not establish that later API growth, SIMD, target
  support, or a stable implementation will retain the same closure.
- Disposition: retain Incubating status. Preserve the isolated target as a
  regression check and require it to remain dependency-free unless a new,
  separately reviewed pressure justifies otherwise.
- Resulting ADR or documentation change: no ADR change; retained the isolated
  C compilation boundary and result.

### Cycle 22 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: the dependency-free isolated C crate successfully builds its
  shared candidate source for `wasm32-unknown-unknown` offline.
- Findings: C's present source has an independently verified native and WASM
  compilation boundary without `glam`. This narrows a closure question only;
  it cannot substitute for WASM execution, behavioral conformance, generated
  size, or target performance evidence.
- Disposition: retain Incubating status. Preserve the isolated WASM build as a
  regression check while leaving the main runtime/measurement blockers open.
- Resulting ADR or documentation change: no ADR change; retained the isolated
  C WASM-build observation.

### Cycle 23 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: independently isolated, fresh release builds took 3.518 s for
  B with its pinned `glam` dependency and 0.322 s for dependency-free C. B
  compiled the provider and surfaced its recorded warning set; C compiled only
  its source boundary.
- Findings: private provider use remains a concrete build-closure and warning-
  surface cost even though B's public signatures conceal it. The deliberately
  small, single-host targets cannot predict workspace, renderer, application,
  incremental, or future candidate build times.
- Disposition: retain Incubating status. Treat the result as a closure
  observation, not a compile-time selection threshold; repeat on an identified
  host with representative targets only when the boundary is better defined.
- Resulting ADR or documentation change: no ADR change; retained the isolated
  B/C build-closure observation.

### Cycle 24 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: Node's WebAssembly engine executes isolated release B and C
  `wasm32-unknown-unknown` probes that perform composed transform/inverse round
  trips. B returns `292.00006103515625`; C returns `292`; both are finite and
  within the study's stated tolerance.
- Findings: B and C now have bounded actual WASM-engine execution rather than
  compilation alone. The probe deliberately excludes A, the shared suite,
  browser and renderer behavior, input/lifecycle integration, and performance
  or allocation evidence.
- Disposition: retain Incubating status. Treat this as a narrow runtime step,
  not satisfaction of the full WASM behavioral gate or a candidate selection.
- Resulting ADR or documentation change: no ADR change; retained the isolated
  B/C WASM-engine execution observation.

### Cycle 25 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: an isolated A control importing the actual stable
  `tokimu_core::math` re-exports builds and executes the same Node WASM probe.
  A and B return `292.00006103515625`; C returns `292`.
- Findings: the bounded WASM runtime comparison now has its real A control,
  rather than an imitation of the provider types. This remains one small
  transform/inverse probe, not execution of the shared conformance suite or a
  browser/render/application boundary.
- Disposition: retain Incubating status. The A/B/C result is differential
  runtime evidence only; full WASM behavioral acceptance remains open.
- Resulting ADR or documentation change: no ADR change; extended the retained
  WASM-engine observation to A/B/C.

### Cycle 26 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: equivalent isolated release WASM checksum modules are 2,782
  bytes for A, 3,011 bytes for B, and 3,953 bytes for C.
- Findings: the owned scalar C mechanics are not automatically the smaller
  target output; C is 1,171 bytes larger than A in this micro-module. The
  result is useful target-specific counterevidence to size assumptions but
  excludes all realistic application, renderer, binding, and packaging costs.
- Disposition: retain Incubating status. Use this only as a bounded WASM size
  input; do not select a candidate from it.
- Resulting ADR or documentation change: no ADR change; retained the isolated
  A/B/C WASM output observation.

### Cycle 27 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: the shared suite now runs 96 fixed-seed, bounded non-singular
  affine transforms through A/B/C inverse round trips. B and C match A within
  `1e-3` for every retained case.
- Findings: C has broader deterministic differential coverage than the prior
  four hand-selected cases, without new dependencies or API surface. The sweep
  intentionally excludes zero/near-zero scales, singular, and non-finite input
  and is not randomized fuzz/property completeness.
- Disposition: retain Incubating status. Preserve the fixed sweep and its
  tolerance as conformance evidence only; keep numerical-edge and recovery
  contract work open.
- Resulting ADR or documentation change: no ADR change; retained the
  deterministic affine differential sweep.

### Cycle 28 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: native layout observations show B exactly matches A for every
  retained type. C's implemented `Vec3`, `Vec4`, and `Mat4` have the same
  observed sizes as A/B, but C `Vec4` and `Mat4` align to 4 bytes rather than
  16 bytes.
- Findings: owned scalar mechanics create a concrete representation boundary
  even where mathematical conformance passes. The current explicit adapter
  reconstruction keeps this visible; a stable C candidate cannot claim direct
  provider-layout, FFI, SIMD, or GPU compatibility without a separate decision
  and evidence.
- Disposition: retain Incubating status. Do not change C's layout merely to
  imitate the provider; first establish whether a named Tokimu boundary
  requires an alignment/representation contract.
- Resulting ADR or documentation change: no ADR change; retained the native
  layout observation and representation blocker.

### Cycle 29 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: Node executes isolated A/B/C WASM layout exports. A and B
  report 16-byte `Vec4`/`Mat4` alignment; C reports 4-byte alignment for both.
- Findings: C's representation boundary is present on both retained native and
  WASM targets. Passing transform conformance and bounded WASM execution do not
  imply direct provider-layout compatibility.
- Disposition: retain Incubating status. Treat cross-target alignment as a
  named representation blocker for any direct FFI/SIMD/GPU claim; do not adopt
  a layout policy before a real Tokimu boundary requires one.
- Resulting ADR or documentation change: no ADR change; retained the isolated
  A/B/C WASM layout observation.

### Cycle 30 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: the current WGPU camera upload path converts
  `projection * view` into a renderer-owned `#[repr(C)]` `[[f32; 4]; 4]`
  `GpuCameraUniform` before `bytemuck` byte upload. It does not upload or cast
  `Mat4` memory directly.
- Findings: C's cross-target alignment difference is a direct-interchange,
  FFI/SIMD, and future-layout constraint, but not an immediate failure of the
  current explicit camera GPU boundary. This validates the existing boundary
  separation without claiming that C has no migration or rendering cost.
- Disposition: retain Incubating status. Keep the explicit scalar-array upload
  boundary visible and reopen the representation question if any path attempts
  direct `Mat4` upload or exposure.
- Resulting ADR or documentation change: no ADR change; retained the renderer
  camera representation-boundary scan.

### Cycle 31 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: the adjacent WGPU instance upload path also contains no `Mat4`
  byte representation. It creates a renderer-local scalar-field uniform from
  `Instance2d` translation, scale, and rotation before byte upload.
- Findings: current camera and instance WGPU paths both preserve a
  renderer-owned representation boundary. C's alignment difference therefore
  does not immediately obstruct current uploads, while direct interchange and
  future exposed-layout questions remain open.
- Disposition: retain Incubating status. Keep the finding scoped to the current
  WGPU paths; re-scan if 3D instance matrices, direct casts, or public layout
  boundaries are introduced.
- Resulting ADR or documentation change: no ADR change; broadened the retained
  renderer representation-boundary scan.

### Cycle 32 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: corpus-local B and C candidate cameras now construct the
  current public `tokimu::Camera`. That facade presently stores provider
  `Mat4` values for `view` and `projection`; B therefore performs two private
  unwraps, while C reconstructs two provider matrices from owned column arrays.
  Exact composed-transform comparisons pass for both candidate handoffs.
- Findings: the existing WGPU scalar-array upload boundary avoids direct
  layout coupling, but the public renderer camera is an independent foreign
  vocabulary seam. A Tokimu-owned math migration would need a deliberate
  renderer-facade migration or an explicitly retained adapter boundary; it is
  not merely an internal upload conversion.
- Disposition: retain Incubating status. Count this as bounded B/C migration
  evidence only; do not change the stable `tokimu::Camera` API or infer a full
  renderer migration from the corpus fixture. D remains blocked by its scoped
  lack of `Mat4`.
- Resulting ADR or documentation change: no ADR change; updated the Slice 6
  protocol, comparison, and decision records with the current camera handoff.

### Cycle 33 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: a bounded source scan confirms that `tokimu-render::Camera`
  exposes public provider `Mat4` `view` and `projection` fields. Eight corpus
  source files directly assign a 3D camera `view`; stereo supplies a distinct
  two-camera writer shape. Other orthographic constructor callers were retained
  as API exposure but not promoted to unproven 3D-math pressure.
- Findings: the B/C handoff is a valid adapter experiment, not a replacement
  for this public surface. If owned math is selected, renderer migration cost
  includes a deliberate decision about `Camera` vocabulary and a recheck of
  these direct writers; leaving provider matrices there is a possible explicit
  Outer-Ring boundary, not an invisible implementation detail.
- Disposition: retain Incubating status. Keep the scan as scope evidence; do
  not modify the stable facade before a selection decision and further caller
  pressure, including the post-DOOM revisit.
- Resulting ADR or documentation change: no ADR change; retained the camera
  public-vocabulary surface scan.

### Cycle 34 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: the corpus-local `hello-3d-stereo` fixture reproduces two
  independent orbit-derived cameras, each with a view and half-width
  projection. B and C agree with A for both composed matrices and preserve a
  distinct left/right result. Their current-renderer handoffs each perform four
  explicit matrix crossings: view and projection for both eyes.
- Findings: the camera public-vocabulary seam scales with camera multiplicity;
  it is not hidden by the single-camera adapter test. The path needs no new
  candidate math operation, so it strengthens migration-cost evidence rather
  than widening the owned subset spec.
- Disposition: retain Incubating status. Treat the result as bounded stereo
  caller evidence only; do not infer multiview renderer support or change the
  stable camera API.
- Resulting ADR or documentation change: no ADR change; recorded the stereo
  fixture in the Slice 6 protocol and decision comparison.

### Cycle 35 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: a corpus-local `hello-asteroids` control reproduces the
  current `Camera::orthographic_2d_with_height` policy, including normal
  world-height bounds and the zero-height aspect fallback. B and C form the
  same current provider-valued camera through explicit private identity and
  projection crossings; both agree with A.
- Findings: orthographic projection is required to reproduce the present
  renderer camera policy, but this one renderer-facing caller does not by
  itself establish universal Native semantic ownership. The evidence supports
  keeping the operation available to the candidate matrix slice while retaining
  the ownership question for selection.
- Disposition: retain Incubating status. Do not elevate a renderer convenience
  into a Ring 0 admission conclusion or modify the stable camera API.
- Resulting ADR or documentation change: no ADR change; recorded the bounded
  orthographic-camera compatibility result in Slice 6 evidence.

### Cycle 36 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: a rotated release-mode native observation constructs 100,000
  complete stereo camera pairs per sample, including B/C current-renderer
  crossings. All A/B/C paths observed zero allocations. The retained medians
  were A 7.323 ms, B 6.663 ms, and C 9.246 ms across nine samples; C was about
  26% above A on this host.
- Findings: C's ownership/provenance benefits do not erase a measured
  performance concern at a realistic bounded camera boundary. B's wrapper
  crossings did not show an adverse result here. Neither finding is portable:
  this is one native host, a small workload, and no WGPU/WASM measurement.
- Disposition: retain Incubating status. Preserve the raw result as ADR-0008
  performance evidence, require target-native/WASM repetition and post-DOOM
  pressure before selection, and make no stable implementation change.
- Resulting ADR or documentation change: no ADR change; retained the stereo
  camera boundary observation and allocation probe.

### Cycle 37 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: isolated A/B/C `wasm32-unknown-unknown` modules now execute a
  100,000-iteration stereo math and matrix-column probe in Node's WASM engine.
  Across nine rotated samples, retained medians were A 9.839 ms, B 9.356 ms,
  and C 9.352 ms. All returned checksum `172113`; raw WASM outputs were A
  11,738, B 11,878, and C 12,843 bytes.
- Findings: the isolated WASM probe does not reproduce C's native
  public-camera slowdown; B and C are approximately 5% below A in this engine
  and scope. C retains the largest raw module. This demonstrates target and
  boundary sensitivity, not a contradiction or a selection result, because
  the WASM probe does not construct `tokimu::Camera` or run WGPU.
- Disposition: retain Incubating status. Keep both performance records, do not
  average them into a claim, and require browser/WGPU plus post-DOOM caller
  evidence before selection.
- Resulting ADR or documentation change: no ADR change; retained the isolated
  WASM stereo-camera math/column observation.

### Cycle 38 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: the complete `tokimu-math-study` crate builds for
  `wasm32-unknown-unknown` after adding the B/C current-`tokimu::Camera`
  handoff, stereo, and orthographic fixtures. Its native suite also passes all
  41 tests.
- Findings: the renderer-facing candidate fixture code is target-compilable;
  there is no compile-time WASM boundary failure introduced by the current
  study work. Compilation neither executes the public camera facade in a WASM
  engine nor proves browser/WGPU behavior or performance.
- Disposition: retain Incubating status. Count this as compile-only
  portability evidence, preserving browser/WGPU execution and post-DOOM
  pressure as selection blockers.
- Resulting ADR or documentation change: no ADR change; refreshed Slice 6
  target-compile evidence.

### Cycle 39 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: a fixed-seed 128-case finite camera/projection differential
  sweep compares `perspective_rh_gl * look_at_rh` matrices for A/B/C at
  `1e-4` tolerance. Inputs retain distinct eye/target values, a nonparallel-up
  construction, positive finite aspect, and `near < far`; B and C pass.
- Findings: the owned scalar candidate now has bounded differential evidence
  for the actual composed camera shape, not only fixed hand-selected views or
  affine inversion. This strengthens finite caller-path correctness without
  choosing the provider's degenerate/non-finite behavior as a Tokimu contract.
- Disposition: retain Incubating status. Keep the sweep's finite-domain
  preconditions visible, preserve separate observed-provider edge records, and
  do not treat bounded differential coverage as exhaustive numeric proof.
- Resulting ADR or documentation change: no ADR change; added the finite
  camera/projection conformance case to the study inventory and comparisons.

### Cycle 40 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: representative migration accounting now records zero stable
  source edits, nine corpus-local modules, and nine explicit current-renderer
  matrix crossings for each B/C bounded path. Both candidates exact-round-trip
  a finite camera matrix at that provider boundary; D remains blocked without
  `Mat4`.
- Findings: B/C crossings are attributable rather than hidden. Their candidate
  signatures remain provider-free, but the existing public `tokimu::Camera`
  is the exposed provider vocabulary seam. Rollback is deletion of the
  corpus-local experiment because no stable caller or crate changed.
- Disposition: retain Incubating status. Close the bounded migration-accounting
  and interop checklist items, while preserving renderer-facade migration as a
  later selection decision rather than silently absorbing it.
- Resulting ADR or documentation change: no ADR change; retained the
  representative migration accounting record.

### Cycle 41 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: B/C API ergonomics and validation ownership are now recorded
  explicitly. B retains private provider mechanics and accessor/reconstruction
  seams; C retains scalar access but owns numerical, recovery, and maintenance
  responsibility. Both avoid copying a stable `w_axis` field contract and
  neither claims full `glam` source compatibility.
- Findings: wrapper ergonomics and validation ownership are architectural
  tradeoffs, not superficial naming differences. B inherits provider behavior;
  C must deliberately choose and maintain any future Tokimu behavior.
- Disposition: retain Incubating status. Close the bounded ergonomics record
  while leaving selection, stable API, and post-DOOM pressure unresolved.
- Resulting ADR or documentation change: no ADR change; retained the candidate
  API ergonomics and validation-ownership comparison.

### Cycle 42 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: the case-study checklist was reconciled against retained
  artifacts. Completed Slice 0–4 boxes now point to actual inventory,
  conformance, isolated-target, layout/source/build, boundary-accounting, and
  interim-finding evidence rather than remaining as stale planning markers.
- Findings: the remaining open items are substantive: full shared-suite WASM
  execution, D's intentionally unearned matrix scope, browser/WGPU and full
  application evidence, post-DOOM caller pressure, maintenance forecasting,
  and final selection. Closing completed evidence boxes does not settle those
  questions or alter the accepted stable boundary.
- Disposition: retain Incubating status. Use the smaller remaining checklist to
  prioritize new evidence; do not close blocked or decision-dependent items by
  inference.
- Resulting ADR or documentation change: no ADR change; retained the checklist
  reconciliation note in the case-study plan.

### Cycle 43 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: a candidate maintenance forecast separates recurring provider
  governance (A/B), Tokimu-owned numerical/target/optimization responsibility
  (C), and the additional provenance/diff/fix burden of derived source (D).
  It names minimum recurring evidence instead of inventing person-hour costs.
- Findings: C's self-implemented Ring 0 hypothesis is maintainable only if the
  admitted core stays small and continued caller pressure justifies its
  numerical and target responsibility. B is mechanically less independent;
  D has no current advantage over C that offsets its lineage burden.
- Disposition: retain Incubating status and formally recommend continued
  incubation pending post-DOOM pressure and the remaining selection evidence.
  This is not a stable ownership decision.
- Resulting ADR or documentation change: no ADR change; retained the candidate
  maintenance forecast and interim continue-incubation recommendation.

### Cycle 44 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: a current-source public-boundary inventory distinguishes the
  stable `glam` re-export and public renderer `Camera` fields from renderer
  internals. WGPU converts composed matrices into renderer-local scalar-array
  uniforms before byte upload; current persistence, native FFI, and TypeScript
  authoring surfaces do not expose a math contract. The study remains
  compile-tested for WASM and isolated math runs in Node, not browser/WGPU.
- Findings: replacement pressure is real at the stable math and `Camera`
  vocabulary seams, but current GPU upload alignment is not a direct blocker.
  Absence of a serialization, FFI, publication, or authoring boundary is not
  compatibility evidence; it is an unclaimed future decision.
- Disposition: retain Incubating status. Close the reusable-method acceptance
  criterion, retain the public-boundary scan, and preserve browser/WGPU,
  downstream-publication, and post-DOOM pressure as selection blockers.
- Resulting ADR or documentation change: no ADR change; retained the public
  math boundary-consequences scan and updated the checklist retrospective.

### Cycle 45 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: Cargo compiles the complete A/B/C study test target for
  `wasm32-unknown-unknown`, then cannot execute its ordinary Rust test harness
  on Windows (error 193). The installed `wasm-bindgen-test-runner` finds no
  registered tests because the suite has no `wasm_bindgen_test` harness.
- Findings: the full shared suite remains compile-only WASM evidence. The
  existing isolated Node probes are useful bounded execution observations but
  cannot be substituted for shared conformance.
- Disposition: retain Incubating status and leave the shared native-and-WASM
  conformance criterion open. A future pinned harness must execute shared
  assertions without creating a reduced duplicate success probe.
- Resulting ADR or documentation change: no ADR change; retained the shared
  WASM conformance-harness observation.

### Cycle 46 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: `wasm-bindgen-test` 0.3.76 is now locked to the workspace's
  `wasm-bindgen` 0.2.126 runner schema. The same twelve `conformance.rs`
  assertion bodies run natively and through `wasm-bindgen-test-runner` on Node
  v22.21.0 for `wasm32-unknown-unknown`; all pass.
- Findings: A/B/C shared finite, observed-degenerate, affine, and camera
  differential coverage now has actual WASM execution evidence rather than
  compile-only evidence. D participates only in its bounded Vec3 case and
  remains unable to cover matrix callers. This does not establish browser/WGPU
  behavior or a stable numerical contract.
- Disposition: retain Incubating status. Close the current A/B/C
  native-and-WASM shared-conformance criteria; preserve browser/WGPU,
  full-application, post-DOOM, and final-selection evidence as open.
- Resulting ADR or documentation change: no ADR change; retained the resolved
  WASM conformance-harness observation and pinned the corpus-only test harness.

### Cycle 47 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: D now has a line-range provenance ledger covering every line
  as either Tokimu corpus scaffolding or a named pinned `glam` source anchor.
  Its bounded report records native/WASM Vec3 conformance, observed 12-byte /
  4-byte native layout, absence of unsafe/SIMD/generated source, unavailable
  matrix workload evidence, and the recurring update-review obligation.
- Findings: D's source lineage is auditable at its deliberately small Vec3
  scope, but it provides no demonstrated correctness, performance, migration,
  or ownership advantage over C for current matrix-driven callers. Its small
  source file is not a low maintenance cost once every expansion needs source
  selection, attribution, diff, and fix-incorporation review.
- Disposition: reject D expansion for now; retain it as a bounded provenance
  case study. Do not mark matrix/application criteria satisfied by its Vec3
  evidence, and reopen only for a measured C deficit or concrete
  upstream-compatibility requirement.
- Resulting ADR or documentation change: no ADR change; retained the D line
  ledger and bounded status report, and updated its layout observation.

### Cycle 48 -- 2026-08-08

- Status entering review: Incubating.
- New evidence: the study checklist is now reconciled to the evidence-phase
  scope. B/C completed equivalent bounded matrix-caller ports and D's
  deliberately ineligible Vec3-only scope is explicit rather than a missing
  result. A phase-close report traces the interim A/B/C/D dispositions to
  retained evidence and tradeoffs.
- Findings: the study has completed its current corpus evidence mission without
  silently turning an incomplete selection into a stable migration. Remaining
  items are selection blockers, not unperformed variants of completed work:
  post-DOOM pressure, browser/WGPU/application evidence, future public-boundary
  choices, numerical contract work, and Vec2/Quat pressure.
- Disposition: close the evidence phase and retain AR-0019 as Incubating.
  Preserve A as the stable control; keep B/C corpus-local; retain D as a
  provenance specimen only. A later ADR-backed selection phase must address
  the named blockers.
- Resulting ADR or documentation change: no ADR change; retained the
  evidence-phase close report and scope-aware checklist reconciliation.

### Cycle 49 -- 2026-08-08

- Status entering review: Incubating.
- New argument: Microsoft's analysis of the March 2026 Axios npm compromise
  describes a trusted package release whose application source was unchanged,
  but whose manifest added a malicious runtime dependency with an automatic
  install-time hook. The chain could execute on developer and CI/CD machines
  during normal dependency installation or update, and package metadata
  differed from the project's normal trusted publishing trail.
- Relevance to this review: source review alone is insufficient provenance
  evidence for an admitted provider. The full resolved closure, exact version
  and lock state, manifest/build-script/proc-macro behavior, update path, and
  publication provenance must remain separately auditable. This supports
  ADR-0010's pinned-source and closure requirements, plus ADR-0011's authority
  and execution-boundary scrutiny. It also strengthens the architectural value
  of a very small Tokimu-owned Ring 0 implementation where it is competent and
  justified: fewer executable foreign closure nodes reduce the trust surface.
- Limit: this incident does not prove that all foreign implementation is
  unacceptable or that self-authored code is inherently safe. A premature
  rewrite can increase correctness and maintenance risk. Provider removal or
  replacement still requires the evidence, performance, and ownership gates
  already retained by this review.
- Disposition: retain Incubating status and no stable vocabulary change. Treat
  supply-chain exposure as an additional named tradeoff in any later A/B/C
  selection: A/B retain provider-closure exposure under ADR-0010 controls; C
  trades that exposure for Tokimu's direct numerical and maintenance burden;
  D adds copied-source update and provenance work without current advantage.
- Resulting ADR or documentation change: no ADR change; recorded the incident
  as external supply-chain evidence for the existing review, not a substitute
  for Tokimu-specific audit or migration evidence.

### Cycle 50 -- 2026-08-09

- Status entering review: Incubating.
- New evidence: focused `cargo test -p hello-doom-e1m1` on the current pinned
  `third-party/ring-0/glam` source emits repeated Rust `unused_attributes`
  warnings. The upstream source applies `#[must_use]` to trait methods in impl
  blocks; the current compiler accepts it with a warning and says that
  acceptance is being phased out and may become a hard error in a future
  compiler. The local E1M1 failure in the same build was an unrelated consumer
  closure error and was fixed; all three focused tests then passed.
- Findings: this is compiler-compatibility and maintenance evidence for the
  executed foreign Ring 0 provider, not a current numerical defect and not a
  DOOM Slice 5B failure. Source pinning and audit make the cause visible, but
  they do not remove a future toolchain-upgrade obligation.
- Disposition: retain Incubating status. Record the warnings as a recurring
  pre-upgrade check for the pinned `glam` revision. If a Rust upgrade turns
  them into errors, assess an audited upstream revision, a local compatible
  patch with provenance, or the existing B/C ownership alternatives under the
  ADR-0010 gate; do not silently suppress or broaden the warning policy.
- Resulting ADR or documentation change: no ADR change; recorded the dated
  foreign-provider compiler-compatibility watch item.

### Cycle 51 -- 2026-08-10

- Status entering review: Incubating.
- New pressure: AR-0026 records future authored chart, transition-map, and
  angular deficit/excess semantics. Those may require spatially qualified local
  values whose meaning is not captured by an unqualified global `Vec3`/`Mat4`.
- Findings: non-Euclidean pressure strengthens the case for separating Tokimu
  semantic vocabulary from provider mechanics, but does not establish that
  new chart types belong in the math layer or that `glam` must be replaced.
- Disposition: retain Incubating status. Revisit after an executable AR-0026
  corpus identifies actual operations and ownership; do not expand the current
  math candidates speculatively.
- Resulting ADR or documentation change: no ADR change; cross-review pressure
  is now explicit.

### Cycle 52 -- 2026-08-11

- Status entering review: Incubating.
- New pressure: reopened AR-0028 proved that the current Doom coordinate lift
  is exactly invertible and round-trippable while reversing a canonical
  landmark orientation relative to world-up. It also opposes lifted
  source-right and observer camera-right.
- Findings: spatial transform evidence needs properties beyond finite and
  invertible. Orientation-preserving versus orientation-reversing is semantic
  information about a framed mapping; an unqualified `Mat4` or pair of `Vec3`
  values does not name that intent. This supports semantic roles above ordinary
  math mechanics, but does not require changing the five math types or their
  implementation provider.
- Disposition: retain Incubating status and the existing A/B/C/D math study
  outcomes. Feed the demonstrated orientation property into future spatial
  vocabulary studies; do not make raw math types infer source or chart intent.
- Resulting ADR or documentation change: no ADR change; AR-0028 supplies a
  concrete case study for future semantic transform admission.

### Cycle 53 -- 2026-08-12

- Status entering review: Incubating.
- New evidence: strict Clippy validation during unrelated AR-0025 work again
  encountered the pinned `glam` 0.29.3 `unused_attributes` warning flood. The
  warning is repaired upstream by commit
  `d5d92e48d628f2232295770c4a7b909e4b81c150`, first contained in the 0.30.2
  release. Comparing audited revision
  `d36e7eeff05338c56c4aa8d59fc2615e7963b1b7` with the 0.30.2 release commit
  `73e8582703ea1790dd41d0faca3df8beda4730a3` changes approximately 170 files,
  with `29,090` insertions and `10,156` deletions, including generated
  swizzle/SIMD implementation and tests.
- Findings: the smallest released maintenance route that clears the warning is
  a substantive foreign Ring 0 source update under ADR-0010, not a warning-only
  pin bump. Source pinning correctly prevented routine corpus validation from
  silently importing that delta. This is measured recurring lifecycle and
  audit pressure against Alternative A; it is not a numerical defect in 0.29.3
  and does not independently establish that Alternative C is correct.
- Disposition: retain the audited 0.29.3 pin and Incubating status. Count the
  update surface in the eventual A/B/C comparison: A and B inherit recurring
  review of a broad generated provider, while C accepts a one-time migration
  and continuing Tokimu-owned numerical/performance burden. Do not let warning
  irritation substitute for the outstanding correctness, performance,
  native/WASM, and caller-pressure gates.
- Resulting ADR or documentation change: no ADR change and no dependency
  update. The submodule was restored cleanly to the audited revision after the
  comparison.

### Cycle 54 -- 2026-08-12

- New cross-review hypothesis: AR-0026's future chart/junction corpus can test
  whether sophisticated Tokimu-owned spatial meaning remains compatible with
  a small, ordinary owned numerical core.
- Findings: this is deliberately two-sided evidence. If A and C0 produce the
  same traversal, transition-composition, orientation-classification, and
  native/WASM observations without materially expanding C0, that supports
  Alternative C. If chart work demands broad operations, unusual numerical
  robustness, or specialized SIMD machinery, it weakens C. Chart identity,
  qualified locations, transitions, and orientation intent remain semantic
  wrappers above raw math and are not counted as C0 operations.
- Disposition: retain Incubating status. Add an A/C0 run beneath the same future
  AR-0026 semantic layer; do not use non-Euclidean ambition as a one-way reason
  to replace `glam` or add speculative operations now.

### Cycle 55 -- 2026-08-12

- Status entering review: Incubating.
- New evidence: post-DOOM A/B/C caller controls found C0's generic
  Gauss--Jordan inverse about 10.4 times A for one repeated well-conditioned
  affine control. That behavior explained material C0 regressions in the
  retained CAD cursor-ray and pinned Khronos Box model/floor paths. A safe
  scalar C1 affine fast path was then tested against the retained C0 reference
  for 128 deterministic affine cases and a non-affine fallback case. It
  recovered the GLB control while leaving the non-affine CAD inverse path open.
  A bounded E1M1 PreserveNorth observer-camera port also remained about 14.5%
  above A on the recorded host.
- Findings: owning a narrow numerical core exposes real implementation and
  maintenance work; it does not merely remove foreign-provider review. C1 adds
  no `unsafe`, SIMD, target-specific code, provider call, or production API,
  but it is only a partial corpus response because projection/picking inversion
  remains on the C0 reference path.
- Disposition: retain Incubating status. Treat the C1 affine path as bounded
  corpus evidence, retain C0 checked/non-affine behavior as a scalar control,
  and require separate numerical/performance evidence before expanding C1 to
  general projection inversion. A four-class A/C remediation model is retained
  separately; it does not recommend changing ADR-0010 or selecting C.

### Cycle 56 -- 2026-08-12

- Status entering review: Incubating.
- New evidence: Slice 11 ran one corpus-local AR-0026 three-chart control under
  both direct A and owned C0. Identical local chart identity, transition order,
  point/direction transport, inverse round-trip, and orientation classification
  produced fingerprint `2520c9de` on native. The control asked only for the
  existing `Vec3`/`Mat4` operation manifest. The actual DOM/WASM control then
  produced the same `2520c9de` fingerprint with no provider acquisition.
- Findings: richer spatial meaning can remain above a bounded ordinary
  numerical core in this control. Invertibility and orientation behavior remain
  separate semantic facts; neither requires raw matrices to infer chart intent.
  This is modest two-sided evidence for C, not a chart API or a general owned
  math conclusion. Any future broad robustness, operation, SIMD, or target
  pressure remains evidence against C.
- Disposition: retain Incubating status and ADR-0010 unchanged. The completed
  two-target control supports the provider-free Ring 0 hypothesis only within
  its bounded corpus scope.

### Cycle 57 -- 2026-08-12

- Status entering review: Incubating.
- Maintainer decision: **retain Alternative A** — the audited pinned `glam`
  re-export remains Tokimu's one stable production math vocabulary. Continue
  C0/C1 only as executable corpus evidence; do not begin public migration.
- Decision basis: C demonstrated a bounded owned surface, explicit numerical
  behavior, 63 local tests, native and actual DOM/WASM chart agreement, and a
  safe affine C1 recovery for measured GLB pressure. It also demonstrated an
  unresolved non-affine inverse deficit, unperformed production-boundary
  migration, and Tokimu's long-term numerical/optimization maintenance burden.
  Those are specific remaining risks, not a tie settled by provenance hygiene
  or upstream warning irritation.
- Compute finding: retain CPU and WGPU as separate corpus mechanisms. E1M1
  scale remains CPU-favored; only large warm batches show WGPU advantage. No
  shared compute contract, provider-selection policy, or specialized provider
  is earned.
- Disposition: ADR-0010 remains unchanged. Reopen a selection decision only if
  independently pressured callers keep the ordinary C surface bounded while
  resolving the named performance, migration, and maintenance gaps, or if
  future evidence makes A's audited-provider cost disproportionate.

### Cycle 58 -- 2026-08-12

- Status entering review: Retain A; C0/C1 remains incubating corpus evidence.
- Maintainer direction: plan a real Alternative A update from audited `glam`
  0.29.3 to the current stable release and measure the level of effort required
  by ADR-0010 rather than estimating its inconvenience.
- Intake evidence: crates.io reported 0.33.3 as current. Upstream tag `0.33.3`
  peels to commit `9928729066db87d97fa779e129469721a289beae`.
  The target must be rechecked and reconciled with its registry artifact before
  the gitlink moves.
- Plan: `docs/Plans/Native-Math/Studies/ar-0019-option-a-glam-current-release-update.md`
  separates active mechanical, source-review, integration, validation,
  documentation, maintainer-decision, automation, blocked, and rework effort.
  It permits admit, reject, or bounded pause outcomes and does not weaken
  ADR-0010 or authorize a public-vocabulary change.
- Disposition: retain A at 0.29.3 until the plan's revision-specific audit and
  maintainer gate accept a new exact pin. Count the completed exercise as
  recurring Option A lifecycle evidence whether 0.33.3 is admitted or rejected.

### Cycle 59 -- 2026-08-12

- Status entering review: executing the Option A 0.33.3 update plan in an
  isolated worktree; production remains pinned to audited 0.29.3.
- Passing evidence: exact registry/Git source reconciliation passed for all 202
  `src/` blobs; the candidate retained a dependency-free `std`-only closure;
  strict core Clippy, 29 core tests, wasm32 core build, existing math-study
  controls, and representative caller compilation passed. The 4,896 old
  generated-swizzle diagnostics disappeared without suppression.
- Architectural finding: 0.33.3 deprecates the `Mat4` camera/projection
  constructors Tokimu currently uses and redirects callers to new
  `glam::camera::*` modules. Strict `tokimu-render` Clippy fails on
  `look_at_rh`, `perspective_rh_gl`, and `orthographic_rh_gl`. Repository search
  found 86 textual calls across those families.
- Ownership consequence: following upstream advice would expand the foreign
  vocabulary beyond the admitted five types; suppressing the warnings violates
  the update plan; a Tokimu compatibility seam would itself require explicit
  semantic ownership review.
- Disposition: pause before production pin movement or repair. Retain 0.29.3
  until maintainers choose among rejecting/pausing the update, authorizing a
  narrow Tokimu-owned camera/projection seam, or separately reviewing the new
  foreign camera vocabulary. Passing compilation does not settle that choice.

### Cycle 60 -- 2026-08-12

- Status entering review: Alternative A retained; the 0.33.3 update paused at
  the camera/projection vocabulary finding.
- Maintainer review: authorize investigation of a narrow Tokimu-owned
  camera/view/projection construction seam while retaining `glam` as the
  ordinary numerical implementation. Do not expose `glam::camera`, suppress
  the warnings, reopen the A/C selection, or move the production pin yet.
- Finding: the update cost is vocabulary-boundary work rather than evidence
  that `glam`'s numerical implementation failed. The SDD, AR-0024, AR-0026,
  and AR-0028 already constrain Tokimu's meaning more narrowly than upstream's
  complete camera module.
- Follow-up: AR-0029 records the four alternatives and authorizes an isolated
  three-family prototype with numerical, invalid-input, caller, native/WASM,
  and placement gates.
- Disposition: continue the Option A update study under AR-0029; retain 0.29.3
  as the production pin until the seam and revision audit are accepted.

### Cycle 61 -- 2026-08-12

- Status entering review: the Option A update remains active under AR-0029;
  production remains pinned to 0.29.3.
- New finding: the 0.33.3 camera deprecations are direct pressure for
  Alternative B, not merely revision-update inconvenience. The numerical
  provider remains credible while its changed camera vocabulary would reach
  86 Tokimu call sites. A Tokimu-owned semantic seam can absorb that churn
  while continuing to delegate arithmetic to `glam`.
- Scope distinction: **Narrow B** retains the admitted foreign `Vec2`, `Vec3`,
  `Vec4`, `Quat`, and `Mat4` vocabulary while Tokimu owns only pressured
  semantics such as checked view and projection construction. **Full B** owns
  wrappers for all five public types while using `glam` privately. Current
  evidence strongly pressures Narrow B; it does not establish Full B's
  conversion, representation, ergonomics, performance, or maintenance case.
- Sequencing decision: finish the active Option A analysis and record its
  evidence-bearing disposition first. Then create and execute a dedicated
  Option B test plan, using the same explicit slices, controls, effort ledger,
  cross-target evidence, and maintainer gate used for Options C and A. The
  AR-0029 prototype is input evidence for that study, not a substitute for it.
- Required comparison: evaluate A, Narrow B, Full B, and retained C evidence
  without assuming that one wrapper scope represents all of B. Measure upgrade
  shock absorption, caller migration, conversion pressure, API duplication,
  layout/ABI implications, optimization barriers, native/WASM behavior, and
  lifetime maintenance ownership.
- Disposition: queue Option B behind completion of the current A update study.
  Do not reopen the production A/C decision or move the pin merely because
  Narrow B currently appears attractive.

### Cycle 62 -- 2026-08-12

- Status entering review: Option A 0.33.3 remains isolated and production
  remains pinned to audited 0.29.3.
- New reproducibility evidence: a fresh parent worktree initialized the
  canonical `glam` submodule, replayed the candidate into 15 byte-identical
  Tokimu files, rolled back cleanly to 0.29.3, and reproduced the candidate a
  second time. With the exact candidate gitlink staged, the Ring 0 audit passed
  and locked/offline core tests and metadata resolution succeeded.
- Security evidence: workspace-local `cargo-audit` scans against one dated
  RustSec snapshot produced identical parent/candidate results and no `glam`
  advisory. Two `quick-xml` vulnerabilities and four `paste`, `ttf-parser`, or
  `lru` warnings remain a real non-Ring-0 workspace baseline; they are not an
  Option A regression and are not described as resolved by this study.
- Caller evidence: the repaired candidate passes the retained differential and
  plain-WASM controls, strict representative compilation, 41 Doom controls,
  and 13 orientation/camera/picking controls. No floating-point tolerance was
  widened. Clean/incremental core build times and rlib size show no material
  bounded difference; the initial camera-cost finding remains documented as
  duplicated work that was repaired and rerun.
- Remaining admission gates: actual browser presentation is blocked by the
  local harness/runtime mismatch; NEON, wasm64, and SIMD-browser execution are
  retained target gaps; the complete workspace suite remains blocked by
  pre-existing `AssetLoader::Error` omissions; and AR-0029's stable seam
  placement still requires maintainer judgment.
- Disposition: continue to retain production 0.29.3. The candidate is now
  reproducible and substantially audited, but it is neither admitted nor
  rejected. Complete or explicitly waive the named target/whole-workspace
  gaps and decide AR-0029 before producing a revision-specific accepted audit.

### Cycle 63 -- 2026-08-12

- Status entering review: Option A 0.33.3 remains isolated; production remains
  pinned to audited 0.29.3.
- Compatibility evidence: dependency-isolated probes against the exact local
  old and candidate sources emitted the same 14-line fingerprint for the five
  public types, including layouts, fields/columns, constants, conversions,
  operations, transforms, inverse observations, quaternion rotation, and
  bounded non-finite behavior. All 44 candidate math-study tests and 38 core
  tests pass under locked/offline resolution. The same Alternative A control
  was also built with WebAssembly `simd128` enabled and both pins produced the
  same five bounded observations under Node 22.22.2; actual-browser SIMD,
  wasm64, and NEON execution remain separate target gaps. The complete
  candidate WASM conformance set passes 12/12 under both default features and
  `simd128`. Both pins compile-check for the NEON-enabled Windows AArch64
  target; stable Rust exposes no prebuilt wasm64 standard-library artifact.
- Browser gate: an official workspace-local Node 22.22.2 control and the
  configured bundled Node 24.14 runtime satisfy the browser harness minimum.
  The active browser bridge inherited stale system Node 22.21 and cannot be
  retargeted in-process, so actual-browser evidence requires a fresh session;
  this is retained as host/tooling burden rather than candidate failure.
- Ownership pressure: 58 of 86 deprecated camera/projection constructor calls
  remain intentionally unmigrated. The evidence no longer points to numerical
  repair; it points to the AR-0029 decision between a narrow Tokimu-owned
  checked construction seam, explicit admission of foreign camera vocabulary,
  or pausing/rejecting the candidate.
- Disposition: retain production 0.29.3 and keep 0.33.3 unadmitted. Resume the
  browser gate in a fresh bridge session, then obtain the AR-0029 maintainer
  decision before complete migration or a revision-specific accepted audit.

### Cycle 64 -- 2026-08-12

- Status entering review: Option A testing continues in isolation; production
  remains pinned to audited 0.29.3.
- Cross-target execution: all 12 WASM-applicable candidate conformance tests
  pass under Node 22.22.2 with default target features and in a separately
  built `simd128` configuration. The exact old/new SIMD control also retains
  five matching observations.
- ARM/wasm64 availability: exact old and candidate controls compile-check for
  Windows AArch64, whose compiler configuration explicitly enables NEON.
  Execution is unavailable on the x86-64 host. Stable Rust recognizes wasm64
  but provides no prebuilt standard-library artifact, and the study did not
  introduce a source-built or alternate-toolchain substitute.
- Harness boundary: a direct core-WASM test attempt compiled but exported zero
  ordinary Rust tests to `wasm-bindgen-test-runner`. The 38 core tests remain a
  native claim; only the 12 conformance tests are claimed as WASM execution.
- Browser status: the candidate fixture remains built, but the active browser
  bridge still resolves stale Node 22.21 despite newer verified runtimes. The
  local server used for the retry was stopped. Actual DOM/WebGPU presentation
  remains open and cannot be inferred from Node execution.
- Disposition: target evidence is stronger but unchanged architecturally.
  Retain production 0.29.3 and require the fresh browser gate plus AR-0029
  maintainer decision before admitting or migrating the candidate.

### Cycle 65 -- 2026-08-12

- Status entering review: the Option A browser gate remained blocked by the
  machine-wide Node 22.21.0 runtime resolved by the active bridge.
- Toolchain maintenance: the official latest 22.x release was 22.23.2. Its x64
  MSI matched the published SHA-256 exactly, and the installed executable has
  a valid OpenJS Foundation signature. The system now reports Node 22.23.2 and
  npm 10.9.8; the workspace-local 22.22.2 control remains retained separately.
- Effort finding: the first silent update rolled back with MSI 1730 because
  replacement of the machine installation required administration. The
  elevated retry succeeded. This adds one failed/recovered installation and
  roughly 85 seconds of successful installer wall time to Option A's measured
  environmental maintenance burden.
- Remaining host boundary: Windows Installer restarted the stale bridge, but
  its service now reports a missing runtime path despite valid system 22.23.2
  and configured bundled 24.14 executables. The Codex host must be restarted
  before actual browser evidence can resume; Node/WASM execution does not
  substitute for that gate.
- Disposition: the requested Node update is complete and recorded. Retain the
  unadmitted 0.33.3 candidate and production 0.29.3 until the post-restart
  browser fixture and AR-0029 maintainer decision complete.

### Cycle 66 -- 2026-08-12

- Status entering review: the Codex host has restarted after the verified
  machine Node update; the production pin remains audited 0.29.3.
- Browser-gate refinement: the candidate fixture builds and serves successfully
  under system Node 22.23.2, closing the earlier runtime-version mismatch. The
  restarted browser controller reports zero available browser instances, so no
  actual DOM/WebGPU observation can be made in this environment. The local
  server was stopped, and Node WASM evidence is not substituted for browser
  evidence.
- Audit status: a revision-specific candidate audit now names exact 0.33.3
  commit `9928729066db87d97fa779e129469721a289beae`, selected closure, reviewed
  source/unsafe surface, validation, reproducibility, measured effort, and open
  gates. It is explicitly a reviewed candidate audit, not an admission record.
- Effort result: the ledger records approximately 124 active agent minutes,
  placing completed work in the study's Routine band. The trust burden remains
  material: 290 changed upstream files, generated-source reproduction and a
  175-package development-tool closure, cross-target work, a machine Node
  update, and an ownership review for 86 deprecated constructor sites.
- Architectural result: no evidence requires rejecting `glam` as numerical
  provider, but the update directly strengthens the case for studying Narrow
  B after Option A closes: Tokimu-owned camera/projection meaning could absorb
  upstream vocabulary churn while retaining the five current foreign value
  types and private numerical implementation. This does not establish Full B
  or reopen the A/C decision.
- Disposition: pause the 0.33.3 admission with explicit resumption conditions:
  an attachable actual browser, the stable AR-0029 ownership decision and
  resulting migration, repair of unrelated full-workspace test/Clippy
  baselines, and an explicit maintainer admission vote. Production remains on
  0.29.3; Option B planning remains sequenced after this evidence-bearing
  Option A pause.

### Cycle 67 -- 2026-08-12

- Status entering review: production retains Alternative A at audited 0.29.3;
  the 0.33.3 update has an evidence-bearing pause; C0/C1 remains executable
  incubation.
- Maintainer direction: create the queued Option B study from the concrete
  Option A findings before changing the production vocabulary.
- Plan structure: the new study treats **Narrow B** and **Full B** as separate
  candidates. Narrow B owns only the AR-0029 camera/view/projection semantics
  while retaining the five current public foreign value types. Full B owns
  wrappers for all five names while keeping the exact pinned provider private.
- Required evidence: both candidates must face the same 0.29.3-to-0.33.3 update
  shock, caller inventory, representative migrations, conversion accounting,
  native/WASM/browser controls, ADR-0008/0009/0011 gates, AR-0026/0028 pressure,
  and maintenance economics. Full B cannot inherit Narrow B's evidence or claim
  supply-chain independence while foreign code still executes in Ring 0.
- Guardrail: retaining A remains a valid outcome. Narrow B is not presumed to
  grow into Full B, and a passing prototype does not authorize a stable public
  API, provider-pin movement, or production migration.
- Resulting artifact:
  `docs/Plans/Native-Math/Studies/ar-0019-option-b-provider-backed-vocabulary-and-semantic-seams.md`.
- Disposition: Option B is planned and ready for isolated execution; production
  remains unchanged until a later evidence-bearing maintainer decision.

### Cycle 68 -- 2026-08-12

- Status entering review: the Option B plan exists; production remains on
  audited Alternative A at 0.29.3; the 0.33.3 candidate remains paused rather
  than admitted or rejected.
- Reviewer interpretation: Monday judged the approximately 124 active minutes
  favorable evidence that ADR-0010 can be strict without making an ordinary
  Ring 0 provider update pathologically expensive. This is not summarized as
  “a `glam` update takes two hours”: automated wall time, tooling, environment
  repair, trust-surface breadth, and unavailable-target gaps remain separately
  retained.
- A/B/C consequence: C remains feasible but owns numerical and performance
  maintenance; A's provider maturity and measured update economics remain
  attractive; B now has concrete pressure because semantic camera/projection
  construction can potentially be insulated while the provider remains useful.
- Narrow-B hypothesis: retain the `glam` implementation and possibly the five
  existing value types, keep `glam::camera` private, and make only the already
  demonstrated view/projection construction Tokimu-owned. GPU bulk compute
  remains a separate Outer Ring question. This is a study hypothesis, not an
  accepted architecture.
- Gate discipline: Node WASM does not satisfy actual browser/WebGPU evidence;
  the reproduced `AssetLoader::Error` failure remains an unrelated workspace
  baseline; and the AR-0029 prototype is not stable admission before its
  ownership decision.
- Disposition: add these interpretive guardrails to the Option B plan. Do not
  let enthusiasm for Narrow B rewrite Option A's evidence, preselect Full B,
  or move the production provider pin.

### Cycle 69 -- 2026-08-12

- Status entering review: the maintainer authorized beginning the isolated
  Option B study. Production remains on audited Alternative A at 0.29.3;
  AR-0029 remains Under Review.
- Frozen candidates: Narrow B remains the three checked camera/projection
  construction families over the existing five foreign value types. Full B
  remains the five-name wrapper prototype with a private pinned provider. They
  have separate identities and ledgers and cannot inherit each other's result.
- Full-B control: the current shared wrapper is 553 lines (459 nonblank), has
  39 literal private provider references, and exposes ten isolated tests. All
  ten pass against exact 0.29.3, while private provider compilation
  reproduces the known generated-warning flood.
- Early finding: B can potentially contain public vocabulary churn, but it
  cannot contain provider compilation, provenance, audit, unsafe/SIMD, target,
  or remediation work. This prevents a wrapper from being misreported as
  supply-chain independence.
- Refreshed pressure: the current tree contains 94 direct calls across the
  three constructor families. The dated Option A finding of 86 calls remains
  intact because its isolated tree and scan scope differ. `Vec3`, `Vec4`, and
  `Mat4` retain real caller pressure; `Vec2` and `Quat` still lack stable Native
  pressure and remain minimal compatibility probes only.
- Public-boundary result: the five direct `tokimu_core::math` re-exports and
  public renderer `Camera` matrices remain the concrete foreign-vocabulary
  seam. No stable math serialization, FFI, POD, reflection, or TypeScript
  layout contract was found.
- Resulting evidence: `2026-08-12-option-b-control.md`,
  `2026-08-12-option-b-result-ledger.md`, `2026-08-12-option-b-loe.md`, and
  `2026-08-12-option-b-pressure-scan.md` complete Slices 0 and 1 without a
  production dependency, pin, export, or stable-contract change.
- Disposition: proceed to provider-neutral contract definition. Keep removal
  or outward movement visible for unpressured names, and do not assume Narrow
  B must grow into Full B.

### Cycle 70 -- 2026-08-12

- Maintainer direction: finish Option A rather than leave its completed Pause
  disposition looking like an active study while Option B proceeds.
- Closure validation: repository-resolvable baseline repairs are complete.
  Locked/offline whole-workspace tests pass against production 0.29.3 and the
  isolated 0.33.3 candidate; both trees pass formatting checks; strict
  whole-workspace Clippy passes for 0.33.3.
- Warning evidence: production 0.29.3 still reproduces 4,896 actionable
  provider-owned generated-source diagnostics. Independently discovered
  Tokimu-owned math-study lint findings were repaired and are not attributed
  to the provider update.
- Unavailable-target result: the actual-browser gate was attempted after the
  host/runtime repair, but the browser controller exposes no attachable
  instance. Node WASM remains separate evidence and is not substituted.
- Final Option A study disposition: **Complete — Pause**. Production retains
  audited 0.29.3; reviewed 0.33.3 remains unadmitted. Resume admission only
  after AR-0029 ownership and migration are resolved, an actual-browser run is
  retained, and the maintainer makes a fresh admission vote.
- Sequencing: this closure does not rewrite Option A as a rejection, select B,
  or authorize a provider-pin movement. It merely satisfies the plan's
  evidence-bearing Pause completion criterion so Option B can use a stable A
  control.

### Cycle 71 -- 2026-08-12

- Status entering review: Option B Slices 0 and 1 were complete; production
  remained on Alternative A at 0.29.3 and AR-0029 remained Under Review.
- Contract result: Narrow B now owns only three checked construction families
  and four bounded failure categories. Full B has a separately retained
  five-name contract that deliberately leaves `Vec2`/`Quat` as compatibility
  probes and rejects provider layout, broad traits, serialization, and API
  mirroring.
- Narrow-B hardening: one independent caller and four scalar/reference contract
  cases pass unchanged against exact 0.29.3 and 0.33.3. The provider switch is
  isolated to three private construction-call pairs; no provider camera module
  or provider error crosses the candidate boundary.
- Failure evidence: invalid and non-finite construction inputs are classified
  by Tokimu operation/failure identity. The study no longer relies on provider
  panics or accidental NaN propagation as its public failure contract.
- Placement result: `tokimu-core::math` is the smallest existing candidate
  location if AR-0029 later admits the seam; renderer camera lifecycle and WGPU
  adaptation do not move inward.
- Disposition: complete Option B Slices 2 and 3. Continue to Full-B hardening
  without treating Narrow B as an incomplete form of Full B or changing the
  production pin/public surface.

### Cycle 72 -- 2026-08-12

- Full-B hardening result: one unchanged five-name wrapper source and external
  contract harness pass against exact private providers 0.29.3 and 0.33.3.
  Provider selection and the three camera/projection API adaptations remain
  private; no foreign type, trait, module, feature, or error crosses the
  candidate boundary.
- Failure contract: checked normalization, inversion, projective point
  transformation, and the three camera/projection constructors now retain
  bounded Tokimu operation/failure identity. Unchecked spellings remain
  explicitly labeled compatibility-pressure controls rather than the admitted
  failure contract.
- Evidence: both pins pass 10 shared unit and five external contract tests.
  The suite includes independent scalar landmarks, fixed-seed metamorphic
  cases, and non-finite, zero-length, singular, zero-homogeneous-W, degenerate
  view, and invalid-frustum controls. The shared parent math study remains
  green at 63 unit and six integration tests.
- Reference inventory: the wrapper retains 39 literal provider-namespace
  references (23 storage/construction mechanics, three semantic adapters, five
  private conversions, eight control-test references) plus 44 delegated
  `.inner` uses. These are visible maintenance surface, not implementation
  independence.
- Ergonomics result: Full B requires accessors and explicit mutation/crossing
  seams and deliberately omits indexing, serialization, POD/FFI, broad generic
  traits, swizzles, and unpressured quaternion/vector breadth. Equal observed
  layout is not a promise and supports no unsafe conversion.
- Provider quality result: focused strict Clippy is clean against 0.33.3. The
  0.29.3 build still reproduces its retained provider-owned warning flood;
  readable test output used warning suppression without reclassifying that
  evidence as fixed.
- Disposition: complete Option B Slice 4 and proceed to representative
  migration/conversion accounting. Full B is feasible, but it has not yet
  shown benefit beyond Narrow B proportional to its larger wrapper surface.

### Cycle 73 -- 2026-08-12

- Representative migration result: Narrow B passes four contract and four
  application-shaped tests unchanged against exact 0.29.3 and 0.33.3. Five
  caller scenarios make eight checked semantic-construction calls with no
  changed value signature, accessor substitution, value conversion, or
  temporary allocation.
- Full-B pressure: nine A/B/C-comparable caller modules now cover camera,
  renderer, GLB, CAD/picking, E1M1 observer, FPS motion, hierarchy mutation,
  stereo/orthographic presentation, and AR-0026 chart orientation. Doom
  collision remains source-scalar/integer code and was not forced through the
  math vocabulary merely to satisfy the study.
- Friction accounting: the bounded Full-B port contains four private
  wrapper-bearing helper signatures, eight scalar accessor substitutions, one
  matrix-column setter, and nine explicit renderer matrix crossings. No
  retained caller lost a required trait, and no unsafe layout equivalence was
  introduced.
- Runtime evidence: transform, upload, and stereo allocation controls remain
  at zero allocations for the measured A/B/C paths. Long-lived camera state
  and renderer transport use owned wrapper values and scalar columns rather
  than provider ABI assumptions.
- Chart result: A, Full B, and C0 produce the same `2520c9de` fingerprint for
  the AR-0026 control. Full B needed no new operation; explicit unit vectors
  avoided manufacturing X/Z constant pressure.
- Interpretation: Narrow B directly contains the semantic-constructor shock
  observed during Option A with much less surface. Full B remains feasible but
  has not yet demonstrated a proportional benefit for owning all five public
  value types. Complete Slice 5 and proceed to native/WASM/browser and
  representation gates without changing production.

### Cycle 74 -- 2026-08-12

- Cross-target result: the shared A/B/C suite, Narrow-B contract and caller
  suites, and Full-B external contract suite all execute successfully in the
  Node WebAssembly engine under both default WASM and explicit `simd128`.
  Narrow and Full B pass unchanged against exact private providers 0.29.3 and
  0.33.3; bounded values and failure classes agree with native evidence.
- ARM result: both candidates and their retained targets compile for
  `aarch64-pc-windows-msvc` under both pins, whose target cfg advertises NEON.
  No ARM64 execution occurred, so this is portability evidence, not a NEON
  runtime or performance claim.
- Browser result: the updated A/B/C chart fixture was available locally, but
  the browser-control runtime exposed no attachable browser. No new B
  actual-browser or browser-WebGPU result is claimed; Node-hosted WASM remains
  explicitly separate from earlier inherited A/C browser evidence.
- Representation result: Full B observes the same five sizes, alignments,
  `Copy` behavior, scalar access, and ordered matrix columns under both pins.
  This does not admit provider layout, SIMD identity, ABI/POD, serialization,
  public fields, unsafe conversion, or GPU buffer equivalence. Narrow B retains
  provider representation by type identity and owns only constructors.
- Availability limits: NVIDIA, wasm64, ARM64/NEON execution, and a fresh actual
  browser remain unobserved. Complete Slice 6 for available targets and proceed
  to the ADR-0008 performance/code-quality gate without changing production.

### Cycle 75 -- 2026-08-12

- Narrow-B cost: bounded validation is real work, not a zero-cost abstraction.
  In a three-constructor stress loop it is 3.56-3.60x the private provider call,
  while the absolute delta is 38.2-39.3 ns per view/perspective/orthographic
  bundle on this host. Mass-camera pressure remains untested.
- Full-B caller result: transform, stereo, CAD, and E1M1 observer medians remain
  competitive with A, and all measured transform/upload/stereo hot paths
  allocate zero. The Khronos Box GLB path regresses 7.8% and affine inverse
  isolation regresses 18.5%; Full B therefore loses those workload gates rather
  than receiving a general zero-cost claim.
- Update comparison: the same Full-B wrapper under exact private pins 0.29.3
  and 0.33.3 shows no material regression in bounded transform, inverse, or
  stereo/column-handoff controls. This does not erase Full B's A-relative
  workload failures.
- Maintenance evidence: Full B has a roughly 4.3x larger candidate rlib and a
  slower incremental isolated build than Narrow B in this observation. Both
  remain small, but the broader owned vocabulary has visible compile and
  artifact cost in addition to source maintenance.
- Quality result: formatting and focused strict all-target Clippy pass against
  0.33.3. The 0.29.3 warning flood remains private-provider evidence; it was
  suppressed only in benchmark output and is not reclassified as clean.
- Disposition: complete Slice 7 without representation shortcuts, unsafe
  conversion, or speculative inlining. Narrow B remains the proportional
  candidate for semantic shock absorption; Full B is feasible but has not
  earned its broader performance burden. Continue to ADR-0009/0011 verification
  and failure-containment pressure without changing production.

### Cycle 76 -- 2026-08-12

- Failure result: Narrow and Full B now exercise malformed typed scalar
  combinations, non-finite input, zero-length/degenerate values, underflow,
  finite-component overflow, invalid frusta, and their applicable singular or
  zero-homogeneous cases under exact private providers 0.29.3 and 0.33.3.
  Public errors retain only bounded operation and failure identities; provider
  diagnostics and raw inputs do not cross either checked boundary.
- Ordinary finding: Full B's checked normalization accepted finite components
  whose squared magnitude overflowed and could return a finite zero. The
  candidate now rejects that case as `NonFiniteResult`, keeps underflowed
  magnitude distinct as `ZeroLength`, and passes unchanged under both pins on
  native and Node-WASM.
- Containment result: checked invalid inputs return without native unwind and
  execute without WASM trap. Native `catch_unwind` is retained only as an
  observation, not a recovery design. Unchecked compatibility methods remain
  outside the bounded-failure claim and would require disposition before Full
  B admission.
- Security result: candidate implementation adds no ambient global, heap
  owner, provider lifetime, thread, I/O, callback, unsafe, or secret-bearing
  authority. Corpus-only scalar WASM probes are not proposed math contracts.
- Provenance result: each candidate's normal/build closure is exactly the
  candidate plus one exact local `glam` provider. B does not remove A's source,
  unsafe/SIMD, advisory, legal, target, rollback, or update-review obligations;
  private vocabulary is not supply-chain independence.
- Disposition: complete Slice 8 for the checked candidate surfaces and
  available native/Node-WASM targets. Actual-browser evidence remains
  unavailable from Slice 6. Continue to provider-update shock and maintenance
  economics without changing production or weakening ADR-0010.

### Cycle 77 -- 2026-08-12

- Update-shock result: the frozen 0.29.3-to-0.33.3 camera API change exposed 86
  direct A construction sites. Once adopted, Narrow B and Full B each replay
  the update with zero public caller changes and the same three private
  constructor-adapter changes. Full B provides no additional insulation over
  Narrow B for the shock that actually occurred.
- Adoption accounting: this is conditional insulation, not free migration.
  The Option A prototype actually migrated 28/86 representative sites and left
  58; adopting Narrow B still incurs that one-time construction migration.
  Full B additionally incurs the much broader five-type/crossing migration and
  its already measured performance, compile, and documentation burden.
- Semantic-shock control: unchanged Full-B wrapper source produces different
  NaN `Vec3::min/max` bit results under the exact two provider pins. A and
  Narrow B expose that foreign value behavior directly; Full B also exposes it
  because its method delegates. Tokimu naming does not stabilize semantics
  unless a specific policy is admitted, implemented, and tested.
- New-operation control: existing `Vec3::dot` pressure costs A and Narrow B no
  Tokimu surface when the provider already supplies it. Full B must grow a
  wrapper contract/delegation/test; C must grow owned mechanics/contract/test.
  Broad ownership therefore has recurring per-operation economics.
- Unchanged burden: provenance, source diff, generated code, unsafe/SIMD,
  targets, advisories, licenses, rollback/replay, offline enforcement, and
  maintainer admission remain identical to Option A for either B candidate.
- Disposition: complete Slice 9. The comparative evidence favors Narrow B as
  proportional shock absorption but does not admit it; Full B has not shown
  benefit proportional to its broader surface. Continue to ergonomics and
  ecosystem pressure with production unchanged.

### Cycle 78 -- 2026-08-12

- Ergonomics result: representative Narrow-B callers state right-handed and GL
  depth construction intent directly, return bounded errors, and otherwise
  retain ordinary provider values. Full-B finite caller source remains
  familiar, but familiar names do not reveal whether Tokimu owns edge semantics
  or delegates them.
- Documentation accounting: normal Rustdoc generation succeeds. A strict
  missing-docs control finds 16 Narrow-B items concentrated in its three
  functions and bounded failures, while Full B has a much broader missing set
  across eight top-level declarations, 60 inherent members, and 10 arithmetic
  trait implementations. Stable documentation work was not performed before
  admission.
- Compatibility finding: real non-study Doom callers require `Sum<Vec3>`, and
  several callers read or mutate provider public components/columns. Full B's
  bounded ports do not provide all of those behaviors. The study retains the
  gaps instead of expanding fields, traits, indexing, conversions, or layout
  merely for source compatibility.
- Ecosystem result: Narrow B leaves renderer camera storage and scalar GPU
  transport unchanged. Full B would change the stable public `Camera` value
  vocabulary and require explicit crossings. No current asset, ECS,
  serialization, reflection, FFI/POD, TypeScript layout, or external-tool
  caller earns a broader wrapper contract.
- Disposition: complete Slice 10. Narrow B improves semantic clarity with a
  small learnable surface; Full B remains understandable only by staying
  bounded and honestly incomplete rather than becoming a disguised `glam`
  clone. Continue to the AR-0026/0028/renderer cross-review without changing
  production.

### Cycle 79 -- 2026-08-12

- Spatial cross-review result: the existing three-chart semantic control
  produces fingerprint `2520c9de` under A, provider-backed Full B, and owned
  C0. Narrow B retains A's exact ordinary value mechanics; its stereo and Doom
  observer callers pass unchanged under both exact provider pins.
- Ownership result: chart identity, qualified location, transition and
  orientation intent, Doom source embedding, camera basis, input policy,
  active-camera selection, viewport, and provider clip conversion remain above
  or outside ordinary math. None became a Full-B method.
- Renderer result: the focused WGPU test confirms Tokimu's GL-style `[-1, 1]`
  camera depth is mapped privately to `[0, 1]` without mutating the camera.
  Neither B candidate moves provider clip meaning into caller code.
- View-pressure result: existing stereo and CAD multi-camera callers require
  independent camera identity, viewport, and submission, but no broader raw
  math surface. Portal-derived and recursive views remain unimplemented
  AR-0026/AR-0029 pressure and cannot be counted as B evidence.
- Operation-growth result: zero new ordinary operations were requested from B
  or C0. Exotic spatial semantics remain above bounded numerical mechanics.
- Disposition: complete Option B Slice 11 without production or public change.
  Proceed only to the comparative Slice 12 maintainer gate.

### Cycle 80 -- 2026-08-12

- Comparative result: A remains a healthy audited production control; Narrow B
  alone proportionally absorbs the observed camera-constructor update shock;
  Full B adds broad migration, documentation, semantic-drift, compatibility,
  and performance costs without additional benefit for that shock; C0/C1
  remains a credible but unselected ownership alternative.
- Recommendation: keep A in production, continue Narrow B incubation, park
  Full B, and retain C0/C1 as executable evidence. This is a recommendation,
  not maintainer acceptance or stable admission.
- Missing Narrow-B gates: actual-browser B execution, the AR-0029 GLB
  runtime/browser observation, real-workload judgment of the bounded checked
  constructor cost, stable documentation/placement, and an explicitly
  authorized rollout/rollback plan.
- AR-0029 recommendation: remain Under Review as bounded review guidance. It
  should become binding ADR material only if maintainers select Narrow B after
  the remaining gates; rejection should close it with no stable change.
- Production result: audited A 0.29.3 remains the only stable vocabulary and no
  production migration or provider-pin movement occurred.
- Disposition: pause at the explicit Slice 12 maintainer gate.

### Cycle 81 -- 2026-08-13

- Reopening pressure: attempting production admission of exact `glam` 0.33.3
  made Alternative A's direct camera/projection constructors deprecations under
  the repository's strict Clippy gate.
- Rejected responses: no blanket deprecation allowance and no public adoption
  of `glam::camera`; both would preserve or expand the foreign semantic
  vocabulary that the completed B study showed could be contained narrowly.
- Maintainer decision: admit Narrow B through ADR-0014. Production retains the
  five audited provider value carriers and provider numerical implementation,
  while Tokimu owns exactly three checked camera/projection constructors.
- Provider decision: admit exact `glam` 0.33.3 revision
  `9928729066db87d97fa779e129469721a289beae`; retain 0.29.3 as rollback and
  historical audit evidence.
- Verification: strict workspace Clippy, whole-workspace tests, native
  contracts, and fresh wasm32 camera-consumer compilation pass. Fresh 0.33.3
  actual-browser replay remains explicit ADR-0005 follow-up because no
  attachable browser was available during admission.
- Scope: Full B and Option C remain unselected. This result does not settle the
  broader Ring 0 self-implementation hypothesis.

## References

- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
- `docs/ADR/ADR-0008-native-kernel-ring-performance-and-code-quality.md`
- `docs/ADR/ADR-0009-ring-based-verification-failure-containment-and-recovery.md`
- `docs/ADR/ADR-0010-ring-zero-third-party-source-admission.md`
- `docs/ADR/ADR-0011-ring-based-security-authority-and-trust-boundaries.md`
- `docs/ADR/ADR-0014-tokimu-owned-semantic-operations-over-admitted-mechanical-values.md`
- `docs/Architectural Reviews/AR-0015-ring-zero-provenance-enforcement-and-audit-closure.md`
- `docs/Architectural Reviews/AR-0026-non-euclidean-spatial-charts-and-authored-angular-topology.md`
- `docs/Dependency Audits/Ring 0/glam-d36e7eeff05338c56c4aa8d59fc2615e7963b1b7.md`
- `docs/Dependency Audits/Ring 0/glam-9928729066db87d97fa779e129469721a289beae.md`
- `crates/tokimu-core/src/math.rs`
- Microsoft Threat Intelligence and Microsoft Defender Security Research Team,
  [Mitigating the Axios npm supply chain compromise](https://www.microsoft.com/en-us/security/blog/2026/04/01/mitigating-the-axios-npm-supply-chain-compromise/),
  2026-04-01.
