# Doom Source-Topology Admission Baseline

| Field | Retained observation |
| --- | --- |
| Date | 2026-08-16 |
| Package | `corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip` |
| Package byte length | `1,810,639` |
| Package SHA-256 | `B99CC1C170CDFC7FE951AD91DF4632C50E3486DF6806ABD8CEFF6E6FF45CBE45` |
| Package browser-intake BLAKE3 | `58146f5aa0e14ef38047a79878307344aec821b9f312da6a9208ec08e399660c` |
| WAD member | `DOOM1.WAD` |
| Default embedding | `PreserveNorth` |
| Default camera | source player-one spawn |
| Default presentation controls | masked cutouts, Doom sky, walk collision |
| Known A limitation | source-invalid distant geometry can participate through sky regions |

## Exact control invocations

The ordinary visual control remains:

```powershell
cargo run -p hello-doom-e1m1 --bin static_scene -- corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD --render-strategy=global-full
```

The headless inventory controls are:

```powershell
cargo run -p hello-doom-e1m1 --bin static_scene -- corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD --render-strategy=global-full --topology-inventory-report
cargo run -p hello-doom-e1m1 --bin static_scene -- corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD --render-strategy=topology-admitted-full --topology-inventory-report
cargo run -p hello-doom-e1m1 --bin static_scene -- corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD --render-strategy=topology-admitted-frustum --topology-inventory-report
```

## Original contribution inventory

All three controls retained the same original inventory before generic
selection:

| Family | Count |
| --- | ---: |
| Floor | 463 |
| Ceiling | 390 |
| Wall upper | 172 |
| Wall lower | 210 |
| Wall middle | 588 |
| Cutout middle | 26 |
| Sky plane | 73 |
| **Map contributions** | **1,922** |
| Presentation-global sky panorama | 1 |
| Runtime-related static contributions | 174 |

The aggregate contribution hash was `30650e57ad9b3c07` for A, provisional B,
and provisional C. Inventory verification reported `unchanged=true` in every
case. Provisional B deliberately classified all 1,922 map contributions as
`unresolved-fail-open`; it admitted and rejected zero contributions. This is
the conservation baseline, not evidence that the Doom topology algorithm is
complete.

Duplicate samples are expected because a source subsector may lower to more
than one original triangle/draw contribution. They are retained as evidence
that source identity and presentation occurrence are not interchangeable.

## Pipeline identities

```text
global-full
    original-complete-geometry
    renderer-full-submission

topology-admitted-full (provisional)
    original-contribution-inventory
    Doom topology admission: all fail open
    renderer-full-submission

topology-admitted-frustum (provisional)
    original-contribution-inventory
    Doom topology admission: all fail open
    generic conservative frustum
    renderer
```

The historical `prepared-full-submission` and
`prepared-frustum-filtered` controls retain their earlier reconstruction
meaning and are not aliases for the new candidates.

## Remaining Slice 0 evidence

Canonical source poses beyond the already retained spawn and EXIT controls
still need a single campaign-owned registry. Per-pose structural subsets will
be recorded after that registry exists; the global structural inventory above
must remain the comparison root.
