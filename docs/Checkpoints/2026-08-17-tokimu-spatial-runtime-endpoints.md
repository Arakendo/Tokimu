# Checkpoint: Conservative Spatial Query Runtime Endpoints

| Field | Value |
| --- | --- |
| Date | 2026-08-17 |
| Status | Door/platform endpoint comparison implemented; lifecycle choice remains open |
| Historical plan filename | `docs/Plans/DOOM/Tokimu BSP capability setup plan.md` |

## Naming Disposition

The architectural candidate is now an optional conservative spatial-query
capability. BSP survives as a historical study name and possible provider, not
as the leading Tokimu-owned semantic concept.

## Implemented

- `--tokimu-spatial-runtime-report`.
- Exact prepared-equivalent baseline reconstruction with geometry-multiset
  fingerprint validation.
- Immutable current-height snapshots for door sector `4` (`ceiling 0 -> 68`)
  and platform sector `70` (`floor 104 -> -48`).
- Immutable rebuild, bounds-only topology-refit eligibility, and reusable
  static-BVH plus dynamic-sidecar comparisons.
- Current-snapshot frustum and exact-ray parity against complete brute force.
- Explicit baseline/current revision mismatch evidence.

Activation, timing, collision, observer carrying and renderer submission remain
outside this diagnostic.

## Evidence

The reconstructed baseline exactly matches all `1,849` prepared triangles by
family and geometry fingerprint `9f394a35516f5567`.

Both immutable rebuild and dynamic sidecar match the brute-force current
geometry oracle at both retained views. There are zero strategy query failures
and zero stale-revision failures.

The reusable sidecar contains `1,831` static members. Door-open uses `22`
current dynamic members; platform-low uses `18`.

Bounds-only refit is unsupported for both snapshots because member identity is
not stable. Door-open adds four triangles. Platform-low retains the total count
but replaces member identities. Treating either as a pure bounds change would
violate the exact-member contract.

Observed debug costs are approximately `10–11 ms` for current-geometry
preparation, `7.5–8.5 ms` for immutable rebuild, and `0.5–0.8 ms` to select the
current sidecar members after its static BVH exists. These are diagnostic
observations, not budgets or release guarantees.

## Ordinary Finding Resolved

The first runtime reconstruction contained 59 extra zero-area triangles: 22
floors, 21 ceilings and 16 walls. Ordinary static lowering already records
those authored empty faces as omissions. Applying the same omission rule
restored exact `1,849`-triangle baseline equivalence before strategy comparison.

## Remaining Work

- Add turbo-floor and intermediate/open/closing/reversal/wait phases.
- Measure release builds.
- Decide immutable replacement versus static-plus-dynamic union only after the
  expanded matrix.
- Build a bounded portable CPU/WASM consumer after lifecycle semantics settle.

## Validation

- `cargo fmt --all`
- `cargo test -p hello-doom-e1m1 --bin static_scene` — `78/78` passed after
  adding the geometry-fingerprint order/winding invariance test
- `cargo clippy -p hello-doom-e1m1 --bin static_scene --no-deps -- -A dead-code -D warnings`
  passed; the allowance isolates the previously retained ordered-occurrence
  report surfaces.
- Canonical `--tokimu-spatial-runtime-report` completed with zero strategy
  query failures and zero stale-revision failures.
- `git diff --check`
