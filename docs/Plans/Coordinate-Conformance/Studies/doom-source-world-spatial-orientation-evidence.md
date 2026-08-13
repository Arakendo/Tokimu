# Doom Source/World Spatial Orientation Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-11 |
| Review | AR-0028 Cycle 4 |
| Package | `corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip` |
| Member / map | `DOOM1.WAD` / `E1M1` |
| Status | Closed structurally; Preserve North is the explicit Doom consumer convention |

## Trigger

A canonical Doom exterior capture and the Tokimu E1M1 observer appeared to
place the small courtyard hut on opposite screen sides. The interactive Tokimu
pose included free movement and had previously used `--spawn-yaw-plus-90`, so
the screenshots alone were insufficient evidence for changing an axis.

The investigation therefore separated two claims:

1. whether the Doom source ground frame preserves signed orientation when
   lifted into Tokimu X/Y/Z; and
2. whether a lifted Doom source-right direction agrees with the current
   observer camera-right direction for the same source heading.

## Reproduction

```powershell
cargo run -q -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --spatial-orientation-report
```

The command is renderer-free. It parses the reviewed package, selects the
canonical E1M1 player-one start, and exits after reporting the two frame
relationships.

## Retained Result

```text
thing=0
source-position=(1056, -3616)
source-angle=90
source-right=(1, 0)
source-forward=(0, 1)
source-cross=+1
lifted-right=(+1, 0, 0)
lifted-forward=(0, 0, +1)
world-up-cross=-1
camera-right=(-1, 0, 0)
source-right/camera-right-alignment=-1
```

The current direct lift remains exactly invertible. It is nevertheless
orientation-reversing when the source `(right, forward)` signed orientation is
compared with `cross(lifted_right, lifted_forward)` about Tokimu world `+Y`.
The current right-handed observer also presents `-X` as screen-right while the
lifted Doom source-right vector is `+X`.

Two focused regressions retain these facts:

- `current_doom_ground_lift_reverses_orientation_about_world_up`
- `current_lifted_source_right_opposes_observer_camera_right`

The source-record narrowing report is available as:

```powershell
target/debug/static_scene.exe `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --spatial-landmark-candidates-report
```

It connects the earlier exact interactive `LOOK` hit on `wall:208:BROWN1` to
canonical source endpoints. The first retained non-collinear landmark fixture
is:

| Landmark | Canonical identity | Source point |
| --- | --- | --- |
| Player start | `THINGS #0` | `(1056, -3616)` |
| Start doorway | `LINEDEFS #0` midpoint | `(1056, -3680)` |
| Exterior hut wall | `LINEDEFS #208` midpoint, `BROWN1` | `(2176, -3824)` |

Using player start as the origin and `(doorway, hut)` as the ordered pair:

```text
source cross2 = +71,680
lifted cross dot world +Y = -71,680
hut dot source-right = +1,120
lifted hut dot observer camera-right = -1,120
```

The regression
`canonical_e1m1_spawn_doorway_and_hut_landmarks_reverse_about_world_up`
retains the exact records and result. This proves the current lift reverses
that source landmark orientation. It also proves that, for the canonical
source heading, the hut is source-right but would be presented screen-left by
the current observer basis. It still does not prove that the two screenshots
used equivalent camera positions or fields of view.

These are observation tests, not desired-contract tests. They intentionally
fail to bless either sign as the repair.

## Interpretation Clamp

- Numeric round-trip success does not prove handedness preservation.
- The result is sufficient to reopen AR-0028; it is not sufficient to change
  Doom point, direction, heading, wall, or UV conversion independently.
- Renderer UVs, WGPU clip-depth adaptation, and normalized platform input are
  not candidate compensation sites.
- Before repair, the hut and two additional non-collinear landmarks still need
  canonical WAD identities and a fixed unmodified source-spawn comparison.
- Any proposed repair must rerun Doom sidedef art, camera controls,
  projection/picking, native/browser orientation, and Box/PNG controls.

## Current Classification

- **H1 has structural support:** the current ground lift reverses the declared
  source signed orientation relative to world `+Y`.
- **H2 has structural support:** lifted source-right and observer camera-right
  oppose each other at the canonical start heading.
- **H3 remains relevant to the screenshots:** the compared images did not yet
  hold pose, yaw offset, FOV, and landmark identity constant.

The corpus consumer now defaults to the orientation-preserving Preserve North
adapter (`doom east -> world -X`, `doom north -> world +Z`). Preserve East also
passes the Doom-relative controls and remains a comparison mode; Doom cannot
choose between their 180-degree world-Y relationship. The old reflected lift
remains available only as an explicit falsification control.
