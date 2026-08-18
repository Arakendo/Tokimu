# 2026-08-18 Doom Source-Covered Walkabout Falsifiers

## Disposition

The `source-covered-global-shell` walkabout materially improves the E1M1 hut
area, but it is falsified as a sufficient presentation policy. Maintainer
walkabout produced both false retention and false omission within the plane
family. The strategy remains useful as an explicit diagnostic comparison; it
must not become the default or stable contract.

The result is stronger than a remaining bbox defect:

```text
subsector reached + exact source plane key absent
        → whole reconstructed plane was retained incorrectly

subsector skipped by source proxy + exact source plane key present
        → nearby reconstructed plane was omitted incorrectly
```

Neither subsector visitation nor source-plane-key existence has Boolean
authority over a complete reconstructed plane mesh.

## Exact Captures

All source coordinates below are replayable with `--look-ray-report`.

| Capture | Complete-shell target | Source traversal | Plane occurrence | Current candidate consequence |
| --- | --- | --- | --- | --- |
| `-227.041458130,-3152.000976562,140,0.511952519,0.852522492,0.105403915` | ceiling, subsector 55, sector 24, `FLOOR7_2`, distance 796.934 | target reached | exact sector/key absent | false retention behind earlier sky boundary |
| `-205.733337402,-3311.999023438,140,0.475691646,-0.874108732,0.098241337` | ceiling, subsector 54, sector 24, `FLOOR7_2`, distance 855.037 | target reached | exact sector/key absent | false retention behind earlier sky boundary |
| `2248.567138672,-3360.645263672,-20,-0.929613948,-0.359226376,0.082306169` | ceiling, subsector 117, sector 41, `CEIL3_5`, distance 1700.966 | target reached | exact sector/key absent | false retention behind earlier sky boundary |
| `2042.021240234,-2975.617919922,-20,0.613750577,-0.787513614,0.055970095` | wall 241 / `BROWN1`, distance 1178.877 | subsector 186 reached; SEG 544 admitted | not applicable to wall | retained wall lies behind an earlier sky boundary; final wall-occurrence authority still needs correlation |
| `2447.991455078,-3216.869628906,-20,-0.941134751,-0.201053977,-0.271740079` | floor, subsector 130, sector 5, `FLOOR7_1`, distance 132.480 in complete shell | target subsector skipped `outside-fov`; hit lies inside inferred region but outside child/SEG proxy | sector/key present with populated spans | false omission: filtered walkabout reports no prepared-triangle hit |

The fifth capture independently replays against the complete shell. Its hit is
only 132.480 source units away. Treating the missing filtered hit as empty
space would therefore hide a local floor, not merely remove distant scenery.

An additional hut-area screenshot reports the subsector 55 ceiling key as
present from a nearby pose while still showing suspicious whole-plane
geometry. That prevents replacing the current rule with the equally coarse
rule “retain every complete plane whenever its source key exists.” A key is an
occurrence identity; its populated view-local spans are not proof that every
point of every correlated reconstructed polygon participates.

## Architectural Finding

A complete reconstructed plane mesh is larger than each of the following
source representations:

- one BSP child or SEG endpoint proxy;
- one reached subsector event;
- one source plane key; and
- one bounded set of Classic plane spans.

Consequently a correct follow-up cannot be another Boolean whole-plane filter.
It must investigate a Doom-private realization that relates source-keyed,
view-local plane occurrence support to the actual reconstructed plane support
without granting source proxy bounds authority over that geometry.

The wall 241 capture is deliberately unresolved. Horizontal BSP admission is
not final wall-pixel or prepared-fragment proof. Before changing wall policy,
the exact retained wall occurrence and its vertical/source interval must be
correlated with the captured hit.

## Preserved Boundaries

- No renderer or stable API change follows from these captures.
- Sky boundaries remain correlated evidence, not generic occluder geometry.
- The complete-shell BVH remains exact-geometry evidence, not source
  participation authority.
- Source-key absence is strong diagnostic evidence for the three ceiling
  false positives, but the existing report correctly labels it weaker than
  arbitrary-pitch pixel proof.
- The current experimental strategy remains available for A/B comparison and
  is not silently tightened around the falsifiers.

## Next Decision

Return the result to AR-0030 before implementing a successor presentation
strategy. The bounded next study would be source-plane occurrence support over
reconstructed geometry, with these five captures plus the prior seven controls
as mandatory falsifiers. Wall 241 should first receive final ordered wall
occurrence diagnostics so plane and wall defects are not conflated.

