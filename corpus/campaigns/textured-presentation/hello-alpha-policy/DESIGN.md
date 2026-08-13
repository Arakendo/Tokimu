# Hello Alpha Policy - Corpus Design

| Field | Value |
| --- | --- |
| Status | AR-0023 comparison is complete; ADR-0013 Cutout is migrating through native/browser corpus targets |
| Purpose | Retain the comparison that admitted categorical Cutout while keeping continuous Blend incubating |
| Governing review | [AR-0023 |
| Inputs | First-party exact RGBA8 fixtures and corpus-owned scene/profile requests |
| Outputs | Deterministic fixture, fragment, depth, ordering, validation, and scene observations |
| Durable state | None; reports are derived from explicit inputs |
| Semantic authority | Corpus owns fixture selection; ADR-0013 owns the narrow renderer Cutout capability |
| Execution authority | Headless CPU evaluation only in the initial slices |

## Boundary Assertion

The default headless crate deliberately has no active dependency on `tokimu`,
`tokimu-render`, WGPU, PNG, GLB, WAD, or Doom providers. It freezes source
bytes and compares candidate semantics before a GPU implementation can make one
API shape look inevitable. Its optional `native-visual` target is a separate
corpus executable; it realizes Cutout through ADR-0013 and retains corpus-local
WGSL only for the unadmitted Blend study.

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
- ADR-0013 admits checked caller-declared Cutout threshold/comparison with
  ordinary opaque depth behavior.
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
128/255` Cutout declarations, plus a cutout-over-opaque depth case.

The target invokes `Pipeline::textured_3d_cutout` rather than passing custom
cutout WGSL. It keeps `Textured3d` unchanged and leaves the Blend study's
custom WGSL explicitly outside the admitted capability.

## References

- [Comparative Corpus Plan
- [Fixture Manifest](fixture-manifest.md)
- [AR-0023
