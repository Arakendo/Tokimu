# AR-0019 Option C Owned Math And Bulk Compute Exploration

| Field | Value |
| --- | --- |
| Status | Maintainer disposition: retain A; C0/C1 stays corpus-local incubation; no production migration authorized |
| Owner | Tokimu maintainers |
| Date | 2026-08-12 |
| Related review | `AR-0019-native-math-vocabulary-and-foreign-type-boundary.md` |
| Related reviews | AR-0015, AR-0025, AR-0026, AR-0028 |
| Related ADRs | ADR-0003, ADR-0005, ADR-0008, ADR-0009, ADR-0010, ADR-0011 |
| Prior study | `docs/Plans/Native-Math/native-math-vocabulary-foreign-type-case-study.md` |
| Candidate root | `corpus/lib/tokimu-math-study/` |

## Purpose

Run a second-stage investigation of AR-0019 Alternative C: an original,
Tokimu-owned implementation of only the ordinary CPU math mechanics earned by
real callers.

The study was reopened by concrete lifecycle pressure. The audited `glam`
0.29.3 source produces compiler-warning noise, while the first upstream release
containing the repair also contains a large foreign Ring 0 source delta. Most
software could reasonably take that update after ordinary dependency review;
Tokimu's ADR-0010 policy deliberately requires stronger evidence because the
source executes inside Ring 0. This plan tests whether that cost is:

1. proportionate and acceptable for the current foreign provider;
2. reduced in practice by a narrow Tokimu-owned implementation; or
3. merely exchanged for greater numerical, portability, and optimization risk.

The maintainer discussion also identified possible GPU acceleration for large
CAD, point-cloud, spatial-classification, or simulation workloads. That is a
separate hypothesis from Option C's ordinary math mechanics. The study will
therefore keep two boundaries explicit:

```text
Ring 0 ordinary math
    Tokimu-owned CPU-friendly values and bounded mechanics
    no device objects
    no provider boundary
    no required GPU

Outer Ring bulk compute experiment
    caller-owned batch operation and authoritative inputs
        ↓
    CPU reference/fallback
        + optional WGPU compute realization
          for demonstrated large workloads
```

The experiment must not make `Vec3` GPU-resident, turn HIP/Vulkan/WGPU into
ordinary math providers, or let device output become authoritative simulation
truth merely because acceleration is available.

## Questions To Answer

### Option C ownership

- Does the current post-DOOM caller set still fit a deliberately small owned
  math implementation?
- Which of `Vec2`, `Vec3`, `Vec4`, `Quat`, and `Mat4` are actually earned now?
- Can Tokimu choose explicit numerical and degenerate-input behavior rather
  than inheriting it accidentally from the oracle?
- Is scalar, safe Rust sufficient on native and WASM for current hot paths?
- Does removing the foreign math closure materially improve build hygiene,
  auditability, update response, and supply-chain containment?
- Is that improvement worth Tokimu owning correctness, performance,
  portability, testing, and maintenance indefinitely?

### AR-0026 complexity pressure

AR-0026 is a two-sided complexity test, not an argument that unusual spatial
semantics automatically require Tokimu to own all mathematics:

```text
charted spatial meaning grows
while ordinary numerical mechanics stay bounded
    -> evidence for a small C0 beneath Tokimu-owned semantic wrappers

charted spatial work requires a broad operation set,
specialized robustness, or extensive SIMD machinery
    -> evidence against the small-owned-subset hypothesis
```

The preferred experimental separation is:

```text
C0 ordinary mechanics
    point/direction transforms, composition, checked inverse,
    determinant/orientation mechanics, bounded rotation/projection
        ↓
Tokimu spatial semantics
    ChartId, qualified location, transition identity,
    orientation-preserving/reversing meaning, query transport
```

AR-0028 already demonstrates why this separation matters: raw inversion can
answer whether a matrix has an inverse, but cannot state the source/chart
frame or whether orientation reversal is intended. The future junction corpus
should therefore run the same semantic chart layer over A and C0. It must
compare traversal traces, transition composition, orientation classification,
native/WASM behavior, and operation growth. It may strengthen C only if C0
survives without absorbing chart identity or growing toward broad provider
compatibility.

### Update-policy proportionality

- What work would an ordinary `glam` update require under ADR-0010 at each
  measured source-delta size and risk class?
- Does the current warning repair expose pathological stagnation, or is
  retaining the audited pin a reasonable bounded choice?
- How would Tokimu respond differently to a warning-only update, correctness
  fix, target regression, security advisory, or critical vulnerability?
- Would a selected Option C actually reduce time-to-safe-remediation, or move
  the same work into Tokimu's own numerical implementation?

### Bulk compute

- Which operations are genuinely batch-shaped rather than ordinary scalar or
  structural math?
- At what batch size, residency pattern, and result-consumption pattern does a
  GPU realization outperform the CPU reference?
- Can WGPU provide useful native/browser parity before specialized native
  APIs such as HIP or raw Vulkan are considered?
- Can the provider fail, be unavailable, or be slower without changing the
  operation's semantic result or leaving authoritative state ambiguous?
- Do Doom and an independent CAD/point-cloud case reveal common provider-
  neutral bulk-operation meaning, or only domain-specific mechanisms?

## Binding Experimental Boundaries

- The current `tokimu_core::math` re-export remains the production control.
- No stable public API, dependency disposition, or ADR is changed by this plan.
- Option C remains corpus-local until AR-0019 records a new disposition and a
  separate migration plan passes ADR-0008 through ADR-0011.
- `glam` remains a pinned correctness and performance oracle during the study;
  oracle use is not evidence that its behavior is automatically Tokimu policy.
- The owned candidate may not copy or mechanically translate foreign source.
  Any derived implementation belongs to Alternative D and its provenance
  obligations, not Option C.
- Ordinary math begins as allocation-free, safe scalar Rust. SIMD or unsafe
  code requires a measured deficit, a bounded invariant, target evidence, and
  a separate gate decision.
- GPU compute remains an Outer Ring experiment. No GPU, renderer, platform,
  WGPU, Vulkan, HIP, browser, or device type may enter the Option C value layer.
- CPU remains the universal executable reference and fallback for every bulk
  operation admitted to this study.
- Bulk providers consume bounded caller-owned inputs and return observations or
  results through explicit synchronization. They do not own CAD, Doom, world,
  resource, visibility, or simulation truth.
- A successful synthetic benchmark cannot by itself admit a general compute
  capability. At least one real caller-shaped workload and one independent
  pressure source are required.
- Exact B-rep booleans, topology healing, constraint solving, feature-history
  regeneration, robust-predicate policy, and other irregular CAD semantics are
  outside scope unless a later plan decomposes a bounded numerical kernel.
- NVIDIA results may remain unavailable on maintainer-owned hardware. Any
  target/device matrix must state that gap rather than infer NVIDIA parity from
  AMD Vulkan/WGPU or Apple Metal observations.

## Candidate Model

| Candidate | Role | Ring interpretation |
| --- | --- | --- |
| A — pinned `glam` | Current production and oracle control | Audited foreign implementation executing in Ring 0 |
| C0 — owned scalar | Original safe scalar implementation of measured mechanics | Candidate Ring 0 implementation |
| C1 — owned optimized | Optional later CPU optimization of C0, only if earned | Candidate Ring 0 implementation under the full gate |
| CPU bulk reference | Deterministic/bounded batch implementation over owned or control values | Corpus-local reference; ownership unresolved |
| WGPU bulk candidate | Native/browser compute realization of the same batch case | Outer Ring provider experiment |
| Specialized native candidate | HIP or raw Vulkan only after WGPU evidence exposes a named deficit | Deferred Outer Ring provider experiment |

C0 and the CPU bulk reference are not automatically the same abstraction.
Ordinary types and operators may belong to Ring 0 while a high-volume batch
operation remains domain-owned or Outer Ring capability machinery.

## Shared Evidence Matrix

| Area | Required evidence |
| --- | --- |
| Caller pressure | Named caller, operation, frequency, data volume, semantic owner, and whether the operation is ordinary or bulk |
| Numerical contract | Coordinate convention, tolerances, finite/degenerate behavior, rejection behavior, determinism, and error meaning |
| Correctness | Fixed examples, property/metamorphic cases, differential oracle results, and independent reference where practical |
| Performance | Workload, target, profile, toolchain, host/device metadata, warm/cold state, allocations, copies, latency, and throughput |
| Portability | Native CPU, WASM CPU, browser execution, and provider-specific gaps; build-only evidence labeled separately |
| Maintenance | Source/test/documentation size, unsafe/SIMD/generated code, review surface, toolchain response, and operation-growth rate |
| Security | Foreign executable closure, build/proc-macro surface, unsafe/device boundary, resource bounds, and failure containment |
| Migration | Stable caller edits, conversion sites, compatibility period, rollback, and whether one vocabulary remains authoritative |
| Provider behavior | Availability, initialization, synchronization, cancellation, device loss, unsupported path, and CPU fallback evidence |
| Decision value | What result would retain A, advance C, reject C, admit a separate study, or leave the issue incubating |

## Slices

### Slice 0: Freeze The Second-Stage Control

#### Deliverables

- [x] Record the exact audited `glam` revision, enabled features, current Rust
      toolchain, warning condition, and upstream warning-fix revision.
- [x] Preserve the measured 0.29.3-to-0.30.2 source delta as lifecycle evidence,
      not as a claim that every changed line is security-sensitive.
- [x] Snapshot C's current source, tests, operations, known native/WASM results,
      and conditional-viability finding.
- [x] Confirm that this plan reuses the existing Alternative C source rather
      than creating a parallel owned-math implementation.
- [x] Add a result ledger that distinguishes inherited evidence from new
      post-DOOM evidence.

#### Acceptance Criteria

- [x] A and C can be reproduced from pinned source without network resolution.
- [x] No production dependency or stable type changes merely to start the test.
- [x] Every inherited result names its original workload and date.

### Slice 1: Post-DOOM Operation And Boundary Rescan

#### Deliverables

- [x] Rescan stable crates and the current Doom, renderer, GLB, CAD, camera,
      collision, picking, animation, and spatial-orientation corpus paths for
      direct math operations and representation assumptions.
- [x] Update the operation manifest with named callers, frequency class, target
      use, mutation/access shape, and hot-path status.
- [x] Separate ordinary mechanics from semantic spatial types demonstrated by
      AR-0026 and AR-0028; do not make `Vec3` infer chart, frame, handedness, or
      orientation-preserving intent.
- [x] Identify whether `Vec2` or `Quat` now has real direct pressure. Leave each
      absent from C if its use is still speculative.
- [x] Inventory public `glam` exposure, provider conversion sites, layout/ABI
      assumptions, serialization/reflection use, and TypeScript-facing pressure.
- [x] Classify each candidate batch operation separately from ordinary math.

#### Acceptance Criteria

- [x] Every proposed C operation is traced to a real caller or a required
      invariant supporting one.
- [x] No API is added solely for source compatibility with `glam`.
- [x] The manifest shows which Doom pressure is provisional because the Doom
      checklist remains incomplete.

### Slice 2: Select Tokimu Numerical Contracts

#### Deliverables

- [x] Decide and document finite-input preconditions and behavior for zero
      normalization, singular inversion, degenerate `look_at`, invalid
      projection parameters, division by zero, NaN, and infinity.
- [x] Separate programmer-contract violations, recoverable invalid data, and
      IEEE floating-point propagation rather than using one all-NaN fallback
      accidentally.
- [x] State matrix storage, multiplication order, handedness neutrality,
      projection conventions, and representation non-guarantees.
- [x] Define comparison tolerances per operation and magnitude range.
- [x] Record which current `glam` observations are adopted, intentionally
      changed, or left unspecified.

#### Acceptance Criteria

- [x] The contract can be tested without naming a provider implementation.
- [x] Failure behavior satisfies the applicable ADR-0009 containment gate.
- [x] Spatial frame meaning remains above ordinary math mechanics.

### Slice 3: Harden The Owned Scalar Candidate

#### Progress Refinements

- [x] Add the post-DOOM vector axes, length, and explicit ordered accumulation
      without copying the provider's `Sum` surface.
- [x] Add checked normalization, view/projection construction, inversion, and
      perspective-dividing projection while retaining unchecked comparison
      paths separately.
- [x] Add explicit nested column conversion for the current renderer handoff.
- [x] Exercise the selected combined absolute/relative comparator and classify
      finite A agreement separately from intentional checked-contract
      divergence.
- [x] Add a bounded fixed-seed 128-case vector/affine property loop.
- [x] Add a reusable generated-case or fuzzable entry with bounded inputs.
- [x] Add an independently expressed scalar reference for operations where A
      and C could plausibly share the same mistake.
- [x] Retain a dedicated zero-allocation observation rather than relying only
      on source inspection.
- [x] Expand conditioning and near-degenerate cases, then confirm or revise the
      provisional inverse residual bound.

#### Deliverables

- [x] Extend the existing C implementation only to the refreshed manifest.
- [x] Keep the candidate dependency-free, allocation-free for ordinary value
      operations, safe Rust, and free of generated code.
- [x] Add deterministic unit tests for each selected contract and rejection
      behavior.
- [x] Add fixed-seed property/metamorphic tests for vector identities, matrix
      composition, inverse round trips within conditioning limits, and
      transform/orientation properties.
- [x] Differentially compare with pinned A and, where useful, an independently
      expressed scalar reference; classify mismatches instead of automatically
      making A correct.
- [x] Add fuzzable entry points or bounded generated cases for numerical edge
      pressure without accepting unbounded test inputs.

#### Acceptance Criteria

- [x] C contains only earned types and operations.
- [x] Every selected operation has success, boundary, and degenerate evidence;
      the result remains bounded C0 evidence rather than a stable API claim.
- [x] No provider type or implementation detail enters C's public candidate
      surface.
- [x] Ordinary value operations allocate zero times in the retained isolated
      host test; target-wide allocation behavior remains Slice 4 evidence.

### Slice 4: Native And WASM Correctness And Representation

#### Deliverables

- [x] Run the selected bounded C0 semantic suite natively and in an actual WASM engine;
      distinguish browser execution from compile-only target evidence.
- [x] Retain size, alignment, copy, field-access, and column-array observations
      without promising ABI, POD, FFI, or serialization guarantees prematurely.
- [x] Exercise explicit renderer/provider handoff conversions and count them.
- [x] Exercise little-used targets or target features where practical; record
      unavailable targets honestly.
- [x] Verify that scalar C requires no target-only code path for correctness.

#### Acceptance Criteria

- [x] Native and WASM agree within the selected bounded semantic outcomes.
- [x] A representation observation is not described as a stable representation
      contract without separate evidence.
- [x] Target differences have a named cause and disposition for the two
      available targets; additional targets remain unavailable evidence.

### Slice 5: Option C Performance And Optimization Gate

#### Deliverables

- [x] Retain the representative native A/B/C transform, stereo-camera,
      CAD/GLB inverse, and Doom-observer controls completed in this study.
      Full production-caller replacement/replay is deliberately deferred: A
      remains the stable vocabulary, and performing it would create an
      unauthorized second stable vocabulary or migration. Reopen only with an
      approved migration decision and a compatibility/rollback plan.
- [x] Record cold build, incremental build, binary/output size, allocations,
      copies, throughput, and tail latency with target/profile/toolchain facts.
      Bounded isolated-control observations are retained; they are explicitly
      not a full-engine size or universal build-time claim.
- [x] Separate operator microbenchmarks from complete caller-shaped work.
      The retained result labels transform, stereo-camera, and boundary-only
      conversion scopes separately.
- [x] Identify any material C0 regression and locate it before proposing SIMD,
      unsafe, layout tricks, or provider delegation. C0's generic
      Gauss--Jordan inverse is a material repeated affine/CAD regression;
      it was isolated before changing the candidate.
- [x] If a regression matters, prototype the smallest C1 optimization behind
      the corpus boundary and apply ADR-0008/0009/0011 evidence to it.
      Refinement: C1 is safe scalar affine inversion with an exact affine
      classifier; non-affine and checked paths retain C0. It recovers the
      pinned GLB control but intentionally leaves CAD projection/picking open.
- [x] Retain the scalar path as the portability/reference control even if C1
      is successful. C0 remains the checked and non-affine fallback.

#### Acceptance Criteria

- [x] No performance claim is universalized beyond its measured workload.
- [x] C1 exists only in response to a measured decision-relevant deficit.
- [x] No unsafe or target-SIMD path was introduced or needed by measured C0/C1
      deficits. Any later path must first provide an explicit invariant,
      scalar-equivalence test, unsupported-target behavior, and maintenance
      estimate under ADR-0008/0009/0011; it is not authorized by this study.

### Slice 6: Ring 0 Update And Remediation Economics

#### Deliverables

- [x] Model at least four provider-update cases: warning/toolchain cleanup,
      bounded correctness fix, target regression, and critical security fix.
- [x] For A, record discovery, diff review, provenance, feature/unsafe/SIMD
      review, conformance, target validation, rollback, and time-sensitive
      exception steps required by ADR-0010/0005.
- [x] For C, record the equivalent diagnosis, implementation, oracle comparison,
      security review, regression, target validation, and rollback steps.
- [x] Compare recurring broad-provider audit cost with C's operation-by-operation
      maintenance and ownership cost.
- [x] Define a stagnation warning: repeated inability to take important fixes
      safely must reopen either ADR-0010 proportionality or Ring 0 foreign-code
      volume; it must not silently weaken the gate.
- [x] State which update classes can use a lighter documented review without
      changing ADR-0010 and which require an ADR revision or ADR-0005 exception.

#### Acceptance Criteria

- [x] The comparison measures responsibilities, not only line counts.
- [x] Self-authorship is never counted as correctness or security proof.
- [x] The result can conclude that A remains lower risk despite its audit cost.
- [x] Any recommendation to adjust ADR-0010 is separate from selecting C.

### Slice 7: Classify Candidate Bulk Operations

#### Deliverables

- [x] Inventory caller-shaped bulk candidates such as point transforms, AABB
      generation, frustum/volume classification, section-plane classification,
      point-cloud filtering, broad-phase pairs, and bounded surface sampling.
- [x] For each candidate, identify semantic owner, authoritative input/output,
      batch size, independence, branching, residency, precision, ordering,
      synchronization, and result-consumption needs.
- [x] Reject ordinary per-object vector/matrix work from GPU consideration.
- [x] Reject or defer irregular/high-authority CAD operations whose topology or
      robust-predicate semantics cannot be represented as a bounded numerical
      kernel.
- [x] Select at most two operations for implementation: one spatial candidate
      workload related to AR-0025 and one independent CAD/point-cloud workload.

#### Acceptance Criteria

- [x] Selected operations have a CPU-executable semantic reference defined for
      Slice 8 implementation.
- [x] Doom does not define generic CAD or compute meaning.
- [x] The independent workload is useful outside a game renderer.
- [x] Unsupported/provider-unavailable behavior is explicit before GPU work.

### Slice 8: CPU Bulk Reference And Scaling Controls

#### Deliverables

- [x] Implement corpus-local CPU reference cases with stable input generation,
      identity-preserving results, bounded memory, and deterministic checksums.
- [x] Exercise sizes spanning dispatch-hostile through plausibly GPU-worthy
      work, for example `1K`, `10K`, `100K`, `1M`, and a memory-safe large case.
- [x] Measure single-use upload-like inputs separately from persistently resident
      and repeatedly queried inputs.
- [x] Separate CPU full-result, compacted-result, and count-only observations;
      retain GPU-consumed-next-stage as an explicit Slice 9 provider question,
      not a CPU-invented synchronization model.
- [x] Include E1M1-scale data as a small negative/control workload, not as a
      reason to claim GPU acceleration.

#### Acceptance Criteria

- [x] The reference remains useful without a GPU or window.
- [x] Identity, ordering, precision, and rejection semantics are asserted.
- [x] Work and memory bounds prevent a benchmark from exhausting the host.

### Slice 9: WGPU Native And Browser Compute Candidate

#### Deliverables

- [x] Prototype WGPU compute only for the selected Slice 7 operations and keep
      shader/binding vocabulary corpus-local.
      - [x] Native ordered-point and ordered-AABB controls use separate local
            WGSL/buffer layouts and compare every ordered flag with Slice 8 CPU
            output.
      - [x] Browser ordered-point execution completed on the DOM-hosted fixture
            with exact CPU/GPU result agreement; native results still do not
            stand in for other browser/adapters.
- [x] Exercise native and browser WebGPU paths using the same ordered-point
      operation meaning and fixed input fixture. Native AABB remains additional
      native-only evidence rather than an inferred browser capability.
- [x] Measure initialization, allocation, upload, dispatch, synchronization,
      readback, warm reuse, and total caller-observed latency separately.
      - [x] Native controls retain cold adapter/device/setup and median-of-three
            warm upload/dispatch/readback values.
      - [x] Browser execution retains cold adapter/device/setup-allocation,
            three same-provider samples, and median warm
            upload/dispatch/readback timing (2026-08-12 DOM observation).
- [x] Retain available backend, adapter, device class, host, build profile, and
      workload metadata with every result. The browser observation retains its
      empty adapter string rather than inventing one; it has no canvas because
      this is a DOM compute host.
- [x] Test provider unavailable, initialization failure, device loss or bounded
      substitute where practical, invalid input, cancellation/disposal, and CPU
      fallback without double commit. Refinement: creation-time validation is
      now locally scoped to prevent an uncaught WGPU abort; the intentional
      invalid-WGSL control returns `provider-validation-rejected` on native and
      the actual browser DOM host; a
      caller-selected CPU-bypass control emits exactly one observation without
      WGPU acquisition; the browser invalid-count fixture is implemented;
      an idle browser-buffer-disposal fixture is implemented; actual
      unavailable, loss, and in-flight cancellation remain explicitly parked
      because the available providers cannot safely manufacture them.
- [x] Verify that GPU results remain observations/mechanics and do not become
      authoritative CAD or simulation state without caller validation/commit.
      The native and DOM fixtures retain ordered CPU-versus-GPU comparisons
      locally, expose no Tokimu compute API, and publish no world/CAD state.

#### Acceptance Criteria

- [x] Native CPU and WGPU results agree under the selected unit-cube control for
      ordered point and AABB workloads.
- [x] Warm-frame improvement is not reported without cold/startup cost: native
      and browser warm figures retain cold lifecycle/setup cost alongside their
      three-sample reused-provider medians.
- [x] Browser evidence is execution evidence, not merely a WASM build.
- [x] Provider failure leaves the caller able to run or select the CPU path.
- [x] Missing NVIDIA coverage is explicitly retained if still unavailable in
      `docs/lessions/gpu-adapter-validation-coverage.md`; no Slice 9 result is
      generalized from the available AMD/Vulkan device.

### Slice 10: Specialized Native Compute Decision Gate

#### Deliverables

- [x] Compare WGPU results with the decision need before adding another
      provider/toolchain. The retained Slice 8/9 comparison finds no named
      WGPU deficit; see `results/2026-08-12-option-c-slice-10-specialized-provider-gate.md`.
- [x] Consider HIP, raw Vulkan compute, CUDA, or another specialized provider
      only if a named native workload shows a material WGPU deficit and the
      maintainer has suitable hardware/tooling evidence. Neither exists.
- [x] Record added unsafe/FFI/toolchain/deployment/provenance/security burden.
- [x] Keep specialized results comparable to the CPU reference and WGPU case.

#### Acceptance Criteria

- [x] This slice closes as `not earned`; that is a complete result.
- [x] No specialized provider is admitted merely because it is theoretically
      faster or available on one maintainer machine.
- [x] A provider-specific gain is not generalized to native or GPU execution.

### Slice 11: Cross-Review Synthesis

#### Deliverables

- [x] Feed the spatial batch findings back to AR-0025 without making Doom BSP,
      SEG, portal, or clip semantics generic renderer vocabulary.
- [x] Feed framed/orientation findings back to AR-0026/0028 as semantic types
      above ordinary vectors and matrices.
- [x] Run the first bounded AR-0026 chart/junction semantic layer over A and C0
      without changing that layer's chart/transition meaning between runs.
      - [x] Native A/C trace, composition, inverse, and orientation controls
            agree; retained in `results/2026-08-12-option-c-slice-11-chart-cross-review.md`.
      - [x] The DOM-hosted WASM control compiles and generated bindings are
            refreshed.
      - [x] Actual DOM/WASM execution retains matching `2520c9de` fingerprint
            with `provider=none`.
- [x] Compare traversal traces, transition composition, orientation-preserving
      and orientation-reversing classification, native/WASM observations, and
      any new ordinary-math operations requested by that corpus.
- [x] Treat bounded operation growth as evidence for C and broad/numerically
      exotic growth as evidence against C; do not count semantic-wrapper
      richness itself as math-library growth.
- [x] Record whether CAD supplies independent pressure for conservative
      candidate identity, query domains, provider guarantees, and rejection
      evidence.
- [x] Separate any earned general bulk operation from its CPU/WGPU mechanisms.
- [x] Update AR-0019 with the lifecycle, correctness, performance, and
      maintenance comparison while leaving ADR-0010 unchanged unless its own
      evidence requires revision.

#### Acceptance Criteria

- [x] No review receives a broader conclusion than its evidence supports.
- [x] Option C selection and bulk-compute admission remain separate decisions.
- [x] AR-0026 pressure is reported in both directions rather than used as a
      one-way justification for ownership.
- [x] A provider boundary begins outside Ring 0 if the no-provider Ring 0
      hypothesis remains under test.

### Slice 12: Decision And Next Artifact

#### Deliverables

- [x] Produce a decision matrix for A versus C0/C1 covering trust surface,
      update response, numerical confidence, native/WASM behavior, performance,
      maintenance, migration, and rollback.
- [x] Produce a separate CPU/WGPU bulk-compute matrix covering usefulness,
      crossover points, target reach, failure behavior, and ownership.
- [x] Recommend one AR-0019 disposition: retain A, continue C incubation,
      select C for a separately planned migration, or reject C.
- [x] Recommend one compute disposition: no capability, retain corpus evidence,
      open a dedicated Architectural Review, or plan a bounded capability.
- [x] If C is selected, write a new migration plan with compatibility,
      deprecation, rollback, and one-stable-vocabulary closeout. Do not perform
      that migration inside this plan. C is not selected, so no migration plan
      is authorized or required.
- [x] If compute is earned, open an independent review before stabilizing an
      operation, provider, scheduling, buffer, or shader/resource contract.
      Compute is not earned, so no review is opened.

#### Acceptance Criteria

- [x] Warning irritation, benchmark novelty, or architectural cleanliness alone
      cannot select C; the recommendation retains A rather than selecting C.
- [x] A remains visible as a valid outcome throughout the decision and remains
      the stable production vocabulary.
- [x] The production workspace retains one stable math vocabulary; all C work
      remains corpus-local and no migration is authorized.
- [x] AR-0019, ADR-0010, the dependency audit, SDD, and implementation remain
      aligned because this study changes no stable dependency or API. Any later
      migration must revalidate this criterion.

## Required Workload Ladder

| Workload | Purpose | Expected decision value |
| --- | --- | --- |
| Ordinary camera/transform operations | Test C as everyday CPU math | Must remain cheap; GPU is ineligible |
| Current E1M1 candidate sets | Small bulk negative/control | Likely shows dispatch overhead dominates |
| Synthetic identified AABBs at increasing counts | Locate CPU/WGPU crossover without domain ambiguity | Performance mechanism evidence only |
| Large CAD assembly-shaped bounds | Independent conservative-candidate pressure | Tests whether a reusable operation exists |
| Point-cloud transform/classification | High-volume, parallel, non-game use | Strong candidate for WGPU native/browser comparison |
| Irregular exact CAD operation | Control showing what is not a bulk kernel | Prevents `CAD → GPU` overgeneralization |

## Evidence Artifacts

The study should retain, at minimum:

- refreshed operation and public-boundary inventories;
- selected numerical contract and mismatch ledger;
- native/WASM correctness results;
- property/fuzz seeds and minimized regressions;
- A/C source, test, unsafe, generated-code, and dependency-closure accounting;
- provider-update/remediation scenario comparison;
- caller-shaped performance reports with machine/target metadata;
- bulk-operation classification and rejection ledger;
- CPU/WGPU crossover reports, including cold/warm and residency distinctions;
- provider failure/fallback evidence;
- NVIDIA or other unavailable-target gaps;
- final A/C and CPU/WGPU decision matrices.

## Validation Direction

Use focused commands first and expand only when a slice changes shared code.
The concrete runners may evolve, but the evidence should include equivalents of:

```powershell
cargo fmt --all --check
cargo clippy -p tokimu-math-study --all-targets -- -D warnings
cargo test -p tokimu-math-study --locked --offline
cargo test --manifest-path corpus/lib/tokimu-math-study/alternative-c-owned-subset/Cargo.toml --locked --offline
cargo build --manifest-path corpus/lib/tokimu-math-study/alternative-c-owned-subset/Cargo.toml --target wasm32-unknown-unknown --locked --offline
```

Browser/WGPU execution requires its own retained command, generated binding
identity, browser/device metadata, and manual or automated observation. A
successful native build or WASM compile does not substitute for execution.

## Stop And Escalation Conditions

Stop Option C expansion and return to AR-0019 when:

- a required type or operation cannot be traced to a real caller;
- numerical behavior cannot be specified or verified honestly;
- C grows toward broad `glam` compatibility rather than a measured subset;
- C requires unsafe/SIMD/provider delegation without a measured deficit;
- native/WASM correctness or performance diverges materially without a bounded
  explanation;
- migration would create two stable public math vocabularies; or
- ownership/update benefits no longer justify maintenance risk.

Stop bulk-compute expansion and open or update an Architectural Review when:

- a stable/public batch-operation or provider contract is proposed;
- provider selection, scheduling, residency, synchronization, or fallback
  becomes shared engine policy;
- GPU output is proposed as authoritative simulation/CAD truth;
- a specialized API introduces FFI, unsafe code, device authority, or a new
  deployment/toolchain requirement;
- CPU and GPU semantics cannot be reconciled within the selected contract; or
- independent caller pressure contradicts the chosen operation boundary.

## Parking Criteria

It is valid to park this plan with Option C still conditionally viable when the
post-DOOM operation set or target evidence is incomplete. It is also valid to
retain only CPU bulk evidence when WGPU has no decision-relevant advantage.

Do not keep adding operations, providers, benchmark sizes, or CAD mechanisms
merely to avoid a `not earned` result.

## References

- `docs/Architectural Reviews/AR-0019-native-math-vocabulary-and-foreign-type-boundary.md`
- `docs/Architectural Reviews/AR-0015-ring-zero-provenance-enforcement-and-audit-closure.md`
- `docs/Architectural Reviews/AR-0025-camera-candidate-selection-and-visibility-culling.md`
- `docs/Architectural Reviews/AR-0026-non-euclidean-spatial-charts-and-authored-angular-topology.md`
- `docs/Architectural Reviews/AR-0028-coordinate-frame-handedness-and-directional-conformance.md`
- `docs/Plans/Native-Math/native-math-vocabulary-foreign-type-case-study.md`
- `corpus/lib/tokimu-math-study/alternative-c-owned-subset/`
- `corpus/lib/tokimu-math-study/maintenance-forecast.md`
- `docs/Dependency Audits/Ring 0/glam-d36e7eeff05338c56c4aa8d59fc2615e7963b1b7.md`
- `docs/ADR/ADR-0008-native-kernel-ring-performance-and-code-quality.md`
- `docs/ADR/ADR-0009-ring-based-verification-failure-containment-and-recovery.md`
- `docs/ADR/ADR-0010-ring-zero-third-party-source-admission.md`
- `docs/ADR/ADR-0011-ring-based-security-authority-and-trust-boundaries.md`
