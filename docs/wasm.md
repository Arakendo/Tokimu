# WASM Notes

Tokimu is desktop-first in early milestones, but the architecture is intended to
support WebAssembly without rewriting the simulation core.

Early rules:

* keep `tokimu-core` free of native OS APIs
* keep asset access abstract
* avoid blocking runtime-critical code paths
* isolate browser-specific startup to `tokimu-wasm` and platform crates