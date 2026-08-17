# Doom E1M1 Ordered Plane Occurrence Association Evidence

## Purpose

This record retains the first E1M1 correlation between the continuous ordered
source-occurrence observation and Doom-owned floor, ceiling, sky, and
paired-sky facts. It is a source-ownership and conservation gate, not a plane
lowering or visual-correctness claim.

## Invocation

```powershell
cargo build -p hello-doom-e1m1 --bin static_scene
.\target\debug\static_scene.exe corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD --render-strategy=ordered-occurrence-prepared-full --topology-inventory-report
```

## Fixed source-spawn observation

```text
occurrences=171
with-marked-planes=157
without-marked-planes=14
associations=259
floor-associations=140
ceiling-associations=119
sky-ceiling-associations=12
paired-sky-adjustments=8
distinct-floor-planes=44
distinct-ceiling-planes=36
distinct-sky-ceiling-planes=6
plane-instances=80
plane-instance-occurrence-references=259
plane-instance-subsector-references=146
plane-destination-references=146
plane-destination-source-triangles=304
plane-destination-unresolved-fail-open=0
boundaries=171
one-sided-boundaries=101
open-two-sided-boundaries=63
closed-two-sided-boundaries=7
wall-consumer-references=321
plane-consumer-references=259
unresolved-fail-open=0
occurrence-conservation=balanced
association-conservation=balanced
plane-instance-conservation=balanced
plane-destination-conservation=balanced
boundary-conservation=balanced
consumer-conservation=balanced
continuous-vertical-coverage-ready=true
legacy-screen-columns-used=false
renderer-mutation=false
plane-lowering=false
```

## Interpretation

- Every retained horizontal occurrence has exactly one accounted outcome:
  marked plane ownership, no marked plane, or explicit unresolved fail-open.
- Every emitted plane association is classified exactly once as floor or
  ceiling. Sky is retained as a ceiling-plane property rather than an
  independent occlusion mechanism.
- Distinct plane identity preserves source sector, plane kind, current source
  height, texture, and light level. Equal heights do not merge unrelated
  sectors.
- Paired-sky adjustments remain explicit source facts and do not independently
  authorize visibility or a world-space occluder.
- The existing 320-column reconstruction was not used. This avoids turning a
  diagnostic raster into semantic geometry.
- Every retained occurrence now has one shared Doom-private vertical boundary.
  Two-sided openings use `max(front floor, back floor)` through
  `min(front ceiling, back ceiling)`; one-sided and closed two-sided boundaries
  remain solid.
- All 321 prepared wall declarations and all 259 plane associations refer to
  those boundaries. Boundary and consumer conservation both balance.
- Exact plane identity groups those associations into 80 source-owned plane
  instances. Their 146 instance/subsector references all resolve to exact
  source-region destinations containing 304 source triangles; no destination
  was hidden or merged across sectors.
- Destination correlation does not claim that an entire referenced source
  region is visible. It is the explicit gate before Doom-owned preparation
  decides which portion survives.
- Paired-sky remains metadata and does not alter the shared opening. Missing or
  reversed source heights fail open rather than fabricating coverage.
- This is still decoded/current sector-height evidence, not application-owned
  door or platform runtime-snapshot evidence.
- The observation does not yet lower floor, ceiling, or sky contributions.
  Original E1M1 plane declarations therefore remain unchanged and fail open.

## Validation

```text
cargo fmt --all                                      PASS
cargo test -p hello-doom-e1m1 --bin static_scene ordered_occurrence
                                                    PASS (13 focused tests)
cargo clippy -p hello-doom-e1m1 --bin static_scene -- -D warnings
                                                    PASS
real E1M1 topology inventory report                 PASS
```

The repeated Windows incremental-cache hard-link fallback warnings are host
tooling noise and not corpus findings.

## Next gate

Use the exact source-region destinations as inputs to source-owned plane
preparation, then lower every retained floor, ceiling, and sky contribution
into an ordinary Tokimu declaration. Do not equate destination existence with
whole-region visibility, lower from mark identity alone, or revive legacy
screen columns as authority. Correlate explicit runtime height snapshots before
claiming dynamic completeness.
