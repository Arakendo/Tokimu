# Hello Alpha Policy - Corpus Design

| Field | Value |
| --- | --- |
| Status | Slices 0–1 complete; Slice 2 native cutout candidate implemented, awaiting retained visual and browser observations |
| Purpose | Compare opaque, categorical cutout, and continuous blend semantics without pre-admitting renderer vocabulary |
| Governing review | [AR-0023](../../docs/Architectural%20Reviews/AR-0023-textured-surface-alpha-and-depth-policy.md) |
| Inputs | First-party exact RGBA8 fixtures and corpus-owned scene/profile requests |
| Outputs | Deterministic fixture, fragment, depth, ordering, validation, and scene observations |
| Durable state | None; reports are derived from explicit inputs |
| Semantic authority | Corpus-local candidate semantics only; no public renderer precedent |
| Execution authority | Headless CPU evaluation only in the initial slices |

## Boundary Assertion

The default headless crate deliberately has no active dependency on `tokimu`,
`tokimu-render`, WGPU, PNG, GLB, WAD, or Doom providers. It freezes source
bytes and compares candidate semantics before a GPU implementation can make one
API shape look inevitable. Its optional `native-visual` target is a separate
corpus executable; it realizes only frozen cases through existing custom-WGSL
and ordinary textured-mesh seams.

```text
exact first-party RGBA8
        +
explicit corpus profile/depth/order request
        |
        v
headless semantic observation
```

Fixture names never select behavior. The same mixed-alpha bytes must be
evaluated under opaque, cutout, and blend requests. Cutout comparison and
threshold are explicit. Blend ordering is explicit caller input; a missing or
empty order is rejected rather than inferred.

## Current Baseline

- Opaque textured 3D presentation is demonstrated by AR-0022 and ADR-0012.
- WGPU source-alpha blending exists as a mechanism.
- The current blend-plus-depth-write combination is not an admitted general
  transparent-surface contract.
- Cutout has no admitted threshold or comparison vocabulary.
- This crate's `StudyProfile` and related types are experimental corpus terms,
  not proposed public names.

## Frozen Scene Rules

- Viewport: 960 by 600.
- Camera: fixed identity study camera; later visual consumers must record their
  exact projection/view matrices without altering scene identities.
- Geometry: ordinary supplied-UV quads with fixed draw IDs and transforms.
- Background: opaque reference quad.
- Variable dimensions: declared profile, threshold comparison, depth-write
  choice, and caller submission order only.
- Target-specific code may realize a frozen case but may not reinterpret it.

## Failure Posture

Non-finite or out-of-range thresholds, malformed RGBA8 payloads, missing blend
ordering, empty order, and duplicate draw identities return typed failures.
There is no fallback to opaque, a magic threshold, depth-write default, or a
renderer-generated sort.

## Experimental Native Cutout Target

`cargo run -p hello-alpha-policy --features native-visual --bin native_scene`
opens the Slice 2 comparison scene. It displays the same `mixed-alpha` source
under explicit opaque, `discard below 128/255`, and `discard at-or-below
128/255` custom-shader candidates, plus a cutout-over-opaque depth case.

The two discard shaders are corpus source strings selected by the executable;
they are not `tokimu-render` pipeline kinds, do not add a public threshold
parameter, and do not change `Textured3d` defaults. The target is evidence that
the existing custom-WGSL mechanism can realize the frozen comparison, not a
decision that custom shader source is the eventual cutout contract.

## References

- [Comparative Corpus Plan](../../docs/Plans/textured-surface-alpha-policy-comparative-corpus.md)
- [Fixture Manifest](fixture-manifest.md)
- [AR-0023](../../docs/Architectural%20Reviews/AR-0023-textured-surface-alpha-and-depth-policy.md)
