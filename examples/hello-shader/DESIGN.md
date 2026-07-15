# Hello Shader - Example Design Document

| Field | Value |
| --- | --- |
| Purpose | Demonstrate Tokimu-owned shader text and custom WGSL pipeline selection |
| Primary Proof | Tokimu can treat shader source as owned asset data, switch between custom pipelines, and render through them |
| Secondary Proof | Shader text can shape appearance without turning the renderer into the owner of meaning |
| Non-Goals | A shader editor, shader graph UI, compute pipelines, or a full material authoring system |

## Purpose

`hello-shader` is Tokimu's custom shader example. It exists to validate that
shader source can be represented as Tokimu-owned data, compiled into a custom
pipeline, and used to shape appearance without pushing shader meaning into the
renderer as hidden architecture.

The example should feel like a tiny shader poster or preview board rather than a
shader authoring tool. It is intentionally small and procedural so the shader
boundary remains easy to inspect.

## What It Proves

- Tokimu can carry custom WGSL as owned shader text and asset identity
- Pipeline choice can be explicit, Tokimu-owned, and switchable at runtime
- Different shader sources can produce visibly different results over the same
  mesh primitives
- The renderer seam consumes shader data but does not own shader meaning
- A compact example can validate custom shader usage without a shader graph
  system

## Scene Shape

The first version should use a simple shader preview layout:

- two or more panels that use different custom WGSL sources
- a shared set of simple mesh primitives so the shader is the thing under test
- subtle animation in transforms so the preview feels alive
- keyboard controls that cycle between shader variants
- a restrained background so the shader output stays legible

## Shader Approach

Tokimu already exposes `Pipeline::custom_wgsl` and the same bind-group layout
used by the built-in 2D renderer. For this example, shader source is tracked as
Tokimu-owned asset identity in local `.wgsl` files, and a custom vertex shader
can pass local position through to the fragment stage while the fragment shader
shapes gradients, bands, or highlights from that data.

That keeps the implementation honest: the example is demonstrating shader text
ownership over the existing 2D renderer, not smuggling in a separate shader
graph or material editor runtime.

## Architecture Boundaries

This example must stay inside the engine boundaries defined in
[ADR-0001: Engine Boundaries](../../docs/ADR/ADR-0001-engine-boundaries.md):

- `tokimu-core` owns simulation truth
- rendering must not mutate simulation state
- the example may upload custom shader text and switch pipelines, but it does
  not let the renderer own shader semantics or shader asset identity
- if Tokimu later grows a richer shader authoring capability, it should still
  feed Tokimu-owned rendering concepts rather than leaking shader graph state
  into the app loop

## Visual Style

Prefer strong color separation, simple geometry, and obvious contrast between
shader variants. The point is to make shader choice and shader behavior obvious
rather than photorealistic.

## Architectural Assertions

This example demonstrates:

- Shader text remains inside Tokimu-owned data.
- Shader asset identity remains inside Tokimu-owned data.
- Rendering consumes compiled shader pipelines but does not own shader meaning.
- World state remains authoritative.
- Geometry is composed before renderer upload.
- Custom WGSL is sufficient to validate the runtime path.

## Next Steps After MVP

Once the basic shader preview works, useful follow-ons include:

1. a small palette of shader variants
2. time-based animation uniforms if Tokimu later needs them
3. a simple shader source loader from assets
4. preview controls for switching variants at runtime
5. comparison with a future shader-authoring capability if Tokimu earns one

## References

- [Tokimu Software Design Document](../../docs/Tokimu%20Software%20Design%20Document.md)
- [Tokimu Future Workspace Layout](../../docs/future-workspace-layout.md)
- [Tokimu Contribution Admission Guide](../../docs/contribution-admission-guide.md)
