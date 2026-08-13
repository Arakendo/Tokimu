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
- 3 lines (codes 11, 36, and 88) explicitly require `Cross` rather than
  silently acting as use lines; and
- 8 code-48 scrolling lines remain explicitly unsupported by the activation
  resolver.

Accepted intent reports `execution=deferred-to-future-runtime-owner`. No door,
floor, platform, exit, or texture offset changes in this slice.

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

The visible proof remains pending native observation of the source-relowered
wall spans: texture scale, boundary closure, correspondence with collision,
and restoration after close. It must not be described as finished door
animation until that observation is retained. Physical reach/side eligibility,
reusable-door reversal, sound, and a reusable dynamic-geometry contract remain
open.
