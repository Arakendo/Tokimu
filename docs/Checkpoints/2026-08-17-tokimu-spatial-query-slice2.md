# Checkpoint: Tokimu Spatial Query Slice 2

| Field | Value |
| --- | --- |
| Date | 2026-08-17 |
| Status | BVH actual-camera control complete; BSP construction parked |
| Plan | `docs/Plans/DOOM/Tokimu BSP capability setup plan.md` |
| Implementation | `corpus/campaigns/doom/hello-doom-e1m1/src/bin/static_scene/diagnostics/tokimu_spatial_bake.rs` |

## Disposition

The unsplit BVH control advances through actual-camera queries. Further BSP
construction is parked until a corpus caller identifies required front/back
partition topology, partition-aligned fragments or another semantic result
that a BVH or smaller conservative spatial-query mechanism cannot honestly
provide.

No shared spatial capability, crate, provider trait or public contract is
admitted by this checkpoint.

## Implemented

- Headless `--tokimu-spatial-query-report`.
- Deterministic BVH rebuild over all `1,849` prepared E1M1 triangles.
- Tokimu checked view/projection queries at spawn, yaw, pitch, retained movement
  and retained off-axis ray poses.
- Hierarchical conservative AABB/frustum query with same-member brute-force
  parity.
- Hierarchical exact nearest-triangle ray query with brute-force parity.
- Per-view bake/view/dynamic identity, traversal work, conservation and timing
  diagnostics.
- Focused synthetic query-parity test.

Nothing changes renderer submission or presentation membership.

## Canonical Evidence

The immutable bake fingerprint remains `599d8ca7411ffd11`. Across nine views:

```text
frustum false negatives    0
frustum false positives    0
nearest-ray mismatches      0
matrix fingerprint          3c80342bb2cfcdf4
```

The BVH tests `287..1,257` members for frustum queries and `43..361` triangles
for rays instead of brute-forcing all `1,849`. The four retained Doom problem
observations hit the same source-correlated floor/wall triangles as the prior
`LOOK` evidence.

## Ordinary Finding Retained

The native `static_scene` binary does not compile as a WASM binary because its
existing startup/runtime path imports the native window runner and constructs
a native window-backed renderer. That failure occurs outside the corpus-local
query algorithm. It is evidence that browser parity needs an authorized
portable consumer location, not justification to publish a spatial API now.

## Architectural Finding

No tested caller consumes BSP partition planes or split-fragment topology.
For current conservative frustum and exact ray needs, the unsplit BVH preserves
the exact Tokimu representation and matches brute force while the Slice 1 BSP
amplifies it `180.82x`.

The evidence therefore favors a possible future optional conservative
spatial-query capability, with BVH/BSP/grid as implementation choices, over a
Tokimu BSP abstraction. That capability is not yet admitted: runtime dynamic
updates, a portable browser consumer and a second non-Doom caller remain
required architectural pressure.

## Remaining Work

- Exercise immutable snapshot replacement/refit policy for doors/platforms.
- Establish browser/WASM consumer parity from an authorized portable location.
- Seek Quake or ordinary non-BSP corpus pressure before choosing Ring 2 shape.
- Resume BSP only when a concrete consumer requires its distinguishing
  semantics.

## Validation

- `cargo fmt --all`
- `cargo test -p hello-doom-e1m1 --bin static_scene` — `77/77` passed
- `cargo clippy -p hello-doom-e1m1 --bin static_scene --no-deps -- -A dead-code -D warnings`
  passed; the allowance isolates six existing unused ordered-occurrence report
  surfaces that fail the otherwise identical strict command.
- Two independent canonical query runs reproduced bake fingerprint
  `599d8ca7411ffd11`, matrix fingerprint `3c80342bb2cfcdf4` and all zero-delta
  correctness totals.
- `cargo check -p hello-doom-e1m1 --bin static_scene --target
  wasm32-unknown-unknown` reaches the retained native-binary lifecycle mismatch
  described above.
- `git diff --check`
