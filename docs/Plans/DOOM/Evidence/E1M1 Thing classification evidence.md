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
records its initial sprite prefix when one exists, but that observation does
not yet select a frame, rotation, billboard, animation, or material.

## Boundaries established

- The original `THINGS` flags remain attached to every source record. This
  report does not silently choose a skill level or multiplayer policy.
- Map-placed weapon pickups are not player weapon state.
- The shootable/explosive barrel is not collapsed into passive decoration.
- E1M1 contains no map-authored projectile record. Released projectile object
  types have no map number and are created by runtime actions; projectile
  creation and collision therefore remain later gameplay work.
- Classification creates no runtime truth and adds no Doom vocabulary to the
  renderer.

Two focused regressions preserve the sorted unique selected table and replay
the complete canonical kind/count inventory into the nine family totals.
