# Tokimu

Tokimu is a Rust-native real-time simulation and rendering engine for games,
interactive tools, technical simulations, and eventually WebAssembly
deployment.

It is not merely a game engine in the mechanical sense. It is a
state-processing runtime: it accepts input, rules, assets, and time, then
produces updated world state and rendered output.

## Naming Conventions

Tokimu uses ownership-first crate naming.

* `tokimu-<role>` for first-party engine crates such as `tokimu-core`,
  `tokimu-runtime`, `tokimu-render`, `tokimu-platform`, `tokimu-assets`,
  `tokimu-input`, `tokimu-rule`, `tokimu-ts-frontend`, and `tokimu-wasm`
* `tokimu-<domain>` for Tokimu-owned optional capability crates, once earned
* `tokimu-<domain>-<backend>` for concrete backend adapters behind those
  capabilities
* `@tokimu/*` for TypeScript authoring packages under `frontends/`

That naming follows the broader architectural rule: names should reveal who
owns the meaning, not which library happened to be convenient.

## Current Workspace

The workspace is past scaffolding and already includes the current engine and
authoring slices.

* Core/runtime: `tokimu-core`, `tokimu-runtime`, `tokimu-rule`, `tokimu`
* Foundational services: `tokimu-render`, `tokimu-platform`, `tokimu-assets`,
  `tokimu-input`
* Authoring/runtime bridges: `tokimu-ts-frontend`, `tokimu-wasm`
* Examples: `hello-window`, `hello-triangle`, `hello-3d-mono`,
  `hello-3d-stereo`, `hello-rule-model`, `hello-3d-openxr`, `hello-snake`,
  `hello-pacman`, `hello-space-invaders`, `hello-missile-command`,
  `hello-asteroids`, `hello-fps-web`, `hello-cad`, `hello-vector-draw`,
  `hello-svg`, `hello-glb`, `hello-2d-physics`, `hello-ui`, `hello-shader`
* Frontends: `frontends/` with `tokimu`, `@tokimu/rules`, and example packages

See the [Software Design Document](docs/Tokimu%20Software%20Design%20Document.md)
for the full architecture, subsystem breakdown, and milestone plan.

## Goals

* Rust-native engine architecture
* Deterministic simulation core where practical
* Modular subsystems with clear ownership boundaries
* ECS-friendly world model
* Desktop-first development, with planned WebAssembly export
* Minimal early scope with room for later expansion

## Related Docs

* [Tokimu Architectural Maxims](docs/architectural-maxims.md)
* [Tokimu Kernel Principles](docs/kernel-principles.md)
* [Tokimu Semantic Kernel Map](docs/semantic-kernel-map.md)
* [Tokimu Primitive Ledger](docs/primitive-ledger.md)
* [Tokimu Example Philosophy](docs/example-philosophy.md)
* [Tokimu Capability Backends](docs/capability-backends.md)
* [Tokimu Future Workspace Layout](docs/future-workspace-layout.md)
* [Tokimu Contribution Admission Guide](docs/contribution-admission-guide.md)

## License

Licensed under the [MIT License](LICENSE).
