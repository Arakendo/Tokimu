# Renderer Resource Identity Baseline Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-11 |
| Plan | [Renderer Resource Identity And Failure Presentation](../renderer-resource-identity-and-failure-presentation.md) |
| Reviews | AR-0024, AR-0027 |
| Synthetic fixture | `corpus/campaigns/renderer-reliability/hello-render-resource-identity` |
| Real caller | `corpus/campaigns/doom/hello-doom-e1m1` dynamic manual door |
| Status | Slice 1 complete; Slice 2 alternatives remain open |

## Claim

The original E1M1 failure was a resource-identity/lifetime defect independent
of Doom geometry, texture decoding, WGPU, and native-window behavior.

The current mesh upload seam intentionally permits replacement: WGPU mesh
upload tests whether the handle already exists, inserts the new GPU mesh at the
same handle, and records a mesh replacement. That behavior is necessary for a
caller intentionally updating one stable mesh. A typed `MeshHandle` alone does
not communicate whether the replacement belongs to the same logical resource.

## Deterministic Synthetic Reproduction

Command:

```powershell
cargo run -p hello-render-resource-identity
```

Retained output:

```text
AR-0024/0027 mutable-offset baseline: cutout=3 dynamic=3 recomputed-cutout=4 dynamic-upload=Replaced { handle: MeshHandle(3), previous: StaticCutout(0), replacement: Dynamic(0) } original-resolves=Some(Dynamic(0)) recomputed-resolves=None
AR-0024/0027 fixed-range baseline: cutout=3 dynamic=4 cutout-resolves=Some(StaticCutout(0)) dynamic-resolves=Some(Dynamic(0))
```

The mutable-offset fixture demonstrates both consequences of the historical
bug:

1. appending an opaque dynamic draw derives handle `3` and replaces the live
   cutout at handle `3`; and
2. recomputing the cutout base from the new opaque count points commands at
   handle `4`, which was never uploaded.

The disjoint-range baseline keeps live identity stable when the dynamic draw is
added. This is evidence for Alternative A only; it does not show that manual
numeric ranges are a desirable shared contract.

## Intentional Replacement Control

The fixture uploads the same logical `Dynamic(4)` identity twice at handle `7`.
The second upload is retained as a replacement candidate rather than rejected.
This prevents the study from adopting the incorrect rule that every repeated
handle is a collision.

The unresolved question is how create/replace intent and logical ownership
become observable with acceptable cost. Slice 2 compares that question across
application registries, generational handles, explicit lifecycle operations,
and validation-only alternatives.

## E1M1 Real-Caller Evidence

The canonical E1M1 source pressure retained two `DOORTRAK` middle spans. Each
span lowers to two triangles when it gains area, producing four dynamic draws:

```text
linedef 155 / sidedef 213 / sector 4 / middle / DOORTRAK
linedef 156 / sidedef 214 / sector 4 / middle / DOORTRAK
```

At the closed door height these spans have zero area and are retained static
degenerate omissions. Raising sector 4 gives them area, so the application must
create four new ordinary textured triangles rather than replace a pre-existing
static draw. The repaired observer reserves fixed static-cutout identities and
a separate monotonic dynamic range. A native manual observation on AMD Radeon
RX 7900 XTX/Vulkan retained the original 1,835 opaque plus 26 cutout scene and
reported that the door and its side tracks animated correctly without the
previous silent exit.

## Deterministic E1M1 Close/Reopen Replay

Command:

```powershell
cargo run -p hello-doom-e1m1 --bin static_scene -- corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD --masked-cutouts --door-resource-replay-report
```

Retained output:

```text
E1M1 Slice 1 dynamic-resource replay: linedef=151; target-sector=4; closed-initial-draws=0; closed-initial-handles=0; opened-handles={1835: MeshHandle(1862), 1836: MeshHandle(1863), 1837: MeshHandle(1864), 1838: MeshHandle(1865)}; opened-sources=1835:wall:155:DOORTRAK | 1836:wall:155:DOORTRAK | 1837:wall:156:DOORTRAK | 1838:wall:156:DOORTRAK; opened-enabled=4; closed-suppressed=4; reopened-handles={1835: MeshHandle(1862), 1836: MeshHandle(1863), 1837: MeshHandle(1864), 1838: MeshHandle(1865)}; reopened-enabled=4; stable-reopen=true; dynamic-after-cutouts=true; cutout-last-handle=Some(1861); source-map-mutated=false; renderer-initialized=false
```

This direct E1M1 composition replay proves the required lifetime sequence:

1. closed zero-area spans create no dynamic draw or handle;
2. opening materializes the four expected `DOORTRAK` triangles at handles
   1862–1865, after the last cutout handle, 1861;
3. closing suppresses all four dynamic-only entries without destroying their
   retained identities; and
4. reopening re-enables all four at the exact original handles.

The report does not initialize WGPU. It is deliberately an application-side
identity/lowering test, not evidence for renderer allocation or GPU lifetime
policy.

## Validation

```text
cargo test -p hello-render-resource-identity
```

Results:

```text
cargo test -p hello-render-resource-identity: 8 passed, 0 failed
cargo test -p hello-doom-e1m1 --bin static_scene: 13 passed, 0 failed
```

The tests prove:

- same-logical-identity upload remains a deliberate replacement candidate;
- mutable offset allocation reproduces alias plus unresolved reference; and
- disjoint ranges preserve both live logical resources.
- the E1M1 static-scene helper validates the deterministic replay command
  compiles alongside the existing geometry/orientation evidence.

## Scope Clamp

The fixture is a corpus-owned observation model using Tokimu's existing typed
`MeshHandle`. It does not modify `tokimu-render`, expose a public allocator,
define retirement, admit generations, choose recovery policy, or claim that a
logical-resource label belongs in renderer vocabulary.
