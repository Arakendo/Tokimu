# Classic Doom Wall V-Placement Evidence

## Scope

This record captures the original renderer's vertical texture anchor rules for
the Slice 5 headless geometry experiment. It does not make Tokimu's renderer
emulate Doom's column renderer, nor does it settle middle-texture clipping.

The primary source is id Software's released
[`r_segs.c`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/r_segs.c),
especially `R_StoreWallRange` and `R_RenderMaskedSegRange`.

## Retained Source Rules

For the source renderer's current front/back sector orientation and a sidedef
row offset `rowoffset`:

| Wall role | Raw flag state | Source vertical anchor before row offset |
| --- | --- | --- |
| One-sided middle | `ML_DONTPEGBOTTOM` set | front floor + texture height |
| One-sided middle | clear | front ceiling |
| Two-sided upper | `ML_DONTPEGTOP` set | front ceiling |
| Two-sided upper | clear | back ceiling + texture height |
| Two-sided lower | `ML_DONTPEGBOTTOM` set | front ceiling |
| Two-sided lower | clear | back floor |
| Two-sided masked middle | `ML_DONTPEGBOTTOM` set | max(front floor, back floor) + texture height |
| Two-sided masked middle | clear | min(front ceiling, back ceiling) |

The original code adds `rowoffset` after choosing each anchor. It also clips
upper/lower drawing against the source renderer's visible portal interval;
that clipping cannot be copied blindly into Tokimu geometry because it depends
on view-dependent column clipping.

## Tokimu Mapping Admitted

`doom-geometry-provider` records the source linedef flags, raw sidedef Y
offset, texture name, dimensions, and owning right/left side. It resolves the
original front/back relationship from the wall candidate's retained ownership,
then emits tested source-texel U/V coordinates for the admitted one-sided,
upper, lower, and shared-opening middle triangles. The mapping retains:

1. source linedef and sidedef identity;
2. selected texture extent and raw row offset;
3. selected anchor rule and flag state; and
4. the final geometry-local V origin.

That work belongs at the Doom geometry boundary. The renderer should receive
ordinary UVs and never infer Doom pegging flags.

## Consequence

The wall V-placement portion of Slice 5 is complete. The remaining presentation
work is deliberately separate: material sampling, alpha treatment for masked
middles, portal behavior, and final renderer submission must not reopen or
duplicate the source-anchor calculation.

## Slice 5B static normalization policy

The first static E1M1 consumer normalizes the already-retained source texels
without recalculating a pegging anchor:

```text
u_normalized = source_u / selected_texture_width
v_normalized = source_v / selected_texture_height
```

The source `v = texture_mid_y - world_y` convention already grows toward
successive top-row-first raster rows as world height decreases. Therefore this
normalization has no additional V inversion. Point/repeat sampling is the
selected initial material policy; it is supplied by the consumer under
ADR-0012, not inferred by `doom-geometry-provider` or the renderer.

Two-sided masked-middle observations remain excluded by their retained source
classification until AR-0023 admits a generic alpha/cutout policy. One-sided
middle walls retain the `Middle` role but are not masked-middle observations;
they remain eligible for the opaque candidate path if their selected raster is
fully covered.
