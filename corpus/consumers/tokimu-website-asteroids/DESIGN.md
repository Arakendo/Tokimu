# Tokimu Website Asteroids Consumer Corpus

## Purpose

`tokimu-website-asteroids` is a browser consumer corpus for a polished,
playable Tokimu game embedded as an optional island on the public website.

It evolves the mechanics proven by `corpus/focused/simulation/hello-asteroids` into a bounded
Rust/WASM consumer with richer presentation evidence:

- score, lives, waves, combo scoring, and game-over flow;
- deterministic asteroid movement, fragmentation, and collisions;
- thrust trails, impact particles, explosions, and screen shake;
- keyboard and pointer input;
- a responsive Canvas host and accessible textual HUD; and
- explicit activation, pause, reset, failure, and release behavior.

This is a consumer corpus, not a new engine subsystem.

## Primary Composition Claim

Can a static Tokimu website host a complete game whose truth remains in Rust
while TypeScript and Canvas provide browser mechanisms and presentation?

```text
keyboard / pointer
        |
        v
TypeScript input adapter
        |
        v
Rust/WASM AsteroidsSession
        |
        v
provider-neutral game snapshot
        |
        v
TypeScript HUD + Canvas presentation
```

At no point does TypeScript own score, collision, lives, waves, fragmentation,
or game-over semantics.

## Relationship To `hello-asteroids`

The native corpus remains the original proof of continuous 2D movement,
shooting, wrapping, collisions, and fragmentation.

This consumer reuses those behaviors as requirements rather than copying its
native renderer:

- `hello-asteroids` proves the native engine/application composition.
- this project proves a browser consumer and public website composition.
- repeated simulation needs may later justify extracting a shared semantic
  game library, but the scaffold does not assume that admission.

## Ownership

### Rust/WASM owns

- deterministic game state and time progression;
- player, projectile, asteroid, and particle lifecycles;
- collision, fragmentation, score, combo, lives, and waves;
- bounded snapshots and diagnostics;
- seeded randomness; and
- pause, restart, and game-over transitions.

### TypeScript owns

- browser keyboard, pointer, visibility, resize, and animation mechanisms;
- mapping browser input into semantic input frames;
- fixed-step accumulation around the WASM session;
- accessible HUD and lifecycle reporting;
- Canvas drawing, gradients, trails, glow, and other pixel effects; and
- mounting and releasing the website island.

### The website owns

- durable documentation and static fallback content;
- explicit island activation;
- generated WASM and JavaScript assets; and
- public maturity and limitation labels.

### Canvas owns

- pixels, not game meaning.

The Canvas host may choose how an explosion looks. It must not decide whether
an asteroid was destroyed or how many points it was worth.

## Particle Integration

The Rust/WASM game consumes the incubating
`corpus/lib/particle-tools` mechanics for seeded spawning, fixed-step
integration, expiration, capacity, and provider-neutral visible instances.
Asteroids retains ownership of:

- thrust, muzzle flash, impact, ship-destruction debris, wave-pulse, and
  score-confirmation effect identity;
- the game events that request each effect;
- role translation at the WASM snapshot boundary; and
- Canvas color, opacity, compositing, and reduced-motion treatment.

TypeScript receives normalized particle age and does not integrate, expire, or
reroll particles.

Pointer Events provide the browser mechanism for desktop aiming and touch
aim-and-fire. They remain presentation input only: the Rust session validates
and applies the resulting semantic input frame. The browser host exposes a
disposal entry point that cancels its animation frame, aborts listeners, and
frees the WASM session for future island lifecycle integration.

## WASM Contract

The first bounded API is intentionally small:

```text
AsteroidsSession(seed)

set_viewport(width, height)
step(input_json, delta_seconds) -> snapshot_json
snapshot() -> snapshot_json
reset(seed) -> snapshot_json
```

Snapshots are provider-neutral observations. They expose positions, radii,
angles, colors, intensity, score, lives, wave, combo, mode, and screen-shake
strength. They do not expose DOM, Canvas, WebGL, GPU, or parser objects.

## Determinism

- Randomness is seeded and owned by the Rust session.
- Simulation advances using bounded fixed steps.
- Browser frame time is never accepted as an unbounded simulation step.
- The same seed and semantic input sequence should produce the same snapshot.
- Presentation-only randomness, if added, must not alter simulation truth.

## Presentation Direction

The visual language should feel like a technical instrument under stress
rather than a nostalgic arcade clone:

- near-black field with subtle grid or star depth;
- icy cyan ship and projectile energy;
- warm amber impact and scoring accents;
- restrained bloom and additive trails;
- short, meaningful screen shake;
- clear wave and game-over transitions; and
- readable score/lives/status at desktop and mobile sizes.

Reduced-motion mode must suppress shake and reduce particle motion without
changing game outcomes.

## Accessibility

- The static page explains controls and current maturity without WASM.
- Activation, pause, restart, and reset are keyboard accessible.
- Score, lives, wave, combo, pause, and game-over states are textual.
- Canvas has an accessible name and does not carry the only status signal.
- Focus is visible.
- Reduced-motion preferences are honored by the presentation adapter.
- Touch controls are a planned first-release requirement, not a silent
  desktop-only assumption.

## Performance Budgets

Initial values are corpus budgets, not universal engine guarantees:

- fixed simulation step: 1/120 second;
- maximum accumulated simulation work per animation frame: 8 steps;
- active entities remain bounded;
- snapshot JSON remains below 256 KiB during normal play;
- no per-frame WASM module initialization;
- no listeners or animation frames survive release; and
- presentation diagnostics report sustained budget misses rather than hiding
  them.

Binary and JavaScript payload budgets will be recorded after the first website
build establishes honest measurements.

## Security And Privacy

- The game requires no network access after its static assets load.
- No user data or score is transmitted.
- High score is session-local until a separate storage policy is admitted.
- The WASM boundary accepts bounded JSON input and rejects malformed values.

## Scaffold Layout

```text
tokimu-website-asteroids/
  DESIGN.md
  README.md
  build.ps1
  package.json
  tsconfig.json
  engine/
    Cargo.toml
    src/lib.rs
  web/
    index.html
    asteroids.ts
    styles.css
```

`dist/` is generated and ignored.

## Implementation Slices

### Slice 1: Compileable Consumer Template

Deliverables:

- [x] Create the Rust `cdylib`/`rlib` engine package.
- [x] Add a bounded WASM session API.
- [x] Add deterministic host-side tests.
- [x] Create a strict TypeScript/Canvas host.
- [x] Add a standalone build script and static harness.

Acceptance criteria:

- The Rust package compiles on the host.
- Equal seeds and input sequences produce equal snapshots.
- TypeScript type-checks without application dependencies.
- Generated files remain outside source control.

### Slice 2: Website Island Integration

Deliverables:

- [x] Register an `asteroids-game` island with the shared lifecycle loader.
- [x] Add a website page with durable static fallback content.
- [x] Extend the interactive build to publish generated bindings.
- [x] Record payload sizes and lifecycle evidence.

Acceptance criteria:

- The site remains useful with JavaScript disabled.
- Activation initializes WASM once.
- Reset and navigation release every listener and animation frame.
- Failure leaves the static explanation available.

Implementation evidence:

- The committed website payload is approximately 201 KiB across the standalone
  document, styles, TypeScript host, generated bindings, and Rust/WASM module.
- The shared website island controller owns explicit activation and page/reset
  teardown; the adapter releases the embedded browsing context by navigating it
  to `about:blank` before removal.
- The static homepage explanation remains visible until activation succeeds and
  is restored after reset.

### Slice 3: Gameplay And Effects Pass

Deliverables:

- [ ] Tune movement, spawning, collision, and wave pacing.
- [ ] Add semantic effect events where snapshots alone are insufficient.
- [ ] Add touch controls.
- [ ] Add reduced-motion behavior.
- [ ] Add sound only after an audio ownership boundary exists.

Acceptance criteria:

- Score, lives, waves, combo, pause, restart, and game over are complete.
- Effects improve feedback without owning game outcomes.
- Desktop keyboard/pointer and touch play are both usable.
- No unbounded entity or effect growth occurs.

### Slice 4: Corpus Evidence

Deliverables:

- [ ] Add deterministic replay fixtures.
- [ ] Capture structural snapshot evidence.
- [ ] Capture separately labeled browser screenshots.
- [ ] Record supported-browser and resize observations.
- [ ] Compare native and WASM semantics where the two corpora overlap.

Acceptance criteria:

- Regressions localize to simulation, boundary, or presentation ownership.
- Screenshots complement rather than replace structural assertions.
- Browser-specific limitations are explicit.

## Non-Goals

The first version does not attempt to provide:

- multiplayer or online scoreboards;
- a general game framework;
- a general particle engine;
- a shared audio subsystem;
- persistence;
- renderer admission based on one browser game; or
- automatic extraction of `hello-asteroids` into a shared crate.

## Graduation Evidence

This project may pressure reusable capabilities, but nothing graduates merely
because the game needs it. Promotion requires repeated independent consumers,
stable provider-neutral semantics, and the Architectural Review process
described by Tokimu governance.

## References

- [`hello-asteroids`](../../focused/simulation/hello-asteroids/DESIGN.md)
- [Website consumer corpus](../tokimu-website/DESIGN.md)
- [Interactive island contract](../../../website/docs/lab/island-contract.md)
- [Tokimu website plan](../../../docs/Plans/Standalone/tokimu-website.md)
- [Particle simulation and presentation plan](../../../docs/Plans/Standalone/particle-simulation-and-presentation.md)
- [ADR-0001: Engine Boundaries](../../../docs/ADR/ADR-0001-engine-boundaries.md)
- [ADR-0007: Kernel Performance Diagnostics](../../../docs/ADR/ADR-0007-kernel-performance-diagnostics.md)
