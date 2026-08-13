# Alternative C0 Experimental Numerical Contract

| Field | Value |
| --- | --- |
| Status | Selected for corpus implementation/testing; not a stable Tokimu API |
| Date | 2026-08-12 |
| Applies to | `Vec3`, `Vec4`, and `Mat4` mechanics in Alternative C0 |
| Caller manifest | `operation-inventory-post-doom.md` |
| Governing plan | `docs/Plans/Native-Math/Studies/ar-0019-option-c-owned-math-and-bulk-compute.md` |

This contract makes the C0 experiment falsifiable without promoting the
current provider's edge behavior into Tokimu policy. It separates raw IEEE-754
mechanics from checked construction/query paths that can reject invalid caller
or external data before it becomes trusted state.

The exact Rust method names of future checked operations remain corpus-local.
The semantic outcomes below are selected; a stable API shape is not.

## Ownership And Representation

- Scalar components are `f32`.
- Vectors and matrices are ordinary CPU values. They have no GPU residency,
  provider, device, synchronization, or allocation semantics.
- Matrices use column-vector multiplication. `matrix * vector` transforms the
  vector, and `a * b` applies `b` before `a`.
- Matrix column order is observable through explicit scalar-array conversion
  because current renderer/importer boundaries require it.
- Size, alignment, field offsets, ABI, POD, FFI, serialization, and
  TypeScript/JavaScript representation are not guaranteed.
- `Vec3` and `Mat4` do not carry frame, chart, source-space, handedness,
  orientation, position/direction/normal, or unit meaning. Those semantics
  belong to caller-owned types and contracts above ordinary math.
- Generic matrix mechanics are handedness-neutral. Named view/projection
  constructors state the convention they construct; C0 currently needs
  right-handed view and OpenGL `[-1, 1]` clip-depth constructors.

## Failure Classes

| Class | Example | Experimental rule |
| --- | --- | --- |
| Raw IEEE arithmetic | Component add/multiply or scalar division by zero | Use Rust `f32` behavior; NaN/infinity may propagate; do not panic or silently clamp |
| Programmer precondition | Calling unchecked normalize with a zero/non-finite vector | Unchecked result is not a recovery contract; caller must use checked/bounded semantics when invalid input is possible |
| Recoverable invalid data | Singular imported transform, degenerate camera, invalid projection parameters | Checked candidate path returns rejection (`None` or a later bounded error); prior valid caller state is unchanged |
| Ill-conditioned finite input | Near-singular matrix or nearly parallel view/up vectors | Outside the selected guarantee until a condition/residual threshold is justified; do not label a numerically unstable value valid merely because it is finite |
| Internal invariant violation | A checked path accepts input but returns a non-finite result for an in-domain case | Test failure/defect; not converted to a plausible fallback value |

Math values do not own diagnostic retention. The caller that validates
external/imported data owns source identity and can convert a bounded rejection
into its structured diagnostic. C0 must expose enough checked outcome to avoid
panic-based or all-NaN control flow.

## Vector Contracts

### Raw component mechanics

Construction, array conversion, component access, add/subtract/negate,
component/scalar multiply, scalar divide, dot, cross, min/max, lerp, truncate,
and extend operate component-wise under `f32` arithmetic. They allocate no
memory and do not inspect semantic frames.

`min`/`max` NaN behavior is not selected beyond Rust's `f32::min`/`max`
mechanics in this experiment. Callers requiring rejection of non-finite bounds
must validate before candidate selection.

### Length and normalization

- `length_squared(v)` is `dot(v, v)`.
- `length(v)` is `sqrt(length_squared(v))`.
- Successful normalization requires finite components and a positive finite
  length whose reciprocal and normalized components are finite.
- The checked normalization outcome rejects zero, subnormal/overflow cases
  that cannot produce a finite unit vector, NaN, and infinity.
- A zero-tolerant convenience may return zero for an exactly zero finite
  vector when the caller explicitly requests that semantic.
- Non-finite input must not be silently converted to zero by the selected
  checked contract. The current provider/C0 observation that
  `normalize_or_zero` does so is not adopted as general recovery policy.
- Unchecked `normalize` remains useful only when its finite/nonzero precondition
  is established by the caller or a preceding checked operation.

### Accumulation

Accumulation follows deterministic caller iteration order. This contract does
not promise reassociation, parallel reduction, compensated summation, or
bit-identical results under reordered inputs. A future bulk provider must
declare ordering/precision separately.

## Matrix Contracts

### Construction and transforms

- Translation affects points (`w = 1`) and not direction vectors (`w = 0`).
- `transform_point3` and `transform_vector3` are affine-style operations and do
  not perform a perspective divide.
- Perspective-dividing point projection is a distinct checked mechanic. It
  rejects non-finite homogeneous output and zero/non-finite `w`; otherwise it
  returns `xyz / w` when all result components are finite.
- Axis rotation inputs are radians.
- Explicit column-array input/output is a semantic conversion boundary, not an
  ABI or memory-cast promise.

### View construction

A checked right-handed look-at construction succeeds only when:

- eye, center, and up components are finite;
- `center - eye` can produce a finite normalized forward vector;
- up can produce a finite normalized vector; and
- forward and up produce a nonzero finite side vector.

Coincident eye/center and parallel/zero/non-finite up are rejected. Behavior
for nearly parallel but technically normalizable inputs remains outside the
selected guarantee until conditioning evidence establishes a threshold.
Unchecked all-NaN output for degenerate input is retained only as provider
observation, not as the selected failure path.

### Projection construction

A checked right-handed GL-depth perspective construction requires finite:

```text
0 < vertical_fov_radians < PI
aspect_ratio > 0
0 < near < far
```

A checked orthographic construction requires finite bounds with:

```text
left < right
bottom < top
near < far
```

Invalid parameters are recoverable rejection. They do not panic, silently
substitute defaults, or produce a matrix represented as valid. Applications
may choose defaults before calling the checked mechanic, but that is
application policy.

### Inversion

- The selected successful domain is finite, non-singular, sufficiently
  conditioned matrices exercised by current affine/camera callers.
- A checked inverse rejects a non-finite matrix, an exact singular pivot, a
  non-finite elimination result, or a result that fails a bounded identity
  residual check selected by the conformance suite.
- Near-singular acceptance is deliberately not fixed by an arbitrary global
  determinant epsilon in this slice. Property/conditioning evidence must
  justify any threshold.
- The current two-sided `1e-3` identity residual remains a conservative C0
  observation: for the retained translated/rotated affine fixture it accepts
  smallest scale `1e-2` and rejects `1e-3` through `1e-10`. This is not a
  general condition-number guarantee; later workload evidence may revise it.
- Singular inversion returning an all-NaN matrix matches current A/C
  observation but is not selected as recoverable failure behavior.
- Unchecked inverse, while retained for A comparison, has a finite,
  sufficiently conditioned precondition and cannot be the boundary used for
  untrusted imported transforms.

## Comparison And Test Tolerances

Tolerances are test/workload bounds, not runtime equality semantics. For a
scalar comparison use:

```text
abs(actual - expected) <= max(absolute_floor,
                              relative_scale * max(abs(actual), abs(expected)))
```

Initial bounded values:

| Operation family | Absolute floor | Relative scale | Existing evidence domain |
| --- | ---: | ---: | --- |
| Basic vector mechanics and unit-length checks | `1e-6` | `2e-6` | Components generally within `[-1e3, 1e3]` |
| Fixed affine inverse round trips | `3e-5` | `3e-5` | Four hand-selected, non-singular caller-shaped transforms |
| Fixed-seed affine differential sweep | `1e-3` | `1e-5` | 96 transforms; scale magnitudes `[0.25, 3.25]`, translation/points within `[-1e3, 1e3]` |
| Finite camera/projection matrices | `1e-4` | `1e-5` | 128 fixed-seed valid camera/projection cases |
| Perspective-dividing projection | `1e-5` | `1e-5` | Finite homogeneous results with nonzero `w`; expansion required in Slice 3 |

The existing absolute-only assertions may be retained as historical evidence,
but Slice 3 should implement the combined comparator for the selected
contract. A mismatch is classified by operation and conditioning; A is an
oracle, not automatically the correct policy.

## Provider Observation Disposition

| Current observation | C0 contract disposition |
| --- | --- |
| Zero `normalize` produces non-finite components | Retain as unchecked IEEE observation; not a recovery guarantee |
| Zero `normalize_or_zero` returns zero | Adopt only for explicitly zero-tolerant finite input |
| Non-finite `normalize_or_zero` returns zero | Do not adopt as checked recovery behavior |
| Degenerate `look_at_rh` produces non-finite matrix entries | Replace at external/caller boundaries with checked rejection |
| Singular `inverse` produces all-NaN matrix | Replace at external/caller boundaries with checked rejection |
| Invalid perspective parameters produce non-finite/unstable values | Replace with checked rejection |
| Finite valid camera/affine cases match A | Required conformance within bounded tolerances |

## Slice 3 Implementation Requirements

The next slice must:

1. add checked corpus-local outcomes for normalization, look-at, projection,
   inversion, and perspective-dividing projection;
2. retain unchecked methods only for comparison/current-caller migration and
   document their preconditions;
3. add the post-DOOM earned mechanics without importing broad provider API;
4. add success, rejection, fixed-seed property/metamorphic, and mismatch
   classification evidence; and
5. keep all operations dependency-free, allocation-free, safe Rust, and free
   of generated code.

Any later stable proposal must review the error/API shape separately. This
document selects experiment behavior, not final names or public compatibility.
