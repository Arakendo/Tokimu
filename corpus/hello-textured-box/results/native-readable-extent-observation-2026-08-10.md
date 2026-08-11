# Native Readable-Extent Observation — 2026-08-10

| Field | Observation |
| --- | --- |
| Target | Native Windows WGPU |
| Backend / adapter | Vulkan / AMD Radeon RX 7900 XTX |
| Geometry | Pinned Khronos `Box.glb` |
| Texture | First-party Tokimu PNG fixture set |
| Default extent | One complete `[0,1]` planar image per face |
| Stress extent | Explicit `E` toggle to `3.25x` addressing input |
| Status | Manually observed; accepted as the Slice 2 independent UV control |

The maintainer observed the revised default with a complete image mapped to
each Box face. The texture remained readable enough to distinguish orientation
and continued rendering through the existing `Textured3d` contract.

Cycling `X` intentionally demonstrated three caller-owned mappings:

- identity retains the corpus planar mapping;
- flip-U makes readable texture content horizontally backward;
- swap-UV rotates/transposes the texture axes.

Those differences are expected evidence. The renderer presented the supplied
UV stream without inferring which mapping the caller intended. `E` retains the
earlier out-of-range addressing stress as a separate explicit mode rather than
making repeated tiling the default visual control.

This observation does not claim that the corpus planar mapping is a universal
cube-unwrapping convention or that every face must make arbitrary text upright
from every camera pose.
