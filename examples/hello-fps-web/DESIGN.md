# Hello FPS Web - Example Design Document

| Field | Value |
| --- | --- |
| Purpose | Demonstrate a browser-hosted FPS loop while validating Rust/TypeScript boundaries |
| Primary Proof | Rust owns simulation truth |
| Secondary Proof | TypeScript stays in the browser shell and presentation layer |
| Non-Goals | Asset pipeline, networking, full FPS content, or a Quake 2 clone |

## Purpose

`hello-fps-web` is Tokimu's monolithic test project for a Quake 2-inspired
first-person starter. It is meant to prove that Tokimu can host a small 3D
shooter loop, exercise TypeScript in the browser shell, and keep simulation
truth in Rust while the web layer handles UI and presentation concerns.

The project should stay procedural, readable, and small enough to act as a
stress test rather than a content platform. Quake 2 is a good target because it
exercises movement, weapons, enemy/target pressure, and level-style space
without demanding a modern AAA content pipeline.

## What It Proves

- First-person movement using WASD
- Cursor-driven look direction
- Left-click projectile firing
- Target interaction in 3D space
- Procedural geometry only, no external asset dependency for MVP
- Rust-owned simulation with a TypeScript browser shell
- A single example project that can eventually grow into a full gameplay proof
- A browser HUD that can visualize frame timing, player pose, and status data
- Rust can publish live frame snapshots into the browser shell

## Monolithic Layout

The project should keep its own small internal structure:

```text
hello-fps-web/
├── Cargo.toml
├── DESIGN.md
├── src/
│   └── main.rs                # Rust game loop, simulation, and rendering setup
└── web/
		├── index.html             # browser host shell
		├── package.json           # TypeScript / browser tooling entry point
		├── tsconfig.json
		└── src/
				├── main.ts            # browser bootstrap
				├── hud.ts             # HUD, labels, and small UI helpers
				└── protocol.ts        # shared browser-side types and view models
```

This is intentionally monolithic for now. The point is to keep the Rust side,
the browser shell, and the TypeScript surface close together while the target
shape is still being learned.

## Architecture Boundaries

This example must stay inside the engine boundaries defined in
[ADR-0001: Engine Boundaries](../../docs/ADR/ADR-0001-engine-boundaries.md):

- `tokimu-core` owns simulation truth
- rendering must not mutate simulation state
- browser or windowing glue must not own the game rules
- TypeScript should author browser-side behavior, HUD state, and presentation
	glue, not a second runtime
- browser HUD can observe and present FPS state without becoming the simulation
- browser shell may receive Rust-owned frame snapshots through a typed bridge

## Language Split

- Rust owns simulation, movement, hit detection, timing, and rendering
- TypeScript owns the browser shell, HUD, and future web-facing presentation
	glue
- The browser shell should remain a consumer of Tokimu-owned state, not a place
	where world truth silently lives

## Input Mapping

- **W/A/S/D**: move on the ground plane
- **Mouse / cursor**: aim the camera
- **Left click**: fire a projectile
- **Escape**: close or exit when supported by the host

## Visual Style

Use simple procedural cubes, a floor plane, and a small number of targets.
Keep the scene readable and avoid visual clutter so the example remains a
clean starter rather than a content showcase.

## Next Steps After MVP

Once the starter is proven, Tokimu can layer on more deliberate Quake 2-style
features:

1. movement smoothing and jump/crouch behavior
2. sound effects and a very small weapon loop
3. enemy movement or target spawning variety
4. level chunks, item pickups, or door triggers
5. browser polish for a real hosted web build

## Local Web Hosting

Run the browser shell through the example's local server, not directly from
`file://`.

- `npm start` in `examples/hello-fps-web/web` builds the TypeScript bundle,
	starts the Rust example, and serves the shell on `http://127.0.0.1:4173`
- the Rust process writes live frame snapshots to `web/live-frame.json`
- the browser shell polls that file and switches from the demo preview to the
	Rust feed as soon as the first snapshot arrives
- opening `index.html` directly from disk will fail for module loading because
	the browser treats `file://` as an isolated origin

## Architectural Assertions

This example demonstrates:

- Rust owns simulation truth.
- Browser shell consumes state.
- TypeScript does not own gameplay.
- Presentation may evolve independently.
- Procedural content is sufficient to validate runtime behavior.

## References

- [Tokimu Software Design Document](../../docs/Tokimu%20Software%20Design%20Document.md)
- [Tokimu Future Workspace Layout](../../docs/future-workspace-layout.md)
- [Tokimu Contribution Admission Guide](../../docs/contribution-admission-guide.md)
