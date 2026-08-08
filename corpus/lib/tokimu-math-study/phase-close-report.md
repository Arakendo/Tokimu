# Native Math Vocabulary Study: Evidence-Phase Close Report

| Field | Value |
| --- | --- |
| Status | Evidence phase complete; stable selection explicitly deferred |
| Date | 2026-08-08 |
| Governing review | `docs/Architectural Reviews/AR-0019-native-math-vocabulary-and-foreign-type-boundary.md` |

## Current Conclusion

Retain Alternative A (`glam` re-exports) as the stable control. Do **not** migrate Tokimu's stable math vocabulary yet. B and C remain retained corpus experiments; D is retained only as a bounded provenance case study and is rejected for expansion.

| Alternative | Evidence-phase disposition | Main tradeoff | Evidence |
| --- | --- | --- |
| A | Stable control, not permanent semantic verdict | Lowest migration cost; public foreign vocabulary remains | `decision-matrix.md`, ADR-0010 audit |
| B | Retain as transition/comparison option | Tokimu names but private-provider/update coupling and explicit seams | `migration-accounting.md`, native/WASM conformance |
| C | Leading self-owned implementation experiment | Semantic/implementation independence, with numerical, target, and maintenance responsibility; native stereo result needs broader pressure | `maintenance-forecast.md`, performance observations, shared conformance |
| D | Retain provenance specimen; reject expansion | Exact lineage/control, but no matrix pressure and added update/diff obligation | `alternative-d-bounded-fork/bounded-status-report.md` |

## What This Phase Established

- A/B/C shared conformance executes natively and on Node WASM using the same assertions.
- B/C have bounded real-caller ports, explicit current-renderer conversions, source-edit accounting, rollback steps, and zero-allocation observations for named paths.
- Current WGPU upload uses renderer-owned scalar arrays, while public `Camera` fields remain a real provider-vocabulary seam.
- D is auditable but has not earned matrix expansion or a performance/migration comparison.
- The reusable checklist separates semantic, public-vocabulary, implementation, source, and representation ownership.

## Selection Blockers Deliberately Preserved

1. Re-run the inventory after the DOOM WAD plan changes actual object, transform, animation, imported-data, collision, and rendering pressure.
2. Obtain browser/WGPU and broader application evidence; the native-window corpus copies are not WASM-shaped.
3. Decide the eventual public camera/serialization/FFI boundary rather than treating the current `Camera` provider type as an internal detail.
4. Choose a stable degenerate/non-finite contract and add proportionate numerical/property evidence if C remains viable.
5. Establish whether `Vec2` and `Quat` have real caller or downstream compatibility pressure.

## Guardrail

No stable crate, public API, ADR decision, or SDD ownership boundary changes as part of this evidence phase. Experimental source remains corpus-local, `publish = false`, and removable by deleting its corpus directories. Any selection requires a new ADR-backed migration/retirement plan after the blockers above are resolved.
