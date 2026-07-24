# Hello UI Font 2

## Purpose

`hello-ui-font2` is a bounded corpus consumer for comparing rasterized font
text with glyph outlines lowered through the incubating vector presentation
path.

The example uses the same providers, strings, requested font size, layout
advances, and baseline policy for both execution strategies. The raster column
is the existing reference path; the vector column is evidence for the optional
outline presentation path.

## Current Proof

The example compares:

- Inter TTF;
- JetBrains Mono OTF;
- Noto TTF;
- prose, digits, punctuation, ascenders, descenders, and repeated glyphs.

The vector side additionally reports startup geometry cost, generated triangle
count, glyph instance count, and cached local mesh count.

Startup diagnostics also report the viewport, shared pixel scale, and the two
column anchors. These values are part of the visual evidence record; they make
an apparent layout change distinguishable from a font/provider change.

Each provider also reports its resolved fixture path, format, and byte length.
This identifies the actual prepared asset used by the comparison rather than
relying on the provider label alone.

The comparison manifest also records the expected fixture set and an optional
`TOKIMU_CORPUS_REVISION` value. If the runner does not provide a revision, the
manifest records `unknown` rather than inventing provenance.

Each run also writes `target/hello-ui-font2/comparison.txt` with the layout
configuration and execution strategies. This is metadata only; it does not
claim to be a GPU screenshot or framebuffer capture.

The headless measurement pass also samples narrow and wide glyphs, digits,
punctuation, and `AV` pairs at 24, 56, and 96 pixels. The first two rendered
frames report draw calls, mesh uploads, and replacements so setup and retained
submission can be compared.

The runtime parity report compares advance-based layout width with raster
visible-ink width. The current Inter, JetBrains Mono, and Noto samples agree
within approximately one pixel, which isolates remaining differences to
presentation quality rather than line placement.

Both columns use the same orthographic presentation convention. Glyph sizes
use the viewport-height-derived pixel scale, while horizontal placement uses
that same scale so the camera aspect ratio is not reintroduced as an
independent width correction. The example must not add a raster-only horizontal
squash or compensate for a provider by changing its placement.

The raster and vector columns use explicit, separated anchors. This keeps wide
provider samples readable during comparison without changing their measured
advances or applying provider-specific geometry corrections.

## Ownership

```text
UiFontRasterizer::layout
        |
        +--> raster text textures
        |
        `--> glyph outline -> VectorPath -> fill triangles -> Mesh
```

The example owns renderer resource upload and draw submission. `ui-tools` owns
provider-neutral outline conversion and renderer-neutral tessellation. Neither
path derives placement from individual outline bounds.

## Non-Goals

This is not a production font viewer and does not establish that vector text
should replace raster text. Shaping, kerning, fallback, color fonts, atlas
policy, and visual parity at small sizes remain future evidence.
Native curve segments are preserved at the font-outline boundary, but
flattening remains the current execution strategy until SVG and font consumers
demonstrate a shared curve contract.

## Run

Prepare the font corpus, then run:

```powershell
cargo run -p hello-ui-font2
```

Startup output records the current geometry measurements. Visual comparison is
still required before this corpus slice can be considered complete.

The size report also records native line, quadratic, and cubic segment counts
before flattening. This keeps the native-curve review grounded in the actual
provider corpus and makes curve representation pressure visible across sizes.
