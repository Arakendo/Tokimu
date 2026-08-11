# Browser EXITSIGN Observation — 2026-08-10

| Field | Observation |
| --- | --- |
| Target | Browser/WASM WebGPU |
| Fixture | DOOM TypeScript boundary workbench `Inspect EXITSIGN` |
| Package | `doom-shareware-corpus-v1.zip` / `DOOM1.WAD` / E1M1 |
| Camera | Corpus-local canonical face inspection, linedef 342 |
| Presentation | `1835` opaque draws, no frustum selection |
| Status | Manually observed; canonical `EXIT` lettering reads correctly |

The browser returned:

```text
browser first frame presented: 1835 draws;
candidates=1835; rejected=0;
opaque=1835/1835; cutouts=0/0;
frustum_aabb=false;
camera=canonical-exitsign;
backend=browser-webgpu;
device=other; adapter=; canvas=960x600
```

The fixed camera sits on the supplied owning-side normal of right/front
linedef 342 and looks at that face's geometric center. The maintainer confirmed
that `EXIT` reads correctly in the presented image.

Two earlier fixture-construction attempts correctly rejected zero summed
normals. First, both opposed sign housings were averaged; then all four faces
of one rectangular housing were averaged. Retained centers and normals showed
that linedefs 342–345 face `+Z`, `-Z`, `+X`, and `-X`. Selecting one face made
the visual question unambiguous without changing geometry, UVs, the renderer,
or the source provider.

This is browser conformance evidence for the current Doom source mapping. It
does not admit a generic sign-inspection camera or turn Doom sidedefs into
renderer vocabulary.
