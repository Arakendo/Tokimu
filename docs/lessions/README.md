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

## Maintenance Rule

Keep a lesson compact and evidence-linked. When a lesson becomes a binding
ownership rule or public contract, promote the decision into an ADR and retain
this file only as implementation guidance.

