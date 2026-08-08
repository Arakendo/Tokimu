# Candidate Maintenance Forecast

## Scope

This is a responsibility forecast derived from the study’s retained source,
provenance, conformance, target, and migration evidence. It intentionally does
not estimate person-hours: Tokimu has no stable scope, release cadence, or
post-DOOM caller set from which a credible numeric estimate could be made.

## Recurring Work by Candidate

| Responsibility | A: direct provider | B: wrapper over provider | C: owned subset | D: derived subset |
| --- | --- | --- | --- | --- |
| Provider pin/security/toolchain review | Required | Required | Not for math mechanics | Required while any derived line remains |
| Public-vocabulary compatibility | Provider vocabulary and release behavior | Wrapper API, traits, constants, debug/equality semantics, conversions | Tokimu API evolution and compatibility | Same as C if expanded, plus upstream compatibility question |
| Numerical correctness/property coverage | Primarily provider audit and caller tests | Provider audit plus wrapper boundary tests | Tokimu owns vectors, matrices, inverse, degenerate policy, and differential/property coverage | Tokimu owns integration tests plus upstream behavioral drift review |
| Target/performance work | Revalidate provider/toolchain changes | Revalidate provider plus wrapper/conversion paths | Revalidate scalar lowering, layout, size, native/WASM behavior; justify any SIMD/unsafe addition | C-style work plus target-specific upstream extraction/provenance work |
| Renderer/FFI boundary work | Existing provider seam | Maintain private unwrap boundary or migrate facade | Maintain column reconstruction or migrate facade/representation | Not available for current `Vec3`-only scope |
| Source/update tracking | Pin and audit update | Pin/audit plus wrapper drift | Tokimu source/tests/docs only | Pin/audit plus exact upstream path, attribution, local diff, and fix-incorporation review |

## Current Evidence and Cost Signals

- A’s Tokimu probe is small, but it retains foreign public vocabulary and the
  provider’s compatibility/toolchain surface.
- B has a private-provider implementation and nine measured renderer crossings
  in the bounded fixtures. Its current wrapper/accessor/setter choices must
  remain synchronized with caller pressure and provider changes.
- C is dependency-free in its isolated crate and has no unsafe code, but it
  owns scalar numerical behavior, inverse behavior, cross-target layout, and
  the native/WASM performance divergence observed by this study.
- D is deliberately only a derived `Vec3`. Every expansion would add an
  upstream source selection, attribution/license check, local-diff record,
  update/fix policy, and target validation before it even reaches C’s matrix
  obligations.

## Minimum Ongoing Evidence If A Candidate Is Selected

### A

- Re-run ADR-0010 provenance/security/toolchain review on provider updates.
- Monitor provider compatibility warnings and public type exposure.

### B

- Perform A’s provider review plus wrapper public-API and conversion-boundary
  regression checks.
- Keep every provider crossing explicit; revisit the facade seam when caller
  pressure changes.

### C

- Maintain finite differential, edge/recovery, allocation, native/WASM,
  representation, and performance evidence for every admitted operation.
- Do not add SIMD or unsafe code without a measured deficit, a contained
  invariant, and target-specific validation under ADR-0008/0009/0011.
- Choose and document singular/degenerate behavior before it becomes public.

### D

- Perform C-equivalent maintenance for the admitted behavior, plus an upstream
  diff/fix/attribution review on every expansion or relevant provider update.
- Keep the source manifest bounded; otherwise D becomes a partial provider
  fork with the combined costs of C and dependency governance.

## Interim Conclusion

Tokimu can accept C’s maintenance burden only if post-DOOM caller pressure
continues to justify a very small, independently testable math core and the
target performance/layout results remain within accepted budgets. B is cheaper
to evolve mechanically but does not resolve executing-implementation ownership.
D has no demonstrated advantage over C and should remain paused. A remains the
lowest-change production control until a decision gate deliberately accepts a
different burden.
