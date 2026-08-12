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

