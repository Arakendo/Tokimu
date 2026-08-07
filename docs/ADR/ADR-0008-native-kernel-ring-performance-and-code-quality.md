# ADR-0008: Native Kernel Ring Performance and Code Quality

## Status

Accepted

## Context

Tokimu's architecture has a small trusted core surrounded by foundational
services, optional capabilities, backends, presentation adapters, and tools.
The trusted part is unusually expensive to change: a needless allocation,
duplicate abstraction, platform dependency, or unclear ownership rule can be
copied into every application and can weaken native/WASM parity.

Tokimu therefore needs an admission discipline that is stricter for the Native
Ring than for code whose responsibility is to adapt, present, or integrate a
specialized capability. The discipline must catch ordinary engineering errors
as well as performance regressions. It must not turn every outer adapter into a
kernel ceremony or require a universal benchmark for code that is not on a
measured path.

This ADR complements, rather than replaces:

- ADR-0003, which decides what meaning belongs to Native Tokimu;
- ADR-0006, which decides ownership of execution policy and mechanisms; and
- ADR-0007, which defines the provider-neutral performance diagnostic contract.

## Decision

Tokimu adopts two proportional admission gates: a full gate for the Native
Ring and a minimum gate for Outer Rings.

### Ring terminology

The rings describe architectural ownership, not the compilation target or
whether a process uses OS-native APIs.

**Native Ring** means engine-owned universal meaning and trusted coordination:
world and resource invariants, schedules and time, commands/signals/events,
diagnostics, stable identity, capability contracts, and other contracts
explicitly admitted as native under ADR-0003. This commonly includes code in
`tokimu-core` and engine-owned portions of `tokimu-runtime`, but a crate name
alone does not decide the ring.

**Outer Rings** means code that consumes or adapts Native Ring contracts:
foundational platform/render/assets/input implementations, optional capability
crates, backend adapters, presentation lowering, corpus consumers, website
islands, and tools. An Outer Ring can still be performance-critical. A hot path
must use the full performance section below. A process-wide mechanism or a
change that alters a Native Ring contract must use the full gate regardless of
its directory.

The ring classification must be recorded in the change description when it is
not obvious. Reclassifying a concept into the Native Ring remains subject to
ADR-0003 and the admission evidence rules in ADR-0005.

### Full Native Ring gate

Every non-mechanical implementation, dependency, behavior, or API change to the
Native Ring must answer the following checklist before merge. A maintainer may
mark an item not applicable, but must record why.

The checkboxes below are a reusable review template, not incomplete work in
this ADR. The applicable answers belong in the change, pull request, review
record, or another retained artifact rather than being checked permanently in
this decision document.

#### Ownership and boundary

- [ ] The change names the Native Ring invariant or contract it serves.
- [ ] The change does not introduce capability-specific meaning, a backend
      object, a window/GPU/filesystem/network dependency, or a target-only
      assumption into an engine-neutral boundary.
- [ ] Existing abstractions and vocabulary were searched before adding a new
      type, trait, function family, or service.
- [ ] A serious decomposition or reuse attempt was made; duplicated concepts
      are not admitted under a new name.
- [ ] Ownership, authority, lifetime, mutation phase, and failure behavior are
      explicit at the boundary.

#### Performance and determinism

- [ ] The likely hot paths, asymptotic complexity, allocation behavior, copy
      behavior, lock/queue behavior, and external calls are identified.
- [ ] The change has a before/after measurement, bounded benchmark, or a
      written reason that measurement would add no decision value.
- [ ] Steady-state work does not allocate, clone, format, parse, or perform
      avoidable I/O without evidence that the cost is acceptable.
- [ ] Work and data have bounded sizes or explicit budget behavior; an
      unbounded queue, recursion path, allocation total, or diagnostic stream
      is not introduced.
- [ ] Scheduling, fixed-step behavior, ordering, and commit semantics remain
      deterministic wherever the existing contract requires them.
- [ ] Native and WASM consequences were considered, including the sequential
      path when parallel execution is unavailable or slower.
- [ ] Any performance claim is tied to a workload, target, build profile, and
      measurement artifact. No local timing is presented as a universal
      guarantee.
- [ ] Sustained budget pressure uses the diagnostic vocabulary from ADR-0007;
      ad hoc logging or a hidden profiler is not added to the kernel.

#### Code hygiene and maintainability

- [ ] `cargo fmt --all` passes for affected Rust code.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes, or the
      exception is recorded with a narrowly scoped reason.
- [ ] Relevant tests exercise the changed contract, including rejection and
      boundary cases where applicable.
- [ ] Public APIs, diagnostics, and migration impact are documented.
- [ ] No duplicated functions, parallel implementations, copy-pasted policy,
      dead compatibility path, or shadow source of truth remains.
- [ ] Names describe semantic meaning rather than a temporary implementation
      or provider mechanism.
- [ ] Unsafe code, synchronization, interior mutability, and global state are
      absent or individually justified with an invariant and a testable safety
      boundary.
- [ ] Errors remain explicit and structured; silent fallback, panic-based
      control flow, and swallowed diagnostics are not used for ordinary input.

#### Evidence and review

- [ ] The change lists the validation commands and their results.
- [ ] A benchmark, profile, size report, trace, or other artifact is retained
      when the change claims a performance effect.
- [ ] The change identifies reopening triggers: workload growth, a failed
      target-parity check, a budget regression, or evidence that ownership is
      wrong.
- [ ] A not-applicable answer or narrow validation exception records its local
      reason. Any waiver of admission evidence, ownership, or stability review
      uses ADR-0005 rather than an informal exception.

The full gate is a review contract, not a demand that every change optimize
every possible machine. Its purpose is to make costs and ownership visible
before a Native Ring decision becomes an ecosystem-wide dependency.

### Minimum Outer Ring gate

Outer Ring changes must satisfy this smaller checklist:

- [ ] The change identifies the Native Ring contract it consumes and does not
      mutate or redefine that contract implicitly.
- [ ] Dependencies and foreign objects remain behind the documented adapter or
      capability boundary.
- [ ] The changed path has a basic test, smoke check, or corpus evidence
      appropriate to its risk.
- [ ] `cargo fmt`/the applicable formatter and the local lint or typecheck pass.
- [ ] Failures, unsupported behavior, and lifecycle cleanup remain visible;
      outer code may choose a degraded presentation but must not hide the
      underlying diagnostic.
- [ ] Obvious duplicate logic is avoided. Outer code may use a small local
      adapter helper when that is clearer than prematurely stabilizing a shared
      Native Ring abstraction, but it must not create competing semantic truth.
- [ ] If the change is on a measured hot path, changes process-wide behavior,
      or claims a performance improvement, the full performance section above
      is required.

Outer Rings are allowed to move faster, experiment, and retain local
implementation helpers. They are not allowed to bypass ownership, lifecycle,
diagnostic, or dependency boundaries merely because they are outside the
kernel.

### Review proportionality

Documentation-only changes, mechanical formatting, and regeneration that does
not change source inputs, tool versions, commands, or artifact identity do not
require a new performance measurement. A behavior change still requires the
gate for its ring even when the diff is small.

When a change crosses rings, the stricter gate applies to the crossing and to
the Native Ring contract it modifies. When classification is uncertain, treat
the change as Native until the ownership question is resolved.

### Checklist maintenance

The checklist is itself subject to evidence. Maintainers should periodically
sample completed change records and revise items that consistently produce
ritual `not applicable` answers or never affect a decision. Items may be
combined, clarified, or removed when doing so preserves the gate's ownership,
performance, determinism, hygiene, and evidence outcomes.

New checklist items should respond to a recurring failure mode or retained
project evidence rather than personal preference. Changing the checklist must
update this ADR or supersede it explicitly; local review custom must not
silently weaken or expand the accepted gate.

## Non-Decisions

This ADR does not:

- define one universal frame rate, memory limit, binary-size limit, or latency
  target;
- make `tokimu-core` depend on a profiler, benchmark framework, logger,
  platform timer, renderer, or executor;
- require all Outer Ring code to be free of duplication or local helpers;
- admit parallel mutation of `World`;
- replace ADR-0005's evidence and maintainer-exception process;
- create a general performance telemetry, tracing, or code-ownership service.

## Consequences

Native changes acquire a repeatable pre-merge discipline covering architecture,
cost, determinism, hygiene, and evidence. This makes it harder to add a
convenient duplicate API, an accidental steady-state allocation, or a
platform-specific shortcut to the trusted core.

Outer Rings retain enough flexibility for provider experiments and application
work, while hot paths and contract crossings cannot evade performance review by
being placed in an adapter crate. The cost is additional review work for
Native Ring changes and occasional measurement work where a qualitative review
would previously have been accepted.

## References

- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
- `docs/ADR/ADR-0006-native-execution-policy.md`
- `docs/ADR/ADR-0007-kernel-performance-diagnostics.md`
- `docs/kernel-principles.md`
- `docs/semantic-kernel-map.md`
- `docs/contribution-admission-guide.md`
- `docs/Tokimu Software Design Document.md`
