# Current Renderer Camera Public-Vocabulary Surface Scan

## Scope

This is a source observation for AR-0019 Slice 6. It counts only the current
Tokimu renderer `Camera` matrix vocabulary outside the math study. It does not
propose an API change or classify every two-dimensional camera constructor as
3D math pressure.

## Observed API

`crates/tokimu-render/src/camera.rs` declares:

```rust
pub struct Camera {
    pub view: Mat4,
    pub projection: Mat4,
}
```

`Camera::new` accepts the same two values. At the current boundary those are
the A/provider `tokimu_core::math::Mat4` values.

## Direct 3D Field Writers

The following eight corpus source files directly assign the public `view`
field with a `Mat4`:

- `corpus/hello-3d-mono/src/main.rs`
- `corpus/hello-3d-stereo/src/main.rs` (one write per eye)
- `corpus/hello-audio-visualizer/src/main.rs`
- `corpus/hello-cad/src/main.rs`
- `corpus/hello-fps-web/src/main.rs`
- `corpus/hello-glb/src/main.rs`
- `corpus/hello-hole-punch/src/main.rs`
- `corpus/hello-shader/src/main.rs`

The first six contain distinct already-retained 3D/camera pressure except the
shader and audio-visualizer identity-view cases, which remain deliberately
unported because they add no operation beyond existing fixtures. Stereo is an
additional public-field-writer shape, but has not yet been made a candidate
fixture.

The wider constructor scan also finds orthographic camera construction in
2D corpus examples. That is `Camera` API exposure, but this observation does
not infer a `Mat4`-operation requirement from every such call.

## Consequence

The B/C `renderer_camera` fixtures prove that the current public renderer can
be reached through explicit private conversions. They do not migrate the
public `Camera` vocabulary. A later accepted ownership decision must choose
between retaining provider matrices at this renderer boundary, changing the
facade to Tokimu-owned matrices, or adding a different renderer-facing camera
contract. Any such work must remeasure these direct writers and the WGPU
adapter boundary.

## Reproducibility

The observation used source searches for the `Camera` definition plus direct
`camera.view`, `camera.projection`, `left_camera.view`, `right_camera.view`,
and `target_camera.view` use outside this study.
