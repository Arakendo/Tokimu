# Option B Slice 11: Spatial And Renderer Cross-Review

| Field | Value |
| --- | --- |
| Date | 2026-08-12 |
| Status | Complete bounded Slice 11 cross-review evidence |
| Scope | AR-0026 chart meaning, AR-0028 orientation ownership, AR-0029 camera construction, and renderer clip adaptation |
| Non-claim | No chart, frame, portal, recursive-view, input, or renderer API is admitted |

## Fixed Chart Control

The retained three-chart control keeps `ChartId`, transition order, qualified
local meaning, and orientation declarations outside every numerical candidate.
The same native control was executed for production A, provider-backed Full B,
and owned C0:

```text
alternative=A;  fingerprint=2520c9de
alternative=B;  fingerprint=2520c9de
alternative=C0; fingerprint=2520c9de
```

All three restore `[2.0, 0.0, -1.0]`, classify rigid composition as
`Preserving`, and classify the explicit reflection as `Reversing`. The three
focused chart tests pass. The earlier DOM/WASM A/C0 observation with the same
fingerprint remains inherited evidence; this Slice does not relabel it as a
new B browser execution.

Narrow B does not replace the five provider value types or ordinary
`Vec3`/`Mat4` mechanics. Its chart mechanics are therefore exactly A by
identity; only the three checked view/projection constructors differ. The
Narrow-B representative suite, including stereo and Doom observer
construction, passes unchanged under exact provider pins 0.29.3 and 0.33.3.
Adding a second chart implementation merely to spell that identity separately
would duplicate AR-0026 semantics inside the adapter study.

## Ownership Ledger

| Meaning | Owner retained by the evidence | Math-provider role |
| --- | --- | --- |
| chart identity and qualified location | AR-0026 semantic fixture / future spatial model | carry local scalar coordinates only |
| transition intent and orientation behavior | authored AR-0026 semantics | compose, invert, and transport ordinary values |
| Doom source embedding | Doom provider under AR-0028 | carry the converted coordinates |
| camera basis | Tokimu camera construction question in AR-0029 | construct the requested right-handed matrix |
| normalized mouse/keyboard policy | application/input boundary under AR-0028 | no authority |
| active camera, viewport, and per-draw selection | renderer/application presentation boundary | no authority |
| WGPU clip-depth conversion | private WGPU upload adapter | provider-private scalar matrix conversion |

No chart ID, frame role, source convention, input sign, viewport, camera
lifecycle, or provider clip-space fact became a Full-B method.

## Clip-Depth Boundary

The SDD continues to define Tokimu camera projection as GL-style `[-1, 1]`.
`Camera::perspective_3d` and `Camera::orthographic_2d_with_height` construct
that meaning. `wgpu_camera_uniform`, private to `tokimu-render`'s WGPU backend,
then maps `-1/0/1` to `0/0.5/1` while leaving the source `Camera` unchanged.

The focused test passed:

```text
cargo test -p tokimu-render \
  wgpu_camera_upload_converts_tokimu_clip_depth_without_changing_camera \
  --locked --offline
```

Neither B candidate moves that conversion into public math or caller code.

## Multi-View And Future View Pressure

Two independent current callers already show that more than one view does not
require broader raw math:

- `hello-3d-stereo` constructs two checked perspective/view pairs, uploads two
  camera handles, and preserves explicit per-eye viewports;
- `hello-cad` uses separate scene and overlay cameras.

Narrow B's retained stereo caller passes under both provider pins. Full B's
existing stereo migration uses only its already inventoried ordinary
construction, vector, and matrix operations. Multiple view instances therefore
pressure camera identity, viewport, submission, and lifecycle ownership rather
than additional `Vec3`/`Mat4` vocabulary.

AR-0026 still lists one portal-derived local view as future evidence and
explicitly rejects a current recursive-rendering claim. No portal or recursive
view caller exists in this study. Such a caller may require a qualified-view
semantic layer or a revised camera decomposition; that question returns to
AR-0029/AR-0026 and is not permission to expand Narrow B or Full B.

## Operation-Growth Result

The cross-review requested **zero new ordinary math operations** from either B
or C0. It reused the existing bounded manifest:

- vector construction, add/subtract, dot, cross, normalize, and array
  observation;
- matrix translation, Y rotation, scale, composition, inverse, point
  transport, and direction transport; and
- the already demonstrated checked look-at and perspective/orthographic
  constructors where a camera caller actually needs them.

This is evidence that richer spatial meaning can remain above boring numerical
mechanics. It is not evidence that portals or recursive views are solved, nor
does it select A, Narrow B, Full B, or C for production.

## Validation

- A/Full-B/C0 chart tests: 3 passed.
- Native chart observer: identical `2520c9de` fingerprints.
- Narrow-B representative tests: 4 passed under 0.29.3 and 4 passed under
  0.33.3 with unchanged caller source.
- WGPU clip conversion: focused renderer test passed.
- Existing provider 0.29.3 warning flood remains a known Option-A input and is
  not attributed to this cross-review.

## Disposition

Complete Slice 11. Exotic spatial meaning remains outside ordinary math;
Narrow B remains exactly the three-family semantic seam; Full B remains a
bounded ordinary-math candidate. The next work is the Slice 12 comparative
decision matrix and explicit maintainer gate. No stable/public change is
authorized by this result.
