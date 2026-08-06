# Tokimu Observation Shell Consumer Corpus

## Status

Implementation evidence is complete for the bounded browser/WASM consumer.
`AR-0013` remains incubating for the broader provider-admission and cross-host
session questions.

This plan investigates a Tokimu observation shell through consumer corpus
evidence. It does not admit a shell capability, command language, remote debug
protocol, editor framework, or new engine crate.

The shell depends conceptually on the bounded observation and semantic command
work in [`runtime-observation-and-command-corpus.md`](runtime-observation-and-command-corpus.md).
That plan discovers what Tokimu can expose and mutate. This plan tests how
different consumers compose, navigate, and present those contracts.

Current corpus implementation covers deterministic read-only routing, bounded
session navigation, owner-qualified command discovery, bounded watches, and
caller-owned application query and mutation adapters. The shell projects a
typed compact query result or mutation receipt and a later owner observation;
it never receives mutable runtime state or validates application arguments
itself. `hole_punch1.glb` clip discovery is now proven through that query
boundary. Playback lifecycle and presentation-selection commands remain
separate evidence gaps.

## Architectural Review Relationship

This plan gathers the evidence requested by
[`AR-0013`](../Architectural%20Reviews/AR-0013-observation-shell-and-ratatui-presentation-provider.md).
It does not yet admit an Observation Shell capability, stable shell crate, or
official Ratatui provider.

The candidate boundary is:

```text
owner-provided observations and commands
        -> provider-neutral shell session
        -> replaceable projection provider
```

Ratatui is the preferred first provider study in two explicit modes:

- an independent terminal host that owns terminal input and output mechanics;
- an embedded normalized-cell projection whose containing Tokimu UI host owns
  outer layout, clipping, focus arbitration, and neighboring content.

The same semantic session and command results must survive both modes before
provider admission is considered.

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

## Current Boundary Inventory

The shell consumes the following owner-qualified observation families. It does
not merge them into a generic state object or reinterpret their fields.

| Family | Semantic owner | Current shell status | Dependency evidence |
| --- | --- | --- | --- |
| World and relationships | simulation and runtime | routed through the deterministic headless script | `WorldSnapshot` and runtime observation Slice 1 |
| Application commands | scenario application | routed; the application validates arguments and applies mutations | runtime observation Slice 2 |
| Animation and playback | application playback policy over imported clip metadata | routed through the runtime browser shell catalog | runtime observation Slice 3 |
| Asset identity and imported metadata | importer and asset services | not routed by the shell yet | runtime observation Slice 4 and asset corpus work |
| Presentation target resolution | presentation semantics | routed by the runtime browser shell catalog | runtime observation Slice 4 |
| Diagnostics | diagnostic producers and diagnostic policy | routed as bounded copied observations in the read-only script | performance diagnostics Slices 1-4 |
| Performance and resource lifecycle metrics | producing capability, runtime integration, and kernel diagnostics | intentionally not routed yet | performance diagnostics Slices 3-7 |

Shell-owned state is limited to session identity, current observation context,
projection choice, bounded history, watch registration, and command routing.
The same command catalog can be projected through a CLI, MUD, native UI,
browser, or later transport adapter; none of those adapters become a second
semantic shell.

The runtime observation corpus establishes the semantic contracts that the
shell consumes. In particular, shell mutation and playback scenarios depend on
its validated application command boundary and fixed-step playback evidence.
Asset and performance commands remain deferred until their owners expose
bounded, consumer-ready observation contracts. This temporary omission is
intentional and is not a capability gap that the shell may fill itself.

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

A Ratatui projection may be hosted directly in a terminal or lowered through a
bounded normalized styled-cell buffer into another presentation host. Ratatui
widget state, terminal events, and provider crate types remain below this
projection boundary and never become shell-session meaning.

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

- [x] Inventory existing M9 text inspection, diagnostics, performance, asset,
      presentation, and runtime observation outputs.
- [x] Classify existing commands and queries by semantic owner.
- [x] Freeze the first deterministic command script and expected observation
      families.
- [x] Define shell-owned session/meta behavior and explicit non-ownership.
- [x] Record dependencies on runtime observation plan slices.

Acceptance criteria:

- [x] The shell is described as a consumer, not a runtime owner.
- [x] Every command and observation has an owner outside the shell unless it is
      explicitly session-local.
- [x] CLI, MUD, native UI, browser, and remote transport are presentation or
      mechanism adapters rather than separate semantic shells.
- [x] No new crate or stable API is required.

### Slice 1: Deterministic Read-Only Script Runner

Deliverables:

- [x] Create `hello-observation-shell` with a fixed input script.
- [x] Route the existing world, relationship, and diagnostic observations
      through explicit owner-qualified commands.
- [x] Defer performance and asset observations until their owners publish
      bounded consumer-ready contracts; `AR-0013` retains this as admission
      evidence rather than treating it as a blocker for the runtime-focused
      consumer proof.
- [x] Emit plain-text and structured JSON projections.
- [x] Add help and command discovery from a bounded catalog.
- [x] Record parse, dispatch, observation, and projection failures separately.

Acceptance criteria:

- [x] The script runs without window, GPU, terminal interactivity, or network.
- [x] Repeated runs produce identical semantic and transcript artifacts.
- [x] Text and JSON project the same observations without changing meaning.
- [x] Unknown commands and invalid arguments are explicit failures.
- [x] The runner receives no mutable `World` access.

Current evidence:

- `corpus/lib/observation-shell` owns only bounded read-only session state,
  command routing, projection selection, and history.
- `corpus/hello-observation-shell` freezes the first script over copied world,
  relationship, and diagnostics observations.
- `runtime-observation-workbench` additionally exposes `application runtime
  world-summary`, `application runtime relationships`, and `application
  diagnostics records` through the same Rust-owned shell boundary. These are
  catalog entries over copied owner observations, not a duplicate browser
  schema.
- Performance and asset owners are intentionally not routed yet. Their
  contracts must be supplied by their owning corpus work before this shell
  catalog expands; the shell does not synthesize availability or metric data.

### Slice 2: Navigation And Session State

Deliverables:

- [x] Add explicit shell session creation and closure.
- [x] Add current context and bounded navigation over observation identities.
- [x] Add output mode, bounded history, and clear/reset meta commands.
- [x] Ensure navigation context expires safely when targets disappear.
- [x] Emit a deterministic session summary.

Acceptance criteria:

- [x] Session state contains no authoritative world or asset copy.
- [x] Stale navigation targets produce diagnostics rather than accidental
      fallback.
- [x] Closing a session releases its watches/history without global state.
- [x] Two sessions may hold different context and output modes over the same
      authoritative observations.

Current evidence:

- `ObservationShell` is an explicitly open, session-local object. It owns a
  bounded transcript, output projection choice, current observation context,
  and a bounded back stack; it never retains an authoritative `World`, asset,
  or live owner reference.
- `select entity <id>` validates the identity against the supplied owner
  observation before changing context. If a later refreshed observation no
  longer contains the selected entity, the shell returns an owner-qualified
  stale-context failure and does not fall back to another target.
- `close` clears local transcript and navigation state, returns one final
  closure report without retaining it as history, and rejects later commands
  until a consumer creates a new shell session.
- Unit coverage proves independent sessions can use different selected context
  and projection formats over the same copied observation source.

### Slice 3: Command Catalog And Typed Argument Validation

Deliverables:

- [x] Describe commands, queries, arguments, result kinds, ownership, and
      authority through bounded catalog entries.
- [x] Parse strings into typed invocation values without arbitrary reflection.
- [x] Distinguish shell meta commands, semantic queries, and mutation commands.
- [x] Add deterministic diagnostics for unknown, duplicate, ambiguous,
      unavailable, and unauthorized operations.

Acceptance criteria:

- [x] Catalog discovery does not grant mutation authority.
- [x] Invalid input cannot reach an owning command handler.
- [x] Owner-qualified identities remove cross-domain naming ambiguity.
- [x] Application commands can be registered without modifying the shell
      parser's world logic.

Current evidence:

- `CommandDescription` records command owner, kind, named arguments, expected
  result kind, and availability. Built-in commands identify catalog,
  observation, or session-local results; application mutations identify a
  future mutation receipt without exposing an execution mechanism.
- `ApplicationCommandInvocation` parses only the stable
  `application <owner> <command> [arguments...]` envelope. Application-specific
  argument interpretation remains outside the shell.
- Registration rejects normalized duplicate owner-and-command identities, so an
  ambiguous application dispatch cannot enter routing. Unknown identities are
  unavailable; registered queries are unavailable until an owner adapter is
  attached; registered mutations are explicitly unauthorized.
- No application handler exists in this slice. That makes the boundary
  testable: malformed input and catalog discovery cannot reach mutable runtime
  state or accidentally acquire mutation authority.

### Slice 4: Watch And Bounded Refresh

Deliverables:

- [x] Add watch, unwatch, and list-watch meta commands.
- [x] Poll bounded observations at explicit application-supplied logical
      cadence.
- [x] Record sequence, revision, unchanged results, truncation, and cancellation.
- [x] Add limits for active watches, result size, and refresh rate.

Acceptance criteria:

- [x] A watch never exposes a live reference to engine state.
- [x] Slow projections or consumers cannot create an unbounded queue.
- [x] Cancellation is deterministic and observable.
- [x] Terminal repaint or browser push remains outside the watch contract.

Current evidence:

- `ObservationShell` supports `watch world [interval]`, `watch diagnostics
  [interval]`, `list watches`, and `unwatch <id>`. It retains only copied,
  fixed-size summary fingerprints, never a `World`, `Diagnostics`, or other
  live owner reference.
- The application calls `refresh_watches(source, sequence)` with its own
  monotonically chosen logical observation sequence. There is no shell timer,
  worker, transport, callback registration, or repaint loop. Missed sequences
  coalesce into one refresh at the next supplied sequence.
- V1 allows at most eight active watches per shell session. Each refresh is a
  fixed-size world or diagnostics summary, so v1 result truncation is always
  explicitly `false`; richer list watches require a later bounded-payload
  contract rather than silently retaining arbitrary result streams.
- Refresh interval is a positive logical-sequence interval, not wall-clock
  time. Consumers own their physical polling, browser push, terminal repaint,
  and backpressure policy around this synchronous result.
- `close` releases watches and their fingerprints. Explicit `unwatch` returns
  the released subscription, so cancellation is visible and deterministic.

Deferred evidence:

- Full observation snapshots, pagination, and payload truncation semantics.
- Persistent subscription recovery, remote delivery, and permission changes.
- Adapter-local repaint scheduling, browser/server push, or background work.

### Slice 5: Semantic Mutation And Animation Commands

Dependencies:

- runtime observation and command corpus Slices 2 through 4.

Deliverables:

- [x] Route one application-owned object mutation through the bounded command
      contract.
- [x] Display command validation result, applied tick, and resulting revision.
- [x] Expose `hole_punch1.glb` clip discovery through an owner-owned query.
- [x] Expose an initial `hole_punch1.glb` playback lifecycle path: select a
      clip, advance one fixed step, and observe the resulting state.
- [x] Extend playback coverage to pause, resume, stop, seek, reset, and their
      explicit rejected or unsupported outcomes.
- [x] Add one selected presentation command without mutating source asset or
      simulation truth.
- [x] Add hotspot and selection-clearing commands with the same independent
      ownership proof.
- [x] Keep shell parsing separate from command validation and application.

Acceptance criteria:

- [x] Editing observation output cannot mutate Tokimu state.
- [x] Rejected lifecycle commands remain unchanged state transitions with
      explicit results.
- [x] Successful mutation is confirmed by a later observation revision.
- [x] Animation discovery remains owned by the runtime scenario rather than
      the shell.
- [x] The initial playback command and state query remain owned by the runtime
      scenario rather than the shell.
- [x] The initial selected presentation target remains owned by the scenario
      provider/capability, not the shell.
- [x] Presentation commands resolve or reject scenario-owned targets without
      teaching the shell target construction or mapping rules.

Current evidence:

- `ObservationShell::execute_with_mutation_handler` accepts only a
  caller-supplied handler over an already-parsed, owner-qualified application
  invocation. The shell preserves its existing catalog, authority, and parsing
  boundaries; it does not acquire `World`, runtime, or presentation access.
- `hello-observation-shell` registers `application runtime set-enabled` and
  translates it in the executable consumer into the runtime corpus's existing
  `CommandRequest`. The runtime validates, queues, and applies the request at
  its own explicit tick.
- The returned `ApplicationMutationReceipt` records only the bounded result:
  acceptance, applied tick, resulting revision, and a diagnostic message. A
  fresh runtime observation then confirms revision `1` after the accepted
  mutation. Unit coverage also proves unavailable application commands do not
  reach the mutation handler.
- `browser_shell_observation_json_is_not_runtime_state` mutates the decoded
  browser-facing playback projection, then requests the same owner-qualified
  query again. The later record preserves the scenario-owned summary and
  fields, proving that the browser edits copied output rather than a mutable
  runtime reference.
- `ObservationShell::execute_with_application_handler` extends the same
  owner-routing seam to caller-owned read-only query results without giving the
  shell a runtime catalog or asset data. It distinguishes a query result from a
  mutation receipt and records an explicit owner failure if the handler returns
  the wrong result kind.
- `hello-observation-shell` registers `application runtime list-animations`.
  Its runtime handler returns the five `hole_punch1.glb` clip names, durations,
  translation-channel counts, and target nodes as a compact caller-owned
  observation. The shell only parses, routes, and projects that observation.
- The same consumer registers `application runtime play`, `advance`, and
  `playback`. Clip `1` (`step2`) is selected by the runtime adapter, advances
  exactly one fixed scenario step to `0.017s`, and is then returned through a
  caller-owned playback-state query. The shell receives neither GLB bytes nor
  playback internals; it projects the application result only.
- The lifecycle catalog now also routes `pause`, `resume`, `seek <seconds>`,
  `stop`, and `reset` through the same application handler. The scenario owns
  command grammar, numeric parsing, validation, state transition, and
  diagnostics; the shell owns only the bounded owner-qualified envelope and
  resulting receipt projection. The deterministic script exercises every
  command, including `pause` after `stop`, which preserves the stopped state
  and returns the runtime diagnostic `pause_not_playing`.
- The fixed script also requests absent clip `99`. The runtime owns the
  rejection and returns an explicit failed receipt without changing the
  selected playback state; the shell does not infer validity from catalog data.
- `application runtime select-arm` invokes the scenario adapter's existing
  presentation command. The later `application runtime presentation` query
  returns only the scenario-owned resolved target identity; the shell does not
  receive source GLB bytes, mutate simulation state, or resolve presentation
  mappings itself.
- The same consumer now routes `set-arm-hotspot`, `clear-arm-selection`, and
  `clear-arm-hotspot` through narrow scenario adapter methods. Each resolves
  the arm mapping inside the runtime scenario before applying the existing
  presentation command; the shell receives only an accepted receipt and the
  provider-neutral target identity.
- `select-missing-target` exercises the scenario's explicit unknown-target
  path. It returns `RejectedUnknownTarget` with the
  `presentation_target_unresolved` diagnostic while keeping mapping and target
  interpretation scenario-owned. The shell neither constructs raw renderer
  handles nor tries to decide whether a target should exist.

Deferred evidence:

- Independent host-session parity and adapter-level presentation controls;
  this slice proves owner routing, not a general presentation editor.

### Slice 6: Interactive Local CLI Adapter

Deliverables:

- [x] Add a minimal local REPL adapter over the scripted session contract.
- [x] Preserve piped/script input and non-interactive execution.
- [x] Keep line editing, terminal colors, clear-screen behavior, and signals in
      a replaceable adapter.
- [x] Add clean EOF, interrupt, and shutdown behavior.

Acceptance criteria:

- [x] All semantic behavior remains testable without a real terminal.
- [x] Piped input and interactive input resolve to identical invocations.
- [x] Terminal capability absence degrades explicitly to plain text.
- [x] CLI shutdown does not alter runtime truth or leave hidden global state.

Current evidence:

- `hello-observation-shell-cli` is a standard-input/plain-output adapter. It
  deliberately has no terminal line editor, ANSI palette, clear-screen
  behavior, signal handler, or clock; those are replaceable host concerns.
- Its interactive loop and piped input both call the same
  `CliSession::execute_line` method. Unit coverage compares the resulting
  projections for `help`, `inspect world`, and `list entities`, and confirms
  blank lines do not create shell history.
- EOF and the explicit local `exit`/`quit` spellings end only the adapter
  process. The adapter owns its fixture `World`, `Diagnostics`, and shell
  session directly, so shutdown does not alter another runtime or leave a
  hidden global session.
- `CliTermination` makes `EndOfFile`, `ExitCommand`, and `InterruptedInput`
  explicit adapter outcomes. The reader/writer loop is unit-tested with a
  bounded in-memory input source, including the guarantee that lines following
  `exit` are not routed to the shell. Host-specific control-C interception,
  line editing, terminal styling, and clear-screen behavior remain deliberately
  replaceable terminal concerns rather than shell semantics.
- `hello-observation-shell-ratatui`, enabled by the optional
  `ratatui-standalone` feature, is a second native terminal adapter. Its
  `TerminalSession` owns prompt editing, history navigation, transcript
  viewport input, raw-mode lifecycle, and Ratatui styling while dispatching
  every submitted line through the same `ShellFixture` and `ObservationShell`.
  Focused tests cover command submission, transcript projection, prompt
  history, adapter-local Page Up/Down/Home/End and mouse-wheel transcript
  navigation, resize-time viewport clamping, and adapter-local `exit`;
  command submission returns the viewport to live output. The current scroll
  model tracks display-width visual rows for ordinary wrapped transcript text,
  including wide characters. It remains an adapter-owned approximation: exact
  Ratatui line breaking for richer styled spans, grapheme segmentation, and
  physical clipping remains later host evidence. The native adapter places a
  bounded prompt cursor inside its input region; full grapheme-aware cursor
  editing remains later host evidence. Raw mode and alternate-screen
  acquisition use separate cleanup
  guards so a failed alternate-screen setup still restores raw mode; mouse
  capture and the alternate screen are restored after they are acquired.
  Neither `tokimu-core` nor
  `tokimu-runtime` depends on Ratatui or Crossterm.

### Slice 7: MUD And Text-First Consumer

Deliverables:

- [x] Build a small application-owned room/player scenario.
- [x] Register `look`, `status`, `inventory`, and `go` as application commands.
- [x] Project application observations into readable room and status text.
- [x] Add a separately authorized inspect/debug command.
- [x] Capture one deterministic transcript of a complete interaction.

Acceptance criteria:

- [x] Room, player, inventory, and movement semantics remain application-owned.
- [x] The same shell/session machinery supports both debug and domain commands
      without confusing their authority.
- [x] The MUD consumer requires no renderer or GUI.
- [x] A different text projection can be supplied without changing application
      state or command meaning.

Current evidence:

- `hello-observation-mud` supplies an application-owned Atrium/Archive
  scenario with `look`, `status`, `inventory`, and `go` commands. The command
  handler retains room state and movement validation; the shell retains no MUD
  state.
- `mud debug` is discoverable but rejected by an unprivileged scenario and
  accepted only by a separately configured scenario-owned inspect capability.
  The shell routes the same owner-qualified request in both cases and grants
  neither authority nor access itself.
- The deterministic transcript demonstrates movement, rejected movement,
  denied debug access, and a JSON projection of the same room status.

### Slice 8: Native Or Browser Observation Workbench

Deliverables:

- [x] Adapt the command catalog and observations into a native or WASM UI.
- [x] Present command discovery, navigation, watch state, diagnostics, and
      command results through panels and controls.
- [x] Keep view layout and interaction adapter-owned.
- [x] Compare its semantic command trace with the scripted shell trace.

Acceptance criteria:

- [x] The graphical adapter introduces no new observation or mutation meaning.
- [x] GUI controls resolve to the same owner-qualified commands as CLI input.
- [x] TypeScript does not own a shadow runtime or parse provider formats.
- [x] The graphical consumer remains optional and replaceable.

Current evidence:

- `hello-observation-workbench` is a native `ui-tools` consumer whose command
  controls select literal shell input such as `inspect world`, `watch world 2`,
  `list watches`, `back`, and `format json`. It delegates all parsing,
  navigation, watch registration, refresh, and projection decisions to
  `ObservationShell`.
- The workbench renders session context, catalog selection, transcript,
  watches, and diagnostics as adapter-owned panels. Its local fixture supplies
  copied observation input only; the UI does not interpret world relationships
  or mutate runtime state.
- Focused unit coverage compares the workbench's selected `inspect world`
  control with a fresh scripted shell invocation and requires the same
  `ShellRecord`. The same corpus resolves its layout at normal and compact
  viewport sizes.
- The native workbench is optional executable evidence. Browser/TypeScript
  ownership remains an independent later consumer proof and is deliberately
  not inferred from this slice.
- `runtime-observation-workbench` now adds that independent browser proof.
  Its `ObservationShellClient` transports only literal input plus a monotonic
  logical sequence to `WasmObservationShellSession`; Rust owns the runtime
  command catalog, application-specific argument validation, observation
  source construction, playback transitions, and the bounded `ShellRecord`
  returned to the browser. The focused engine suite proves both a successful
  `application runtime list-animations` query and the Rust-owned rejection of
  `application runtime play not-a-clip`.
- Browser semantic controls and the embedded Ratatui projection now call the
  same retained `WasmObservationShellSession`. The browser holds no second
  runtime: it transports normalized input/actions and blits completed RGBA
  frames. A focused engine test selects presentation through the semantic
  facade, proves the shared runtime observation changed, and proves the
  Ratatui frame changed with it. Terminal commands then expose that same
  runtime owner and result through the shared facade.
- The same workbench now hosts that Rust-owned session through a focused,
  bounded Ratatui region. Ratatui composes transcript, prompt, history, and
  scrolling cells through its public `Backend::draw` seam; a corpus-local
  Tokimu backend retains those cells and `ui-tools` rasterizes them with
  Departure Mono before Rust/WASM returns an RGBA frame. TypeScript forwards
  normalized keyboard and wheel input, owns canvas focus and pixel blitting,
  and does not parse commands, position terminal glyphs, or interpret Ratatui
  styles. This completes the embedded browser-provider proof, but not the
  standalone Ratatui-host or cross-host session-parity evidence.
- The embedded surface now derives a bounded frame size from its host region,
  and its Rust-side transcript scroll estimate accounts for ordinary
  display-width wrapping before Ratatui renders the authoritative line breaks.
  The browser observes that host region and coalesces redraws when it changes.
  Browser pointer interaction explicitly focuses the canvas before forwarding
  keyboard input. This prevents common host-layout and page-focus failures;
  it does not claim grapheme-aware editing, rich-span wrapping parity, or a
  shared live session with the native host.
- `hello-observation-shell-ratatui` is now an optional native
  `ratatui-standalone` corpus binary. Its Crossterm host owns raw terminal
  input, alternate-screen lifecycle, and terminal pixels, while its prompt,
  history navigation, keyboard transcript viewport navigation, transcript, and
  literal command dispatch reuse the shared `ShellFixture` path used by the
  plain-text adapter. Focused tests run the same command trace through
  independent fixtures and require identical projections; they also prove that
  Page Up/Down/Home/End and wheel scrolling plus resize-time viewport clamping
  and bounded prompt-cursor placement are adapter-local and that submitted
  output follows live output. This is
  standalone-host and
  command-semantics evidence; it is not yet one live retained session shared
  by native, embedded, and headless hosts, nor cross-host viewport parity.

### Slice 9: Authority, Budgets, And Adversarial Input

Deliverables:

- [x] Add read-only and control session authorities.
- [x] Bound input length, argument count, output size, history, watches, and
      command rate.
- [x] Test unusual valid Unicode/control input, oversized input, unknown targets, stale context,
      command flooding, and projection failures.
- [x] Omit oversized projections from retained shell output with an explicit
      boundary failure.
- [x] Define an owner-supplied visible-or-redacted field contract for otherwise
      valid sensitive observations.

Acceptance criteria:

- [x] Valid Rust text input, including unusual Unicode/control characters,
      cannot panic or partially mutate state. Malformed byte decoding remains
      an input-adapter responsibility because the shell receives `&str`.
- [x] Read-only sessions cannot invoke mutation commands.
- [x] Budget failures identify the owning boundary and preserve session
      usability.
- [x] Help/catalog output does not reveal unavailable private implementation
      details.
- [x] A redacted field projects its owner-supplied reason without crossing the
      source value through the application-to-shell boundary.

Current evidence:

- `ShellAuthority::{ReadOnly, Control}` is session-local. Registered mutation
  commands remain discoverable but a read-only session rejects them before a
  caller-supplied mutation handler can run.
- `ShellBoundaryLimits` now bounds input bytes, application-envelope argument
  count, retained projection bytes, and commands accepted within one
  caller-supplied logical sequence. Existing shell history, navigation, and
  watch limits remain session-local bounds.
- Input, argument, command-rate, and projection failures return a structured
  `budget_exceeded` status owned by the shell boundary. The shell retains the
  bounded failure record rather than the oversized projection, and the session
  remains usable for later commands.
- Focused tests cover unusual valid Unicode/control input, unknown targets,
  stale context, oversized input, command flooding, oversized query results,
  read-only mutation rejection, and catalog isolation from invocation-local
  handler details. The shell deliberately receives valid `&str`; malformed
  byte decoding must be rejected by its terminal, browser, or transport input
  adapter before this boundary.
- `ApplicationQueryField` now carries either `Visible { value }` or
  `Redacted { reason }`. The application owner makes that disclosure decision;
  the shell projects the decision in text and JSON without classifying data or
  receiving a withheld value. A focused mixed-field query proves that a visible
  field remains visible while a redacted field exposes only its reason.

Deferred evidence:

- A real sensitive-data owner, secret classification vocabulary, and reusable
  policy engine. The current contract is intentionally only an owner-supplied
  disclosure decision, not a shell-wide authorization system.
- Adapter-level malformed-byte decoding, interrupt, and transport flood
  policies.

### Slice 10: Architectural Review

Deliverables:

- [x] Compare scripted, CLI, MUD, and graphical consumer evidence.
- [x] Record repeated session, catalog, routing, watch, and projection
      semantics.
- [x] Defer the application-pattern, corpus-library, first-party-tool, or
      admitted-capability decision to `AR-0013`.
- [x] Open an Architectural Review for Application Observation Shell ownership
      before creating `tokimu-shell`; AR-0013 now owns that question.
- [x] Update AR-0013 with standalone Ratatui, embedded cell-grid, headless, and
      graphical projection evidence before proposing provider admission.
- [x] Record the bounded consumer findings in this plan, the runtime
      observation workbench, and the published website lab. Broader SDD,
      roadmap, and ADR changes remain an `AR-0013` outcome.

Admission handoff criteria:

- [x] A future promotion proposal must name irreducible semantics shared by at
      least two independent presentation modalities.
- [x] Convenience, a common command syntax, or a large UI is not treated as
      capability evidence by itself.
- [x] Observation and mutation ownership remains with existing domains in the
      bounded consumer.
- [x] Deferred remote, editor, persistence, and security questions remain
      explicit in `AR-0013`.

Current evidence:

- AR-0013 Cycle 9 compares the scripted runner, plain CLI adapter, MUD
  consumer, native workbench, and browser workbench. Their common semantics
  are bounded session state, catalog discovery, owner-qualified routing,
  authority checks, structured projection, and explicit failures.
- The host-specific differences remain replaceable: piped versus interactive
  input, terminal controls, native panels, browser controls, and text/cell
  projection mechanics do not change command meaning.
- The direct Ratatui browser surface has a focused end-to-end contract test:
  a Rust-owned prompt edit changes the rendered Tokimu frame, and submitting
  that prompt through the semantic shell changes the frame again with the
  runtime-owned result. The browser still supplies only input and displays the
  returned RGBA frame.
- Promotion remains deliberately deferred. A real sensitive-observation owner,
  independent standalone-host/shared-session evidence, and bounded asset and
  performance observation contracts are still required before the shell can be
  evaluated as a first-party capability.

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
- standalone Ratatui versus embedded-cell semantic trace parity;
- host ownership of focus, keyboard, mouse wheel, resize, clipping, and cursor;
- bounded diagnostics for unsupported cell width, Unicode, style, and cursor
  states.

Workspace validation remains:

```text
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Plan Closure

This plan is complete for its bounded consumer claim:

- one browser/WASM `WasmObservationShellSession` owns both semantic controls
  and embedded Ratatui shell state;
- TypeScript acts only as the interaction, focus, resize, and whole-frame
  blit adapter;
- direct semantic actions and terminal commands observe the same retained
  runtime scenario;
- the focused engine suite proves a cross-surface state change rather than
  relying on visual similarity.

The following are deliberately handed to `AR-0013` rather than treated as
unfinished implementation work here:

- preserving one retained session across native standalone and browser hosts;
- proving viewport, prompt, and history parity across those hosts;
- deciding whether Ratatui or any shell-facing subset becomes a permanent
  Tokimu presentation provider;
- admitting a general shell capability beyond this consumer corpus.

## Completion Criteria

This effort is complete for its bounded consumer claim when:

- one deterministic headless shell composes existing observations;
- plain-text and JSON projections expose the same meaning;
- command discovery, namespaces, typed arguments, and session authority are
  explicit;
- bounded watch behavior is proven;
- mutation and animation requests pass through the runtime command seam rather
  than shell-owned shortcuts;
- one local CLI and one MUD/text-first consumer reuse the session boundary;
- one graphical adapter proves presentation independence;
- one embedded Ratatui projection and browser semantic controls preserve the
  same retained Rust/WASM session, command result, output, and diagnostic
  meaning;
- artifacts localize failures across input, parse, catalog, authority,
  observation/command, projection, and output boundaries;

Capability graduation is intentionally not a completion condition for this
consumer plan. `AR-0013` owns that later decision.

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
- [`Observation Shell.md`](../Conversations/Observation%20Shell.md)
- [`On Ratatui.md`](../Conversations/On%20Ratatui.md)
- [`runtime-observation-and-command-corpus.md`](runtime-observation-and-command-corpus.md)
- [`consumer-corpora.md`](consumer-corpora.md)
- [`performance-diagnostics-and-runtime-observation.md`](performance-diagnostics-and-runtime-observation.md)
- [`networking-and-transport.md`](networking-and-transport.md)
- [`Tokimu Software Design Document.md`](../Tokimu%20Software%20Design%20Document.md)
- [`kernel-principles.md`](../kernel-principles.md)
- [`AR-0005-runtime-observation-and-performance-telemetry.md`](../Architectural%20Reviews/AR-0005-runtime-observation-and-performance-telemetry.md)
- [`AR-0013-observation-shell-and-ratatui-presentation-provider.md`](../Architectural%20Reviews/AR-0013-observation-shell-and-ratatui-presentation-provider.md)
- `crates/tokimu-core/src/world.rs`
- `corpus/lib/performance-diagnostics-corpus/README.md`
- `corpus/consumers/aspnet-wasm-asset-workbench/DESIGN.md`
- `corpus/consumers/aspnet-wasm-presentation-workbench/DESIGN.md`
