# Raster Image Fixtures

## Purpose

This directory records raster-image fixture provenance and bounded selections.
It does not duplicate source bytes that are already pinned elsewhere in the
repository.

The first source is the PNG Suite bundled inside the pinned W3C SVG
1.1 Second Edition archive:

```text
third-party/fixtures/w3c-svg-1.1-2nd-edition/
    upstream/images/PngSuite/
```

The PNG bytes remain authoritative at that location. This directory owns only
the raster corpus inventory, selection, capability mapping, and verification
metadata.

The second source is a deliberately small snapshot from the official
libjpeg-turbo test-image directory, pinned at commit
`5966050641633f9298b03f6b99a625067b22a91c`:

```text
upstream/libjpeg-turbo/
    LICENSE.md
    README.ijg
    testimages/
        LICENSE.txt
        testorig.jpg
        testimgint.jpg
        testimgari.jpg
        monkey12.jpg
        shira_bird8.bmp
        vgl_6434_0018a.bmp
        vgl_6548_0026a.bmp
```

Only selected source bytes and the complete applicable license chain are
preserved. The source files are unmodified.

The third source is a single reference fixture from `image-rs/jpeg-decoder`,
pinned at commit `eb2d7c0f6a2d0298aba7a7f8b9ca1440353e8f8c`:

```text
upstream/jpeg-decoder/
    LICENSE-APACHE
    LICENSE-MIT
    grayscale_square.jpg
```

It supplies the first reviewed real baseline grayscale JPEG. The repository's
Apache-2.0 or MIT license records are preserved alongside the unmodified source
byte.

## Admission Status

The PNG Suite documentation identifies the suite and its author:

```text
PNGSUITE
testset for PNG-(de)coders
created by Willem van Schaik
```

The bundled documentation does not repeat a redistribution grant, but the
upstream PngSuite license record is preserved at `upstream/PngSuite.LICENSE`.
It grants permission to use, copy, modify, and distribute the images for any
purpose without fee. The selected PNG cases are therefore **admitted source
fixtures**; the W3C archive remains their pinned distribution container.

This distinction means:

- local and CI verification may reference the already-vendored bytes;
- no new copy of the PNG Suite is created here;
- the selected PNG cases count as admitted executable source fixtures;
- executable profile coverage remains bounded and is not general PNG
  conformance.

The libjpeg-turbo image license explicitly identifies `testorig*` and
`testimg*` as IJG-licensed and the other selected images as BSD-3-Clause. Those
bytes are admitted as source fixtures. All seven files are now exercised by
the headless corpus crate. BMP output uses exact RGBA fingerprints. Baseline
JPEG output uses pinned-provider fingerprints plus test-only differential
comparison against `jpeg-decoder` `0.3.2`; the production provider remains
`zune-jpeg`.

Hand-authored BMP boundary fixtures live in the executable corpus crate and are
counted separately from the three external executable BMP fixtures.

The `jpeg-decoder` fixture is admitted separately from libjpeg-turbo because
its purpose is an actual one-component grayscale source frame, not JPEG color
regression pressure. Its dual-license records and source checksum are verified
offline with the other raster selections.

## Layout

```text
raster-images/
    README.md
    provenance.json
    inventory.json
    upstream/
        libjpeg-turbo/
        jpeg-decoder/
    selected/
        selection-v1.toml
        jpeg-selection-v1.toml
        jpeg-decoder-selection-v1.toml
        bmp-selection-v1.toml
        feature-matrix.md
```

## Verification

Run:

```powershell
pwsh -NoProfile -File .\scripts\verify-raster-image-corpus.ps1
```

The verifier is offline. It confirms the pinned source roots, license-document
hashes, selected file sizes and hashes, and manifest uniqueness. It does not
decode pixels or claim format conformance.
