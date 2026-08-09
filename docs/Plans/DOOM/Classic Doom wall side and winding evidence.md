# Classic Doom Wall Side And Winding Evidence

## Scope

This note records the side and triangle-winding convention used by the Slice 5
headless WAD geometry experiment. It concerns only source-traceable static
wall candidates. It does not admit backface culling, lighting, visibility, or
a renderer coordinate convention.

## Original Source Convention

The released Doom loader assigns linedef sidedef slot `0` to `frontsector` and
slot `1` to `backsector` in
[`P_LoadLineDefs`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/p_setup.c).
Tokimu therefore names the decoded WAD fields and lowered wall sides as:

| WAD field | Tokimu side | Original relation |
| --- | --- | --- |
| `right_sidedef` / slot `0` | `DoomWallSideKind::Right` | front |
| `left_sidedef` / slot `1` | `DoomWallSideKind::Left` | back |

The original renderer obtains each SEG's front sector from the selected side
and uses it for wall processing; see
[`P_LoadSegs`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/p_setup.c)
and [`R_Subsector`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/r_bsp.c).

## Tokimu Headless Geometry Convention

The current geometry study embeds source positions as
`(doom_x, height, doom_y)`. For a source linedef from `(x0, y0)` to `(x1, y1)`,
with direction `(dx, dy)`, the lowered triangle normal must face:

| Owning side | Required horizontal normal |
| --- | --- |
| `Right` / WAD front | `(dy, 0, -dx)` |
| `Left` / WAD back | `(-dy, 0, dx)` |

`doom_wall_quad_triangles` is the sole helper that selects this winding for
one-sided walls, two-sided height bands, and two-sided middle textures. Its
unit test uses a non-axis-aligned line and verifies both directions with a
cross product. The normal SVG derives the same two side vectors, so its cyan
right/front and magenta left/back arrows cannot silently disagree with lowered
geometry.

SVG deliberately negates map Y for ordinary top-down screen presentation.
That display transform mirrors the apparent left/right direction; readers
should interpret cyan and magenta as source front/back evidence, not as a
claim that screen-space “right” is the geometric normal.

## Consequence

The earlier SVG was valuable: it exposed that the diagnostic arrows and the
triangle winding had both adopted the *opposite* of the documented WAD side.
Slice 5 now has a tested, source-backed side convention. Whether the eventual
renderer enables culling, reverses a view transform, or consumes two-sided
materials remains a later explicit presentation decision.
