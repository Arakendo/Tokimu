---
title: Rust And TypeScript
description: How Tokimu separates its Rust engine from its intended TypeScript-first authoring experience.
---

<p class="page-kicker">Architecture / authoring boundary</p>

# A Rust engine with a TypeScript-first authoring surface

Tokimu is implemented primarily in Rust. Rust owns the world model, runtime,
rules, diagnostics, rendering contracts, and the semantic representations that
must remain stable across native and browser targets.

Developers building games, tools, simulations, scenarios, and content on top of
Tokimu should eventually be able to work primarily in TypeScript.

That does **not** make TypeScript a second engine.

<div class="quote-panel">
  <span>Authoring boundary</span>
  <blockquote>TypeScript supplies syntax, types, and tooling. Tokimu owns the semantics.</blockquote>
</div>

## The intended relationship

```text
TypeScript source
      ↓
Tokimu authoring packages and type checking
      ↓
domain-specific validation and lowering
      ↓
Tokimu-owned semantic model
      ↓
Rust runtime and engine services
      ↓
native, WASM, and future presentation targets
```

Applications communicate through stable Tokimu meaning rather than through
ad hoc JavaScript objects. The Rust engine does not import TypeScript packages,
and core engine crates do not learn about the DOM, Node.js, npm, or a JavaScript
runtime.

## Why TypeScript

TypeScript is the intended first high-level frontend because it offers:

- a familiar language for application and tool authors;
- a mature compiler, type checker, editor ecosystem, and package model;
- a practical fit for browser and desktop authoring tools;
- explicit domain APIs that can be recognized and lowered;
- useful diagnostics before authored intent reaches the engine.

TypeScript is a frontend choice, not the definition of Tokimu's semantic model.
Other frontends may target the same engine-owned representations later.

## Two explicit execution modes

The TypeScript design allows authored behavior to have an explicit destination:

| Mode | Intended use | Execution |
| --- | --- | --- |
| `lowered` | Deterministic simulation, replay, lockstep, portability | Compiled ahead of time into Tokimu-owned semantics; no JavaScript runtime required |
| `runtime` | UI events, dialogue, quest flow, application glue | Remains behind a narrow TypeScript/JavaScript host boundary |
| `auto` | Author permits either path | Resolves through a recorded execution manifest rather than silently changing between builds |

A `lowered` unit must either lower successfully or fail with a specific
diagnostic. Tokimu must not quietly move it into a JavaScript runtime.

## Domain packages, not arbitrary JavaScript

The intended authoring surface is a family of bounded packages:

```text
tokimu             umbrella import anchor
@tokimu/rules      rule, query, signal, relation, command
@tokimu/scenes     planned scene authoring
@tokimu/ui         planned presentation authoring
@tokimu/shader     exploratory shader authoring
```

Tokimu recognizes its own exported APIs through resolved symbol identity. It
does not attempt to interpret arbitrary TypeScript or JavaScript source.

For example, an authored rule may eventually look like:

```typescript
import { query, rule } from "tokimu";

rule("movement", {
  execution: "lowered",
  run(ctx) {
    for (const entity of query("Transform", "Velocity")) {
      const transform = entity.get<{ x: number }>("Transform");
      const velocity = entity.get<{ x: number }>("Velocity");

      entity.set("Transform", {
        ...transform,
        x: transform.x + velocity.x * ctx.fixedDelta,
      });
    }
  },
});
```

The durable output is not the callback itself. It is a validated Tokimu rule
with declared reads, writes, time policy, and execution intent.

## Current maturity

The direction is architectural and partially implemented, not yet a packaged
authoring SDK:

- the `tokimu` and `@tokimu/rules` frontend packages exist in the repository;
- `tokimu-rule` owns the language-neutral rule model;
- `tokimu-ts-frontend` provides an early validation and lowering boundary;
- corpus examples exercise lowered and runtime execution intent;
- scene, UI, and shader authoring packages remain planned or exploratory;
- a general embedded JavaScript engine is neither required nor admitted into
  the core runtime.

The public claim today is therefore **TypeScript-first authoring direction**,
not a complete TypeScript application platform.

## Website TypeScript is a different role

This website currently uses TypeScript to mount a bounded Rust/WASM evidence
island, handle browser events, select local files, and present
provider-neutral observations.

That browser adapter demonstrates the host boundary:

```text
TypeScript interaction
        ↓
Rust/WASM request
        ↓
Tokimu-owned observation
        ↓
TypeScript presentation
```

It does not yet demonstrate authored scenes or rules. TypeScript owns browser
interaction there; Rust still owns SVG parsing, vector lowering, and the
meaning returned to the page.

## Source of truth

The [Software Design Document](https://github.com/Arakendo/Tokimu/blob/main/docs/Tokimu%20Software%20Design%20Document.md)
defines the engine-owned semantic model and dependency direction. The
[TypeScript Design Document](https://github.com/Arakendo/Tokimu/blob/main/docs/Tokimu%20TypeScript%20Design%20Document.md)
defines the authoring surface, execution modes, package family, and lowering
policy.
