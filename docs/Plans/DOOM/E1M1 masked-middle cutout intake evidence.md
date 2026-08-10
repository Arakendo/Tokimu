# E1M1 Masked-Middle Cutout Intake Evidence

Date: 2026-08-09  
Scope: Slice 5 of the alpha-policy comparative corpus / AR-0023.  
Status: corpus-local real-caller evidence; not a renderer admission.

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

## Corpus-local declaration

The Doom consumer lowers only those retained source classifications into an
`ExperimentalCutoutWall`. Each candidate carries the generic declaration:

```text
discard at or below RGBA8 alpha 0
depth write true
```

The choice is local to this experiment. Doom patch composition has binary
coverage, which the raster provider lowers to alpha `0` or `255`; the
declaration is not a general threshold recommendation, an original-Doom
behavior claim, or a `tokimu-render` API.

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

This startup observation establishes that the generic corpus-local shader and
pipeline can present the selected real inputs. A fixed-camera visual capture
and browser/WASM counterpart remain separate Slice 5 evidence; neither is
implied by successful native initialization.

### Browser/WASM visual observation

On 2026-08-09, a reviewer selected the same local reviewed package in the
browser workbench and presented both fixed-camera actions at `960x600`:

| Action | Returned observation | Visual result |
| --- | --- | --- |
| Static opaque E1M1 | `1835 draws; cutouts=false; backend=browser-webgpu; device=other; adapter=` | textured fixed-camera overview presented |
| E1M1 masked cutouts | `1861 draws; cutouts=true; backend=browser-webgpu; device=other; adapter=` | the corresponding overview presented with the 26 corpus-local candidate draws enabled |

The blank adapter field is retained as browser capability absence, not guessed
metadata. The native/browser comparison is structural and visual at this
bounded scene level: same reviewed package, fixed camera, and an exact 26-draw
delta. It is not a pixel-golden comparison, a general transparency claim, or a
stable renderer cutout admission.

## Result

This establishes independent real cutout pressure beyond the shared synthetic
fixture, while preserving the boundary required by AR-0023:

```text
Doom source classification and exact threshold choice
    -> corpus-local generic cutout declaration
    -> no Doom terms or policy inference in the renderer
```

Native visual capture remains useful, but the browser/WASM rendering of these
candidates is now observed. Neither result admits a cutout capability.
Continuous blending remains unadmitted and needs a separate real caller.
