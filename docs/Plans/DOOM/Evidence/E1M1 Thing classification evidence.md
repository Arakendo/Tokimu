# E1M1 Thing Classification Evidence

## Scope

This Slice 9 intake classifies the numeric kinds actually present in the
reviewed shareware E1M1 `THINGS` lump. It does not create actors, gameplay
state, collision bodies, or renderer declarations, and it is not a complete
Doom/Heretic object registry.

The selected table follows id Software's released
[`info.c`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/info.c):
`mobjinfo` supplies map numbers, initial states, physical flags, and sprite
state roots, while the generated state/sprite tables provide the initial
sprite prefixes. Player and deathmatch start records remain source spawn
markers rather than ordinary `mobjinfo` actors.

## Canonical observation

```powershell
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --map=E1M1 --thing-classification-report
```

The report classifies all 138 source records across all 30 observed kinds:

| Family | Count | Observed kinds |
| --- | ---: | --- |
| Player start | 1 | 1 |
| Multiplayer starts | 8 | 2, 3, 4, 11 |
| Monsters | 29 | 9, 3001, 3004 |
| Weapon pickups | 3 | 2001, 2002, 2003 |
| Ammo pickups | 20 | 2007, 2008, 2046, 2048, 2049 |
| Health pickups | 17 | 2011, 2012, 2014 |
| Armor pickups | 27 | 2015, 2018, 2019 |
| Decorations/corpses | 27 | 10, 12, 15, 24, 35, 48, 2028 |
| Explosive props | 6 | 2035 |

The retained result is `classified=138; unknown=0`. Each classified kind also
records its exact initial sprite root and frame when one exists. This matters
for the corpses: kinds 10/12 begin at `PLAYW`, and kind 15 begins at `PLAYN`,
not at an invented frame A.

The raster observation now retains horizontal mirroring for the second pair
in an eight-character sprite lump, matching `R_InitSpriteDefs`. A corpus-local
resolver applies the released `R_ProjectSprite` eight-way relative-angle rule
and falls back to rotation zero only for non-rotating frames. At the E1M1
source-spawn view it resolves all 129 sprite-bearing records: 100 use
rotation-zero frames, 29 select an eight-way rotation, three select mirrored
pairs, and no frame or rotation is missing. See id Software's released
[`r_things.c`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/r_things.c).

The live corpus consumer now decodes the 36 unique initial-frame source patches
needed by those records and realizes all 129 selections as actual-camera
cylindrical billboards. Patch width/height plus Classic `left_offset` and
`top_offset` define the finite world-space quad. Mirroring changes U direction,
transparent pixels use Tokimu's existing categorical cutout declaration, and
retained pixels use ordinary depth testing and writing. The billboards remain
world-vertical under pitch; free-look changes their projection, not their
vertical axis.

The first physical-quad walkabout exposed a representation difference for
corpses and several standing sprites: Doom can composite covered sprite pixels
below the Thing origin over a floor visplane, whereas a 3D billboard extending
below the floor is removed by ordinary floor depth. The lowering now measures
the last covered row and adds only the clearance needed to place that texel's
lower edge on the owning floor. Transparent bottom padding causes no lift and
already aligned sprites remain unchanged. At the E1M1 spawn selection, 103 of
129 sprites need between one and five map units of clearance; the maximum is
five. This is presentation lowering, not a mutation of Thing position.

With grouped-sky parity enabled, the same categorically covered sprite quads
participate in the full-world depth prepass and the even-parity color pass.
This prevents the new draw family from bypassing the established sky mask while
keeping Doom vocabulary out of the renderer.

One source Thing at `(2752,-2640)` exposed a placement-specific boundary case:
the conservative topology/collision locator correctly refuses points exactly
on a BSP partition. Map-authored Thing placement now follows Classic's
deterministic equality choice (left child), locally and explicitly; the
general locator retains its ambiguity diagnostic.

## Boundaries established

- The original `THINGS` flags remain attached to every source record. This
  report and first live realization do not silently choose a skill level or
  multiplayer policy. The initial visual proof submits every classified
  sprite-bearing source record; flag-conditioned actor admission belongs to
  the later gameplay policy.
- Map-placed weapon pickups are not player weapon state.
- The shootable/explosive barrel is not collapsed into passive decoration.
- E1M1 contains no map-authored projectile record. Released projectile object
  types have no map number and are created by runtime actions; projectile
  creation and collision therefore remain later gameplay work.
- Classification creates no runtime truth. Live realization lowers the result
  to ordinary meshes, RGBA textures, materials, categorical coverage, and draw
  commands, adding no Doom vocabulary to the renderer.

Two focused regressions preserve the sorted unique selected table and replay
the complete canonical kind/count inventory into the nine family totals.
