# Native Execution and Multithreading

## Status

Proposed. Ownership is accepted by ADR-0006; implementation and public API shape
remain unproven.

## Purpose

Prove that Tokimu can execute independent work sequentially or in parallel
without changing domain meaning, observable commit requirements, capability
ownership, or native/WASM architecture.

This is an execution-policy plan, not a requirement to make every subsystem or
simulation system multithreaded.

The governing boundary is:

> Capabilities describe dependency. Tokimu execution policy exploits
> independence. Platforms provide mechanisms.

```text
domain-owned deterministic work
              |
              v
Tokimu-owned dependency and execution policy
              |
              v
runtime orchestration and global budget
              |
              v
engine-owned execution-mechanism contract
              ^
              |
platform implementation supplied at application composition
              |
              v
validated results and ordered commit
```

## Motivation And Evidence

Tokimu already has independent or potentially independent workloads:

- presentation geometry and tessellation;
- SVG and font-outline conversion;
- raster image decoding and asset transformation;
- corpus cases and artifact generation;
- future physics island computation;
- future network packet decoding.

The current schedule and system registry also provide useful constraints:

- phases and dependency ordering are Tokimu-owned;
- systems currently receive exclusive `&mut World`;
- deterministic sequential behavior is the baseline;
- target behavior must remain coherent across native and WASM.

The plan should exploit independent pure or read-only transformations before
attempting concurrent world mutation.

## Goals

- Preserve identical semantic inputs, results, diagnostics, and required commit
  order across sequential and parallel mechanisms.
- Establish one application-wide owner for worker and queue budgets.
- Keep domain/provider APIs independent of threads, Rayon, async runtimes, and
  browser worker objects.
- Make sequential execution easy to test, debug, and select explicitly.
- Measure whether parallel execution pays for its scheduling, allocation,
  transfer, and merge costs.
- Surface task identity, queueing, execution, merge, and failure diagnostics.
- Preserve native/WASM semantic parity.

## Non-Goals

- Parallel mutation of `World`.
- Replacing `Schedule`, phases, or the existing system dependency model.
- Making every `System` implement `Send` or `Sync`.
- Selecting Rayon or another executor library before measurement.
- Promising parallel speedup.
- Introducing a general async I/O runtime.
- Implementing Web Workers in the first native proof.
- Exposing platform-native threads, workers, locks, or futures in capability
  contracts.
- Creating a separate optional execution capability crate.

## Ownership And Dependency Boundaries

### `tokimu-core`

May own small engine-neutral execution semantics once a real caller proves each
one. Candidate meanings include stable work identity, dependency/readiness,
commit ordering, affinity requirements, budget requests, and diagnostic
classification.

It must not own thread creation, a pool implementation, browser workers, or an
external executor dependency.

### `tokimu-runtime`

Owns application-wide coordination, configured budgets, dispatch, joining,
draining, ordered commit, lifecycle, and execution diagnostics.

The first implementation may remain runtime-internal. A public executor trait
is not required for the first proof.

### `tokimu-platform`

Owns target-specific mechanisms and capability discovery. The first native
mechanism may use scoped standard-library threads, a small retained pool, or a
measured external implementation. WASM remains sequential until worker
requirements are deliberately designed.

The platform crate implements an engine-owned mechanism contract and is composed
with the runtime by the application or facade. `tokimu-runtime` must not acquire
a dependency on `tokimu-platform`.

### Domains And Providers

Own deterministic transformations, work granularity, validation, and domain-
specific merge/commit behavior. They may describe demand but do not allocate
the application's worker budget.

Renderer backends continue to own GPU synchronization and submission. An
external backend's unavoidable internal threading must be isolated and
diagnosed rather than promoted into Tokimu's semantic contract.

## Execution Invariants

1. The sequential and parallel paths consume the same domain work description.
2. A supported workload remains correct when configured for one worker.
3. Work does not gain ambient mutable access merely because it is dispatched.
4. Result association uses stable work identity, not completion order.
5. Observable commit order is explicit where semantics require it.
6. A failed work item produces structured diagnostics tied to its identity and
   stage.
7. Shutdown drains or cancels according to a documented boundary; it does not
   silently abandon stateful commits.
8. Domains do not create uncoordinated ambient pools.
9. Performance measurements are observations, not API guarantees.
10. Unsupported target parallelism selects or reports sequential execution
    explicitly; it does not change semantic results.

## First Workload Selection

Choose one workload only after recording:

- input and output ownership;
- independence boundaries;
- expected batch size and granularity;
- deterministic result association and commit order;
- current sequential timing;
- allocation and copy costs;
- failure and cancellation behavior;
- native and WASM constraints.

Preferred first candidates are presentation-geometry corpus cases or bounded
vector tessellation batches because they transform immutable inputs into owned
outputs and already have structural comparison artifacts.

The corpus runner is attractive for mechanism validation but is not by itself
enough to stabilize an application-facing executor API. A production
presentation or asset caller should follow before graduation.

## Implementation Slices

### Slice 0: Record The Baseline

- Select one bounded workload.
- Record representative small, medium, and large batches.
- Measure sequential wall time, allocation, output hashes, diagnostic order,
  and merge cost.
- Identify which results are semantic and which measurements are observational.

Acceptance:

- the workload has repeatable structural outputs;
- work independence and ordered commit requirements are written down;
- the baseline runs on native and, where currently supported, WASM.

### Slice 1: Decompose Work Without Threads

- Refactor the selected workload into bounded input-to-output work items.
- Keep execution sequential.
- Associate every result and diagnostic with stable work identity.
- Make merge/commit a separate explicit step.

Acceptance:

- output and diagnostic expectations match the pre-refactor baseline;
- work items do not borrow ambient mutable world or provider state;
- no executor abstraction is public merely to complete this slice.

### Slice 2: Add Runtime-Internal Sequential Coordination

- Add the smallest runtime-owned coordination path needed by the caller.
- Support an explicit one-worker/sequential policy.
- Define lifecycle, drain, failure aggregation, and global budget configuration
  for the bounded scope.
- Emit diagnostics identifying the selected execution mechanism and why.

Acceptance:

- identical results are produced through direct and coordinated sequential
  paths;
- zero/invalid budgets fail or normalize explicitly;
- application shutdown leaves no queued work or hidden worker state.

### Slice 3: Add One Native Parallel Mechanism

- Compare scoped threads, a retained pool, and an external executor only against
  the selected workload's measured needs.
- Implement one native mechanism behind the runtime/platform boundary.
- Preserve stable result association and ordered commit.
- Prevent nested domain pools and cap total workers at the application budget.

Acceptance:

- sequential and parallel structural outputs are identical;
- diagnostics remain deterministically associated even if completion order
  differs;
- repeated startup/shutdown is clean;
- small workloads may select sequential execution when parallel overhead loses;
- measurement demonstrates where the parallel path helps or hurts.

### Slice 4: Exercise Failure, Cancellation, And Affinity

- Inject work-item failures at deterministic identities.
- Define cancellation boundaries for queued, computing, completed, and
  committing work.
- Prove main-thread-only commit or submission where the selected workload needs
  it.
- Surface queue depth, active workers, completed work, failures, and merge time.

Acceptance:

- partial failure cannot silently commit incomplete state;
- shutdown and cancellation behavior is deterministic at semantic boundaries;
- platform affinity is expressed without exposing native thread handles.

### Slice 5: Add A Second Independent Consumer

- Apply the same internal execution policy to a second production or tool
  workload from a different domain boundary.
- Reuse only the parts genuinely shared by both callers.
- Record any domain-specific behavior that must not enter the common contract.

Acceptance:

- the second consumer does not require domain terms in the common execution
  model;
- application-wide budgeting coordinates both consumers;
- the common surface is smaller or clearer after the second use, not merely
  more generic.

### Slice 6: Decide Public API And Crate Placement

- Review whether any execution types belong in `tokimu-core`.
- Review which orchestration surface, if any, should become public from
  `tokimu-runtime`.
- Decide whether the native mechanism remains internal to `tokimu-platform`.
- Update the primitive ledger only for meanings proven irreducible by callers.

Acceptance:

- one Architectural Review cycle records the resulting API and ownership
  evidence;
- every public type has at least one non-test caller and focused tests;
- no external executor or platform-native type leaks through public domain APIs.

### Slice 7: Evaluate Browser Workers Separately

- Keep the normal WASM path sequential while investigating worker support.
- Record browser security/header, module loading, transfer, shared-memory,
  startup, and bundling constraints.
- Add workers only if the same Tokimu-owned contract survives without creating a
  browser-specific runtime.

Acceptance:

- WASM remains semantically correct without workers;
- worker availability and selection are explicit diagnostics;
- browser mechanisms do not alter domain or execution-policy meaning.

### Slice 8: Reconsider Parallel Simulation Only With New Evidence

- Do not change `System::run(&mut World)` as part of earlier slices.
- Open a new Architectural Review if real simulation pressure requires parallel
  world access.
- Require an access/dependency model, conflict diagnostics, deterministic-enough
  commit policy, and migration path before changing the system contract.

Acceptance:

- none within this plan; this slice is a guarded reopening path, not scheduled
  implementation.

## Validation

For affected slices, prefer:

- exact structural output and fingerprint comparisons;
- deterministic diagnostic identity and ordering checks;
- repeated initialization, drain, and shutdown tests;
- injected failure and cancellation tests;
- one-worker versus multi-worker equivalence;
- worker-budget and oversubscription tests;
- native/WASM compilation and sequential parity;
- observational timing across multiple workload sizes.

Performance results must name target, build profile, workload revision, worker
count, and whether setup or retained execution was measured.

## Risks

### Premature General Executor

Mitigation: keep the first coordination path internal and bounded to one proven
workload; generalize only after a second independent consumer.

### Nondeterministic Diagnostics Or Commit

Mitigation: carry stable work identity through execution and merge results in a
defined semantic order.

### Oversubscription

Mitigation: one runtime-owned application budget; domains express demand but do
not create ambient pools.

### Fine-Grained Work Costs More Than It Saves

Mitigation: measure granularity and allow explicit sequential selection and
batching.

### WASM Architecture Fork

Mitigation: preserve a sequential WASM path and keep worker mechanisms behind
the same contracts.

### Parallelism Leaks Into World Ownership

Mitigation: begin with owned input-to-output transformations and retain
exclusive world mutation until a separate review admits a safer model.

## Completion Criteria

This plan is complete for its initial scope when:

- two independent workloads use one runtime-owned execution policy;
- one sequential and one native parallel mechanism produce identical semantic
  results;
- application-wide budgets and lifecycle are tested;
- deterministic result association and required commit order are proven;
- failure, cancellation, and affinity behavior are diagnostic;
- WASM remains correct through the sequential policy;
- a review decides the smallest justified public API and crate placement;
- parallel simulation remains unchanged unless separately admitted.

## References

- `docs/ADR/ADR-0006-native-execution-policy.md`
- `docs/Architectural Reviews/AR-0002-native-execution-and-multithreading.md`
- `docs/Conversations/multithreading.md`
- `docs/kernel-principles.md`
- `docs/Tokimu Software Design Document.md`
- `docs/ADR/ADR-0001-engine-boundaries.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `crates/tokimu-core/src/schedule.rs`
- `crates/tokimu-core/src/system.rs`
