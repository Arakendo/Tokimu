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

## Headless disposition

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

## Slice 5B selected static presentation policy

Slice 5B selects option 2 for its first static E1M1 presentation: a bounded,
Tokimu-authored map-axis mapping. It is not a claim of original Doom
screen-space-plane equivalence.

For a retained `DoomSurfaceTriangle` position `[x, height, z]`, the consuming
presentation lowerer supplies the `Textured3d` coordinate:

```text
u =  x / 64
v = -z / 64
```

`64` is the decoded flat width and height, not a renderer constant. The
formula is written here using E1M1's fixed 64-by-64 flat sources. A later
caller with another flat extent must use its selected source extent and retain
that choice in its evidence.

The negative V is a local bridge between the retained Doom map `z` axis and
the top-row-first raster coordinate convention used by the RGBA8 upload. It
is a declared presentation convention, not a deduction from `r_plane.c`.
Point filtering and repeat addressing are selected for the initial static
proof. This keeps palette texels categorical and makes coordinates outside the
unit square visible without adding a mip or anisotropy policy.

The initial scene further constrains its material use as follows:

- palette zero is selected by the Doom presentation consumer and its RGB bytes
  are uploaded through the existing sRGB color-texture path; the WAD did not
  itself declare this color-space metadata;
- only fully covered floor and ceiling flat pixels plus fully covered wall
  texture pixels participate in opaque draws;
- the presentation pipeline explicitly uses `BlendMode::Opaque`, depth test,
  and depth writes rather than inheriting `Textured3d`'s current alpha-blend
  default;
- `F_SKY1` remains a source-traceable omission in the first capture, not a
  textured plane; and
- two-sided masked-middle observations remain omitted and counted until
  AR-0023 chooses a provider-neutral alpha policy.

This policy is intentionally smaller than a Doom compatibility renderer. It
does not reproduce original fixed-point span sampling, colormap lighting,
sky drawing, masked-middle clipping, or source-port texture rules. The static
scene must label those omissions and retain their source identities.

## Validation required before static-scene admission

The future E1M1 presentation consumer must test that:

1. its flat UV lowerer emits the formula above for representative source
   triangle vertices;
2. the selected sampler is point/repeat and the draw pipeline is opaque with
   depth writes;
3. every submitted floor or ceiling draw retains a flat name, palette choice,
   source subsector, and source sector; and
4. the artifact reports sky and masked-middle counts separately rather than
   silently rendering either through this policy.
