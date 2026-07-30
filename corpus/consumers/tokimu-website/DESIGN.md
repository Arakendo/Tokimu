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
cold startup duration with each observation and performs event-driven drawing
only. It does not own a permanent animation loop.

## Known Limits

- The first proof admits only SVG observation and vector preview.
- The preview is diagnostic Canvas execution, not backend framebuffer
  equivalence.
- Browser compatibility and repeated reset memory behavior still need recorded
  evidence.
- The site remains a first-party consumer corpus, not independent admission
  evidence.
