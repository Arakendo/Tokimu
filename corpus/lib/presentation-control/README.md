# Presentation Control

`presentation-control` incubates provider-neutral, transient presentation
semantics for corpus consumers.

It currently owns:

- stable typed target IDs for vector records, mesh primitives, model nodes, UI
  regions, text runs, and generic renderables;
- validated source color, opacity, and visibility;
- deterministic theme, application, selection, hover, warning, and hotspot
  override composition;
- tint, opacity, visibility, and emphasis intent;
- target enumeration, unknown-target diagnostics, and exact reset to source
  presentation;
- serializable semantic values suitable for a later bounded WASM API.

It explicitly does not own:

- imported asset truth;
- ECS identity or simulation state;
- shader source, WGSL, pipelines, render state, or GPU handles;
- renderer materials, bindings, caches, draw ordering, or transparency policy;
- TypeScript packages or browser state;
- model-format parsing or target discovery.

`hello-glb` and `hello-cgm` are the first independent consumers. Both use `E`
to cycle the same source, selected, hotspot, and transparent vocabulary. The
GLB corpus also demonstrates hidden and restored states.

These integrations currently lower resolved presentation into existing
renderer materials. They do not yet prove per-draw overrides over one shared
material, transparent 3D depth ordering, steady-state binding reuse, or a WASM
command boundary. Those remain explicit later slices in
[`typescript-shader-material-presentation-control.md`](../../../docs/Plans/typescript-shader-material-presentation-control.md).
