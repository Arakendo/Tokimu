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

This crate does not define a Tokimu CGM capability. It performs no rendering
and does not create `VectorPath`, mesh, or engine state. CGM semantics remain
inside this adapter until independent consumers and Architectural Review
justify a different boundary.

The current profile admits only binary encoding. Unknown elements remain
visible as structured diagnostics carrying class, element ID, picture, and
source offset.

See `docs/Libraries/cgm-corpus-testing.md` for the corpus plan, fixture
provenance, selection policy, and graduation criteria.
