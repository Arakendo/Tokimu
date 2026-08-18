# Hardware Doom Arbitrary-Pitch Plane Preparation Precedent

## Question

What world/render representation do established hardware Doom renderers use
for floors, ceilings and sky when arbitrary pitch makes Classic screen-column
and visplane coordinates insufficient?

This is a bounded AR-0030 precedent study. It does not admit source-port code,
a generic visibility resolver, a renderer portal contract, or a new Tokimu
subsystem.

## GZDoom

GZDoom separates persistent render preparation from per-view traversal.

Its map loader explicitly prepares subsectors for hardware rendering by:

- assigning render sectors, including self-referencing-sector handling;
- grouping connected subsectors into map sections;
- calculating rendering bounds;
- marking malformed/hacked subsectors; and
- preparing portal and transparent-door relationships.

Its hardware level mesh creates floor and ceiling surfaces from each
subsector's ordered SEG vertices. These are world-space plane polygons, not
reverse-projected Classic visplane rows. Sky is retained as surface meaning on
the ceiling representation.

Per view, the hardware renderer still traverses the Doom BSP near first and
uses an angular clipper to reject covered SEG/subtree ranges. Subsector
processing additionally tests the actual floor/ceiling endpoint heights
against a pitch-aware clipper. Surviving render sections schedule their flat
processing once, while portal coverage and hacked subsectors receive explicit
source-specific handling.

Primary evidence:

- <https://github.com/ZDoom/gzdoom/blob/master/src/maploader/renderinfo.cpp>
- <https://github.com/ZDoom/gzdoom/blob/master/src/rendering/hwrenderer/doom_levelmesh.cpp>
- <https://github.com/ZDoom/gzdoom/blob/master/src/rendering/hwrenderer/scene/hw_bsp.cpp>

## Doom iOS / PrBoom-Style GL

Doom iOS demonstrates a smaller but coarser hardware translation:

- level initialization triangulates persistent sector geometry;
- BSP traversal retains a horizontal screen-column occlusion array;
- reaching any uncovered subsector admits that sector's floor and ceiling once
  for the frame;
- complete line geometry can then be drawn using GPU depth; and
- sky-marked sector planes are omitted while a separate sky presentation is
  enabled, with source-specific sky walls used at boundaries.

The source itself records the weakness of sector-wide admission: sprites in a
rear portion of a non-convex sector can be admitted even when subsector-order
processing would have occluded them. The same granularity warning applies to
planes. This makes the design useful precedent but not a sufficient Tokimu
answer for the retained E1M1 leak cases.

Primary evidence:

- <https://github.com/id-Software/DOOM-iOS/blob/master/code/iphone/iphone_render.c>

## Comparison

| Concern | GZDoom | Doom iOS / PrBoom GL | Tokimu evidence |
| --- | --- | --- | --- |
| Final plane representation | World-space render-subsector/section surfaces | Persistent triangulated sector planes | Classic row cells visually falsified |
| View participation | BSP plus horizontal and pitch-aware clippers | BSP plus horizontal occlusion | Doom ordered absence is required |
| Plane granularity | Subsector/section with render-sector meaning | Whole sector once reached | Whole-sector/whole-plane authority is too coarse |
| Sky | Source-aware surface/portal handling | Omit sky plane geometry; separate sky plus sky walls | Sky is presentation, not the far-geometry rejector |
| Hacks | Prepared render sectors, sections and hacked subsectors | Targeted fixes and sector merging | Must remain Doom-private |

## Finding

Established hardware paths do not use Classic visplane rows as their final
arbitrary-pitch geometry. They manufacture a persistent Doom-aware world/render
representation, then execute a view-conditioned source traversal over that
representation.

The smallest representation suggested for the next E1M1 experiment is:

```text
Doom render subsector
    source subsector identity
    ordered finite boundary loop
    current floor and ceiling plane facts
    render-sector association
    ordinary world-space plane triangles
    sky/ordinary plane role
    hack/unresolved provenance
```

Per view, a Doom-private traversal would produce zero or one participation
decision for each such render subsector using horizontal coverage plus the
actual pitched camera. Surviving ordinary surfaces lower to normal Tokimu
declarations. Sky regions remain source presentation meaning and do not become
ordinary occluding geometry.

## Boundaries And Falsifiers

This is not authorization to implement the full GZDoom model. An E1M1 slice
must stop if it requires generic portal primitives, stable renderer changes,
copying compatibility-hack taxonomies, or provider-wide public contracts.

The first experiment must retain these falsifiers:

- complete spawn room under yaw, movement and pitch;
- valid hut and far-left structure;
- absence of the five retained leak contributions;
- bounded treatment of the retained partial ceiling;
- runtime-height freshness for doors and platforms; and
- ordinary renderer declarations with no Doom vocabulary.

The experiment is rejected if it merely substitutes whole-sector admission for
the falsified Classic row-cell reconstruction.
