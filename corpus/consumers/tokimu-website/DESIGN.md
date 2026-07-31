# Tokimu Website Consumer Corpus

## Primary Composition Claim

Can a curated static documentation site load Tokimu through an explicitly
bounded WASM island without duplicating importer semantics or making the
surrounding knowledge depend on JavaScript, WASM, Canvas, or a live renderer?

The first proof follows this path:

```text
repository-owned SVG fixture or local browser bytes
                            |
                            v
                 Rust/WASM inspect_asset
                            |
                            v
              provider-neutral observation
                            |
                            v
              TypeScript and Canvas evidence
```

At no point does TypeScript parse SVG, infer Tokimu semantics from source
markup, or replace an unavailable engine path with browser-native SVG
rendering.

## Ownership

- MkDocs owns documents, routing, metadata, and static fallback content.
- TypeScript owns activation, local file selection, bounded browser lifecycle,
  accessible reporting, and Canvas drawing.
- Rust/WASM owns importer invocation and provider-neutral observations.
- Tokimu and incubating corpus libraries own engine and importer semantics.
- Canvas owns pixels, not meaning.

## Dependencies

This is a Tier 2 incubating consumer. Its first island reuses:

- `corpus/consumers/aspnet-wasm-asset-workbench/engine`;
- `corpus/lib/ui-tools` through that engine;
- the selected W3C SVG geometry fixture; and
- the website's declarative island lifecycle contract.

Generated WASM and binding files are committed under the website assets so a
static documentation build does not require the Rust toolchain.

## Lifecycle

The island starts idle and loads only after explicit visitor activation. It
then imports the generated binding, initializes WASM, loads the known fixture,
and asks Tokimu for an observation. Reset or page teardown releases listeners,
clears retained local-file state, removes mounted output, and restores the
static fallback.

## Security And Privacy

- Known fixture bytes are served from the same static site.
- User-selected files remain local to the browser tab.
- Inputs are rejected before crossing WASM when they exceed the configured
  byte limit.
- The first island accepts SVG files only.
- Diagnostics are bounded in the browser presentation.
- Imported SVG is never inserted into the DOM or executed as active content.

## Accessibility

The static explanation remains useful without scripting. Activation, reset,
fixture reload, and local file selection are keyboard accessible. The canvas
has a label, but the observation summary, properties, verdict, and diagnostics
remain available as text.

## Performance Budget Evidence

Initial timings are observations rather than guarantees. The island reports
WASM startup, importer inspection, first useful textual evidence, and Canvas
presentation separately. A hidden or offscreen canvas reports deferred
presentation rather than inventing a timing for work that did not run.

The first published payload has executable size budgets:

- WASM module: at most 1,280 KiB across supported build hosts;
- generated WASM binding: at most 24 KiB;
- asset-observation adapter: at most 24 KiB;
- shared island lifecycle: at most 12 KiB; and
- complete first-island payload, including its fixture: at most 1,330 KiB.

The original 2026-07-30 source-tree measurement was 887,389 bytes across those
five published files. On 2026-07-31, the expanded importer set measured
1,080,689 bytes for the committed Windows-built WASM module and 1,230,522 bytes
for the Linux deployment build. The automated check measures generated files
directly and applies an explicit cross-host ceiling so growth cannot hide behind
a stale handwritten total or fail merely because equivalent toolchains produce
different binary layouts.

The lifecycle corpus runs 32 activation/reset cycles and requires one release
for every mount while retaining only the controller's single delegated click
handler. These are first-release evidence bounds, not universal engine
guarantees.

Presentation remains event-driven only. The island does not own a permanent
animation loop and suppresses Canvas redraws while hidden or offscreen.

## Known Limits

- The first proof admits only SVG observation and vector preview.
- The preview is diagnostic Canvas execution, not backend framebuffer
  equivalence.
- Browser compatibility and heap-level memory behavior still need recorded
  evidence in supported browsers.
- The site remains a first-party consumer corpus, not independent admission
  evidence.

## Public Launch Review Evidence

The 2026-07-30 source and generated-site review found and corrected stale
language that still described the deployed website, WASM island, and
interactive evidence as pending. Public maturity labels continue to distinguish
experimental browser execution from general engine or format support.

The deploy artifact is validated after strict MkDocs generation:

- every generated page carries an `.org` canonical URL and non-empty
  description;
- internal links and linked assets resolve inside the generated site;
- `CNAME` and `.nojekyll` are present;
- the homepage retains useful architectural and evidence context without
  JavaScript; and
- the website remains a static knowledge owner with an optional Tokimu
  consumer, preserving the SDD, ADR, and Architectural Review ownership
  direction.

Open manual launch evidence remains:

- verification of the configured `.com` and `.net` DNS forwarding after edge
  propagation. On 2026-07-30, `.com` still returned a Squarespace `302` to
  `http://tokimuengine.org` without preserving the requested path; `.net`
  still served a Squarespace root page and redirected a non-root path to the
  same non-path-preserving HTTP destination;
- supported-browser keyboard, pointer, resize, failure, and reset review;
- browser heap-retention observation across repeated activation cycles; and
- native-window screenshots, where useful, labeled separately from structural
  and deterministic evidence.

Public evidence maintenance begins with the authoritative repository record.
Its website representation and drift test change together. The generated-site
validator runs after every deployment build so routing and metadata regressions
cannot hide behind a successful Markdown build.
