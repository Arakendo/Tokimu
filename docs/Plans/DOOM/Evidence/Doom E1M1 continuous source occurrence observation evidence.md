# Doom E1M1 Continuous Source Occurrence Observation Evidence

## Purpose

This record retains the first canonical E1M1 execution of the continuous
source-domain observer used by the
[Doom Ordered Source Occurrence Preparation](../Studies/Doom%20ordered%20source%20occurrence%20preparation.md)
campaign.

It proves that positive whole, partial, rejected, and unresolved source SEG
outcomes can be derived at the real E1M1 source-spawn pose without treating
Classic Doom's 320 diagnostic columns as authoritative renderer geometry.
It does **not** claim that the original global render declarations have already
been correlated, replaced, or rejected.

## Command

```text
cargo run -p hello-doom-e1m1 --bin static_scene -- corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD --render-strategy=ordered-occurrence-prepared-full --topology-inventory-report
```

## Retained Observation

```text
strategy=ordered-occurrence-prepared-full
source-segs-visited=732
whole-retained=16
partial-retained=16
whole-rejected=563
unresolved-fail-open=137
occurrences=171
occurrence-fingerprint=69cbc0a1e53db469
continuous-source-domain=true
diagnostic-columns-authoritative=false
renderer-mutation=false
original-contributions=all-fail-open
```

The unchanged original contribution inventory remains:

```text
records=1922
presentation_global=1
runtime_related=174
floor=463
ceiling=390
wall-upper=172
wall-lower=210
wall-middle=588
cutout-middle=26
sky-plane=73
aggregate_hash=30650e57ad9b3c07
unchanged=true
unresolved-fail-open=1922
```

## Interpretation

The observer traverses Doom source topology near-first, clips directed source
SEGs against continuous horizontal view half-spaces, subtracts continuous
projected solid coverage, and maps survivors back to normalized source-relative
SEG intervals. A source contribution may therefore yield zero, one, or more
bounded occurrences without deriving geometry from integer screen columns.

The 137 fail-open outcomes currently consist of near-plane-ambiguous source
SEGs. Each retains a full `[0,1]` occurrence and a bounded reason instead of
being hidden. Coarse BSP far-child pruning is also intentionally absent in this
baseline.

## Boundary

- The observer is enabled only by the named
  `ordered-occurrence-prepared-full` trial strategy.
- It does not mutate renderer input or introduce a renderer visibility API.
- It does not use inverse raster reconstruction.
- It has not yet correlated occurrences to the original wall, plane, sky,
  cutout, door, or platform contributions.
- It is horizontal occurrence evidence only; vertical wall-tier, plane, sky,
  and aperture conservation remain Slice 6 work.
- No visual correctness claim follows from these counts alone.

## Validation

Five focused tests pass for continuous FOV clipping, coverage splitting,
coverage merging, projected/source inversion, and strategy separation. The
canonical headless command completes successfully. The only emitted warnings
are the repository's known incremental-cache hard-link fallback notices.
