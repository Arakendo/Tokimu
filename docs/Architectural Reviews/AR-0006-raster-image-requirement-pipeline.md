# AR-0006: Raster Image Requirement Pipeline

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-07-30 |
| Last reviewed | 2026-07-30 |
| Scope | Foundational asset, image, presentation, and renderer capability boundary |
| Trigger | PNG, JPEG, and BMP corpus providers now converge on one decoded-image and sampled-texture path without source-format types leaking into materials or shaders |
| Related ADRs | ADR-0001, ADR-0003, ADR-0004, ADR-0005 |
| Related evidence | Raster image corpus, `hello-raster-image`, asset-resolution tests, shader/material presentation plan, native renderer |
| Admission exception | None |

## Architectural Question

Where must raster-image requirements become provider-neutral, and which layer
owns each semantic and execution decision from encoded image bytes through
asset identity, texture preparation, material binding, sampler policy, and
shader sampling?

This review does not assume that the current `DecodedImage`, `TextureUse`, or
renderer `Texture` types are the final public contracts. It studies the
ownership boundary before any image capability, texture-requirement model, or
new crate is admitted.

## Context

Tokimu's bounded PNG, JPEG, and BMP corpus providers now produce the same
normalized result shape. The native `hello-raster-image` consumer carries that
result through explicit color-texture preparation, renderer upload, a material
texture slot, the renderer-owned default sampler, alpha blending, and shader
sampling:

```text
PNG / JPEG / BMP provider
        ↓
DecodedImage
        ↓
ColorSrgb texture preparation
        ↓
renderer texture resource
        ↓
material texture slot
        ↓
shader sampling
```

No PNG, JPEG, or BMP object crosses into the material or shader contract. That
convergence is useful evidence, but it exposes an ownership question hidden by
the overloaded word "texture":

- an application may require an image with a particular interpretation;
- an asset may identify decoded or still-encoded image content;
- presentation may require a sampled color or data resource;
- a renderer may allocate a backend texture, view, and sampler;
- a shader may declare a compatible sampled binding.

Those are related states, not one object with one owner.

The current corpus implementation deliberately supports only top-down RGBA8
color preparation targeting `Rgba8UnormSrgb`. Linear data textures, HDR,
multiple planes, compressed GPU formats, mip policy, general sampler control,
color conversion, and framebuffer equivalence remain unproven.

## Trigger And Evidence

- Corpus providers: bounded PNG, baseline JPEG, and BMP providers produce one
  provider-neutral `DecodedImage` shape and preserve format observations below
  that boundary.
- Automated tests: asset-resolution tests register PNG, JPEG, and BMP results
  behind the same opaque `AssetHandle<DecodedImage>` and prepare the same
  material texture-slot shape.
- Structural artifacts: the raster runner emits decoded-image and pre-GPU
  texture-preparation artifacts with dimensions, color and alpha observations,
  orientation, target format, and deterministic fingerprints.
- Native corpus evidence: `hello-raster-image` uploads five fixed fixtures and
  samples each through the same `Texture2d` pipeline. Source and translucent
  inspection materials reuse the same immutable uploaded texture.
- Shader evidence: the built-in texture shader receives a material texture and
  sampler binding and has no encoded source-format input.
- Diagnostic evidence: decode and texture-preparation failures are explicit;
  unsupported linear/data texture intent stops before renderer upload.
- Independent consumer pressure: the selected Khronos `BoxTextured` case
  preserves an external PNG dependency through asset identity, while the
  native raster viewer consumes decoded images directly.
- Missing evidence:
  - PNG, JPEG, and BMP encodings of equivalent source pixels compared after
    the same render path;
  - deterministic GPU framebuffer capture or cross-backend comparison;
  - a WASM consumer that uploads and samples through the same semantic path;
  - linear/data textures such as normal, mask, or height data;
  - HDR, higher precision, multi-plane, or compressed texture requirements;
  - explicit sampler and mip requirements;
  - production asset dependency and residency policy;
  - evidence that top-down RGBA8 is a stable canonical image contract rather
    than the first bounded interchange profile.

## Ownership Analysis

### Encoded formats and providers

PNG, JPEG, BMP, and future formats own syntax, profile validation, provider
selection, and source-specific diagnostics. Provider-native decoder objects
must stop below the provider-neutral image boundary.

They must not own asset identity, material slots, shader bindings, GPU formats,
samplers, residency, or application presentation policy.

### Provider-neutral image meaning

The candidate shared meaning includes:

- dimensions;
- normalized pixel orientation;
- pixel layout and precision;
- color interpretation;
- alpha interpretation;
- bounded metadata observations;
- deterministic diagnostics and resource limits.

Today this meaning incubates in `raster-image-corpus::DecodedImage`. The
evidence supports continued reuse of that boundary, but does not yet prove
that its RGBA8 representation should be frozen as a first-party Tokimu image
capability.

### Asset identity and resolution

`tokimu-assets` owns stable identity, lifecycle, and dependency observations.
It may identify encoded source content, decoded image content, or a future
prepared presentation resource without redefining image semantics.

Asset identity must not imply GPU residency or make a source path part of the
material or shader contract.

### Presentation requirement

Applications and material definitions own intent such as:

- sampled color image;
- sampled linear/data image;
- required alpha or coverage policy;
- required filtering, addressing, mip, or comparison behavior when those
  semantics are admitted.

The requirement should be provider-neutral and should not contain decoder or
backend objects. Whether all sampler and mip choices belong to the material,
shader interface, pipeline, or a separate sampled-resource requirement remains
under review.

### Texture preparation

Texture preparation reconciles decoded-image meaning with a declared
presentation requirement and an execution profile. It may validate or perform
orientation normalization, color conversion, pixel-format conversion, mip
generation, or compression selection when those capabilities are admitted.

The current corpus bridge only validates top-down RGBA8 `ColorSrgb` input and
records `Rgba8UnormSrgb` as the current renderer target. That is evidence of a
handoff, not proof that preparation belongs wholly to the raster provider or
renderer.

### Renderer and backend

The renderer owns execution-facing texture handles and compatible draw
contracts. The backend owns GPU allocation, views, uploads, backend formats,
sampler objects, residency mechanisms, synchronization, and cache lifetime.

Neither layer owns source-format interpretation or application image intent.
A backend may reject an unsupported requirement explicitly; it must not
silently reinterpret it.

### Shader

A shader owns its declared sampled binding and how sampled values contribute
to fragment output. It may require a color or data interpretation through a
Tokimu-owned binding contract.

A shader must not observe PNG, JPEG, BMP, source paths, decoder state, or asset
loading mechanisms.

## Dependency Direction

```text
Current incubation:

encoded bytes
    -> raster-image-corpus providers
    -> DecodedImage
    -> corpus texture preparation
    -> tokimu-render::Texture
    -> Material texture handle
    -> renderer-owned texture/sampler binding
    -> shader

Candidate stable direction:

application/material requirement ----┐
                                     v
encoded provider -> image semantics -> requirement resolution/preparation
                                     |
                                     v
                              renderer texture request
                                     |
                                     v
                            backend texture + sampler
                                     |
                                     v
                              compatible shader binding
```

Rules under review:

- source-format providers may depend on provider-neutral image contracts, but
  image contracts do not depend on source formats;
- asset identity may refer to image content, but image semantics do not depend
  on filesystem, browser, or package mechanisms;
- presentation requirements may consume image meaning, but do not depend on
  decoder-native or backend-native types;
- renderer adapters consume resolved requirements and own execution resources;
- shaders consume compatible bindings and never source-format identity;
- `tokimu-core` and `tokimu-runtime` do not acquire image codecs, GPU texture
  types, or sampler mechanisms.

## Alternatives Considered

### A: Renderer Owns Everything After Decode

- Benefits: direct path from pixels to GPU resources and few intermediate
  contracts.
- Costs: renderer must infer color, alpha, orientation, data use, and sampling
  intent from a decoded buffer.
- Failure mode: backend convenience becomes presentation meaning, headless
  preparation becomes difficult, and different renderers reinterpret the same
  image differently.

### B: Image Capability Owns Decoding Through GPU Preparation

- Benefits: one apparent place for image conversion and texture preparation.
- Costs: combines source-format semantics, color and pixel conversion,
  presentation requirements, and backend execution.
- Failure mode: a broad image subsystem becomes the accidental owner of
  materials, samplers, GPU formats, and residency.

### C: Material Or Shader Owns Encoded Image Sources

- Benefits: authoring APIs can name files directly beside shader parameters.
- Costs: source acquisition, decoding, asset identity, and presentation become
  coupled.
- Failure mode: materials and shaders branch on PNG/JPEG/BMP and browser/native
  loading behavior diverges.

### D: Separate Image Meaning, Presentation Requirement, And Execution

- Benefits: preserves headless decoded-image evidence, makes intent explicit,
  and keeps GPU mechanisms replaceable.
- Costs: introduces more handoff contracts and requires deterministic
  compatibility diagnostics.
- Failure mode: premature abstraction freezes today's RGBA8 and default-sampler
  proof as a universal requirement model.

### E: Continue Corpus-Side Incubation Without A Shared Contract

- Benefits: maximum reversibility while linear textures, WASM upload, and
  equivalent-source visual comparison are still missing.
- Costs: asset, material, and renderer consumers may begin to duplicate
  requirement vocabulary.
- Failure mode: duplicated local bridges become incompatible before the review
  reaches a disposition.

## Initial Findings

The current evidence supports these provisional findings:

1. Encoded image format identity must stop before material and shader APIs.
2. Provider-neutral decoded-image meaning is independently useful for assets,
   headless diagnostics, CPU artifacts, and renderer preparation.
3. Application and material image requirements are semantic intent; GPU
   allocation and sampler objects are renderer/backend mechanisms.
4. A renderer texture resource, an image asset, a sampled binding, and a
   texture requirement are distinct concepts even when one implementation
   currently connects them directly.
5. Alpha and color behavior cannot be inferred solely from the encoded format.
6. Structural decode and preparation artifacts remain authoritative for those
   stages; a native screenshot does not prove framebuffer equivalence.
7. The current `ColorSrgb -> Rgba8UnormSrgb` path is a valid bounded profile,
   not evidence that every raster image is color RGBA8.

The evidence does not yet establish:

- the final public shape or package location of provider-neutral image
  semantics;
- whether texture preparation is one capability or a collaboration between
  image and presentation capabilities;
- ownership of sampler, addressing, mip, and comparison requirements;
- a universal texture-cache or residency model;
- an ADR-worthy commitment to a new crate;
- promotion of raster decoding, image semantics, or texture requirements into
  the kernel.

## Cross-Capability Observation

This review may be exposing a broader pattern:

```text
application intent
        |
        v
provider-neutral requirement
        |
        v
requirement resolution
        |
        v
execution mechanism
```

Raster images currently provide evidence for that pattern through image
interpretation, texture preparation, renderer allocation, and shader sampling.
The pattern is an observation, not an admitted Tokimu abstraction. This review
must not introduce a universal `Requirement` type, service, or crate from one
capability's evidence.

Future capabilities may test whether the pattern generalizes. Audio is a
particularly useful comparison because an application could request semantic
playback behavior while an audio capability resolves clips, channels, spatial
properties, and device execution. Tokimu has not implemented or corpus-tested
audio, so audio contributes no evidence to this review yet. It is a future
falsification case: audio may reinforce the pattern, require materially
different boundaries, or show that the apparent commonality is only
terminological.

The same caution applies to fonts, compute, networking, and other possible
consumers. Similar-looking pipelines do not justify shared ownership until
independent corpus pressure demonstrates a stable common contract.

## Disposition

Under Review. Preserve the current provider-neutral handoffs and continue
incubating implementation in the corpus. Do not create an ADR or promote
`DecodedImage`, `TextureUse`, or a generalized texture requirement until the
named missing evidence tests the boundary beyond the first sRGB color path.

## Consequences

Current PNG, JPEG, BMP, asset, material, and native shader work may proceed
through the bounded corpus bridge. New consumers should keep source-format,
image, presentation-requirement, and backend-resource terminology distinct and
emit failures at the owning stage.

The review deliberately accepts temporary duplication or corpus-local types
over prematurely stabilizing a universal image or sampled-resource model.
Renderer backends remain free to optimize upload, caching, and sampler
allocation as long as they preserve explicit semantic requirements.

## Required Follow-Up

- [x] Open the Architectural Review and record initial evidence.
- [x] Preserve PNG, JPEG, and BMP convergence through one provider-neutral
      decoded-image and sampled-color path.
- [x] Emit separate decoded-image, pre-GPU preparation, and shader-contract
      artifacts without claiming framebuffer equivalence.
- [ ] Add equivalent-source PNG, JPEG, and BMP cases with a reviewed lossy
      comparison policy before comparing rendered output.
- [ ] Add deterministic framebuffer or backend capture evidence as a separate
      execution artifact.
- [ ] Exercise the same image requirement through a WASM texture-upload and
      shader-sampling consumer.
- [ ] Add one linear/data texture consumer, such as a normal or mask texture.
- [ ] Define and test explicit sampler and mip requirements without backend
      types.
- [ ] Test whether a higher-precision or non-RGBA8 image invalidates the current
      decoded-image contract.
- [ ] Reassess whether stable image semantics, requirement resolution, and
      renderer texture requests warrant one capability, multiple capabilities,
      or no new first-party package.
- [ ] When audio work receives a concrete trigger, compare its
      intent-resolution-execution boundary against this review without assuming
      a shared requirement abstraction.

## Reopening Triggers

This review is already active. After a disposition, reopen it when:

- a second non-corpus consumer requires the same provider-neutral image
  semantics;
- a WASM or alternate renderer cannot preserve the selected requirement;
- a data, HDR, multi-plane, or compressed image cannot pass through the current
  decoded-image boundary honestly;
- materials or shaders begin receiving source-format details;
- sampler or mip policy is duplicated incompatibly across consumers;
- backend resource types leak into image or author-facing contracts;
- deterministic framebuffer evidence contradicts structural preparation
  evidence;
- a simpler ownership decomposition becomes available.

## Review History

### Cycle 1 -- 2026-07-30

- Status entering review: Proposed
- New evidence: bounded PNG/JPEG/BMP convergence, opaque decoded-image asset
  identity, pre-GPU texture artifacts, native renderer upload, immutable source
  and inspection materials, and source-format-neutral shader sampling.
- Participants or reviewers: Arakendo, Monday conversation review, Codex
  working review
- Findings: format identity stops cleanly below presentation; image meaning,
  presentation requirements, and backend texture resources appear distinct;
  only the first sRGB RGBA8 execution profile is proven.
- Disposition: Under Review
- Resulting ADR or documentation change: AR-0006 opened; no ADR yet.

### Cycle 2 -- 2026-07-30

- Status entering review: Under Review
- New evidence: no new runtime evidence; reviewer feedback identified semantic
  requirement propagation as a possible cross-capability pattern.
- Participants or reviewers: Arakendo, Monday conversation review, Codex
  working review
- Findings: raster evidence may reveal a reusable
  intent-to-requirement-to-resolution-to-execution pattern, but untouched audio
  and other future capabilities cannot yet support that conclusion.
- Disposition: Under Review
- Resulting ADR or documentation change: recorded the broader pattern as a
  watch item and audio as a future falsification case; no shared `Requirement`
  abstraction and no ADR admitted.

### Cycle 3 -- 2026-08-01

- Status entering review: Under Review
- New evidence: `tokimu-render` now distinguishes explicit RGBA8 linear and
  sRGB creation from whole-resource in-place writes. The
  `hello-streaming-texture` corpus creates one texture and material, writes
  deterministic application-owned frames through the same handle, and checks
  allocation, replacement, and write counters without DMX or Spout
  dependencies. `hello-raster-image` also consumes the explicit immutable
  creation path. `hello-texture-color-space` constructs identical encoded
  RGBA8 bytes under distinct linear and sRGB descriptors. Both focused texture
  corpus packages compile for native and `wasm32-unknown-unknown`; their
  native visual comparisons remain manual evidence, not a browser proof or
  color-management guarantee. The streaming corpus also exposes an explicit
  1920 by 1080 native stress profile while preserving its small default case.
  A bounded native run completed 300 stress-profile updates with one texture
  allocation, zero replacements, and 301 lifetime writes.
- Participants or reviewers: Arakendo, Codex working review
- Findings: mutable pixel contents do not require mutable texture identity;
  stable texture, view, handle, and material binding identity belong to the
  renderer resource lifecycle. Linear versus sRGB interpretation must be
  explicit before backend allocation. This evidence still does not establish
  a generalized image requirement, sampler/mip model, resize contract, or
  final output color policy. Final output interpretation remains a consumer
  decision; this renderer evidence does not choose monitor, browser, Spout,
  OBS, HDR, or transfer-function policy.
- Disposition: Under Review
- Resulting ADR or documentation change: bounded renderer creation/update
  contracts and texture lifecycle counters admitted for incubation; no new
  crate, kernel ownership, or generalized `Requirement` abstraction.

## References

- `docs/Conversations/AR - Raster Image Requirement Pipeline.md`
- `docs/Libraries/raster-image-corpus-testing.md`
- `docs/Plans/Standalone/typescript-shader-material-presentation-control.md`
- `corpus/lib/raster-image-corpus/DESIGN.md`
- `corpus/lib/raster-image-corpus/src/model.rs`
- `corpus/lib/raster-image-corpus/src/asset.rs`
- `corpus/lib/raster-image-corpus/tests/asset_resolution.rs`
- `corpus/focused/data-interchange/hello-raster-image/DESIGN.md`
- `corpus/focused/data-interchange/hello-raster-image/src/main.rs`
- `corpus/campaigns/textured-presentation/hello-streaming-texture/DESIGN.md`
- `corpus/campaigns/textured-presentation/hello-streaming-texture/src/main.rs`
- `crates/tokimu-assets`
- `crates/tokimu-render`
- `docs/Tokimu Software Design Document.md`
- `docs/ADR/ADR-0001-engine-boundaries.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0004-foundational-presentation-text-and-icons.md`
- `docs/ADR/ADR-0005-admission-evidence-and-maintainer-exceptions.md`
