# Checkpoint: Tokimu Spatial Bake Slice 1

| Field | Value |
| --- | --- |
| Date | 2026-08-17 |
| Status | Slice 1 implementation complete enough to expose a material BSP/BVH result; paused before query work |
| Plan | `docs/Plans/DOOM/Tokimu BSP capability setup plan.md` |
| Implementation | `corpus/campaigns/doom/hello-doom-e1m1/src/bin/static_scene/diagnostics/tokimu_spatial_bake.rs` |

## Implemented

- Corpus-local `--tokimu-spatial-bake-report`.
- Exact prepared triangles as the first finite Tokimu member representation.
- Stable original identity retained by every BSP fragment.
- Deterministic median-axis BSP with triangle-plane clipping.
- Global generated-fragment budget with conservative leaf fallback.
- Same-inventory median BVH control without member splitting.
- Independent node/leaf/member containment audits.
- Original-member and triangle-area conservation checks.
- Structural fingerprints and family-specific amplification.
- Focused split/conservation/containment/determinism tests.

No Doom BSP topology is consumed. No renderer membership, runtime ownership,
shared crate, stable contract or presentation behavior changes.

## Canonical E1M1 Evidence

Input is `1,849` prepared triangles: floor `463`, ceiling `390`, wall `970`,
cutout `26`.

The bounded BSP reaches its `500,000` generated-fragment budget and retains
`334,345` final fragments (`180.824770x`) in `14,231` nodes and `7,116` leaves
at maximum depth `20`. It has zero containment failures, no missing originals,
and fingerprint `78b7e9300f148c33`. The fragment payload has a 16,048,560-byte
lower bound before tree/vector allocation overhead.

The BVH retains all `1,849` unsplit members in `255` nodes and `128` leaves at
depth `7`, with zero containment failures, no missing/duplicate members, and
fingerprint `599d8ca7411ffd11`.

Observed debug-build construction is approximately `406–412 ms` for the
bounded BSP and `4.9–5.2 ms` for the BVH. These are diagnostic observations,
not release performance guarantees.

Family amplification is:

```text
floor      254.246220x
ceiling    173.261538x
wall       150.635052x
cutout     113.115385x
```

## Ordinary Finding Resolved

The first implementation checked its nominal fragment limit per node rather
than globally. It produced `650,337` final fragments (`351.723634x`) after
`974,955` generated fragments. The corrected implementation uses a global work
budget and conservatively stops subdividing affected nodes. The first result is
retained only as fragmentation evidence.

## Architectural Finding

Naive median-axis triangle splitting is not a viable default spatial index for
this E1M1 representation. The BVH currently supplies the same Slice 1
containment and conservation properties with no splitting and far lower build
cost.

This does not prove that no BSP can be useful. It does require an explicit next
decision:

1. test a different BSP split-plane/member policy with a stated reason it
   preserves useful partition semantics;
2. test a non-splitting partition and explain how it differs semantically from
   the BVH/control; or
3. advance the BVH control to actual-camera queries and require BSP to identify
   missing semantic value before receiving more implementation effort.

Slice 2 is not started automatically because that choice affects what Tokimu
means by BSP versus a smaller spatial-query capability.

## Validation

- `cargo fmt --all`
- `cargo test -p hello-doom-e1m1 --bin static_scene` — `76/76` passed
- Two independent canonical E1M1 report runs reproduced both structural
  fingerprints and all structural/conservation totals.
- `git diff --check` remains required at handoff.
