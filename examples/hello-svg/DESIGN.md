# Hello SVG - Example Design Document

| Field | Value |
| --- | --- |
| Purpose | Demonstrate Tokimu-owned SVG-inspired scene composition through 2D mesh composition |
| Primary Proof | SVG-inspired path semantics can be represented through Tokimu-owned mesh composition |
| Secondary Proof | Rendering consumes path data but does not own the scene model |
| Non-Goals | SVG parsing completeness, DOM integration, text layout, or a full drawing editor |

## Purpose

`hello-svg` is Tokimu's SVG-inspired scene composition example. It exists to
validate that simple path-based illustrations can be represented as
Tokimu-owned geometry, animated over time, and uploaded repeatedly without
turning the renderer into a shape owner.

The example should feel like a tiny illustration, icon, or decorative vector
scene rather than a full authoring application. It is intentionally small and
procedural so the geometry remains easy to inspect.

## What It Proves

- Tokimu can represent SVG-inspired paths using reusable mesh primitives
- Path segments can be re-uploaded each frame without losing ownership boundaries
- Simple animation is enough to make the scene feel like a living illustration
- The renderer seam consumes path data but does not own the drawing model
- A compact example can validate SVG-style composition without a separate
  graphics stack

## Scene Shape

The first version should use a small illustration-style composition:

- a framed canvas or viewBox boundary
- a main path shape made from connected segments
- a few accent shapes so the scene feels like a small SVG illustration
- subtle motion so the path editing remains visible

## SVG Approach

Tokimu already exposes `Mesh::quad()`, `Mesh::triangle()`, `Mesh::diamond()`,
and `Instance2d` transforms. For this example, each SVG-inspired path segment
can be represented as a thin quad placed between two points, while fills can be
built from small reusable mesh primitives.

That keeps the implementation honest: the example is demonstrating
SVG-inspired composition over the existing 2D renderer, not smuggling in a
separate DOM or parser runtime.

## Architecture Boundaries

This example must stay inside the engine boundaries defined in
[ADR-0001: Engine Boundaries](../../docs/ADR/ADR-0001-engine-boundaries.md):

- `tokimu-core` owns simulation truth
- rendering must not mutate simulation state
- the example may rebuild paths every frame, but it does not own the render
  surface as world truth
- if a richer SVG backend ever appears, it should enter through Tokimu-owned
  semantics rather than leaking foreign DOM or vector objects into the app

## Visual Style

Prefer strong silhouette contrast, layered fills, and a composition that reads
as illustration or icon work rather than "random lines." The point is to make
path composition obvious.

## Architectural Assertions

This example demonstrates:

- Mesh ownership remains inside Tokimu.
- Rendering consumes path meshes but does not mutate them.
- World state remains authoritative.
- Geometry is composed before renderer upload.
- Procedural path geometry is sufficient to validate SVG-inspired composition.

## Next Steps After MVP

Once the basic SVG-style sketch works, useful follow-ons include:

1. multiple icon presets
2. fill versus stroke variants
3. simple pan and zoom controls
4. path editing controls for moving control points
5. comparison with a future dedicated SVG capability if Tokimu earns one

## References

- [Tokimu Software Design Document](../../docs/Tokimu%20Software%20Design%20Document.md)
- [Tokimu Future Workspace Layout](../../docs/future-workspace-layout.md)
- [Tokimu Contribution Admission Guide](../../docs/contribution-admission-guide.md)
