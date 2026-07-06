# Tokimu

Tokimu is a Rust-native real-time simulation and rendering engine intended for
games, interactive tools, technical simulations, and eventually WebAssembly
deployment.

It is not merely a "game engine" in the mechanical sense — it is a
state-processing runtime: it accepts input, rules, assets, and time, then
produces updated world state and rendered output.

## Status

Early design and scaffolding. See the
[Software Design Document](docs/Tokimu%20Software%20Design%20Document.md) for the
full architecture, subsystem breakdown, and milestone plan.

## Goals

- Rust-native engine architecture
- Deterministic simulation core where practical
- Modular subsystems with clean boundaries
- ECS-friendly world model
- Desktop-first development, with planned WebAssembly export
- Minimal early scope with room for later expansion

## Planned workspace layout

```text
crates/
  tokimu           # public facade crate
  tokimu-core      # engine-neutral world/ECS/time/math
  tokimu-runtime   # app loop, scheduling, plugins
  tokimu-render    # rendering abstraction (wgpu)
  tokimu-platform  # windowing, input, timing (winit / wasm)
  tokimu-assets    # asset ids, handles, loaders
  tokimu-wasm      # WebAssembly entry point
examples/          # hello-window, hello-triangle, wasm-demo
```

## License

Licensed under the [MIT License](LICENSE).
