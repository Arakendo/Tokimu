# Option B Caller And Public-Boundary Pressure Scan

| Field | Value |
| --- | --- |
| Status | Slice 1 complete; repository callers only |
| Date | 2026-08-12 |
| Revision | `c84108cd2eabe2dbe13b658f4f493f996ca33d74` |
| Scope | Stable crates and non-study Rust corpus, excluding foreign source |

## Five-Type Source Pressure

Counts are textual occurrences followed by distinct Rust files. They are a
repeatable pressure indicator, not a public-API or runtime-frequency measure.

| Type | Stable crates | Non-study corpus | Interpretation |
| --- | ---: | ---: | --- |
| `Vec2` | 1 / 1 | 89 / 3 | stable occurrence is the re-export; corpus callers use application-local 2D types, not the stable math type |
| `Vec3` | 7 / 3 | 325 / 18 | direct pressure from camera, geometry, transforms, collision, picking, Doom, CAD, and GLB |
| `Vec4` | 6 / 2 | 11 / 4 | renderer columns/clip conversion and homogeneous projection pressure |
| `Quat` | 1 / 1 | 0 / 0 | only the stable re-export; no current repository caller |
| `Mat4` | 20 / 4 | 116 / 15 | direct camera, renderer, GLB, CAD, orientation, and Doom pressure |

The stable public exposure remains one direct re-export:

```rust
pub use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
```

`tokimu_render::Camera` additionally exposes `Mat4` through public `view` and
`projection` fields and its public constructor. No scanned stable signature
adds a provider trait bound or associated type. Renderer GPU transport already
crosses through renderer-owned scalar arrays; no public FFI, POD, reflection,
serialization, or TypeScript math-layout contract was found.

## Camera And Projection Pressure

The Option A candidate recorded **86** direct deprecated calls in its isolated,
dated source snapshot. That number remains an Option A fact. A refreshed scan
of the current working source finds **94** direct calls:

| Family | Stable crates | Non-study corpus | Math study | Current total |
| --- | ---: | ---: | ---: | ---: |
| right-handed look-at | 1 | 19 | 36 | 56 |
| right-handed GL-depth perspective | 1 | 5 | 25 | 31 |
| right-handed GL-depth orthographic | 1 | 0 | 6 | 7 |
| **Total** | **3** | **24** | **67** | **94** |

The difference does not invalidate the earlier 86-call observation: caller
source and study fixtures changed, and the scan scopes differ. Later update-
shock tests must use one frozen tree and command. The semantic ownership is
still narrow: callers request a view or projection contract; `glam::camera`
module organization is provider vocabulary.

## Current Mechanics With Named Pressure

| Type | Required mechanics | Named pressure |
| --- | --- | --- |
| `Vec3` | construction/components, arrays, arithmetic, component/scalar mul/div, axes, normalization, length/squared length/distance, dot/cross, min/max/lerp, extend, accumulation | renderer camera, Doom observer/collision/visibility, orientation, CAD picking, GLB transforms/animation |
| `Vec4` | construction/components, arrays, axes/explicit columns, truncate | renderer clip-depth conversion/upload, CAD/orientation projection |
| `Mat4` | identity, checked view/projection families, translation/scale/axis rotation, columns/arrays, multiply, inverse/transpose, point/vector/project transforms, explicit final-column mutation | renderer, Doom, GLB, CAD, orientation, stereo and textured-box corpus |
| `Vec2` | no stable mechanic earned | application-local 2D corpus types are evidence for outward/local ownership, not the public re-export |
| `Quat` | no stable mechanic earned | no direct non-study caller |

Formatting and `PartialEq` are exercised by tests and diagnostics. Arithmetic
traits are caller conveniences for the pressured `Vec3`/matrix mechanics.
No current evidence earns serialization, indexing, hashing, ordering, POD, or
foreign conversion as stable public meaning.

## What Tokimu Would Lose If A Type Disappeared

| Type | Demonstrated loss | Ownership consequence |
| --- | --- | --- |
| `Vec2` | source compatibility for an advertised name; no demonstrated stable engine behavior | removal/outward movement remains a live alternative; do not broaden Full B |
| `Quat` | source compatibility for an advertised name; no demonstrated direct caller | removal/incubation remains visible; do not invent quaternion policy |
| `Vec3` | common geometry, position/direction mechanics across renderer and corpus | a replacement needs a bounded real contract, while frame/chart/source meaning stays above it |
| `Vec4` | homogeneous and explicit column mechanics at projection/provider seams | keep transport and clip-depth policy outside the value wrapper |
| `Mat4` | current public camera vocabulary and transform mechanics | largest migration seam; Narrow B can own constructors without owning the value type |

## Compatibility Versus Meaning

Source compatibility pressures Full B toward five familiar names, public
components, operator traits, constants, and upstream-like spellings. Actual
Tokimu meaning currently requires only three ordinary value types plus the
three Narrow-B construction families. Even there, camera lifecycle, selected
camera, WGPU `[0, 1]` clip conversion, Doom embedding, chart identity,
orientation metadata, and input policy remain in their existing semantic
owners.

Accordingly:

- every later operation must cite the named callers above or an explicit
  conformance control;
- `Vec2` and `Quat` remain minimal compatibility probes;
- removal or outward movement remains an honest result for unpressured names;
- the post-Doom operation inventory 0.2 is the mechanics manifest for later
  Full-B work; and
- Narrow B is tested as a complete candidate, not as the first stage of Full B.
