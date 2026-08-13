# Post-DOOM Math Operation And Boundary Inventory

| Field | Value |
| --- | --- |
| Status | Second-stage Slice 1 operation manifest 0.2 |
| Date | 2026-08-12 |
| Repository revision | `c84108cd2eabe2dbe13b658f4f493f996ca33d74` |
| Baseline | `glam` 0.29.3 at `d36e7eeff05338c56c4aa8d59fc2615e7963b1b7` |
| Previous manifest | `operation-inventory.md` (2026-08-07 manifest 0.1) |
| Scope | Stable crates plus current Doom, renderer, GLB, CAD, camera, collision, picking, animation, and orientation corpus paths |

This is a caller-pressure inventory, not a proposed public API. It names the
mechanics exercised after the Doom work and separates them from source-space,
frame, visibility, collision, or provider semantics. An existing foreign API
spelling is migration evidence, not automatic admission into Option C.

The scan used direct `tokimu_core::math`/`tokimu::math` imports and associated
operation searches across `crates`, `corpus`, and `tests`, excluding foreign
source. Thirty-three Rust files import the stable math surface, including the
study itself; 21 non-study files currently consume it directly.

## Stable And Provider Boundaries

| Boundary | Current fact | Consequence for Option C |
| --- | --- | --- |
| `tokimu_core::math` | Publicly re-exports `Mat4`, `Quat`, `Vec2`, `Vec3`, and `Vec4` | A migration still has five public foreign names even though only three have direct caller pressure |
| `tokimu` facade | Re-exports `tokimu_core`, making the math module available as `tokimu::math` | Public-vocabulary migration is workspace/facade-visible |
| `tokimu_render::Camera` | Public `view` and `projection` fields are `Mat4` | This remains the largest stable provider-vocabulary seam |
| WGPU camera upload | Builds an explicit clip-depth conversion and uploads `[[f32; 4]; 4]` | Provider handoff is already scalar/column data; GPU representation need not enter owned math |
| WGPU uniform/resource structs | Renderer-owned structs, not math values, implement `Pod`/`Zeroable` | No evidence currently requires owned math types to promise POD/ABI layout |
| Corpus crates | Exercise public math but are architecture-driving, unpublished evidence | Their pressure is real; their exact convenience API is not automatically stable |
| Doom TypeScript workbench | Rust/WASM engine uses `Mat4`/`Vec3`; TypeScript receives bounded observations and presentation controls | No TypeScript-facing math layout or serialization contract is demonstrated |

No scanned math caller derives serialization/reflection/POD traits on the
public math values. No production crate imports `glam` directly outside
`tokimu-core`; the foreign identity enters through the public re-export.

## Caller Pressure After Doom

| Area | Values | Mechanics | Frequency/access shape | Ownership classification |
| --- | --- | --- | --- | --- |
| `tokimu-render::Camera` | `Mat4`, `Vec3` | identity, orthographic/perspective construction, right-handed view | Per resize/camera update; public mutable fields | Ordinary camera mechanics; camera meaning belongs above raw math |
| WGPU adapter | `Mat4`, `Vec4` | explicit columns, multiplication, 2D column-array upload, projected-depth test | Per camera upload/test; read/convert | Provider conversion; not a device-backed math provider |
| E1M1 static observer | `Mat4`, `Vec3`, `Vec4` | camera, point/direction vectors, length/dot/cross, clip projection, AABB corners, ray/triangle tests | Per input/frame/query; thousands of prepared surfaces | Ordinary mechanics plus separate Doom-owned spatial/visibility semantics |
| E1M1 collision | `Vec3` | source/world lift/lower, point/delta construction | Per movement step; mutable observer state | Doom collision policy above raw vectors |
| E1M1 AR-0025 trials | `Mat4`, `Vec3`, `Vec4` | AABB/sphere classification, clip coordinates, grid selection, SEG/BSP evidence | Bulk candidate scans over current prepared scene | Bulk/source-specific experiment; not ordinary type ownership |
| Orientation conformance | `Mat4`, `Vec3`, `Vec4` | signed axes, lengths, dot/cross, inverse projection, picking rays | Test and interactive conformance | Frame/handedness meaning is AR-0028 vocabulary above math |
| `hello-cad` | `Mat4`, `Vec3`, `Vec4` | model transform, inverse view-projection, perspective divide, ray/AABB, normal transform | Interactive per edit/pick over one bounded model | Ordinary mechanics; no large-assembly bulk pressure yet |
| GLB/hole-punch | `Mat4`, `Vec3`, `Vec4` | decoded columns, hierarchy composition, animation lerp, vertex/normal transforms, bounds | Load/update over bounded imported scenes | Caller-owned import/animation semantics; loops may become batch-shaped but are not a provider contract |
| FPS and presentation corpus | `Mat4`, `Vec3` | camera, movement, transform construction | Per input/frame | Ordinary mechanics |
| Doom browser workbench | `Mat4`, `Vec3` inside Rust/WASM | fixed/source cameras, sums, normalized owning-side observations | User-triggered browser presentation | Native/WASM pressure without a TypeScript math ABI |

The Doom checklist still contains 71 open items at this scan. Its operations
are therefore strong corpus pressure but provisional as a final replacement
manifest. In particular, animation, gameplay objects, more moving sectors, and
later presentation work can still change the measured set.

## Refreshed Candidate Mechanics

### Vec3

The original C0 mechanics remain exercised: construction, `ZERO`/`ONE`/`Y`,
array conversion, public components, arithmetic, component/scalar multiply and
divide, normalization, squared length, distance, dot/cross, min/max, lerp, and
homogeneous extension.

New or newly direct pressure since manifest 0.1:

| Mechanic | Caller evidence | Candidate implication |
| --- | --- | --- |
| Positive/negative axis basis values | Doom and orientation conformance use `X`, `Z`, `NEG_X`, `NEG_Y`, and `NEG_Z` | Raw basis values are earned mechanics; source/chart/frame meaning remains separate |
| Euclidean length | Doom frustum/screen and orientation tests call `length` | Add only with selected finite/non-finite behavior and property evidence |
| Summation | Doom centroid/normal evidence uses `sum::<Vec3>()` | Accumulation is required; the exact `Sum` trait surface can be retained as migration friction rather than copied automatically |

### Vec4

C0 construction, public components, array conversion, and truncation remain
exercised. The WGPU boundary now uses `Vec4::X` and `Vec4::Y` to describe an
explicit clip-depth conversion matrix. Explicit column vectors are required;
copying those convenience constants is optional if the selected owned surface
can express the same bounded construction clearly.

### Mat4

The original C0 mechanics remain exercised: identity, right-handed GL-depth
view/projection construction, translation/scale/axis rotation, column-array
input/output, multiplication, transpose/inverse, point/vector transforms, and
explicit final-column mutation.

New or newly direct pressure since manifest 0.1:

| Mechanic | Caller evidence | Candidate implication |
| --- | --- | --- |
| Construct from four columns | WGPU clip-depth conversion uses `from_cols` | C0 already has equivalent `from_columns`; foreign spelling is not required |
| Convert to/from 2D column arrays | WGPU uniform upload and its test | Explicit scalar boundary is earned; exact nested-array API remains selectable |
| Perspective-dividing point projection | WGPU depth test and alpha-policy corpus use `project_point3` | Semantic operation is earned and needs explicit `w == 0`/non-finite behavior |
| All-zero matrix | Orientation invalid-picking test uses `Mat4::ZERO` | A deterministic invalid control is required; a public zero constant is not independently earned |

## Still-Unpressured Public Types

`Vec2` and `Quat` have no Rust caller outside the math study in the refreshed
direct-import scan. WGSL `vec2<f32>` tokens are shader-language types and do not
create Rust `Vec2` pressure. Application-local 2D vector/rotation types likewise
do not earn the stable re-export.

Option C therefore continues to omit `Vec2` and `Quat`. This makes C0 an
incomplete replacement for A's current five-name public vocabulary and keeps
public compatibility/migration as an explicit blocker.

## Semantic Types Remain Above Mechanics

The scan reinforces rather than weakens AR-0026/AR-0028 separation:

- a `Vec3` does not identify a Doom source frame, Tokimu world frame, chart,
  handedness, orientation-preserving transition, position, direction, or
  normal;
- a `Mat4` being finite/invertible does not state whether a transformation
  intentionally preserves or reverses orientation;
- Doom embedding, sidedef ownership, BSP traversal, collision membership, and
  sky/visibility rules remain Doom semantics;
- camera screen-right, yaw policy, picking, and source correspondence require
  framed tests above vector arithmetic.

Option C may provide mechanics used by those models. It must not absorb their
semantic identities merely to make the math API appear complete.

## Public Boundary And Migration Inventory

| Concern | Current evidence | Status |
| --- | --- | --- |
| Stable public fields | `Camera::{view, projection}: Mat4` | Real migration seam |
| Direct provider identity | Five public re-exports equal `glam` types | Real foreign-vocabulary coupling |
| Renderer conversion | Matrix becomes renderer-owned nested scalar array | Explicit and provider-neutralizable |
| Layout/alignment | Earlier native/WASM artifacts observe differences | Observation only; no stable ABI |
| Serialization/reflection | No direct current math-value contract found | Not earned |
| FFI/POD | Renderer uniforms are separate POD structs | Math POD promise not earned |
| TypeScript | No TS-visible Rust math objects/layout | Not earned |
| Browser/WASM | Rust-side camera/math executes in browser workbench | Target/caller pressure, not ABI pressure |

## Candidate Bulk Classification

| Work | Current scale/shape | Classification now |
| --- | --- | --- |
| E1M1 bounds/frustum classification | Approximately two thousand prepared draws, repeated by view | Real bulk-shaped CPU control related to AR-0025; Doom visibility rules remain source-owned |
| E1M1 uniform-grid/BSP/SEG/clip trials | Viewer-relative selection with identity/order constraints | Source-specific comparative evidence, not a generic math operation |
| E1M1 picking/collision | Small query sets per input/update | Ordinary CPU query mechanics; GPU ineligible at current pressure |
| GLB vertex/normal transforms | Loops over bounded imported meshes | Potential batch shape, but no independent performance/provider deficit yet |
| Current CAD cube transforms/picking | One bounded model and ray/AABB test | Ordinary CPU negative control, not large-CAD evidence |
| Proposed CAD assembly/point cloud | No implemented caller in this scan | Independent bulk pressure remains unearned and must not be inferred |

No GPU/provider work is authorized by this inventory. Slice 7 must still choose
at most two bounded operations after numerical contracts and C0 correctness are
settled.

## Manifest 0.2 Disposition

- Keep C0 limited to `Vec3`, `Vec4`, and `Mat4`.
- Treat axis values, vector length, accumulation, explicit column boundaries,
  and perspective-dividing projection as new mechanics to evaluate.
- Do not copy exact foreign spellings when an existing owned mechanic or an
  explicit adapter satisfies the caller.
- Keep Doom/CAD bulk hypotheses separate from ordinary owned values.
- Revisit the manifest when the still-open Doom checklist materially changes
  object, animation, collision, or presentation pressure.
