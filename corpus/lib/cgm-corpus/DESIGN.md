# CGM Corpus

`cgm-corpus` is an example-side inspection and importer incubator for selected
WebCGM fixtures.

Its first boundary is deliberately source-structural:

```text
CGM bytes
    -> bounded binary element framing
    -> metafile and picture lifecycle
    -> inspectable elements and diagnostics
```

This crate does not define a Tokimu CGM capability. It performs no rendering,
mesh creation, or engine-state mutation. Its admitted primitive adapter can
lower bounded source geometry into `VectorPath` while preserving CGM-only
provenance and fill/edge/stroke intent beside that provider-neutral path. CGM
solid-interior classification is source evidence only; colour, palette,
default-style, clipping, edge-width, and renderer policy remain inside this
adapter until independent consumers and Architectural Review justify a
different boundary.

The selected text fixture also preserves bounded restricted-text and
append-text source records beside geometry. These records retain the source
position, restriction values, final flag, active CGM state, and string, but do
not select a font, shape or lay out text, generate glyph outlines, or emit
renderer commands. Text remains a foundational presentation capability outside
the CGM importer.

`CHRHGT01`, `CHRORI01`, `TXTALN01`, `CHRSPA01`, and `TXTPTH01` establish a
bounded source-state profile for text: integer character height, encoded
character spacing, up/base orientation vectors, text-path direction, alignment
enums, and encoded continuous-alignment values. These remain snapshots on CGM
text records; they are not interpreted as a font size, shaped run, layout
policy, glyph placement, or renderer command.

The selected cell-array fixture preserves bounded raster source metadata beside
geometry: its three VDC corners, dimensions, local colour precision,
representation, active source state, and encoded payload length. It does not
decode pixel data, select a texture representation, create image resources, or
emit renderer commands. Raster presentation remains outside this importer.

The current profile admits only binary encoding. Unknown elements remain
visible as structured diagnostics carrying class, element ID, picture, and
source offset.

See `docs/Libraries/cgm-corpus-testing.md` for the corpus plan, fixture
provenance, selection policy, and graduation criteria.
