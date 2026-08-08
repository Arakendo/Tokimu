# Native A/B/C Layout Observation

| Field | Value |
| --- | --- |
| Status | Current compiler/target observation; not a stable ABI claim |
| Date | 2026-08-08 |
| Target | `x86_64-pc-windows-msvc` |
| Rust | `rustc 1.95.0 (59807616e 2026-04-14)` |
| Method | `observe_layouts` executable using `size_of` and `align_of` |

## Output

| Candidate | Type | Size | Alignment |
| --- | --- | ---: | ---: |
| A | `Vec2` | 8 | 4 |
| A | `Vec3` | 12 | 4 |
| A | `Vec4` | 16 | 16 |
| A | `Quat` | 16 | 16 |
| A | `Mat4` | 64 | 16 |
| B | `Vec2` | 8 | 4 |
| B | `Vec3` | 12 | 4 |
| B | `Vec4` | 16 | 16 |
| B | `Quat` | 16 | 16 |
| B | `Mat4` | 64 | 16 |
| C | `Vec3` | 12 | 4 |
| C | `Vec4` | 16 | 4 |
| C | `Mat4` | 64 | 4 |
| D | `Vec3` | 12 | 4 |

## Finding

B preserves the currently observed provider layout for every retained type. C
keeps the same observed sizes for its implemented types but not the 16-byte
alignment of A/B `Vec4` and `Mat4`. D's only admitted type matches the observed
12-byte, 4-byte-aligned Vec3 shape. These observations do not make either
candidate representation-compatible with A/B beyond the stated type/target.
The C adapter reconstruction keeps the current renderer upload boundary safe
to observe, but C cannot claim direct representation, FFI, SIMD, or GPU-upload
compatibility with A/B.

## Interpretation Limits

These are native-target compiler facts, not a contract. They do not establish
Rust field-layout guarantees, `repr` policy, serialization format, C ABI,
WASM layout, GPU buffer rules, or whether a future Tokimu-owned type should
preserve provider alignment. Any such choice requires a named boundary and
separate measurement.
