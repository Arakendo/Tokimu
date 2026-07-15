# Tokimu Capability Backends

| Field      | Value                                                                                        |
| ---------- | -------------------------------------------------------------------------------------------- |
| Status     | Draft — exploratory, grow-then-fold                                                           |
| Title      | "Capability Backends" (kept) — emphasizes what is supplied and how, over mechanism           |
| Scope      | A general, pluggable way to add optional heavy libraries (geometry/CAD kernels, databases, others) to Tokimu behind engine-owned semantic models. Not a committed subsystem yet. |
| Relates to | ADR-0001 Engine Boundaries, SDD ownership model, Tokimu TypeScript Design Document (authoring surface) |
| Relates to | `docs/diagnostics-model.md` (diagnostic quality expectations)                                 |

## 1. Purpose

Tokimu's core stays small and engine-neutral. But applications built on Tokimu
will reach for heavy, specialized libraries that the engine should not absorb:
a CAD geometry kernel, a database, a physics solver, an audio engine, an asset
importer, a scripting runtime, and things nobody has asked for yet.

We do not know what people will use Tokimu for. So the goal is not to pick those
libraries — it is to define a **pluggable mechanism** for adding them, where the
external library is a swappable **backend** behind a **Tokimu-owned semantic
model**, and never becomes a hidden owner of engine state.

This is a side document on purpose. It captures the general pattern and a couple
of worked examples (CAD geometry, persistence) so the mechanism can grow against
a real first slice and then fold into the SDD once the shape stabilizes.

The central claim: Tokimu should own the *meaning* of a capability, and treat
concrete libraries as **replaceable backends selected through a capability
registry**, executed on the Rust side, with authoring surfaces only describing
intent.

## 2. Design Constraints

These follow from the existing architecture and must not be violated.

- `tokimu-core` stays engine-neutral. No capability library — no CAD kernel, no
  database driver, no solver — leaks into core or runtime.
- Each capability lives in its own crate (e.g. `tokimu-geometry`,
  `tokimu-persistence`), outside `tokimu-core` and `tokimu-runtime`, the same
  way ADR-0001 keeps persistence separate.
- Capabilities are optional. Core and runtime must compile and run without any
  of them present. Depending on a capability is a decision an application makes,
  not a default the engine imposes.
- Adapters *observe and serve*; they do not become hidden owners of engine
  state. Mirror of "renderer consumes world state but does not own it."
- Authoring surfaces (including TypeScript) describe capability *meaning*; Rust
  owns semantics and execution. No foreign backend object (a kernel handle, a DB
  connection, a solver context) is ever exposed to the author API.
- Prefer explicit diagnostics over silent fallback (SDD guiding principle, and
  `docs/diagnostics-model.md`). A capability that cannot satisfy a request must
  say why, not degrade quietly.
- Native/WASM parity is a goal, not a day-one requirement. Because the semantic
  model is Tokimu-owned, backend parity can arrive later without rewriting the
  authoring surface.

## 3. Core Invariants

These four invariants close the biggest future escape hatches. Everything else
in this document should be read as serving them.

1. **Registry ownership.** The capability registry is application-owned — a
   collection of registered providers and their descriptors held by the
   application/runtime composition root. It is not global state, not a hidden
   singleton, and does not own engine truth.
2. **Deterministic resolution.** The same request, against the same registry
   manifest, on the same target, selects the same provider. Registration order,
   linker order, and dependency-graph mood must not decide `auto`.
3. **Typed capability contracts.** Shared registry metadata may be generic for
   discovery and indexing, but the provider-facing operational interface is
   typed per capability (`GeometryProvider`, `PersistenceProvider`, …). Authors
   asking for a "transactional store" get a typed contract, not a boolean map.
4. **No foreign object leakage.** Even backend-specific extensions expose only
   Tokimu-owned types. A raw `TopoDS_Shape` or a SQLite handle never crosses
   into engine-owned or author-facing APIs.

## 4. The General Shape

Every pluggable capability has the same anatomy:

```
Authoring surface (describes intent)
        ↓
Tokimu-owned semantic model  (what the capability means)
        ↓
Capability provider trait    (Rust-owned interface)
        ↓
Selected backend             (external library adapter)
        ↓
Result mapped back into Tokimu-owned data
```

The parts Tokimu owns and keeps stable:

- **Semantic model** — the vocabulary of the capability (profiles/solids for CAD,
  records/queries for a store), expressed in Tokimu types.
- **Provider trait** — the operations a backend must implement.
- **Capability descriptor** — what a given backend can honestly do.
- **Selection/resolution** — how a backend is chosen for a request.
- **Diagnostics** — one vocabulary for "cannot do that, because…".
- **Lifecycle/ownership** — who creates, holds, and tears down the backend.

The part that is swappable: the concrete library behind the provider trait.

## 5. Ownership Boundary — Native vs Capability vs Backend

Not everything that "many applications may need" belongs in core. Left
unchecked, everything eventually grows a coffee machine bolted to it. Three
tiers keep that honest:

> Native Tokimu owns universal meaning. Capability crates own domain meaning.
> Backends own specialized execution.

### Native Tokimu responsibilities

These belong in Tokimu because nearly every higher-level application depends on
them and they define the engine's worldview:

- world / entity / component / resource model, and relationships
- schedules and time
- commands, signals, and events
- normalized input
- diagnostics
- asset identity and ownership
- platform abstraction
- presentation / render abstraction
- serialization and reflection primitives
- stable names and IDs
- capability discovery and resolution contracts

Tokimu natively understands:

```
Entity  Component  Relation  Resource  Rule  Signal
Command  Time  Asset  View  Diagnostic  Capability
```

It does not natively understand:

```
STEP file  SQL transaction  NavMesh agent
RigidBody solver  NURBS surface  Quest dialogue tree
```

Those belong to capability crates or application layers.

### Native semantic model, optional implementation

A useful middle category: things Tokimu defines *semantically* but does not
implement deeply. The distinction that matters is **Tokimu-owned**, not
necessarily **core-owned** — a Tokimu-owned type can live in a capability crate.

- **Geometry.** Tokimu may natively define lightweight `Mesh`, vertex
  attributes, `Transform`, bounds, `Ray`, `Plane`. Heavier `Profile`,
  `SolidHandle`, and `GeometryDiagnostic` likely belong in `tokimu-geometry`,
  not `tokimu-core`. Robust `offset`/`loft`/`fillet`/STEP/B-rep boolean live
  behind backends.
- **Persistence.** Tokimu natively owns enough reflection/serialization hooks to
  describe world-facing data. Databases, migrations, SQLite, IndexedDB live in
  `tokimu-persistence` backends.
- **Scripting.** Tokimu owns rule semantics, command/query/signal APIs,
  execution mode, diagnostics, and host capability contracts. QuickJS/Boa/V8
  stay optional backend crates.
- **Physics.** Tokimu may own transforms, velocities, collision shapes as data,
  collision events, and query contracts — but not a rigid-body solver. Rapier,
  Jolt, or PhysX implement a physics capability later.

### The native baseline ("standard library")

Enough for Snake, Pac-Man, Asteroids, simple editors, and technical simulations
without summoning an industrial CAD kernel merely to decide whether two
rectangles overlap:

- **Core runtime:** ECS/world, resources, relations, schedules, fixed/variable
  time, a deterministic RNG resource, commands/signals/events, diagnostics,
  reflection/type registry.
- **Spatial foundation:** `Vec2/3/4`, matrices and quaternions, `Transform2/3`,
  bounds, rays, planes, basic intersection tests, simple grid coordinates.
- **Rendering:** meshes, textures, materials, cameras, pipelines, render
  commands, 2D/3D placement, render-resource registries (text later).
- **Input and interaction:** actions, axes, pointer state, controller state,
  commands (focus/selection later).
- **Assets:** asset IDs and handles, provider-agnostic byte loading, an importer
  interface, source metadata, lifecycle/diagnostics.
- **Authoring substrate:** stable names/IDs, semantic rule model, scene document
  model, component/resource registry, query model, serialization/reflection,
  versioned diagnostics.
- **Capability framework:** capability descriptors, provider registration,
  deterministic selection, lifecycle ownership, explicit backend diagnostics —
  and this one stays small. Tokimu provides the electrical outlet, not every
  appliance.

### Not native initially

Kept outside the native surface until a concrete application proves a specific
backend is worth supporting: industrial CAD kernel, database engine, full
physics solver, audio middleware, pathfinding/navmesh, scripting VM, network
protocol stack, STEP/USD/FBX importers, terrain engine, animation graph, full UI
framework, ML runtime. Any can become an *official* Tokimu capability crate
later — "official" does not mean "core."

### The practical ownership test

A concept is **native** when most of these hold:

1. nearly every Tokimu application needs it;
2. it expresses engine meaning, not one domain's meaning;
3. multiple subsystems depend on it;
4. its absence would fragment the ecosystem into incompatible vocabularies;
5. it can be implemented without a heavy specialized dependency;
6. it strengthens native/WASM parity rather than weakening it.

A concept is a **capability backend** when most of these hold:

1. only some application classes need it;
2. multiple mature external libraries already compete;
3. implementations differ by platform or precision needs;
4. the dependency is large, native, oddly licensed, or hard to build;
5. users may reasonably want different implementations;
6. it owns specialized algorithms rather than Tokimu's world truth.

### Crate rings

```
Ring 1 — engine kernel (defines Tokimu)
  tokimu-core  tokimu-runtime  tokimu-input  tokimu-assets
  tokimu-render  tokimu-platform  tokimu-rule
  (+ facade tokimu, wasm entry tokimu-wasm, authoring host tokimu-ts-frontend)

Ring 2 — official semantic capabilities (Tokimu-owned meaning)
  tokimu-persistence  tokimu-geometry  tokimu-physics
  tokimu-script-host  tokimu-net  tokimu-audio

Ring 3 — concrete backend adapters (external libraries)
  tokimu-geometry-manifold  tokimu-geometry-truck  tokimu-geometry-occt
  tokimu-persistence-sqlite  tokimu-persistence-indexeddb
  tokimu-script-boa  tokimu-script-quickjs
```

Ring 1 defines Tokimu. Ring 2 defines optional Tokimu-owned capability meaning.
Ring 3 integrates external libraries. That keeps a broad ecosystem possible
without making `cargo build tokimu` feel like assembling an aircraft carrier.

> This ownership boundary is recorded as a durable architectural rule in
> ADR-0003 (Capability Ownership Boundary). This document remains the detailed
> mechanism; the ADR is the decision.

## 6. Capability Registry

Different backends win at different things. Do not design the public API around
the lowest common denominator — that yields a capability that cannot express
what capable backends actually do. Instead:

1. a small shared core of common operations,
2. capability queries,
3. backend-specific extension surfaces when needed.

### What "registry" means here

The registry is narrow on purpose: an **application-owned collection of
registered providers and their descriptors**, held by the application/runtime
composition root. It is a resolver — not a service locator, not a plugin
discovery daemon, and not a global mutable singleton (invariant 1).

```rust
pub struct CapabilityRegistry {
    providers: HashMap<CapabilityKind, Vec<ProviderEntry>>,
}
```

### Provider identity vs. capability identity

Keep the product identity of a backend separate from its compatibility
contract, or `"manifold"` becomes both a name and a version promise and
versioning gremlins get tenure:

```
capability kind:   geometry
provider id:       manifold
provider version:  3.x
adapter version:   tokimu-manifold 0.2
target support:    native, wasm
feature set:       mesh_csg, watertight, extrusion
```

A generic descriptor carries just enough for discovery and indexing:

```rust
pub struct CapabilityDescriptor {
    pub kind: CapabilityKind,       // Geometry, Persistence, Physics, ...
    pub provider_id: &'static str,  // "manifold", "sqlite", ...
    pub provider_version: Version,
    pub adapter_version: Version,
    pub targets: TargetSupport,     // native, wasm
    pub features: FeatureSet,       // generic, for discovery only
}
```

### Typed contracts, generic envelope

The generic `FeatureSet` is fine as a discovery/indexing aid, but the
provider-facing *operational* interface is typed per capability (invariant 3):

```rust
pub trait GeometryProvider {
    fn capabilities(&self) -> GeometryCapabilities;
    // typed operations: offset_profile, extrude, loft, boolean, tessellate
}

pub trait PersistenceProvider {
    fn capabilities(&self) -> PersistenceCapabilities;
    // typed operations: put, get, query, transaction, migrate
}
```

The registry may erase providers behind a common envelope for storage, but real
use recovers the typed capability interface. Authors asking for a "transactional
store" should get a typed contract, not a boolean map with the warmth of an
airport kiosk.

### Deterministic resolution

When two providers are valid, rank them in a fixed order so `auto` never depends
on registration or linker order (invariant 2):

1. explicit author/application selection
2. target compatibility (native/wasm)
3. required features satisfied
4. application preference order
5. declared quality/performance profile
6. stable provider id (final tie-breaker)

```
same request + same registry manifest + same target → same selected provider
```

This parallels the TTSDD execution manifest: selection is reproducible, and the
chosen provider is reported as a diagnostic, not a hidden default. If nothing
satisfies the request, that is an explicit diagnostic:

```
needs:  transactional durable store + browser
error:  no registered persistence backend satisfies both
```

### Provider lifecycle classes

Not every backend is a disposable function bag. Distinguish lifecycle classes
early so a database connection and a one-shot mesh operation are not modeled the
same way:

```
StatelessProvider    — pure op, disposable (mesh conversion, one-shot import)
SessionProvider      — holds a session (CAD document, script VM)
WorldBoundProvider   — tied to a world/simulation lifetime (physics world)
ProcessBoundProvider — owns a durable external resource (database, audio device)
```

These need not be exact traits yet, but the model must not assume every provider
is stateless.

## 7. Three Levels (Per Capability)

The same layering applies to any capability, not just geometry:

```
Level 1: primitives
- the Tokimu-owned data types for the capability

Level 2: common operations
- the operations most backends can support

Level 3: backend-specific capabilities
- advanced features only some backends provide, gated by capability queries
```

Level 1 and 2 are the shared Tokimu-owned surface. Level 3 is opt-in per
backend, and bound by invariant 4: a backend-specific extension may expose
additional Tokimu-owned semantics but never raw foreign objects. An OCCT
extension may offer `fillet_edges` or `export_step`; it must not offer
`raw_shape() -> TopoDS_Shape` across an engine-owned or author-facing API.

## 8. Worked Example — Geometry / CAD

CAD-style authoring is a strong motivating target: extrude a closed loop, offset
a profile inward by `0.1`, loft one profile into another, boolean solids, keep
geometry watertight. These need a real **geometry kernel**, not `vec3` and
`mat4`. Robust offsets, extrusions, booleans, fillets, topology, and tolerances
are where geometry stops being pleasant algebra and becomes a dispute between
floating-point numbers.

Mapped onto the general shape:

- **Semantic model:** four distinct layers — a feature graph (Sketch, Offset,
  Extrude, Loft, Revolve, Sweep, Boolean), an evaluated geometry model (Wire,
  Face, Shell, Solid), a tessellation cache (regenerable at different
  tolerances), and a render mesh. An extrude should not immediately collapse
  into triangle soup if the geometry is meant to stay editable; the render mesh
  is a cacheable, disposable presentation artifact, not the canonical CAD
  result.
- **Provider trait:** `offset_profile`, `extrude`, `loft`, `boolean`,
  `tessellate`, plus optional B-rep / mesh-CSG extension surfaces.
- **Candidate backends:**
  - **Manifold** — watertight triangle-mesh CSG, robust booleans, extrusion,
    print-friendly closed output. Strong *first proof* because its output
    already resembles what the renderer wants. <https://manifoldcad.org/docs/html/>
  - **Truck** — pure-Rust CAD kernel, WASM-friendly, no C++ FFI; maturity below
    OCCT. <https://github.com/ricosjp/truck>
  - **Open CASCADE (OCCT) via Rust bindings** — serious B-rep: sweeps, lofts,
    fillets, STEP/IGES, exact surfaces; heavy native dependency, awkward WASM.
    <https://github.com/bschwind/opencascade-rs>
  - **Fornjot** — early-stage Rust B-rep kernel; research inspiration for now.
    <https://github.com/hannobraun/fornjot>
  - JS-side references for prototyping authoring ergonomics: **OpenCascade.js**
    (<https://dev.opencascade.org/project/opencascadejs>) and **JSCAD**
    (<https://openjscad.xyz/docs/>).
- **Numeric choices (must be explicit):** units, tolerance, coordinate scale,
  angle units, exact-vs-approximate, failure behavior, topology identity after
  edits. A bare `0.1` must mean something — mm, not municipal-building units.

A deeper CAD-specific design (feature-graph serialization, topology identity,
loft correspondence rules) can graduate into its own document once this general
mechanism is settled.

## 9. Worked Example — Persistence / Database

A different target with the same shape. Some applications need durable storage;
Tokimu core must not.

- **Semantic model:** capability-owned, Tokimu-defined persistence semantics —
  the `tokimu-persistence` crate owns its own records, keys, and queries. These
  are not raw driver handles, and they are not promoted into engine-wide core
  vocabulary either.
- **Provider trait:** `put`, `get`, `query`, `transaction`, `migrate`.
- **Candidate backends:** an in-memory store (default/test), a file-backed store,
  an embedded database (e.g. SQLite) for native, and a browser-appropriate store
  (e.g. IndexedDB) for WASM — selected by capability, not hard-coded.
- **Capability differences:** transactions, durability guarantees, query
  richness, and WASM availability differ per backend and must be advertised, not
  assumed.

The point is not the specific databases; it is that persistence plugs in the
same way geometry does.

## 10. Other Potential Targets

The mechanism should not be tuned only for the two examples above. Plausible
future capabilities include physics/collision solvers, audio, asset import
(glTF/USD/STEP), pathfinding/navmesh, constraint solvers, and scripting
runtimes. Each would define its own Tokimu-owned semantic model and provider
trait, and register one or more backends. We do not need to know the full list
to make the *mechanism* right.

## 11. Selection and Diagnostics

Authoring should express intent, optionally hint a backend, and let Tokimu
resolve deterministically (invariant 2) using the ranking in section 6:

```
request: geometry.extrude(profile, { taper: -0.1, backend: "auto" })
resolve: features {tapered_extrude, watertight_mesh} → backend "manifold"
report:  chosen backend is a diagnostic, not a hidden default
```

If nothing fits, that is an explicit, greppable diagnostic tied into the engine
diagnostics model — never a silent no-op or a NaN in a trench coat.

## 12. Native and WASM

Backends need not be identical across targets initially:

```
Native:   heavier backend (e.g. OCCT geometry, SQLite persistence)
Browser:  precomputed/serialized results, or a WASM-friendly backend
Later:    WASM-native backends (e.g. Truck / Manifold, IndexedDB)
```

Because the semantic model is Tokimu-owned, target parity can arrive later
without changing the authoring surface.

## 13. First Slice Guidance

Keep the mechanism honest by proving it with one capability and one backend
before generalizing. Integrating four libraries at once is how a project becomes
a dependency zoo where every animal needs its own CMake habitat.

1. Define the registry shape honoring the four invariants: an application-owned
   registry, a generic `CapabilityDescriptor` for discovery, a *typed* provider
   trait for the chosen capability, deterministic resolution, and a shared
   `Diagnostic` vocabulary.
2. Pick one capability (geometry is the strongest candidate) and define its
   Tokimu-owned semantic model in a dedicated crate.
3. Implement exactly one backend behind the provider trait.
4. Author intent on the authoring surface, lower it into the Rust model, run the
   operation, map the result back into Tokimu data, and consume it (e.g. render
   a tessellated mesh).
5. Only then add a second backend or a second capability — earning each one.

The architecture should *allow* a collection of options. The implementation
should still earn them one at a time.

## 14. Open Questions

- Where the generic registry envelope lives vs. per-capability crates, and how
  much is truly shared across capabilities as different as geometry and
  persistence.
- How much provider/adapter version metadata the descriptor needs before a
  capability ships (provider version, adapter version, target support).
- How authored intent serializes across the authoring→Rust seam (ties into the
  TypeScript Design Document lowering path).
- The minimum viable `Diagnostic` vocabulary and how it maps onto the engine
  diagnostics model.
- Concrete lifecycle traits for `ProcessBound`/`WorldBound` providers (e.g. an
  open database) without becoming hidden global state.
- Exactly which spatial primitives are core-owned vs `tokimu-geometry`-owned
  (e.g. `Profile`, `Solid`, `GeometryDiagnostic`).

## References

- ADR-0001 Engine Boundaries — `docs/ADR/ADR-0001-engine-boundaries.md`
- ADR-0003 Capability Ownership Boundary — `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- Tokimu Software Design Document — `docs/Tokimu Software Design Document.md`
- Tokimu TypeScript Design Document — `docs/Tokimu TypeScript Design Document.md`
- Diagnostics Model — `docs/diagnostics-model.md`
- Manifold — <https://manifoldcad.org/docs/html/>
- Truck — <https://github.com/ricosjp/truck>
- opencascade-rs — <https://github.com/bschwind/opencascade-rs>
- Fornjot — <https://github.com/hannobraun/fornjot>
- OpenCascade.js — <https://dev.opencascade.org/project/opencascadejs>
- JSCAD — <https://openjscad.xyz/docs/>
