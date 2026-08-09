# Doom Shareware Package Inspection Observation

## Scope

Manual, non-CI Slice 2 evidence for the reviewed canonical package:

```text
corpus/assets/archive/DOOM/DOSBOX_DOOM.ZIP
member: DOOM1.WAD
```

The command retained the ZIP as a single in-memory Resource Space resource,
opened a read-only archive-derived view, read the selected member transiently,
and inspected it with `doom-wad-provider`:

```powershell
cargo run -q -p hello-wad-inspect -- --zip corpus/assets/archive/DOOM/DOSBOX_DOOM.ZIP DOOM1.WAD E1M1
```

The retained top-down topology artifact was generated independently with:

```powershell
cargo run -q -p hello-wad-inspect -- --zip corpus/assets/archive/DOOM/DOSBOX_DOOM.ZIP DOOM1.WAD --map-svg E1M1 corpus/lib/doom-wad-package/results/E1M1-source-topology.svg
```

This is an observation artifact, not a CI fixture and not authorization to
deploy either the package or a loose WAD.

## Result

| Observation | Value |
| --- | --- |
| Archive identity | Existing canonical ZIP SHA-256 remains `9ed3172e728d403962f874eaba93b4b973af1e57a8608bd803fc6e02d137fbc6` |
| Resource Space archive fingerprint | BLAKE3 `4b25c413b21334cf88c6661e58aab41f76a6414514cab371e1d838c3e698203b` |
| Selected member | `DOOM1.WAD` |
| WAD source label | `corpus/assets/archive/DOOM/DOSBOX_DOOM.ZIP:DOOM1.WAD` |
| WAD kind | `IWAD` |
| Lump count | 1,264 |
| Declared lump bytes | 4,175,556 |
| WAD BLAKE3 | `2a0c5f3c001228980409e483c06c5510e5a1f392d9a3551bc955b55b04aa930b` |
| Projected namespaces | sprites: 483 members; patches: 167 members; flats: 56 members |
| Global raster observations | 14 PLAYPAL palettes; 34 COLORMAP index-remapping tables; first palette entry RGB `(0, 0, 0)`; palette 0 canonical 256x1 index ramp lowered to RGBA8 fingerprint `1e8014f87f760ee5`; visual artifact [`PLAYPAL-0.ppm`](PLAYPAL-0.ppm) |
| Decoded patch observation | `WALL00_1`: 64x144, 64 columns, 128 posts, 9,216 opaque pixels, origin `(31, 139)`; explicitly lowered with palette 0 to RGBA8 fingerprint `616af5f6877e97a5`; visual artifact [`WALL00_1-palette-0.ppm`](WALL00_1-palette-0.ppm) |
| Texture catalog observation | 350 PNAMES entries; 125 `TEXTURE1` records; 598 patch references |
| Composed texture observation | `STARTAN3`: 128x128, 16,384 opaque indexed pixels; explicitly lowered with palette 0 to RGBA8 fingerprint `26b26979f038b245`; visual artifact [`STARTAN3-palette-0.ppm`](STARTAN3-palette-0.ppm) |
| Decoded flat observation | `FLOOR0_1`: 64x64, 4,096 indexed pixels; explicitly lowered with palette 0 to RGBA8 fingerprint `f8b8705bd27d91c5`; visual artifact [`FLOOR0_1-palette-0.ppm`](FLOOR0_1-palette-0.ppm) |
| Sprite naming observation | 611 classic frame/rotation observations, including paired rotations; ordered structural fingerprint `54b2c32376eedddc`; visual artifact [`TROOA1-palette-0.ppm`](TROOA1-palette-0.ppm) |
| Selected map | `E1M1`, marker index 6, local lump indices 7 through 16 |
| Top-down source topology artifact | [`E1M1-source-topology.svg`](E1M1-source-topology.svg), 146,512 bytes, SHA-256 `3df7c26f3eb768a93fa4de8cd858a3c22bb7d3d0841630598f0d835275aab3ee`; it contains all 475 decoded `LINEDEFS` (one-sided teal, two-sided gray), 137 ordinary raw `THINGS` (amber), the reviewed player-one source thing (cyan), and the raw 36 by 23 classic `BLOCKMAP` extent (dashed violet). Every record carries its source identity in SVG metadata; linedef tooltips retain raw flags, special, tag, sidedef references, and the intentionally non-semantic diagnostic sector selection. It does not claim visible geometry, gameplay, collision, or renderer behavior. |
| Top-down sector-color artifact | [`E1M1-source-sectors.svg`](E1M1-source-sectors.svg), 150,146 bytes, SHA-256 `5b17f93a1905e739cc30df42da0eb4c09c70e6929572cf1138b09bfe36a9e903`; it uses a deterministic palette selected from each linedef's right sidedef sector, falling back to its left side only when necessary. Raw sidedef references and the selected source sector remain in each tooltip. This is a diagnostic color key, not an assertion of sector fills, visible walls, or portal behavior. |
| Top-down wall-normal artifact | [`E1M1-wall-normals.svg`](E1M1-wall-normals.svg), 342,771 bytes, SHA-256 `8ac6551f75ffc5a319f74fb76e56b085df3e0f490305c5f732ea78eab8d74dcc`; cyan arrows retain WAD slot 0/right/front normals and magenta arrows retain slot 1/left/back normals. The SVG Y inversion mirrors their apparent screen-side direction. The arrows match the shared lowered-wall winding and diagnose headless source-side orientation only; they make no lighting, culling, visibility, or collision claim. |
| Required map-lump validation | `THINGS`, `LINEDEFS`, `SIDEDEFS`, `VERTEXES`, `SEGS`, `SSECTORS`, `NODES`, `SECTORS`, `REJECT`, and `BLOCKMAP` each occurred once and in the admitted order |
| Decoded map core | 138 things; 467 vertices; 475 linedefs; 648 sidedefs; 85 sectors; 732 segs; 237 subsectors; 236 nodes |
| Raw thing inventory | 30 distinct numeric kinds across the 138 decoded `THINGS`, with 6 distinct raw flag sets. Deterministic kind counts: `[1:1,2:1,3:1,4:1,9:16,10:2,11:5,12:2,15:4,24:7,35:2,48:2,2001:1,2002:1,2003:1,2007:2,2008:3,2011:1,2012:3,2014:13,2015:25,2018:1,2019:1,2028:8,2035:6,2046:3,2048:6,2049:6,3001:4,3004:9]`. This is source inventory only; no gameplay interpretation is asserted. |
| Player-one start observation | Exactly one classic player-one start (`THING` type `1`) resolves: source `THINGS` record 0, position `(1056, -3616)`, angle `90`, flags `0x0007`. Missing or duplicate player-one starts are structured importer diagnostics; this is not yet runtime spawn state. |
| Player-one BSP/sector observation | The reviewed start locates uniquely through the retained strict BSP paths in source subsector 103, whose source-traceable sector is 38. A point on a BSP partition plane is intentionally rejected rather than assigned by an unreviewed tie-break. |
| Player-one vertical source interval | Resolved sector 38 has raw floor height `0` and ceiling height `72`. This evidence does not select a player height, clearance requirement, or spawn elevation policy. |
| Raw special inventory | Nonzero linedef codes: `[1:8,11:1,36:1,48:8,88:1]`; nonzero sector codes: `[1:1,7:4,8:2,9:3,12:1]`. These are source counts only; no activation or runtime behavior is assigned. |
| Special-semantics disposition | The bounded code set is classified with original-source evidence and minimum Tokimu ownership in [`E1M1 special semantics evidence`](../../../../docs/Plans/DOOM/E1M1%20special%20semantics%20evidence.md). All remain unsupported observations until their listed runtime/application/presentation boundaries exist. |
| Core reference validation | Every decoded linedef, sidedef, seg, subsector, and BSP-child reference was in range |
| REJECT validation | 904 bytes, exactly the 904-byte minimum for 85 sectors |
| BLOCKMAP validation | origin and 36 by 23 grid accepted; 828 row-major cells retain their validated linedef candidate lists, with 828 unique lists and 973 in-range references. This remains source broad-phase evidence, not collision behavior. |
| Player-one BLOCKMAP observation | The reviewed start at `(1056, -3616)` lies in source cell 338 (column 14, row 9), whose list contains 6 candidate linedefs. This demonstrates a measured first-proof broad-phase option against the full 475-linedef map, without selecting traversal or collision policy. |
| Headless wall-topology audit | 475 source-traceable candidates: 302 one-sided, 173 two-sided, including 6 two-sided candidates whose sides reference the same sector; no zero-length or side-less linedefs admitted |
| BSP topology audit | All 237 subsectors are reachable from the root BSP node; retained root-to-leaf partition paths range from depth 5 through 18 |
| BSP-region observation | The admitted classic node side convention clips the map extent to 237 bounded candidate regions with 959 boundary vertices; 34 of 1,464 source `SEG` endpoints do not satisfy the idealized integer half-planes, with maximum outside distance 512 map units, so they remain explicit topology evidence rather than a tolerable rounding difference |
| Subsector sector ownership | All 237 subsectors resolve one consistent, source-traceable sector through their `SEG` direction and linedef sidedef; all 85 decoded sectors occur |
| Strict `SEGS`-only loop-closure audit | 55 of 237 subsectors form a strict closed source-seg loop. The remaining 182 are retained as diagnostics: 104 have fewer than three segs and 78 are open; none are ambiguous or degenerate. This neither repairs nor invalidates the separately bounded BSP-region evidence. |
| Headless floor/ceiling lowering | 970 renderer-neutral BSP-leaf triangles: 485 floor and 485 ceiling candidates, each retaining source subsector and sector identities and using that sector's decoded height |
| Headless one-sided-wall lowering | 604 renderer-neutral, full-height untextured triangles from 302 one-sided linedefs, each retaining source linedef, sidedef, and sector identities |
| Headless two-sided-band lowering | 398 renderer-neutral height-discontinuity triangles: 188 upper and 210 lower. Each retains the authored sidedef texture name; source-texel coordinates are available only through the separately admitted textured-triangle lowering |
| Headless two-sided-middle lowering | 26 renderer-neutral triangles from 13 authored middle-texture observations. Each is clipped to its positive shared sector opening; closed/inverted openings emit no geometry. This does not choose alpha mode, portal behavior, or collision semantics. |
| Vertical topology audit | 85 decoded sectors include 4 without positive raw floor-to-ceiling clearance. Of 173 two-sided source relationships, 9 have no positive shared vertical opening. Both facts remain diagnostics for later portal, middle-texture, and collision policy; no source repair is applied. |
| Two-sided middle-texture inventory | 13 authored middle-texture observations across 4 distinct names; each retains its source side and adjacent-sector vertical opening, but no middle geometry is emitted |
| Sky-surface classification | 74 `F_SKY1` floor/ceiling triangle observations are classified at the Doom geometry boundary; this adds no generic mesh or renderer sky behavior |
| Raw wall texture axes | 613 authored sidedef texture observations retain deterministic U start/end from linedef length plus X offset, and their raw Y offset; no texture-size, V-placement, or pegging rule is asserted |
| Wall texture extent bindings | All 613 authored wall-texture axes resolve to plain named width/height extents derived at the raster-catalog boundary; geometry does not depend on raster implementation types |
| Wall texture source-anchor placements | All 613 authored wall-texture records resolve an original-renderer-compatible `texturemid` world-space anchor from their extent, sidedef row offset, pegging flags, and retained right/left source ownership. |
| Textured ordinary-wall lowering | 1,024 one-sided and two-sided upper/lower/middle triangle records carry deterministic, source-texel U/V coordinates derived from their retained axis and `texturemid` anchor. This makes no material, wrapping, raster-upload, or renderer claim; middle triangles use only the admitted shared-opening clip. |
| Pegging-flag audit | Of 148 authored upper texture axes, 108 carry the classic top-unpegged bit; of 150 authored lower axes, 79 carry the bottom-unpegged bit. The admitted ordinary-wall texture placement applies those flag-derived source anchors; middle-texture treatment remains separate. |
| Permanent loose WAD resource retained | No; the Resource Space contained only the package resource |

The observed directory begins with global resources and then `E1M1`; it
contains paired sprite, patch, and flat namespace markers. The provider did
not reinterpret marker records as file resources or introduce a host-path
meaning for WAD names.

## Consequence

The existing archive and Resource Space seams can carry the reviewed package
to the WAD container provider without a Doom-specific rule or permanent
extraction. Map-local selection and structural diagnostics are also complete.
The complete classic map block is structurally decoded with source-indexed
diagnostics. Palette, COLORMAP, a marker-scoped patch, the texture catalog, and
a composed wall texture, a flat, and sprite frame/rotation names are now decoded.
A headless geometry seam now also retains linedef, sidedef, and sector identity
while classifying basic wall topology. It emits only source-traceable, renderer-
neutral triangle candidates: the six same-sector two-sided observations show
why later wall semantics must not equate a source-side count with visible
geometry. An exploratory `SEGS`-only
subsector-loop reconstruction also remains explicitly unresolved at E1M1
subsector 1 (between source seg records 6 and 4), even after deterministic
source-order and direction recovery. An all-leaf audit shows this is not an
isolated defect: only 55 of 237 leaves close strictly from `SEGS`; the other
182 remain source-indexed diagnostic evidence rather than improvised polygons.
The root-to-leaf BSP paths now show the
required additional partition evidence for every leaf, ranging from 5 to 18
planes. Combining those paths with the finite map extent produces a bounded
candidate region for every leaf, but 34 of 1,464 source `SEG` endpoints fall
outside the idealized integer half-planes by as much as 512 map units. This is
not a floating-point tolerance problem. A `SEGS`-only result—or an unreviewed
rounded BSP region—must not become a floor/ceiling triangulation input. The next
boundary is combining source-traceable sector ownership (which now resolves for
all E1M1 subsectors) with the distinct map-wall and BSP-partition boundary
evidence. The first result of that combination is a headless floor/ceiling
triangle candidate set; it remains separate from texture semantics, wall
lowering, and renderer submission. One-sided full-height walls now lower under
the same headless rule. Two-sided upper and lower height bands now lower with
their source texture names retained, while 13 authored two-sided middle cases
lower only across their positive shared opening. Ordinary one-sided and all
two-sided wall triangles additionally retain deterministic source-texel
coordinates from the original renderer's pegging anchors, without committing
materials, alpha mode, portal behavior, or renderer submission.
`F_SKY1` source surfaces are now likewise explicit evidence, rather than an
implicit renderer convention.
