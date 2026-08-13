# Native Slice 2 Cutout Observation — 2026-08-09

## Capture Identity

| Field | Value |
| --- | --- |
| Target | native WGPU window |
| Backend | Vulkan |
| Device kind | discrete GPU |
| Adapter | AMD Radeon RX 7900 XTX |
| Viewport | 960 × 600 |
| Corpus command | `cargo run -p hello-alpha-policy --features native-visual --bin native_scene` |
| Fixture source | manifest-locked `mixed-alpha` and `binary-mask` exact RGBA8 arrays |
| Candidate threshold | `128/255` (`0.5019608`) |
| Declared depth state | `LessEqual`, depth write enabled |

This is a retained visual observation, not a pixel-golden artifact or a public
renderer contract.

## Observed Result

The top row rendered the identical five-texel `mixed-alpha` source three
times, left to right:

1. explicit opaque control: all five texels were visible, including the
   zero-alpha texel;
2. corpus-local `discard below 128/255`: the 0 and 64 alpha texels were
   absent while the exact 128 alpha texel remained visible;
3. corpus-local `discard at or below 128/255`: the 0, 64, and exact 128 alpha
   texels were absent.

The lower binary-mask panel showed retained source texels over the foreground
and the opaque blue backing where the cutout shader discarded pixels. This is
consistent with the headless assertion that a discarded fragment writes neither
color nor depth.

## Interpretation Boundary

The observation establishes that the existing custom-WGSL provider mechanism
can distinguish the two candidate comparisons on the native target. It does
not choose either comparison as a stable Tokimu default, establish a generic
cutout API, or establish browser parity. Those questions remain in AR-0023.
