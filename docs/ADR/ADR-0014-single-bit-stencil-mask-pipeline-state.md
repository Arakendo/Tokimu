# ADR-0014: Single-Bit Stencil Mask Pipeline State

## Status

Accepted

## Context

The E1M1 sky-crossing study produced a concrete presentation requirement that
ordinary depth cannot express. A nearest-depth write can close presentation
behind the first surface, but it cannot reopen presentation after a second
surface on the same pixel. Exact crossing parity requires a bounded per-pixel
bit that can be inverted by passing fragments and tested by later draws.

Implementing the effect through Doom-specific renderer vocabulary, triangle
centroid filtering, or backend-native handles would violate Tokimu's renderer
boundary or provide misleading non-pixel behavior.

## Decision

Tokimu admits a narrow provider-neutral `StencilMode` pipeline declaration:

- `Disabled` preserves ordinary rendering;
- `InvertOnDepthPass` inverts the low stencil bit when a fragment passes its
  declared depth test; and
- `RequireZero` retains fragments only where that low bit is zero.

The renderer clears the stencil bit to zero with each render target. The WGPU
backend realizes the combined depth/stencil attachment with
`Depth24PlusStencil8`. Stencil reference remains fixed at zero and only the low
bit participates.

Pipeline choice and draw ordering remain caller-owned. The renderer does not
infer volumes, portals, crossings, sky, BSP, or source semantics. Applications
must explicitly submit any depth prepass, mask geometry, and masked color pass.

Categorical cutout additionally admits a color-suppressed depth-prepass state.
Its shader still applies the declared alpha discard before retained fragments
write depth.

## Consequences

- Native and WebGPU backends use one combined depth/stencil target.
- Callers can express bounded parity/mask workflows without backend objects.
- Ordinary pipelines default to `Disabled` and retain their prior behavior.
- A parity workflow may require duplicate geometry submission for depth and
  color passes; performance policy remains caller-owned and observable.

## Non-Decisions

This ADR does not admit arbitrary stencil reference values, masks, comparison
functions, increment/decrement operations, stencil readback, render graphs,
portals, Doom sky semantics, or renderer-owned pass scheduling. Additional
operations require new corpus evidence and review.

## Verification

- Pipeline tests retain default-disabled behavior and both admitted modes.
- Cutout tests prove its depth-only prepass remains categorically validated.
- WGPU native execution must create the combined attachment and complete a
  two-frame parity workload.
- WASM compilation must retain the same provider-neutral declarations.

## References

- `docs/ADR/ADR-0001-engine-boundaries.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0013-caller-declared-categorical-cutout-surfaces.md`
- `docs/Architectural Reviews/AR-0030-source-owned-presentation-preparation-boundary.md`
- `docs/Plans/DOOM/Studies/Doom grouped sky-crossing parity shadow.md`
