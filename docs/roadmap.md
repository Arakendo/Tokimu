# Tokimu Roadmap

Short-term focus:

1. Close out MVP 1 by finishing the M1 loop seam and the M4 renderer gaps
2. Get a monoscopic 3D scene rendering (perspective camera + depth + a cube)
3. Add the second camera for stereo only after the mono 3D scene is solid

The main planning detail remains in the software design document.

Each in-progress milestone below now carries a **Deliverables** checklist of
concrete, buildable tasks. A milestone flips to `[x]` when its deliverables are
done and exercised by an example or test; the prose above each list is context,
not the task.

## MVP Phase Tracker

This groups the SDD milestones (M0–M12) into MVP phases for progress tracking.
The SDD remains the source of truth for intended architecture and acceptance
criteria; this tracker only reflects delivery status. Update the checkboxes as
milestones land.

Status legend: `[x]` done, `[~]` in progress / partial, `[ ]` not started.

### MVP 1 — Runnable Engine Core

Goal: a native engine that boots, ticks, takes input, and draws.

- [x] M0 workspace scaffold — crates, facade feature flags, `cargo test` green
- [~] M1 runtime loop skeleton — `App`/`RuntimeConfig`/`Plugin`, fixed-step
  accounting, run-loop diagnostics, callback-driven host-loop helpers, and a
  real example caller now exist; the native platform still owns the outer loop
  - Deliverables:
    - [ ] Give `App` a single `run_frame(delta) -> FrameOutcome` entry the
      platform calls, so cadence lives in runtime not the winit callback
    - [ ] Return an explicit `FrameOutcome::{Continue, Exit}` and have the
      native loop honor it instead of tracking exit state itself
    - [ ] Cover the frame seam with a headless test that ticks N frames without
      a window
- [x] M2 minimal ECS — `EntityId`, `Schedule`/`Phase`, `FixedTimeStep` exist;
  `World` now has typed component/resource storage, simple iteration helpers,
  directional relationship edges, a minimal query surface, simple system
  execution and registration, configurable phase ordering, and phase-based
  system removal, named system sets, and explicit priorities; dependency-aware
  ordering is present
- [x] M3 window + input — `hello-window` proves the native input-to-intent path
  (WASM side of the platform seam still pending)
- [~] M4 renderer spike — `hello-triangle` proves real `wgpu` bring-up, explicit
  mesh/material/pipeline upload, renderable handles, per-draw placement, and an
  orthographic camera; custom WGSL pipeline support now exists, but deeper
  shader/pipeline management is still missing
  - Deliverables:
    - [ ] Add a pipeline registry so multiple named pipelines resolve by handle
      at draw time
    - [ ] Add a depth texture + depth-stencil state to `WgpuBackend` (required
      before any 3D draw is correct)
    - [ ] Add a perspective camera mode alongside the existing orthographic path

MVP 1 exit criteria: a native example opens a window, reads normalized input,
and renders Tokimu-owned resources through a real backend without the renderer
owning simulation state.

### MVP 2 — First Playable + Browser Parity

Goal: a small playable loop that also proves the shared core runs in a browser.

- [~] M5 WASM spike — the browser demo now runs through `tokimu-runtime::App`
  and world resources while the canvas remains the presentation layer; the
  shared-core/browser parity path is still being tightened up
  - Deliverables:
    - [ ] Run the same toy step system in the browser as native (shared code,
      not a parallel `wasm-demo` loop)
    - [ ] Drive the canvas draw from world state instead of demo-only fields
    - [ ] Confirm identical input-to-state behavior native vs browser for one
      documented input sequence
- [~] M6 first playable toy — `hello-triangle` carries a small collect-the-target
  loop over shared input and Tokimu world state; still native-only and not yet
  a distinct world-corpus scenario
  - Deliverables:
    - [ ] Add a win/lose or round condition so the loop has a beginning and end
    - [ ] Move the toy step logic into a reusable system callable from native
      and browser
    - [ ] Name the scenario as a small world-corpus example, not just a triangle
- [x] M6.5 networking boundary note — future transport/replication boundary
  documented before any socket code appears

MVP 2 exit criteria: the same core powers a native playable toy and a browser
build, with input, simulation, presentation, and rendering visibly separated.

### MVP 3 — Content and Persistence

Goal: describe worlds as data and move them in and out of the runtime.

- [~] M7 persistence boundary — engine-facing save/load model and crate boundary
  now documented; scene/project documents remain distinct from runtime world
  state
  - Deliverables:
    - [ ] Pick the first serialization format (RON is the leading candidate) and
      record the choice in the SDD
    - [ ] Define the save/load trait seam that a future `tokimu-persistence`
      crate would implement, without adding the crate yet
    - [ ] Round-trip one resource (e.g. the toy state) to a string and back in a
      test to prove the boundary shape
- [~] M8 scene and history model — scene document shape, scene-to-world compile
  path, and a provenance/timeline history direction
  - Deliverables:
    - [ ] Define a minimal scene document struct (entities + components as data)
    - [ ] Write a `compile_scene(&SceneDoc) -> World` translation step with a
      test that spawns the described entities
    - [ ] Sketch the diff/history record shape (what changed, which system, why)
      as a documented type, even if unused at first

MVP 3 exit criteria: declarative scene documents compile into runtime world
state, with persistence staying downstream of the world model.

### MVP 4 — Tooling and Authoring

Goal: make the world inspectable and author-facing content approachable.

- [~] M9 inspector and rule frontends — inspection-first editor target, visual
  rule-graph direction, and TypeScript-first authoring direction documented
  - Deliverables (inspector v0):
    - [ ] Add a read-only world snapshot API the inspector can walk (entities,
      components, resources) without mutating state
    - [ ] Ship a text or console dump of that snapshot as the editor v0 proof
  - Deliverables (semantic rule model — prerequisite for any frontend):
    - [ ] Define a v0 rule as a named system-like transformation with declared
      inputs, outputs, and emitted signals (SDD 5.11)
    - [ ] Land the rule model in an engine-owned `tokimu-rule` crate, exercised
      by at least one real example before any frontend crate exists
    - [ ] Prove scene data and a hand-written rule both lower into the same
      runtime systems (SDD phases 1–2)
  - Deliverables (TypeScript authoring — the primary user-facing surface):
    - [ ] Stand up a top-level `frontends/` npm workspace kept separate from
      `crates/`, with `@tokimu/rules` as the first package targeting the rule
      model
    - [ ] Add a Rust `tokimu-ts-frontend` host that owns semantic validation and
      lowering via the TypeScript Compiler API (ts-morph for the first
      prototype), rather than reimplementing a parser or typechecker
    - [ ] Support only the explicit v0 subset — `rule()`, `query()`, `signal()`,
      `relation()`, `command()`, deterministic loops, and arithmetic — and
      reject ambient I/O, DOM, `async`/`Promise`, `Date`, `Math.random`,
      `fetch`, and `eval`
    - [ ] Implement the ahead-of-time lowering path (TS source → tsc + typecheck
      → recognized `tokimu` API calls → lowering pass → semantic rule model →
      runtime systems), SDD phase 3; keep an embedded JS host (phase 4)
      explicitly out of scope
    - [ ] Prove one `@tokimu/rules`-authored rule lowers and runs identically to
      its hand-written Rust equivalent, so "Tokimu supports the `tokimu`
      package" stays truer than "Tokimu supports TypeScript"

MVP 4 exit criteria: an inspection-first editor direction exists and authoring
frontends are framed over the world/rule model rather than as alternate cores.

### MVP 5 — Platform and Presentation Expansion

Goal: prove Tokimu can grow into new presentation and transport surfaces.

- [~] M10 VR/XR architecture spike — monoscopic 3D camera proof first, then
  stereo views, tracked spaces, and headset frame submission as a presentation
  over the shared core
  - Deliverables (mono 3D first):
    - [ ] Add a `Mesh::cube()` (or similar) 3D mesh with real vertex normals
    - [ ] Add a perspective camera + depth testing to the renderer (shared with
      the M4 depth deliverable)
    - [ ] New `hello-3d` example: one rotating lit cube under a perspective
      camera, driven by world state
  - Deliverables (stereo second):
    - [ ] Extend the camera concept to emit two per-eye views from one shared
      pose input
    - [ ] Render both eyes to side-by-side viewports in `hello-3d` as a desktop
      stereo proof
    - [ ] Keep any headset/session setup behind a platform adapter, not in core
- [~] M11 networking and transport spike — replication unit and transport seam
  documented for native and browser targets
  - Deliverables:
    - [ ] Name the first replication unit concretely (commands vs snapshots vs
      deltas) and record the choice
    - [ ] Define a transport trait seam that native and browser backends could
      implement, with no protocol code yet
    - [ ] Prove the seam by serializing one replication message to bytes and
      back in a test
- [~] M12 text / MUD architecture spike — text-first presentation, command
  dispatch, and transcript flow documented as an adapter over the shared core
  - Deliverables:
    - [ ] Implement a tiny command parser for the primitives look, list, step,
      emit, why
    - [ ] Add a text adapter that renders world state as room/status text from
      the same world used by the graphical toy
    - [ ] Prove one command loop end-to-end in a `hello-text` example or test

MVP 5 exit criteria: VR/XR, networking, and text-first presentation each have a
named integration path layered over the same simulation core.

## Rendering, Shaders, and Assets

Cross-cutting capability deliverables that several milestones share. These
sharpen the "draws" half of MVP 1 and directly unblock 3D (M10) and browser
parity (M5). The SDD keeps rendering a consumer of world state and keeps asset
loading independent of direct filesystem access.

### Asset loading (`tokimu-assets`, SDD 5.5)

- [ ] Implement the first concrete `AssetLoader` (the trait is still a stub) that
  turns `&[u8]` into a Tokimu-owned resource such as a mesh or texture
- [ ] Keep loading filesystem-free: the platform layer supplies bytes so the same
  loader path works on native and WASM (SDD design invariant)
- [ ] Add a WASM-friendly async fetch abstraction that feeds bytes into the same
  `AssetStore` / `AssetHandle<T>` flow used natively
- [ ] Prove the path in an example: load a mesh or texture from bytes through
  `AssetStore` and hand its handle to the renderer
- [ ] Defer embedded asset bundles until an example actually needs them

### Drawing (`tokimu-render`, SDD 5.3)

- [ ] Add a textured draw path: a texture + sampler bind group so a material can
  reference a `TextureHandle` instead of only a solid color
- [ ] Keep pipeline selection explicit at draw submission time — materials
  describe bound data, draw commands pick mesh + material + pipeline
- [ ] Add batching/instancing for repeated renderables so the toy and 3D scene
  scale past a handful of draws
- [ ] Keep 2D and 3D on one shared command/handle model rather than forking
  sprite and mesh paths

### Shader authoring (WGSL)

- [ ] Document the minimal author path: supply WGSL + a Tokimu pipeline
  descriptor, upload it by handle, and select it at draw time
- [ ] Ship a small shared shader set: solid-color 2D, textured 2D, and a lit 3D
  shader for the `hello-3d` cube
- [ ] Add a shader/pipeline registry so multiple named WGSL pipelines resolve by
  handle (shared with the M4 pipeline-registry deliverable)
- [ ] Surface shader compile failures as explicit diagnostics rather than silent
  fallback (SDD prefers explicit diagnostics over silent fallback)

### Shader authoring in TypeScript (exploratory, lowers to WGSL)

Authors should be able to write most shader logic in TypeScript, consistent with
the TS-first authoring direction (SDD 5.11). This reuses the generalized frontend
pattern the SDD already describes — TypeScript syntax → domain-specific Tokimu
API → domain-specific semantic model → target compiler — applied to shading
instead of rules. WGSL above stays the engine-owned target; TypeScript is an
ahead-of-time frontend that lowers into it, never a runtime transpiler.

- [ ] Define a Tokimu-owned shader/material semantic model that both WGSL and a
  future TS frontend lower into (land this before the TS frontend, mirroring the
  rule-model-before-frontend ordering in M9)
- [ ] Prototype a `@tokimu/shaders` package exposing a restricted, typed shading
  API (vertex/fragment entry points, uniforms, samplers, vector/matrix math,
  deterministic control flow) that lowers ahead of time to WGSL
- [ ] Reuse the `tokimu-ts-frontend` lowering host and its diagnostics rather
  than standing up a second TS toolchain; the shader frontend is another lowering
  target, not a new compiler
- [ ] Support only an explicit subset and reject host-dependent features (ambient
  I/O, DOM, `async`/`Promise`, `fetch`, `eval`), the same way the rule frontend
  does
- [ ] Prove parity: one shader authored in `@tokimu/shaders` lowers to WGSL that
  renders identically to a hand-written WGSL equivalent
- [ ] Keep this gated behind the WGSL path and the rule-model lowering work — it
  is a phase-3 ahead-of-time frontend, not an embedded runtime shader compiler

### Other TypeScript authoring surfaces to explore

Same rule for all of these: author in a restricted, typed TypeScript subset that
lowers ahead of time into a Tokimu-owned semantic model, reusing the shared
`tokimu-ts-frontend` host and diagnostics. Each is a phase-3 AOT frontend, not an
embedded runtime, and none should exist before its engine-owned model has real
Rust callers. WGSL, world state, and the rule model stay the engine-owned
targets; TypeScript is always the frontend, never the runtime.

Named in the SDD frontends layout (5.11–5.12):

- [ ] `@tokimu/scenes` → scene model: declarative scene/prefab documents that
  lower into the M8 scene-to-world compile path
- [ ] `@tokimu/query` → query model: entity/component queries that lower into the
  engine query surface instead of ad hoc iteration
- [ ] `@tokimu/ui` → presentation model: screen-space HUD and world-space
  interface bindings over shared world state, not a separate UI runtime

Further candidates grounded in the v0 primitives (`signal()`, `relation()`,
`command()`) and existing crates:

- [ ] `@tokimu/materials` → shader/material model: material definitions and bound
  data that pair with `@tokimu/shaders` and lower to the same pipeline model
- [ ] `@tokimu/commands` → command model: the M12 text/MUD command surface (look,
  list, step, emit, why) authored via the `command()` primitive
- [ ] `@tokimu/signals` + `@tokimu/relations` → signal and relationship models:
  `signal()` and `relation()` declarations lowering into engine events and
  relationship edges
- [ ] `@tokimu/input` → action-map model: input action maps that lower into
  `tokimu-input` bindings rather than hand-wiring key codes
- [ ] Visual rule graphs stay a frontend over the same rule model, interchangeable
  with the TypeScript rule frontend rather than a separate runtime

### Cross-cutting considerations to keep visible

These are not separate MVPs, but they will shape whether the authoring and 3D
work lands cleanly.

- [ ] Diagnostics: every asset, shader, scene, and TypeScript lowering path
  should emit structured, actionable diagnostics rather than opaque failures
- [ ] Frontend/model versioning: define how authored TS content and semantic
  models version together so older content can still lower or fail clearly
- [ ] Asset inspection/importers: keep room for an asset browser, importer
  pipeline, and content metadata without making `tokimu-render` own asset
  translation
- [ ] Validation workflow: prove examples and frontends through runnable checks,
  not docs alone; authored content should have a compile/check command
- [ ] Hot reload / iteration loop: decide when scene, shader, or TS-authored
  content should reload in-place versus requiring rebuild/restart
- [ ] Packaging and distribution: define how authored scenes, rules, shaders,
  and assets are bundled for native and WASM without coupling core crates to
  npm or filesystem assumptions

### Phase Status At A Glance

Phases can overlap. The tracker shows where work is currently being invested,
not a strict promise that every earlier MVP exit criterion is complete first.

- MVP 1: partial — native runnable core is in place, but the host loop remains
  partial and the renderer spike is still incomplete
- MVP 2: partial — the first playable toy exists, but WASM parity is still
  pending
- MVP 3: partial — persistence, scene, and history groundwork is now being
  documented
- MVP 4: partial — inspector and rule frontend direction is now being
  documented
- MVP 5: partial — VR/XR, networking, and text-first presentation spikes are
  now being documented

## Implementation Status Snapshot

Snapshot as of the last audit. This is volatile; the SDD remains the source of
truth for intended architecture. Update this list as milestones land.

### Core And Runtime

- M0 skeleton: done. Workspace builds, all documented crates exist, facade
  feature flags (`render`, `platform`) are wired correctly, `cargo test` passes.
- `tokimu-core`: partial. `EntityId`, `Schedule`/`Phase`, `FixedTimeStep`, and
  `Diagnostics` exist. `World` now has typed component/resource storage,
  iteration helpers, directional relationship edges, and a minimal query
  surface; `Schedule` now supports custom phase order and a system registry.
  `Component`, `Resource`, and `Event` are still trait stubs.
- `tokimu-runtime`: partial. `App`, `RuntimeConfig`, `Plugin`, and
  `tick_fixed_updates()` exist, plus input application, run-loop diagnostics,
  callback-driven host-loop helpers, and simple system wiring through
  `Schedule` and the app-owned registry, including phase-based removal and
  named sets. The native example now calls the frame seam.
- `tokimu-input`: the most complete crate. Keyboard, mouse, controller, action
  map, and aggregate `InputState` are implemented with tests.
- `tokimu-assets`: partial. `AssetId`, `AssetHandle<T>`, and `AssetStore` exist;
  `AssetLoader` is still a trait stub.

### Platform And Rendering

- `tokimu-render`: partial. Types and handles exist, and `WgpuBackend` now does
  real `wgpu` bring-up, surface creation, and explicit mesh/material/pipeline
  uploads. Reusable renderable handles and per-draw placement are in place, and
  custom WGSL pipeline support exists, but generalized shader/pipeline
  management is still missing.
- `tokimu-platform`: partial. `WindowConfig`, `Clock`, and Tokimu-owned input
  event types exist; the native path creates a real `winit` window, translates
  keyboard/mouse/resize/close events, and exposes window-run helpers. WASM
  support is still a placeholder.
- `tokimu-wasm`: browser demo scaffold driven by `tokimu-runtime::App`, with a
  Tokimu world resource holding scene state and the canvas acting as the view.

### Example Proofs

- `hello-window` is the live proof for M3. It opens a native window, translates
  WASD into runtime-owned input state, and keeps the surface intentionally blank.
- `hello-triangle` is the live proof for M4 and the current M6 seed. It opens a
  native window, brings up `wgpu`, draws multiple 2D shapes with explicit
  Tokimu-owned resources, and now includes a small collect-the-target loop with
  world-owned toy state.
- `wasm-demo` is now a browser host loop scaffold that is driven from Rust and
  responds to shared input, but it still needs real simulation/runtime content.

## Execution Guardrails

The roadmap is a planning aid, not a work product by itself. Implementation
should prioritize proving new runtime capability over repeatedly polishing
existing examples, documentation, or visual presentation.

For each implementation slice:

1. Name one concrete capability being proven.
2. Define its smallest runnable acceptance check.
3. Make the narrowest implementation that proves it.
4. Run the affected tests and one behavioral smoke check where practical.
5. Update documentation only enough to reflect the resulting implementation
   state.
6. Stop once the acceptance check passes and move to the next unresolved
   capability.

A completed proof should not continue accumulating cosmetic refinements unless a
real application, test, or architectural boundary requires them.

Examples and documents should not become indefinite work loops:

- Do not keep beautifying a demo after its subsystem behavior is proven.
- Do not repeatedly restructure roadmap or SDD wording without a corresponding
  implementation change.
- Do not add another abstraction solely because the current slice could
  theoretically become more general.
- Do not widen a milestone after its acceptance criteria are satisfied.
- Do not create a new crate, framework, DSL, editor, or generalized manager
  until at least one concrete caller requires it.

Before continuing work in the same area, ask:

> Does the next change prove a new capability, remove a demonstrated
> limitation, or protect a real boundary?

If not, stop and choose the next roadmap gap.

## Current Architectural Risks

Work should prefer unresolved architectural risks over cosmetic or incremental
embellishment.

Current priority order:

1. **3D presentation proof**
   - perspective camera
   - 3D transforms
   - depth buffer
   - indexed cube or similarly minimal mesh
   - orbit or free camera
2. **External asset proof**
   - load one GLB
   - convert it into Tokimu-owned mesh/material resources
   - render it natively
   - then prove the same asset path in the browser
3. **Runtime/browser parity**
   - shared world and simulation behavior
   - browser canvas remains a presentation adapter
   - avoid a browser-specific parallel runtime
4. **Scene model**
   - only after hardcoded examples reveal stable repeated concepts
   - scene data remains separate from runtime world state
5. **Authoring frontend**
   - TypeScript comes after the scene/rule vocabulary has real Rust callers
   - TypeScript authors Tokimu-owned meaning rather than becoming a second
     engine core

## Demo Completion Rules

A demo is complete when it proves its intended capability.

`hello-window` is complete enough when it proves:

- native startup
- normalized input
- runtime ticking
- clean shutdown
- readable diagnostics

It should not become a general native application shell unless another
milestone requires that.

`hello-triangle` is complete enough when it proves:

- renderer bring-up
- explicit resources
- camera/view
- input-driven state
- world-to-presentation flow

It should not continue accumulating decorative scene elements, additional
mini-games, or UI polish merely because they are locally easy to add.

`wasm-demo` should now focus on:

- reuse of runtime/world behavior
- browser-host integration
- parity with native concepts
- eventual external asset presentation

It should not become an isolated canvas application with its own private engine
conventions.

## Documentation Budget

Documentation changes should accompany one of these:

- a new implemented capability
- a changed ownership boundary
- a revised acceptance criterion
- a newly discovered architectural gap
- a decision that prevents likely future drift

Pure wording improvements should be batched rather than interrupting
implementation repeatedly.

Spend no more than one documentation pass per completed implementation slice
unless the requested task is explicitly documentation-focused.

### Next Wiring Steps

### 1. First 3D Proof

Acceptance criteria:

- Tokimu-owned perspective camera exists.
- Tokimu-owned 3D placement uses translation, rotation, and scale.
- A depth attachment is created and used correctly.
- An indexed cube or equivalent 3D mesh renders.
- Camera movement or orbit demonstrates spatial depth.
- Existing 2D support remains intact.

Explicit non-goals:

- lighting
- shadows
- PBR
- GLB loading
- scene graphs
- skeletal animation
- render graphs
- more roadmap grooming

### 2. First GLB Proof

Acceptance criteria:

- one deliberately chosen GLB file loads through `tokimu-assets`
- importer output becomes Tokimu-owned resources
- renderer does not know that the source format was GLB
- unsupported features produce structured diagnostics
- the same imported representation is usable by native and WASM hosts

Explicit non-goals:

- complete glTF coverage
- FBX
- hot reload
- asset editor
- animation blending
- production-grade material fidelity

### 3. Browser Parity Proof

Acceptance criteria:

- the browser and native examples run the same small world behavior
- both consume the same runtime-owned state
- only platform and presentation adapters differ
- no duplicate browser-only simulation loop exists

After these three proofs, keep M11 transport and M12 text / MUD behind the 3D
and parity work.
