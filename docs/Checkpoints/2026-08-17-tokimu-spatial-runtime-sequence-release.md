# Checkpoint: Spatial Runtime Sequence And Release Economics

| Field | Value |
| --- | --- |
| Date | 2026-08-17 |
| Status | Door/platform sequence complete; immutable replacement selected as corpus reference |
| Historical plan filename | `docs/Plans/DOOM/Tokimu BSP capability setup plan.md` |

## Completed Matrix

The runtime report now covers nineteen immutable height snapshots: nine door
states from closed through opening/open/closing/closed and ten platform states
from high through descent/low/wait/ascent/high.

For every revision:

- immutable rebuilt BVH equals complete current-geometry brute force;
- reusable static BVH plus current dynamic sidecar equals brute force;
- baseline artifact identity is rejected as stale;
- activation, phase timing, collision and observer policy remain absent.

Repeated heights receive distinct application revision identities even when
their geometry and structural fingerprints repeat.

## Member Behavior

The reusable static set remains `1,831` members. Door snapshots contain `18`
dynamic members while closed and `22` while moving. Platform snapshots contain
`18` at endpoints and `20` during motion.

Bounds-only refit is valid only at baseline-equivalent closed/high states. All
genuinely moving snapshots add or replace exact members, so refit remains
semantically ineligible for the runtime workload.

## Release Economics

A direct release run was followed by twenty complete release replays, yielding
`380` snapshot samples:

```text
geometry preparation mean       2.719 ms
BVH rebuild mean                 0.430 ms
immutable total mean             3.149 ms
sidecar extraction mean          0.133 ms
sidecar total mean               2.852 ms
rebuilt query mean               0.0568 ms
sidecar-union query mean         0.0602 ms
```

The sidecar saves about `0.297 ms`, or `9.4%` of total measured update cost,
because both experimental paths currently reconstruct the complete geometry
first. Its composite query is slightly slower on average.

## Corpus Disposition

Immutable replacement becomes the reference lifecycle for portable evidence:

```text
snapshot revision N
    -> exact current geometry N
    -> immutable spatial artifact N
```

The sidecar remains a retained optimization candidate. It can reopen if a
caller naturally supplies changed fragments without complete reconstruction or
an explicit performance budget makes the saved rebuild material. No shared
capability or lifecycle contract is admitted here.

## Remaining Gate

A portable CPU/WASM consumer is next. Selecting its repository location and
deciding how much corpus-local machinery may move out of the native binary is
an architectural placement decision; implementation stops here for review.

## Validation

- `cargo fmt --all`
- `cargo test -p hello-doom-e1m1 --bin static_scene` — `79/79` passed
- `cargo clippy -p hello-doom-e1m1 --bin static_scene --no-deps -- -A dead-code -D warnings`
  passed with only the previously retained ordered-occurrence dead-code
  allowance
- Debug and release nineteen-snapshot reports completed with zero strategy
  query failures and zero stale-revision failures
- Twenty additional release replays produced the retained `380`-sample
  aggregate
- `git diff --check`
