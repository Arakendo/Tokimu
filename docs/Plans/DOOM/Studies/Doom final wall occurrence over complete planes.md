# Doom Final Wall Occurrence Over Complete Planes Study

| Field | Value |
| --- | --- |
| Status | Slices 0–1 complete; live walkabout ready |
| Scope | Isolate final ordered wall participation while leaving complete persistent planes untouched |
| Parent review | [AR-0030](../../../Architectural%20Reviews/AR-0030-source-owned-presentation-preparation-boundary.md) |
| Geometry baseline | `global-full-submission` |
| Wall evidence | Final ordered wall-tier occurrences and finite source-relative fragments |
| Plane policy | Preserve all global-full floor and ceiling declarations |
| Sky policy | Presentation output and diagnostics only; no ordinary-geometry rejection authority |
| Stable API authority | None |

## Question

How much of E1M1's far-field resurrection can be removed using exact final
Doom wall participation alone, without allowing the unresolved plane problem
to create holes or allowing sky geometry to decide omission?

## Candidate Dataflow

```text
complete global-full declarations
    ├── floors and ceilings → retain unchanged
    └── walls
          ↓
       final ordered wall-tier occurrence
          ├── present  → finite ordered wall fragment
          ├── partial  → proved clipped fragment
          ├── absent   → no wall declaration
          └── ambiguous→ fail open to complete global-full set

ordinary declarations
    ↓
renderer
```

This is a family-isolation experiment, not the final Doom visibility system.

## Why Walls First

Retained evidence already distinguishes:

```text
wall 241
    SEG admission                 yes
    final retained wall cells     no
    final declarations             0

wall 135
    final retained wall support   yes
    final declaration             yes
```

Walls preserve SEG, linedef, side, tier, horizontal support and vertical
support strongly enough to produce finite world fragments. Planes do not yet
preserve equally useful source provenance after Classic visplane-style
merging. Combining the families previously made plane holes dominate visual
evaluation.

## Binding Invariants

1. Every original floor and ceiling declaration remains byte-for-byte
   unchanged in the prepared set.
2. Sky hits, materials and boundaries have no wall omission authority.
3. Only final ordered wall declarations replace global wall declarations.
4. Whole and partial ordered wall results retain their finite source-relative
   geometry and UVs.
5. Terminally absent wall occurrences produce no replacement fragment.
6. Any unresolved wall preparation fails open to the complete global-full
   declaration set for that refresh; it does not install a partial candidate.
7. Candidate preparation completes and verifies conservation before atomic
   replacement.
8. Doors/platforms remain application-owned movement policy. Preparation
   consumes the current runtime map snapshot.
9. Renderer vocabulary remains ordinary declarations only.
10. No stable Tokimu API follows from this corpus-private A/B.

## Controls

The first headless gate retains:

- wall 241 as the exact negative wall control;
- wall 135 / `SUPPORT2` as the exact positive wall control;
- walls 159/160 and wall 203 from the sky-correlation falsifiers as valid
  walls that must remain source-present;
- the five historical terminal wall/plane targets, while treating plane
  outcomes as observations only; and
- complete plane declaration identity/count conservation.

## Slice 0 — Structural Preparation

- [x] Add an explicit `final-wall-occurrence-global-planes` strategy name.
- [x] Prepare ordered wall declarations from the actual camera/runtime
      snapshot.
- [x] Retain all global-full plane declarations unchanged.
- [x] Fail open atomically when wall preparation is unresolved.
- [x] Report wall and plane family conservation separately.

Acceptance: no plane can disappear because of this strategy, and the renderer
receives only ordinary declarations.

## Slice 1 — Headless Exact Controls

- [x] Prove wall 241 remains absent.
- [x] Prove wall 135 remains present.
- [x] Prove valid walls 159/160 and 203 remain available in their retained
      exact views.
- [x] Prove all global-full plane declarations are retained unchanged.
- [x] Run twice with a deterministic fingerprint.

Acceptance: all exact wall controls agree and plane conservation is exact.

## Implemented Slices 0–1 Result

The headless gate is:

```text
cargo run -q -p hello-doom-e1m1 --bin static_scene -- \
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD \
  --final-wall-occurrence-report
```

It passes all five exact controls:

```text
wall 241    absent
wall 135    present at 104.838
wall 159    present at 492.224
wall 160    present at 421.274
wall 203    present at 848.388
```

All `853` global-full plane declarations remain identical in every prepared
control. No source plane cell, sky surface, SEG proxy or inferred bound can
remove them. The unresolved-wall path returns the complete global-full opaque
and cutout sets instead of installing a partial candidate.

Two runs produced fingerprint `cb3ed7d517e1b942`. Conservation is balanced
and renderer vocabulary remains ordinary declarations only.

A native `--measure-two-frames` smoke run also completed successfully. The
source-spawn preparation emitted `309` opaque wall declarations, `12` cutout
wall declarations and all `853` global planes (`1,162` opaque plus `12`
cutout draws total). The first live refresh installed the same family counts
atomically and the warm frame submitted all `1,174` candidates.

The live command is:

```text
cargo run -p hello-doom-e1m1 --bin static_scene -- \
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD \
  --render-strategy=final-wall-occurrence-global-planes
```

This command is ready for Slice 2 human walkabout. Its visual question is
specifically which wall resurrection disappears while all plane leakage stays
available for separate diagnosis.

## Slice 2 — Live Walkabout

Only after Slices 0–1 pass:

- [ ] Launch the opt-in live strategy with normal free look and movement.
- [ ] Inspect spawn, hut approaches, far-left structure, windows/stairs,
      green-room cutout and sharply pitched views.
- [ ] Compare visible wall cleanup separately from untouched plane leakage.
- [ ] Exercise one door/platform snapshot and verify prepare-before-replace.

Acceptance: the experiment clarifies how much resurrection belongs to walls
without producing missing planes. Missing valid walls park the candidate.

## Decision Gate

The result may establish a workable wall-family preparation policy. It cannot
settle plane participation, sky composition or a provider-neutral visibility
contract. Those remain separate decisions.
