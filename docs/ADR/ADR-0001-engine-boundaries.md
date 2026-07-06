# ADR-0001: Engine Boundaries

## Status

Accepted

## Decision

Tokimu keeps simulation state, runtime orchestration, rendering, platform
integration, and persistence as separate concerns.

* `tokimu-core` owns engine-neutral simulation state and scheduling concepts.
* `tokimu-runtime` owns the application loop and plugin orchestration.
* `tokimu-render` and `tokimu-platform` adapt output and OS/browser concerns.
* Persistence, if introduced later, remains outside core/runtime crates.

## Consequences

The engine can evolve native and WASM support without hard-coding platform logic
into the simulation model, and future tool or persistence integrations do not
become hidden owners of world state.