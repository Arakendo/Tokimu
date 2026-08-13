# Streaming RGBA8 Texture Updates

## Status

In progress from incoming change request `TOKIMU-CR-001`. The bounded renderer
contract, lifecycle counters, immutable raster caller, and focused streaming
corpus are implemented. Native visual/stress evidence, browser execution, and
the downstream DMX/Spout handoff remain open.

The first implementation should remain a bounded `tokimu-render` capability.
It supplies additional evidence to
[`AR-0006`](../../Architectural%20Reviews/AR-0006-raster-image-requirement-pipeline.md),
which remains under review.

## Source Request

- [TOKIMU-CR-001: Streaming RGBA8 Texture Updates](../../../.workbench/Change%20Requests/DMX%20Project/tokimu-001-streaming-texture.md)
- [Tokimu Grid Output Design](../../../.workbench/Change%20Requests/DMX%20Project/design-document.md)
- Consumer: `DMX Project/apps/visual-output`
- Blocked consumer work: native Tokimu preview and Spout2 output from one
  application-owned RGBA8 pixel frame

The change request is evidence and a concrete consumer need. It is not an ADR
and does not override Tokimu's accepted ownership boundaries.

## Purpose

Tokimu can currently create and upload an RGBA8 texture, but
`WgpuBackend::upload_texture` creates a new backend texture and view every time
it is called. It also hardcodes `Rgba8UnormSrgb`.

That behavior is adequate for immutable corpus fixtures. It is not an honest
contract for a producer that emits a complete 1920 by 1080 RGBA8 frame at 60
Hz. Recreating the resource would replace the `TextureHandle` entry, invalidate
the view used by an already uploaded material, and hide resource churn behind
an operation named only as an upload.

The first target capability is deliberately narrow:

```text
application-owned RGBA8 frame
        |
        v
explicit texture creation
        |
        v
stable TextureHandle + texture + view + material binding
        |
        v
whole-image writes into the existing texture
```

Creation establishes resource identity and color interpretation. Updates copy
new bytes into that resource without replacing its identity.

## Architectural Assessment

### Current finding

This request belongs to renderer resource lifecycle, not the kernel and not a
source image provider.

- The application owns DMX channels, generators, serialization, scheduling,
  frame arbitration, and the immutable `PixelFrame` snapshot.
- Raster providers own PNG, JPEG, BMP, or other encoded-format decoding.
- Presentation intent chooses whether RGBA8 bytes represent sRGB color or
  linear data before backend allocation.
- `tokimu-render` owns `TextureHandle`, backend allocation, whole-resource
  writes, compatible views and samplers, execution errors, and renderer stats.
- Spout and OBS remain consumer/platform integrations outside Tokimu.

No code from the DMX project, HNode, Spout, or OBS may enter Tokimu's
dependency graph.

### Relationship to AR-0006

AR-0006 distinguishes image meaning, presentation requirements, renderer
texture requests, and backend resources. This plan does not collapse them.

The work adds two useful forms of evidence:

1. explicit linear versus sRGB interpretation at renderer allocation;
2. mutable backend resource contents under stable renderer identity.

It does not settle the larger AR questions around decoded-image ownership,
sampler and mip requirements, higher-precision formats, compressed textures,
or a universal requirement model. If implementation requires such a model,
pause this plan and update AR-0006 before stabilizing the API.

### Relationship to ADR-0007

Texture operation counts are renderer-produced measurements. They do not make
texture policy kernel-owned.

The renderer may expose bounded frame and lifetime counters. Applications may
apply budgets and route sustained transitions through the kernel diagnostic
stream under ADR-0007. A successful per-frame write must not emit one kernel
diagnostic per frame.

## Current Implementation Baseline

The repository currently has:

- an opaque `TextureHandle` in `tokimu-render`;
- an owned `Texture { width, height, rgba8 }` value;
- `WgpuBackend::upload_texture`, which always creates a new
  `Rgba8UnormSrgb` texture and view;
- `GpuTexture`, which stores only the backend texture and view;
- materials that retain an `Arc<TextureView>` and a sampler when uploaded;
- frame and lifetime statistics for meshes, pipelines, bindings, and uniform
  writes, but not textures;
- native and WASM-capable wgpu backend construction;
- immutable texture consumers in raster and font corpus entries.

The compatibility upload callers recorded at implementation start were:

- `corpus/focused/data-interchange/hello-raster-image`;
- `corpus/ui/hello-ui-font`;
- `corpus/ui/hello-ui-font2`;
- `corpus/ui/hello-ui-glyph-corpus`.

All recorded immutable callers now use explicit sRGB creation. The legacy
`upload_texture` method remains only as a documented compatibility bridge for
external callers until its final API disposition is decided.

The important existing behavior is that uploading a material captures the
current texture view. Replacing the texture map entry later does not update
that material. An in-place queue write, however, preserves the view already
held by the material.

## Required Contract

### Creation

The public renderer API must create one two-dimensional RGBA8 texture with:

- non-zero width and height;
- checked `width * height * 4` payload sizing;
- explicit linear or sRGB interpretation;
- one mip level and one array layer for the initial profile;
- texture-binding and copy-destination usage;
- point filtering and clamp addressing preserved for this consumer profile;
- an explicit result when the requested handle already exists.

The exact public names remain an implementation decision. A representative
shape is:

```rust
pub enum Rgba8TextureColorSpace {
    Linear,
    Srgb,
}

pub struct Rgba8TextureDescriptor {
    pub width: u32,
    pub height: u32,
    pub color_space: Rgba8TextureColorSpace,
}

pub fn create_texture_rgba8(
    &mut self,
    handle: TextureHandle,
    descriptor: Rgba8TextureDescriptor,
    rgba8: &[u8],
) -> Result<(), WgpuBackendError>;
```

Borrowed upload data avoids requiring callers to clone a complete video frame
only to cross the renderer API.

### Update

A whole-resource update must:

- look up an existing `TextureHandle`;
- validate width, height, and payload size against stored metadata;
- write the complete payload into the existing backend texture;
- preserve the backend texture, view, sampler, handle, and dependent material
  binding identity;
- return a typed error rather than panic or silently recreate a resource.

A representative shape is:

```rust
pub fn update_texture_rgba8(
    &mut self,
    handle: TextureHandle,
    width: u32,
    height: u32,
    rgba8: &[u8],
) -> Result<(), WgpuBackendError>;
```

Passing dimensions on update is intentional in the first API. It permits the
renderer to reject accidental producer resize instead of interpreting the
payload against stale metadata.

### Compatibility upload

The existing `upload_texture` operation must not remain semantically
ambiguous. During migration it may be retained as a documented create-or-
replace compatibility operation, but it must:

- preserve its current sRGB behavior for existing callers;
- report whether it allocated or replaced a resource;
- not be used by the streaming corpus or DMX consumer;
- have an explicit migration or removal decision before this plan closes.

Creation and replacement should not share a name if callers can accidentally
destroy stable resource identity.

## Failure Semantics

The bounded public errors must distinguish at least:

- missing texture handle;
- duplicate handle during create;
- zero or invalid dimensions;
- arithmetic overflow while computing expected byte count;
- payload length mismatch;
- width or height mismatch during update;
- unsupported color interpretation or backend format, if encountered.

Validation should happen before backend allocation or queue submission where
possible. Failures must leave the previous texture resource and its contents
valid.

## Statistics Contract

Renderer frame and lifetime statistics should distinguish:

- texture allocations;
- texture replacements;
- texture writes;
- optionally, texture bytes written if the value remains cheap and bounded.

The initial creation with data is expected to record one allocation and one
write. An explicit replacement records an allocation, a replacement, and one
write. A steady-state update records only one write.

After frame warm-up, repeated updates must not increase:

- texture allocations;
- texture replacements;
- material or instance binding allocations caused by the texture update;
- pipeline creations;
- mesh uploads or replacements.

These counters report renderer-observed work. They do not claim GPU completion
time, PCIe transfer cost, frame pacing, or Spout performance.

## Implementation Slices

### Slice 0: Intake, Baseline, And Review Linkage

#### Deliverables

- [x] Record the existing `upload_texture` behavior and all current callers.
- [x] Add the incoming change request and DMX design as non-authoritative
      evidence references.
- [x] Record this work as a new AR-0006 evidence cycle without changing its
      disposition.
- [x] Freeze the current point-filtering and clamp-addressing behavior in a
      test or explicit descriptor before refactoring.
- [x] Decide how the compatibility `upload_texture` method will be migrated.

#### Acceptance criteria

- [x] No accepted ADR is contradicted.
- [x] The plan names every existing caller that may observe changed behavior.
- [x] The consumer-local supporting review is not treated as Tokimu authority.
- [x] No new crate, kernel dependency, or generalized requirement type is
      introduced in this slice.

### Slice 1: Add Explicit RGBA8 Texture Semantics

#### Deliverables

- [x] Add a provider-neutral linear/sRGB RGBA8 color interpretation to
      `tokimu-render`.
- [x] Add a bounded texture creation descriptor or equivalent arguments.
- [x] Store width, height, and backend format metadata with each `GpuTexture`.
- [x] Map linear RGBA8 to `Rgba8Unorm` and sRGB RGBA8 to
      `Rgba8UnormSrgb` in the wgpu adapter.
- [x] Re-export the public contract through the `tokimu` facade.
- [x] Add checked payload sizing without changing the corpus-local
      `DecodedImage` contract.

#### Acceptance criteria

- [x] Color interpretation is explicit at allocation.
- [x] Zero dimensions, overflow, and incorrect payload lengths return typed
      errors before resource insertion.
- [x] No PNG, JPEG, BMP, DMX, Spout, or wgpu type appears in the public
      semantic descriptor.
- [x] Native and `wasm32` builds compile against the same public contract.

### Slice 2: Separate Creation From Replacement

#### Deliverables

- [x] Add the explicit creation path for a new handle.
- [x] Reject duplicate handles rather than replacing them silently.
- [x] If replacement remains necessary, expose it as an explicitly named
      operation or retain only the documented compatibility wrapper.
- [x] Migrate immutable corpus callers to the new creation operation.
- [x] Verify creation failure is rejected before the allocation path through
      backend-independent contract tests.

#### Acceptance criteria

- [x] A caller cannot accidentally replace stable resource identity through
      the new creation API.
- [x] Existing immutable texture corpus entries compile against explicit sRGB
      creation and preserve their startup-only resource lifecycle.
- [x] Replacement behavior, if retained, is visible in both API naming and
      statistics.

### Slice 3: Add Whole-Texture In-Place Writes

#### Deliverables

- [x] Add `update_texture_rgba8` or the accepted equivalent.
- [x] Validate handle existence, dimensions, and complete payload size.
- [x] Write through `Queue::write_texture` into the stored texture.
- [x] Preserve the existing texture and view objects.
- [x] Add backend-independent tests for missing handles, mismatched dimensions,
      short payloads, long payloads, and successful whole-resource requests.
- [x] Confirm validation completes before queue submission, leaving any
      registered resource untouched on rejected requests.

#### Acceptance criteria

- [x] Repeated writes do not call `Device::create_texture` or create a new
      view.
- [ ] A material uploaded before an update observes the new pixels without
      material re-upload.
- [x] The update accepts borrowed bytes and performs no required application-
      visible full-frame clone.
- [x] Partial rectangle updates and resize remain unsupported and explicit.

### Slice 4: Add Texture Lifecycle Statistics

#### Deliverables

- [x] Extend frame and lifetime renderer stats with texture allocation,
      replacement, and write counts.
- [x] Define whether initial-data creation contributes to each counter.
- [x] Update renderer-stat tests and performance corpus artifact schemas.
- [x] Add a steady-state assertion that writes do not imply allocation,
      replacement, or binding churn.
- [x] Keep texture counters producer-owned in `tokimu-render`.

#### Acceptance criteria

- [x] One initial creation and multiple updates produce the documented exact
      counter values.
- [x] Existing frame reset and lifetime accumulation semantics remain intact.
- [x] No per-frame kernel diagnostic is emitted for a successful write.
- [x] Applications can apply an ADR-0007 budget without the renderer owning
      policy.

### Slice 5: Build A Streaming Texture Corpus Proof

#### Deliverables

- [x] Add a focused `corpus/campaigns/textured-presentation/hello-streaming-texture` entry or justify reuse of
      a narrower existing corpus consumer.
- [x] Generate deterministic RGBA8 frames in application memory.
- [x] Create one texture and one material binding, then update only the pixel
      payload over time.
- [ ] Display motion or color change that makes stale bindings visible.
- [x] Emit bounded frame/lifetime statistics and a final validation summary.
- [x] Add a small deterministic automated case for frame generation and the
      steady-state lifecycle invariant.
- [x] Write frame-zero and validated CPU source-frame artifacts with manifests
      that explicitly distinguish them from GPU framebuffer capture.
- [x] Expose an explicit `--stress-1080p` native profile without changing the
      small deterministic default workload.
- [x] Add an opt-in bounded native run that exits only after the lifecycle
      validation checkpoint.
- [x] Define a case-specific manual native-window evidence location and
      manifest contract without treating it as automated proof.
- [x] Run the explicit 1920 by 1080 bounded native stress profile through 300
      validated writes without resource replacement.

#### Acceptance criteria

- [ ] The visible image changes for at least 300 consecutive updates.
- [ ] Texture allocations and replacements remain zero after warm-up.
- [ ] Binding allocations and pipeline creations remain zero after warm-up.
- [ ] The corpus remains useful without DMX or Spout installed.
- [ ] Native-window screenshots are labeled manual evidence rather than
      structural proof.

### Slice 6: Prove Linear And sRGB Interpretation

#### Deliverables

- [x] Create equivalent deterministic source bytes under linear and sRGB
      texture interpretations.
- [x] Sample both through a bounded shader/material corpus scene in
      `hello-texture-color-space`.
- [x] Capture structural descriptor evidence through the deterministic
      `hello-texture-color-space` contract tests.
- [x] Define a separately labeled manual native-window capture protocol for
      `hello-texture-color-space` without treating it as framebuffer evidence.
- [ ] Capture separately labeled native-window visual evidence.
- [x] Document where final output color-space policy remains consumer-owned.
- [x] Record the bounded evidence in AR-0006.

#### Acceptance criteria

- [x] Both backend formats are selected explicitly and inspectably in the
      public descriptor and wgpu adapter mapping.
- [x] The test does not claim source bytes have the same visual meaning under
      both interpretations.
- [x] No heuristic based on file extension, payload contents, or shader name
      selects the color space.
- [x] The work does not prematurely settle HDR, transfer functions, or final
      Spout/OBS color policy.

### Slice 7: Native And WASM Parity Check

#### Deliverables

- [x] Compile and exercise creation and repeated writes on native wgpu.
- [x] Compile the renderer contract plus focused streaming and color-space
      corpus packages for `wasm32-unknown-unknown`; their visual window entry
      points are intentionally native-only until browser execution is
      independently hosted.
- [ ] Add a browser/WASM update proof if the existing website or consumer
      corpus can host it without unrelated framework work.
- [ ] Record any WebGPU validation, row-layout, or resource-limit difference
      as adapter evidence.

#### Acceptance criteria

- [x] Native and WASM callers use the same semantic descriptor and update
      contract.
- [ ] Backend limitations fail explicitly without changing the semantic API.
- [x] A missing browser proof is recorded as a remaining AR-0006 evidence gap,
      not silently treated as parity.

### Slice 8: DMX Consumer Integration And Handoff

#### Deliverables

- [x] Prepare a consumer handoff that maps the request onto the implemented
      `Rgba8TextureDescriptor`, `create_texture_rgba8`, and
      `update_texture_rgba8` contract without adding DMX or Spout dependencies
      to Tokimu.
- [ ] Publish or identify the Tokimu revision containing the accepted API.
- [ ] Update the DMX consumer's pinned Tokimu revision.
- [ ] Create one renderer texture during consumer startup.
- [ ] Update it from each immutable application-owned `PixelFrame`.
- [ ] Feed preview and Spout from the same frame without sharing ownership.
- [ ] Add one preview-versus-Spout golden color test before finalizing the
      consumer's linear/sRGB choice.
- [ ] Record the upstream resolution in the incoming change request.

#### Prepared Consumer Handoff

The DMX consumer source is not part of this workspace, so this plan does not
claim consumer integration. The upstream contract is ready for the consumer to
adopt after a Tokimu revision is published:

1. At renderer startup, construct
   `Rgba8TextureDescriptor::new(1920, 1080, color_space)` using the
   application-selected color interpretation, then call
   `create_texture_rgba8` once for the chosen `TextureHandle`.
2. Bind that handle into the preview material once after creation.
3. For every immutable `PixelFrame`, borrow its RGBA8 bytes and call
   `update_texture_rgba8(handle, 1920, 1080, bytes)`.
4. Continue to pass the same application-owned frame to Spout through the
   consumer's own adapter. Tokimu neither owns nor depends on Spout.
5. Treat `MissingTexture`, `TextureDimensionsMismatch`, and
   `InvalidTexture` as explicit consumer errors. A resolution change requires
   a deliberate consumer rebuild, not an update call.

The downstream validation must record the published Tokimu revision and show
that renderer statistics remain at zero steady-state texture allocations,
replacements, and material-binding allocations while texture writes advance.

#### Acceptance criteria

- [x] The upstream handoff names the exact bounded API and preserves the
      application/Spout ownership boundary.
- [ ] Steady-state DMX output does not recreate Tokimu texture or material
      resources.
- [ ] A resize request is rejected or handled by an explicit consumer rebuild
      path outside the update method.
- [ ] Preview remains functional when Spout is unavailable.
- [ ] Spout integration introduces no dependency into Tokimu.
- [ ] The consumer records the exact Tokimu revision used for validation.

### Slice 9: Review, Documentation, And API Stabilization

#### Deliverables

- [x] Update AR-0006 with streaming, stable-identity, linear/sRGB, native, and
      available WASM evidence.
- [x] Document the current create, replace, and update semantics in renderer API
      docs.
- [ ] Update the SDD only if implementation changes an architectural boundary.
- [x] Retain the compatibility `upload_texture` API as an explicitly documented
      sRGB create-or-replace bridge; future removal or deprecation requires
      another caller inventory.
- [x] Decide that current evidence justifies only the concrete renderer API,
      not a broader texture request model.
- [ ] Close this plan without creating a new crate unless independent consumers
      prove a stable broader boundary.

#### Acceptance criteria

- [x] Public names describe resource lifecycle honestly.
- [x] AR-0006 retains unresolved questions that this consumer did not test.
- [x] No backend-native resource escapes through the public facade's RGBA8
      descriptor contract.
- [x] Documentation distinguishes semantic guarantees from observed wgpu
      behavior.

## Validation Matrix

| Area | Required validation |
| --- | --- |
| Contract | linear/sRGB mapping, dimension checks, checked payload length |
| Creation | duplicate handle rejection, registry unchanged on failure |
| Update | missing handle, dimension mismatch, short/long payload, repeated writes |
| Identity | stable texture/view/material binding across writes |
| Statistics | exact frame and lifetime allocation/replacement/write counts |
| Rendering | material uploaded once samples changing texture contents |
| Compatibility | existing raster and font corpus texture consumers still pass |
| Native | deterministic small case plus 1920 by 1080 manual stress run |
| WASM | shared API compile and browser execution when available |
| Consumer | preview and Spout consume the same immutable `PixelFrame` |

Repository validation should include:

```text
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check --target wasm32-unknown-unknown -p tokimu-render
```

Targeted tests should run before the full workspace validation so failures
remain localized to texture contract, backend execution, or consumer behavior.

## Non-Goals

- Partial or dirty-rectangle updates.
- Texture resize through the update API.
- Asynchronous upload queues or staging-ring design.
- Compressed, multi-plane, HDR, or higher-precision texture formats.
- Mip generation or general sampler policy.
- A universal image or sampled-resource requirement abstraction.
- DMX, HNode, Spout, OBS, or MIDI semantics in Tokimu.
- Final consumer color-space policy before golden evidence.
- GPU completion timing or a general profiler.
- Parallel mutation of renderer resources from producer callbacks.

## Risks And Mitigations

### Existing `upload_texture` callers depend on replacement

Inventory and migrate all callers before changing behavior. Keep replacement
explicit during transition rather than silently changing old semantics.

### Stable handle but stale material view

The proof must upload the material once before repeated writes. Renderer stats
and a changing visual fixture together demonstrate that no hidden material
rebind is required.

### Color-space names become policy claims

Name the texture interpretation precisely as linear UNORM versus sRGB UNORM.
Do not imply that either choice settles monitor, Spout, OBS, HDR, or transfer-
function policy.

### Full-frame writes are expensive

The first contract intentionally exposes measured cost rather than designing a
staging system without evidence. Record CPU call time and renderer operation
counts separately. Revisit partial updates or staging only after sustained
consumer measurements identify the owning bottleneck.

### Statistics create diagnostic noise

Keep high-frequency measurements in renderer stats. Emit only bounded budget
transitions through kernel diagnostics when an application opts into policy.

### Renderer API accidentally absorbs image semantics

The public descriptor describes already prepared RGBA8 bytes and their sampled
interpretation. It must not acquire encoded format, EXIF, ICC, decoder,
filesystem, or asset-provider concerns.

### wgpu implementation details leak into public contracts

Keep `wgpu::TextureFormat`, `TextureView`, `Sampler`, `Queue`, and device limits
inside the adapter. Translate public semantics at the backend boundary.

## Definition Of Done

This plan is complete when:

- the renderer exposes explicit linear/sRGB RGBA8 creation;
- complete borrowed RGBA8 payloads update an existing handle in place;
- dimensions and payload errors are deterministic and non-destructive;
- texture, view, sampler, handle, and dependent material binding identity stay
  stable across updates;
- statistics distinguish allocations, replacements, and writes;
- a Tokimu-owned corpus proves changing pixels without steady-state resource
  churn;
- native behavior is validated and WASM status is explicit;
- the DMX consumer integrates against a pinned upstream revision;
- preview and Spout consume the same application-owned frame without moving
  Spout semantics into Tokimu;
- AR-0006 records the evidence and preserves remaining open questions;
- the compatibility upload API has an explicit final disposition.

## References

- [Tokimu Software Design Document](../../Tokimu%20Software%20Design%20Document.md)
- [ADR-0001: Engine Boundaries](../../ADR/ADR-0001-engine-boundaries.md)
- [ADR-0003: Capability Ownership Boundary](../../ADR/ADR-0003-capability-ownership-boundary.md)
- [ADR-0007: Kernel Performance Diagnostics](../../ADR/ADR-0007-kernel-performance-diagnostics.md)
- [AR-0006: Raster Image Requirement Pipeline](../../Architectural%20Reviews/AR-0006-raster-image-requirement-pipeline.md)
- [TypeScript Shader, Material, And Presentation Control](typescript-shader-material-presentation-control.md)
- [Performance Diagnostics And Runtime Observation](performance-diagnostics-and-runtime-observation.md)
- [Raster Image Corpus Testing](../../Libraries/raster-image-corpus-testing.md)
