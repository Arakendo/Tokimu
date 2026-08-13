# Native Slice 4 Interaction Observation — 2026-08-09

## Capture Identity

| Field | Value |
| --- | --- |
| Target | native WGPU window |
| Backend | Vulkan |
| Device | AMD Radeon RX 7900 XTX discrete GPU |
| Viewport | 960 × 600 |
| Source build identity | `c5473d17c45145c6df378858a79141a02d083f0f` + uncommitted Slice 4 corpus work |
| Command | `cargo run -p hello-alpha-policy --features native-visual --bin native_interaction_scene` |
| Fixtures | manifest-locked `binary-mask` and `mixed-alpha` RGBA8 sources |
| Interaction manifest | `0a99c714c258bac7f91eb5dd39748651abca8db96bfc1a410d823a18d2c23d93` |
| Candidate states | opaque depth-write; corpus-local cutout `< 128/255` depth-write; straight-alpha Blend with depth-write off or on |
| Geometry | fixed cutout plane at `z=0.5`; sloped blended plane from `z=0.6` to `z=0.3`; opaque backing at `z=0.0` |

## Observed Result

The fixture presented successfully with:

```text
7 draws, 7 material resolutions, 7 pipeline switches,
manifest=0a99c714c258bac7f91eb5dd39748651abca8db96bfc1a410d823a18d2c23d93,
diagnostic=none
```

The upper-left cutout-over-opaque panel exposed the blue backing through the
binary mask's transparent texels. The upper-right Blend-over-opaque panel
showed continuous mixed-alpha contribution. The lower panel visibly combined
the binary cutout with the sloped depth-crossing blended surface; its bands
change where the two planes exchange depth ordering.

## Interpretation Boundary

This is one native AMD/Vulkan observation of the bounded Slice 4 fixture. It
does not establish browser parity, NVIDIA behavior, general transparent
intersection handling, renderer-owned sorting, batching, or a stable material
contract.
