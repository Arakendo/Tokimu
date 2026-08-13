# Option B API Ergonomics, Documentation, And Ecosystem Pressure

| Field | Value |
| --- | --- |
| Date | 2026-08-12 |
| Evidence | representative caller ports, current stable/corpus source scan, generated Rustdoc |
| Production migration | none |

## Caller Readability And Discovery

Narrow B changes only construction intent:

```rust
let view = view_look_at_rh(eye, target, Vec3::Y)?;
let projection = projection_perspective_rh_gl(fov, aspect, near, far)?;
```

The function names expose right-handed and GL-depth meaning without exposing
the provider's `camera::rh::view` or `camera::rh::proj::opengl` organization.
The returned `Result` makes rejection visible rather than inheriting unchecked
provider behavior. Representative stereo, CAD, Doom observer, renderer
transport, and stored orthographic camera source remains ordinary provider
value code after construction.

Full B's common finite caller source is intentionally familiar:

```rust
let transform = Mat4::from_translation(position)
    * Mat4::from_rotation_y(angle)
    * Mat4::from_scale(scale);
let point = transform.transform_point3(point);
```

That familiarity has a cost: readers cannot tell from a familiar method name
whether Tokimu owns its edge semantics or merely delegates them. Slice 9's
`min`/`max` observation demonstrates the distinction. The wrapper therefore
needs its own documentation for every admitted method rather than relying on
visual similarity to `glam`.

## Surface And Documentation Accounting

| Candidate | Public surface observation |
| --- | --- |
| Narrow B | two retained provider value names; three semantic functions; four error/result declarations |
| Full B | eight top-level type/error declarations; 60 inherent public constants/functions; 10 arithmetic trait implementations |

Normal Rustdoc generation succeeds for both candidates. A deliberate
`-D missing-docs` gate fails both experimental surfaces. Narrow B has 16
missing documentation items concentrated in its operations, failures, and
three functions. Full B has a much larger missing set spanning error variants,
constructors, constants, accessors, transforms, and checked/unchecked policy.

This is not repaired inside the study because stable admission and final API
wording are not authorized. It is retained maintenance evidence: Narrow B has
a small, semantically focused documentation bill; Full B must duplicate or
independently specify a broad portion of upstream behavior without pretending
that upstream documentation is Tokimu's contract.

## Trait And Source-Compatibility Pressure

Both Full-B values derive only the traits deliberately exercised by the
candidate (`Clone`, `Copy`, `Debug`, `PartialEq`; errors also use `Eq`). The
pressured arithmetic surface includes `Vec3` add/add-assign/subtract/negate,
scalar and component multiply/divide, plus `Mat4 * Mat4` and `Mat4 * Vec4`.

A current non-study scan found no demonstrated requirement for math-value
`Default`, indexing, generic `From`/`Into`, serde, bytemuck/POD, hashing, or
ordering. Those requests remain unsupported rather than being copied from the
provider speculatively.

The same scan did find production-corpus pressure that the bounded Full-B ports
do not yet satisfy:

- two Doom paths use iterator `Sum<Vec3>`;
- FPS and other callers read and mutate public `.x/.y/.z` fields;
- hole-punch mutates a public matrix column; and
- current callers gain many provider traits and methods transitively through A.

Full B currently offers component getters and one bounded final-column setter,
but no component setters or `Sum`. These are honest migration gaps. They are
not added now because doing so would broaden the candidate solely for source
compatibility. Narrow B retains the provider values and therefore retains all
of this foreign ergonomics—including the accidental API commitment risk.

## Ecosystem Boundary Review

| Boundary | Narrow B | Full B |
| --- | --- | --- |
| renderer `Camera` | unchanged provider `Mat4` values; semantic construction changes only | stable public camera value types would change; nine representative scalar-column crossings retained |
| GPU upload | existing renderer-owned scalar columns | explicit allocation-free scalar-column conversion; no ABI/POD claim |
| assets/mesh data | no change; current mesh transport is scalar arrays | no need to introduce wrappers into asset byte/array formats |
| ECS/world state | no new storage model | owned values can be stored by value, but no independent ECS caller earns migration |
| serialization/reflection | provider `serde` is not selected and no contract is demonstrated | no serde/reflection/FFI layout is admitted |
| TypeScript authoring/lowering | no TypeScript-visible math change | scalar semantic models remain the boundary; wrapper layout must not leak into TS |
| external/provider tools | foreign values remain publicly coupled | must cross explicitly through scalar arrays or reviewed adapters |

The renderer is the largest stable compatibility consequence for Full B:
`tokimu_render::Camera` currently exposes provider `Mat4` values. Narrow B
leaves that commitment intact while containing construction vocabulary. Full B
would intentionally replace it and therefore needs a separate migration and
semver decision rather than an internal refactor.

## Disposition

Narrow B improves discoverability of the demonstrated semantic intent and
keeps its documentation burden small, but continues the broad foreign-value
ergonomics and semantics of A. Full B remains understandable as a bounded
subset and works in representative ports, but it is neither source-compatible
with all real callers nor permitted to grow into a disguised `glam` clone.

The real `Sum`, public-component mutation, and strict-documentation gaps are
retained as evidence against premature Full-B admission. No unsupported trait,
field, conversion, serialization, or tooling contract was added. Slice 10
therefore strengthens Narrow B's proportional case without selecting or
stabilizing it.
