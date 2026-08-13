# ADR-0013: Caller-Declared Categorical Cutout Surfaces

## Status

Accepted

## Context

ADR-0012 admits supplied UV coordinates and bounded sampler intent for ordinary
textured 3D surfaces, while deliberately deferring alpha interpretation. AR-0023
then held one RGBA8 source constant across opaque, categorical-cutout, and
continuous-blend corpus fixtures. It exercised threshold comparisons, depth
states, caller submission order, native WGPU, browser/WebGPU, DOOM E1M1 masked
middles, and an independent translucent GLB caller.

The evidence does not support a shared `AlphaPolicy` vocabulary. Cutout is a
categorical fragment-coverage decision with a caller-declared threshold and
ordinary depth participation. Continuous blending instead needs explicit
depth-write and submission-order responsibility, whose stable ownership is not
yet earned.

## Decision

Tokimu admits a narrow, provider-neutral categorical-cutout capability for
textured 3D presentation. It is separate from both ordinary opaque texturing
and the renderer's existing blend mechanism.

### Declared cutout coverage

A caller that requests cutout must explicitly declare all of the following:

- a finite threshold in the inclusive range `[0.0, 1.0]`; and
- whether a fragment is discarded **below** that threshold or **at or below**
  it.

There is no implicit threshold, source-format heuristic, or default derived
from texture alpha bytes. The declaration names categorical coverage only; it
is not a general alpha or material-policy enum.

The renderer evaluates the resolved fragment alpha for the declared textured
surface, performs the requested keep/discard decision, and uses ordinary opaque
color output for retained fragments. The public implementation vocabulary must
make malformed thresholds rejectable before backend submission.

### Depth and ordering

Retained cutout fragments participate in ordinary textured-3D depth testing
and depth writing. Cutout does not introduce transparent sorting, a relative
render-order API, a renderer-owned scheduling service, or a relaxed depth
contract. Its result is therefore insensitive to submission order where
ordinary opaque depth behavior would be insensitive.

### Ownership

Applications and corpus consumers own source selection, classification, and
the declared generic cutout comparison and threshold. Asset providers own
encoded-format interpretation and normalized pixels. `tokimu-render` owns
validated provider-neutral cutout declaration, shader/pipeline realization,
and diagnostics. Backends own GPU realization.

The renderer must not receive PNG, GLB, WAD, palette, masked-middle, or other
source-domain terms, and it must not infer cutout merely because a texture has
an alpha channel.

### Continuous blend remains incubating

`BlendMode::AlphaBlend` remains an available renderer/backend mechanism and
corpus-study tool. It is not, by this ADR, a stable general blended-3D
capability. No renderer promise is made about blended draw ordering, sorting,
depth-write defaults, material semantics, or scene orchestration. Those remain
incubating under AR-0023 until independent pressure establishes a bounded
ownership contract.

## Consequences

- DOOM may migrate masked-middle presentation from corpus-local cutout WGSL to
  the admitted generic capability after implementation and verification.
- A caller cannot accidentally obtain cutout by loading alpha-bearing data; it
  must choose and validate the categorical rule explicitly.
- Cutout stays small: it does not admit blended materials, PBR, source asset
  semantics, texture arrays, mip policy, shader authoring, or material graphs.
- The existing blend mechanism remains useful for experiments, but callers must
  not treat it as a stable transparency contract.

## Non-Decisions

This ADR does not decide or admit:

- a public continuous-blend, sorting, or transparency-ordering contract;
- renderer-owned scene ordering or order-independent transparency;
- alpha-to-coverage, premultiplied-alpha, or color-space policy;
- PBR or any broader material model;
- GLB material import, WAD rendering rules, PNG semantics, or automatic source
  classification; or
- cross-target pixel-identical output.

## Verification

Implementation must reopen and complete the ADR-0008 and ADR-0009 full gates
in `docs/Plans/Textured-Presentation/Studies/categorical-cutout-capability-admission.md`. In particular it
must retain typed invalid-threshold behavior, native and browser/WASM evidence,
cutout-versus-opaque depth/order regressions, the E1M1 real-caller migration,
and first/warm-frame performance observations. Blend remains excluded from
this admission gate unless a later decision proposes it independently.

## References

- `docs/ADR/ADR-0001-engine-boundaries.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0008-native-kernel-ring-performance-and-code-quality.md`
- `docs/ADR/ADR-0009-ring-based-verification-failure-containment-and-recovery.md`
- `docs/ADR/ADR-0012-supplied-mesh-texture-coordinates-and-sampling-policy.md`
- `docs/Architectural Reviews/AR-0023-textured-surface-alpha-and-depth-policy.md`
- `docs/Plans/Textured-Presentation/textured-surface-alpha-policy-comparative-corpus.md`
- `docs/Plans/Textured-Presentation/Studies/categorical-cutout-capability-admission.md`
