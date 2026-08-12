# Classic Doom Visibility Clipping Evidence

## Scope

This note records primary-source evidence used to shape the AR-0025 Stage 3B
corpus experiment. It does not make `tokimu-render` a Doom renderer, admit a
generic occlusion feature, or claim that the current E1M1 study recreates
classic Doom presentation.

## Primary Source

The released id Software renderer separates BSP traversal and horizontal wall
range clipping in
[`r_bsp.c`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/r_bsp.c)
and per-wall column/span work in
[`r_segs.c`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/r_segs.c).

Relevant retained observations:

| Source function | Retained behavior | Stage 3B consequence |
| --- | --- | --- |
| `R_RenderBSPNode` | Traverses the viewer-side child first and checks the other child’s bounding box before visiting it. | A flat near-first list of every leaf is not the complete traversal protocol. |
| `R_CheckBBox` | Consults the accumulated `solidsegs` horizontal ranges before traversing a far BSP child. | Far-subtree rejection is driven by source screen-range state, not a generic world-space occlusion claim. |
| `R_AddLine` | Rejects back-facing SEGs, clips to horizontal view range, then classifies a line as solid or pass-through. | A successor must not feed every in-frustum SEG into a coverage model. |
| `R_ClipSolidWallSegment` | Adds a solid line’s horizontal screen interval to `solidsegs`. | The core solid-coverage state is a union of horizontal ranges. |
| `R_ClipPassWallSegment` | Stores visible portions of a window/portal interval but does not add it to `solidsegs`. | Portal/opening authority and solid occluder authority must remain distinct. |
| `R_RenderSegLoop` | Uses per-column upper/lower clip arrays while drawing wall tiers and marking planes. | Vertical clipping is renderer/source-span detail after line admission; it is not evidence that Stage 3B should begin with a generic 2D occupancy grid. |

## Consequence for Tokimu’s Corpus

The previous Stage 3B grid was a useful falsifier, but it was not a faithful
model of this source protocol. It incorrectly combined:

```text
all near-first leaves
  + all in-frustum SEGs
  + broad vertical span
  + boolean 2D coverage
```

The next bounded Doom-only control should instead establish, headlessly and
without renderer changes:

```text
viewer-relative BSP recursion
  + backface/FOV SEG admission
  + source solid vs pass classification
  + horizontal solid-range union
  + far-child bbox rejection
```

Only after that control survives the retained E1M1 false-negative poses can a
later experiment ask whether source wall-tier/plane clipping is needed for a
more faithful visual comparison.

The current trace also follows the exterior-hut suspect from source identity
to a far-child decision. Linedef `247` belongs to subsectors `190` and `192`.
At the retained close-wall, courtyard, and hut-facing controls, the recursive
experiment reaches neither subsector: a preceding solid range covers each
projected far bbox interval. This is useful causal evidence for a Doom-only
presentation reconstruction; it does not make that source wall invalid or
authorize deletion from the normal static shell.

## Current Corpus Limitation

The first recursive control now selects bbox silhouette corners with the source
`checkcoord` table and maps their angles through a perspective plane. A far
child that is definitely outside the source horizontal FOV is rejected; a bbox
that is behind, contains the viewer, or cannot be projected safely remains
fail-open. It is still **not** a reproduction of the released renderer's
binary-angle/FOV lookup tables. Until a bounded source-interval regression
validates the remaining approximation (or replaces it), the control remains
headless and must not be used as a presentation/culling mode.

The latest trace also inventories the source-labelled static flat meshes whose
owning subsectors the recursive walk reaches. Across the retained controls it
finds `184/230/164/150` floor draws and `149/157/136/120` ceiling draws. Those
are deliberately not classic Doom plane-span claims: they show that the wall
SEG protocol cannot yet be promoted into a presented candidate set while
source wall-tier and plane-span reconstruction remain absent.

The same source-only trace separates already lowerable SEG-wall triangles by
source tier. Near-wall A, near-wall B, courtyard, and hut-control retain
upper/lower/middle counts of `0/0/6`, `4/34/53`, `0/0/4`, and `0/0/8`.
This proves that the surviving wall set is not uniformly opaque: horizontal
solid-range admission cannot decide later tier, cutout, or plane presentation.

The next source-only checkpoint records classic wall-stage plane eligibility at
source eye height `36`. The retained controls mark floor/ceiling eligibility
as `4/3`, `53/33`, `5/3`, and `4/4`, with paired-`F_SKY1` ceiling adjustments
of `2/15/0/0`. These values are not plane spans, flat draws, or screen coverage:
they deliberately stop before the original renderer's per-column clip arrays
and visplane construction.

The subsequent bounded wall-tier trace makes that missing layer concrete. At
the source-spawn control, 37 recursively admitted SEGs contribute 8 upper, 7
lower, and 23 middle tier spans. The wall-tier/plane trace reports 823 ceiling
and 875 floor clip-boundary changes, while 36/37 floor/ceiling plane marks remain
separate source facts. Near-wall B retains 2/17/25 upper/lower/middle spans and
355/706 boundary changes. Marked planes can advance a boundary even where an
upper/lower texture tier is absent; one-sided middles are terminal, while
two-sided/masked middles remain open. Therefore no boolean occupancy result can
stand in for wall-tier, opening, and plane-span reconstruction. The trace is
still headless: it creates no visplanes, flat selection, renderer state, or
presentation result.

Before attempting any span construction, a separate source-key inventory now
records the grouping identity that classic Doom applies before accumulating a
plane: height, flat identity, and light level, with `F_SKY1` ceilings sharing a
common sky key. At source spawn, 36 floor and 37 ceiling contributors resolve
to 6 and 7 distinct source keys respectively (including 3 sky-ceiling
contributors). Near-wall B expands that to 53/33 contributors and 10/7 keys,
including 17 sky contributors. This establishes that a future span experiment
must preserve plane identity independently of both sector identity and wall
clip updates. The inventory remains source evidence only; it allocates no
visplanes, produces no spans, and selects no flat mesh.

This remains a Doom presentation-provider experiment. Generic Tokimu callers
may use different source-owned selection methods; no `SEG`, `solidsegs`, Doom
portal rule, or classic renderer policy belongs in `tokimu-render`.
