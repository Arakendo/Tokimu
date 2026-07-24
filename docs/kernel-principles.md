# Tokimu Kernel Principles

| Field      | Value                                                                                        |
| ---------- | -------------------------------------------------------------------------------------------- |
| Status     | Draft — exploratory, grow-then-fold                                                           |
| Title      | "Kernel Principles" (kept) — names the discipline rather than a subsystem                    |
| Scope      | Kernel-like design principles Tokimu should borrow to keep a small trusted core with explicit authority. Not a committed subsystem; a lens over existing architecture. |
| Relates to | ADR-0001 Engine Boundaries, ADR-0003 Capability Ownership Boundary, SDD ownership model        |
| Relates to | `docs/capability-backends.md`, `docs/diagnostics-model.md`, `docs/semantic-kernel-map.md`     |

## 1. Purpose

The SDD already frames Tokimu as an "engine kernel": a reusable runtime that
higher-level engines and applications build upon. This document collects the
kernel-design lessons worth borrowing — not to imitate an operating system, but
to steal the parts of kernel discipline that stop shared mutable state and
ambient authority from becoming everybody's problem.

The goal is a small trusted core with explicit authority, stable handles, clear
lifetimes, message-based coordination, deterministic scheduling, and native
observability. Most of these ideas are already latent in Tokimu; this document
names them so they are protected deliberately rather than by accident.

This is a side document on purpose. It is a lens over the existing architecture
(ADR-0001, ADR-0003, `capability-backends.md`, `semantic-kernel-map.md`,
`diagnostics-model.md`) and can fold into the SDD once the principles
stabilize. It does not introduce new subsystems; it records the discipline the
trusted core should hold to.

The one warning worth keeping in view: microkernel elegance can curdle into
message-passing bureaucracy. Keep boundaries strong, but do not force every
local operation through three adapters and a ceremonial envelope unless it
protects a real boundary.

## 2. The Minimal Trusted Core

The most important kernel lesson: keep the trusted core small. The code that
must be correct for the whole engine to stay coherent should be limited to:

- world identity and state ownership
- scheduling and time
- commands / signals / events
- handles and resource lifetime
- the capability registry and grants
- diagnostics and provenance
- core serialization / version contracts

Everything else is layered outside that core in two rings. **Foundational
services** (platform, render, assets, input) are first-party and effectively
always present, but they still consume the core rather than own it. **Optional
capabilities** (geometry, persistence, physics, scripting, networking, audio)
are earned per ADR-0003. Both rings are replaceable in principle, but the
distinction matters: `tokimu-render` is not as optional as an OCCT adapter, and
the layering should say so. Not malicious, just specialized, fallible, and prone
to arriving with seventeen build scripts.

```
Trusted kernel
- world identity and state ownership
- schedule and time
- commands/signals/events
- handles and resource lifetime
- capability grants
- diagnostics and provenance
- version contracts

Foundational services (first-party, effectively always present)
- platform  render  assets  input

Optional capabilities (earned per ADR-0003)
- geometry  persistence  physics  scripting  networking  audio
```

This three-tier view matches how Tokimu is actually structured and lines up with
ADR-0003's native/capability/backend tiers without making foundational
subsystems sound as optional as a backend adapter.

## 3. Capability-Based Authority

A subsystem should receive only the authority it needs, granted through handles
or capabilities rather than ambient reach:

```
geometry backend:  may produce geometry results, may emit diagnostics,
                   may NOT mutate arbitrary world state
runtime script:    may query selected data, may emit commands/signals,
                   may NOT receive raw World access
```

This is already the direction of `capability-backends.md` (capability crates own
domain meaning; backends only execute) and of the SDD's runtime-host boundary.
The kernel framing just makes it a first-class rule: privileged reach is granted,
scoped, and revocable — never assumed.

## 4. Handles, Not Raw Ownership Leakage

Kernels do not hand applications internal pointers and ask them to be gentle.
Tokimu should prefer stable handles over exposing backend-native objects:

```
EntityId  AssetHandle  MeshHandle  ProviderId  SessionHandle  CapabilityHandle
```

Handles buy validation, lifecycle control, versioning, diagnostics, revocation,
and clean serialization boundaries. A raw OCCT shape, database connection, or JS
runtime object crossing layers is the architectural equivalent of handing a
toddler the fuse box because they seemed curious. This restates
`capability-backends.md` invariant 4 (no foreign object leakage) as a core
property, and Tokimu already uses `EntityId`/`AssetHandle`/`MeshHandle` today.

One handle invariant is worth making explicit:

> A stale handle must fail deterministically and diagnostically, never alias a
> newly allocated resource.

That usually implies generation/version protection on slot-based handles, so a
reused index cannot silently impersonate a freed resource:

```rust
struct Handle {
    index: u32,
    generation: u32,
}
```

Otherwise a deleted physics world eventually starts referring to a texture
because the slot got reused. Integers enjoy that kind of practical joke.

## 5. A Narrow Syscall-Like Surface

Treat commands, signals, queries, asset requests, and capability calls as
Tokimu's equivalent of system calls: all privileged operations cross one small,
inspectable boundary.

```
author / runtime code
    ↓
Tokimu command / query / capability API
    ↓
engine-owned execution
```

A narrow surface is easier to validate, trace, sandbox, replay, version, expose
to TypeScript, and support across native and WASM. This pairs with the TTSDD
lowering seam: authoring frontends target that surface, not engine internals.

Keep the metaphor, but resist collapsing these into one universal "kernel call"
trait too early. They have different semantics and should share diagnostics,
authority, and tracing without a single trait that eventually grows twelve
generic parameters and a prayer:

- **queries** read
- **commands** request mutation
- **signals** notify
- **capability calls** invoke specialized services

## 6. Scheduling as Policy, Not Accident

"Whatever runs first" is not a strategy. Tokimu already has phases and
priorities; the kernel mindset is to make the surrounding questions explicit:

- who owns ordering?
- which phases may mutate world state?
- which systems may block?
- which jobs may run in parallel?
- what happens when a system overruns its budget?
- are commands applied immediately or at a phase boundary?

The aim is not a preemptive OS scheduler. It is that execution order is a
declared policy rather than repository folklore passed down through frightened
maintainers. Budgets (section 14) should be enforced through this scheduler
policy rather than as subsystem-local timers, so no service invents its own
little police department.

ADR-0006 makes the execution ownership boundary explicit:

- the trusted kernel owns engine-neutral dependency, ordering, permitted
  independence, deterministic commit, affinity, budget, and diagnostic
  semantics only as concrete workloads require them;
- `tokimu-runtime` owns application-wide coordination, dispatch, joining,
  draining, and budget enforcement;
- `tokimu-platform` owns native threads, browser workers, and other
  target-specific execution mechanisms;
- domains expose bounded deterministic work and do not create ambient private
  pools as their public execution model.

Sequential execution remains a first-class policy. Parallel completion may
vary, while observable commit order must be defined wherever it affects engine
semantics. This does not admit parallel `World` mutation or place a thread-pool
dependency in `tokimu-core`.

## 7. Resource Lifetime Scopes

Kernels obsess over lifetime because leaked resources become system problems.
Tokimu benefits from explicit scopes:

```
application lifetime   — e.g. renderer device
runtime lifetime
world lifetime         — e.g. a physics world
scene lifetime
frame lifetime         — e.g. a command buffer
operation lifetime     — e.g. a one-shot mesh conversion
```

This extends the provider lifecycle classes in `capability-backends.md`
(`Stateless`/`Session`/`WorldBound`/`ProcessBound`) into a single scope
vocabulary the whole core can share.

## 8. Isolation and Failure Containment

One failed driver must not destroy unrelated subsystems.

```
asset importer fails → asset diagnostic → placeholder / failed handle
                     → runtime continues where safe
```

not:

```
asset importer fails → process-wide existential collapse
```

Useful policies: per-capability failure boundaries, structured diagnostics,
optional restart/reinitialize, backend disablement after repeated failure, and a
clear fatal-vs-recoverable distinction. A scripting VM, database backend,
renderer, and CAD kernel should not share one emotional state.

## 9. Stable Internal Contracts

Kernel-like systems keep interfaces between parts stable even as implementations
change. For Tokimu, the meaning at subsystem boundaries should survive refactors:

- semantic models
- provider contracts
- command / event shapes
- handle identity
- lifecycle hooks
- version negotiation
- diagnostic structure

Not every Rust trait must be permanent, but boundary meaning should be. This
matters most for TypeScript frontends, saved scenes, capability manifests, and
WASM targets.

## 10. Versioned Data Boundary

Even in one process, think in ABI-like discipline — likely a versioned
serialized contract rather than a true binary ABI. Questions to answer:

- can authored content declare the semantic-model version it targets?
- can a capability adapter declare compatible model versions?
- can old saved scenes be migrated?
- can runtime scripts discover the host API version?
- can a backend refuse an incompatible request clearly?

These are likely three independently evolving version concepts, not one number:

```
semantic model version      — the meaning of the world/rule model
host API version            — what runtime/authoring callers bind to
serialized document version — the on-disk scene/schema shape
```

A scene file may be schema v3, target semantic model v2, and run against host
API v5. Treating "version" as a single number eventually makes migrations feel
like solving a murder using only elevator music.

The kernel lesson: boundaries outlive implementations. This ties into SDD
question 25 (frontend and model versioning) and the M7/M8 persistence and scene
work.

## 11. Message Passing Over Direct Coupling

Prefer a message fabric over every subsystem calling every other subsystem:

```
physics → audio, particles, quests, UI     (avoid)

physics emits CollisionStarted → interested systems react   (prefer)
```

Signals, commands, events, and capability requests are Tokimu's message-passing
fabric and fit the "society of ignorant systems" idea in the SDD. This is one of
the strongest kernel-like features to lean into — without forcing trivial local
work through ceremonial envelopes.

## 12. Observability as a Native Service

A kernel without diagnostics is a black box with authority issues. Tokimu should
natively expose system timing, command flow, signal flow, provider selection,
resource lifetime, state-mutation provenance, failure reasons, capability
grants, and world/scene snapshots.

Treat this as a kernel service, not "debug tooling later." One invariant makes
this concrete:

> Every privileged boundary crossing should be traceable in debug/inspection
> builds.

That covers command accepted/rejected, signal emitted/delivered, capability
granted/revoked, backend selected, handle invalidated, provider failed, and
attributed world mutation. Release builds need not log every event, but the
architecture should make provenance possible without invasive patching. It
reinforces the provenance and inspection goals already in the SDD (M9 inspector)
and the `diagnostics-model.md` direction. A semantic runtime that cannot explain
what changed is just an elaborate rumor.

## 13. Deterministic Clocks and Controlled Randomness

Make time and randomness explicit resources, never ambient wall-clock or global
RNG calls in simulation code:

```
FixedTime  FrameTime  SimulationTime  SeededRng
```

This buys deterministic tests, replay, pause/step, simulation-speed control, and
easier networking and tooling. Tokimu already has `FixedTimeStep`; the seeded RNG
resource is a named roadmap deliverable. Time is too powerful to be left lying
around loose.

## 14. Backpressure and Budgets

Easy to ignore early, but kernel-like and worth designing for: subsystems may
eventually need budgets — max commands per frame, max script execution time, max
asset work per tick, max provider memory, max queued signals, max async tasks.

No enforcement is needed now. But queues and diagnostics should be shaped so
limits *can* exist later, before one enthusiastic script emits two million
signals and Tokimu discovers thermodynamics. When budgets do arrive, they should
be enforced through scheduler policy (section 6) — suspend the unit, emit a
diagnostic, and defer or drop remaining work — rather than as subsystem-local
timers.

## 15. Revocation

Capabilities should be revocable:

- scene unload removes a script's access
- a disposed provider invalidates its handles
- world destruction revokes world-bound services
- hot reload invalidates old callbacks
- a disconnected network session loses command authority

A capability that cannot be revoked is often just global access with better
branding. Revocation likely needs both broad and targeted forms: scope teardown
handles broad invalidation (scene unload revokes scene-bound capabilities and
invalidates scene-bound handles), while targeted revocation handles the rest (a
script loses `ui.open`; a network client loses command authority; a provider is
disabled after failure). So a capability grant should stay an identifiable,
revocable object even if v0 implements revocation through handle invalidation.
This pairs naturally with the capability registry and the lifetime scopes in
section 7.

## 16. Relationship to Existing Docs

This document is a lens, not a new subsystem:

- **ADR-0001 / ADR-0003** fix the ownership boundaries; kernel principles explain
  *why* the trusted core stays small and how authority is granted across it.
- **`capability-backends.md`** defines the provider/registry mechanism; sections
  3–4, 7, and 15 here generalize its authority, handle, lifetime, and revocation
  ideas to the whole core.
- **`diagnostics-model.md`** and the SDD inspector work realize section 12.
- The SDD remains the authority; principles here graduate into it (or an ADR)
  once a concrete slice exercises them.

## 17. Open Questions

Strongest promotion candidates (ready to move into the SDD or an ADR once a slice
exercises them): minimal trusted core, no ambient authority, handles over
foreign/raw ownership leakage, explicit time/RNG, observability as a native
service, deterministic scheduling policy, and revocable scoped capabilities. The
rest — budgets, the exact lifetime taxonomy, and any syscall-surface
formalization — stay exploratory until a real slice forces them.

- Whether lifetime scopes (section 7) should be one shared vocabulary or stay
  per-subsystem until a second capability crate exists.
- How far the "syscall-like surface" is formalized vs. left as a convention over
  distinct commands/queries/signals/capability calls (kept separate, not one
  trait).
- The concrete shape of a first-class capability grant object and how targeted
  revocation coexists with scope teardown.
- The minimal shape of the three version concepts (semantic model, host API,
  serialized document) so M7/M8 persistence and the authoring frontends share
  one migration story.

## References

- ADR-0001 Engine Boundaries — `docs/ADR/ADR-0001-engine-boundaries.md`
- ADR-0003 Capability Ownership Boundary — `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- Tokimu Software Design Document — `docs/Tokimu Software Design Document.md`
- Capability Backends — `docs/capability-backends.md`
- Diagnostics Model — `docs/diagnostics-model.md`
