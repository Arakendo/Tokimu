# Performance Diagnostics And Runtime Observation

## Status

Active. Slices 1-6 are complete. Slice 7 has a structured report consumer but
still lacks visual object attribution. The narrow kernel diagnostics foundation
is implemented and accepted by ADR-0007; broader runtime observation remains
incubating under AR-0005.

## Purpose

Make performance pressure observable through shared Tokimu contracts so corpus
examples and future applications do not repeatedly invent timing, counters,
warning latching, and evidence capture.

The plan gathers evidence for the broader architectural question without
assuming that profiling, telemetry, aggregation, resource diagnostics, and
editor presentation belong to one subsystem.

## Governing Boundary

```text
producer
    measures a fact

application/tool policy
    supplies an explicit budget

kernel diagnostics
    captures structured evidence
    emits degraded/recovered transitions

consumer
    presents, exports, compares, or visualizes
```

- Kernel diagnostics own provider-neutral records, bounded capture, ordering,
  and deterministic transition semantics.
- Runtime owns frame-lifecycle integration and only proven cross-domain
  coordination.
- Capabilities own domain-specific metric meaning.
- Providers own mechanism-specific measurements.
- Applications and tools own budgets, severity policy, visualization, and
  author guidance.

## Goals

- Give all examples immediate access to structured performance diagnostics.
- Distinguish facts, configured policy, derived diagnosis, and presentation.
- Make metric cadence and lifetime semantics explicit.
- Localize performance pressure to the narrowest honest ownership boundary.
- Build deterministic corpus cases for warning, recovery, bounds, and
  attribution.
- Gather enough evidence to resolve AR-0005 without prebuilding a profiler.

## Non-Goals

- Flame graphs, CPU sampling, GPU captures, or vendor profiling APIs.
- A complete telemetry storage or export platform.
- A universal FPS, memory, triangle, texture, audio, or upload budget.
- Automatically blaming an asset from temporal correlation alone.
- Persisting runtime metrics into `.tasset` before an asset report contract is
  reviewed.
- Editor overlays or red-outline debug presentation in the initial slices.
- Promoting one broad `tokimu-observation` crate before independent consumers
  stabilize the meaning.

## Existing Evidence

`hello-cgm` demonstrated:

- visible lag from per-draw GPU resource allocation;
- 5517 initial binding allocations and zero steady-state allocations after
  renderer retention;
- presentation command construction CPU duration around 0.7-1.2 ms;
- persistent reported draw/submission pressure;
- CPU wall duration inside the renderer `present()` call around 47-59 ms in
  the observed debug run;
- successful warning capture after three consecutive budget violations;
- responsive window movement after the allocation fix.

This proves that shared diagnostics can reveal pressure and confirm recovery.
It does not prove that the renderer call duration identifies one causal stage,
that it measures GPU completion, or that one general observation schema fits
assets and execution.

## Slice 1: Structured Kernel Diagnostic Foundation

### Deliverables

- [x] Add structured diagnostic severity, kind, source, message, and sequence.
- [x] Add bounded capture with dropped-record accounting.
- [x] Preserve existing startup-message access.
- [x] Add provider-neutral numeric performance observations and units.
- [x] Add sustained budget violation, warning latching, and recovery.
- [x] Keep measurement mechanisms out of `tokimu-core`.

### Acceptance Criteria

- [x] A single slow observation does not emit when the configured consecutive
      threshold is greater than one.
- [x] Sustained violations emit exactly one degraded warning.
- [x] Continued violations do not flood diagnostics.
- [x] Returning within budget emits one recovery record.
- [x] Capture remains bounded and reports dropped records.
- [x] Unit tests run without a window, renderer, filesystem, or network.

## Slice 2: Runtime Frame Integration

### Deliverables

- [x] Add an explicit opt-in runtime frame-time budget.
- [x] Feed runtime-owned frame delta observations into kernel diagnostics.
- [x] Keep the default runtime free of an assumed FPS target.
- [x] Test structured frame-budget evidence.
- [ ] Add accessors or capture routing only when a second consumer needs them.

### Acceptance Criteria

- [x] An application can configure and clear its frame-time budget.
- [x] Runtime emits through the same kernel diagnostic stream.
- [x] Unconfigured applications produce no performance warnings.
- [x] Frame timing remains observational and does not alter simulation policy.

## Slice 3: Renderer And UI Counter Semantics

### Deliverables

- [x] Record draw calls, submit calls, binding allocations, uniform writes,
      mesh uploads, and mesh replacements.
- [x] Retain stable instance and camera uniform bindings.
- [x] Record UI presentation construction and renderer present duration in
      `hello-cgm`.
- [x] Document each counter as per-frame, cumulative, high-water, or lifetime.
- [x] Reset per-frame counters at one verified frame boundary.
- [x] Split renderer timing into command preparation, command encoding, queue
      submission, and surface acquisition/presentation where observable.
- [x] Separate CPU duration from GPU completion and frame pacing.
- [x] Add diagnostics for repeated layout or geometry generation with unchanged
      semantic input.

### Acceptance Criteria

- [x] Two consecutive reports cannot disagree about counter cadence.
- [x] A steady static scene reports zero resource creation and zero unchanged
      uploads after warm-up.
- [x] Timing labels state exactly what interval and mechanism they measure.
- [x] No metric claims GPU execution time without a GPU timing mechanism.
- [x] The first divergent stage identifies the owning diagnostic boundary.

`RenderStats` now exposes cadence structurally:

```text
frame
    draw calls
    submit calls
    binding allocations
    uniform writes
    mesh uploads
    mesh replacements

lifetime
    binding allocations
    uniform writes
    mesh uploads
    mesh replacements
```

`begin_frame` resets only frame counters. Lifetime resource counters survive
that boundary. Deterministic renderer tests verify that a second static frame
reports zero resource creation and uploads while retaining lifetime totals.

The `hello-cgm` renderer timer is CPU wall time around the complete
`renderer.present()` call. It can include surface acquisition, resource
preparation, command encoding, queue submission calls, surface presentation,
and provider or driver pacing. It is not a GPU execution or completion timer.
The `wgpu` backend additionally reports each of those CPU-observable phases
separately. Unsupported phases remain `None` rather than being reported as
zero.

`hello-cgm` now owns an explicit presentation revision. Rebuilding its command
stream repeatedly without a revision change emits a structured
`hello-cgm.presentation` warning and a later revision change emits recovery.
This localizes the first known unchanged-input repetition to application-side
presentation construction rather than inferring it from renderer draw counts.

Slice 3 is complete. Batching and retained semantic presentation remain
follow-up optimization evidence, not missing counter semantics.

## Slice 4: Performance Diagnostics Corpus

### Deliverables

- [x] Add an example-side support library or focused test module only after two
      corpus cases share setup.
- [x] Add deterministic healthy, transient-spike, sustained-pressure, recovery,
      and bounded-overflow cases.
- [x] Add a static presentation case comparable to `hello-cgm`.
- [x] Add a repeated-upload case that must trigger resource pressure.
- [x] Add a stable-resource case that must remain quiet after warm-up.
- [x] Emit machine-readable observation and diagnostic artifacts.
- [x] Record build profile, target, workload revision, budgets, and algorithm
      identity with measured evidence.

### Acceptance Criteria

- [x] Corpus results distinguish expected warning, expected silence, and
      unsupported measurement.
- [x] Repeated runs produce the same diagnostic transition sequence for
      deterministic inputs.
- [x] Structural artifacts are authoritative; screenshots remain complementary
      visual evidence.
- [x] Performance values are not golden-tested across machines unless the test
      controls the mechanism.
- [x] The corpus can detect a deliberately reintroduced per-draw allocation.

`examples/lib-example/performance-diagnostics-corpus` now owns the shared
example-side setup for nine deterministic cases:

```text
diagnostics
    healthy
    transient spike
    sustained pressure
    recovery
    bounded overflow

renderer
    stable resources after warm-up
    repeated binding allocation
    repeated mesh upload
    unsupported GPU completion time
```

The renderer cases consume explicit `RenderStats` snapshots. They do not launch
a GPU provider or compare machine wall time. The stable case preserves
lifetime totals while frame resource counters return to zero. The two
regression cases deliberately keep frame allocation or upload counters above a
zero-after-warm-up policy and must emit one latched warning.

Each JSON artifact records:

- schema, producer, case identity, build profile, and target;
- workload revision and monitor/policy algorithm identity;
- the configured source, metric, budget, unit, and sustained threshold;
- ordered observations and renderer counter snapshots;
- structured diagnostics, transition order, capture capacity, and dropped
  records;
- expected and actual outcome, including explicit unsupported measurement.

Controlled count fixtures and transition sequences are authoritative corpus
evidence. Native-window screenshots may complement a real application run, but
they do not replace these structural assertions. Machine timing remains
observational until a separately controlled benchmark contract exists.

## Slice 5: Resource Lifecycle Observation

### Deliverables

- [x] Select one non-renderer producer, preferably asset loading or execution
      queueing.
- [x] Define the smallest resource event vocabulary required by that producer.
- [x] Observe created, loaded/prepared, replaced, and released lifecycle points
      where they are factual.
- [x] Attach stable Tokimu identity without exposing provider-native objects.
- [x] Record bytes, duration, counts, or dependencies only when the producer
      can measure them honestly.
- [x] Compare event observations with numeric samples before generalizing one
      schema.

### Acceptance Criteria

- [x] The producer owns measurement and domain terminology.
- [x] Kernel-facing records remain provider-neutral.
- [x] Missing measurement is explicit rather than reported as zero.
- [x] Resource identity remains stable for the observed lifetime.
- [x] No renderer, platform, importer, or foreign object enters `tokimu-core`.

`tokimu-assets` is the first non-renderer producer. `AssetStore` returns
ordered lifecycle observations for allocation, preparation, replacement, and
release. An `AssetId` remains stable while replacement advances a
provider-neutral generation. The handle does not expose loader, filesystem,
network, decoded asset, or renderer-native objects.

The existing `hello-glb` application consumes allocation and preparation
observations from its real model load. The performance corpus exercises the
complete lifecycle and records:

- sequence, asset identity, generation, transition, and optional source;
- `None` for bytes and duration because `AssetStore` does not measure either;
- explicit errors for lifecycle operations on unknown assets.

This slice rejects a premature universal observation schema. Numeric
performance samples describe a measured value with cadence and units.
Resource lifecycle observations describe ordered state transitions over stable
identity. They may share bounded capture or artifact infrastructure later, but
their current semantic records remain distinct.

Implementing this slice also exposed an accidental generic bound on
`AssetHandle<T>`. Handles are now unconditionally copyable and comparable by
their opaque `AssetId`; payload traits no longer affect identity semantics.

## Slice 6: Aggregation Evidence

### Deliverables

- [x] Identify which consumers actually need last, total, average, peak,
      percentile, or rolling-window values.
- [x] Implement one aggregation outside the kernel first unless universal
      ownership is already proven.
- [x] Record sample cadence, window size, reset behavior, and missing data.
- [x] Compare frame aggregation with resource-lifetime aggregation.
- [x] Report findings to AR-0005.

### Acceptance Criteria

- [x] Aggregation does not erase raw evidence required for diagnosis.
- [x] Window and reset semantics are deterministic and documented.
- [x] Unbounded history is not retained by default.
- [x] The implementation demonstrates whether aggregation is kernel, runtime,
      capability, or tool-owned rather than assuming the answer.

The performance corpus is the first aggregation consumer. Its controlled
numeric comparisons need count, last, total, average, and peak. No current
consumer justifies percentile calculation. Each summary covers only the raw
samples retained by one case, records its window size and controlled-step
cadence, and resets per case.

Resource lifecycle aggregation remains a separate type. It reports transition
counts, final active resources, and last generation over the retained event
sequence. Numeric summaries do not reinterpret resource events as samples, and
resource summaries do not invent units or cadence.

Artifact schema 2 preserves raw samples and events alongside both summaries.
The current evidence places aggregation in corpus/tool code. It does not
justify kernel-owned rolling windows, runtime-owned global totals, or
unbounded telemetry history.

## Slice 7: Diagnostic Attribution And Tool Consumption

### Deliverables

- [x] Add one non-console structured diagnostic consumer.
- [x] Test attribution to one stable asset, entity, resource, subsystem, or
      task identity.
- [x] Keep classification policy outside renderer presentation.
- [x] Prototype an author-facing explanation before a red-outline overlay.
- [ ] Ensure an emphasized object links to a specific diagnostic and cause.
- [x] Distinguish individual cost from collective scene/region cost.

### Acceptance Criteria

- [x] A tool consumes structured records without parsing human-readable text.
- [x] Presentation can change without changing producer or kernel semantics.
- [x] Every presented policy state maps to explicit configured policy.
- [x] Selecting a diagnostic reveals metric, observation, budget, source, and
      relevant identity.
- [x] No vague "expensive" marker appears without actionable evidence.

The performance corpus now builds and persists a separate diagnostic report.
This consumer derives explanations exclusively from structured diagnostic
kind, sequence, source, metric, observation, budget, and unit. A test replaces
all human messages with unrelated prose and proves that the report remains
identical.

Current attribution is deliberately limited to the stable producer subsystem
named by the diagnostic source. Reports label that scope as
`collective-subsystem`, retain the originating diagnostic sequence, and mark
causal status as `not-inferred`. They provide a next action directed at the
owning producer rather than calling an asset, entity, or draw "expensive"
without evidence.

No visual object is emphasized yet, so the object-to-diagnostic-and-cause
deliverable remains open. That requires a real UI consumer with independently
proven object identity; the JSON report is intentionally not pretending to be
that evidence.

## Slice 8: Architectural Review And Graduation

### Deliverables

- [x] Summarize independent producer and consumer evidence in AR-0005.
- [x] Decide whether events and samples share one contract.
- [x] Decide initial aggregation ownership.
- [x] Decide current resource attribution ownership.
- [ ] Split the review if profiler, telemetry, leak detection, or asset reports
      prove distinct seams.
- [x] Accept, defer, reject, or further incubate broader capability admission.
- [x] Update the SDD and ADRs only for findings supported by evidence.

### Acceptance Criteria

- [ ] Examples no longer duplicate the admitted diagnostic mechanics.
- [ ] The accepted boundary has at least two independent producers and one
      non-console consumer.
- [ ] Provider details do not leak through public semantic APIs.
- [ ] Native and WASM targets can preserve the semantic contract even when
      measurements differ.
- [ ] Unsupported observations and target limitations are explicit.
- [ ] The disposition names concrete reopening triggers.

AR-0005 currently records a deliberate `Incubating` disposition:

- numeric samples and resource lifecycle events remain distinct contracts;
- the first aggregation belongs to its corpus/tool consumer;
- asset lifecycle identity remains capability-owned;
- stable subsystem source is sufficient for current reports but does not prove
  universal asset/entity/task subject attribution;
- ADR-0007 remains the accepted narrow kernel boundary.

Graduation remains open because visual object attribution, target-parity
validation, and a non-corpus application/tool consumer have not yet supplied
the remaining independent evidence.

## Metric Naming Guidance

Metric names should describe the measured fact, not an inferred cause:

```text
Good:
platform-reported frame interval
presentation command construction CPU duration
renderer present call CPU wall duration
queue submit calls
binding allocations
mesh upload bytes

Avoid:
bad frame
expensive asset
renderer slow
problem mesh
```

Derived diagnostics may explain likely causes only when supporting observations
and attribution are retained.

## Risks

### Diagnostic Overhead Becomes The New Performance Problem

Mitigation: bounded capture, explicit sampling cadence, transition-based
warnings, and corpus measurement of diagnostics-disabled versus enabled cost.

### Counters Mix Per-Frame And Lifetime Meaning

Mitigation: make cadence part of every metric contract and validate reset
boundaries in Slice 3.

### Policy Leaks Into The Kernel

Mitigation: applications/tools supply budgets; kernel code does not hardcode
asset or frame thresholds.

### Correlation Is Reported As Causation

Mitigation: preserve stage observations and only derive causes from explicit
identity/dependency evidence.

### One Observability Subsystem Absorbs Everything

Mitigation: keep AR-0005 open and split reviews when event capture,
aggregation, profiling, persistence, or presentation demonstrate distinct
ownership.

## Completion Criteria

This plan is complete when:

- ADR-0007 remains implemented and tested;
- counter cadence and timing semantics are explicit;
- deterministic corpus cases protect warning and recovery behavior;
- at least one resource producer and one non-console consumer provide
  independent evidence;
- AR-0005 reaches a recorded disposition;
- docs and code agree about observation, policy, aggregation, and presentation
  ownership.

## References

- `docs/Architectural Reviews/AR-0005-runtime-observation-and-performance-telemetry.md`
- `docs/ADR/ADR-0007-kernel-performance-diagnostics.md`
- `docs/Conversations/On Performence Metrics.md`
- `docs/Notes/ui-presentation-performance-evidence.md`
- `docs/testing-strategy.md`
- `crates/tokimu-core/src/diagnostics.rs`
- `crates/tokimu-runtime/src/app.rs`
- `crates/tokimu-render/src/renderer.rs`
- `crates/tokimu-render/src/wgpu_backend.rs`
- `examples/hello-cgm/`
