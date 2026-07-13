# Tokimu

Tokimu is a Rust-native real-time simulation and rendering engine intended for
games, interactive tools, technical simulations, and eventually WebAssembly
deployment.

It is not merely a "game engine" in the mechanical sense — it is a
state-processing runtime: it accepts input, rules, assets, and time, then
produces updated world state and rendered output.

## Naming Note

Tokimu is a peer project to Tosumu, and both names descend from the same
conlang tradition represented by the Tonesu project.

Tonesu is explicitly designed around compositional meaning and epistemic
honesty: it cares not only about what is claimed, but about what kind of claim
is being made and what structural boundaries make that claim valid. Those ideas
had a stronger direct influence on Tosumu, but they matter here as well.

In Tokimu, that influence shows up as engineering preference rather than
terminology: clear simulation truth, explicit subsystem boundaries, visible
diagnostics, and caution about letting tooling or persistence layers silently
overclaim ownership of runtime state.

## Related Projects

Tokimu sits alongside two related projects:

* **Tosumu** — a peer Rust project focused on storage and database design,
  where these philosophical influences are applied more directly to integrity,
  explicit state claims, and persistence boundaries.
* **Tonesu** — the broader conlang project from which the naming lineage comes.
  Tonesu emphasizes compositional meaning, explicit structural relations, and
  epistemic honesty.

Tokimu does not adopt Tonesu terminology as an API surface. It borrows the
design pressure toward clearer truth boundaries and more honest system claims.

## Status

The workspace has moved beyond scaffolding: the core crates, platform seam,
renderer proof, and two native examples are all runnable. See the
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
  tokimu-input     # normalized input state and action mapping
  tokimu-wasm      # WebAssembly entry point
examples/          # hello-window, hello-triangle, wasm-demo
```

## License

Licensed under the [MIT License](LICENSE).
