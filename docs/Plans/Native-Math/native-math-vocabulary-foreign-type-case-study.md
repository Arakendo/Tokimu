# Native Math Vocabulary And Foreign-Type Case Study

| Field | Value |
| --- | --- |
| Status | Proposed experiment |
| Owner | Tokimu maintainers |
| Related review | `AR-0019-native-math-vocabulary-and-foreign-type-boundary.md` |
| Related ADRs | ADR-0003, ADR-0005, ADR-0008, ADR-0009, ADR-0010, ADR-0011 |
| Corpus root | `corpus/lib/tokimu-math-study/` |

## Purpose

Use Tokimu's current `glam` public-type exposure as a controlled case study for
deciding whether Native Ring public vocabulary should remain foreign,
Tokimu-owned over a provider, independently implemented, or derived from a
bounded upstream fork/copy.

Alternative A—the current direct re-export—remains the comparison baseline.
The experiment implements and measures Alternatives B, C, and D without
changing `tokimu_core::math`, admitting a second stable API, or changing the
accepted `glam` dependency disposition before the evidence is reviewed.

The output is both a math decision and a reusable evaluation pattern for future
foreign types or traits found in Native Ring public APIs.

## Deferred DOOM Pressure Revisit

This plan's 0.1 inventory represents only the current caller set. Before a
final recommendation for A, B, C, or D, revisit the study after
[`DOOM WAD Checklist.md`](../DOOM/DOOM%20WAD%20Checklist.md) is complete. That
work is expected to alter or increase pressure from objects, transforms,
animation, imported scene data, collision, and rendering.

At that point, rerun the source scan, update the operation manifest with each
new named caller pressure, extend shared conformance cases where warranted, and
repeat affected measurements. Do not treat the current favorable B probes as a
permanent recommendation until this revisit is retained.

## Alternatives Under Test

| Alternative | Candidate | Ownership hypothesis |
| --- | --- | --- |
| A — Current baseline | Directly re-export the five `glam` types | Foreign provider owns public vocabulary and implementation |
| B — Provider-backed vocabulary | Tokimu-owned public shapes delegate mechanics to pinned `glam` | Tokimu owns meaning; `glam` remains a replaceable implementation |
| C — Narrow owned implementation | Implement only operations required by real callers | Tokimu owns meaning and implementation |
| D — Bounded fork/copy | Derive only the required subset from the pinned audited source with provenance retained | Tokimu owns the modified implementation but inherits identifiable upstream lineage and obligations |

No candidate is favored by the folder layout or sequence. B runs first because
it tests semantic separation with the smallest implementation change. C and D
remain independent candidates rather than fallbacks hidden inside B.

## Corpus Layout

```text
corpus/lib/tokimu-math-study/
    README.md
    baseline-a/
        README.md
    alternative-b-provider-backed/
        README.md
    alternative-c-owned-subset/
        README.md
    alternative-d-bounded-fork/
        README.md
```

Each candidate will become a separately measurable corpus target or feature
composition. Candidate types remain corpus-local until AR-0019 reaches a
disposition and an ADR intentionally changes the stable ownership boundary.

## Shared Experimental Rules

- The study is limited initially to `Vec2`, `Vec3`, `Vec4`, `Quat`, and `Mat4`.
- Operations are admitted from source scans and compile pressure from existing
  renderer and corpus callers, not by cloning an upstream API for completeness.
- Semantic behavior, coordinate conventions, exceptional floating-point
  behavior, layout, and conversion contracts must be explicit.
- Native and `wasm32-unknown-unknown` use the same semantic conformance cases.
- No candidate may add allocation to ordinary value operations.
- Measurements use the same workloads, profiles, toolchain, and reporting
  format. Results from different machines are not presented as universal.
- Direct `glam` interop is permitted at corpus boundaries for comparison, but
  no foreign type may be described as a Tokimu-owned contract by renaming it.
- D must preserve exact source provenance, licensing, attribution, and local
  modifications. It is not a license to import arbitrary upstream files.
- Failure, rejection, and unsupported-operation results are retained evidence;
  candidates do not silently grow until they appear successful.

## Shared Evidence Matrix

Every candidate report must cover:

| Area | Required evidence |
| --- | --- |
| API pressure | Required types, constructors, constants, fields, methods, operators, traits, and conversions by caller |
| Correctness | Shared deterministic vectors, matrices, quaternions, transforms, edge values, and rejection cases |
| Performance | Representative hot-path throughput, copy/conversion counts, allocation evidence, and stated workload/profile/target |
| Representation | `size_of`, `align_of`, field access, `repr` claims, POD/FFI implications, and provider layout assumptions |
| Portability | Native and WASM build/test results; target-specific implementation or SIMD differences |
| Build cost | Clean and incremental compile observations, binary/code-size comparison, selected dependency closure |
| Maintenance | Rust source files/lines, unsafe surface, generated code, tests, documentation, and update burden |
| Migration | Compiler-visible caller changes, conversion sites, compatibility strategy, and rollback cost |
| Ecosystem | Serialization, reflection, authoring frontend, renderer, and provider boundary consequences |

## Slices

### Slice 0: Freeze The Control And Experimental Boundary

#### Deliverables

- [x] Record Alternative A's exact five-type re-export and pinned `glam`
      revision as the control (`baseline_a` and `operation-inventory.md`).
- [x] Confirm the study remains outside `tokimu-core` and cannot be imported by
      stable engine crates.
- [x] Add an isolated A WASM control that imports the actual stable
      `tokimu_core::math` re-exports, so the bounded Node engine probe compares
      A/B/C rather than an imitation of the current public vocabulary.
- [x] Define an initial compileable study runner for the shared profile and
      target conditions; candidate-specific execution remains later work.
- [x] Record toolchain, target, profile, selected provider features, and host
      metadata (or an explicit collection limitation) in retained generated
      results (`results/2026-08-07-initial-a-b-transform-run.md`).
- [x] Add a corpus-local warning that candidate APIs are experimental and may
      be retired completely.

#### Acceptance Criteria

- [x] Alternative A reproduces current caller behavior without modifying the
      stable math API.
- [x] No B, C, or D candidate is reachable from a public Tokimu crate.
- [x] A reviewer can distinguish architectural ownership claims from measured
      implementation observations.
- [x] Candidate comparison does not require network source retrieval after the
      audited submodule and Rust toolchain are present.

### Slice 1: Inventory The Real Math Contract

#### Deliverables

- [x] Scan `tokimu-render`, the facade, and all current math-using corpus code
      for types, constants, constructors, fields, methods, operators, and
      conversions.
- [x] Group requirements by caller and by semantic role rather than producing
      one undifferentiated method list.
- [x] Record currently observed coordinate handedness, column-major
      multiplication, radians, affine transform behavior, normalization
      fallback, singular inversion behavior, and writable-column pressure in
      `operation-inventory.md`. These remain observations pending an ownership
      decision, not new stable contracts.
- [x] Identify unused portions of each foreign type's API explicitly.
- [x] Freeze operation manifest 0.1 shared by B, C, and D
      (`corpus/lib/tokimu-math-study/operation-inventory.md`).

#### Acceptance Criteria

- [x] Every operation implemented by a candidate maps to at least one real
      caller or named conformance requirement.
- [x] Current callers compile against an instrumented inventory or are covered
      by an equivalent retained source-scan artifact.
- [x] No candidate can claim completeness merely by matching the five type
      names.
- [x] Adding an operation changes the reviewed manifest and names the pressure
      that required it.

### Slice 2: Build Shared Conformance And Measurement Evidence

#### Deliverables

- [x] Add the initial shared `Vec3` tranche for construction, arithmetic,
      dot/cross, nonzero normalization, and zero-normalization observations
      (`corpus/lib/tokimu-math-study/src/conformance.rs`). Its evidence labels
      distinguish current caller pressure from observed provider behavior.
- [x] Add the initial transform tranche for translation, position/direction
      separation, inversion, right-handed view construction, and OpenGL-depth
      perspective observations (`conformance.rs`).
- [x] Add observed edge cases for non-finite normalization fallback and
      degenerate right-handed view input. These remain provider observations,
      not promised recovery behavior.
- [x] Add a deterministic composed affine inverse sweep spanning translation,
      rotation, non-uniform scale, and varied point ranges. It compares A/B/C
      round trips at an explicit tolerance without choosing singular-matrix
      recovery semantics.
- [x] Add a 96-case deterministic affine differential sweep with bounded
      non-singular translation, rotation, scale, and point inputs. It compares
      B/C against A at a stated tolerance without adding a random dependency or
      claiming fuzz/property completeness.
- [x] Add deterministic shared cases for construction, arithmetic, dot/cross,
      normalization, quaternion composition, matrix composition, projection,
      view transforms, and vector transformation where required by Slice 1.
- [x] Add edge cases for zero length, non-finite values, degenerate view data,
      and normalization behavior without inventing guarantees absent from the
      current contract.
- [x] Add compile-time size/alignment assertions or observations per target.
- [x] Add an allocation-counting executable for the shared A/B transform
      workload (`measure_transform_allocations`). It resets before each run and
      fails if either ordinary value-operation path allocates; C and D must use
      it unchanged or retain a reviewed extension.
- [x] Define the shared A/B/C transform workload and checksum-producing,
      warmed repeated-sample manual measurement entry point (`workloads.rs` and
      `measure_transform_workload`). The runner rotates candidate order and
      reports min/median/max; D must use this unchanged workload or retain a
      reviewed extension.
- [x] Add candidate-isolated release link targets for the same shared transform
      workload (`measure_transform_binary_{a,b,c}`), so a binary-size
      observation does not link every candidate by construction.
- [x] Add a native A/B/C size/alignment observation executable
      (`observe_layouts`) that reports current compiler/target facts without
      promoting any candidate representation to a stable ABI or GPU contract.

#### Acceptance Criteria

- [x] The A baseline passes the shared conformance suite before candidate
      results are compared.
- [x] Each case states whether it captures a Tokimu guarantee, observed `glam`
      behavior, or deliberately unspecified behavior.
- [x] Measurements name workload, target, profile, toolchain, and repetition
      method.
- [x] A candidate failure remains visible and does not weaken the common suite.

### Slice 3: Alternative B — Provider-Backed Tokimu Vocabulary

#### Deliverables

- [x] Define corpus-local candidate type names and public accessors without
      re-exporting `glam` types (`src/alternative_b.rs`). `Vec2` and `Quat`
      remain intentionally minimal because they have no direct caller pressure.
- [x] Implement the Slice 1 operation manifest by delegating to pinned `glam`
      where useful.
- [x] Make every `glam` conversion explicit and inventory conversion sites in
      representative renderer and corpus migrations.
- [x] Add a corpus-local renderer-shaped B migration fixture with one explicit,
      crate-private provider upload conversion (`migration_b.rs`). This records
      the boundary shape but does not substitute for a real renderer migration.
- [x] Add the equivalent owned-C fixture, which reconstructs a provider matrix
      from the candidate column array at the same adapter boundary
      (`migration_c.rs`). This makes the representation-conversion difference
      explicit for later measurement.
- [x] Compile the shared B source through an isolated corpus crate with its
      one pinned private `glam` dependency (`alternative-b-provider-backed/
      Cargo.toml`), making its build closure comparable to C's dependency-free
      isolated crate.
- [x] Compare an inner-provider representation with a plain Tokimu-owned
      representation if layout or conversion cost materially differs.
- [x] Record trait ergonomics, debugging, equality, constants, and ownership of
      validation behavior (`candidate-api-ergonomics.md`).
- [x] Produce native/WASM correctness, performance, layout, size, compile-time,
      unsafe, and source-size reports against A.

#### Acceptance Criteria

- [x] No `glam` type or trait appears in the candidate's public signature.
- [x] The candidate passes the shared conformance suite on native and WASM.
- [x] Steady-state caller paths have no hidden allocation.
- [x] Conversion and copy sites are counted and attributable to actual
      boundaries; “zero cost” is claimed only with retained evidence.
- [x] The candidate demonstrates that `glam` could be replaced internally, or
      records precisely where provider assumptions still leak.
- [x] The report concludes B is viable, conditionally viable, or rejected; it
      does not silently proceed to stable migration.

### Slice 4: Alternative C — Narrow Tokimu-Owned Implementation

#### Deliverables

- [x] Begin the original owned implementation with the frozen `Vec3` manifest,
      no unsafe code, and no provider reference (`src/alternative_c.rs`).
- [x] Add the non-inversion owned `Mat4` slice: column-major representation,
      affine transforms, composition, transpose, right-handed view, and
      OpenGL-depth projection. Singular-matrix behavior remains explicitly
      deferred before `inverse` is admitted.
- [x] Add owned `Mat4::inverse` using bounded stack-only pivoted
      Gauss-Jordan elimination. Alternative C records all-NaN output for a
      non-invertible matrix as provisional experiment behavior; this must be
      reviewed before any stable contract is proposed.
- [x] Retain C's initial independence and maintenance finding
      (`alternative-c-owned-subset/initial-findings.md`); it is conditionally
      viable evidence, not a public migration recommendation.
- [x] Compile the shared C source through a dependency-free isolated corpus
      crate (`alternative-c-owned-subset/Cargo.toml`) to verify that the owned
      candidate has no inherited provider, build-script, macro, or runtime
      dependency from the wider A/B comparison target.
- [x] Implement only the frozen Slice 1 operation manifest in original
      corpus-local Tokimu source.
- [x] Record algorithms and numeric conventions for vectors, quaternions, and
      matrices, including normalization and projection behavior.
- [x] Begin without unsafe code or target-specific SIMD; propose either only
      when a measured deficit and testable invariant justify it.
- [x] Add differential evidence against A without using A to define behavior
      where Tokimu intentionally chooses a different explicit contract.
- [x] Produce native/WASM correctness, performance, layout, size, compile-time,
      unsafe, and source/test-size reports against A and B.
- [x] Estimate ongoing maintenance for correctness, optimization, target
      support, fuzz/property tests, and future operation growth
      (`maintenance-forecast.md`).

#### Acceptance Criteria

- [x] C has no runtime, build, macro, copied-source, or hidden `glam`
      dependency.
- [x] C passes the shared conformance suite on native and WASM, with documented
      intentional differences reviewed separately.
- [x] Ordinary value operations allocate nothing.
- [x] Required numerical tolerances and degenerate cases are explicit and
      tested.
- [x] Any material regression or complexity increase is retained rather than
      optimized away without a new measurement.
- [x] The report concludes C is viable, conditionally viable, or rejected and
      identifies the maintenance burden Tokimu would accept.

### Slice 5: Alternative D — Bounded Fork/Copy Subset

#### Deliverables

- [x] Begin D with a provenance-preserving scalar `Vec3` derivation. Its
      upstream path, pinned revision, licensing, local modifications, and
      intentional source bound are retained in
      `alternative-d-bounded-fork/UPSTREAM-NOTICE.md`.
- [x] Retain D's initial maintenance finding and pause expansion pending a
      measured C deficit or specific upstream-compatibility need
      (`alternative-d-bounded-fork/initial-findings.md`).
- [x] Select the smallest audited source unit: scalar `Vec3` only. Matrix,
      quaternion, other vector, SIMD, generated, and swizzle sources remain
      unearned.
- [x] Create a copied-source manifest mapping the derived file and retained
      sections to upstream revision, path, and license relationship
      (`COPY-MANIFEST.md`).
- [x] Preserve attribution, dual-license references, and local modification
      history in `UPSTREAM-NOTICE.md` and `COPY-MANIFEST.md`.
- [x] Record all retained rewrites and exclusions, including the absence of
      generated code, unsafe blocks, and architecture intrinsics.
- [x] Exclude unused swizzles, generators, target implementations, and APIs
      until manifest pressure requires a reviewed expansion.
- [x] Define how upstream security/correctness fixes are detected by the pinned
      provider audit and compared, selectively incorporated, or rejected.
- [x] Produce native/WASM correctness, performance, layout, size, compile-time,
      unsafe, and source/test-size reports against A, B, and C.

#### Acceptance Criteria

- [x] Every derived line has reviewable upstream provenance; no copied code is
      presented as original Tokimu source.
- [x] D contains only the concepts and operations earned by its own bounded
      manifest, plus locally justified supporting mechanics.
- [x] D passes its shared `Vec3` conformance scope on native and WASM; it is
      explicitly ineligible for matrix cases.
- [x] Unsafe or SIMD code is individually inventoried with its invariant and
      target behavior.
- [x] The source/update burden is measured against retaining the pinned
      provider, not treated as free because the files are local.
- [x] The report concludes D is viable, conditionally viable, or rejected and
      states whether it is a maintainable fork, a one-time extraction, or an
      unacceptable provenance/update burden.

### Slice 6: Representative Caller Migration

#### Deliverables

- [x] Define the bounded renderer, basic-3D, and imported-scene migration
      protocol, evidence table, and rollback rule
      (`representative-migration-protocol.md`).
- [x] Reserve independently copied `hello-3d-mono` candidate cases for A, B,
      C, and an explicit D-blocked record. These copies will provide the source
      edit and integration evidence a shared helper cannot.
- [x] Port the bounded renderer camera path to B and C in corpus-local
      fixtures, including explicit handoff to the current public
      `tokimu::Camera`; retain D as blocked because it has no `Mat4` slice.
- [x] Port at least one 3D scene consumer and one transform-heavy consumer to
      each viable matrix candidate; retain D's `Mat4` ineligibility explicitly.
- [x] Count source edits, explicit conversions, compatibility helpers, and
      provider leaks for each port (`migration-accounting.md`).
- [x] Exercise round-trip interop only at the retained current-renderer matrix
      boundary for B/C; D remains blocked without `Mat4`.
- [x] Record rollback steps for every candidate (`migration-accounting.md`).

#### Acceptance Criteria

- [x] All viable matrix candidates exercise equivalent bounded caller behavior;
      D's explicit `Vec3`-only ineligibility is not counted as equivalence.
- [x] Migration cost is derived from real edits rather than an API-shape guess.
- [x] No viable candidate gains an unfair result by omitting a required caller
      or performing work outside the measured path; D's scope is recorded.
- [x] The original callers remain unchanged outside the corpus experiment.

#### Current Slice-6 Evidence (2026-08-07)

- The original `corpus/focused/foundations/hello-3d-mono` remains the A control; independently
  compileable B and C window/render-shell copies now exercise its rotating cube
  and camera path with their candidate types.
- A retained `hello-shader` / `hello-audio-visualizer` source scan found only
  identity-equivalent camera-view translation construction, already covered by
  stronger shared and corpus fixtures. It deliberately adds no duplicate
  candidate migration; reopen only if those callers gain distinct pressure.
- Native offline checks passed for B and C. The study library also builds for
  `wasm32-unknown-unknown`.
- B and C candidate cameras now form the current public `tokimu::Camera` only
  at a private boundary. B unwraps candidate `view` and `projection`; C
  reconstructs both provider matrices from columns. Exact composed-transform
  comparisons pass. This closes the bounded renderer-fixture deliverable for
  B/C, while exposing the stable facade's provider-valued `Camera` fields as a
  real future API migration seam; D remains intentionally blocked without a
  matrix implementation.
- A bounded surface scan records the current public `Camera` seam: its two
  public provider-matrix fields and eight corpus source files with direct 3D
  `view` assignment. It separates that real future migration scope from the
  unaffected/operation-unproven 2D constructor callers.
- A bounded `hello-3d-stereo` fixture now covers its distinct two-camera
  writer shape. A/B/C agree for separate left/right orbit views and half-width
  projections; B/C each make four explicit current-renderer matrix crossings.
  This is multiplicity and public-boundary evidence only, not a new math API
  requirement or a full stereo renderer port.
- A representative `hello-asteroids` orthographic-camera fixture now matches
  normal and zero-height-fallback renderer policy through B/C private
  boundaries. It is retained as renderer compatibility evidence rather than
  promoting orthographic projection to independently demonstrated universal
  Native meaning.
- A rotated native release observation now builds complete stereo camera pairs
  100,000 times per sample. A/B/C all allocate zero times; C's retained median
  is approximately 26% above A on this host, while B remains in A's range.
  This is deliberately a target-local performance finding, requiring native
  and WASM repeat under the eventual ADR-0008 gate rather than a selection.
- The isolated A/B/C WASM crates now execute a repeated stereo math and
  column-array probe in Node. B/C are each about 5% below A in that narrow
  engine observation, unlike C's native public-camera result; C's raw WASM
  output is larger. The scope difference is retained rather than averaged
  away, and browser/WGPU evidence remains open.
- The full study crate now also builds for `wasm32-unknown-unknown` with the
  current B/C renderer-camera handoff, stereo, and orthographic fixtures
  present. This closes a compile-only portability check, not browser/WGPU
  runtime evidence.
- A fixed-seed 128-case finite camera/projection differential sweep now checks
  `perspective_rh_gl * look_at_rh` across A/B/C at `1e-4` matrix tolerance.
  It is caller-shaped conformance evidence only and deliberately excludes
  degenerate/non-finite behavior from any implied contract.
- Representative migration accounting now records zero stable-source edits,
  nine corpus-local modules and nine explicit renderer matrix crossings for
  each B/C path, plus rollback. Both candidates exact-round-trip the retained
  renderer matrix boundary without provider types entering their public
  signatures; the current public `Camera` remains the visible provider seam.
- The B/C candidate ergonomics record now separates wrapper access and
  provider-delegated validation from C's scalar access and Tokimu-owned
  validation burden; neither candidate is presented as full `glam`
  source-compatibility.

#### Checklist Reconciliation (2026-08-08)

Completed boxes in Slices 0–4 are backed by the retained operation inventory,
conformance suite, isolated native/WASM targets, layout/source/build records,
boundary accounting, and B/C interim findings. A checked item means the
bounded study evidence exists; it does not silently elevate that evidence into
a stable Tokimu contract. Remaining open boxes are intentionally limited to
full shared-suite WASM execution, D's unearned matrix scope, browser/WGPU and
complete application evidence, post-DOOM pressure, maintenance forecasting,
and final AR/ADR selection work.
- A bounded `hello-glb` imported-scene fixture now exercises composed model and
  floor transforms, non-uniform scale, inverse-transpose normal handling, and
  `normalize_or_zero` across A/B/C. Its independent comparison decodes the
  pinned Khronos `Box.glb` fixture and uses the decoded positions and normals;
  it passed without adding candidate operations. Full GLB application copies
  remain future work.
- A bounded `hello-cad` cursor-ray fixture now exercises the previously
  untested real caller pressure for homogeneous `Mat4 * Vec4`, perspective
  division, and degenerate-ray rejection. It passes A/B/C comparison and adds
  only `Vec3::length_squared` plus matrix-vector multiplication to the reviewed
  operation inventory; it is not a CAD application migration.
- A bounded `hello-hole-punch` node-resolution fixture now exercises imported
  column-array transforms, animation translation override through the final
  matrix column, and parent-child composition. A/B/C pass comparison including
  a real decoded pinned GLB node; this is not a scene, animation, mesh, or
  renderer migration.
- The expanded study library, including the CAD and animated-node fixtures,
  passes fresh native and `wasm32-unknown-unknown` compile checks. This is
  compile-only target evidence; browser execution and target measurements
  remain open work.
- A reproducible source-surface observation now records the current A/B/C/D
  candidate files and the pinned provider tree. It makes the implementation and
  provenance footprint visible without claiming binary-size, performance, or
  lifetime-maintenance equivalence.
- Shared conformance now compares A/B/C non-finite output masks for degenerate
  `look_at_rh` input and singular matrix inversion. These remain observed
  provider behavior, not selected Tokimu error or recovery semantics.
- A refreshed direct-import scan confirms that `Vec2` and `Quat` have no
  current in-repository caller. Application-local 2D vectors are recorded as
  possible future pressure, not justification to widen C speculatively.
- A bounded `hello-fps-web` motion fixture now exercises real caller pressure
  for `Vec3::distance`, add-assign, and component mutation. A/B/C agree; B's
  required getter/reconstruction remains explicit migration evidence.
- A shared `tokimu-platform/src/wasm.rs` moved-`event_handler` compile error
  was repaired while unblocking target evidence. B/C app-copy WASM checks still
  match the A control's native-window limitation (`run_window_with_app`, native
  window value, and synchronous backend creation are not browser-shaped). Do
  not classify that common application-shape result as a candidate portability
  difference.
- D remains intentionally blocked because its admitted slice has no `Mat4`.

### Slice 7: Compare Alternatives And Generalize The Finding

#### Deliverables

- [x] Retain an interim A/B/C/D comparison snapshot with explicit incomplete
      evidence and deferred DOOM pressure (`interim-comparison.md`).
- [x] Produce one side-by-side decision matrix for A, B, C, and D using the
      shared evidence categories (`decision-matrix.md`).
- [x] Separate semantic independence, implementation independence, source
      provenance, runtime performance, build cost, and maintenance cost.
- [x] Record any result that is workload-specific or target-specific rather
      than collapsing it into a universal ranking.
- [x] Recommend continue incubation for Tokimu math until post-DOOM caller
      pressure and the remaining selection evidence are available.
- [x] Extract a reusable foreign-public-type review checklist from the case
      study and apply it retrospectively to this experiment
      (`foreign-public-type-review-checklist.md`).
- [x] Update AR-0019 and preserve the `glam` dependency audit unchanged while
      the stable ownership boundary does not change; revise an ADR only on
      selection.

#### Acceptance Criteria

- [x] Every evidence-phase recommendation traces to retained evidence and names
      its tradeoff (`phase-close-report.md`).
- [x] A and all non-selected candidates remain visible in the phase-close
      report and decision matrix.
- [x] The reusable method distinguishes public-vocabulary ownership from
      implementation and source ownership.
- [x] No stable API migration begins until AR-0019 records a disposition and
      the applicable ADR-0008 through ADR-0011 gates are satisfied.

### Slice 8: Retire Or Graduate The Corpus

> **Selection-phase gate:** the items below are not remaining evidence-phase
> implementation work. They activate only after the post-DOOM re-scan and the
> browser/WGPU, public-boundary, and numerical-contract blockers recorded in
> AR-0019 are resolved. Until then A remains the stable control and all corpus
> alternatives stay retained, clearly experimental evidence.

#### Deliverables

- [ ] If A remains preferred, retain the comparison report and retire candidate
      code that would create a competing vocabulary.
- [ ] If B, C, or D is selected, write a separate compileable migration plan
      with compatibility, deprecation, rollback, and release slices.
- [x] Preserve the smallest reproductions and measurement artifacts needed to
      audit the decision later.
- [x] Mark copied or forked source clearly as retained, retired, or promoted;
      do not leave ambiguous derived code in the corpus.

#### Acceptance Criteria

- [ ] The workspace has one stable Native math vocabulary after any migration.
- [x] Experimental code cannot be mistaken for supported engine API.
- [x] License and provenance obligations remain satisfied for retained D code
      or are removed with the retired candidate.
- [ ] AR-0019, the dependency audit, the SDD, and implementation agree on the
      resulting ownership boundary.

## Validation Direction

The exact runner will be created in Slice 0. It should support a sequence
equivalent to:

```powershell
cargo fmt --all --check
cargo clippy -p tokimu-math-study --all-targets -- -D warnings
cargo test -p tokimu-math-study --locked --offline
cargo build -p tokimu-math-study --target wasm32-unknown-unknown --locked --offline
cargo bench -p tokimu-math-study --no-run
pwsh -NoProfile -File corpus/lib/tokimu-math-study/scripts/compare-alternatives.ps1
```

The shared conformance runner must add actual WASM execution evidence before a
candidate can satisfy its native/WASM behavioral acceptance criterion; a
successful WASM build proves portability only. Benchmark execution,
binary-size tooling, and clean-build timing may require separate commands and
artifacts. The plan must record unavailable tooling or unsupported targets
honestly rather than substituting a different claim.

## Stop Conditions

Stop or reject a candidate when:

- it requires semantics or operations not supported by real callers;
- it leaks foreign types through the proposed Tokimu-owned public boundary;
- it cannot preserve required native/WASM behavior;
- it adds unbounded allocation or materially worse hot-path cost without a
  compensating architectural benefit;
- its unsafe, generated, copied, or target-specific source cannot be bounded
  and audited honestly;
- its migration creates a second stable source of truth; or
- its maintenance and update burden exceeds the semantic independence it
  provides.

## References

- `docs/Architectural Reviews/AR-0019-native-math-vocabulary-and-foreign-type-boundary.md`
- `docs/Architectural Reviews/AR-0015-ring-zero-provenance-enforcement-and-audit-closure.md`
- `docs/Dependency Audits/Ring 0/glam-d36e7eeff05338c56c4aa8d59fc2615e7963b1b7.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0008-native-kernel-ring-performance-and-code-quality.md`
- `docs/ADR/ADR-0009-ring-based-verification-failure-containment-and-recovery.md`
- `docs/ADR/ADR-0010-ring-zero-third-party-source-admission.md`
- `docs/ADR/ADR-0011-ring-based-security-authority-and-trust-boundaries.md`
- `crates/tokimu-core/src/math.rs`
