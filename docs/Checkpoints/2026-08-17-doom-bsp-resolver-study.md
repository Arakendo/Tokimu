# Checkpoint: Doom BSP Resolver Study

| Field | Value |
| --- | --- |
| Date | 2026-08-17 |
| Campaign | DOOM WAD / E1M1 viewer-relative presentation |
| Active plan | `docs/Plans/DOOM/Studies/Doom BSP presentation-domain resolver study.md` |
| Parked predecessor | `docs/Plans/DOOM/Studies/Doom ordered source occurrence preparation.md` |
| Parent reviews | `AR-0025`, `AR-0030` |
| Current phase | Slice 3A full-submission classification visualization and source-spawn failure attribution |

## Resume Here

The literal ordered-result candidate is parked. Preserve its code and evidence:
it has exhaustive dispositions, balanced conservation, a shared Rust-owned
preparation entry, live native refresh and runtime-height snapshots. It is not
visually correct. At source spawn its complete 458-draw frame exposes severe
sky leakage through large peripheral and smaller interior holes.

The active R&D direction is a Doom-private BSP presentation-domain resolver:

```text
decoded BSP + runtime snapshot + exact Tokimu camera
    -> near-first BSP participation analysis
    -> source-attributed shadow manifest
    -> disagreements against global full, parked Slice 6B and reference planner
```

Do not start by changing renderer submission. Freeze the exact source-spawn
view and attribute four marked hole rays through source identity, BSP path,
subtree decision, parked disposition, lowering and final handoff.

The first shadow instrument is now implemented as `--bsp-diagnostic-full`.
It retains the original global-full geometry, substitutes checked-in PNG
families only at draw-command realization, and uses draw-local tint overrides
for accepted, positive solid-range rejection and unresolved/fail-open. Focus
modes (`all`, `accepted`, `rejected`, `unresolved`) dim nonmatching draws but
never remove them. They can be selected live with `Z`, `X`, `M` and `Q`,
respectively. Runtime movement/free look recomputes the shadow manifest
from the current camera and current-height snapshot. Headless `LOOK` replay
reports family, classification, reason and source identity.

The source-spawn two-frame GPU control reports `1,823` opaque plus `26`
cutout declarations, `1,849` candidates, `1,849` submitted and zero renderer
removals in both `all` and `rejected` focus modes. The separate original
source inventory remains `1,922` records because it also retains `73` omitted
source sky-plane evidence records; the presented panorama is one global
background declaration. Before the family-specific prepared-frustum guard,
the source-spawn shadow classifications were `477` accepted, `420` positive
solid-range rejected and `952` unresolved/fail-open.
This is conservation evidence for the diagnostic instrument, not evidence
that the classifications are correct.

Initial human review found the first asset palette semantically illegible:
ceiling and generic two-sided-boundary assets were both near-white, while
stair risers disappeared into that generic boundary family. The revised
instrument uses neutral patterned PNGs plus strong shader category colors and
adds `height-transition-boundary` for two-sided boundaries whose current floor
or ceiling heights differ. This is a source-height relation, not an invented
“stair” identity.

The first live purple-floor report is now retained as a concrete plane-domain
counterexample. The exact ray hits subsector `113` inside its nine-step
root-to-leaf inferred convex region (`x=896..928, y=-3104..-2992`) but `45.585`
units beyond its sole SEG's degenerate endpoint box (`x=896..928, y=-3072`).
Node `107` consequently and consistently marks that far-child SEG box outside
the FOV even though the reconstructed floor hit is nearly centered. `LOOK`
now prints both domains, and the classification reason is refined to
`source-plane-child-seg-bounds-outside-fov`. The contribution remains
unresolved/fail-open and fully submitted.

Center-only `LOOK` proved inadequate when a later capture's crosshair hit the
accepted wall at linedef `135` while the suspicious purple floor lay off-axis.
The console now accepts `LOOK PIXEL <x> <y>` in client-area coordinates and
`LOOK NDC <x> <y>` in `-1..1`. These inspect an off-axis ray without rotating
the camera, so the colored contribution and the BSP manifest retain the same
view identity.

`SCAN [columns rows]` now performs the corresponding bounded whole-viewport
inspection. The default `32x20` grid freezes the current view and current-
height manifest, groups nearest rejected/unresolved prepared-triangle samples
by exact source identity, and prints representative `LOOK PIXEL` commands.
Custom grids are capped at `4,096` samples and output at `24` suspicious
groups. This is prepared-geometry evidence, not pixel parity; AABB/frustum
selection has no classification or removal authority in the scan.

Headless automation is available through
`--bsp-diagnostic-scan-report=<source-x,source-y,source-z,center-dx,center-dy,center-dz,width,height[,columns,rows]>`
alongside `--bsp-diagnostic-full`. It calls the same scan implementation and
automatically runs the full LOOK waterfall for each suspicious representative
pixel. Live `CAMERA` prints a copy-ready flag. A `16x10` subsector `64` replay
passed with one suspicious group and an automatically expanded exact hit.
Headless runtime heights are explicitly the static-scene snapshot.

First live scan result: `640` samples produced `570` prepared-triangle hits,
`70` misses, `509` accepted hits, zero rejected hits and `61` unresolved hits.
Every unresolved sample grouped into floor subsector `64`, sector `29`, with
reason `source-plane-child-seg-bounds-outside-fov` across pixel bounds
`(820,460)..(1260,780)`. This identifies the screenshot's lower-right purple
patch as a second plane/SEG-bounds disagreement, not the adjacent window or
height-transition boundary.

The exact representative hit is `(-240.941,-3291.664,104)`, `27.680` units
outside subsector `64`'s sole SEG box but inside its thirteen-step inferred
convex region. Its off-axis sample ray has heading `-168.884°` and reaches the
leaf when traced as its own centered view. The unresolved classification still
correctly describes the unchanged camera-wide manifest. Sampled LOOK output
now distinguishes the frozen BSP view heading from the sample-ray heading.

A second scan after the viewer/view changed reported `543` hits, `97` misses,
`469` accepted, zero rejected and `74` unresolved. All suspicious samples
still belonged to floor subsector `64` with the same reason, while its pixel
bounds shifted to `(660,540)..(1260,780)`. This corroborates live view-relative
refresh rather than a stale manifest.

A third view added ceiling subsector `67` and two extreme-right walls. The
ceiling repeats the plane-region/SEG-box mismatch. Wall linedefs `101` and
`107` hit inside their SEG and inferred-region bounds, and their SEGs are
admitted when each off-axis ray is traced as a centered view. Relative to the
frozen camera heading, however, their sample headings are `-48.688°` and
`-45.416°`, outside the shadow resolver's fixed classic `±45°` source domain.
This is direct free-look/camera-domain evidence rather than broken wall
provenance. The refined wall reason is
`source-wall-seg-bounds-outside-fov`; sampled LOOK also prints the normalized
sample-minus-view heading.

This is evidence against using Doom child SEG bounds as whole-plane rejection
authority. It is not evidence that the BSP-path flat reconstruction is
oversized. Do not “fix” the purple patch by accepting or rejecting the whole
subsector until the plane participation rule has been separated from the wall
SEG traversal rule.

That separation now exists in shadow classification. Wall contributions keep
their exact matching-SEG solid-range rule. A solid-range-pruned floor or
ceiling is diagnostically rejected only when its actual prepared AABB is also
definitely outside the actual Tokimu frustum. An intersecting prepared plane
is retained purple with reason
`prepared-geometry-frustum-vetoed-plane-rejection`; unavailable camera or
bounds evidence fails open as `projection-or-traversal-ambiguous`. This is a
conservative veto against unsafe omission, not AABB/frustum visibility or
occlusion authority, and it changes no submitted membership.

The rule has a focused three-outcome test. The rebuilt `16x10` headless replay
still reports floor subsector `64` as its sole suspicious group: `145` hits,
`143` accepted, zero rejected and two unresolved. Its automatic LOOK waterfall
again places the representative hit inside the inferred plane region and
`55.046` units beyond the sole SEG box.

A rebuilt two-frame source-spawn GPU control (now superseded by the later
positive-authority correction) conserves all `1,849`
declarations with zero renderer removals. It reports `477` accepted, `262`
positive solid-range rejected and `1,110` unresolved/fail-open; exactly `158`
planes carry `prepared-geometry-frustum-vetoed-plane-rejection`. This accounts
for the entire disposition shift from the pre-guard baseline while leaving
membership unchanged.

The first live post-guard walk/free-look scan is also retained. Its `32x20`
grid produced `631` hits, `9` misses, `511` accepted, zero rejected and `120`
unresolved samples. The four suspicious groups are exactly the floor and
ceiling contributions for subsectors `97` and `99`, all with the prepared-
frustum veto reason. The center LOOK hit on subsector `97` is inside its
inferred region but `46.273` units outside its sole SEG box. Node `96` marks
the leaf solid-range covered while the hit lies approximately `0.052°` from
view center. This confirms that solid-range coverage of the wall-derived
proxy is not sufficient whole-plane rejection evidence.

After closing the live viewer, the exact headless replay reproduced all scan
totals and all four groups. Its automatic LOOK representatives lie `10.699`,
`53.255`, `6.538` and `123.321` units outside the relevant SEG boxes while
remaining inside the inferred regions. The live finding is now replayable
without manual pixel selection.

The canonical baked-data audit is now implemented through
`--doom-bsp-bounds-audit-report`. All `236` raw NODES records exactly match
their decoded partition, box and child fields. All `472` decoded child boxes
contain the complete descendant SEG-endpoint envelope, with zero source-proxy
underbounds. In contrast, only `149/472` boxes contain the inferred convex
plane-region envelope; `323/472` are smaller than that reconstructed support.
Nodes `95` and `96` reproduce the retained subsector `97`/`99` evidence.

Conclusion: the loader and canonical bake are internally consistent for the
source structures being traversed. The defect was granting those bounds
negative authority over larger reconstructed plane meshes. Keep the canonical
WAD unchanged; any later rebake is A/B diagnostic perturbation only.

The positive side of plane authority has also been corrected. Reaching a
source subsector selects possible Classic Doom plane destinations but does not
prove occurrence of the whole reconstructed Tokimu plane. Reached planes now
remain `unresolved-fail-open` with reason
`source-plane-subsector-reached-occurrence-unproven`. A distinct
`rejected-outside-frustum` disposition now names actual prepared-geometry
rejection rather than attributing it to BSP solid coverage.

The corrected source-spawn manifest conserves all `1,849` declarations and
reports `584` out-of-frustum planes, `269` unresolved planes, `70` accepted
wall-family draws, `208` solid-range-rejected wall-family draws and `718`
unresolved wall/cutout draws. The retained `32x20` walk/free-look replay has
`631` hits: `203` accepted wall-family hits, zero rejected hits and `428`
unresolved plane hits. Thus no sampled visible plane is rejected or falsely
promoted to accepted.

The next plane-specific diagnostic step is also implemented. Interactive
`LOOK`, headless `--look-ray-report` and the automatic LOOK expansion from
`--bsp-diagnostic-scan-report` now compare a flat hit's exact source plane key
and sector against Classic's frozen-view `320x200` plane-span instances. The
report retains instance, populated-column, populated-cell and source-SEG
counts, while explicitly denying whole-mesh or Tokimu-pixel authority.

For the retained subsector `97` ray, the exact occurrence remains rejected by
the source occurrence trace, but the matching sector-38 `FLOOR4_8` key is
present elsewhere in one Classic instance with `249` populated columns,
`17,381` populated cells and eight source SEGs. This is not contradictory: it
proves that plane identity participation and exact prepared occurrence are
different questions. The whole inferred plane remains unresolved/fail-open.

The runtime-snapshot control was rerun after this refinement. Door sector `4`
at ceiling heights `0/68` changes the occurrence fingerprint and twelve target
declarations. Platform sector `70` at floor heights `104/-48` retains its
horizontal occurrence fingerprint but changes nineteen target declarations.
Both use immutable current-height snapshots through the same preparation seam,
contain no activation/timing policy and report zero unresolved lowering.

## Boundaries

- No Doom BSP or visibility ownership in `tokimu-render` or `tokimu-platform`.
- No stable/public API is authorized.
- Use the original complete contribution inventory as the shadow input.
- Runtime supplies current heights; the resolver owns no activation or timing.
- Unknown or uncorrelated source facts fail open.
- Do not use sky depth, generic filtering or another global-shell patch to hide
  the source-spawn failure.
- If safe BSP participation cannot be separated from exact coupled
  wall/vertical-plane coverage, retain that as a successful falsification of
  the prefilter hypothesis and return to AR-0030.

## Immediate Work

1. Retain the marked screenshot at a repository evidence path when available.
2. Capture exact source/Tokimu camera and projection facts for that frame.
3. Capture `all`, `rejected` and `unresolved` full-submission diagnostic frames
   at the exact source-spawn pose and verify membership is visually conserved.
4. Select four deterministic rays: two large leaks and two interior leaks.
5. Replay each suspicious surface through the diagnostic `LOOK` reason, then
   add its root-to-leaf BSP waterfall.
6. Classify each failure as view-domain, traversal/disposition, lowering,
   culling or renderer realization.
7. Expand the current conservative shadow manifest only from retained source
   evidence; do not grant it presentation authority.
8. Audit plane participation separately from wall/SEG child-box traversal,
   beginning with the retained subsector `113` counterexample. The first
   conservative prepared-AABB/frustum guard is implemented in shadow mode;
   source-key/sector plane-span reporting is also implemented. Continue the
   pose/runtime matrix and seek exact occurrence-cell evidence only where the
   camera domains can be proven equivalent before granting presentation
   authority.
