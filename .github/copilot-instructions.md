# Tokimu Copilot Instructions

## Project Intent

Tokimu is a Rust-native real-time simulation and rendering engine with planned
WebAssembly support. Treat it as an engine project, not as a general app or a
database-backed tool.

The current source-of-truth design document is `docs/Tokimu Software Design Document.md`.
If code changes alter architecture, boundaries, or milestone expectations,
update the SDD or add an ADR rather than letting code and docs drift apart.

## Architecture Boundaries

- Keep `tokimu-core` engine-neutral. Do not add windowing, GPU, filesystem,
  browser, or database dependencies to it.
- Keep `tokimu-runtime` focused on application execution: app lifecycle,
  scheduling, fixed-step updates, and plugin orchestration.
- Keep `tokimu-render` and `tokimu-platform` as adapters around rendering and
  OS/browser concerns. They must not become hidden owners of simulation state.
- Keep persistence optional and outside `tokimu-core` and `tokimu-runtime`.
  If database-backed persistence appears later, it belongs in its own crate.
- Prefer structural boundaries over advisory comments. If a bad dependency or
  ownership direction is dangerous, redesign so the code shape makes it harder.

## Design Habits

These habits are intentionally borrowed from the better parts of the Tosumu and
Tonesu projects.

- Examples before abstractions. Let `hello-window`, `hello-triangle`, and later
  demos shape the engine more than speculative framework design.
- Stabilize traits and public abstractions only after real callers exist. Do
  not generalize early because an API "might" be useful.
- Prefer plain English names for crates, types, functions, and comments. The
  Tonesu/Tosumu lineage is conceptual, not an API vocabulary requirement.
- Prefer explicit diagnostics over silent fallback. Startup failures, backend
  choices, schedule behavior, and platform constraints should be visible.
- Be precise about what a system guarantees. Do not blur "observed behavior"
  with "architecturally guaranteed behavior" in docs or code comments.
- If a behavior is intentionally unspecified, say so explicitly rather than
  letting callers infer a contract accidentally.

## Source of Truth and Documentation

- Treat hand-authored docs as authoritative for current architecture until code
  intentionally supersedes them through an explicit update.
- Keep README, SDD, and ADRs consistent with actual workspace structure.
- When adding a new subsystem or changing a boundary, prefer updating the SDD
  before or alongside the code.
- Do not invent philosophical jargon in code. If conceptual lineage matters,
  record it in docs, not in public API names.

## Implementation Preferences

- Favor small, compileable increments over large speculative rewrites.
- Keep early implementations boring and concrete. Simplicity beats premature
  flexibility.
- Avoid hidden global state where a resource, config object, or explicit input
  would make control flow clearer.
- Rendering must not mutate simulation state.
- Determinism-related behavior should be explicit: fixed timestep policy,
  seeded randomness, and clear ownership of time progression.

## Validation Expectations

- After code changes, prefer validating with:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
- New public APIs should be exercised by at least one example or test.
- If a milestone adds a new boundary or subsystem, update docs and validation
  expectations in the same change.

## Current Workspace Shape

- `crates/tokimu-core` — engine-neutral simulation concepts
- `crates/tokimu-runtime` — app/runtime loop and plugins
- `crates/tokimu-render` — rendering abstractions
- `crates/tokimu-platform` — native/WASM platform integration
- `crates/tokimu-assets` — asset IDs, handles, loaders
- `crates/tokimu-input` — normalized input state
- `crates/tokimu-wasm` — WASM entry surface
- `crates/tokimu` — public facade crate
- `examples/` — architecture-driving demos, not throwaway samples