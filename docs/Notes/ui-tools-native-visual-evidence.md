# UI Tools Native Visual Evidence

## Evidence Class

This note records manual native-window observations. It is complementary
backend evidence, not a deterministic golden artifact and not a substitute for
the headless structural UI corpus.

## 2026-08-01 Review

### Runtime Observation Inspector

- The header, two primary body panes, and footer remain inside the native
  viewport.
- World-observation and presentation/playback content no longer overlap.
- The commands heading has distinct vertical space from its instruction rows.
- Footer diagnostics remain in their allocated column.
- Resizing is protected structurally by the desktop and constrained viewport
  tests; this manual review confirms that the native bitmap presentation agrees
  with those bounds at the reviewed desktop size.

### CGM Inspection

- The CGM source and vector panes remain bounded by the shared frame and split
  layout.
- Domain vector content is anchored to the resolved vector pane rather than the
  window.
- Diagnostic content remains inside its allocated region; verbose source
  diagnostics use bounded presentation instead of expanding the canvas.
- Desktop, 4:3, and square structural tests are authoritative for geometry.
  Native review only confirms readability at the captured desktop size.

## Limitations

- These observations do not prove GPU framebuffer equivalence.
- They do not establish typography or timing guarantees across machines.
- A future automated native capture capability must identify its backend,
  viewport, scale, color assumptions, and capture stage explicitly.
