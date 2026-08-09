# GPU Adapter Validation Coverage

## Current Hands-On Coverage

As of 2026-08-09, the maintainer can directly exercise these GPU families and
native backend paths:

- AMD Radeon through native Vulkan;
- Apple GPU hardware through native Metal.

Equivalent hands-on NVIDIA hardware is not currently available. NVIDIA paths,
including their Vulkan and D3D12 realizations where applicable, are therefore
**unverified rather than known-good or known-bad** and may contain gaps that
the AMD, Apple, and browser observations do not expose.

## Evidence Rule

Every retained renderer observation should record the adapter, backend, device
kind, operating system, target, and build identity when available. A passing
AMD/Vulkan, Apple/Metal, or browser/WebGPU observation must not be generalized
into a claim of NVIDIA conformance.

Until NVIDIA evidence is available:

- keep NVIDIA coverage visibly open in renderer corpus plans and review
  records;
- distinguish compilation from execution and first-frame presentation;
- do not weaken validation or introduce NVIDIA-specific behavior based on
  speculation;
- retain any user- or CI-supplied NVIDIA observation with exact adapter,
  driver, backend, and reproduction details; and
- prioritize NVIDIA checks when a stable renderer contract, shader path,
  surface format, depth/blend state, or performance claim is being admitted.

This is a validation-coverage limitation, not an architectural decision. If a
specific NVIDIA failure appears, preserve it in the relevant corpus or
Architectural Review rather than treating this general note as the defect
record.
