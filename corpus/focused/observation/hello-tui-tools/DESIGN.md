# Hello TUI Tools

## Purpose

This executable is the first independent consumer of the corpus-local
`tui-tools` study. It asks whether a non-shell application can project its own
status data into a deterministic bounded terminal surface without linking
Ratatui or acquiring terminal-host authority.

## Ownership

- The consumer owns runtime, presentation, asset, and diagnostic meaning.
- `tui-tools` owns rectangles, directional layout, cell projection, clipping,
  style roles, and layout diagnostics.
- A future terminal-surface provider may rasterize the cells.
- Ratatui remains an oracle and optional provider, not this consumer's contract.

## Evidence

The executable emits a normal 72 by 24 board and an intentionally undersized
24 by 6 projection. The latter must stay bounded and report why information
was clipped or constraints could not be satisfied.

This is corpus evidence, not a stable Tokimu public API.
