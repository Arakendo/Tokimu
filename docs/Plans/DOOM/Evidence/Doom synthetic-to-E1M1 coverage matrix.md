# Doom Synthetic-to-E1M1 Coverage Matrix

Status: active Slice-7 gate; source-protocol evidence, not a claim of original
Doom renderer parity.

This matrix prevents a plausible synthetic result from becoming an E1M1
presentation mode by implication. A canonical E1M1 symptom may be revisited
only when its listed synthetic guard is green; passing a guard permits
investigation, not promotion.

| Canonical E1M1 observation | Synthetic guard | Current status | What it does and does not establish |
| --- | --- | --- | --- |
| Spawn-room floor disappears at a pillar or subsector boundary | Continuous-plane and pillar controls; shared-key/disjoint-plane-instance control | Partial | Source plane identity and BSP leaf relationships are guarded. Native and Browser WebGPU now present both equal-key source sectors as separate plane instances with zero warm/jitter mesh churn. Remaining plane-instance controls still prevent normal candidate selection. |
| A nearby wall vanishes after an observer move/turn | Viewer-plane, projection-epsilon, camera-jitter, and cross-representation wall controls | Partial | The existing one-dimensional and per-column candidate branches are falsified for presentation. Native and Browser WebGPU projection evidence visibly retain thin/close valid walls while the behind-viewer case fails open, with zero warm/jitter mesh churn. No selected E1M1 wall set is eligible for normal use. |
| First-door opening shows sky until crossing the doorway | Dynamic door aperture and stationary dynamic-transition controls | Partial | Runtime snapshots prove preparation consumes explicit current heights. Native and Browser WebGPU show the actual closed two-sided source band while the identical decoded map under the declared open snapshot produces zero source bands, with zero warm/jitter mesh churn. E1M1 composition evidence remains open. |
| Platform/floor motion leaves stale geometry or clearance | Moving-platform boundary and stationary dynamic-transition controls | Partial | Native and Browser WebGPU rendered controls visibly distinguish the same immutable source under declared floors `0` and `48`, with zero warm/jitter mesh churn. E1M1 platform rendering/collision composition remains open. |
| Hut/exterior sky aperture exposes distant sector geometry | Paired-sky, unequal-paired-sky, one-sky negative, partial horizontal occlusion, and vertical partial occlusion controls | Partial | Native and Browser WebGPU observations prove the small paired-boundary and one-sky roles. They do not reconstruct viewer-relative Doom plane/clip coverage for E1M1. |
| One sky ceiling accidentally hides an ordinary wall | One-sky negative | Green | Native and Browser WebGPU retain the ordinary upper wall and no paired-sky boundary authority. This guard is source-local and does not decide unrelated E1M1 aperture coverage. |
| Paired sky fails to exclude a far wall at its exact boundary | Paired-sky | Green | Native and Browser WebGPU retain two source boundary triangles, depth exclusion of the far control, and zero static mesh churn under the bounded camera update. |
| Upper/lower wall tiers erase an opening | Vertical aperture | Green | Native and Browser WebGPU retain two upper and two lower source triangles around a visible far control. It is not a complete visplane or screen-clip reconstruction. |
| Masked middle becomes a solid visibility blocker | Masked-middle negative control | Green | Headless evidence denies solid range authority. Native and Browser WebGPU visibly retained green categorical-cutout texels over an orange far wall visible through transparent texels; browser first/warm/jitter frames retained three draws and zero warm/jitter mesh churn. No E1M1 visibility branch may infer solid authority from alpha. |
| Orientation/source lift swaps source-right and presented right | Coordinate-frame directional conformance campaign | Green, separate campaign | PreserveNorth is accepted as the current E1M1 embedding. This does not validate visibility or sky coverage. |

## Gate Rule

```text
synthetic guard red or partial
    -> E1M1 remains an observation-only falsifier

all applicable synthetic guards green
    -> E1M1 may run the named experimental candidate
    -> retain source identity, pose, frame metadata, and counterexample if any
```

No passing row grants a generic renderer visibility contract. All currently
named synthetic browser target observations are retained; the remaining rows
are E1M1 composition/falsification work, not unobserved browser mechanics.
