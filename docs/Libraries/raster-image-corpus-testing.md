# Raster Image Corpus Testing

## Status

Planning and bounded implementation began on 2026-07-30.

The PNG Suite already contained in Tokimu's pinned W3C SVG archive is
inventoried as a 157-file source. A versioned 19-case selection pins 16 valid
and 3 malformed files by size and SHA-256 without duplicating their bytes. The
PngSuite license grants permission to use, copy, modify, and distribute its
images for any purpose and without fee; Tokimu preserves that license record
with the fixture provenance. All 19 selected PNG fixtures are now executed in
place from the pinned archive:
supported 1-, 2-, 4-, and 8-bit profiles produce deterministic RGBA8 evidence;
the selected 16-bit RGBA source stops explicitly at the current profile boundary;
documented filter profiles verify their encoded scanline filter and decoded
fingerprint, deferred Adam7 profiles stop explicitly, and malformed cases fail
at the PNG boundary. The W3C archive remains a pinned distribution container;
Tokimu's selection does not duplicate the PNG bytes.

Tokimu now also preserves a license-complete, checksum-pinned selection from
libjpeg-turbo revision `5966050641633f9298b03f6b99a625067b22a91c`.
The admitted source selection contains two baseline 8-bit YCbCr JPEG cases,
one baseline grayscale JPEG case from the separately pinned `jpeg-decoder`
repository, explicit arithmetic and 12-bit rejection cases, and three 24-bit
BMP cases including odd-width row-padding pressure. These files remain
upstream source fixtures; Tokimu-authored decoded fingerprints are derived
evidence, not upstream conformance oracles.

`corpus/lib/raster-image-corpus` now provides the first corpus-owned,
headless decoded-image evidence, a bounded uncompressed BMP profile, a bounded
non-interlaced PNG profile, and an incubating bounded baseline JPEG adapter.
The PNG implementation expands 1-, 2-, 4-, and 8-bit grayscale and indexed
samples plus 8-bit RGB, grayscale-alpha, and RGBA samples to RGBA8,
reconstructs all five scanline filters, expands palettes and `tRNS`, validates
CRCs and chunk order, observes `sRGB`, `gAMA`, and `iCCP` declarations without
converting pixels, and bounds decompression and allocation. The JPEG adapter
preflights source
framing, frame profile, dimensions, source color model, and output allocation
before invoking pinned `zune-jpeg` `0.5.15` in strict, safe, RGBA output mode.
The same preflight now emits provider-neutral JFIF version and density, EXIF
orientation, and ICC chunk-completeness observations. Metadata observation
does not silently rotate pixels, perform color conversion, or expose provider
objects.

The JPEG boundary classifies one-component baseline frames as `grayscale` and
three-component frames as `ycbcr` before provider decoding. A separately
pinned `jpeg-decoder` baseline grayscale fixture now proves that branch with a
real 10x10 source frame; the libjpeg-turbo selection remains deliberately
focused on YCbCr and unsupported-profile regression pressure.
Tokimu also has useful local evidence:

- 78 first-party PNG texture fixtures under `corpus/assets/PNG`;
- deterministic CPU-owned RGBA8-to-BMP export in
  `corpus/lib/screenshot`;
- a narrow renderer contract that uploads RGBA8 pixels as
  `Rgba8UnormSrgb`;
- an external PNG dependency in the selected Khronos `BoxTextured` glTF case;
- CGM cell-array and FBX texture-reference boundaries that can become future
  independent consumers.

The implementation decodes hand-authored and external 24-bit bottom-up BMP
fixtures, a hand-authored 32-bit top-down fixture, and synthetic 8-bit indexed
palette rows. It preserves source orientation, produces deterministic top-down
RGBA8 output, and rejects unsupported compression, invalid offsets, truncated
rows, invalid palette tables or indices, source-limit violations, and
dimensions beyond policy. The three admitted baseline JPEG fixtures decode
deterministically for the pinned provider profile, including a real
one-component grayscale source. Arithmetic, progressive, 12-bit, malformed
framing, and resource-limit cases stop explicitly at the corpus adapter
boundary. The admitted baseline source provides JFIF evidence; Tokimu-authored
marker fixtures exercise EXIF orientation and complete, incomplete, duplicate,
and inconsistent ICC sequencing without claiming an external metadata corpus.

The first direct consumer-pressure bridge is now executable: the selected
Khronos `BoxTextured` glTF case preserves its `CesiumLogoFlat.png` reference,
the raster corpus decodes its bytes, and `tokimu-assets::AssetStore` allocates
and prepares an opaque `AssetHandle<DecodedImage>`. The source label remains
lifecycle provenance only; no source path or PNG provider object enters a
renderer contract. This proves asset identity plus normalized pixel evidence,
not filesystem loading or GPU texture upload.

PNG, JPEG, and BMP now also converge through the same explicit `ColorSrgb`
texture-preparation result and material texture-slot shape. This is the
provider-neutral handoff consumed by `hello-raster-image`: renderer upload,
material texture binding, the renderer-owned default sampler, and `Texture2d`
shader sampling all operate without source-format knowledge. The example also
uses immutable source and translucent inspection materials to prove that tint
and opacity change through material data rather than decoded-pixel mutation.
It proves neither framebuffer equivalence nor a general sampler-control API;
those remain separate renderer-owned observations under the TypeScript
shader/material plan.

The bounded corpus runner writes the full V1 PNG selection plus the admitted
JPEG and BMP review cases under `target/raster-image-corpus/review-v1`. Its
JSON reports remain the
authoritative structural evidence. Successful cases additionally emit a
separate `texture-upload-preparation` artifact, then a deterministic CPU BMP
preview and a manifest that explicitly records the decoded-image source stage
and `gpu_framebuffer_capture=false`. These artifacts make conversion policy
observable, but do not claim GPU allocation, sampler policy, or native-window
framebuffer output. Each source artifact also records its observed decode time
in microseconds. It is descriptive evidence for review and diagnostics, never
a wall-clock correctness threshold.

Run the full representative review set with:

```powershell
cargo run -p raster-image-corpus --bin raster-image-corpus
```

Run the bounded native visual inspector with:

```powershell
cargo run -p hello-raster-image
```

Use the left and right arrow keys to cycle the fixed PNG, JPEG, and BMP
fixtures. Press space to switch between source-color sampling and a translucent
cyan inspection material. The example emits
`target/hello-raster-image/raster-shader-contract.json`, which composes each
pre-GPU preparation artifact with the material slot, sampler, blend,
orientation, and shader-sampling declarations while explicitly recording that
no GPU framebuffer was captured. The native presentation remains manual
evidence; the runner's structural reports remain authoritative for decoding.

The runner can narrow its already-declared cases without searching fixture
directories, for example:

```powershell
cargo run -p raster-image-corpus --bin raster-image-corpus -- `
  --format jpeg --expected candidate-rejection
```

The asset boundary is now explicitly provider-neutral: PNG, baseline JPEG, and
BMP adapters each decode their bounded source bytes before registering the same
opaque `AssetHandle<DecodedImage>`. Asset lifecycle evidence retains only the
caller-supplied source label; neither a source-format type nor decoder object
crosses into asset identity or the renderer bridge.

The next narrow bridge is also now corpus-proven: normalized top-down RGBA8
pixels can become the existing `tokimu-render::Texture` input only when the
caller explicitly declares `ColorSrgb` use. The bridge records source color
and alpha observations plus the current `Rgba8UnormSrgb` target, performs no
implicit color or alpha conversion, and rejects `LinearData` intent until the
renderer exposes a distinct data-texture contract. GPU allocation, sampler
policy, and residency remain renderer-owned.

Run the offline acquisition verifier with:

```powershell
pwsh -NoProfile -File .\scripts\verify-raster-image-corpus.ps1
```

These are acquisition, synthetic implementation, and consumer pressures, not
evidence of general PNG, JPEG, BMP, color-management, or texture support.

## Purpose

Tokimu needs a bounded and observable way to turn encoded raster-image bytes
into provider-neutral pixel evidence without making a source format, decoder
library, filesystem, or GPU backend canonical engine meaning.

The corpus should exercise distinct ownership stages:

```text
encoded image bytes
    -> format inspection and decoding
    -> provider-neutral decoded image
    -> explicit color, alpha, and orientation policy
    -> asset or presentation consumer
    -> optional texture upload
    -> optional renderer evidence
```

PNG, JPEG, and BMP are source formats. They must not become Tokimu's canonical
image representation.

The central claim under test is:

> Applications and assets own image intent. Format providers own encoded image
> technology. Renderers own GPU execution.

## Motivation

Raster images already appear at several Tokimu boundaries:

- textures applied to meshes and shader materials;
- images referenced by glTF and FBX assets;
- deterministic CPU diagnostic artifacts;
- future UI, icon, sprite, and document presentation;
- possible CGM cell-array lowering;
- browser and native consumer corpora.

Without a deliberate boundary, these callers can accidentally conflate:

- decoding with texture upload;
- encoded file format with pixel format;
- color images with linear data textures;
- straight alpha with premultiplied alpha;
- source orientation with normalized orientation;
- saved CPU artifacts with GPU framebuffer capture;
- successful decoding with visually correct sampling.

The corpus exists to make those distinctions observable before a shared image
capability is admitted.

## Architectural Ownership

### Applications And Asset Semantics

Applications and asset systems own:

- image identity;
- intended use such as color, normal, mask, height, or diagnostic data;
- fallback and missing-asset policy;
- whether metadata normalization is requested;
- lifetime and dependency relationships.

Applications should not parse PNG chunks, JPEG markers, or BMP headers.

### Format Providers

PNG, JPEG, BMP, and future format providers own:

- container and header parsing;
- decompression;
- palette expansion;
- source-format metadata;
- format-specific validation;
- bounded conversion into a provider-neutral decoded-image result;
- source-specific diagnostics.

Provider-native objects must stop at the importer boundary.

### Provider-Neutral Image Semantics

An incubating image contract may own:

- width and height;
- pixel layout and component precision;
- row stride;
- color-space interpretation;
- alpha interpretation;
- orientation policy;
- immutable pixel bytes;
- bounded decode diagnostics.

The first implementation may use a narrow RGBA8 result, but the architecture
must not silently imply that all images are RGBA8, sRGB, straight-alpha color
images.

### Renderer

The renderer owns:

- GPU texture allocation;
- upload layout required by the backend;
- texture views and sampler objects;
- mipmap execution;
- filtering and addressing mechanisms;
- residency, replacement, batching, and cache lifetime;
- backend-specific texture formats;
- framebuffer capture when admitted separately.

The renderer must not parse source image formats or infer application image
meaning from filenames.

### Diagnostic And Export Tools

Image-export and screenshot tools own:

- deterministic artifact encoding;
- artifact manifests;
- comparison inputs;
- manual or automated capture labeling.

An image encoder is not an image decoder contract. A CPU source-buffer export
is not a GPU framebuffer capture.

### Trusted Core

`tokimu-core` owns none of:

- PNG, JPEG, BMP, or other codecs;
- filesystem or browser image APIs;
- image metadata parsing;
- GPU textures;
- pixel conversion;
- image export.

If provider-neutral image semantics earn promotion, they belong to an asset or
presentation capability outside the trusted core.

## Dependency Direction

The intended direction is:

```text
application image intent
        |
        v
asset identity and dependency resolution
        |
        v
provider-neutral decoded image
        ^
        |
PNG / JPEG / BMP providers

provider-neutral decoded image
        |
        +--------------------+
        |                    |
        v                    v
CPU diagnostics        renderer texture upload
                             |
                             v
                         GPU execution
```

No source-format type should appear in renderer contracts. No renderer handle
should appear in decoder output.

## Goals

- Acquire a lawful, pinned, and reproducible raster-image fixture selection.
- Validate source framing, dimensions, metadata, and payload bounds before
  allocating decoded storage.
- Define bounded provider-neutral decoded-image evidence.
- Prove lossless PNG and BMP decoding with exact pixel fingerprints.
- Prove baseline JPEG decoding with explicit tolerant comparison.
- Make color-space, alpha, and orientation policy visible.
- Validate native and WASM-compatible decoding without requiring a window or
  GPU.
- Exercise asset resolution and renderer upload as later, separate stages.
- Preserve exact failure ownership through stage-specific artifacts.
- Report source, feature, executable, and rendered coverage separately.

## Non-Goals

- Complete conformance with every raster-image standard.
- A general image editor.
- Camera RAW, video, or medical-image support.
- Animated GIF, animated WebP, or APNG in the first profile.
- A complete color-management system in the first profile.
- GPU block compression or runtime texture transcoding in the first profile.
- Making one third-party decoder a Tokimu semantic dependency.
- Treating browser-native image decoding as authoritative Tokimu behavior.
- Treating successful texture upload as proof of decoder correctness.
- Treating a visually plausible render as proof of color correctness.
- General framebuffer capture; that remains a separate renderer question.

## Current Tokimu Evidence

### First-Party PNG Texture Fixtures

`corpus/assets/PNG` contains 78 PNG files across color and presentation
variants. They exercise:

- grid and center-line alignment;
- diagonal and checker patterns;
- low-contrast presentation;
- color tint and material pressure;
- UV, filtering, and shader inspection.

They are first-party reference assets, not external standards fixtures or
reviewed decoder goldens.

### Deterministic BMP Export

`corpus/lib/screenshot` validates CPU-owned RGBA8 buffers and writes
deterministic BMP artifacts. This proves an output encoding path only:

```text
CPU RGBA8 source buffer -> deterministic BMP artifact
```

It does not prove:

- BMP decoding;
- PNG or JPEG decoding;
- texture upload;
- GPU readback;
- display color equivalence.

### Current Renderer Texture Contract

`tokimu-render::Texture` currently stores:

```text
width
height
rgba8
```

The wgpu backend currently uploads those bytes as `Rgba8UnormSrgb`.

This is a useful narrow execution contract. It is not yet a complete image
semantic model because it does not distinguish color/data use, source color
space, alpha mode, row stride, orientation, or component precision.

### Independent Consumer Pressure

The selected Khronos `BoxTextured` fixture references
`CesiumLogoFlat.png`. The current glTF corpus preserves and verifies the image
dependency but stops before image decoding and texture lowering.

FBX texture references and CGM cell arrays provide later independent pressure.
Neither should define the shared image contract by itself.

The first named integration targets are:

- native asset resolution for the selected Khronos `BoxTextured` external PNG
  (now corpus-proven through a narrow `AssetStore` bridge);
- the ASP.NET/WASM asset workbench as a browser consumer of the same
  provider-neutral decoded-image observation. The first proof covers bounded
  PNG, BMP, and baseline JPEG metadata and deterministic pixel fingerprints;
  it does not expose decoded pixels to TypeScript or delegate parsing to the
  browser. Truncated JPEG framing is rejected by the Rust/WASM boundary before
  browser presentation can occur.

Browser pixel preview and renderer texture-upload remain deferred consumer
claims. JPEG/WASM compatibility is also intentionally still unproven.

The bridge also treats unavailable dependency bytes as an explicit
asset-resolution failure before allocating an asset record. It deliberately
distinguishes an unavailable dependency from malformed PNG bytes, which fail
at the PNG provider boundary instead.

The current bridge resolves a supplied image byte sequence, not a recursive
package graph. Repeated requests intentionally allocate distinct typed handles
until a future dependency resolver proves a canonical-identity and
deduplication contract. Cycles are therefore not silently ignored: they are
outside this narrow operation and must be rejected by the future resolver that
owns graph traversal. Unsupported encoded profiles stop at their format
provider boundary, while unsupported renderer texture intent stops at the
renderer-preparation boundary.

Texture preparation emits a deterministic pre-GPU artifact that records its
RGBA8 fingerprint, source metadata, intended color use, and the current target
format. It explicitly says `gpu_upload_performed: false`: GPU upload, sampler
selection, residency, and framebuffer evidence remain renderer-owned and must
be emitted separately when that backend diagnostic contract exists.

The admitted PNG Suite `tp1n3p08.png` fixture also travels through the same
decode, asset-resolution, and preparation path before the shared `screenshot`
helper writes a deterministic CPU BMP review artifact. Re-decoding that BMP
must preserve every RGBA8 byte, including palette-derived alpha. This is an
export/import boundary test, not a GPU texture or framebuffer claim.

## Format Profiles

### PNG V1

Highest-return PNG pressure:

- signature and chunk bounds;
- IHDR dimensions and legal combinations;
- RGB and RGBA 8-bit images;
- 1-, 2-, 4-, and 8-bit grayscale;
- 1-, 2-, 4-, and 8-bit indexed color and palette expansion;
- 8-bit grayscale-alpha;
- `tRNS` transparency;
- all five scanline filters;
- non-interlaced decoding;
- deterministic row and pixel fingerprints;
- `sRGB`, `gAMA`, and `iCCP` metadata observation.

Deferred PNG pressure:

- Adam7 interlace;
- 16-bit component conversion;
- APNG animation;
- unknown critical and ancillary chunk policy beyond the bounded first
  profile;
- full ICC color conversion.

### BMP V1

Highest-return BMP pressure:

- file and DIB header bounds;
- 8-bit indexed palette rows;
- 24-bit BGR rows;
- 32-bit BGRA rows;
- four-byte row padding;
- bottom-up and top-down orientation;
- finite dimensions and checked image-size calculations;
- uncompressed `BI_RGB` images.

Deferred BMP pressure:

- bitfields;
- RLE4 and RLE8;
- embedded JPEG or PNG payloads;
- uncommon OS/2 and extended DIB variants.

### JPEG V1

Highest-return JPEG pressure:

- SOI/EOI and marker bounds;
- baseline sequential DCT;
- grayscale and ordinary YCbCr images, with source-model observation;
- common chroma-subsampling profiles;
- deterministic dimensions and decoded pixel fingerprints;
- EXIF orientation observation;
- ICC and JFIF metadata observation;
- truncated and malformed entropy data.

Deferred JPEG pressure:

- progressive JPEG until baseline decoding is stable;
- arithmetic coding;
- CMYK and YCCK conversion;
- lossless JPEG modes;
- full ICC color conversion.

JPEG is lossy. Its validation policy must not require source-independent byte
identity across every conforming decoder. Differential comparison must use a
named reference decoder, explicit color assumptions, and reviewed tolerances.

The admitted baseline profile compares production `zune-jpeg` `0.5.15` output
against test-only `jpeg-decoder` `0.3.2`. Both outputs are normalized to opaque
top-down RGBA8 without ICC conversion or EXIF reorientation. Across the three
admitted baseline fixtures, the contract is a maximum RGB-channel delta of four
levels and a mean absolute RGB delta of at most 0.5 levels. Alpha, dimensions,
and normalized byte lengths must agree exactly. This comparison is provider
evidence, not a public raster API dependency.

### Future Formats

WebP, GIF, TIFF, QOI, HDR, EXR, and platform-native formats may be considered
after concrete consumers appear. Listing them here does not admit them.

## Candidate Upstream Sources

Raster sources are admitted for distinct evidence roles. A large mixed-format
collection must not displace a smaller purpose-built correctness corpus merely
because it contains more files.

### Primary Corpus Sources

The first source-admission order is:

1. **PNG Suite** for intentional PNG decoder semantics:
   - color types and bit depths;
   - palette and `tRNS` behavior;
   - scanline filters;
   - interlace pressure;
   - malformed structural cases.
2. **libjpeg-turbo test images** for JPEG correctness and regression pressure:
   - baseline grayscale and YCbCr;
   - common chroma subsampling;
   - marker and entropy boundaries;
   - malformed and truncated inputs;
   - a named differential reference implementation.
3. **A curated BMP suite** for BMP-specific representation pressure:
   - DIB header variants;
   - row padding;
   - top-down and bottom-up storage;
   - palettes and bitfields;
   - malformed offsets and sizes.

These sources do not have interchangeable roles. PNG Suite is not a JPEG
oracle, libjpeg-turbo is not the owner of Tokimu image semantics, and a BMP
collection is useful only when its cases intentionally expose BMP-specific
behavior.

### Secondary Stress Sources

After the primary sources establish correctness profiles, evaluate:

- libpng regression fixtures for additional malformed PNG and decoder
  regression pressure;
- image-rs fixtures for broad multi-format stress and Rust ecosystem
  interoperability;
- selected Web Platform Tests for browser interoperability and metadata
  behavior;
- standards-published examples where provenance and redistribution are clear.

Secondary sources are stress corpora, not automatic conformance oracles. Their
expected results must be reviewed independently before they become executable
goldens.

### Downstream Consumer Corpora

Khronos glTF assets and other model corpora exercise a different boundary:

```text
model dependency
        ↓
asset identity
        ↓
image decoding
        ↓
texture upload
```

Those cases validate composition and dependency resolution. They do not replace
format-focused PNG, JPEG, or BMP correctness fixtures.

Every source must be reviewed for:

- stable identity and pinned revision or archive hash;
- license and redistribution terms;
- whether expected pixels or only source bytes are supplied;
- malformed-input intent;
- fixture dependencies;
- total size;
- whether generated derivatives may be committed.

No candidate is admitted merely because it is publicly downloadable.

## Proposed Fixture Layout

```text
third-party/fixtures/raster-images/
    README.md
    provenance.json
    inventory.json
    upstream/
        png/
        jpeg/
        bmp/
    selected/
        selection-v1.toml
        feature-matrix.md
        expected/
```

`upstream/` should preserve admitted source bytes verbatim. The selection
manifest should reference those bytes rather than duplicate them.

Tokimu-authored fixtures should remain distinct:

```text
corpus/assets/PNG/
tests/fixtures/raster/
```

Derived reductions and malformed mutations must identify their source and must
not increase upstream source coverage.

## Selection Manifest

Each selected case should record:

```toml
[[case]]
id = "png-rgba8-basic"
format = "png"
source = "upstream/png/example.png"
capabilities = ["rgba8", "non-interlaced", "srgb-metadata"]
reason = "Proves the first lossless color-image decode boundary."
expected_stage = "decoded-image"
expected = "pass"
```

Additional fields should include:

- source identity and hash;
- license;
- dimensions;
- expected format profile;
- required metadata;
- expected invalid or unsupported boundary;
- reference decoder and version when used;
- exact or tolerant comparison policy;
- whether the case may be uploaded or rendered.

## Candidate Provider-Neutral Evidence

The first corpus-owned model may resemble:

```rust
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub row_stride: usize,
    pub pixel_format: PixelFormat,
    pub color_space: ColorSpace,
    pub alpha_mode: AlphaMode,
    pub orientation: ImageOrientation,
    pub pixels: Vec<u8>,
}
```

This is an evidence shape, not an admitted public API.

The contract must distinguish:

- encoded image metadata from normalized output policy;
- source component layout from decoded pixel layout;
- color data from non-color data;
- straight, premultiplied, opaque, and absent alpha;
- source orientation from physically reordered pixels;
- row stride from tightly packed width.

The v1 implementation may intentionally support only a subset. Unsupported
combinations must stop explicitly rather than being mislabeled.

## Color, Alpha, And Orientation Policy

### Color

The corpus must separately observe:

- explicit sRGB metadata;
- gamma metadata;
- embedded ICC profiles;
- no declared color space;
- application-declared data textures.

Decoders may expose metadata before color conversion is implemented. They must
not silently claim converted sRGB pixels when no conversion occurred.

### Alpha

The corpus must record whether decoded pixels are:

- opaque;
- straight alpha;
- premultiplied alpha;
- alpha absent;
- alpha interpretation unknown.

Renderer upload and blending must consume an explicit policy. They must not
infer it from a file extension.

### Orientation

BMP row direction and JPEG EXIF orientation are source semantics. The importer
must state whether output pixels:

- retain source orientation with metadata;
- are physically normalized;
- are unsupported.

Orientation normalization must be deterministic and testable.

## Safety And Resource Limits

Image decoding is hostile-input parsing. Before allocation, validate:

- maximum source byte count;
- non-zero, bounded dimensions;
- checked width-by-height and stride arithmetic;
- maximum decoded byte count;
- legal channel and bit-depth combinations;
- chunk, segment, palette, and metadata bounds;
- decompression output limits;
- row and scanline lengths;
- truncated payload behavior;
- excessive ICC, EXIF, comment, and ancillary metadata;
- decoder recursion and frame-count limits where relevant.

A compressed image that declares an unreasonable decoded footprint must fail
before allocating that footprint.

Limits belong to importer or asset policy, not the renderer.

## Structural Validation

For every decoded case, record:

- source format and size;
- dimensions;
- decoded pixel format;
- row stride;
- color-space status;
- alpha mode;
- source and output orientation;
- pixel byte count;
- finite checked allocation calculations;
- source hash;
- decoded-pixel hash;
- alpha coverage;
- per-channel minimum and maximum;
- diagnostic count and severity.

For exact lossless cases, compare decoded pixels byte-for-byte against an
independently produced expected buffer.

For JPEG cases, preserve:

- reference decoder and version;
- comparison color space;
- maximum per-channel error;
- mean absolute error;
- an explicitly reviewed perceptual or signal metric when useful;
- tolerance rationale.

One decoder must not generate both the implementation result and its sole
oracle.

## Diagnostic Artifacts

Each selected case should be able to emit:

```text
reports/<case-id>/
    source.json
    decode.json
    metadata.json
    pixels.bin
    pixels.manifest
    comparison.json
    graph.json
    preview.bmp
    texture-upload.json
    render.manifest
```

The artifact envelope should record:

- schema version;
- case ID;
- source hash;
- provider and version;
- algorithm/profile identity;
- decode limits;
- output hash;
- elapsed time;
- authoritative stage;
- exact or tolerant comparison policy.

Structural decode artifacts are authoritative for decoding. CPU previews, GPU
captures, and native-window screenshots are complementary and must identify
their source.

The first stage whose artifact diverges is the owning diagnostic boundary.

## Texture Upload And Render Evidence

Texture upload should be tested only after decoded-image evidence is stable:

```text
decoded image
    -> explicit upload conversion
    -> renderer-neutral texture request
    -> backend texture
    -> sampled mesh or vector presentation
```

Upload evidence should record:

- source decoded-image hash;
- chosen GPU format;
- upload dimensions and row layout;
- mip-level policy;
- sampler policy;
- color/data interpretation;
- alpha and blending policy.

The render corpus should include:

- a pixel-aligned 1:1 presentation;
- nearest and linear filtering;
- UV orientation;
- clamp and repeat addressing;
- alpha blending;
- color-image and linear-data distinctions;
- one textured glTF mesh;
- one first-party diagnostic texture.

A render mismatch should not automatically be assigned to the decoder. Compare
decode, upload, sampling, shader, and capture artifacts in order.

The next consumer proof is tracked in
[`typescript-shader-material-presentation-control.md`](../Plans/Standalone/typescript-shader-material-presentation-control.md):
the shader receives a prepared texture identity and declared interpretation, not
encoded PNG/JPEG/BMP bytes. The first proof must retain the diagnostic sequence
above so a sampled-texture mismatch is localized before it is blamed on a
decoder.

## Coverage Accounting

Report these denominators separately:

1. Total files in each pinned upstream source.
2. Unique upstream files selected.
3. Derived and synthetic fixtures.
4. Cases reaching inspection.
5. Cases reaching decoded-image evidence.
6. Cases reaching exact or tolerant comparison.
7. Cases reaching texture upload.
8. Cases reaching CPU or GPU visual evidence.
9. Expected-invalid cases.
10. Explicitly unsupported cases.

The primary source-coverage calculation is:

```text
unique selected upstream files
------------------------------ x 100
files in the named pinned source
```

Feature coverage must be reported from the feature matrix, not inferred from
file count.

Do not combine PNG, JPEG, and BMP counts into one compatibility percentage.
Do not report local mutations as additional upstream coverage.

## Feature Matrix

The first matrix should track at least:

| Capability | PNG | JPEG | BMP |
| --- | --- | --- | --- |
| Header and bounds inspection | Proven | Proven | Proven |
| 8-bit color decode | Proven | Proven | Proven |
| Grayscale decode | Proven | Proven | Deferred |
| Alpha decode | Proven | Not applicable | Proven |
| Palette expansion | Proven | Not applicable | Synthetic evidence |
| Packed 1-, 2-, and 4-bit samples | Proven | Not applicable | Deferred |
| Top-down/bottom-up orientation | Not applicable | EXIF observed | Proven |
| Interlaced/progressive | Deferred | Deferred | Not applicable |
| Color metadata observation | sRGB/gAMA/iCCP observed | JFIF/EXIF/ICC observed | Limited |
| Exact pixel oracle | Proven | Provider-qualified | Proven |
| Tolerant differential oracle | Optional | Proven | Optional |
| Texture upload preparation | Pre-GPU proven | Pre-GPU proven | Pre-GPU proven |
| Native and WASM decode | Proven | Proven | Proven |

Unsupported entries should remain visible rather than being omitted.

## Performance Evidence

Record, but do not universally assert:

- source bytes;
- decoded bytes;
- compression ratio;
- inspection and decode time;
- conversion time;
- texture upload time;
- peak bounded allocation when measurable;
- artifact size.

Producers own measurements. Applications, corpus runners, and tools own budgets
and policy in accordance with `ADR-0007-kernel-performance-diagnostics.md`.

Ordinary correctness tests should not fail on wall-clock timing. Dedicated
benchmarks and sustained-budget diagnostics should observe performance trends.

## Implementation Slices

### Slice 0: Confirm Boundaries And Existing Evidence

Deliverables:

- [x] Inventory existing PNG fixtures, BMP export, renderer texture upload, and
      model-import image references.
- [x] Record decoding, image semantics, texture upload, export, and framebuffer
      capture as separate stages.
- [x] Identify the first concrete native and WASM consumers.
- [x] Record the current RGBA8+sRGB renderer constraint as execution evidence,
      not a universal image semantic.

Acceptance criteria:

- [x] The plan keeps codecs and GPU resources outside `tokimu-core`.
- [x] Existing BMP export is not described as BMP decoding.
- [x] Existing texture upload is not described as general raster support.
- [ ] At least two independent consumers justify any proposed shared image
      field.

### Slice 1: Acquire And Inventory Fixtures

Deliverables:

- [x] Define the primary source order and distinct correctness, regression,
      stress, and downstream-consumer roles.
- [x] Review candidate PNG, JPEG, and BMP sources and licenses. The PngSuite
      grant and the libjpeg-turbo JPEG/BMP license chain are preserved with
      their fixture provenance.
- [x] Pin revisions or archives and preserve provenance for the PNG source
      source by referencing its existing checksum-pinned W3C archive.
- [x] Build source inventories and `selection-v1.toml`.
- [x] Add a feature matrix and offline integrity verifier.
- [x] Select a small first PNG batch of 15 valid and 3 malformed cases.
- [x] Pin a small libjpeg-turbo source snapshot and select baseline,
      unsupported-profile JPEG, and BMP candidates.
- [ ] Expand the first JPEG selection with reviewed metadata and malformed
      source cases.
- [x] Admit redistribution-reviewed external BMP source cases for ordinary
      24-bit, odd-width row-padding, and non-four-multiple height pressure.
- [x] Exercise all three admitted 24-bit BMP source profiles in the bounded
      review runner.

Acceptance criteria:

- [x] Every selected source byte has a source and hash; the PngSuite license
      record admits redistribution while keeping the W3C archive distinct as
      the pinned distribution container.
- [x] Ordinary tests require no network access.
- [x] Upstream, selected, derived, synthetic, and executable counts remain
      separate.
- [x] Correctness, regression, stress, and downstream-consumer cases are
      reported separately.
- [x] No conformance claim is made from acquisition.

### Slice 2: Establish Provider-Neutral Image Evidence

Deliverables:

- [x] Define a corpus-owned decoded-image evidence model.
- [x] Define pixel format, color space, alpha mode, stride, and orientation
      vocabulary.
- [x] Define decode limits and structured diagnostics.
- [x] Add deterministic artifact serialization and pixel fingerprints.

Acceptance criteria:

- [x] The evidence model contains no PNG, JPEG, BMP, filesystem, browser, or
      renderer-native object.
- [x] Unsupported pixel interpretations stop explicitly.
- [x] Measurement and decoding can run without a window or GPU.
- [x] Repeated output is deterministic.

### Slice 3: Prove BMP Lossless Decoding

Deliverables:

- [x] Decode bounded 8-bit indexed, 24-bit, and 32-bit uncompressed BMP.
- [x] Handle row padding and top-down/bottom-up orientation explicitly.
- [x] Compare decoded pixels against independent expected buffers.
- [x] Exercise synthetic palette bounds, index failures, and row padding.
- [x] Add truncated, oversized, invalid-offset, and unsupported-compression
      cases.

Acceptance criteria:

- [x] Selected BMP cases produce exact pixel fingerprints.
- [x] Checked arithmetic prevents out-of-bounds rows and oversized allocation.
- [x] Output orientation and alpha policy are explicit.
- [x] The existing BMP writer is not the sole decode oracle.

### Slice 4: Prove PNG Lossless Decoding

Deliverables:

- [x] Decode bounded non-interlaced 1-, 2-, 4-, and 8-bit grayscale and
      indexed PNG plus 8-bit RGB, RGBA, and grayscale-alpha PNG.
- [x] Exercise filters, palette expansion, and `tRNS` with synthetic fixtures.
- [x] Observe `sRGB` metadata without claiming or performing color conversion.
- [x] Observe bounded `gAMA` and `iCCP` declarations without applying gamma or
      exposing a provider-native color-management object.
- [x] Add CRC, chunk-order, truncation, decompression-limit, and malformed
      palette cases.

Acceptance criteria:

- [x] Synthetic non-interlaced PNG cases produce exact, filter-independent
      pixel fingerprints. Selected PngSuite fixtures execute in place and
      produce deterministic decoded fingerprints under their preserved
      redistribution grant.
- [x] Illegal chunk and color-type combinations represented by the first
      profile fail at the PNG boundary.
- [x] Color and alpha interpretation are explicit.
- [x] Unknown ancillary metadata does not silently change decoded pixels.
- [x] Malformed or conflicting first-profile color metadata fails
      deterministically.

### Slice 5: Prove Baseline JPEG Decoding

Deliverables:

- [x] Decode bounded baseline 8-bit YCbCr JPEG through pinned `zune-jpeg`
      `0.5.15`.
- [x] Classify one-component grayscale and three-component YCbCr source frames
      before provider decoding without leaking raw component counts into the
      provider-neutral observation contract.
- [x] Add a reviewed baseline grayscale JPEG case from the separately pinned
      `jpeg-decoder` source selection.
- [x] Observe JFIF version/density, EXIF orientation, and ICC chunk
      completeness without applying presentation policy.
- [x] Define an independent differential oracle and tolerance policy.
- [x] Add malformed framing, truncation, source, dimension, decoded-allocation,
      progressive, arithmetic, and 12-bit rejection cases.
- [x] Exercise the admitted arithmetic and 12-bit JPEG profile rejections in
      the bounded review runner.

Acceptance criteria:

- [x] Selected baseline JPEG cases decode deterministically for the named
      `zune-jpeg` `0.5.15` provider profile.
- [x] Differential results identify `jpeg-decoder` `0.3.2`, normalized RGBA8
      assumptions, exact alpha/dimension checks, and reviewed maximum/mean RGB
      tolerances.
- [x] EXIF orientation behavior is explicit: the adapter reports the value and
      leaves pixel reorientation to a later, separately owned normalization
      policy.
- [x] Progressive, arithmetic, and 12-bit forms stop visibly.
- [x] CMYK is represented explicitly during inspection and rejected before
      provider decode. JFIF is covered by admitted upstream evidence; EXIF and
      ICC marker behavior currently use Tokimu-authored fixtures.

### Slice 6: Integrate Asset Identity And Dependencies

Deliverables:

- [x] Resolve one direct image asset through Tokimu asset identity.
- [x] Resolve the selected glTF external PNG dependency.
- [x] Preserve an FBX texture reference without giving FBX ownership of image
      semantics.
- [x] Define missing, duplicate, and unsupported dependency behavior at the
      current narrow resolver boundary; cyclic graph handling remains deferred
      to a future dependency resolver that owns traversal.

Acceptance criteria:

- [x] Missing image bytes fail at asset resolution, not rendering.
- [x] Importers exchange provider-neutral image identity or evidence for the
      selected glTF PNG dependency.
- [x] No source-format path leaks into the exercised asset-result contract;
      the source label is lifecycle provenance only.
- [x] Native and WASM consumers use the same semantic result for bounded PNG,
      baseline JPEG, and BMP decoded-image observations. This evidence covers
      the Rust/WASM API boundary, not browser-native pixel presentation.
- [x] The one-component grayscale JPEG emits the same pre-GPU
      texture-preparation and deterministic CPU-preview artifacts as the
      admitted YCbCr JPEG, PNG, and BMP cases.

### Slice 7: Integrate Texture Upload

Deliverables:

- [x] Define explicit conversion from decoded image to the current texture
      upload contract.
- [x] Record color/data and alpha decisions at conversion.
- [x] Emit deterministic pre-GPU texture-preparation evidence separately from
      decoded-image artifacts.
- [x] Preserve palette-derived alpha through deterministic CPU review export
      and re-decode without claiming GPU framebuffer equivalence.
- [ ] Exercise 1:1, filtered, UV-oriented, repeated, and alpha-blended cases.
- [ ] Emit upload and renderer artifacts separately.
- [x] Exercise the normalized color texture through renderer upload, a material
      texture slot, the renderer-owned default sampler, alpha blending, and
      `Texture2d` shader sampling without source-format leakage.

Acceptance criteria:

- [x] Upload preparation receives decoded pixels, not source image bytes.
- [x] The chosen GPU format is observable; sampler policy remains renderer
      work and is intentionally not inferred by this bridge.
- [ ] Decoder, upload, shader, and capture failures can be localized.
- [ ] Texture cache and residency remain renderer-owned.

### Slice 8: Add Corpus Runner And Visual Inspector

Deliverables:

- [x] Add a focused, bounded review runner inside `raster-image-corpus` for
      representative admitted PNG, JPEG, and BMP cases. It writes structural
      reports under `target/raster-image-corpus/review-v1` and does not scan
      directories or alter the fast unit-test selection.
- [x] Add filterable format, feature, expected-status, and expected-stage
      execution over the declared review cases only.
- [x] Emit structural reports and deterministic CPU previews with companion
      manifests that distinguish decoded-image export from GPU framebuffer
capture.

Each runner invocation replaces only the generated, corpus-owned case artifacts
under that review output root. This keeps a filtered execution from presenting
stale reports as current evidence; it never mutates upstream fixtures or
unrelated output files.
- [x] Add `hello-raster-image`, a small visual consumer corpus for manual
      texture-presentation evidence.

Acceptance criteria:

- [x] Headless execution remains authoritative for the runner's structural
      validation; the first review set has explicit expected decode/rejection
      outcomes.
- [x] Manual/native-window screenshots are explicitly non-authoritative;
      deterministic CPU exports and their manifests identify their own source
      stage without claiming framebuffer equivalence.
- [x] Add `hello-raster-image`, a bounded native visual inspector which consumes
      provider-neutral decoded images, explicitly prepares `ColorSrgb`
      textures, and presents five fixed known-decodable corpus fixtures without
      parsing source files inside renderer code.
- [x] The inspector exposes source and translucent inspection materials over
      the same uploaded texture and emits a structural shader-contract artifact
      without claiming GPU framebuffer equivalence.
- [x] The inspector does not redefine decoder semantics.
- [x] Large fixture sets do not run implicitly in the fast unit-test tier; the
      runner executes only its declared static review cases and tests that it
      does not discover inputs by directory search.

### Slice 9: Expand Color, Alpha, And Precision Pressure

Deliverables:

- [x] Prove that a legal PNG stream split across consecutive `IDAT` chunks
      decodes identically to its single-chunk equivalent.
- [ ] Add interlaced/progressive cases only after ordinary decode is stable.
- [ ] Add 16-bit, unusual alpha, and reviewed external
      paletted-BMP pressure.
- [ ] Add color-image versus data-texture cases.
- [ ] Add reviewed ICC and gamma observations.

Acceptance criteria:

- [x] PNG chunk partitioning does not alter decoded RGBA8 pixels, while
      nonconsecutive `IDAT` chunks continue to fail deterministically.
- [ ] Every new profile has a named consumer or diagnostic purpose.
- [ ] Metadata observation and pixel conversion remain distinct.
- [ ] Precision loss is explicit and testable.
- [ ] Unsupported profiles remain represented in reports.

### Slice 10: Harden Safety And Performance

Deliverables:

- [x] Prove that selected PNG, JPEG, and BMP inputs apply the same configured
      source-byte, declared-dimension, and normalized-RGBA output-byte budgets
      before provider decode/allocation.
- [x] Add deterministic real-fixture truncation smoke cases across PNG, JPEG,
      and BMP; each must reject without panicking at the bounded adapter
      boundary.
- [x] Add adversarial real-fixture dimension, layout, metadata framing, and
      compressed-payload smoke cases; each must reject without panicking at
      the bounded adapter boundary.
- [x] Add deterministic byte-mutation smoke probes at format headers,
      mid-stream data, and terminal bytes to verify parser boundaries cannot
      panic when source bytes change unexpectedly.
- [x] Record per-case decode duration in review artifacts without making
      correctness depend on hardware or wall-clock budgets.
- [ ] Fuzz parser and decoder boundaries where practical.
- [ ] Add decode and upload benchmarks.
- [ ] Integrate bounded performance observations without universal timing
      assertions.

Acceptance criteria:

- [x] The three admitted raster adapters reject source, dimension, and decoded
      output budget violations deterministically at their bounded decode
      boundary.
- [x] Representative truncated PNG, JPEG, and BMP sources cannot turn a
      malformed-input probe into a panic or silent successful decode.
- [x] Representative oversized dimensions, invalid BMP layout, and malformed
      PNG payloads reject before becoming allocation or provider failures.
- [ ] Malformed inputs cannot panic, overrun, or allocate beyond policy.
- [ ] Performance reports identify the owning decode, conversion, or upload
      stage.
- [ ] Correctness tests remain deterministic and hardware-independent.
- [ ] GPU-dependent evidence is separately labeled and invocable.

### Slice 11: Architectural Review

Deliverables:

- [ ] Review whether provider-neutral image semantics have at least two
      independent consumers.
- [ ] Review whether image semantics belong with assets, presentation, or a
      separate capability.
- [ ] Review whether format-provider traits have stabilized through repeated
      use.
- [ ] Record accepted, deferred, and rejected findings.

Acceptance criteria:

- [ ] No crate extraction occurs solely because several codecs share a helper.
- [ ] Source formats, image semantics, and renderer execution retain distinct
      ownership.
- [ ] Corpus-owned duplicate logic is identified before promotion.
- [ ] Any admission updates the SDD and relevant ADRs deliberately.

## Highest-Return Priority

The recommended order is:

1. Expand baseline JPEG with reviewed metadata-bearing and malformed cases.
2. Resolve `BoxTextured`'s external PNG through asset identity.
4. Upload one decoded texture with explicit sRGB/data and alpha policy.
5. Add baseline JPEG tolerant independent comparison.
6. Expand BMP only when new cases add DIB, palette, bitfield, or malformed
   pressure beyond the admitted V1 evidence.
7. Exercise the same contract from native and WASM consumers.
8. Add libpng and image-rs regression/stress cases without merging their
   counts into format conformance.
10. Expand malformed-input, color, precision, and performance pressure.

This order produces useful application evidence early without asking the GPU to
validate the decoder.

## Edge-Case Backlog

After the first profiles are stable, consider:

- Adam7 PNG;
- progressive JPEG;
- 16-bit PNG;
- BMP bitfields and RLE;
- CMYK and YCCK JPEG;
- malformed ICC and EXIF metadata;
- extreme aspect ratios;
- zero and maximum legal dimensions;
- duplicate and conflicting metadata;
- palette alpha corner cases;
- premultiplication round trips;
- mip generation and gamma-aware filtering;
- normal, mask, height, and signed data textures;
- animated raster formats;
- GPU-compressed texture containers;
- renderer framebuffer capture comparison.

These items should be admitted in small named batches, not enabled as one
"support all images" milestone.

## Validation Commands

Commands will be added as implementation begins. The intended shape is:

```powershell
pwsh -NoProfile -File .\scripts\verify-raster-image-corpus.ps1
cargo test -p raster-image-corpus
cargo run -p hello-raster-image
```

These names are planning placeholders, not admitted package commitments.

## Risks And Mitigations

### RGBA8 Becomes Universal Meaning

Mitigation: keep pixel format, color, alpha, stride, and orientation explicit
in corpus evidence even when the first execution path supports only RGBA8.

### Renderer Becomes The Decoder

Mitigation: require headless decoded-image artifacts before texture upload.

### One Library Defines Tokimu Semantics

Mitigation: keep provider-native objects below the decoder boundary and compare
at least two provider or independent oracle paths where useful.

### Browser Decoding Hides WASM Gaps

Mitigation: browser-native decoding may be differential evidence, but the
consumer corpus must visibly identify which implementation produced pixels.

### Color Errors Look Like Decoder Errors

Mitigation: record metadata interpretation, conversion, upload format, shader,
and capture stages independently.

### Lossy Comparison Produces Fragile Tests

Mitigation: use named reference decoders, explicit comparison color space,
reviewed tolerances, and structural assertions in addition to pixels.

### Fixture Volume Produces False Coverage

Mitigation: report named source and feature denominators; expand in small,
diagnostic batches.

### Image Bombs Exhaust Memory

Mitigation: validate dimensions and checked decoded size before allocation and
enforce decompression and metadata limits.

## Completion Criteria

This corpus plan reaches its first useful completion point when:

- PNG, BMP, and baseline JPEG selections are pinned and verifiable offline;
- every selected case has provenance, purpose, expected stage, and status;
- lossless PNG and BMP cases have independent exact pixel evidence;
- JPEG cases have an explicit independent tolerant comparison policy;
- color, alpha, stride, and orientation are observable;
- malformed and oversized inputs fail deterministically;
- one direct image asset and one model-referenced image use asset identity;
- one decoded image reaches texture upload without source-format leakage;
- native and WASM execution produce the same provider-neutral observations;
- reports separate decode, conversion, upload, and render evidence.

## Graduation Criteria

A shared Tokimu image capability should be considered only when:

- at least two independent consumers depend on the same provider-neutral image
  semantics;
- PNG, JPEG, and BMP providers preserve those semantics without leaking
  provider-native types;
- headless decoding and measurement remain independent of renderer startup;
- asset dependency resolution and renderer upload consume the contract without
  redefining it;
- corpus applications no longer duplicate decoded-image validation logic;
- the SDD and Architectural Review evidence identify stable ownership;
- promotion removes repeated ownership ambiguity rather than merely moving
  files.

Until then, raster-image work should incubate in corpus and importer support
code.

## References

- `docs/Tokimu Software Design Document.md`
- `docs/testing-strategy.md`
- `docs/Libraries/khronos-gltf-corpus-testing.md`
- `docs/Libraries/fbx-corpus-testing.md`
- `docs/Libraries/cgm-corpus-testing.md`
- `corpus/assets/README.md`
- `corpus/lib/screenshot/DESIGN.md`
- `crates/tokimu-render/src/texture.rs`
- `docs/ADR/ADR-0001-engine-boundaries.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0007-kernel-performance-diagnostics.md`
