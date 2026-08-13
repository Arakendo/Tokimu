# AR-0019 Option C Second-Stage Result Ledger

| Field | Value |
| --- | --- |
| Status | Active ledger; inherited evidence frozen before post-DOOM work |
| Opened | 2026-08-12 |
| Control | `results/2026-08-12-option-c-second-stage-control.md` |
| Rule | Every result is labeled inherited or new and names its workload/date |

This ledger prevents earlier evidence from acquiring a new workload or claim
merely because the Option C study resumed. `Inherited` means the artifact was
produced by the first evidence phase. `New` means it was produced under the
second-stage plan. Neither label implies that a result is decision-complete.

## Inherited Evidence

| Date | Workload or question | Artifact | Retained claim |
| --- | --- | --- | --- |
| 2026-08-07 | Initial A/B transform workload | `results/2026-08-07-initial-a-b-transform-run.md` | Initial provider-direct/provider-backed transform comparison only |
| 2026-08-07 | Candidate source surface | `results/2026-08-07-source-surface-observation.md` | Dated source-size and ownership observation |
| 2026-08-08 | Repeated A/B/C transform workload | `results/2026-08-08-repeated-transform-observation.md` | Caller-shaped transform checksum/timing on the recorded host |
| 2026-08-08 | Native A/B/C layout | `results/2026-08-08-native-layout-observation.md` | Size/alignment observation, not an ABI contract |
| 2026-08-08 | Shared WASM conformance harness | `results/2026-08-08-shared-wasm-conformance-harness-observation.md` | Shared assertions compile/run in the retained harness |
| 2026-08-08 | Isolated A/B/C WASM engine execution | `results/2026-08-08-isolated-wasm-engine-execution.md` | Actual plain-WASM engine execution for the bounded probe |
| 2026-08-08 | Isolated WASM output comparison | `results/2026-08-08-isolated-wasm-output-observation.md` | Bounded output agreement under the recorded probe |
| 2026-08-08 | Isolated WASM layout | `results/2026-08-08-isolated-wasm-layout-observation.md` | Target layout observation, not stable representation |
| 2026-08-08 | Isolated WASM stereo-camera math | `results/2026-08-08-isolated-wasm-stereo-camera-observation.md` | Bounded repeated camera-math evidence below the renderer |
| 2026-08-08 | Isolated C WASM build | `results/2026-08-08-owned-subset-isolated-wasm-build.md` | Dependency-free target compilation only |
| 2026-08-08 | Native stereo-camera boundary | `results/2026-08-08-stereo-camera-boundary-observation.md` | Native caller/boundary observation on the recorded workload |
| 2026-08-08 | Camera public vocabulary | `results/2026-08-08-camera-public-vocabulary-surface-scan.md` | Current stable `Camera` exposes provider vocabulary |
| 2026-08-08 | Renderer camera representation boundary | `results/2026-08-08-renderer-camera-representation-boundary-scan.md` | Current scalar/column handoff observations |
| 2026-08-08 | Public-boundary consequences | `results/2026-08-08-public-boundary-consequences-scan.md` | Migration/ownership consequences of foreign public types |
| 2026-08-08 | Presentation caller pressure | `results/2026-08-08-presentation-caller-pressure-scan.md` | No new operation pressure beyond the then-current manifest |
| 2026-08-08 | Candidate-isolated link output | `results/2026-08-08-candidate-isolated-link-output-observation.md` | Bounded release-output observation only |
| 2026-08-08 | Isolated B/C fresh build closure | `results/2026-08-08-isolated-build-closure-observation.md` | One-host minimal closure/build observation; not workspace timing |

The first-phase operation inventory, migration accounting, maintenance
forecast, decision matrix, and close report remain interpretive supporting
artifacts. They are not silently re-dated as second-stage results.

## New Second-Stage Evidence

| Date | Workload or question | Artifact | Current disposition |
| --- | --- | --- | --- |
| 2026-08-12 | Exact A/C control, toolchain, warning, hashes, offline reproduction | `results/2026-08-12-option-c-second-stage-control.md` | Slice 0 complete; A unchanged, C0 reused |
| 2026-08-12 | Post-DOOM stable/corpus operation, representation, and bulk-boundary scan | `operation-inventory-post-doom.md` | Slice 1 complete; three values remain earned, several mechanics added, `Vec2`/`Quat` still unpressured |
| 2026-08-12 | Candidate numerical, degenerate-input, comparison, and failure contract | `numerical-contract-c0.md` | Slice 2 contract selected for corpus testing; stable API/error shape remains unselected |
| 2026-08-12 | C0 checked scalar mechanics and first hardening evidence | `results/2026-08-12-option-c-slice-3-first-hardening.md` | Slice 3 partial; checked contract works, isolated C gate is clean, and ownership growth is now measured |
| 2026-08-12 | C0 generated, independent-reference, conditioning, and allocation evidence | `results/2026-08-12-option-c-slice-3-boundary-evidence.md` | Slice 3 complete as bounded corpus evidence; no production admission |
| 2026-08-12 | C0 native/Node-WASM checked semantic parity and scalar representation | `results/2026-08-12-option-c-slice-4-wasm-parity.md` | Slice 4 bounded two-target evidence; no browser, ABI, or stable layout claim |
| 2026-08-12 | Native A/B/C transform, stereo-camera/boundary, allocation, and isolated build/output observations | `results/2026-08-12-option-c-slice-5-first-performance-observation.md` | Slice 5 partial; no C1 deficit established, while Doom/collision/import replays remain open |
| 2026-08-12 | CAD/GLB/Doom-observer caller replay, affine inverse isolate, and safe C1 affine fast path | `results/2026-08-12-option-c-slice-5-inverse-isolation-and-c1.md` | Slice 5 partial; C0 general inverse is a material regression, C1 recovers affine GLB work, CAD/picking and complete Doom replay remain open |
| 2026-08-12 | A/C Ring 0 warning, correctness, target, and security remediation responsibilities | `results/2026-08-12-option-c-slice-6-remediation-economics.md` | Slice 6 complete as a maintenance model; neither a provider update nor C selection |
| 2026-08-12 | Bounded spatial and independent point-cloud bulk-operation classification | `bulk-operation-candidate-classification.md` | Slice 7 selection complete; CPU reference implementation remains Slice 8 work |
| 2026-08-12 | CPU-only ordered AABB/point classification, result-consumption, residency, and 1K--1M scaling controls | `results/2026-08-12-option-c-slice-8-cpu-bulk-reference.md` | Slice 8 complete; no GPU provider, synchronization, or public contract selected |
| 2026-08-12 | Native WGPU ordered point/AABB and browser-WebGPU ordered-point parity, lifecycle timing, readback, scoped shader-validation, input rejection, idle disposal, and caller-selected CPU bypass | `results/2026-08-12-option-c-slice-9-native-wgpu-point-control.md` | Slice 9 complete as bounded corpus evidence; actual unavailable/device-loss/in-flight-cancellation and NVIDIA coverage are retained limitations |
| 2026-08-12 | Specialized native compute-provider comparison gate | `results/2026-08-12-option-c-slice-10-specialized-provider-gate.md` | Slice 10 complete as not earned; WGPU/CPU evidence has no named deficit requiring raw Vulkan, CUDA, HIP, or another provider |
| 2026-08-12 | AR-0026 corpus-local chart transition A/C trace | `results/2026-08-12-option-c-slice-11-chart-cross-review.md` | Native and actual DOM/WASM A/C agreement; bounded ordinary operation surface retained |
| 2026-08-12 | A/C and CPU/WGPU provisional decision matrices | `results/2026-08-12-option-c-slice-12-provisional-decision-matrices.md` | Retain A and corpus-only compute; continue C incubation pending Slice 11 completion |

Future entries must name the exact caller-shaped workload, target, profile,
toolchain, and machine/provider metadata needed to bound their claim. A summary
row may link several artifacts but may not erase their distinct dates or
limitations.
