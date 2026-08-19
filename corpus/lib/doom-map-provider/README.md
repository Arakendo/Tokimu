# Doom Map Provider

Corpus-only classic Doom map-record decoding. This crate consumes a selected,
bounded WAD map block and returns source-indexed observations. It owns no WAD
container, ZIP, Resource Space, rendering, or runtime state.

The decoder covers fixed-record `THINGS`, `VERTEXES`, `LINEDEFS`, `SIDEDEFS`,
`SECTORS`, `SEGS`, `SSECTORS`, and `NODES`, plus bounded `REJECT` and
`BLOCKMAP` observations and their cross-table references.

Callers must supply explicit per-table record limits, auxiliary-lump byte and
reference limits, and one aggregate map-record byte limit. The provider checks
those limits before allocating decoded record vectors. Map selection decodes
one requested classic map block; the enclosing WAD provider's lump-count limit
bounds how many source map markers can be inspected.
