# Doom Map Provider

Corpus-only classic Doom map-record decoding. This crate consumes a selected,
bounded WAD map block and returns source-indexed observations. It owns no WAD
container, ZIP, Resource Space, rendering, or runtime state.

The initial decoder deliberately covers fixed-record `THINGS`, `VERTEXES`,
`LINEDEFS`, `SIDEDEFS`, and `SECTORS` plus their cross-table references. BSP,
seg, subsector, REJECT, and BLOCKMAP work remains a separate next increment.
