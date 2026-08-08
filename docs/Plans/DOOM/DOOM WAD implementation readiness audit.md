# DOOM WAD Implementation Readiness Audit

| Field | Value |
| --- | --- |
| Status | Initial implementation audit; no checklist item is completed by this report |
| Date | 2026-08-08 |
| Scope | Current repository state against `DOOM WAD Checklist.md` before Slice 1 begins |

## Baseline Finding

The checklist's fixture/provenance phase is complete. No WAD container
provider, Doom semantic importer, geometry lowerer, Doom consumer, or gameplay
implementation currently exists. The next implementable work is Slice 1,
provided it stays corpus-local and provider-shaped.

## Evidence Inventory

| Checklist area | Current evidence | Audit result |
| --- | --- | --- |
| Canonical packages | The reviewed Doom and Heretic source ZIPs, prepared compact packages, inventory, and provenance records are tracked under `corpus/assets/`. | Present; do not treat extracted WADs as canonical artifacts. |
| Preparation | `scripts/prepare-doom-corpus-packages.ps1` verifies source SHA-256 values, produces deterministic compact archives, and excludes DOS executables from internal runs. | Present; script output is not an importer/security substitute. |
| Archive boundary | `archive-provider` provides bounded archive inspection/read operations; `resource-space-archive` retains package fingerprint and logical identity without learning Doom names. | Suitable existing outer-ring seam for Slice 2; do not add Doom concepts to either library. |
| WAD code | Repository scan found no `IWAD`/`PWAD`, lump-directory, linedef, sidedef, sector, or WAD parser implementation outside the plan/assets. | Slice 1 is unstarted. |
| Synthetic invalid-input coverage | No Tokimu-authored synthetic WAD fixture or WAD malformed-input tests exist. | Required before real-package parsing can become CI evidence. |
| CI | Current workflows do not run a Doom importer or consume Doom data. | Good: CI does not yet depend on reviewed game data. Add synthetic WAD coverage first. |
| Website deployment | The Pages workflow does not list Doom assets or a Doom consumer as inputs. The canonical source ZIPs are nevertheless tracked in Git. | No current deployment inclusion found; repository accessibility and any future site artifact need explicit legal/deployment review. |

## Slice 1 Entry Recommendation

1. Create a corpus-local `doom-wad-provider` with a small provider-neutral
   observation model: header kind, ordered lump observations, source identity,
   and structured diagnostics.
2. Start with a Tokimu-authored synthetic IWAD/PWAD fixture and malformed
   fixtures for short headers, truncated directories, unknown signatures,
   out-of-bounds ranges, and overlapping ranges.
3. Make byte, count, entry-size, and total decoded-allocation limits explicit
   inputs. Preserve duplicate names and directory order.
4. Add `hello-wad-inspect` only after the parser emits diagnostics that make
   its bounded observation useful.
5. Defer mounting `DOOM1.WAD` through Resource Space until the synthetic
   provider contract and failure behavior are established.

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
