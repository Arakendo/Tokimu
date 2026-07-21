# Text Corpus v1 Validation

The corpus examples are visual regression probes, not pixel-identical
cross-provider tests. Builtin text, TTF, and OTF providers are expected to
preserve semantic content and layout contracts while differing in glyph shape,
advance widths, and visible ink.

## Capture Set

Capture the following after fixture identities and layout are stable:

- `hello-ui-text`: builtin semantic text, alignment, clipping, and roles;
- `hello-ui-font`: glyph grid through the TTF and OTF raster paths;
- `hello-ui-font2`: the three shared natural-text samples through Inter,
  JetBrains Mono, and Noto fixtures;
- `hello-ui-glyph-corpus`: font, Unicode, icon, and metric-torture pages.

## Comparison Rules

- Compare semantic content, baseline behavior, clipping, and alignment first.
- Do not require identical pixels across different font providers.
- A glyph-shape change is expected when the fixture, provider, or format changes.
- A missing glyph, baseline shift, unexpected advance change, or layout drift is
  a regression unless intentionally recorded.
- Record the window title, corpus version, provider identity, fixture format, and
  source checksum with each reference capture.

## Change Classification

Every visual difference should be classified as one of:

1. Intentional layout or renderer improvement.
2. Intentional provider, fixture, or version change.
3. Regression requiring investigation.

Screenshots should be captured manually on the supported development platform;
the examples remain the executable regression specification, while this note
defines how their evidence is interpreted.
