# ADR-0006: Native Execution Policy

## Status

Accepted

## Context

Tokimu owns schedules, phases, time progression, and system dependencies.
Presentation, asset processing, future physics, and future networking can all
produce independent work that may benefit from parallel execution.

Treating multithreading as a domain-owned feature would allow each subsystem to
create its own pools, budgets, lifecycle, diagnostics, and target assumptions.
Treating it solely as a platform concern would make native threads or browser
workers define engine-visible ordering and completion semantics.

Tokimu also targets WebAssembly, deterministic tests, and small workloads where
parallel execution may be unavailable or slower than sequential execution.
Threads are therefore one execution mechanism, not the semantic contract.

AR-0002 found sufficient cross-cutting evidence to decide ownership while
leaving the concrete executor API and first implementation workload open.

## Decision

Tokimu natively owns execution policy and cross-domain execution coordination.

> Capabilities describe dependency. Tokimu execution policy exploits
> independence. Platforms provide mechanisms.

The ownership boundary is:

### Trusted kernel semantics

Tokimu-owned contracts define, only as concrete workloads require:

- work identity and dependency/readiness semantics;
- whether work is permitted to execute independently;
- observable ordering and deterministic commit requirements;
- affinity or execution constraints expressed without platform-native handles;
- budget, cancellation, shutdown, and diagnostic vocabulary shared across
  domains.

These contracts remain engine-neutral. `tokimu-core` must not create operating-
system threads or browser workers and must not depend on Rayon, a general async
runtime, or platform threading APIs.

### Runtime orchestration

`tokimu-runtime` owns application-wide execution coordination:

- selecting ready work according to Tokimu policy;
- applying the configured global worker and queue budgets;
- dispatching, joining, and draining work;
- coordinating parallel compute with ordered or phase-bound commit;
- selecting sequential execution when required;
- reporting execution diagnostics and lifecycle failures.

The precise API is deferred until a concrete workload proves it.

### Platform execution mechanisms

`tokimu-platform` and target-specific adapters own mechanism:

- native thread or thread-pool integration;
- browser worker integration when explicitly supported;
- target capability discovery;
- main-thread or worker affinity enforcement;
- target-specific wake-up, transfer, and shutdown behavior.

Platform objects do not cross into domain or engine-owned semantic APIs.
Platform adapters implement an engine-owned mechanism contract and are supplied
at the application or facade composition root. `tokimu-runtime` does not depend
on `tokimu-platform`.

### Domain responsibilities

Foundational services, optional capabilities, and providers:

- expose bounded deterministic work and its dependencies where natural;
- preserve domain semantics under sequential execution;
- define result validation and commit behavior;
- avoid ambient domain-owned worker pools;
- do not expose threads, locks, or executor-specific types as domain meaning.

A backend may use internal concurrency required by an external library, but it
must respect Tokimu's declared budgets and lifecycle where integration permits,
must diagnose material exceptions, and must not redefine Tokimu execution
semantics.

### Determinism and world mutation

Sequential execution is a first-class policy, not a fallback error.

Parallel work may complete in any order unless a contract states otherwise.
Observable commit order must be explicitly stable wherever ordering affects
world state, diagnostics, serialization, replay, or other engine semantics.

This decision does not admit parallel mutation of `World`. The existing
exclusive `&mut World` system boundary remains valid. Parallel simulation
requires separate evidence and a design that proves safe access,
deterministic-enough commit, diagnostics, and scheduling behavior.

## Consequences

Tokimu gains one ownership model for execution across rendering preparation,
asset processing, geometry, physics, networking, and future domains. The
application can eventually control total concurrency and inspect execution
without every subsystem inventing a scheduler.

Native and WASM targets preserve the same semantic contracts even when one uses
multiple threads and another executes sequentially. Performance is
observational and target-dependent, not a semantic guarantee.

The project must resist stabilizing a generic executor too early. Initial
implementation should begin with deterministic work decomposition and a
sequential mechanism, then add one measured native parallel path.

## References

- `docs/Architectural Reviews/AR-0002-native-execution-and-multithreading.md`
- `docs/Plans/native-execution-and-multithreading.md`
- `docs/Conversations/multithreading.md`
- `docs/kernel-principles.md`
- `docs/Tokimu Software Design Document.md`
- `docs/ADR/ADR-0001-engine-boundaries.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
