# AR-0002: Native Execution and Multithreading Ownership

| Field | Value |
| --- | --- |
| Status | Accepted |
| Opened | 2026-07-23 |
| Last reviewed | 2026-07-23 |
| Scope | Kernel / runtime / platform / cross-cutting |
| Trigger | Independent presentation, asset, physics, and networking work needs one coherent execution owner |
| Related ADRs | ADR-0001, ADR-0003, ADR-0005, ADR-0006 |
| Related evidence | `docs/Conversations/multithreading.md`, current schedule/system APIs, native/WASM requirements |
| Admission exception | Permanent evidence substitution under ADR-0005 |

## Architectural Question

Should Tokimu natively own execution policy and cross-domain coordination while
runtime and platform adapters provide sequential, threaded, or worker-based
execution mechanisms?

## Context

Tokimu already treats schedules and time as trusted-kernel concerns. Several
domains can produce independent work: presentation geometry, font outlines,
asset decoding, physics islands, and network decoding. If each domain creates
its own pool, Tokimu cannot coherently control total concurrency, priority,
shutdown, diagnostics, or target-specific availability.

The current `System` contract receives exclusive `&mut World` access. That is a
useful deterministic baseline and is not evidence that general simulation
systems are ready for parallel execution.

The architectural question is therefore about ownership, not an immediate job-
system API or a promise that every workload runs on multiple threads.

## Trigger And Evidence

- Corpus examples: presentation geometry, SVG, font-outline, and mixed UI
  workloads already decompose into independent transformations.
- Current code: `Schedule`, phases, priorities, and system dependencies are
  Tokimu-owned; `System::run` currently has exclusive world access.
- Cross-domain pressure: assets, presentation, future physics, and future
  networking all benefit from shared resource budgeting and diagnostics.
- Platform pressure: native threads and browser workers have materially
  different availability, startup, transfer, and affinity constraints.
- Missing evidence: no production executor contract, measured native speedup,
  Web Worker adapter, or parallel world-mutation model exists yet.

## Ownership Analysis

The universal meaning is execution policy:

- dependency and readiness;
- permitted independence;
- deterministic commit order where observable;
- task identity and diagnostic attribution;
- budgets, affinity requirements, shutdown, and cancellation boundaries.

This meaning is kernel-native because it coordinates multiple subsystems and
must not fragment into incompatible domain vocabularies.

Runtime orchestration owns dispatch, joining, budget enforcement, and lifecycle
coordination. Platform adapters own native threads, browser workers, and target
capability discovery. Domain providers own their transformations and expose
independent work; they do not own ambient worker pools. Render backends retain
ownership of GPU synchronization and submission.

Execution policy must not own simulation truth, domain algorithms, provider
state, or renderer-native objects.

## Dependency Direction

```text
Domain capability/provider
        |
        v
Tokimu-owned work and dependency contract
        |
        v
tokimu-runtime orchestration
        |
        +---- calls an engine-owned mechanism contract
                           ^
                           |
              tokimu-platform implementation
              supplied at application composition

Results
        |
        v
ordered domain/runtime commit where required
```

`tokimu-core` must not depend on Rayon, operating-system thread APIs, browser
worker APIs, or an async runtime. Platform mechanisms depend on Tokimu-owned
contracts rather than defining them. `tokimu-runtime` must not depend on
`tokimu-platform`; the application or facade composition root supplies a
mechanism implementation to runtime orchestration.

## Alternatives Considered

### Domain-Owned Executors

- Benefits: domains can optimize locally and adopt libraries independently.
- Costs: oversubscription, duplicated lifecycle and diagnostics, inconsistent
  target behavior, and no application-wide budget.
- Failure mode: several pools compete for the same cores while no owner can
  explain or constrain total execution.

### Executor As An Optional Capability

- Benefits: appears replaceable and keeps the early runtime small.
- Costs: treats universal coordination as optional domain meaning and permits
  incompatible scheduling vocabularies.
- Failure mode: foundational and capability crates cannot rely on one coherent
  execution policy.

### Platform-Owned Threading

- Benefits: target-specific mechanisms are naturally platform concerns.
- Costs: platform code would own dependency, ordering, and determinism semantics
  that should remain stable across targets.
- Failure mode: native and browser builds become different runtimes rather than
  adapters executing the same policy.

### Native Policy With Runtime/Platform Mechanisms

- Benefits: one semantic contract, one resource budget, target portability,
  deterministic sequential testing, and replaceable mechanisms.
- Costs: requires careful separation between policy, orchestration, and
  mechanism.
- Failure mode: an over-general executor API could be stabilized before a real
  workload proves it.

## Findings

The ownership evidence is sufficient even though the API and implementation
evidence are incomplete. Scheduling is already native Tokimu meaning, and
cross-domain execution cannot be coherently delegated to any one optional
capability.

The evidence does not justify parallel simulation, a public executor trait, a
specific dependency such as Rayon, or Web Worker support today. Those remain
implementation-plan questions.

Under ADR-0005, the multiple independent domain pressures, platform parity
requirement, and need for a global budget substitute for waiting until three
production capability crates each implement and then duplicate their own pool.
The accountable project maintainer accepts the API-shape uncertainty while
keeping implementation provisional under the linked plan. Failure to preserve
dependency inversion or semantic parity is a reopening trigger.

## Disposition

Accepted. Tokimu natively owns execution policy and coordination semantics.
Runtime orchestration and platform adapters provide execution mechanisms.
ADR-0006 records the binding boundary; implementation remains incremental under
the native execution and multithreading plan.

## Consequences

- Domain crates and providers must not create ambient private worker pools as
  their public execution model.
- Sequential execution remains first-class for testing, small workloads, and
  constrained targets.
- Parallel completion may be nondeterministic, but observable commit order must
  be defined where semantics require it.
- Parallel world mutation is not admitted by this review.
- Thread/executor dependencies remain outside `tokimu-core`.
- Diagnostics should eventually attribute work, queueing, and failures without
  exposing platform-native worker objects.

## Required Follow-Up

- [x] Record the decision in ADR-0006.
- [x] Create a focused implementation plan.
- [ ] Select one measured independent workload for the first sequential
  decomposition and native parallel proof.
- [ ] Preserve identical semantic results and diagnostics across sequential and
  parallel execution.
- [ ] Keep WASM sequential until worker support is explicitly designed and
  tested.

## Reopening Triggers

- a supported target cannot preserve the Tokimu-owned execution contract;
- a concrete workload proves that the policy/mechanism boundary causes
  unacceptable copying, latency, or ownership inversion;
- a domain requires a private executor for correctness rather than optimization;
- parallel world mutation becomes necessary and cannot be expressed as
  independent compute followed by ordered commit;
- implementation evidence reveals a simpler ownership decomposition.

## Review History

### Cycle 1 -- 2026-07-23

- Status entering review: Proposed
- New evidence: archived multithreading design discussion, current schedule and
  system contracts, presentation/asset workload decomposition, and native/WASM
  constraints.
- Participants or reviewers: project maintainer and Codex as critical review
  assistant.
- Findings: execution policy is universal engine meaning; concrete threading is
  runtime/platform mechanism; API shape remains unproven.
- Disposition: Accepted.
- Resulting ADR or documentation change: ADR-0006 and
  `docs/Plans/native-execution-and-multithreading.md`.

## References

- `docs/Conversations/multithreading.md`
- `docs/kernel-principles.md`
- `docs/Tokimu Software Design Document.md`
- `crates/tokimu-core/src/schedule.rs`
- `crates/tokimu-core/src/system.rs`
- `docs/ADR/ADR-0001-engine-boundaries.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
- `docs/ADR/ADR-0006-native-execution-policy.md`
