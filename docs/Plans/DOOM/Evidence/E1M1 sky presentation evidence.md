# E1M1 Sky Presentation Evidence

Status: native panorama observed; distant static-shell sector leak unresolved

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

The static panorama is not a classic Doom screen-space visibility
implementation. Source-valid world geometry remains foreground when it is
submitted. It does not determine whether the original renderer would have
submitted a particular span in a fixed view and does not reopen AR-0025's
no-shared-capability disposition.

## Rejected world-space aperture mask

On 2026-08-13, the corpus temporarily submitted the retained `F_SKY1` flat
meshes after the panorama and before world geometry through a depth-only
pipeline. Native inspection falsified the approach: a horizontal source plane
can bound upward view rays, but it does not express Doom's viewer-relative sky
coverage near the horizon. Distant static-shell sector geometry remained
visible in the sky region. The mask was removed rather than expanded into
unprincipled skirts, depth exceptions, or hidden geometry deletion.

The remaining defect is presentation-model mismatch, not missing `SKY1`
raster data. A future Doom-owned continuation must present retained source sky
spans directly or establish an exact shared wall/plane screen-boundary model.
The rejected mask does not justify a generic renderer sky or visibility API.

On 2026-08-14, a second explicit control submitted *all* retained `F_SKY1`
subsector flats depth-only. It removed the reported distant-sector leaks, but
also masked valid nearby hut geometry. This separates two facts that the first
control had blurred: the source flat identity is correct, while global
submission authority is not. The raw mode remains available only as
`--source-sky-plane-depth-global-control`; the narrower
`--source-sky-plane-depth` mode first admitted source sectors from the current
Doom BSP/vertical-clip sky spans. That narrower control also failed: the hut
remained visible nearby but became masked as the observer backed away. The
mode now reconstructs only the exact retained sky screen cells on their
owning source-sector ceiling heights. This remains a visual falsification
control and does not claim historic pixel parity.

The exact-cell run then separated the paired-sky wall mechanism from ceiling
coverage. The retained paired-sky depth wall clipped valid hut geometry when
viewed from the spawn-room window, so E1M1 no longer presents those meshes
unconditionally. Their source identities remain available to `LOOK`, and the
synthetic paired-sky fixture remains the bounded mechanism control. With the
screen-cell mesh active, a nearby unrelated room still survived through a sky
ceiling interval. This is evidence that exact screen ownership alone is not
enough when ordinary world-space depth order disagrees with Doom's
viewer-relative source reachability; it is not authorization for a broader
hidden wall or whole-sector mask.

## Targeted Classic-Source Finding

The released renderer resolves this case before there is any analogue of a
global depth-tested shell. During near-to-far SEG processing,
`R_RenderSegLoop` marks ceiling/floor plane columns from the live
`ceilingclip`/`floorclip` bounds and then mutates those same bounds for the
current one-sided wall or two-sided upper/lower opening. The paired-sky rule in
`R_StoreWallRange` makes the two sky ceilings share the back-sector top for
this calculation; it does not manufacture an invisible vertical occluder.
Later, `R_DrawPlanes` draws SKY only over the top/bottom intervals already
retained in the sky visplane.

Chocolate Doom preserves this division, providing an independent faithful
implementation control. The source evidence therefore explains both observed
Tokimu failures:

```text
paired-sky world-depth wall
    -> can hide valid foreground hut geometry

source-height sky-cell depth
    -> can lose to unrelated geometry that classic traversal never admitted
```

The missing invariant is not a better depth value. It is shared,
viewer-relative wall/plane coverage with source ordering or an equivalent
Doom-owned preparation that excludes source-unreachable candidates before
generic rendering. This finding does not admit Doom visplanes, `solidsegs`, or
column clip arrays into `tokimu-render`; those remain reference mechanisms and
Doom-provider research.

The synthetic partial-coverage follow-up also falsified whole-source Boolean
filtering as a sufficient replacement. One far SEG occupied `97` diagnostic
columns. A nearer paired-sky interval governed the middle `81` columns
(`[120,200]`), while the same far SEG remained required in `[112,119]` and
`[201,208]`. The next bounded realization therefore needs Doom-owned retained
fragments or intervals. Those runs are evidence about source preparation, not
renderer scissors, pixel identity, or a generic visibility contract.

## Remaining evidence

- Native observation of the paired-sky vertical boundary control is retained
  as a rejected E1M1 presentation mechanism. It successfully blocked one
  farther-sector leak, but both the first double-sided state and the later
  owning-face-only state hid valid hut geometry in real camera paths. Paired
  sky remains source evidence and a synthetic control, not an unconditional
  map-wide depth wall.
- A Doom-owned viewer-relative presentation that removes distant-sector leaks
  without introducing visible false negatives at wall/plane boundaries.
- Browser/WASM realization using the same source raster and declared policy.
- Pressure from another sky consumer before considering any generic contract.

Interactive source-ray inspection retained two concrete lower-aperture leak
identities: ordinary wall linedef 249 / sidedef 348 / sector 56 and ceiling
flat subsector 104 / sector 40. The replay diagnostic reported no paired-sky
depth-boundary intersection for the captured ceiling direction. This is
evidence that the remaining leak is outside the current unequal
paired-sky-ceiling rule; it is not evidence to enlarge, reverse, or make the
linedef-252 boundary double-sided. Live nearest-hit identity may differ from a
static headless replay after doors or platforms change prepared geometry, so
those states remain explicit rather than being hidden by the replay command.
