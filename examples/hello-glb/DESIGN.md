# Hello GLB - Example Design Document

| Field | Value |
| --- | --- |
| Purpose | Demonstrate Tokimu-owned GLB asset identity and render it through engine-owned mesh data |
| Primary Proof | The example can carry a `.glb` source through the asset system without the renderer owning file-format meaning |
| Secondary Proof | Tokimu can load a model-shaped scene and keep the render seam format-agnostic |
| Non-Goals | Full glTF coverage, animation import, material authoring, or a general asset pipeline |

## Purpose

`hello-glb` is Tokimu's GLB proof example. It exists to validate that a GLB
source can be represented as Tokimu-owned asset identity, mapped into engine-
owned geometry, and shown in a simple scene without giving the renderer any
file-format responsibility.

The example should stay small and explicit. It is not a full importer, not a
viewer with a file browser, and not a glTF authoring tool. It is a boundary proof
for the asset seam.

## What It Proves

- Tokimu can track a GLB source through the asset system
- Asset identity can stay separate from renderer-owned mesh truth
- A model-shaped scene can be shown without exposing file-format concerns to the renderer
- Repeated uploads still preserve Tokimu ownership boundaries
- A compact example can validate the first GLB-proof slice without pretending to be a complete importer

## Scene Shape

The first version should use a simple model presentation:

- one main model mesh or model proxy loaded from the Tokimu asset path
- a floor or backdrop so the model has context
- a slow orbit camera so the object can be inspected
- a small title or status readout that names the GLB source being proved

## GLB Approach

Tokimu's current asset layer can already track asset identity and source labels,
so this example should treat `cube.glb` or a similar sample model as an asset
entry first and a renderable mesh second.

That keeps the implementation honest: the GLB file remains a source of asset
meaning, while Tokimu-owned mesh data becomes the actual render input. The
renderer stays format-agnostic.

## Architecture Boundaries

This example must stay inside the engine boundaries defined in
[ADR-0001: Engine Boundaries](../../docs/ADR/ADR-0001-engine-boundaries.md):

- `tokimu-core` owns simulation truth
- rendering must not mutate simulation state
- the example may rebuild model meshes from asset input, but it does not let the
  renderer own GLB or glTF semantics
- if a real GLB importer grows later, it should feed Tokimu-owned geometry and
  metadata rather than leaking foreign importer state into the app loop

## Visual Style

Prefer simple model lighting, clear silhouettes, and a restrained palette. The
point is to make the format boundary visible, not to build a polished asset
browser.

## Architectural Assertions

This example demonstrates:

- GLB source identity remains separate from renderable mesh data.
- Rendering consumes geometry but does not own file-format meaning.
- World state remains authoritative.
- Asset meaning is resolved before renderer upload.
- A GLB proof can stay small while still validating Tokimu's asset boundary.

## Next Steps After MVP

Once the basic GLB proof works, useful follow-ons include:

1. loading a real cube or character asset from bytes
2. comparing source identity versus imported mesh identity
3. more explicit material or node metadata if Tokimu earns it
4. animation playback once the asset seam has been proven
5. eventually moving from a staged proof to a true importer slice

## References

- [Tokimu Software Design Document](../../docs/Tokimu%20Software%20Design%20Document.md)
- [Tokimu Capability Backends](../../docs/capability-backends.md)
- [Tokimu Future Workspace Layout](../../docs/future-workspace-layout.md)
