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
- [ ] Add a tiny Tokimu-authored synthetic WAD for deterministic CI coverage.
- [ ] Reject unknown, truncated, overlapping, and out-of-bounds lump ranges.
- [ ] Bound input bytes, lump count, individual lump size, map counts, and
      decoded allocation totals.

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

- [ ] Parse the `IWAD` and `PWAD` signatures.
- [ ] Decode the little-endian lump count and directory offset.
- [ ] Decode bounded directory entries: offset, size, and 8-byte name.
- [ ] Preserve source order and duplicate lump names.
- [ ] Expose each lump through a provider-neutral observation.
- [ ] Project marker-delimited namespaces without pretending markers contain
      bytes or are ordinary files.
- [ ] Add diagnostic output for malformed names, duplicate identities,
      impossible ranges, and unsupported container variants.
- [ ] Add a `hello-wad-inspect` corpus consumer.

Acceptance criteria:

- A valid WAD produces deterministic header, directory, namespace, and hash
  observations.
- Inspection does not require a renderer, window, or live runtime.
- Duplicate names remain addressable without accidental overwrite.

## Slice 2: Doom Resource Namespaces

- [ ] Mount the reviewed Doom ZIP through the bounded archive provider.
- [ ] Resolve `DOOM1.WAD` through a Resource Space package/member address.
- [ ] Retain package identity, member identity, member hash, and archive hash
      in importer observations.
- [ ] Prove the first consumer does not require permanent extraction or a
      Doom-specific Resource Space rule.
- [ ] Identify global lumps and map-marker boundaries.
- [ ] Recognize sprite, flat, and patch marker ranges.
- [ ] Represent WAD names independently from host filesystem paths.
- [ ] Decide and document case normalization without losing source spelling.
- [ ] Resolve map-local lump sets without depending on directory adjacency
      outside the reviewed Doom map format.
- [ ] Diagnose missing, duplicated, or reordered required map lumps.
- [ ] Exercise WAD contents through Resource Space without changing Resource
      Space semantics to fit Doom.

Acceptance criteria:

- `E1M1` and its required lumps can be selected unambiguously.
- WAD namespaces do not leak into renderer or core APIs.
- Similar or duplicate lump names cannot produce silent identity collisions.

## Slice 3: Doom Map Decoding

- [ ] Decode `THINGS`.
- [ ] Decode `LINEDEFS`.
- [ ] Decode `SIDEDEFS`.
- [ ] Decode `VERTEXES`.
- [ ] Decode `SEGS`.
- [ ] Decode `SSECTORS`.
- [ ] Decode `NODES`.
- [ ] Decode `SECTORS`.
- [ ] Inspect `REJECT` and `BLOCKMAP` with bounded validation.
- [ ] Preserve source indices so diagnostics can reference original records.
- [ ] Validate every cross-table index before semantic lowering.
- [ ] Emit a provider-neutral map summary and a top-down diagnostic view.

Acceptance criteria:

- `E1M1` yields deterministic counts, bounds, references, sector relationships,
  and thing observations.
- Broken references identify the source lump, record, field, and invalid value.
- Doom, Hexen, and UDMF formats are not conflated; non-Doom formats are
  explicitly deferred until separately admitted.

## Slice 4: Palette, Texture, Flat, And Sprite Assets

- [ ] Decode `PLAYPAL` palettes.
- [ ] Inspect `COLORMAP` without prematurely baking software-lighting behavior
      into Tokimu's renderer.
- [ ] Decode patch image column/post data and transparent regions.
- [ ] Decode `PNAMES` and `TEXTURE1`/`TEXTURE2` composition records.
- [ ] Compose wall textures from patches with deterministic clipping.
- [ ] Decode 64x64 flats.
- [ ] Decode sprite rotations and frame-name conventions.
- [ ] Lower decoded indexed images into provider-neutral raster observations.
- [ ] Record color-space and alpha/coverage assumptions explicitly.
- [ ] Add visual and structural artifacts for palette, patch, texture, flat,
      and sprite samples.

Acceptance criteria:

- The raster pipeline receives decoded pixels, never WAD or Doom patch bytes.
- Equivalent source inputs produce deterministic dimensions and fingerprints.
- Missing patches, palettes, or texture references remain visible diagnostics.

## Slice 5: Static World Geometry

- [ ] Build floor and ceiling regions from sector/subsector evidence.
- [ ] Build one-sided wall geometry.
- [ ] Build two-sided upper, lower, and middle wall geometry.
- [ ] Respect floor and ceiling heights.
- [ ] Compute stable texture coordinates.
- [ ] Handle upper/lower unpegged flags explicitly.
- [ ] Identify sky surfaces without making sky behavior a generic mesh concern.
- [ ] Preserve source linedef, sidedef, sector, and subsector identities on
      lowered presentation records.
- [ ] Detect and diagnose degenerate or unsupported topology.
- [ ] Add wireframe, sector-color, normal, and textured diagnostic modes.

Acceptance criteria:

- A static `E1M1` scene renders with recognizable walls, floors, ceilings, and
  texture placement.
- Presentation geometry can be rebuilt headlessly and inspected before GPU
  submission.
- The renderer consumes ordinary geometry and material requirements only.

## Slice 6: Camera, Movement, And Collision

- [ ] Spawn from a reviewed player-start thing.
- [ ] Add first-person yaw and pitch policy appropriate to the selected proof.
- [ ] Normalize keyboard, mouse, and gamepad input through `tokimu-input`.
- [ ] Implement bounded player radius and height.
- [ ] Implement wall collision and sliding.
- [ ] Apply floor/ceiling clearance and step-height policy.
- [ ] Decide whether the first proof uses `BLOCKMAP`, BSP traversal, or a
      simpler deterministic broad phase; record the choice as implementation
      evidence rather than universal Doom behavior.
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

- [ ] Classify Doom linedef and sector specials used by `E1M1`.
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
