# E1M1 Hut Sky-Boundary Evidence

## Scope

This record investigates the wall-like fragment visible above the small hut in
the exterior E1M1 courtyard. It distinguishes malformed geometry, missing
generic visibility, incorrect sector heights, and a missing Doom presentation
rule without treating a UZDoom screenshot as source truth.

The reproducible source report is:

```powershell
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --hut-wall-candidates-report
```

The interactive console `LOOK` command identified the visible fragment as
`wall:252:STARTAN3`, sourced from linedef 252, sidedef 353, sector 5.

## Source Evidence

Linedef 252 is two-sided and spans `(1984, -3648)` to `(1376, -3648)`.
Its adjacent source sectors are:

| Side | Sidedef | Sector | Floor / ceiling | Floor / ceiling flat | Upper texture |
| --- | ---: | ---: | ---: | --- | --- |
| right/front | 353 | 5 | `-56 / 216` | `FLOOR7_1 / F_SKY1` | `STARTAN3` |
| left/back | 354 | 20 | `-56 / 24` | `FLOOR7_1 / F_SKY1` | `-` |

The previous lowerer therefore emitted a right/front upper wall band from
height 24 through 216. Its two triangles were internally consistent with the
sector heights and source identity.

## Finding

This is not a culling failure and not a degenerate triangle. It is a missing
classic Doom sky-adjacency rule: when both neighboring ceiling flats are
`F_SKY1`, the source renderer presents a continuous sky opening across their
height discontinuity rather than the higher sector's ordinary upper wall.

The finding is therefore classified as source-specific presentation/lowering
behavior. Doom owns the `F_SKY1` comparison and omission; generic meshes,
materials, cameras, and visibility selection must not learn Doom sky names.

## Bounded Repair

The Doom geometry provider now omits only an upper two-sided wall band when
both adjacent sector ceilings are `F_SKY1`.

- Lower wall bands remain unaffected.
- A boundary with only one `F_SKY1` ceiling still emits its ordinary upper
  band.
- One-sided walls remain unaffected.
- No renderer or generic visibility contract changes.

Focused tests retain both the dual-sky omission and the one-sky control. The
hut report retains the exact linedef, sidedefs, sectors, heights, flats, and
generated spans needed to reproduce this case.

## Retained Depth-Boundary Control

The color omission alone exposed a second role previously collapsed into the
same wall. Linedef 252's upper span is not visible wall color, but it bounds the
near sky aperture against unrelated farther static-shell geometry. The Doom
provider therefore also retains a separate, source-labelled paired-sky
boundary only when both adjacent ceilings are `F_SKY1` and their heights
differ.

The E1M1 composition draws the sky panorama first, submits these boundary
triangles with color writes disabled and depth writes enabled, then submits
ordinary world geometry. This does not restore `STARTAN3`, inspect texture
alpha, or alter generic visibility. Equal-height paired-sky and one-sky
controls produce no boundary.

Native inspection confirmed that the exact linedef-252 span blocks the farther
sector geometry previously visible through it. The first depth pipeline used
no face culling and exposed an overreach: the same span could hide the hut when
viewed through adjacent sky/ceiling geometry from its opposite source side.
The control now uses the retained source-owned winding with back-face culling;
that narrower visual observation remains open. Separate lower sky-aperture
leaks toward the main buildings also remain and are not evidence that this
wall-shaped span should be enlarged.

Subsequent `LOOK` evidence made that separation concrete. One direction
selected wall linedef 249 / sidedef 348 / sector 56; another selected ceiling
subsector 104 / sector 40. The headless source-ray comparison found no retained
paired-sky depth boundary on the latter direction. The remaining aperture is
therefore a distinct Doom viewer-relative wall/plane presentation case, not a
missing triangle from the existing linedef-252 boundary control.

The investigation was then broadened before any further geometry repair.
Interactive captures selected ordinary geometry from sectors 24, 40, 49, 56,
and 72 through the same outdoor aperture. `LOOK` now also compares the ray with
the retained omitted `F_SKY1` flat meshes and emits a bounded Stage-3B source
trace. A reproducible wall-249 ray reported:

```text
viewer subsector:       141
target subsectors:      190, 216
target SEG records:     560, 657
reached target leaves:  190
admitted target SEGs:   none
subsector 216:          pruned at node 219 by closed [0,319] range
paired-sky boundary:    none
source sky plane:       none
global-shell result:    wall 249 is the nearest ordinary prepared hit
```

This is the causal distinction the narrower control needed. Wall 249 is valid
source geometry and valid static-shell geometry, but it is not admitted by the
bounded Doom viewer-relative source protocol for that ray. Broadening the
linedef-252 depth span, restoring a visible wall, or inventing a generic sky
occluder would hide the symptom while discarding this evidence.

## Live Source-Protocol Control

The follow-up `--doom-seg-classic-dynamic` mode now tests that distinction in
presentation without adding another mask. It prepares stable SEG-derived wall
draws once and recalculates the Doom BSP/solid-range admission set from the
live source observer. Wall draws follow admitted SEG identity; ordinary flat
draws follow reached subsector identity as a deliberately coarse interim plane
control. Unclassified draws and source walls whose SEG textures lack prepared
materials remain enabled as explicit fail-open fallbacks.

The first bounded source-spawn observation retained `2,095` candidates and
submitted `496` (`482` opaque and `14` owning-side cutouts). No mesh upload or
replacement occurred on the warm frame. This does not close the visual issue:
the mode must still prove that wall 249 disappears while close walls, camera
turns, planes, doors, and the earlier false-negative poses remain intact.

Native free-movement inspection rejected that first composition. Crossing BSP
subsector boundaries caused floor portions around the spawn pillars to vanish;
opening the first door exposed the sky enclosure until the observer crossed
the doorway; and the hut aperture retained unrelated distant geometry despite
improved masking. These are three distinct failures rather than one tuning
target:

- reached subsectors are not equivalent to Doom plane/visplane coverage;
- immutable decoded sector heights are stale while a door or platform moves;
- horizontal solid-range wall admission does not establish the exact shared
  wall/plane/sky boundary needed by the outdoor aperture.

The follow-up keeps the control opt-in and rejected as a complete presentation
path. Whole-subsector flats now fail open instead of following reached leaves,
and each traversal receives a temporary source-map snapshot containing the
already-authoritative active door/platform heights. Those corrections remove
known false premises; they do not claim to solve the remaining hut leak. That
case still requires exact Doom-owned screen-span/plane evidence rather than a
broader depth patch or another per-object culling rule.
