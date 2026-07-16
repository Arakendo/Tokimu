# Hello CAD - Example Design Document

## Purpose

`hello-cad` is Tokimu's mesh-editing example. It exists to demonstrate
Tokimu-owned mesh semantics through direct vertex editing. It starts from a
cube, keeps the scene simple, and leaves room for manual selection, movement,
and user-authored animation instead of baking motion into the mesh itself.

The example now also consumes the shared example-side `ui-tools` crate for its
toolbar layout, button hit testing, and cursor-to-world conversion so the UI
plumbing stays reusable across future demo projects.

The example is intentionally not a full CAD kernel or parametric modeling
system. It should stay focused on Tokimu-owned mesh data, simple procedural
editing, and visual feedback that makes shape changes easy to read.

## What It Proves

- Tokimu can load and render a cube mesh
- Vertices can be selected and moved as part of manual editing workflows
- User-authored animations can be layered on top of the scene later
- The renderer seam can handle repeated mesh replacement without owning shape
  truth
- The example can stay small while still feeling like a modeling workflow

## Scene Shape

The scene is now deliberately closer to Blender's default startup feel:

- one editable cube as the primary object
- a clean camera angle that reads like a basic modeling viewport
- minimal extra chrome so the object stays the focus

The current goal is to keep the scene simple enough that edits are obvious,
while still feeling like a modeling workspace instead of a toy render demo.

## Mesh Editing Approach

Tokimu already exposes `Mesh` as a lightweight vertex-and-normal container. For
this example, the world loop should rebuild the visible mesh from a stable base
cube and then apply direct edits from user interaction before upload.

That keeps the example focused on vertex-level manipulation without pretending
to be a full CAD kernel. The shape is still Tokimu-owned mesh data, not a
backend CAD object, and animation should come from explicit authored systems
instead of hidden per-frame deformation.

## Architecture Boundaries

This example must stay inside the engine boundaries defined in
[ADR-0001: Engine Boundaries](../../docs/ADR/ADR-0001-engine-boundaries.md):

- `tokimu-core` owns simulation truth
- rendering must not mutate simulation state
- the example may replace meshes, but it does not own a hidden scene graph
  outside the app loop
- if CAD-style semantics grow later, they should enter through Tokimu-owned
  capability or content layers, not by leaking backend objects into the app

## Visual Style

Prefer clean, readable geometry and a restrained palette. The point is to make
shape change obvious, not to build a busy editor UI.

## Next Steps After MVP

Once basic vertex selection and movement work, useful follow-ons include:

1. a transform gizmo or handle overlay for the cube
2. a property sidebar or inspection panel for the selected mesh
3. scene-tree or hierarchy navigation if more objects return later
4. import-from-bytes experiments for mesh data
5. authored animation tracks layered over edited geometry

## Architectural Assertions

This example demonstrates:

- Mesh ownership remains inside Tokimu.
- Rendering consumes mesh data but does not mutate it.
- World state remains authoritative.
- Geometry edits occur before renderer upload.

## References

- [Tokimu Software Design Document](../../docs/Tokimu%20Software%20Design%20Document.md)
- [Tokimu Capability Backends](../../docs/capability-backends.md)
- [Tokimu Future Workspace Layout](../../docs/future-workspace-layout.md)
