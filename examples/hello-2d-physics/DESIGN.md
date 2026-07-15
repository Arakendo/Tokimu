# Hello 2D Physics - Example Design Document

| Field | Value |
| --- | --- |
| Purpose | Demonstrate Tokimu-owned 2D physics semantics through a marble-drop / pachinko-style board |
| Primary Proof | Gravity, collision, and bin scoring can live in the application layer while rendering stays consumer-only |
| Secondary Proof | Procedural geometry is enough to validate the runtime path without importing a physics engine |
| Non-Goals | Rigid-body completeness, physics engine integration, continuous contact solving, or a commercial pachinko clone |

## Purpose

`hello-2d-physics` is Tokimu's marble-drop example. It exists to validate that
simple 2D physics-like behavior can be modeled as Tokimu-owned simulation data,
updated every frame, and rendered without handing truth to the renderer.

The example should feel like a pachinko machine or marble drop board: gravity,
bounce, pegs, bins, and score zones. It is intentionally small and procedural
so the physical behavior stays easy to inspect.

## What It Proves

- Tokimu can simulate gravity-driven 2D motion
- Collisions with pegs and walls can be resolved in the app loop
- Marbles can settle into bins and produce score
- Rendering consumes physics state but does not own the simulation model
- A compact example can validate 2D collision and binning without a dedicated
  physics engine

## Scene Shape

The first version should use a simple pachinko-style board:

- a top drop lane that releases marbles
- staggered pegs that redirect motion
- side walls and a floor that keep motion inside the board
- several score bins at the bottom
- visible marbles so the bouncing path reads clearly

## Physics Approach

Tokimu already exposes 2D transforms and a simple render path, so the example
should implement a lightweight simulation loop directly in the app:

- gravity accelerates each marble downward
- wall collisions clamp and reflect horizontal motion
- peg collisions push a marble out along the contact normal and reflect its
  velocity
- bottom contact settles a marble into a score bin

That keeps the implementation honest: the example is proving Tokimu-owned motion
and collision semantics, not importing a general physics solver.

## Architecture Boundaries

This example must stay inside the engine boundaries defined in
[ADR-0001: Engine Boundaries](../../docs/ADR/ADR-0001-engine-boundaries.md):

- `tokimu-core` owns simulation truth
- rendering must not mutate simulation state
- the example may resolve collisions and update score every frame, but it does
  not let the renderer own motion or contact state
- if Tokimu later earns a dedicated physics capability, this example should be
  able to migrate toward it without changing its architectural claim

## Input Mapping

- **Space**: drop a marble
- **ArrowDown**: reset the board
- **W/A/S/D**: reserved for future nudges or launch control

## Visual Style

Prefer clear pegs, obvious marble silhouettes, and visible score bins. The
point is to make motion and bounce legible rather than physically perfect.

## Architectural Assertions

This example demonstrates:

- Gravity and collision state remain inside Tokimu.
- Rendering consumes simulation state but does not mutate it.
- World state remains authoritative.
- Geometry is composed before renderer upload.
- Simple procedural collision is enough to validate the runtime path.

## Next Steps After MVP

Once the basic board works, useful follow-ons include:

1. multiple marble launch lanes
2. bumpers, gates, or score multipliers
3. nicer visual feedback for bin scoring
4. more explicit pause/reset controls
5. eventual comparison with a Tokimu-owned physics capability if one is earned

## References

- [Tokimu Software Design Document](../../docs/Tokimu%20Software%20Design%20Document.md)
- [Tokimu Future Workspace Layout](../../docs/future-workspace-layout.md)
- [Tokimu Contribution Admission Guide](../../docs/contribution-admission-guide.md)
