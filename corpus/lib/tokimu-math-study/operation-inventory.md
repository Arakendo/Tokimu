# Initial Math Operation Inventory

> Historical manifest 0.1. The active second-stage caller scan is
> `operation-inventory-post-doom.md`; this file remains frozen so the study can
> distinguish pre-DOOM and post-DOOM pressure.

| Field | Value |
| --- | --- |
| Status | Slice 1 initial source-scan evidence; frozen as operation manifest 0.1 |
| Date | 2026-08-07 |
| Scope | Current direct imports from `tokimu_core::math` in engine and corpus source |
| Baseline | `glam` 0.29.3 at `d36e7eeff05338c56c4aa8d59fc2615e7963b1b7` |

This inventory is a bounded starting point for the case study, not a claim that
these operations are a stable Tokimu contract. It records observable caller
pressure so Alternatives B, C, and D do not copy a broad foreign API merely
because five type names are publicly re-exported.

Any change to this manifest must identify the caller or conformance case that
requires it and must be reviewed before it becomes available to B, C, or D.

## Importing Callers

| Area | Imported types | Observed role |
| --- | --- | --- |
| `tokimu-render::Camera` | `Mat4`, `Vec3` | View and projection construction; public camera fields |
| `hello-3d-mono`, `hello-3d-stereo` | `Mat4`, `Vec3` | Camera and transformed mesh evidence |
| `hello-fps-web` | `Mat4`, `Vec3` | First-person camera, motion, and mesh transforms |
| `hello-cad` | `Mat4`, `Vec3`, `Vec4` | Object transforms, picking ray, normal transforms |
| `hello-glb`, `hello-hole-punch` | `Mat4`, `Vec3`, `Vec4` | Imported-scene transforms, animation interpolation, clipping |
| `hello-shader`, `hello-audio-visualizer` | `Mat4`, `Vec3` | Presentation transforms |

The retained presentation-caller scan finds only identity-equivalent camera
view assignments through `Mat4::from_translation` and `Vec3::new`/`ZERO`.
Those paths add no distinct operation pressure beyond the existing fixtures;
see `results/2026-08-08-presentation-caller-pressure-scan.md`.

`Vec2` and `Quat` have no current direct `tokimu_core::math` import in the
source scan. They remain in the study because they are presently public
re-exports, but their candidate operation set begins empty until caller or
compatibility evidence earns one.

The refreshed 2026-08-08 scan also found application-local 2D vector types in
corpus consumers. Those are not evidence to absorb their API into Ring 0 or to
claim that they depend on the current `Vec2` re-export; see
`unpressured-public-types.md`.

## Initial Required Operations

| Type | Observed requirements |
| --- | --- |
| `Vec3` | `new`, `ZERO`, `ONE`, `Y`, `splat`, `from_array`; public `x/y/z`; add and add-assign, subtract, negate, component/scalar multiply and divide; `normalize`, `normalize_or_zero`, `length_squared`, `distance`, `cross`, `min`, `max`, `lerp`, `extend`. `dot` is admitted as a named shared-conformance primitive, not yet as a direct caller requirement. |
| `Vec4` | `new`; public `w`; `truncate`; homogeneous matrix multiplication result handling |
| `Mat4` | `IDENTITY`, `look_at_rh`, `perspective_rh_gl`, `orthographic_rh_gl`, `from_translation`, `from_scale`, `from_rotation_x/y/z`, `from_cols_array`; matrix multiplication including `Mat4 * Vec4`; `inverse`, `transpose`, `transform_point3`, `transform_vector3`; writable `w_axis` |
| `Vec2` | No direct current caller requirement beyond retained public re-export. Alternative B retains only name, `ZERO`, construction, and array observation as a compatibility probe; this does not admit general vector operations. |
| `Quat` | No direct current caller requirement beyond retained public re-export. Alternative B retains only name, `IDENTITY`, construction, and array observation as a compatibility probe; this does not admit composition or rotation operations. |

## Semantic Questions To Resolve Before Candidate Implementation

- **Observed matrix representation:** the study's A/B/C transform cases use
  column-major `Mat4` data and column-vector multiplication. `from_cols_array`
  and `w_axis` observations refer to this layout. This remains provider and
  experiment evidence, not a stable representation guarantee.
- **Observed affine transforms:** `transform_point3` treats input as `w = 1`
  and `transform_vector3` as `w = 0`; neither performs a perspective divide.
  Projection handling therefore remains a distinct operation concern.
- **Observed coordinate conventions:** `look_at_rh` produces a right-handed
  view with `+X` right, `+Y` up, and `+Z` back; `perspective_rh_gl` and
  `orthographic_rh_gl` use the OpenGL `[-1, 1]` depth convention. These are
  conformance observations, not yet declared Tokimu guarantees.
- **Observed units:** rotation constructors and perspective vertical FOV take
  radians. No current caller evidence supports a second angle-unit API.
- Normalization behavior for zero and non-finite values differs between
  `normalize` (NaN components) and `normalize_or_zero` (zero fallback);
  candidates must preserve or explicitly revise behavior only after a reviewed
  decision.
- `Mat4::inverse` behavior for singular matrices remains deliberately
  unresolved as a Tokimu contract. Alternative C currently returns all-NaN
  output for a singular matrix as visible provisional experiment behavior.
- The shared conformance suite includes four fixed non-singular affine inverse
  round trips combining translation, X/Y rotation, non-uniform scale, and
  varied point ranges. This is required evidence for current caller-shaped
  inversion pressure; it is neither an exhaustive numeric proof nor a choice
  of behavior for singular or non-finite matrices.
- A fixed-seed, 96-case affine differential sweep extends that evidence across
  bounded non-singular transforms and compares B/C against A at `1e-3` point
  tolerance. It excludes zero/near-zero scale, singular, and non-finite input;
  those remain separate behaviour and recovery questions.
- A fixed-seed, 128-case finite camera/projection differential sweep compares
  `perspective_rh_gl * look_at_rh` A/B/C matrices at `1e-4` tolerance. It keeps
  eye and target distinct, uses a nonparallel-up construction, and requires
  finite positive aspect plus `near < far`; it is camera-path conformance
  evidence rather than a promise about degenerate or non-finite inputs.
- Direct writable matrix-column semantics remain a migration concern: B/C use
  `w_axis()` plus `set_w_axis(...)`, rather than claiming that a public field
  shape is a Tokimu contract.
- `Vec3` layout and `Mat4::w_axis` mutability are current provider facts; they
  must not be copied into a Tokimu representation claim without measurement.

## Excluded Until Earned

- Swizzles, SIMD internals, generated source, wide trait families, and all
  unrelated `glam` vector/matrix types.
- Serialization, FFI/POD, reflection, GPU-buffer, and TypeScript layout
  guarantees.
- Any operation seen only in an application-local vector type rather than a
  `tokimu_core::math` import.

## Evidence Commands

```powershell
rg -n --glob '*.rs' "use tokimu_core::math" crates corpus tests
rg -n --glob '*.rs' "\b(Mat4|Vec3|Vec4)::|\.(inverse|transpose|transform_point3|transform_vector3|normalize|normalize_or_zero|cross|lerp|extend|min|max|truncate)\b" crates corpus tests
cargo test -p tokimu-math-study --locked --offline
```
