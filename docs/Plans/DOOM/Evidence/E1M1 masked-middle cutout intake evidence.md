# E1M1 Masked-Middle Cutout Intake Evidence

Date: 2026-08-10
Scope: Slice 5 of the alpha-policy comparative corpus / AR-0023.  
Status: independent real-caller evidence for ADR-0013 Cutout; Blend remains
unadmitted.

## Reproduction

From the repository root:

```powershell
cargo run -q -p hello-doom-e1m1 --bin hello-doom-e1m1 -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD
```

The reviewed package fingerprint is BLAKE3
`58146f5aa0e14ef38047a79878307344aec821b9f312da6a9208ec08e399660c`.

## Observed source pressure

E1M1 has 13 retained, source-classified two-sided masked-middle observations.
They name four texture definitions:

| Texture | Raster coverage observation |
| --- | --- |
| `BRNBIGC` | 4,546 uncovered pixels |
| `BRNBIGL` | 213 uncovered pixels |
| `BRNBIGR` | 303 uncovered pixels |
| `BROWNGRN` | fully covered |

The source classification produces 26 non-degenerate wall-triangle candidates
(two triangles per observed middle) and no cutout-candidate degeneracy.
`BROWNGRN` is the important control: it is source-classified as a masked
middle despite its fully covered current raster. Therefore alpha/coverage
bytes are not used to decide whether the Doom consumer requests the candidate.

## Caller declaration and renderer crossing

The Doom consumer lowers only those retained source classifications into an
`ExperimentalCutoutWall`. Each candidate carries the generic declaration:

```text
discard at or below RGBA8 alpha 0
depth write true
```

The choice remains Doom-local: patch composition has binary coverage, which
the raster provider lowers to alpha `0` or `255`. The consumer crosses this
specific declaration through ADR-0013's `CategoricalCutout` capability using
`CutoutThreshold(0.0)` and `DiscardAtOrBelow`; `tokimu-render` receives no WAD
term or source-format policy. It is not an original-Doom behavior claim or a
general threshold recommendation.

The opaque static E1M1 draw plan remains unchanged: its 13 masked-middle
observations are still deferred and no experimental candidate is uploaded or
drawn. The candidate path retains source linedef/sidedef/sector/side identity,
and only confirmed `DegenerateTriangle` errors become explicit omissions.
Other lowering failures remain fatal.

### Native first-frame integration

The opt-in native presentation path is reproducible from the repository root:

```powershell
cargo run -p hello-doom-e1m1 --bin static_scene -- `
  corpus/assets/DOOM/packages/doom-shareware-corpus-v1.zip DOOM1.WAD `
  --masked-cutouts
```

On 2026-08-09 it initialized successfully on the available native target:

```text
opaque_draws=1835
cutout_draws=26
cutouts_enabled=true
backend=vulkan
device=discrete-gpu
adapter=AMD Radeon RX 7900 XTX
```

The path initially failed with `MissingMaterial { source_name: "BROWNGRN" }`.
That ordinary corpus-integration defect exposed an important boundary check:
the first opaque preparation intentionally withholds fully covered textures
from its deferred-alpha list, but source-classified masked middles must select
their candidate upload by their retained Doom classification, not by coverage.
The local candidate selector now includes both uncovered and fully covered
selected source textures; a focused regression covers that distinction.

The earlier startup observation retained the pre-admission custom-shader path.
On 2026-08-10 the native E1M1 consumer and browser/WASM engine were migrated
to the admitted generic categorical-cutout constructor; focused E1M1 tests and
the browser engine's `wasm32-unknown-unknown` check pass. The browser visual
observation below verifies the migrated browser path; native visual observation
of that implementation remains separate evidence and is not implied by build
or initialization.

### Browser/WASM visual observation of the admitted implementation

On 2026-08-10, after rebuilding the browser bundle from the migrated Rust/WASM
engine, a reviewer selected the same local reviewed package in the browser
workbench and presented both fixed-camera actions at `960x600`:

| Action | Returned observation | Visual result |
| --- | --- | --- |
| Static opaque E1M1 | `1835 draws; cutouts=false; backend=browser-webgpu; device=other; adapter=` | textured fixed-camera overview presented |
| E1M1 masked cutouts | `1861 draws; cutouts=true; backend=browser-webgpu; device=other; adapter=` | corresponding overview presented through the admitted generic Cutout path, with the expected 26 source-selected draws enabled |

The blank adapter field is retained as browser capability absence, not guessed
metadata. The native/browser comparison is structural and visual at this
bounded scene level: same reviewed package, fixed camera, and an exact 26-draw
delta. It is not a pixel-golden comparison, a general transparency claim, or a
stable renderer cutout admission.

## Result

This establishes independent real cutout pressure beyond the shared synthetic
fixture, while preserving the boundary required by ADR-0013:

```text
Doom source classification and exact threshold choice
    -> explicit generic categorical-cutout declaration
    -> no Doom terms or policy inference in the renderer
```

Native visual capture remains useful, but the browser/WASM rendering of these
candidates is now observed. The admitted browser/WASM implementation is
  visually observed; native/Vulkan observation is retained below. Continuous
  blending remains unadmitted and needs a separate real caller.

### Native visual observation of the admitted implementation

On 2026-08-10, the source-spawn observer visibly presented the migrated
masked-cutout path on the available native target:

```text
opaque_draws=1835
cutout_draws=26
cutouts_enabled=true
camera=source-spawn-observer
backend=vulkan
device=discrete-gpu
adapter=AMD Radeon RX 7900 XTX
```

The fixed view used reviewed `THINGS` record 0 at `(1056, -3616)`, heading
`90`, in sector 38 with a corpus-only vertical-midpoint eye at `y=36`. The
window title reported `1861 draws`. This is a manual native observation of the
same fixed package/camera specification as the browser evidence, not a PNG
golden or a claim that the midpoint is Doom player-height policy.

## Canonical sidedef-ownership pressure

On 2026-08-13, interactive comparison against UZDoom identified a distinct
source-ownership requirement for E1M1 linedef 464. Its right/front sidedef 634
in sector 57 names middle texture `BROWNGRN`; left/back sidedef 635 in sector
62 has no middle texture. The reference presents this surface as solid from
the pit-facing owning side, omits it from the secret-catwalk/back side, and
allows traversal through the two-sided nonspecial line.

The earlier Tokimu path presented the right/front cutout candidate from both
sides because the admitted generic categorical-cutout pipeline is deliberately
two-sided. That is not an alpha-policy defect and does not justify changing the
renderer contract. The Doom consumer now applies source-owning-face selection
after ordinary camera candidate selection, using each lowered wall triangle's
retained owning-side normal. A focused regression proves that the owning side
survives and the reverse side is rejected; incomplete mesh evidence fails open.
Interactive native confirmation on 2026-08-13 established that linedef 464 is
solid from the pit-facing owning side, absent from the secret-catwalk/back
side, and remains traversable. The checklist regression is closed without
changing the generic two-sided categorical-cutout renderer contract.
