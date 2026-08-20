# ADR-0017: Observable Terminal Failure And Host-Crash Conformance

## Status

Accepted — 2026-08-19

## Context

During repeated Doom browser/WebGPU map replacement, Microsoft Edge closed
while the user was moving through a map. Tokimu returned no Rust/WASM error,
no structured fatal diagnostic, and no device-loss record. The retained Edge
log contained unrelated browser warnings but did not identify an out-of-memory
condition, WGPU validation failure, GPU-process restart, or crash cause.

Later automated replacement runs completed successfully. The evidence therefore
does not establish why Edge closed, but it does establish a diagnostic failure:
the run ended without a Tokimu-visible terminal outcome.

ADR-0009 already requires native and WASM fatal failures to be visible and
requires irrecoverable states to preserve safe bounded evidence before explicit
termination. AR-0024 preserves the first causal callback failure. The draft
diagnostics model says `fatal` is the only severity allowed to end a run. None
of those statements fully covers a browser, renderer, GPU process, or native
process that takes its in-process diagnostic sinks down with it.

Treating such disappearance as an ordinary error, an unavailable observation,
or a browser peculiarity would make the most severe failures the least
observable ones. It would also weaken ADR-0011's availability boundary: a
provider able to terminate its host without retained evidence cannot support a
claim of contained failure merely because it runs behind a Rust adapter.

## Decision

Tokimu adopts **terminal outcome closure** as a cross-target diagnostic and
verification invariant.

> Every started Tokimu run or bounded operation must end in an observed success,
> an observed structured rejection/fatal outcome, or an independently observed
> external termination. Disappearance is never a successful error path.

### Terminal outcomes

At the boundary responsible for invoking or supervising work, every completed
observation belongs to exactly one of these categories:

```text
completed
    operation returned or the run shut down normally

rejected / fatal
    Tokimu preserved the first causal structured failure and stopped or
    continued according to its owning contract

externally terminated
    an independent host observed process, worker, page, renderer, GPU-process,
    or device termination before Tokimu could produce a terminal record

unresolved disappearance
    expected liveness ended or timed out, but neither Tokimu nor an independent
    observer retained enough evidence to classify the terminal boundary
```

`unresolved disappearance` is an immediate conformance failure. It blocks the
affected acceptance, admission, release, or recovery claim and reopens the
failure boundary. It must not be reported as `passed`, `skipped`, `unavailable`,
an ordinary returned error, or evidence of one guessed cause.

`externally terminated` is an observed outcome, not a fabricated Tokimu
diagnostic. When the cause is unknown, records say `cause=unknown`; they do not
invent OOM, device loss, driver failure, panic, or browser defect.

### First-cause and pre-termination behavior

When Tokimu still controls execution:

- ordinary invalid input, provider rejection, missing resources, and supported
  failure paths return structured errors or diagnostics rather than panicking,
  aborting, closing the host, or leaving a dead presentation surface;
- a fatal path preserves the first causal record before dispatch stops;
- secondary shutdown, presentation, or cleanup failures cannot replace the
  first cause;
- a visible native window or browser page presents a bounded fatal state when
  the host remains trustworthy enough to do so; and
- diagnostic presentation failure does not silently redefine the underlying
  operation as successful.

This decision does not require unsafe recovery from corrupted or poisoned
state. It requires honest termination and evidence.

### Independent observation for shared failure domains

An in-process sink cannot prove behavior after its own process, worker, page,
renderer, device callback domain, or event loop disappears. Tests that claim
crash containment, fatal visibility, browser survival, device-loss handling, or
long-running stability must place the relevant liveness and terminal observer
outside the failure domain being tested.

Depending on the claim, retained evidence may include:

- process identity, exit status, and unexpected-close observation;
- page/worker/renderer liveness and a bounded heartbeat or completion deadline;
- browser and GPU-process start/exit/restart observations;
- the last acknowledged Tokimu diagnostic sequence and operation identity;
- device-loss callbacks and provider diagnostics when delivered;
- bounded browser/native logs and crash-artifact presence;
- target, browser/runtime, adapter, backend, build profile, and corpus revision.

A console hook, DOM error panel, Rust panic hook, JavaScript `try/catch`, or WGPU
callback is useful but insufficient when it shares the failure domain whose
survival is under test.

### Immediate violation rule

Any crash-to-desktop, browser-window disappearance, page/worker termination,
renderer/GPU-process loss, abort, or equivalent terminal loss without a prior
causal Tokimu record or independent external-termination record is an immediate
violation of this ADR.

The immediate response is to:

1. stop claiming the affected path passed, recovered, or failed safely;
2. retain the last trustworthy operation, liveness, target, and host evidence;
3. reduce or isolate the reproduction without guessing a cause;
4. add an observer outside the suspected failure domain where practical; and
5. reopen the responsible diagnostic, containment, provider, resource, and
   security assumptions before admission resumes.

This rule applies even when the underlying defect is ultimately in a browser,
driver, operating system, third-party library, or hardware. Tokimu may not own
that mechanism, but Tokimu owns the honesty of its conformance claim.

### Ownership

- Producers preserve structured failure facts while they remain able to run.
- `tokimu-core` owns provider-neutral diagnostic identity and bounded capture,
  not browser/process supervision or crash-report mechanisms.
- Runtime preserves causal operation and shutdown ordering.
- Renderer and other providers expose delivered provider/device failure facts
  without inventing recovery or host causes.
- Platform adapters present fatal state and expose target lifecycle events that
  their host actually provides.
- Applications and corpus harnesses own continuation policy and supply
  out-of-domain supervision when their test or product claim requires it.
- Tools present and correlate retained evidence; they do not redefine unknown
  termination as a diagnosed cause.

Continued operation after a process-killing or state-poisoning failure requires
an isolation and recovery boundary strong enough to support that claim under
ADR-0009 and ADR-0011. This ADR does not make an in-process panic hook into such
a boundary.

## Required Diagnostic Principles

1. **No silent terminal state.** A dead process, page, worker, event loop,
   renderer, device, or canvas is an outcome requiring evidence, not an error
   presentation strategy.
2. **Preserve the first cause.** Later symptoms and cleanup failures cannot
   replace the earliest trustworthy causal record.
3. **Do not diagnose by absence.** Missing callbacks or logs establish missing
   evidence, not OOM, device loss, panic, or driver fault.
4. **Observe outside the failure domain.** Survival claims require an observer
   that can remain alive when the subject does not.
5. **Separate capture, presentation, containment, and recovery.** Any one may
   exist without the others; claims must name which was demonstrated.
6. **Bound retained evidence.** Heartbeats, logs, records, and crash artifacts
   need explicit size, time, and retention limits.
7. **Preserve provenance and privacy.** Retained terminal evidence identifies
   target and operation without copying arbitrary assets, secrets, or personal
   browser data.
8. **Fail admission immediately.** Unresolved disappearance blocks the affected
   claim until the boundary is instrumented and the result is classified.

## Consequences

- Browser/WASM success now requires more than the absence of a returned error.
  The page/window and relevant host process must survive, or an independent
  observer must retain termination evidence.
- Hardware/browser corpus tests may remain manual, but their terminal outcome
  and observability limits must be recorded explicitly.
- Tests that can kill their own diagnostic sink require process-, worker-, or
  browser-level harnessing proportional to the claim.
- An unexplained host close becomes a blocking failure even if a later run
  succeeds; successful reruns narrow reproducibility but do not retroactively
  make the earlier disappearance conformant.
- Providers remain replaceable and platform-specific. This decision admits no
  browser controller, crash uploader, process supervisor, telemetry service, or
  renderer-owned recovery policy into the kernel.

## Non-Decisions

This ADR does not:

- guarantee that Tokimu can catch an operating-system kill, browser crash,
  driver reset, abort, allocation failure, undefined behavior, or hardware
  failure in-process;
- assign an unknown Edge closure to memory exhaustion, WGPU, the GPU driver, or
  Tokimu;
- require every application to run under a permanent supervisor;
- mandate one browser automation framework, crash reporter, heartbeat period,
  timeout, or persisted log format;
- admit automatic restart, retry, device recreation, page reload, or degraded
  rendering;
- make console strings a substitute for structured diagnostic identity; or
- permit crash artifacts to retain secrets or unbounded user data.

## Verification

- Browser and native lifecycle fixtures must distinguish success, structured
  rejection/fatal return, externally observed termination, timeout/liveness
  loss, and observer unavailability.
- A focused fixture must prove that a returned Rust/WASM error reaches the host
  without closing the page/window.
- A controlled subject termination must be observed by a harness outside the
  subject failure domain and reported as termination rather than a fabricated
  engine diagnostic.
- First-cause preservation must remain covered when shutdown would otherwise
  dispatch secondary work.
- Long-running renderer/resource tests must retain page/window/process survival
  evidence alongside provider diagnostics and logical resource observations.

## Post-Decision Validation Case

The incident that prompted this ADR later produced a useful validation of the
decision's restraint. External supervision separated three outcomes that had
previously looked alike to the operator:

1. an Edge launcher handed a URL to an existing session and exited before page
   acknowledgement;
2. a retained-session Doom rotation completed and the supervisor deliberately
   closed its owned browser during cleanup; and
3. an acknowledged Doom walkabout ended when Edge performed an orderly,
   code-zero browser shutdown before Tokimu emitted a terminal record.

The third run retained no Crashpad dump, WGPU/device-loss, OOM, fatal, or crash
record. Subsequent input audit found that the browser workbench mapped descend
to `Ctrl` and forward to `W`, colliding with Edge's reserved `Ctrl+W` close
shortcut. Browser descend now uses `C`.

This is strong evidence for a host-input explanation, but the precise causal
link remains pending an exact reproduction/falsification run. The earlier
records are therefore not retroactively relabeled as GPU, browser, input, or
renderer failures beyond what each observer actually established.

The case demonstrates why terminal outcome closure is independent of the
eventual defect mechanism:

```text
unexplained disappearance
    -> preserve unknown cause
    -> add out-of-domain observation
    -> distinguish launcher handoff, observer cleanup, and orderly host exit
    -> discover a radically different causal candidate without rewriting
       missing evidence as an earlier diagnosis
```

Had the original disappearance been labeled WebGPU OOM from silence, the
browser-reserved shortcut collision would have been hidden behind a false
renderer diagnosis. ADR-0017 remains applicable even if the shortcut is
confirmed as the full cause: the invariant concerns honest terminal evidence,
not whether the eventual defect belongs to Tokimu, its host, or its controls.

## References

- `docs/diagnostics-model.md`
- `docs/testing-strategy.md`
- `docs/Tokimu Software Design Document.md`
- `docs/ADR/ADR-0001-engine-boundaries.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0007-kernel-performance-diagnostics.md`
- `docs/ADR/ADR-0009-ring-based-verification-failure-containment-and-recovery.md`
- `docs/ADR/ADR-0011-ring-based-security-authority-and-trust-boundaries.md`
- `docs/Architectural Reviews/AR-0024-renderer-failure-observation-and-diagnostic-boundary.md`
- `docs/Plans/Renderer-Reliability/renderer-scene-resource-lifetime-and-replacement.md`
