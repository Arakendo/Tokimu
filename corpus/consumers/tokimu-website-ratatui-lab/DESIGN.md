# Tokimu Rendered Ratatui Lab

## Purpose

This consumer proves that a browser can display deterministic Ratatui layouts
composed by Rust/WASM and rasterized by Tokimu without JavaScript recreating
terminal widgets or interpreting terminal cells.

## Boundary

```text
template selection (browser)
        ↓
Rust/WASM Ratatui widgets
        ↓
TokimuBackend::draw(cell diffs)
        ↓
retained bounded Tokimu cell surface
        ↓
Tokimu font/raster presentation
        ↓
RGBA frame
        ↓
browser host blits Tokimu output
```

Ratatui's public `Backend::draw` contract is the lowest stable capture point
after widget layout and buffer diffing. A `TokimuBackend` must retain the full
bounded grid because Ratatui supplies only changed cells during a draw. The
`CompletedFrame` buffer remains useful for deterministic full-frame tests and
diagnostic artifacts, while `TestBackend` remains a reference test provider.

TypeScript may select templates, size the host region, and blit Tokimu output.
It must not interpret Ratatui styles, position glyphs, or become the owner of
terminal-cell presentation semantics. Incremental dirty-region delivery and
interactive host input remain later evidence; this static laboratory currently
transfers one complete bounded Tokimu frame after each explicit selection.

The templates contain dummy observations. They do not inspect host telemetry,
assets, files, or a native terminal.

## Templates

- System monitor: synthetic runtime measurements.
- Asset inspector: a provider-neutral GLB observation fixture.
- Command transcript: deterministic observation-shell history.

## Non-goals

- terminal emulation;
- keyboard command execution;
- making Ratatui a required Tokimu runtime dependency;
- admitting a universal terminal presentation capability.

## Source Pin

The reviewed Ratatui source is pinned at `v0.29.0` under
`third-party/presentation-providers/ratatui`. The source checkout is review and
corpus evidence; it does not make Ratatui an engine-core dependency.
