# Doom Raster Provider

Corpus-only decoding for Doom's indexed raster resources. The initial scope
exposes `PLAYPAL` RGB palette entries and `COLORMAP` index-remapping tables as
bounded source observations.

This crate does not choose renderer lighting, color space, texture upload, or
software-rendering compatibility policy.
