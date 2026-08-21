# ADR-0019: Fixed-Descriptor Set-Scoped Texture-Content Replacement

## Status

Accepted — 2026-08-20

## Context

ADR-0018 admits atomic replacement of one complete render resource set. It
does not answer repeated small presentation changes inside the current set.
The persistent Doom browser console changes one RGBA8 raster while its texture
identity, descriptor, material dependency, mesh, pipeline, camera, command
topology, composition, and resource set remain unchanged.

AR-0033 compared complete set replacement, replacement behind an existing
identity, a declared dynamic-resource class, transient submission data, and a
DOM overlay. Whole-set replacement was the correctness control but staged
1,241 persistent resources and regenerated 2,069 commands for one E1M1 console
texture. A dynamic class added vocabulary without removing the transaction
requirements. No transient texture path existed, and DOM presentation did not
exercise Tokimu's renderer boundary.

The existing-identity candidate then passed:

- provider-neutral failure, scope, and ordering shadows;
- native WGPU and browser WebGPU execution;
- abandoned-candidate preservation and atomic commit;
- continued presentation through the same scoped commands;
- stale-candidate rejection after whole-set replacement;
- 27 console-sized updates with five prepared candidate drops;
- external browser terminal closure with zero provider diagnostics; and
- an independent procedural-texture caller.

The evidence supports a texture-content transaction. It does not support
general resource mutation.

## Decision

Tokimu admits **atomic fixed-descriptor texture-content replacement inside the
current authoritative render resource set** in `tokimu-render`.

The contract is:

```text
current texture realization A remains authoritative
    -> prepare isolated candidate content realization B
    -> validate provider-session and current-set scope
    -> validate existing source-texture identity and unchanged descriptor
    -> prepare dependent provider bindings with B
    -> failure or abandonment: A remains authoritative and presentable
    -> commit: B and its dependent provider bindings become authoritative
       at one observable boundary
```

The stable provider-neutral surface is
`RenderTextureContentUpdateLifecycle`. A provider owns its associated opaque
candidate. The preparation operation accepts an existing texture handle and
replacement RGBA8 bytes; it does not accept a width, height, format,
color-space, sampler, role, or replacement descriptor. Consequently, the
candidate can replace content only within the descriptor already owned by the
current texture identity.

Successful commit does not change resource-set identity. Commands already
scoped to that current set remain valid and observe the committed realization.
If a whole-set commit occurs after preparation but before texture commit, the
older texture candidate must reject as stale before its local texture handle
is resolved. A candidate from another provider session must likewise reject
before lookup.

### Ownership

- Applications and compositions own source pixels, update timing, and policy
  after preparation or commit failure.
- `tokimu-render` owns the provider-neutral prepare/commit, fixed-descriptor,
  set-scope, failure-preservation, visibility, and ordering invariants.
- Renderer providers own concrete texture allocation, upload, binding
  construction, synchronization, swap, drop, and reclamation mechanics.
- Materials retain their semantic texture dependency; commit may replace the
  provider bindings required to preserve that dependency, but it does not
  mutate material meaning.
- Simulation, UI, console, asset, and composition truth remain outside the
  renderer.

### Ordering With ADR-0018

ADR-0018 whole-set replacement and ADR-0019 texture-content replacement are
orthogonal transactions sharing one authority boundary:

- texture commit inside the current set preserves that set identity;
- whole-set commit retires the complete preceding set;
- any texture candidate prepared against that retired set becomes stale;
- no texture update may land in a whole-set candidate or successor by local-key
  coincidence; and
- neither transaction exposes raw backend submission through the resource-set
  session.

## Consequences

- Repeated console, procedural-texture, and similar fixed-shape pixel changes
  need not reconstruct an otherwise unchanged resource set.
- Failure containment remains candidate-based rather than a best-effort
  in-place byte upload.
- Existing scoped command topology remains reusable across successful content
  commits.
- Providers may temporarily hold the old and candidate realizations and their
  bindings simultaneously.
- A provider-neutral commit observation may report set, texture, unchanged
  descriptor, source-byte count, and dependent-material count without exposing
  provider objects.

## Non-Decisions

This ADR does not:

- admit a general `update_resource` operation or `DynamicResource` class;
- admit texture resize, format, color-space, sampler, role, or descriptor
  replacement;
- admit mesh, material-semantic, pipeline, camera, render-target, or command
  mutation;
- define partial-region, mip-level, compressed-texture, streaming, or
  asynchronous update semantics;
- promise immediate or bounded physical GPU-memory reclamation;
- define individual handle encoding or make bare handles portable across
  sessions or resource sets;
- permit raw backend access or unscoped submission from a resource-set session;
  or
- make the renderer owner of source pixels or update policy.

## Verification

- Dropping a fully prepared candidate must leave the prior realization
  presentable through the same scoped commands.
- Commit must retain resource-set identity and make the complete new
  realization visible without partial pixels or mixed dependent bindings.
- Wrong-session and stale-set candidates must reject before local texture
  lookup.
- A whole-set commit between texture preparation and commit must deterministically
  stale the older texture candidate, including when local texture keys are
  reused.
- Payload size must match the current descriptor exactly; callers cannot supply
  a new descriptor through this operation.
- Native and browser providers must retain structured diagnostics and ADR-0017
  terminal-outcome closure.
- Repeated-pressure evidence must distinguish logical completion from physical
  reclamation.

## References

- `docs/ADR/ADR-0017-observable-terminal-failure-and-host-crash-conformance.md`
- `docs/ADR/ADR-0018-atomic-staged-render-resource-set-replacement.md`
- `docs/Architectural Reviews/AR-0033-scoped-in-set-presentation-resource-updates.md`
- `docs/Plans/Renderer-Reliability/Evidence/AR-0033 Slice 2 provider and pressure.md`
- `docs/Plans/DOOM/Evidence/D1 debug console evidence.md`
