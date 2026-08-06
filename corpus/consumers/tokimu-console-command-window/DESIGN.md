# Tokimu Console Command Window

This corpus application tests a native graphical command-window presentation
using Tokimu's provisionally admitted Departure Mono font provider.

It intentionally owns only a bounded local command transcript. It does not
parse TQL, open a host shell, spawn a PTY, or inspect Tosumu storage. Submitted
TQL text is forwarded to Tosumu's public schema-versioned CLI JSON contract,
then projected into this presentation model.

## Current Evidence

- Native window input feeds the shared `UiTextInputState` editor.
- Transcript lines and the editable prompt are rasterized through
  `UiFontSource::from_native_default()`.
- `help` and `clear` are console-local corpus commands, not TQL.
- `STATUS`, `CHECK`, `DESCRIBE <key>`, and `WAL STATUS` run through Tosumu's
  existing CLI boundary; no Tokimu code parses or dispatches TQL.
- The prompt remains explicitly focused and reports its bounded state.
- The optional Ratatui comparison renders the same retained session through a
  headless `TestBackend`, retains every cell in the reviewed terminal grid, and
  lowers that grid into a corpus-local Tokimu cell-layout artifact. Departure
  Mono metrics supply explicit row baselines and glyph coverage checks without
  exposing parser or rasterizer objects in the artifact.
- The lowered cell layout also produces a deterministic CPU bitmap and
  manifest. This artifact proves foreground, explicit background, selection,
  caret, clipping, and empty-cell behavior while remaining explicitly distinct
  from a GPU framebuffer capture.

## Ratatui Cell Boundary

The headless Ratatui projection is evidence, not a Tokimu terminal API. The
current adapter preserves a complete grid, text symbols, foreground and
background colors, modifier names, explicit draw intent, row baselines,
corpus-owned selection state, and one bounded caret. It only maps the fixture's
`Reset`, `Black`, `Cyan`/`LightCyan`, and `Green`/`LightGreen` colors.

The comparison adapter is compiled only through the corpus-local
`ratatui-evidence` feature. The native console and direct Tosumu session
evidence compile without Ratatui, so the package boundary makes the provider
optional rather than merely treating it as optional by convention.

Unsupported Ratatui modifiers and missing provider glyphs produce explicit
lowering diagnostics rather than silent fallback. ANSI escapes, palette
indices, double-width cells, combining graphemes, terminal selection rules,
and terminal event-loop behavior are not preserved. A malformed grid or
invalid baseline/caret policy is rejected before lowering; it is never treated
as a valid partial layout. Empty symbols remain observable cells but do not
request foreground glyph geometry.

The corpus-local CPU raster adapter consumes only the provider-neutral layout
and a font rasterizer. It does not inspect Ratatui cells, Tosumu observations,
or renderer resources. Its bitmap fingerprint is deterministic evidence for
the layout-to-pixel boundary, not a claim that native and headless providers
must be pixel-identical.

## Native Projection Lifecycle

The native projection owns a fixed, bounded text viewport: title, metadata,
eight visible transcript rows, scroll status, prompt, and prompt help. Each
region maps to a deterministic renderer texture and material slot. A command,
scroll event, resize, or prompt edit may replace the affected source texture,
but it must not allocate an unbounded sequence of renderer resources as the
transcript evolves.

Transcript content wraps against the resolved font's measured pixel width, not
an estimated character count. Scroll state indexes those visible wrapped rows:
wheel up reviews older output and wheel down returns toward the live tail.
This keeps long TQL diagnostics inside the transcript viewport without
changing the semantic command result.

The corpus emits a warm-frame observation only after 120 consecutive frames
with unchanged text content. Prompt edits, submitted commands, transcript
review, and resize reset the interval, so a deliberate text-slot replacement
cannot be mislabeled as warm-frame churn. Each report includes draw and submit
counts together with texture, binding, and mesh churn so a long-running console
session can distinguish intentional replacement from unchanged-frame resource
growth. This is renderer lifecycle evidence; it does not make renderer texture
handles part of terminal semantics.

The native window is intentionally a line-based projection rather than a
pixel-identical terminal emulator. Ratatui remains the reference provider for
cell wrapping, style modifiers, styled empty cells, and terminal cursor rules.
The native projection preserves the reviewed transcript content, focus,
history, scroll state, and editable prompt through Tokimu UI contracts.

## Readiness Findings

The corpus now supports a conservative ownership decision without promoting a
shell capability or Ratatui provider.

| Concern | Current owner | Finding |
| --- | --- | --- |
| TQL grammar and execution | Tosumu | Command text remains opaque to Tokimu; Tosumu owns outcomes and storage diagnostics. |
| Retained command evidence | Corpus adapter | Ordered input, output lines, outcomes, and envelopes are useful parity evidence, but `SessionEvidence` is still Tosumu-adapter-shaped rather than a universal shell contract. |
| Prompt editing and focus | Native host | The shared text editor supplies editing mechanics; the host owns focus and input arbitration. |
| History recall | Native host | Proven as bounded interaction behavior, but not yet repeated enough to promote as shell semantics. |
| Transcript scrolling and wrapping | Native host | Viewport navigation, measured wrapping, clipping, and resize remain projection mechanics. |
| Help | Command owner | The current `help` command is an explicitly local corpus fixture; a reusable shell must not invent owner vocabulary. |
| Completion | Split responsibility | A command owner may provide candidates and meaning; a host owns completion interaction. No completion contract is admitted here. |
| Command routing | Candidate session boundary | A future session may route opaque requests, but each command owner remains responsible for interpretation and results. |

Standalone and embedded Ratatui hosting are distinct modes. A standalone host
would own terminal lifecycle, event polling, resize, cursor, and terminal input.
An embedded Tokimu host owns bounded regions, clipping, focus, input
arbitration, and renderer resources while consuming only normalized cells.
Ratatui may produce those cells, but the cells do not become application or
Tosumu meaning.

The Ratatui buffer adapter remains corpus-local. It has one composed consumer,
not two independent hosts, so extracting it would freeze assumptions before
the provider boundary is proven. TypeScript and other structured consumers may
present retained semantic observations directly and are not required to
emulate a terminal grid.

The resulting decision is to proceed with Tosumu's provider-neutral
observation and command extraction, while parking official Ratatui admission
and any reusable Observation Shell extraction. Departure Mono remains a
replaceable provider under ADR-0004; this corpus supplies reviewed native
readability and deterministic raster evidence, not broad-script,
accessibility, DPI, WASM-distribution, or permanent-default approval.

## Controls

- Click the prompt field to focus it.
- Type text, then press `Enter` to append it to the transcript.
- `help` and `clear` exercise the local fixture.
- `STATUS`, `CHECK`, `DESCRIBE demo/message`, and `WAL STATUS` exercise the
  disposable Tosumu fixture when `tosumu-cli` is available.
- `Esc` clears the current prompt.
- Mouse wheel reviews retained transcript output; a submitted command returns to
  the live end of the transcript.
- `Up` and `Down` recall submitted local commands without changing transcript
  scroll position.

## Boundary

Tosumu owns TQL and storage semantics. Ratatui owns terminal-cell layout.
This application provides a native graphical projection over the real TQL CLI
boundary; the later Ratatui comparison must target the same session evidence.
