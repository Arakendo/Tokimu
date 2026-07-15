# Tokimu Primitive Ledger

| Field      | Value |
| ---------- | ----- |
| Status     | Living guidance |
| Scope      | Drift-prone terms that deserve a stable, reviewed ledger entry. |
| Relates to | `docs/semantic-kernel-map.md`, `docs/kernel-principles.md`, `docs/Tokimu Software Design Document.md` |

## Purpose

This ledger records the terms Tokimu is most likely to blur if they are left to
spread informally through the docs and codebase.

It is intentionally smaller and more concrete than the semantic-kernel-map.
The map explains the admission discipline; this ledger tracks the current
candidate meanings and the boundaries that should not drift silently.

Use this file when a term is important enough to deserve a stable Includes /
Excludes record, but not yet so settled that it belongs in the SDD as fixed
architecture.

`Pressure` records which domains repeatedly exercise a term and keep it
drifting.

## Status Values

- `candidate` - still under review
- `accepted` - treated as stable for now
- `derived` - useful, but not a primitive root
- `rejected` - examined and not admitted
- `deprecated` - once used, now intentionally retired

## Ledger Entries

### Entity

| Field | Value |
| ----- | ----- |
| Status | candidate |
| Meaning | identity-bearing participant in the world |
| Pressure | ECS, rules, scene models, inspectors |
| Evidence | `crates/tokimu-core/src/world.rs` entity snapshots/spawn tests; `crates/tokimu-core/src/scene.rs` scene compile tests |
| Includes | lifecycle, relations, component attachment, stable identity |
| Excludes | components themselves, shared state, presentation objects |
| Notes | May reduce to `Id` + world membership + component association. |

### Component

| Field | Value |
| ----- | ----- |
| Status | candidate / derived |
| Meaning | typed state attached to a single entity |
| Pressure | ECS, reflection, inspection, scene compilation |
| Evidence | `crates/tokimu-core/src/world.rs` component storage/query/inspection APIs; `crates/tokimu-core/src/scene.rs` component insertion in scene compilation |
| Includes | per-entity data, inspectable fields, storage-backed state |
| Excludes | entity identity, cross-entity links, shared world state |
| Notes | Often better treated as a runtime/storage realization than a root. |

### Resource

| Field | Value |
| ----- | ----- |
| Status | candidate |
| Meaning | world- or runtime-owned state not attached to one entity |
| Pressure | runtime state, persistence, diagnostics, registries |
| Evidence | `crates/tokimu-core/src/world.rs` resource storage/query APIs; `docs/Tokimu Software Design Document.md` world/resource ownership language |
| Includes | clocks, configuration, shared simulation state, registries |
| Excludes | global singleton by accident, capability grant, backend service |
| Notes | Must remain distinguishable from both `Component` and `Capability`. |

### Relation

| Field | Value |
| ----- | ----- |
| Status | accepted candidate |
| Meaning | a connection between world participants |
| Pressure | world modeling, graphs, rules, ownership, targeting |
| Evidence | `crates/tokimu-core/src/world.rs` relationship storage/query APIs; `crates/tokimu-core/src/scene.rs` parent-child relationship compilation |
| Includes | directed edges, ownership links, targeting, dependency edges |
| Excludes | persistent state that is not really a connection |
| Notes | Relation is structural; event is temporal. |

### Command

| Field | Value |
| ----- | ----- |
| Status | accepted candidate |
| Meaning | intentional request for mutation or privileged action |
| Pressure | rules, runtime host, input, scheduling |
| Evidence | `docs/Tokimu Software Design Document.md` command semantics; `docs/contribution-admission-guide.md` command/request language |
| Includes | apply, spawn, delete, focus, request, invoke |
| Excludes | passive notification, historical record, steady-state relation |
| Notes | Requests may be rejected; they are not facts by themselves. |

### Signal

| Field | Value |
| ----- | ----- |
| Status | candidate |
| Meaning | delivered notification of an event, state change, or attention |
| Pressure | rules, diagnostics, event logs, UI notifications |
| Evidence | `crates/tokimu-core/src/signal_log.rs` signal arrival ordering; `docs/Tokimu Software Design Document.md` signal/event architecture |
| Includes | emitted notices, observations, command-adjacent notification flow |
| Excludes | state itself, queued request, durable relation |
| Notes | Keep separate from `Event` until implementation proves otherwise. |

### Event

| Field | Value |
| ----- | ----- |
| Status | candidate |
| Meaning | temporal occurrence or fact that happened |
| Pressure | input, scheduling, persistence, history, replay |
| Evidence | `docs/semantic-kernel-map.md` event/relation distinction; `docs/Tokimu Software Design Document.md` signal/event vocabulary |
| Includes | happened state, recorded change, time-stamped occurrence |
| Excludes | relation, persistent structural state, authority |
| Notes | Events are historical; relations are structural. |

### Rule

| Field | Value |
| ----- | ----- |
| Status | accepted candidate |
| Meaning | semantic behavior that transforms or emits under conditions |
| Pressure | rule authoring, lowering, runtime host, examples |
| Evidence | `docs/Tokimu TypeScript Design Document.md` rule model and lowering boundary; `docs/Tokimu Software Design Document.md` rule frontend architecture |
| Includes | declarative behavior, lowering targets, emitted signals |
| Excludes | execution mechanism, scheduler, callback machinery |
| Notes | A rule is not the Rust executor that runs it. |

### Handle

| Field | Value |
| ----- | ----- |
| Status | accepted candidate |
| Meaning | stable reference with lifecycle validity |
| Pressure | assets, geometry, persistence, rendering, sessions |
| Evidence | `docs/kernel-principles.md` handle invariants; `docs/semantic-kernel-map.md` handle vs ID distinction |
| Includes | identity, revocation, validation, stale-reference protection |
| Excludes | raw foreign object, provider implementation, plain ID only |
| Notes | A handle can fail; an ID alone does not prove validity. |

### Capability

| Field | Value |
| ----- | ----- |
| Status | accepted candidate |
| Meaning | scoped authority to invoke a bounded service surface |
| Pressure | geometry, persistence, scripting, networking, audio |
| Evidence | `docs/ADR/ADR-0003-capability-ownership-boundary.md`; `docs/capability-backends.md`; `docs/kernel-principles.md` capability authority section |
| Includes | grants, scope, revocation, validation, allowed operations |
| Excludes | provider implementation, global lookup, raw backend handle |
| Notes | This is authority, not merely a named service. |

### Asset

| Field | Value |
| ----- | ----- |
| Status | candidate |
| Meaning | durable/source identity for a loadable or inspectable thing |
| Pressure | asset loading, importers, rendering, metadata |
| Evidence | `crates/tokimu-assets/src/*`; `docs/Tokimu Software Design Document.md` asset identity and ownership language |
| Includes | source metadata, identity, provenance, durable naming |
| Excludes | live backend resource, runtime-only realization |
| Notes | Asset identity and runtime resource should not be conflated. |

### View

| Field | Value |
| ----- | ----- |
| Status | candidate |
| Meaning | projection of authoritative world state for consumption |
| Pressure | rendering, text UI, inspectors, queries |
| Evidence | `docs/semantic-kernel-map.md` view vs world-state distinction; `docs/Tokimu Software Design Document.md` inspection/projection language |
| Includes | render projection, text projection, inspector projection |
| Excludes | world state itself, UI widget, ECS query helper |
| Notes | View is representation, not authority. |

### Time

| Field | Value |
| ----- | ----- |
| Status | accepted candidate |
| Meaning | ordered progression of change and scheduling context |
| Pressure | fixed-step simulation, determinism, replay, scheduling |
| Evidence | `crates/tokimu-core/src/time.rs`; `docs/Tokimu Software Design Document.md` fixed-step/runtime loop language |
| Includes | fixed time, frame time, simulation time, ordering policy |
| Excludes | wall clock by default, raw platform timer, unrelated duration |
| Notes | Time is a core engine concern, not a backend detail. |

### Diagnostic

| Field | Value |
| ----- | ----- |
| Status | accepted candidate |
| Meaning | structured explanation of what happened and why |
| Pressure | validation, backend selection, runtime failures, authoring errors |
| Evidence | `crates/tokimu-core/src/diagnostics.rs`; `docs/diagnostics-model.md`; `docs/contribution-admission-guide.md` evidence/review language |
| Includes | errors, warnings, provenance, traces, explicit failure reasons |
| Excludes | silent fallback, raw stack dump as the only interface |
| Notes | Diagnostics should help distinguish boundary failures from misuse. |

## Maintenance Notes

- Update this file when a drift-prone term gains a clearer boundary.
- Move a term to `accepted`, `derived`, `rejected`, or `deprecated` only when
  the evidence is stable enough to justify the change.
- Keep `Evidence` tied to current code, tests, examples, or governing docs;
  if the term has no such anchor, leave it candidly under-specified.
- If a term becomes core architecture rather than a candidate, mirror the final
  wording in the SDD and, if needed, an ADR.
- Keep `Pressure` focused on repeated domain stress, not on single examples or
  implementation trivia.
- If the ledger grows beyond the terms that are repeatedly causing drift, split
  off a narrower ledger rather than turning this into a second SDD.
