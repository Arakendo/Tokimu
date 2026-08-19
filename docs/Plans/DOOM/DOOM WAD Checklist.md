# DOOM WAD Checklist

## Status

Proposed on 2026-08-06; reconciled with the implemented corpus on 2026-08-19.
The bounded WAD, map, raster, geometry, native presentation, movement, and
selected E1M1 runtime-special proofs now exist as corpus-owned providers and
consumers. No Doom gameplay-compatibility capability or Doom-specific renderer
or trusted-core contract is admitted to Tokimu.

The work should begin as corpus evidence. It must not make `tokimu-core`, the
renderer, or the Resource Space understand Doom-specific concepts.

## Purpose

Use a bounded Doom WAD consumer to pressure Tokimu's existing resource, raster,
geometry, rendering, input, audio, runtime-observation, and WASM boundaries.

The first useful outcome is intentionally smaller than a Doom source port:

> Mount the reviewed Doom shareware ZIP through Resource Space, resolve its
> `DOOM1.WAD` member, select `E1M1`, render its static world, and let a player
> walk through it with explicit diagnostics for unsupported semantics.

User-supplied loose WADs remain a useful later consumer path. They are not the
canonical corpus or hosted-distribution boundary for the first proof.

Faithful gameplay, demo compatibility, multiplayer, and broad source-port
compatibility are later evidence targets, not requirements for the first
viewer.

## Current Local Evidence

The maintainer supplied two reviewed package fixtures under
`corpus/assets/archive/DOOM/`, with their inventory and provenance metadata
under `corpus/assets/DOOM/`:

- `DOSBOX_DOOM.ZIP`: 2,357,547 bytes, SHA-256
  `9ed3172e728d403962f874eaba93b4b973af1e57a8608bd803fc6e02d137fbc6`;
  it contains `DOOM1.WAD` and `README.TXT`, but no separate `LICENSE.DOC`;
- `DOSBOX_HERETIC.ZIP`: 2,794,870 bytes, SHA-256
  `f4ca7bffd27ab3e671beb3cadee7a39c3b7b8c330e5e0591f4a42c2f7b6bb944`;
  it contains `HERETIC1.WAD`, `LICENSE.DOC`, `VENDOR.DOC`, and `README.TXT`.

The extracted WAD observations remain useful for importer validation:

- `DOOM1.WAD`: 4,196,020 bytes, `IWAD`, 1,264 lumps, directory offset
  4,175,796, SHA-256
  `1d7d43be501e67d927e415e0b8f3e29c3bf33075e859721816f652a526cac771`;
- `HERETIC1.WAD`: 5,120,920 bytes, `IWAD`, 1,374 lumps, directory offset
  5,098,936, SHA-1 `b4c50ca9bea07f7c35250a1a11906091971c05ae` and
  SHA-256
  `3ab2f21828877e49e5eb3220785aaf8798050b7c4132003b5db7b8f3678bede4`.

The `doom1.wad` size and MD5
`f0cefca49926d00903cf57551d901abe` match the Doom 1.9 shareware IWAD. Debian's
preserved copyright record permits unchanged, no-fee distribution of the Doom
shareware WAD and records John Carmack's clarification that this WAD is freely
distributable. This data remains non-free and is not open source or public
domain. The finding does not permit derivative game data and does not cover
commercial Doom WADs or Heretic.

The Doom package's `README.TXT` contains the shareware release documentation
and a request not to modify shareware levels. It does not contain a standalone
copy of the Limited Use Software License Agreement. The Debian source record
is therefore supplemental redistribution evidence, not a package member and
not something Tokimu should represent as having shipped in the original ZIP.

The Heretic package has a stronger self-contained provenance chain. Its
`LICENSE.DOC` explicitly permits royalty-free electronic distribution only in
compressed form, while `VENDOR.DOC` preserves vendor-distribution context.
The verified SHA-1 values are:

- `LICENSE.DOC`: `c97b176fe0458039219eb426ad315dc5ff155324`;
- `VENDOR.DOC`: `a4360e93169602b3daa7e87364e1c341cbc02282`.

The ZIP packages are the canonical corpus artifacts. Extracted trees and WADs
are local inspection products and remain ignored so that repository and
website consumers cannot accidentally separate package members from their
documentation. `DOOM1.WAD` is the initial semantic target;
`HERETIC1.WAD` is comparative WAD-container pressure and must not silently
expand this plan into Heretic compatibility. Both WAD directories contain
`E1M1` through `E1M9`; an initial bounded inventory found no negative or
out-of-file lump ranges. The importer must still validate those facts
independently on every load.

On 2026-08-19 the native Doom corpus path also loaded `HERETIC1.WAD` E1M1 from
its reviewed ZIP. That is useful container/map/raster portability evidence,
not Heretic presentation admission. The maintainer observed a structurally
coherent scene and sky but materially brighter surfaces. The current consumer
uses palette zero with sRGB upload and decodes `COLORMAP` without applying
classic sector/colormap lighting, so resolving that observation crosses the
still-open presentation-lighting ownership question below; it is not licensed
as an implicit Heretic compatibility fix by this checklist.

The licensing review also establishes a boundary that must remain explicit:
Doom and Heretic engine source licensing is separate from game-data licensing.
Doom's later GPLv2 source release, and the later GPL rerelease of Heretic
source, do not license either WAD. Doom's shareware data is admitted only under
its own preserved distribution evidence. Heretic is admitted only as its
complete compressed shareware package under the original documents retained
inside that package. These records describe the reviewed corpus boundary; they
are not a general legal conclusion about other releases, modified packages,
commercial data, or derivative assets.

## Architectural Thesis

```text
shareware ZIP package
    |
    v
archive provider
    bounded member inventory and extraction
    |
    v
Resource Space
    package/member identity + logical WAD bytes
    |
    v
WAD container provider
    directory + bounded lump resources
    |
    v
Doom semantic importer
    maps + textures + things + specials
    |
    +-------------------+
    |                   |
    v                   v
Tokimu world data       presentation requirements
collision + state       geometry + materials + sprites
    |                   |
    +---------+---------+
              v
        runtime + renderer
```

- The archive provider owns bounded ZIP parsing, member validation, and
  decompression mechanisms.
- Resource Space owns package/member identity, logical addressing, folders,
  bytes, and visibility without learning Doom semantics.
- The WAD provider owns headers, directories, lump bounds, and WAD namespaces.
- The Doom importer owns Doom map, texture, thing, and special semantics.
- Tokimu world/runtime owns admitted simulation truth and deterministic updates.
- Presentation lowering owns conversion from Doom semantics to geometry,
  materials, sprites, and camera requirements.
- The renderer owns GPU resources, draw execution, and pixels.
- Audio providers own decoding and playback mechanisms; Doom owns which sound
  or music event is requested.

The renderer must never know what a linedef, sector, thing, IWAD, or map lump
means.

## Fixture And Legal Boundary

- [x] Record both canonical ZIP hashes and member inventories.
- [x] Record the observed `doom1.wad` byte length, header, directory facts,
      and hash.
- [x] Match the observed `doom1.wad` size and MD5 to Doom 1.9 shareware.
- [x] Inventory `heretic1.wad` as container-only comparative evidence.
- [x] Record the source-code/game-data licensing distinction and review
      references.
- [x] Preserve a stable Debian source record of the Doom shareware distribution
      terms and rights-holder clarification.
- [x] Determine that the unchanged Doom shareware WAD may be distributed
      without charge or consideration.
- [x] Preserve Heretic's original `LICENSE.DOC`, `VENDOR.DOC`, and README in
      the canonical archive and verify their hashes.
- [x] Record Heretic's compressed-format electronic-distribution requirement.
- [x] Keep extracted WADs and package trees ignored and local-only.
- [x] Make the complete ZIP packages, rather than naked WADs, the canonical
      corpus and potential hosted-distribution artifacts.
- [x] Keep Doom's supplemental Debian license record distinct from the members
      that shipped in the reviewed Doom ZIP.
- [x] Add a deterministic, source-hash-checked package preparation script that
      omits DOS executables and unrelated support tools from internal corpus
      runs.
- [x] Keep compact derived ZIPs explicitly internal-only while the complete
      source archives remain the authoritative redistribution artifacts.
- [ ] Complete an explicit repository, CI, and website deployment review before
      publishing either package from Tokimu infrastructure.
- [x] Add a tiny Tokimu-authored synthetic WAD fixture builder for deterministic
      CI coverage.
- [x] Reject unknown, truncated, overlapping, and out-of-bounds WAD directory
      and lump ranges.
- [x] Bound WAD input bytes, lump count, individual lump size, and total
      declared lump bytes through explicit provider inputs.
- [x] Bound map counts and decoded allocation totals when map/asset decoding is
      introduced; the container provider does not decode those formats.
  - [x] Decode one explicitly selected map block at a time; the WAD provider's
        bounded lump count limits the source map-marker scan.
  - [x] Require per-table record limits, auxiliary byte/reference limits, and
        an aggregate map-record byte limit before map record allocation.
  - [x] Require raster source-byte, record/reference, dimension, pixel, post,
        and aggregate decoded-byte limits; retain focused rejection tests for
        the map record-count/total-byte and raster aggregate-byte boundaries.

Acceptance criteria:

- CI does not depend on proprietary or unreviewed game data.
- Every fixture records source, license status, hash, and intended evidence.
- Invalid inputs fail with structured diagnostics rather than panics.

Review references:

- <https://doomwiki.org/wiki/Licences>
- <https://doomwiki.org/wiki/Shareware>
- <https://sources.debian.org/src/doom-wad-shareware/1.9.fixed-2/debian/copyright/>
- <https://doomwiki.org/wiki/DOOM1.WAD>

The Debian record preserves the terms relied upon for the Doom shareware
finding. DoomWiki provides historical context and checksum corroboration.

## Resolved Packaging Decision

The first consumer uses the archive and Resource Space capabilities already
under pressure elsewhere in Tokimu:

```text
reviewed shareware ZIP
    -> read-only archive mount
    -> Resource Space package/member address
    -> logical WAD byte resource
    -> WAD inspection and Doom semantic import
```

The archive remains the canonical distribution and provenance unit. Resource
Space may expose `DOOM1.WAD` or `HERETIC1.WAD` as logical members without
materializing a permanent loose copy. The WAD provider receives bytes and
source identity; it does not need a ZIP special case. The archive provider does
not need a Doom special case.

The Heretic package must not be deployed as a naked `HERETIC1.WAD` download.
The Doom package should likewise remain intact so its release documentation
and observed package identity stay attached even though its license evidence
also relies on an external preserved record.

## Slice 1: WAD Container Inspection

- [x] Parse the `IWAD` and `PWAD` signatures.
- [x] Decode the little-endian lump count and directory offset.
- [x] Decode bounded directory entries: offset, size, and 8-byte name.
- [x] Preserve source order and duplicate lump names.
- [x] Expose each lump through a provider-neutral observation.
- [x] Project marker-delimited namespaces without pretending markers contain
      bytes or are ordinary files.
- [x] Add diagnostic output for malformed names, duplicate source-name
      observations, impossible ranges, malformed marker pairs, and unsupported
      container signatures. Source order plus index remains the identity until
      Slice 2 introduces consumer lookup rules.
- [x] Add a `hello-wad-inspect` corpus consumer.

Acceptance criteria:

- A valid WAD produces deterministic header, directory, namespace, and hash
  observations.
- Inspection does not require a renderer, window, or live runtime.
- Duplicate names remain addressable without accidental overwrite.

## Slice 2: Doom Resource Namespaces

- [x] Mount the reviewed Doom ZIP through the bounded archive provider.
- [x] Resolve `DOOM1.WAD` through a Resource Space package/member address.
- [x] Retain package identity, member identity, member hash, and archive hash
      in container observations.
- [x] Prove the first consumer does not require permanent extraction or a
      Doom-specific Resource Space rule.
- [x] Identify global lumps and map-marker boundaries.
- [x] Recognize sprite, flat, and patch marker ranges.
- [x] Represent WAD names independently from host filesystem paths.
- [x] Retain exact source spelling without case normalization; any later lookup
      normalization must be admitted and tested at its consumer boundary.
- [x] Resolve map-local lump sets using the reviewed classic Doom map-block
      contract, bounded by the next `E#M#` marker or directory end.
- [x] Diagnose missing, duplicated, or reordered required map lumps.
- [x] Exercise WAD contents through Resource Space without changing Resource
      Space semantics to fit Doom.
  - [x] The native `static_scene`, headless preflight, and browser workbench
        retain the reviewed ZIP at the Resource Space edge and derive the WAD
        member through the corpus-only package bridge; Resource Space receives
        no WAD namespace, lump, map, or Doom lookup rule.

Acceptance criteria:

- `E1M1` and its required lumps can be selected unambiguously.
- WAD namespaces do not leak into renderer or core APIs.
- Similar or duplicate lump names cannot produce silent identity collisions.

## Slice 3: Doom Map Decoding

- [x] Decode `THINGS`.
- [x] Decode `LINEDEFS`.
- [x] Decode `SIDEDEFS`.
- [x] Decode `VERTEXES`.
- [x] Decode `SEGS`.
- [x] Decode `SSECTORS`.
- [x] Decode `NODES`.
- [x] Decode `SECTORS`.
- [x] Inspect `REJECT` and `BLOCKMAP` with bounded validation.
- [x] Preserve source indices so diagnostics can reference original records.
- [x] Validate decoded linedef vertex/sidedef, sidedef sector, seg
      vertex/linedef/direction, subsector seg-range, BSP child, REJECT bitset,
      and BLOCKMAP table/list/linedef references before semantic lowering.
- [x] Emit a provider-neutral map summary and a deterministic top-down source
      topology diagnostic view. `hello-wad-inspect --map-svg E1M1 <output.svg>`
      writes decoded linedef boundaries (one-sided teal, two-sided gray) and
      raw `THINGS` positions (amber, with source-indexed metadata). Every
      boundary and thing retains a source record in SVG metadata/tooltips. It
      also overlays the raw classic `BLOCKMAP` extent as a dashed violet
      diagnostic boundary and marks the reviewed player-one source thing cyan,
      without asserting collision or runtime spawn behavior. It
      is explicitly not renderer input or a claim about visible geometry,
      sectors, materials, or textures.
- [x] Retain a bounded, deterministic raw `THINGS` inventory by numeric kind
      and flag-set count before assigning any gameplay meaning to those values.

Acceptance criteria:

- `E1M1` yields deterministic counts, bounds, references, sector relationships,
  and thing observations.
- Broken references identify the source lump, record, field, and invalid value.
- Doom, Hexen, and UDMF formats are not conflated; non-Doom formats are
  explicitly deferred until separately admitted.

## Slice 4: Palette, Texture, Flat, And Sprite Assets

- [x] Decode `PLAYPAL` palettes.
- [x] Inspect `COLORMAP` as source index-remapping data without prematurely
      baking software-lighting behavior into Tokimu's renderer.
- [x] Decode patch image column/post data and transparent regions.
- [x] Decode `PNAMES` and `TEXTURE1`/`TEXTURE2` composition records.
- [x] Compose wall textures from patches with deterministic clipping.
- [x] Decode 64x64 flats.
- [x] Decode sprite rotations and frame-name conventions.
- [x] Lower decoded indexed images into provider-neutral raster observations.
- [x] Record color-space and alpha/coverage assumptions explicitly.
- [x] Add a deterministic palette structural artifact.
- [x] Add a deterministic patch structural artifact.
- [x] Add a deterministic texture structural artifact.
- [x] Add a deterministic flat structural artifact.
- [x] Add a deterministic sprite-frame structural artifact.
- [x] Add a representative palette visual artifact.
- [x] Add a representative patch visual artifact.
- [x] Add a representative texture visual artifact.
- [x] Add a representative flat visual artifact.
- [x] Add a representative sprite visual artifact.

Acceptance criteria:

- The raster pipeline receives decoded pixels, never WAD or Doom patch bytes.
- Equivalent source inputs produce deterministic dimensions and fingerprints.
- Missing patches, palettes, or texture references remain visible diagnostics.

## Slice 5: Static World Geometry

Implementation sequence:

1. Create a Doom-specific, headless geometry provider outside `tokimu-render`.
   It consumes decoded map observations, produces ordinary position/UV/index
   records plus source identities, and does not own WAD bytes, palette choice,
   GPU resources, or draw submission.
2. Establish a bounded E1M1 sector/subsector topology audit before emitting
   floor or ceiling triangles. Ambiguous, degenerate, or unsupported topology
   must remain a diagnostic instead of being silently triangulated.
3. Lower floor/ceiling surfaces first, then one-sided walls, then two-sided
   openings. Each step retains source record identities and produces headless
   structural artifacts before a renderer consumes it.
4. Admit texture coordinates, pegging, sky classification, materials, and
   diagnostic presentation modes only after their source semantics are visible
   in the headless output.

- [x] Resolve linedef endpoints and right/left sidedef-sector ownership into
      source-traceable headless wall candidates; reject zero-length linedefs
      and linedefs with neither side instead of inventing geometry.
- [x] Audit E1M1 wall topology before mesh emission: retain one-sided,
      two-sided, and same-sector two-sided counts as structural evidence.
- [x] Retain root-to-leaf BSP partition paths for every E1M1 subsector (depth
      5 through 18), so the later floor/ceiling proof can account for partition
      boundaries not represented by map-wall `SEGS` alone.
- [x] Produce bounded candidate BSP regions for all E1M1 subsectors without
      emitting triangles (959 boundary vertices total); retain the 34 of 1,464
      source `SEG` endpoints outside idealized integer half-planes as explicit
      topology evidence. The maximum outside distance is 512 map units, so no
      epsilon or rounding repair is admitted.
- [x] Resolve source-traceable sector ownership for all 237 E1M1 subsectors
      from `SEG` direction and `LINEDEF` sidedef relationships; all 85 decoded
      sectors are represented and no subsector mixes ownership.
- [x] Resolve E1M1 floor/ceiling boundaries through bounded BSP-region polygons,
      not an invented `SEGS`-only loop repair. The strict `SEGS` reconstruction
      closes 55 of 237 leaves and retains the remaining 182 rejections (104
      fewer than three source segs and 78 open chains, including subsector 1)
      as source diagnostics. Every admitted surface boundary instead derives
      from the finite map extent clipped by its retained root-to-leaf BSP path;
      it has at least three vertices and a non-zero signed area before triangle
      lowering. This is the resolved headless geometry boundary, not a claim
      that raw `SEGS` alone encode every closed leaf.
- [x] Build renderer-neutral floor and ceiling triangle candidates from bounded
      BSP leaves and source-traceable sector ownership (E1M1: 485 floor and
      485 ceiling triangles). This does not yet assert texture placement,
      visible-wall correctness, or a rendered scene.
- [x] Build renderer-neutral full-height, untextured one-sided wall candidates
      from their sole owning sector (E1M1: 604 triangles from 302 linedefs).
      Two-sided opening and texture semantics remain separate work.
- [x] Build two-sided upper, lower, and middle wall geometry.
  - [x] Build renderer-neutral upper and lower height-discontinuity bands while
        retaining the authored sidedef texture name (E1M1: 188 upper and 210
        lower triangles); no material or UV interpretation is asserted.
  - [x] Build two-sided middle-texture geometry with an explicit openness and
        clipping policy: emit the positive shared sector opening only (26 E1M1
        triangles from 13 observations), and emit nothing for closed/inverted
        openings. Material alpha, portal behavior, and gameplay collision are
        deliberately not inferred by this geometry rule.
  - [x] Inventory authored middle textures before choosing that policy (E1M1:
        13 observations across 4 texture names); retain their source side and
        vertical opening without emitting geometry.
- [x] Respect floor and ceiling heights for the admitted floor/ceiling, one-sided,
      and two-sided wall geometry. Middle walls use their explicit positive
      shared-opening clip; their material alpha and portal behavior remain
      separate presentation concerns.
- [x] Compute stable source texture coordinates where the classic source format
      has a map-space contract, and explicitly characterize the plane case
      where it does not.
  - [x] Retain deterministic raw sidedef texture axes before pegging (E1M1:
        613 authored texture observations with U start/end, V offset, and raw
        linedef flags for later unpegged interpretation).
  - [x] Apply texture-size-aware V placement and pegging from the admitted
        source texture extents and retained linedef flags. The original anchor
        rules are retained in
        [`Classic Doom wall V placement evidence.md`](Evidence/Classic%20Doom%20wall%20V%20placement%20evidence.md);
        613 E1M1 texture records now resolve a source-traceable `texturemid`
        anchor through the explicit right/left-to-front/back mapping. The
        admitted one-sided and all two-sided wall triangle records now
        derive per-vertex source-texel U/V coordinates from those anchors
        (1,024 E1M1 triangles); material alpha remains separate.
  - [x] Resolve every authored E1M1 wall-texture axis against plain named
        width/height extents supplied by the raster catalog (613 bindings),
        without making geometry depend on raster implementation types.
  - [x] Establish that original Doom floor/ceiling mapping is a view-dependent
        span operation, not a source-map per-vertex UV contract. The retained
        source evidence is in
        [`Classic Doom plane mapping evidence.md`](Evidence/Classic%20Doom%20plane%20mapping%20evidence.md);
        a later presentation decision must choose original span behavior or an
        explicitly non-equivalent static mapping.
- [x] Handle upper/lower unpegged flags explicitly for the admitted textured
      triangle lowering, using the retained original-renderer anchor rules.
  - [x] Carry raw linedef flags with texture-axis observations; no flag-derived
        coordinate transform is applied before texture-size-aware placement.
  - [x] Audit E1M1 source pressure before choosing that transform: 108 of 148
        authored upper axes and 79 of 150 lower axes carry their respective
        unpegged bits.
- [x] Identify `F_SKY1` source surfaces without making sky behavior a generic
      mesh concern (E1M1: 74 headless sky-surface observations).
  - [x] Diagnose the exterior-hut floating upper wall as linedef 252's
        source-valid `STARTAN3` height band between two `F_SKY1` ceilings, then
        apply the classic sky-to-sky upper-band omission in the Doom geometry
        provider. Dual-sky and one-sky controls retain the bounded rule; see
        [`E1M1 hut sky-boundary evidence.md`](Evidence/E1M1%20hut%20sky-boundary%20evidence.md).
  - [ ] Exercise an actual E1 sky presentation without weakening the retained
        source classification or turning sky into an ordinary flat texture.
    - [x] Compose `SKY1` through the bounded Doom raster provider and expose an
          opt-in `--doom-sky` corpus path distinct from the purple AR-0027
          omission diagnostic.
    - [x] Present the panorama on a corpus-local static enclosure before world
          geometry with horizontal repeat, vertical clamp, ordinary depth
          testing, and no depth writes. This is explicitly non-equivalent to
          the original view-dependent Doom sky and adds no renderer sky API.
    - [x] Establish bounded `SKY1` coverage handling: the reviewed package has
          a wholly empty bottom eight-row band, so the panorama retains only
          rows 0--119 and rejects any partial or internal coverage gap without
          inventing texels or weakening ordinary texture-alpha deferral.
    - [x] Falsify a Doom-local world-space sky-aperture depth mask. Retained
          `F_SKY1` flat meshes can bound upward rays but cannot reproduce
          viewer-relative sky coverage near the horizon; canonical native
          inspection still showed distant static-shell sectors through the
          sky region. The experimental mask was removed rather than tightened
          until it hid the symptom.
    - [ ] Observe the narrower paired-sky boundary control. The provider now
          retains the omitted upper band between unequal adjacent `F_SKY1`
          ceilings as separately identified depth coverage: the panorama is
          drawn first, the color-suppressed boundary writes depth, and ordinary
          world geometry follows. Focused controls prove the band exists only
          for paired sky ceilings with unequal heights; native hut-area visual
          acceptance remains open.
      - [x] Confirm the exact hut boundary blocks farther sector geometry.
      - [ ] Confirm owning-side-only depth after rejecting double-sided depth:
            the latter also masked the hut when the boundary was viewed from
            its opposite source sector. The depth pipeline now back-face culls
            using the retained source-owned winding.
      - [ ] Identify the remaining lower sky-aperture leaks toward the main
            buildings. These occur below/alongside the repaired hut boundary
            and must not be hidden by broadening that wall's authority.
        - [x] Retain concrete `LOOK` identities: wall linedef 249 / sector 56
              and ceiling subsector 104 / sector 40. Headless comparison finds
              no paired-sky boundary on the captured ceiling direction, so
              this is a distinct viewer-relative wall/plane case rather than
              missing coverage from the linedef-252 control.
        - [x] Classify the multi-sector pattern rather than patching individual
              surfaces. Captures retain ordinary geometry from sectors 24, 40,
              49, 56, and 72 through the same aperture. A replayable wall-249
              ray crosses neither a paired-sky wall boundary nor an omitted
              source-sky plane before its ordinary hit.
        - [x] Compare that wall-249 ray with the Stage-3B Doom source protocol.
              The global shell hits wall 249, while classic horizontal
              traversal admits neither source SEG and explicitly prunes one
              owning subsector behind an already closed full-screen range.
              This establishes viewer-relative presentation mismatch rather
              than a missing wall or permission for another depth patch.
        - [x] Classify the remaining lower-hut ray family separately from the
              paired-sky boundary. Fresh `LOOK` records retain ordinary
              ceiling/wall hits only *after* a nearer `F_SKY1` source ceiling
              plane, with `sky-boundary=none`; the classic source trace can
              elide the later target through a solid range. This is evidence
              for bounded Doom source-plane coverage, not permission to make
              every one-sky wall a depth occluder.
        - [x] Falsify the new single-sky-plane source control against the
              lower-hut/main-building captures. It must retain a named source
              plane and a bounded projected interval, preserve the existing
              one-sky upper-wall negative, and never become a generic renderer
              sky or occlusion rule. The synthetic bounded control passed, but
              submitting all retained `F_SKY1` subsector meshes globally in
              E1M1 fixed the leak by incorrectly masking the nearby hut. Global
              source identity is therefore insufficient presentation authority.
        - [x] Falsify viewer-relative sky-span source-sector admission. The
              narrower opt-in control recomputes the current Doom BSP/vertical
              clip observation and enables only retained sky flats owned by
              emitted sky spans. Visual review showed that the hut remains
              visible nearby but is masked after backing away: sector
              admission still grants a whole subsector flat more authority
              than the projected source span earned.
        - [ ] Falsify exact viewer-relative sky screen-cell depth coverage.
              Reconstruct only current `F_SKY1` ceiling cells from the shared
              classic BSP/vertical-span observation, place them on their
              owning source-sector ceiling heights, and replace one bounded
              corpus mesh as the observer moves. Reject it if the hut is still
              masked, leaks return, or camera motion exposes cracks.
    - [ ] Resolve the remaining distant-sector leak through Doom-owned
          viewer-relative presentation evidence. A successful continuation
          must either present source sky spans directly or establish exact
          shared wall/plane screen boundaries; a world-space sky enclosure,
          frustum filter, depth-only source plane, or generic renderer
          exception is not sufficient.
      - [x] Add an opt-in live classic-BSP source control distinct from the
            falsified per-column selector. Stable SEG walls upload once;
            recursive source admission updates the caller mask per observer;
            flats follow reached subsectors; unknown identities and missing
            SEG materials fail open; survivor order remains unchanged.
      - [x] Correct two ordinary source-projection defects before visual use:
            opposite-side exterior endpoint bearings cross the FOV instead of
            being rejected, and viewer-plane-straddling solid walls remain
            visible without closing an unsafe horizontal range.
      - [x] Falsify the first `--doom-seg-classic-dynamic` composition through
            live native inspection. Changing subsectors removed spawn-room
            floor portions around pillars; the first door exposed sky until
            the observer crossed it; and the hut aperture improved without
            removing all distant geometry. A smaller submission count is not
            accepted over these visible false negatives.
        - [x] Remove reached-subsector flat selection from the live control.
              Reached BSP leaves are not presented plane coverage, so ordinary
              whole-subsector flats now fail open pending an exact plane/span
              experiment.
        - [x] Apply active door and moving-floor heights to a short-lived Doom
              topology snapshot before each source traversal. The decoded WAD
              remains immutable, but visibility no longer reasons from a
              knowingly stale closed-door map.
        - [ ] Reinspect the bounded wall-only source filter for close-wall and
              dynamic-door regressions. This may validate the two local repairs
              but cannot resolve the retained hut plane/sky leak.
      - [ ] Establish exact Doom-owned wall/plane screen boundaries or present
            retained source sky spans directly. Do not restore flat filtering,
            add overlap epsilon, or hide the remaining hut geometry with a
            broader depth wall.
    - [ ] Retain native visual evidence that the exterior has no purple or
          black sky gaps, the panorama seam is acceptable, and ordinary world
          surfaces continue to occlude the enclosure.
    - [ ] Exercise the same bounded presentation in the browser/WASM consumer
          before treating the experiment as cross-target evidence.
- [x] Preserve source linedef, sidedef, sector, and subsector identities on
      lowered presentation records. Focused lowering tests now assert the
      required identities across BSP surfaces, one-sided walls, and two-sided
      height bands; the types make those fields non-optional.
- [x] Detect and diagnose degenerate or unsupported topology for every admitted
      headless geometry input; retain source-indexed diagnostics rather than
      repairing unsupported source structure.
  - [x] Establish and test the raw WAD sidedef-to-winding convention: slot 0
        (`right_sidedef`) is the original front side and slot 1
        (`left_sidedef`) is the back side. The shared wall-quad helper, all
        static wall families, and the cyan/magenta normal SVG now agree; see
        [`Classic Doom wall side and winding evidence.md`](Evidence/Classic%20Doom%20wall%20side%20and%20winding%20evidence.md).
  - [x] Audit every E1M1 subsector under the strict `SEGS`-only closure rule,
        retaining all 182 source-indexed rejections instead of reporting only
        the first failure. This is diagnostic evidence, not a repair or a
        reason to discard the separately bounded BSP-region geometry.
  - [x] Audit raw vertical clearance before portal/middle-texture policy: E1M1
        has 4 sectors and 9 of 173 two-sided relationships without positive
        clearance. These remain source diagnostics, not repaired geometry.
- [x] Add headless source-geometry diagnostic modes without admitting material
      sampling or GPU presentation.
  - [x] Emit a source-topology wireframe SVG with raw linedef, thing, and
        BLOCKMAP identities and tooltips; it remains outside renderer input.
  - [x] Emit a separate deterministic source-sector-color SVG. Each linedef
        is colored from one explicit right-then-left sidedef sector selection,
        while raw right/left sidedef references remain visible in the tooltip;
        it does not assert a sector fill or portal policy.
  - [x] Emit a top-down wall-normal SVG whose cyan right-side and magenta
        left-side arrows are the tested WAD front/back normals for each source
        linedef direction. SVG's top-down Y inversion mirrors their apparent
        screen-side direction; it diagnoses candidate winding only and makes
        no lighting, culling, or visibility claim.
  - [x] Retain source texture names, extent bindings, pegging anchors, and
        per-vertex texel coordinates as the texture diagnostic for every
        admitted wall triangle. Raster sampling, alpha behavior, and a visual
        textured view are deliberately moved to Slice 5B because they require
        the renderer/material boundary rather than further WAD geometry work.

Acceptance criteria:

- [x] Presentation geometry can be rebuilt headlessly and inspected before GPU
      submission.
- [x] The headless lowerer exposes only ordinary triangle, source-identity,
      texture-name, and source-texel-coordinate requirements; it has no GPU,
      material, or raster implementation dependency.
- [x] Unsupported raw `SEGS` closure, invalid vertical openings, sky behavior,
      material alpha, and plane mapping remain visible diagnostics or explicit
      deferrals instead of silent geometry repairs.

## Slice 5B: Static E1M1 Presentation Admission

Slice 5B is intentionally separate from the completed headless geometry seam.
It consumes Slice 5's ordinary triangles and material requirements but must not
move Doom format semantics, WAD ownership, or mutable game state into the
renderer. It is also a direct consumer of the orientation evidence tracked by
[`AR-0021`](../../Architectural%20Reviews/AR-0021-geometry-orientation-and-facing-conformance.md).

- [x] Establish the provisional renderer orientation contract and native/WASM
      conformance fixture required by AR-0021 before relying on front/back
      culling for Doom walls. The shared fixture's native and browser/WASM
      captures agree; any later binding public renderer contract remains an
      AR-0021 decision.
- [x] Select and document a material representation for indexed Doom textures,
      palette choice, wrapping, filtering, and masked-middle alpha behavior.
  - [x] Record the existing honest inputs: palette-selected, top-down RGBA8
        pixels with source coverage; source-traceable wall texel coordinates;
        and source-traceable flat names. These do not yet select a renderer
        color-space, sampler, alpha, or plane policy.
  - [x] Resolve the generic textured-mesh UV and sampler boundary under
        [ADR-0012](../../ADR/ADR-0012-supplied-mesh-texture-coordinates-and-sampling-policy.md).
    AR-0022's corpus evidence now binds a generic supplied-UV `Textured3d`
    contract and declared point/linear plus clamp/repeat sampling. This is not
    Doom material admission.
  - [x] Select the first consumer material profile for fully covered Doom
        rasters: palette zero, sRGB upload interpretation, and point/repeat
        sampling. A source raster with any uncovered pixel returns a counted
        deferred-alpha result rather than selecting blend or cutout behavior.
  - [x] Preserve alpha/cutout as an AR-0023 decision rather than treating the
        generic fixture's opaque profile as masked-middle behavior. E1M1's 13
        source-classified masked middles lower to 26 candidates under a
        Doom-owned binary-coverage declaration; their explicit categorical
        cutout crosses through ADR-0013's generic renderer capability and they
        remain outside the static opaque draw plan. See
        [E1M1 masked-middle cutout intake evidence](Evidence/E1M1%20masked-middle%20cutout%20intake%20evidence.md).
    - [x] Close the canonical linedef-464 sidedef-ownership regression:
          `BROWNGRN` is named only by the right/front sidedef and must remain
          visible from the pit-facing owning side while disappearing from the
          secret-catwalk/back side. Keep the line non-blocking and retain this
          as Doom candidate-selection policy rather than changing the generic
          two-sided categorical-cutout renderer contract. Source-reference
          behavior, a focused native ownership test, and interactive native
          confirmation from both the pit and secret-catwalk sides are retained.
- [x] Select either original view-dependent plane spans or a documented,
      intentionally non-equivalent plane mapping; do not imply that Slice 5's
      wall texel coordinates decide this.
  - [x] Select the bounded map-axis static mapping documented in
        [Classic Doom plane mapping evidence](Evidence/Classic%20Doom%20plane%20mapping%20evidence.md):
        `u = x / 64`, `v = -z / 64` for the E1M1 64-by-64 flat sources, with
        point/repeat sampling. This is a declared Tokimu presentation policy,
        not original Doom span equivalence. Sky and masked middles remain
        source-traceable omissions in the first opaque scene.
  - [x] Implement and test the application-local `hello-doom-e1m1` flat
        lowerer. It converts one retained `DoomSurfaceTriangle` into a
        supplied-UV `tokimu::Mesh` while retaining source subsector, sector,
        plane, and flat-name evidence beside the mesh. It rejects zero extents
        and degenerate source triangles rather than fabricating a draw.
  - [x] Assemble an opaque-candidate flat batch from retained surface and sky
        observations. The first consumer excludes sky only by the retained
        source classification and preserves the omitted records for capture;
        it does not infer sky behavior from a texture-name check.
  - [x] Normalize retained wall source texels by their selected source texture
        extent without duplicating Doom pegging. The first wall batch excludes
        only retained two-sided masked-middle classifications, preserving them
        for AR-0023 evidence while leaving one-sided middle walls eligible for
        a later opaque-coverage check.
    - [x] Preserve Doom's side-local horizontal texture direction after the
          Doom-2D-to-Tokimu-3D lift: right/front sidedefs advance opposite the
          stored linedef while left/back sidedefs advance along it. The
          canonical `EXITSIGN` evidence is right/front; this repair remains a
          Doom-provider mapping, not a change to the generic supplied-UV
          renderer contract, and endpoint regressions cover both directions.
- [x] Submit a static E1M1 scene with its source-traceable floors, ceilings,
      walls, sky classification, and admitted material requirements.
  - [x] Prepare E1M1 flats and selected flat rasters through the existing map,
        geometry, and raster providers before renderer allocation. The consumer
        retains the static assembly and upload-ready RGBA8 payloads separately.
  - [x] Add a canonical-package E1M1 preflight executable. It accepts the
        reviewed ZIP/member path, retains only the ZIP at the Resource Space
        edge, reads the WAD as a bounded derived member, prepares flats/walls
        and their selected rasters, then prints the deterministic report.
        Execution against the local reviewed package remains required evidence;
        no WAD or ZIP payload is committed for this corpus consumer.
  - [x] Emit a deterministic pre-upload summary containing selected flat-triangle,
        sky-omission, opaque-texture, and deferred-alpha-texture counts.
    - [x] Extend that summary with opaque-wall, masked-middle, selected-wall-name,
          wall-opaque, and wall-deferred-alpha counts.
    - [x] Record individual degenerate flat candidates as source-traceable
          omissions and include their count in the report. Only a confirmed
          zero-area candidate may be omitted; all other lowering errors remain
          fatal. Reopen topology/triangulation evidence if omissions eliminate
          an expected surface, cluster materially by subsector/sector, or show
          visible topology loss.
    - [x] Canonical `DOOM1.WAD` preflight reports 43 omitted flat candidates
          across 21 subsectors and 13 sectors, but no completely omitted
          subsector; the flat omission rule remains a bounded retained-evidence
          result. The 16 omitted wall candidates belong to eight completely
          omitted linedefs (155, 156, 245, 246, 320, 323, 337, and 338), which
          were inspected through the retained report: each is an authored
          zero-height middle span (`minimum_height == maximum_height`) using
          `DOORTRAK` or `DOORSTOP`, not a nonzero source surface lost during
          conversion. Keep the detailed source identities in the canonical
          preflight and do not weaken the omission rule or replace it with a
          generic invalid-surface skip.
  - [x] Decode and classify the source wall-texture catalog required by the
        prepared wall batch; retain missing, uncovered, and masked-middle
        source identities as diagnostics.
    - [x] Add bounded palette-zero wall texture preparation through the
          existing PNAMES/TEXTURE catalog and patch-composition provider.
          The consumer canonicalizes requested names and returns each source
          raster as an opaque candidate or counted deferred-alpha result.
    - [x] Connect the prepared wall batch's source names and extents to this
          catalog; record missing-name, masked-middle, and uncovered outcomes
          in the pre-upload report. Canonical E1M1 selected 29 wall names;
          all 29 classify opaque under the deliberately narrow palette-zero
          profile, while 13 source-classified masked middles remain omissions
          for AR-0023 rather than alpha-policy inference.
      - [x] Derive a sorted, deduplicated source-name inventory from opaque-candidate
            wall meshes; it remains separate from renderer handles.
      - [x] Build the wall batch from E1M1 map records and catalog-derived
            extents. It preserves source-classified masked-middle omissions and
            ordinary source-textured wall candidates before raster upload.
  - [x] Allocate textures and opaque point/repeat materials only for selected
        fully covered source rasters; retain a stable source-name-to-handle map.
    - [x] Build and test the application-local deterministic upload plan. It
          allocates handles only for opaque flat/wall rasters, sorts within each
          source kind, preserves palette-zero sRGB plus point/repeat intent,
          and gives deferred-alpha rasters no renderer handle.
  - [x] Convert the prepared opaque static scene to ordinary mesh/material draw
        entries for the native first-frame proof.
    - [x] Build and test the application-local draw-plan seam: retained Doom
          source labels stay diagnostic-only while the renderer-facing list is
          only ordinary `Mesh` plus `MaterialHandle` entries.
    - [x] Add the native `hello-doom-e1m1 --bin static_scene` first-frame
          target. It consumes the prepared upload/draw plans, selects explicit
          opaque/depth-writing/back-culling `Textured3d` state, derives a fixed
          overview camera from ordinary mesh bounds, and compiles without
          putting a Doom type in the renderer.
  - [x] Build an ordinary `Textured3d` draw list for the prepared flat and wall
        meshes with explicit opaque, depth-writing state and reviewed culling.
  - [x] Add a fixed E1M1 observer camera and submit the first native scene.
    - [x] Native visual execution rendered the 1,835-draw static scene from the
          canonical package. The initial blank result was camera-only: the
          general perspective helper has a 100-unit far plane while E1M1 spans
          thousands of units. The corpus target now owns an explicit
          bounds-based overview projection; it does not change a renderer-wide
          camera default or encode Doom scale in `tokimu-render`.
  - [x] Add the equivalent browser/WASM scene bootstrap and record readiness
        separately from first-frame presentation.
    - [x] Add the consumer-local `render_static_e1m1(canvas)` Rust/WASM bridge
          to the existing explicit-local-selection workbench session. It reads
          only the Rust-owned retained ZIP, derives `DOOM1.WAD` through the
          bounded package provider, and passes a supplied browser canvas only
          to `WgpuBackend`; TypeScript neither receives nor constructs Doom
          geometry, textures, materials, or renderer policy.
    - [x] Compile the bridge for `wasm32-unknown-unknown`, generate its Web
          binding, and type-check the browser caller. This is build evidence,
          not browser readiness or first-frame evidence.
    - [x] Exercise one selected local reviewed package through the browser and
          retain the readiness/first-frame result separately from the native
          result. The workbench reported `browser first frame presented: 1835
          draws` and showed the fixed-camera textured E1M1 scene. This is a
          manual browser observation, not yet a committed capture artifact.
- [ ] Retain fixed-camera native and browser visual observations with their
      provenance. These are not pixel-deterministic rendering specifications;
      deterministic evidence is the fixed scene/package, draw and handle
      inventory, source omissions, and structural manifests.
  - [x] Persist the pre-upload report and a source-name/handle inventory in
        [`E1M1 static presentation evidence.md`](Evidence/E1M1%20static%20presentation%20evidence.md).
        The executable remains the authoritative reproducible source for the
        complete deterministic inventory; the retained document records the
        reviewed package invocation, counts, visual observation, and scope.
  - [x] Retain deterministic scene evidence: the reviewed package identity,
        fixed observer policy, 1,823 submitted draws after the bounded
        sky-to-sky upper-wall repair, source-to-handle
        inventory, and explicit sky/masked-middle/degenerate omission counts.
  - [ ] Commit fixed-camera native and browser image observations beside target,
        adapter, build, package, and camera provenance. Do not compare their
        PNG bytes as a rendering contract.
    - [x] Expose a browser-presentation-only PNG download after a successful
          fixed-camera frame. It serializes the displayed canvas and does not
          receive Doom semantics or render policy; retaining a reviewed output
          and a comparable native image remains pending.
  - [x] Emit companion structural evidence for submitted/omitted walls, flats,
        sky, and masked-middle records. The deterministic preflight now reports
        463 submitted floors, 390 submitted ceilings, and 588/172/210 submitted
        middle/upper/lower walls beside retained sky, masked-middle, and
        degenerate omission counts; the retained evidence document records the
        same bounded categories.
  - [x] Open [AR-0025](../../Architectural%20Reviews/AR-0025-camera-candidate-selection-and-visibility-culling.md)
        after interactive source-spawn observation exposed camera-motion
        submission pressure. It separates corpus-local frustum evidence from
        Doom visibility data and any future generic culling capability.
    - [x] Retain the Stage 0 full-submission and initial Stage 1
          frustum/AABB measurements in
          [`E1M1 camera candidate-selection evidence.md`](Evidence/E1M1%20camera%20candidate-selection%20evidence.md).
          The opt-in corpus filter preserves draw order, fails open on uncertain
          bounds, changes no renderer API, and leaves full submission as the
          default contract.
    - [x] Add a deterministic yaw-plus-90 source-spawn pose that retains 1,025
          opaque and all 26 cutout draws under conservative selection.
    - [x] Disposition AR-0025 Stage 1 visual/target evidence honestly. Browser
          selection presented the exact retained `1,051`-draw native count,
          but no native side-by-side artifact was retained, so the review makes
          no no-visible-omission claim and preserves that comparison as a
          reopening gate. AR-0025 closes with full submission as the renderer
          fallback and no shared culling capability admitted.

Acceptance criteria:

- A static E1M1 scene renders recognizable walls, floors, ceilings, and the
  selected texture-placement policy.
- The renderer consumes ordinary geometry and material requirements only.
- Any unsupported texture/alpha/plane behavior is visible in the capture and
  diagnostics rather than silently approximated.

## Slice 6: Camera, Movement, And Collision

- [x] Spawn from a reviewed player-start thing.
  - [x] Resolve exactly one classic player-one start (`THING` type `1`) as a
        source-traceable import observation; missing or duplicate starts fail
        explicitly. E1M1 resolves `THINGS` record 0 at `(1056, -3616)`, angle
        `90`, flags `0x0007`. Runtime-owned spawn state remains later work.
  - [x] Locate the observed E1M1 player-one start through a strict retained BSP
        path to subsector 103 and source sector 38. A point on a partition
        boundary is rejected rather than assigned through an implicit tie-break.
  - [x] Retain the resolved start sector's raw vertical interval: E1M1 sector
        38 has floor height 0 and ceiling height 72. No player height or
        clearance policy is inferred from this observation.
  - [x] Add an opt-in static source-spawn observer to `static_scene`. It maps
        the reviewed start X/Y and heading into the corpus X/Z world, uses the
        source sector midpoint only for a capture camera, and reports complete
        source/vertical provenance. It does not create runtime player state or
        settle the later player-height policy.
- [x] Add first-person yaw and pitch policy appropriate to the selected proof.
  - [x] Retain source-heading-derived yaw, bounded pitch, captured relative
        mouse look, and corrected camera-relative strafe direction in the
        native observer. These remain corpus/application controls rather than
        a universal Tokimu camera controller.
- [ ] Normalize keyboard, mouse, and gamepad input through `tokimu-input`.
  - [x] Route native keyboard and positional mouse events through the existing
        `PlatformInputEvent::as_input_event` adapter into `tokimu_input::InputState`.
        Raw captured mouse motion remains a separate local look input because
        the normalized state intentionally records position/buttons, not a
        platform-relative delta.
  - [ ] Add a platform gamepad event path before making an equivalent gamepad
        claim; `tokimu-input` has controller state but `tokimu-platform` does
        not yet surface controller events.
- [x] Implement bounded player radius and height.
  - [x] Retain a 16-unit source-map radius and 56-unit clearance policy in the
        corpus collision/floor proof. These match the reviewed classic source
        constants but do not define a general character shape.
- [x] Implement wall collision and sliding.
  - [x] Add a corpus-local X/Z disc proof for reviewed E1M1: a 16-unit
        observer disc uses one-sided and explicitly blocking source linedefs,
        applies fixed small movement substeps plus bounded overlap resolution,
        and retains contacted source linedef identities. It is not yet player
        height, opening-clearance, door/lift, step, or generic engine collision
        semantics.
  - [x] Retain a renderer-free fixed-command replay plus nearest-wall probe.
        On the reviewed package, the five-command replay is deterministic with
        no fallback and the source-start nearest-wall probe contacts linedef 1
        at initial distance 45.255. This proves the selected source wall is
        consulted; it does not prove a complete walkable-map policy.
  - [x] Retain a deterministic tangential-wall regression showing that a
        blocked component is removed while the free component continues; no
        generic physics or collision-response contract is inferred.
- [x] Apply floor/ceiling clearance and step-height policy.
  - [x] Add a corpus-local source-sector transition lookup after horizontal
        collision: retained BSP/subsector ownership selects a candidate sector,
        allowing descents and upward steps through 24 map units while rejecting
        insufficient 56-unit vertical clearance or ambiguous source points.
        The observer adjusts its camera height by the accepted floor delta and
        logs the retained sector/floor/ceiling result.
  - [x] Retain native walk observations across an actual E1M1 stair ascent and
        descent. The maintainer confirmed both directions in the interactive
        native observer; this bounded policy is not yet a claim of complete
        classic player movement, lifts, or dynamic clearance.
- [x] Decide whether the first proof uses `BLOCKMAP`, BSP traversal, or a
      simpler deterministic broad phase; record the choice as implementation
      evidence rather than universal Doom behavior.
  - [x] Retain bounded, row-major source `BLOCKMAP` cells and their validated
        linedef candidate lists as one measured option. At E1M1's reviewed
        player start, cell 338 (column 14, row 9) yields 6 candidates rather
        than scanning all 475 linedefs. This is not yet a collision choice,
        nor does it define blockmap traversal outside the decoded grid.
  - [x] Select source `BLOCKMAP` only as the first corpus-local broad-phase
        accelerator for the disc proof. Candidate lookup covers the swept disc;
        a missing/out-of-range or non-blocking candidate set falls back to all
        known blocking source lines rather than risking a false pass-through.
        This does not define a general `BLOCKMAP` traversal or visibility API.
- [x] Add reset, noclip diagnostic mode, and current-sector observations.
  - [x] Add `R` source-pose reset and an explicit `--noclip` diagnostic mode to
        the corpus observer. Noclip retains `E` for physical use inspection and
        adds diagnostic vertical flight on `Space` (up) and physical Left Ctrl
        (down); these bindings do not affect collision-enabled movement.
  - [x] Update retained sector/floor/ceiling state after each accepted source
        transition and expose it through `CAMERA` and `COLLISION`; reset
        restores the reviewed source-start interval.

Acceptance criteria:

- A player can traverse the static `E1M1` start area without passing through
  blocking walls.
- Collision and movement are deterministic under fixed-step input replay.
- Presentation does not mutate player or map truth.

## Slice 7: Runtime Observation And Diagnostics

- [ ] Observe current map, player thing, position, angle, sector, and health.
- [ ] Observe selected linedef, sidedef, sector, and thing source identities.
- [ ] Expose importer warnings separately from runtime warnings.
- [ ] Capture parse, decode, lowering, upload, and frame timing boundaries.
- [ ] Record draw, material, texture, mesh, and allocation counts.
- [ ] Add deterministic screenshots and structural artifacts.
- [ ] Make unsupported specials, assets, and object kinds visible in the UI.
- [ ] Add a small observation-shell catalog for read-only map inspection.

Acceptance criteria:

- The first diverging artifact identifies the owning diagnostic boundary.
- A visible scene cannot silently omit unsupported source semantics.
- Diagnostics remain bounded under repeated frames.

## Debug Slice D.1: Embedded Console And Cursor Inspection

This is application/tooling evidence for the Doom corpus. It composes the
existing Tokimu console interaction and text-presentation work, but does not
make Doom commands, picking, or an embedded shell part of Ring 0.

- [ ] Toggle a bounded embedded console with the physical backquote/tilde key
      (`~`), using the normalized Tokimu key identity on native and browser
      hosts.
  - [x] Add normalized physical `Backquote` identity in native and browser
        platform adapters and use it to toggle the native Doom console.
  - [ ] Exercise the embedded console through the browser/WASM Doom host.
- [x] Transfer focus explicitly: opening the console releases captured mouse
      look and suppresses movement; closing it does not synthesize gameplay
      input.
- [x] Reuse the reviewed native default font and ordinary Tokimu textured-2D
      presentation path for a transcript, prompt, and visible input cursor.
- [x] Add corpus-local commands for `HELP`, `CLEAR`, `CAMERA`, `COLLISION`,
      `NOCLIP`, and retained diagnostic status.
- [x] Add a center-screen inspection cursor/crosshair and report the nearest
      prepared draw candidate intersected by its camera ray.
  - [x] Retain ordinary draw/source identity, candidate distance, material,
        and whether the result came from opaque or cutout preparation.
  - [x] Label initial AABB or triangle-ray results precisely; do not call a
        conservative candidate an exact selected surface.
- [ ] Add source-aware Doom inspection after the generic draw candidate:
      linedef/sidedef/sector/thing identities remain owned by the Doom corpus.
  - [x] Retain compact linedef/sidedef/sector identity for wall hits and
        subsector/sector/plane identity for flat hits; these remain
        corpus-owned source descriptions attached to an exact
        prepared-triangle result.
  - [x] Report the source-space ray as `x,y,height` plus map-plane and vertical
        direction, retain the corresponding Tokimu world ray, and emit a
        copyable `--look-ray-report=...` token for deterministic headless
        replay of observed problem locations. The emitted replay values retain
        nine decimal places: the earlier three/six-decimal token could move a
        hut-aperture ray across a narrow edge and turn a visible hit into a
        headless miss. A targeted replay-format regression protects that
        diagnostic evidence path.
  - [x] Compare each replay ray with paired-sky depth-boundary meshes and
        retain whether the closest boundary lies before or behind the ordinary
        prepared hit, including its Doom source identity.
  - [x] Compare each replay with omitted `F_SKY1` source planes and the bounded
        Stage-3B classic horizontal traversal. Retain viewer/target subsectors,
        target SEG admission, and watched BSP elisions so a global-shell hit
        can be distinguished from a surface Doom source preparation would not
        submit.
  - [ ] Add Thing inspection when Things enter prepared caller data. Do not
        invent a diagnostic radius, height, or billboard solely to make a
        source point selectable.
- [x] Make “no candidate,” unavailable source identity, unsupported command,
      and truncated transcript states explicit.
- [ ] Retain native and browser/WASM observations before claiming target
      parity.
  - [x] Retain the first native console observation in
        [`D1 debug console evidence.md`](Evidence/D1%20debug%20console%20evidence.md).
  - [ ] Exercise the console through a persistent browser/WASM input/frame
        host; the current intake intentionally presents one frame and exits
        its renderer lifecycle.
- [x] Review whether repeated non-Doom pressure justifies extracting any
      embedded-console or picking contract; until then this remains corpus
      composition under AR-0013.
  - [x] Record the negative admission result in
        [`D1 debug console evidence.md`](Evidence/D1%20debug%20console%20evidence.md):
        consumers repeat presentation and focus mechanics, but command/session
        meaning and exact picking identity have not converged across two
        independent persistent hosts.

Acceptance criteria:

- `~` opens and closes the console without leaving movement or mouse-look input
  stuck.
- The console can explain the current camera/collision state and one bounded
  center-ray draw candidate without mutating map or renderer truth.
- Console and cursor presentation use ordinary Tokimu UI/render seams; Doom
  source meaning remains outside renderer and generic shell vocabulary.
- Unsupported or approximate inspection claims are visible in the transcript.

Current disposition: the native D.1 console/cursor evidence is sufficient for
continued Doom investigation. Browser parity is parked until the browser
workbench has a persistent renderer/input/frame lifecycle. Sidedef, sector, and
Thing inspection are parked until those identities are carried into the
prepared caller data; neither gap is silently promoted into a generic engine
contract. The extraction review is complete and retains corpus-local
composition under AR-0013.

## Slice 8: Interactive Map Semantics

- [x] Classify Doom linedef and sector specials used by `E1M1`; retain the
      source evidence and minimum future owner in
      [`E1M1 special semantics evidence.md`](Evidence/E1M1%20special%20semantics%20evidence.md).
  - [x] Inventory raw nonzero codes before assigning behavior: linedef
        `[1:8,11:1,36:1,48:8,88:1]`; sector `[1:1,7:4,8:2,9:3,12:1]`.
- [x] Add deterministic use/activation requests.
  - [x] Resolve a source-indexed `Use` request against immutable E1M1
        linedefs, retaining line source, special code, tag, and future owner
        intent without changing map, runtime, or renderer state.
  - [x] Expose `USE <linedef>` in the native debug console and retain the
        canonical-package report in
        [`E1M1 special semantics evidence.md`](Evidence/E1M1%20special%20semantics%20evidence.md).
  - [x] Make no-special, unknown-linedef, wrong-activation, and unsupported
        special outcomes explicit. Code 11 is retained as a front-side `Use`
        exit switch; code 36/88 remain `Cross` behavior; and code-48 periodic
        scrolling remains outside activation requests because it is admitted
        separately as application-owned periodic texture state.
  - [x] Record that E1M1's code-1 manual-door lines have tag `0`; resolve their
        candidate target through the opposite sidedef's retained sector rather
        than treating the line tag as target identity. Player reach and side
        eligibility remain future interaction work.
- [x] Implement doors as runtime-owned moving-sector state.
  - [x] Keep a corpus-local manual-door state machine separate from immutable
        source sectors and renderer resources: opening, bounded top wait,
        closing, and closed phases retain target-sector identity and current
        ceiling height.
  - [x] Derive the normal manual-door destination from the source target
        sector's lowest adjacent ceiling minus a retained four-unit clearance;
        retain explicit no-adjacency/invalid-policy failures rather than
        inventing a destination.
  - [x] Exercise all eight canonical E1M1 code-1 targets through a full
        deterministic open/wait/close cycle without WAD or presentation
        mutation; retain the report in
        [`E1M1 special semantics evidence.md`](Evidence/E1M1%20special%20semantics%20evidence.md).
  - [x] Lower active runtime ceiling heights into updated flat/wall geometry
        and collision queries without reparsing WAD bytes.
    - [x] Lower the observed E1M1 manual-door ceiling flats from runtime height
          changes, replacing only changed GPU meshes. This is a bounded visual
          proof, not yet a complete dynamic wall-span/UV policy.
    - [x] Overlay the active corpus-runtime ceiling height onto the
          source-sector floor/clearance query without mutating WAD records;
          retained native play evidence traversed sector 4 only after its
          code-1 door raised to ceiling `68`.
    - [x] Re-lower target-sector and affected boundary upper-wall spans from a
          clone of the retained decoded map at the active runtime ceiling
          height, retaining existing Doom texture-span/UV semantics rather
          than stretching vertices. Confirm native visual correspondence with
          the collision opening and closed-state restoration before calling it
          a door-animation claim.
  - [x] Connect eligible physical use/re-use requests to runtime creation and
        reversal policy; the debug `USE <linedef>` command remains a source
        diagnostic until reach and player-side state are owned.
    - [x] Bind physical `E` to a source-space forward trace bounded by the
          reviewed classic 64-map-unit `USERANGE`. No-intercept, blocked-line,
          and back-side failures remain explicit, and the distance bound is
          covered at, below, and above its edge.
    - [x] Reconstruct ordered source-line traversal and player-side
          eligibility; the current prepared-triangle ray is not equivalent to
          classic fixed-point `P_PathTraverse`, but it now preserves the
          reviewed ordered-intercept, closed-line, and front-side rules.
    - [x] Define and prove reusable-door reversal behavior while a door is
          opening, waiting, or closing.
- [x] Implement lifts and moving floors needed by the selected map.
  - [x] Retain separate corpus-local runtime state for E1M1 code 36 turbo
        lowering and code 88 down/wait/up/stay platforms. Both select immutable
        source sectors by tag, derive their destinations from adjacent source
        floors, and reject absent tags, missing adjacency, or invalid timing
        rather than inventing motion.
  - [x] Exercise the canonical E1M1 code-36 and code-88 targets through their
        complete released-source cycles without WAD, collision, or presentation
        mutation; retain the report in
        [`E1M1 special semantics evidence.md`](Evidence/E1M1%20special%20semantics%20evidence.md).
  - [x] Detect eligible physical line crossings in source order and start the
        tagged runtime exactly once for code 36 or when inactive for reusable
        code 88.
    - [x] Filter accepted source-space movement against retained code-36/88
          linedefs, preserve intersection order, and cover ordered crossing
          independently from camera rays or prepared geometry.
    - [x] Consume code 36 only after successful runtime creation; keep code 88
          inactive while its platform is moving and permit a new runtime only
          after the prior cycle completes. Code 11 remains an explicit
          front-side-use map-transition observation.
  - [x] Overlay active floor heights into walk clearance and re-lower affected
        flats/wall spans without reparsing WAD bytes.
    - [x] Overlay active code-36/code-88 floor heights by retained sector
          identity after BSP ownership resolution; immutable source floors and
          active door ceiling overlays remain separate inputs.
    - [x] Re-lower affected floor flats and boundary wall spans from runtime
          heights, and carry a stationary observer standing on a moving
          platform without assigning motion to rendering.
    - [x] Retain a no-window canonical resource replay proving both completed
          E1M1 effects update their exact floor vertices, regenerate affected
          wall spans without a visual diagnostic, reuse the existing dynamic
          handle seam, and carry a source-sector-matched observer.
  - [x] Retain native traversal and visual observations for both E1M1 effects
        before calling either progression path complete.
- [x] Implement switches and texture-state changes required by E1M1.
  - [x] Resolve the released shareware switch pairs against the line's
        front/right sidedef in upper/middle/lower slot order without mutating
        imported sidedefs. E1M1 linedef 330 resolves middle texture
        `SW1STRTN -> SW2STRTN` on sidedef 452.
  - [x] Prepare the paired texture even though the initial static scene does
        not reference it, retain the active switch choice in application
        state, and lower it to an ordinary material handle at draw submission.
  - [x] Advance all eight code-48 front-sidedef wall UVs by one source texel
        per 35 Hz Doom tic and refresh only those ordinary meshes. This is a
        corpus-local realization, not a Doom-aware renderer contract or an
        admission of a generic material-transform feature.
- [x] Implement teleports if required by the admitted map slice. E1M1's
      reviewed nonzero-linedef inventory contains no teleport special, so the
      admitted map slice requires no teleport implementation.
- [x] Track secrets, exits, and map transitions as application semantics.
  - [x] Correct E1M1 code 11 to a front-side `Use` exit switch and connect an
        accepted physical or console use to the next bounded WAD-catalog map
        through the corpus application's existing replacement-process
        lifecycle. Switch texture state remains a separate application-owned
        presentation choice.
  - [x] Count the three immutable E1M1 code-9 source sectors and record each
        sector's first grounded player entry in application state. Repeat
        entry is idempotent, noclip inspection cannot discover secrets, and
        neither source sectors nor renderer state carry progression truth.
- [x] Keep unsupported specials explicit. Unadmitted line and sector codes
      remain retained source observations or explicit request failures; none
      silently execute as a nearby admitted effect.

Acceptance criteria:

- Required `E1M1` progression can be completed without hardcoded renderer
  behavior.
- Commands mutate runtime-owned state through reviewed requests.
- Moving geometry updates do not require reparsing the WAD.

## Slice 9: Things, Combat, And Gameplay

- [x] Classify decorations, pickups, monsters, projectiles, and weapons for
      the admitted E1M1 corpus; retain the source table and observations in
      [`E1M1 Thing classification evidence.md`](Evidence/E1M1%20Thing%20classification%20evidence.md).
  - [x] Classify all 138 map-authored records across 30 numeric kinds with
        zero unknowns, preserving source flags rather than applying a hidden
        skill/network filter.
  - [x] Keep map-placed weapons distinct from weapon runtime state, classify
        the six shootable barrels separately from passive decorations, and
        record that projectiles are runtime-created rather than authored
        `THINGS` records in E1M1.
- [x] Render sprites with frame, rotation, and billboard policy.
  - [x] Retain the initial frame separately from the four-character sprite
        root, including E1M1's `PLAYW` bloody-mess and `PLAYN` dead-player
        records rather than assuming every Thing begins on frame A.
  - [x] Retain second-pair horizontal mirroring from eight-character sprite
        lump names and implement the reviewed eight-way view/Thing-angle
        rotation selection. The source-spawn report resolves all 129
        sprite-bearing E1M1 records: 100 rotation-zero, 29 view-rotated, with
        three mirrored selections and no missing frame/rotation.
  - [x] Realize the 129 sprite-bearing E1M1 source records as actual-camera
        cylindrical vertical billboards. Classic patch `left_offset` and
        `top_offset` define their finite quads. Because Doom's screen-space
        sprite composition may cover pixels below a Thing origin while a
        physical billboard would intersect the floor depth, lift only patches
        whose lowest covered texel would fall below the owning floor; ignore
        transparent bottom padding. Transparent patch pixels use categorical
        coverage with ordinary depth. Pitched views reproject the same
        world-vertical quads rather than tilting them toward the camera.
        Grouped-sky presentation includes sprites in its cutout-aware depth
        prepass and even-parity color pass. Player/deathmatch starts remain
        non-rendered spawn markers, and source difficulty/network flags remain
        retained but unapplied until gameplay policy is admitted.
  - [x] Give map-authored Thing placement its own reviewed Classic BSP equality
        rule (left child on a partition tie). Do not weaken the existing
        collision/topology locator, which continues to diagnose non-unique
        partition-boundary points.
- [x] Add deterministic thing state machines.
  - [x] Keep the mutable state index, remaining tics, and elapsed-tic count in
        application-owned runtime records separate from immutable WAD Things.
        Advance them on an integer 35 Hz clock so different frame-time
        chunking produces the same resulting state.
  - [x] Admit the source-authored E1M1 visual loops needed by the current
        corpus: monster A/B idle, barrel A/B idle, health/armor-bonus
        A/B/C/D/C/B, and green/blue armor A/B. Static Things hold their exact
        initial frame indefinitely. Resolve and upload every reachable source
        patch before live presentation.
  - [x] Retain monster idle `A_Look` as a deferred gameplay action rather than
        silently executing perception or activation from a presentation clock.
        Retain source full-bright state bits as state evidence; applying
        `COLORMAP`/lighting remains separately unadmitted.
- [x] Add pickups, inventory, health, armor, ammo, and keys.
  - [x] Keep player health, armor/type, four ammo pools, weapon ownership,
        six key slots, and item count in application-owned mutable state rather
        than rewriting decoded `THINGS` records.
  - [x] Apply the admitted E1M1 health, armor, ammo, and weapon pickup
        transitions with bounded Classic capacities. Retain all six Classic
        key transitions in the same deterministic inventory model; E1M1 has no
        authored key Thing to exercise live.
  - [x] Use the admitted 16-unit player and 20-unit pickup radii plus Classic's
        `-8..=player-height` vertical touch interval. A successful pickup
        disables only that runtime sprite occurrence and emits the resulting
        inventory diagnostic. Full inventory leaves the source occurrence
        present.
  - [x] Keep difficulty-based ammo doubling, dropped-item policy, pickup
        sounds/messages, and automatic weapon switching explicit later work.
- [x] Add hitscan and projectile collision.
  - [x] Add a deterministic nearest-hit kernel over finite vertical actor
        cylinders, with source-record tie-breaking and a caller-supplied
        nearest world-surface distance. World wins equal-distance ties so an
        actor cannot leak through its occluder.
  - [x] Add finite projectile-cylinder sweep by expanding target radius and
        vertical support by the projectile volume, then resolving actor versus
        world collision over exactly one caller-owned movement delta.
  - [x] Wire captured left-click to a 2,048-unit actual-camera hitscan probe.
        The corpus application supplies active source-backed monster/barrel
        cylinders and the nearest active prepared opaque surface, then reports
        actor/world/miss before the separately owned damage transition.
  - [x] Keep prepared-surface distance explicitly corpus-private rather than
        making presentation geometry generic gameplay authority. Masked-middle
        declarations do not block this first trace. Projectile creation,
        movement scheduling, and impact effects remain separate work.
- [x] Add damage, death, and respawn policy.
  - [x] Retain E1M1 monster/barrel spawn health in application-owned runtime
        state. Pistol shots consume bullets and use a replayable private copy
        of Doom's play-RNG table and released `5 * (random % 3 + 1)` damage.
  - [x] Make death terminal for collision and live sprite participation while
        retaining the immutable imported Thing. `R` restores the source-spawn
        observer, inventory, Thing occurrences, actor health, animation clocks,
        and play-RNG index as one corpus-local respawn operation.
  - [x] Retain player health/armor damage policy independently: green armor
        absorbs one third, blue armor one half, bounded by remaining armor;
        zero health is a deterministic terminal outcome.
  - [x] Replay two exact pistol ray hits against a source-backed zombieman from
        reset state and prove identical RNG, damage, and killed state. Death
        sprites/pain states, drops, barrel explosions, monster attacks, and a
        general level restart remain explicit later work.
- [x] Add monster perception and movement only after observation boundaries are
      useful enough to diagnose behavior.
  - [x] Separate gameplay sight from rendering: use `REJECT` only as a negative
        sector-pair prefilter, then trace the source segment through one-sided
        blockers and finite two-sided vertical openings before applying the
        initial 180-degree front arc and close-range exception.
  - [x] Add `--monster-perception-report`. At the E1M1 source spawn it retains
        all 29 source monsters with a specific outcome (5 REJECT-forbidden and
        24 source-linedef blocked), while 29/29 same-sector near-front positive
        controls acquire the player. The report moves no actor.
  - [x] Let both sight and actor movement consume caller-owned current
        door/platform floor and ceiling overlays. A regression proves the same
        source sight line changes from blocked to acquired when its door
        ceiling opens.
  - [x] Add a separate application-edge actor movement oracle over all source
        linedefs. It evaluates one-sided walls, dynamic two-sided openings,
        24-unit steps/dropoffs, vertical clearance, and explicit actor bodies
        without promoting the narrower first-walk helper. At E1M1 spawn, 27/29
        eight-unit source-direction probes move and 2/29 retain explicit
        vertical blocks; none mutates a Thing.
  - [x] Add chase clocks and application-owned mutable monster positions behind
        an opt-in live candidate. Rebuild ordinary sprite declarations from the
        resulting positions; do not mutate imported Things. Attack selection
        and sound wake-up remain separate.
        `--monster-chase-live` evaluates retained `A_Look` every 10 tics, uses
        source three/four-tic A-D run cadences, quantizes pursuit to eight
        source directions, tries the other bounded headings when a direct step
        is blocked, and retains a successful escape heading for one 64-unit
        run. Existing actor overlap may decrease step-by-step but cannot be
        introduced or increased. Each accepted application-owned pose remains
        an ordinary billboard. Hitscan actor construction preserves source
        Thing indices across inactive records, so damage follows a chasing
        monster's runtime pose. A two-frame native smoke run retains the
        grouped sky and sector-boundary preparation unchanged.
- [x] Add save/replay evidence without treating WAD bytes as mutable world
      state.
  - [x] `--gameplay-snapshot-replay-report` captures only admitted mutable
        gameplay state (inventory, Thing activity/state clocks, combat health,
        play RNG, and monster runtime poses), restores the baseline, and proves
        an identical bounded replay. Imported Things and renderer resources are
        excluded; this is an in-memory evidence shape, not a persistence format.

Acceptance criteria:

- Gameplay state remains separate from immutable imported map/resources.
- One deterministic encounter can be replayed to the same resulting state.
- Unsupported thing types are retained and diagnosed rather than discarded.

## Slice 10: Sound And Music

- [x] Decode Doom sound-effect lump metadata and PCM semantics.
  - [x] `doom-audio-provider` performs replacement-friendly last-match lookup,
        validates the eight-byte format-3 header and explicit decode limits,
        retains unsigned eight-bit mono samples, and lowers them to bounded
        finite normalized `audio_tools::PcmClip` values.
  - [x] The headless `doom_sound_report` corpus proof decodes canonical
        `DSPISTOL` (11,025 Hz, 5,661 samples) and `DSPOSACT` (11,025 Hz,
        10,774 samples) without initializing a device, renderer, or window.
- [x] Map game events to provider-neutral sound requests.
  - [x] Pistol-fire and zombieman-alert events first produce logical
        `SoundRequest` values; the corpus-private Doom mapping resolves their
        clip keys to `DSPISTOL` and `DSPOSACT` only afterward.
- [x] Parse MUS or lower it through an optional reviewed MUS-to-MIDI provider.
  - [x] `doom-audio-provider` parses MUS directly into the provider-neutral
        `NoteSequence` model at its explicit 140 Hz timebase; no MIDI file or
        Doom parser object crosses the provider boundary.
  - [x] All 13 music lumps present in the canonical shareware WAD decode under
        the same bounds. The unavailable retail-only `D_BUNNY` name reports an
        explicit missing-lump failure.
- [x] Exercise the planned MIDI sequencing/synthesis provider without making
      Doom music define Tokimu's audio contracts.
  - [x] `audio-tools::SequenceTransport` dispatches all 5,825 `D_E1M1` events
        exactly under application-supplied 35-unit steps and an explicit
        start/pause/resume/finish/stop lifecycle.
  - [x] `simple-audio-synth-provider` produces a bounded five-second,
        22,050-Hz stereo preview from the same generic sequence. Its triangle
        oscillator and instrument substitution are corpus-provider behavior,
        not Doom or stable Tokimu semantics.
  - [x] The canonical preview contains 110,250 stereo frames, peaks at
        `0.160282` without clipping, and encodes as a 441,044-byte PCM16 WAVE
        artifact with fingerprint `40b890766bb076a3`.
- [x] Add positional sound requirements separately from decoder mechanisms.
  - [x] `SoundEmission` distinguishes listener-relative playback from a finite
        world-space source position and contains no Doom or backend vocabulary.
- [x] Add deterministic event observations even when live audio output is
      unavailable.
  - [x] `doom_sound_report` records logical requests, emission, resolved source,
        metadata, normalized extrema, duration, and source-sample fingerprint;
        it explicitly reports `audio-device=false`, `playback=false`, and
        `clock=none`.
- [x] Exercise native output without making the device or callback Doom-owned.
  - [x] The corpus-local `cpal-audio-output-provider` consumes the same bounded
        synthesized `D_E1M1` PCM and decoded `DSPISTOL` PCM used by the headless
        proofs. Its callback owns sample-rate/channel adaptation and mixing but
        contains no MUS parsing, Doom event selection, or application policy.
  - [x] `doom_audio_playback` proves start, looping music, an independent
        one-shot pistol cue, a separately synthesized one-note cue, pause,
        resume, and stop through a private application adapter. The first
        native observation recorded 235
        callbacks and 104,164 frames with zero starvation, queue rejection,
        xrun, device-unavailable, or other device errors; manual listening
        confirmed the pistol cue was audible.
  - [x] Queue overflow rejects the newest command explicitly. Device open,
        configuration, stream lifecycle, nominal buffer latency, xruns, and
        invalidation remain provider-attributed diagnostics. Production
        lock-free callback behavior remains unclaimed.
  - [x] The native walkabout accepts opt-in `--audio`. It prepares a bounded
        30-second loop from the selected map's `D_E#M#` score, then routes an
        ammo-consuming pistol shot and a successful opt-in monster wake through
        the existing logical Doom sound requests. Audio preparation/device/cue
        failure is diagnosed while gameplay continues; omitting `--audio`
        preserves the previous audio-free path.
  - [x] The first E1M1 two-frame composition opened the 44.1-kHz stereo device,
        resolved both admitted cues, started `D_E1M1`, and retained the grouped
        sky and sector-boundary presentation. Spatial monster requests are
        explicitly mixed listener-relative pending a separate spatial-audio
        slice; this implementation does not imply positional realization.

Acceptance criteria:

- The application requests sound by semantic event; the runtime and renderer
  do not parse Doom audio formats.
- Audio-disabled and headless runs retain useful event evidence.
- Music timing follows an explicit clock and lifecycle.

Headless reproduction:

```text
cargo run -p hello-doom-e1m1 --bin doom_music_report -- corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD D_E1M1
```

Append an output path to write the five-second PCM16 listening artifact; WAVE
is evidence output, not the runtime audio contract.

Native audible reproduction:

```text
cargo run -p hello-doom-e1m1 --bin doom_audio_playback -- corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD
```

Opt-in walkabout with map music and gameplay cues:

```text
cargo run -p hello-doom-e1m1 --bin static_scene -- corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD --map=E1M1 --skywall-parity-full --sector-boundary-trim --monster-chase-live --audio
```

## Slice 11: Consumer And WASM Proof

- [x] Add a native walkable Doom corpus consumer. The existing
      `hello-doom-e1m1 --bin static_scene` target now owns source-start spawn,
      normalized keyboard/mouse input, collision, runtime doors/platforms,
      diagnostics, map rotation, and the reviewed E1M1-E1M9 walkabout; a
      cosmetic binary rename to `hello-doom-walk` is not a separate milestone.
- [ ] Add drag-and-drop WAD inspection to the Asset Workbench.
- [ ] Add a bounded WASM map viewer only after native static rendering is
      stable.
  - [x] Add the browser working-model test surface. After explicit local ZIP
        selection, Rust/WASM accepts only E1M1-E1M9 markers and prepares one
        source-spawn frame with the sector-boundary plane bake and the current
        grouped skywall-plus-source-sky-plane stencil sequence. Browser
        previous/next buttons and `[` / `]` replace the complete map frame;
        TypeScript never receives WAD geometry or presentation policy.
  - [x] Add a corpus-private browser walkabout over the retained working-model
        scene. Rust owns the camera and WebGPU submissions; pointer-lock mouse
        look plus W/A/S/D, Space/Ctrl, and Shift provide noclip inspection
        without claiming browser Doom player simulation or a public camera API.
        Idle animation frames do not resubmit the scene, and map replacement
        pauses input until the complete replacement is presented.
  - [ ] Retain a real WebGPU observation and capture for at least E1M1 plus one
        swapped map. WASM compilation and strict TypeScript checking are only
        readiness evidence until the browser executes the stencil path.
    - [x] Retain the first real execution: E1M2 presented through Browser
          WebGPU at 960x600 with 1,921 opaque draws, 20 paired skywalls, 242
          source sky planes, 3,635 surface triangles, and 642 edge-conformance
          insertions. The user-observed frame showed the source-spawn interior;
          E1M1 plus an explicit live map-swap/walkabout capture remains open.
- [ ] Keep user-supplied WAD bytes in the browser session.
- [ ] Avoid publishing unreviewed commercial or shareware data.
- [ ] Record native/WASM importer and geometry parity.
- [ ] Bound startup payload, decoded resources, and browser memory.

Acceptance criteria:

- Native and WASM consumers use the same Rust-owned WAD and Doom semantic
  implementation.
- TypeScript transports bytes and presents observations; it does not parse WAD
  or redefine Doom semantics.
- Failure to load the WAD remains visible and does not falsely report readiness.

## Slice 12: Compatibility Expansion

Only begin these after the static viewer and selected gameplay proof are
stable:

- [ ] Additional Doom episodes and maps.
- [ ] PWAD overlay and lookup precedence.
- [ ] Demo playback and deterministic compatibility study.
- [ ] Savegame compatibility study.
- [ ] Vanilla rendering and gameplay quirks where deliberately targeted.
- [ ] DeHackEd/BEX provider study.
- [ ] Boom-compatible map semantics.
- [ ] Hexen map format provider.
- [ ] UDMF provider.
- [ ] Multiplayer and network replication.

Each compatibility family requires an explicit corpus selection and must not
silently widen the initial Doom-format contract.

## First Milestone Definition Of Done

- [x] A user can select or mount the reviewed Doom shareware ZIP.
- [x] Resource Space resolves its `DOOM1.WAD` member without separating the
      canonical package from its documentation and provenance.
- [x] Tokimu validates and inspects the WAD directory.
- [x] `E1M1` map and visual assets decode through Rust-owned providers.
- [x] Tokimu renders a recognizable textured static scene.
- [x] A player can walk through the start area with collision.
- [ ] Headless structural artifacts and a deterministic screenshot are saved.
- [x] Unsupported map, asset, and gameplay semantics are listed explicitly in
      the slice deferrals and surfaced by bounded importer/runtime diagnostics.
- [x] No Doom-specific type appears in renderer or trusted-core public APIs.
- [x] No unreviewed WAD data is committed or deployed.

## Open Questions

- After the first read-only archive mount, should editable consumers copy WAD
  members into another store, retain virtual archive-backed addresses, or
  support both as explicitly different workflows?
- Should immutable imported map identity be source-index-based, hash-based, or
  assigned by the consuming application?
- Which representation should own sector/subsector topology before lowering?
- Is BSP traversal needed for the first renderer, or only later for visibility,
  collision, and compatibility evidence?
- Should classic palette/colormap lighting be an optional Doom presentation
  provider or lower into general material parameters?
- Which Doom quirks are compatibility requirements versus intentionally modern
  Tokimu behavior?
- Does a future Doom source-port consumer justify a reusable 2.5D world
  capability, or should those semantics remain application-owned?

## Parking Criteria

This work is at a good pause when the first milestone is complete, artifacts
are reproducible, unsupported behavior is explicit, and the next slice would
require choosing between authentic Doom compatibility and a Tokimu-authored
game using Doom assets. That choice should be made from corpus evidence rather
than assumed by this checklist.
