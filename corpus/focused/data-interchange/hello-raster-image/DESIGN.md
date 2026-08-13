# Hello Raster Image

## Purpose

`hello-raster-image` is a focused native consumer of the raster-image corpus.
It proves that bounded PNG, JPEG, and BMP providers can supply normalized
RGBA8 pixels to the existing texture pipeline without passing source-format
objects or source paths into renderer contracts.

## Primary Proof

```text
fixture bytes
    -> bounded decoder
    -> provider-neutral DecodedImage
    -> explicit ColorSrgb texture preparation
    -> renderer-owned GPU texture
    -> material texture slot + renderer-owned sampler
    -> Texture2d shader sampling
    -> native presentation
```

Arrow keys cycle a small, fixed set of known-decodable fixtures. Space switches
between faithful source-color sampling and a translucent cyan inspection
material. Both modes sample the same immutable uploaded texture; presentation
changes through material data rather than pixel mutation.

The example writes
`target/hello-raster-image/raster-shader-contract.json`. The artifact records
each pre-GPU texture-preparation artifact plus the material slot, sampler,
pipeline, blend, orientation, and shader-sampling contracts. It explicitly
does not claim GPU framebuffer capture. The native window remains manual visual
evidence; structural runner artifacts remain authoritative for decoder
validation.

## Non-Goals

- General filesystem browsing or image loading.
- A production texture cache or residency policy.
- User-selectable sampler controls, mip generation, image editing, or color
  conversion.
- GPU framebuffer capture or decoder conformance claims.
