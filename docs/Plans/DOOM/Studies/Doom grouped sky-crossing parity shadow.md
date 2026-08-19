# Doom Grouped Sky-Crossing Parity Shadow Study

| Field | Value |
| --- | --- |
| Status | Live visual falsification authorized after failed correlation gate |
| Scope | Reclassify ordered, grouped sky intersections before complete-world targets |
| Parent review | [AR-0030](../../../Architectural%20Reviews/AR-0030-source-owned-presentation-preparation-boundary.md) |
| Predecessor | [Doom One-Way Sky-Occlusion Correlation Shadow](Doom%20one-way%20sky-occlusion%20correlation%20shadow.md) |
| Renderer changes authorized | None |
| Stable API authority | None |

## Correction And Question

The predecessor falsified only the coarse rule:

```text
any sky-related hit before target → hide target
```

It did not test whether the number of distinct, ordered hit groups has useful
parity. Its eight exact-present counterexamples each reported two groups,
while the five retained unwanted far-field controls reported one and the five
required nearby controls reported zero:

```text
even grouped crossings → World candidate
odd grouped crossings  → Sky candidate
```

This study tests that correlation without claiming that Doom's open sky
surfaces form a closed volume.

## Independent Evidence

Every retained ray records:

- the complete ordinary target and exact frozen-view source result;
- every ordered distinct sky group before the target;
- distance, semantic identity, family and raw triangle multiplicity;
- raw winding orientation, with diagnostic-only authority;
- source semantic side only where the source ceiling-plane kind supports it;
- grouped count, parity and family sequence.

`source-sky-open-plane` and `paired-sky-height-discontinuity` remain separate.
A source sky ceiling has a locally meaningful underside/topside. A paired-sky
height discontinuity has sky ceilings on both sides and remains semantically
unoriented. Neither fact proves a global World/Sky volume.

Exact-present, partial and absent remain distinct. Ordered frozen-view source
participation is a correlation axis; it is not a canonical free-look pixel
oracle and does not override the complete world.

## Slice 1 — Existing 36-Ray Reclassification

- [x] Reuse the exact deterministic eight-pose `32 × 20` scan.
- [x] Select the same 36 rays with one or more grouped sky hits.
- [x] Retain the entire ordered group sequence for every ray.
- [x] Cross-tab grouped parity against exact, partial and absent results.
- [x] Keep raw winding diagnostic-only and paired-sky semantic side unresolved.
- [x] Run twice and preserve deterministic results.

The first correlation gate inspects `odd + exact-present` and `even + absent`
counterexamples. Partial results stay separate. Any disagreement is retained
for inspection before broadening; it is not silently reclassified.

### Implemented Result

Run:

```text
cargo run -q -p hello-doom-e1m1 --bin static_scene -- \
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD \
  --grouped-sky-crossing-parity-report
```

The retained corpus was reproduced exactly:

```text
rays                         36
raw/grouped hits          46/46
paired/source-plane groups 38/8

odd  + exact-present          0
odd  + absent                26
even + exact-present          8
even + absent                 2
```

The eight former one-way falsifiers do support the even/World correlation.
The matrix nevertheless contains two even/absent specimens at hut-east cells
`(115,105)` and `(125,105)`, targeting walls 279 and 290. Both have the same
family sequence as six successful hut exact-present specimens:

```text
paired-sky-height-discontinuity
    → source-sky-open-plane
```

The source-plane hit is backside in all eight such rays. Raw winding remains
diagnostic-only, and the paired boundary remains semantically unresolved.
Consequently raw parity, family sequence, and the locally proved ceiling side
cannot distinguish the two absent targets from the six exact-present targets.

Two runs produced fingerprint `26afe710bce75ebc`. Conservation is balanced
and renderer mutation is false.

### Slice 1 Disposition

Grouped parity is an interesting correlation, but it fails its authorized
exact-36 gate. Slice 2 and live A/B work are not entered. Repairing the result
would require a new semantic discriminator rather than more parity sampling.
The complete 36-row report remains available for any future hypothesis.

## Slice 2 — Conditional Full Corpus

Only if Slice 1 survives without unexplained counterexamples:

- [ ] Classify all 5,120 existing rays, including zero-crossing targets.
- [ ] Preserve canonical expected-presentation controls separately from
      ordered source-participation observations.
- [ ] Compare family sequences and exact frozen controls.
- [ ] Run twice and preserve conservation and fingerprint evidence.

Zero crossings leave the complete world available by hypothesis. Therefore a
zero-crossing ordered-source absence is not automatically a canonical parity
failure: the retained steep-pitch holes already demonstrate that frozen Doom
occurrence absence and desired free-look presentation are different claims.

Not executed: Slice 1 contains two unexplained even/absent counterexamples.

## Slice 3 — Conditional Live A/B

The maintainer subsequently authorized a reversible visual experiment despite
the failed source-correlation gate. This does not reinterpret either
even/absent specimen or promote parity to source truth. It asks only whether
grouped paired-skywall and source-sky-plane parity produces a useful
actual-camera presentation over the complete world.

- [x] Preserve the complete ordinary world as the input declaration set.
- [x] Render a complete-world opaque/cutout depth prepass.
- [x] Invert one stencil bit for every double-sided paired-skywall fragment
      that lies before the nearest world fragment.
- [x] Invert the same stencil bit for every source `F_SKY1` plane fragment
      that lies before the nearest world fragment.
- [x] Render ordinary world color only where the crossing bit is even.
- [x] Preserve the original full submission as the no-flag control.
- [x] Complete a native two-frame mechanism proof.
- [x] Extend interactive `LOOK` and headless `--look-ray-report` with ordered
      paired-skywall crossings, seam-collapsed source identities, parity, and
      the predicted retain/mask result.
- [x] Refine reconstructed plane support from validated closed subsector SEG
      loops or compatible SEG half-planes plus implicit BSP boundaries, with
      BSP-path regions retained as an explicit fail-open fallback.
- [ ] Conduct the adversarial E1M1 walkabout.

Run:

```text
cargo run -q -p hello-doom-e1m1 --bin static_scene -- \
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD \
  --skywall-parity-full
```

The first native mechanism proof used 1,823 opaque and 14 currently-facing
cutout draws, 16 paired-skywall triangles, and 3,693 total draw calls across
the panorama, depth prepass, parity mask, world color, and cursor. It completed
both first and warm frames on Vulkan. Warm command construction was `224 µs`;
warm whole-frame CPU time was `104,483 µs`. These are observations, not an
accepted budget.

The first skywall-only walkabout exposed admitted wall 205 behind a source sky
ceiling at distance `139.203` and paired skywall 253 at `170.985`. Counting
only the wall produced odd parity and masked the wall incorrectly. The grouped
rule counts both crossings, produces even parity, and retains wall 205.

The revised native proof adds 73 source-sky-plane triangles to the stencil
pass and produces 3,766 total draw calls. Its warm command construction was
`325 µs`; warm frame CPU time was `87,929 µs`. These remain observations, not
an accepted budget.

`LOOK` now emits a separate `grouped_sky_parity` line. It orders both source
families before the nearest prepared world hit, collapses triangles sharing
one source-surface identity, and reports `even/retained` or `odd/masked`. This
is an exact CPU-ray prediction over the same prepared geometry, not a GPU
stencil-buffer readback; raster-edge and precision differences therefore
remain possible at boundary pixels.

### Source-boundary surface refinement

The first broad noclip walkabout exposed a different defect from parity. At an
eye height above ordinary play, one ray crossed paired skywall 252 and source
sky ceiling 49, then hit floor subsector 104 far outside its finite SEG
boundary. Even parity correctly retained the target it was given; the target
itself was an oversized BSP-path reconstruction:

```text
subsector 104 SEG boundary: x=1024..1088, y=-3680..-3648
reported reconstructed hit: x=1298.917, y=-3738.395
distance outside SEG envelope: 218.852
```

The opt-in live experiment now performs a Doom-private source-boundary bake
before ordinary flat and source-sky-plane lowering. It joins SEG endpoints by
identity regardless of record order or direction. A recovered loop is used
only when it:

- consumes every subsector SEG exactly once;
- forms one convex cycle;
- has nonzero area; and
- remains contained by the decoded BSP leaf path.

Failure retains the existing BSP-path region. This is deliberately not
player-reachability trimming, visibility filtering, or a claim that all Doom
subsectors have explicit closed SEG loops. On canonical E1M1, 55 of 237 leaves
produce validated loops, 32 materially refine their BSP-path regions, and 182
retain the fallback. The resulting bake contains 998 floor/ceiling triangles.

The exact subsector-104 replay now reports no prepared ordinary hit, while the
wall-205 positive control remains an exact hit after two grouped crossings and
is retained by even parity. Ordinary planes and diagnostic source sky planes
are derived from the same refined surface set, preventing the parity mask and
world geometry from silently using different plane boundaries.

The first walkabout after this closed-loop-only pass still exposed broad
fallback planes. That was expected from the audit: 182 leaves did not have a
standalone closed SEG cycle. A second refinement now starts with each finite
BSP-path region and clips it by the owning/right half-plane of every decoded
SEG in that leaf. This recovers leaves whose boundary is intentionally split
between explicit map lines and implicit BSP partitions. It rejects the result
if the SEG constraints contradict one another, become degenerate, or stop
containing the leaf's decoded SEG endpoints.

Canonical E1M1 now reports:

```text
validated closed loops             55
loop refinements                    32
compatible SEG-half-plane regions 179
SEG-half-plane refinements          92
BSP-path fallbacks                   3  (subsectors 59, 137, 173)
surface triangles                 1020
```

The original subsector-104 negative replay remains absent. The required wall
205 remains exact and retains its source-sky-plane plus paired-skywall even
sequence. In that same elevated negative replay, the oversized source sky
plane 49 is now absent too; only its legitimate paired skywall crossing
remains before empty space.

### One-sided wall-back containment refinement

An elevated walkabout found a narrow view into the legitimate hut tunnel. The
ray crossed paired skywall 251, hit the back of one-sided BROWN1 wall 203, and
then crossed paired skywall 254 behind that wall. The tunnel is valid map
geometry; trimming it would therefore misclassify the symptom. The leak
occurred because the ordinary depth prepass used back-face culling: wall 203
wrote neither color nor depth from its non-owning side, so the later skywall
could toggle parity before farther tunnel geometry was depth-tested.

The opt-in experiment now treats source-proven one-sided walls as containment
boundaries in its depth prepass:

```text
front of one-sided wall
    color normally + terminate depth

back of one-sided wall
    no color + terminate depth

either face
    no sky-parity toggle
```

This is implemented with ordinary provider-neutral pipeline state: only the
Doom composition identifies one-sided source walls and selects a double-sided,
depth-only pipeline for those draws. The normal color pass remains back-face
culled. Two-sided walls, cutouts, sky crossings, collision and prepared
geometry are unchanged.

The retained exact replay is:

```text
skywall 251 at 25.476
one-sided wall 203 backside at 108.856
skywall 254 at 151.106
```

`LOOK` reports wall 203 as `facing:back,color:culled`, with
`parity-depth:terminating` and `parity-toggle:none`. This is CPU/source evidence
for the selected pipeline behavior, not a GPU depth-buffer readback. Human
walkabout remains the visual acceptance step. A native Vulkan two-frame proof
accepted the additional pipeline and completed with 3,787 draws, nine pipeline
switches, `466 µs` warm command construction and `113,726 µs` warm frame CPU
time. These are observations, not accepted budgets.

### Coplanar leaf-edge conformance

A later E1M1 walkabout found a narrow black seam in sector 60. The reported
ray landed on the floor-height boundary shared by subsectors 168 and 172;
nearby rays hit either side. The same exact miss reproduced with the
sector-boundary candidate, the source-boundary control, and the older finite
BSP global-full bake. It is therefore a plane triangulation defect, not a sky
crossing or trimming decision.

The plane bake now conforms T-junctions before triangulation. Existing vertices
from shorter neighboring edges are inserted into a collinear longer edge, so
both independently triangulated leaves use identical finite edge segmentation
after renderer-precision conversion. This preserves polygon area, source
identity, and the candidate's sector-support decisions. Canonical E1M1 records
202 insertions and 1,968 plane triangles under `--sector-boundary-trim`; a
synthetic three-region fixture proves exact area conservation. Triangulation
chooses a fan anchor away from subdivided incident edges and uses an interior
centroid fan only when every corner touches a subdivision; it does not depend
on a strict-ear removal order for collinearly subdivided polygons.

That conformance pass did not remove the reported E1M1 seam. A local ray sweep
showed why: both the sector candidate and its local-SEG control inherited the
same finite miss band before sector refinement ran. The sector candidate had
been applying its complete authored boundary graph to an already-gapped local
SEG result.

The corrected candidate separates authority by representation:

```text
BSP leaf paths
    own internal plane partitioning

directed LINEDEF/SIDEDEF sector graph
    owns authored exterior sector support

local SEG half-planes
    remain the unchanged comparison control
```

The sector candidate therefore starts from the finite BSP leaf and applies the
sector graph directly. Around the retained sector 60 seam ray, all tested
offsets from `-0.004` through `+0.008` now hit `FLOOR5_2`; the local control
retains its earlier miss band. Canonical E1M1 now records 169 sector
refinements, 339 fragments, 201 conformance insertions and 1,920 triangles.

### Exact-source realization batching

Cross-map walkabout exposed a separate realization cost: grouped parity draws
the complete world in both its depth and color stages, while the composition
had allocated one ordinary mesh and draw per prepared triangle. This was not a
reason to reject more geometry. The composition now concatenates triangle-list
streams only within an identical source/material owner (subsector plane or wall
tier). No triangle, UV, winding or provenance is removed, and dynamic wall
refresh uses the same grouping.

The retained native comparison is E1M3: 11,622 draws and about 190 ms warm
before grouping; 4,102 draws and about 94 ms warm afterward. E1M4 loads after
the independent Doom texture-name case-fold fix and measures 3,195 draws and
about 75 ms warm. These are diagnostic observations, not performance budgets.

E1M6 subsequently supplied a positive live walkabout: no sky defect was found
in the inspected areas. An apparently wrong electronics-faced moving door was
traced separately to authored asymmetric sidedefs (`EXITDOOR` outside and
`COMPUTE2` inside on the sector-30 door bounded by linedefs 614/615), not to
sky parity or runtime material corruption.

## Binding Invariants

1. Global-full geometry remains the world input.
2. The experiment suppresses color only through a per-pixel parity mask; it
   does not source-filter world declarations.
3. Raw triangle winding does not become Doom semantic authority.
4. Paired-sky sheets do not acquire invented World/Sky orientation.
5. Partial source participation remains separate.
6. Every adverse specimen remains replayable.
7. Doom vocabulary remains corpus/provider-private.
8. No BSP, BVH or provider-neutral contract follows from parity correlation.
9. SEG-loop refinement has authority only over the plane geometry baked from
   that validated loop; it does not alter collision, source membership, walls,
   or generic spatial-query semantics.
10. Source one-sidedness may select double-sided containment depth, but it does
    not make the wall back color-visible or turn that wall into a sky crossing.
11. Coplanar edge conformance may subdivide a retained polygon edge but may not
    change its support, area, source identity or admission decision.

## Stop Conditions

Return to AR-0030 before promoting the experiment if visual inspection exposes
required-world holes, unwanted leakage, unstable edges, or a need to invent a
semantic side/volume claim. Ordered source absence remains diagnostic and may
not become canonical free-look omission authority.
