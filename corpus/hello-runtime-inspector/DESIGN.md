# Hello Runtime Inspector

## Purpose

`hello-runtime-inspector` is a native consumer corpus for the bounded runtime
observation and command contracts established by `hello-runtime-observation`.

It does not receive a `World`, importer records, or animation state directly.
It renders only observations and command results supplied by the scenario
adapter, then requests semantic commands through the same bounded queue.

## Interaction

- `Left` / `Right`: select a scenario entity.
- `D`: queue a small move for the selected arm.
- `E`: queue an enabled-state change for the selected arm.
- `Space`: apply the current queue at the next inspector tick.
- `R`: select the mapped presentation target for the scenario arm.
- `A`: select the next assembly-step clip.
- `S`: play the selected clip; playback advances at the scenario's fixed step.

The displayed revision changes only after a later observation confirms the
application phase completed. This is intentionally a corpus proof, not a
general inspector framework.

## Ownership

- The runtime-observation corpus owns the world, command validation, playback,
  and presentation mapping.
- This inspector owns native-window interaction and presentation of bounded
  responses.
- The renderer owns pixels.

## Non-goals

- No generic entity editor.
- No raw component or world access.
- No importer or animation duplication.
- No promoted inspector capability.
