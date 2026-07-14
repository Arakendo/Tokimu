# WASM Notes

Tokimu is desktop-first in early milestones, but the architecture is intended to
support WebAssembly without rewriting the simulation core.

Early rules:

* keep `tokimu-core` free of native OS APIs
* keep asset access abstract
* avoid blocking runtime-critical code paths
* isolate browser-specific startup to `tokimu-wasm` and platform crates

## M5 target

The first WASM spike should stay small and prove only the browser-facing seam:

* the same core crates compile for native and WASM targets
* browser startup reuses core/runtime concepts rather than bypassing them
* platform-specific startup stays isolated from engine-neutral crates
* the browser path does not hard-code assumptions that would block a future
	VR/XR adapter or a different presentation host

The current `tokimu-wasm` crate now exposes a small browser bootstrap with an
automatic `wasm_bindgen(start)` hook that installs Tokimu input bridging on a
canvas, but it is still a narrow spike and not yet the full browser runtime
path.

The `examples/wasm-demo` page now loads that bootstrap from a browser host
loop scaffold, and the loop itself is owned by Rust through `tokimu-wasm`, so
the seam is exercised by an actual HTML entry point with shared input-driven
canvas feedback even though the full runtime remains pending.
