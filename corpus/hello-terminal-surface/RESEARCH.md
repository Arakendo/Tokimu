# Hello Terminal Surface

## Research Abstract

This corpus studies a narrow handoff between a terminal provider and Tokimu
presentation. It does not implement a terminal, shell, or Ratatui substitute.
It only tests whether a provider can send a complete resolved cell surface,
followed by bounded changes, to a consumer that preserves the provider's
layout decisions.

The initial model is intentionally headless. It proves the lifecycle before a
window, font, or renderer can hide an invalid update behind a plausible image.

## Claim Under Test

```text
full surface -> compatible changed cells -> deterministic reconstructed surface
```

An epoch or extent change invalidates the old baseline. A continuation cell is
layout metadata, not a separately rendered glyph.

## Evidence Scope

- Full-frame baseline is required before deltas.
- Epoch and extent mismatches are explicit errors.
- A wide grapheme's continuation cell cannot be independently overwritten.
- The optional Ratatui adapter preserves provider-resolved foreground,
  background, and supported emphasis modifiers without exposing Ratatui types.
- The independent fixture uses default cell styling only; it does not yet claim
  stylistic parity with Ratatui.

## CPU Presentation Evidence

The corpus now lowers both producers through the existing
`ui-tools::UiFontRasterizer` and the embedded Departure Mono provider. The CPU
adapter accepts only the resolved extent and retained cells; it does not infer
terminal width, wrapping, or continuation placement.

The current deterministic artifacts are bounded `288x120` RGBA surfaces:

- independent fixture: `851900b19a379bcd`;
- Ratatui `TestBackend` fixture: `36a3ae80a8dcbce9`.

The accompanying tests cover edge clipping, foreground and background color,
style-only deltas, and the real Ratatui producer path. This is a CPU reference
artifact, not a claim of native renderer, GPU, or WASM equivalence.

The package also builds as a `wasm32-unknown-unknown` `cdylib` with a local
browser presenter. Its opaque exports accept an explicit fixture producer and
provide dimensions, RGBA bytes, and a diagnostic summary only; they do not
expose Ratatui, cells, or font-provider types across the boundary. A Ratatui
request on a build without the optional feature is an explicit error, never an
independent-fixture fallback. The browser owns only the `ImageData` blit. This
is browser display evidence, not native/GPU renderer or cross-host image-parity
evidence.

Cursor position and italic emphasis remain preserved in the corpus model but
are not yet painted by the CPU adapter. That limitation is intentional evidence
of unfinished presentation behavior, not permission for a downstream host to
reinterpret terminal layout.

## Native Renderer Evidence

The native binary also submits this exact CPU raster once as an sRGB texture on
a centered `Texture2d` quad. Its startup observation records the same CPU
fingerprint, raster dimensions, and one texture upload while explicitly
reporting `framebuffer_readback=false`. This proves a narrow native execution
handoff only; it does not compare GPU pixels with the CPU artifact or make a
native/WASM parity claim.

## Browser Export Boundary Measurement

The browser facade selects `independent` or `ratatui` explicitly, requests the
complete RGBA surface from Rust/WASM, and presents it with Canvas 2D
`putImageData`. It does not recreate terminal layout, cells, or styles.

On the 2026-08-06 development machine, release `wasm32-unknown-unknown`
artifacts measured `319158` bytes without `ratatui-producer` and `454152`
bytes with that feature enabled: a `134994` byte local payload delta. These
are deployment observations, not a provider-admission threshold or
cross-consumer budget.

## Native Producer Link Measurement

Release corpus executables measured `6191104` bytes for the independent
configuration and `6298624` bytes with `ratatui-producer` enabled. The
`107520` byte increase is executable dependency-size evidence because this
native binary links the optional producer. It is not a production performance
budget or a terminal-provider admission result: startup, frame behavior, and
an independent consumer are still unmeasured.

## Headless CPU Pipeline Measurement

`cargo run -p hello-terminal-surface --features ratatui-producer -- --measure`
repeats each producer's existing bounded `READY` to `DONE` lifecycle 256 times:
full frame, changed-cell delta, replica reconstruction, and the same Departure
Mono CPU rasterizer. It rejects unstable dimensions or fingerprints and prints
elapsed and average times as local observations. This is not a CI performance
budget, native startup measurement, GPU-frame measurement, or terminal-host
benchmark.

On the 2026-08-06 development machine, a 256-repetition run measured
`129301 us` total (`505 us` average) for the independent producer and
`232715 us` total (`909 us` average) for the optional Ratatui producer. For
each producer, the retained CPU-frame path performed one Departure Mono
provider load and one CPU rasterization, followed by 255 complete-frame cache
hits. These values are local CPU observations, not cross-machine budgets or a
provider-admission threshold.

The retained cache is intentionally corpus-local. Equality of complete
resolved terminal surfaces reuses the existing CPU raster; a changed resolved
surface records one complete CPU invalidation before rerasterizing. This is not
evidence for partial texture uploads, GPU cache lifetime, or a general
`tui-tools` cache policy.

## Browser Input Boundary

The `runtime-observation-workbench` browser consumer independently translates
DOM keyboard and wheel observations into semantic Rust/WASM Ratatui-session
actions through a DOM-free, consumer-local mapper. `npm run test:input`
validates the mapping without a browser. Canvas focus, pointer delivery, and
pixel presentation remain browser responsibilities. This supports the intended
boundary: `tui-tools` receives semantic actions rather than DOM event types;
native-host adapter evidence remains open.

## Independent Consumer Evidence

`hello-terminal-surface` exercises a terminal-shaped transcript and prompt,
while `hello-tui-tools` exercises a non-shell status dashboard. Both compose
caller-owned facts into `tui-tools::Surface` values and use the same bounded
CPU raster seam. The dashboard deliberately has no prompt, transcript,
command-history, or viewport behavior, so the shared path does not require a
consumer to inherit shell semantics.

The website Ratatui corpus is a third, provider-backed consumer. Its optional
`tui-tools::ratatui-bridge` adapter converts an already-composed Ratatui
`Buffer` into normalized cells and sends those cells through the same CPU
raster path. Ratatui still owns widget layout, terminal composition, and its
style vocabulary. The website retains its font selection and Canvas blit.

This removed duplicate Ratatui-buffer style mapping and CPU-frame allocation
from the website consumer and the Ratatui oracle. It does not admit Ratatui
types into the base `tui-tools` contract, make fonts provider-neutral terminal
semantics, or claim a shared shell session. Those seams remain intentionally
provider- or consumer-owned evidence.

## Continuation Evidence

The independent producer resolves `A界B` into an explicit wide-grapheme lead
cell and a trailing continuation cell. The shared `tui-tools` raster input
preserves that continuation as layout metadata: it paints any cell background
but emits neither glyph ink nor underline/crossed-out decoration for the
trailing cell. This proves the lower raster handoff can execute an already
resolved continuation without calculating Unicode width itself.

Ratatui 0.29 clears the public `Buffer` cells after a multi-width grapheme to
ordinary blanks and exposes no public trailing-cell marker. The optional
Ratatui adapter therefore does not infer continuations from blank cells or
reinterpret Ratatui's graphics-diff `skip` bit as text layout. Provider parity
for wide and combining text remains an open comparison task.

## Native Execution Instrumentation

The native corpus viewer uploads the already-resolved CPU raster as one RGBA8
texture and renders it through Tokimu's normal textured-quad path. It emits a
startup-ready observation and periodic warm-frame observations containing
CPU-side surface-acquire, resource-preparation, command-encoding,
queue-submit, and surface-present call durations plus renderer resource-churn
counters.

This isolates the execution boundary without teaching the renderer about
Ratatui, cells, terminal layouts, or fonts. It does not measure GPU completion,
display latency, framebuffer readback, terminal-host behavior, partial texture
updates, or session-lifetime GPU cache policy. The viewer instrumentation is
the bounded native warm-frame evidence for this study; any future manual run
must be retained as a local observation rather than promoted to a universal
budget.

## Not A Contract

The types in `src/main.rs` are corpus-local research vocabulary. They may be
replaced, split, or discarded after comparison with Ratatui and an independent
producer. See `docs/Plans/terminal-surface-provider-study.md` and AR-0014.
