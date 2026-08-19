# E1M1 Special-Semantics Evidence

## Scope

This record classifies only the nonzero classic Doom special codes observed in
the reviewed `DOOM1.WAD` E1M1 fixture. It is an importer/corpus evidence
artifact, not an admission of classic Doom gameplay or a commitment to
source-port compatibility.

The code meanings below are read against id Software's released
`linuxdoom-1.10` special-effects sources. Those sources distinguish line
activation, sector effects, periodic update work, and runtime state. Tokimu
must preserve that distinction rather than placing all of it in rendering.

## Observed Linedef Codes

| Code | E1M1 count | Source classification | Minimum future owner |
| --- | ---: | --- | --- |
| 1 | 8 | Manual/reusable door raise (`DR`). | Runtime moving-sector state and explicit use request. |
| 11 | 1 | Single-use exit-level line (`S1`). | Application/map-transition request, not a renderer action. |
| 36 | 1 | Single-use turbo floor lower (`W1`). | Runtime moving-floor state. |
| 48 | 8 | Persistent scrolling wall effect; the original update loop advances the source sidedef texture offset. | Presentation-time texture-coordinate policy, driven by admitted runtime time; no simulation-state mutation by renderer. |
| 88 | 1 | Reusable crossing platform down/wait/up/stay (`WR`). | Runtime platform state plus a deterministic crossing request. |

The original implementation’s crossing dispatch identifies code 88 as a
retriggerable platform operation, its periodic special update explicitly
advances code 48 texture offset, and its event dispatch names code 36 as turbo
floor lowering. See [id Software `p_spec.c`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/p_spec.c).
The released [`p_switch.c`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/p_switch.c)
places code 11 in `P_UseSpecialLine`, changes its switch texture, and requests
level exit. It is therefore a front-side `Use` special, not a crossing special.

## Observed Sector Codes

| Code | E1M1 count | Source classification | Minimum future owner |
| --- | ---: | --- | --- |
| 1 | 1 | Flickering light. | Runtime/presentation light-policy boundary. |
| 7 | 4 | Nukage damage while grounded, periodically. | Runtime hazard policy and health state. |
| 8 | 2 | Glowing light. | Runtime/presentation light-policy boundary. |
| 9 | 3 | Secret-sector discovery; original behavior clears the sector special after counting it. | Application/runtime progression state. |
| 12 | 1 | Synchronized slow strobe. | Runtime/presentation light-policy boundary. |

`P_PlayerInSpecialSector` identifies codes 7 and 9 as grounded player effects,
while `P_SpawnSpecials` initializes codes 1, 8, and 12 as light effects. See
[id Software `p_spec.c`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/p_spec.c).

## Consequence

The first interactive proof needs more than a static scene, but it does not
need all classic Doom behavior. Its smallest known semantic set is:

1. a source-traceable use request for code 1 door lines;
2. deterministic runtime-owned floor/platform transitions for codes 36 and 88;
3. a visible, bounded policy for code 48 scrolling and sector-light effects;
4. explicit application handling for exit and secret progression; and
5. explicit hazard policy before code 7 can affect health.

Until those pieces exist, the viewer must expose the retained codes as
unsupported observations. No code in this record authorizes a generic special
dispatcher, source-port trigger emulation, or render-time mutation of map
truth.

## Slice 8 request evidence

The corpus now resolves a source-indexed line request without mutating map,
runtime, or renderer state. `DoomLineActivationRequest` carries the retained
linedef identity and the attempted source interaction (`Use` or `Cross`). Its
resolution either retains a future owner intent or reports why no such request
is currently valid.

The native console exposes this as `USE <source-linedef-index>`. It is a
diagnostic request, not a player-reach query: `LOOK` supplies the wall index
for inspection, and the user may ask the source resolver about it explicitly.
This keeps reach, crossing detection, and moving-sector execution out of an
otherwise pure source-classification step.

Canonical-package command:

```powershell
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --special-activation-report
```

The deterministic `Use` report found 19 nonzero E1M1 lines:

- 8 code-1 lines accepted as retained `raise-door-from-interacting-side`
  intent; all have tag `0`, and each target is instead resolved from the
  opposite sidedef's retained sector identity;
- 1 code-11 line accepted as a retained `exit-level` use intent;
- 2 lines (codes 36 and 88) explicitly require `Cross` rather than silently
  acting as use lines; and
- 8 code-48 scrolling lines remain explicitly unsupported by the activation
  resolver.

Accepted intent reports `execution=deferred-to-future-runtime-owner`. No door,
floor, platform, exit, or texture offset changes in this slice.

The later native application now consumes the accepted code-11 intent as an
application-owned map transition. A successful front-side physical or console
use requests the next map in the bounded WAD catalog through the same
replacement-process lifecycle as the explicit `]` diagnostic control. The
renderer receives no exit or map identity. Switch-texture mutation remains
unimplemented and explicit; the transition does not pretend that presentation
state has been completed.

The source resolver follows classic front-side manual-door targeting: it uses
the line's opposite/left sidedef sector as the candidate target. Actual player
reach and side detection still belong to future interaction state; the native
diagnostic `USE <linedef>` command is intentionally an explicit source request,
not a claim that an arbitrary player position is eligible to activate that
line.

## Slice 8 manual-door runtime evidence

The next bounded step keeps mutable door behavior in a corpus-local runtime
state machine. `DoomManualDoorRuntime` retains only target-sector identity,
closed/open/current ceiling heights, a phase, and a small explicit policy. It
does not contain imported mutable map records, meshes, collision shapes, or
renderer resources.

For a normal manual door, the reported source calculation is the released
classic rule: opening destination is the target sector's lowest adjacent
ceiling minus four map units; speed is two map units per tick and top wait is
150 ticks. See id Software's [`p_doors.c`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/p_doors.c)
and [`p_spec.h`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/p_spec.h).

Canonical-package command:

```powershell
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --door-runtime-report
```

The deterministic report starts all eight E1M1 code-1 source requests without
fallback or rejection. The paired lines target four sectors: sector 4 cycles
from ceiling `0` to `68`; sectors 68, 76, and 81 cycle from `-24` to `44`.
Each completes a 218-tick opening/waiting/closing cycle and ends `Closed` at
its original height. The report explicitly states that neither source map nor
presentation state changed.

This proves only the runtime state transition and source-derived destination.
Complete dynamic mesh lowering, player-side/reach checks, reuse reversal,
sound, and finished door presentation remain separate Slice 8 work.

### Native visual activation

The native debug console now connects an explicit source request for an
accepted code-1 line to its corpus-local runtime state. For the first observed
door, run the native walk command, aim at the `BIGDOOR2` wall, open `~`, and
enter `USE 151`.

The response reports target sector 4 plus the retained closed/open heights;
subsequent frames advance the door. The first presentation step replaces only
the active target ceiling flat. The in-progress follow-up re-lowers affected
target-sector and boundary upper-wall spans from a clone of the decoded source
map with the runtime ceiling substituted; it retains the existing Doom
texture-span calculation instead of stretching prior vertices. The WAD remains
immutable and no generic dynamic-mesh or door contract is admitted.

The same runtime-owned ceiling is now a narrowly declared overlay for the
corpus floor/clearance lookup. Native walk evidence begins with sector 4
rejected at clearance `0`; after `E` starts linedef 151 (or its paired 152),
the lookup accepts sector 4 at ceiling `68`, and the observer continues into
sectors 3 and 0. The WAD source sector is not mutated. This demonstrates
physical passage through the opened sector, not a complete player-side reach
model.

The maintainer subsequently observed the native source-relowered door through
its complete open/wait/close cycle. The ceiling and door face move, the door
texture retains its scale instead of stretching, newly exposed `DOORTRAK`
boundary spans appear, collision admits passage only after sufficient
clearance, and closing restores the source-height presentation. Earlier
resource-refresh failures are retained against AR-0024/AR-0027 rather than
being reclassified as door semantics. This is sufficient for the bounded
native door-animation claim; it does not admit reusable dynamic geometry.

### Physical use reach

The native `E` path now applies the released classic `USERANGE` bound of 64
map units before resolving the nearest exact prepared wall hit. The bound is
defined in id Software's
[`p_local.h`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/p_local.h),
while `P_UseLines` and `PTR_UseTraverse` in
[`p_map.c`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/p_map.c)
show that classic interaction traces source lines in order, stops at a closed
nonspecial line, and activates the first eligible special.

The corpus now lowers the observer pose back to retained Doom coordinates and
intersects source linedefs in distance order for the 64-unit trace. Open
two-sided nonspecial lines permit traversal; a one-sided or vertically closed
nonspecial line stops it; the first special line is accepted only from its
directed front/right side. Active door ceiling heights participate in the open
range without mutating source sectors. This preserves the reviewed source
rules but does not claim bit-exact fixed-point `P_PathTraverse` parity.

Repeated player use of an active code-1 door now follows the released reversal
rule: closing reopens, while opening or waiting begins closing immediately.
The state transition has a deterministic regression and remains inside the
corpus runtime. Sound and a reusable dynamic-geometry contract remain open.

## Slice 8 moving-floor runtime evidence

The code-36 and code-88 experiments retain two distinct corpus-local state
machines rather than hiding their different lifetimes behind a generic mover.
Both resolve all target sectors by the source line tag and derive destinations
from adjacent immutable source sectors.

The released constants are four map units per tick for both effects. Code 36
selects the highest surrounding floor and adds eight units when it differs
from the starting floor. Code 88 selects the lowest surrounding floor, waits
three seconds (`105` ticks) at the bottom, returns to its original height, and
may be triggered again after completing. See id Software's
[`p_floor.c`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/p_floor.c),
[`p_plats.c`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/p_plats.c),
and [`p_spec.h`](https://raw.githubusercontent.com/id-Software/DOOM/master/linuxdoom-1.10/p_spec.h).

Canonical-package command:

```powershell
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --moving-floor-runtime-report
```

The retained report resolves both E1M1 lines without rejection:

- linedef 308, code 36, tag 1 lowers sector 59 from floor `96` to `-40`
  over 34 ticks and completes as a one-shot runtime;
- linedef 195, code 88, tag 2 lowers sector 70 from `104` to `-48`, waits
  105 ticks, returns to `104`, and completes after 181 ticks ready for a later
  retrigger.

The report explicitly retains `source_map_mutated=false` and
`presentation_mutated=false`. It proves destination selection, timing, and
lifetime behavior only.

The native walk path now compares each accepted source-space movement segment
against retained code-36/88 lines and handles intersections in movement order.
A successful code-36 start consumes that one-shot line; code 88 refuses to
duplicate an active platform but can start again after completion. Code 11 is
retained through the front-side use path and reports that map transition
remains unimplemented. A deterministic local fixture proves crossing order and
excludes ordinary `Use` specials from this path. Active runtime floor heights
now overlay the matching retained sector
after BSP ownership resolution, alongside but separate from active door
ceiling overrides.

The application presentation path now uses the same source-preserving pattern
as dynamic doors: it clones the decoded map, overlays only the active runtime
floor heights, and re-lowers affected target-sector and boundary lower-wall
spans. Exact floor-flat vertices move by retained sector/plane identity. A
stationary observer is carried only when both its retained sector and previous
floor height match the moving surface. No motion or Doom vocabulary is assigned
to `tokimu-render`.

Canonical no-window presentation replay:

```powershell
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --moving-floor-resource-replay-report
```

The retained result completes code 36 in 34 ticks with 12 sector-59 floor
vertices at `-40`, then completes code 88 in 181 ticks with six sector-70 floor
vertices restored to `104`. Both stationary-observer carry checks are `true`.
The replay materializes two source-derived dynamic wall draws with two distinct
handles, retains 32 dirty meshes for a later renderer upload, reports
`visual-diagnostic=none`, and explicitly records
`source-map-mutated=false; renderer-initialized=false`.

This closes the deterministic runtime-to-presentation seam. The maintainer then
traversed the native E1M1 scene with Shift-assisted movement and observed both
canonical effects: the exit-side one-shot surface lowers on approach, while the
inner platform lowers and subsequently raises. That supplies the outstanding
player-visible traversal and motion evidence, so the bounded Slice 8
lift/moving-floor item is complete. Sound and generalized moving-surface
contracts remain outside this slice.
