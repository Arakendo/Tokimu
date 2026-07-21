# Tokimu Testing Strategy

| Field      | Value |
| ---------- | ----- |
| Status     | Draft -- implementation-guiding |
| Scope      | Unit, integration, contract, platform, golden, and corpus validation across the Tokimu workspace |
| Relates to | SDD section 16, `docs/roadmap.md`, `docs/example-philosophy.md`, ADR-0001, ADR-0003, ADR-0004 |

## 1. Purpose

Tokimu treats tests and runnable examples as architectural evidence, not as
after-the-fact polish. The testing structure must answer more than whether a
function returned the expected value. It must also show that:

- crate boundaries remain usable through public APIs;
- engine-owned semantics do not leak backend implementation details;
- native, WASM, and future platform adapters preserve the same contracts;
- deterministic behavior remains deterministic;
- render, asset, font, SVG, and other data pipelines survive real inputs;
- examples continue to prove that the engine can express useful applications.

The strategy therefore uses several validation layers. No single layer replaces
the others.

## 2. Testing Principles

### 2.1 Test at the narrowest honest boundary

Place a test beside the smallest boundary whose contract it proves. A local
algorithm belongs in a unit test. A public crate interaction belongs in that
crate's integration tests. A workspace composition claim belongs in the
workspace test harness. A real application capability belongs in an example.

### 2.2 Test public meaning, not incidental implementation

Tests should prefer observable state, returned values, diagnostics, stable
handles, and engine-owned output over private field layouts or foreign backend
objects. Implementation-detail tests are appropriate for genuinely local
invariants, but they must not accidentally freeze replaceable mechanics.

### 2.3 Determinism must be visible

Tests involving time, randomness, scheduling, provider resolution, serialization,
or replay must control their inputs explicitly. Registration order, wall-clock
time, process-global state, filesystem ordering, and platform accidents must not
silently determine a result.

### 2.4 Failure behavior is part of the contract

Missing providers, stale handles, unsupported targets, malformed assets, absent
glyphs, unavailable GPU features, and startup failures should produce explicit,
testable diagnostics. A test that proves deterministic failure is often as
important as a success-path test.

### 2.5 Examples and tests have different jobs

Tests prove bounded assertions automatically. Examples prove that architectural
concepts compose into understandable applications. An example may also contain
unit tests, but it should not be reduced to a hidden test fixture merely because
it participates in validation.

### 2.6 The corpus is an API design tool

Corpus applications exist not only to prevent regressions, but to reveal
repeated patterns that should be promoted into engine capabilities and to expose
responsibilities that should move from applications into Tokimu. A corpus result
may confirm an existing boundary, refine it, or demonstrate that the proposed
boundary is wrong. All three outcomes are useful architectural evidence.

Promotion is not automatic. Repetition across independent corpus examples is
pressure to review an abstraction, not permission to generalize it immediately.
The normal path remains:

```text
Need appears
    ↓
Focused corpus example
    ↓
Repeated independent pressure
    ↓
Ownership review
    ↓
Capability admission or boundary correction
```

## 3. Validation Layers

### 3.1 Crate-local unit tests

Location:

```text
crates/<crate>/src/*.rs
examples/lib-example/<library>/src/*.rs
```

Use `#[cfg(test)]` modules for local invariants and implementation-adjacent
behavior, including:

- entity and handle lifecycle;
- schedule ordering;
- geometry and path calculations;
- text measurement and layout arithmetic;
- input normalization;
- parser edge cases;
- deterministic helper algorithms;
- error classification.

These tests may access private implementation details when that is the smallest
honest way to prove a local invariant.

### 3.2 Crate integration tests

Location:

```text
crates/<crate>/tests/*.rs
```

Use these tests when the behavior should be exercised exactly as an external
consumer sees the crate. They compile as separate crates and therefore cannot
reach private implementation details.

Examples include:

- the public facade re-exporting the intended types;
- scene serialization round trips through public APIs;
- a provider implementation satisfying a public capability contract;
- an asset loader returning Tokimu-owned results and diagnostics;
- runtime composition through documented entry points.

Tests should stay with the owning crate when no cross-workspace composition is
required.

### 3.3 Workspace integration and smoke tests

Location:

```text
tests/
```

The workspace root is a virtual Cargo workspace, so a root `tests/` directory
has no automatic Cargo meaning. Tokimu should make this directory an explicit,
non-published workspace package named `tokimu-workspace-tests`.

Its purpose is to prove cross-crate and public-facade behavior without assigning
ownership to an arbitrary production crate.

Recommended shape:

```text
tests/
  Cargo.toml
  src/
    lib.rs
  tests/
    facade_smoke.rs
    capability_contracts.rs
    backend_contracts.rs
    native_smoke.rs
  support/
    mod.rs
    fake_backend.rs
    golden.rs
  fixtures/
    golden/
      rendering/
      text/
      svg/
      serialization/
```

The package should be registered as `"tests"` in the root workspace members and
set `publish = false`.

The workspace harness owns tests such as:

- the public `tokimu` facade links and creates a minimal application;
- multiple crates compose without reaching through private boundaries;
- a capability registry resolves two fake providers deterministically;
- backend results cross boundaries only as Tokimu-owned types;
- feature combinations preserve intended dependency direction;
- target-specific startup contracts remain reachable.

The existing `crates/tokimu/tests/smoke.rs` remains a valid facade-crate
integration test. It should move to the workspace harness only when the claim
being tested becomes workspace composition rather than facade ownership.

### 3.4 Capability contract tests

Capability contract tests prove engine-owned semantics independently from any
one backend. They should use small fake or reference providers where possible.

They should cover:

- deterministic provider selection;
- target-aware availability;
- explicit unsupported-operation diagnostics;
- lifecycle and teardown behavior;
- stable, provider-neutral handles;
- no leakage of foreign-library objects;
- equivalent semantic results from multiple providers where equivalence is
  promised.

Reusable contract suites may be introduced after at least two real providers
need to satisfy the same assertions. Until then, direct tests are preferred over
a speculative testing framework.

### 3.5 Backend adapter tests

Backend tests prove the adapter between a Tokimu-owned contract and a concrete
implementation. They belong in the backend crate when one exists. Cross-backend
comparison belongs in the workspace harness.

Backend tests should distinguish:

- semantic contract failures;
- backend initialization failures;
- unsupported target or feature failures;
- malformed foreign data;
- backend-specific precision or tolerance;
- conversion into Tokimu-owned output.

A backend test must not redefine the semantic contract around whatever the
foreign library happens to expose.

### 3.6 Platform and target tests

Platform checks prove target-specific seams without making every normal test
run depend on a display server, browser, headset, controller, or GPU.

Categories include:

- headless platform-independent tests;
- native compile and startup smoke tests;
- WASM compile and visible-boot checks;
- renderer/backend tests on supported adapters;
- future XR or hardware validation.

Target-specific tests should use `cfg` gates, Cargo features, or dedicated CI
jobs. A test requiring interactive hardware must identify itself clearly and
must not masquerade as an ordinary unattended unit test.

Native startup tests should verify clean initialization and shutdown. They
should not rely on a human closing a window during the default test suite.

### 3.7 Architectural corpus examples

Location:

```text
examples/hello-*/
examples/ui/hello-ui-*/
```

Architectural corpus examples ask:

> Can Tokimu express this behavior cleanly through the intended boundaries?

Each example should prove one capability or composition pressure and then stop
growing unless new work adds a distinct architectural claim. Examples should
remain runnable and understandable. Snake, Pac-Man, UI text, UI layout, file
I/O, and image export are executable specifications of different engine seams.

When multiple examples repeatedly implement the same semantic operation, that
repetition should be recorded as admission evidence. Conversely, when an example
must reach through a boundary, duplicate backend mechanics, or own behavior that
applications should not own, the friction should be recorded as evidence that
the current API or ownership model needs review.

Where practical, examples should expose deterministic state or focused tests
for their simulation logic. Visual acceptance may still require a reference
capture or manual inspection until automated readback exists.

### 3.8 Data corpus tests

Data corpus tests ask:

> Can this implementation survive representative real-world input?

Examples include:

- Lucide SVG paths for vector parsing and stroking;
- Inter, JetBrains Mono, and Noto fonts for text metrics and rasterization;
- glTF sample models for asset import;
- texture assets for UV, filtering, color-space, and shader validation.

External corpora should be pinned and documented. Generated working copies may
live under `target/`, while small stable regression fixtures may be checked into
the owning test package when their license and provenance are recorded.

The files under `examples/Assets/` are example material and mesh/shader texture
inputs. They are not automatically golden outputs.

## 4. Golden and Snapshot Validation

Golden validation compares a current deterministic result with a reviewed,
versioned expected result. `golden` is a verification method, not a subsystem or
ownership category.

Golden fixtures therefore live under:

```text
tests/fixtures/golden/<domain>/
```

rather than parallel executable directories such as `golden/backend` or
`golden/native`.

Suitable golden artifacts include:

- serialized semantic documents;
- normalized diagnostics;
- deterministic text-layout records;
- SVG path or mesh statistics;
- renderer captures produced under a defined backend and color-space contract;
- compact reference images where pixel comparison is meaningful.

Every golden fixture should record enough context to explain its validity:

- producer and format version;
- target/backend when relevant;
- dimensions and color-space assumptions for images;
- tolerance policy when comparison is not exact;
- source asset and license where applicable;
- reason for the fixture's existence.

Golden updates must be explicit and reviewed. A normal test run must never
silently rewrite expected results. The update workflow should show which files
changed and why before they are accepted.

### 4.1 Exact versus tolerant comparison

Use exact comparison for deterministic engine-owned data such as IDs, schedules,
serialized canonical forms, diagnostics, and integer geometry.

Use documented tolerance for floating-point geometry, rasterization, and GPU
output when exact equality is not an architectural promise. Tolerances must be
domain-specific and narrow enough to catch meaningful regressions. "Looks close"
is a manual observation, not an automated comparison policy.

### 4.2 Rendering captures

A rendering golden must identify whether it validates:

- semantic draw-command generation;
- CPU rasterization;
- one specific GPU backend;
- cross-backend visual equivalence.

These are different claims. Cross-platform GPU pixel identity should not be
promised accidentally. Prefer engine-owned intermediate-output tests alongside a
smaller number of backend capture tests.

## 5. Shared Test Support

The workspace harness may provide small helpers for:

- deterministic temporary paths;
- fake capability providers;
- normalized diagnostic assertions;
- golden loading and comparison;
- fixed clocks and seeded randomness;
- test application construction.

Helpers belong in `tests/support/` only when multiple integration test targets
need them. Production crates must not depend on the test harness. Test support
must not become a second runtime or a hidden source of engine semantics.

Filesystem tests should use isolated temporary directories and must not depend
on the caller's current working directory. Tests must not modify checked-in
fixtures during ordinary execution.

## 6. Test Placement Decision Table

| Behavior under test | Preferred location |
| --- | --- |
| Private algorithm or local invariant | `#[cfg(test)]` beside the implementation |
| One crate's public API | `crates/<crate>/tests/` |
| Public facade behavior owned by `tokimu` | `crates/tokimu/tests/` |
| Cross-crate workspace composition | `tests/tests/` in `tokimu-workspace-tests` |
| Provider-independent capability semantics | Workspace capability contract test or capability crate test |
| One concrete backend adapter | Backend crate tests |
| Native/WASM/backend startup seam | Target-gated workspace smoke test or CI job |
| Expected deterministic artifact | Owning test plus `tests/fixtures/golden/<domain>/` |
| Architectural application proof | Focused `hello-*` example |
| Real-world parser/renderer pressure | Data corpus example or corpus test |
| Performance trend | Dedicated benchmark, not a timing assertion in a unit test |

## 7. Naming

Test names should describe the promised behavior:

```text
stale_handle_is_rejected_after_slot_reuse
provider_resolution_is_independent_of_registration_order
facade_builds_a_minimal_headless_app
missing_font_emits_a_deterministic_diagnostic
```

Avoid names that merely repeat implementation functions or broad labels such as
`test_backend`. Test files may be grouped by architectural boundary, but each
test should state the contract it proves.

## 8. Execution Tiers

### 8.1 Fast local tier

Expected during normal development:

```text
cargo test -p <affected-package>
cargo check -p <affected-package>
```

### 8.2 Workspace tier

Expected before merging substantial changes:

```text
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### 8.3 Target and corpus tier

Run when the affected boundary requires it:

- WASM build and browser boot validation;
- native startup/shutdown checks;
- GPU/backend capture tests;
- full Lucide, font, model, or texture corpus runs;
- hardware-dependent XR/input checks;
- reviewed golden comparisons.

Long-running corpus and hardware checks should be separately invocable and
reported. They must not disappear merely because they are too expensive for the
fast tier.

## 9. CI Direction

CI should eventually separate jobs by evidence type:

1. Formatting and linting.
2. Workspace unit and integration tests.
3. Native target compilation and unattended smoke checks.
4. WASM compilation and boot checks.
5. Golden and deterministic corpus validation.
6. Optional backend, GPU, and hardware matrices.

Required jobs should fail explicitly when their environment is unavailable.
Optional hardware jobs should report that they were skipped rather than claiming
the capability passed.

## 10. Initial Adoption Plan

1. Turn the root `tests/` directory into the non-published
   `tokimu-workspace-tests` package.
2. Add a minimal facade/workspace boot test without duplicating crate-local
   assertions.
3. Add one capability registry proof with two fake providers and deterministic
   selection.
4. Add explicit target gating for native smoke tests.
5. Move the proposed `tests/golden/*` directories to
   `tests/fixtures/golden/<domain>/` as actual fixtures are introduced.
6. Add shared support only after at least two test targets need the same helper.
7. Keep examples as corpus proofs and link each promoted regression to an
   automated assertion where practical.

## 11. Admission Rule for New Test Infrastructure

A new test framework, snapshot library, mock layer, fixture format, or harness
abstraction must solve a demonstrated recurring problem. One test should remain
direct and concrete. Repetition across independent tests is the evidence for
promoting shared infrastructure.

The test tree should make architectural ownership clearer. If a helper obscures
which layer owns a behavior, it is the wrong abstraction even when it reduces
line count.
