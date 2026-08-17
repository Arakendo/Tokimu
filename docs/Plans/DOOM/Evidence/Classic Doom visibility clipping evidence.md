# Classic Doom Visibility Clipping Evidence

## Scope

This note records primary-source evidence used to shape the AR-0025 Stage 3B
corpus experiment. It does not make `tokimu-render` a Doom renderer, admit a
generic occlusion feature, or claim that the current E1M1 study recreates
classic Doom presentation.

The later end-to-end source reading, including frame order, coupled wall/plane
state, sky emission, masked work, and the proposed Tokimu preparation seam, is
retained separately in
[Classic Doom Renderer Dataflow And Tokimu Preparation Seam](Classic%20Doom%20renderer%20dataflow%20and%20Tokimu%20preparation%20seam.md).

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

The next bounded trace reconstructs source-keyed floor/ceiling cells from the
clip state immediately before each admitted wall range mutates it. It preserves
multiple plane instances when a new horizontal range collides with an existing
instance of the same `(kind, height, flat, light)` key; taking one bounding
union would manufacture coverage. The five fixed controls report:

| Pose | Source keys (floor/ceiling) | Plane instances | Collision splits | Horizontal spans | Empty after clip |
| --- | ---: | ---: | ---: | ---: | ---: |
| source spawn | 5 / 1 | 8 | 2 | 24 | 1,525 |
| near-wall A | 1 / 1 | 2 | 0 | 4 | 27 |
| near-wall B | 4 / 1 | 8 | 3 | 13 | 577 |
| courtyard-loss | 3 / 1 | 4 | 0 | 4 | 257 |
| hut control | 1 / 1 | 2 | 0 | 2 | 188 |

No retained instance contains overlapping column writes after splitting. The
source-spawn and near-wall-B controls prove that semantic plane key and plane
instance identity are different facts; the other three controls show that the
split is conditional rather than ceremonial. `empty after clip` counts source
mark attempts whose bounded interval had already collapsed under current clip
limits. These counts are diagnostic source cells, not historic pixel parity,
flat selection, triangulation, or presentation visibility.

Each retained instance now also preserves its contributing source-sector and
source-SEG identities. Resolving those identities against the prepared
source-labelled flat geometry produces:

| Pose | Resolved instances | Unresolved | Sky instances | Whole-subsector triangle candidates |
| --- | ---: | ---: | ---: | ---: |
| source spawn | 8 | 0 | 0 | 121 |
| near-wall A | 2 | 0 | 0 | 48 |
| near-wall B | 7 | 0 | 1 (`F_SKY1`) | 109 |
| courtyard-loss | 4 | 0 | 0 | 34 |
| hut control | 2 | 0 | 0 | 48 |

This proves that every non-sky plane instance has source and material
provenance in the existing prepared scene. It also demonstrates that those
prepared meshes are too coarse to realize the retained horizontal spans:
multiple split instances can select the same whole subsector triangle. Direct
submission would therefore recreate the over-inclusive static shell. The next
experiment must clip or reconstruct plane geometry from the retained
viewer-relative spans rather than treating flat lookup as presentation
selection.

The next headless control reconstructs each populated non-sky column as one
source-plane quad. Its four diagnostic screen boundaries are intersected with
the declared plane height; the source-space coordinates remain available for
continuous flat UV derivation. Results are:

| Pose | Retained non-sky cells | Reconstructed quads | Triangles | Horizon / behind / degenerate rejected | Maximum source distance |
| --- | ---: | ---: | ---: | ---: | ---: |
| source spawn | 49,712 | 1,110 | 2,220 | 0 / 0 / 0 | 1,073.713 |
| near-wall A | 27,288 | 628 | 1,256 | 0 / 0 / 0 | 150.619 |
| near-wall B | 24,028 | 652 | 1,304 | 0 / 0 / 0 | 1,370.424 |
| courtyard-loss | 29,952 | 652 | 1,304 | 0 / 0 / 0 | 356.831 |
| hut control | 25,173 | 480 | 960 | 0 / 0 / 0 | 337.920 |

One quad per retained column avoids both failure modes already observed: it
does not resubmit whole source-subsector triangles, and it does not manufacture
one quad for every diagnostic pixel. A focused negative fixture also proves
that a horizon-crossing strip is rejected explicitly rather than producing an
infinite plane intersection.

The fixed source-spawn continuation retains each column's source sector and
SEG owner, groups the reconstructed quads by plane key and source owner, and
applies the already prepared flat material plus the continuous Doom
source-spatial UV field. The separate plane-only presentation reports:

```text
source cells:       49,712
grouped meshes:         22
triangles:           2,220
warm mesh uploads:       0
warm replacements:       0
warm frame:          5.594 ms (development profile, AMD/Vulkan workstation)
```

This proves that the reconstructed plane geometry can use the existing generic
mesh/material path without allocating one draw per diagnostic column or adding
renderer vocabulary. It deliberately omits every wall and cutout. A manual
fixed-pose observation is still required before making even a bounded
no-visible-plane-omission claim; clean counts and successful presentation are
not visual correctness evidence.

Manual AMD/Vulkan inspection confirms that multiple reconstructed floor and
ceiling portions are visibly realized with their flat materials. Because the
control intentionally omits walls, that observation cannot classify every
visible opening as a correct wall boundary or a missing plane span. It is
therefore retained as positive presentation evidence, not false-negative
closure.

The fixed-pose contextual companion adds the recursively admitted opaque wall
tiers without changing the plane reconstruction:

```text
plane meshes / triangles: 22 / 2,220
whole-SEG wall meshes:         80
omitted wall triangles:         0
total corpus draws:           102
warm mesh uploads:              0
warm replacements:              0
warm frame:                 7.553 ms (development profile, AMD/Vulkan workstation)
```

This makes the manual plane-gap falsifier better framed while retaining an
important limitation: the wall meshes are admitted whole-SEG tier geometry,
not exact projected wall-tier spans. They may provide context but cannot serve
as wall-presentation parity evidence.

The first contextual manual inspection showed missing floor and ceiling
regions after the camera had been allowed to move away from the one
source-spawn pose used to reconstruct the meshes. That is a valid rejection of
the scene as dynamic presentation, but it conflated two claims. Both classic
plane presentation modes now lock the source-spawn observer and identify the
lock in the window title. A second observation at that structurally fixed pose
is required to decide whether the reconstruction itself has visible omissions.
No whole-subsector flat fallback was added to conceal the result.

The locked source-spawn rerun retained visible floor and ceiling gaps, making
the pre-correction reconstruction a genuine fixed-pose false negative. A
fixture audit then found that wall admission, cell reconstruction, and native
presentation did not share one perspective mapping: admission used tangent
screen columns, reconstruction interpolated angles linearly, and presentation
used an independent 60-degree vertical field of view. Those paths now share
the tangent-space mapping implied by a 90-degree horizontal field of view at
320x200. This is a controlled correction, not positive evidence; the aligned
mode still requires visual rerun and is rejected if the gaps remain.

The aligned locked rerun removed the broad omissions but retained thin
openings at some wall/plane edges. This proves the projection correction was
necessary while still falsifying the plane-cell variant under the required
no-visible-false-negative rule. The remaining evidence points to a boundary
representation mismatch: integer diagnostic cells reconstructed as world
triangles do not inherently share the exact continuous edge used by contextual
whole-SEG walls. The study does not hide this with overlap, an epsilon, or
whole-subsector fallback geometry.

This remains a Doom presentation-provider experiment. Generic Tokimu callers
may use different source-owned selection methods; no `SEG`, `solidsegs`, Doom
portal rule, or classic renderer policy belongs in `tokimu-render`.

## Successor Slice 0 Source-Occurrence Trace

The ordered-occurrence successor study returned to the primary sources to
answer a representation question left open by the AR-0025 experiments. The
full trace is retained in
[Doom Ordered Source-Occurrence Reference Trace](Doom%20ordered%20source%20occurrence%20reference%20trace.md).

### Direct observations

- Classic Doom passes one `seg_t *curline` into `R_AddLine` after near-first
  BSP traversal.
- After angular view clipping and horizontal projection,
  `R_ClipSolidWallSegment` and `R_ClipPassWallSegment` scan accumulated
  `solidsegs` ranges.
- One source SEG can cause several `R_StoreWallRange` calls: the routines emit
  a visible prefix, every uncovered internal gap, and a visible suffix as
  separate calls.
- Every individual `R_StoreWallRange` call remains one contiguous inclusive
  horizontal interval and continues to refer to the same source SEG.
- `R_RenderSegLoop` uses the same `ceilingclip` and `floorclip` state to bound
  wall tiers, mark floor/ceiling plane intervals, and update later coverage.
- Masked two-sided middles defer texture-column drawing and do not gain solid
  horizontal occlusion authority merely because a mask exists.
- Chocolate Doom preserves these behaviors while containing bounded storage
  and portability maintenance differences.

Primary files inspected on 2026-08-16:

- [Classic Doom `r_bsp.c`](https://github.com/id-Software/DOOM/blob/master/linuxdoom-1.10/r_bsp.c)
- [Classic Doom `r_segs.c`](https://github.com/id-Software/DOOM/blob/master/linuxdoom-1.10/r_segs.c)
- [Chocolate Doom `r_bsp.c`](https://github.com/chocolate-doom/chocolate-doom/blob/master/src/doom/r_bsp.c)
- [Chocolate Doom `r_segs.c`](https://github.com/chocolate-doom/chocolate-doom/blob/master/src/doom/r_segs.c)

### Tokimu inference

The directly observed multiplicity justifies a Doom-private model in which one
source contribution produces `0..N` presentation occurrences. It does not
justify copying integer columns, clip arrays, or arbitrary raster regions.
Each private occurrence can be constrained initially to one contiguous
horizontal source-relative interval with bounded upper/lower domains, while
multiple occurrences retain correlation to one source identity.

The normalized source interval is a Tokimu reconstruction choice. Classic Doom
retains fixed/projective values and integer range endpoints rather than an
explicit normalized `t` range. Later slices must derive that interval from
original source geometry and prepared view intersections, never from
screen-column inverse projection.

## Final AR-0025 Disposition

The source protocol study is complete enough to answer its architectural
question without claiming classic-Doom renderer parity. SEG splitting can
preserve linedef/sidedef identity and continuous texture coordinates, and the
recursive BSP controls explain why classic Doom can omit source-valid shell
geometry. However, the uploaded horizontal, per-column, and bounded
plane/context representations all produced visible false negatives.

Accordingly, these modes remain falsification fixtures and Doom-owned research.
They do not define Tokimu visibility, renderer culling, source-independent
occluder policy, or a production Doom presentation provider. AR-0025 closes
with full caller-owned submission as the renderer fallback; a future Doom
presentation reconstruction must share exact wall/plane boundaries or present
source screen spans directly before it can make a no-visible-omission claim.
