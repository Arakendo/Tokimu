# Option B Narrow Semantic-Construction Contract

| Field | Value |
| --- | --- |
| Status | Slice 2 contract; isolated study only |
| Candidate | Narrow B |
| Provider vocabulary | private; no `glam::camera` in the contract |
| Value vocabulary | current Tokimu `Vec3` and `Mat4` names |
| Clip depth | GL-style `[-1, 1]` |

## Owned Meaning

Narrow B owns exactly three pure construction operations:

1. a right-handed look-at view with `-Z` forward;
2. a right-handed perspective projection with Y-up output and GL depth; and
3. a right-handed orthographic projection with Y-up output and GL depth.

These operations do not own camera lifecycle, selected-camera state, input,
viewport resize policy, world or chart identity, Doom embedding, matrix
storage, GPU uniforms, or the WGPU `[-1, 1]` to `[0, 1]` conversion.

## Success Contract

### View

`view_look_at_rh(eye, target, up)` maps `eye` to the origin and a target in
front of the observer onto the negative view-space Z axis. The supplied up
direction establishes the Y orientation; it need not be unit length. The
result must contain only finite values.

### Perspective

`projection_perspective_rh_gl(vertical_fov, aspect, near, far)` accepts radians
and maps view-space `z = -near` to NDC Z `-1` and `z = -far` to NDC Z `+1`.
It produces Y-up output. The result must contain only finite values.

### Orthographic

`projection_orthographic_rh_gl(left, right, bottom, top, near, far)` maps the
declared X/Y extents to NDC `[-1, 1]` and the right-handed view-space near/far
planes to GL NDC Z `[-1, 1]`. The result must contain only finite values.

## Rejection Contract

Every failure returns a bounded provider-neutral category carrying the owning
operation. No input is allowed to reach a provider panic as ordinary control
flow.

| Category | Required cause |
| --- | --- |
| `NonFiniteInput` | any scalar or vector component is NaN or infinite |
| `DegenerateView` | eye equals target, up has zero length, or up is collinear with the view direction |
| `InvalidFrustum` | FOV is outside `(0, PI)`, aspect is not positive, near is not positive for perspective, or an ordered extent/depth interval is empty or reversed |
| `NonFiniteResult` | validated input still produces a non-finite provider result |

Perspective requires `0 < near < far`. Orthographic permits either sign for
near and far but requires `near < far`, matching the existing 2D camera use.
The contract does not silently repair, clamp, substitute defaults, or reorder
invalid inputs.

The degenerate-view classification uses exact zero/collinearity controls plus
finite-result validation. It does not introduce a hidden near-degenerate
epsilon that could reject an otherwise finite camera. Near-degenerate cases
remain explicit differential inputs and may be promoted only with new caller
evidence.

## Independent Checks And Tolerances

Provider comparison alone is insufficient. Tests must include scalar controls:

- eye and target mapping for view construction;
- orthonormal basis dot products and handedness sign;
- near/far NDC depth mapping;
- left/right/bottom/top NDC extent mapping; and
- finite-value scans over all 16 matrix scalars.

For ordinary values, scalar and matrix comparisons use
`abs_error <= 1e-5 + 1e-5 * abs(expected)`. Exact categorical rejection is
required. Large-scale and near-degenerate controls may state a wider local
tolerance, but cannot silently change the common contract.

## Representation Non-Guarantees

Narrow B does not guarantee provider module names, row/column storage, SIMD
layout, alignment, ABI, POD behavior, serialization, or a particular internal
formula. Matrix multiplication and vector convention retain Tokimu's current
public `Mat4` behavior; the construction seam does not redefine them.

## Claim Limit

This document defines an executable experimental contract. It is not a stable
API admission, an AR-0029 disposition, or authorization to update production.
