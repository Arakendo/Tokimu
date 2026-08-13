# Option C Slice 10: Specialized Native Compute Gate

| Field | Value |
| --- | --- |
| Date | 2026-08-12 |
| Disposition | Not earned; retain CPU reference and WGPU corpus evidence only |
| Decision scope | Whether to add a specialized native bulk-compute provider |
| Existing provider evidence | Native WGPU/Vulkan on AMD Radeon RX 7900 XTX; browser WebGPU DOM host |

## Decision Need

No named Tokimu caller currently requires a specialized provider. The evaluated
operation is intentionally narrow: ordered conservative point/AABB
classification under a fixed unit-cube control. It has no admitted stable API,
no GPU-resident world state, and no provider-owned continuation path. E1M1's
1,861 AABB control is explicitly a small negative/control case, not a compute
admission argument.

The relevant question is therefore not whether CUDA, HIP, or raw Vulkan could
be faster. It is whether the measured WGPU route has a named deficit that they
must repair. It does not.

## Measured Comparison

The table compares already-retained same-host native observations. CPU values
are Slice 8 median resident-input/reused-result times. WGPU values are Slice 9
median reused-buffer upload + dispatch + readback times; neither row hides the
separate WGPU cold adapter/device/setup cost.

| Workload | Count | CPU reused result | WGPU warm end-to-end | WGPU cold caller time | Result |
| --- | ---: | ---: | ---: | ---: | --- |
| AABB/E1M1 control | 1,861 | 19.0 µs | 872.8 µs | 412.1 ms | CPU clearly preferable |
| AABB | 100K | 1.998 ms | 1.529 ms | 415.2 ms | modest WGPU warm advantage only |
| AABB | 1M | 20.524 ms | 4.495 ms | 429.2 ms | WGPU is useful evidence at scale |
| Point | 100K/browser debug | 1.428 ms native control | 13 ms browser warm median | 56 ms browser caller time | browser execution/parity only; no crossover claim |

WGPU therefore has two sufficient outcomes for this gate:

- it is not a universal answer: the E1M1-sized operation strongly favors CPU;
- it is already a viable optional provider mechanism for a large, bounded
  native batch when its cold lifecycle and readback requirements are acceptable.

No observation identifies a workload for which WGPU's semantics, target reach,
or warm performance is inadequate and a specialized native provider repairs a
material deficit.

## Specialized-Provider Cost Without an Earned Deficit

| Candidate | New responsibility if added | Why it is not justified now |
| --- | --- | --- |
| Raw Vulkan compute | provider-specific resource/synchronization/shader path, target-specific deployment and diagnostics, and a second execution mechanism beside WGPU | WGPU already exercised Vulkan on the available native host |
| CUDA | NVIDIA-specific SDK/driver/toolchain and deployment boundary, FFI/unsafe review surface, and new source/provenance evidence | no NVIDIA hardware evidence and no WGPU deficit requiring NVIDIA-only execution |
| HIP or another accelerator API | accelerator-specific toolchain/driver/FFI/deployment boundary plus provider-specific diagnostics and target coverage | no caller requirement or hardware/tooling evidence that its capability is needed |

Each candidate would be an Outer Ring provider, never a Ring 0 math dependency.
Even there, a new foreign source/toolchain/FFI path would require security,
provenance, failure, and performance evidence proportionate to ADR-0008 through
ADR-0011. ADR-0010's audited Ring 0 source policy does not make a specialized
Outer Ring provider free of provenance or supply-chain responsibility.

## Conclusion

Slice 10 closes as **not earned**. No specialized provider is added, and no
provider-specific performance observation is generalized to native or GPU
execution. Future reconsideration requires all of the following:

1. a named caller-owned bulk operation with an independently retained semantic
   reference;
2. a measured material WGPU deficit on that workload;
3. maintainer hardware/tooling evidence for the proposed provider; and
4. a separate provider review covering ownership, failure, target reach,
   deployment, provenance, and security.
