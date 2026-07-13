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
  `tick_fixed_updates()` exist and are test-covered. `App` now owns a current
  engine-facing `InputState`, can apply input events, and exposes a minimal
  `tick(frame_delta_seconds)` seam that updates elapsed time and fixed-step
  accounting. `RunLoopSummary` now reports requested fixed updates and whether
  the per-frame fixed-step cap was hit. `App` also owns `RunLoopDiagnostics`
  for cumulative frame/fixed-step counters and last/max frame timing. There is
  still no fully driven runtime loop yet.
- `tokimu-input`: most complete crate. Keyboard, mouse, controller, action map,
  and aggregate `InputState` are implemented with tests.
- `tokimu-assets`: partial. `AssetId`, `AssetHandle<T>`, and `AssetStore` exist;
  `AssetLoader` is a trait stub.
- `tokimu-render`: partial. Types (`Color`, `RenderCommand`, `Renderer`,
  handles) exist, and `WgpuBackend` now performs a real `wgpu`
  instance/adapter/device bring-up. It can also create/configure a native
  surface and present uploaded mesh geometry through an uploaded pipeline.
  Mesh, material, and pipeline data can now be uploaded explicitly, with draw
  commands selecting those resources by handle or through a reusable
  renderable handle. Draw submission now also carries a minimal per-draw 2D
  instance payload for placement reuse, but generalized
  shader/pipeline management still does not exist.
- `tokimu-platform`: partial. `WindowConfig`, `Clock`, and Tokimu-owned
  `PlatformInputEvent` types exist; the native path now creates a real `winit`
  window, translates basic keyboard/mouse/resize/close events, and exposes
  `run_window()`, `run_window_with_handler()`, and `run_window_with_app()`
  helpers with a small platform event-handler seam. Native apps can now receive
  a native window-created callback and propagate handler errors back out of the
  run loop. WASM support is still a placeholder. The native proof is already
  runnable in `hello-window`, and its blank surface is explicitly labeled as
  intentional.
- `tokimu-wasm`: placeholder `boot_message()` only.
- Examples: `hello-window` now opens a real native window and proves a tiny
  input-to-intent path through runtime-owned `InputState` by translating WASD
  state into a movement vector, and it exercises a minimal per-frame runtime
  tick path with runtime-owned fixed-step diagnostics. The surface is
  intentionally blank, and the title makes that intent explicit. `hello-window`
  is the current live proof for the M3 window/input spike.
  `hello-triangle` now opens a real native window through `tokimu-platform`,
  creates a `wgpu` surface, uploads explicit triangle mesh, material, and
  pipeline data, uploads a reusable renderable handle, uploads an active
  aspect-aware orthographic camera handle sized from the native window with
  separate matrix-based view/projection data via `tokimu-core`, keeps an
  explicit logical world height in the example, refreshes that camera on
  resize, renders it, and prints real adapter/backend info. The same
  renderable can now be submitted at more than one placement through per-draw
  instance data, including rotation and animated placement. The proof now uses
  six differently colored 2D shapes, including square and diamond meshes plus
  a breathing background, and keyboard plus mouse input steer different parts
  of the composition and update the window title, with Space resetting the
  visible offsets, left click toggling paused motion, right click toggling a
  palette mode, middle click reversing motion direction, and the title showing
  active versus neutral, moving versus paused, default versus alternate
  palette, forward versus reverse direction, and drag versus idle mouse modes,
  to make that reuse obvious. `hello-triangle` is the current live proof for
  the M4 renderer spike. It still does not prove transforms, scene ownership,
  camera/view management, or general shader/pipeline management.
  `wasm-demo` remains a browser host placeholder.

### Coherence notes / next wiring steps

- `tokimu-render` and `tokimu-platform` now prove the window-to-surface path,
  but M4 still needs generalized shader/pipeline management and a renderer API
  that grows beyond the current uploaded-renderable, active-camera, plus
  per-draw-placement proof.
- `tokimu-platform` still needs `web-sys`/`wasm-bindgen` for the WASM side of
  M3, plus broader event coverage and tighter integration with runtime-owned
  input collection. The native side of M3 is already proven by `hello-window`.
- No `tokimu-rule` or authoring-frontend crates exist yet; per SDD 5.11/5.12
  they are intentionally deferred until the rule model has real callers.