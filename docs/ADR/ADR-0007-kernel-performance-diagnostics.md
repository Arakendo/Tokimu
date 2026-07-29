# ADR-0007: Kernel Performance Diagnostics

## Status

Accepted

## Context

Multiple Tokimu examples have required temporary timing and resource counters
to explain poor responsiveness. `hello-cgm` provided concrete cross-layer
evidence:

- UI presentation construction was inexpensive;
- renderer resource allocation was repeated per draw and per frame;
- removing steady-state binding allocation materially improved interaction;
- remaining frame and renderer-present budgets were still exceeded;
- local console strings alone were insufficient as reusable diagnostic
  evidence.

Diagnostics are already kernel-native under ADR-0003. The architectural
question for this decision is narrower than a profiler or general telemetry
service: who owns the semantic contract for reporting and capturing sustained
performance pressure?

AR-0005 records the broader unresolved questions around resource events,
aggregation, identity attribution, asset reports, and diagnostic presentation.

## Decision

Tokimu kernel diagnostics own a provider-neutral contract for bounded
structured diagnostic capture and sustained performance-budget state
transitions.

> Producers measure facts. Applications supply policy. Kernel diagnostics
> capture meaning. Tools present evidence.

### Producers Own Measurements

The subsystem that owns a mechanism or domain fact produces its measurement:

- runtime measures frame lifecycle timing;
- UI measures layout or presentation construction;
- render backends measure submissions, uploads, allocations, and GPU-facing
  work where observable;
- asset/model/audio/physics capabilities measure their domain-specific facts;
- platform adapters measure target-specific mechanisms.

The kernel does not read platform clocks, issue GPU queries, inspect foreign
asset objects, or infer capability-specific metrics.

### Applications And Tools Own Budgets

Performance acceptability is application- and target-dependent. Applications,
test corpora, editors, or tools explicitly configure budgets and warning
policy. Tokimu does not assume a universal frame rate, memory limit, triangle
limit, texture size, or upload threshold.

The kernel may provide deterministic transition machinery that evaluates an
explicitly supplied budget:

```text
healthy
    -> required consecutive violations
    -> degraded warning
    -> suppress repeated warnings while degraded
    -> observation returns within budget
    -> recovered diagnostic
```

This machinery classifies observations against supplied policy; it does not
invent the policy.

### Kernel Diagnostics Own Capture Semantics

Kernel-owned diagnostic records may contain:

- stable sequence identity;
- severity and diagnostic kind;
- provider-neutral source and metric names;
- human-readable context;
- observed numeric value;
- explicit budget;
- provider-neutral unit;
- bounded retention and dropped-record accounting.

Capture must remain usable headlessly and without a renderer, window, tracing
backend, filesystem, or network connection.

### Runtime Owns Lifecycle Integration

`tokimu-runtime` may feed frame lifecycle observations into kernel diagnostics
when an application explicitly configures a budget. Runtime does not impose a
default FPS target.

Future cross-domain collection cadence and aggregation require evidence under
AR-0005 and are not admitted by this ADR.

### Tools Own Presentation

Console output, editor panels, graphs, overlays, red outlines, persisted
reports, telemetry export, and author guidance consume structured diagnostics.
They do not redefine the measurements or make presentation concepts
kernel-native.

## Non-Decisions

This ADR does not admit:

- a general profiler or tracing framework;
- rolling averages, histograms, percentiles, or causal analysis;
- an asset-cost schema;
- automatic resource leak detection;
- persistent telemetry storage;
- editor or overlay APIs;
- hardcoded asset or frame budgets;
- GPU framebuffer captures or vendor-specific profiling objects;
- one universal event schema for all runtime observations.

Those remain evidence questions in AR-0005.

## Dependency Direction

```text
runtime / capability / provider measurements
        -> tokimu-core diagnostic records and transitions
        -> application or tool consumers
```

`tokimu-core` remains free of platform timers, renderer dependencies, external
profilers, logging subscribers, asset importers, and editor UI.

## Consequences

Every example and capability can use one diagnostic vocabulary without
rebuilding warning latching, recovery, bounded capture, or console-string
parsing. New examples benefit immediately when they opt into relevant budgets.

Measurements remain observational and target-dependent. A warning proves that
one supplied observation exceeded one configured budget; it does not establish
a portable performance guarantee or causal diagnosis.

The narrow contract can evolve independently from broader observation tooling.
Breaking changes to diagnostic meaning require architectural review. Producers
may improve measurement mechanisms without architectural review if they
preserve the provider-neutral contract.

## References

- `docs/Architectural Reviews/AR-0005-runtime-observation-and-performance-telemetry.md`
- `docs/Plans/performance-diagnostics-and-runtime-observation.md`
- `docs/Notes/ui-presentation-performance-evidence.md`
- `docs/Conversations/On Performence Metrics.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0006-native-execution-policy.md`
- `crates/tokimu-core/src/diagnostics.rs`
- `crates/tokimu-runtime/src/app.rs`
- `corpus/hello-cgm/`
