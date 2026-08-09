# DOOM WAD Implementation Readiness Audit

| Field | Value |
| --- | --- |
| Status | Slice 4 source decoding, provider-neutral lowering, and a structural artifact are complete; visual artifacts remain |
| Date | 2026-08-08 |
| Scope | Current repository state against `DOOM WAD Checklist.md` before visual artifact evidence begins |

## Baseline Finding

The checklist's fixture/provenance phase and Slice 1 container inspection are
complete. A corpus-local WAD container provider validates synthetic `IWAD` and
`PWAD` headers, ordered directories, and marker-delimited namespaces with
bounded structural diagnostics. A headless consumer observes that contract.
The first Slice 2 package path is also proven manually: the reviewed Doom ZIP
is retained as one Resource Space resource, `DOOM1.WAD` is read transiently
through the archive-derived view, and the WAD observation retains package and
member provenance. The fixed-record map core (`THINGS`, `VERTEXES`,
`LINEDEFS`, `SIDEDEFS`, `SECTORS`, `SEGS`, `SSECTORS`, and `NODES`) now decodes
with bounded input and source-indexed cross-table diagnostics. REJECT and
BLOCKMAP are structurally bounded against decoded sector/linedef observations.
Global PLAYPAL and COLORMAP resources, individual patch column/post streams, and
PNAMES/TEXTURE1/TEXTURE2 catalog records and deterministically clipped wall-texture
pixels now decode as bounded indexed-raster observations without renderer policy.
Sprite frame/rotation names now decode as source observations. An explicit
palette-and-coverage lowerer emits the existing provider-neutral RGBA8 raster
contract with an unspecified color space. A separate headless geometry provider
now retains wall topology, BSP paths/regions, source sector ownership,
floor/ceiling candidates, one-sided walls, two-sided height bands, raw texture
axes, plain extent bindings, pegging-flag evidence, and `F_SKY1` observations.
The corpus consumer can also emit a deterministic source-topology SVG. Neither
geometry path submits rendering work or asserts gameplay behavior.

## Evidence Inventory

| Checklist area | Current evidence | Audit result |
| --- | --- | --- |
| Canonical packages | The reviewed Doom and Heretic source ZIPs, prepared compact packages, inventory, and provenance records are tracked under `corpus/assets/`. | Present; do not treat extracted WADs as canonical artifacts. |
| Preparation | `scripts/prepare-doom-corpus-packages.ps1` verifies source SHA-256 values, produces deterministic compact archives, and excludes DOS executables from internal runs. | Present; script output is not an importer/security substitute. |
| Archive boundary | `archive-provider` provides bounded archive inspection/read operations; `resource-space-archive` retains package fingerprint and logical identity without learning Doom names. `doom-wad-package` composes their read-only selected-member view with `doom-wad-provider`. | First Slice 2 path proven with synthetic CI and a manual reviewed-Doom inspection; do not add Doom concepts to either shared library. |
| WAD code | `corpus/lib/doom-wad-provider` inspects `IWAD`/`PWAD` headers, ordered lump directories, and paired flat/patch/sprite marker ranges. `corpus/hello-wad-inspect` is its headless caller. Neither depends on Doom map, renderer, Resource Space, or gameplay APIs. | Slice 1 complete; no reviewed package is read by tests. |
| Map core | `corpus/lib/doom-map-provider` decodes fixed-record classic Doom map observations and validates linedef, sidedef, seg, subsector, BSP-node, REJECT, and BLOCKMAP structural references. `hello-wad-inspect --map-svg` emits a deterministic top-down `LINEDEFS` diagnostic without presentation semantics. | Slice 3 complete; no geometry or renderer dependency exists. |
| Raster source observations and lowering | `corpus/lib/doom-raster-provider` decodes PLAYPAL RGB entries, COLORMAP index-remapping tables, marker-scoped Doom patch column/post streams, PNAMES/TEXTURE1/TEXTURE2 records, deterministically clipped indexed wall textures, fixed-size flat indices, and classic sprite frame/rotation names with explicit limits. It lowers indexed images only through an explicitly selected palette and coverage rule into `raster-image-corpus`'s provider-neutral RGBA8 contract. | The manual `STARTAN3` observation retains a palette-0 RGBA8 fingerprint; visual artifacts and renderer lighting policy remain absent. |
| Headless geometry observations | `corpus/lib/doom-geometry-provider` consumes decoded map records and emits source-traceable structural candidates only. | Slice 5 is complete: bounded BSP regions, sector ownership, floor/ceiling surfaces, one-sided walls, upper/lower/middle bands, sky classification, source-side winding, and wall U/V/pegging evidence are retained. Slice 5B owns renderer orientation, materials, plane presentation mapping, and static E1M1 submission. |
| Synthetic invalid-input coverage | The provider tests construct Tokimu-authored valid IWAD/PWAD bytes and malformed short-header, unknown-signature, truncated-directory, out-of-bounds, overlapping-range, malformed-name, and limit fixtures. | Present; CI still does not read reviewed game packages. |
| CI | Current workflows do not run a Doom importer or consume Doom data. | Good: CI does not yet depend on reviewed game data. Add synthetic WAD coverage first. |
| Website deployment | The Pages workflow does not list Doom assets or a Doom consumer as inputs. The canonical source ZIPs are nevertheless tracked in Git. | No current deployment inclusion found; repository accessibility and any future site artifact need explicit legal/deployment review. |

The retained manual package observation is
[`2026-08-08-doom-shareware-package-inspection.md`](../../../corpus/lib/doom-wad-package/results/2026-08-08-doom-shareware-package-inspection.md).

## Next Implementation Recommendation

1. Add deterministic structural and visual artifact evidence for representative
   palette, patch, texture, flat, and sprite observations.
2. Preserve the selected-map observation and missing/duplicate/reordered cases
   as synthetic CI evidence; the reviewed package remains manual-only evidence.
3. Keep geometry and renderer lowering out of the decoder until texture, flat,
   and sprite evidence is independently inspectable.

## Architecture Guardrails

- The WAD provider belongs in corpus/outer-ring evidence, not `tokimu-core`,
  `tokimu-runtime`, or the renderer.
- Resource Space receives a logical byte member and provenance; it does not
  receive Doom map, marker, or lump semantics.
- The first implementation must not use permanent extraction as a hidden
  transport mechanism.
- No Doom/Heretic ZIP or extracted WAD may be added to website deployment
  inputs until the checklist's explicit repository/CI/deployment review is
  completed.

## Next Audit Inputs

- Maintainer-provided deployment/legal facts for the reviewed packages.
- Selected size/count/allocation budgets for untrusted/user-supplied WADs.
- The first synthetic fixture and the initial provider diagnostic design.
