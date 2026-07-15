# Hello CAD - Example Design Document

## Purpose

`hello-cad` is Tokimu's mesh-editing example. It exists to demonstrate
Tokimu-owned mesh semantics through simple procedural editing. It starts from a
cube, mutates the geometry over time, and layers in a few small animated
companion objects so the scene feels like a lightweight modeling and inspection
loop rather than a pure rendering demo.

The example is intentionally not a full CAD kernel or parametric modeling
system. It should stay focused on Tokimu-owned mesh data, simple procedural
editing, and visual feedback that makes shape changes easy to read.

## What It Proves

- Tokimu can load and render a cube mesh
- The mesh can be mutated over time before upload
- Animated companion items can share the same scene
- The renderer seam can handle repeated mesh replacement without owning shape
  truth
- The example can stay small while still feeling like a modeling workflow

## Scene Shape

The first version should use a simple inspection-style layout:

- a main cube that deforms or "breathes" over time
- a floor or pedestal so the object has spatial context
- one or two companion items, such as a tool marker or reference block
- a camera angle that keeps the mutation legible

## Mesh Mutation Approach

Tokimu already exposes `Mesh` as a lightweight vertex-and-normal container. For
this example, the world loop should rebuild the visible mesh each frame from a
stable base cube, then apply a small deformation story before upload: chamfer
one edge, bulge one face, and then return to the original cube.

That gives the example a CAD-like feel without pretending to be a solid
modeling engine. The shape is still Tokimu-owned mesh data, not a backend CAD
kernel object.

## Architecture Boundaries

This example must stay inside the engine boundaries defined in
[ADR-0001: Engine Boundaries](../../docs/ADR/ADR-0001-engine-boundaries.md):

- `tokimu-core` owns simulation truth
- rendering must not mutate simulation state
- the example may animate and replace meshes, but it does not own a hidden
  scene graph outside the app loop
- if CAD-style semantics grow later, they should enter through Tokimu-owned
  capability or content layers, not by leaking backend objects into the app

## Visual Style

Prefer clean, readable geometry and a restrained palette. The point is to make
shape change obvious, not to build a busy editor UI.

## Next Steps After MVP

Once the basic cube mutation works, useful follow-ons include:

1. animated part insertion or removal
2. a second mesh family for panels, brackets, or tool heads
3. simple selection or inspection hints
4. import-from-bytes experiments for mesh data
5. eventual comparison with a more formal CAD capability if Tokimu earns one

## Architectural Assertions

This example demonstrates:

- Mesh ownership remains inside Tokimu.
- Rendering consumes mesh data but does not mutate it.
- World state remains authoritative.
- Geometry mutation occurs before renderer upload.

## References

- [Tokimu Software Design Document](../../docs/Tokimu%20Software%20Design%20Document.md)
- [Tokimu Capability Backends](../../docs/capability-backends.md)
- [Tokimu Future Workspace Layout](../../docs/future-workspace-layout.md)
