# Tokimu Roadmap

Short-term focus:

1. Name the first replication unit concretely and prove the transport seam
2. Keep the browser-parity proof documented and stable while transport work
  lands
3. Keep OpenXR shelved until explicitly reopened

The main planning detail remains in the software design document.

Each in-progress milestone below now carries a **Deliverables** checklist of
concrete, buildable tasks. A milestone flips to `[x]` when its deliverables are
done and exercised by an example or test; the prose above each list is context,
not the task.

Roadmap scope:

- The roadmap answers what Tokimu should prove next, what "done" means for a
  slice, what is blocked, and what is not yet being worked on.
- The SDD remains the place for subsystem design, invariants, rationale, and
  ownership boundaries.
- The roadmap is permission to change direction when implementation reveals a
  better boundary or a truer architectural risk. It is sequencing guidance, not
  a contract.

## MVP Phase Tracker

This groups the SDD milestones (M0–M12) into MVP phases for progress tracking.
The SDD remains the source of truth for intended architecture and acceptance
criteria; this tracker only reflects delivery status. Update the checkboxes as
milestones land.

Status legend: `[x]` done, `[~]` in progress / partial, `[ ]` not started.

### MVP 1 — Runnable Engine Core

Goal: a native engine that boots, ticks, takes input, and draws.

- [x] M0 workspace scaffold — crates, facade feature flags, `cargo test` green
- [x] M1 runtime loop skeleton — `App`/`RuntimeConfig`/`Plugin`, fixed-step
  accounting, run-loop diagnostics, a shared `FrameOutcome` frame seam, and a
  real example caller now exist; the native platform delegates frame outcomes
  to runtime
  - Deliverables:
    - [x] Give `App` a single `run_frame(delta) -> FrameOutcome` entry the
      platform calls, so cadence lives in runtime not the winit callback
    - [x] Return an explicit `FrameOutcome::{Continue, Exit}` and have the
      native loop honor it instead of tracking exit state itself
    - [x] Cover the frame seam with a headless test that ticks N frames without
      a window
- [x] M2 minimal ECS — `EntityId`, `Schedule`/`Phase`, `FixedTimeStep` exist;
  `World` now has typed component/resource storage, simple iteration helpers,
  directional relationship edges, a minimal query surface, simple system
  execution and registration, configurable phase ordering, and phase-based
  system removal, named system sets, and explicit priorities; dependency-aware
  ordering is present
- [x] M3 window + input — `hello-window` proves the native input-to-intent path
  (WASM side of the platform seam still pending)
- [x] M4 renderer spike — `hello-triangle` proves real `wgpu` bring-up, explicit
  mesh/material/pipeline upload, renderable handles, per-draw placement, and a
  perspective camera with depth-buffer support; custom WGSL pipeline support
  now exists, and the named pipeline registry is in place
  - Deliverables:
    - [x] Add a pipeline registry so multiple named pipelines resolve by handle
      at draw time
    - [x] Add a depth texture + depth-stencil state to `WgpuBackend` (required
      before any 3D draw is correct)
    - [x] Add a perspective camera mode alongside the existing orthographic path

MVP 1 exit criteria: a native example opens a window, reads normalized input,
and renders Tokimu-owned resources through a real backend without the renderer
owning simulation state.

### MVP 2 — First Playable + Browser Parity

Goal: a small playable loop that also proves the shared core runs in a browser.

- [x] M5 WASM spike — the browser demo now runs through `tokimu-runtime::App`
  and world resources while the canvas remains the presentation layer; the
  shared-core/browser parity path is proven for the documented `right, right,
  up, up` sequence in the prototype
  - Deliverables:
    - [x] Run the same toy step system in the browser as native (shared code,
      not a parallel `wasm-demo` loop)
    - [x] Drive the canvas draw from world state instead of demo-only fields
    - [x] Confirm identical input-to-state behavior native vs browser for one
      documented input sequence
- [x] M6 first playable toy — `hello-triangle` carries a small collect-the-target
  loop over shared input and Tokimu world state; it now uses the shared field
  sprint step logic and a named round loop
  - Deliverables:
    - [x] Add a win/lose or round condition so the loop has a beginning and end
    - [x] Move the toy step logic into a reusable system callable from native
      and browser
    - [x] Name the scenario as a small world-corpus example, not just a triangle
- [x] M6.5 networking boundary note — future transport/replication boundary
  documented before any socket code appears

MVP 2 exit criteria: the same core powers a native playable toy and a browser
build, with input, simulation, presentation, and rendering visibly separated.

### MVP 3 — Content and Persistence

Goal: describe worlds as data and move them in and out of the runtime.

- [x] M7 persistence boundary — engine-facing save/load model and crate boundary
  now documented; scene/project documents remain distinct from runtime world
  state
  - Deliverables:
    - [x] Pick the first serialization format (RON is the leading candidate) and
      record the choice in the SDD
    - [x] Define the save/load trait seam that a future `tokimu-persistence`
      crate would implement, without adding the crate yet
    - [x] Round-trip one resource (e.g. the toy state) to a string and back in a
      test to prove the boundary shape
- [x] M8 scene and history model — scene document shape, scene-to-world compile
  path, and a provenance/timeline history direction
  - Deliverables:
    - [x] Define a minimal scene document struct (entities + components as data)
    - [x] Write a `compile_scene(&SceneDoc) -> World` translation step with a
      test that spawns the described entities
    - [x] Sketch the diff/history record shape (what changed, which system, why)
      as a documented type, even if unused at first

MVP 3 exit criteria: declarative scene documents compile into runtime world
state, with persistence staying downstream of the world model.

### MVP 4 — Tooling and Authoring

Goal: make the world inspectable and author-facing content approachable.

- [~] M9 inspector and rule frontends — inspection-first editor target, visual
  rule-graph direction, and TypeScript-first authoring direction documented;
  the TTSDD now defines the authoritative authoring surface, so the remaining
  work is host/tooling execution rather than frontend-direction discovery
  - Deliverables (inspector v0 — SDD 5.9 names six concrete surfaces):
    - [x] Add a read-only world snapshot API the inspector can walk (entities,
      components, resources) without mutating state
    - [x] Ship a text or console dump of that snapshot as the editor v0 proof
    - [x] Entity/world tree: list live entities and their relationship edges
      from the snapshot
    - [x] Trait inspector: show one entity's component values by name
    - [x] Asset browser: list loaded `AssetId`/`AssetHandle<T>` entries and
      their source
    - [x] System timing panel: surface `Schedule`/`Diagnostics` timing already
      collected by the runtime, not a new profiler
    - [x] Signal log: print emitted rule/event signals in arrival order
    - [x] Relationship viewer: render the directional relationship edges
      `World` already tracks
    - [x] All six stay text/console output for v0; a graphical `egui` shell is
      out of scope until the console proof is solid
  - Deliverables (semantic rule model — prerequisite for any frontend):
    - [x] Define a v0 rule as a named system-like transformation with declared
      inputs, outputs, and emitted signals (SDD 5.11)
    - [x] Land the rule model in an engine-owned `tokimu-rule` crate, exercised
      by at least one real example before any frontend crate exists
    - [x] Prove scene data and a hand-written rule both lower into the same
      runtime systems (SDD phases 1–2)
  - Deliverables (TypeScript authoring — the primary user-facing surface):
    - [x] Stand up a top-level `frontends/` npm workspace kept separate from
      `crates/`, with `@tokimu/rules` as the first package targeting the rule
      model
      - [x] Write the Tokimu TypeScript Design Document (TTSDD) as the
        authoritative authoring-surface spec, superseding the exploratory
        `archive/scripting-typescript.md`
      - [x] Scaffold the authoring skeleton: `tokimu` import anchor, an enriched
        `@tokimu/rules` (execution mode + `rule`/`query`/`signal`/`relation`/
        `command` + `loweredRule`/`runtimeAction`), and an `@tokimu/examples`
        package with lowered and runtime samples
    - [ ] Add a Rust `tokimu-ts-frontend` host that owns semantic validation and
      lowering via the TypeScript Compiler API (ts-morph for the first
      prototype), rather than reimplementing a parser or typechecker
      - [x] Prototype host lowers constrained `rule(...)` source and emits
        explicit diagnostics for runtime-only or unsupported constructs
      - [ ] Retool the current hand-rolled recognizer into resolved-symbol
        Compiler API / `ts-morph` recognition so `tokimu` re-exports, aliases,
        and direct `@tokimu/*` imports all resolve through symbol identity
    - [ ] Support only the explicit v0 subset — `rule()`, `query()`, `signal()`,
      `relation()`, `command()`, deterministic loops, and arithmetic — and
      reject ambient I/O, DOM, `async`/`Promise`, `Date`, `Math.random`,
      `fetch`, and `eval`
      - [x] Rule declarations can carry execution mode plus declared inputs,
        outputs, and signals in the prototype
      - [x] The prototype host recognizes `query()`, `signal()`, `relation()`,
        `command()`, deterministic loops, and arithmetic as explicit lowering-
        boundary constructs
      - [ ] Lower those recognized constructs into semantic rule-model meaning
        instead of only reporting them in the prototype plan
    - [ ] Implement the ahead-of-time lowering path (TS source → tsc + typecheck
      → recognized `tokimu` API calls → lowering pass → semantic rule model →
      runtime systems), SDD phase 3; keep the runtime host boundary separate
      from any later embedded JS engine choice
      - [x] Define the lowering-boundary API-call model for recognized Tokimu
        calls in the host prototype
      - [ ] Add the execution-manifest path for `auto` so execution mode does
        not silently migrate when the lowerer improves
      - [ ] Carry source locations through recognition/lowering so diagnostics
        point back to authored `.ts` instead of internal host state
      - [ ] Define the first semantic-model/version stamp the manifest records
      - [ ] Replace the prototype string-matching recognizer with resolved-
        symbol Compiler API / `ts-morph` recognition so lowering decisions come
        from Tokimu symbol identity rather than source text heuristics
    - [ ] Prove one `@tokimu/rules`-authored rule lowers and runs identically to
      its hand-written Rust equivalent, so "Tokimu supports the `tokimu`
      package" stays truer than "Tokimu supports TypeScript"
      - [x] A TS-authored lowered rule matches the hand-written Rust rule model
        in runtime plan shape
    - [ ] Define the runtime-host contract without committing to an embedded JS
      engine yet
      - [ ] Capability model: runtime code starts with no authority and only
        receives allowlisted capabilities such as `world.query`, `signal.emit`,
        or `ui.open`
      - [ ] Runtime state ownership: ephemeral script-local state is allowed,
        but durable simulation state must live in Tokimu-owned resources or
        components
      - [ ] Lifecycle vocabulary: load, initialize, invoke, suspend, reload,
        dispose

MVP 4 exit criteria: an inspection-first editor direction exists and authoring
frontends are framed over the world/rule model rather than as alternate cores.

### MVP 5 — Platform and Presentation Expansion

Goal: prove Tokimu can grow into new presentation and transport surfaces.

- [x] M10 mono 3D architecture spike — prove the first 3D render path with a
  desktop mono camera over the shared core
  - Deliverables:
    - [x] Add a `Mesh::cube()` (or similar) 3D mesh with real vertex normals
    - [x] Add a perspective camera + depth testing to the renderer (shared with
      the M4 depth deliverable)
    - [x] New `hello-3d-mono` example: a cube under a perspective camera,
      proving
      the first 3D render path
    - [x] Add cube rotation/orbit once the base 3D proof is stable
    - [x] Add lighting once the base 3D proof is stable
    - [x] Render the stereo proof as a separate desktop example in
      `hello-3d-stereo`
- [~] M10.5 VR/XR architecture spike — stereo views, tracked spaces, and
  headset frame submission as a presentation over the shared core
  - Shelved for now. Do not add to or advance this slice unless it is explicitly
    reopened.
  - Deliverables:
    - [x] Extend the camera concept to emit two per-eye views from one shared
      pose input
    - [x] Render both eyes to side-by-side viewports in `hello-3d-stereo` as a
      desktop stereo proof
    - [x] Add `hello-3d-openxr` as a lightweight proof app around the OpenXR
      readiness contract
    - [~] Keep any headset/session setup behind an OpenXR platform adapter, not in core
      - [x] Define a platform-level OpenXR session boundary in `tokimu-platform`
      - [ ] Validate the first headset proof on Quest Pro via SteamVR-backed OpenXR (pending hardware validation)
      - [x] Define a Quest Pro via SteamVR OpenXR session profile in `tokimu-platform`
      - [x] Define a first headset proof plan with stereo eye geometry in `tokimu-platform`
      - [x] Define an OpenXR session capability contract for proof-plan validation
      - [x] Define an OpenXR render bridge contract for stereo frame submission
      - [x] Define an OpenXR session readiness contract for backend validation
      - [ ] Wire a real runtime backend to the boundary on a supported headset
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
  shader for the `hello-3d-mono` cube
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
targets. For these semantic-model surfaces TypeScript is a lowered frontend, not
an interpreter; runtime-executed TypeScript, if any, is a separate tier covered
under the TypeScript execution model below.

These are future directions, not commitments. They should become deliverables
only when a milestone has a concrete caller for them.

Named in the SDD frontends layout (5.11–5.12):

- `@tokimu/scenes` → scene model: declarative scene/prefab documents that lower
  into the M8 scene-to-world compile path
- `@tokimu/query` → query model: entity/component queries that lower into the
  engine query surface instead of ad hoc iteration
- `@tokimu/ui` → presentation model: screen-space HUD and world-space interface
  bindings over shared world state, not a separate UI runtime

Further candidates grounded in the v0 primitives (`signal()`, `relation()`,
`command()`) and existing crates:

- `@tokimu/materials` → shader/material model: material definitions and bound
  data that pair with `@tokimu/shaders` and lower to the same pipeline model
- `@tokimu/commands` → command model: the M12 text/MUD command surface (look,
  list, step, emit, why) authored via the `command()` primitive
- `@tokimu/signals` + `@tokimu/relations` → signal and relationship models:
  `signal()` and `relation()` declarations lowering into engine events and
  relationship edges
- `@tokimu/input` → action-map model: input action maps that lower into
  `tokimu-input` bindings rather than hand-wiring key codes
- Visual rule graphs stay a frontend over the same rule model, interchangeable
  with the TypeScript rule frontend rather than a separate runtime

### Cross-cutting considerations to keep visible

These are not separate MVPs, but they will shape whether the authoring and 3D
work lands cleanly.

- Diagnostics: every asset, shader, scene, and TypeScript lowering path should
  emit structured, actionable diagnostics rather than opaque failures.
- Frontend/model versioning: authored TS content and semantic models need a
  versioning story so older content can still lower or fail clearly.
- Asset inspection/importers: leave room for an asset browser, importer
  pipeline, and content metadata without making `tokimu-render` own asset
  translation.
- Validation workflow: examples and frontends should prove themselves through
  runnable checks, not docs alone; authored content should have a compile/check
  command.
- Hot reload / iteration loop: decide when scene, shader, or TS-authored
  content reloads in place versus requiring rebuild/restart.
- Packaging and distribution: define how authored scenes, rules, shaders, and
  assets are bundled for native and WASM without coupling core crates to npm or
  filesystem assumptions.

## Rust Engine Capabilities Required for the TypeScript API

TypeScript only ever lowers into engine-owned meaning, so before any `@tokimu/*`
package can control a capability, the Rust engine must first expose that
capability as: (a) addressable by a stable name or id, (b) describable as data,
and (c) diagnosable when misused. Today most engine surfaces are compile-time
Rust types and numeric handles — `MeshHandle(u64)`, a `Material` of just label +
color, `Component`/`Event` as blanket marker traits — which a data-driven
frontend cannot name or configure. These are the Rust-side features that unlock
the TS API. Each one must land and gain a real Rust caller before the matching
frontend package targets it (same rule-model-before-frontend ordering as M9).

### Reflection and naming (foundation for every frontend)

- [ ] Component/resource type registry: register component and resource types by
  stable name with typed field get/set, so scene data and TS can address them
  without Rust generics (today `Component`/`Resource` are marker traits with no
  name or field access)
- [ ] Name-addressable render resources: add a `name -> handle` registry over the
  numeric `MeshHandle`/`MaterialHandle`/`PipelineHandle`/`TextureHandle`/
  `CameraHandle` ids so authored content references resources by stable string id
- [ ] One shared id/naming scheme across scenes, assets, rules, and render
  resources so a frontend uses one addressing model, not five

### Shaders and materials (backs `@tokimu/shaders`, `@tokimu/materials`)

- [ ] Material parameter schema: extend `Material` beyond label + base color to a
  set of typed, named parameters (floats, vec2/3/4, colors, texture slots) so a
  material describes its bound data as data
- [ ] Texture binding path: actually wire `TextureHandle` into material bind
  groups + samplers (the handle type exists but is unused in draws today)
- [ ] Shader module abstraction: replace the fixed `PipelineKind` enum + hardcoded
  WGSL with a shader module + binding/uniform descriptor the engine can describe
  and validate as data
- [ ] Vertex layout descriptor: describe vertex inputs as data (currently the
  pipeline hardcodes a single `vec2` position layout) so an authored shader can
  declare its inputs and the engine can validate them against a mesh
- [ ] These four are the engine substrate the `@tokimu/shaders` lowering targets;
  WGSL stays the engine-owned target

### Meshes and model loading (backs the GLB proof and 3D)

- [ ] 3D vertex format: extend `Mesh` from `Vec<[f32; 2]>` to named attributes —
  3D position, normal, UV, and optional vertex color
- [ ] Indexed meshes: add index buffers (there are none today; required for the
  cube and any GLB import)
- [ ] Model importer: a GLB importer that produces engine-owned mesh/material/
  texture resources, with the renderer staying format-agnostic (the GLB proof)
- [ ] Defer skeletal data, morph targets, and animation channels until an example
  needs them

### World control (backs `@tokimu/scenes`, `@tokimu/query`)

- [ ] Named spawn/despawn: spawn entities and set components by registered name
  (depends on the reflection registry) so scene compile and TS share one path
- [ ] Name-addressable query surface: expose the query surface by component
  name/id, not only Rust generics, so `@tokimu/query` can lower into it
- [ ] Named relationship API: create and traverse the existing relationship edges
  by name so authored content can declare relations

### Rules, signals, and commands (backs `@tokimu/rules`, `@tokimu/signals`, `@tokimu/commands`)

- [ ] Engine-owned rule model: a rule with declared reads/writes/emitted signals
  that schedules like a system (the M9 prerequisite)
- [ ] Named signal/event bus: emit and subscribe to signals by name (today
  `Event` is a marker trait with no dispatch), backing `signal()`
- [ ] Command registry + dispatch: register named commands with typed arguments
  and dispatch them, backing `command()` and the M12 text surface

### Input and time (backs `@tokimu/input` and determinism)

- [ ] Named action/axis registry: register input actions and axes by name so
  `@tokimu/input` lowers into `tokimu-input` bindings instead of raw key codes
- [ ] Authorable time/RNG resources: expose fixed timestep and a seeded RNG as
  named resources authored content can configure, keeping determinism explicit

### Cross-cutting engine substrate

- [ ] Serialization/reflection so every registry above can round-trip as data
  (ties directly to the M7 persistence and M8 scene deliverables)
- [ ] Structured diagnostics on every registry lookup and lowering failure so
  authors get actionable errors, not silent fallback
- [ ] Stable ids/versioning on registries so authored content survives engine
  changes and frontends interoperate through shared meaning (SDD Q25)

## TypeScript Execution Model: Lowered Rules vs Runtime Actions

This is no longer an open binary of "either everything lowers" or "runtime TS is
an accidental escape hatch." The TTSDD settles the direction: lowered and
runtime TypeScript are both legitimate execution modes, but they carry different
guarantees and must stay under explicit author intent.

**Tier 1 — lowered (deterministic-eligible simulation).** Rules, queries, scene
data, and shaders — anything inside the fixed-timestep deterministic core —
should lower ahead of time (SDD phase 3). Lowering excludes known host-level
nondeterminism, preserves native/WASM parity, and keeps lockstep/replay on the
table. Lowered does not magically guarantee full determinism; scheduling,
ordering, numeric behavior, and seeded RNG discipline still matter.

**Tier 2 — runtime actions / runtime rules.** Higher-level game actions, quest
and orchestration logic, UI event handlers, and glue that does not need
deterministic simulation may run in a JS/TS runtime host at runtime. This is a
first-class mode, not a quiet fallback. The host boundary stays narrow and
capability-based, and the choice of a specific embedded JS engine remains a
later, separate decision.

Invariants for either tier:

- Both tiers touch the engine only through Tokimu-owned APIs (commands, queries,
  signals, events, relations); neither reads or mutates `World` state directly.
- The engine owns world truth. A runtime host must never become a hidden second
  engine core or an alternate owner of simulation state.
- Runtime-local ephemeral state may exist for convenience, but durable
  simulation state must live in Tokimu-owned resources or components rather than
  inside private script scope.
- `lowered` means lower-or-error. Only `auto` may fall back, and even then the
  resolution must stay visible and stable through an execution manifest rather
  than silently changing when the compiler improves.

Practical decision rule:

- If logic must be deterministic, replay-authoritative, or lockstep-safe →
  lower it.
- If logic is orchestration or UX glue and iteration speed matters more than
  determinism → runtime is acceptable behind the host boundary.
- If the author does not care which path wins → `auto`, but only with explicit
  manifest-backed resolution rather than compiler whim.

Roadmap implications:

- [ ] Keep the command/query/signal/event APIs (see the engine-capabilities
  section) rich enough that a future runtime host calls into them rather than
  needing raw world access
- [ ] Define the execution manifest and acceptance flow for `auto` before the
  real Compiler API path ships broadly
- [ ] Define the boundary between authoritative deterministic simulation and
  non-authoritative or host-mediated runtime actions before choosing any
  embedded JS engine
- [ ] Pick the runtime host deliberately when the time comes, rather than
  defaulting into one because it was easiest to bind first

Net: the architecture already leaves room for runtime TypeScript, but the
retooling pressure is clear too — the current prototype must still grow an
execution manifest, source-mapped diagnostics, a capability model, and a real
Compiler API recognizer before the TypeScript story is stable.

## Determinism and Validation (SDD 8, 16)

Cross-cutting deliverables that keep the existing milestones honest rather than
adding new scope.

- [ ] Add a determinism test that ticks `FixedTimeStep` a fixed number of times
  and asserts identical accumulated state across two runs
- [ ] Add a seeded RNG resource (explicit seed, not `rand::thread_rng()`) and use
  it anywhere the toy or `hello-3d-mono` needs randomness
- [ ] Add a test proving the renderer never mutates `World` state during
  `begin_frame`/`submit`/`present` (SDD: rendering must not mutate simulation
  state)
- [ ] Extend the M1 headless frame test into a target-specific smoke check:
  native opens/closes cleanly; WASM reaches a visible boot path with no
  native-only code compiled in
- [ ] Treat each new example as corpus-driven validation: it should prove one
  world relation, transformation, or rule cleanly enough to read as a reusable
  engine sentence, not just a demo

## Deferred Crates

These crates are intentionally not in the workspace yet. Listed here so
"should we add crate X" has a written trigger condition instead of being
decided ad hoc.

- `tokimu-persistence` — add only after the M7 save/load trait seam is proven
  by the round-trip test.
- `tokimu-net` — add only after the M11 transport trait seam is proven by the
  serialize round-trip test.
- `tokimu-audio`, `tokimu-tools` — no current trigger; do not add until a
  concrete example needs them.

`tokimu-rule` and `tokimu-ts-frontend` are already in the workspace, so they
are no longer part of the deferred list.

## Open Questions Tracker

Active design questions from SDD section 17, narrowed to the ones the current
work will actually force a decision on. Resolve these as part of the milestone
listed, not as a separate documentation pass.

- Renderer abstraction depth (SDD Q5) — resolve while building the M10
  perspective camera and depth attachment.
- Relationship representation (SDD Q6) — resolve if the M9 relationship viewer
  or M8 scene compile step needs more than the existing edge model.
- Asset lifecycle (SDD Q4) — resolve while building the GLB proof: is
  synchronous loading still acceptable, or does WASM force async now.
- Scene document format (SDD Q13) — resolve as part of the M7 serialization
  format choice.
- Replication unit (SDD Q20) — resolve as part of the M11 deliverable that
  names the first replication unit.
- VR/XR abstraction seam (SDD Q17) — resolve while building the M10 stereo
  deliverables, once mono 3D is solid.

### Phase Status At A Glance

Phases can overlap. The tracker shows where work is currently being invested,
not a strict promise that every earlier MVP exit criterion is complete first.

- MVP 1: done — native runnable core and renderer proof are in place
- MVP 2: partial — the first playable toy exists, but WASM parity is still
  pending
- MVP 3: done — persistence, scene, and history groundwork is in place
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
- `hello-snake` is the live proof for 2D grid movement and growth. Its unit
  tests cover movement, pellet growth, wall collision, and reverse-turn
  handling.
- `hello-pacman` is the next 2D example. It will prove maze navigation with AI
  agents and pathfinding-driven ghost movement over shared Tokimu state.
- `hello-space-invaders` is the next 2D shooter example. It will prove wave-
  based enemy movement and projectile collisions over shared Tokimu state.
- `hello-missile-command` is the next 2D defense example. It will prove turret
  aiming, interceptor firing, and falling missile interception over shared
  Tokimu state.
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
6. **AI agents / pathfinding**
   - 2D enemy movement should stay on shared simulation state, not a separate
     presentation loop
   - the first proof should be `hello-pacman`

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
