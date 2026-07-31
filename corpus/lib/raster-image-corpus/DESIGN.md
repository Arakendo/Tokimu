# Raster Image Corpus

## Purpose

`raster-image-corpus` incubates bounded raster-image decoding and
provider-neutral image evidence. It exists to make codec, image, asset, and
renderer ownership observable before Tokimu admits a shared image capability.

The first implementation deliberately supports narrow BMP, PNG, and JPEG
profiles.

BMP:

- uncompressed Windows BITMAPINFOHEADER data;
- 8-bit indexed rows with a bounded RGB palette;
- 24-bit BGR pixels;
- 32-bit BGRA pixels;
- top-down and bottom-up row order;
- deterministic top-down RGBA8 output.

PNG:

- non-interlaced 1-, 2-, 4-, and 8-bit grayscale and indexed samples;
- non-interlaced 8-bit RGB, grayscale-alpha, and RGBA samples;
- scanline filters None, Sub, Up, Average, and Paeth;
- palette and `tRNS` expansion;
- CRC, chunk-order, allocation, and decompression bounds;
- provider-neutral `sRGB`, `gAMA`, and compressed `iCCP` observation without
  implicit pixel conversion;
- deterministic top-down RGBA8 output.

JPEG:

- baseline sequential 8-bit grayscale or YCbCr-family input;
- bounded marker and frame preflight before provider decoding;
- strict decode through the pinned `zune-jpeg` provider;
- provider-neutral JFIF, EXIF orientation, and ICC chunk observations;
- deterministic top-down RGBA8 output for a named provider version;
- explicit rejection of progressive, arithmetic, 12-bit, and unsupported
  component profiles.

## Owns

- source-format validation for its admitted corpus profiles;
- decode limits and checked allocation arithmetic;
- corpus-owned decoded-image evidence;
- a narrow asset-resolution bridge that proves source bytes can become an
  opaque Tokimu asset handle without exposing provider objects;
- source and output orientation evidence;
- explicit color and alpha interpretation;
- deterministic pixel fingerprints;
- provider-version-qualified JPEG fingerprints;
- deterministic provider-neutral JSON artifacts;
- structured decoder failures.

## Dependency Boundary

The corpus can resolve one supplied image byte sequence into a typed
`DecodedImage` asset. It does not own a recursive dependency graph.

- Missing bytes fail before an asset record is allocated.
- Malformed or unsupported bytes fail at the format-provider boundary.
- Repeated resolution is explicit per request; the current `AssetStore` does
  not silently coalesce requests by a source label.
- Cycles cannot exist in this narrow image-byte operation. Detecting cycles
  belongs to a future package or model dependency resolver that owns graph
  traversal and canonical asset identity.

## Does Not Own

- Tokimu asset identity or dependency resolution;
- production image-provider traits;
- filesystem or browser loading;
- GPU textures, samplers, uploads, or residency;
- image display, UI, or shader policy;
- screenshot export or framebuffer capture;
- unadmitted PNG, JPEG, or BMP profiles.

## Boundary

```text
encoded image bytes
    -> bounded BMP, PNG, or JPEG provider
    -> DecodedImage evidence
    -> corpus-owned asset-resolution evidence
    -> future presentation consumer
```

The decoder always emits top-down RGBA8 rows. `source_orientation` preserves
whether the BMP stored rows top-down or bottom-up.

8-bit indexed and 24-bit BMP output are explicitly opaque. The alpha byte in
uncompressed 32-bit BMP is preserved, but its meaning is recorded as
`Unspecified` because the basic BMP profile does not provide a universal alpha
contract.

PNG alpha is recorded as straight alpha. PNG images without alpha or `tRNS`
are recorded as opaque. The first PNG profile observes an `sRGB` chunk but
does not perform color conversion. `gAMA` and `iCCP` declarations remain
metadata observations; they do not alter decoded pixels or admit a color
management provider. Adam7 and 16-bit sample depths stop explicitly.

The selected PngSuite fixtures execute directly from Tokimu's pinned W3C
archive. Supported profiles produce deterministic decoded evidence; deferred
profiles and malformed cases produce explicit boundary failures. Their
preserved PngSuite license record separately admits redistribution.

JPEG output is recorded as opaque with unspecified color-space interpretation.
The adapter reports the bounded grayscale or YCbCr source model, then validates
marker framing, frame precision, source color model,
dimensions, and allocation bounds before provider decoding. Exact JPEG pixel
fingerprints are regression evidence for the pinned provider profile, not
portable conformance oracles across independent JPEG decoders. Differential
comparison and its tolerance policy remain future corpus work. EXIF
orientation is observed rather than applied, and ICC metadata records chunk
completeness without performing color conversion.

## Admission

This crate is corpus evidence, not an admitted Tokimu image capability.
Promotion requires independent consumers and format providers to pressure the
same provider-neutral semantics without leaking source-format or renderer
objects.

The initial asset bridge resolves selected PNG bytes through `AssetStore` and
returns a typed `AssetHandle<DecodedImage>` plus lifecycle observations. Its
source label is diagnostic provenance only. It does not create filesystem
loading, texture upload, a production asset loader, or a renderer contract.
