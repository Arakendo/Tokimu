# Doom Raster Provider

Corpus-only decoding for Doom's indexed raster resources. The initial scope
exposes `PLAYPAL` RGB palette entries and `COLORMAP` index-remapping tables as
bounded source observations.

The provider also decodes bounded patches, sprites, flats, texture catalogs,
and composed textures. Callers supply explicit source-byte, record/reference,
dimension, pixel, post, and aggregate decoded-byte limits before any result is
admitted.

Sprite frame/rotation observations also retain whether an eight-character
lump's second pair requires horizontal mirroring. The existing ordered
fingerprint remains stable because source lump identity and pair order already
encode that fact.

This crate does not choose renderer lighting, color space, texture upload, or
software-rendering compatibility policy.
