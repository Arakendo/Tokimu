# AR-0013: Observation Shell And Ratatui Presentation Provider

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-08-04 |
| Last reviewed | 2026-08-05 |
| Scope | Foundational observation tooling and replaceable presentation provider |
| Trigger | The native console corpus, Tosumu TQL composition, and repeated terminal, graphical, browser, and headless inspection needs now pressure the same session and projection boundary |
| Related ADRs | ADR-0003, ADR-0004, ADR-0007 |
| Related evidence | `tokimu-console-command-window`, Observation Shell plan, Tosumu inspection-island plan, Ratatui normalized-cell artifacts, native console input and scrolling evidence |
| Admission exception | None |

## Architectural Question

Should Tokimu define a provider-neutral Observation Shell session contract, and
should Ratatui become an official replaceable presentation provider that can
operate both as an independent terminal host and as a bounded cell-grid
projection inside another Tokimu presentation host?

## Context

Tokimu increasingly needs to expose observations, diagnostics, command
catalogs, retained transcripts, and owner-routed semantic actions to graphical,
terminal, browser, headless, and text-first consumers. The
`tokimu-console-command-window` corpus already composes Tosumu-owned TQL
meaning, a Ratatui `TestBackend` projection, Tokimu text and UI presentation,
and a native window.

The current corpus proves useful composition, but it does not yet prove which
parts should become stable Tokimu contracts. Treating a terminal widget tree as
the shell would couple application meaning to one provider. Treating the native
Tokimu console as the shell would make graphical presentation the reference
implementation. Both outcomes conflict with ADR-0003's distinction between
capability meaning and replaceable mechanisms.

The conversations that triggered this review describe two legitimate Ratatui
roles:

1. an independent terminal host for servers, command-line tools, and text-first
   applications; and
2. an embedded projection provider that renders into a normalized cell buffer
   owned by a bounded region of another presentation host.

Those roles may share provider code, but they do not transfer host lifecycle,
focus, input arbitration, resizing, or outer layout ownership to Ratatui.

## Trigger And Evidence

- Corpus examples: `tokimu-console-command-window` exercises Departure Mono,
  text input, retained transcript output, wrapping, scrolling, command history,
  Tosumu TQL execution, and Ratatui normalized-cell projection.
- Automated tests: the console corpus verifies deterministic Ratatui cell
  artifacts, Tosumu CLI JSON behavior, transcript navigation, wrapping, and
  provider-facing rendering behavior.
- Audits or diagnostics: native console work repeatedly separated semantic
  command results from text layout, cell projection, input direction, and
  clipping failures.
- Independent consumers: the planned Observation Shell, Tosumu inspection
  island, native console window, headless scripts, MUD/text-first consumers,
  browser workbenches, and the website Ratatui template lab all need related
  observation and command behavior.
- Repeated implementation friction: command discovery, retained output,
  history, navigation context, projection choice, and focus behavior otherwise
  risk being rebuilt independently in every inspector.
- Missing evidence: there is not yet one provider-neutral shell session used by
  two independent hosts, no standalone Ratatui terminal host has been compared
  with the embedded cell-grid path, and browser projection parity remains
  unproven.

## Ownership Analysis

### Application And Capability Owners

Application and capability owners retain:

- observed facts and their revision or freshness meaning;
- diagnostics and performance measurements;
- command definitions, argument semantics, validation, and execution;
- authority decisions and mutation policy;
- world, asset, animation, resource, and domain truth.

### Observation Shell Candidate

A provider-neutral Observation Shell may own:

- session lifecycle and bounded retained output;
- command catalog composition and owner-preserving routing metadata;
- navigation, selection, history, and watch context local to the session;
- help, completion, and output-projection selection protocols;
- typed query and command results together with explicit diagnostics.

The shell must not receive unrestricted `&mut World`, redefine owner-provided
commands, parse provider storage, or become the authority for observed facts.

### Ratatui Provider Candidate

A Ratatui provider may own:

- terminal widgets, layout, cursor, style, and interaction mechanics;
- projection of shell observations into a Ratatui buffer;
- normalization of a bounded Ratatui buffer into styled cells;
- terminal-specific keyboard and mouse translation when Ratatui is the host.

Ratatui widget state and crate types must not cross the public Observation
Shell contract. Ratatui is not kernel-native and does not belong in
`tokimu-core` or `tokimu-runtime`.

### Presentation Host

The active host owns lifecycle, focus arbitration, input dispatch, dimensions,
and mechanism-specific output. When Ratatui is embedded, Tokimu UI owns outer
composition, region bounds, clipping, and integration with neighboring visual
content. Ratatui owns only the contents and interaction model of its assigned
cell-grid region.

## Dependency Direction

```text
Current corpus:

Tosumu TQL meaning
        -> corpus-local command session
        -> Ratatui TestBackend buffer
        -> normalized styled cells
        -> Tokimu text/UI/native renderer

Proposed candidate boundary:

owner-provided observations and semantic commands
        -> provider-neutral Observation Shell session
        -> replaceable output projection
             |-> Ratatui independent terminal host
             |-> Ratatui normalized cells -> bounded Tokimu UI region
             |-> plain text / JSON / browser / MUD adapter
```

Dependencies point from providers and hosts toward the shell contract. The
shell contract depends only on provider-neutral observations, commands,
diagnostics, and session data. No Ratatui, Tosumu, terminal, browser, renderer,
or operating-system type may appear in that contract.

## Alternatives Considered

### Alternative A: Use Tokimu UI As The Only Official Inspector

- Benefits: one graphical implementation and no terminal provider dependency.
- Costs: weak headless and server support; graphical presentation becomes the
  accidental semantic reference.
- Failure mode: every non-graphical consumer recreates session and command
  composition behavior.

### Alternative B: Make Ratatui The Observation Shell Contract

- Benefits: mature widgets, terminal interaction, and deterministic
  `TestBackend` output are available immediately.
- Costs: provider types, terminal assumptions, and cell-grid constraints leak
  into application and capability semantics.
- Failure mode: browser, native graphical, and structured consumers must
  emulate or translate a terminal UI instead of consuming Tokimu meaning.

### Alternative C: Provider-Neutral Shell With A Ratatui Provider

- Benefits: preserves owner-provided meaning, supports headless use, and lets
  Ratatui serve both terminal and embedded-cell hosts without becoming the
  semantic authority.
- Costs: requires a deliberately small shell/session contract and parity tests
  across projections.
- Failure mode: a broad shell contract could still become a hidden editor or
  duplicate capability-specific command policy.

### Alternative D: Continue Corpus-Local Composition

- Benefits: maximally reversible while evidence remains incomplete.
- Costs: repeated consumers may duplicate session, history, routing, and
  projection logic.
- Failure mode: incompatible local shells become established before the common
  boundary is observable.

## Findings

1. Observation Shell session semantics and Ratatui presentation mechanics are
   distinct ownership boundaries.
2. The shell is an application-level observation and command composition
   capability, not a terminal emulator, arbitrary process shell, or new owner
   of world state.
3. Ratatui is a strong official provider candidate because it supports both
   independent terminal hosting and deterministic embedded cell projection.
4. Official provider admission is not yet justified. The existing native
   console is one composed corpus, not two independent hosts.
5. A normalized styled-cell buffer is useful provider evidence, but it is not a
   universal Tokimu text or UI semantic model.
6. Standalone and embedded Ratatui modes have different host ownership even if
   they share widgets and projection code.
7. The provider-neutral shell must remain usable in headless scripts and
   structured-output tests without Ratatui or a renderer.
8. Current evidence does not justify a new engine-crate dependency or an ADR.
9. Ratatui's stable embedded-provider seam is its `Backend::draw` contract,
   which receives changed cells after widget layout and buffer diffing. A
   Tokimu provider can consume that seam without routing runtime presentation
   through `TestBackend`.
10. `CompletedFrame.buffer` is useful full-frame diagnostic evidence, while
    `TestBackend` is a reference test implementation rather than a required
    production bridge.

## Disposition

**Incubating.** Tokimu provisionally accepts the ownership direction of a
provider-neutral Observation Shell with replaceable projections. Ratatui is the
preferred first provider candidate for terminal and embedded cell-grid studies,
but it is not yet an admitted official provider or a stable public dependency.
The console and Observation Shell corpora must gather independent-host and
projection-parity evidence before this review can recommend an ADR.

## Consequences

- The console corpus becomes the first embedded Ratatui-provider evidence, not
  the definition of the Observation Shell.
- The Observation Shell plan must keep command meaning with capability and
  application owners.
- Ratatui remains in corpus, support, or provider code and outside engine core
  and runtime crates.
- Native graphical hosts must own region bounds, clipping, focus, and input
  arbitration around embedded cell content.
- Unsupported Unicode width, combining marks, styles, and cursor states must
  produce bounded diagnostics rather than silent semantic changes.
- A future official provider may use a normal pinned Cargo dependency. A git
  submodule is not required for the provider mechanism.
- The next embedded proof must implement a retained bounded `TokimuBackend`
  that applies Ratatui cell diffs and lowers the resulting surface through
  Tokimu presentation. TypeScript must not repaint or reinterpret individual
  Ratatui cells in that proof.

## Required Follow-Up

- [x] Complete deterministic semantic transcript and normalized-cell artifacts
      in `tokimu-console-command-window`.
- [x] Build one standalone Ratatui terminal host over the shared shell-session
      execution path.
- [x] Build one bounded embedded Ratatui cell-grid region in a Tokimu UI host.
- [x] Replace the website lab's transitional `TestBackend`/JSON/canvas cell
      path with a direct retained `TokimuBackend` presentation path.
- [ ] Compare command results, retained history, selection, and diagnostics
      across standalone, embedded, and headless projections.
- [ ] Add focus, keyboard, mouse-wheel, resize, clipping, and cursor ownership
      tests for both hosting modes.
- [ ] Define bounded normalization and diagnostics for unsupported cell width,
      Unicode, styles, and cursor states.
- [ ] Update the Observation Shell and Tosumu inspection-island plans with the
      resulting evidence.
- [ ] Reopen this review before extracting a shell capability or admitting an
      official Ratatui provider.

## Reopening Triggers

- Two independent hosts preserve the same shell session and command semantics.
- Ratatui widget or terminal types leak into the provider-neutral shell API.
- Browser or structured consumers require meaning that the candidate shell
  contract cannot express without provider-specific branches.
- Standalone and embedded modes cannot share a bounded normalized-cell
  projection honestly.
- Warm-frame or resize evidence shows that the projection model causes
  unacceptable rebuild or resource churn.
- A simpler existing Tokimu observation or presentation capability can own the
  repeated behavior without a new shell contract.

## Review History

### Cycle 1 -- 2026-08-04

- Status entering review: Proposed
- New evidence: native console corpus, Tosumu TQL JSON execution, Ratatui
  `TestBackend` normalization, Departure Mono rendering, input/history/scroll
  behavior, and the Observation Shell/Ratatui design conversations.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: shell session meaning and Ratatui projection are separable;
  Ratatui is promising but independent-host evidence is incomplete.
- Disposition: Incubating.
- Resulting ADR or documentation change: no ADR; console and Observation Shell
  plans are aligned with this review.

### Cycle 2 -- 2026-08-05

- Status entering review: Incubating
- New evidence: deterministic session, Ratatui normalized-cell, Tokimu cell
  layout, projection-conformance, and CPU raster artifacts; hard-failing
  transcript/cell/cursor parity tests; corpus-local native interaction state;
  measured native-font wrapping and oversized-token retention tests.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: interaction mechanics can remain consumer-owned without leaking
  into command meaning; normalized-cell projection remains separable from both
  shell semantics and Tokimu renderer state; deterministic embedded evidence is
  now complete enough to compare but does not substitute for an independent
  standalone Ratatui host.
- Disposition: Incubating.
- Resulting ADR or documentation change: no ADR; deterministic artifact
  follow-up is complete, while independent-host and official-provider evidence
  remain open.

### Cycle 3 -- 2026-08-05

- Status entering review: Incubating
- New evidence: `tokimu-website-ratatui-lab` renders three deterministic
  dummy-data scenes (`system-monitor`, `asset-inspector`, and
  `command-transcript`) through Ratatui's headless `TestBackend` in
  Rust/WASM. The browser chooses a template and grid density, then paints only
  the normalized styled-cell snapshot returned by the WASM boundary. Native
  tests require a complete bounded grid for each template and reject
  undersized dimensions explicitly.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: a browser can host a real Ratatui cell projection without
  reimplementing terminal layout in TypeScript. This is independent embedded
  presentation evidence, not evidence for a provider-neutral shell session or
  a standalone terminal host. The template scenes deliberately use consumer
  dummy data and do not claim Tosumu, world, or command semantics.
- Disposition: Incubating. The new consumer reinforces the normalized-cell
  boundary while the standalone-host, shared-session, focus, and command-parity
  requirements remain open.
- Resulting ADR or documentation change: no ADR; the website lab records
  provider evidence only and keeps Ratatui outside engine-core dependencies.

### Cycle 4 -- 2026-08-05

- Status entering review: Incubating
- New evidence: the exact Ratatui `v0.29.0` source is pinned under
  `third-party/presentation-providers/ratatui`. Source inspection confirms that
  `Terminal::flush` computes `previous_buffer.diff(current_buffer)` and passes
  the resulting changed cells to `Backend::draw`. `CompletedFrame` separately
  exposes the complete current `Buffer`, and `TestBackend::draw` only copies
  changed cells into its retained test buffer.
- Participants or reviewers: project maintainer and Codex source review.
- Findings: the website lab currently takes an avoidable path through
  `TestBackend`, a custom snapshot, JSON, and TypeScript cell painting. The
  lower and more honest embedded-provider seam is a custom Ratatui `Backend`
  that retains a bounded Tokimu cell surface and forwards that surface to
  Tokimu text and rendering capabilities. `CompletedFrame.buffer` remains the
  preferred full-frame diagnostic seam; neither seam transfers shell meaning
  or host ownership to Ratatui.
- Disposition: Incubating. The source-backed boundary is accepted for the next
  corpus slice, but official provider admission still awaits a Tokimu-rendered
  embedded proof and independent standalone-host evidence.
- Resulting ADR or documentation change: no ADR; the website lab design and
  console plan now identify `TestBackend` as test evidence and direct
  `TokimuBackend` delivery as the target implementation.

### Cycle 5 -- 2026-08-05

- Status entering review: Incubating
- New evidence: `tokimu-website-ratatui-lab` now implements Ratatui's public
  `Backend` trait with a retained bounded `TokimuBackend`. Ratatui widget diffs
  update that retained surface, `ui-tools` rasterizes its glyphs with the
  pinned Departure Mono provider, and Rust/WASM returns a completed RGBA frame.
  The browser selects fixtures and blits pixels without interpreting cells.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: the lower backend seam works without `TestBackend`, JSON cell
  serialization, or TypeScript glyph placement. This proves static embedded
  Tokimu rendering, but not incremental dirty-region upload, interactive host
  input, shared shell-session parity, or standalone-terminal parity.
- Disposition: Incubating. The transitional website delivery path is retired;
  official provider admission remains deferred pending the other hosting and
  interaction evidence.
- Resulting ADR or documentation change: no ADR; the website lab and Slice 7
  progress now distinguish completed static raster evidence from open runtime
  and shell evidence.

### Cycle 6 -- 2026-08-05

- Status entering review: Incubating
- New evidence: `corpus/lib/observation-shell` now has explicit read-only and
  control session authorities plus bounded input, application-argument,
  per-logical-sequence command-rate, retained-projection, history, navigation,
  and watch limits. Its focused suite proves that read-only sessions reject
  registered mutations before an owner handler runs; oversized input, command
  floods, and oversized owner query results become structured boundary
  failures while the session remains usable. The native
  `hello-observation-workbench` independently maps graphical controls to the
  same literal shell inputs and checks one result against a fresh scripted
  session.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: bounded shell-session semantics remain independent of Ratatui and
  of any particular host. Authority constrains dispatch but does not alter
  discovery; bounded output can omit an oversized projection without retaining
  owner output. Owner-specific redaction policy and malformed-byte decoding
  remain outside this `&str` shell boundary and require their own adapter or
  capability evidence.
- Disposition: Incubating. The shell boundary has enough evidence to continue
  as a corpus library, but standalone Ratatui-host parity, shared-session
  parity across independent hosts, and policy-driven sensitive-observation
  handling are still required before capability admission.
- Resulting ADR or documentation change: no ADR; the Observation Shell plan
  records the authority and budget evidence and keeps semantic redaction as an
  explicit open policy question.

### Cycle 7 -- 2026-08-05

- Status entering review: Incubating
- New evidence: `hello-observation-shell` now routes the existing runtime
  playback lifecycle through the application-owner seam: pause, resume, seek,
  stop, and reset join the earlier play, fixed advance, and state query. The
  deterministic script also pauses after stopping, and the runtime returns
  `RejectedUnsupported` with its `pause_not_playing` diagnostic while retaining
  the stopped playback state. The same corpus now routes set-hotspot,
  clear-selection, and clear-hotspot commands through scenario-owned target
  mappings, and proves `RejectedUnknownTarget` with the
  `presentation_target_unresolved` diagnostic for a deliberately absent target.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: lifecycle command syntax and numeric parsing may remain
  scenario-local without making the shell a playback owner. Presentation target
  resolution and target rejection likewise remain scenario-owned. The shell's
  stable responsibility is the authority-checked, bounded owner-qualified
  envelope and receipt projection; the runtime remains the sole owner of
  lifecycle and presentation transition/rejection semantics.
- Disposition: Incubating. The lifecycle and presentation-command paths are
  now materially broader, but independent host session parity, standalone
  terminal-host evidence, and policy-driven observation redaction remain open.
- Resulting ADR or documentation change: no ADR; the plan records the completed
  lifecycle evidence and preserves the remaining admission gaps.

### Cycle 8 -- 2026-08-05

- Status entering review: Incubating
- New evidence: the browser-facing `runtime-observation-workbench` now exposes
  `WasmObservationShellSession`. Its TypeScript client transports only raw
  owner-qualified text and a monotonic logical sequence; the WASM boundary
  returns owned JSON `ShellRecord` values. Rust registers the runtime catalog,
  constructs observations, validates application arguments, and invokes the
  scenario-owned playback and presentation operations. Focused tests prove
  catalog discovery, a successful `application runtime list-animations` query,
  an accepted `application runtime play 0` mutation, and rejection of
  `application runtime play not-a-clip` without a browser command parser.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: the raw-text browser route can consume the same bounded shell
  contract without constructing a TypeScript shadow runtime or exposing
  provider-specific runtime requests. Direct JSON workbench controls remain
  useful comparison evidence but are not part of the shell route. This proves
  browser transport and owner dispatch only; it does not prove a standalone
  terminal host, shared session parity across independent hosts, or Ratatui
  command-session parity. A focused projection-isolation regression also
  proves that browser-side edits to a returned playback JSON record cannot
  alter the scenario-owned runtime observed by a subsequent shell query.
- Disposition: Incubating. Browser command-boundary evidence is complete for
  the current Slice 8 scope; the remaining host and provider questions remain
  separate admission evidence.
- Resulting ADR or documentation change: no ADR; Slice 8 now records the
  completed TypeScript ownership criterion.

### Cycle 9 -- 2026-08-05

- Status entering review: Incubating
- New evidence: the browser-facing runtime observation workbench now routes
  copied, owner-labeled observations through `application runtime
  world-summary`, `application runtime relationships`, and `application
  diagnostics records`. The existing scripted shell, plain CLI adapter, MUD
  consumer, native workbench, and browser workbench therefore all exercise the
  same bounded session, catalog, typed invocation, authority, and projection
  semantics while retaining different host interaction mechanics.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: the repeated semantics are session-local command history,
  owner-qualified routing, bounded catalogs, structured result projection,
  authority checks, and explicit boundary failures. World, relationship, and
  diagnostic meaning remains with their respective owners; neither a terminal
  nor TypeScript receives a shadow observation model. Asset and performance
  commands are intentionally absent because their owners have not yet supplied
  bounded consumer-ready observations. This absence is explicit evidence, not
  an incomplete browser implementation.
- Disposition: Incubating. The shell remains justified as a corpus library,
  while policy-driven redaction for valid sensitive observations and
  independent standalone-host/shared-session evidence remain required before
  capability admission.
- Resulting ADR or documentation change: no ADR; the observation-shell plan
  now marks the completed cross-consumer comparison and identifies the exact
  deferred owner contracts.

### Cycle 10 -- 2026-08-05

- Status entering review: Incubating
- New evidence: `ApplicationQueryField` now carries an owner-supplied
  disclosure result: either `Visible { value }` or `Redacted { reason }`.
  Text and JSON shell projections preserve that result as a bounded structured
  field. The focused shell suite exercises a mixed query where one field is
  visible and another is redacted, proving that the withheld value never
  reaches the projection boundary.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: redaction is an owner decision, not shell classification,
  authorization, or secret discovery. The shell can faithfully carry a
  redaction reason without receiving the corresponding value. This supports
  bounded disclosure while retaining application ownership of what may be
  exposed.
- Disposition: Incubating. The contract is sufficient corpus evidence for
  owner-supplied withholding, but a real independent sensitive-data owner,
  reusable policy vocabulary, standalone-host/shared-session parity, and
  bounded asset/performance owners remain necessary before capability
  admission.
- Resulting ADR or documentation change: no ADR; the Observation Shell plan
  records owner-supplied disclosure as complete Slice 9 evidence and preserves
  the remaining policy and host gaps.

### Cycle 11 -- 2026-08-05

- Status entering review: Incubating
- New evidence: `runtime-observation-workbench` now hosts its existing
  Rust-owned `ObservationShell` session inside a focused bounded Ratatui
  terminal region. A corpus-local `TokimuBackend` implements Ratatui's public
  `Backend::draw` seam, retains changed cells, and lowers the completed cell
  surface through `ui-tools` and the pinned Departure Mono provider into an
  RGBA frame. Rust/WASM accepts normalized text, editing, history, paging, and
  wheel actions; TypeScript owns browser focus and frame blitting only.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: the embedded provider now drives a real semantic shell session
  without `TestBackend`, JSON cell serialization, browser command parsing, or
  TypeScript terminal layout. Ratatui remains provider-local: it owns terminal
  composition, while the browser host owns focus, input dispatch, bounds, and
  pixels. The existing plain JSON projection remains useful comparison
  evidence, not a second shell.
- Disposition: Incubating. Embedded browser proof is complete. A standalone
  Ratatui host and trace parity across standalone, embedded, and headless
  sessions remain material admission evidence.
- Resulting ADR or documentation change: no ADR; the Observation Shell plan
  records the completed embedded host and preserves the remaining parity work.

### Cycle 12 -- 2026-08-05

- Status entering review: Incubating
- New evidence: `hello-observation-shell-ratatui` now provides an optional
  native Crossterm/Ratatui terminal host. The host owns raw-mode lifecycle,
  alternate-screen execution, terminal input, and terminal pixels. It reuses
  the corpus-local `ShellFixture` execution path shared with the plain-text
  adapter; a deterministic `help`, `inspect world`, and `list entities` trace
  produces identical projections from independent host fixtures. Focused
  tests pass and the optional `ratatui-standalone` feature compiles.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: Ratatui can host the same literal Observation Shell command
  semantics without entering engine crates or exposing Ratatui types through
  the shell boundary. Native terminal mechanics remain host-owned, while the
  shell continues to own command parsing and projection. This is not evidence
  for a shared live retained session across native, embedded, and headless
  hosts.
- Disposition: Incubating. The standalone-host gap is closed at the command
  execution boundary; retained-history, selection, diagnostics, and live
  session parity across all hosts remain material admission evidence.
- Resulting ADR or documentation change: no ADR; the plan now records the
  standalone adapter and preserves the cross-host state-parity requirement.

### Cycle 13 -- 2026-08-05

- Status entering review: Incubating
- New evidence: the optional `hello-observation-shell-ratatui` native binary
  now owns a real Crossterm/Ratatui terminal lifecycle while its
  `TerminalSession` routes submitted commands through the same `ShellFixture`
  used by the plain-text adapter. Focused tests exercise character entry,
  Enter submission, transcript projection, Up/Down history navigation,
  Page Up/Down/Home/End and mouse-wheel viewport navigation, live-output
  follow behavior, resize-time viewport clamping, and adapter-local `exit`
  without a live terminal. Raw mode and alternate-screen acquisition now use
  independent cleanup guards: raw mode is restored if alternate-screen setup
  fails, and mouse capture plus the alternate screen are restored after their
  acquisition.
- Findings: Ratatui and Crossterm are replaceable presentation/input adapters.
  The terminal host owns prompt editing, terminal mode, transcript viewport
  navigation, and styling; `ObservationShell` remains the owner of command
  parsing, retained command history, and projections. This is meaningful
  native-host evidence, but not proof that browser, headless, and native hosts
  share one retained session or one input-focus policy.
- Disposition: Incubating. The standalone host criterion is satisfied. Shared
  retained-session parity, selection/diagnostic parity, and independently
  tested focus, clipping, and cursor behavior remain required before provider
  or capability admission can be considered. Keyboard and native mouse-wheel
  transcript navigation, bounded resize clamping, display-width wrapped
  transcript scroll accounting, and bounded prompt-cursor placement are now
  covered; none is evidence for cross-host viewport parity, exact rich-span
  line breaking, grapheme-aware cursor editing, or physical clipping.
  Terminal cleanup is host-local lifecycle evidence, not session semantics.
- Resulting ADR or documentation change: no ADR. The corpus plan now records
  native terminal mechanics as adapter-owned rather than shell-owned.

### Cycle 14 -- 2026-08-05

- Status entering review: Incubating
- New evidence: the browser workbench's direct Rust/WASM Ratatui surface now
  derives a bounded raster frame from the available host width, preserves its
  intended surface aspect ratio, and explicitly focuses its canvas before
  forwarding terminal input. Rust estimates transcript viewport bounds using
  ordinary Unicode display width so long semantic records contribute wrapped
  rows before Ratatui performs the authoritative final layout.
- Findings: browser sizing, focus acquisition, and pixel blitting remain host
  mechanics. Ratatui still owns terminal layout and style, while the shell
  owns history and command meaning. A width-aware scroll estimate belongs with
  the retained Ratatui surface rather than TypeScript.
- Disposition: Incubating. This improves embedded-host resilience but does not
  prove grapheme-aware editing, rich styled-span line-breaking parity, native
  and browser viewport parity, or a shared retained session.
- Resulting ADR or documentation change: no ADR; the plan records the
  browser-host evidence and its explicit limits.

### Cycle 15 -- 2026-08-05

- Status entering review: Incubating
- New evidence: the runtime-observation browser workbench retained one
  Rust-owned `ObservationShell` for both the TypeScript semantic controls and
  the Tokimu-rendered Ratatui surface, but direct toolbar actions originally
  updated the JSON observation without adding a corresponding Ratatui
  transcript record. The hosts therefore observed related runtime state while
  presenting different retained interaction histories. The shell now exposes a
  typed application-query recording path. Browser controls use it to append
  explicit `[ui]` records with a source field of `browser toolbar`; these
  records are visible in Ratatui but excluded from prompt Up/Down command
  recall. Focused regression coverage verifies that a toolbar selection
  changes the returned Ratatui raster and retains the expected structured
  shell record.
- Participants or reviewers: project maintainer and Codex implementation review.
- Findings: a shared semantic session is not by itself sufficient cross-host
  evidence. Each accepted interaction that changes observable state must have
  an explicit projection policy for every retained host surface. The shell owns
  the retained record and its bounded structure; the browser host owns the
  toolbar gesture; Ratatui owns how that record is composed and rasterized.
  Host-originated records are not user-entered commands and must not silently
  alter command-history navigation.
- Disposition: Incubating. The browser toolbar/Ratatui divergence is resolved
  for this corpus workbench. Cross-host parity still requires broader evidence
  for selection, diagnostics, redaction, and independently hosted live
  sessions; this fix does not claim universal event sourcing or a generic UI
  telemetry model.
- Resulting ADR or documentation change: no ADR; this cycle records the
  retained-transcript projection requirement and the explicit distinction
  between host controls and command recall history.

## References

- [`On Ratatui.md`](../Conversations/On%20Ratatui.md)
- [`Observation Shell.md`](../Conversations/Observation%20Shell.md)
- [`tokimu-console-command-window-corpus.md`](../Plans/Standalone/tokimu-console-command-window-corpus.md)
- [`tokimu-observation-shell-consumer-corpus.md`](../Plans/Standalone/tokimu-observation-shell-consumer-corpus.md)
- [`ADR-0003-capability-ownership-boundary.md`](../ADR/ADR-0003-capability-ownership-boundary.md)
- [`ADR-0004-foundational-presentation-text-and-icons.md`](../ADR/ADR-0004-foundational-presentation-text-and-icons.md)
- [`ADR-0007-kernel-performance-diagnostics.md`](../ADR/ADR-0007-kernel-performance-diagnostics.md)
- [`tosumu-inspection-island-and-ui-providers.md`](../../third-party/tosumu/docs/Plans/tosumu-inspection-island-and-ui-providers.md)
- `corpus/consumers/tokimu-console-command-window`
- `corpus/consumers/tokimu-website-ratatui-lab`
- `third-party/presentation-providers/ratatui` (`v0.29.0` source evidence)
