# Hello Texture Color Space

## Purpose

`hello-texture-color-space` is a focused presentation corpus that proves the
renderer allocates RGBA8 textures with an explicit sampled interpretation.

The entry submits identical encoded RGBA8 bytes twice. One texture is declared
as linear UNORM data and the other as sRGB UNORM color. The default texture
pipeline samples both without knowing where the bytes came from.

## Primary Proof

```text
identical application-owned RGBA8 bytes
        |
        +-- linear descriptor --> Rgba8Unorm texture --> left material
        |
        +-- sRGB descriptor ----> Rgba8UnormSrgb texture -> right material
```

The visual difference is manual native-window evidence of backend
interpretation. The structural proof is the explicit descriptor passed to each
allocation and the renderer test that maps it to the corresponding wgpu
format. The package still compiles for `wasm32-unknown-unknown`, but does not
claim browser visual execution until a browser host exercises the same scene.

Final display policy remains outside this corpus. It does not choose monitor,
browser, HDR, Spout, OBS, or transfer-function policy; a consumer must make
those output decisions explicitly.

Native screenshots belong to the separately labeled
[`manual/hello-texture-color-space`](../lib/screenshot/manual/hello-texture-color-space/README.md)
evidence path. They are manual observations, not framebuffer captures or
cross-platform color guarantees.

## Non-Goals

- image decoding;
- color-management policy beyond linear versus sRGB RGBA8;
- HDR, ICC profiles, or transfer-function selection;
- texture mutation, mipmaps, or sampler policy.
