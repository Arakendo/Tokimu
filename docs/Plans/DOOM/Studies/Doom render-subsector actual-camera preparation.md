# Doom Render-Subsector Actual-Camera Preparation

| Field | Value |
| --- | --- |
| Campaign | DOOM |
| Role | Authorized bounded representation and preparation experiment |
| Status | Active — Slices 0–2B complete; Slice 3 prepared-view shadow complete, live installation pending |
| Parent review | [AR-0030](../../../Architectural%20Reviews/AR-0030-source-owned-presentation-preparation-boundary.md) |
| Controlling plan | [DOOM WAD Checklist](../DOOM%20WAD%20Checklist.md) |
| Falsified predecessor | [Doom Ordered Source-Occurrence Preparation](Doom%20ordered%20source%20occurrence%20preparation.md) |
| Parked diagnostic predecessor | [Doom BSP Presentation-Domain Resolver Study](Doom%20BSP%20presentation-domain%20resolver%20study.md) |
| Primary precedent | [Hardware Doom Arbitrary-Pitch Plane Preparation Precedent](../Evidence/Hardware%20Doom%20arbitrary-pitch%20plane%20preparation%20precedent.md) |
| Initial corpus | Reviewed `DOOM1.WAD` E1M1 |
| Stable API authority | None |
| Renderer changes authorized | None |

## Decision And Purpose

AR-0030 authorizes one corpus-local experiment using a Doom-private render
subsector as the persistent world/render representation consumed by
actual-camera preparation.

The experiment replaces neither Tokimu's renderer nor its conservative spatial
query capability. It asks whether the Doom adapter can prepare a complete,
source-faithful E1M1 view by combining:

- finite world-space geometry built from Doom subsectors;
- source wall, floor, ceiling and sky meaning;
- current runtime sector heights;
- Doom BSP ordering and coverage evidence; and
- the actual Tokimu camera, including yaw, viewport, field of view and pitch.

The output remains ordinary opaque and cutout render declarations plus
conservation evidence. No Doom, BSP, render-sector or sky-boundary vocabulary
crosses into `tokimu-render`.

## Why This Representation Is Different

The falsified predecessor treated Classic Doom's exact screen-row plane cells
as final world-space occurrence geometry. Those cells were valid evidence for
the unpitched source projection that produced them, but they did not remain a
complete plane representation under Tokimu movement and free look.

This study instead begins with persistent finite world-space surfaces:

```text
decoded Doom map + current runtime-height snapshot
        ↓
Doom-private render subsectors
        ↓
actual-camera Doom traversal and participation
        ↓
ordinary Tokimu declarations + conservation evidence
        ↓
composition-local atomic replacement
```

Classic column, span and ordered-occurrence observations remain diagnostic
oracles. They do not define the new geometry.

## Initial Proof Fence

Before compatibility expansion, the experiment must prove exactly three
things:

1. **Geometry completeness** — at the retained E1M1 views, finite
   render-subsector geometry contains every required nearby floor and ceiling
   surface without the `365`/`458`-draw holes.
2. **Source participation** — the five known far-field leaks still produce no
   presentation, while the retained partial ceiling is represented without
   admitting its whole source-sector plane.
3. **Pitch continuity** — neutral pitch, bounded up/down pitch, yaw, movement
   and return poses remain continuous without reverse-projecting Classic
   visplane rows.

Neutral-pitch results must agree with the retained Classic evidence wherever
that evidence has exact authority. A disagreement must be explained by the
new representation; arbitrary pitch is not permission to discard the neutral
oracle.

## Private Candidate Model

The first implementation should remain a concrete corpus-private data model,
not a public trait or new crate. Each render-subsector unit records at least:

```text
source subsector identity
ordered finite boundary loop and owning SEGs
source sector and resolved render-sector association
current floor and ceiling heights, textures and light
ordinary world-space floor and ceiling triangles
sky or ordinary role for each plane
finite wall-tier sources along the boundary
supported, unresolved or hack-required provenance
```

The model must preserve source identity separately from generated surface and
declaration identity. One source sector may own several render subsectors; a
reached subsector never grants authority over an entire non-convex sector
plane.

For E1M1, render-sector association begins as a proved map fact, not an assumed
identity. Self-reference, transparent-door, portal or malformed-map behavior
is recorded as unsupported or unresolved unless the corpus demonstrates that
the slice needs a narrowly implementable case.

## Authority Rules

- Ordered SEG vertices or another source-proved finite loop define a
  render-subsector plane domain. Classic plane rows do not.
- Doom BSP nodes order traversal and may conservatively prune only the source
  representation their bounds actually contain.
- A child or SEG endpoint box cannot reject a larger reconstructed plane.
- Actual plane and wall geometry is tested against the actual Tokimu frustum.
- Horizontal coverage may become terminal only from a source contribution
  proven to close the relevant Doom presentation interval.
- Pitch-aware participation uses actual plane/wall endpoint heights and the
  actual camera. It must not reinterpret Classic row coordinates.
- Sky is source presentation meaning. It does not become depth-bearing world
  closure geometry and does not itself reject far ordinary geometry.
- Ambiguity fails open during shadow stages and remains explicitly diagnosed.
  Presentation-affecting omission requires positive, source-appropriate
  evidence.
- The BVH may provide conservative query and ray-test controls. It has no Doom
  participation or occlusion authority.

The practical authority split is:

```text
render-subsector world geometry
        ↓
Tokimu BVH: where geometry is / conservative spatial relevance
        ↓
Doom-private preparation: whether source presentation belongs
        ↓
ordinary Tokimu declarations
```

Query and preparation may be scheduled in another order when useful, but this
authority direction may not invert.

## Slice 0 — Freeze Baselines And Harness

- [x] Name the new strategy and keep it opt-in; do not silently replace the
      global-full or parked ordered-occurrence strategies.
- [x] Retain the global `1,849`-triangle inventory and the Cycle 31 `365`-draw
      visual falsifier as separate baselines.
- [x] Retain the six leak/partial rays, source-spawn pose, marked-hole
      observations, scan/LOOK replay and door/platform snapshot controls.
- [x] Add a deterministic headless report identity containing map, camera,
      viewport/projection and runtime-height snapshot fingerprints.
- [x] Define source, render-subsector, surface, prepared-view and declaration
      conservation ledgers before presentation-affecting omission is enabled.

Acceptance: all later reports can be compared to immutable predecessor
evidence without launching a GPU window.

## Slice 1 — Build Persistent E1M1 Render Subsectors

- [x] Reconstruct each supported subsector's ordered finite boundary loop from
      decoded source records and retain the exact SEG correlation.
- [x] Compare the ordered-loop domain with the existing BSP-path inferred
      convex region; explain every mismatch rather than selecting whichever
      polygon gives a preferred image.
- [x] Triangulate ordinary floor and ceiling surfaces in world space without
      using camera rows, columns or a fixed reconstruction view.
- [x] Attach finite wall-tier sources without merging distinct linedef,
      sidedef, SEG or sector provenance.
- [x] Resolve and audit E1M1 render-sector association, sky roles, degenerate
      leaves and unsupported source patterns.
- [x] Emit a deterministic headless inventory for every render subsector:
      source subsector, render-sector association, boundary fingerprint,
      floor/ceiling triangle fingerprints, sky roles, runtime-height revision
      and unresolved/hack status.
- [x] Prove triangle containment in the owning finite domain, non-degenerate
      winding, deterministic ordering and source/surface conservation.

Acceptance: every supported E1M1 render subsector has deterministic finite
surface geometry at neutral runtime heights, and every unsupported case is
enumerated with source identity and reason. The inventory is required before
the first presentation-affecting visual candidate.

Stop and return to AR-0030 if E1M1 requires a general portal contract, a broad
compatibility-hack taxonomy, or geometry whose ownership cannot be established
from repository/source evidence.

Broader E1M1 hack or compatibility coverage begins only after the initial
geometry, source-participation and pitch-continuity proofs pass.

## Slice 2 — Actual-Camera Shadow Traversal

- [x] Traverse the decoded Doom BSP near first using the actual Tokimu camera's
      position, yaw, viewport and horizontal field of view.
- [x] Add pitch-aware conservative tests over actual render-subsector floor,
      ceiling and wall endpoint heights.
- [x] Distinguish ordering, outside-frustum, source-covered, retained and
      unresolved outcomes. Do not collapse traversal absence into rejection.
- [x] Keep source child bounds limited to source-subtree pruning; veto unsafe
      plane omission with the actual render-subsector geometry.
- [x] Compare shadow candidates with brute-force actual-geometry frustum/ray
      controls and the retained Classic source observations.
- [x] Extend headless scan replay to neutral pitch, positive/negative pitch,
      off-axis pixels and near-wall poses.

Acceptance: the traversal explains each retained ray and every sampled
disagreement without modifying renderer submission.

Slices 0–2 acceptance evidence is retained in
[the 2026-08-18 checkpoint](../../../Checkpoints/2026-08-18-doom-render-subsector-shadow.md).
The shadow remains non-authoritative: no declaration or renderer submission is
changed until Slice 3 produces and conserves a complete prepared view.

## Slice 2B — Connectivity/BVH Shadow

This authorized refinement tested whether camera-local reachability over the
finite render-subsector representation could supply the missing semantic value
without replaying Classic Doom's incremental renderer.

- [x] Build a deterministic graph over shared finite render-subsector
      boundaries, not raw source-sector adjacency.
- [x] Classify edges as implicit partition, closed solid, positive opening,
      masked-middle opening, paired-sky opening or unresolved fail-open.
- [x] Run a conservative variant in which paired-sky boundaries remain open
      and a falsifier variant in which they are terminal.
- [x] Correlate graph reachability with exact BVH ray relevance and the retained
      six-ray ordered-coverage oracle.
- [x] Report shortest boundary chains, source linedefs, edge reasons and the
      first paired-sky terminal on the conservative chain.
- [x] Keep both variants shadow-only and make no declaration or renderer
      submission change.

E1M1 produced 237 cells, 607 undirected relationships, no isolated cells and a
deterministic graph fingerprint of `13500e039c076c04`. Conservative traversal
reached 236 cells from every specimen. Treating paired-sky edges as terminal
still reached 233 cells and did not change the classification of any retained
specimen: both variants retained all five exact BVH-visible contributions that
the ordered source oracle rejects. Each therefore disagreed with the oracle in
five of six cases.

Acceptance disposition: complete as a negative shadow result. Plain
connectivity establishes navigable/topological reachability, not presentation
participation. Paired-sky terminality is neither necessary nor sufficient for
the retained far-field exclusions and must not be promoted to authority.

Replay the retained report with:

```powershell
cargo run -q -p hello-doom-e1m1 --bin static_scene -- corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD --render-subsector-connectivity-report
```

## Slice 3 — Doom-Private Prepared View

- [x] Produce a complete prepared-view result from render-subsector traversal,
      including whole retained surfaces, source-authorized bounded wall work,
      terminal omissions and explicit unresolved records.
- [x] Lower surviving surfaces to ordinary opaque/cutout declarations using
      existing provider-neutral material and mesh facilities.
- [x] Preserve sky as the existing non-world presentation pass plus explicit
      Doom-private boundary decisions; do not add sky depth geometry.
- [ ] Replace the current composition-local declaration set only after the
      complete new result passes conservation.
- [x] Keep the parked Classic row-cell lowering available only as a diagnostic
      comparison; it must not fill holes in the new result.

Acceptance: all sources, surfaces and outcomes balance, unresolved items remain
visible in diagnostics, and the renderer receives no new semantic vocabulary.

## Slice 4 — Runtime And Camera Lifecycle

- [ ] Rebuild or reuse preparation from an explicit identity covering camera,
      viewport/projection and runtime-height snapshot.
- [ ] Exercise stationary frames, bounded yaw, forward/back movement, positive
      and negative pitch, camera jitter and near-wall motion.
- [ ] Exercise the retained door and platform snapshots through the same
      preparation entry point.
- [ ] Prove prepare-then-replace atomicity: a failed or incomplete preparation
      cannot retire the currently installed declaration set.
- [ ] Record build/query/lowering timings diagnostically without promoting a
      performance budget or incremental-update contract.

Acceptance: camera and runtime changes cannot display stale or half-prepared
geometry, and application-owned movement policy does not enter preparation.

## Slice 5 — Native Visual And Semantic Gate

- [ ] At source spawn, verify the complete room at neutral pitch and through
      bounded pitch/yaw movement without the large opaque partitions or marked
      sky leaks.
- [ ] Verify the hut, far-left structure, window/opening regions, first door,
      moving platform, green-room cutout and EXIT controls.
- [ ] Verify that the five retained far-field leak contributions remain absent
      where source presentation excludes them.
- [ ] Verify bounded treatment of the retained partial ceiling without
      resurrecting the whole global plane.
- [ ] Use diagnostic textures plus automated SCAN/LOOK representatives to
      retain exact source reasons for any suspicious visible region.
- [ ] Capture reproducible screenshots and headless reports for accepted and
      falsified poses.

Acceptance requires both structural conservation and maintainer visual
inspection. A balanced ledger, reduced draw count or passing screenshot at one
pose is insufficient by itself.

## Slice 6 — Portability Gate

This slice begins only after native visual acceptance.

- [ ] Extract or reuse the same Rust-owned render-subsector preparation unit
      from the browser E1M1 consumer.
- [ ] Keep native/browser differences limited to provider and realization
      mechanics.
- [ ] Replay matching camera/runtime snapshots and compare preparation
      fingerprints and conservation ledgers.
- [ ] Record any WASM-specific allocation or latency finding without changing
      stable contracts merely to satisfy the experiment.

Acceptance: native and browser call the same semantic preparation unit and
produce equivalent ordinary declaration membership for retained controls.

## Falsification And Parking Criteria

Park the candidate and preserve evidence if any of these occurs:

- complete render-subsector geometry still leaks the retained far field;
- required spawn-room geometry disappears under neutral pitch or ordinary
  movement;
- whole-sector admission is needed to conceal missing subsector semantics;
- Classic row cells or the global shell are required as an unexplained repair;
- sky must become depth-bearing occlusion geometry;
- correctness requires renderer-owned Doom/BSP vocabulary;
- safe behavior requires a new stable/public contract before a corpus-local
  result exists; or
- required portal/hack behavior creates an unresolved ownership decision.

An ordinary local triangulation, correlation, diagnostic or conservation defect
is implementation work and should be repaired inside this study. A finding
that changes authority, ownership, stable contracts or dependency direction
returns early to AR-0030.

## Completion Evidence

The final review must retain:

- render-subsector construction and unsupported-case inventories;
- deterministic geometry and conservation reports;
- actual-camera traversal and sampled disagreement reports;
- the six retained ray outcomes;
- door/platform runtime-snapshot fingerprints;
- native visual captures at the required poses;
- draw/declaration counts as observations, not correctness claims; and
- explicit confirmation that `tokimu-render` remains Doom-neutral.
