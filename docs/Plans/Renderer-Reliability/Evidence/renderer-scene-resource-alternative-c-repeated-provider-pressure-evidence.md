# Alternative C Repeated Provider Pressure Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-19 |
| Status | Complete -- 27-cycle browser WebGPU pressure passed |
| Scope | Sustainability pressure for the fixed corpus-private WGPU staging mechanism |
| Stable API admission | None |

## Workload

The browser fixture holds one WGPU provider session and alternates two complete
scene-resource sets over 27 committed replacements. Each set contains:

- 64 meshes;
- 64 source textures at 16x16 RGBA8;
- 64 textured materials;
- one pipeline;
- one camera;
- 64 draw commands.

Every fifth replacement first creates the complete candidate set and then
attempts a 65th material referencing absent texture 65. The candidate must fail
explicitly, drop, and leave all 64 draws of the current set presentable before
the valid candidate is rebuilt and committed.

JavaScript yields through `requestAnimationFrame` before and after every
replacement. This supplies ordinary browser/provider progress boundaries and
avoids treating one long synchronous WASM call as replacement pressure.

## Required Observations

Every successful cycle requires:

```text
current logical sets after commit = 1
maximum logical sets during staging = 2

retired = committed =
    meshes: 64
    textures: 64
    materials: 64
    pipelines: 1
    cameras: 1
    commands: 64

presented draws = 64
provider diagnostics = 0
provider/device/surface creations = 1
```

Failure cycles additionally require 64 preserved current-scene draws before
the retry.

The fixture reports a bounded estimate of source texture payload plus mesh
vertex bytes for one live set and the current-plus-candidate overlap. These are
not GPU allocation or residency measurements. Commit drops the retired WGPU
object owners, but physical reclamation remains unobserved.

## Structural Validation

- browser WASM debug check passed;
- optimized browser WASM build passed;
- generated browser bindings rebuilt;
- browser JavaScript syntax check passed;
- strict `tokimu-render` clippy remained clean.

## Live Browser Result

The browser WebGPU run completed all 27 alternating replacements in 1,244.3
ms on one backend/device/surface session.

| Observation | Result |
| --- | ---: |
| committed replacements | 27 / 27 |
| injected late failures | 5 / 5 |
| failures reporting `MissingTexture(65)` | 5 / 5 |
| failures preserving 64 current draws | 5 / 5 |
| cycles presenting 64 successor draws | 27 / 27 |
| cycles retaining the exact steady logical inventory | 27 / 27 |
| cycles reporting zero provider diagnostics | 27 / 27 |
| cycles retaining one backend/device/surface | 27 / 27 |
| post-commit logical sets | 1 |
| maximum logical sets during staging | 2 |
| estimated live source bytes | 74,752 |
| estimated overlap source bytes | 149,504 |

After excluding the first 108.3 ms provider/setup-bearing replacement, warm
replacement observations ranged from 2.8 to 6.7 ms and averaged 3.854 ms.
These timings describe this fixture and machine only; they are not an engine
budget or portable performance guarantee.

Every commit reported symmetric retirement and installation of 64 meshes,
textures, materials, and commands plus one pipeline and one camera. The
instance-binding cache remained at 64 entries rather than accumulating with
replacement count.

The run shows no obvious logical accumulation, lifecycle corruption, provider
diagnostic, or failure-containment regression under the defined workload. Each
commit issued drops for the retired provider-object owners. Physical GPU
reclamation timing remains unobserved.

## Limits

This slice does not:

- claim leak freedom beyond the defined workload;
- prove physical GPU reclamation timing;
- introduce reclamation heuristics;
- alter the staging or identity semantics;
- define public provider, generation, or handle APIs;
- authorize architectural admission.
