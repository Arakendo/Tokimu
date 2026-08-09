# Doom WAD Provider

This is corpus-local, outer-ring evidence for the Doom WAD checklist. It owns
only bounded WAD container inspection: signatures, directory records, lump
ranges, names, order, duplicates, source identity, and structural diagnostics.

It does not own ZIP transport, Resource Space identity, Doom map semantics,
renderer objects, or gameplay state. Those are intentionally separate later
slices of `docs/Plans/DOOM/DOOM WAD Checklist.md`.

The test fixtures are constructed from Tokimu-authored bytes. CI does not read
the reviewed Doom or Heretic packages.

Names retain their source spelling as printable ASCII with only trailing NUL
padding removed. This provider does not apply case normalization or host-path
interpretation; any later lookup policy must be explicit at its consumer
boundary.
