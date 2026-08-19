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

The first deterministic Thing-state increment follows the released `states`
table at an integer 35 Hz application-owned clock. Of the 129 sprite-bearing
E1M1 records, 75 now have animated visual state programs and 54 hold their
exact initial frame indefinitely. The animated set covers the three present
monster kinds' A/B idle cadence, barrel A/B, health/armor-bonus
A/B/C/D/C/B, and green/blue armor A/B. The headless report resolves all 280
Thing/frame occurrences required by those programs with zero state-frame
errors; the live E1M1 consumer uploads 60 unique source patches.

Runtime state stores the program, current state index, remaining tics, and
elapsed tics separately from the decoded WAD records. A chunking regression
proves that advancing the same total tick count in one call or several calls
produces the same state. Frame transitions invalidate billboard realization
even for a stationary camera. The 29 monster clocks retain their source
`A_Look` action as explicitly deferred: this increment animates the idle frame
but does not quietly introduce perception, movement, or combat. Armor
full-bright bits are retained in the state observation but are not yet applied,
because Doom `COLORMAP`/lighting realization remains a separate concern.

The next runtime increment adds a deterministic player inventory separately
from the WAD: health, armor points/type, bullets/shells/rockets/cells, five
currently relevant weapon slots, six Classic key slots, and item count. The
present E1M1 weapon, ammunition, health, and armor kinds now apply the released
`p_inter.c` capacities and single-player pickup amounts. Key transitions are
retained, although the E1M1 source inventory contains no key Thing. See id
Software's released
[`p_inter.c`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/p_inter.c).

The live consumer tests pickup contact with the already-admitted 16-unit
player radius, the reviewed 20-unit E1M1 pickup radius, and Classic's vertical
touch interval from eight units below the player floor through the 56-unit
player height. A successful transition disables that runtime sprite occurrence
and reports the resulting inventory; it does not erase or mutate the imported
Thing. A pickup which cannot change a full inventory remains present. Sounds,
messages, dropped-item policy, difficulty ammo doubling, and automatic weapon
selection remain deliberately unapplied.

The first combat-collision increment adds no engine physics API. A
corpus-private deterministic kernel traces a finite ray against source-backed
vertical actor cylinders and accepts an independently supplied nearest
world-surface distance. It selects the nearest result, uses source Thing index
as the actor tie-break, and lets a world surface win an equal-distance tie.
The E1M1 dimensions admitted here are 20 by 56 map units for its three monster
kinds and 10 by 42 for barrels; billboard dimensions remain unrelated.

Projectile collision uses the same ordering after expanding actor radius and
vertical support by a finite projectile cylinder over one caller-owned
movement delta. Tests retain actor/world occlusion, deterministic actor ties,
pitched vertical misses, and swept-volume contact. Runtime projectile creation,
tic scheduling, and effects are still deferred.

In the native corpus, the first click captures the pointer and later left
clicks issue a 2,048-unit ray along the actual yaw/pitch camera direction. The
application supplies active monsters/barrels plus the nearest active prepared
opaque surface and prints `actor`, `world`, or `miss`. A shot consumes one
bullet and advances a corpus-private copy of Doom's play RNG before applying
the released pistol damage expression. See id Software's released
[`m_random.c`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/m_random.c)
and
[`p_pspr.c`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/p_pspr.c).
This use of prepared geometry is a corpus-local live probe, not a decision that
render declarations own general gameplay collision.

E1M1 monsters and barrels retain source-backed spawn health in mutable runtime
state. A killed occurrence stops participating in collision and live sprite
presentation without changing its decoded Thing record. `R` restores the
source-spawn observer, default player inventory, all Thing occurrences, actor
health, Thing animation clocks, and the play-RNG index. It is deliberately not
a claim to reset unrelated map-special progression. Headless replay traces two
identical finite rays to a zombieman, applies the first two deterministic pistol
rolls (15 then 10), and reaches the same killed state from two independent
resets. Player damage separately retains Classic green/blue armor absorption
and terminal zero health from released
[`p_inter.c`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/p_inter.c).
Pain/death sprites, drops, barrel explosions, and monster-owned attacks remain
unapplied.

The first perception refinement is deliberately observational and can be
replayed without a renderer:

```powershell
cargo run -q -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --map=E1M1 --monster-perception-report
```

At the source spawn, all 29 monsters receive an explicit result: 5 sector
pairs are forbidden by `REJECT`, while 24 otherwise-permitted pairs are blocked
by a named source linedef or its vertically clipped opening. No monster is
acquired at that pose. A same-sector player placed 32 units in front of each
monster provides 29/29 positive controls, demonstrating that the trace can
authorize sight rather than merely reject it. This follows the division in id
Software's released
[`p_sight.c`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/p_sight.c)
and the initial front-arc/close-range policy in
[`p_enemy.c`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/p_enemy.c).
It does not use rendered pixels or declarations, and it does not move an actor.
At that observation checkpoint, live chase remained parked pending current
door/platform openings and an honest two-sided/actor movement collision policy.

That prerequisite now has a non-mutating oracle. Sight and movement accept the
same caller-owned sector-height overlay shape already used by player floor
transitions. The movement world is separately constructed from every source
linedef and tests one-sided boundaries, current two-sided opening height,
24-unit step/drop limits, destination-sector vertical clearance, and finite
actor bodies. It does not reuse the narrower player walk helper as an implicit
authority. The E1M1 report records 27 successful eight-unit forward probes and
2 explicit vertical refusals across the 29 source monsters, with no unresolved
outcome and no source mutation.

The reversible live candidate is now available through
`--monster-chase-live`. The application owns one optional runtime pose per
source monster and leaves decoded `THINGS` records unchanged. Sleeping monsters
evaluate the retained sight observation on their 10-tic look cadence; an
acquired monster turns toward the player on an eight-direction source grid and
submits an eight-unit move through the same dynamic-height actor oracle.
Accepted positions, source sectors, floor heights, and headings rebuild
ordinary sprite billboards. Source-Thing order is the deterministic update
order, and each accepted pose updates the actor-body snapshot before the next
monster moves. A blocked direct step tries a bounded deterministic set of the
other seven eight-direction headings. This prevents another actor or a local
wall corner from permanently pinning the chase without claiming exact
`P_NewChaseDir` parity. E1M1's former human uses its retained four-tic run
cadence, while the sergeant and imp use three tics; all use the source
A-A-B-B-C-C-D-D run-frame sequence retained from released
[`info.c`](https://github.com/id-Software/DOOM/blob/master/linuxdoom-1.10/info.c).
`R` discards this mutable state and recreates it from source placements.
Hitscan construction retains the original Thing index across inactive pickups
and killed actors so a moving monster's runtime pose remains attached to its
source identity. Monster attacks and sound activation remain explicitly absent.

The Slice 9 save/replay boundary is exercised independently:

```powershell
cargo run -q -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --map=E1M1 --gameplay-snapshot-replay-report
```

The report captures mutable inventory, Thing activity and state clocks, combat
health, play RNG, and monster runtime poses. It runs a bounded damage/state/
pose script, restores the baseline, reruns the script, and observes the same
15 damage against source Thing 10 and an identical final snapshot. The retained
source Thing identity/placement tuples remain exactly equal at the application boundary.
Renderer resources, imported WAD bytes, and any persistence encoding are not
members of the snapshot. This proves separation and deterministic restoration;
it deliberately does not establish a stable save-file contract.

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
- The new clock creates application-owned mutable state from the immutable
  classification at startup. It does not mutate the imported Thing record, and
  gameplay actions remain deferred until their owning systems are admitted.

Two focused regressions preserve the sorted unique selected table and replay
the complete canonical kind/count inventory into the nine family totals.
