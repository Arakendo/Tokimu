# ADR-0018: Atomic Staged Render Resource-Set Replacement

## Status

Accepted — 2026-08-19

## Context

Tokimu applications can replace a complete presentation resource set while
retaining the renderer provider session, device, and surface. Rebuilding the
whole backend for an ordinary map or composition change conflates provider
lifetime with application-owned presentation membership. Mutating the current
set in place is also insufficient: a late replacement failure can damage the
last-known-good presentation, and reused local resource keys can make retained
references silently resolve as unrelated successor resources.

AR-0032 compared those alternatives against three evidence layers:

- a provider-neutral semantic model proved isolated staging, failure
  containment, atomic commit, and stale-generation rejection;
- a corpus-private WGPU prototype staged a complete candidate beside a live
  set, preserved the live set through a late failure, and committed a successor
  without recreating the backend, device, or surface; and
- repeated browser pressure completed 27 replacements, including five
  contained late failures, with bounded logical overlap and no delivered
  provider diagnostics.

The evidence supports a narrow replacement transaction. It does not establish
a final handle representation, physical GPU reclamation timing, or a general
resource allocator.

## Decision

Tokimu admits **atomic staged replacement semantics for a bounded render
resource set** in `tokimu-render`.

The contract is:

```text
current resource set remains authoritative
    -> construct an isolated candidate set
    -> validate the candidate and its dependency closure
    -> failure: discard candidate; current remains authoritative and usable
    -> success: perform one observable commit
    -> successor becomes current; prior set retires
    -> identities retained from the retired set cannot alias successor
       resources merely because local keys were reused
```

The current set must remain resolvable and presentable until a candidate has
been completely staged and validated. Candidate resources must not become
visible through current-set resolution before commit. Commit changes set
authority as one observable transition; it is not a sequence of individually
visible resource replacements.

After commit, commands and resource identities retained from the retired set
must reject deterministically when resolved against the successor set. This
requirement applies even when the successor reuses every local mesh, texture,
material, pipeline, camera, or command key from the retired set.

### Ownership

- Applications and compositions own resource-set membership, replacement
  timing, and continuation policy after staging failure.
- `tokimu-render` owns the provider-neutral staging, validation, commit,
  retirement, and stale-identity invariants.
- Renderer backends own concrete allocation, upload, synchronization, drop,
  and reclamation mechanisms while preserving the observable Tokimu contract.
- Simulation state remains application/world truth. Resource-set replacement
  does not make the renderer a scene or simulation owner.
- `tokimu-assets` asset identity and generation remain distinct from renderer
  set identity. Neither silently substitutes for the other.

### Conformance gate

A stable implementation is not conformant until an integrated provider-backed
test proves this exact case:

```text
commit set A
    -> retain a real command from A
    -> stage and commit set B using A's local resource keys
    -> resolve/submit the retained A command
    -> deterministic stale-set rejection before it can resolve B resources
```

The same implementation must prove that a late candidate failure preserves all
current resource families and leaves current commands presentable. Native and
WASM implementations must share the observable contract even where backend
storage and synchronization differ.

The corpus-private Alternative C semantic model and feature-gated WGPU staging
prototype are evidence for this decision. They are not promoted unchanged by
this ADR and do not satisfy the integrated stale-command gate merely because
they proved its semantic and provider halves separately.

## Consequences

- Ordinary composition replacement can retain a provider session without
  exposing a partially constructed successor.
- Failed staging has an explicit last-known-good containment boundary.
- Cross-set identity must carry or encounter sufficient set scope to reject a
  retired reference, but this ADR does not prescribe how that scope is encoded.
- A backend may temporarily hold current and candidate allocations at once.
  Bounded logical overlap is part of the transaction; bounded physical VRAM
  overlap is not established by the current evidence.
- Backends remain free to use maps, arenas, generations, indirection, scoped
  resolvers, or another internal mechanism that satisfies the contract.
- In-place reset without isolated staging is not a substitute when atomic
  last-known-good replacement and stale-key rejection are required.

## Non-Decisions

This ADR does not:

- choose a public handle bit layout, `{set, local_handle}` representation,
  generation token, arena API, or scoped resolver API;
- admit a general allocator, scene graph, asset manager, or renderer-owned
  composition model;
- promise immediate or bounded physical GPU-memory reclamation;
- prescribe provider fences, queue-idle behavior, deferred drops, or other
  synchronization policy;
- admit incremental per-resource release as a stable contract;
- combine ordinary resource-set replacement with device-loss recovery, backend
  recreation, surface lifecycle, or process/page restart;
- change existing same-handle replacement semantics within one authoritative
  set; or
- promote the current corpus-private WGPU prototype to stable API.

## Verification

- Provider-neutral contract tests must cover complete staging, injected late
  failure, unchanged current-set resolution after failure, atomic commit, and
  deterministic stale identity after local-key reuse.
- At least one provider-backed test must retain and exercise an actual command
  from the retired set after successor commit; testing resource handles and
  provider staging separately is insufficient.
- Failure tests must cover meshes, textures, materials, pipelines, cameras,
  and commands participating in the staged dependency closure.
- Native and browser/WASM evidence must report the same logical lifecycle
  outcomes and must preserve structured failure behavior under ADR-0009 and
  terminal outcome closure under ADR-0017.
- Provider-specific measurements must distinguish logical retirement from
  observed physical reclamation and must not infer the latter from Rust drops.
- An independent resource-rich caller must exercise the accepted semantics
  before the implementation is treated as broadly conformant rather than a
  Doom-specific realization.

## References

- `docs/Tokimu Software Design Document.md`
- `docs/ADR/ADR-0001-engine-boundaries.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0009-ring-based-verification-failure-containment-and-recovery.md`
- `docs/ADR/ADR-0017-observable-terminal-failure-and-host-crash-conformance.md`
- `docs/Architectural Reviews/AR-0032-atomic-staged-render-resource-set-replacement.md`
- `docs/Plans/Renderer-Reliability/renderer-scene-resource-lifetime-and-replacement.md`
- `docs/Plans/Renderer-Reliability/Evidence/renderer-scene-resource-alternative-c-semantic-generation-evidence.md`
- `docs/Plans/Renderer-Reliability/Evidence/renderer-scene-resource-alternative-c-real-provider-staging-evidence.md`
- `docs/Plans/Renderer-Reliability/Evidence/renderer-scene-resource-alternative-c-repeated-provider-pressure-evidence.md`
