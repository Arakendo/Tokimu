# Tokimu Roadmap

Short-term focus:

1. MVP 1 — finish the runnable engine core and keep the M3/M4 proofs healthy
2. MVP 2 — land the WASM spike and keep the first playable toy moving

The main planning detail remains in the software design document.

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

MVP 1 exit criteria: a native example opens a window, reads normalized input,
and renders Tokimu-owned resources through a real backend without the renderer
owning simulation state.

### MVP 2 — First Playable + Browser Parity

Goal: a small playable loop that also proves the shared core runs in a browser.

- [ ] M5 WASM spike — core crates compile for WASM and a browser canvas clears
  screen while reusing core/runtime concepts
- [~] M6 first playable toy — `hello-triangle` carries a small collect-the-target
  loop over shared input and world state; still native-only and not yet a
  distinct world-corpus scenario
- [ ] M6.5 networking boundary note — future transport/replication boundary
  documented before any socket code appears

MVP 2 exit criteria: the same core powers a native playable toy and a browser
build, with input, simulation, presentation, and rendering visibly separated.

### MVP 3 — Content and Persistence

Goal: describe worlds as data and move them in and out of the runtime.

- [ ] M7 persistence boundary — engine-facing save/load model and crate boundary
- [ ] M8 scene and history model — scene document shape, scene-to-world compile
  path, and a documented diff/history direction

MVP 3 exit criteria: declarative scene documents compile into runtime world
state, with persistence staying downstream of the world model.

### MVP 4 — Tooling and Authoring

Goal: make the world inspectable and author-facing content approachable.

- [ ] M9 inspector and rule frontends — inspection-first editor target, visual
  rule-graph direction, and TypeScript-first authoring direction documented

MVP 4 exit criteria: an inspection-first editor direction exists and authoring
frontends are framed over the world/rule model rather than as alternate cores.

### MVP 5 — Platform and Presentation Expansion

Goal: prove Tokimu can grow into new presentation and transport surfaces.

- [ ] M10 VR/XR architecture spike — stereo views, tracked spaces, headset frame
  submission documented as presentation over the shared core
- [ ] M11 networking and transport spike — replication unit and transport seam
  documented for native and browser targets
- [ ] M12 text / MUD architecture spike — text-first presentation, command
  dispatch, and transcript flow documented as an adapter over the shared core

MVP 5 exit criteria: VR/XR, networking, and text-first presentation each have a
named integration path layered over the same simulation core.

### Phase Status At A Glance

- MVP 1: partial — native runnable core is in place, but the host loop remains
  partial and the renderer spike is still incomplete
- MVP 2: partial — the first playable toy exists, but WASM parity is still
  pending
- MVP 3: not started
- MVP 4: not started
- MVP 5: not started

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
- `tokimu-wasm`: placeholder `boot_message()` only.

### Example Proofs

- `hello-window` is the live proof for M3. It opens a native window, translates
  WASD into runtime-owned input state, and keeps the surface intentionally blank.
- `hello-triangle` is the live proof for M4 and the current M6 seed. It opens a
  native window, brings up `wgpu`, draws multiple 2D shapes with explicit
  Tokimu-owned resources, and now includes a small collect-the-target loop.
- `wasm-demo` remains a browser host placeholder.

### Next Wiring Steps

1. Extend M4 beyond the current uploaded-renderable proof by adding
  generalized shader/pipeline management while keeping renderer ownership
  Tokimu-shaped rather than backend-shaped.
2. Close the WASM side of M3 by adding `web-sys`/`wasm-bindgen` to
  `tokimu-platform` and broadening event coverage so browser input follows the
  same engine-facing model.
3. Keep authoring and rule systems deferred; there is no `tokimu-rule` or
  authoring-frontend crate yet, and they should wait until the world/rule
  model has real callers.