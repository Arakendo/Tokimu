# Unpressured Public Types: `Vec2` and `Quat`

| Field | Value |
| --- | --- |
| Status | Source-scan finding; not permission to remove a public re-export |
| Date | 2026-08-08 |
| Scope | Direct production and corpus imports of `tokimu_core::math` |

## Finding

The refreshed source scan found no direct production or corpus import of
`Vec2` or `Quat` from `tokimu_core::math`. Their only direct references in the
scan scope are the current re-export and this study's baseline/candidate probes.

Current 2D corpus entries such as `hello-asteroids`, `hello-2d-physics`, and
the website-asteroids consumer define application-local `Vec2` types. Those
types may demonstrate future two-dimensional math pressure, but they do **not**
show that the present Ring 0 `glam::Vec2` re-export is their required semantic
owner or that their local APIs should be copied into Tokimu math.

No direct `Quat` caller was found.

## Consequences For The Study

- B retains minimal `Vec2` and `Quat` name/constant/array probes solely to
  make public-vocabulary coupling visible without importing broad mechanics.
- C does not implement `Vec2` or `Quat`; adding them would be speculative and
  would weaken the bounded owned-subset experiment.
- D remains `Vec3` only and paused.
- This finding does not remove `Vec2` or `Quat` from stable `tokimu_core::math`.
  Removing or changing a public re-export requires downstream compatibility,
  release, and migration evidence beyond repository source search.

## Reopening Triggers

- A direct caller imports `Vec2` or `Quat` from `tokimu_core::math`.
- DOOM introduces a named 2D, rotation, animation, collision, or transform
  requirement that needs either type.
- A public API, serialization, FFI, or authoring boundary exposes either type.
- Downstream-consumer evidence establishes a compatibility commitment not
  visible in this repository.

## Evidence Commands

```powershell
rg -n --glob '*.rs' "use tokimu_core::math::.*\b(Vec2|Quat)\b|use tokimu_core::math::\{[^}]*\b(Vec2|Quat)\b" crates corpus tests
rg -n --glob '*.rs' "tokimu_core::math" crates corpus tests
```
