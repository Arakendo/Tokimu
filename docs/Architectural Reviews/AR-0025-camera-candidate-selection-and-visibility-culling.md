# AR-0025: Camera Candidate Selection and Visibility Culling

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-08-10 |
| Last reviewed | 2026-08-10 |
| Scope | Corpus presentation, renderer-facing scene preparation, and performance diagnostics |
| Trigger | The interactive E1M1 source-spawn observer now resubmits all 1,861 static draws for every camera update. Startup mesh upload was repaired, but view-dependent candidate selection remains unstudied. |
| Related ADRs | ADR-0003, ADR-0007, ADR-0008, ADR-0009, ADR-0012, ADR-0013 |
| Related reviews | AR-0021, AR-0023, AR-0024 |
| Related evidence | `hello-doom-e1m1` source-spawn observer; E1M1 static presentation evidence; native Vulkan/AMD manual observation |
| Admission exception | None |

## Architectural Question

What, if any, provider-neutral camera-candidate-selection or visibility-culling
capability should Tokimu admit after corpus evidence establishes a reusable
need, and which visibility facts must remain application/source specific?

## Context

The E1M1 static scene prepares 1,835 opaque draws plus 26 admitted categorical
cutout draws. The source-spawn observer initially re-uploaded every immutable
mesh on every camera update; that local performance defect is repaired by
uploading static geometry once at startup. Camera motion still submits every
prepared draw because the current renderer consumes an explicit caller-provided
command list and owns no scene graph, map topology, or visibility authority.

The visible black regions in current E1M1 observation remain separate bounded
presentation limits: sky drawing and original Doom plane-span/visibility
reconstruction are not claimed by the static scene. This review must not use
"culling" to hide unsupported source surfaces, missing source semantics, or
renderer defects.

## Trigger and Retained Evidence

- The available native Vulkan/AMD source-spawn observation visibly presents
  1,861 draws under mouse look.
- Static mesh buffers now upload once during startup. Subsequent observer
  frames change the camera and submit draws without replacing static GPU mesh
  resources.
- The scene still supplies all prepared candidates on every frame. No frustum,
  BSP, portal, `REJECT`, `BLOCKMAP`, depth-pyramid, or occlusion-query culling
  is currently claimed.
- E1M1 retains a bounded Doom BSP and source-sector ownership path. That is
  source interpretation evidence, not generic renderer visibility semantics.
- AR-0024 established that accepted commands and successful presentation do not
  by themselves prove expected pixels; any culling experiment must retain
  candidate, rejected, submitted, and visible-observation facts separately.

## Ownership Analysis

- A corpus/application owns scene membership, source-format topology, Doom BSP
  interpretation, source-specific `REJECT` data, and any rule that decides
  which source objects are candidates for a camera.
- A provider-neutral renderer owns realization of explicit mesh/material/camera
  declarations. It must not acquire WAD terms, mutable map truth, or hidden
  application scene ownership merely to skip draws.
- A possible Tokimu capability, if earned, would own only a generic declared
  candidate-selection contract and bounded observability. It must not infer
  visibility from asset names, invent an application scene graph, or promise
  occlusion correctness beyond its stated evidence.
- A WGPU backend may realize provider-native frustum or GPU culling only behind
  an admitted provider-neutral contract. Backend query handles and WGPU types
  must not become caller vocabulary.

## Dependency Direction

```text
Current:
E1M1 source/map preparation -> corpus draw list -> tokimu-render -> WGPU

Possible bounded direction:
application/corpus candidate declaration + camera
    -> admitted Tokimu candidate-selection capability (only if earned)
    -> renderer/provider realization

Not proposed:
WAD BSP/REJECT -> tokimu-render
renderer hidden scene graph -> simulation/source truth
```

## Alternatives Considered

### A. Retain Explicit Full Submission

- Benefits: current ownership is simple, transparent, and proven; no new
  renderer contract or hidden culling errors.
- Costs: cost grows with prepared scene size and camera movement submits work
  that is plainly outside the camera frustum.
- Failure mode: applications duplicate ad-hoc selection logic without retained
  diagnostics or target comparison.

### B. Corpus-Local CPU Frustum Candidate Selection

- Benefits: tests whether ordinary mesh bounds plus a camera are enough to
  reduce E1M1 submission without changing `tokimu-render` ownership.
- Costs: requires a conservative bounds policy, instrumentation, and proof
  that source-visible surfaces are not incorrectly removed.
- Failure mode: a Doom-specific or one-off helper is prematurely promoted as a
  universal renderer API.

### C. Admit Generic Renderer/Scene Culling

- Benefits: could centralize a repeatedly demonstrated provider-neutral
  candidate/bounds contract and diagnostics across independent consumers.
- Costs: raises public vocabulary, cache/state, performance, and failure
  containment obligations; may imply scene scheduling/ownership that Tokimu has
  not admitted.
- Failure mode: renderer-owned opaque sorting or culling obscures application
  ordering, cutout, and future Blend responsibilities.

### D. Reuse Doom BSP, REJECT, BLOCKMAP, or Portal Semantics Directly

- Benefits: may be effective for Doom maps.
- Costs: source-format rules are not generic camera semantics; each has
  distinct reliability and completeness assumptions.
- Failure mode: Doom terms leak across the corpus/renderer boundary and become
  accidental engine ontology.

### E. GPU Occlusion Queries, Hierarchical Depth, or Hardware-Specific Culling

- Benefits: potentially stronger reduction for large scenes.
- Costs: asynchronous visibility, temporal behavior, platform variance,
  diagnostic complexity, and likely new resource/scheduling contracts.
- Failure mode: provider mechanics become a premature public capability or
  valid-but-late results are mistaken for authoritative scene truth.

## Initial Findings

1. Static mesh upload replacement was an ordinary corpus performance defect,
   not evidence for a culling architecture.
2. E1M1 has one real camera-pressure source, but no independent caller yet.
3. Conservative CPU frustum selection is the smallest next experiment because
   it can remain corpus-local and report exact candidate/rejected/submitted
   counts without changing renderer meaning.
4. Doom BSP, `REJECT`, and `BLOCKMAP` are inputs to a Doom-specific experiment,
   not candidates for direct renderer admission.
5. Blend remains incubating under AR-0023; no candidate-selection work may
   reorder or otherwise conceal its caller-owned ordering requirements.

## Disposition

**Under Review.** Keep full explicit submission as the current renderer
contract. Run a bounded corpus-local conservative frustum-candidate experiment
first, with retained before/after counts and false-negative investigation. Do
not add a public culling API, scene graph, Doom visibility coupling, or
provider-native occlusion contract from this record's initial evidence.

## Required Follow-Up

- [ ] Add first/warm observer-frame measurements for submitted draws, mesh
      uploads/replacements, material resolutions, pipeline switches, and frame
      time without culling.
- [ ] Define a corpus-local conservative world-space bounds representation for
      E1M1 prepared meshes; prove source/candidate identity survives it.
- [ ] Implement and test CPU frustum candidate selection without changing
      `tokimu-render` public vocabulary or reordering caller draw order.
- [ ] Retain candidate, rejected, submitted, and manual visual-observation
      records for native and browser/WASM where feasible.
- [ ] Compare against a second independent camera/scene consumer before
      proposing any generic contract.
- [ ] Evaluate Doom BSP/REJECT only as source-specific alternatives after the
      ordinary frustum baseline is known.
- [ ] Apply ADR-0008 and ADR-0009 if a shared capability or hot-path contract
      is proposed.

## Reopening Triggers

- a second independent consumer needs the same camera/bounds selection;
- a conservative corpus-local experiment repeatedly preserves visible content
  while materially reducing submission cost;
- a source-specific Doom visibility method conflicts with generic bounds
  selection or reveals an ownership ambiguity;
- an attempted solution requires renderer-owned ordering, scene truth,
  provider-native handles, or a stable public API; or
- culling hides visible geometry, produces nondeterministic results, or masks
  an unsupported presentation behavior.

## Review History

### Cycle 1 -- 2026-08-10

- Status entering review: Proposed.
- New evidence: source-spawn E1M1 camera interaction made per-frame static
  mesh replacement visible as lag; the local repair uploads static meshes once.
  The camera still submits every prepared draw, while current black regions
  remain explicit sky/plane-presentation limits rather than inferred culling.
- Findings: CPU frustum candidate selection is the smallest non-binding
  experiment; generic culling, Doom visibility data, and GPU occlusion each
  require distinct evidence and must not be conflated.
- Disposition: Under Review; start with corpus-local evidence only.
- Resulting ADR or documentation change: none.

## References

- `docs/ADR/ADR-0007-kernel-performance-diagnostics.md`
- `docs/ADR/ADR-0008-native-kernel-ring-performance-and-code-quality.md`
- `docs/ADR/ADR-0009-ring-based-verification-failure-containment-and-recovery.md`
- `docs/Architectural Reviews/AR-0023-textured-surface-alpha-and-depth-policy.md`
- `docs/Architectural Reviews/AR-0024-renderer-failure-observation-and-diagnostic-boundary.md`
- `docs/Plans/DOOM/DOOM WAD Checklist.md`
- `docs/Plans/DOOM/E1M1 static presentation evidence.md`
- `corpus/hello-doom-e1m1/src/bin/static_scene.rs`
