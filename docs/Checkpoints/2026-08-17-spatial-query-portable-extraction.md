# Checkpoint: Portable Spatial-Query Extraction

| Field | Value |
| --- | --- |
| Date | 2026-08-17 |
| Status | Portable corpus extraction complete; prospective capability moved to AR-0031 |
| Parent study | `docs/Plans/DOOM/Tokimu BSP capability setup plan.md` |

## Completed Deliverables

- Recorded the ADR-0015 decomposition before extraction.
- Extracted deterministic immutable BVH construction, audit, fingerprinting,
  frustum candidates, nearest-triangle rays, refit and revision checking into
  `corpus/lib/tokimu-spatial-query-study`.
- Kept source conversion, semantic families, runtime motion policy, oracle
  comparison, renderer submission and visibility interpretation outside.
- Switched E1M1 bake, camera-query and runtime-snapshot BVH paths to the shared
  implementation. The splitting BSP remains historical adversarial evidence.
- Executed one shared fixture in native Rust and on
  `wasm32-unknown-unknown` through `wasm-bindgen-test-runner`.
- Added a separate portable Doom consumer that adapts four exact retained E1M1
  triangles and rays without moving their source vocabulary into the library.
- Opened AR-0031 without adding an engine crate, provider contract, renderer
  integration or facade export.

## Retained Identity Evidence

```text
portable fixture structure fingerprint   a7ab8dffa4f4b487
portable fixture bound revision          0c2f9ba483384480
E1M1 complete BVH fingerprint            599d8ca7411ffd11
E1M1 nine-pose matrix fingerprint         3c80342bb2cfcdf4
E1M1 prepared geometry fingerprint        9f394a35516f5567
portable E1M1 subset fingerprint          3189fb35dfba3bdc
```

The portable fixture also retains candidates `[0,1,2]`, nearest identity `1`,
stale-revision failure and revised-refit queries on both targets.

## Validation

- Native study tests: `2/2` passed.
- Executed WASM study tests: `1/1` passed.
- Portable E1M1-subset consumer: native `1/1` and executed WASM `1/1` passed;
  all four retained rays resolved to the same four identities.
- E1M1 `static_scene` tests: `79/79` passed.
- Full E1M1 bake retained `255` BVH nodes, zero containment failures, zero
  missing/duplicate members and fingerprint `599d8ca7411ffd11`.
- Full nine-pose report retained zero disagreements and matrix fingerprint
  `3c80342bb2cfcdf4`.
- Full nineteen-snapshot report retained zero strategy-query failures and
  baseline-stale rejection at every revision.

## Disposition

These mechanics are reusable incubation infrastructure. AR-0031 now owns
possible capability admission. The next meaningful slice is an independent
non-Doom consumer, not public API design or another topology experiment.
