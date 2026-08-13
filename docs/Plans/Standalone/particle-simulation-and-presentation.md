# Particle Simulation And Presentation

## Status

In progress. The bounded mechanics, focused corpus, provider-neutral lowering,
and first Asteroids consumer are implemented. Deterministic state and emitter
semantics incubate in `particle-tools`, and `hello-particles` is their first
visual consumer. Remaining work is evidence gathering: separately labeled
native captures, workload measurements, and a second independent consumer.
No particle capability, asset format, renderer contract, or TypeScript
authoring API is admitted by this plan.

The first implementation incubates under `corpus/lib/particle-tools` and is
proved by a focused `corpus/focused/simulation/hello-particles` application before the Asteroids
consumer adopts it.

## Purpose

Tokimu needs a bounded way to create, simulate, observe, and present short-lived
particle effects without making particles a hidden renderer feature.

The immediate consumer is the website Asteroids game:

- ship thrust;
- projectile impacts;
- asteroid fragmentation;
- ship-destruction debris;
- wave pulse and score feedback; and
- later presentation treatments such as trails, flashes, and debris.

Particles also put useful pressure on simulation, execution, resources,
rendering, authoring, diagnostics, and native/WASM parity. That pressure is
evidence to collect, not permission to combine those responsibilities into one
large subsystem.

## Architectural Thesis

> Applications own why an effect occurs. Emitters own spawning. Particle
> simulation owns particle state and lifetime. Presentation owns appearance.
> Renderers own pixels.

```text
application signal or command
            |
            v
bounded emitter request
            |
            v
deterministic particle simulation
            |
            v
provider-neutral visible instances
            |
            v
presentation lowering
            |
            v
renderer
```

The renderer never needs to know that an asteroid exploded. The application
never needs to manufacture renderer-native instances.

## Governing Boundaries

- [`Tokimu Software Design Document.md`](../../Tokimu%20Software%20Design%20Document.md)
  keeps simulation truth separate from presentation and treats signals as
  first-class coordination surfaces.
- [`ADR-0001-engine-boundaries.md`](../../ADR/ADR-0001-engine-boundaries.md)
  keeps rendering outside simulation ownership.
- [`ADR-0003-capability-ownership-boundary.md`](../../ADR/ADR-0003-capability-ownership-boundary.md)
  separates Tokimu-owned semantics from replaceable providers.
- [`ADR-0006-native-execution-policy.md`](../../ADR/ADR-0006-native-execution-policy.md)
  permits execution policy to exploit independent work without admitting
  parallel `World` mutation.
- [`ADR-0007-kernel-performance-diagnostics.md`](../../ADR/ADR-0007-kernel-performance-diagnostics.md)
  provides bounded performance observation without turning particle policy
  into kernel policy.

If corpus evidence justifies a new native capability or changes an accepted
ownership boundary, open an Architectural Review before extracting a permanent
crate.

## Current Evidence

`corpus/consumers/tokimu-website-asteroids` began with a useful local proof and
now provides the first integrated consumer evidence:

- seeded particle spawning;
- application-owned thrust, impact, ship-destruction, muzzle, wave, and score
  roles;
- position, velocity, drag, lifetime, and size;
- a fixed maximum of 320 active particles;
- deterministic fixed-step updates;
- WASM snapshot serialization; and
- TypeScript Canvas presentation with color and opacity derived from particle
  role and normalized lifetime;
- touch input that maps to the same aim-and-fire semantic action as pointer
  input; and
- lifecycle-safe browser cleanup for input listeners, pointer capture,
  animation frames, and the WASM session.

That implementation proved the game needs particles. The consumer now reuses
the incubating `particle-tools` mechanics while retaining Asteroids'
`ParticleKind`, effect requests, colors, and WASM snapshot translation as
application-owned semantics.

The migration confirms the first extraction pressure:

```text
Reusable mechanics
    spawn bounds
    seeded variation
    integration
    drag
    lifetime expiration
    active-count bounds

Asteroids meaning
    thrust
    impact
    ship destruction
    toroidal wrapping
    game-specific colors
```

The plan must separate these rather than moving the current game structs
unchanged into a shared library.

## Ownership

### Application and game rules own

- the event or rule that requests an effect;
- effect identity such as `ship-destroyed` or `asteroid-impact`;
- inherited position, direction, velocity, intensity, and presentation role;
- whether an effect is cosmetic or participates in gameplay; and
- score, damage, collision, audio, and screen-shake outcomes.

The first particle corpus supports cosmetic particles only. Particle state must
not feed back into Asteroids collision or scoring.

### Emitter semantics own

- burst and rate-based spawning;
- deterministic range sampling;
- initial position and velocity distribution;
- initial lifetime, size, rotation, and other admitted values;
- the maximum active-particle policy; and
- explicit behavior when capacity is exhausted.

The first bounded overflow policy is `drop-newest`: existing particles retain
their deterministic lifetimes and excess spawn requests are counted and
diagnosed. Additional policies require evidence.

### Particle simulation owns

- active particle identity and state;
- fixed-step integration;
- age and lifetime expiration;
- acceleration and bounded drag;
- deterministic update order;
- active and expired counts; and
- provider-neutral snapshots.

Particle simulation does not own GPU buffers, Canvas objects, textures, blend
state, application events, ECS queries, or ambient randomness.

### Presentation owns

- mapping an application effect role and normalized particle age into visible
  color, opacity, scale, trail, glow, or shape intent;
- reduced-motion treatment;
- visibility and culling decisions that do not alter game outcomes; and
- lowering visible particle instances into existing vector, mesh, sprite, or
  renderer-facing commands.

The initial proof uses simple untextured 2D shapes. Texture atlases,
billboarding, blend modes, and 3D presentation remain later evidence.

### Renderer and platform providers own

- GPU buffers, instance buffers, batching, uploads, draw submission, and
  backend limits;
- Canvas or native-window mechanisms;
- shader execution and blend/depth state; and
- GPU-specific performance measurements.

Providers may optimize particle execution without becoming owners of emitter or
particle meaning.

### Resources and authoring own

Nothing in the first slice.

An immutable particle definition may later become an asset or TypeScript-authored
resource after file, embedded, and generated definitions demonstrate the same
contract. The first implementation uses ordinary Rust values and does not
invent a particle file format.

## First Bounded Model

The provisional incubation vocabulary is deliberately 2D and CPU simulated:

```rust
ParticleSystem2d
ParticleEmitter2d
ParticleSpawn2d
ParticleState2d
ParticleInstance2d
ParticleStepReport
```

Candidate state:

```text
ParticleState2d
    stable id
    position
    velocity
    acceleration
    age
    lifetime
    initial size
    final size
    rotation
    angular velocity
    presentation role
```

Candidate spawn controls:

```text
ParticleSpawn2d
    count
    origin
    inherited velocity
    direction range
    speed range
    lifetime range
    size range
    acceleration
    drag
    presentation role
```

These names are provisional. The corpus should simplify or replace them when
implementation evidence shows a smaller honest vocabulary.

The first model does not generalize over vector dimensions. A later 3D consumer
must demonstrate whether 2D and 3D share one semantic contract or merely share
internal algorithms.

## Determinism And Time

- The caller supplies a seed or deterministic random source.
- Ambient or platform randomness is forbidden.
- Simulation advances through explicit fixed steps.
- Spawn and update order are stable.
- Particle IDs do not depend on memory addresses or parallel completion order.
- Equal definitions, seeds, commands, and steps produce equal structural
  snapshots.
- Sequential execution remains the semantic reference implementation.
- Parallel updates are considered only after a measured workload proves useful
  granularity and deterministic ordered commit.

## Bounds And Failure Semantics

Every system has explicit limits:

- maximum active particles;
- maximum particles requested by one burst;
- maximum accepted lifetime;
- finite position, velocity, acceleration, size, rotation, and drag;
- bounded diagnostics and snapshot size; and
- no unbounded retained history.

Invalid definitions and spawn requests fail before partial mutation. Capacity
pressure reports requested, admitted, and dropped counts. Unsupported rendering
features are diagnosed rather than silently approximated as a different
semantic effect.

## Native And WASM Contract

Native and WASM consumers use the same particle semantics.

The browser boundary may expose bounded visible instances or semantic effect
events:

```text
Rust particle state
        |
        v
bounded provider-neutral snapshot
        |
        v
TypeScript presentation adapter
```

TypeScript may choose Canvas drawing details. It must not resimulate particles,
reroll spawn values, expire particles, or redefine overflow behavior.

JSON is acceptable for the first bounded corpus. Snapshot size, serialization
time, and allocation pressure must be measured before treating it as the
long-term high-count transport.

## Implementation Slices

### Slice 0: Boundary And Baseline

Deliverables:

- [ ] Record the current Asteroids particle fields, algorithms, entity counts,
      snapshot bytes, and update/serialization timings.
- [x] Classify every current behavior as reusable mechanics, application
      meaning, presentation, or provider mechanism.
- [x] Define synthetic burst, continuous emission, expiration, capacity, and
      determinism cases.
- [x] Record explicitly unsupported 3D, texture, collision, GPU-simulation, and
      authoring behavior.

Acceptance criteria:

- [ ] The baseline can be reproduced without visual inspection.
- [x] No Asteroids-specific effect kind appears in the proposed shared
      mechanics.
- [x] Every proposed public value has one named owner.
- [ ] The first corpus can fail independently for spawning, simulation,
      snapshot lowering, and presentation.

### Slice 1: Deterministic Particle State

Deliverables:

- [x] Create `corpus/lib/particle-tools`.
- [x] Add finite validated 2D particle state and system configuration.
- [x] Implement stable identity, fixed-step integration, drag, acceleration,
      age, and lifetime expiration.
- [x] Implement explicit capacity and `drop-newest` behavior.
- [x] Return structured spawn and step reports with active, spawned, expired,
      and dropped counts.
- [x] Add headless unit tests.

Acceptance criteria:

- [x] Equal seeds and step sequences produce byte-equivalent structural
      snapshots.
- [x] Expiration and overflow order are deterministic.
- [x] Invalid inputs leave the system unchanged.
- [x] Tests require no window, renderer, filesystem, network, or WASM runtime.

### Slice 2: Emitter Semantics

Deliverables:

- [x] Add burst emission.
- [x] Add fixed-rate emission with deterministic fractional accumulation.
- [x] Add bounded scalar and angular ranges.
- [x] Add inherited velocity and directional spread.
- [x] Keep random sampling independent from presentation and providers.
- [x] Define reset, reseed, caller-owned pause, and emitter-disable behavior.

Acceptance criteria:

- [x] Spawn counts remain stable across equivalent fixed-step partitions.
- [x] Burst and rate emitters share particle-state output without sharing
      application meaning.
- [x] Pausing does not consume time or random samples.
- [x] Disabled emitters produce no hidden work.

### Slice 3: Provider-Neutral Presentation Instances

Deliverables:

- [x] Lower active state into bounded `ParticleInstance2d` observations.
- [x] Include normalized age and application-owned presentation role.
- [x] Keep color ramps, glow, texture handles, and shader objects outside
      particle simulation.
- [x] Add deterministic filtering and visible-count diagnostics.
- [x] Measure structural snapshot size and lowering duration.

Acceptance criteria:

- [x] One particle snapshot can feed a native presentation and the browser
      Canvas host without changing simulation.
- [x] Reduced-motion presentation does not change score, spawning, expiration,
      or deterministic state.
- [x] No DOM, Canvas, wgpu, shader, mesh, or texture type enters particle
      simulation.

### Slice 4: `hello-particles` Corpus

Deliverables:

- [x] Create `corpus/focused/simulation/hello-particles` with a focused `DESIGN.md`.
- [x] Present burst, stream, directional spray, drag, acceleration, expiration,
      pause, reset, and capacity-pressure cases.
- [x] Add a deterministic seed selector or fixed case sequence.
- [x] Emit structural artifacts for each case.
- [ ] Capture separately labeled native visual evidence.

Acceptance criteria:

- [x] The example proves particle mechanics without Asteroids.
- [x] Every visible case maps to a structural assertion.
- [x] The application contains no duplicate integration or range-sampling
      implementation outside `particle-tools`.
- [x] Capacity pressure remains responsive and produces explicit diagnostics.

### Slice 5: Asteroids Consumer Integration

Deliverables:

- [x] Replace reusable local particle mechanics in
      `tokimu-website-asteroids` with the incubating library.
- [x] Keep thrust, impact, and ship-destruction meaning in the game.
- [x] Map game effect roles into browser presentation styles.
- [x] Preserve current seeded gameplay determinism.
- [x] Add stronger explosion, debris, thrust-trail, and wave-feedback effects.
- [x] Add touch behavior required by the Asteroids design.
- [x] Add presentation-only reduced-motion particle treatment.
- [x] Add lifecycle-safe host disposal for input listeners, pointer capture,
      animation frames, and the WASM particle session.

Acceptance criteria:

- [x] Score, collision, fragmentation, lives, and waves are unchanged by
      particle presentation.
- [x] TypeScript does not simulate or expire particles.
- [x] No active particle or serialized snapshot grows without a configured
      bound.
- [ ] Capture manual desktop and mobile usability evidence. Responsive layout
      and touch semantics are implemented, but a device-level corpus capture is
      still required before this is a verified claim.
- [x] Release removes listeners, animation frames, and particle-session state.

Implementation observation:

- Asteroids now requests bounded muzzle flashes, speed-sensitive thrust,
  asteroid impacts, score sparks, ship-debris rings, and wave-start pulses
  through application-owned roles. The shared library remains unaware of game
  events, colors, and Canvas compositing.
- The browser host exposes `disposeAsteroids()` and releases its owned browser
  mechanisms without making lifecycle or event ownership part of
  `particle-tools`.

### Slice 6: Performance And Execution Evidence

Deliverables:

- [ ] Measure update, spawn, expiration, lowering, serialization, upload, and
      draw costs at small, medium, and capacity workloads.
- [ ] Record debug and release profiles separately.
- [ ] Add sustained performance budgets through existing diagnostics.
- [ ] Compare individual draws with retained or instanced presentation.
- [ ] Record sequential execution as the semantic baseline.
- [ ] Prototype parallel CPU updates only if measurement demonstrates useful
      granularity.

Acceptance criteria:

- [ ] Metrics name the measured stage and do not infer GPU completion.
- [ ] A static or paused system performs no hidden spawn/update/upload churn.
- [ ] Capacity workloads remain bounded and diagnosable.
- [ ] Any optimized path produces the same ordered structural state as the
      sequential path.

Implementation observation:

- `hello-particles` emits a lowering-duration observation alongside its
  structural artifacts. This establishes a useful local measurement seam, but
  it is not yet a workload profile, performance budget, or renderer-cost
  contract.

### Slice 7: Definitions, Resources, And Authoring Evidence

Deliverables:

- [ ] Add a second non-game consumer that loads or constructs reusable particle
      definitions.
- [ ] Compare embedded Rust, generated, and file-backed definition needs.
- [ ] Decide whether a definition is an asset, plain application data, or a
      separate provider-owned format.
- [ ] Prototype TypeScript authoring only after the Rust semantic model has two
      independent consumers.
- [ ] Keep authoring one-way into Tokimu-owned validated data.

Acceptance criteria:

- [ ] No file syntax or TypeScript object becomes the canonical runtime model.
- [ ] Provider failure and unsupported fields produce deterministic
      diagnostics.
- [ ] A definition can be inspected and validated headlessly.
- [ ] Asset admission is based on lifecycle evidence rather than convenience.

### Slice 8: Architectural Review And Disposition

Deliverables:

- [ ] Summarize evidence from `hello-particles`, Asteroids, and any second
      independent consumer.
- [ ] Decide whether particle semantics warrant a native Tokimu capability.
- [ ] Decide whether 2D and 3D particles share a contract.
- [ ] Decide whether particle definitions belong to assets.
- [ ] Decide whether visible particle instances need a renderer-facing batch
      contract.
- [ ] Accept, defer, reject, or continue incubation through an Architectural
      Review.

Acceptance criteria:

- [ ] The disposition names accepted ownership and explicit reopening triggers.
- [ ] Application, simulation, presentation, and provider responsibilities
      remain structurally separate.
- [ ] No permanent crate is extracted solely because Asteroids needs effects.
- [ ] Documentation and implementation report the same maturity.

## Asteroids Evidence Status

The first gameplay-facing presentation treatments are now implemented:

- [x] consistent thrust stream with speed-sensitive size and rate;
- [x] asteroid-impact burst with inherited momentum;
- [x] ship-destruction burst with a short-lived debris ring;
- [x] projectile muzzle flash;
- [x] wave-start field pulse;
- [x] combo and score-confirmation sparks; and
- [x] reduced-motion presentation treatment that leaves simulation unchanged.

Next evidence priorities are native and browser visual captures, manual mobile
input validation, and measured costs before decorative ambient particles or
additional effects are considered.

## Non-Goals

The initial effort does not provide:

- GPU particle simulation;
- particle collision, damage, or physics;
- 3D billboards or volumetric effects;
- arbitrary behavior graphs or scripting;
- a particle file format or editor;
- texture atlas ownership;
- audio or screen-shake ownership;
- a universal curve, gradient, or animation system;
- transparent-render sorting guarantees;
- a general job system; or
- automatic promotion into `tokimu-core`, `tokimu-runtime`, or `tokimu-render`.

## Risks

### Asteroids Vocabulary Becomes The Shared API

Mitigation: prove the mechanics first in `hello-particles`; retain game effect
identity in the game.

### Particles Become Renderer-Owned GPU Magic

Mitigation: keep deterministic CPU state and headless structural tests as the
reference semantics.

### Cosmetic Effects Mutate Game Outcomes

Mitigation: use one-way application commands into the particle system and
forbid particle state from Asteroids collision and scoring.

### Snapshot JSON Becomes The Bottleneck

Mitigation: measure bytes and serialization cost before adding a binary or
shared-memory transport.

### Generality Arrives Before Evidence

Mitigation: begin with concrete 2D scalar ranges and simple presentation roles;
defer 3D, assets, authoring, textures, and GPU simulation.

### Capacity Pressure Produces Visual Or Diagnostic Chaos

Mitigation: use deterministic `drop-newest`, bounded transition diagnostics,
and corpus cases at and beyond capacity.

## Completion Criteria

This plan is complete when:

- deterministic particle mechanics are independently proved by
  `hello-particles`;
- Asteroids consumes the shared incubating mechanics without moving game
  meaning into the library;
- native and WASM consumers preserve the same semantic state;
- presentation and renderer providers remain replaceable;
- structural, visual, and performance evidence is recorded;
- unsupported behavior and bounds are explicit; and
- an Architectural Review records whether the implementation graduates,
  remains incubating, or is rejected.

## References

- [`On Particles.md`](../../Conversations/On%20Particles.md)
- [`hello-asteroids`](../../../corpus/focused/simulation/hello-asteroids/DESIGN.md)
- [`tokimu-website-asteroids`](../../../corpus/consumers/tokimu-website-asteroids/DESIGN.md)
- [`Native Execution and Multithreading`](native-execution-and-multithreading.md)
- [`Performance Diagnostics and Runtime Observation`](performance-diagnostics-and-runtime-observation.md)
- [`TypeScript Shader, Material, And Presentation Control`](typescript-shader-material-presentation-control.md)
