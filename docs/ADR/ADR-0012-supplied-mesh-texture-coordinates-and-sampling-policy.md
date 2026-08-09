# ADR-0012: Supplied Mesh Texture Coordinates and Sampling Policy

## Status

Accepted

## Context

`tokimu-render` already supports textures, materials, mesh resources, and
backend-neutral pipeline declarations. Its former `Texture2d` path derives
coordinates from 2D position, which is useful for that screen-oriented
mechanism but cannot truthfully represent arbitrary textured 3D geometry.

AR-0022 used an intentionally independent corpus: pinned Khronos `Box.glb`
geometry plus first-party PNG inputs. Native and browser/WASM consumers prove
that caller-owned UV coordinates and declared sampler intent compose through
the same provider-neutral renderer seam without importing GLB, PNG, WAD, or
Doom semantics. The corpus also showed that alpha/depth policy is a separate
question; AR-0023 now owns it.

This decision complements ADR-0001's engine boundaries, ADR-0003's ownership
test, ADR-0008's proportional implementation discipline, and ADR-0009's
verification and failure-containment requirements.

## Decision

Tokimu admits the following narrow generic textured-3D presentation contract
to `tokimu-render`.

### Mesh coordinates

A `Mesh` may carry an optional per-vertex UV stream. When supplied, the stream
must have exactly one coordinate for every position; checked construction
rejects a mismatch. An empty stream remains valid for mesh uses that do not
require texture coordinates.

`Textured3d` requires a valid supplied UV stream. It must consume those values
directly and must not derive 3D texture coordinates from position.

### Sampler intent

Materials may declare the bounded generic source-texture sampling vocabulary:

- filtering: `Point` or `Linear`; and
- addressing independently for U and V: `Clamp` or `Repeat`.

The declaration is provider-neutral. A renderer backend realizes it with its
own sampler objects; WGPU types do not enter the public renderer contract.

### Ownership boundary

The renderer owns mesh-stream validation, semantic shader/pipeline
compatibility, material sampler declaration, backend realization, and explicit
diagnostics for malformed admitted inputs.

Applications and corpus consumers own geometry conversion, UV generation,
texture selection, source asset decode, color-space request, sampler choice,
camera, and scene meaning. Asset providers own encoded-format interpretation
and normalized pixel production. The backend owns GPU/browser acquisition and
resource realization.

### Legacy 2D path

The existing position-derived `Texture2d` behavior remains a separate 2D
mechanism. It is not precedent for, fallback from, or an implementation of
textured 3D mesh semantics.

## Consequences

- Ordinary 3D consumers, including future DOOM presentation work, can submit
  caller-owned UV geometry and generic sampler intent without renderer-owned
  asset or domain semantics.
- Missing or malformed UV streams fail through typed construction or semantic
  draw validation rather than backend fallback or panic.
- The contract remains small enough to preserve native/WASM parity evidence and
  avoid a material graph or source-format API.
- Existing untextured and 2D texture callers keep their prior behavior.
- `tokimu-render` must not add GLB material terms, PNG decode types, WAD names,
  Doom pegging/plane rules, palette semantics, or game state to support this
  contract.

## Non-Decisions

This ADR does not admit:

- GLB material import or any encoded-asset interpretation;
- Doom texture composition, palette/`COLORMAP`, pegging, flats, sprites, sky,
  source texel axes, or plane mapping;
- alpha test/cutout threshold, transparent draw ordering, depth policy for
  blended surfaces, or automatic alpha interpretation (AR-0023 owns those
  questions);
- mipmaps, anisotropic filtering, texture arrays, atlases, streaming, normal
  maps, or a material graph; or
- cross-platform pixel-identical rendering guarantees.

## Verification

The admitting evidence is retained by AR-0022 and the completed textured-Box
corpus plan. Future changes to this contract must satisfy the applicable
ADR-0008 performance/code-quality gate and ADR-0009 layered verification gate,
including focused validation and native/WASM evidence when the changed surface
crosses those targets.

## References

- `docs/ADR/ADR-0001-engine-boundaries.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0008-native-kernel-ring-performance-and-code-quality.md`
- `docs/ADR/ADR-0009-ring-based-verification-failure-containment-and-recovery.md`
- `docs/Architectural Reviews/AR-0022-textured-mesh-coordinate-and-sampling-boundary.md`
- `docs/Architectural Reviews/AR-0023-textured-surface-alpha-and-depth-policy.md`
- `docs/Plans/textured-box-glb-png-corpus.md`
- `docs/Plans/DOOM/DOOM WAD Checklist.md`
- `docs/Tokimu Software Design Document.md`
