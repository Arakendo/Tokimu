# Hello UI Text Vectors

## Purpose

`hello-ui-text-vectors` is a focused corpus example for font outlines lowered
into provider-neutral vector geometry. It isolates glyph topology and
tessellation from the larger raster/vector comparison in `hello-ui-font2`.

## Corpus

The initial proof uses one prepared Inter TTF fixture and renders glyphs that
exercise distinct geometry:

- `A`, `O`, `8`: counters and nested contours;
- `a`, `e`: compact curved forms;
- `g`: descender and curves;
- `@`: nested contours and curve density.

Every glyph uses the same requested font size, baseline origin policy, output
scale, fill rule, and solid-color vector pipeline.

## Success Criteria

- glyphs remain inside their diagnostic cells;
- glyphs share a visible baseline within each row;
- counters remain open;
- curves do not show missing wedges or spikes;
- startup diagnostics report triangle counts and any outline failures;
- the example contains no font-specific geometry correction.

## Non-Goals

This is not a font browser or a raster parity proof. It does not establish
shaping, fallback, kerning, atlas policy, or final vector-text admission.
