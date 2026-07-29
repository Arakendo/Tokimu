# Hello Vector Draw - Example Design Document

| Field | Value |
| --- | --- |
| Purpose | Demonstrate Tokimu-owned vector-style drawing using 2D mesh composition |
| Primary Proof | Reusable strokes can be built from simple mesh primitives |
| Secondary Proof | Rendering consumes vector-like shape data without owning it |
| Non-Goals | Text layout, SVG import, font rendering, or a full drawing editor |

## Purpose

`hello-vector-draw` is Tokimu's vector-style drawing example. It exists to
validate that simple 2D linework can be expressed as Tokimu-owned mesh data,
animated over time, and uploaded repeatedly without turning the renderer into a
shape owner.

The example should feel like a drafting sketch or wireframe plot rather than a
bitmap illustration. It is intentionally small and procedural so the geometry
remains easy to inspect.

## What It Proves

- Tokimu can compose vector-like shapes from reusable mesh primitives
- Strokes can be re-uploaded each frame without losing ownership boundaries
- Simple animation is enough to make the drawing feel alive
- The renderer seam consumes shape data but does not own the drawing model
- A compact example can validate 2D composition without a separate drawing stack

## Scene Shape

The first version should use a simple wireframe composition:

- a cube-like outline built from thin stroke segments
- vertex markers so the shape reads like editable vector work
- a moving pen tip or highlight to show the current draw step
- a restrained dark background so the lines remain legible

## Vector Drawing Approach

Tokimu already exposes `Mesh::quad()` and `Instance2d` transforms. For this
example, each stroke can be represented as a thin quad placed between two
points, with width, rotation, and midpoint computed in the app loop.

That keeps the implementation honest: the example is demonstrating vector-style
composition over the existing 2D renderer, not smuggling in a separate text or
SVG engine.

## Architecture Boundaries

This example must stay inside the engine boundaries defined in
[ADR-0001: Engine Boundaries](../../docs/ADR/ADR-0001-engine-boundaries.md):

- `tokimu-core` owns simulation truth
- rendering must not mutate simulation state
- the example may rebuild strokes every frame, but it does not own the render
  surface as world truth
- if a richer vector backend ever appears, it should enter through Tokimu-owned
  semantics rather than leaking foreign drawing objects into the app

## Visual Style

Prefer clear stroke contrast, visible vertices, and a composition that reads as
"editing" rather than "wobbling." The point is to make shape construction
obvious.

## Architectural Assertions

This example demonstrates:

- Mesh ownership remains inside Tokimu.
- Rendering consumes stroke meshes but does not mutate them.
- World state remains authoritative.
- Geometry is composed before renderer upload.
- Procedural linework is sufficient to validate the runtime path.

## Next Steps After MVP

Once the basic vector sketch works, useful follow-ons include:

1. multiple shape presets
2. dashed or grouped stroke styles
3. simple selection or hover feedback
4. path editing controls for moving vertices
5. comparison with a future dedicated vector capability if Tokimu earns one

## References

- [Tokimu Software Design Document](../../docs/Tokimu%20Software%20Design%20Document.md)
- [Tokimu Future Workspace Layout](../../docs/future-workspace-layout.md)
- [Tokimu Contribution Admission Guide](../../docs/contribution-admission-guide.md)
