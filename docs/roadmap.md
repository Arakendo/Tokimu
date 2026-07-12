# Tokimu Roadmap

Short-term focus:

1. M0 workspace scaffold
2. M1 runtime loop skeleton
3. M2 minimal ECS
4. M3 window and input spike
5. M4 renderer spike

The main planning detail remains in the software design document.

## Implementation Status Snapshot

Snapshot as of the last audit. This is volatile; the SDD remains the source of
truth for intended architecture. Update this list as milestones land.

- M0 skeleton: done. Workspace builds, all documented crates exist, facade
  feature flags (`render`, `platform`) are wired correctly, `cargo test` passes.
- `tokimu-core`: partial. `EntityId`, `Schedule`/`Phase`, `FixedTimeStep`, and
  `Diagnostics` exist. `World` only supports `spawn()`; `Component`, `Resource`,
  and `Event` are trait stubs with no storage or queries yet.
- `tokimu-runtime`: partial. `App`, `RuntimeConfig`, `Plugin` trait, and
  `tick_fixed_updates()` exist and are test-covered. No driven frame loop yet.
- `tokimu-input`: most complete crate. Keyboard, mouse, controller, action map,
  and aggregate `InputState` are implemented with tests.
- `tokimu-assets`: partial. `AssetId`, `AssetHandle<T>`, and `AssetStore` exist;
  `AssetLoader` is a trait stub.
- `tokimu-render`: stubs only. Types (`Color`, `RenderCommand`, `Renderer`,
  handles) exist, but `WgpuBackend` is a non-functional placeholder and the
  crate declares no `wgpu` dependency.
- `tokimu-platform`: stubs only. `WindowConfig`, `Clock`, and event types exist;
  `native`/`wasm` modules only return backend names. No `winit`/`web-sys` yet.
- `tokimu-wasm`: placeholder `boot_message()` only.
- Examples: `hello-window`, `hello-triangle`, and `wasm-demo` are console-only
  placeholders. They compile and exercise the API surface but do not yet open a
  window, render, or boot in a browser.

### Coherence notes / next wiring steps

- `tokimu-render` exports `WgpuBackend` without a `wgpu` dependency; wiring the
  real backend (SDD open question 5) is the natural M4 unblock.
- `tokimu-platform` needs `winit` (native) and `web-sys`/`wasm-bindgen` (WASM)
  before `hello-window` can become a real window (M3).
- No `tokimu-rule` or authoring-frontend crates exist yet; per SDD 5.11/5.12
  they are intentionally deferred until the rule model has real callers.