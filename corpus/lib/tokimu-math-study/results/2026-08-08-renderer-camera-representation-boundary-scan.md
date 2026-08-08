# Renderer Matrix Representation-Boundary Scan

| Field | Value |
| --- | --- |
| Status | Current source-boundary evidence; not a migration implementation |
| Date | 2026-08-08 |
| Scope | `tokimu-render` WGPU camera and instance upload paths |
| Question | Do current GPU upload paths require `Mat4`'s in-memory alignment? |

## Observed Boundary

`Camera` stores `tokimu_core::math::Mat4` values for `view` and `projection`.
Before either WGPU upload path writes a camera uniform, it performs:

```text
(camera.projection * camera.view).to_cols_array_2d()
    -> GpuCameraUniform { view_projection: [[f32; 4]; 4] }
    -> bytemuck::bytes_of(&uniform)
```

`GpuCameraUniform` is a renderer-local `#[repr(C)]`, `Pod`, and `Zeroable`
type. The camera matrix itself is not cast directly to bytes or directly used
as the GPU uniform representation.

The current instance path likewise has no `Mat4` byte representation:
`GpuInstanceUniform` carries scalar `[f32; 2]` translation, scale, and
rotation fields plus padding. Both present and render-target passes construct
that uniform from the `Instance2d` semantic fields before byte upload.

## Finding

C's observed 4-byte `Mat4` alignment is **not an immediate incompatibility
with the current WGPU upload paths**: the renderer already owns explicit
scalar-array/scalar-field conversion boundaries. It remains an incompatibility
with any proposed direct A/B-compatible `Mat4` interchange, FFI, SIMD, or
upload claim. The current result validates boundary separation, not zero
migration cost or a stable C representation.

## Reopening Triggers

- A renderer path begins to upload/cast `Mat4` directly.
- A public FFI, serialization, SIMD, or GPU-layout promise exposes `Mat4`.
- A C migration proposes removing or changing the scalar-array adapter.
