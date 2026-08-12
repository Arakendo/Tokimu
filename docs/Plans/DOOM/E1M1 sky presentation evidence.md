# E1M1 Sky Presentation Evidence

Status: native visual observation pending; source raster coverage resolved

## Scope

This is a corpus-local experiment over the reviewed E1M1 package. The Doom
geometry provider continues to identify `F_SKY1` source surfaces and to omit
the upper wall between adjacent sky ceilings. The executable separately
composes the episode-one `SKY1` raster and presents it on an enclosure.

The experiment does not:

- treat `F_SKY1` as an ordinary flat texture;
- add Doom terminology or a sky capability to `tokimu-render`;
- claim the original Doom view-dependent sky projection;
- replace AR-0027's explicit purple missing/error presentation; or
- make the panorama part of static candidate selection.

## Native invocation

```powershell
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts --doom-sky --spawn-observer --embedding-north --noclip
```

## Declared presentation

- Source raster: composed `SKY1`, palette zero. The reviewed shareware WAD
  composes to a 256x128 raster with rows 0--119 fully covered and rows 120--127
  fully empty (30,720 covered pixels). The corpus panorama retains the 256x120
  full-width coverage band and rejects partial or internal gaps; it does not
  invent replacement texels or relax normal wall/alpha handling.
- Geometry: 64-segment static panorama cylinder enclosing E1M1.
- Sampling: point; horizontal repeat; vertical clamp.
- Scheduling: submitted before ordinary world geometry.
- Depth: `LessEqual`, no depth writes.
- Failure behavior: `--doom-sky` and `--diagnostic-sky-omissions` are mutually
  exclusive; missing, partial, or internally uncovered `SKY1` coverage is an
  explicit failure.

The static panorama is a depth-tested background, not a classic Doom
screen-space visibility implementation. Source-valid world geometry remains
foreground when it is submitted. AR-0025 retains the later Doom BSP/clipping
comparison required to determine whether the original renderer would have
submitted a particular span in a fixed view.

## Remaining evidence

- Native inspection of horizon placement, panorama seam, and exterior gaps.
- Browser/WASM realization using the same source raster and declared policy.
- Pressure from another sky consumer before considering any generic contract.
