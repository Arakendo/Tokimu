# Doom Raster Provider

Corpus-only decoding for Doom's indexed raster resources. The initial scope
exposes `PLAYPAL` RGB palette entries and `COLORMAP` index-remapping tables as
bounded source observations.

The provider also decodes bounded patches, sprites, flats, texture catalogs,
and composed textures. Callers supply explicit source-byte, record/reference,
dimension, pixel, post, and aggregate decoded-byte limits before any result is
admitted.

This crate does not choose renderer lighting, color space, texture upload, or
software-rendering compatibility policy.
