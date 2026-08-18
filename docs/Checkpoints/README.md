# Working Checkpoints

This directory contains resumable working-state snapshots for long-running
Tokimu campaigns. Create a checkpoint when a chat or working session has
accumulated enough context that a reset or handoff is likely. A checkpoint
records what repository evidence currently supports, what remains uncertain,
and the safest next action after that reset.

Do not create a checkpoint for every slice, test pass, implementation finding,
or review cycle. Keep ordinary progress, validation evidence, and remaining
work in the controlling plan; keep architectural findings and decisions in the
relevant Architectural Review or ADR. Several completed slices may therefore
share one checkpoint, and a focused session may need none.

Checkpoints are operational aids. They do not override the SDD, an accepted
ADR, an Architectural Review, or the authoritative checklist in a plan.

Create or refresh a checkpoint when it materially improves resumability, such
as before starting a new chat after the current conversation approaches its
useful context limit. Prefer one coherent handoff snapshot over a sequence of
incremental diary entries.

Current checkpoint:

- [2026-08-17 Doom ordered presentation](2026-08-17-doom-ordered-presentation.md)
