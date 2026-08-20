# ADR-0009: Ring-Based Verification, Failure Containment, and Recovery

## Status

Accepted

## Context

ADR-0008 applies a higher engineering burden to Native Ring changes because a
mistake in universal engine meaning or trusted coordination propagates across
the ecosystem. Performance and code hygiene are only part of that burden. A
fast, tidy kernel can still be unsafe to evolve when its success paths are
tested narrowly, malformed inputs can panic, diagnostics lose the original
failure, or a failed operation leaves world and resource state half-mutated.

Tokimu already treats unit, integration, target, golden, and corpus validation
as different forms of evidence. It also distinguishes recoverable `error`
diagnostics from run-ending `fatal` diagnostics and prefers explicit failure
over silent fallback. Those principles need a pre-merge gate that establishes:

- which contracts are tested at which boundary;
- how realistic corpus evidence complements narrow automated tests;
- how errors retain identity, context, and provenance;
- how failures are contained before they corrupt unrelated state; and
- when recovery is genuinely supported, deliberately unavailable, or fatal.

The gate must remain proportional. A corpus experiment should not need the same
recovery proof as a Native Ring transaction or scheduler change. Outer code
must still handle external inputs, lifecycle failure, and boundary errors
honestly.

ADR-0017 specializes this gate for terminal outcome closure. A process, page,
worker, renderer, device domain, or window that disappears without a causal
Tokimu record or independently retained termination observation is an immediate
conformance failure, not an ordinary error path.

## Decision

Tokimu adopts a full verification and resilience gate for Native Ring changes
and a minimum gate for Outer Ring changes. ADR-0008 defines the ring
terminology; this ADR uses the same ownership classification.

> Tests prove contracts. Corpus proves composition. Diagnostics preserve
> failure meaning. Containment protects unrelated state. Recovery is a tested
> transition, not an optimistic log message.

The checkboxes below are reusable review templates. Applicable answers and
validation results belong in a change, pull request, Architectural Review
Record, or another retained artifact. Unchecked boxes in this ADR do not
represent incomplete implementation work.

### Validation evidence has distinct jobs

No single test category substitutes for all the others:

- **Unit tests** prove local invariants and bounded algorithms at the smallest
  honest boundary.
- **Contract and integration tests** prove public behavior without privileged
  access to implementation details.
- **Workspace and target tests** prove crate composition, lifecycle, and
  native/WASM seams.
- **Architectural corpus tests** prove that intended boundaries compose into a
  focused application without duplicated engine mechanics.
- **Data corpus tests** pressure parsers, importers, renderers, and providers
  with representative, pinned, and provenance-recorded inputs.
- **Regression tests** retain the smallest reproduction of a defect and its
  expected failure or corrected behavior.
- **Property, fuzz, and fault-injection tests** explore input or failure spaces
  where a few hand-authored cases provide weak evidence.
- **Golden or snapshot tests** retain reviewed deterministic artifacts; they do
  not establish correctness merely because output stayed unchanged.

Tests should assert engine-owned meaning, stable diagnostic identity, and
observable state. Tests that freeze private layout or replaceable provider
mechanics require a local reason.

### Full Native Ring gate

Every non-mechanical implementation, dependency, behavior, or API change to the
Native Ring must answer the applicable sections below before merge. A
maintainer may mark an item not applicable, but must record why.

#### Contract and unit evidence

- [ ] The change identifies the invariant, success behavior, rejection
      behavior, and externally observable state it promises.
- [ ] Local algorithms and state transitions have unit tests at the narrowest
      honest boundary.
- [ ] Public or cross-crate behavior has a contract or integration test that
      uses only the supported public surface.
- [ ] Boundary values, empty values, maximum admitted values, stale identities,
      invalid ordering, and unsupported operations are tested where relevant.
- [ ] A corrected defect retains a focused regression test that fails for the
      original reason, rather than only adding a broad end-to-end scenario.
- [ ] Tests control time, randomness, registration order, filesystem order,
      concurrency, and other nondeterministic inputs that affect the contract.
- [ ] Test-only helpers do not reproduce Native Ring semantics or become a
      second runtime, scheduler, parser, or source of truth.

#### Corpus and composition evidence

- [ ] A new or materially changed Native Ring abstraction has at least one real
      caller or focused corpus proof appropriate to its architectural claim.
- [ ] The corpus evidence states whether it proves engine semantics,
      composition, one backend, one target, or only a visual/manual observation.
- [ ] Representative data inputs are pinned and record origin, license,
      expected role, and relevant hashes where external data is retained.
- [ ] Corpus tests include important malformed or unsupported inputs when the
      boundary consumes data outside Tokimu's trust.
- [ ] Golden updates are explicit and reviewed; an ordinary test run cannot
      silently rewrite expected results.
- [ ] Corpus success is backed by narrower automated assertions where
      practical. A runnable example alone does not prove every internal
      invariant it happens to exercise.

#### Error capture and diagnostic evidence

- [ ] Recoverable failures return structured errors or emit structured
      diagnostics according to the owning contract; they are not swallowed,
      converted only to strings, or reported solely through console output.
- [ ] `error` and `fatal` classifications match whether coherent execution can
      continue. A fatal condition is not mislabeled to keep a test green.
- [ ] Error and fatal diagnostics retain stable identity, owning subsystem,
      operation or source identity, and enough bounded context to reproduce or
      route the failure.
- [ ] Diagnostic assertions prefer stable code, class, severity, and structured
      context over exact human wording unless wording is the public contract.
- [ ] Capture and retention are bounded and expose dropped-record behavior;
      repeated failure cannot create an unbounded diagnostic stream.
- [ ] Sensitive or arbitrarily large user data is not copied into diagnostics,
      panic messages, crash artifacts, or test snapshots without an explicit
      redaction and size policy.
- [ ] Native and WASM presentation edges make fatal startup or runtime failure
      visible rather than leaving a silent process, window, canvas, or loading
      state.
- [ ] Every started lifecycle observation closes as success, structured
      rejection/fatal, independently observed external termination, or an
      explicit unresolved disappearance; disappearance is never inferred to be
      success or one guessed cause.
- [ ] A survival or crash-containment claim uses a liveness/terminal observer
      outside the failure domain whose loss is under test.

#### Failure containment and state integrity

- [ ] Ordinary invalid input, missing resources, stale handles, unsupported
      targets, and provider rejection do not panic or abort the process.
- [ ] Mutation is validated before commit or uses an explicit transactional
      boundary. Failure leaves prior valid state intact or produces a documented
      partial state that can still be inspected safely.
- [ ] A failed capability, provider, plugin, task, or resource does not corrupt
      unrelated world state or silently invalidate unrelated handles.
- [ ] Early return, cancellation, timeout, initialization failure, and normal
      shutdown release owned resources and restore host state.
- [ ] Threads, workers, queues, callbacks, and in-flight work have explicit
      drain, cancellation, detach, or abandonment semantics.
- [ ] Unsafe and foreign-function boundaries validate inputs and ownership;
      unwinding does not cross a boundary that forbids it.
- [ ] Failure injection or an equivalent test reaches cleanup and partial-
      initialization paths that cannot be proven by the happy path.

#### Crash protection and recovery evidence

- [ ] The change states which failures are prevented, contained, recoverable,
      retryable, restartable, degradable, or fatal. These terms are not used
      interchangeably.
- [ ] A claimed recovery path is automated where practical and proves the
      post-recovery invariant, not merely that control flow continued.
- [ ] Retry and reinitialization are bounded, observable, and safe against
      duplicate commit or repeated side effects.
- [ ] Recovery from persistent or serialized state rejects incomplete,
      incompatible, or corrupt data without presenting it as valid state.
- [ ] A panic boundary, when deliberately used, documents unwind safety and the
      state that remains trustworthy. `catch_unwind` is not treated as general
      crash isolation.
- [ ] Irrecoverable states terminate through an explicit fatal path after
      preserving the bounded evidence that is safe and available.
- [ ] Crash-to-desktop, browser/page/worker disappearance, abort, renderer or
      GPU-process loss, and equivalent terminal loss without causal or external
      termination evidence immediately fail the affected conformance claim
      under ADR-0017.
- [ ] Subprocess or process-level isolation is used when a provider can abort,
      violate memory safety, or poison process state and the application claims
      continued operation after that failure.

#### Target, lifecycle, and regression evidence

- [ ] Focused affected-package tests and `cargo test --workspace` pass, or a
      narrowly scoped exception records why the workspace command is not an
      honest or available check for this change.
- [ ] Relevant native and WASM contracts are tested or compiled, and any
      target-specific limitation is reported rather than silently skipped.
- [ ] Startup, steady-state operation, cancellation, teardown, and restart are
      exercised in proportion to the lifecycle changed.
- [ ] Headless or sequential execution remains a first-class test path when the
      semantic contract does not require graphics, hardware, or parallelism.
- [ ] Hardware- or environment-dependent validation reports `skipped` or
      `unavailable` distinctly from `passed`.
- [ ] Applicable long-running corpus, golden, browser, backend, or hardware
      checks remain separately invocable and their executed or deferred status
      is reported explicitly.
- [ ] Validation commands, target/tool versions when material, and results are
      retained with the change.

### Minimum Outer Ring gate

Every non-mechanical Outer Ring behavior change must satisfy this smaller gate:

- [ ] The supported success path has a focused automated test, smoke check, or
      corpus proof at the boundary the change owns.
- [ ] At least one representative failure or unsupported path is exercised and
      remains visible through the owning error or diagnostic contract.
- [ ] External input is validated and bounded before it can allocate excessive
      resources, cross an unsafe/foreign boundary, or mutate Native Ring state.
- [ ] Initialization failure, cancellation where supported, and teardown do not
      leak owned resources or leave host state altered.
- [ ] Provider/backend errors remain distinguishable from Tokimu contract
      errors and do not redefine Native Ring semantics.
- [ ] The exercised run ends in an observed terminal category; unexplained
      process, page, window, worker, or device-domain disappearance fails the
      check immediately rather than becoming `skipped` or `unavailable`.
- [ ] Manual, visual, hardware, or target-specific evidence is labeled honestly
      and is not reported as an unattended automated pass.
- [ ] A fixed regression retains a focused test when practical.

Outer Ring experiments may use local fixtures and direct tests without first
creating shared test infrastructure. A disposable corpus executable may choose
to exit after a fatal error; reusable libraries and adapters must still return
or capture ordinary external-input failures without panicking.

### Escalation rules

An Outer Ring change must use the applicable full sections above when it:

- changes a Native Ring contract or invariant;
- parses untrusted or adversarially variable data;
- introduces unsafe code or a foreign-function boundary;
- owns persistent commit, reopen, migration, or recovery behavior;
- creates process-wide threads, workers, hooks, callbacks, or global state;
- claims isolation, restart, retry, or continued operation after provider
  failure; or
- fixes a defect that crossed a ring boundary or corrupted unrelated state.

The full gate applies to a ring crossing even when the implementation lives in
an adapter or application directory. When classification is uncertain, treat
the affected contract as Native until ownership is resolved.

### Recovery claims must be precise

Tokimu distinguishes:

```text
prevention   invalid or dangerous work never begins
containment  failure cannot corrupt unrelated state
capture      bounded evidence preserves what failed and where
recovery     a tested transition restores a documented valid state
degradation  operation continues with explicitly reduced capability
fatal        coherent execution cannot continue
```

One does not imply another. Capturing a panic is not recovery. Returning an
error is not containment if state was already corrupted. Restarting a provider
is not safe when its previous side effects may be committed twice.

Out-of-memory aborts, process termination, undefined behavior, non-unwinding
panics, and catastrophic device or host failures are not assumed recoverable.
If an application requires continued operation across such failures, the
responsible Outer Ring must provide and test an isolation boundary strong
enough for that claim.

Unknown termination remains unknown. Missing in-process evidence does not
authorize a diagnosis of OOM, device loss, panic, driver failure, or host fault.
ADR-0017 defines the required external observation and admission consequence.

### Review proportionality and checklist maintenance

Documentation-only changes, mechanical formatting, and deterministic
regeneration with unchanged source identity do not require new runtime tests.
A behavior change still requires the gate for its ring even when the diff is
small. Test count or line coverage alone is not evidence that the relevant
contract and failure modes were exercised.

Maintainers should periodically sample completed change records and revise
items that repeatedly produce ritual `not applicable` answers or never affect
a decision. New items should respond to recurring failures or retained project
evidence. Changes to this gate must update or supersede this ADR rather than
emerging as unwritten review custom.

## Non-Decisions

This ADR does not:

- mandate one code-coverage percentage or test-count target;
- require every local helper to have a separate unit test;
- require a corpus application for every bug fix or mechanical refactor;
- require fuzzing when a bounded direct test honestly covers the input space;
- forbid every panic inside Tokimu; an unreachable invariant failure remains
  distinct from ordinary input or provider rejection;
- make `catch_unwind` a universal recovery mechanism;
- guarantee recovery from allocation failure, abort, process kill, undefined
  behavior, GPU/driver loss, or host failure;
- add a crash reporter, telemetry uploader, test framework, or recovery service
  to `tokimu-core`;
- allow diagnostics or recovery work to mutate simulation truth implicitly; or
- replace the test placement and execution guidance in
  `docs/testing-strategy.md`.

## Consequences

Native Ring changes acquire an explicit reliability burden alongside ADR-0008's
performance and hygiene burden. New kernel meaning must be proven locally,
through its public contract, and through realistic composition where the claim
requires it. Failure behavior becomes part of the contract rather than an
afterthought.

Outer Rings retain room for direct tests, local fixtures, manual visual
evidence, and experiments. They cannot use their location to avoid input
validation, cleanup, diagnostic visibility, or stronger proof when they own an
unsafe, persistent, process-wide, or cross-ring failure boundary.

The cost is more deliberate failure-path testing and occasional subprocess,
fault-injection, target, or corpus infrastructure. The benefit is that Tokimu
can distinguish a feature that works in one demonstration from a contract that
fails predictably, preserves evidence, and leaves the engine in a known state.

## References

- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
- `docs/ADR/ADR-0006-native-execution-policy.md`
- `docs/ADR/ADR-0007-kernel-performance-diagnostics.md`
- `docs/ADR/ADR-0008-native-kernel-ring-performance-and-code-quality.md`
- `docs/ADR/ADR-0017-observable-terminal-failure-and-host-crash-conformance.md`
- `docs/testing-strategy.md`
- `docs/diagnostics-model.md`
- `docs/kernel-principles.md`
- `docs/Tokimu Software Design Document.md`
