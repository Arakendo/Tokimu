# Hello CGM

## Purpose

`hello-cgm` is the first visible consumer of the example-side CGM corpus
decoder and its admitted source-to-vector lowering boundary.

The example loads the selected `POLYLN01.cgm` WebCGM fixture and presents:

- binary source identity and size;
- metafile and picture lifecycle;
- initial VDC descriptor state and source-ordered extent corners;
- element counts by CGM class;
- a source-ordered element map;
- unsupported-element diagnostics retained by the decoder.
- a cached, neutral diagnostic outline of the lowered provider-neutral paths;
- vector topology counts, contours, and flattened point counts.

## Current Boundary

```text
CGM bytes
    -> bounded binary inspection
    -> lifecycle, descriptor state, and element provenance
    -> admitted primitive lowering
    -> provider-neutral vector paths
    -> diagnostic presentation
```

The vector preview is deliberately a neutral diagnostic outline. It is not a
CGM picture renderer: fill, edge, colour-table, clipping, and paint semantics
remain owned by future CGM corpus slices. The example exposes source VDC
corners without sorting them so descending source Y axes remain observable.

## Success Criteria

- The fixture is loaded through `cgm-corpus`.
- The window reports the decoded metafile and picture.
- Element classes and source order are visible.
- The selected picture lowers through `cgm-corpus` into finite,
  provider-neutral paths.
- The cached preview reuses `ui-tools` vector stroke tessellation rather than
  implementing local geometry generation.
- No CGM parser, lowering, paint, edge, or clipping policy exists in this
  application.
- Missing or malformed source data fails explicitly.

## Implementation Observation

The first native run was noticeably laggy even though the inspection is static
and contains only a modest number of labels and markers. This is recorded as
shared UI performance evidence in
`docs/Notes/ui-presentation-performance-evidence.md`; it should be addressed
through UI invalidation, retained geometry, batching, and diagnostics rather
than an application-local workaround.
