# Tokimu Software Design Document

| Field        | Value                                             |
| ------------ | ------------------------------------------------- |
| Status       | Draft                                             |
| Version      | 0.2.0                                             |
| Last updated | 2026-07-12                                        |
| Scope        | v0 architecture and initial milestones            |
| Language     | Rust (edition 2021)                               |

## 1. Purpose

Tokimu is a Rust-based real-time simulation and rendering engine intended for
games, interactive tools, technical simulations, and eventually WebAssembly
deployment, with VR/XR support planned as a first-class future capability.

Tokimu is not merely a "game engine" in the mechanical sense. It is a
state-processing runtime: it accepts input, rules, assets, and time, then
produces updated world state and rendered output.

Tokimu is also a strong fit for the label "engine kernel": a reusable runtime
that provides core services higher-level engines and interactive applications
build upon. That analogy is about scope and ownership, not about being an
operating-system kernel.

For public-facing copy, lead with the description before the label: Tokimu is
a Rust-native runtime for building interactive engines, simulations, and
applications. It provides reusable runtime services, including simulation,
scheduling, rendering, input, resources, diagnostics, and world management,
that higher-level engines build upon. Use "engine kernel" as the architectural
term once the reader already has that mental model.

Tokimu should be understood in layers:

```text
Tokimu
  └─ Simulation Engine
    └─ Interactive Simulation Engine
      └─ Game Engine
```

Games are one important expression of the engine, but not the only one. The
deeper primitive is a universe where entities exist, rules execute,
relationships change, and observers view the result.

Longer term, Tokimu should be understood less as "a game engine with a few
extra modes" and more as a semantic runtime platform for interactive
applications. Games, technical simulators, digital twins, creative tools,
robotics dashboards, circuit-analysis tools, and domain-specific editors are
all plausible applications built on the same core ideas: entities,
relationships, rules, state, tools, and presentation.

That does not mean Tokimu itself should absorb all of those applications into
the engine. The healthier ambition is to make building such applications
straightforward without turning core Tokimu into a monolithic editor suite.

If Tokimu succeeds at that, many users may never think of themselves as "using
Tokimu" directly. They may instead use higher-level engines, editors, or
application kits built on top of Tokimu for specific domains. That is a healthy
outcome. The core runtime should disappear into the background while
domain-specific applications become the visible products.

## 2. Core Goals

- Rust-native engine architecture
- Deterministic simulation core where practical
- Modular subsystems
- ECS-friendly world model
- Semantic world model that remains inspectable as it evolves over time
- Shared world model that can support 2D, 3D, and text-first presentation
- Desktop-first development
- Planned WebAssembly export support
- Planned VR/XR support as a first-class architectural concern
- Planned networking and transport boundary for future remote simulation use
- Strong early input architecture, including controller and joystick support planning
- Clean separation between engine core, platform backends, renderer, and tools
- Minimal early scope with room for later expansion
- A core architecture reusable across multiple application classes, not only games

## 3. Non-Goals for v0

- Full editor application
- Networked multiplayer
- AAA rendering stack
- Physics engine from scratch
- Built-in scripting language
- Complete asset pipeline
- Mobile support
- Console support

## 3.1 Guiding Principles

1. Finishable by a mortal. Each milestone should leave the project in a runnable,
  testable state.
2. Examples before abstractions. A window, a triangle, and a playable toy should
  shape the engine more than speculative framework design.
3. Structural boundaries over advisory rules. Crate boundaries and ownership
  rules should make bad layering difficult, not merely discouraged.
4. Core is the source of simulation truth. Rendering, platform, and tooling
  observe or adapt simulation state; they do not silently own it.
5. Prefer one concrete path before generalized infrastructure. The first native
  backend should work cleanly before Tokimu grows broader backend abstraction.
6. Stabilize public abstractions after real callers exist. Plugin APIs,
  scheduling APIs, and asset interfaces should be shaped by concrete use rather
  than theory alone.
7. Prefer explicit diagnostics over silent fallback. Engine startup, asset
  failures, backend selection, and schedule ordering issues should be visible.
8. Declarative content, imperative runtime. Scene descriptions, asset metadata,
   project files, and saved state describe what should exist; the runtime,
   renderer, and platform layers decide how it executes frame to frame.
9. Prefer a small set of world primitives over genre nouns. Avoid baking
   domain-specific concepts like player, enemy, weapon, quest, or vehicle into
   the engine core unless repeated examples prove they are truly general.
10. Make the world model the engine center. Rendering is proof and
  presentation, not the primary architectural gravity well.
11. Prefer explicit world meaning over hidden transient meaning. Relations,
  rule preconditions, and important state changes should be representable in a
  form that tools can inspect rather than existing only as fleeting control
  flow inside runtime code.
12. Stage editors and scripting behind the world model. Do not let a custom
  language, full editor, or visual graph system become the architecture before
  the underlying world, rule, and inspection model are stable.
13. Plan for multiple presentation surfaces early. Desktop, WASM, and future
  VR/XR support should share the same simulation truth even when platform and
  rendering adapters differ.
14. Treat networking as an adapter layer, not a second simulation core. Future
  transports, replication, and remote session concerns should translate engine
  meaning across process or machine boundaries rather than redefining it.
15. Normalize input devices early. Keyboard, mouse, controllers, and joysticks
  should feed a coherent engine-facing input model so later device expansion is
  additive rather than architectural rework.
16. Treat 2D and 3D as presentation choices over shared world meaning. Tokimu
  should not split into separate engine architectures for sprites versus meshes
  unless real examples later prove an unavoidable boundary.
17. Treat overlays and diegetic interfaces as presentation layers too. A 2D HUD
  and a 3D in-world interface should consume shared engine meaning rather than
  becoming separate UI runtimes with their own hidden game state.
18. Treat text-first and MUD-like presentation as another adapter surface over
  the same world model, not as a separate engine or special-case runtime.
19. Prefer platform value over built-in specialization. Tokimu should make
  specialized applications easier to build without forcing every such
  application to become part of the engine itself.
20. Prefer canonical primitives over domain packages in core. Tokimu should
  standardize foundational concepts such as transform, hierarchy, asset,
  command, signal, relation, and time, while leaving genre- or domain-specific
  vocabularies such as health, quests, dialogue, or traffic to higher-level
  engines built on top.

### 3.1.1 AI Implementation Principles

This SDD should also be treated as a constraint system for AI-assisted
implementation.

As code generation gets cheaper, the main risk shifts from "can the code be
written" to "did the implementation preserve the intended architecture." The
document exists partly to keep many separate coding sessions, human or agent,
converging on the same engine rather than on many locally reasonable but
incompatible designs.

The document intentionally emphasizes responsibilities and boundaries more than
function signatures. For Tokimu, a sentence such as "presentation consumes
state but does not own simulation truth" is more valuable than prematurely
standardizing ten APIs around the wrong ownership model.

Before introducing a new abstraction, crate, trait, service, or subsystem,
implementations should answer:

1. Can an existing abstraction be extended instead?
2. Is this required by a concrete example, milestone, or acceptance criterion?
3. Does this preserve world-first architecture and simulation-owned truth?
4. Does this accidentally push a presentation concern into simulation?
5. Is this concept semantic, or merely implementation-specific?
6. Does this duplicate an existing concept under a new name?
7. Does this make native, WASM, or future VR/XR support harder by baking in
  assumptions too early?

For milestone work, smaller concrete success criteria should beat speculative
completeness. If the current milestone needs one window, one triangle, or one
playable toy, the correct implementation is usually the one that proves exactly
that behavior while adding the fewest irreversible abstractions.

## 3.2 Naming and Conceptual Influence

Tokimu is a peer project to Tosumu, and its name is derived from the same
conlang tradition that shaped Tosumu's identity and parts of its design
reasoning.

That broader language project is Tonesu: a constructed language built around
compositional meaning, explicit structural relations, and epistemic honesty.
Its own name contracts from `to-ne-su` — pattern, relation, structure — which
is also a useful summary of the intellectual style behind these sibling
projects.

That conlang carries strong opinions about reality, truth-claims, boundaries,
and the difference between what exists, what is observed, and what can be
honestly asserted. Those ideas were especially important in the Tosumu database
project, where they influenced decisions around integrity, explicit state
claims, and structural boundaries.

For Tokimu, the influence is lighter but still relevant. It reinforces several
engine design instincts already captured in this document:

* simulation state should have a clear source of truth
* subsystem boundaries should be structural, not merely advisory
* diagnostics should make state and failure visible rather than implicit
* tooling and persistence layers should not silently overclaim ownership of
  runtime reality

This is a conceptual influence, not a requirement that Tokimu adopt conlang
terminology in its APIs. Code, crate names, and public engine concepts should
remain plain English unless there is a strong reason to do otherwise.

## 4. Architecture Overview

```text
Corpus / Examples
    ↓
 World Graph / State
    ↓
 Systems / Signals / Rules
    ↓
  State Change
  ↙       ↘
Presentation  Synchronization
  ↓             ↓
Renderer / Platform / Audio / WASM Host / Network Transport
```

Tokimu should avoid hard-coding platform assumptions into the simulation layer.
The same core should eventually run on native desktop and WebAssembly.
The same core should also remain compatible with a future VR/XR presentation
path rather than assuming a permanently flat-screen engine model.
The renderer should be treated as a consumer of world state, not the center of
the engine architecture.
Future networking should be treated as an adapter around synchronized engine
meaning, carrying replicated state, events, or remote commands without becoming
the owner of the simulation model.
Likewise, 2D and 3D should be treated as different presentation mappings over
the same simulation truth rather than as separate cores.
The same rule should apply to interface surfaces: screen-space HUDs and world-
space interfaces should remain presentation consumers over shared simulation
state rather than hidden parallel state machines.
The same rule should apply to text surfaces: room descriptions, command
prompts, transcripts, and remote text sessions should stay over the same world,
rule, signal, and input model.

## 4.1 World Corpus

Tokimu should treat examples and scenarios as a world corpus.

Tonesu grows meaning through vocabulary, composition, and sentence corpus.
Tokimu can grow behavior through components, composition, and simulation corpus.

This suggests a development shape like:

```text
components
  ↓
composition
  ↓
simulation corpus
  ↓
behavior emerges
```

Examples are not side material. Each example is a world sentence that proves a
specific relationship, rule, or transformation. A game becomes a paragraph. A
world becomes a document.

Illustrative corpus progression:

```text
examples/
  S0001_transform_motion/
  S0002_parent_child_entity/
  S0003_collision_event/
  S0004_sprite_animation/
  S0005_scene_transition/
  S0006_ai_behavior/
```

Tokimu should prefer growing from such examples over inventing broad engine
taxonomies too early.

## 4.2 Core Vocabulary

Tokimu's lowest-level vocabulary should stay small, reusable, and semantically
clear.

Suggested fundamental nouns:

```text
World      — container of existence
Entity     — thing with identity
Trait      — property attached to entity
Relation   — connection between entities
State      — current truth
Rule       — transformation logic
Signal     — notification of change
Time       — ordered progression
Resource   — external knowledge/data
View       — observation of state
```

This vocabulary supports games, simulations, workflow models, AI worlds, and
digital-twin style systems without forcing Tokimu to commit to one genre's noun
set too early.

In code, `Component` is the implementation term and `Trait` is the semantic
design term. If Tokimu uses both words, `Component` should name the storage or
runtime unit, while `Trait` should name the inspectable capability or meaning in
the document model.

## 5. Major Subsystems

### 5.1 tokimu-core

Owns the engine-neutral model:

* world state
* entities/components
* relationship edges
* resources
* events
* provenance and timeline primitives later
* schedules
* time-step policy
* math primitives
* spatial meaning that is not prematurely locked to only 2D or only 3D
* opaque asset references used by simulation state
* diagnostics

This crate should not depend on windowing, GPU, filesystem, or native OS APIs.

### 5.2 tokimu-runtime

Owns application execution:

* engine loop
* fixed/update/render phases
* system scheduling
* lifecycle hooks
* plugin registration
* runtime configuration

Note: the loop module is named `run_loop.rs`, not `loop.rs`, because `loop` is a
reserved Rust keyword and would otherwise require `mod r#loop;`.

### 5.3 tokimu-render

Owns rendering abstraction:

* cameras
* render commands
* sprites/meshes/materials
* pipeline descriptors and handles
* reusable renderable descriptors and handles
* per-draw instance payloads
* camera/view uniforms
* texture handles
* draw commands
* backend-neutral renderer API

Tokimu should support both 2D and 3D rendering as first-class presentation
targets over time, but it should do so through shared renderer abstractions
rather than by growing separate engine cores. Early examples may be 2D-biased
for speed, while the render architecture should remain able to grow toward 3D
meshes, cameras, depth, and scene composition without a rewrite.

That same presentation model should leave room for both 2D HUD layers and 3D
in-world interface surfaces. Tokimu should not assume every interface is either
pure screen-space UI or pure diegetic world geometry forever.

Tokimu should use `wgpu` as the first renderer backend rather than starting
from raw Vulkan. `wgpu` already spans native and WASM targets while hiding much
of the platform-specific GPU setup burden.

Recommended layering:

```text
tokimu-render abstraction
    ↓
   wgpu backend
    ↓
Native: Vulkan / Metal / D3D12 / OpenGL
WASM:   WebGPU / WebGL2
```

Tokimu's public renderer API should not simply expose `wgpu` everywhere. The
backend should stay behind Tokimu-owned concepts such as renderer traits,
render commands, and resource handles.

Pipeline choice should remain explicit at draw submission time rather than
being hidden inside material state. Materials describe bound data, while draw
commands decide which mesh, material, and pipeline resources participate in a
specific presentation step.

Reusable renderable resources may bundle mesh, material, and pipeline handles
for convenience, but they should remain presentation-facing tuples rather than
quietly becoming scene ownership, transform storage, or simulation truth.

Per-draw instance payloads may supply small placement-oriented data such as 2D
translation, scale, and rotation so the same renderable can be submitted more
than once.
The current proof also uses multiple materials and renderable handles for the
same mesh so distinct presentation variants stay explicit, and it can vary the
clear color over time to keep the background part of the presentation proof.
The current proof has since grown to six differently colored 2D shapes,
including a square mesh and a diamond mesh, which keeps the example squarely
in 2D presentation while making the reuse and motion more scene-like without
introducing a scene graph or transform hierarchy.
Keyboard input can also nudge part of the composition, with both WASD and the
arrow keys mapped into the same Tokimu-owned input state, which keeps the
platform event path visibly connected to presentation without turning the
example into a gameplay loop.
Mouse movement can nudge the square and diamond placements too, so the proof
now visibly reacts to both cursor and keyboard state while remaining a 2D
presentation example rather than a gameplay loop.
The keyboard and mouse controls now steer different portions of the scene,
which keeps the interaction legible without needing a gameplay-specific input
model.
The example also mirrors the current input state into the native window title,
which gives the platform seam a visible readout without needing text rendering
or a HUD layer yet, and Space now resets the visible offsets back to neutral.
Left click toggles paused motion so the title can distinguish moving versus
paused mode without inventing a gameplay loop.
Right click toggles an alternate palette mode so the same scene can visibly
shift without introducing a separate rendering abstraction.
Middle click reverses motion direction so the proof can show another
Tokimu-owned input path without adding a scene graph or a gameplay system.
Holding a mouse button briefly boosts the background tint and motion amplitude
and makes cursor-driven placement more pronounced, so the scene also reacts to
pressed-state input, not only click toggles. In practice, the held state now
behaves more like a visible drag gesture than a quiet modifier.
That still does not make `tokimu-render` the owner of transform hierarchies,
visibility graphs, or world truth.

Camera/view uniforms are the next distinct lane: they define how a frame is
seen, not what the world is. That keeps placement, view, and world meaning
separable even in the first 2D render proofs.

Tokimu's first concrete camera helper may be orthographic, since the early
render proofs are 2D-oriented. That still serves the same boundary: projection
is explicit, aspect-aware, and not implicit in the backend.

Camera resources may be uploaded and selected by handle just like meshes,
materials, pipelines, and renderables. Tokimu's first camera upload can carry
separate matrix-based view and projection payloads, can be sized from the
native window while preserving an explicit logical world height owned by the
example, and the active camera is still a presentation choice, not a
simulation owner.

That renderer API should also avoid baking in assumptions that only sprites or
only meshes matter. Cameras, transforms, render commands, visibility, and
resource handles should leave room for both 2D and 3D content, even if the
first examples emphasize one side.

It should likewise leave a clean path for layering screen-space interface
presentation and world-space interface presentation without forcing UI logic to
fork into unrelated architectures.

VR/XR should be treated as an additional presentation mode, not a separate
engine architecture. Early renderer abstractions should leave room for stereo
views, per-eye camera data, tracked spaces, and different frame submission
models without forcing those concerns into `tokimu-core`.

Illustrative direction:

```rust
trait Renderer {
  fn begin_frame(&mut self);
  fn submit(&mut self, commands: &[RenderCommand]);
  fn end_frame(&mut self);
}
```

Early file shape should stay small and concrete:

```text
renderer.rs       // Tokimu abstraction
commands.rs       // draw and clear commands
resources.rs      // texture/mesh/material/pipeline handles
pipeline.rs       // minimal Tokimu-owned pipeline descriptors
wgpu_backend.rs   // backend implementation
```

### 5.4 tokimu-platform

Owns platform integration:

* window creation
* input devices
* timing
* filesystem abstraction
* native/WASM platform differences

Likely native stack:

* `winit`
* `wgpu`
* `gilrs` or similar early for gamepads, controllers, and joystick-class devices
* OpenXR later for VR/XR session and device integration

Likely WASM stack:

* `wasm-bindgen`
* `web-sys`
* browser canvas target
* `wgpu` WebGPU path where available

VR/XR support should likely arrive as a platform adapter concern coordinated
with render abstractions rather than as special logic inside the simulation
core. OpenXR is the most plausible first-class native integration target when
Tokimu reaches that stage.

Text/MUD support should arrive in the same spirit: a text adapter, command
parser, and session surface coordinated with the same world and input model
instead of a separate game loop or bespoke world representation.

That makes text a simulation console for inspectable systems such as process
models, digital twins, maintenance workflows, training scenarios, and other
worlds where relations, rules, and traces matter more than visual polish.
The surface should feel like textual inspection and command control, not a
fantasy MUD by default.

Early commands can stay intentionally small:

* `look <entity>`
* `list relations <entity>`
* `step <seconds>`
* `emit <signal>`
* `why <entity/state>`

### 5.5 tokimu-assets

Owns asset loading and management:

* asset IDs
* handles
* loaders
* provider abstraction over file, network, embedded, and generated sources
* asset storage and lifetime
* hot reload later
* embedded asset bundles later
* WASM-friendly asset fetch abstraction

See [on-assets.md](Conversations/on-assets.md) for the importer-oriented
boundary between asset translation and rendering.

### 5.6 tokimu-input

Owns normalized input state:

* keyboard
* mouse
* controller and gamepad support planned early
* joystick and analog axis normalization planned early
* touch later
* action mapping

The input layer should be organized around engine-facing actions, axes, button
states, and device capabilities rather than around one-off keyboard events.
That allows Tokimu to start small while still leaving a clean place for analog
sticks, triggers, hats, deadzone handling, rebinding, and per-device quirks.

The same design habit applies to spatial interaction. Input meaning such as
move, aim, select, pan, orbit, or confirm should not assume a permanently 2D
or permanently 3D application shape.

Early input architecture should distinguish:

* raw device events from normalized engine input state
* digital actions from analog axes
* per-device capabilities from game/application intent mappings
* input sampling from simulation decisions

### 5.7 tokimu-net

Deferred early, but eventually owns remote simulation plumbing:

* connection/session management
* transport abstraction
* message framing and channel semantics
* replication protocols
* authority and ownership strategy
* snapshot/delta/event delivery later
* loopback and local transport for tests later

Tokimu should not begin with full multiplayer features, but it should avoid
hard-coding assumptions that every simulation is permanently single-process.
If networking arrives, the engine should replicate meaningful world changes,
commands, and observations rather than exposing raw socket concerns through the
simulation core.

Early direction:

* keep transport details out of `tokimu-core`
* keep wire formats and session concerns out of renderer/platform APIs
* prefer a Tokimu-owned transport or channel abstraction over leaking a
  specific socket library everywhere
* support both reliable and unreliable delivery semantics when the design earns
  them, rather than assuming one universal channel
* keep browser-facing transport constraints in mind from day one; native-only
  UDP assumptions often become a trap if WASM matters later

### 5.8 tokimu-audio

Deferred early, but eventually owns:

* sounds
* music
* spatial audio
* mixer state

### 5.9 tokimu-tools

Optional future tooling:

* scene inspection
* debug overlays
* asset inspection
* editor support

Tokimu should eventually be able to inspect both conventional 2D HUD state and
3D in-world interface state as presentation layers over shared world meaning.
The editor/tooling story should not assume that every interface is a flat debug
overlay.

Tokimu should also be able to expose text-first and MUD-like views over the same
world meaning: room descriptions, command history, prompt state, and
inventory/status summaries should be inspectable without becoming a separate
authoring system.

The first editor layer should be inspection-oriented rather than a full content
authoring environment. A Rust-native tool stack such as `egui` is a plausible
early fit for entity trees, inspectors, signal logs, relationship views, and
system timing panels without committing Tokimu to a heavyweight editor shell too
early.

### 5.10 tokimu-persistence

Optional future persistence layer:

* scene serialization and save/load flows
* project metadata and tool state
* asset indexing or content metadata later
* optional editor-facing storage later

This layer must stay outside the simulation core. `tokimu-core` should model
world state in memory; persistence code should translate to and from that model
without making the ECS, runtime, or renderer depend on a database.

The local Database project may be useful here, but only as an optional
dependency or as a source of design patterns. If reused, it should live behind
its own crate boundary rather than leaking storage concerns into engine core
crates.

Scene and prefab data should start as plain declarative documents rather than a
custom scripting language. Rust-friendly formats such as RON are a good early
fit, with TOML, YAML, JSON5, or similar alternatives acceptable if they better
serve the actual tool chain.

Scene data should also be able to describe both 2D-oriented and 3D-oriented
content without forking into unrelated document models. Tokimu can start with a
small shared scene vocabulary and let examples prove when separate 2D or 3D
authoring conveniences are worth adding.

That scene vocabulary should also be able to describe interface-bearing content,
whether that becomes a screen-space HUD binding, a world-space panel, a labeled
interaction point, or some other inspectable presentation attachment.

Likewise, early text examples may be simple room-and-command flows because they
are cheaper to ship, but the data model should not imply that MUD-like or
command-driven presentation is architecturally separate special-casing.

Illustrative direction:

```text
entity "player" {
  traits: [
    Transform { x: 0, y: 0 },
    Sprite { texture: "player.png" },
    InputController {},
    Collider { shape: "capsule" }
  ]
}
```

This keeps scene data inspectable and editable without inventing a language
before the engine has earned one.

Text-first scenes should be able to represent command prompts, room
descriptions, inventory/status summaries, and transcript-friendly labels using
the same shared scene vocabulary.

Early scene examples may be sprite-heavy because they are cheaper to ship, but
the data model should not imply that mesh-based or depth-aware worlds are
second-class concepts.

Likewise, early HUD examples may be flat overlays because they are cheaper to
ship, but the data model should not imply that in-world diegetic interfaces are
architecturally separate special cases.

### 5.11 Rule Frontends

Tokimu should own a semantic rule model in the middle of the stack. Scene data,
visual graphs, and future scripting should compile or translate into that
engine-owned model rather than each frontend inventing its own runtime.

The important architectural point is not "Tokimu uses TypeScript." It is that
Tokimu owns the semantics while authoring frontends remain adapters over that
engine-owned meaning.

That semantic rule model should be intentionally language-agnostic. TypeScript
is the strongest early frontend candidate because of its mature tooling, not
because Tokimu semantics should become tied to JavaScript. Tokimu itself should
remain primarily a Rust engine, while developers building games, tools,
scenarios, and content on top of Tokimu should be able to live mostly in
TypeScript.

For v0, a rule is a named system-like transformation with declared inputs,
outputs, and emitted signals. That is enough to anchor the architecture without
building a full IR before the corpus proves the need.

Illustrative shape:

```text
Scene Data         ┐
Visual Graphs      ├→ Tokimu Semantic Rule Model → Runtime Systems
TypeScript APIs    ┘
```

This keeps the world model and rule execution architecture primary, while
treating editors and scripting as frontends.

The same pattern should generalize beyond rules when the corpus earns it:

```text
TypeScript syntax
  ↓
domain-specific Tokimu API
  ↓
domain-specific semantic model
  ↓
target compiler/runtime
```

Independent frontends should target interoperable Tokimu semantic models rather
than communicating through TypeScript itself. Scenes, rules, queries,
presentation bindings, and later specialized frontends should converge through
Tokimu-owned meaning, not through ad hoc package conventions.

That same rule should apply to higher-level domain engines and application kits.
An RPG toolkit, CAD-oriented package, painting application, or circuit-analysis
suite may each define richer domain semantics on top of Tokimu, but they should
do so by extending or composing Tokimu-owned primitives rather than by forcing
changes into the core runtime for every new application class.

Recommended early implementation path:

```text
Phase 1  Declarative scene/rule data, no scripting
Phase 2  Semantic rule model stabilizes with real callers
Phase 3  TypeScript authoring lowers ahead of time into that model
Phase 4  Optional embedded JS host, only if real use cases justify it
```

The preferred path is phase 3, not phase 4. Tokimu should favor ahead-of-time
compilation of a restricted TypeScript frontend into the semantic rule model
rather than embedding a general-purpose JavaScript runtime in core engine
paths. That keeps determinism, native/WASM parity, and engine boundaries easier
to preserve.

Practical lowering pipeline:

```text
TypeScript source
  ↓
TypeScript compiler + typechecker
  ↓
typed AST / recognized Tokimu API calls
  ↓
Tokimu lowering pass
  ↓
Tokimu semantic rule model
  ↓
Runtime systems
```

Tokimu should not try to understand arbitrary JavaScript. The frontend should
recognize specific Tokimu-owned API calls and lower those. In practice, that
means "Tokimu supports the `tokimu` package" is a more accurate statement than
"Tokimu supports TypeScript."

Illustrative direction:

```ts
import { rule, query } from "tokimu";

rule("movement", (ctx) => {
  for (const e of query(Transform, Velocity)) {
    e.get(Transform).x += e.get(Velocity).x * ctx.fixedDelta;
  }
});
```

Conceptual lowering target:

```text
rule "movement" {
  reads: [Transform, Velocity]
  writes: [Transform]
  time: fixed_step
  operation: integrate Transform.x with Velocity.x
}
```

The first frontend should deliberately support only a small, explicit subset of
constructs such as `rule()`, `query()`, `signal()`, `relation()`, `command()`,
deterministic loops, and arithmetic. It should explicitly reject ambient I/O,
DOM access, `async`, `Promise`, `Date`, `Math.random`, `fetch`, `eval`, and
similar host-dependent features.

Likely prototype toolchain:

* TypeScript Compiler API for authoritative parsing, symbol resolution, diagnostics, and type information.
* `ts-morph` as a friendlier wrapper for the first lowering prototypes.
* `Oxc` as a later option only if more of the frontend pipeline moves fully into Rust.

The toolchain is important because Tokimu does not need to build a TypeScript
parser or type checker. It only needs to own the semantic validation and the
lowering step into engine-owned meaning.

### 5.12 Dependency Rules

The crate graph should stay intentionally narrow:

* `tokimu-core` depends only on foundational libraries such as math, error, and diagnostics crates.
* `tokimu-runtime` depends on `tokimu-core` and orchestrates schedules, plugins, and the app lifecycle.
* `tokimu-runtime` may also depend on `tokimu-input` once the runtime needs to own a current engine-facing input snapshot; `tokimu-platform` should still remain the adapter from OS or browser events into that state.
* `tokimu-input`, `tokimu-assets`, and `tokimu-render` may depend on `tokimu-core`, but not on each other unless a concrete use case justifies it.
* `tokimu-platform` adapts OS or browser events into engine-facing abstractions; it should not absorb simulation logic.
* `tokimu-net`, if added, should depend on engine-facing world, command, and replication shapes rather than making core types depend on socket or protocol libraries.
* `tokimu-persistence`, if added, depends on stable engine-facing data formats or translation APIs; engine crates should not depend on it.
* Editor tooling, visual rule graphs, and future scripting frontends should depend on Tokimu-owned world and rule abstractions, not become parallel runtimes.
* If Tokimu grows TypeScript frontends, the shared TypeScript compiler integration, diagnostics, and lowering infrastructure should live in frontend-facing crates rather than leaking into core engine crates.
* Independent authoring frontends should share infrastructure where useful, but each frontend should own its own API surface and semantic lowering rules rather than expanding into one monolithic "Tokimu understands TypeScript" compiler.
* The facade crate `tokimu` may re-export internal crates, but internal crates should avoid depending on the facade.

## 6. Engine Loop

Initial loop:

```text
start
  ↓
initialize platform
  ↓
load assets
  ↓
while running:
    collect input
    update fixed simulation as needed
    update variable systems
    render frame
    present
  ↓
shutdown
```

Recommended phases:

```text
Startup
PreUpdate
FixedUpdate
Update
PostUpdate
RenderPrepare
Render
PostRender
Shutdown
```

Pipeline view:

```text
Input
  ↓
Intent
  ↓
Simulation
  ↓
Rules
  ↓
World Mutation
  ↓
Presentation
  ↓
Render
```

Phase invariants should be documented and enforced where practical.

Early invariants:

* Render-phase code must not mutate the simulation world.
* Input collection should normalize external device state, not silently own game logic.
* World mutation should happen in simulation-owned phases, not in presentation code.
* Physics or movement resolution should have a clear owning phase once introduced.
* HUD and interface presentation should observe or request state changes through
  explicit engine-facing inputs or signals rather than mutating world state as a
  hidden side effect.

## 7. ECS Model

Tokimu should support an ECS-style architecture.

Initial entities:

```rust
EntityId
Component
World
Query
System
Resource
```

Early implementation can be simple. Do not overbuild the ECS before the engine
has real examples. The ancient curse of engine developers is spending three
months designing archetype storage before a triangle appears. Avoid the triangle
goblin.

### 7.1 Primitive-First World Model

Tokimu should resist growing core engine nouns like `Player`, `Enemy`,
`Weapon`, `Vehicle`, `NPC`, or `Quest` as first-class engine concepts.

Prefer a smaller set of reusable primitives such as:

```text
World
Entity
Trait
Relation
State
Change
Signal
Rule
Time
```

Genre-specific meaning should emerge from composition.

Spatial presentation should also emerge from composition. An entity should not
be forced into a permanent "2D entity" or "3D entity" identity in the core if
the real difference is how the presentation layer interprets its state.

Examples:

* player = entity + input relation + camera relation + physics state
* enemy = entity + AI rule + physics state
* projectile = entity + motion rule + collision relation
* chess piece = entity + position trait + color trait + threat relations
* factory machine = entity + feeds_into relations + consume/produce rules

This keeps Tokimu from accidentally hardening into one game genre too early.

### 7.2 Relationship Layer

Tokimu's world model should allow first-class relationships alongside ordinary
components.

Most ECS designs focus on `entity has components`. Tokimu can use that and also
model `thing relates to thing` explicitly where it improves clarity.

Examples:

* camera follows player
* player owns sword
* door requires key
* projectile created_by weapon
* NPC belongs_to faction

Illustrative API direction:

```rust
world.link(camera, Relation::Follows, player);
```

Possible uses:

* query everything following a target
* query ownership or dependency graphs
* detect deletion constraints and cascading cleanup

This does not require a fully general graph engine in v0, but it is a useful
directional constraint on how the world model should grow.

Relationship meaning should remain stable even if relationship storage changes.
A relation can begin as a simple map, table, or dedicated component and later
grow into a richer graph representation without changing what the relation
means to tools or rules.

The same relationship model can carry game-like and simulation-like structure
without introducing genre nouns into the core:

* entity owns item
* entity targets entity
* entity belongs_to faction
* entity triggered_by event

### 7.3 No Hidden Meaning

Tokimu should avoid hiding important world meaning only inside momentary code
paths.

Instead of logic that only briefly implies a relation:

```text
if player.distance(enemy) < 5.0 {
  enemy.attack();
}
```

Tokimu should prefer a model that can expose inspectable meaning such as:

```text
Relation:
Enemy targets Player

Rule:
Target within range enables Attack
```

That makes world state more intelligible to tools, editors, diagnostics, and
future AI-assisted inspection.

### 7.4 Knowledge Layer

Tokimu may eventually benefit from a lightweight knowledge layer above raw
state.

Not just:

```text
entity 123 has component 55
```

But also:

```text
sword is held_by player
door is locked_by key
fire damages wood
```

This should be understood as meaningful world structure, not as a mandate to
turn the engine core into a general-purpose database. The useful insight is that
world semantics should stay inspectable and composable rather than collapsing
into anonymous IDs too early.

For AI or tooling integration, the useful target is not opaque rows like:

```text
id=38483, x=5, y=10, flags=128
```

But a more semantically meaningful world surface such as:

```text
Guard Bob
- located in hallway
- protecting vault
- suspicious of player
- following patrol route
```

### 7.5 Signals as First-Class Flow

Signals and events should be treated as first-class coordination surfaces.

Example flow:

```text
Physics emits CollisionStarted
  ↓
Audio plays an impact
  ↓
Quest logic checks objective state
  ↓
Particles spawn an effect
```

The goal is a society of ignorant systems: subsystems react to world signals
without acquiring hard dependencies on one another.

### 7.6 Provenance and Timeline Direction

Tokimu should eventually be able to explain where state came from.

The important question is not only what the world is now, but also:

* what changed
* which system changed it
* why it changed
* when it changed

Illustrative debug view:

```text
Frame 8271

Entity 552 moved

Position:
  (10,20)
      ↓
  (11,20)

Reason:
  MovementSystem

Cause:
  PlayerInput.Forward
```

This does not mean every runtime build must carry full audit history. It does
mean Tokimu should leave room for optional provenance, state diffing, replay,
rollback, and timeline inspection without fighting the core architecture.

## 8. Determinism

Tokimu should prefer deterministic simulation where practical.

Rules:

* fixed timestep available for simulation
* random number generation should use explicit seeded RNG resources
* floating-point determinism is best-effort, not guaranteed across all platforms
* rendering must not mutate simulation state

If Tokimu grows networked simulation later, it should not assume perfect
cross-platform floating-point identity as a hidden requirement. Transport,
replication, rollback, prediction, reconciliation, or authoritative correction
strategies should be chosen explicitly rather than implied by wishful
determinism.

## 9. WASM Strategy

WASM support is a planned architectural requirement, not a v0 blocker.

Design constraints from day one:

* avoid direct `std::fs` in core crates
* avoid blocking APIs in runtime-critical paths
* isolate platform code
* use feature flags for native-only behavior
* keep asset loading abstract
* avoid threads in early WASM path unless explicitly designed
* avoid choosing rendering APIs that force a native-only architecture
* avoid presentation assumptions that make future VR/XR support awkward to add
* avoid transport assumptions that make future browser-capable networking
  impossible without a rewrite

Rendering strategy:

* Tokimu should target `wgpu` first for both native and browser-facing rendering.
* Vulkan should be treated as one possible native backend under `wgpu`, not as
  Tokimu's direct public rendering API.
* WASM rendering should prefer WebGPU where available, with WebGL2 compatibility
  handled by the backend rather than by leaking browser-specific rendering
  assumptions into the engine core.
* Early render abstractions should avoid assuming exactly one camera, one output
  surface, or one presentation path per frame, because those assumptions tend
  to fight VR/XR later.

Target commands eventually:

```bash
cargo build --target wasm32-unknown-unknown
wasm-pack build crates/tokimu-wasm
```

## 10. Diagnostics

Tokimu should expose structured diagnostics:

* startup logs
* asset load failures
* renderer backend info
* active presentation mode info such as desktop, WASM, or VR/XR session state later
* frame timing
* system timing later
* WASM console bridge

Native logging:

```rust
tracing
tracing-subscriber
```

WASM logging:

```rust
console_error_panic_hook
tracing-wasm
```

## 11. Project Skeleton

```text
tokimu/
├── Cargo.toml
├── README.md
├── docs/
│   ├── Tokimu Software Design Document.md
│   ├── ADR/
│   │   ├── ADR-0001-engine-boundaries.md
│   │   └── ADR-0002-conceptual-influence.md
│   ├── wasm.md
│   ├── other-ideas.md
│   └── roadmap.md
│
├── crates/
│   ├── tokimu-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── entity.rs
│   │       ├── component.rs
│   │       ├── world.rs
│   │       ├── resource.rs
│   │       ├── event.rs
│   │       ├── schedule.rs
│   │       ├── time.rs
│   │       ├── math.rs
│   │       └── diagnostics.rs
│   │
│   ├── tokimu-runtime/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── app.rs
│   │       ├── plugin.rs
│   │       ├── run_loop.rs
│   │       └── config.rs
│   │
│   ├── tokimu-render/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── renderer.rs
│   │       ├── commands.rs
│   │       ├── resources.rs
│   │       ├── camera.rs
│   │       ├── color.rs
│   │       ├── texture.rs
│   │       ├── mesh.rs
│   │       ├── material.rs
│   │       └── wgpu_backend.rs
│   │
│   ├── tokimu-platform/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── input.rs
│   │       ├── window.rs
│   │       ├── clock.rs
│   │       ├── native.rs
│   │       └── wasm.rs
│   │
│   ├── tokimu-assets/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── asset.rs
│   │       ├── handle.rs
│   │       ├── loader.rs
│   │       └── store.rs
│   │
│   ├── tokimu-input/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── keyboard.rs
│   │       ├── mouse.rs
│   │       ├── action_map.rs
│   │       └── state.rs
│   │
│   ├── tokimu-wasm/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   │
│   └── tokimu/
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
│
├── examples/
│   ├── hello-window/
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   ├── hello-triangle/
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   └── wasm-demo/
│       ├── index.html
│       └── package.json
│
└── crates/tokimu/tests/
  └── smoke.rs
```

`tokimu-audio`, `tokimu-tools`, and `tokimu-net` are intentionally omitted from
the initial workspace skeleton. `tokimu-persistence` is also intentionally
omitted at this stage. These crates should be added only once a concrete example
or tool flow proves the need.

The authoring-frontend crates implied by sections 5.11 and 5.12 are likewise
deferred, not yet present. When the semantic rule model earns real callers, the
first additions would be an engine-owned `tokimu-rule` crate, followed only
later by TypeScript frontend crates. None of these should exist before the rule
model is exercised by concrete examples. See
[scripting-typescript.md](scripting-typescript.md) for the full frontend design.

Proposed future layout, added only when earned (illustrative, not current):

```text
crates/
  tokimu-rule/          # engine-owned semantic rule model (Rust)
  tokimu-ts-frontend/   # shared TS compiler integration + lowering host (Rust)

frontends/              # TypeScript authoring packages (npm workspace)
  tokimu-rules/         # @tokimu/rules  -> tokimu-rule model
  tokimu-scenes/        # @tokimu/scenes -> tokimu-scene model
  tokimu-query/         # @tokimu/query  -> tokimu-query model
  tokimu-ui/            # @tokimu/ui     -> tokimu-presentation model
```

The split keeps Rust engine crates under `crates/` and TypeScript authoring
packages under a separate top-level `frontends/` tree, so the two toolchains do
not entangle build systems. Independent frontends target interoperable Tokimu
semantic models rather than importing one another.

## 12. Workspace Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    "crates/tokimu",
    "crates/tokimu-core",
    "crates/tokimu-runtime",
    "crates/tokimu-render",
    "crates/tokimu-platform",
    "crates/tokimu-assets",
    "crates/tokimu-input",
    "crates/tokimu-wasm",
    "examples/hello-window",
    "examples/hello-triangle"
]

[workspace.package]
edition = "2021"
license = "MIT"
repository = "https://github.com/Arakendo/Tokimu"
version = "0.1.0"

[workspace.dependencies]
anyhow = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
tracing-wasm = "0.2"
console_error_panic_hook = "0.1"
glam = "0.29"
wgpu = "23"
winit = "0.30"
pollster = "0.4"
wasm-bindgen = "0.2"
web-sys = "0.3"
js-sys = "0.3"
```

`examples/wasm-demo` is intentionally not listed as a Cargo workspace member,
because it is a browser host surface for the WASM build rather than a Rust
crate in its own right.

## 13. Public Facade Crate

`crates/tokimu` should re-export the stable public API:

```rust
pub use tokimu_core::*;
pub use tokimu_runtime::*;
pub use tokimu_assets::*;
pub use tokimu_input::*;

#[cfg(feature = "render")]
pub use tokimu_render::*;

#[cfg(feature = "platform")]
pub use tokimu_platform::*;
```

This keeps users from depending directly on every internal crate.

## 14. Initial Milestones

### M0 — Skeleton

* workspace builds
* crates created
* docs added
* native example compiles

Acceptance criteria:

* `cargo test --workspace` passes on stable Rust.
* `hello-window` compiles as a runnable example binary.
* The facade smoke test proves the public crate re-exports core runtime types.
* The workspace contains the documented crate, docs, and example skeleton.

### M1 — Runtime Loop

* app builder
* plugin trait
* fixed timestep clock
* basic diagnostics
* create world, relate entities, run systems, mutate state

Acceptance criteria:

* An `App` type exists as the runtime entry point.
* A plugin can register itself through a stable early API.
* Fixed-step update accounting is test-covered.
* Diagnostics can record startup/runtime messages without platform coupling.
* A minimal world can be created, mutated by a system, and inspected after the mutation.

### M2 — Minimal ECS

* entity IDs
* components
* resources
* simple systems
* relationship edges direction chosen
* tests

Acceptance criteria:

* Entity creation is deterministic and test-covered.
* Core ECS-facing types live in `tokimu-core`, not the runtime or renderer.
* The minimal world model supports at least one real example-driven call site.
* The engine direction for entity-to-entity relationships is documented, even if
  the first implementation stays intentionally small.

### M3 — Window + Input

* native window
* keyboard/mouse input
* normalized input resource
* controller/joystick-oriented input model chosen

Acceptance criteria:

* Platform input is translated into engine-facing input state.
* Input collection is separated from simulation decisions.
* At least one example proves the input-to-intent path rather than only device capture.
* The input model leaves a clear place for controllers, joysticks, analog axes,
  deadzones, and rebinding even if the first shipped example uses only keyboard
  and mouse.
* `hello-window` is the current live proof for the M3 window/input spike, and
  its intentionally blank surface is labeled as deliberate in the live title.

### M4 — Renderer Spike

* `wgpu` initialization
* clear screen
* triangle
* explicit mesh/material/pipeline resource upload
* reusable renderable-handle submission
* minimal per-draw placement payload
* first camera/view uniform
* camera abstraction

Acceptance direction:

* The earliest concrete render examples may be 2D-oriented, 3D-oriented, or one
  of each, but the abstraction work should avoid trapping Tokimu in only one.

Acceptance criteria:

* A real render backend initializes successfully.
* The render path proves the render phases can observe world state.
* Render code does not mutate simulation state.
* Rendering remains downstream of world mutation rather than the primary organizing center.
* The public render layer stays Tokimu-owned rather than exposing raw backend
  objects as the default engine surface.
* Mesh, material, and pipeline identity become explicit Tokimu-owned resources
  before generalized shader systems are introduced, and draw commands select
  those resources directly rather than materials implicitly choosing pipelines.
* Reusable renderable handles may reduce repeated submission tuples, but they
  do not become transforms, visibility systems, or scene ownership by default.
* Small per-draw instance payloads may support repeated placement of the same
  renderable, but they do not imply a scene graph, transform hierarchy, or
  runtime-owned visibility system.
* The first camera/view uniform may define a frame-wide orthographic view and
  projection transform using Tokimu-shared math types and may be refreshed on
  resize with aspect awareness, but it should remain separate from per-draw
  placement and from simulation truth.
* Camera handles may select the active view without turning the renderer into
  the owner of camera semantics or world-space meaning.
* The spike avoids premature features such as PBR, shadows, deferred rendering,
  or a generalized render graph.
* The render abstraction leaves a coherent growth path for both 2D and 3D
  presentation rather than overfitting only to sprites or only to meshes.
* The render abstraction leaves room for both 2D HUD layering and 3D in-world
  interface presentation without inventing separate UI engines prematurely.
* `hello-triangle` is the current live proof for the M4 renderer spike, and it
  still intentionally stops short of transforms, scene ownership, or a general
  render graph.

### M5 — WASM Spike

* wasm crate builds
* browser canvas starts
* clear screen in browser
* shared core reused

Acceptance criteria:

* The same core crates compile for native and WASM targets.
* Browser startup reuses core/runtime concepts rather than bypassing them.
* Platform-specific code remains isolated from engine-neutral crates.
* The native/WASM split does not hard-code flat-screen assumptions that would
  block a future VR/XR adapter.

### M6 — First Playable Toy

* movable entity
* sprite or simple mesh
* input-driven update
* native + WASM demo

Acceptance criteria:

* A small world scenario demonstrates the engine's composition model.
* Input, simulation, presentation, and rendering remain visibly separated.
* The playable toy reads like one entry in the broader world corpus, not as a
  special-case app.
* The playable toy may be 2D-first for speed, but it must not imply that the
  engine core only makes sense for 2D worlds.
* If the toy includes interface elements, they should already follow the same
  architectural rule: presentation consumes shared world meaning rather than
  storing hidden gameplay truth off to the side.
* If the toy includes text commands or a transcript, those should be just
  another presentation/input path over the same world state.
* `hello-triangle` is the current native seed for the M6 first playable toy
  direction: a small collect-the-target loop over shared input and world state.

### M6.5 — Networking Boundary Note

* remote simulation remains out of scope for v0 delivery
* transport and replication boundary documented before ad hoc socket code appears

Acceptance criteria:

* The SDD clearly distinguishes future networking architecture from a promise of
  near-term multiplayer delivery.
* No early example or crate introduces socket code into `tokimu-core`.
* Any future networking spike is routed through a dedicated boundary such as a
  `tokimu-net` crate rather than scattered through runtime and platform code.

### M7 — Persistence Boundary

* engine-facing save/load model defined
* persistence crate boundary chosen
* scene or project serialization spike proves the boundary
* optional database integration, if any, stays outside core/runtime crates

Acceptance criteria:

* Scene or prefab data starts as declarative document data, not a custom language.
* The save/load boundary distinguishes editor-facing documents from runtime state.
* Persistence remains downstream of the world model rather than redefining it.

### M8 — Scene and History Model

* scene document shape defined
* scene-to-world compilation path proven
* state diff or history direction documented

Acceptance criteria:

* The engine distinguishes declarative scene documents from runtime world state.
* A compile or translation step from scene description to runtime world exists
  conceptually and, ideally, in a small spike.
* World diff/history is documented as an explicit future capability for replay,
  debugging, rollback, or inspection.
* Provenance questions such as what changed, who changed it, and why are given
  a concrete debug or inspection direction.
* The scene model can describe both 2D-oriented and 3D-oriented content without
  splitting the engine into unrelated authoring paths too early.
* The scene model can describe both screen-space HUD bindings and world-space
  interface attachments without requiring a separate ad hoc UI document system.
* The scene model can describe text-first and MUD-like presentation elements
  without requiring a separate text-only world model.

### M9 — Inspector and Rule Frontends

* inspection-first editor direction chosen
* visual rule graph direction documented
* TypeScript-first authoring direction documented
* scripting implementation deferred until real behaviors justify it

Acceptance criteria:

* Tokimu has a clear editor v0 target: entity/world tree, trait inspector,
  asset browser, system timing panel, signal log, and relationship viewer.
* The tooling direction leaves room to inspect both overlay HUD elements and
  world-space interface attachments as presentation over shared state.
* The tooling direction leaves room to inspect text-first and MUD-like views,
  including room descriptions, command history, and transcript logs.
* A visual rule graph is treated as a frontend over Tokimu rule execution, not
  as a separate runtime architecture.
* Tokimu documents a clear product split: engine implementers mostly work in
  Rust, while engine users should be able to author rules, scenarios, and other
  high-level content primarily in TypeScript.
* Any early scripting evaluation treats TypeScript as the preferred
  author-facing frontend, while explicitly rejecting a general-purpose embedded
  runtime in core engine crates until enough real behaviors have been written
  to justify the cost.

### M10 — VR/XR Architecture Spike

* presentation-layer requirements for VR/XR documented
* render and platform seams reviewed for headset support
* first candidate integration path identified

Acceptance criteria:

* Tokimu documents VR/XR as a presentation and platform concern layered over
  the same simulation core rather than as a forked engine mode.
* The design identifies the minimum abstractions needed for stereo views,
  tracked spaces, and headset-driven frame submission.
* A likely integration direction such as OpenXR plus Tokimu-owned render and
  input abstractions is named without forcing implementation before the engine
  earns it.

### M11 — Networking and Transport Architecture Spike

* remote simulation requirements documented
* transport abstraction seam chosen
* browser and native constraints reviewed together

Acceptance criteria:

* Tokimu documents networking as a transport and replication concern layered
  over the same simulation model rather than as a separate gameplay core.
* The design names the unit of replication clearly enough to reason about,
  whether that becomes commands, events, snapshots, state deltas, or a hybrid.
* A likely future direction for native and browser-capable transport support is
  identified without binding the engine to premature protocol decisions.

### M12 — Text / MUD Architecture Spike

* text-first presentation requirements documented
* command parsing and transcript flow reviewed
* remote text-session adapter path identified

Acceptance criteria:

* Tokimu documents text-first and MUD-like presentation as a view/control
  adapter over the same simulation core rather than as a separate engine or
  bespoke runtime.
* The design identifies the minimum abstractions needed for room descriptions,
  command dispatch, prompt state, inventory/status summaries, and transcripts.
* The use case includes simulation-console-style inspection for engineering,
  digital-twin, training, maintenance, and process-model workflows.
* A likely integration direction such as a text adapter plus command parser,
  layered over Tokimu-owned world and input abstractions, is named without
  forcing a full MUD server or natural-language parser.
* The first command surface remains deliberately small, with primitives such as
  look, list relations, step, emit, and why.

## 15. Design Invariants

* Core must not depend on platform APIs.
* Renderer must not own simulation state.
* Platform backends must be replaceable.
* Asset loading must work without direct filesystem access.
* WASM support must not be bolted on after native design hardens.
* VR/XR support should be planned early enough that flat-screen assumptions do
  not become accidental architecture.
* Networking, if added, must not make `tokimu-core` depend on sockets,
  transport libraries, or protocol-specific data shapes.
* Persistence must not become a hidden owner of world state.
* Examples should drive engine growth.
* No subsystem becomes "the engine" by accident.
* Core engine concepts should remain primitive-first rather than genre-first.
* Rendering must remain a consumer of world state, not the owner of engine meaning.
* Editors, blueprints, and scripting must remain frontends over the world and
  rule model, not alternate engine centers.
* Any new crate or major subsystem must justify which boundary it protects.
* Relationship meaning should remain stable even if relationship storage is
  replaced later.
* 2D and 3D support should remain presentation-level differences over shared
  engine meaning unless real corpus examples prove a harder boundary.
* HUD and interface layers must not become hidden alternate owners of gameplay
  state, whether they are screen-space overlays or world-space surfaces.
* Text-first and MUD-like presentation must also remain presentation-level over
  shared engine meaning unless corpus examples prove a harder boundary.
* Domain-specific applications such as painters, CAD tools, circuit analyzers,
  robotics dashboards, or digital-twin viewers should build on Tokimu rather
  than being absorbed into core engine crates unless a shared engine boundary
  is clearly proven.
* Core crates should prefer canonical primitives that enable many ecosystems
  over genre- or application-shaped concepts that prematurely lock Tokimu to
  one family of tools.

## 16. Testing and Validation

Tokimu should treat tests and runnable examples as part of the architecture, not
as after-the-fact polish.

Required validation layers:

* Unit tests in each crate for local data structures such as entity IDs,
  schedules, clocks, input state, and asset handles.
* Integration or smoke tests in `tests/` that prove the workspace boots and the
  public facade crate links correctly.
* Example-driven validation: `hello-window`, `hello-triangle`, and `wasm-demo`
  are architectural tests, not merely demos.
* Corpus-driven validation: new examples should prove one world relation,
  transformation, or rule cleanly enough that they can function as reusable
  engine sentences.
* Inspection-oriented validation: the architecture should preserve a path toward
  diff, replay, provenance, and timeline debugging even when those features are
  not yet fully implemented.
* Tooling-oriented validation: editor and scripting experiments should prove
  they consume Tokimu-owned world/rule abstractions rather than bypassing them.
* Determinism-focused tests for fixed timestep behavior, seeded RNG resources,
  and the rule that rendering must not mutate simulation state.
* Target-specific smoke checks: native startup should open and close cleanly;
  WASM should compile and reach a visible boot path without native-only code.
* Future-facing checks: major render and platform refactors should be reviewed
  against both WASM and planned VR/XR constraints, even before VR/XR exists.
* Networking-facing checks, once networking exists: loopback tests should prove
  transport independence, message ordering assumptions, and separation between
  replicated meaning and local presentation concerns.
* Input-facing checks: device normalization should prove that keyboard, mouse,
  and later controller/joystick inputs map into the same engine-facing action
  and axis model rather than producing fractured special cases.
* Presentation-facing checks: early render and scene work should prove that 2D
  and 3D examples can grow from shared engine concepts rather than from forked
  architecture.
* Interface-facing checks: HUD and in-world interface experiments should prove
  that presentation reads shared state and routes interaction back through
  engine-facing inputs, commands, or signals rather than inventing parallel UI
  truth.
* Text-facing checks: text-first and MUD-like experiments should prove that
  descriptions, commands, and transcripts are just another view/controller over
  shared world state rather than a separate game model.

Document rule:

* During M0-M2, this SDD is the source of truth for intended boundaries and
  milestone scope. If implementation diverges, either the code or the document
  should be corrected immediately rather than allowing architectural drift.

## 17. Open Questions

These are active design questions, not silent deferrals:

1. ECS storage model. Should the first implementation use a simple sparse-set
  style layout, or an even smaller map-based model until examples force a more
  specialized design?
2. Scheduling API shape. How much of the schedule should be declarative in M1-M2
  versus hard-coded phase ordering?
3. Plugin boundaries. Should plugins register only systems and resources in the
  early engine, or also renderer and platform hooks?
4. Asset lifecycle. Does M3-M4 need asynchronous asset loading immediately, or
  is synchronous loading acceptable until WASM constraints force the split?
5. Renderer abstraction depth. How much of `wgpu` should leak through the first
  renderer spike before a Tokimu-owned renderer API becomes mandatory?
6. Relationship representation. Should entity-to-entity links be stored as
  dedicated relation components, edge tables, or a small graph resource?
7. Save/load boundary. Which data is durable engine content versus transient
  runtime state, and what translation layer should separate them?
8. Database reuse. If Tokimu adopts parts of the local Database project, is the
  first use case scene persistence, project metadata, asset indexing, or tool
  cache data?
9. Threading model. When should Tokimu introduce a job system, if at all,
  instead of relying on a single-threaded runtime plus explicit background
  loading?
10. Knowledge layer. How much semantic structure should the world expose for
   inspection and rules without collapsing into a database-shaped engine?
11. History and diff model. Should replay, rollback, and timeline inspection be
   based on event logs, state diffs, snapshots, or some hybrid?
12. Resource provider model. How early should Tokimu formalize assets as
   provider-backed resources rather than file-centric loads?
13. Scene document format. Should the first durable scene format be RON, TOML,
  YAML, JSON5, or another plain-data representation?
14. Editor shell. Is `egui` the right first inspection/debug layer, or does the
  project need a different Rust-native tool surface?
15. Semantic rule model shape. What is the smallest Tokimu-owned rule
  representation that can serve scene data, visual graphs, and later scripting
  equally well?
16. Scripting threshold. After how many real behaviors should Tokimu reassess
  whether Rhai, Lua, or no scripting at all is the right next step?
17. VR/XR abstraction seam. Which concepts belong in render, platform, and
  input layers to support OpenXR-style integration without leaking headset
  details into the simulation core?
18. Networking authority model. If Tokimu grows remote simulation, should the
  first architecture assume authoritative host, deterministic lockstep,
  snapshot interpolation, rollback, or another hybrid?
19. Transport surface. What is the narrowest Tokimu-owned transport abstraction
  that can span native and browser-capable environments without leaking a
  specific networking library everywhere?
20. Replication unit. What engine-level meaning should cross the wire first:
  player commands, signals, world diffs, snapshots, or some mixed model?
21. Input abstraction shape. What is the smallest Tokimu-owned action and axis
  model that cleanly spans keyboard, mouse, controllers, joysticks, and later
  VR/XR inputs without collapsing into device-specific special cases?
22. Spatial abstraction shape. What is the smallest Tokimu-owned spatial model
  that supports both 2D-oriented and 3D-oriented worlds without forcing a false
  split too early or hiding real differences too long?
23. Interface model. What is the smallest Tokimu-owned presentation model that
  can describe both 2D HUD elements and 3D in-world interfaces without creating
  a separate hidden gameplay state machine for UI?
24. Text presentation model. What is the smallest Tokimu-owned presentation and
  command model that can support room descriptions, prompts, transcripts, and
  MUD-like interaction without turning text mode into a separate engine?
25. Frontend and model versioning. As authoring frontends and semantic models
  evolve, how are they versioned so older authored content still compiles, and
  how do independent frontends stay interoperable through shared engine meaning
  rather than through ad hoc package conventions?
26. Canonical primitive surface. Which concepts belong in Tokimu's shared
  foundational vocabulary, and which should be left to higher-level engines so
  the ecosystem does not fragment around incompatible names for the same idea?

## 18. Definition of Done

A milestone is only done when its behavior is implemented, exercised, and still
fits the engine boundaries described above.

1. The relevant workspace crates compile on stable Rust.
2. The milestone's example or smoke target runs successfully.
3. New public APIs are exercised by at least one example or test, not only by
  unit-level construction.
4. Any boundary change between core, runtime, platform, renderer, input, or
  assets is reflected in this SDD or an ADR before the change is treated as
  settled.
5. `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and
  `cargo test --workspace` are clean once the workspace exists.
6. WASM-related milestones additionally prove that native-only assumptions have
  not leaked into core crates.
7. If a change introduces a new abstraction, crate, trait, or service layer,
  it is justified against the AI Implementation Principles above and tied to a
  concrete example, milestone need, or acceptance criterion.

## 19. Decision Summary

v0 succeeds when Tokimu can run one native playable toy whose world state,
input, signals, and rendering path are inspectable and separated by crate
boundaries.

That v0 toy may be 2D-first for implementation speed, but the surrounding
architecture should still preserve a clean path toward 3D-capable scenes and
rendering.

Tokimu is a Rust-native real-time state-processing engine.

It should start small:

```text
runtime loop
world state
relationships
signals
state mutation
input
inspection
renderer as proof
```

Then grow only as examples prove the need.

## 20. Design Influences

These are external principles Tokimu adopts in its own terms. They are only
useful when translated into Tokimu-specific decisions.

* Favor graceful degradation and actionable diagnostics over opaque failures.
  Tokimu expresses this through structured diagnostics, startup messages,
  explicit failure surfaces, and examples that fail loudly enough to be fixed.
* Build great tools, not just products. Tokimu therefore treats editor and
  authoring capabilities as first-class, and the platform aims to support a
  tool ecosystem rather than a single shipped application.
* Use tests and runnable examples as the development team. Tokimu reflects this
  through milestone-driven implementation, validation after each behavior
  change, and examples that prove architectural boundaries.
* Fix visible bugs immediately. When an implementation or API mismatch appears,
  Tokimu should stop, repair the local slice, and rerun the narrow validation
  before widening the scope.
* Use a development system that is better than the target. Tokimu leans on
  Rust, TypeScript frontends, diagnostics, and AI-assisted iteration to build a
  platform that can target native, browser, and future semantic-front-end
  environments.
* Encapsulate functionality to preserve design consistency. Tokimu expresses
  this as explicit ownership boundaries between core, runtime, platform,
  rendering, assets, and future authoring frontends.
* Code transparently. Design decisions, hypotheses, and validation should be
  documented alongside implementation so the implementation history becomes a
  useful engineering artifact rather than an after-the-fact guess.
* Write for the current milestone, but keep the platform reusable. Tokimu is an
  infrastructure project, so it should avoid speculative abstraction while
  still proving reusable boundaries one small step at a time.

Principles that are philosophically true but not especially architectural, such
as programming being a creative art form, are intentionally left out of the
core design rules unless they become actionable for Tokimu's implementation.
