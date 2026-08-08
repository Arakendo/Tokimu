# Alternative D Upstream Notice

This candidate is a deliberately narrowed, modified derivation of the audited
local `glam` source at commit `d36e7eeff05338c56c4aa8d59fc2615e7963b1b7`.

| Field | Value |
| --- | --- |
| Upstream package | `glam` 0.29.3 |
| Upstream source | `third-party/ring-0/glam/src/f32/vec3.rs` |
| Derived scope | construction/constants; arithmetic; dot/cross; normalization; min/max; lerp |
| Local derived file | `corpus/lib/tokimu-math-study/src/alternative_d.rs` |
| License terms | `third-party/ring-0/glam/LICENSE-APACHE` and `third-party/ring-0/glam/LICENSE-MIT` |

## Local Changes And Bound

- Retained only frozen `Vec3` manifest behavior; no swizzles, SIMD backends,
  generated code, broad trait families, or unrelated numeric types are copied.
  The small standard arithmetic operator implementations are explicitly
  declared in `COPY-MANIFEST.md`.
- Replaced upstream crate helpers and assertions with direct corpus-local
  scalar Rust.
- Similarity with Alternative C is intentional evidence of D's upstream
  lineage and maintenance burden. Neither duplicate may become stable by
  accident.
- Any added source must record its exact upstream path, revision, licensing,
  local modifications, and manifest pressure here.
