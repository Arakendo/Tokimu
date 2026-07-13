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

The current `tokimu-wasm` crate remains a placeholder until that spike is ready.