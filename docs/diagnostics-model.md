# Tokimu Diagnostics Model

| Field      | Value                                                        |
| ---------- | ------------------------------------------------------------ |
| Status     | Draft — grow-then-fold                                       |
| Scope      | Diagnostic classification, codes, provenance, and presentation. Not yet v0. |
| Relates to | SDD 10 Diagnostics, SDD 3.1 guiding principle 7 (visible diagnostics) |
| Relates to | SDD M9 Inspector, ADR-0001 Engine Boundaries                 |

## 1. Purpose

The SDD states that Tokimu should prefer explicit diagnostics over silent
fallback, and it lists the *kinds* of diagnostics Tokimu should expose (startup
logs, asset failures, backend info, frame timing, WASM console bridge). What the
SDD does not yet define is *how diagnostics are structured, identified,
related, and presented*.

This document fills that gap. It is intentionally a side document so it can be
grown against real subsystems and then folded into the SDD (likely expanding
section 10) once the shape stabilizes.

The central claim: Tokimu should have **one engine-owned diagnostic model** with
a stable code format and a small set of presentation sinks, so that native,
WASM, tooling, and future authoring frontends all describe notable state and
failure the same way.

Design goals:

* One vocabulary for observation and failure across every crate and platform.
* Stable, greppable diagnostic codes that survive refactors and localization.
* Presentation is a *consumer* of diagnostics, never the owner of them.
* Determinism-friendly: emitting a diagnostic must not change simulation truth.
* Native and WASM parity: the same diagnostic renders through platform-specific
  sinks without changing its identity or meaning.

## 2. Design Constraints

These follow directly from the existing architecture and must not be violated.

* `tokimu-core` stays engine-neutral. The core diagnostic *types* may live in
  or below core, but no logging backend, console bridge, or UI sink leaks into
  it. Core produces diagnostics; it does not present them.
* Rendering, platform, and tooling *observe* diagnostics. They do not become the
  hidden owner of diagnostic state (mirrors SDD world-first ownership).
* Emitting a diagnostic is a side channel. It must never mutate simulation
  state, advance time, or alter scheduling. A diagnostic describes reality; it
  does not change it.
* Diagnostics must be representable as plain data so tools, inspectors, and the
  WASM bridge can consume them without engine-internal knowledge.
* Presentation policy is platform-adaptable, but diagnostic *identity* (code,
  severity, class) is platform-invariant.

## 3. Core Model

A diagnostic is structured data, not a formatted string. The formatted string is
derived at the edge, by a sink.

```text
Diagnostic
├─ code        DiagnosticCode   stable identifier (see §4)
├─ severity    Severity         info | warning | error | fatal
├─ class       DiagnosticClass  subsystem origin (see §5)
├─ message     String           human-readable summary (localizable later)
├─ context     Vec<(key, val)>  structured detail, not baked into message
├─ source      Option<SourceRef> where it originated (crate/file/rule/asset)
└─ timing      Option<FrameStamp> frame index + elapsed seconds when relevant
```

Two important separations:

* **Identity vs. text.** `code`, `severity`, and `class` are stable and
  machine-usable. `message` is human text and may later be localized or
  templated. Tools key off identity, not off message wording.
* **Summary vs. context.** The message is a short summary. Variable detail
  (paths, ids, counts, backend names) goes in `context` as structured pairs, so
  the same diagnostic can be filtered, grouped, and inspected without string
  parsing.

### 3.1 Severity

| Severity  | Meaning                                              | Continues? |
| --------- | ---------------------------------------------------- | ---------- |
| `info`    | Notable but normal (backend chosen, asset loaded).   | Yes        |
| `warning` | Degraded or suspicious, engine proceeds.             | Yes        |
| `error`   | An operation failed; a feature or asset is unusable. | Usually    |
| `fatal`   | The engine cannot continue in a coherent state.      | No         |

`fatal` is the only severity that is allowed to end a run. Everything else is
recoverable and must leave the engine in a describable state. The distinction
between `error` and `fatal` is deliberate: a missing texture is an `error`; a
failed GPU device or event loop is `fatal`.

## 4. Diagnostic Code Format

Codes are the durable contract. They are stable, greppable, and independent of
message text and localization.

```text
TKM-<CLASS>-<NUMBER>

TKM        fixed engine prefix (Tokimu)
CLASS      3–4 letter subsystem tag (see §5)
NUMBER     zero-padded 4-digit code, grouped by range
```

Examples:

```text
TKM-PLAT-0001   native event loop failed to start
TKM-PLAT-0002   window creation failed
TKM-REND-0001   no compatible GPU adapter found
TKM-REND-0002   surface configuration failed
TKM-ASST-0001   asset not found
TKM-ASST-0002   asset failed to decode
TKM-RUNT-0001   fixed-step cap hit (frame overrun)      [warning]
TKM-CORE-0001   schedule phase ordering conflict
```

Rules:

* A code, once published, keeps its meaning forever. To change meaning, retire
  the old code and add a new one.
* The number encodes nothing except identity within a class. Do **not** encode
  severity in the number; severity is a separate field, because the same failure
  class can differ in severity by context.
* Number ranges within a class group related failures, for example
  `0001–0099` lifecycle, `0100–0199` resources, `0900–0999` internal invariants.
* Every emitted `error` or `fatal` diagnostic must carry a code. `info` and
  `warning` should carry a code when a tool might reasonably want to filter or
  count them (frame overruns being the motivating example).

## 5. Diagnostic Classes

Classes map to architectural boundaries, so a code immediately tells you which
crate owns the failure. This keeps ownership honest and greppable.

| Class    | Tag      | Owned by            | Examples                             |
| -------- | -------- | ------------------- | ------------------------------------ |
| Core     | `CORE`   | `tokimu-core`       | schedule conflicts, invariant breaks |
| Runtime  | `RUNT`   | `tokimu-runtime`    | frame overrun, plugin build failure  |
| Render   | `REND`   | `tokimu-render`     | adapter/device/surface failures      |
| Platform | `PLAT`   | `tokimu-platform`   | window, event loop, WASM bootstrap   |
| Assets   | `ASST`   | `tokimu-assets`     | not found, decode, load failure      |
| Input    | `INPT`   | `tokimu-input`      | device/binding issues                |
| Author   | `AUTH`   | authoring frontends | rule lowering / typecheck failures   |

`AUTH` is reserved now even though authoring frontends do not exist yet, so that
when TypeScript authoring lands (see `scripting-typescript.md`) its diagnostics
already have a home and do not leak into core classes.

## 6. Presentation Sinks

Presentation is a consumer. A sink takes engine-owned diagnostics and renders
them for a specific surface. The engine never assumes a particular sink.

```text
Diagnostic (data)
   ↓ emitted through a diagnostic channel
   ├─→ Log sink          tracing / structured console (native)
   ├─→ WASM console sink browser console + panic hook bridge
   ├─→ Inspector sink    signal log / timing panel (SDD M9 tooling)
   ├─→ Startup gate sink blocking/visible surface for fatal boot failures
   └─→ Test/capture sink collect diagnostics for assertions
```

Presentation policy, not identity, is what varies by platform:

* Native: `tracing` + `tracing-subscriber` for logs; a visible, non-silent
  surface for `fatal` startup failures.
* WASM: `console_error_panic_hook` + `tracing-wasm`; `fatal` maps to a visible
  page/console state rather than a silent dead canvas.
* Tooling (later): the inspector's signal log and system timing panel are just
  sinks subscribing to the same diagnostic stream.

A single diagnostic may fan out to multiple sinks. The mapping from
`(severity, class)` to sinks is a policy table owned by the runtime/platform
edge, not by the emitting subsystem.

## 7. Developer vs. End-User Presentation

Tokimu serves both engine implementers and engine users (SDD product split).
Diagnostics must not blur those audiences.

* **Developer-facing**: full detail — code, class, context pairs, source ref,
  timing. This is the default for logs, tests, and the inspector.
* **End-user-facing**: a curated subset. An end user of a Tokimu-built
  application should see a friendly surface for `fatal` conditions, not a raw
  `TKM-REND-0001` dump — though the code should still be discoverable for
  support and bug reports.

The same underlying diagnostic drives both; only the presentation detail level
differs. This keeps a single source of truth while allowing an application built
on Tokimu to shape its own error UX.

## 8. Retention and Query

Two shapes of diagnostic data coexist:

* **Latest-state / counters** — e.g. `RunLoopDiagnostics` already tracks frame
  counts, total fixed updates, cap hits, and last/max frame delta. This is
  sampled, cheap, and overwritten.
* **Event log** — an append-only stream of discrete diagnostics (a texture
  failed, a backend was chosen). This is what an inspector signal log consumes.

Guidance:

* Do not force every diagnostic into an unbounded log. High-frequency, per-frame
  signals belong in counters/rolling windows; discrete events belong in the log.
* The event log should be bounded (ring buffer) with an explicit policy so it
  cannot grow without limit in a long-running session.
* Tools query by identity: filter by `class`, `severity`, or `code`; group by
  `code`; and correlate by `timing`. This is why identity is structured data.

## 9. Provenance and Origin

Tokimu already leans toward explicit relations and inspectable meaning. That
suggests diagnostics should eventually carry provenance rich enough for tools to
answer not just "what failed?" but also "what led here?"

This does **not** mean diagnostics should become world entities or permanent
graph nodes. It means a diagnostic should be able to point at the most relevant
origin surface for inspection.

Early provenance targets:

* subsystem origin: runtime, render, platform, assets, input, authoring
* source object: asset id, rule id, scene id, backend name, plugin name
* authoring source later: TypeScript file, lowered rule, generated artifact
* timing anchor: frame index, elapsed seconds, startup phase

Conceptually:

```text
Diagnostic
   ↓ emitted by
Subsystem
   ↓ refers to
Source object (asset / rule / scene / backend / plugin)
   ↓ optionally traced back to
Authoring source (file / frontend / generated artifact)
```

Examples:

```text
TKM-ASST-0002
   ↓
AssetId("materials/metal-floor")
   ↓
Scene("hangar")
   ↓
Application startup
```

```text
TKM-AUTH-0012
   ↓
RuleId("spawn-wave")
   ↓
Frontend("typescript")
   ↓
scripts/encounters/spawn-wave.ts
```

This is valuable because it gives the future inspector a better model than log
scraping. It can filter, group, and traverse from failure to origin without
parsing human text. That fits Tokimu's platform/tooling direction well.

Near-term guidance:

* Start with lightweight references inside `context` or `source`, not a full
  relationship subsystem.
* Treat provenance as inspectable metadata, not simulation truth.
* Grow richer links only when the asset pipeline, rule frontend, or inspector
  has a real caller that benefits from them.

## 10. Relationship to Existing Code

Where things stand today, so this document tracks reality:

* `tokimu-core::Diagnostics` is currently a simple startup-message collector. It
  is the natural seed for the core diagnostic *type* but does not yet carry
  code, severity, or class.
* `tokimu-runtime::RunLoopDiagnostics` is the first real counter surface and is
  the concrete motivating case for the counter-vs-event split in §8. A frame
  overrun is the first candidate coded diagnostic: `TKM-RUNT-0001` (warning).
* No presentation sinks exist yet beyond example `println!` calls in
  `hello-window`. Those prints are placeholders for the Log sink in §6.

## 11. Open Questions

These are deliberately unresolved and should be answered by real usage before
folding into the SDD.

1. Where does the canonical `Diagnostic` type live — inside `tokimu-core`, or in
   a dedicated `tokimu-diagnostics` crate that core depends on?
2. Is emission push-based (a channel/callback the runtime owns) or pull-based
   (subsystems return structured errors that the runtime records)? A hybrid is
   likely: structured `Result` errors for control flow, plus a diagnostic
   channel for observable events.
3. How do subsystem-local error enums (e.g. a render backend error) map onto
   diagnostic codes without every crate hard-coding the `TKM-...` strings?
4. What is the minimum viable sink set for M4 (renderer spike) — probably just
   the Log sink plus a visible `fatal` startup gate?
5. Do codes need a machine-readable registry file (for docs, tooling, and
   dedupe), or is a source-of-truth Rust enum sufficient early on?
6. When provenance grows, what is the minimal stable reference shape for assets,
   rules, scenes, and generated artifacts?

## 12. Suggested First Increment

Kept small and concrete, matching Tokimu's "one concrete path first" habit:

1. Define `Severity` and a `DiagnosticClass` enum in the chosen home crate.
2. Add a `DiagnosticCode` newtype that formats as `TKM-<CLASS>-<NUMBER>`.
3. Give the existing frame-overrun warning a real code (`TKM-RUNT-0001`) instead
   of an ad hoc `println!`.
4. Add one Log sink so that coded diagnostic flows through a real channel, and
   have `hello-window` route through it instead of raw prints.
5. Only after M4 has a real backend, add the `fatal` startup gate for
   `TKM-REND-*` / `TKM-PLAT-*` boot failures.

This earns the model against real subsystems before it is generalized or folded
into the SDD.
