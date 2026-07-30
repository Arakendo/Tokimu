# AR-0005: Runtime Observation And Performance Telemetry

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-07-29 |
| Last reviewed | 2026-07-29 |
| Scope | Kernel diagnostics, foundational runtime observation, capability metrics, and tooling policy |
| Trigger | Multiple corpus examples required temporary performance instrumentation; `hello-cgm` exposed severe renderer allocation and submission pressure |
| Related ADRs | ADR-0001, ADR-0003, ADR-0005, ADR-0006, ADR-0007 |
| Related evidence | `hello-cgm`, renderer frame statistics, runtime timing diagnostics, UI performance evidence note |
| Admission exception | None |

## Architectural Question

Should Tokimu admit a first-party runtime observation capability for shared
resource and execution telemetry, and if so, which parts belong to the kernel,
runtime, capabilities, providers, policy, and presentation tools?

## Context

Several examples have become slow enough to require investigation. The repeated
workflow has been:

```text
example becomes slow
    -> add temporary counters and timers
    -> identify one local cause
    -> remove or abandon the instrumentation
    -> repeat in another example
```

That is evidence of missing shared observation infrastructure, not evidence
that every example should become its own profiler.

The first focused investigation in `hello-cgm` found that the renderer created
instance and camera uniform resources per draw, per frame. Retaining those
bindings removed visible window-interaction lag. The same investigation also
recorded persistent presentation pressure:

```text
steady-state binding allocations: 0
presentation command construction CPU duration: approximately 0.7-1.2 ms
reported draw count: 5516
submit calls: 76
renderer present call CPU wall duration: approximately 47-59 ms
```

The renderer-call duration does not claim GPU execution or completion time. It
measures CPU wall time around the provider call and may include surface
acquisition, resource preparation, command encoding, queue submission calls,
surface presentation, and backend or driver pacing.

Kernel diagnostics now support bounded structured records and sustained
performance-budget transitions. ADR-0007 accepts that narrow boundary. This
review remains broader: it asks whether resource events, rolling aggregation,
asset attribution, cross-capability telemetry, and diagnostic presentation
form one capability or several related consumers.

## Trigger And Evidence

- Corpus examples: `hello-cgm` directly benefits from shared frame,
  presentation, and renderer observations. Prior UI, SVG, glyph, GLB, CGM, and
  networking corpus work has repeatedly required local counters or timing.
- Automated tests: `tokimu-core` tests bounded diagnostic capture, sustained
  budget violations, warning latching, and recovery. `tokimu-runtime` tests
  opt-in frame budgets.
- Audits or diagnostics: `docs/Notes/ui-presentation-performance-evidence.md`
  records the confirmed per-draw GPU allocation defect and remaining
  submission pressure.
- Independent consumers: runtime frame timing and renderer/UI timing already
  produce independent measurements through one diagnostic vocabulary.
- Repeated implementation friction: examples should demonstrate their domain,
  not repeatedly reconstruct profiling infrastructure.
- Missing evidence: resource lifetime events, memory accounting, upload
  attribution, rolling aggregation, sampling policy, identity association,
  persistent asset reports, and non-console diagnostic presentation.

`tokimu-assets` now supplies the first non-renderer resource-lifecycle
evidence. It returns ordered allocation, preparation, replacement, and release
observations over one stable `AssetId`; replacement advances a generation.
The contract does not expose provider-native resources, and measurements the
asset store cannot make remain absent rather than becoming zero-valued facts.

## Questions Under Review

### What Is The Unit Of Observation?

Candidates include:

- event: resource created, uploaded, replaced, or destroyed;
- sample: frame duration, queue depth, resident bytes, or draw count;
- interval summary: average, percentile, peak, or accumulated total;
- attributed diagnostic: an observation associated with an asset, entity,
  subsystem, task, or frame.

The initial implementation supports numeric samples against explicit budgets.
That does not yet establish one universal observation schema.

### Who Owns Aggregation?

Bounded capture and degraded/recovered state transitions are kernel diagnostic
semantics. It remains unresolved whether rolling averages, histograms,
percentiles, causal correlation, and long-lived telemetry storage belong in:

- the kernel;
- runtime coordination;
- a dedicated foundational observation capability;
- diagnostic tools; or
- specialized capability providers.

### Who Owns Policy?

The kernel must not decide that a model with a particular triangle count or a
texture of a particular size is universally bad.

Current candidate boundary:

```text
capability/provider
    measures facts

application/tool configuration
    supplies budgets and policy

kernel diagnostics
    records observations and deterministic state transitions

editor/debug presentation
    explains and visualizes the result
```

### How Is Cost Attributed?

Useful author diagnostics may need identity associations:

- asset;
- entity;
- resource handle;
- subsystem;
- frame;
- execution task;
- dependency graph or scene region.

The first diagnostic record supports a source and metric but does not yet prove
the correct general identity model.

## Ownership Analysis

The stable meaning already supported is:

- a producer-neutral metric name, numeric observation, unit, and explicit
  budget;
- bounded structured diagnostic capture;
- warning and recovery transitions under sustained pressure;
- deterministic ordering through diagnostic sequence identity.

ADR-0007 assigns those semantics to kernel diagnostics.

Candidate broader ownership:

- `tokimu-core`: provider-neutral observation identity, diagnostic records,
  bounded capture, and only the smallest universally useful transition
  semantics.
- `tokimu-runtime`: frame lifecycle integration, cross-domain collection,
  cadence, and any application-wide observation coordination proven necessary.
- capabilities: domain facts such as vertices, bones, decoded samples, bodies,
  queue depth, or import complexity.
- providers/backends: mechanism facts such as upload duration, resident GPU
  bytes, compilation time, or platform queue behavior.
- applications and tools: budgets, severity policy, author guidance,
  visualization, persistence, export, overlays, and red-outline debug views.

The kernel must not own wall-clock mechanisms, GPU queries, font/model/audio
semantics, renderer resources, editor UI, asset-specific budget defaults, or a
particular telemetry backend.

## Dependency Direction

```text
Current:

runtime/UI/renderer measurements
        -> tokimu-core diagnostic semantics
        -> example console capture

Candidate:

capability/provider facts
        -> provider-neutral observations
        -> bounded kernel/runtime collection
        -> application policy and diagnostic tools
        -> console/editor/overlay/export presentation
```

No platform timer, GPU query object, tracing subscriber, editor type, or foreign
asset object may enter the kernel contract.

## Alternatives Considered

### A: Keep Metrics Inside Each Example

- Benefits: no new shared contract.
- Costs: repeated instrumentation, incompatible naming, and no automatic
  benefit for new examples.
- Failure mode: every lag investigation begins with rebuilding counters.

### B: Put All Profiling In The Renderer

- Benefits: renderer already owns many visible costs.
- Costs: cannot represent runtime, networking, assets, audio, import, physics,
  or headless observations honestly.
- Failure mode: renderer becomes the accidental owner of application-wide
  performance truth.

### C: Admit A Complete Profiler Service Now

- Benefits: one ambitious place for timing, aggregation, budgets, tracing, and
  UI.
- Costs: combines observation, aggregation, interpretation, persistence, and
  presentation before their boundaries are proven.
- Failure mode: a broad observability junk drawer becomes permanent kernel
  architecture.

### D: Accept Narrow Diagnostics And Incubate General Observation

- Benefits: preserves the proven structured diagnostic boundary while corpus
  tests discover the wider schema and ownership.
- Costs: some instrumentation remains provisional.
- Failure mode: temporary metric names become accidental public contracts
  unless the review stays active.

## Findings

The evidence supports the narrow decision in ADR-0007:

- measurement belongs to the mechanism or capability that owns the fact;
- kernel diagnostics own provider-neutral capture and deterministic diagnostic
  state transitions;
- budgets are explicit application/tool policy, not universal kernel opinion;
- presentation belongs outside the kernel.

The evidence also supports continued corpus work. It does not yet prove:

- one general runtime-observation event schema;
- kernel-owned rolling aggregation;
- universal asset/resource attribution;
- persistent telemetry or asset report formats;
- red/amber severity policy;
- one service combining profiling, tracing, leak detection, and author
  diagnostics.

Resource lifecycle evidence strengthens the case for shared capture and tool
consumption, but it also demonstrates that events and numeric samples should
not yet share one semantic record. A lifecycle transition has stable identity,
generation, and ordering; a performance sample has a measured value, unit, and
cadence.

The first aggregation consumer is the deterministic corpus, not the kernel.
Numeric cases need count, last, total, average, and peak over one bounded
per-case window. Resource cases need transition counts, final active resources,
and last generation. Raw evidence remains present. No current consumer
justifies percentiles, global rolling windows, or unbounded history.

The first non-console consumer is also corpus/tool-owned. It produces
author-facing JSON reports from structured fields without parsing diagnostic
messages. Current attribution stops at stable subsystem source identity and
labels the cost as collective. Reports retain diagnostic sequence and mark
causality as not inferred rather than inventing an asset or entity blame model.

## Disposition

Incubating. Keep ADR-0007 as the binding narrow diagnostics boundary. Continue
collecting resource, execution, renderer, and example evidence through
`docs/Plans/performance-diagnostics-and-runtime-observation.md`. Do not admit a
general profiler, telemetry crate, asset-budget model, or editor presentation
contract until the corpus identifies stable shared meaning.

## Consequences

New examples can immediately use one bounded diagnostic vocabulary and explicit
performance budgets. Producers remain responsible for measurement accuracy and
units. Tools can capture structured warnings without parsing console strings.

The project must distinguish facts from policy and avoid treating one measured
backend result as a portable guarantee. The broader observation architecture
remains deliberately revisable.

## Required Follow-Up

- [x] Record the narrow accepted boundary in ADR-0007.
- [x] Add bounded structured diagnostic capture and sustained budget monitors.
- [x] Integrate runtime frame timing behind an explicit opt-in budget.
- [x] Capture frame, presentation, and renderer timing in `hello-cgm`.
- [x] Verify counter cadence and distinguish per-frame from lifetime totals.
- [x] Add a deterministic performance-diagnostics corpus.
- [x] Add one resource-lifecycle producer outside the renderer.
- [x] Test identity attribution without exposing provider-native objects.
- [x] Evaluate initial aggregation ownership from measured corpus pressure.
- [x] Add one non-console consumer before proposing broader admission.

## Reopening Triggers

This review should advance or be re-scoped when:

- two independent capability providers require the same observation event
  schema;
- a resource-lifecycle producer and an execution producer share stable identity
  and aggregation semantics;
- a non-example editor or tool needs structured observations;
- target-specific measurement objects leak into the provider-neutral contract;
- the bounded diagnostic stream cannot support required capture volume;
- corpus evidence proves that observation, aggregation, and interpretation need
  separate Architectural Reviews;
- persisted asset reports or scene overlays require a stable attribution model.

## Review History

### Cycle 1 -- 2026-07-29

- Status entering review: Proposed
- New evidence: repeated example-side instrumentation, `hello-cgm` lag,
  retained renderer binding fix, renderer frame counters, kernel performance
  budget diagnostics, and responsive window interaction after the fix.
- Participants or reviewers: project author, Monday review, Codex implementation
  review.
- Findings: the narrow diagnostic boundary is stable enough for ADR-0007;
  general runtime observation and resource telemetry remain under-specified.
- Disposition: Incubating.
- Resulting ADR or documentation change: ADR-0007 and the performance
  diagnostics/runtime observation implementation plan.

### Cycle 2 -- 2026-07-29

- Status entering review: Incubating
- New evidence: deterministic renderer counter corpus cases, an asset
  lifecycle producer in `tokimu-assets`, real allocation/preparation
  consumption in `hello-glb`, and complete lifecycle artifact coverage.
- Participants or reviewers: project author and Codex implementation review.
- Findings: stable provider-neutral resource identity is viable without
  entering `tokimu-core`; unknown measurements remain explicitly absent;
  lifecycle events and numeric samples do not yet justify one semantic schema.
- Disposition: Incubating. Proceed to aggregation and non-console consumption
  evidence.
- Resulting ADR or documentation change: no new ADR; Slice 5 findings recorded
  in the active plan and this review.

### Cycle 3 -- 2026-07-29

- Status entering review: Incubating
- New evidence: schema-2 corpus artifacts with bounded numeric and resource
  lifecycle summaries while retaining their ordered raw evidence.
- Participants or reviewers: project author and Codex implementation review.
- Findings: the first aggregation belongs to its corpus/tool consumer;
  numeric-window and resource-lifetime summaries have different semantics;
  percentiles and global rolling history remain unsupported.
- Disposition: Incubating. Do not promote aggregation into kernel or runtime
  until an independent consumer proves common ownership.
- Resulting ADR or documentation change: Slice 6 completed without a new ADR.

### Cycle 4 -- 2026-07-29

- Status entering review: Incubating
- New evidence: a persisted JSON diagnostic report consumer built solely from
  structured fields, with tests proving independence from human message text.
- Participants or reviewers: project author and Codex implementation review.
- Findings: tool-owned explanation can evolve without producer or kernel
  changes; subsystem attribution is currently honest and collective; object
  attribution and causal diagnosis remain unsupported.
- Disposition: Incubating. A real UI/tool object-selection consumer remains
  necessary before accepting universal subject attribution.
- Resulting ADR or documentation change: Slice 7 tool-consumption evidence
  recorded without expanding ADR-0007.

## References

- `docs/Conversations/On Performance Metrics.md`
- `docs/Notes/ui-presentation-performance-evidence.md`
- `docs/Plans/performance-diagnostics-and-runtime-observation.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
- `docs/ADR/ADR-0006-native-execution-policy.md`
- `docs/ADR/ADR-0007-kernel-performance-diagnostics.md`
- `crates/tokimu-core/src/diagnostics.rs`
- `crates/tokimu-runtime/src/app.rs`
- `crates/tokimu-render/src/renderer.rs`
- `corpus/hello-cgm/`
