# Alternative D Copied-Source Manifest

| Field | Value |
| --- | --- |
| Status | Initial bounded slice; expansion paused |
| Upstream revision | `d36e7eeff05338c56c4aa8d59fc2615e7963b1b7` |
| Upstream file | `third-party/ring-0/glam/src/f32/vec3.rs` |
| Derived file | `corpus/lib/tokimu-math-study/src/alternative_d.rs` |

## Section Mapping

| Local concern | Upstream anchor | Local modification |
| --- | --- | --- |
| Constants and construction | `Vec3` constants beginning at line 28 | Reduced to `ZERO`, `ONE`, `Y`, `new`, `splat`, `from_array`, and `to_array` |
| Dot and cross | lines 203 and 217 | Scalar direct expression; no upstream helpers/assertions retained |
| Component min/max | lines 230 and 243 | Scalar direct expression |
| Normalization | lines 523 and 573 | Scalar reciprocal-length expression; finite-positive fallback retained |
| Linear interpolation | line 759 | Retained as `self + (other - self) * scalar` |
| Scalar/component operators | `Div<f32>` line 1102; `Mul<f32>` line 1242; `Neg` line 1806 | Reduced direct scalar and component operators only |
| Add/subtract | upstream vector operator implementation family | Reduced direct operators; helper/macro machinery excluded |

## Local Line Ledger

Every line in `src/alternative_d.rs` is either corpus scaffolding or a bounded
derivation from the upstream file and anchors above. This ledger is reviewed
when the derived file changes; it does not pretend the scalar rewrite is an
unmodified upstream copy.

| Local lines | Classification | Upstream relationship / review reason |
| --- | --- | --- |
| 1-8 | Tokimu corpus scaffolding | Study notice and standard operator imports; not copied provider implementation. |
| 10-15 | Derived `Vec3` representation | Upstream `Vec3` declaration beginning at line 28; reduced to three scalar fields. |
| 17-40 | Derived construction and conversion | Upstream constants/construction beginning at line 28; only earned constants and arrays retained. |
| 42-54 | Derived dot/cross | Upstream lines 203 and 217; direct scalar rewrite. |
| 56-69 | Derived normalization | Upstream lines 523 and 573; direct scalar rewrite with retained finite-positive fallback. |
| 71-92 | Derived min/max/lerp | Upstream lines 230, 243, and 759; direct scalar rewrite. |
| 95-140 | Derived operators | Upstream operator families including lines 1102, 1242, and 1806; direct scalar/component forms only. |
| 142-160 | Tokimu corpus tests | Original study assertions for the admitted slice; not copied provider tests. |

## Explicit Exclusions

- Swizzles, generated code, SIMD/architecture backends, serialization,
  reflection, POD traits, indexing, and all types outside `Vec3`.
- Upstream assertion and helper infrastructure not needed for the retained
  scalar behavior.
- Matrix, quaternion, and `Vec2`/`Vec4` sources; their D maintenance burden
  has not been earned.

## Update And Fix Policy

Before changing D, compare the pinned revision with these source anchors;
record whether an upstream security, correctness, portability, or performance
fix affects a retained section; then port it with a new modification entry or
explicitly reject it. ADR-0010's provenance audit detects a changed pinned
provider revision. No upstream update is silently inherited by D.
