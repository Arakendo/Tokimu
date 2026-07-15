# Tokimu TypeScript Design Document (TTSDD)

| Field        | Value                                                       |
| ------------ | ----------------------------------------------------------- |
| Status       | Draft (revised after first review)                          |
| Version      | 0.2.0                                                       |
| Last updated | 2026-07-14                                                  |
| Scope        | The TypeScript authoring surface, its boundaries, and how it maps into Tokimu-owned semantics |
| Supersedes   | `docs/archive/scripting-typescript.md` (kept as archived background) |
| Relates to   | SDD 5.11 Rule Frontends, SDD 5.12 Dependency Rules, ADR-0001 |

## 0. How to read this document

The Software Design Document (SDD) is the source of truth for the **engine**.
This document (the TTSDD) is the source of truth for the **TypeScript authoring
surface** that sits on top of the engine.

Where the SDD says "presentation consumes state but does not own simulation
truth," this document says the parallel thing for authoring: **TypeScript
supplies syntax, types, and tooling; Tokimu owns the semantics.**

If this document and the SDD ever disagree about a boundary, the SDD wins and
this document is wrong until corrected. This document may go into more detail
about the TypeScript surface than the SDD, but it may not contradict it.

Terminology follows the SDD's semantic-vs-runtime split:

- **Trait** names an authored or inspectable semantic property in the shared
  model vocabulary.
- **Component** names the runtime/storage unit the Rust engine currently uses.

When this document discusses author-facing meaning, prefer `Trait`; when it
discusses current engine APIs, storage, query execution, or reflection work,
`Component` may still be the accurate term.

## 1. Purpose

Tokimu is a Rust-native engine. This document defines how developers who build
*on* Tokimu — games, tools, simulations, scenarios, content — can author that
work primarily in TypeScript without turning TypeScript into a second engine.

The durable claim is not "Tokimu supports TypeScript." It is:

> Tokimu owns a language-agnostic semantic model. TypeScript is the first, and
> currently primary, pleasant frontend for authoring against that model.

The engine is free to execute authored intent in more than one way. What must
stay stable is *the meaning*, not the surface syntax and not the execution
backend.

## 2. The central idea: two execution modes, both first-class

Earlier exploration (`archive/scripting-typescript.md`) framed ahead-of-time lowering as
"recommended" and a runtime JavaScript host as "risky, maybe later." That framing
is now out of date.

The engine-owned rule model already carries **explicit execution intent**
(`ExecutionMode::{Auto, Lowered, Runtime}` in `crates/tokimu-rule`). The TTSDD
adopts that as its spine:

```text
                 author declares intent
                          │
        ┌─────────────────┼─────────────────┐
   execution:"lowered"  "auto"          "runtime"
        │                 │                 │
        ▼                 ▼                 ▼
  lower at build     try to lower,     keep in the TS
  into engine        fall back to      runtime host
  semantics          runtime with a
                     visible reason
```

Both **Lowered** and **Runtime** are legitimate, supported destinations. Neither
is a second-class citizen. The engine's job is to:

1. respect the author's declared intent,
2. lower what the author's mode permits — for `lowered`, everything or a build
   error; for `auto`, resolve *stably* rather than opportunistically (see §2.3),
3. and make the boundary — what lowered, what stayed runtime, and *why* —
   explicit rather than folklore.

### 2.1 What each mode is for

| Mode      | Best fit                                                                 | Guarantees                                              |
| --------- | ------------------------------------------------------------------------ | ------------------------------------------------------- |
| `lowered` | deterministic simulation, replay, lockstep networking, portability, engine-facing tooling | runs as Tokimu-owned systems; no JS at runtime; native/WASM identical |
| `runtime` | gameplay glue, quest/dialogue orchestration, UI event handlers, one-off flow | flexible; runs in a TS/JS host behind a narrow API; not determinism-guaranteed |
| `auto`    | "lower it if you can, otherwise run it"                                   | engine chooses, and reports which path each rule took   |

### 2.2 The lowering-boundary contract

The engine must never silently downgrade a `lowered` rule to `runtime`. If an
author marks a rule `lowered` and it uses a construct the lowering pass cannot
express, that is a **build-time error with a specific reason**, not a quiet
fallback. `auto` is the only mode allowed to fall back, and even then the outcome
is reported.

This is the TypeScript expression of SDD guiding principle 7 ("prefer explicit
diagnostics over silent fallback") and principle 11 ("prefer explicit world
meaning over hidden transient meaning").

### 2.3 How `auto` resolves across compiler evolution

`auto` means "lower this if you can, otherwise run it." The hazard is that the
lowerer improves over time, so the *same source* silently migrates from runtime
to lowered on some future build. That can change scheduling, numeric behavior,
error timing, side-effect ordering, debugging, and hot-reload behavior — none of
which the author asked for when they asked for flexibility.

So `auto` does not re-decide freely on every build. It resolves into a committed
**execution manifest** (see §12). The rules:

* The manifest records each authored unit's resolved mode and the reason.
* A resolved entry does not change execution mode just because the compiler got
  smarter. Migration is an **explicit, accepted change**, not a side effect.
* A compiler upgrade may *report* new opportunities, e.g.
  `door-flow is now lowerable; execution remains runtime until accepted`, but it
  does not act on them unaccepted.
* Release builds resolve against a committed manifest. A recommended policy is to
  treat unresolved or unaccepted `auto` drift as a build error in release, so
  `auto` never becomes "surprise me, compiler" in shipped artifacts.

This keeps `auto` convenient in development without letting execution strategy
wander underneath a project between compiler versions.

## 3. Anchor on the `tokimu` package, not the language

Tokimu does not try to understand arbitrary TypeScript or JavaScript. It
recognizes calls that originate from the `tokimu` authoring packages and lowers
*those*.

```ts
import { rule, query, signal } from "tokimu";
```

The import is the anchor. When the frontend sees a recognized `rule()`,
`query()`, `signal()`, `relation()`, or `command()` call, it validates the
allowed constructs and lowers them. It never attempts to interpret general host
behavior. This is closer to how JSX, LINQ, or SQL builders work than to
"general language support."

### 3.1 Recognized construct scope (v0)

The recognized construct set is small and explicit. The Rust host
(`crates/tokimu-ts-frontend`) already models these as a first-class
`TsRecognizedApiCall` enum so the lowering boundary is inspectable, not implicit.

```text
Recognized (lowerable)     Rejected in `lowered` (non-deterministic / out-of-sandbox)
--------------------       ---------------------------------------------------------
✓ rule()                   ✗ DOM: window, document
✓ query()                  ✗ async / await / Promise
✓ signal()                 ✗ Date
✓ relation()               ✗ Math.random
✓ command()                ✗ fetch
✓ deterministic loops      ✗ eval
✓ arithmetic               ✗ setTimeout / setInterval
                           ✗ ambient I/O: console, process, require, fetch, storage
```

Rejected constructs are only rejected for **lowered** rules. In **runtime** mode
an author may use more of TypeScript, subject to the runtime host's own sandbox
policy (section 8). Lowering excludes known host-level nondeterminism, so lowered
rules are *deterministic-eligible by construction* — the precise, fully accurate
claim is stated in §8.1.

### 3.2 Recognition is by resolved symbol identity

Recognition is based on the **resolved exported symbol identity** of the `tokimu`
authoring packages, not on function names or raw import text. The frontend uses
the TypeScript Compiler API's symbol resolution to decide whether a given
`rule()` / `query()` / `signal()` call actually refers to a Tokimu authoring
export.

Consequences that must hold:

* Aliased imports still work:
  `import { rule as defineBehavior } from "tokimu";` is recognized.
* A user-defined local function coincidentally named `rule` is **not** mistaken
  for the Tokimu primitive.
* Re-exports resolve transitively: `tokimu` re-exporting `@tokimu/rules` does not
  break recognition, and authored content may import a domain package directly
  if it prefers.
* `"tokimu"` is the recommended anchor, but recognition follows symbol identity,
  so the anchor is a convenience, not a textual gate.

## 4. Semantic targets and the package family

Each authoring domain targets its own Tokimu-owned semantic model. Domains do
**not** talk to each other through TypeScript; they converge through
Tokimu-owned meaning.

```text
@tokimu/rules   → tokimu-rule model        (implemented, v0)
tokimu          → umbrella re-export        (the import anchor)
@tokimu/scenes  → tokimu-scene model        (planned)
@tokimu/query   → tokimu-query model        (planned)
@tokimu/ui      → tokimu-presentation model (planned)
@tokimu/shader  → tokimu-shader model       (later)
```

Only `@tokimu/rules` and the `tokimu` anchor exist today. The others are named
here so the *shape* is agreed, not so they get built speculatively. A new domain
package earns its existence through a concrete example, exactly like an engine
crate does under the SDD.

The hard rule (SDD 5.11): these must **not** collapse into one monolithic
"Tokimu understands TypeScript" compiler. Shared *infrastructure* is fine;
shared *semantics* are not — each domain owns its own API surface and its own
lowering rules.

## 5. Architecture and boundaries

### 5.1 Two sides of the boundary

```text
  Authoring side (npm workspace: frontends/)      Engine side (cargo workspace: crates/)
  ─────────────────────────────────────────      ─────────────────────────────────────
  tokimu            (import anchor)                tokimu-ts-frontend  (host: validate + lower)
    └─ @tokimu/rules (authoring API + types)   →   tokimu-rule         (semantic rule model)
    └─ @tokimu/…     (future domains)               tokimu-runtime      (executes lowered systems)
  examples/          (authored content)             tokimu-core         (world truth; TS-unaware)
```

The arrow is one-directional: authored TypeScript lowers *into* engine-owned
semantics. Nothing on the engine side imports the TypeScript packages, and
nothing in `tokimu-core`/`tokimu-runtime` learns about TypeScript at all.

### 5.2 Rust crate boundaries (restating SDD 5.12 for this surface)

* `tokimu-core` — no TypeScript, no JS engine, no script host types. Ever.
* `tokimu-runtime` — no hard dependency on any scripting backend. It runs
  lowered systems; it does not know they came from TypeScript.
* `tokimu-rule` — the engine-owned semantic model. May predate and outlive any
  particular frontend. Depends only on core-facing concepts.
* `tokimu-ts-frontend` — owns semantic validation and lowering. Depends on
  `tokimu-rule`. Does **not** depend on the renderer, platform, or the facade.
* A future embedded runtime host (Strategy B) lives in its own crate behind a
  feature flag and is never the default.
* The facade `tokimu` crate may re-export frontend crates behind a feature flag;
  frontend crates must not depend on the facade.

### 5.3 npm workspace boundaries

* `frontends/` is a separate npm workspace, deliberately not inside `crates/`.
* Authoring packages (`tokimu`, `@tokimu/*`) contain **authoring API surface and
  types only** — no engine, no runtime, no bundler assumptions baked in.
* Authored *content* (games, scenarios, samples) lives in its own packages
  (e.g. `frontends/packages/examples`) and depends on the authoring packages,
  never the reverse.
* `tokimu-core`/`tokimu-runtime` must never gain a dependency on Node,
  TypeScript tooling, or `frontends/`.

### 5.4 Dependency direction (must hold)

```text
examples  ──▶  @tokimu/rules  ──▶  (recognized by)  tokimu-ts-frontend  ──▶  tokimu-rule
   │                                                                              │
   └────────────────────────── never ──────────────────────────────▶  tokimu-core / runtime
```

## 6. The lowering pipeline (Strategy A)

```text
author.ts
   ↓  TypeScript compiler + typechecker            (authoritative types, diagnostics)
typed AST / recognized `tokimu` API calls
   ↓  recognition pass  → TsRecognizedApiCall       (which primitives appear)
   ↓  validation        → reject unlowerable ops    (explicit, per construct)
   ↓  lowering pass      → RuleDefinition            (engine-owned intent)
tokimu-rule semantic model
   ↓  load
Runtime systems
```

Key properties:

* The compiler front (parse, typecheck, symbol resolution) is **not** Tokimu's
  job. `ts-morph` over the TypeScript Compiler API is the intended prototype
  toolchain; `Oxc` is a later option if the pipeline moves fully into Rust.
* Tokimu owns only two steps: **semantic validation** and **lowering** into
  `tokimu-rule`.
* The recognized-call model (`TsRecognizedApiCall`) makes "what lowered" an
  inspectable value, not a side effect. Tooling and tests can assert on it.

### 6.1 Current implementation status

The `tokimu-ts-frontend` host today lowers a constrained `rule(...)` object form
via a hand-rolled recognizer, emits explicit diagnostics for runtime-only or
unsupported constructs, and proves that a TS-authored lowered rule produces the
**same runtime-system plan** as the hand-written Rust rule. It does not yet use
the real TypeScript Compiler API. Replacing the hand-rolled recognizer with a
`ts-morph`-based pass is the next major slice (see the roadmap).

## 7. The runtime execution path (Strategy B, but first-class now)

Runtime TypeScript is no longer treated as a discouraged escape hatch. It is the
supported home for logic that legitimately does not need engine-owned
determinism: UI handlers, dialogue, quest flow, editor glue.

The rules that keep it safe:

* Runtime logic runs behind a **narrow, versioned host API**, never against raw
  engine objects.
* It observes and requests changes through the same world/rule/command/signal
  vocabulary that lowered rules use, so tooling can still inspect its effects.
* A rule that *requires* determinism (lockstep, replay-authoritative) is not
  allowed to be `runtime`. Feature requirements make the mode non-optional.
* The runtime host is a separate, feature-gated crate. It is never a dependency
  of `tokimu-core` or `tokimu-runtime`.

The host engine choice (QuickJS, Boa, or otherwise) is intentionally deferred.
This document fixes the *boundary*, not the *engine*.

### 7.1 Capability tiers: shared, runtime-only, lowered-only

"First-class" does not mean runtime and lowered code have identical
capabilities. Three tiers keep the difference honest:

* **Shared semantic operations** — `query`, `command`, `signal`, `relation`.
  Available to both modes and expressed through the same Tokimu-owned vocabulary,
  so tooling can inspect either mode's effects.
* **Runtime-only facilities** — timers, host callbacks, UI integration, and
  possibly async orchestration. Available only to runtime code, and only through
  the host's granted capability surface (§8.2).
* **Lowered-only guarantees** — deterministic phase ordering, replay
  eligibility, and lockstep eligibility. Available only to lowered rules, because
  only they execute as Tokimu-owned systems.

Runtime-local script state is governed separately; see §13.

## 8. Determinism and sandboxing

| Concern            | Lowered                                  | Runtime                                              |
| ------------------ | ---------------------------------------- | ---------------------------------------------------- |
| Time               | only `ctx.fixedDelta` survives lowering  | only engine-injected time is exposed by the host     |
| Randomness         | seeded, engine-owned, or rejected        | seeded, engine-owned; `Math.random` shimmed/blocked  |
| Async              | rejected at build time                   | scheduling mediated by the host, never free `Promise`|
| I/O (fs/net/DOM)   | rejected at build time                   | not exposed by the host sandbox                       |
| Determinism        | deterministic-eligible by construction   | best-effort; not a determinism guarantee              |

If authored logic needs something the lowered model cannot express, the correct
response is to **grow the semantic model deliberately**, not to quietly relax
into arbitrary runtime behavior.

### 8.1 What "deterministic-eligible" means (and doesn't)

Lowering guarantees the *absence of prohibited host behavior* — no `Date`, no
`Math.random`, no `fetch`, no ambient I/O sneaks into a lowered rule. It does
not, by itself, guarantee bit-identical execution everywhere. Overall
determinism still depends on:

* floating-point variation across targets (the SDD treats this as best-effort),
* iteration and data ordering,
* schedule ordering,
* seeded-RNG discipline,
* nondeterministic resource access,
* and, honestly, engine bugs.

So the accurate claim is: **lowered rules exclude known host-level
nondeterminism and execute within Tokimu's deterministic simulation
constraints.** They are deterministic-*eligible*; the engine's scheduling,
numeric, and data-ordering rules decide the rest.

### 8.2 The host boundary is capability-based, not a blacklist

Blocking `fetch`, `process`, and `window` is useful, but a blacklist is a
"museum of JavaScript features someone forgot existed." The runtime host's real
contract is capability-based:

> A script begins with **no authority**. Tokimu injects only explicitly granted
> capabilities.

Illustrative (v0 may not require author-declared capabilities, but the design
commits to an allowlisted surface):

```ts
runtimeAction("quest-dialogue", {
  capabilities: ["world.query", "signal.emit", "ui.open"],
  run(ctx) {
    // ctx only exposes the granted capabilities
  },
});
```

The blacklist of prohibited globals remains as defense in depth, but the primary
boundary is "nothing is available unless granted," not "everything is available
unless removed."

## 9. Native / WASM parity

* Lowered rules become plain `tokimu-rule` data. They load identically on native
  and WASM with no extra runtime — the cleanest possible browser story.
* A runtime host, if enabled, must pick a `wasm32`-capable engine (Boa fits;
  QuickJS needs a WASM build; V8-class engines do not fit a clean WASM module).
* The early WASM path avoids threads, consistent with SDD section 9.

## 10. Project structure

Current and near-term shape of the authoring workspace:

```text
frontends/
  package.json              # npm workspace root (packages/*)
  tsconfig.base.json        # shared compiler options
  README.md
  packages/
    tokimu/                 # "tokimu": umbrella import anchor, re-exports stable surface
      package.json
      tsconfig.json
      src/index.ts
    rules/                  # "@tokimu/rules": rule authoring API + types
      package.json
      tsconfig.json
      src/
        index.ts            # public surface
        execution.ts        # ExecutionMode, mirrors tokimu-rule
        primitives.ts       # rule(), query(), signal(), relation(), command(), RuleContext
        runtime.ts          # runtimeAction(), loweredRule() convenience shapes
    examples/               # authored content that consumes the authoring packages
      package.json
      tsconfig.json
      src/
        movement.rule.ts    # lowered example (parity with the Rust host)
        quest-dialogue.ts   # runtime example
```

Planned domains (`@tokimu/scenes`, `@tokimu/query`, …) slot in under
`packages/` when a concrete example needs them. Shared frontend infrastructure
(compiler integration, diagnostics, source maps, constant evaluation, lowering
interfaces) belongs to the Rust `tokimu-ts-frontend` side, not to a giant TS
mega-package.

## 11. Authoring surface and worked examples

### 11.1 Shared authoring shape with explicit execution mode

```ts
import { rule } from "tokimu";

rule("enemy-step", {
  execution: "lowered",
  inputs: ["Transform", "Velocity"],
  outputs: ["Transform"],
  signals: ["enemy-stepped"],
  run(ctx) {
    // deterministic simulation logic — lowered into Tokimu-owned semantics
  },
});

rule("quest-dialogue", {
  execution: "runtime",
  inputs: ["QuestState"],
  outputs: ["DialogueUI"],
  signals: ["dialogue-opened"],
  run(ctx) {
    // flexible orchestration — runs in the TS runtime host
  },
});
```

This object form is deliberately identical to what the Rust host already parses,
so authored samples and the engine stay in lockstep during the first draft.

### 11.2 Intent-first convenience shapes

```ts
import { loweredRule, runtimeAction } from "tokimu";

const enemyStep = loweredRule("enemy-step", {
  inputs: ["Transform", "Velocity"],
  outputs: ["Transform"],
  signals: ["enemy-stepped"],
  run(ctx) { /* ... */ },
});

const openDoor = runtimeAction("open-door", (ctx) => {
  /* ... */
});
```

### 11.3 The lowered example (`movement.rule.ts`)

Uses only recognized constructs (`query`, a deterministic loop, arithmetic) so
it lowers cleanly and matches the hand-written Rust rule's runtime-system plan.

### 11.4 The runtime example (`quest-dialogue.ts`)

Declares `execution: "runtime"` and stays orchestration-shaped; it is reported as
runtime-only rather than lowered, on purpose.

## 12. Execution manifest

`auto` resolution (§2.3), reproducible builds, and honest migration all depend on
a committed record of how each authored unit resolved. That record is the
**execution manifest**.

For each authored unit the manifest records:

* the resolved execution mode (`lowered` or `runtime`),
* the reason (especially why a `runtime` unit did not lower),
* the semantic-model version it lowered against,
* and a source hash so drift is detectable.

Illustrative:

```text
unit            mode     reason                          model  source
enemy-step      lowered  —                               v0     9f2a…
quest-dialogue  runtime  execution: "runtime" (declared) v0     11c7…
door-flow       runtime  unsupported async callback      v0     4b0e…
```

A compiler upgrade reports opportunities without acting on them:

```text
door-flow is now lowerable; execution remains runtime until accepted
```

The manifest is what makes "lower what we can" safe: improvements surface as
*proposals*, and execution only migrates when the change is accepted and the
manifest is updated. Release builds resolve against the committed manifest.

## 13. Runtime state ownership

Runtime code will inevitably want local variables:

```ts
let dialogueIndex = 0;
```

That single variable is where scripting systems start breeding hidden world state
in the walls. The governing rule:

> Ephemeral runtime-local state is allowed, but any state that affects durable
> simulation behavior must live in Tokimu-owned resources or runtime
> components.

Concretely:

* **Ephemeral / script-local** state (a loop counter, a cached lookup within one
  invocation) is fine. It is not saved, not replayed, not authoritative, and may
  be reset on reload.
* **Durable simulation** state (anything that changes what the world *is*, or
  that must survive save/load, replay, hot reload, or scene changes) must be a
  Tokimu-owned resource or runtime component, reachable through the shared
  vocabulary and inspectable by tooling.

This preserves the convenience of closures without pretending closures are a
persistence model. If a runtime script needs durable state, that is a signal to
put the state in the world, not in the script's private scope.

## 14. Source mapping and diagnostics

Authors write `.ts`; the engine runs lowered systems or hosted runtime code.
Errors must not surface in terms the author cannot map back.

* **Semantic validation errors** (e.g. "async is not allowed in a lowered rule")
  must point at the original `.ts` location, not an internal representation.
* **Lowering diagnostics** must name the construct and the rule, as the current
  host already does, and carry source position once the Compiler API path lands.
* **Runtime stack traces** from the host must map back through source maps to the
  original TypeScript, not to generated or bundled output.
* Diagnostics should reuse Tokimu's existing diagnostics model where practical,
  so authoring errors and engine diagnostics feel like one system.

## 15. Lifecycle

Even before a runtime host exists, the vocabulary for its lifecycle should be
fixed so every future host does not invent a slightly different ritual. The
stages:

* **load** — source/manifest is read and recognized; nothing runs yet.
* **initialize** — a unit is prepared with its granted capabilities (§8.2).
* **invoke** — a rule/action runs for a step or event.
* **suspend** — execution is paused without disposal (e.g. scene transition).
* **reload** — source changes are re-applied; ephemeral state (§13) may reset,
  while durable Tokimu-owned state does not.
* **dispose** — the unit is torn down and its capabilities revoked.

Lowered rules use only a degenerate slice of this (load → invoke as a system);
the full lifecycle mainly governs runtime-hosted units. Naming it now keeps the
eventual host from growing an ad hoc lifecycle of its own.

## 16. Stability and versioning of the authoring surface

Every exported symbol in `tokimu` / `@tokimu/*` is a stability obligation. The
authoring surface should stay **small and intentional**, and grow only when a
concrete example proves the need — the same discipline the SDD applies to engine
abstractions.

The `tokimu-rule` name is sufficient while this is specifically about rule
authoring. If it later becomes the home for schedules, commands, signals, and
broader execution semantics, a more general name (`tokimu-semantics`,
`tokimu-model`) may be warranted. That is a naming pressure to watch, not a
reason to rename early.

## 17. Phasing (aligned with SDD 5.11)

```text
Phase 1  Declarative scene/rule data, no scripting                 (done)
Phase 2  Semantic rule model stabilizes with real callers          (done: tokimu-rule + hello-rule-model)
Phase 3  TypeScript authoring lowers ahead of time into the model  (in progress: tokimu-ts-frontend)
Phase 4  Optional embedded JS runtime host behind a feature flag   (deferred; boundary defined, engine not chosen)
```

Runtime TypeScript being first-class does **not** move Phase 4 earlier. Phase 4
is specifically about an *embedded* JS engine inside a Tokimu binary/WASM module.
The runtime authoring mode can be served initially by an external host process or
tooling; the embedded engine is a later, gated decision.

## 18. Open questions

* How much of TypeScript's type system should the lowering pass honor, versus a
  deliberately restricted DSL-in-TS subset?
* Should lowered rules be pure transformations only, or may they emit
  commands/signals the runtime applies after the step?
* What is the minimum viable `ts-morph` recognition surface to replace the
  hand-rolled recognizer without regressing the current parity proof?
* Where does the runtime host actually live first — external process, CLI, or
  embedded — and what is the smallest honest sandbox for it?
* How is the execution manifest stored, and how is a migration "accepted" — a
  committed file, a CLI acceptance step, or a review gate?
* When are capabilities author-declared versus engine-inferred, and what is the
  minimum capability set for the first runtime host?
* When does `@tokimu/scenes` become real, and does it share a recognition pass
  with `@tokimu/rules` or stay fully independent?

## 19. Summary

TypeScript is Tokimu's primary authoring language. The engine owns a
language-agnostic semantic model; TypeScript is a frontend over it. Authoring
declares execution intent, and both **lowered** and **runtime** are first-class:
Tokimu lowers everything it can, keeps the rest runtime, and reports the boundary
explicitly instead of guessing, while `auto` resolves through a committed
execution manifest rather than migrating whenever the compiler improves. Lowered
rules are deterministic-eligible by construction; runtime logic runs behind a
narrow, capability-based host boundary, with durable state kept in Tokimu-owned
resources rather than script scope. The core engine never learns about
TypeScript, and no authoring package ever owns simulation truth.
