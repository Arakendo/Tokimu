# Doom E1M1 Slice 6B Literal Handoff Evidence

## Claim

The Doom-private `ordered-occurrence-prepared-full` path now gives every
prepared wall and plane source contribution one exhaustive disposition before
the renderer handoff:

```text
whole-retained
terminal-rejected
partial-seg
partial-plane
unresolved-fail-open
```

The final prepared declaration list is checked against that ledger. A
terminally rejected source contribution cannot re-enter through the original
global shell, and unresolved contributions remain explicit fail-open results.
No AABB/frustum filter participates in this proof.

## Canonical source-spawn conservation

The canonical E1M1 headless report balances the following source and output
domains:

```text
source SEGs                         732
  whole retained                    16
  partial                           16
  terminal rejected                563
  unresolved fail open             137

wall source triangles              303
wall declarations                  321

plane source triangles             304
  with survivors                    72
  terminal rejected                211
plane fragments                    166
plane declarations                 136
bounded degenerate omissions        30

final declarations
  opaque                           445
  cutout                            12
generic camera rejections            0
```

The differing triangle/declaration counts are expected for partial
contributions. Conservation is checked by source identity and declared output
count rather than by assuming one source triangle always becomes one output
triangle.

## Six-ray final-handoff replay

The six retained investigation rays rebuild the ordered preparation from
their own source position and heading, then inspect the final declaration
boundary:

```text
hut-east wall, SEGs 415/423
    terminal rejected; declarations 0

wall 247 east, SEGs 559/567
    terminal rejected; declarations 0

ceiling subsector 104, reached case
    two partial source triangles
    two authorized horizontal intervals
    eight final declarations

wall 247 west, SEGs 559/567
    terminal rejected; declarations 0

ceiling subsector 149, rejected case
    source-protocol associations/destinations/dispositions/declarations 0

ceiling subsector 104, rejected case
    source-protocol associations/destinations/dispositions/declarations 0
```

The integrated report concludes:

```text
cases=6
conservation=balanced
generic-filter=none
```

The distinction between the two ceiling-104 rays is deliberate: the same
source plane may be partially authorized from one view and rejected before
lowering from another. The renderer receives only the resulting ordinary
declarations and no Doom visibility vocabulary.

## Commands

```powershell
cargo test -p hello-doom-e1m1 --bin static_scene --quiet

cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --render-strategy=ordered-occurrence-prepared-full `
  --ordered-occurrence-prepared-report `
  --no-walk-collision

cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --render-strategy=ordered-occurrence-prepared-full `
  --ordered-occurrence-six-ray-report `
  --no-walk-collision
```

The focused native test gate passed 67 `static_scene` tests. Windows emitted
known incremental-cache hard-link fallback warnings; they did not affect the
test or replay result.

## Deliberate limits

- This is native structural evidence. Browser final-handoff parity remains
  open.
- The six deterministic rays close their source-identity claims, not the full
  manual E1M1 visual matrix.
- Free look, near-wall movement, runtime state refresh and camera jitter still
  require integrated visual/structural evidence.
- The current integration prepares once at startup and intentionally fixes the
  reconstruction camera. The six-ray report proves independent view inputs,
  not live camera-driven replacement of prepared declarations.
- The browser E1M1 consumer does not yet share this Doom-private preparation
  seam. Synthetic browser controls therefore do not close final-handoff
  parity.
- Balanced lowering does not by itself prove that every required visible
  floor, ceiling or shared boundary is present.
- No stable renderer contract is admitted by this result.

## Validation limitation

Focused tests and both canonical headless reports pass. A focused strict
Clippy run currently stops in neighboring dirty campaign code at
`hello-doom-visibility-conformance/src/relational_classifier.rs` because
`OrderedAuthorityResolution` triggers `clippy::large_enum_variant`. That
baseline warning is outside the Slice 6B handoff changes and is not counted as
a 6B failure or silently suppressed.
