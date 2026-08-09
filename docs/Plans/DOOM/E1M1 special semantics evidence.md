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
