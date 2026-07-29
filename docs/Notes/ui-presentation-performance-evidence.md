# UI Presentation Performance Evidence

## Status

Open observation recorded from `hello-cgm` on 2026-07-28.

This is active evidence for UI presentation hardening. Investigation has now
confirmed one renderer-side allocation bottleneck, but it has not yet isolated
or closed every source of repeated presentation work.

## Observation

`examples/hello-cgm` presents a mostly static inspection screen containing:

- one panel;
- several short labels;
- five class-count bars;
- approximately 50 small source-order markers;
- no animation;
- no changing application state after startup.

The native UI is noticeably laggy despite the small and static workload.

That workload should be inexpensive. A corpus inspection screen of this size
must not require application-specific optimization to remain responsive.

## Current Example Behavior

The example currently performs presentation work every frame:

- reconstructs text specifications;
- runs bitmap text layout repeatedly;
- allocates command vectors for text;
- submits glyphs as individual draw commands;
- submits bars and element markers separately;
- rebuilds the logical presentation even though the decoded CGM inspection is
  immutable.

These are candidate contributors, not a completed diagnosis. Debug-build GPU
validation, platform event behavior, renderer submission overhead, and frame
pacing must also be measured before assigning ownership.

## Confirmed Finding: Per-Draw GPU Resource Allocation

The initial renderer path created two GPU uniform resources for every draw,
every frame:

- one instance uniform buffer and bind group;
- one camera uniform buffer and bind group.

`hello-cgm` emits many glyph draws and therefore multiplied this resource
creation cost despite using stable instances and one stable camera.

The renderer now:

- retains instance uniform bindings by draw queue slot;
- retains camera uniform bindings by camera handle;
- updates retained buffers only when their values change;
- reports per-frame submit calls, binding allocations, and uniform writes.

This keeps GPU cache lifetime renderer-owned and benefits all applications
without adding an application-local retained renderer.

`hello-cgm` now reports presentation command-construction CPU duration,
renderer-present-call CPU wall duration, and explicitly separated frame and
lifetime renderer counters. Frame counters cover draw calls, submit calls,
binding allocations, uniform writes, mesh uploads, and mesh replacements
between verified frame boundaries. Lifetime counters retain resource churn
since renderer creation. The expected steady state is zero frame binding
allocations, uniform writes, mesh uploads, and mesh replacements until layout,
instances, camera state, or mesh resources change.

The kernel diagnostics contract now supports bounded structured records and
sustained performance-budget monitoring. `hello-cgm` is the first capture
consumer: it observes the platform-reported frame interval, presentation
command-construction CPU duration, and renderer-present-call CPU wall duration,
emits a warning after three consecutive violations, suppresses repeated
warnings while degraded, and emits a recovery record when the metric returns
within budget. The renderer-call observation is not GPU execution or completion
time; it covers whatever CPU-visible work and provider pacing occurs before
`present()` returns. Clocks and renderer counters remain adapter-owned; the
kernel owns only diagnostic meaning and capture behavior.

The `wgpu` backend also separates CPU wall durations for surface acquisition,
renderer resource preparation, command encoding, the queue submission call,
and the surface presentation call. These phase observations localize
CPU-visible pressure without pretending that a queue submission call measures
when the GPU completed the submitted work. Providers that cannot observe a
phase report it as unavailable rather than zero.

The example also owns a monotonically increasing presentation revision. It
increments when the loaded inspection or window-dependent layout changes.
Repeated command construction at the same revision emits a structured
`hello-cgm.presentation` warning after three observations and emits recovery
when a new revision is built. This is direct evidence that unchanged semantic
presentation is being reconstructed at the application boundary; it is not
inferred from renderer submission volume.

## Architectural Pressure

The UI library should make the efficient path ordinary and the pathological
path visible.

Desired direction:

```text
semantic UI
    -> measured layout
    -> retained or invalidated presentation geometry
    -> batched renderer submissions
    -> frame diagnostics
```

Static UI should not require every application to independently invent:

- layout caching;
- glyph batching;
- surface batching;
- dirty-region or generation tracking;
- retained mesh ownership;
- draw-call and upload diagnostics.

The UI API should make unnecessary repeated layout, tessellation, upload, or
submission difficult to introduce accidentally.

## Investigation Checklist

- [ ] Capture release and debug frame timings separately.
- [ ] Record CPU time spent in semantic construction, text layout, geometry
      generation, renderer submission, and presentation.
- [x] Record per-frame draw calls, submit calls, allocations, mesh uploads, and
      mesh replacements.
- [x] Verify whether the app is continuously redrawing without meaningful
      invalidation.
- [ ] Compare one-command-per-glyph submission with batched text geometry.
- [ ] Compare one-command-per-marker submission with one batched marker mesh.
- [ ] Determine whether unchanged `UiTextSpec` and surface inputs can produce
      stable cache keys or generation IDs.
- [ ] Add a static-UI corpus benchmark with deterministic workload counts.
- [x] Add diagnostics that identify repeated layout or geometry work for
      unchanged semantic inputs.
- [x] Capture sustained frame and presentation budget violations through
      provider-neutral kernel diagnostics without assuming a global FPS target.

## Candidate Acceptance Criteria

Before this observation is closed:

- an unchanged UI scene performs no repeated text measurement or geometry
  generation unless explicitly requested;
- repeated frames do not upload or replace unchanged UI meshes;
- glyphs and repeated primitive markers are submitted in bounded batches rather
  than one call per item;
- frame statistics distinguish semantic rebuilds, layout passes, geometry
  rebuilds, uploads, submit calls, and draw calls;
- a static screen comparable to `hello-cgm` remains responsive in debug and
  release builds;
- examples receive the efficient behavior through shared UI facilities rather
  than local caching patches.

## Ownership Guardrails

- UI semantics own invalidation intent and stable presentation identity.
- UI layout owns reuse of unchanged measurement and placement results.
- Presentation geometry owns reusable, provider-neutral geometry output.
- The renderer owns GPU uploads, batching execution, and cache lifetime.
- Individual examples should describe semantic content, not implement a second
  retained UI framework.

## Related Evidence

- `examples/hello-cgm`
- `examples/lib-example/ui-tools`
- `docs/Plans/ui-box-vector-presentation.md`
- `docs/Plans/font-outline-vector-presentation.md`
- `docs/testing-strategy.md`
