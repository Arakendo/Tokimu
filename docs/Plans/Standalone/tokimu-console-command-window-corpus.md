# Tokimu Console Command Window Corpus

## Status

Active prerequisite corpus.

Slices 0 through 3 and the Slice 6 readiness review are complete. The
remaining work is bounded native-session and warm-frame lifecycle evidence in
Slices 4 and 5. Official Observation Shell or Ratatui provider admission
remains parked until another independent host proves that the corpus-local
adapters are stable semantics rather than convenient implementation shape.

This plan creates a graphical Tokimu console/command window before work begins
on Tosumu's
[`tosumu-inspection-island-and-ui-providers.md`](../../../third-party/tosumu/docs/Plans/tosumu-inspection-island-and-ui-providers.md).
It is a focused presentation and provider-composition proof, not admission of a
terminal emulator, command language, shell capability, or Ratatui dependency
into Tokimu's engine crates.

## Architectural Review Relationship

This corpus is the first embedded-provider evidence for
[`AR-0013`](../../Architectural%20Reviews/AR-0013-observation-shell-and-ratatui-presentation-provider.md).
It does not define the Observation Shell contract and does not admit Ratatui as
an official provider by itself.

The review distinguishes three responsibilities:

- Tosumu and other capability owners own observations and command meaning;
- a candidate Observation Shell owns provider-neutral session composition;
- Ratatui and Tokimu UI own replaceable projection and host mechanics.

Provider admission requires the same shell session to survive both a standalone
Ratatui terminal host and a bounded embedded cell-grid host without semantic
drift.

## Purpose

Build a native Tokimu corpus application that looks and behaves like a bounded
console window, uses Departure Mono through the provider-neutral text path, and
can present a deterministic Tosumu command session through both Tokimu UI and
Ratatui-facing projections.

The corpus should answer:

> Can Tokimu present an interactive, text-dense command session with stable
> cell metrics, cursor behavior, scrolling, and diagnostics while Tosumu owns
> command meaning and Ratatui remains a replaceable presentation provider?

```text
Tosumu command/session meaning
        |
        v
bounded observation and command results
        +---------------------------+
        |                           |
        v                           v
Ratatui TestBackend buffer     Tokimu console model
        |                           |
        +-------------+-------------+
                      v
          Tokimu text and UI presentation
                      |
                      v
            Departure Mono provider
                      |
                      v
                 native window
```

## Relationship To Existing Plans

This plan is deliberately narrower than
[`tokimu-observation-shell-consumer-corpus.md`](tokimu-observation-shell-consumer-corpus.md).
That plan investigates reusable session, catalog, routing, watch, and output
projection semantics. This plan only supplies presentation evidence for one
bounded command window.

The Tosumu inspection-island plan remains responsible for extracting a shared
inspection observation and command contract from Tosumu's current Ratatui
inspector. This prerequisite must not preempt that ownership work by treating
Ratatui widgets or CLI JSON as Tosumu semantics.

## Existing Evidence

- ADR-0004 admits provider-neutral text semantics and keeps font technologies
  replaceable.
- ADR-0005 and AR-0012 provisionally admit Departure Mono as Tokimu's
  first-party native default font provider.
- `hello-ui-textinput` exercises native text input, focus, editing, and caret
  behavior.
- `hello-ui-font2` exercises Departure Mono beside other font providers.
- `hello-ui-text-vectors` exercises glyph outline and fill behavior.
- Tosumu already has TQL parsing and dispatch in `tosumu-cli` and a Ratatui +
  Crossterm inspector in `tosumu view`.
- Ratatui can render deterministically to a headless cell buffer without a real
  terminal.
- Tokimu's runtime-inspector and UI hardening corpora provide layout,
  clipping, scrolling, and consumer-safety evidence.

The missing evidence is one sustained, text-dense command interaction that
combines these parts without leaking ownership across them.

## Current Implementation Status

The first native corpus scaffold is available at
`corpus/consumers/tokimu-console-command-window`.

It now proves the local presentation mechanics and a provisional Tosumu CLI
adapter boundary:

- Departure Mono resolves through `UiFontSource::from_native_default()`;
- a bounded transcript renders in a native Tokimu window;
- prompt focus, text insertion, punctuation, spaces, backspace/delete, Escape,
  and Enter submission are handled through the existing UI input contracts;
- the transcript has a fixed visible viewport, follows new output, supports
  wheel review, and keeps prompt recall separate through Up/Down history; and
- `HELP` and `CLEAR` are explicitly local fixture commands; all other submitted
  input is forwarded unchanged to `tosumu tql <store> <command> --json`;
- Tosumu's schema-versioned JSON envelope is projected into a bounded
  owner-labeled transcript without linking Tosumu or reproducing TQL grammar;
- resize invalidates the presentation projection without treating it as a
  session or command-semantics change; and
- native text uses a finite set of deterministic resource slots for the title,
  metadata, visible transcript rows, prompt, and footer. A text change replaces
  its existing source texture/material slot rather than growing renderer
  resource maps for the lifetime of a command session.

This is not a TQL implementation or a reusable Ratatui adapter. The real TQL
process boundary is now exercised. Slice 1 produces deterministic headless
session artifacts, Slice 2 produces deterministic Ratatui `TestBackend` cell
evidence at reviewed dimensions, and Slice 3 now lowers that complete cell grid
into a corpus-local provider-neutral layout artifact. A deterministic CPU
raster adapter now consumes that artifact and emits visual evidence without
claiming GPU framebuffer equivalence.

## Goals

- Exercise Departure Mono as the native command-window font.
- Validate fixed-advance text, baseline, caret, selection, and scroll behavior.
- Present deterministic Tosumu commands, outcomes, and diagnostics.
- Compare Tokimu-native and Ratatui-facing projections of the same session.
- Prove Ratatui cell buffers can be lowered into Tokimu presentation without
  importing terminal mechanics into engine crates.
- Produce artifacts that can distinguish session, cell-layout, text-layout,
  and renderer failures.
- Identify reusable evidence needed by the later Tosumu inspection island.

## Non-Goals

- PTY, subprocess, operating-system shell, or arbitrary command execution.
- ANSI/VT100/xterm compatibility.
- Running Crossterm inside a Tokimu window or browser.
- Reimplementing Tosumu TQL parsing in Tokimu.
- Promoting Ratatui, Tosumu, Departure Mono parsing, or terminal cells into
  `tokimu-core` or `tokimu-runtime`.
- Pixel-identical Ratatui and Tokimu layouts.
- Replacing `tosumu view` or beginning the full Tosumu inspection island.
- Persistent history, remote sessions, clipboard ownership, IME, bidi, or rich
  terminal escape-sequence support in the first corpus.

## Ownership And Dependency Boundary

### Tosumu Owns

- TQL grammar, parsing, command identity, validation, and outcomes;
- database and inspection facts;
- command diagnostics and storage failure meaning;
- any provider-neutral Tosumu command-session observation.

### Ratatui Provider Owns

- terminal-cell layout, styles, cursor placement, and widget composition;
- translating a shared observation into a `Buffer`;
- terminal-specific colors and focus affordances;
- Crossterm input only in the existing native terminal application.

### Tokimu Console Corpus Owns

- corpus-window composition and viewport layout;
- input editing before a line is submitted;
- local scroll position, selection, and bounded visible history;
- lowering normalized styled cells or transcript spans into Tokimu UI;
- visual diagnostics and evidence capture;
- corpus-only adapter glue that has not earned capability admission.

### Tokimu Text And UI Own

- provider-neutral measurement and glyph placement;
- clipping, bounds, alignment, and focus contracts;
- font resolution and explicit fallback diagnostics;
- rendering submission through existing presentation adapters.

### Departure Mono Provider Owns

- the pinned OTF asset and its font metrics, outlines, and glyph coverage.

The console must never infer Tosumu meaning from colored cells or rendered
text. Cells and pixels are projections, not the semantic source of truth.

## Proposed Corpus Location

```text
corpus/
    consumers/
        tokimu-console-command-window/
            Cargo.toml
            DESIGN.md
            src/
            fixtures/
            expected/
```

Reusable but still provisional adapters may incubate under `corpus/lib/` only
after a second corpus consumer needs them. No new engine crate is justified by
this plan.

## Provisional Presentation Model

Names are planning vocabulary rather than accepted APIs.

```rust
struct ConsoleObservation {
    title: String,
    transcript: Vec<ConsoleLine>,
    prompt: PromptObservation,
    status: ConsoleStatus,
    diagnostics: Vec<ConsoleDiagnostic>,
}

struct ConsoleLine {
    kind: ConsoleLineKind,
    spans: Vec<ConsoleSpan>,
}

struct ConsoleCellObservation {
    columns: u16,
    rows: u16,
    cells: Vec<StyledCell>,
    cursor: Option<CellPosition>,
}
```

The semantic transcript and the styled cell buffer remain distinct:

```text
command result
    -> semantic transcript projection
    -> Ratatui cell projection
    -> Tokimu UI projection
```

Cell styles may carry foreground, background, and bounded modifiers. They must
not carry Tosumu page objects, parser values, closures, terminal handles, or
renderer resources.

## Font Evidence Matrix

Departure Mono should be exercised at its intended integer pixel rhythm before
testing scaled presentation:

| Case | Evidence |
| --- | --- |
| Native size | fixed advance, baseline, caret, prompt, punctuation |
| 2x integer scale | stable outline scaling and cursor-cell alignment |
| Narrow viewport | clipping, horizontal policy, and no glyph overlap |
| Tall transcript | bounded scrolling and stable line height |
| Resize | deterministic reflow or explicit no-wrap behavior |
| Missing glyph | explicit fallback/provider diagnostic |
| Dense symbols | `0O1Il|{}[]()<>`, paths, hashes, and TQL punctuation |

Font metrics come from the provider. The console must not hardcode a glyph
width, ascent, descent, or baseline that only happens to fit Departure Mono.

## Deterministic Tosumu Scenario

The first reviewed scenario should use a disposable fixture and a bounded TQL
script such as:

```text
STATUS
CHECK
DESCRIBE demo/message
DESCRIBE missing/key
WAL STATUS
STATUS trailing
```

The disposable fixture setup inserts `demo/message` outside the script. The
script itself uses Tosumu's implemented read-only TQL grammar. The corpus calls
Tosumu's owned schema-versioned CLI JSON boundary and does not duplicate its
parser, dispatcher, or error semantics.

## Evidence Artifacts

Each deterministic run should retain:

```text
artifacts/console-command-window/<case>/
    session.json
    transcript.txt
    transcript.json
    ratatui-cells.json
    tokimu-layout.json
    tokimu-cell-grid.bmp
    tokimu-cell-grid-manifest.txt
    diagnostics.json
    screenshot.png
    manifest.json
```

The first diverging artifact identifies the owning diagnostic boundary:

- session/transcript divergence: Tosumu adapter or command-session issue;
- cell divergence: Ratatui projection issue;
- text-layout divergence: Tokimu text/UI issue;
- screenshot-only divergence: renderer or capture issue.

Screenshots are complementary evidence. Semantic transcript and cell/layout
artifacts remain authoritative for this corpus.

## Implementation Slices

### Slice 0: Freeze Boundaries And Fixtures

**Objective:** Establish the corpus claim before implementation creates an
accidental API.

#### Deliverables

- [x] Select one disposable Tosumu fixture and one deterministic TQL script.
- [x] Inventory the callable TQL parser/dispatcher boundary: `tosumu tql
      <store> <command> --json` exposes schema version 1 without linking
      Tosumu into Tokimu.
- [x] Pin Departure Mono revision, checksum, license, and chosen test sizes by
      reference to AR-0012 evidence and `third-party/fonts/README.md`.
- [x] Record the supported Ratatui cell-style subset.
- [x] Freeze transcript, resize, input-editing, and failure cases.

#### Acceptance Criteria

- [x] Tokimu does not duplicate Tosumu grammar or command semantics.
- [x] No engine crate depends on Tosumu, Ratatui, or the font asset repository.
- [x] The fixture contains no secrets or irreplaceable data.
- [x] Unsupported terminal and text behavior is explicit.

### Slice 1: Headless Command Session

**Objective:** Prove command meaning and transcript behavior without a window,
GPU, Ratatui terminal, or Tokimu UI.

#### Deliverables

- [x] Execute the frozen command script through the Tosumu-owned boundary.
- [x] Produce bounded semantic observations and typed diagnostics.
- [x] Emit deterministic text and JSON transcripts.
- [x] Add ordinary, invalid, missing-key, oversized-input, and unavailable
      provider-process tests.

#### Acceptance Criteria

- [x] Repeated runs produce identical semantic artifacts.
- [x] Invalid input cannot partially mutate the fixture.
- [x] Command output is bounded and owner-labeled.
- [x] No presentation provider is required for semantic tests.

Current evidence: two consecutive `console-session-evidence` runs produced the
same `session.json` SHA-256 (`D07EE2F94F61FE36FAC685AC383CE77E4719B05911AA7D31F6E99A314E420DCE`).
The fixed script covers ordinary reads, a missing-key observation, and an
intentional parser failure. Corpus-local tests separately reject oversized
input before launching the provider process and label a missing CLI process as
an explicit provider-boundary failure. A literal database-session-close
operation is not part of Tosumu's current CLI contract, so this corpus does not
invent one merely to satisfy a terminal fixture.

### Slice 2: Ratatui Headless Projection

**Objective:** Render the same session through a deterministic Ratatui buffer
without Crossterm or an interactive terminal.

#### Deliverables

- [x] Add a Ratatui `TestBackend` projection for transcript, prompt, status,
      diagnostics, and cursor.
- [x] Serialize the retained cell grid and bounded style attributes.
- [x] Test narrow, ordinary, and resized terminal dimensions.
- [x] Keep key mapping and event-loop behavior outside the shared projection.

#### Acceptance Criteria

- [x] The buffer is generated from semantic observations, not storage objects.
- [x] Ratatui does not redefine command outcomes or diagnostics.
- [x] Cell snapshots are deterministic for fixed dimensions.
- [x] Crossterm is not required by the headless corpus runner.

Current evidence: `ratatui-session-evidence` emits a complete `96x28` cell
artifact with 2,688 cells, including styled empty cells that retain terminal
layout and background evidence. Narrow (`32x9`), ordinary (`64x18`), and
resized (`96x28`) snapshots keep every cell and cursor position inside the
declared buffer. The current `ratatui-cells.json` SHA-256 is
`AD02410DB1BF8951D54631FB92E52BBAB40CDAB2754872EC7B81F41FC8B34B7C`.
Dimensions below the current `8x5` minimum fail explicitly rather than
producing ambiguous partial layout.

### Slice 3: Tokimu Styled-Cell Lowering

**Objective:** Present a normalized Ratatui-style cell buffer through Tokimu's
existing text and surface contracts.

#### Deliverables

- [x] Define a corpus-local conversion from supported styled cells to
      provider-neutral cell rectangles, glyph text, and RGBA colors.
- [x] Emit deterministic `tokimu-layout.json` evidence from the same
      disposable Tosumu fixture and Ratatui snapshot.
- [x] Resolve Departure Mono through `UiFontSource::from_native_default()`.
- [x] Render foreground, background, cursor, selection, and empty cells.
- [x] Diagnose unsupported modifiers and missing glyphs explicitly.
- [x] Add cell-to-pixel bounds tests.

#### Acceptance Criteria

- [x] Every lowered cell and cursor stays inside its computed cell rectangle.
- [x] Baseline and caret placement are stable across each row.
- [x] Empty cells do not create accidental glyph or background geometry.
- [x] Font fallback is observable rather than silent.
- [x] Ratatui types do not cross into engine public APIs.

Current evidence: `tokimu-cell-layout-evidence` lowers the deterministic
`96x28` snapshot into 2,688 rectangles at a corpus-local `[10.0, 20.0]` cell
size. Departure Mono is parsed through the provider-neutral font source, its
ascent and descent determine one explicit baseline per row, and the visible
cursor lowers to a bounded two-pixel caret rather than occupying its complete
cell. The real fixture emits zero diagnostics. Synthetic tests prove missing
glyph and unsupported-modifier diagnostics, empty-cell draw suppression,
selection intent, invalid-grid rejection, and cell/caret/baseline bounds. The
current `tokimu-layout.json` SHA-256 is
`0F9C10E5F02690FA94CA1CA9622540252E8ECDEB95EFE2C313C2DCD7CCB33317`.
The same layout now produces a deterministic `960x560` CPU bitmap with FNV-1a
fingerprint `acc4f4c82805f402`. The raster adapter draws explicit foregrounds,
backgrounds, selection, and the bounded two-pixel caret; empty cells leave the
canvas unchanged unless their style explicitly requests background geometry.
Its manifest records source stage, font size, baseline, dimensions, format,
fingerprint algorithm, and `gpu_framebuffer_equivalent=false`. This proves the
style-cell, font-metric, and deterministic visual-evidence boundary without
misrepresenting a CPU artifact as native GPU capture.

The current native window intentionally renders a readable transcript model,
not a pixel-for-pixel terminal grid. Its fixed eight-line viewport clips older
content during live presentation while the `120x80` headless conformance view
retains the full reviewed fixture transcript. Ratatui remains responsible for
terminal wrapping, palette semantics, border behavior, styled empty cells, and
cell-cursor shape. The native provider instead owns pixel text layout, its
bounded viewport, and an underscore prompt caret. These are recorded
provider-only differences, not semantic divergence.

#### Current Cell-Style Boundary

The corpus adapter preserves a complete rectangular cell grid, per-cell text
symbols, foreground colors, background colors, bounded modifier names, one
visible cursor position, explicit row baselines, and corpus-owned selection
state. It intentionally supports only the colors emitted by the reviewed
fixture: `Reset`, `Black`, `Cyan`/`LightCyan`, and
`Green`/`LightGreen`. Unknown terminal colors resolve to a documented corpus
fallback rather than becoming a Tokimu palette contract.

Ratatui modifiers are retained as evidence but are not silently presented:
unsupported modifiers produce owner-labeled lowering diagnostics. ANSI escape
semantics, terminal palette indices, double-width cells, combining graphemes,
terminal selection rules, and terminal event-loop behavior remain outside the
adapter. Empty cells remain in the layout artifact to preserve grid and
background evidence while carrying explicit draw intent that suppresses
foreground glyph geometry for an empty symbol.

### Slice 4: Native Console Window

**Objective:** Build the interactive Tokimu corpus application.

#### Deliverables

- [x] Compose title/status, transcript viewport, prompt, caret, and diagnostics
      regions through UI tools.
- [x] Add click-to-focus, text insertion, space, punctuation, numbers,
      backspace/delete, Home/End, and submission.
- [x] Add bounded command history and Up/Down recall.
- [x] Add transcript scrolling, resize handling, and clear/reset actions.
- [x] Wrap transcript output against measured native-font width and keep wheel
      review directional: up reveals older rows and down returns toward the
      live tail.
- [x] Keep a visible label that identifies Departure Mono and the active
      projection mode.

Current evidence: the native corpus now presents a terminal-like command
fixture with top-anchored transcript output, a bounded eight-line live
viewport, transcript review state, and a separate editable prompt. Manual
native review confirms ordinary commands, lowercase text, punctuation,
scrolling, history, and prompt focus remain readable. It is intentionally not
yet treated as a DPI, accessibility, WASM, or pixel-cell parity claim.

`console-session-evidence` now opens a disposable real Tosumu fixture and
executes `STATUS`, `CHECK`, two `DESCRIBE` cases, `WAL STATUS`, and one invalid
`STATUS trailing` command through Tosumu's public JSON CLI boundary. The
retained `session.json` records five successful outcomes and one structured
`TQL_UNEXPECTED_TOKEN` provider failure, with no command being interpreted by
the native console itself. Together with the native interaction tests for
punctuation-bearing text entry and submission, this closes the complete
command-session evidence for the current local fixture.

The native transcript now wraps by rasterized pixel width rather than a fixed
character estimate. This makes long Tosumu diagnostics visible within the
bounded viewport while retaining the original transcript content as the
semantic artifact.

Native interaction state now lives in a corpus-local
`ConsoleInteractionState`. It owns only editable prompt text, focus, bounded
transcript history, transcript review offset, and command-history navigation.
Command interpretation remains with the consumer, Tosumu execution remains at
the provider boundary, and renderer invalidation remains with the native host.
Automated tests cover complete punctuation-bearing submission, history recall,
scroll clamping, focus independence, measured wrapping, and preservation of
oversized unbroken tokens.

#### Acceptance Criteria

- [x] A complete Tosumu command session can be entered without dropped input.
- [x] Focus, history, caret, scroll, and resize state remain consumer-owned.
- [x] Text never escapes the viewport or overlaps status/prompt regions.
- [ ] Repeated warm frames do not rebuild unchanged font or transcript assets.
- [x] The corpus remains usable with the Ratatui comparison disabled. Native
      targets compile without `ratatui-evidence`; comparison binaries require
      it explicitly.

### Slice 5: Projection Parity And Diagnostics

**Objective:** Compare semantics and layout evidence without requiring pixel
identity.

#### Deliverables

- [x] Compare normalized retained transcript content and cursor identity across
      headless, Ratatui, and Tokimu projections.
- [x] Record expected provider-only differences such as wrapping, color,
      empty-cell backgrounds, and cursor rendering.
- [x] Add deterministic headless layout and projection-conformance manifests at
      reviewed dimensions; native-window screenshots remain separately labeled
      manual evidence.
- [x] Add native warm-frame observations for draw count, submits, bindings,
      mesh uploads, and texture allocation/replacement/write churn. Initial
      font-load and resize timing remain manually observed provider evidence.
- [x] Assign deterministic native text-resource slots for the fixed viewport so
      changed transcript content replaces source textures instead of growing
      renderer resource maps over a long command session.

#### Acceptance Criteria

- [x] Semantic divergence fails the corpus.
- [x] Provider-only visual differences are classified rather than hidden.
- [x] Performance diagnostics identify the responsible producer boundary.
- [ ] No per-frame provider, glyph-atlas, mesh, or binding churn remains after
      warm-up for unchanged content.

Current performance instrumentation emits a structured warm-frame observation
only after 120 consecutive frames with unchanged text content. Input edits,
submitted commands, transcript review, and resize each reset that interval
before any later observation is considered warm. The report distinguishes
unchanged-frame churn from a deliberate text-slot replacement and records draw
and submit counts beside the relevant resource counters. The native window
does not claim GPU completion timing; renderer CPU timing remains separate
provider evidence.

The native host now emits `resource_churn=true|false` from one corpus-local
classification that rejects binding allocation, pipeline creation or
replacement, derived-material cache misses, mesh upload or replacement, and
texture allocation, replacement, or payload writes. Draw submission, material
resolution, pipeline selection, and the camera uniform write remain expected
per-frame work and are deliberately excluded. Unit tests pin that
classification and the reset-after-change behavior, while a retained live
native run remains required to close the criterion with provider evidence. The
retained run must wait for one report at `unchanged_frames=120`, exercise
prompt input, scroll, command submission, and resize, then retain a second
clean 120-frame report after each relevant reset.

Projection conformance is a failing assertion boundary rather than a report-only
comparison: retained transcript content, dimensions, cell identity, and cursor
identity must agree. Provider-only wrapping, palette lowering, empty-cell
backgrounds, and cursor drawing remain explicitly classified presentation
differences. The remaining warm-frame criterion stays open until a retained
native run demonstrates zero unchanged-content churn after startup and resize.

### Slice 6: Observation Shell And Tosumu Inspection-Island Readiness Review

**Objective:** Decide what evidence may safely feed the later Tosumu plan.

#### Deliverables

- [x] Record which Tosumu state remained semantic versus provider-local.
- [x] Classify session history, navigation, help, completion, and routing as
      shell-session behavior or owner-provided command behavior.
- [x] Record standalone Ratatui hosting and embedded Ratatui cell projection as
      distinct host modes with explicit focus, input, resize, and clipping
      owners.
- [x] Record whether the Ratatui buffer adapter is reusable or corpus-only.
- [x] Record Departure Mono readability, scaling, fallback, and accessibility
      findings in AR-0012.
- [x] Update the Tosumu inspection-island plan with proven prerequisites and
      rejected shortcuts.
- [x] Feed semantic-parity, host-ownership, and normalization evidence into
      AR-0013.
- [x] Decide whether to start that plan, refine this corpus, or park the
      provider experiment.

#### Acceptance Criteria

- [x] The Tosumu island does not inherit Ratatui widget state as semantics.
- [x] Any candidate shell session exposes no Ratatui, terminal, or Tosumu
      provider-native type.
- [x] The TypeScript provider is not required to emulate a terminal grid.
- [x] Official Ratatui provider admission remains blocked until an independent
      standalone host consumes the same session contract.
- [x] Any reusable adapter has at least two concrete consumers or remains in
      corpus incubation.
- [x] Departure Mono remains replaceable through ADR-0004 contracts.

#### Readiness Result

The Tosumu inspection-island plan may proceed with provider-neutral
observation and command extraction. It must not reuse Ratatui widget state,
the current CLI JSON envelope, or this corpus's `SessionEvidence` as its
semantic contract merely because those representations are available.

Official Ratatui provider admission and reusable Observation Shell extraction
are parked. The current normalized-cell and buffer adapters remain corpus-local
until a standalone Ratatui host independently consumes the same semantic
session and preserves command outcomes, retained history, and diagnostics.
This closes Slice 6 without closing the still-open live-session and warm-frame
resource-lifecycle evidence in Slices 4 and 5.

### Slice 7: Direct Ratatui Backend Into Tokimu Presentation

**Objective:** Replace the transitional test-backend snapshot chain with a
bounded Ratatui backend that delivers cell changes directly to Tokimu-owned
presentation.

#### Deliverables

- [x] Implement a corpus-local `TokimuBackend` against Ratatui's public
      `Backend` trait.
- [x] Retain a complete bounded cell grid while applying the changed cells
      received by `Backend::draw`.
- [x] Lower retained cells into Tokimu text and background presentation without
      serializing cell semantics through TypeScript. Cursor and interactive
      clipping ownership remain open.
- [ ] Keep `TestBackend` snapshots as deterministic reference tests only.
- [ ] Use `CompletedFrame.buffer` for optional full-frame diagnostics and
      parity artifacts, not runtime delivery.
- [ ] Route host resize, clear, cursor, focus, keyboard, and mouse ownership
      explicitly through the embedding host.
- [x] Migrate `tokimu-website-ratatui-lab` to display Tokimu-produced output;
      TypeScript may choose templates and host the output but may not position
      or style individual Ratatui cells.
- [ ] Record warm-frame changed-cell counts and Tokimu presentation churn.

#### Acceptance Criteria

- [x] The bounded Ratatui scene is visibly rendered by Tokimu rather than a
      TypeScript cell painter.
- [ ] One-cell changes do not require rebuilding or transferring the complete
      grid during runtime presentation.
- [ ] Resize and clear operations cannot leave stale cells outside the active
      bounded region.
- [ ] The direct backend and `TestBackend` reference agree on complete cell
      content at reviewed dimensions.
- [ ] Unsupported symbols, widths, colors, modifiers, and cursor states emit
      bounded owner-labeled diagnostics.
- [ ] No Ratatui type enters `tokimu-core`, `tokimu-runtime`, or a
      provider-neutral Observation Shell contract.

#### Progress Note -- 2026-08-05

The website lab now proves the retained backend and Tokimu-owned raster path.
It intentionally performs a complete bounded RGBA transfer after an explicit
template selection. Incremental uploads, `TestBackend` parity artifacts,
cursor/focus/input ownership, diagnostics normalization, and warm-frame churn
remain required before Slice 7 is complete.

#### Source Finding

Ratatui `v0.29.0` computes a diff between its previous and current buffers in
`Terminal::flush`, then invokes `Backend::draw` with only the changed cells.
`TestBackend` is one implementation of that contract, not a required bridge.
This makes a retained `TokimuBackend` the lowest stable source-backed adapter
for embedded presentation. The pinned review source lives at
`third-party/presentation-providers/ratatui`.

## Validation Matrix

| Boundary | Validation |
| --- | --- |
| Tosumu session | deterministic command/result and failure tests |
| Ratatui reference projection | `TestBackend` cell snapshots at fixed dimensions |
| Ratatui runtime projection | direct `TokimuBackend` changed-cell delivery into a retained bounded surface |
| Font provider | metrics, fallback, glyph coverage, and checksum evidence |
| Tokimu UI | bounds, clipping, focus, input, scroll, and resize tests |
| Renderer | screenshot plus warm-frame diagnostics |
| Composition | normalized semantic parity across all projections |

Expected commands include:

```text
cargo fmt --all
cargo test -p ui-tools
cargo test -p <console-corpus-package>
cargo test --manifest-path third-party/tosumu/Cargo.toml
cargo clippy --workspace --all-targets -- -D warnings
```

Exact package names should be recorded when the corpus is scaffolded.

## Failure Semantics

| Boundary | Example failures |
| --- | --- |
| Tosumu | parse, validation, storage, unsupported command |
| Session | closed, stale, history/output limit |
| Ratatui | unsupported style, invalid dimensions, projection truncation |
| Font | provider unavailable, glyph missing, invalid metrics |
| UI | no focus, clipped prompt, invalid scroll/caret position |
| Renderer | missing pipeline, capture failure, sustained budget violation |

Each failure must retain its owner. A missing glyph must not become a Tosumu
error, and a rejected TQL command must not be reported as a renderer failure.

## Risks

### A Console Becomes A Terminal Emulator

Restrict the first corpus to transcript, prompt, caret, selection, scroll, and
bounded styles. ANSI parsing, PTYs, process control, and terminal negotiation
remain explicitly deferred.

### Ratatui Becomes The Semantic Contract

Compare Ratatui against semantic observations. Never infer commands or Tosumu
facts by reading the resulting cell buffer.

### Tokimu Reimplements TQL

Expose or adapt Tosumu's existing parser/dispatcher. Do not create a second
grammar for convenience.

### Monospace Assumptions Leak Into Text Semantics

Cell-grid lowering is a console adapter concern. Provider-neutral text remains
capable of proportional layout.

### The Corpus Preempts The Tosumu Island Review

Keep inspection facts and the full Ratatui inspector out of scope. This plan
only proves command-window and provider composition mechanics.

## Open Questions

- Should the first Tokimu view consume semantic spans directly, a normalized
  cell grid, or expose both as explicit modes?
- Does Tosumu need a small public TQL session crate, or is a provisional corpus
  adapter sufficient?
- Should wrapping be semantic, Ratatui-provider-owned, or Tokimu-view-owned?
- Is selection text-only, cell-based, or deferred?
- Which Ratatui modifiers map honestly onto current Tokimu text presentation?
- Should the cursor be a text semantic, console semantic, or UI focus artifact?
- When does a cell-buffer adapter have enough independent use to leave the
  corpus?

## Completion Criteria

This prerequisite is complete when:

- Departure Mono renders a readable and stable native command session;
- deterministic Tosumu commands execute through Tosumu-owned semantics;
- Ratatui and Tokimu projections agree on retained semantic content;
- input, caret, history, scroll, clipping, resize, and failure behavior are
  covered by tests;
- artifacts localize divergence across session, cell, text-layout, and render
  boundaries;
- no engine crate owns Tosumu, Ratatui, terminal, or font-provider internals;
- the Tosumu inspection-island plan is updated with the resulting evidence.

## References

- [`On Ratatui.md`](../../Conversations/On%20Ratatui.md)
- [`Observation Shell.md`](../../Conversations/Observation%20Shell.md)
- [`tokimu-observation-shell-consumer-corpus.md`](tokimu-observation-shell-consumer-corpus.md)
- [`runtime-observation-and-command-corpus.md`](runtime-observation-and-command-corpus.md)
- [`ui-tools-consumer-safety-and-hardening.md`](ui-tools-consumer-safety-and-hardening.md)
- [`ADR-0004-foundational-presentation-text-and-icons.md`](../../ADR/ADR-0004-foundational-presentation-text-and-icons.md)
- [`ADR-0005-admission-evidence-and-maintainer-exceptions.md`](../../ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md)
- [`AR-0012-bundled-native-default-font-provider.md`](../../Architectural%20Reviews/AR-0012-bundled-native-default-font-provider.md)
- [`AR-0013-observation-shell-and-ratatui-presentation-provider.md`](../../Architectural%20Reviews/AR-0013-observation-shell-and-ratatui-presentation-provider.md)
- [`tosumu-inspection-island-and-ui-providers.md`](../../../third-party/tosumu/docs/Plans/tosumu-inspection-island-and-ui-providers.md)
- [`Tosumu Command Language.md`](../../../third-party/tosumu/docs/Tosumu%20Command%20Language.md)
