# Classic Doom Renderer Dataflow And Tokimu Preparation Seam

## Status

This is primary-source analysis for the DOOM campaign and AR-0030. It explains
how the released Linux Doom renderer determines and emits visible walls,
planes, sky, masked walls, and sprites, then identifies the smallest plausible
Doom-private seam before `tokimu-render`.

It does **not** admit Doom vocabulary into `tokimu-render`, select a stable
Tokimu render framework, or authorize copying GPL source into Tokimu.

## Sources Inspected

The analysis used temporary, untracked reference checkouts. They are not
Tokimu dependencies or submodules.

| Implementation | Revision | Role |
| --- | --- | --- |
| [id Software DOOM](https://github.com/id-Software/DOOM/tree/a77dfb96cb91780ca334d0d4cfd86957558007e0/linuxdoom-1.10) | `a77dfb96cb91780ca334d0d4cfd86957558007e0` | Released Linux Doom 1.10 primary source |
| [Chocolate Doom](https://github.com/chocolate-doom/chocolate-doom/tree/353cf5001dfd5777c13327010fa58acb57b913b2/src/doom) | `353cf5001dfd5777c13327010fa58acb57b913b2` | Maintained source-faithful control |

The principal files are:

- [`r_main.c`](https://github.com/id-Software/DOOM/blob/a77dfb96cb91780ca334d0d4cfd86957558007e0/linuxdoom-1.10/r_main.c), which orders the frame;
- [`r_bsp.c`](https://github.com/id-Software/DOOM/blob/a77dfb96cb91780ca334d0d4cfd86957558007e0/linuxdoom-1.10/r_bsp.c), which traverses source topology and clips horizontal wall ranges;
- [`r_segs.c`](https://github.com/id-Software/DOOM/blob/a77dfb96cb91780ca334d0d4cfd86957558007e0/linuxdoom-1.10/r_segs.c), which realizes wall tiers, changes vertical coverage, and marks planes;
- [`r_plane.c`](https://github.com/id-Software/DOOM/blob/a77dfb96cb91780ca334d0d4cfd86957558007e0/linuxdoom-1.10/r_plane.c), which groups and draws retained floor, ceiling, and sky spans; and
- [`r_things.c`](https://github.com/id-Software/DOOM/blob/a77dfb96cb91780ca334d0d4cfd86957558007e0/linuxdoom-1.10/r_things.c), which applies retained wall clipping to sprites and deferred masked middles.

The released source is GPL-2.0. This record extracts observable behavior and
control/data-flow facts. Any later implementation must be original Tokimu code
or receive an explicit licensing decision; this analysis is not permission to
copy source text.

## Executive Finding

Classic Doom does not first construct a complete world-space scene and then
ask a visibility filter which objects to submit. It incrementally constructs
the visible presentation while traversing the BSP near-to-far:

```text
source/runtime snapshot + view
        ↓
near-first BSP traversal
        ↓
horizontal SEG admission against accumulated solid ranges
        ↓
for every surviving wall range and every covered screen column:
    derive wall tiers
    draw/retain visible wall portions
    update upper/lower vertical clip limits
    mark the floor/ceiling intervals that remain visible
        ↓
draw retained plane spans (sky is a plane-paint special case)
        ↓
draw sprites and deferred masked middles using retained wall clips
```

The crucial state is not a set of visible sectors or whole meshes. It is an
ordered, viewer-relative coverage ledger whose wall and plane effects are
coupled.

Therefore a correct Tokimu precursor can be built, but **a Boolean preselection
filter over complete global geometry is not expressive enough**. The Doom
stage must be allowed to turn one source contribution into zero, one, or many
view-local presentation occurrences and to retain bounded plane coverage.

## Frame Lifecycle

`R_RenderPlayerView` in `r_main.c` gives the authoritative order:

1. `R_SetupFrame` snapshots player position, eye height, angle, lighting, and
   trigonometric view state.
2. `R_ClearClipSegs` resets the horizontal solid-wall coverage list.
3. `R_ClearDrawSegs` resets retained wall/sprite-clipping records.
4. `R_ClearPlanes` resets `ceilingclip`, `floorclip`, visplanes, and openings.
5. `R_ClearSprites` resets visible sprite state.
6. `R_RenderBSPNode` traverses source topology and processes walls while
   accumulating plane and sprite-clipping state.
7. `R_DrawPlanes` paints the accumulated floor, ceiling, and sky coverage.
8. `R_DrawMasked` draws sprites and deferred masked middle textures.

This order is semantic evidence. Planes are not an independent pass over all
subsector polygons, and sky is not a world-space depth shell. Their visible
regions are products of the preceding ordered wall traversal.

## BSP Traversal And Horizontal Admission

### Near child first

`R_RenderBSPNode` classifies the viewer against each BSP partition, recursively
visits the near child, then calls `R_CheckBBox` before visiting the far child.

`R_CheckBBox` projects a far-child bounding box to a horizontal screen range
and rejects the child when the current `solidsegs` union already covers the
entire range. This is a source-topology traversal optimization driven by
already processed solid walls. It is not generic AABB occlusion.

### Subsector entry

`R_Subsector`:

- chooses a candidate floor plane when the floor is below the eye;
- chooses a candidate ceiling plane when the ceiling is above the eye, or the
  ceiling is sky;
- adds the sector's sprites; and
- sends each source SEG to `R_AddLine`.

The plane values selected here are only possible destinations. They do not
make the whole subsector floor or ceiling visible.

### SEG classification

`R_AddLine`:

1. projects both SEG endpoints relative to the view;
2. rejects a back-facing SEG;
3. clips its angular range to the horizontal field of view;
4. rejects a range that crosses no screen column;
5. classifies the SEG from current front/back sector state; and
6. sends it to solid or pass-wall clipping.

The source classification includes:

- one-sided walls: solid;
- closed two-sided openings: solid;
- height-changing two-sided windows: pass;
- semantically different but height-equal two-sided boundaries: pass; and
- truly empty trigger lines with equal planes/light and no middle texture:
  rejected.

Dynamic sector heights participate in this decision. A reusable plan cannot
classify a door or platform once from immutable WAD data.

### Solid and pass wall clipping

`R_ClipSolidWallSegment` and `R_ClipPassWallSegment` compare a projected SEG
range with the accumulated sorted `solidsegs` ranges. Both call
`R_StoreWallRange` for every uncovered prefix, internal gap, or suffix.

The difference is authority:

- a solid wall merges its full admitted range into `solidsegs`; and
- a pass wall emits its currently uncovered ranges without closing them.

One SEG may consequently produce `0..N` disjoint horizontal occurrences. A
source SEG identity and a presentation-occurrence identity are different
facts.

## The Coupled Wall And Plane Core

`R_StoreWallRange` plus `R_RenderSegLoop` is the most important seam in the
released renderer.

### Range setup

For one contiguous admitted screen range, `R_StoreWallRange`:

- retains the source SEG in a `drawseg`;
- computes perspective scale and texture-coordinate stepping;
- derives current front/back floor and ceiling heights;
- classifies middle, upper, lower, and masked texture work;
- determines whether floor and ceiling differences need marks;
- applies the paired-sky ceiling rule; and
- calls `R_CheckPlane` before the core column loop when a plane needs marks.

The paired-sky rule is narrow: when both adjacent ceiling flats are sky,
`worldtop` is adjusted to the back-sector ceiling. It prevents an ordinary
upper-wall discontinuity between joined sky ceilings. It does not create a
world-space sky occluder.

### Per-column mutation

For every column in the admitted wall range, `R_RenderSegLoop` uses the current
`ceilingclip[x]` and `floorclip[x]` as a vertical open window. In this single
loop it:

1. intersects the projected front-sector top/bottom with the current window;
2. records any surviving ceiling interval in the selected ceiling visplane;
3. records any surviving floor interval in the selected floor visplane;
4. emits the visible one-sided middle or two-sided upper/lower wall tier;
5. mutates `ceilingclip[x]` and/or `floorclip[x]` to reflect that wall tier;
6. saves masked middle texture columns for deferred drawing; and
7. advances perspective, height, texture, and lighting state.

A one-sided middle closes both vertical limits for its column. Upper and lower
tiers close only their respective sides. A two-sided opening leaves a bounded
window for later source contributions.

This explains the failed Tokimu candidate. Horizontal occurrence domains do
not contain enough information to decide complete walls or planes. The result
depends on the **ordered vertical state at each admitted horizontal location**.

### Plane instance splitting

`R_FindPlane` initially groups ordinary planes by height, flat identity, and
light level. Sky planes deliberately normalize height and light so all sky
destinations share one semantic key.

`R_CheckPlane` then distinguishes semantic key from presentation instance. If
a new horizontal range overlaps columns already written in the selected
visplane, it creates another visplane with the same key rather than manufacture
one overlapping plane.

Consequently:

```text
same sector       does not imply same presentation plane
same plane key    does not imply same presentation instance
same source SEG   does not imply one presentation occurrence
```

## Plane And Sky Emission

After BSP traversal finishes, `R_DrawPlanes` consumes only the coverage that
the wall loop wrote into visplane `top[x]` and `bottom[x]` bounds.

For an ordinary floor or ceiling, `R_MakeSpans` converts changes between
adjacent column bounds into horizontal raster spans, and `R_MapPlane` maps
those spans onto the source plane.

For sky, `R_DrawPlanes` instead samples the sky texture from view angle for
each retained sky column and paints only the retained `top..bottom` interval.

Thus:

```text
sky texture data
    does not decide visibility

sky flat identity
    selects special paint for already retained plane coverage

wall traversal + vertical clip state
    decides which sky intervals exist
```

This directly falsifies both earlier global sky-depth approaches:

- a world-space sky wall grants sky occlusion authority that Doom did not
  assign to it and can incorrectly hide nearby geometry such as the hut; and
- a complete sky ceiling tile paints or writes depth without reproducing the
  ordered wall/plane coverage that excluded unrelated distant geometry.

## Masked Middles And Sprites

Masked two-sided middle textures do not close `solidsegs`. Their texture
columns and top/bottom wall clips are retained in `drawseg` records.

`R_DrawMasked` sorts visible sprites, clips them against retained drawsegs,
draws sprites back-to-front, then walks remaining drawsegs in reverse to draw
masked middle textures. Player sprites are last.

This supports the AR-0023 result: categorical alpha consumption does not make a
surface a coarse occluder. A Doom preparation stage must retain masked work and
its bounded clips separately from opaque solid authority.

## Why The Current Ordered-Occurrence Candidate Fails

The current candidate preserved several true facts:

- near-first source traversal;
- source SEG identity;
- `0..N` horizontal occurrence domains;
- continuous source-relative interpolation;
- plane destination identities; and
- conservation through lowering.

The canonical E1M1 result nevertheless removed required walls, floors,
ceilings, and junction regions. The source explains why.

The candidate approximated authority as:

```text
horizontal admitted SEG occurrence
        ↓
associate whole source wall/plane contribution
        ↓
clip geometry by that horizontal domain
```

Classic Doom's authority is instead:

```text
ordered admitted SEG occurrence
        ↓
per-column current upper/lower window
        ↓
wall-tier emission + clip mutation + plane marking
        ↓
plane-instance accumulation
```

Conservation accounting proved that the implementation did not accidentally
drop its own records. It also proved that the records were an insufficient
model of the source behavior. A perfectly balanced lossy representation is
still lossy.

## The Viable Doom-Private Seam

The source supports a seam, but it is preparation/emission rather than whole-
mesh preselection.

```text
Doom decoded source + explicit runtime snapshot + prepared view
        ↓
Doom-private ordered coverage planner
        ↓
Doom-private prepared-view occurrences
        ↓
lower every retained occurrence into bounded Tokimu render declarations
        ↓
prepared-full-submission
        ↓
optional generic conservative post-filter
        ↓
tokimu-render
```

The planner belongs to the Doom campaign/provider. `tokimu-render` receives no
SEG, subsector, visplane, screen-column, sky-flat, or Doom-door vocabulary.

### Required planner input

- exact decoded BSP, subsector, SEG, linedef, sidedef, and sector facts;
- an explicit immutable snapshot of current floor/ceiling heights and other
  source presentation state;
- view position, direction, projection, and viewport identity; and
- the material/texture facts needed to classify ordinary, sky, and masked
  contributions.

### Required private output

The first reference implementation should retain enough detail to prove the
source invariant before optimizing its representation:

- ordered solid/pass horizontal range decisions;
- each `0..N` wall-range occurrence with source identity and visit ordinal;
- per-location upper/lower clip state before and after the occurrence;
- visible middle/upper/lower wall portions and continuous texture domains;
- floor, ceiling, and sky plane marks with plane key and instance identity;
- deferred masked-middle contribution and its retained clip bounds;
- bounded rejection/fail-open reasons; and
- source snapshot, view, viewport, and structural fingerprint.

### Required ordinary lowering

The lowerer may initially emit view-local triangles, but every output must have
a source-correlated destination and all retained coverage must be conserved.
It must not derive survival again from AABBs or whole source meshes.

Walls can be reconstructed from their source line domain and vertical tier
bounds. Planes are harder: their retained coverage can have a staircase-shaped
screen boundary. The study must compare these two bounded realizations:

1. reconstruct view-local plane geometry from the exact retained coverage,
   with wall/plane edges derived from one shared boundary representation; or
2. if ordinary geometry cannot preserve the coverage without cracks or
   forbidden overlap, retain a Doom-private screen-local realization as the
   falsification control while AR-0030 decides whether Tokimu has any reusable
   renderer-facing need.

The second alternative is not automatically a public API proposal.

## A Reference Planner Before An Optimized Provider

The fastest trustworthy path is to build a slow, bounded, headless reference
planner that follows the extracted state transitions. It should be an oracle
for later Tokimu-specific optimization, not a literal port of the C rasterizer.

The reference planner should:

1. fix the initial target at the source-faithful `320 x 200` projection;
2. traverse BSP nodes in source order;
3. classify dynamic solid/pass walls from an explicit runtime snapshot;
4. retain every horizontal range emitted by solid/pass clipping;
5. evolve upper/lower vertical coverage for each admitted range;
6. retain exact wall-tier and plane-mark events before raster output;
7. split same-key plane instances on conflicting coverage;
8. retain sky as a plane-paint classification, never as occluding geometry;
9. defer masked middles with their clip evidence; and
10. emit a deterministic manifest rather than pixels.

Once its manifests agree with the extracted reference behavior, a faster
continuous/run-based provider can be compared against it. Optimization must
not be allowed to redefine the invariant.

## Exact Clip-State Parity Audit

The first implementation audit compared the provider's row conventions with
`R_ClearPlanes` and `R_RenderSegLoop` at released revision
`a77dfb96cb91780ca334d0d4cfd86957558007e0`. The reference checkout remained
temporary and outside Tokimu's dependency graph.

Doom uses a signed, inclusive upper boundary and an exclusive lower boundary:

```text
ceilingclip[x] = -1          // last closed row
floorclip[x]   = viewheight  // first closed row
open rows      = ceilingclip[x] + 1 .. floorclip[x] - 1
```

Tokimu's observer intentionally normalizes that into unsigned values:

```text
upper_open[x]  = ceilingclip[x] + 1
lower_closed[x] = floorclip[x]
open rows       = upper_open[x] .. lower_closed[x] - 1
```

That normalization is valid, but every transition must translate once and
only once. The audit found three concrete integration errors:

- ceiling-plane retention began at `upper_open + 1`, dropping row zero and one
  row after every previous upper closure;
- the no-upper/marked-ceiling path applied Doom's `-1` a second time even
  though the stored value was already first-open; and
- the reported wall-open interval treated `lower_closed` as an inclusive row,
  although the last open row is `lower_closed - 1`.

The provider and the retained legacy diagnostic copy now use the normalized
rules explicitly. A focused regression proves initial rows `0..199`, initial
ceiling-plane rows `0..39`, upper-wall advancement to first-open row `64`, the
no-upper transition to projected row `40`, and terminal one-sided closure.

This repair is source-parity work, not evidence that the current 320-column
inverse-projection lowering is the final prepared-submission representation.
It prevents edge cracks caused by bookkeeping errors while preserving the
larger falsification question: exact Doom plane coverage is viewer-relative
screen coverage, and ordinary world geometry may not be able to carry it
without a different Doom-private lowering.

## Next Executable Evidence

### Gate 1: synthetic source protocol

Run the existing synthetic fixtures through the reference planner and retain
the complete coverage ledger:

- paired sky;
- one-sky negative;
- vertical aperture and partial vertical occlusion;
- shared plane key with conflicting instances;
- dynamic door snapshots;
- dynamic platform snapshots;
- near-plane and thin-projection fail-open controls; and
- cutout non-occluder.

Each fixture must state which source occurrence changed which horizontal and
vertical bounds and which wall/plane/masked outputs survived.

### Gate 2: E1M1 fixed-pose trace

At source spawn, retain a waterfall rather than only a final draw count:

```text
source BSP visits
  → admitted/rejected SEG ranges
  → wall-tier fragments
  → vertical clip mutations
  → floor/ceiling/sky marks
  → split plane instances
  → ordinary lowered declarations
  → renderer handoff
```

The known spawn-room, hut/window, courtyard, close-wall, and sky-boundary
suspects need source identities in that trace.

### Gate 3: prepared full submission

Submit **all** outputs of the reference planner. Do not apply AABB/frustum or a
second Doom selector. Compare it with global full submission at fixed poses and
during bounded camera movement.

Acceptance requires no unexplained loss, overlap, crack, or forbidden far
geometry. A lower draw count is not evidence of correctness.

### Gate 4: optimized private provider

Only after Gate 3 succeeds, replace exact arrays with continuous intervals,
runs, polygons, or another cheaper Doom-private representation one dimension
at a time. Compare every manifest against the reference planner.

### Gate 5: generic post-filter

Only after the prepared submission is clean may the existing AABB/frustum
stage consume it. That filter must preserve occurrence order and fail open. It
may remove additional definitely off-frustum work; it cannot repair or recreate
source contributions.

## What Is General And What Is Not

Potentially reusable facts for AR-0030:

- callers/providers may prepare view-local presentation before handoff;
- one source identity may yield multiple presentation occurrences;
- resource, source, occurrence, view, and submission identities differ;
- preparation and renderer failures need separate attribution;
- final renderer submissions can remain ordinary and bounded; and
- conservative generic selection may run after authoritative preparation.

Doom-private facts:

- BSP/SEG traversal rules;
- `solidsegs` behavior;
- upper/lower column coverage;
- visplane grouping/splitting;
- paired-sky behavior;
- Doom masked-middle rules; and
- the source-faithful reference viewport/raster protocol.

Quake and other campaigns must decide whether the reusable lifecycle facts are
enough to justify a stable Tokimu submission framework. Doom cannot do that
alone.

## Conclusion

The requested clean submission is achievable, but not by asking a conventional
prefilter to choose among the current complete meshes.

The source-backed path is:

> **Reconstruct Doom's ordered viewer-relative coverage as a Doom-private frame
> plan, lower every retained wall/plane/sky/masked occurrence into a complete
> prepared Tokimu submission, and only then apply optional generic conservative
> selection.**

This is a real seam. It keeps Doom authority outside the renderer while giving
Tokimu an ordinary, inspectable handoff. It also gives the campaign a reference
oracle capable of proving whether a faster preselection/preparation strategy is
actually equivalent rather than merely visually plausible at one pose.
