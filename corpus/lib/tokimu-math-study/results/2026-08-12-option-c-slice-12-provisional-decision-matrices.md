# Option C Slice 12: Maintainer Decision Matrices

| Field | Value |
| --- | --- |
| Date | 2026-08-12 |
| Status | Maintainer disposition recorded; Slice 11 browser chart execution complete |
| Rule | No stable migration or compute capability follows from this artifact |

## A Versus C0/C1

| Dimension | A — pinned direct `glam` | C0/C1 — owned bounded candidate | Current evidence-based conclusion |
| --- | --- | --- | --- |
| Trust/provenance | Audited foreign Ring 0 source pin; broad future-delta review | Tokimu-authored candidate; no provider closure | C reduces foreign execution surface but inherits full correctness duty |
| Public vocabulary | Foreign types remain public | Candidate names/mechanics are corpus-local and owned | Public migration has not been earned |
| Correctness | Mature provider control | 63 local tests, differential/degenerate checks, A/C chart agreement | Bounded evidence only; neither self-authorship nor test count proves general correctness |
| Native/WASM | Native and retained isolated WASM evidence | Native and actual DOM/WASM chart agreement; target scope is still bounded | Browser chart execution closes this control, not all target questions |
| Performance | Baseline; avoids migration work | C1 safely recovers measured affine GLB pressure; C0 general inverse remains materially slow | C cannot yet replace A for all caller shapes |
| Maintenance | Foreign source audit/toolchain-upgrade pressure | Local numerical, optimization, and long-term target maintenance | Economics are an active comparison, not a settled win |
| Spatial/chart pressure | Supports ordinary mechanics below semantic adapters | Same fixed chart trace with no new operation request | Small evidence for C only; rich chart semantics stay above math |
| Migration/rollback | Present stable production state | Would require a separately approved compatibility, deprecation, and rollback plan | No migration in this study |

### Maintainer Alternative Disposition

**Retain A as the stable production vocabulary; continue C0/C1 incubation.**

C remains plausible because its demonstrated source surface is bounded, it has
no foreign execution closure, and the first chart control did not expand its
ordinary math manifest. C is not selected because full caller migration,
public-boundary compatibility, broader target observation, C1's non-affine
inverse performance, and long-term numerical maintenance remain unresolved.

D remains paused. B remains a comparison/transition mechanism, not a selected
destination.

## CPU Versus WGPU Bulk Compute

| Dimension | CPU reference | WGPU corpus mechanism | Current evidence-based conclusion |
| --- | --- | --- | --- |
| Meaning | Caller-owned ordered candidate/rejection facts | Exact local comparison against CPU output | CPU remains semantic reference |
| E1M1 AABB scale | 19.0 µs reused result at 1,861 | 872.8 µs warm / 412.1 ms cold | CPU preferred |
| 100K synthetic AABB | 1.998 ms | 1.529 ms warm / 415.2 ms cold | Small warm advantage only |
| 1M synthetic AABB | 20.524 ms | 4.495 ms warm / 429.2 ms cold | WGPU is useful scale evidence, not capability admission |
| Browser point control | Native reference only | Actual DOM/browser WGPU parity at 100K; retained cold/warm timing | Target execution evidence; no browser crossover claim |
| Failure | Deterministic input rejection and caller execution | Scoped shader rejection, idle disposal, and caller CPU bypass | Actual loss/in-flight cancellation remain open |
| Ownership | Caller retains IDs, order, query domain, and final interpretation | Local provider mechanism only | No world/CAD/renderer authority moves to GPU |
| Target coverage | Available everywhere CPU runs | AMD/Vulkan and browser WebGPU evidence; NVIDIA unavailable | Coverage gap prevents generalization |

### Maintainer Compute Disposition

**Retain CPU and WGPU corpus evidence; admit no shared operation or provider
capability.**

No stable batch operation, provider selection rule, shader/buffer contract, or
scheduler is needed by a named caller. Raw Vulkan, CUDA, HIP, and comparable
specialized providers remain **not earned**.

## Required Before Any Different Decision

1. Retain the Slice 11 DOM chart observation and continue AR-0026 only through
   its separately authorized synthetic-junction corpus.
2. Establish a real public-boundary/migration case before changing the stable
   math vocabulary.
3. Revisit C1 only if a named non-affine inverse workload remains materially
   decision-relevant.
4. Require an independent caller and a named lifecycle/authority need before
   proposing a shared CPU/WGPU capability or another provider.

No ADR-0010 revision, math migration plan, or compute Architectural Review is
recommended by this provisional result.
