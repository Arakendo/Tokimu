# Tokimu Codex Instructions

## Project Intent

Tokimu is a Rust-native real-time simulation and rendering engine with planned
WebAssembly support. Treat it as an engine project, not as a general app or a
database-backed tool.

Longer term, Tokimu may support multiple application classes on top of the same
core architecture: games, technical simulators, creative tools, robotics or
industrial dashboards, and other semantic interactive applications. Treat
that as a platform ambition, not as a reason to absorb domain-specific
applications into the engine core.

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
- Prefer making specialized applications straightforward to build on Tokimu
  over baking those applications directly into core engine crates.

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
- Read the relevant ADRs in `docs/ADR/` before introducing a subsystem,
  changing ownership, or adding a dependency across an established boundary.
- Read relevant Architectural Review Records in `docs/Architectural Reviews/`
  when corpus pressure, ownership questions, or prior deferred/rejected findings
  overlap the change. Open a review record before making an unresolved
  architectural question look settled in code.
- ADRs record accepted architectural decisions. Do not work around one with a
  local implementation; revise the ADR and its related design docs deliberately
  when the decision itself needs to change.
- Architectural Review Records preserve evidence and findings but do not
  override ADRs. If reopened evidence changes a binding decision, update or
  supersede the ADR explicitly and retain the earlier review history.
- When adding a new subsystem or changing a boundary, prefer updating the SDD
  before or alongside the code.
- Do not invent philosophical jargon in code. If conceptual lineage matters,
  record it in docs, not in public API names.
- Treat the SDD as an architectural constraint system, not just background
  reading. If a locally reasonable implementation conflicts with the SDD's
  ownership model or boundaries, the SDD wins unless the architecture is being
  intentionally revised.

### Accepted ADR Boundaries

- `ADR-0001-engine-boundaries.md` defines the engine's high-level ownership
  boundaries.
- `ADR-0003-capability-ownership-boundary.md` distinguishes native Tokimu
  meaning, Tokimu-owned capability semantics, and replaceable backends.
- `ADR-0004-foundational-presentation-text-and-icons.md` defines text and icon
  semantics as a foundational presentation capability. Keep `tokimu-core` free
  of font parsers, icon libraries, rasterizers, atlases, and renderer-native
  objects. Treat TTF/OTF/system fonts and Lucide as replaceable providers;
  preserve provider-neutral handles, measurement, layout, fallback, and
  diagnostics contracts.
- `ADR-0005-admission-evidence-and-maintainer-exceptions.md` permits documented
  provisional admission or permanent evidence substitution without weakening
  accepted ownership and dependency boundaries.
- `ADR-0006-native-execution-policy.md` makes execution policy and cross-domain
  coordination native Tokimu concerns while keeping concrete thread, worker,
  and executor mechanisms in runtime/platform adapters. It does not admit
  parallel `World` mutation.

## AI Implementation Principles

- Before introducing a new abstraction, crate, trait, service, or subsystem,
  ask whether an existing abstraction can be extended instead.
- Tie new abstractions to a concrete example, milestone need, or acceptance
  criterion. Do not generalize because another engine happens to have a
  similar layer.
- Preserve world-first architecture: simulation owns truth; presentation,
  platform, tooling, and persistence adapt or observe it.
- Prefer semantic concepts over implementation-shaped concepts. Do not create
  a new named layer when the real need is only an internal helper.
- Avoid duplicating an existing concept under a different name.
- Check whether a change bakes in assumptions that make native/WASM parity,
  future VR/XR support, or replaceable presentation adapters harder.

## Implementation Preferences

- Favor small, compileable increments over large speculative rewrites.
- Keep early implementations boring and concrete. Simplicity beats premature
  flexibility.
- Avoid hidden global state where a resource, config object, or explicit input
  would make control flow clearer.
- Rendering must not mutate simulation state.
- Determinism-related behavior should be explicit: fixed timestep policy,
  seeded randomness, and clear ownership of time progression.
- When a milestone or example only needs one concrete behavior, implement that
  behavior directly before inventing extra generalized systems around it.
- Prefer acceptance-criteria-complete work over speculative completeness.

## Authoring Frontends

- Tokimu itself remains primarily a Rust engine.
- If TypeScript frontends are added later, treat them as author-facing adapters
  over Tokimu-owned semantic models, not as alternate runtimes.
- Prefer a Rust engine / TypeScript authors split: engine implementers mostly
  work in Rust, while engine users should be able to author high-level content
  in TypeScript.
- Do not make `tokimu-core` or `tokimu-runtime` depend on TypeScript tooling,
  JavaScript runtimes, or authoring frontend infrastructure.
- Independent frontends should target interoperable Tokimu semantic models
  rather than integrating through ad hoc TypeScript package conventions.

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
