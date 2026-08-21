# AR-0033 Slice 0 Console Whole-Set Control Evidence

Date: 2026-08-20

## Scope

This evidence inventories the presentation resources affected by one Doom
debug-console edit and models AR-0033 Alternative A against the retained E1M1
browser working-model inventory. It is an accounting and correctness control,
not an in-set update implementation, provider timing measurement, or physical
GPU reclamation observation.

## Edit Inventory

The current native console raster path establishes the concrete first caller:

- one RGBA8 console texture changes on prompt or transcript edits;
- the textured quad mesh remains stable;
- the material-to-texture dependency remains stable;
- the texture pipeline and orthographic camera remain stable;
- the open-console draw command topology remains stable;
- the raster extent at the browser workbench's 960-pixel canvas width is
  `960 x 264`, or 1,013,760 source RGBA8 bytes.

`DoomDebugConsole::raster_dimensions` now exposes that fixed composition extent
without invoking a font provider. Rasterization uses the same function, so the
inventory and presentation path cannot silently disagree about dimensions.

The native implementation currently re-uploads the material after rebuilding
the texture. That is a mechanism observation, not evidence that the material's
semantic contents changed. Slice 0 therefore counts one changing texture and
does not manufacture a changing-material requirement.

## Alternative-A Accounting Control

The retained E1M1 browser observation used by the focused regression contains:

```text
authoritative map resources
    meshes       1,117
    textures         55
    materials        56
    pipelines         7
    cameras           1
    commands       2,068
    mesh bytes   284,736
    texture bytes 1,879,040

modeled map-plus-console candidate
    meshes       1,118
    textures         56
    materials        57
    pipelines         8
    cameras           2
    commands       2,069
```

For one changing logical texture, whole-set replacement would stage 1,241
logical persistent resources, a `1,241x` resource-count amplification. The
modeled source payload is 3,177,536 bytes versus 1,013,760 changed console
bytes, approximately `3.13x`, before counting WAD reopening, map decoding,
geometry lowering, pipeline work, command construction, allocation overhead,
or provider-private copies. It would also regenerate 2,069 commands and turn
over the resource-set identity once per edit even though 2,068 map commands
are unchanged.

This does not falsify Alternative A's correctness. ADR-0018 already supplies
the known-safe whole-set failure and commit invariant. It shows that the unit
is structurally disproportionate for this caller and gives Alternatives B-D a
bounded control to beat.

## Executable Evidence

- `doom-ts-boundary-workbench-engine` exposes
  `observe_ar0033_console_whole_set_control()` only as a corpus-private
  accounting observation against the currently retained map and resource-set
  identity.
- The browser workbench retains this accounting observation beside the live
  **Exercise ADR-0019 console** control after a working map is retained.
- The observation labels provider execution `not-run-accounting-model-only`,
  physical reclamation `unobserved`, and its authority
  `corpus-private-accounting-not-update-contract-or-performance-proof`.
- Focused tests prove the one-changing-texture inventory and the E1M1
  amplification arithmetic.

## Validation

- `cargo test -p hello-doom-e1m1 --lib`: 95 passed.
- `cargo test -p doom-ts-boundary-workbench-engine`: 8 passed.
- `pwsh -NoProfile -File .\build.ps1` from the browser workbench: passed;
  emitted startup payload 6,228,905 bytes under the 12,582,912-byte limit.
- The first `npm run typecheck` was invoked from the workspace root and failed
  because that directory has no `package.json`. Running the workbench build
  from its owning directory regenerated the WASM bindings and completed the
  TypeScript build. This was an ordinary harness invocation error.

## Disposition

AR-0033 Slice 0's inventory and accounting control are complete. Actual CPU
timing and provider diagnostics for executing the whole-set control remain
open and should be captured beside the first real-provider candidate so both
use the same workload and observation boundary. Alternative A remains the
correctness control, while its modeled amplification supports advancing
corpus-private semantic shadows for scoped existing-identity replacement, an
explicitly dynamic resource class, and submission-local data. No stable
renderer update surface or provider escape hatch was added.
