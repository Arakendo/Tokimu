# Hello UI Glyph Corpus

## Purpose

`hello-ui-glyph-corpus` is a data-corpus test for Tokimu text and glyph
presentation. It is intentionally separate from the `hello-ui-*` semantic
examples: those prove UI concepts, while this example supplies varied,
independent content to expose renderer and layout weaknesses.

## Pages

- `FONT SAMPLES`: representative prepared TTF samples rasterized through
  `ui-tools::UiFontRasterizer`.
- `ICON BATCH`: batches of Lucide-style symbol names and identifier shapes.
- `UNICODE`: ASCII, punctuation, whitespace, digits, and multilingual samples.

The font page uses the shared rasterizer and baseline-aware metrics. The icon
and Unicode pages continue to use the bitmap text path for labels and page
navigation. Prepared assets live in the generated `target/glyph-corpus`
directory and are intentionally not copied into the repository.

## Controls

- Left/Right: change corpus page.
- Up/Down: move through the current page's rows.
- Home/End: jump to the first/last row.

## Success Criteria

- Every page remains readable with the shared text layout path.
- Digits, spaces, punctuation, and unsupported glyphs are visibly distinct.
- Long rows can be inspected without changing the rendering code.
- Page and row state are deterministic and independent from glyph geometry.

## Non-Goals

- Font selection policy or rasterizer implementation; those belong to
  `ui-tools`.
- SVG parsing or icon triangulation.
- A general asset browser.
- Full text shaping, bidi, or IME support.
