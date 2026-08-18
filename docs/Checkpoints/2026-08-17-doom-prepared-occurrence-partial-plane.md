# Doom Prepared-Occurrence Partial-Plane Checkpoint

| Field | Value |
| --- | --- |
| Date | 2026-08-17 |
| Scope | Authoritative ordered-result handoff and focused partial-plane realization |
| Status | Structurally conserved but visually falsified; parked |
| Governing review | AR-0030 Cycles 31-33 |

## Semantic Result

The corpus-private unit is a prepared presentation occurrence:

```text
source contribution + current view + runtime snapshot
    -> zero bounded occurrences
    -> one bounded occurrence
    -> several bounded occurrences
```

Zero is authoritative. The global source shell is not submitted afterward.
Spatial relevance remains separate evidence and does not reopen an absent
occurrence.

## Partial Plane

The former candidate clipped inferred whole-subsector triangles only against
continuous horizontal occurrence wedges. It lacked Doom's authoritative
vertical plane coverage and was visually falsified by missing foreground.

The focused replacement observes the exact ordered vertical plane cells and
matches them by:

- plane kind;
- source sector and subsector;
- current source height;
- texture and light level;
- owning source SEG.

Only source triangles already classified `partial-plane` are intersected with
these cells. Whole plane triangles retain their ordinary geometry. Terminally
rejected triangles emit no declaration. Cell intersections lower to ordinary
triangles and are combined into one mesh per surviving source triangle; the
renderer learns no Doom vocabulary.

The retained ceiling-104 ray proves the direct representation is available:

```text
plane                         sector 40 / subsector 104 / CEIL3_5
continuous horizontal domains 2
exact ordered plane cells      13
cell owners                    SEG 310, SEG 311
owning subsectors              {104}
final ordinary declarations    1
```

The other five retained leak contributions still produce zero declarations.

## Source-Spawn Structural Result

```text
opaque wall declarations       309
cutout declarations             12
floor declarations              28
ceiling declarations            15
total opaque declarations      352

partial plane-domain cells    2265
partial plane fragments       3632
lowered plane triangles       3432
combined plane meshes           43
degenerate omissions           212
unresolved plane lowering        0
```

All source, destination, source-triangle, fragment and declaration
conservation checks are balanced.

## Lifecycle

The live native composition still rebuilds from the current camera and an
application-owned immutable runtime-height projection. It now retains the
identity of the last completely installed preparation and skips identical
stationary frames. Camera or runtime changes prepare and validate the complete
next result before declarations and identity are replaced.

## Validation

```powershell
cargo test -p hello-doom-e1m1 --bin static_scene

cargo run -p hello-doom-e1m1 --release --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --render-strategy=ordered-occurrence-prepared-full `
  --ordered-occurrence-six-ray-report

cargo run -p hello-doom-e1m1 --release --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --render-strategy=ordered-occurrence-prepared-full `
  --ordered-occurrence-live-refresh-report
```

Observed: `80/80` focused tests pass; six-ray and five-pose refresh reports are
balanced.

## Pitch Falsifier

A focused attempt added camera pitch to the inverse projection of the exact
Classic Doom plane rows. It failed the retained ceiling-104 control:

```text
source disposition          partial plane
continuous intervals        unchanged
expected declarations       1
pitched-row declarations     0
```

The attempted code was removed. These rows retain authority only for the
unpitched source projection that generated them; deriving coverage for a
pitched Tokimu camera requires an explicit AR-0030 decision. Horizontal yaw,
movement, runtime snapshots, atomic replacement and conservation evidence are
unchanged.

## Native Visual Falsifier

The native walkabout subsequently produced an active `365`-draw view with
large opaque foreground regions obscuring roughly half of the E1M1 spawn room.
This invalidates the partial-plane handoff as a live rendering candidate even
though its structural ledgers are balanced. The implementation is retained for
diagnosis only; no visual acceptance claim survives.

## Open Acceptance

- verify the hut and valid far-left building remain complete;
- verify beside-hut and above-wall leaks disappear;
- verify no large peripheral holes or finite view box remain;
- exercise yaw, movement, near-wall jitter and pitch continuously;
- exercise doors/platforms visually;
- route a browser E1M1 consumer through the same shared Rust preparation unit.

The native interactive path was launched for this visual pass. No claim of
visual completion is made until those observations return.
