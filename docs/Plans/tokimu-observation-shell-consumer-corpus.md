# Tokimu Observation Shell Consumer Corpus

## Status

Proposed.

This plan investigates a Tokimu observation shell through consumer corpus
evidence. It does not admit a shell capability, command language, remote debug
protocol, editor framework, or new engine crate.

The shell depends conceptually on the bounded observation and semantic command
work in [`runtime-observation-and-command-corpus.md`](runtime-observation-and-command-corpus.md).
That plan discovers what Tokimu can expose and mutate. This plan tests how
different consumers compose, navigate, and present those contracts.

## Purpose

Not everything Tokimu believes is visible through a renderer. Worlds also
contain relationships, resources, pending commands, diagnostics, performance
state, capabilities, asset observations, and application-owned meaning.

Tokimu also intends to support consumers that may have no graphical interface:

- headless servers;
- command-line tools;
- automated tests;
- MUD and text-first applications;
- remote or SSH sessions;
- native inspection tools;
- browser/WASM dashboards.

The corpus should answer:

> Can one bounded consumer environment inspect and interact with Tokimu through
> owner-provided observations and semantic commands while remaining independent
> of terminal, GUI, browser, renderer, and transport mechanisms?

## Primary Composition Claim

```text
Tokimu-owned observation families
        |
        v
shell session and command routing
        |
        +-------------------+-------------------+
        |                   |                   |
        v                   v                   v
terminal text          structured JSON      native/web UI
        |
        v
human or automated consumer
        |
        v
semantic command request
        |
        v
owning runtime/application capability
```

The shell is a consumer and composition environment. It is not the owner of
the state it exposes.

## Architectural Thesis

> The shell makes Tokimu-owned meaning observable and addressable across
> presentation modalities. It does not become another owner of that meaning.

The shell may own:

- session lifecycle;
- command-line parsing and dispatch;
- current navigation context;
- output-format selection;
- watch subscriptions and refresh cadence;
- bounded command history;
- panel or view composition in graphical adapters;
- help and capability discovery presentation.

The shell must not own:

- world, application, asset, presentation, or diagnostic truth;
- mutation rules or application invariants;
- renderer policy or GPU resources;
- importer semantics;
- animation semantics;
- performance measurements;
- network transport;
- a browser shadow scene graph;
- arbitrary reflection over private engine state.

## Relationship To Runtime Observation And Commands

The two plans are related but not interchangeable.

### Runtime observation and command corpus owns the question

```text
What can be observed?
What commands may request mutation?
Who validates them?
When are they applied?
How are revisions and failures reported?
```

### Observation shell corpus owns the question

```text
How does a consumer discover those operations?
How are observations navigated and composed?
How are commands entered and routed?
How is output adapted to text, JSON, GUI, or MUD presentation?
How does a session remain bounded and understandable?
```

The shell must consume the runtime contracts rather than defining convenient
duplicates. Static read-only slices may begin with the existing
`WorldSnapshot` and M9 console evidence. Mutation and animation-control slices
depend on the corresponding runtime observation plan slices becoming stable.

## Existing Evidence

Tokimu already has useful shell-shaped evidence:

- M9 provides console-first world, component, relationship, asset, signal, and
  timing inspection;
- `WorldSnapshot` supports read-only structural world inspection;
- kernel performance diagnostics support bounded structured consumption;
- corpus harnesses emit deterministic summaries and artifacts;
- the Asset and Presentation Workbenches compose multiple observations in
  TypeScript without owning importer or presentation semantics;
- networking work separates observation payloads from movement mechanisms;
- `hello-fps-web` proves a browser shell can remain a read-only consumer of
  Rust-owned state;
- the runtime observation and command plan defines the candidate boundary for
  validated mutation and animation commands.

The missing evidence is whether these observations can be discovered,
navigated, presented, and commanded through a coherent non-graphical session
without creating a second runtime API.

## Shell Vocabulary

These are planning terms, not accepted public APIs.

### Shell session

An explicitly created consumer context containing navigation, output mode,
watch state, bounded history, and authority. A session does not contain a copy
of authoritative world state.

### Command catalog

A bounded owner-provided description of available query, action, and shell
meta commands. Catalog entries include names, arguments, help, authority, and
result schema information where available.

The catalog describes operations. It does not grant permission to execute
them.

### Shell meta command

A command affecting only shell/session behavior, such as:

- `help`;
- `pwd` or current context;
- `cd` or context navigation;
- `format text|json`;
- `watch` and `unwatch`;
- `history`;
- `quit`.

### Semantic query

A read-only request routed to an owning observation provider, such as:

- `inspect world`;
- `inspect entity <id>`;
- `list relationships <id>`;
- `observe diagnostics`;
- `observe performance`;
- `list animations <target>`.

### Semantic command

A mutation request routed to its application/runtime owner, such as:

- move an application-owned object;
- enable or disable an object through a named domain command;
- play, pause, seek, or reset an admitted animation playback state;
- set or clear a presentation selection/hotspot role.

The shell parses and routes. The owning subsystem validates and applies.

### Output projection

A consumer-specific representation of an observation or command result:

- plain text;
- structured JSON;
- terminal table/tree;
- MUD description;
- native UI region;
- browser panel.

Output projections do not redefine observation meaning.

## Candidate Command Flow

```text
input line or UI action
        |
        v
tokenize and parse
        |
        v
resolve shell meta command OR owner-provided catalog entry
        |
        v
validate arguments, authority, and budgets
        |
        v
query observation OR submit semantic command
        |
        v
receive typed result and diagnostics
        |
        v
select output projection
        |
        v
terminal / JSON / MUD / GUI / web
```

No parser branch receives `&mut World` or backend-native state.

## First Corpus Scenarios

### Scenario A: deterministic scripted shell

A headless session consumes a fixed command script:

```text
help
inspect world
list entities
inspect entity <known-id>
list relationships <known-id>
observe diagnostics
format json
inspect world
```

It produces deterministic text and JSON transcripts over the same underlying
observations.

### Scenario B: interactive local CLI

A small REPL provides line editing only where a platform/library already does
so cleanly. The architectural proof is parse, route, observe, and present, not
terminal emulation.

### Scenario C: mutation and animation session

Once the runtime command corpus is available, the shell:

- inspects an addressable object;
- requests one named mutation;
- observes the resulting revision;
- lists the five `hole_punch1.glb` animation clips;
- plays, pauses, seeks, advances, and resets the assembly sequence;
- shows accepted and rejected command results.

### Scenario D: text-first application or MUD

A small application-owned domain exposes:

- `look`;
- `inventory`;
- `status`;
- `go <direction>`;
- one inspect/debug command available only to an authorized session.

The MUD language remains application-owned. The shell proves routing,
observation, session, and output composition without baking room, player, or
inventory semantics into Tokimu.

### Scenario E: graphical observation adapter

A later native or browser consumer presents the same catalogs, observations,
and results through panels and controls. It must not become the reference
semantics merely because it is richer visually.

## Command Namespace And Discovery

The initial corpus should keep namespaces explicit:

```text
shell.*          session and output behavior
world.*          structural world queries
app.*            application-owned queries and commands
diagnostics.*    diagnostic observations
performance.*    performance observations
assets.*         asset observations
presentation.*   target and override observations/commands
animation.*      playback observations/commands
```

User-facing aliases may be shorter, but artifacts should retain the resolved
owner-qualified identity.

Discovery should answer:

- what commands exist;
- which owner supplied each command;
- whether the current session may invoke it;
- argument names and bounded types;
- whether it is a query, command, or shell-meta operation;
- its result/observation kind;
- whether the operation is currently unavailable or unsupported.

Discovery must not become arbitrary Rust reflection.

## Session And Authority Model

Every session should have explicit:

- stable session identity for its lifetime;
- local, scripted, browser, MUD, or future remote origin;
- authority/capability set;
- current navigation context;
- output projection;
- command and observation budgets;
- watch cadence and limits;
- bounded history;
- open/closed lifecycle.

The first proof may use one local read-only authority and one local control
authority. Authentication, users, roles, and remote security remain deferred.

Read-only access and mutation authority are separate. Discovering a command
does not imply permission to execute it.

## Watch And Streaming Semantics

`watch` should be modeled as repeated bounded observations, not an ambient live
reference.

The first proof should define:

- observation kind and arguments;
- refresh cadence;
- maximum active watches;
- update sequence and revision;
- unchanged-result behavior;
- truncation and backpressure diagnostics;
- explicit cancellation.

Terminal refresh, server push, browser callbacks, and network transport remain
adapter concerns.

## Evidence Artifacts

Each deterministic shell run should emit:

```text
tokimu-observation-shell/
    manifest.json
    command-catalog.json
    input-script.txt
    command-trace.json
    transcript.txt
    transcript.json
    session-summary.json
    diagnostics.json
```

Artifacts should record:

- schema and artifact versions;
- owner-qualified command identity;
- parsed arguments without unbounded secret or binary data;
- session authority and output mode;
- observation sequence, tick, and revision where applicable;
- command acceptance/rejection and resulting revision;
- output projection identity;
- truncation, unavailable, and unsupported diagnostics;
- deterministic hashes for scripted runs.

Text output is authoritative only for the text projection. The typed
observation and command result remain authoritative for engine meaning.

## Implementation Location

Incubate shared consumer support under:

```text
corpus/lib/observation-shell/
```

Exercise it first through:

```text
corpus/hello-observation-shell/
```

Add independent consumers only after the scripted proof:

```text
corpus/consumers/<tokimu-mud-or-text-consumer>/
corpus/consumers/<native-observation-workbench>/
corpus/consumers/<wasm-observation-workbench>/
```

Names may be refined during implementation. Do not create `tokimu-shell` until
independent consumers prove irreducible semantics beyond application
composition.

## Implementation Slices

### Slice 0: Boundary Inventory And Corpus Freeze

Deliverables:

- [ ] Inventory existing M9 text inspection, diagnostics, performance, asset,
      presentation, and runtime observation outputs.
- [ ] Classify existing commands and queries by semantic owner.
- [ ] Freeze the first deterministic command script and expected observation
      families.
- [ ] Define shell-owned session/meta behavior and explicit non-ownership.
- [ ] Record dependencies on runtime observation plan slices.

Acceptance criteria:

- [ ] The shell is described as a consumer, not a runtime owner.
- [ ] Every command and observation has an owner outside the shell unless it is
      explicitly session-local.
- [ ] CLI, MUD, native UI, browser, and remote transport are presentation or
      mechanism adapters rather than separate semantic shells.
- [ ] No new crate or stable API is required.

### Slice 1: Deterministic Read-Only Script Runner

Deliverables:

- [ ] Create `hello-observation-shell` with a fixed input script.
- [ ] Route existing world, relationship, diagnostic, performance, and asset
      observations through explicit owner-qualified commands.
- [ ] Emit plain-text and structured JSON projections.
- [ ] Add help and command discovery from a bounded catalog.
- [ ] Record parse, dispatch, observation, and projection failures separately.

Acceptance criteria:

- [ ] The script runs without window, GPU, terminal interactivity, or network.
- [ ] Repeated runs produce identical semantic and transcript artifacts.
- [ ] Text and JSON project the same observations without changing meaning.
- [ ] Unknown commands and invalid arguments are explicit failures.
- [ ] The runner receives no mutable `World` access.

### Slice 2: Navigation And Session State

Deliverables:

- [ ] Add explicit shell session creation and closure.
- [ ] Add current context and bounded navigation over observation identities.
- [ ] Add output mode, bounded history, and clear/reset meta commands.
- [ ] Ensure navigation context expires safely when targets disappear.
- [ ] Emit a deterministic session summary.

Acceptance criteria:

- [ ] Session state contains no authoritative world or asset copy.
- [ ] Stale navigation targets produce diagnostics rather than accidental
      fallback.
- [ ] Closing a session releases its watches/history without global state.
- [ ] Two sessions may hold different context and output modes over the same
      authoritative observations.

### Slice 3: Command Catalog And Typed Argument Validation

Deliverables:

- [ ] Describe commands, queries, arguments, result kinds, ownership, and
      authority through bounded catalog entries.
- [ ] Parse strings into typed invocation values without arbitrary reflection.
- [ ] Distinguish shell meta commands, semantic queries, and mutation commands.
- [ ] Add deterministic diagnostics for unknown, duplicate, ambiguous,
      unavailable, and unauthorized operations.

Acceptance criteria:

- [ ] Catalog discovery does not grant mutation authority.
- [ ] Invalid input cannot reach an owning command handler.
- [ ] Owner-qualified identities remove cross-domain naming ambiguity.
- [ ] Application commands can be registered without modifying the shell
      parser's world logic.

### Slice 4: Watch And Bounded Refresh

Deliverables:

- [ ] Add watch, unwatch, and list-watch meta commands.
- [ ] Poll bounded observations at explicit fixed or application-supplied
      cadence.
- [ ] Record sequence, revision, unchanged results, truncation, and cancellation.
- [ ] Add limits for active watches, result size, and refresh rate.

Acceptance criteria:

- [ ] A watch never exposes a live reference to engine state.
- [ ] Slow projections or consumers cannot create an unbounded queue.
- [ ] Cancellation is deterministic and observable.
- [ ] Terminal repaint or browser push remains outside the watch contract.

### Slice 5: Semantic Mutation And Animation Commands

Dependencies:

- runtime observation and command corpus Slices 2 through 4.

Deliverables:

- [ ] Route one application-owned object mutation through the bounded command
      contract.
- [ ] Display command validation result, applied tick, and resulting revision.
- [ ] Expose `hole_punch1.glb` clip discovery and playback commands.
- [ ] Add selected/hotspot presentation commands without mutating source asset
      or simulation truth.
- [ ] Keep shell parsing separate from command validation and application.

Acceptance criteria:

- [ ] Editing observation output cannot mutate Tokimu state.
- [ ] Stale, unauthorized, unsupported, and rejected commands remain unchanged
      state transitions with explicit results.
- [ ] Successful mutation is confirmed by a later observation revision.
- [ ] Animation and presentation meanings remain owned by their respective
      providers/capabilities, not the shell.

### Slice 6: Interactive Local CLI Adapter

Deliverables:

- [ ] Add a minimal local REPL adapter over the scripted session contract.
- [ ] Preserve piped/script input and non-interactive execution.
- [ ] Keep line editing, terminal colors, clear-screen behavior, and signals in
      a replaceable adapter.
- [ ] Add clean EOF, interrupt, and shutdown behavior.

Acceptance criteria:

- [ ] All semantic behavior remains testable without a real terminal.
- [ ] Piped input and interactive input resolve to identical invocations.
- [ ] Terminal capability absence degrades explicitly to plain text.
- [ ] CLI shutdown does not alter runtime truth or leave hidden global state.

### Slice 7: MUD And Text-First Consumer

Deliverables:

- [ ] Build a small application-owned room/player scenario.
- [ ] Register `look`, `status`, `inventory`, and `go` as application commands.
- [ ] Project application observations into readable room and status text.
- [ ] Add a separately authorized inspect/debug command.
- [ ] Capture one deterministic transcript of a complete interaction.

Acceptance criteria:

- [ ] Room, player, inventory, and movement semantics remain application-owned.
- [ ] The same shell/session machinery supports both debug and domain commands
      without confusing their authority.
- [ ] The MUD consumer requires no renderer or GUI.
- [ ] A different text projection can be supplied without changing application
      state or command meaning.

### Slice 8: Native Or Browser Observation Workbench

Deliverables:

- [ ] Adapt the command catalog and observations into a native or WASM UI.
- [ ] Present command discovery, navigation, watch state, diagnostics, and
      command results through panels and controls.
- [ ] Keep view layout and interaction adapter-owned.
- [ ] Compare its semantic command trace with the scripted shell trace.

Acceptance criteria:

- [ ] The graphical adapter introduces no new observation or mutation meaning.
- [ ] GUI controls resolve to the same owner-qualified commands as CLI input.
- [ ] TypeScript does not own a shadow runtime or parse provider formats.
- [ ] The graphical consumer remains optional and replaceable.

### Slice 9: Authority, Budgets, And Adversarial Input

Deliverables:

- [ ] Add read-only and control session authorities.
- [ ] Bound input length, argument count, output size, history, watches, and
      command rate.
- [ ] Test malformed Unicode, oversized input, unknown targets, stale context,
      command flooding, and projection failures.
- [ ] Redact or omit sensitive/unbounded values according to owner policy.

Acceptance criteria:

- [ ] Malformed input cannot panic or partially mutate state.
- [ ] Read-only sessions cannot invoke mutation commands.
- [ ] Budget failures identify the owning boundary and preserve session
      usability.
- [ ] Help/catalog output does not reveal unavailable private implementation
      details.

### Slice 10: Architectural Review

Deliverables:

- [ ] Compare scripted, CLI, MUD, and graphical consumer evidence.
- [ ] Record repeated session, catalog, routing, watch, and projection
      semantics.
- [ ] Decide whether the shell remains an application pattern, corpus library,
      first-party tool, or admitted capability.
- [ ] Open an Architectural Review for Application Observation Shell ownership
      before creating `tokimu-shell`.
- [ ] Update SDD, roadmap, ADRs, and consumer corpus records with observed
      findings.

Acceptance criteria:

- [ ] Any promotion proposal names irreducible semantics shared by at least two
      independent presentation modalities.
- [ ] Convenience, a common command syntax, or a large UI is not treated as
      capability evidence by itself.
- [ ] Observation and mutation ownership remains with existing domains.
- [ ] Deferred remote, editor, persistence, and security questions remain
      explicit.

## Failure Semantics

| Boundary | Example failure |
| --- | --- |
| Input adapter | invalid encoding, EOF, interrupt, oversized line |
| Parser | malformed syntax, invalid argument type |
| Catalog | unknown, ambiguous, unavailable operation |
| Authority | operation discoverable but not permitted |
| Session | stale context, closed session, history/watch limit |
| Observation | unavailable, truncated, unsupported schema |
| Command | rejected, stale revision, unknown target, queue full |
| Projection | unsupported format or bounded rendering failure |
| Output adapter | terminal/browser/stream unavailable or backpressured |

No failure may silently become an empty successful result or direct mutation.

## Non-Goals

- A general operating-system shell.
- A full editor or IDE.
- A universal debug protocol.
- Raw ECS reflection or arbitrary component mutation.
- A stable textual scripting language.
- Remote transport, authentication, encryption, or multi-user administration.
- Persistent command history or event sourcing.
- Terminal emulator implementation.
- Renderer/GPU inspection.
- Importing, authoring, or mutating source assets through shell shortcuts.
- Replacing application-owned MUD or gameplay command semantics.
- Creating `tokimu-shell` before Architectural Review.

## Risks

### Shell Becomes A Second Runtime

Mitigation: the shell owns only session, routing, discovery, and projection.
Owners continue to produce observations and validate commands.

### String Commands Become Engine API

Mitigation: strings parse into typed owner-qualified invocations. Engine
contracts do not depend on terminal syntax.

### Observation Composition Becomes Universal State

Mitigation: preserve owner-labeled observation families and compose them only
inside consumer views and transcripts.

### Shell Becomes An Editor

Mitigation: admit mutation commands one at a time through existing semantic
owners. Importing, authoring, and arbitrary property editing remain outside the
first corpus.

### GUI Becomes The Reference Implementation

Mitigation: make the deterministic headless script and typed artifacts the
baseline. GUI evidence is an independent projection.

### MUD Semantics Leak Into Tokimu

Mitigation: Tokimu provides observation, routing, authority, and lifecycle
pressure only. Rooms, players, movement verbs, and prose remain application
meaning.

### Remote Debugging Is Assumed

Mitigation: keep transport outside the shell contract. A future remote adapter
must use the networking boundary and requires separate authority/security
review.

## Validation

Focused validation should cover:

- deterministic parse and dispatch;
- text/JSON projection equivalence;
- command catalog discovery and namespace resolution;
- independent multi-session navigation and output modes;
- stale identity and session closure;
- watch cadence, cancellation, budgets, and backpressure;
- read-only versus control authority;
- runtime command result and revision confirmation;
- scripted versus interactive CLI parity;
- MUD transcript determinism;
- CLI versus GUI semantic trace parity.

Workspace validation remains:

```text
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Completion Criteria

This effort reaches a useful pause when:

- one deterministic headless shell composes existing observations;
- plain-text and JSON projections expose the same meaning;
- command discovery, namespaces, typed arguments, and session authority are
  explicit;
- bounded watch behavior is proven;
- mutation and animation requests pass through the runtime command seam rather
  than shell-owned shortcuts;
- one local CLI and one MUD/text-first consumer reuse the session boundary;
- one graphical adapter proves presentation independence;
- artifacts localize failures across input, parse, catalog, authority,
  observation/command, projection, and output boundaries;
- an Architectural Review decides whether any shell concept deserves
  first-party admission.

## Graduation Triggers

Consider a first-party shell capability only when:

- at least two independent modalities need the same session, discovery,
  routing, and watch semantics;
- the shared contract remains useful headlessly;
- command syntax and output adapters remain replaceable;
- world, application, asset, presentation, diagnostic, and animation ownership
  do not leak into the shell;
- extraction removes demonstrated duplication or enables a real consumer;
- an Architectural Review accepts the ownership boundary.

A first-party observation tool may still be justified even if no shell
capability is admitted. Product/tool usefulness and architectural admission are
separate decisions.

## References

- [`Tokimu Shell.md`](../Conversations/Tokimu%20Shell.md)
- [`runtime-observation-and-command-corpus.md`](runtime-observation-and-command-corpus.md)
- [`consumer-corpora.md`](consumer-corpora.md)
- [`performance-diagnostics-and-runtime-observation.md`](performance-diagnostics-and-runtime-observation.md)
- [`networking-and-transport.md`](networking-and-transport.md)
- [`Tokimu Software Design Document.md`](../Tokimu%20Software%20Design%20Document.md)
- [`kernel-principles.md`](../kernel-principles.md)
- [`AR-0005-runtime-observation-and-performance-telemetry.md`](../Architectural%20Reviews/AR-0005-runtime-observation-and-performance-telemetry.md)
- `crates/tokimu-core/src/world.rs`
- `corpus/lib/performance-diagnostics-corpus/README.md`
- `corpus/consumers/aspnet-wasm-asset-workbench/DESIGN.md`
- `corpus/consumers/aspnet-wasm-presentation-workbench/DESIGN.md`
