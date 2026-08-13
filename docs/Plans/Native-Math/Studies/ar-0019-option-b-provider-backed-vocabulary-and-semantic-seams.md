# AR-0019 Option B Provider-Backed Vocabulary And Semantic-Seam Study

| Field | Value |
| --- | --- |
| Status | Complete — Narrow B incubation retained; Full B parked; no production migration authorized |
| Owner | Tokimu maintainers |
| Date | 2026-08-12 |
| Related reviews | AR-0019 and AR-0029 |
| Production control | Alternative A: audited `glam` 0.29.3 re-exports |
| Update control | Reviewed but unadmitted `glam` 0.33.3 candidate |
| Existing prototype | `corpus/lib/tokimu-math-study/alternative-b-provider-backed` |
| Comparison | Narrow B, Full B, retained A, and incubating C0/C1 |

## Purpose

Test whether Tokimu can own the math meaning that proved vulnerable during the
Option A update while continuing to delegate numerical mechanics to the pinned,
audited `glam` provider.

Option A established two different pressures that must not be collapsed:

```text
0.33.3 camera/projection deprecations
    -> 86 Tokimu call sites
    -> new foreign camera vocabulary
    -> semantic ownership pressure

0.29.3 -> 0.33.3 implementation update
    -> 290 changed upstream files
    -> generated-source, unsafe/SIMD, target, and audit work
    -> implementation provenance pressure
```

Option B may absorb the first pressure. It does not automatically remove the
second because `glam` continues executing in Ring 0. The study must measure
both facts rather than crediting wrappers with supply-chain independence they
do not provide.

The study has two explicitly separate candidates:

- **Narrow B:** Tokimu owns only independently earned semantic construction,
  initially the right-handed view and GL-depth projection contract pressured by
  AR-0029. The five existing public `glam` value types remain unchanged.
- **Full B:** Tokimu owns public wrappers for `Vec2`, `Vec3`, `Vec4`, `Quat`, and
  `Mat4`; the pinned `glam` types and implementation remain private.

Narrow B is not a partial implementation of Full B. Either may be the correct
answer, and the study may retain A if neither earns its cost.

## Option A Findings This Study Must Test

1. The numerical provider remained credible across native, Node-WASM,
   `simd128`, AArch64 compile, caller-shaped performance, security, and legal
   gates.
2. Upstream camera/projection reorganization nevertheless reached 86 Tokimu
   sites and attempted to expand the foreign vocabulary beyond the five types
   admitted by AR-0019.
3. A three-family Tokimu constructor prototype stayed bounded and preserved the
   existing right-handed, Y-up, `-Z`-forward, GL-depth contract.
4. The first checked-construction benchmark exposed duplicated caller/provider
   work. After removing redundant default construction and normalization, the
   retained workload showed no wrapper-boundary throughput penalty.
5. Full provider updates still require ADR-0010 provenance, source-delta,
   generated-code, unsafe/SIMD, target, advisory, legal, and replay review.
6. Option A's recorded active effort was Routine, but the trust surface was
   materially larger than the mechanical pin change.
7. The existing Full-B corpus prototype is technically viable for a bounded
   subset, but already contains visible accessor, mutation, conversion, and
   wrapper-drift pressure. Earlier evidence is input, not a final decision.
8. AR-0026 and AR-0028 suggest unusual spatial and orientation meaning belongs
   above raw numerical mechanics. Full B must not absorb chart, frame, camera,
   source-orientation, or input policy merely because it owns type names.

## Interpretation Guardrails From The Option A Pause

Monday's review of the completed Option A pause identified an important
provisional interpretation that this study must try to falsify:

> Alternative A's implementation economics may be acceptable while its
> semantic exposure is too broad.

The approximately 124 active minutes are favorable evidence that ADR-0010's
strict update procedure is not inherently dysfunctional. The update crossed a
large source delta and retained identity, closure, generated-source,
unsafe/SIMD, numerical, performance, target, rollback, and candidate-audit
evidence within the study's Routine active-effort band.

That result must be stated precisely. It does not mean “updating `glam` takes
two hours.” Automated wall time, environment repair, tooling acquisition,
unavailable-target gaps, and the substantial trust surface remain separate
maintenance facts. Future updates may also cross different or riskier source.

The evidence therefore changes the starting hypothesis for B:

```text
Provider implementation maturity and update economics
    currently favor retaining audited glam

Foreign camera/projection vocabulary churn
    creates demonstrated semantic coupling

Narrow B
    may insulate Tokimu-owned meaning without replacing mechanics

Full B
    still needs independent evidence that five wrappers earn their cost
```

This is not a conclusion. Narrow B must prove that it absorbs caller churn
without duplicating provider work or growing into a camera framework. Full B
must prove additional value beyond that narrow seam. Retaining A remains the
control and a valid final result.

Option A's remaining gates also remain distinct inputs rather than B evidence:

- Node-hosted WASM does not satisfy the unavailable actual-browser/WebGPU gate;
- the reproduced `AssetLoader::Error` workspace failure is an unrelated
  baseline, not a candidate regression; and
- AR-0029 must make the semantic-ownership decision before its prototype can
  be treated as admitted Narrow B.

The likely architecture suggested by current evidence is deliberately only a
hypothesis for the study:

```text
glam numerical implementation             retain
five existing foreign value types         test retaining
foreign camera/projection vocabulary       hide
Tokimu camera/projection construction       own narrowly
Tokimu numerical implementation            do not own without new evidence
GPU bulk compute                            keep as a separate Outer Ring review
```

## Questions To Answer

### Narrow B

1. Can the AR-0029 seam remain limited to the three demonstrated construction
   families and engine-neutral validation?
2. Can callers remain unchanged across the 0.29.3 and 0.33.3 providers while
   provider-specific constructor organization changes privately?
3. Does the seam belong in foundational math, renderer camera code, or another
   already-existing boundary without creating a new subsystem?
4. Are invalid-input behavior, floating-point policy, and provider depth
   adaptation explicit and consistent across native and WASM?
5. Does Narrow B materially reduce future caller churn while adding little API,
   runtime, or maintenance cost?

### Full B

1. Can Tokimu own useful public contracts for all five names without copying
   `glam`'s broad API or exposing its traits and representations?
2. Which current fields, operators, traits, constants, constructors, mutation
   patterns, and conversions are genuinely required by real callers?
3. How many explicit crossings appear at renderer, asset, serialization,
   shader, corpus, or other provider boundaries?
4. Do wrappers introduce measurable conversion, layout, ABI, compile-time,
   binary-size, allocation, SIMD, or ergonomics costs?
5. Does Full B actually contain provider-update shock, or merely move it into a
   large private implementation and compatibility layer?
6. Can a provider replacement occur without public caller migration, and what
   private migration work remains?

### Shared maintenance and ownership

1. Which Option A tasks disappear under Narrow B or Full B, and which ADR-0010
   tasks remain unchanged?
2. Does either candidate create duplicate semantic sources of truth or an API
   that must shadow upstream indefinitely?
3. Does the evidence support retaining A, admitting Narrow B, admitting Full B,
   combining a narrow semantic seam with the existing foreign types, or
   continuing incubation?

## Binding Experimental Boundaries

- Keep production on audited `glam` 0.29.3 throughout the study.
- Use isolated crates, modules, worktrees, or patches for candidate work.
- Do not move the production submodule, manifest, lockfile, or public exports.
- Keep the provider exact, local, pinned, and auditable under ADR-0010.
- Do not add provider features beyond the current `std`-only selection.
- Do not expose `glam::camera`, another new `glam` type, or provider traits in a
  candidate public signature.
- Do not use transparent layout, transmute, pointer casts, or ABI equivalence as
  the default wrapper conversion strategy. Any representation shortcut requires
  its own measured need and architectural review.
- Do not make Full B source-compatible with every `glam` API. Add only
  corpus-pressured operations with named callers.
- Keep camera lifecycle, active-camera selection, GPU uniform layout, WGPU clip
  conversion, chart identity, source embedding, and input policy outside the
  ordinary math wrapper.
- Keep Option C as an oracle/comparison. This plan does not reopen the retained
  A decision or authorize a C migration.
- A passing wrapper prototype is not stable admission.

## Candidate Models

```text
A — production control

caller -> public glam types and constructors -> pinned glam implementation


Narrow B — semantic seam

caller -> Tokimu camera/projection construction
       -> public existing glam value types
       -> private pinned glam mechanics


Full B — owned vocabulary, private provider

caller -> Tokimu Vec/Quat/Mat contracts
       -> explicit private provider boundary
       -> pinned glam implementation


C0/C1 — retained comparison only

caller -> Tokimu contracts -> Tokimu-owned mechanics
```

## Shared Evidence Matrix

Every candidate claim must name:

- exact provider revision and selected features;
- public vocabulary and provider references visible at the boundary;
- caller and operation represented;
- native, WASM-engine, and actual-browser status;
- numerical contract and tolerance;
- invalid/degenerate behavior;
- conversions and allocations per caller path;
- first/warm or cold/steady-state distinction where relevant;
- build profile, toolchain, target, adapter, and host;
- source edits and migration friction;
- failure and diagnostic behavior;
- update/remediation effort; and
- whether the observation is a stable guarantee, provisional evidence, or an
  unavailable target gap.

## Work Slices

### Slice 0: Freeze Controls And Start The Effort Ledger

#### Deliverables

- [x] Record the exact production A pin, reviewed 0.33.3 candidate, selected
      features, toolchain, host, target, and current AR-0019/AR-0029 status.
- [x] Snapshot the existing Alternative B wrapper source, tests, public surface,
      provider-reference count, migration artifacts, and known measurements.
- [x] Identify which earlier B results remain reproducible and which must be
      rerun after Doom, AR-0026, AR-0028, and Option A pressure.
- [x] Create a result ledger separating Narrow B, Full B, A, and C0/C1 evidence.
- [x] Start a level-of-effort ledger using the same actor, active time, wall
      time, recurring-cost, and blocked/rework distinctions as Option A.

#### Acceptance Criteria

- [x] No production dependency, public type, or stable contract changes merely
      to begin the experiment.
- [x] Inherited observations retain their original date, workload, provider,
      and limitations.
- [x] Narrow B and Full B cannot be conflated in later result summaries.

### Slice 1: Refresh Caller And Public-Boundary Pressure

#### Deliverables

- [x] Rescan stable crates and current corpus for all uses of the five public
      math types, associated constructors, fields, operators, traits, constants,
      conversions, collections, formatting, and serialization behavior.
- [x] Reconcile the 86 camera/projection sites from Option A by crate, caller,
      construction family, frequency, and semantic owner.
- [x] Update the operation inventory with Doom observer/collision/visibility,
      GLB, CAD, renderer, orientation, AR-0026 chart, and bulk-compute pressure.
- [x] Identify direct foreign type exposure in stable public signatures,
      associated types, trait bounds, examples, and generated documentation.
- [x] Ask for each current public type: what would Tokimu lose if the type
      disappeared entirely?
- [x] Separate operations required for source compatibility from operations
      required by actual Tokimu meaning.

#### Acceptance Criteria

- [x] Every Narrow-B function and Full-B operation considered later has a named
      real caller or explicit required conformance control.
- [x] `Vec2` and `Quat` are not expanded merely because Full B names five types.
- [x] Outward movement or removal remains visible when a concept lacks Native
      pressure.

### Slice 2: Define Provider-Neutral Semantic Contracts

#### Deliverables

- [x] Write the Narrow-B contract for right-handed look-at, GL-depth
      perspective, and GL-depth orthographic construction without naming
      `glam::camera`.
- [x] Define finite input requirements, degenerate eye/target/up behavior,
      invalid frustum behavior, error categories, and finite-result checks.
- [x] Define the minimal Full-B contract for each pressured type: values,
      operations, field/access behavior, constants, conversions, and failure
      semantics.
- [x] State handedness neutrality, matrix storage observations, multiplication
      order, vector convention, and which representation facts are explicitly
      not guaranteed.
- [x] Keep orientation-preserving/reversing spatial meaning, chart identity,
      qualified positions, and provider clip-depth adaptation in their existing
      semantic layers.
- [x] Define tolerances and independent scalar/reference checks before running
      provider differentials.

#### Acceptance Criteria

- [x] Contracts can be tested without importing a provider type in their public
      signatures.
- [x] Invalid-input behavior satisfies ADR-0009 rather than relying on provider
      panics or undocumented floating-point accidents.
- [x] The Full-B contract is smaller than “whatever `glam` currently exposes.”

### Slice 3: Harden Narrow B As Its Own Candidate

#### Deliverables

- [x] Extract or rebuild the AR-0029 prototype as an isolated Narrow-B candidate
      over both exact provider revisions.
- [x] Ensure the same caller-facing functions compile without source changes
      while the private provider implementation switches from 0.29.3 to 0.33.3.
- [x] Add deterministic success, finite-boundary, degenerate, and rejection
      tests for all three constructor families.
- [x] Verify the implementation constructs each intended matrix once and does
      not duplicate normalization, default construction, or provider work.
- [x] Scan for public `glam::camera` or provider-specific error leakage.
- [x] Determine the smallest existing crate/module placement that respects the
      SDD and renderer boundaries.

#### Acceptance Criteria

- [x] Provider revision changes do not require Narrow-B caller edits.
- [x] The seam remains exactly corpus-pressured construction and validation,
      not a general camera framework.
- [x] Placement does not move camera lifecycle or provider adaptation into
      `tokimu-core`.

### Slice 4: Harden Full B As A Distinct Candidate

#### Deliverables

- [x] Refresh the isolated provider-backed wrappers for the five retained names
      against the Slice 1 inventory and Slice 2 contracts.
- [x] Keep provider values private and expose only Tokimu-owned values,
      operations, errors, and documented semantics.
- [x] Add deterministic unit, fixed-seed property/metamorphic, degenerate, and
      independent-reference tests for every admitted operation.
- [x] Inventory all internal provider references and classify each as mechanics,
      conversion, validation, compatibility, or unnecessary duplication.
- [x] Test both provider revisions behind the same public wrapper harness.
- [x] Record missing ergonomics honestly: field mutation, indexing, traits,
      formatting, const construction, operators, and generic interoperability.
- [x] Reject conveniences that would merely recreate upstream API breadth.

#### Acceptance Criteria

- [x] No foreign type, trait, module, or error crosses the candidate boundary.
- [x] Every wrapper method has caller pressure and contract evidence.
- [x] Provider switching requires private changes only, while the amount of
      those changes remains measured rather than assumed negligible.
- [x] Full B does not claim implementation or supply-chain independence.

### Slice 5: Representative Migration And Conversion Accounting

#### Deliverables

- [x] Port representative camera, renderer, GLB, CAD, Doom observer/collision,
      picking, orientation, stereo, and AR-0026 chart controls to Narrow B where
      applicable and Full B separately.
- [x] Duplicate the same corpus test for A, Narrow B, Full B, and C where a
      direct comparison is meaningful.
- [x] Count source edits, changed signatures, accessor/setter substitutions,
      trait losses, explicit conversions, provider-boundary crossings, and
      temporary allocations per caller.
- [x] Exercise renderer upload/download and resource storage without treating
      observed wrapper layout as provider ABI.
- [x] Classify each conversion as necessary ownership boundary, avoidable
      impedance mismatch, or transitional artifact.
- [x] Retain at least one caller with frequent per-frame transforms and one with
      broad API/ergonomics pressure.

#### Acceptance Criteria

- [x] Migration evidence includes real application-shaped code, not only unit
      tests and microbenchmarks.
- [x] Narrow B and Full B source churn are reported separately.
- [x] A visible conversion is never hidden with unsafe layout equivalence solely
      to improve the benchmark.

### Slice 6: Native, WASM, Browser, And Representation Gate

#### Deliverables

- [x] Run the shared numerical/degenerate suite natively and in an actual WASM
      engine for A, Narrow B, and Full B.
- [x] Attempt retained actual-browser AR-0026 execution; record that no
      attachable browser surface was available without substituting Node/WASM.
- [x] Compare default WASM and `simd128`; compile or execute NEON and other
      available target paths without inferring unavailable results.
- [x] Retain size, alignment, copy, field/access, and column-array observations
      for wrappers and providers while distinguishing observation from contract.
- [x] Verify provider conversions preserve values and failure classifications
      across native/WASM.
- [x] Record unavailable NVIDIA, wasm64, NEON execution, or other target gaps.

#### Acceptance Criteria

- [x] Native and WASM semantic results agree within declared tolerances.
- [x] Actual browser evidence is distinguished from Node-hosted WASM and
      compile-only evidence.
- [x] Full B does not accidentally promise provider layout or SIMD identity.

### Slice 7: ADR-0008 Performance And Code-Quality Gate

#### Deliverables

- [x] Measure caller-shaped transform, inverse, stereo-camera, Doom observer,
      GLB, CAD, renderer handoff, and hot conversion paths for A, Narrow B,
      Full B, and relevant C controls.
- [x] Record cold/incremental compile time, output/binary size, allocations,
      throughput, and bounded tail observations on the same host/toolchain.
- [x] Separate wrapper method cost, provider conversion cost, caller repair,
      and complete workload cost.
- [x] Confirm steady-state hot paths do not allocate or perform redundant
      normalization/construction.
- [x] Inspect material regressions before proposing representation shortcuts,
      unsafe conversion, inlining policy, SIMD, caching, or API widening.
- [x] Run formatting and focused strict Clippy without suppressing candidate
      warnings against 0.33.3; retain 0.29.3's provider warning blocker.

#### Acceptance Criteria

- [x] No performance conclusion exceeds its measured caller and target.
- [x] Narrow B demonstrates whether semantic shock absorption is effectively
      free or carries recurring runtime cost.
- [x] Full B loses the GLB and affine-inverse workload gates because their
      material regressions remain unexplained; no broader equivalence claimed.
- [x] Code review finds no within-candidate duplicated function or shadow
      source of truth; duplication between Narrow and Full B is retained
      experimental-alternative isolation, not proposed production duplication.

### Slice 8: ADR-0009 And ADR-0011 Failure/Security Gate

#### Deliverables

- [x] Exercise malformed, non-finite, singular, zero-length, near-degenerate,
      overflow, and unsupported-operation cases at public candidate boundaries.
- [x] Verify errors retain bounded operation/input identity and do not leak
      provider-specific diagnostics as Tokimu contracts.
- [x] Confirm wrappers introduce no hidden global state, heap retention,
      provider lifetime ambiguity, thread authority, I/O, or host callbacks.
- [x] Compare panic behavior and failure containment across both provider pins,
      native, and WASM.
- [x] Re-run the Ring 0 dependency/provenance audit for the selected private
      provider closure.
- [x] State explicitly which supply-chain, unsafe/SIMD, advisory, and source
      review obligations remain identical to Option A.

#### Acceptance Criteria

- [x] Wrapper ownership does not become a security claim based on Tokimu naming.
- [x] Failure behavior is provider-neutral and bounded where Tokimu claims it.
- [x] No candidate weakens ADR-0010 provenance or update enforcement.

### Slice 9: Provider-Update Shock And Maintenance Economics

#### Deliverables

- [x] Replay the 0.29.3-to-0.33.3 change behind A, Narrow B, and Full B using
      the same immutable revisions and Option A evidence.
- [x] Count public caller edits, private adapter/wrapper edits, test changes,
      documentation changes, and review judgments for each candidate.
- [x] Identify which 86 camera/projection call sites are insulated by Narrow B
      and whether Full B provides additional real insulation.
- [x] Record the ADR-0010 work that remains unchanged: provenance, source delta,
      generated code, unsafe/SIMD, targets, advisories, licenses, rollback, and
      revision audit.
- [x] Simulate one ordinary new-operation request and one provider semantic/API
      change unrelated to camera construction.
- [x] Compare wrapper drift risk with direct foreign-vocabulary churn and C's
      owned-implementation maintenance burden.
- [x] Classify recurring versus one-time work and retain active/automation time
      without converting agent speed into proof of safety.

#### Acceptance Criteria

- [x] Shock absorption is demonstrated by unchanged callers, not asserted from
      the presence of a wrapper.
- [x] Full B must show benefit beyond Narrow B proportional to its larger API.
- [x] The report does not claim Option B removes foreign implementation audits.

### Slice 10: API Ergonomics, Documentation, And Ecosystem Pressure

#### Deliverables

- [x] Compare representative A, Narrow-B, and Full-B caller source for clarity,
      discoverability, diagnostics, documentation, and IDE behavior.
- [x] Test common generic Rust expectations such as `Copy`, `Clone`, `Debug`,
      `Default`, `PartialEq`, arithmetic traits, indexing, and conversions only
      where actual callers require them.
- [x] Identify friction with renderer, assets, serialization, ECS/world data,
      TypeScript lowering, external tools, and future provider crates.
- [x] Check whether Full B forces Tokimu to duplicate upstream documentation or
      teach users two nearly identical APIs.
- [x] Confirm Narrow B names semantic intent rather than provider mechanics.
- [x] Retain unsupported ergonomic requests instead of silently broadening the
      candidate.

#### Acceptance Criteria

- [x] Ergonomics evidence comes from real migrations and reviewable examples.
- [x] Full B remains learnable without becoming a disguised `glam` clone.
- [x] Narrow B improves semantic clarity at the pressured sites.

### Slice 11: Cross-Review With AR-0026, AR-0028, And Renderer Ownership

#### Deliverables

- [x] Run the same chart-transition and orientation-preserving/reversing control
      over A, Narrow B, Full B, and C0 where applicable.
- [x] Verify chart identity, qualified location, transition intent, source
      embedding, camera basis, and input policy remain above ordinary math.
- [x] Verify WGPU `[0, 1]` clip conversion remains provider-private while
      Tokimu camera construction retains GL `[-1, 1]` meaning.
- [x] Test whether portals, recursive views, stereo, or multiple view instances
      create pressure for a different camera semantic seam without expanding
      Full B's raw type contract.
- [x] Record any operation growth requested by the cross-review in both the B
      and C evidence ledgers.

#### Acceptance Criteria

- [x] Exotic spatial semantics do not become wrapper methods by convenience.
- [x] A broader camera/view ownership question returns to AR-0029 rather than
      silently widening Narrow B.
- [x] Full B remains ordinary math vocabulary, not Tokimu's entire spatial model.

### Slice 12: Decision Matrix And Maintainer Gate

#### Deliverables

- [x] Produce a matrix comparing A, Narrow B, Full B, and retained C0/C1 for
      semantic ownership, public coupling, implementation trust, update shock,
      caller migration, conversions, performance, targets, failure behavior,
      API size, and lifetime maintenance.
- [x] Recommend one bounded disposition:
  - retain A with no B admission;
  - admit Narrow B only;
  - continue Narrow B incubation;
  - admit Full B;
  - continue Full B incubation; or
  - reopen the broader AR-0019 decision because evidence invalidates the
    existing alternatives.
- [x] State whether AR-0029 should become an ADR, remain review guidance, or be
      closed with no stable change.
- [x] If a B candidate is selected, write a separate production migration plan
      covering compatibility, semver, crate placement, rollout, rollback,
      documentation, and provider pin handling. No candidate is selected by
      this recommendation, so no migration plan is authorized.
- [x] If no B candidate is selected, retain the executable corpus and explain
      which future pressure would reopen it.
- [x] Obtain explicit maintainer disposition: retain A in production, continue
      Narrow B only as incubation evidence, park Full B, and make no
      stable/public change.

#### Acceptance Criteria

- [x] The recommendation follows comparative evidence rather than architectural
      cleanliness, wrapper familiarity, or frustration with provider updates.
- [x] Narrow B remains visible as a complete outcome rather than a stepping
      stone that must grow into Full B.
- [x] Retaining A remains a valid outcome throughout the study.
- [x] The production workspace still has one stable math vocabulary when this
      study concludes.

## Required Workload Ladder

Use the same fixed inputs for all applicable candidates:

1. unit-level vector/matrix/quaternion contracts;
2. 100, 10,000, and 100,000 repeated transforms;
3. finite, singular, near-degenerate, and non-finite inputs;
4. view plus perspective and orthographic construction;
5. stereo camera construction;
6. renderer camera upload and frame preparation;
7. GLB node transforms and Khronos Box pressure;
8. CAD ray/unprojection pressure;
9. Doom observer, collision, orientation, and visibility preparation;
10. AR-0026 chart transition and orientation classification; and
11. native, Node-WASM, `simd128`, and actual-browser controls where available.

Additional workloads may be added only with a named caller and must not replace
the fixed comparison ladder.

## Planned Evidence Artifacts

- `corpus/lib/tokimu-math-study/results/<date>-option-b-control.md`
- `corpus/lib/tokimu-math-study/results/<date>-option-b-pressure-scan.md`
- `corpus/lib/tokimu-math-study/results/<date>-option-b-narrow-contract.md`
- `corpus/lib/tokimu-math-study/results/<date>-option-b-full-contract.md`
- `corpus/lib/tokimu-math-study/results/<date>-option-b-migration-accounting.md`
- `corpus/lib/tokimu-math-study/results/<date>-option-b-target-evidence.md`
- `corpus/lib/tokimu-math-study/results/<date>-option-b-performance.md`
- `corpus/lib/tokimu-math-study/results/<date>-option-b-update-economics.md`
- `corpus/lib/tokimu-math-study/results/<date>-option-b-spatial-cross-review.md`
- `corpus/lib/tokimu-math-study/results/<date>-option-b-decision-matrix.md`
- AR-0019 and AR-0029 review-cycle updates
- a separate migration plan or ADR only if maintainers select a stable candidate

## Validation Direction

Use the narrowest relevant commands first, then expand without hiding unrelated
baseline failures. Expected gates include:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p tokimu-math-study
cargo test --manifest-path corpus/lib/tokimu-math-study/alternative-b-provider-backed/Cargo.toml
cargo build/check for wasm32-unknown-unknown
actual browser execution of retained camera/orientation fixtures
Ring 0 dependency audit with the exact selected provider pin
```

When full-workspace gates fail for a reproduced unrelated baseline, retain the
exact first failure and continue with focused candidate gates. Do not describe
the workspace as clean until the complete gate actually passes.

## Stop And Escalation Conditions

Return to maintainers before continuing if:

- a candidate requires a stable/public production API change;
- Narrow B expands beyond independently demonstrated semantic construction;
- Full B requires broad API mirroring, unsafe layout equivalence, or a new ABI
  guarantee to remain viable;
- a provider type, trait, error, or module must cross the candidate boundary;
- camera lifecycle, active selection, renderer resources, chart identity,
  source orientation, or input policy begins moving into ordinary math;
- native/WASM semantics diverge materially;
- a material performance or conversion regression cannot be repaired within
  the selected contract;
- provider switching requires public caller changes despite the claimed seam;
- provenance, security, licensing, or selected dependency closure changes;
- a new abstraction, crate, service, or provider contract appears necessary;
  or
- evidence materially reopens the retained A versus C decision.

## Completion And Parking Criteria

The study is complete when it records one evidence-bearing disposition from
Slice 12, preserves all unavailable target and baseline gaps, and either:

1. produces a separately authorized migration/ADR plan for a selected B
   candidate; or
2. parks both B candidates with explicit reopening pressure while production A
   remains unchanged.

The study may pause earlier when a stop condition requires architectural
judgment. A pause must name the exact evidence, risk, owner, and resumption
condition. It must not leave a partially migrated production vocabulary.

## References

- `docs/Architectural Reviews/AR-0019-native-math-vocabulary-and-foreign-type-boundary.md`
- `docs/Architectural Reviews/AR-0029-camera-view-and-projection-construction-ownership.md`
- `docs/ADR/Proposed/ADR-XXXX-tokimu-owned-semantic-operations-over-admitted-mechanical-values.md`
- `docs/Architectural Reviews/AR-0026-non-euclidean-spatial-charts-and-authored-angular-topology.md`
- `docs/Architectural Reviews/AR-0028-coordinate-frame-handedness-and-directional-conformance.md`
- `docs/Plans/Native-Math/Studies/ar-0019-option-a-glam-current-release-update.md`
- `docs/Plans/Native-Math/Studies/ar-0019-option-c-owned-math-and-bulk-compute.md`
- `docs/Dependency Audits/Ring 0/glam-d36e7eeff05338c56c4aa8d59fc2615e7963b1b7.md`
- `docs/Dependency Audits/Ring 0/glam-9928729066db87d97fa779e129469721a289beae.md`
- `corpus/lib/tokimu-math-study/alternative-b-provider-backed/README.md`
- `corpus/lib/tokimu-math-study/decision-matrix.md`
- `corpus/lib/tokimu-math-study/migration-accounting.md`
- `docs/Tokimu Software Design Document.md`
