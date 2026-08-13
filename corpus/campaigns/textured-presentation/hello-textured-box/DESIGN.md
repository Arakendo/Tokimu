# Hello Textured Box - Corpus Design

| Field | Value |
| --- | --- |
| Status | Native and browser/WASM consumers implemented, manually observed, and retained as renderer-boundary evidence |
| Purpose | Exercise generic supplied mesh UVs, texture sampling, and alpha-policy boundaries with independently selected Box geometry and PNG texture inputs |
| Governing review | [AR-0022 |
| Geometry source | Pinned Khronos `Box.glb` decoded through `gltf-corpus` |
| Texture source | First-party `corpus/assets/PNG` fixtures decoded through the raster-image boundary |
| Non-goal | glTF material import, PNG-aware renderer behavior, or Doom presentation |

## Boundary Assertion

This corpus application intentionally combines two independent
inputs:

```text
Box.glb -> corpus GLB decoder -> Tokimu-owned positions/normals/indices
PNG     -> raster decoder      -> normalized RGBA8 pixels
                                    |
                                    v
                         generic mesh/material/sampler contract
```

The selected PNG is not a texture declared by `Box.glb`. The renderer must not
receive a GLB, PNG bytes, source path, decoder object, WAD type, or Doom
semantic value. The application owns the selected texture, supplied UV values,
sampler declaration, alpha declaration, camera, and retained observation.

## Required Future Modes

| Mode | Input difference | Required conclusion |
| --- | --- | --- |
| UV orientation | Door/axis fixture and supplied UV mapping | A reviewer can identify a U/V inversion or face rotation. |
| Addressing | Default UVs map one complete image per face; `E` explicitly selects a `3.25` stress extent for clamp versus repeat | The material declaration, not source format, selects visible edge smear or tiling behavior. |
| Filtering | Fine grid at a non-integer screen scale, point versus linear | The material declaration reaches the backend sampler. `R` cycles all four filter/address combinations independently. |
| Palette variation | Matching dark and green door inputs | Texture identity may change without changing source geometry or UVs. |
| Alpha | A future documented first-party alpha fixture | The renderer behavior is declared explicitly, or returns an explicit unsupported result. |

## Deliberate Deferrals

- The initial 78 first-party PNG fixtures have no `tRNS` chunk and no intrinsic
  grayscale-alpha/RGBA color type. They are not an alpha execution case.
- The initial corpus will not create a cutout alpha contract merely to make a
  checkbox green. A small first-party alpha fixture needs its own documented
  intent and the unresolved AR-0023 alpha/depth policy.
- A Box GLB material import is unrelated to this test. The separate Khronos
  `BoxTextured` source remains source-format evidence, not an implementation
  shortcut for this consumer.

## Interactive Native Controls

- `M`: cycle grid, dark-door, and green-door inputs.
- `R`: cycle point/clamp, point/repeat, linear/clamp, and linear/repeat.
- `X`: cycle identity, U-flipped, and U/V-swapped corpus UV mappings.

The controls change only their named corpus input. Geometry and decoded pixel
data otherwise remain stable, so visual changes remain attributable.

## Implemented Scope

- `Mesh` carries optional caller-supplied UVs whose length must match positions.
- `Textured3d` requires those UVs and has a separate 3D shader/pipeline from
  the existing derived-coordinate 2D texture path.
- Material sampling is declared as point/linear and clamp/repeat; it is not a
  global WGPU default.
- The native consumer uses a fixed three-face camera. This keeps `M`, `R`, and
  `X` comparisons attributable to their named input rather than camera motion.
- The initial texture profile is opaque. Alpha/cutout remains explicitly out
  of scope until a documented source fixture and threshold policy exist.

The browser consumer has the same narrowly scoped source conversion and uses
the asynchronous WebGPU readiness pattern derived from AR-0021. Successful
WASM compilation/package generation is intentionally not recorded as browser
rendering evidence.

## References

- [Fixture Manifest](fixture-manifest.md)
- [Textured Box GLB And PNG Corpus Plan
- [Hello GLB Design
- [Raster Image Corpus Testing
