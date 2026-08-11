# Doom Coordinate-Conversion Evidence — 2026-08-10

This artifact retains Slice 4 structural evidence for AR-0028. It describes
the current Doom corpus conversion; it does not establish a global Tokimu
world, camera, or input convention.

## Declared Conversion

Classic Doom map positions and directions use a two-dimensional `(x, y)`
plane plus a separately supplied height. The current corpus provider performs
this explicit lift:

```text
Doom point     ([x, y], height) -> Tokimu-facing [x, height, y]
Doom direction ([x, y], vertical) -> Tokimu-facing [x, vertical, y]
```

The exact inverses are:

```text
Tokimu-facing [x, y, z] -> Doom point/direction ([x, z], y)
```

`doom_point_to_tokimu`, `tokimu_point_to_doom`,
`doom_direction_to_tokimu`, and `tokimu_direction_to_doom` live in the Doom
geometry corpus provider. Floor, ceiling, one-sided wall, two-sided wall, and
masked-middle lowering all use this named conversion. No Doom conversion was
added to `tokimu-render`.

The current source heading conversion is:

```text
Doom 0 degrees   -> world +X
Doom 90 degrees  -> world +Z
Doom 180 degrees -> world -X
Doom 270 degrees -> world -Z

observer yaw = atan2(forward.x, forward.z)
Doom degrees = (90 - observer_yaw_degrees) modulo 360
```

Tests round-trip `0`, `45`, `90`, `180`, `270`, and `359` degrees within
`0.0001` degrees. Point and direction lifts use component permutation only and
round-trip exactly for their bounded samples.

## Bounded Synthetic Wall Evidence

The provider fixture has one linedef from source `(10, 20)` to `(30, 40)` and
retains both sidedefs:

| Source side | Label identity | X offset | Source U at start | Source U at end | Facing expectation |
| --- | --- | ---: | ---: | ---: | --- |
| right/front | `RIGHT_LABEL` | 7 | `7 + sqrt(800)` | `7` | `(20, 0, -20)` |
| left/back | `LEFT_LABEL` | 11 | `11` | `11 + sqrt(800)` | `(-20, 0, 20)` |

This deliberately distinguishes the two authored texture axes. Separate
winding tests calculate the supplied triangle normals and assert that the
right/front and left/back faces oppose one another. Renderer validation only
checks the resulting ordinary position, normal, and UV streams; it contains no
Doom sidedef or texture-axis branch.

The structural fixture proves the labeled identities and generated data. A
second native presentation fixture sends two one-sided walls through the real
Doom lowerer: a right/front `FRONT` wall and an oppositely directed left/back
`BACK` wall. Both face one camera, and the maintainer confirmed both complete
asymmetric panels read correctly. The camera basis places world `+X` on
screen-left, so `BACK` appears on screen-left and `FRONT` on screen-right.
The labels remain corpus evidence rather than generic renderer vocabulary.

An earlier visual attempt used two cameras plus half-surface `ViewportRect`
values. That clipped two full-surface projections because the admitted field
is a pixel scissor, not an independent NDC remapping viewport. The fixture was
reduced to one camera and two spatially separated walls rather than changing
renderer semantics to satisfy test presentation.

## Source-Spawn Replay

The reviewed E1M1 player-one source record is retained as:

```text
source position = (1056, -3616)
source angle = 90 degrees
source sector = 38
floor / ceiling = 0 / 72
corpus evidence eye height = midpoint = 36
world eye = (1056, 36, -3616)
world forward = +Z
world screen-right under the current RH observer = -X
```

A deterministic unit replay asserts that forward by 16 reaches
`(1056, 36, -3600)`, right strafe by 16 reaches `(1040, 36, -3616)`, and a
screen-right quarter turn points along the same `-X` local-right basis. This
is observer evidence, not original Doom movement or player-eye-height policy.

## Canonical EXITSIGN Package Evidence

The canonical-package preflight was rerun against
`doom-shareware-corpus-v1.zip` / `DOOM1.WAD`. It retained 16 submitted
triangles, two per wall segment:

| Linedefs | Sidedefs | Side | Role | Normalized U range |
| --- | --- | --- | --- | --- |
| 342–343 | 467, 469 | right/front | upper | `[0.000, 0.500]` |
| 344–345 | 471, 473 | right/front | upper | `[0.500, 0.625]` |
| 347–348 | 477, 479 | right/front | upper | `[0.000, 0.500]` |
| 349–350 | 481, 483 | right/front | upper | `[0.500, 0.625]` |

There is no submitted `EXITSIGN` entry for linedef 346. This evidence is why
the earlier left/back-only diagnosis is false: every canonical submitted sign
surface in this package is right/front. Native presentation has been manually
observed with readable art after the provider correction. Browser evidence was
captured through a dedicated single-face inspection view rather than inferred
from the distant overview.

The first browser inspection camera incorrectly averaged both retained sign
groups. Their opposed owning-side normals cancelled exactly, and the fixture
rejected the zero vector rather than inventing a view direction. A second
attempt treated 342–345 as one planar group, but retained centers and normals
showed that they are the four faces of a rectangular sign housing: `+Z`, `-Z`,
`+X`, and `-X`. The repaired inspection camera selects the single right/front
face at linedef 342. The other faces and the 347–350 housing remain separate
package evidence. These were camera-grouping defects, not evidence that one
right/front surface has an ambiguous normal.

The repaired single-face browser view subsequently presented all 1,835 opaque
draws with `camera=canonical-exitsign`, and the maintainer confirmed that the
`EXIT` lettering on face 342 reads correctly. The retained observation is in
[`browser-exitsign-observation-2026-08-10.md`](browser-exitsign-observation-2026-08-10.md).

## Validation Commands

```powershell
cargo test -p doom-geometry-provider
cargo test -p hello-doom-e1m1
cargo run -q -p hello-doom-e1m1 --bin hello-doom-e1m1 -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD
```

The tests cover exact point/direction round trips, bounded orientation round
trips, opposed right/front and left/back U axes, opposing wall normals,
cardinal headings, source-spawn placement, and deterministic forward/strafe/yaw
replay.
