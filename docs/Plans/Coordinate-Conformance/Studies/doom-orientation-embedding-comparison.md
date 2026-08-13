# Doom Orientation Embedding Comparison

| Field | Value |
| --- | --- |
| Date | 2026-08-11 |
| Review | AR-0028 Cycle 4 |
| Status | Complete — Preserve North selected as the Doom consumer default |
| Source authority | Unchanged decoded `DOOM1.WAD` E1M1 facts |

## Candidates

The comparison introduces corpus-only adapters above identical decoded Doom
coordinates:

```text
CurrentReflected: east -> +X, north -> +Z
PreserveEast:     east -> +X, north -> -Z
PreserveNorth:    east -> -X, north -> +Z
```

`DoomComparativeEmbedding` is evidence machinery in `hello-doom-e1m1`. The
active `doom-geometry-provider` conversion remains unchanged.

## First Structural Matrix

| Invariant | Current | Preserve East | Preserve North |
| --- | ---: | ---: | ---: |
| Exact direction round trip | pass | pass | pass |
| Source orientation about world `+Y` | `-1` | `+1` | `+1` |
| Lifted source-right / camera-right alignment | `-1` | `+1` | `+1` |
| Doom east | `+X` | `+X` | `-X` |
| Doom north | `+Z` | `-Z` | `+Z` |
| Canonical hut source-right -> screen-right | fail | expected pass | expected pass |
| Rebuilt right/front and left/back winding | n/a | pass | pass |
| Readable U advances toward camera-right | compensated | pass | pass |
| Source heading / W-D strafe / right-look replay | baseline | pass | pass |
| Native fixed-spawn hut source-right | fail | observed pass | observed pass |
| Native fixed-spawn `EXITSIGN` readable | compensated pass | observed pass | observed pass |

The two candidates differ by a 180-degree rotation about world `+Y` when
geometry and camera are converted together. Doom source-relative screenshots
therefore cannot choose between them. An independent consumer or an explicit
Tokimu/Doom world-axis policy must determine whether east/X or north/Z is the
relationship worth preserving.

## Coupled Existing Behavior

The current provider already states that its `(doom_x, height, doom_y)` lift
reverses horizontal screen direction for right/front sidedefs. Its current U
axis deliberately decreases along right/front stored linedefs to keep source
art readable under that reflected presentation.

Consequently, readable `EXITSIGN` is not an independent endorsement of the
current embedding. It is evidence that wall texture-axis policy has already
compensated for the reflection. Either orientation-preserving candidate must
retest and probably revise that Doom-owned compensation together with wall
winding and normals; the generic supplied-UV renderer contract remains
unchanged.

## Source-Derived Sidedef Probe

`doom_sidedef_conformance` can now apply either candidate above identical
decoded fixture facts without changing `doom-geometry-provider`:

```powershell
cargo run -p hello-doom-e1m1 --bin doom_sidedef_conformance -- east
cargo run -p hello-doom-e1m1 --bin doom_sidedef_conformance -- north
```

The corpus-only migration probe transforms positions, rebuilds triangle
winding and normals, and removes the current reflection-compensating U
direction. Headless checks establish for both candidates that right/front and
left/back triangles still face their source-side observer and that U increases
toward presented camera-right. This is migration-surface evidence only; it has
not changed the active provider.

The candidates also survive a five-heading command replay (`0`, `45`, `90`,
`180`, and `270` degrees): W follows transformed source-forward, D follows
transformed source-right, and a screen-right pointer turn reaches the same
right direction without candidate-specific input signs.

## Canonical Native Observation

On 2026-08-11 the maintainer inspected both candidate E1M1 compositions at
the unchanged source player spawn with masked cutouts and diagnostic sky
surfaces enabled:

```powershell
target/debug/static_scene.exe `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts --diagnostic-sky-omissions --spawn-observer --embedding-east

target/debug/static_scene.exe `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts --diagnostic-sky-omissions --spawn-observer --embedding-north
```

For both candidates, the identified exterior hut appeared on source-right and
`EXITSIGN` remained readable. These are retained visual observations rather
than pixel-identical rendering claims. Their agreement confirms the coherent
migration but does not distinguish the candidates: rotating transformed
geometry and the source-derived camera together preserves Doom-relative
presentation.

The candidate executable converts interactive disc movement back through the
unchanged Doom collision/BSP source frame and lifts its resolved position into
the candidate frame. Conservative Doom-membership bounds are transformed with
the candidate. Dynamic door re-lowering now applies the same explicit adapter
before rebuilt wall spans rejoin the prepared scene.

## Remaining Matrix

- [x] Exact inverse and round-trip for current, Preserve East, and Preserve
      North.
- [x] Source determinant and camera-right alignment for all three.
- [x] Explicit east/north cardinal mappings.
- [x] Canonical hut signed-side result under both candidates (`+1120` for
      source-right and presented camera-right).
- [x] Prove both candidates must rebuild triangle winding to retain transformed
      right/front and inverse left/back ownership; renderer culling is not a
      valid compensation.
- [x] Right/front and left/back asymmetric U-axis behavior under both
      candidates.
- [x] Canonical `EXITSIGN` structural and native/browser visual regression.
  - [x] Native Preserve East and Preserve North visual observations.
  - [x] Browser WebGPU Preserve North observation: `1823/1823` opaque draws,
        `camera=canonical-exitsign`, `embedding=preserve-north`; the maintainer
        confirmed that `EXIT` reads properly.
- [x] Source heading, strafe, pointer-look, and fixed command replay.
- [x] Headless exact picking and horizontal collision source correspondence:
      both candidates preserve ray-hit distance, contacted linedef identity,
      broad-phase evidence, and resolved source position.
- [x] Canonical interactive collision/floor observations under both candidates:
      maintainer observed natural movement, blocking, and floor transitions.
- [x] Flat winding/normals and conservative BSP/source-membership invariance:
      both candidates report `463` floor-up, `390` ceiling-down, and identical
      membership controls (`237/237`, `1861` overview; `61/237`, `474`
      source-spawn-yaw-plus-90).
- [x] Asymmetric flat-UV observation: the diagnostic sky stand-in's source
      label `WALL` initially presented right-to-left under both candidates;
      reversing the continuous source-spatial flat U coordinate made it
      readable under both. This is a Doom lowering migration result, not a
      renderer/global PNG correction.
- [x] Explicit finding: Doom-relative evidence cannot select east/X versus
      north/Z. The corpus consumer chooses Preserve North as its declared local
      convention; no global Tokimu cardinal-axis policy is admitted.
- [x] Dynamic-door source correspondence: both candidate reports materialize
      four `DOORTRAK` draws on open, suppress four on close, reuse the same
      handles on reopen, and retain unchanged decoded source records.

## Acceptance Clamp

- Do not mutate decoded Doom coordinates.
- Do not choose a candidate from a source-relative screenshot that is invariant
  under the candidates' 180-degree relationship.
- Do not preserve the current Doom U compensation without retesting why it
  exists.
- Do not change generic renderer UVs, WGPU adaptation, or platform input.
- Do not change the active provider conversion until the remaining matrix
  identifies a coherent migration surface and a justified candidate.

## Architectural Result

The source reflection is incorrect because it reverses orientation and a
source-relative screen-side invariant. Preserve East and Preserve North are
both coherent repairs. Their remaining 180-degree world-Y difference is a
world-axis convention that Doom cannot select. A future owner may choose an
alignment, but a Doom adapter requirement should be limited to preserving
orientation and making any absolute alignment explicit.

The Doom corpus consumer now uses Preserve North by default because it keeps
Doom north aligned with world `+Z` while satisfying the orientation invariant.
That choice is an explicit application/source-adapter convention, not evidence
that all Tokimu worlds must use the same cardinal alignment. Preserve East
remains a coherent comparison control related by a 180-degree world-Y rotation.
