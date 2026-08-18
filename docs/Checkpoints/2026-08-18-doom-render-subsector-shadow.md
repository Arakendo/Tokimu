# Doom Render-Subsector Slices 0–2 Checkpoint

Date: 2026-08-18  
Controlling study: [Doom Render-Subsector Actual-Camera Preparation](../Plans/DOOM/Studies/Doom%20render-subsector%20actual-camera%20preparation.md)  
Review: [AR-0030](../Architectural%20Reviews/AR-0030-source-owned-presentation-preparation-boundary.md)

## Disposition

Slices 0–2B are complete as a corpus-private, headless shadow. Slice 3 now also
produces a complete conserved prepared view made only of ordinary declarations,
but that view is not installed into renderer submission. Live atomic
replacement remains the next implementation gate.

## Persistent Geometry

The deterministic E1M1 inventory reports:

- map fingerprint `7a0b92a71c0e0a6d`;
- baseline prepared-view fingerprint `63baad28804df6fb`;
- `237/237` source subsectors represented as render subsectors;
- `474/474` plane units represented by `926` finite triangles;
- `732/732` source SEGs and `1,256/1,256` wall-tier triangles correlated;
- `439` ordinary and `35` sky plane roles;
- zero missing surfaces, unresolved boundaries, degenerate triangles,
  containment failures or winding failures;
- two exact ordered SEG loops, one source loop that strictly refines unused BSP
  path space, and 234 finite BSP-path implicit boundaries.

The retained predecessor baselines remain separate: global-full contains
`1,849` triangles and the falsified Cycle 31 visual contained `365` draws.

## Actual-Camera Shadow

Nine fixed poses cover source spawn, pitch `+20/-20`, yaw `+30/-30`, forward
movement and return, the retained near-wall scan sample, and an off-axis wall
edge. Actual 3D render-subsector bounds are compared with per-triangle brute
force controls. The result has zero geometric false negatives.

Repeated runs produced identical fingerprints:

- camera matrix `20042a967aaec227`;
- six-ray replay `64b7e8b802b10d14`.

Surface outcomes remain distinct: retained, outside frustum, source covered and
unresolved. Exact finite horizontal domains convert conservative AABB-only
overlaps into diagnosed outside-frustum results. No unresolved surface remains
in the retained pose matrix.

## Six-Ray Falsifiers

All retained controls agree with the new shadow:

- hut east wall 230: rejected;
- wall 247 from east and west: rejected;
- subsector 104 ceiling from the reached pose: retained;
- subsector 149 ceiling from its rejected pose: source covered;
- subsector 104 ceiling from its rejected pose: source covered.

The same subsector 104 ceiling therefore changes participation with the actual
view rather than being admitted or rejected globally.

## Ordinary Failure Found And Repaired

The first continuous-range implementation let a solid SEG crossing the camera
near plane close an entire horizontal interval. That incorrectly classified
the required subsector 104 ceiling as source covered. Classic source evidence
showed the leaf was reached.

The repaired rule requires all of the following before a SEG contributes
terminal solid coverage:

1. source-facing directed SEG orientation;
2. both finite endpoints strictly in front of the actual camera near plane;
3. source classification as one-sided or otherwise closed; and
4. a finite interval in the actual horizontal field of view.

Near-plane ambiguity now fails open. Focused unit coverage protects facing and
endpoint-depth behavior.

## Authority And Remaining Work

- Source child and SEG endpoint boxes do not reject reconstructed planes.
- Persistent surfaces and the actual Tokimu frustum veto unsafe omission.
- Doom-private near-first order and source closure supply participation
  evidence.
- Sky remains presentation meaning, not depth-bearing closure geometry.
- `tokimu-render` remains Doom- and BSP-neutral.

## Slice 3 Prepared-View Shadow

Seven fixed prepared-view poses cover spawn, yaw/pitch, movement/return and an
automatically derived owning-side view of green-room linedef 464. Every source
triangle has one terminal outcome: ordinary declaration, sky-background
reference, outside-frustum omission or source-covered omission. The matrix is
deterministic at fingerprint `d46154cd27ec89a9`; spawn and return both produce
declaration fingerprint `1d6d1c0a1f161103`.

An initial prepared ledger reported zero cutouts because texture-name matching
was not source classification. The repaired correlation uses the exact
linedef/sidedef identities from the existing 26 cutout source draws. The
green-room owning-side control now produces 36 declarations: 19 ordinary
plane, 15 opaque wall and two cutout wall declarations, with fingerprint
`b9153eb917601947`.

This repaired an incomplete source universe before live installation. The
parked Classic row-cell lowering and global shell do not participate.

The remaining Slice 3 item is composition-local atomic installation of a
complete prepared result. Slice 4 lifecycle refresh and native visual review
remain open; the current headless report explicitly states
`renderer-submission=unchanged`.

## Slice 2B Connectivity/BVH Shadow

The finite render-subsector graph contains 237 cells, 607 undirected shared
boundary relationships and no isolated cell. Its edge inventory is:

- 274 implicit partitions;
- 146 positive two-sided openings;
- 130 closed solids;
- 28 paired-sky openings;
- 28 unresolved fail-open relationships; and
- one masked-middle opening.

The graph fingerprint is `13500e039c076c04`; the six-specimen two-policy
matrix fingerprint is `1d5228cf89a8478b`.

The conservative policy crosses every positive, masked, paired-sky, implicit
and unresolved edge and reaches 236/237 cells. The paired-sky-terminal
falsifier reaches 233/237 cells. Exact BVH rays still hit all six retained
targets, but both connectivity policies reach all six target cell sets. They
therefore disagree with the ordered source oracle on all five rejected
far-field specimens and agree only on the retained subsector 104 ceiling.

The recorded chains explain why. Only the hut-east wall's shortest
conservative chain crosses a paired-sky boundary. Both wall-247 views and both
rejected ceiling targets remain connected by ordinary or implicit paths; sky
terminality cannot distinguish them. Connectivity and the BVH are useful
topology/geometric-relevance evidence, but neither supplies source
presentation participation. Both remain shadow-only and no renderer
submission changed.
