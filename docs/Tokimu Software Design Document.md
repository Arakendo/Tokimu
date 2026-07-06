# Tokimu Software Design Document

| Field        | Value                                             |
| ------------ | ------------------------------------------------- |
| Status       | Draft                                             |
| Version      | 0.1.0                                             |
| Last updated | 2026-07-06                                        |
| Scope        | v0 architecture and initial milestones            |
| Language     | Rust (edition 2021)                               |

## 1. Purpose

Tokimu is a Rust-based real-time simulation and rendering engine intended for
games, interactive tools, technical simulations, and eventually WebAssembly
deployment.

Tokimu is not merely a "game engine" in the mechanical sense. It is a
state-processing runtime: it accepts input, rules, assets, and time, then
produces updated world state and rendered output.

## 2. Core Goals

- Rust-native engine architecture
- Deterministic simulation core where practical
- Modular subsystems
- ECS-friendly world model
- Desktop-first development
- Planned WebAssembly export support
- Clean separation between engine core, platform backends, renderer, and tools
- Minimal early scope with room for later expansion

## 3. Non-Goals for v0

- Full editor application
- Networked multiplayer
- AAA rendering stack
- Physics engine from scratch
- Built-in scripting language
- Complete asset pipeline
- Mobile support
- Console support

## 4. Architecture Overview

```text
Application
    ↓
Tokimu Runtime
    ↓
World / ECS
    ↓
Systems
    ↓
Platform Backend
    ↓
Renderer / Audio / Input / WASM Host
```

Tokimu should avoid hard-coding platform assumptions into the simulation layer.
The same core should eventually run on native desktop and WebAssembly.

## 5. Major Subsystems

### 5.1 tokimu-core

Owns the engine-neutral model:

* world state
* entities/components
* resources
* events
* schedules
* time-step policy
* math primitives
* asset handles
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
* render graph
* sprites/meshes/materials
* texture handles
* draw commands
* backend-neutral renderer API

Early implementation may use `wgpu`, since it supports native and WASM targets.

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
* `gilrs` later for gamepads

Likely WASM stack:

* `wasm-bindgen`
* `web-sys`
* browser canvas target
* `wgpu` WebGPU path where available

### 5.5 tokimu-assets

Owns asset loading and management:

* asset IDs
* handles
* loaders
* hot reload later
* embedded asset bundles later
* WASM-friendly asset fetch abstraction

### 5.6 tokimu-input

Owns normalized input state:

* keyboard
* mouse
* gamepad later
* touch later
* action mapping

### 5.7 tokimu-audio

Deferred early, but eventually owns:

* sounds
* music
* spatial audio
* mixer state

### 5.8 tokimu-tools

Optional future tooling:

* scene inspection
* debug overlays
* asset inspection
* editor support

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

## 8. Determinism

Tokimu should prefer deterministic simulation where practical.

Rules:

* fixed timestep available for simulation
* random number generation should use explicit seeded RNG resources
* floating-point determinism is best-effort, not guaranteed across all platforms
* rendering must not mutate simulation state

## 9. WASM Strategy

WASM support is a planned architectural requirement, not a v0 blocker.

Design constraints from day one:

* avoid direct `std::fs` in core crates
* avoid blocking APIs in runtime-critical paths
* isolate platform code
* use feature flags for native-only behavior
* keep asset loading abstract
* avoid threads in early WASM path unless explicitly designed

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
│   │   └── ADR-0001-engine-boundaries.md
│   ├── wasm.md
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
└── tests/
    └── smoke.rs
```

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
    "crates/tokimu-wasm",
    "examples/hello-window",
    "examples/hello-triangle"
]

[workspace.package]
edition = "2021"
license = "MIT"
repository = "https://github.com/Arakendo/tokimu"
version = "0.1.0"

[workspace.dependencies]
anyhow = "1"
thiserror = "1"
tracing = "0.1"
glam = "0.29"
wgpu = "23"
winit = "0.30"
pollster = "0.4"
wasm-bindgen = "0.2"
web-sys = "0.3"
js-sys = "0.3"
```

## 13. Public Facade Crate

`crates/tokimu` should re-export the stable public API:

```rust
pub use tokimu_core::*;
pub use tokimu_runtime::*;
pub use tokimu_assets::*;

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

### M1 — Runtime Loop

* app builder
* plugin trait
* fixed timestep clock
* basic diagnostics

### M2 — Minimal ECS

* entity IDs
* components
* resources
* simple systems
* tests

### M3 — Window + Input

* native window
* keyboard/mouse input
* normalized input resource

### M4 — Renderer Spike

* wgpu initialization
* clear screen
* triangle
* camera abstraction

### M5 — WASM Spike

* wasm crate builds
* browser canvas starts
* clear screen in browser
* shared core reused

### M6 — First Playable Toy

* movable entity
* sprite or simple mesh
* input-driven update
* native + WASM demo

## 15. Design Invariants

* Core must not depend on platform APIs.
* Renderer must not own simulation state.
* Platform backends must be replaceable.
* Asset loading must work without direct filesystem access.
* WASM support must not be bolted on after native design hardens.
* Examples should drive engine growth.
* No subsystem becomes "the engine" by accident.

## 16. Decision Summary

Tokimu is a Rust-native real-time state-processing engine.

It should start small:

```text
runtime loop
world state
input
renderer
native example
WASM spike
```

Then grow only as examples prove the need.
