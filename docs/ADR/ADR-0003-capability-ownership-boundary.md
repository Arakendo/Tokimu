# ADR-0003: Capability Ownership Boundary

## Status

Accepted

## Context

Applications built on Tokimu will reach for heavy, specialized libraries — CAD
geometry kernels, databases, physics solvers, scripting runtimes, audio,
importers, and more. Tokimu does not know in advance which of these any given
application will need.

There is a recurring failure mode: confusing "many applications may need this"
with "this belongs in core." Under that pressure, every optional dependency gets
declared fundamental, core grows without bound, native/WASM parity erodes, and
`cargo build tokimu` starts to feel like assembling an aircraft carrier.

The engine already keeps simulation, runtime, rendering, platform, and
persistence as separate concerns (ADR-0001). This ADR extends that discipline to
optional heavy capabilities, and records the ownership rule that
`docs/capability-backends.md` develops in detail.

## Decision

Tokimu is organized into three ownership tiers:

> Native Tokimu owns universal meaning. Capability crates own domain meaning.
> Backends own specialized execution.

**Native (engine kernel).** Tokimu natively owns only the concepts that define
its worldview and that nearly every application depends on: the
world/entity/component/resource model and relationships, schedules and time,
commands/signals/events, normalized input, diagnostics, asset identity and
ownership, platform and presentation abstractions, serialization and reflection
primitives, stable names and IDs, and the capability discovery/resolution
contracts. It also provides lightweight spatial and mathematical primitives.

**Capability crates (Tokimu-owned domain meaning).** Domain semantics that only
some applications need live in dedicated capability crates (for example
`tokimu-geometry`, `tokimu-persistence`, `tokimu-physics`, `tokimu-script-host`)
outside `tokimu-core` and `tokimu-runtime`. "Tokimu-owned" does not mean
"core-owned": a Tokimu-defined type such as a geometry `Profile` or `Solid` may
belong in a capability crate rather than core.

**Backends (specialized execution).** Concrete external libraries are integrated
as replaceable backend adapter crates (for example a Manifold, Truck, or OCCT
geometry backend; a SQLite or IndexedDB persistence backend) behind
Tokimu-owned provider contracts. Even backend-specific extensions expose only
Tokimu-owned types; no foreign library object crosses into engine-owned or
author-facing APIs.

A concept is **native** when most of these hold: nearly every application needs
it; it expresses engine meaning rather than one domain's meaning; multiple
subsystems depend on it; its absence would fragment the ecosystem into
incompatible vocabularies; it can be implemented without a heavy specialized
dependency; and it strengthens native/WASM parity.

A concept is a **capability backend** when most of these hold: only some
application classes need it; multiple mature external libraries already compete;
implementations differ by platform or precision; the dependency is large,
native, oddly licensed, or hard to build; users may reasonably want different
implementations; and it owns specialized algorithms rather than Tokimu's world
truth.

Capabilities are earned one at a time. A capability crate and its first backend
are added only when a concrete application or example justifies them, never
preemptively because a future user might want them.

## Consequences

Tokimu keeps a small, coherent kernel that stays portable across native and
WASM, while a broad ecosystem of optional capabilities and interchangeable
backends can grow around it without bloating core or fragmenting the engine's
vocabulary. The trade-off is more crates and an explicit capability-resolution
layer to design and maintain; `docs/capability-backends.md` covers the
mechanism (semantic model, provider traits, capability registry, deterministic
resolution, lifecycle, and diagnostics). Deciding exactly which spatial
primitives are core-owned versus `tokimu-geometry`-owned is deferred until the
first geometry capability crate lands.
