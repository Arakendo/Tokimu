# Tokimu TUI Tools Corpus Study

| Field | Value |
| --- | --- |
| Status | Complete - continued incubation |
| Opened | 2026-08-06 |
| Owner | Tokimu maintainers |
| Proposed incubator | `corpus/lib/tui-tools` |
| Related reviews | AR-0013, AR-0014 |
| Related ADRs | ADR-0003, ADR-0004, ADR-0005 |
| Related plans | `terminal-surface-provider-study.md`, `tokimu-observation-shell-consumer-corpus.md`, `tokimu-console-command-window-corpus.md` |
| Reference provider | `third-party/presentation-providers/ratatui` |

## Current Progress

As of 2026-08-06, the first independent vertical slice is complete and the
study remains deliberately incubating:

- `corpus/lib/tui-tools` provides corpus-local bounded rectangles, insets,
  directional fixed/minimum/remaining layout, resolved cells, style roles,
  alignment, clipping, wrapping, and explicit diagnostics;
- `corpus/hello-tui-tools` is a non-shell consumer that projects
  application-owned operations data into a 72 by 24 status dashboard; and
- the same consumer exercises a 24 by 6 undersized surface and retains the
  resulting clipping and empty-region diagnostics without escaping bounds or
  panicking; it also renders a bounded embedded-console projection whose
  transcript review state and focused prompt remain caller-owned.

The default path does not depend on Ratatui. Terminal-surface adaptation,
viewport/focus semantics, Unicode width pressure, and broader paired Ratatui
oracle comparison remain open evidence rather than implied behavior. The
paired console and supported-size status-dashboard fixtures establish narrow
structural evidence for transcript/prompt and caller-owned label/value
retention respectively. The remaining uncertainty belongs to AR-0014 rather
than another implementation slice in this study: native raw-input behavior,
complete wide/combining-text conformance, cold and incremental build cost,
and GPU-side timing or cache behavior still need independent evidence before a
public capability or provider decision can be proposed.

## Slice 0 Inventory - 2026-08-06

The current corpus uses Ratatui in five deliberately separate roles. This is
an inventory of the public APIs actually used, not a claim that any of those
APIs belong to Tokimu.

| Consumer | Provider role | Public Ratatui surface | Boundary finding |
| --- | --- | --- | --- |
| `hello-terminal-surface` | Headless oracle | `TestBackend`, `Terminal`, `Paragraph`, `Block`, `Borders`, `Style`, `Color` | Converts resolved provider cells into a local terminal-surface candidate. It has no terminal host or application command authority. |
| `hello-observation-shell` | Optional native TUI host | `CrosstermBackend`, `Terminal`, `Layout`, `Constraint`, `Line`, `Paragraph`, `Block`, `Borders`, styles | Owns a complete terminal-host experiment behind `ratatui-standalone`; it is not a dependency of the default corpus path. |
| `tokimu-console-command-window` | Headless comparison evidence | `TestBackend`, `Terminal`, `Layout`, `Constraint`, `Line`, `Paragraph`, `Block`, `Borders` | Compares terminal composition with the native console corpus. Ratatui remains feature-gated as `ratatui-evidence`. |
| `tokimu-website-ratatui-lab` | Browser retained-cell producer | `Terminal`, corpus-local `TokimuBackend`, `Layout`, `Constraint`, `List`, `Gauge`, `Paragraph`, `Block`, `Borders`, `Line`, `Span`, styles | Ratatui composes deterministic dummy-data scenes; the custom backend retains changed cells and Tokimu rasterizes them. TypeScript only controls and blits pixels. |
| `runtime-observation-workbench` | Browser interactive shell projection | `Backend`, `WindowSize`, `Terminal`, vertical `Layout`, `Constraint`, `Text`, `Line`, `Paragraph`, `Block`, `Borders`, styles | Ratatui owns terminal composition for a Rust/WASM session. The application still owns semantic commands and observations. |

`tui-tools` itself has no normal dependencies (`cargo tree -p tui-tools -e
normal` contains only the crate). Its `hello-tui-tools` consumer depends only
on `tui-tools`. By contrast, even the default-feature-disabled Ratatui oracle
pulls Ratatui's layout, widget, style, Unicode, cache, and support dependency
surface. That is useful cost evidence, but it is not yet a payload or runtime
measurement and does not decide provider composition.

The inventory supports one narrow conclusion: Ratatui is currently an optional
provider for rich composition and terminal hosting, while `tui-tools` is only
testing a smaller dependency-free embedded-console projection. Neither path
owns shell meaning, application state, font parsing, host input, or renderer
resources.

## Purpose

Build a corpus-local Tokimu TUI implementation that can teach us which
terminal-shaped presentation concepts Tokimu may need to own and which should
remain provider behavior.

Ratatui is the behavioral oracle and teacher for this study. It is not the
definition of the Tokimu API, a source from which code should automatically be
copied, or evidence by itself that Tokimu should own a terminal framework.

The study begins in `corpus/lib/tui-tools`. It does not create a public
`tokimu-tui` crate, add Ratatui to engine-core dependencies, or settle AR-0014.

## Research Thesis

The existing terminal-surface study has established a useful lower boundary:
a provider can resolve a bounded grid of styled cells, cursor state, and
validated full or changed-cell observations before Tokimu rasterizes it.

This plan studies the layer immediately above that boundary:

```text
application or tool meaning
    -> TUI composition and interaction
    -> resolved terminal surface
    -> Tokimu text and font presentation
    -> renderer or browser output
```

The central question is not "How do we clone Ratatui?" It is:

> Which small, durable TUI semantics recur across Tokimu consumers after
> Ratatui's implementation vocabulary is removed?

Ratatui may show a working pattern, expose edge cases, and provide differential
evidence. Tokimu should own a concept only when independent consumers need its
meaning and can express it without depending on Ratatui types or internals.

## Embedded Console Scope Hypothesis

The study does not assume that one terminal presentation provider is the right
default at every scale.

```text
Full terminal application
    -> optional Ratatui provider

Embedded "~" console inside an application with an existing UI
    -> small Tokimu-authored composition candidate in tui-tools

Both
    -> the same lower terminal-surface handoff if later evidence supports it
```

An embedded console-sized consumer may need only a bounded transcript, prompt
line, cursor presentation, history review, focus indication, scrolling, and a
small set of style roles. It must not cause `tui-tools` to recreate Ratatui's
full composition model, widget catalog, terminal host, or application command
authority.

The embedded path is justified only if it stays materially smaller and simpler
than the Ratatui provider while covering its narrow consumer boundary. Ratatui
remains the optional rich provider for full terminal applications.

Evidence required before admitting a small embedded path beyond this corpus:

- record the Ratatui surface area and adapter complexity that an embedded
  console actually uses;
- compare native and WASM compile, payload, startup, and warm costs;
- prove prompt, transcript, scroll, focus, and resize behavior remain bounded
  without a hidden second widget framework; and
- observe at least two non-Ratatui consumers needing the same small semantics.

## Relationship To Existing Work

This plan consumes rather than replaces the terminal-surface study.

`terminal-surface-provider-study.md` owns evidence for:

- bounded rows and columns;
- resolved cells and styles;
- cursor state;
- complete frames and changed-cell observations;
- epoch, extent, continuation, clipping, and invalidation behavior; and
- provider-neutral raster handoff.

This plan studies candidate behavior above that surface:

- region composition and bounded layout;
- styled text placement into regions;
- viewport and scroll state;
- focus and normalized actions;
- small reusable view components;
- diagnostics for impossible or undersized layouts; and
- deterministic projection into the existing terminal-surface vocabulary.

AR-0013 remains authoritative for shell sessions, command catalogs, history,
requests, and command outcomes. A TUI may present those semantics but must not
own them.

AR-0014 remains authoritative for the terminal-surface and Ratatui dependency
boundary. This plan supplies evidence to that review; it cannot admit a public
capability by itself.

## Ownership Boundary

```text
Application or tool
    owns domain meaning, commands, records, and authoritative state

Corpus-local tui-tools
    studies bounded composition, view state, normalized actions,
    deterministic projection, and explicit layout diagnostics

Terminal-surface candidate
    owns resolved cells, styles, cursor, extent, frame lifecycle, and damage

Ratatui oracle/provider
    owns its widgets, layout solver, Buffer, style types, backend protocol,
    Unicode policy, and provider-specific behavior

Tokimu text and font presentation
    owns provider-neutral text contracts and raster execution through
    replaceable font providers

Renderer and platform adapters
    own pixels, GPU resources, window/browser mechanisms, focus delivery,
    and normalized device input
```

The study must not let `tui-tools` become an owner of application truth,
command authority, PTYs, ANSI parsing, filesystem access, browser DOM meaning,
or renderer-native resources.

## Ratatui As An Oracle

Ratatui should be used in four ways:

1. **Behavioral reference** - render bounded fixtures through Ratatui and
   record the resulting cell surface, cursor, clipping, and resize behavior.
2. **Edge-case teacher** - inspect how mature Ratatui APIs make ambiguity
   explicit, especially around undersized regions, Unicode width, style reset,
   scrolling, selection, and stateful widgets.
3. **Differential pressure** - compare Ratatui and Tokimu-authored projections
   using shared semantic fixtures and classify every divergence.
4. **Cost reference** - record source, dependency, linked payload, startup,
   allocation, and warm-frame costs for comparable native and WASM scenes.

The oracle does not require byte-for-byte or pixel-for-pixel identity where
multiple presentations satisfy the same semantic contract. Comparisons should
prefer invariants such as bounds, visible item identity, selected row, scroll
offset, style role, cursor location, and rejected-layout diagnostics.

If exact parity is required for a fixture, the fixture must explain why that
specific output is contractually meaningful rather than merely familiar.

## Shared Renderer-Seam Evidence

Ratatui's `TestBackend` remains a feature-gated composition oracle. It produces
provider-local terminal cells, not presentation pixels.

`tui-tools::rasterize_cells` is the shared deterministic CPU cell-to-RGBA seam:

- The native `hello-tui-tools` corpus maps its `Surface` through
  `rasterize_surface`, selects Departure Mono outside `tui-tools`, and records
  deterministic dimensions and a frame fingerprint.
- The website Ratatui workbench maps a Ratatui buffer into `TuiRasterCell`,
  selects the same font at its presentation boundary, and calls
  `rasterize_cells` directly.

The paired dashboard oracle now exercises both routes at `48 x 14` and
`64 x 18`: the corpus-local dashboard calls `rasterize_surface`, while the
Ratatui `TestBackend` buffer is translated into `TuiRasterCell` and calls
`rasterize_cells`. Each route must produce the expected RGBA dimensions and
byte count, non-empty pixels, and a repeatable provider-local fingerprint.
The oracle deliberately does not require identical fingerprints across the
two providers: border glyphs and style choices remain provider-local rather
than shared terminal semantics.

Expected provider-local differences are typed records rather than a broad
ignore rule. The only admitted dashboard divergences are border composition
and style composition, and each record requires a non-empty reason. Missing
caller-owned text, invalid dimensions, empty frames, and nondeterministic
raster output still produce oracle findings.

Default `tui-tools` exposes no Ratatui types and selects no font. The
provider-specific adapter owns only Ratatui cell and style translation. Native
and website-adapter tests assert output dimensions, RGBA length, and repeatable
CPU fingerprints; they do not claim browser-canvas or GPU-framebuffer
equivalence.

## Paired Console Oracle Evidence

The optional `ratatui-oracle` feature now compares the same caller-owned
embedded-console fixture through both composition paths. Its public report
contains only provider-neutral evidence:

- the bounded extent;
- the viewport-selected transcript-line count;
- the expected prompt row; and
- explicit findings identifying a missing visible line, missing prompt, or a
  prompt that was not pinned to the shared terminal row.

The comparison intentionally does not compare border symbols, style encodings,
or pixel output. Those are provider implementation details. A passing paired
fixture currently establishes only that Tokimu and Ratatui preserve the same
shared transcript and prompt invariants for a bounded console. Wider layout,
Unicode continuation, focus, and widget evidence remains separate work.

The same feature-gated oracle also compares the caller-owned status dashboard
at `48 x 14` and `64 x 18`. It requires the title, subtitle, section headings,
field labels and values, and footer to survive both composition paths. This is
supported-extent layout and resize evidence only; it does not establish
undersized-layout equivalence, visual parity, or a shared style system.

## Clean-Room And Fork Boundary

The default implementation strategy is independent Tokimu-authored code based
on observed behavior and public concepts, not copied Ratatui implementation.

Before adapting or lifting any implementation block, record:

- why the behavior cannot be expressed from the observed contract alone;
- the upstream file and pinned revision;
- license and attribution obligations;
- whether an upstream extension point is available;
- whether the work belongs in an optional Ratatui adapter instead; and
- how divergence and future upstream fixes would be maintained.

A Ratatui fork or vendored experiment is governed by AR-0014 Alternative E and
is outside this plan unless opened as a separately scoped experiment. Such an
experiment remains an optional provider and may not leak Ratatui-derived types
into a public Tokimu contract.

## Candidate Vocabulary

All names are corpus-local and intentionally unstable:

```text
TuiExtent
TuiRect
TuiInsets
TuiConstraint
TuiLayout
TuiStyleRole
TuiTextRun
TuiViewport
TuiFocusPath
TuiAction
TuiViewState
TuiProjection
TuiDiagnostic
ConsolePrompt
```

The vocabulary should shrink when two names describe the same ownership. It
should grow only when a fixture exposes a decision that cannot be represented
honestly by an existing concept.

## Proposed Incubator Shape

The first implementation may use this shape, but module names are not a public
contract:

```text
corpus/lib/tui-tools/
    Cargo.toml
    src/
        lib.rs
        geometry.rs
        layout.rs
        text.rs
        viewport.rs
        input.rs
        projection.rs
        diagnostics.rs
        views/
        oracle/
            ratatui.rs
    tests/
        layout.rs
        viewport.rs
        projection.rs
        oracle_conformance.rs
```

The Ratatui adapter should be feature-gated so consumers that use only the
Tokimu-authored path do not link Ratatui accidentally.

## Initial Fixture Matrix

| Fixture | Pressure | Required observations |
| --- | --- | --- |
| Status panel | Bounded labels and values | Stable regions, clipping, style roles |
| Command transcript | Append, wrap, scroll, cursor | Live-tail behavior, review offset, prompt focus |
| Asset inspector | List/detail composition | Selection identity, viewport, narrow-layout rejection |
| System monitor | Table-like repeated rows | Column policy, truncation, changing values |
| Resource browser | Tree/list navigation | Focus path, expansion state, selected resource |
| Form/dialog | Focus traversal and actions | Normalized actions, disabled state, explicit submit/cancel |
| Resize torture | Small and changing extents | Deterministic fallback or rejection, no escaping cells |
| Unicode torture | Width and continuation pressure | Provider decisions retained without reinterpretation |

Each fixture should have one application-owned semantic input, one
Tokimu-authored projection, an optional Ratatui projection, and structural
artifacts that identify the first divergent stage.

## Diagnostic Artifacts

Each fixture should be able to emit a bounded artifact set:

```text
input.json
layout.json
view-state.json
surface.json
delta.json
diagnostics.json
metrics.json
reference.png        optional presentation evidence
```

Artifacts should identify schema version, producer, algorithm or provider,
extent, fixture revision, input hash, and target. Structural artifacts are
authoritative for ownership analysis; screenshots are complementary evidence.

The first stage whose artifact diverges is the owning diagnostic boundary.

## Non-Goals

- Reimplementing all Ratatui widgets or preserving Ratatui source compatibility.
- Creating a public TUI crate before independent consumers prove the boundary.
- Building a PTY, terminal emulator, ANSI parser, or host shell.
- Moving shell command meaning, history, or application state into `tui-tools`.
- Making `tokimu-core` or `tokimu-runtime` depend on Ratatui or `tui-tools`.
- Replacing ADR-0004 text semantics or owning a font parser in the TUI layer.
- Defining one global Unicode width or shaping policy from one provider.
- Claiming screenshot similarity proves semantic conformance.
- Forking Ratatui as a shortcut around ownership analysis.

## Implementation Slices

### Slice 0: Freeze The Study Boundary

Deliverables:

- [x] Link this plan from AR-0014 without changing its disposition.
- [x] Record which terminal-surface types and fixtures are reused rather than
      duplicated.
- [x] Inventory current Ratatui usage across Tokimu corpus and website consumers.
- [x] Classify every use as application meaning, composition, surface mechanics,
      text behavior, host input, rendering, or diagnostics.

Acceptance criteria:

- [x] The study has no proposed dependency from `tokimu-core` or
      `tokimu-runtime` to Ratatui or corpus code.
- [x] Shell/session meaning and resolved terminal-surface behavior remain owned
      by their existing reviews and plans.
- [x] The inventory identifies exact public Ratatui APIs used by each consumer.

Boundary evidence: `tui-tools` declares Ratatui as an optional dependency
behind `ratatui-bridge` and `ratatui-oracle`. The normal dependency trees for
`tokimu-core` and `tokimu-runtime` contain neither Ratatui nor corpus crates.
AR-0013 and the observation-shell plan retain ownership of shell and session
meaning; this study only evaluates bounded terminal-surface composition and
its renderer-facing handoff.

### Slice 1: Scaffold `corpus/lib/tui-tools`

Deliverables:

- [x] Add a corpus library with no Ratatui dependency in its default feature set.
- [x] Add deterministic test fixtures and versioned artifact metadata.
- [x] Reuse or adapt the terminal-surface candidate through an explicit local
      boundary.
- [x] Add a feature-gated Ratatui oracle module without exposing its types from
      the default API.

Acceptance criteria:

- [x] Default tests compile and run without Ratatui linked.
- [x] Enabling the oracle changes only the corpus comparison path.
- [x] No corpus-local type is re-exported from a stable Tokimu crate.

### Slice 2: Bounded Geometry And Layout

Deliverables:

- [x] Implement explicit terminal-space rectangles and insets.
- [x] Implement a minimal directional split using fixed, minimum, and remaining
      constraints only where fixtures require them.
- [x] Diagnose impossible, empty, and undersized layouts without coordinate
      underflow or escaping cells.
- [x] Compare supported-size status-panel and resize fixtures with Ratatui.

Acceptance criteria:

- [x] Layout output is deterministic for the same extent and inputs.
- [x] Child regions never escape their parent.
- [x] Unsatisfied constraints produce explicit diagnostics.
- [x] Supported-size structural differences are classified instead of silently
      copied; undersized-layout divergence remains open evidence.

### Slice 3: Styled Text Projection

Deliverables:

- [x] Project bounded text runs and style roles into resolved cells.
- [x] Exercise alignment, truncation, wrapping, style reset, and empty content.
- [x] Preserve explicit provider-resolved continuation behavior at the
      terminal-surface boundary.
- [x] Add an independent ASCII and wide-grapheme fixture without declaring one
      provider's width algorithm to be Tokimu truth.

Acceptance criteria:

- [x] Text never writes outside its assigned region.
- [x] Style does not leak from one run or cell into another.
- [x] Explicit continuation behavior is visible in independent-provider
      observations; Ratatui's public trailing-cell metadata remains an open
      comparison limit.
- [x] ADR-0004 text and font ownership remains unchanged.

### Slice 4: Viewport And Scroll State

Deliverables:

- [x] Add explicit viewport extent, content extent, offset, and live-tail state.
- [x] Exercise transcript append, list scrolling, resize, and return-to-live
      behavior through the `hello-tui-tools` transcript fixture.
- [x] Define clamping and invalid-offset diagnostics.
- [x] Compare command-transcript and a bounded asset-inspector fact panel with
      Ratatui. The initial inspector deliberately contains four portable facts
      (`name`, `kind`, `meshes`, and `primitives`) so it remains complete at
      the published 48x14 minimum extent; richer inspector composition needs
      separate resize evidence rather than silent clipping.

Acceptance criteria:

- [x] Appending content follows the tail only while live-tail is active.
- [x] Reviewing history does not lose the authoritative selection or content.
- [x] Resize and content shrink clamp offsets deterministically.
- [x] Scroll state remains view state, not application truth.

### Slice 5: Normalized Focus And Actions

Deliverables:

- [x] Define corpus-local semantic actions such as move, activate, cancel,
      page, home, end, and text input.
- [~] Map keyboard, mouse-wheel, and pointer observations through host adapters.
      `hello-tui-tools` now proves a corpus-local raw-input adapter reduces
      keyboard, normalized wheel direction, text, and transcript hit-test
      observations into `TuiAction` without importing platform types. The
      browser `runtime-observation-workbench` now independently reduces DOM
      keyboard and wheel observations into its Rust/WASM Ratatui session
      actions through a DOM-free consumer-local mapper, validated by
      `npm run test:input`. Canvas focus, pointer delivery, and browser event
      conventions remain browser-owned. Native-host adapter evidence remains
      required.
- [x] Add explicit focus path and disabled-action behavior.
- [x] Exercise form, resource-browser, and transcript fixtures. The
      `hello-tui-tools` resource-browser fixture keeps filter content,
      resource selection, and inspection activation in the consumer while
      exercising only the library's bounded surface, focus path, and
      normalized action vocabulary.

Acceptance criteria:

- [x] `tui-tools` receives normalized actions rather than platform key codes.
- [x] Focus changes are deterministic and bounded.
- [x] Disabled actions cannot mutate view or application state.
- [x] Application commands remain application-owned requests.

Implementation evidence: `TuiAction`, `TuiFocusPath`, and
`TuiViewport::apply_action` distinguish view-local navigation from
caller-owned activation and text handling. The library has no platform key
codes or event-loop dependency; hosts map raw input at their own boundary.
`hello-tui-tools` applies the same normalized actions through its transcript
and embedded-console projections before rasterizing them through the Tokimu
text path, providing a non-Ratatui consumer proof. This does not yet prove
the native raw-event mapping.
Its corpus-local `host_input` fixture additionally establishes the intended
direction: host key/wheel/pointer observations are translated before they
enter `tui-tools`; hit-testing and raw event conventions remain host-owned.
The browser runtime-observation consumer independently follows that direction:
its DOM-free mapper reduces keyboard and wheel input to semantic Rust/WASM
session actions, while canvas focus and pixel delivery remain browser-local.

### Slice 6: Minimal Reusable Views

Deliverables:

- [~] Implement only the smallest repeated views proven by the fixture matrix.
      A bounded label/value row is admitted and reused by the status dashboard;
      selectable lists, bordered regions, and a transcript viewport remain
      pending independent consumer pressure.
- [x] Add one embedded-console projection with caller-owned transcript and
      prompt display; do not add command parsing, history ownership, or
      terminal-host behavior.
- [x] Keep application data outside view implementations.
- [x] Record where composition helpers end and provider-style widgets begin.
- [x] Reject pressure to mirror Ratatui's complete widget catalog.

Acceptance criteria:

- [x] At least two fixtures reuse each admitted helper.
- [x] Adapt helpers to the terminal-surface boundary; they currently project
      into a compatible corpus-local bounded resolved-cell vocabulary.
- [x] Default Tokimu-authored consumer builds and deterministic fixtures run
      with the Ratatui oracle absent; optional oracle comparisons do not alter
      their independently produced output.
- [x] The embedded-console prompt stays pinned while transcript review state
      changes, and its focus/cursor state is explicit caller-owned view data.
- [x] No helper acquires shell, filesystem, network, or renderer authority.

Evidence and boundary result:

`tui-tools` now admits one narrow reusable handoff: a complete resolved cell
buffer rasterized through a caller-selected `ui-tools` font rasterizer with
explicit cell metrics, baseline, canvas, and terminal-style flags. Both
`hello-tui-tools` and `hello-terminal-surface` consume that handoff. The
terminal-surface corpus retains terminal-specific color resolution, reverse
video, grapheme/continuation interpretation, and Ratatui-to-cell adaptation;
Ratatui itself retains terminal layout and widget composition. No shared
selectable-list or generic bordered-widget API is admitted. The bounded
`label/value` row is the only composition helper so far: it preserves distinct
label and value style roles while reserving a caller-selected value column.
The present dashboard and console uses do not yet demonstrate broader repeated
semantic view contracts. This deliberately rejects a mirror of the Ratatui
widget catalog while proving that provider-composed cells can reach the same
Tokimu font-raster seam as Tokimu-authored cells.

Default `cargo test` and `cargo tree -e normal` checks now prove that
`tui-tools`, `hello-tui-tools`, and `hello-terminal-surface` validate without a
normal Ratatui dependency. The `ratatui-oracle` feature remains an opt-in
comparison path. This is dependency-isolation evidence, not a claim of
provider output equivalence.

## Continuation Evidence

The shared raster seam now carries an explicit `continuation` bit on its
normalized cell input. A continuation preserves the cell background but emits
neither glyph ink nor text decorations. `hello-terminal-surface` proves this
with an independent `A界B` fixture: its provider resolves the wide grapheme and
marks the trailing cell as a continuation before the common raster seam runs.

This is deliberately not a Tokimu-wide Unicode width policy. Ratatui 0.29
resets the trailing cells of a multi-width grapheme to blank cells in its
public `Buffer` and does not expose a public continuation marker; its `skip`
flag concerns graphics diff delivery, not text layout. The Ratatui bridge
therefore preserves only metadata it can observe and does not guess which blank
cell follows a wide grapheme. Cross-provider wide/combining equivalence remains
open evidence.

### Slice 7: Oracle Conformance Harness

Deliverables:

- [x] Run the paired embedded-console fixture from shared caller-owned input.
- [x] Compare transcript visibility and pinned-prompt structural invariants
      without treating provider border or style encoding as a mismatch.
- [~] Extend the report to input, layout, view-state, surface, and raster stages
      as additional shared fixtures are admitted. Console viewport/prompt,
      normalized viewport navigation (`MovePrevious`, `PagePrevious`, `Home`,
      and `End`), supported-size dashboard layout/resize, and provider-local
      CPU-raster comparisons are complete. The paired dashboard oracle now
      publishes one `8 x 6` minimum extent and rejects smaller grids identically
      for semantic and raster comparisons. The browser
      `runtime-observation-workbench` additionally validates its DOM-free
      keyboard and wheel mapping before it calls the Rust/WASM Ratatui session.
      Native-host raw-event translation, editable focus/text-entry, and
      cross-provider visual-contract comparisons remain open. The native
      terminal viewer currently accepts resize events only and is therefore
      presentation evidence, not native terminal-input evidence. The
      Tokimu-authored dashboard's diagnostic degraded layout
      below that oracle threshold remains separate behavior rather than a
      claimed provider-parity contract.
- [x] Add expected-divergence records for provider-specific behavior.

Acceptance criteria:

- [x] A failing embedded-console comparison identifies the first divergent
      projection boundary: Tokimu surface or Ratatui composition.
- [x] Expected differences require a reason and cannot be blanket-approved.
- [x] Ratatui types remain confined to the feature-gated oracle adapter.
- [x] Embedded-console conformance does not depend on screenshots.

### Slice 8: Native And WASM Presentation

Deliverables:

- [x] Present the same Tokimu-authored fixtures through native and browser paths.
- [x] Use the existing text/font presentation and terminal-surface raster path.
- [x] Record linked payload, startup, allocation, warm-frame, and update costs
      with the Ratatui oracle enabled and disabled. Local linked-payload,
      CPU-composition, retained CPU-frame, and native warm-frame observations
      are recorded. They remain local observations, not cross-machine budgets.
- [x] Prove that browser TypeScript forwards controls and pixels without
      recreating TUI layout.

Acceptance criteria:

- [x] Native and WASM artifacts identify provider and target composition.
- [x] A consumer that does not request Ratatui does not link it transitively.
- [~] Identical resolved CPU surfaces reuse immutable font state and their
      complete raster frame; changed surfaces record an explicit complete CPU
      invalidation. Pipeline reuse, partial GPU updates, and session-lifetime
      cache policy remain separate evidence.
- [x] Deployment measurements are observations, not universal budgets.

Slice 8 evidence as of 2026-08-06:

- The native viewer submits the selected producer's already-resolved RGBA
  surface through Tokimu's normal sRGB texture upload and textured-quad path.
  It does not re-run terminal layout, Ratatui composition, or font rasterizing.
- The native viewer records startup separately, then reports its first frames
  and periodic warm frames with CPU-side acquire, resource preparation,
  command encoding, queue-submit, and surface-present call timings alongside
  renderer churn counters. This is execution evidence for the retained raster,
  not a claim about GPU completion, display latency, framebuffer readback, or
  a general GPU cache policy.
- The browser facade selects `independent` or `ratatui`, asks Rust/WASM for the
  complete raster dimensions and bytes, and presents them with Canvas 2D
  `putImageData`. TypeScript does not position glyphs, interpret styles, or
  recreate terminal layout.
- Local release artifacts measured `319,158` bytes without the optional
  Ratatui producer and `454,152` bytes with it: a `134,994` byte WASM delta.
  Native executables measured `6,191,104` and `6,298,624` bytes respectively:
  a `107,520` byte local link delta.
- In one 256-iteration local CPU composition run, the independent producer
  averaged `505 us` and the Ratatui producer averaged `909 us`. These are
  corpus observations, not admission thresholds or cross-machine budgets.
- The same measured lifecycle now retains a corpus-local complete CPU raster.
  On the 2026-08-06 development machine, both the independent and optional
  Ratatui producers performed one Departure Mono provider load and one CPU
  rasterization across 256 identical resolved surfaces, followed by 255 cache
  hits. A changed resolved surface is deliberately recorded as a complete CPU
  invalidation; this does not yet claim partial texture upload or GPU cache
  behavior.

### Slice 9: Independent Consumer Pressure

Deliverables:

- [x] Exercise the Tokimu-authored terminal fixture through the shared surface
      and raster path in `hello-terminal-surface`.
- [x] Exercise one non-shell consumer through the same path in
      `hello-tui-tools`.
- [x] Keep the Ratatui-backed website consumer on the same lower raster
      boundary through the optional `ratatui-bridge` feature.
- [x] Record duplicated logic that remains outside `tui-tools`.

Acceptance criteria:

- [x] The terminal fixture and the status-dashboard consumer reuse the shared
      `Surface` and CPU raster concepts while retaining different semantics.
- [x] The status dashboard has no prompt, transcript, command history, or
      viewport assumptions.
- [x] Caller-owned state remains authoritative: `tui-tools` composes and
      rasterizes supplied facts rather than owning runtime or shell state.
- [x] Ratatui `Buffer` to normalized-cell mapping and CPU frame allocation now
      live in the optional `tui-tools` bridge instead of being duplicated by
      the website consumer and Ratatui oracle.

Retained provider-local work is intentional: Ratatui retains widget layout,
terminal composition, and its native style vocabulary; the Tokimu-authored
terminal fixture retains transcript and prompt policy; consumers select fonts,
own browser/native presentation, and decide how a raster frame reaches pixels.

### Slice 10: Admission And Provider Decision

Deliverables:

- [x] Update AR-0014 with conformance, dependency, payload, and consumer evidence.
- [x] Classify each candidate concept as Tokimu-owned, provider-owned,
      application-owned, rejected, or still incubating.
- [x] Decide the current provider disposition: retain upstream Ratatui as an
      optional provider and oracle, while deferring a default-provider choice,
      upstream-seam request, fork/vendor experiment, public capability
      admission, and retirement of the Tokimu-authored corpus path.
- [ ] Record any permanent decision in an ADR only after the ownership boundary
      survives independent use.

Acceptance criteria:

- [x] No decision is justified only by source similarity or binary size.
- [x] No fork proposal exists; any future fork proposal remains subject to every
      AR-0014 Alternative E gate.
- [x] No public terminal-capability contract is admitted, and the corpus-local
      base surface contains no Ratatui-specific types.
- [x] Continued incubation is the recorded outcome.

#### Slice 10 Review Result

The present result closes this study slice without graduating a capability.

- **Still incubating:** `Surface`, normalized terminal cells, and the CPU raster
  path are Tokimu-authored corpus candidates. `hello-terminal-surface` and the
  non-shell `hello-tui-tools` dashboard prove that these lower concepts have
  independent pressure, but not yet the repeated composition, viewport, action,
  and diagnostic semantics required for public admission.
- **Provider-owned:** Ratatui retains its `Buffer`, layout, widget composition,
  terminal-host behavior, and native style vocabulary. The optional
  `tui-tools` bridge maps its provider-local buffer into the normalized-cell
  seam; the base `tui-tools` path does not depend on Ratatui.
- **Application-owned:** transcript policy, prompts, command history, focus,
  viewport policy, session lifetime, and command/action meaning remain owned by
  the terminal fixture or another calling application.
- **Consumer and platform-owned:** font selection, browser canvas or native
  target selection, event forwarding, texture upload, and final pixel delivery
  remain outside the terminal-surface candidate.
- **Rejected for now:** a public terminal capability, default TUI provider,
  Ratatui fork or vendor experiment, and a provider-neutral shell-session
  contract. These need more independent evidence than the current corpus has
  produced.

This is an evidence result, not an ADR. AR-0014 remains the authoritative
record for reopening the question when the graduation criteria are met.

## Candidate Graduation Criteria

A permanent Tokimu TUI capability should be proposed only when all of the
following are true:

- at least two independent non-Ratatui consumers need the same composition,
  viewport, action, and diagnostic semantics;
- one Ratatui provider can project through the same lower surface without
  leaking its implementation vocabulary;
- native and WASM consumers select provider composition explicitly;
- Unicode, clipping, resize, cursor, style reset, viewport, focus, and invalid
  layout cases have deterministic evidence;
- application meaning remains outside the TUI layer;
- text, font, renderer, and platform ownership still match accepted ADRs;
- deployment and runtime costs are measured with equivalent scenes; and
- AR-0014 selects admission rather than continued incubation or retirement.

Graduation does not require replacing Ratatui. A small Tokimu-owned capability
and a richer optional Ratatui provider may coexist if the evidence proves that
split.

## Open Questions

- Are bounded layout constraints terminal-specific, or are they a projection
  of a more general presentation-layout capability?
- Should focus and viewport state live in a TUI capability or remain entirely
  consumer-owned with only helper algorithms shared?
- Which style concepts are provider-neutral roles versus terminal-cell details?
- Can wide and combining text remain fully provider-resolved while preserving
  deterministic interaction and selection behavior?
- Is changed-cell delivery sufficient for all retained views, or do some
  consumers need semantic damage regions?
- Does a web-focused minimal composition justify a separate provider from the
  richer native Ratatui composition?
- Would an upstream Ratatui seam remove enough copying and payload cost to make
  a Tokimu-authored TUI unnecessary?
- If a fork experiment becomes necessary, what exact seam, maintainer, rebase
  policy, compatibility suite, and retirement trigger keep it bounded?

## Pause Conditions

Pause and return to AR-0014 rather than widening this plan when:

- a proposed type cannot be described without Ratatui vocabulary;
- Unicode behavior would require Tokimu to silently invent a width policy;
- a helper starts owning shell or application meaning;
- a provider-specific workaround is being generalized after one fixture;
- the browser build inherits Ratatui without an explicit consumer choice; or
- a fork is proposed without the Alternative E evidence and maintenance gates.

## Definition Of Done For This Study

This plan is complete when the corpus can answer, with retained artifacts and
measured consumers:

1. Which TUI semantics Tokimu consumers independently require.
2. Which behaviors remain Ratatui or provider responsibilities.
3. Whether a small Tokimu-authored path is materially useful on native and WASM.
4. Whether upstream Ratatui, a bounded fork, or no first-party TUI is the right
   long-term provider composition.
5. Whether the evidence warrants an ADR, continued incubation, or retirement.
