# Doom Viewer-Relative Presentation Shared-Seam Audit

| Field | Value |
| --- | --- |
| Campaign | DOOM |
| Plan | [Doom Viewer-Relative Presentation Synthetic Conformance](../Studies/Doom%20viewer-relative%20presentation%20synthetic%20conformance.md) |
| Slice | 0 — Baseline And Shared Seam Audit |
| Status | Slice-0 and the first Slice-2 source-only extraction complete; continuous-plane fixture pressure remains |
| Scope | Doom corpus/provider code only |

## Result

The synthetic campaign can reuse several existing production Doom-provider
seams, but the first classic viewer-relative preparation path is still embedded
in the E1M1 executable. The synthetic target must not copy that path.

No `tokimu-render`, Native Ring, or provider-neutral visibility API change is
required. The smallest honest next step is a Doom-provider-local extraction,
followed by an E1M1 call-site migration and then the synthetic caller.

## Shared-Seam Inventory

| Responsibility | Current production seam | Current location | Disposition |
| --- | --- | --- | --- |
| Decoded source model | `DoomMapCore` and source-labelled records | `doom-map-provider` | Reuse unchanged below WAD decoding. |
| Viewer-relative BSP leaf order | `resolve_doom_viewer_subsector_order` | `doom-geometry-provider` | Reuse as a structural oracle and control. |
| SEG wall lowering | `lower_doom_seg_textured_wall_triangles` and `clip_doom_seg_textured_wall_triangle_to_linedef_interval` | `doom-geometry-provider` | Reuse directly; fixture code may not synthesize expected triangles. |
| SEG occluder classification | `observe_doom_seg_occluders` | `doom-geometry-provider` | Reuse directly; source wall roles remain Doom-owned. |
| Plane-mark classification | `observe_doom_seg_plane_marks` | `doom-geometry-provider` | Reuse directly; do not infer plane authority from rendered pixels. |
| Sky surfaces and paired-sky boundaries | `observe_doom_sky_surfaces` and `lower_doom_paired_sky_boundary_triangles` | `doom-geometry-provider` | Reuse directly for positive and negative sky controls. |
| Classic near-first traversal and solid-range admission | `observe_doom_classic_bsp` | `doom-geometry-provider` | Extracted. Both E1M1 and synthetic fixtures call this source-only provider seam. |
| Vertical tier clipping and plane-span observation | `observe_doom_classic_vertical_clip_state`, plane-key/instance helpers, and span finalization | `doom-geometry-provider` | Extracted. E1M1 and synthetic fixtures call one source-only observation; flat-cell reconstruction remains presentation-local. |
| Plane-cell reconstruction | `reconstruct_doom_seg_classic_plane_cells` and `resolve_doom_seg_classic_plane_flats` | E1M1 `static_scene.rs` | Keep Doom-campaign presentation preparation; Level 1 consumes its structural result, Level 2 lowers it. |
| Live door/platform state | `App::current_doom_visibility_map` plus runtimes in `specials.rs` | E1M1 application | Replace the app method with an explicit Doom-local height-override input/helper; decoded records remain immutable. |
| Source-labelled diagnostics | `format_source_classic_ray_trace`, `report_doom_seg_classic_*`, and bounded sample formatting | E1M1 `static_scene.rs` | Separate semantic observations from E1M1 console formatting. Share structured facts; keep presentation local. |
| Renderer realization | ordinary `StaticDrawPlanEntry`, meshes, materials, and draw commands | E1M1 and Tokimu renderer | No visibility semantics move into the renderer. |

## Minimum Extraction Sequence

1. **Complete:** move the classic traversal/admission input, result, and
   helpers into a Doom-provider-local module; preserve source SEG order,
   identity, admitted intervals, rejection reasons, and fail-open cases.
2. **Complete:** change E1M1 and the Slice-1 fixture builder to call that
   module. Do not expose an alternate test-only traversal.
3. **Complete:** extract source-only vertical clipping and plane-span
   observation when its first fixture needs it. Keep flat-cell reconstruction
   and all draw construction in the E1M1 presentation consumer.
4. Express dynamic doors/platforms as bounded sector-height overrides applied
   to a temporary source snapshot. Do not make runtime objects or mutable WAD
   records part of the shared interface.

This sequence intentionally leaves E1M1 material lookup, GPU upload, console
formatting, and interactive controls outside the semantic helper.

## First Extracted Seam

`doom_geometry_provider::observe_doom_classic_bsp` now owns the bounded
near-first BSP / horizontal solid-range observation. Both E1M1 and
`hello-doom-visibility-conformance` call it. It accepts only decoded
`DoomMapCore`, source viewer facts, and watched subsector identities; it emits
only Doom-source observation facts. It does not lower meshes, construct plane
spans, select renderer commands, or define a generic visibility abstraction.

The former E1M1 implementation remains only as test-only local scaffolding
while its narrow helper tests are migrated; production E1M1 execution no
longer calls that copy. This is tracked explicitly so test convenience cannot
be mistaken for a second production algorithm.

`doom_geometry_provider::observe_doom_classic_vertical_clip_state` now owns
the next source-only seam: admitted SEG wall-tier ranges, per-column vertical
clip facts, and source-keyed plane-span observations. The synthetic fixture
supplies only explicit wall extents and source view height, then calls this
same provider path; E1M1 keeps flat resolution, cell reconstruction, material
selection, and renderer realization outside the shared seam.

## Falsified Baselines

The campaign begins with failures, not desired screenshots:

- selecting whole flat meshes by reached BSP subsector removed continuous
  spawn-room floor and ceiling contributions around pillars;
- opening the first door exposed the sky enclosure until the observer crossed
  the doorway when traversal used immutable decoded heights;
- the hut aperture retained unrelated distant geometry even after coarse sky
  masking improved;
- close/viewer-plane walls disappeared under unsafe projected-range closure;
- the initial horizontal-only classic control admitted wall fragments without
  establishing matching plane/sky coverage.

The canonical wall-249 replay is:

```text
viewer subsector:       141
target linedef:         249
target sidedef:         348
target sector:          56
target subsectors:      190, 216
target SEG records:     560, 657
reached target leaves:  190
admitted target SEGs:   none
subsector 216:          pruned at node 219 by closed [0,319] range
paired-sky boundary:    none
source sky plane:       none
global-shell result:    wall 249 is the nearest ordinary prepared hit
```

A synthetic repair is not eligible for E1M1 until it explains these source
facts rather than hiding the wall with unrelated geometry.

## Fixture Schema And Bounds

The first builder uses explicit zero-based record indices in insertion order,
separately for each Doom record category. A fixture may name a record for human
readability, but the name never becomes identity. Tests comparing irrelevant
record-order perturbations retain an explicit normalization map; ordinary
fixtures do not silently renumber records.

Initial per-fixture limits are deliberately small:

| Record/fact | Maximum |
| --- | ---: |
| vertices | 64 |
| sectors | 16 |
| sidedefs | 64 |
| linedefs | 64 |
| SEGs | 128 |
| subsectors | 32 |
| BSP nodes | 31 |
| Things/viewers | 8 |
| dynamic sector-height overrides | 16 |
| retained expanded trace entries per category | 32 |

Each fixture manifest records the source records, viewer source position,
height and direction, runtime height overrides, expected semantic invariants,
and a deterministic fingerprint. The builder validates references but does not
select candidates, lower expected triangles, or decide visibility.

## Required Visibility And Refactoring Changes

- The existing `doom-geometry-provider` functions listed above require no
  visibility change.
- The classic traversal result types and helpers need Doom-provider-local
  visibility so E1M1 and the new corpus crate can call one implementation.
- Plane-span types are now provider-local source observations because Slice 2
  exercises them through both E1M1 and the synthetic fixture. They remain
  deliberately outside `tokimu-render` and any provider-neutral API.
- `current_doom_visibility_map` should become a small explicit snapshot helper
  accepting source-sector identities and temporary floor/ceiling heights.
- E1M1 trace strings remain application-local wrappers over shared structured
  observations.

## Ownership Check

The audited path can remain entirely below Doom decoding and above ordinary
Tokimu rendering declarations. No stable/public Tokimu contract, Native Ring
type, renderer scheduling policy, or provider-neutral visibility algorithm is
needed. Pressure to change that conclusion stops the campaign for review.

Quake remains standby evidence only. It is not part of this plan unless a Doom
result later needs an independently sourced comparison to decide whether an
invariant transfers.
