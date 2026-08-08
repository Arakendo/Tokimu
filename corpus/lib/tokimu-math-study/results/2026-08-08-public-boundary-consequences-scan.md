# Public Math Boundary-Consequences Scan

| Field | Value |
| --- | --- |
| Status | Current-source inventory; not a stable migration implementation |
| Date | 2026-08-08 |
| Scope | Current Rust public, persistence, renderer upload, WASM, and TypeScript-authoring surfaces |
| Question | Which current public or cross-domain boundaries would actually constrain a future replacement of `glam` vocabulary? |

## Observed Boundaries

| Boundary | Current observation | Consequence for B/C |
| --- | --- | --- |
| Stable core vocabulary | `tokimu_core::math` directly re-exports `glam::{Mat4, Quat, Vec2, Vec3, Vec4}`. | Replacing the stable vocabulary is a deliberate API migration, not an internal implementation swap. |
| Renderer API | `tokimu_render::Camera` publicly stores `view: Mat4` and `projection: Mat4`. | This is the concrete current provider-vocabulary seam; the corpus B/C camera helpers model it but do not migrate it. |
| GPU upload | Both WGPU camera paths convert the composed camera matrix with `to_cols_array_2d()` into renderer-local `GpuCameraUniform { view_projection: [[f32; 4]; 4] }` before `bytemuck` byte upload. | C's observed 4-byte matrix alignment is not an immediate incompatibility with current upload paths. The scalar-array adapter remains required evidence, not a stable C layout promise. |
| Persistence | `tokimu::persistence` is generic over `serde` documents, and its current test type contains only scalar/string fields. No current math type is serialized by that facade. | There is no demonstrated math serialization migration yet; any future serialized math schema requires separate compatibility and recovery evidence. |
| Native FFI/layout | The scanned core, facade, platform, renderer, and TypeScript-frontend Rust surfaces contain no public math FFI declaration or C-layout math contract. Renderer-local `repr(C)` uniforms do not expose `Mat4`. | No current FFI compatibility claim blocks the study. A future FFI boundary must not infer compatibility from A/B/C's observed Rust layout. |
| WASM | The complete study crate compiles for `wasm32-unknown-unknown`; isolated A/B/C stereo math executes in Node WASM. Browser/WGPU execution has not been measured. | Compile and isolated-execution evidence are positive but incomplete; browser renderer behavior remains open. |
| TypeScript authoring | `tokimu-ts-frontend` currently models rule-lowering data and has no math-type import or public math schema. | There is no demonstrated authoring-vocabulary migration. Future authoring models need their own provider-neutral contract decision. |

## Finding

The current architecture already separates GPU byte representation from the public `Mat4` representation, which prevents an immediate upload-layout blocker for B or C. It does **not** erase the public `Camera` migration seam, or establish serialization, FFI, browser/WGPU, downstream-publication, or authoring compatibility for a new Tokimu vocabulary.

This inventory makes the public-boundary checklist concrete: ownership of meaning, public vocabulary, executing implementation, and transport representation must be assessed independently. The absent boundaries are not automatically safe; they are unclaimed until a real caller introduces them.

## Reopening Triggers

- A stable `Camera`, math, serialization, FFI, GPU, browser, or authoring API exposes a candidate representation.
- A renderer upload path casts a math type directly rather than using its explicit scalar-array boundary.
- DOOM or another corpus path introduces a material transform, scene, asset, or authoring caller that changes this inventory.
