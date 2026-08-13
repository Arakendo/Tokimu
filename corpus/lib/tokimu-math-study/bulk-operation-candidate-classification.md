# Option C Bulk Operation Candidate Classification

| Field | Value |
| --- | --- |
| Date | 2026-08-12 |
| Status | Slice 7 selection; no GPU/provider work admitted |
| Rule | A selected workload requires a CPU semantic reference before any acceleration experiment |

## Candidate Inventory

| Candidate | Current pressure | Classification | Disposition |
| --- | --- | --- |
| E1M1 prepared-draw AABB/frustum classification | AR-0025 repeatedly scans roughly two thousand prepared draws | Bounded spatial candidate selection; Doom source identities/order remain authoritative | Select as spatial control |
| CAD/robotics point-cloud frustum classification | Independent non-game workload with explicit point IDs and camera/query plane input | Bounded numerical filter; no topology, physics, or semantic commit is delegated | Select as independent control |
| Point transforms | Existing GLB loops are bounded and C1 affine work is already measured | Ordinary per-vertex mechanics; insufficient independent acceleration pressure | Retain CPU only |
| Broad-phase collision pairs | Doom collision is small and policy-heavy | Collision contacts and movement resolution are semantic/policy owned | Reject from GPU study |
| BSP/SEG/clip visibility | AR-0025 source-specific comparison | Viewer-relative Doom presentation semantics | Reject as a generic compute candidate |
| CAD robust predicates / topology | No bounded authoritative numerical kernel yet | Numerical precision and topology authority unresolved | Defer |
| Surface sampling | No independent caller or result-consumption requirement | Speculative | Defer |

## Selected A: Ordered Conservative Bounds/Frustum Classification

| Field | Selected meaning |
| --- | --- |
| Semantic owner | Caller owns camera/view meaning, world bounds, source identity, and final submission policy |
| Authoritative input | Ordered `[(identity, world AABB)]` plus explicit world-to-clip matrix/planes |
| Output | Ordered per-item `candidate`/`rejected-by-plane` observation; no sorting, batching, or draw scheduling |
| Batch scale | `1K`, `10K`, `100K`, `1M` synthetic controls; E1M1 current scale is a small negative/control case |
| Independence | Each AABB test is independent after shared plane preparation |
| Branching | Six conservative plane tests; first rejecting plane may be retained diagnostically |
| Residency | CPU source is authoritative; upload/residency is a future separated measurement, never assumed free |
| Precision | Current bounded `f32` conservative test; false negatives are correctness failures, false positives tolerated/measured |
| Ordering | Input order must survive filtering exactly, including blend-sensitive downstream ordering |
| Synchronization/result use | CPU submits or records results; any future GPU result must not double-commit or become renderer-owned visibility truth |

This is useful beyond games: CAD viewports, spatial dashboards, robotics camera
inspection, and scientific scene previews all need conservative candidate
filtering without inheriting Doom BSP or renderer scheduling semantics.

## Selected B: Ordered Point-Cloud Frustum Classification

| Field | Selected meaning |
| --- | --- |
| Semantic owner | Caller owns point coordinates, stable IDs, coordinate frame, and interpretation of surviving points |
| Authoritative input | Ordered `[(point_id, position)]` and explicit planes/matrix |
| Output | Ordered visible IDs/count plus optional fixed-size classification bitmap; never a reordered compacted geometry authority |
| Batch scale | `1K`, `10K`, `100K`, `1M`, and a memory-safe large case |
| Independence | One point/clip test per point after shared camera preparation |
| Branching | Finite-input rejection and six plane comparisons; no topology traversal |
| Residency | Distinguish one-shot upload-like input, reusable resident-like input, and CPU-resident reference explicitly |
| Precision | `f32` fixture coordinates; identity and classification equality required under selected finite domain |
| Ordering | Stable source order and IDs are part of the CPU reference, even if a later provider returns a bitmap/count first |
| Synchronization/result use | Point cloud remains caller data; readback/consumption mode must be reported separately before a provider experiment |

This independent case is useful for point-cloud CAD, lidar/robotics inspection,
scientific visualization, and monitoring dashboards. It does not imply that
Tokimu owns those domains or their persistent data models.

## Explicit Non-Selections

- No E1M1 BSP, SEG, portal, occlusion, alpha, sky, or screen-coverage rule is
  promoted into generic compute vocabulary.
- No GPU work is selected merely because operations are parallelizable.
- No result becomes authoritative simulation, CAD, collision, visibility, or
  renderer scheduling state without a later caller-owned validation/commit
  decision.
