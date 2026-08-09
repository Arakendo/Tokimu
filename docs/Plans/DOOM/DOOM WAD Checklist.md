# DOOM WAD Checklist

## Status

Proposed on 2026-08-06. No WAD importer, Doom world provider, or Doom gameplay
compatibility capability is currently admitted to Tokimu.

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
- [ ] Bound map counts and decoded allocation totals when map/asset decoding is
      introduced; the container provider does not decode those formats.

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
- [ ] Exercise WAD contents through Resource Space without changing Resource
      Space semantics to fit Doom.

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
        [`Classic Doom wall V placement evidence.md`](Classic%20Doom%20wall%20V%20placement%20evidence.md);
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
        [`Classic Doom plane mapping evidence.md`](Classic%20Doom%20plane%20mapping%20evidence.md);
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
        [`Classic Doom wall side and winding evidence.md`](Classic%20Doom%20wall%20side%20and%20winding%20evidence.md).
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
- [ ] Select and document a material representation for indexed Doom textures,
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
  - [ ] Keep alpha/cutout open; do not treat the generic fixture's opaque
        profile as masked-middle behavior. Alpha/cutout policy is tracked
        separately by
        [AR-0023](../../Architectural%20Reviews/AR-0023-textured-surface-alpha-and-depth-policy.md).
- [x] Select either original view-dependent plane spans or a documented,
      intentionally non-equivalent plane mapping; do not imply that Slice 5's
      wall texel coordinates decide this.
  - [x] Select the bounded map-axis static mapping documented in
        [Classic Doom plane mapping evidence](Classic%20Doom%20plane%20mapping%20evidence.md):
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
        [`E1M1 static presentation evidence.md`](E1M1%20static%20presentation%20evidence.md).
        The executable remains the authoritative reproducible source for the
        complete deterministic inventory; the retained document records the
        reviewed package invocation, counts, visual observation, and scope.
  - [x] Retain deterministic scene evidence: the reviewed package identity,
        fixed observer policy, 1,835 submitted draws, source-to-handle
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
        463 submitted floors, 390 submitted ceilings, and 588/184/210 submitted
        middle/upper/lower walls beside retained sky, masked-middle, and
        degenerate omission counts; the retained evidence document records the
        same bounded categories.

Acceptance criteria:

- A static E1M1 scene renders recognizable walls, floors, ceilings, and the
  selected texture-placement policy.
- The renderer consumes ordinary geometry and material requirements only.
- Any unsupported texture/alpha/plane behavior is visible in the capture and
  diagnostics rather than silently approximated.

## Slice 6: Camera, Movement, And Collision

- [ ] Spawn from a reviewed player-start thing.
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
- [ ] Add first-person yaw and pitch policy appropriate to the selected proof.
- [ ] Normalize keyboard, mouse, and gamepad input through `tokimu-input`.
- [ ] Implement bounded player radius and height.
- [ ] Implement wall collision and sliding.
- [ ] Apply floor/ceiling clearance and step-height policy.
- [ ] Decide whether the first proof uses `BLOCKMAP`, BSP traversal, or a
      simpler deterministic broad phase; record the choice as implementation
      evidence rather than universal Doom behavior.
  - [x] Retain bounded, row-major source `BLOCKMAP` cells and their validated
        linedef candidate lists as one measured option. At E1M1's reviewed
        player start, cell 338 (column 14, row 9) yields 6 candidates rather
        than scanning all 475 linedefs. This is not yet a collision choice,
        nor does it define blockmap traversal outside the decoded grid.
- [ ] Add reset, noclip diagnostic mode, and current-sector observations.

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

## Slice 8: Interactive Map Semantics

- [x] Classify Doom linedef and sector specials used by `E1M1`; retain the
      source evidence and minimum future owner in
      [`E1M1 special semantics evidence.md`](E1M1%20special%20semantics%20evidence.md).
  - [x] Inventory raw nonzero codes before assigning behavior: linedef
        `[1:8,11:1,36:1,48:8,88:1]`; sector `[1:1,7:4,8:2,9:3,12:1]`.
- [ ] Add deterministic use/activation requests.
- [ ] Implement doors as runtime-owned moving-sector state.
- [ ] Implement lifts and moving floors needed by the selected map.
- [ ] Implement switches and texture-state changes.
- [ ] Implement teleports if required by the admitted map slice.
- [ ] Track secrets, exits, and map transitions as application semantics.
- [ ] Keep unsupported specials explicit.

Acceptance criteria:

- Required `E1M1` progression can be completed without hardcoded renderer
  behavior.
- Commands mutate runtime-owned state through reviewed requests.
- Moving geometry updates do not require reparsing the WAD.

## Slice 9: Things, Combat, And Gameplay

- [ ] Classify decorations, pickups, monsters, projectiles, and weapons.
- [ ] Render sprites with frame, rotation, and billboard policy.
- [ ] Add deterministic thing state machines.
- [ ] Add pickups, inventory, health, armor, ammo, and keys.
- [ ] Add hitscan and projectile collision.
- [ ] Add damage, death, and respawn policy.
- [ ] Add monster perception and movement only after observation boundaries are
      useful enough to diagnose behavior.
- [ ] Add save/replay evidence without treating WAD bytes as mutable world
      state.

Acceptance criteria:

- Gameplay state remains separate from immutable imported map/resources.
- One deterministic encounter can be replayed to the same resulting state.
- Unsupported thing types are retained and diagnosed rather than discarded.

## Slice 10: Sound And Music

- [ ] Decode Doom sound-effect lump metadata and PCM semantics.
- [ ] Map game events to provider-neutral sound requests.
- [ ] Parse MUS or lower it through an optional reviewed MUS-to-MIDI provider.
- [ ] Exercise the planned MIDI sequencing/synthesis provider without making
      Doom music define Tokimu's audio contracts.
- [ ] Add positional sound requirements separately from decoder mechanisms.
- [ ] Add deterministic event observations even when live audio output is
      unavailable.

Acceptance criteria:

- The application requests sound by semantic event; the runtime and renderer
  do not parse Doom audio formats.
- Audio-disabled and headless runs retain useful event evidence.
- Music timing follows an explicit clock and lifecycle.

## Slice 11: Consumer And WASM Proof

- [ ] Add a native `hello-doom-walk` corpus consumer.
- [ ] Add drag-and-drop WAD inspection to the Asset Workbench.
- [ ] Add a bounded WASM map viewer only after native static rendering is
      stable.
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

- [ ] A user can select or mount the reviewed Doom shareware ZIP.
- [ ] Resource Space resolves its `DOOM1.WAD` member without separating the
      canonical package from its documentation and provenance.
- [ ] Tokimu validates and inspects the WAD directory.
- [ ] `E1M1` map and visual assets decode through Rust-owned providers.
- [ ] Tokimu renders a recognizable textured static scene.
- [ ] A player can walk through the start area with collision.
- [ ] Headless structural artifacts and a deterministic screenshot are saved.
- [ ] Unsupported map, asset, and gameplay semantics are listed explicitly.
- [ ] No Doom-specific type appears in renderer or trusted-core public APIs.
- [ ] No unreviewed WAD data is committed or deployed.

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
