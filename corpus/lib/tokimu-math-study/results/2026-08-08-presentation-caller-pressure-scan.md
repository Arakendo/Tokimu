# Presentation Caller-Pressure Scan

| Field | Value |
| --- | --- |
| Status | Retained negative evidence; no new candidate migration |
| Date | 2026-08-08 |
| Scope | `corpus/hello-shader/src/main.rs`; `corpus/hello-audio-visualizer/src/main.rs` |
| Method | Direct source scan for imported math types and constructor/method use |

## Observed Uses

| Caller | Observed use | Distinct operation pressure |
| --- | --- | --- |
| `hello-shader` | `Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0))` assigned to an orthographic camera view | Translation construction and `Vec3::new` only |
| `hello-audio-visualizer` | Four `Mat4::from_translation(Vec3::ZERO)` assignments to orthographic camera views | Translation construction and `Vec3::ZERO` only |

The scan found seven textual occurrences each of `Mat4` and `Vec3`, including
imports and type context. It found no `Vec4`, matrix inversion, composition,
projection construction, matrix-vector multiplication, or component-access
pressure in either caller.

## Disposition

Do not add a separate A/B/C migration fixture for either caller now. The shared
and existing corpus fixtures already exercise `Mat4::from_translation`,
`Vec3::new`, and `Vec3::ZERO` while adding stronger transform, inverse, scene,
and adapter-boundary evidence. Reopen this finding if either caller acquires a
distinct math operation or a renderer migration makes its upload boundary
observable.
