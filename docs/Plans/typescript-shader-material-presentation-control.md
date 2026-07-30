# TypeScript Shader, Material, And Presentation Control

## Status

In progress. Slice 0 boundary vocabulary, Slice 1, the explicit render-state
portion of Slice 2, the contract and native-cache portions of Slice 3, and the
first native mesh corpus proof in Slice 5 began on 2026-07-29 in
`corpus/lib/presentation-control`, `tokimu-render`, and `corpus/hello-glb`.

This plan does not admit a new crate or settle an unresolved architectural
boundary by itself. If implementation evidence changes an accepted SDD, TTSDD,
ADR, or capability boundary, open or update the corresponding Architectural
Review before treating that change as established architecture.

## Purpose

Tokimu applications need an author-facing way to control how addressable
vectors, meshes, and model parts are presented without moving rendering
semantics into TypeScript or mutating imported source assets.

The first concrete consumer is an asset inspection workflow where a user can:

- select a mesh, node, vector record, or hotspot;
- tint the selected object;
- change its opacity to inspect geometry beneath it;
- emphasize it through an outline or other bounded presentation treatment;
- restore the source presentation deterministically;
- receive explicit diagnostics when a requested effect cannot be represented by
  the active pipeline or backend.

The longer-term authoring workflow also needs a restricted TypeScript shader and
material API that lowers ahead of time into Tokimu-owned semantics and WGSL.

These are related but distinct paths:

```text
Interactive presentation control
    TypeScript intent
        ↓
    provider-neutral presentation override
        ↓
    existing material/pipeline execution

Shader authoring
    restricted TypeScript shader definition
        ↓
    Tokimu shader semantic model
        ↓
    validated WGSL + pipeline description
        ↓
    renderer execution
```

A hotspot color change must not require shader compilation. Authoring a new
lighting function must not become an unrestricted runtime JavaScript facility.

## Architectural Thesis

> Applications communicate presentation intent. Tokimu owns material and shader
> semantics. TypeScript lowers into those semantics. Renderers execute them.

This follows the SDD and TTSDD dependency direction:

```text
authoring TypeScript
        ↓
domain-specific frontend validation and lowering
        ↓
Tokimu-owned shader, material, and presentation models
        ↓
explicit mesh + material + pipeline draw selection
        ↓
renderer backend
```

The engine must remain usable from Rust without TypeScript. The TypeScript
packages describe authoring APIs and types; they do not become a second renderer,
material system, or owner of scene state.

## Governing Documents

- [`Tokimu Software Design Document.md`](../Tokimu%20Software%20Design%20Document.md)
  keeps pipeline choice explicit at draw submission and materials responsible
  for bound data.
- [`Tokimu TypeScript Design Document.md`](../Tokimu%20TypeScript%20Design%20Document.md)
  requires domain-specific TypeScript APIs to lower one-way into engine-owned
  semantic models.
- [`ADR-0001-engine-boundaries.md`](../ADR/ADR-0001-engine-boundaries.md)
  keeps rendering outside simulation truth.
- [`ADR-0003-capability-ownership-boundary.md`](../ADR/ADR-0003-capability-ownership-boundary.md)
  separates Tokimu-owned meaning from specialized execution.
- [`ADR-0004-foundational-presentation-text-and-icons.md`](../ADR/ADR-0004-foundational-presentation-text-and-icons.md)
  provides the precedent that presentation semantics remain provider-neutral.

## Current Evidence

The existing renderer already proves a narrow substrate:

- `Material` carries a label, RGBA base color, and optional texture handle;
- draw commands select mesh, material, and pipeline handles explicitly;
- native pipelines pass material alpha through their fragment output;
- the wgpu backend enables alpha blending;
- renderables can reuse one mesh with different material handles;
- custom WGSL 2D pipelines can be registered with explicit entry points;
- SVG lowering preserves fill color, stroke color, and opacity intent;
- the ASP.NET/WASM asset workbench already exposes addressable imported
  observations to browser TypeScript.

The current substrate does not yet provide:

- typed, named material parameters;
- per-draw or per-instance material overrides;
- stable author-facing names for render resources and imported model parts;
- an explicit transparent 3D render-state policy;
- provider-neutral hotspot or selection presentation semantics;
- a shader module and binding schema describable as data;
- TypeScript material or shader authoring packages;
- parity validation between TypeScript-authored and hand-written WGSL;
- deterministic diagnostics for unsupported parameters or render states.

## Vocabulary And Ownership

### Source material

The material information decoded from GLB, FBX, SVG, CGM, or another provider.
Importers own decoding it. Applications may inspect it, but interactive
highlighting must not destructively rewrite it.

### Material definition

A provider-neutral Tokimu value describing named parameters and their defaults.
It describes bound data and does not silently select a pipeline.

### Material instance

A concrete set of parameter values associated with a material definition.
Instances may be immutable and shared.

### Presentation override

A transient, composable application request applied to one addressable
presentation target. The initial bounded vocabulary is:

- tint or replacement color;
- opacity multiplier;
- visibility;
- selected, hovered, warning, or hotspot emphasis role.

An override is not imported asset truth, simulation truth, or shader source.

### Shader module definition

A Tokimu-owned description of shader stages, entry points, parameter bindings,
vertex inputs, and generated or supplied WGSL. It contains no TypeScript AST,
browser object, or backend-native shader handle.

### Pipeline definition

The explicit presentation state that combines compatible shader modules, vertex
layout, blending, depth, culling, and other bounded execution policy. Materials
do not own pipeline selection.

### Presentation target

A stable identity that can refer to an imported node, mesh primitive, vector
record, UI region, or other renderable presentation unit. A target does not need
to be an ECS entity and must not create simulation ownership accidentally.

## Dependency Direction

```text
@tokimu/materials ─┐
@tokimu/shader    ─┼─▶ tokimu-ts-frontend lowering host
@tokimu/ui        ─┘              │
                                  ▼
                     Tokimu-owned semantic models
                                  │
                                  ▼
                         tokimu-render contracts
                                  │
                                  ▼
                          renderer backend
```

Rules:

- `tokimu-core` and `tokimu-runtime` do not depend on TypeScript, WGSL parsers,
  wgpu objects, or browser APIs.
- TypeScript packages do not import Rust engine implementation code.
- `tokimu-ts-frontend` validates and lowers recognized authoring APIs; it does
  not render.
- source-format providers do not expose native material objects through the
  authoring API.
- renderer backends may cache GPU resources but do not own application
  selection, hotspot, or source-material state.
- browser consumers may invoke bounded WASM presentation commands but may not
  reimplement importer or shader semantics in TypeScript.

## Two TypeScript Control Tiers

### Tier 1: runtime presentation commands

This tier supports interaction such as selecting a model part and changing its
presentation. It crosses a narrow WASM or runtime-host API using data:

```ts
presentation.setOverride("mesh:Housing", {
  role: "hotspot",
  tint: [1.0, 0.35, 0.1, 1.0],
  opacity: 0.45,
});

presentation.clearOverride("mesh:Housing");
```

The TypeScript API communicates intent. Rust validates target identity,
parameter ranges, capability support, and override composition.

### Tier 2: ahead-of-time shader and material authoring

This tier defines reusable material schemas and restricted shader behavior:

```ts
const inspectionMaterial = material("inspection-surface", {
  parameters: {
    baseColor: color([0.75, 0.8, 0.9, 1.0]),
    opacity: float(1.0, { min: 0.0, max: 1.0 }),
    hotspotMix: float(0.0, { min: 0.0, max: 1.0 }),
  },
});
```

A later restricted shader form may use typed stage inputs, uniforms, samplers,
vector and matrix operations, and deterministic control flow. It lowers ahead
of time to the Tokimu shader model and WGSL. DOM access, ambient I/O, `fetch`,
`eval`, `async`, and arbitrary JavaScript execution remain invalid in lowered
shader definitions.

The provisional package name is `@tokimu/shader`, matching the TTSDD and
roadmap. The name records a future authoring boundary only; no package is
published until the Rust shader semantic model has independent callers.

## Transparency Requirements

RGBA output and alpha blending alone are not sufficient for correct transparent
3D presentation.

The bounded transparency proof must define:

- opaque versus blended pipeline state;
- depth-test and depth-write behavior;
- back-face culling policy;
- deterministic transparent draw ordering;
- behavior for intersecting transparent geometry;
- whether opacity applies to fill, stroke, texture, and lighting output;
- diagnostics when a backend cannot honor the requested behavior.

The initial proof may sort transparent instances by a documented camera-space
key and reject unsupported intersecting-order guarantees. It must not claim
order-independent transparency.

## Implementation Slices

### Slice 0: Boundary Review And Corpus Definition

Deliverables:

- [x] Reconcile the provisional package name as `@tokimu/shader` across the
      TTSDD and roadmap. This does not create or publish the package.
- [x] Record the initial presentation-override vocabulary and target identity
      rules.
- [x] Define a small Rust-first corpus scene with opaque, tinted, selected, and
      transparent mesh/vector cases.
- [x] Define one asset-workbench hotspot scenario over a known GLB mesh. The
      `Box.glb` mesh-primitive target can be focused through the higher-priority
      hotspot layer and cleared independently of application-layer controls.
- [x] Record unsupported behavior explicitly, including transparent
      intersections and arbitrary shader code.

Acceptance criteria:

- [x] Every implemented public type has a named owner.
- [x] Runtime overrides and AOT shader authoring are documented as separate
      paths.
- [x] The corpus can fail independently for target resolution, material
      lowering, pipeline declaration, and resolved-output artifact generation.
      GPU framebuffer validation remains a separate, explicitly unproven
      backend concern.

### Slice 1: Provider-Neutral Rust Material Model

Deliverables:

- [x] Extend the Rust material model with typed, named parameter declarations.
- [x] Support bounded parameter kinds: float, vector, color, texture, and
      boolean or enum policy where justified.
- [x] Separate material definition identity from material instance values.
- [x] Validate missing, unknown, duplicate, non-finite, and out-of-range
      parameters.
- [x] Preserve the existing base-color material path through a compatibility
      constructor or lowering adapter.

Acceptance criteria:

- [x] A Rust caller can define and instantiate a material without WGSL or wgpu
      types.
- [x] Invalid parameter data emits deterministic diagnostics.
- [ ] Capture native visual evidence that existing solid-color corpus examples
      remain behaviorally equivalent after lowering through the material model.
- [x] A material still cannot choose its pipeline implicitly.

### Slice 2: Explicit Pipeline And Render-State Description

Deliverables:

- [x] Describe blend mode, depth test/write, culling, and color-write policy as
      bounded Tokimu data.
- [x] Keep shader module identity and pipeline identity separate.
- [x] Validate the current shared material-binding contract before submission:
      every built-in pipeline receives color, texture, and sampler bindings,
      with a deterministic white fallback texture when source material has none.
      Custom WGSL declarations must supply source and non-empty entry points.
- [x] Preserve current built-in pipelines through explicit descriptors.
- [ ] Surface backend WGSL compilation and pipeline validation failures through
      Tokimu diagnostics. Provider-neutral declaration failures are now
      returned before backend submission.

Acceptance criteria:

- [x] Opaque and transparent policies are distinguishable without inspecting
      backend code.
- [x] The native renderer can recreate current built-in pipelines from the
      admitted description.
- [ ] Unsupported backend state produces an explicit diagnostic rather than a
      silent fallback.

### Slice 3: Presentation Override Semantics

Deliverables:

- [x] Add a provider-neutral `PresentationOverride` value.
- [x] Define deterministic override composition for source material, theme,
      selection, hover, warning, and hotspot intent.
- [x] Add per-draw override submission without mutating shared material
      instances.
- [x] Cache compatible derived bindings below the semantic boundary, keyed by
      source material plus the bounded override value.
- [x] Add clear/reset behavior that restores source presentation exactly.

Acceptance criteria:

- [x] Two draws sharing one source material can carry different tint and opacity
      overrides without changing that source material.
- [x] Clearing one target's override does not alter another target.
- [x] Override application emits an explicit corpus warning if unchanged
      presentation state allocates a derived material binding after its first
      frame. The `hello-glb` telemetry permits the initial cache fill and
      makes repeated churn visible for review.
- [x] Imported source material data remains unchanged.

### Slice 4: Stable Presentation Target Identity

Deliverables:

- [x] Define stable provider-neutral target IDs for model nodes, mesh
      primitives, vector records, and renderables.
- [x] Preserve optional source names for inspection while treating stable target
      IDs, rather than names, as the authoritative selection key.
- [x] Expose target enumeration and lookup diagnostics.
- [x] Keep presentation target identity independent from ECS entity identity.

Acceptance criteria:

- [x] A known GLB mesh part can be selected by a stable ID across repeated
      loads of identical bytes.
- [x] Unknown and ambiguous target requests are diagnosed.
- [x] TypeScript does not parse GLB, FBX, SVG, or CGM to discover target IDs.

### Slice 5: Rust Corpus Proof

Deliverables:

- [x] Extend the GLB corpus example with a focused native material and
      presentation-override proof.
- [x] Render one mesh and one vector source through the same override
      vocabulary.
- [x] Demonstrate source, selected, hotspot, transparent, hidden, and restored
      mesh states through an opt-in per-draw override command.
- [x] Capture structural artifacts and deterministic CPU or native visual
      evidence where appropriate. `hello-glb` emits
      `target/hello-glb/presentation-state.json` for the source material,
      resolved presentation, selected pipeline, and transparency policy. This
      is structural evidence, not a GPU framebuffer capture.
- [x] Record draw count, binding allocations, uniform writes, mesh uploads, and
      selected pipeline in the native GLB override corpus telemetry.

Acceptance criteria:

- [x] The example proves color and opacity overrides without custom application
      WGSL.
- [x] Transparent presentation exposes its documented depth and ordering
      behavior: alpha blending, `LessEqual` depth test, no depth writes, and
      submission-order compositing only. Intersecting transparent geometry
      remains explicitly unsupported.
- [x] Structural artifacts identify source material, the resolved override
      result, parameters, and selected pipeline.
- [ ] The example remains interactive within the admitted performance budget.

### Slice 6: WASM Runtime Presentation API

Deliverables:

- [x] Expose target enumeration and bounded override commands through the
      provider-neutral WASM consumer boundary.
- [x] Add asset-workbench controls for selection, tint, opacity, visibility, and
      reset.
- [x] Keep TypeScript state limited to user interaction and displayed
      observations.
- [x] Return structured diagnostics for invalid targets and values. Unsupported
      effects remain outside this bounded v1 command surface.
- [x] Preserve native/WASM semantic parity for the admitted target and override
      command: one serialized GLB hotspot request resolves to the same
      provider-neutral value through the WASM session boundary and direct native
      `PresentationControl`.

Acceptance criteria:

- [x] A browser user can select a known model part, recolor it, make it
      translucent, and restore it.
- [x] The browser does not parse source asset formats or construct backend
      shaders.
- [x] The same override request produces equivalent resolved Tokimu data on
      native and WASM paths.
- [x] Preview layout remains bounded while controls and diagnostics change.

### Slice 7: TypeScript Material Authoring Package

Deliverables:

- [ ] Add a focused material authoring package only after the Rust model has a
      real caller.
- [ ] Define typed constructors for material schemas and parameter values.
- [ ] Re-export the stable surface through the `tokimu` anchor.
- [ ] Teach `tokimu-ts-frontend` to recognize, validate, and lower the material
      API.
- [ ] Emit source-mapped diagnostics against authored TypeScript.
- [ ] Generate or validate a serializable engine-owned material artifact.

Acceptance criteria:

- [ ] A TypeScript material definition lowers to the same semantic value as an
      equivalent Rust definition.
- [ ] Type errors and semantic errors identify the original TypeScript source.
- [ ] The npm package contains types and authoring helpers but no renderer or
      runtime.
- [ ] `tokimu-core` and `tokimu-runtime` remain TypeScript-free.

### Slice 8: Tokimu Shader Module Model And WGSL Path

Deliverables:

- [ ] Define shader stages, entry points, bindings, vertex inputs, and parameter
      references as Tokimu-owned data.
- [ ] Support validated hand-written WGSL as the first author path.
- [ ] Validate shader bindings against material schemas and mesh vertex layouts.
- [ ] Store generated backend handles only inside renderer adapters.
- [ ] Preserve explicit pipeline selection at draw submission.

Acceptance criteria:

- [ ] A Rust-authored shader module and pipeline can render the corpus scene.
- [ ] Binding and vertex-layout mismatches fail before an invalid draw.
- [ ] Shader compile diagnostics retain module and entry-point identity.
- [ ] No wgpu shader module or bind group leaks into public semantic types.

### Slice 9: Restricted TypeScript Shader Lowering

Deliverables:

- [ ] Add the reconciled shader authoring package after the Rust shader model is
      stable under corpus use.
- [ ] Support a deliberately small typed subset: stage declarations, uniforms,
      samplers, scalar/vector/matrix math, and deterministic control flow.
- [ ] Reject unsupported TypeScript and host-dependent APIs explicitly.
- [ ] Lower ahead of time through `tokimu-ts-frontend` into the shader semantic
      model and generated WGSL.
- [ ] Preserve source maps or equivalent source-location diagnostics.

Acceptance criteria:

- [ ] One TypeScript-authored shader renders equivalently to a hand-written WGSL
      reference.
- [ ] Generated WGSL is deterministic for identical input.
- [ ] Unsupported syntax produces a stable diagnostic and never falls back to
      runtime JavaScript.
- [ ] Runtime shader compilation from arbitrary browser TypeScript is not
      introduced.

### Slice 10: Diagnostics, Performance, And Security Hardening

Deliverables:

- [ ] Add diagnostics for unknown targets, unknown parameters, invalid values,
      incompatible pipelines, shader compilation, and backend capability gaps.
- [ ] Add counters for material resolution, binding writes, pipeline switches,
      transparent draws, and derived-resource cache behavior.
- [ ] Bound shader source size, parameter count, generated WGSL size, and
      lowering complexity.
- [ ] Ensure authored shader definitions cannot access files, network, DOM,
      timers, process state, or ambient randomness.
- [ ] Add malformed and adversarial authoring fixtures.

Acceptance criteria:

- [ ] Normal steady-state overrides do not recreate pipelines.
- [ ] Repeated unchanged frames do not allocate material bindings.
- [ ] Invalid or excessive input fails within documented bounds.
- [ ] Diagnostics identify the owning stage: TypeScript recognition, semantic
      validation, WGSL generation, pipeline validation, or backend compilation.

### Slice 11: Admission Review And Documentation

Deliverables:

- [ ] Compare the implemented boundaries against the SDD, TTSDD, and roadmap.
- [ ] Record evidence from Rust, native rendering, WASM, and TypeScript corpus
      consumers.
- [ ] Decide whether shader/material semantic models have earned dedicated
      capability crates.
- [ ] Update the SDD and TTSDD with accepted package names and lifecycle.
- [ ] Open or update an ADR if ownership or dependency direction becomes a
      binding decision.
- [ ] Archive superseded exploratory notes without deleting evidence.

Acceptance criteria:

- [ ] Accepted and deferred findings are explicit.
- [ ] Package or crate extraction follows demonstrated independent consumers.
- [ ] No plan checkbox is treated as architectural authority by itself.
- [ ] Future work can distinguish presentation overrides, material authoring,
      shader authoring, and renderer execution without ambiguity.

## Validation Matrix

| Boundary | Required evidence |
|---|---|
| Material model | Rust unit tests and serialization round trips |
| Pipeline state | native opaque/transparent render corpus |
| Overrides | shared source material with independent per-target results |
| Target identity | repeated GLB/FBX load and stable lookup |
| WASM API | asset-workbench interaction and structured diagnostics |
| TS materials | TypeScript typecheck plus Rust semantic parity |
| WGSL path | shader compile success/failure corpus |
| TS shaders | deterministic lowering and visual parity |
| Performance | warm-up versus steady-state counters |
| Portability | native and `wasm32-unknown-unknown` validation |

Preferred validation commands include:

```text
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run typecheck
```

Focused native and browser corpus runs remain necessary because successful
compilation does not prove blending, depth ordering, target selection, or visual
parity.

## Non-Goals

This plan does not initially provide:

- a node-based shader editor;
- arbitrary runtime TypeScript-to-WGSL compilation;
- a general JavaScript shader language;
- order-independent transparency;
- physically based material fidelity;
- importer-specific material objects in public APIs;
- automatic mutation of simulation or imported asset truth;
- unrestricted custom bind-group layouts;
- silent substitution by Three.js, Babylon.js, browser SVG, or another
  presentation implementation;
- promotion of shader or material semantics into `tokimu-core`.

## Risks

### Material and pipeline ownership collapse

Allowing a material to select its shader or pipeline would contradict the SDD's
explicit draw-submission boundary. Keep compatible defaults convenient without
making selection implicit.

### Runtime and authoring APIs blur together

Interactive tint and opacity are data updates. Shader definition is compilation.
Keep separate APIs, artifacts, diagnostics, and lifecycle.

### Transparency appears correct only in simple scenes

Alpha blending can hide incorrect depth writes and ordering. Use overlapping
and nested geometry in the corpus and state unsupported guarantees precisely.

### Imported names are unstable

Source names may be absent or duplicated. Stable target identity must derive
from deterministic importer observations, not browser labels alone.

### TypeScript becomes an alternate engine

TypeScript must author or request Tokimu semantics. It must not own importer
state, GPU resources, render scheduling, or simulation truth.

### Generalized shader APIs arrive before callers

Land the Rust material and override model first. Add the shader semantic model
only after hand-written WGSL and binding validation expose the necessary shape.
Add TypeScript packages last.

## Completion Criteria

The first complete version of this effort is reached when:

- a Rust and WASM consumer can address one model part or vector record;
- independent targets sharing one source material can receive tint and opacity
  overrides;
- transparent 3D behavior has explicit blend, depth, culling, and ordering
  semantics;
- browser TypeScript can apply and clear bounded presentation overrides without
  parsing source formats or authoring backend shaders;
- a typed TypeScript material definition lowers to a Tokimu-owned material
  artifact;
- one restricted TypeScript shader lowers deterministically to WGSL and matches
  a hand-authored reference;
- failures are diagnosed at the owning boundary;
- native and WASM evidence remain semantically equivalent;
- documentation records which parts remain incubating and which boundaries have
  earned admission.
