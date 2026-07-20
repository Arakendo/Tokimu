# Hello UI Glyph Corpus

## Purpose

`hello-ui-glyph-corpus` is a data-corpus test for Tokimu text and glyph
presentation. It is intentionally separate from the `hello-ui-*` semantic
examples: those prove UI concepts, while this example supplies varied,
independent content to expose renderer and layout weaknesses.

## Pages

- `FONT SAMPLES`: representative UI, code, and document font labels.
- `ICON BATCH`: batches of Lucide-style symbol names and identifier shapes.
- `UNICODE`: ASCII, punctuation, whitespace, digits, and multilingual samples.

The first implementation uses the current bitmap text path and page/scroll
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

- Font rasterization or font selection implementation.
- SVG parsing or icon triangulation.
- A general asset browser.
- Full text shaping, bidi, or IME support.
