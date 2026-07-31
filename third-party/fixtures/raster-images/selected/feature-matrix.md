# Raster Image Selection V1 Feature Matrix

## Scope

This matrix describes why each pinned source fixture exists. It does not claim that
Tokimu currently decodes every case. Admission and execution status are
reported separately.

| Capability | Candidate cases | Current status |
| --- | --- | --- |
| Grayscale, 8-bit | `basn0g08.png` | executable in place, deterministic RGBA8 |
| RGB, 8-bit | `basn2c08.png` | executable in place, deterministic RGBA8 |
| Indexed color, 8-bit | `basn3p08.png` | executable in place, deterministic RGBA8 |
| Grayscale plus alpha, 8-bit | `basn4a08.png` | executable in place, deterministic RGBA8 |
| RGBA, 8-bit | `basn6a08.png` | executable in place, deterministic RGBA8 |
| RGBA, 16-bit | `basn6a16.png` | executable, explicit unsupported-bit-depth result |
| Adam7 interlaced RGBA | `basi6a08.png` | executable, explicit unsupported-interlace result |
| PNG filters 0-4 | `f00n2c08.png` through `f04n2c08.png` | executable in place, documented filters and deterministic fingerprints |
| Palette transparency | `tp1n3p08.png` | executable in place, deterministic RGBA8 |
| One-pixel indexed, 1-bit | `s01n3p01.png` | executable in place, deterministic RGBA8 |
| Odd interlaced dimensions, 2-bit | `s07i3p02.png` | executable, explicit unsupported-interlace result |
| Non-power-of-two indexed, 4-bit | `s33n3p04.png` | executable in place, deterministic RGBA8 |
| Empty/corrupt structure | `x00n0g01.png` | executable, rejected at PNG boundary |
| Injected carriage returns | `xcrn0g04.png` | executable, rejected at PNG boundary |
| Injected line feeds | `xlfn0g04.png` | executable, rejected at PNG boundary |

Execution references the bytes in the already-pinned W3C archive. PngSuite
redistribution is admitted under its preserved license record; this fixture
root still does not duplicate the selected source bytes.

## JPEG Selection

| Capability | Candidate cases | Current status |
| --- | --- | --- |
| Baseline 8-bit YCbCr 4:2:0 | `testorig.jpg`, `testimgint.jpg` | executable, pinned-provider fingerprints and tolerant `jpeg-decoder` differential evidence |
| Baseline 8-bit grayscale | `grayscale_square.jpg` | executable, one-component source, opaque grayscale RGBA8, and tolerant `jpeg-decoder` differential evidence |
| CMYK source model | Tokimu synthetic SOF0 frame | inspectable as CMYK, explicitly rejected before provider decode |
| Arithmetic sequential JPEG | `testimgari.jpg` | executable, rejected during profile preflight |
| Extended sequential 12-bit JPEG | `monkey12.jpg` | executable, rejected during profile preflight |

The two libjpeg-turbo baseline files intentionally share dimensions and
sampling while retaining distinct encoded bytes. The separate `jpeg-decoder`
fixture exercises a real one-component frame under the same bounded JPEG
profile. The arithmetic and 12-bit files make unsupported-mode behavior
observable without expanding JPEG V1.

## libjpeg-turbo BMP Selection

| Capability | Candidate cases | Current status |
| --- | --- | --- |
| 24-bit bottom-up BI_RGB | `shira_bird8.bmp` | executable, exact RGBA fingerprint |
| Odd-width row padding | `vgl_6434_0018a.bmp` | executable, exact RGBA fingerprint |
| Non-four-row-height and ordinary width | `vgl_6548_0026a.bmp` | executable, exact RGBA fingerprint |
| 8-bit palette, padding, and invalid-index boundary | Tokimu synthetic fixture | executable, exact RGBA and explicit index failure |

These cases supplement, but do not replace, Tokimu's exact synthetic BMP
boundary fixtures.

## Deferred High-Value PNG Features

- 16-bit sample conversion;
- Adam7 reconstruction for packed and 8-bit samples;
- reviewed gamma, ICC, and chromaticity fixture coverage beyond synthetic
  metadata observations;
- background and transparency composition policy;
- ancillary chunk preservation or rejection;
- multiple IDAT chunk layouts;
- compression-level equivalence;
- invalid color types, bit depths, and checksums not present in the vendored
  archive.

## Deferred Sources And Formats

- libpng regression selection;
- image-rs malformed-input stress selection;
- additional BMP header, palette, bitfield, and RLE sources;
- animated PNG, GIF, WebP, HDR, EXR, TIFF, and platform-native image formats.
