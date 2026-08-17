# Doom Ordered Reference Planner Synthetic Gate Evidence

## Scope

This evidence records the bounded synthetic gate for the Doom-private ordered
reference planner required before another E1M1 prepared-full-submission
attempt. The planner composes existing production-provider observations; it is
not a second renderer, historical pixel-parity implementation, public span
API, or application movement controller.

## Command

```powershell
cargo run -p hello-doom-visibility-conformance --bin ordered_reference_planner_report
```

Focused regression tests:

```powershell
cargo test -p hello-doom-visibility-conformance ordered_reference::tests
```

## Retained result

```text
cases=14
balanced=14
solid and pass ranges present=true
vertical clip mutations present=true
wall tiers present=true
plane instances present=true
sky intervals present=true
deferred masked work present=true
fail-open evidence retained=true
application movement policy present=false
fingerprint=3733cd002d19388cbb0460e478090c5cec708793d613d184c030249aaff5dcff
```

The 14 states are paired-sky, one-sky-negative, vertical-aperture,
shared-plane-key, four declared door snapshots, two declared platform
snapshots, three projection-epsilon controls, and cutout-non-occluder.

## Invariants established

- Near-first admitted SEG order is retained with solid/pass classification.
- Per-column vertical transition chains are contiguous.
- Retained wall tiers remain inside both their source raw interval and current
  open interval.
- Plane instances cite only admitted source SEGs.
- Paired-sky produces paired-sky intervals while the one-sky negative control
  does not acquire that authority.
- The vertical aperture retains both wall-tier and plane-mark evidence.
- A shared plane key retains multiple source instances rather than collapsing
  them into one visibility fact.
- Declared door and platform snapshots change the planner fingerprint without
  importing activation, timing, waiting, reversal, or movement policy.
- Near-plane ambiguity remains explicit fail-open evidence.
- A two-sided masked middle remains deferred work and does not close source
  coverage.

## Validation and limitation

The four focused planner tests pass. The full package currently has one
pre-existing failing regression,
`two_sided_aperture_retains_independent_upper_lower_opening_and_plane_intervals`,
which expects a floor-plane key absent from the present provider observation.
That baseline failure is not caused by this planner and remains separately
open; it is not hidden by the focused result.

The retained fingerprint changed after the released Doom clip-state audit
repaired three signed/inclusive-to-unsigned/exclusive translation errors in
the production provider: the first ceiling-plane row, the no-upper ceiling
transition, and the last open row below `floorclip`. All 14 cases remain
balanced after that repair. The fingerprint change is therefore retained as
expected semantic evidence rather than normalized back to the earlier value.

This gate authorizes the next integration step only: E1M1 must consume one
coherent planner result, lower every retained semantic contribution with an
explainable destination, and submit all resulting declarations without a
generic camera filter. E1M1 visual correctness and contribution conservation
remain unproven.
