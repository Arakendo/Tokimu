# Projection And Picking Conformance Manifest

| Field | Retained value |
| --- | --- |
| Review pressure | AR-0028 Slice 5 |
| Scope | CPU projection, picking-ray reconstruction, and WGPU clip-boundary comparison |
| Status | Incubating corpus evidence; no public projection/picking contract admitted |
| Camera position | `[0, 0, -6]` |
| Camera forward / up / right | `+Z / +Y / -X` |
| Projection | right-handed GL-style perspective, 60° vertical FOV, near `0.1`, far `100` |
| CPU NDC depth | `[-1, 1]` |
| WGPU uploaded depth | `[0, 1]`, converted once inside the WGPU backend |

## Shared Matrix Path

Native and browser camera fixtures now obtain their view and projection from
the same `camera_conformance_matrices` helper used by the CPU projection and
picking suite. The renderer receives an ordinary `Camera`; the WGPU backend
privately converts only the uploaded view-projection matrix.

```text
pose + declared basis
    -> GL-style view/projection
        +-> CPU project / unproject in [-1,1]
        `-> renderer Camera
              -> WGPU upload converts depth once to [0,1]
```

The existing WGPU regression maps Tokimu clip depths `-1`, `0`, and `+1` to
`0`, `0.5`, and `1` without mutating the caller's camera. Its exact endpoint
assertions fail if the conversion is removed or applied twice.

## Landmark Projection

The six non-Doom landmarks retain these structural expectations from the
initial pose:

| Landmark | Expected observation |
| --- | --- |
| world `+X` | screen-left (`ndc.x < 0`) because camera right is `-X` |
| world `-X` | screen-right (`ndc.x > 0`) |
| world `+Y` | screen-up (`ndc.y > 0`) |
| world `-Y` | screen-down (`ndc.y < 0`) |
| world `-Z` | centered and nearer |
| world `+Z` | centered and farther |

All observed depths remain inside the declared GL interval. For every
landmark, `picking_ray_from_ndc` unprojects the projected screen point at GL
near/far depths and the resulting world ray returns to the same labeled world
center within `0.0002` units. A zero/non-invertible matrix is rejected rather
than fabricating a ray.

## Known Falsification Pressure

- The existing `hello-cad` oblique view now projects its model center, rebuilds
  a picking ray through the projected point, and derives screen-right from its
  own inverse view matrix. It does not import the first-person fixture's
  initial world `-X` shortcut. A future orbit-drag gesture must still declare
  its interaction policy separately from this geometric right direction.
- Stereo eyes should share world meaning while carrying distinct view
  matrices and projection centers.
- Reflected views may reverse presentation orientation; compensation must be
  declared rather than inferred from a matrix determinant.
- Portal-derived views may make one world object visible through multiple
  view instances.
- AR-0026 chart transitions can relocate or reverse a local frame without one
  global Euclidean embedding.

Two theoretical chart-transition specimens are retained for later
falsification:

```text
orientation-preserving specimen
    local rotation + translation
    same labeled handedness after an explicitly declared transition

orientation-reversing specimen
    local reflection + translation
    reversed handedness is explicit transition meaning
```

These are vocabulary pressure only. A positive or negative raw `Mat4`
determinant is an observation about one representation, not sufficient
semantic authority to classify a future chart transition.

## Validation

```powershell
cargo test -p render-orientation-conformance
cargo test -p tokimu-render `
  wgpu_camera_upload_converts_tokimu_clip_depth_without_changing_camera
cargo check -p hello-render-orientation-web --target wasm32-unknown-unknown
```
