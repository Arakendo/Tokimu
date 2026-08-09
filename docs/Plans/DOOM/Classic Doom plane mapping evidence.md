# Classic Doom Plane Mapping Evidence

## Scope

This note records why Slice 5 retains source-texel coordinates for walls but
does not yet claim static per-vertex UV coordinates for floors and ceilings.
It is evidence for a deferred presentation decision, not a reason to invent a
replacement plane-mapping convention in the geometry provider.

## Original-source observation

In the original renderer's
[`r_plane.c`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/r_plane.c),
`R_MapPlane` derives each span's `ds_xfrac` and `ds_yfrac` from the current
view position, current view angle, screen position, and plane distance. The
regular-flat path then supplies the selected 64 by 64 flat to that span
mapper. This is a screen-space renderer operation rather than a map-record
per-vertex coordinate convention.

Consequently, the following would be an overclaim at the present headless
boundary:

```text
classic flat name + BSP triangle
    => original-Doom static per-vertex UVs
```

That implication does not follow from the source algorithm.

## Current disposition

Tokimu retains source floor/ceiling texture names and emits bounded,
source-traceable surface triangles. Wall lowering additionally retains the
authored sidedef axes and original renderer-compatible `texturemid` anchors,
which are source-local wall semantics.

Floor/ceiling sampling, projection, wrapping, and any modern static UV
representation remain a presentation/material decision. A later slice must
either:

1. implement the original view-dependent plane span behavior at an appropriate
   presentation boundary; or
2. admit a different static mapping contract explicitly and label it as a
   Tokimu presentation choice rather than classic Doom equivalence.

Neither option belongs in the current headless geometry provider.
