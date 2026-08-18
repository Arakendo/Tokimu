# Lessons Learned

This folder retains short operational references earned through corpus work.
These notes help repeat a successful setup or avoid a known debugging trap;
they are not architectural decisions and do not override the SDD, an accepted
ADR, or an active Architectural Review.

## Quick References

- [Browser WebGPU and WASM initialization](webgpu-wasm-quick-reference.md) —
  async provider construction, surface setup, readiness evidence, portable
  timing, and failure isolation.
- [Geometry orientation and orbit controls](geometry-and-orbit-quick-reference.md)
  — winding versus normals, Canvas projection, reflections, culling, and drag
  sign conventions.
- [GPU adapter validation coverage](gpu-adapter-validation-coverage.md) —
  currently available AMD/Vulkan and Apple/Metal evidence, plus the explicit
  NVIDIA hardware coverage gap.
- [Camera clip depth and provider adaptation](camera-clip-depth-provider-adaptation.md)
  — why accepted draws can produce an empty frame and where GL-to-WebGPU depth
  conversion belongs.
- [Read available reference source earlier](read-reference-source-early.md) —
  when bounded source inspection can expose the invariant sooner, and how to
  keep a reference implementation from becoming Tokimu's architecture.
- [Bounds authority follows the bounded representation](bounds-authority-follows-bounded-representation.md)
  — audit raw data, declared source members, and derived geometry separately
  before granting a hierarchy bound rejection authority.

## Maintenance Rule

Keep a lesson compact and evidence-linked. When a lesson becomes a binding
ownership rule or public contract, promote the decision into an ADR and retain
this file only as implementation guidance.
