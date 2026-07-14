# TypeScript Scripting for Tokimu

> **Superseded.** This exploratory note has been folded into the authoritative
> [Tokimu TypeScript Design Document (TTSDD)](../Tokimu%20TypeScript%20Design%20Document.md).
> It is kept as background for the reasoning that produced the TTSDD. Where the
> two disagree — especially on runtime TypeScript being a first-class execution
> mode rather than a discouraged later backend — the TTSDD is correct.

| Field        | Value                                             |
| ------------ | ------------------------------------------------- |
| Status       | Superseded background (see TTSDD)                 |
| Scope        | Scripting frontend design, not a v0 commitment    |
| Relates to   | SDD 5.11 Rule Frontends, SDD 5.12 Dependency Rules |
| Relates to   | ADR-0001 Engine Boundaries                        |

## 1. Purpose

This document explores how TypeScript could serve as a scripting language for
Tokimu without violating the engine boundaries already established in the
software design document.

Built-in scripting is a v0 non-goal (SDD section 3). This note is forward
planning only. It exists so that when scripting is earned by the corpus, the
engine already has a defensible integration shape instead of an ad hoc bolt-on.

The central claim: TypeScript is viable, but it should enter Tokimu as a
**frontend over an engine-owned semantic rule model**, not as a second runtime
embedded inside the simulation core.

The more durable architectural point is not "Tokimu uses TypeScript." It is
"Tokimu owns the semantics; TypeScript is one pleasant way to author them."

The semantic rule model is intentionally language-agnostic. TypeScript is the
first plausible frontend because of its mature tooling, not because Tokimu
semantics are inherently tied to JavaScript.

That split is also a product goal. Tokimu itself should remain primarily a Rust
engine, while developers building games, tools, scenarios, and content on top
of Tokimu should be able to live mostly in TypeScript.

## 2. Design Constraints

Any scripting integration must respect the existing architecture:

* `tokimu-core` stays engine-neutral. No JS engine, no TypeScript, no script
  host types leak into it.
* `tokimu-runtime` owns lifecycle and orchestration, but should not hard-depend
  on a specific scripting backend either.
* Scripting is a frontend that compiles or translates into Tokimu-owned rule
  data, matching SDD 5.11.
* Determinism must remain explicit. A script must not silently introduce
  wall-clock time, unseeded randomness, hidden async, or ambient I/O.
* The same integration shape should survive both native and WASM targets.
* Public engine surface area exposed to scripts should be narrow and versioned,
  because every exposed symbol becomes a stability obligation.

## 3. Three Integration Strategies

There are three broad ways to "use TypeScript," and they differ enormously in
cost and risk.

```text
Strategy A  TS → Tokimu Semantic Rule Model  (compile ahead of time, no runtime JS)
Strategy B  TS → JS → hosted engine          (embedded interpreter at runtime)
Strategy C  TS as authoring only             (tooling emits declarative scene/rule data)
```

### 3.1 Strategy A — Compile ahead of time to Tokimu rule data (recommended)

TypeScript source is treated as an authoring language. A build step type-checks
it against a Tokimu API definition, then lowers a restricted subset into the
engine-owned semantic rule model described in SDD 5.11.

At runtime, Tokimu executes rule data. There is no JavaScript engine in the
shipped binary or WASM module.

```text
author.ts
   ↓  tsc / typechecker (build time)
typed AST / restricted subset
   ↓  Tokimu lowering pass
Tokimu Semantic Rule Model (RON/JSON/binary)
   ↓  load
Runtime Systems
```

Why this fits Tokimu best:

* Keeps `tokimu-core` and `tokimu-runtime` free of any JS runtime dependency.
* Preserves determinism, because only the lowered, engine-understood operations
  can run.
* Matches "declarative content, imperative runtime" (SDD guiding principle 8).
* Matches "compile or translate into that engine-owned representation" (SDD 5.11).
* Ships the same rule data on native and WASM with no host-boundary cost.
* Gives authors real TypeScript editor tooling (types, autocomplete, refactor)
  without paying a runtime interpreter tax.
* Keeps the center of gravity in Tokimu's own semantic model rather than in a
  general-purpose language runtime.

Cost:

* Only a subset of TypeScript is expressible. Arbitrary control flow, closures
  over engine state, and dynamic dispatch may be restricted.
* Requires building a lowering pass from typed TS to the Tokimu semantic rule
  model.
* Author mental model must accept "this compiles to rules," not "this runs live."

This is the strategy this document recommends adopting first.

### 3.2 Strategy B — Embedded JS engine at runtime

TypeScript compiles to JavaScript, and a JS engine executes it inside a hosted
sandbox behind a narrow Tokimu API.

```text
author.ts
   ↓  tsc (build or load time)
author.js
   ↓  load into hosted engine
JS engine (QuickJS / Boa / V8-class)
   ↕  narrow FFI boundary
Tokimu runtime host API
```

Candidate hosts:

* QuickJS via a Rust binding — small, embeddable, ES2020-ish, native-friendly.
* Boa — pure Rust JS engine, WASM-friendly, but less complete and slower.
* V8-class engines — powerful but heavy, awkward for a clean WASM story.

Why it is tempting:

* Full dynamic scripting. Live behavior, hot reload, expressive game logic.
* Familiar to web developers.

Why it is risky for Tokimu:

* Determinism becomes hard. Time, randomness, async, and host object access must
  be banned or wrapped, or the simulation core loses its determinism guarantee.
* Per-frame Rust ↔ JS boundary crossings can dominate cost if scripts touch the
  world every tick.
* The browser/WASM path becomes a JS-in-WASM-calling-out-to-browser-JS problem,
  which is more moving parts than Strategy A.
* Exposing engine objects directly freezes more engine surface earlier than the
  project wants (SDD principle 6, principle 12).

This strategy is viable later, but only behind its own crate and feature flag,
and only after the rule model is stable.

### 3.3 Strategy C — TypeScript as authoring tooling only

TypeScript is used to build editor tooling and generators that emit declarative
scene and rule documents (RON/JSON/TOML). Tokimu never sees TypeScript or
JavaScript at runtime.

This is effectively a lighter-weight sibling of Strategy A: scripts are a build
convenience that produce the declarative content the SDD already endorses
(SDD 5.10). It is a good stopgap and pairs well with Strategy A adoption.

## 4. Recommended Path

Adopt Strategy A as the primary architecture, keep Strategy C available as a
tooling convenience, and treat Strategy B as an optional advanced backend that
must never become the default.

```text
Phase 1  Declarative scene/rule data (RON), no scripting                     (SDD status quo)
Phase 2  Tokimu semantic rule model stabilizes with real example callers
Phase 3  TypeScript authoring + ahead-of-time lowering to that model         (Strategy A)
Phase 4  Optional embedded JS host behind a feature flag                      (Strategy B)
```

Scripting should not overtake the world model. Phase 4 is explicitly gated
behind a stable rule model, matching SDD principle 12 ("stage editors and
scripting behind the world model").

## 5. Proposed Crate Shape

Scripting lives in its own crates so engine core stays clean.

```text
tokimu-core          no scripting knowledge
tokimu-runtime       no hard scripting dependency
tokimu-rule          engine-owned semantic rule model (may predate scripting)
tokimu-script-ts     TypeScript frontend: typecheck + lower to tokimu-rule
tokimu-script-host   optional embedded JS host (Strategy B, feature-gated)
```

The `tokimu-rule` name is sufficient for the first iteration because this
document is specifically about rule authoring. If the crate later grows into a
broader home for schedules, commands, signals, and other execution semantics,
it may deserve a more general name such as `tokimu-semantics` or
`tokimu-model`. That is a naming pressure to watch, not a reason to rename it
early.

Dependency direction (consistent with SDD 5.12):

* `tokimu-script-ts` depends on `tokimu-rule` and Tokimu-owned world/rule
  abstractions. It does not depend on the renderer, platform, or the facade.
* `tokimu-core` and `tokimu-runtime` never depend on `tokimu-script-ts` or
  `tokimu-script-host`.
* The facade crate `tokimu` may re-export scripting crates behind a feature
  flag, but scripting crates must not depend on the facade.

## 6. Compiler Toolchain

The lowering pass needs authoritative type information. The ecosystem already
provides it, so the choice is which frontend to lean on.

* **TypeScript Compiler API** — the official route. Provides parsing, type
  checking, symbol resolution, diagnostics, AST traversal, and full type
  information. If Tokimu grows serious TypeScript support, this is the
  authoritative foundation.
* **ts-morph** — a friendlier wrapper over the compiler API. Recommended for the
  first prototype: it makes AST navigation and type inspection pleasant while
  still exposing the underlying compiler objects when needed.
* **Oxc** — a fast Rust-based JS/TS tooling ecosystem (parser, transformer,
  linter, resolver). Interesting if the lowering pipeline later moves fully into
  Rust, but today it is more JS-tooling focused than semantic-lowering focused,
  and the official TypeScript tooling still gives the richest type information.

Recommended prototype stack:

* ts-morph for AST navigation and type inspection.
* TypeScript Compiler API underneath for authoritative type information.
* A very small lowering pass (10–20 supported constructs to start).

This answers the most important early question cheaply: can people write rules in
a TypeScript style that naturally lowers into Tokimu semantics? Optimization,
caching, and incremental builds are later rabbit holes, not first steps.

Improvements in TypeScript tooling performance continue to reduce the cost of an
ahead-of-time compilation workflow, reinforcing Strategy A without changing the
underlying architectural tradeoffs. Faster tooling helps authoring, but it does
not remove the sandboxing, determinism, and host-boundary problems of embedded
runtime scripting (Strategy B).

## 7. Anchor on the `tokimu` Package, Not Arbitrary JavaScript

The single most important scope decision: do not lower arbitrary ASTs. Lower
specific, recognized API calls.

Instead of claiming "Tokimu supports TypeScript," claim "Tokimu supports the
`tokimu` package." The import becomes the anchor:

```ts
import { rule, query } from "tokimu";
```

The lowering pass only understands constructs that originate from that API. When
it sees a `rule()` declaration, it inspects the callback, validates the allowed
constructs, rejects unsupported ones, and emits rule data. It never tries to
understand JavaScript in general.

This is the actual architectural decision. Tokimu is not trying to understand
JavaScript as a language. It is recognizing a Tokimu language frontend built
from specific primitives, with TypeScript supplying syntax, type checking, and
tooling.

This is conceptually closer to React, JSX, LINQ, or SQL builders than to
"general JavaScript support." Those systems do not try to understand arbitrary
host-language behavior. They recognize known constructs and lower them into a
separate semantic model. Tokimu can lean on the same discipline, favoring
recognizable primitives:

```ts
rule("movement")
  .reads(Transform, Velocity)
  .writes(Transform)
  .system(/* ... */);
```

Supported and unsupported construct scope should be explicit. An initial cut:

```text
Supported          Not supported
---------          -------------
✓ rule()           ✗ DOM access
✓ query()          ✗ Promise / async
✓ signal()         ✗ Date
✓ relation()       ✗ Math.random
✓ command()        ✗ fetch
✓ deterministic loops  ✗ eval
✓ arithmetic       ✗ arbitrary I/O
```

The unsupported list is not incidental: each entry is a non-deterministic or
out-of-sandbox construct that the lowering pass should reject at build time,
keeping Strategy A deterministic by construction (see the determinism section).

## 8. The Script-Facing API

Whatever strategy runs, scripts should see a small, intentional surface, not the
raw engine.

Illustrative TypeScript authoring shape (Strategy A input):

```ts
import { rule, query, Transform, Velocity } from "tokimu";

export const applyVelocity = rule("apply_velocity", (ctx) => {
  for (const e of query(Transform, Velocity)) {
    e.get(Transform).x += e.get(Velocity).x * ctx.fixedDelta;
    e.get(Transform).y += e.get(Velocity).y * ctx.fixedDelta;
  }
});
```

This lowers to engine rule data conceptually equivalent to:

```text
rule "apply_velocity" {
  reads:  [Transform, Velocity]
  writes: [Transform]
  step:   integrate_velocity(fixed_delta)
}
```

That distinction matters: the TypeScript is authoring syntax, while the Tokimu
semantic rule model is the engine-owned meaning that other frontends could also
target later.

Rules for the API surface:

* Only fixed-step time is exposed (`ctx.fixedDelta`), never wall-clock time.
* Randomness, if exposed, is seeded and engine-owned.
* No filesystem, network, or DOM access from script scope.
* Component access is typed and mediated by the query API, not raw pointers.
* Emitted signals are declared, so tools can inspect script effects (SDD
  principle 11).

## 9. Example Lowering

One concrete example demystifies the whole proposal.

Authoring input:

```ts
rule("movement", (ctx) => {
  for (const e of query(Transform, Velocity)) {
    e.get(Transform).x += e.get(Velocity).x * ctx.fixedDelta;
  }
});
```

Typed AST and type-checked meaning:

```text
CallExpression rule("movement", callback)
  callback parameter: ctx : FixedStepContext
  query arguments: Transform, Velocity : Component types
  component writes: Transform.x
  component reads: Velocity.x
  time source: ctx.fixedDelta
```

Tokimu semantic rule model:

```text
rule "movement" {
  reads: [Transform, Velocity]
  writes: [Transform]
  time: fixed_step
  operation: integrate Transform.x with Velocity.x
}
```

Runtime execution:

```text
schedule fixed update
  ↓
evaluate movement query over matching entities
  ↓
apply validated transform updates
  ↓
emit declared signals if any
```

This is why the document should be read as a frontend design, not as "embed
TypeScript and see what happens." TypeScript is one input surface. Tokimu keeps
ownership of the semantic model and runtime behavior.

## 10. Broader Frontend Pattern

The rule frontend described here is likely the first example of a broader
Tokimu architecture, not a one-off exception.

That broader architecture supports a simple language story for users of the
engine: Tokimu implementers mostly work in Rust, while Tokimu authors mostly
work in TypeScript. The engine is free to execute whatever backend forms it
needs internally, but the primary author-facing surface can still stay narrow.

```text
TypeScript syntax
  ↓
domain-specific Tokimu API
  ↓
domain-specific semantic model
  ↓
target compiler/runtime
```

That pattern could plausibly extend beyond rules:

* scenes and entity declarations
* schemas and component metadata
* deterministic scenarios and acceptance tests
* diagnostic queries
* UI bindings
* specialized shader or compute frontends later

The important constraint is that these should not collapse into one giant
"Tokimu understands TypeScript" compiler. Each domain should own its own API
surface and semantic model, even if the frontend infrastructure is shared.

Illustrative direction:

```text
@tokimu/rules   → tokimu-rule model
@tokimu/scenes  → tokimu-scene model
@tokimu/query   → tokimu-query model
@tokimu/ui      → tokimu-presentation model
@tokimu/shader  → tokimu-shader model
```

Shared infrastructure could still exist underneath:

```text
tokimu-ts-frontend
  ├─ project loading
  ├─ TypeScript compiler integration
  ├─ symbol recognition
  ├─ diagnostics
  ├─ source maps
  ├─ constant evaluation
  └─ lowering interfaces
```

This preserves the durable idea behind the document: TypeScript supplies syntax
and tooling; Tokimu supplies meaning.

## 11. Determinism and Sandboxing

Determinism is the deciding factor between the strategies.

* Strategy A is deterministic by construction: only lowered operations run, and
  the lowering pass can reject non-deterministic constructs at build time.
* Strategy B requires an active sandbox: shim or remove `Date`, `Math.random`,
  `setTimeout`, `Promise` scheduling, and any host globals; inject engine-owned
  time and RNG instead.

If a script needs behavior the semantic rule model cannot express, that is a
signal to grow the model deliberately, not to open a general escape hatch into
arbitrary JS.

## 12. WASM Considerations

* Strategy A produces plain rule data. It loads identically on native and WASM
  with no extra runtime, which is the cleanest possible browser story.
* Strategy B must pick a host that compiles to `wasm32`. Boa is pure Rust and
  fits; QuickJS needs a WASM-capable build; V8-class engines do not fit a clean
  WASM module.
* Avoid threads in the early WASM scripting path, consistent with SDD 9.

## 13. Open Questions

* How much of TypeScript's type system should the lowering pass honor versus a
  deliberately restricted DSL-in-TS subset?
* Should rules be pure transformations only, or may they emit commands/signals
  that the runtime applies after the step?
* What is the minimum semantic rule model needed before Strategy A is worth building?
* Is hot reload a Strategy A concern (recompile IR and swap) or strictly a
  Strategy B feature?

## 14. Summary

TypeScript can absolutely be Tokimu's scripting language. The safest and most
architecturally consistent path is ahead-of-time compilation of a restricted
TypeScript subset into an engine-owned semantic rule model, keeping the
simulation core free of any JavaScript runtime. More importantly, Tokimu should
be understood as owning the semantics while TypeScript remains one particularly
pleasant frontend for authoring them. A live embedded JS host remains possible
later, but only as an opt-in backend behind the stabilized world and rule
model, never as the engine's center of gravity. The same pattern can later
support other language frontends and semantic models, but each one should earn
its existence through a concrete use case rather than expanding a single,
monolithic compiler.
