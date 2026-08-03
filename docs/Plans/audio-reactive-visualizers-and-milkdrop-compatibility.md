# Audio-Reactive Visualizers And MilkDrop Compatibility

## Status

Implementation started on 2026-08-01. Deterministic visualizer observations,
a bounded native feedback corpus application, and a selected first-party
MilkDrop parser/evaluator now incubate under `corpus/`. No general MilkDrop
compatibility provider, provider-neutral multipass renderer, or visualizer
capability is currently admitted.

Plan checkpoint: 2026-08-02. The corpus now emits validated three-pass and
previous-frame feedback structural graphs with bounded pass/resource summaries.
The structural graph remains provider-neutral evidence. The native WGPU corpus
executes a deliberately narrow ping-pong feedback proof, but does not establish
a provider-neutral multipass renderer API. The native route reuses material
and instance bindings across ordinary frames; a native frequency-sweep run
recorded zero frame-local resource churn after warmup. Three
original native visualizer definitions now serialize as corpus-side structural
evidence, with stable identities, bounded parameters, and distinct pass-graph
shapes. They do not yet establish a shared visualizer execution model.
The website consumer now executes the same selected first-party scalar fixture
inside Rust/WASM and exposes its controls to Canvas as explicitly labeled
browser observation evidence; it does not establish arbitrary preset support,
projectM integration, microphone capture, or native feedback-renderer
equivalence.

The first implementation should incubate in focused corpus libraries and
applications. This plan does not create `tokimu-audio`,
`tokimu-visualizer`, or a MilkDrop compatibility crate by itself. New
capability ownership requires corpus evidence and Architectural Review.

## Purpose

Tokimu should be able to produce rich audio-reactive visualizations and, if the
compatibility evidence holds, consume the large ecosystem of MilkDrop presets.

MilkDrop compatibility is not merely loading a shader. A useful implementation
combines:

- audio PCM ingestion, spectrum analysis, and beat observations;
- a preset format containing parameters and per-frame equations;
- custom waveform and shape evaluation;
- shader translation or a compatible shader execution path;
- textures, samplers, blending, and multiple rendering passes;
- render-to-texture and previous-frame feedback; and
- deterministic time, diagnostics, resource bounds, and preset lifecycle.

The work should therefore produce two sibling outcomes over shared Tokimu
presentation infrastructure:

1. a small Tokimu-native visualizer model that is pleasant to author, inspect,
   test, and run on native and WASM targets; and
2. an optional MilkDrop compatibility provider that lowers supported `.milk`
   presets into that model or explicitly diagnoses semantics that cannot be
   represented.

## Architectural Thesis

> Applications own why audio is visualized. Audio capabilities own audio
> meaning and analysis. Visualizer providers own preset languages. Tokimu
> presentation owns pass requirements. Renderers own targets, shaders, and
> pixels.

```text
audio mechanism
    microphone / file / host PCM
            |
            v
provider-neutral audio observations
    waveform / spectrum / beat / time
            |
            +-----------------------------+
            |                             |
            v                             v
Tokimu-native visualizer           MilkDrop provider
definition                         parser + evaluator
            |                             |
            +--------------+--------------+
                           v
              visualizer frame requirements
              passes / parameters / textures
                           |
                           v
                    renderer execution
```

MilkDrop is a provider and compatibility target. It must not define Tokimu's
audio, shader, material, timing, or renderer ownership boundaries.

## Governing Documents

- [`Tokimu Software Design Document.md`](../Tokimu%20Software%20Design%20Document.md)
  keeps audio and rendering outside the trusted simulation core.
- [`Tokimu TypeScript Design Document.md`](../Tokimu%20TypeScript%20Design%20Document.md)
  requires TypeScript authoring to lower one-way into Tokimu-owned models.
- [`ADR-0001-engine-boundaries.md`](../ADR/ADR-0001-engine-boundaries.md)
  keeps presentation from owning simulation truth.
- [`ADR-0003-capability-ownership-boundary.md`](../ADR/ADR-0003-capability-ownership-boundary.md)
  separates Tokimu-owned semantics from replaceable providers.
- [`ADR-0007-kernel-performance-diagnostics.md`](../ADR/ADR-0007-kernel-performance-diagnostics.md)
  permits bounded performance observations without making the kernel a
  profiler.
- [`AR-0006-raster-image-requirement-pipeline.md`](../Architectural%20Reviews/AR-0006-raster-image-requirement-pipeline.md)
  records audio as a future test of requirement propagation and keeps encoded
  image formats out of shader contracts.
- [`AR-0008-audio-observation-and-visualizer-boundary.md`](../Architectural%20Reviews/AR-0008-audio-observation-and-visualizer-boundary.md)
  records the initial PCM-analysis evidence and defers audio capability
  admission until provider and second-consumer pressure exists.
- [`typescript-shader-material-presentation-control.md`](typescript-shader-material-presentation-control.md)
  owns the provider-neutral shader module, material, WGSL, and TypeScript
  authoring direction.
- [`streaming-rgba8-texture-updates.md`](streaming-rgba8-texture-updates.md)
  proves bounded CPU-to-GPU texture allocation and updates, but does not prove
  render targets or feedback textures.

## Current Evidence

Tokimu already provides useful pieces:

- provider-neutral shader-module declarations and bounded hand-written WGSL;
- explicit material, texture, pipeline, and draw identities;
- source-format-neutral raster decoding and texture preparation;
- reusable and streaming RGBA8 texture upload;
- native WGPU and planned/evolving WASM presentation paths;
- particles and browser consumers that prove deterministic time and bounded
  presentation snapshots; and
- kernel-native structured performance diagnostics.

The current implementation does not yet prove:

- audio capture, file playback, PCM ingestion, FFT, or beat analysis;
- resource-bound diagnostics or a renderer memory-budget policy;
- provider-neutral ping-pong previous-frame feedback;
- a provider-neutral multipass frame description;
- stable visualizer parameter or preset semantics;
- a safe equation evaluator;
- MilkDrop preset parsing or shader compatibility;
- custom MilkDrop waves and shapes;
- preset texture-pack resolution; or
- equivalent native and browser behavior.

`create_texture_rgba8` and `update_texture_rgba8` are valuable input-texture
evidence. They are not substitutes for renderer-owned render-to-texture and
feedback lifecycle.

## Ownership Boundaries

### Application owns

- which audio source and visualizer are selected;
- playlist, transition, and user-facing control policy;
- whether visualization state has gameplay or simulation meaning;
- presentation placement, viewport, and interaction; and
- permission to use microphones, files, or host audio.

### Audio input provider owns

- native device, browser `AudioContext`, file decoder, or host callback;
- PCM sample delivery and source-specific buffering;
- sample-rate/channel observations; and
- platform failures and permission diagnostics.

It must not own visualizer equations, render passes, presets, or simulation
truth.

### Audio analysis owns

- bounded waveform windows;
- spectrum bins and named frequency bands;
- smoothing policy;
- beat or onset observations;
- analysis latency and timestamp provenance; and
- deterministic synthetic-analysis fixtures.

The first proof may incubate in a corpus library. It must not enter
`tokimu-core`, and it must not require a window or GPU.

### Visualizer model owns

- stable preset identity and parameter defaults;
- named pass requirements and dependency order;
- references to audio observations, time, textures, and prior-pass outputs;
- bounded wave, shape, and compositing descriptions; and
- transition-compatible visualizer state.

It does not own backend textures, command encoders, bind groups, audio devices,
or a platform clock.

### MilkDrop provider owns

- `.milk` syntax and version quirks;
- MilkDrop variables, equations, and evaluation order;
- compatibility defaults and undocumented behavior that evidence requires;
- custom wave and shape semantics;
- shader-dialect translation; and
- compatibility diagnostics and preset-specific assets.

MilkDrop-native objects stop at the provider boundary.

### Renderer owns

- offscreen texture allocation and resize/replacement;
- pass execution, load/store operations, and dependency ordering;
- bind groups, samplers, shader compilation, and backend resources;
- ping-pong target selection and previous-frame preservation;
- surface presentation and framebuffer capture; and
- render timing and resource lifecycle measurements.

## Recommended Product Direction

Build both paths, but do not begin with full MilkDrop compatibility.

The Tokimu-native path provides a small, explicit semantic target and prevents
the engine from inheriting every historical MilkDrop behavior. The MilkDrop
provider then supplies a large, adversarial real-world corpus and a useful
library of existing presets.

```text
Phase A: Tokimu visualizer substrate
    synthetic audio -> one pass -> feedback -> multipass

Phase B: Tokimu-native authoring
    data model -> Rust definitions -> TypeScript lowering

Phase C: MilkDrop compatibility
    parser -> equations -> waves/shapes -> shaders -> preset corpus
```

Each phase remains useful if a later MilkDrop compatibility limit proves too
expensive or platform-specific.

## Implementation Slices

### Slice 0: Boundary Review And Corpus Definition

Deliverables:

- [x] Record the visualizer, audio-analysis, renderer, and provider ownership
      boundaries in the initial corpus design.
- [ ] Inventory the MilkDrop 1 and MilkDrop 2 preset constructs exercised by a
      deliberately small candidate set.
- [x] Review projectM only as an external compatibility reference, not as a
      Tokimu dependency, adapter, backend, or architectural model.
- [ ] Select only presets and texture assets with recorded provenance,
      redistribution terms, upstream revision, and hashes.
- [x] Define maturity labels: `native`, `compatible`, `partial`, `unsupported`,
      and `invalid`.
- [x] Open `AR-0008` before admitting audio-analysis or multipass presentation
      as a permanent Tokimu capability.

Initial reference review, 2026-08-01:

- [x] Record projectM as an external compatibility reference, not a Tokimu
      dependency, adapter, backend, or architectural template. Its public
      project description combines preset parsing, PCM analysis, beat
      detection, and OpenGL rendering, which reinforces the need to keep those
      responsibilities separate in Tokimu. The core library is LGPL-2.1;
      preset packs are separately distributed and require their own provenance
      review.
- [x] Record the initial MilkDrop construct inventory from the authoring
      documentation: preset parameters, initialization/per-frame/per-vertex
      equations, variable persistence, custom waves, custom shapes, and pixel
      shaders. These are candidate provider semantics, not accepted Tokimu
      visualizer semantics.

Reference links:

- [projectM repository](https://github.com/projectM-visualizer/projectm)
  describes its combined library responsibilities, the separate preset-pack
  distribution, and the LGPL-2.1 license.
- [MilkDrop preset authoring guide](https://milkdrop.org/resources/preset-authoring)
  lists the equation, variable, custom wave/shape, and pixel-shader constructs
  that a compatibility provider would need to inventory deliberately.

This review admits neither projectM code, binaries, libraries, runtime
integration, nor external `.milk` presets. Before
any preset becomes a fixture, record its exact upstream repository, revision,
author/license information, redistribution terms, hash, intended construct,
and expected maturity label.
`corpus/hello-milkdrop/EXTERNAL-PRESET-ADMISSION.md` provides the required
candidate record; it does not admit a preset by itself.

Acceptance criteria:

- [ ] Every selected preset has a reason for inclusion and a legal/provenance
      record.
- [ ] Unsupported semantics are expected evidence, not silent approximation.
- [ ] No external preset pack is copied into the repository before its license
      is reviewed.
- [ ] The plan does not imply that projectM, OpenGL, or MilkDrop owns Tokimu
      presentation semantics.

### Slice 1: Deterministic Visualizer Input Contract

Deliverables:

- [x] Define a bounded frame input containing explicit visualizer time, delta,
      frame index, viewport, waveform samples, spectrum bins, named bands, and
      beat/onset observations.
- [x] Keep application time separate from provider wall-clock mechanisms.
- [x] Add deterministic synthetic fixtures: silence, impulse, steady tone,
      frequency sweep, and seeded band pulses.
- [x] Serialize structural observations for corpus comparison.
- [x] Bound sample counts, spectrum bins, values, and update frequency.

Acceptance criteria:

- [x] Identical synthetic input and explicit time produce identical structural
      frame input.
- [x] The contract can be produced headlessly without an audio device, window,
      or GPU.
- [x] Invalid dimensions, non-finite values, and excessive buffers fail with
      stable diagnostics.
- [x] No platform audio object enters the visualizer contract.

### Slice 2: Fullscreen Shader Corpus

Deliverables:

- [x] Create `corpus/hello-audio-visualizer` with one fullscreen render pass.
- [x] Feed explicit time and synthetic audio bands into a hand-written WGSL
      shader through provider-neutral bindings.
- [x] Add runtime controls for pause, time scale, synthetic input selection,
      and visualizer reset.
- [x] Emit deterministic CPU review artifacts for every synthetic fixture,
      including serialized input, a signal-preview image, and a manifest that
      explicitly rejects GPU framebuffer-equivalence claims.
- [x] Emit deterministic shader-module, material-binding, and timing evidence.
- [x] Keep native-window screenshots as separately labeled manual evidence.
- [x] Keep the first proof free of feedback, real audio, and MilkDrop parsing.

Acceptance criteria:

- [x] The corpus renders a deterministic visual response to each synthetic
      fixture.
- [x] Pausing freezes visualizer time even while the application continues to
      present frames.
- [x] Shader and binding failures identify the owning diagnostic stage before
      backend submission.
- [x] The visualizer does not mutate world state or require an ECS entity.

Implementation note: the semantic shader declaration uses a provider-neutral
`Vector4` input, while current renderer execution temporarily carries that
value through the legacy four-float material slot. This is recorded as
`partial` evidence in the corpus design and must not be mistaken for a stable
arbitrary material-parameter execution API.

The corpus preflights its `Vector4` material definition against the declared
shader and quad mesh at native startup. A focused negative test supplies a
`Color` in place of that `Vector4` and verifies that the resulting error is
classified as `DrawContractValidation`. Renderer submission still stores the
legacy execution material, so per-draw enforcement of arbitrary semantic
parameters remains future renderer work rather than an implied guarantee.

Artifact note: `hello-audio-visualizer --write-artifacts` writes deterministic
CPU signal previews under `target/audio-visualizer/`. These artifacts make the
fixture-to-visual-response mapping reviewable without a window or GPU. Their
manifests label them as source-side evidence; native-window screenshots and
backend framebuffer captures remain separately labeled manual evidence.

### Slice 3: Renderer-Owned Offscreen Targets

Deliverables:

- [x] Define corpus-local, provider-neutral render-target requirements without
      exposing WGPU textures or views. This remains incubation evidence while
      `AR-0006-raster-image-requirement-pipeline.md` keeps the broader sampled
      resource contract under review.
- [x] Support explicit size, color interpretation, sampling, and clear/load
      behavior in the corpus contract.
- [x] Allocate backend render targets inside `tokimu-render` adapters.
- [x] Support sampling one completed offscreen target in a later pass.
- [x] Add explicit non-destructive render-target release diagnostics.
- [x] Add renderer-local render-target footprint diagnostics without claiming a
      portable GPU-memory budget.
- [x] Add deterministic resize and replacement diagnostics.
- [x] Distinguish source textures, render targets, and surface images.

Acceptance criteria:

- [x] A two-pass corpus renders pass A offscreen and samples it in pass B.
- [x] No pass samples a target while that target is being written.
- [x] Resize behavior is deterministic and reports allocation/replacement
      diagnostics.
- [x] Render-target resources remain renderer-private.

Current evidence: `visualizer-tools::VisualizerPassGraph` serializes a
validated two-pass `signal -> present` graph to
`target/audio-visualizer/two-pass-signal.graph.json`. Its preflight rejects
missing targets, read-before-write ordering, same-pass read/write hazards,
invalid target dimensions, and non-final surface writes before backend
allocation or submission. It additionally rejects sampling a target declared
`not-sampled` and writing one target more than once in a graph. This is
structural corpus evidence rather than a stable cross-capability API.

`tokimu-render::WgpuBackend::create_render_target_rgba8` now allocates an
opaque, sampleable RGBA8 target with render-attachment usage from the existing
`TextureHandle` identity. The renderer rejects source-pixel updates to a
renderer-owned target, preserving the source-texture versus render-target
distinction at the adapter boundary. The corpus command
`hello-audio-visualizer --offscreen-probe` successfully allocated a headless
640x360 sRGB target on 2026-08-01, including a repeat verification after the
PCM16 WAVE fixture work.

`WgpuBackend::draw_meshes_to_render_target` now provides the first deliberately
narrow execution proof. It accepts a bounded slice of ordinary mesh draws,
clears an existing renderer-owned target with a matching depth attachment, and
submits that pass before normal surface presentation. `hello-audio-visualizer`
writes its feedback pass into the alternate history target, then presents that
completed target through an ordinary `Texture2d` material. The backend rejects
target self-sampling during the write pass and target formats that do not match
the active surface pipeline format. This is a backend-specific adapter method,
not a new `Renderer` trait contract or a provider-neutral multipass API.

Target replacement is intentionally not treated as an ordinary texture upload:
uploaded materials cache backend bind groups built from a texture view.
`WgpuBackend::replace_render_target_rgba8` therefore preserves the opaque target
handle while reporting how many dependent materials require rebinding and
invalidating derived material bindings that retain the previous view. The native
visualizer replaces both history targets when its window viewport changes, then
immediately re-uploads their target-sampling materials. It emits the
replacement dimensions plus rebind and invalidation counts.
`WgpuBackend::render_target_resource_observation` now additionally reports the
live target count, color-image pixels, and an estimated RGBA8-plus-Depth32
image footprint. The visualizer records that snapshot with its warm and resize
observations. It deliberately excludes driver overhead, surface images, views,
samplers, caches, and GPU residency; it is diagnostic evidence rather than a
portable memory budget.

`WgpuBackend::release_render_target` now makes target cleanup explicit without
silently invalidating a material bind group: it refuses release while a material
still samples the target and reports the opaque target handle plus reference
count. Callers detach or replace the material binding first, then retry
release. This remains backend-scoped lifecycle evidence; it does not admit a
general public target-management API or a renderer memory-budget policy.

Execution boundary finding: the public `RenderCommand` stream remains surface
oriented. The execution proof is intentionally exposed only by the concrete
WGPU adapter, where it can reuse its pipeline layouts and keep target views,
attachments, and submission ordering renderer-private. Cross-format pipelines,
resource-bound policy, and a public multipass command shape remain deferred
until a second consumer establishes the right boundary.

### Slice 4: Previous-Frame Feedback

Deliverables:

- [x] Add a bounded ping-pong target pair with explicit current and previous
      frame roles as structural corpus evidence.
- [x] Define structural first-frame initialization and reset intent.
- [x] Add bounded decay and blend parameters to the visualizer corpus.
- [x] Preserve feedback across ordinary frames and reset it on explicit preset,
      size, or lifecycle transitions.
- [x] Capture structural pass graphs and native renderer-local feedback
      execution evidence. Deterministic saved feedback images remain deferred.

Acceptance criteria:

- [x] No frame reads and writes the same texture subresource.
- [x] Reset and first-frame output are deterministic.
- [x] Feedback survives steady-state rendering without CPU texture readback or
      per-frame resource allocation.
- [x] The corpus diagnoses unsupported feedback sampling features. Feedback
      format families remain deferred until a second concrete format appears.

Current evidence: `VisualizerPassGraph::three_pass_feedback` records a bounded
`signal -> feedback -> surface` frame plan. A named `history` pair has distinct
`history-previous` and `history-current` targets; the feedback pass reads only
the previous-frame role and writes only the current-frame role. The pair records
an explicit clear initialization policy, and validation rejects unknown pairs,
same-target pairs, unsampleable members, a current-frame write to the previous
member, and a missing current-frame write. The structural summary records the
pair and previous-frame read counts. This proves temporal resource intent and
preflight diagnostics. The native WGPU corpus now creates two same-sized sRGB
history targets, clears both at startup, samples only the previous target while
writing only the alternate target, and swaps their roles after surface
presentation. Explicit reset and resize clear history again. The feedback
shader applies bounded decay and synthetic-signal injection. This remains a
one-sampled-texture proof: it does not execute the separate structural signal
target or establish a provider-neutral multipass API. Ordinary native feedback
frames update the active material uniform in place and reuse the target-pass
instance binding; target replacement deliberately rebinds affected materials
at the resize boundary. A native frequency-sweep observation after frame 120
reported `resource_churn=0`, `binding_allocations=0`,
`pipeline_creations=0`, `mesh_uploads=0`, and `texture_allocations=0` while
the feedback route continued submitting frames. This is CPU-side lifecycle
evidence, not a GPU completion metric or automatic performance policy. The
same live proof exposed a concrete WGPU execution defect: material uniforms
created for runtime update required `COPY_DST` in addition to `UNIFORM`.
Without it, `queue.write_buffer` could be counted by diagnostics while the GPU
retained the initial phase value and the scan marker remained fixed. The
backend now allocates `UNIFORM | COPY_DST`, protected by a focused renderer
test.

### Slice 5: Provider-Neutral Multipass Description

Deliverables:

- [x] Describe named passes, inputs, outputs, dependencies, and final surface
      output as bounded data.
- [x] Validate acyclic pass ordering and resource compatibility before backend
      execution.
- [x] Keep pipeline selection explicit per pass.
- [x] Add bounded intermediate-target and pass-count limits.
- [x] Expose bounded resource counts through structural diagnostics.
- [x] Expose separately labeled CPU pass timings after an execution path exists.

Acceptance criteria:

- [x] One corpus visualizer declares at least three passes, including feedback
      and final compositing.
- [x] Cycles, missing outputs, incompatible formats, and excessive graphs fail
      before backend submission.
- [x] The graph contains no backend-native resources or command objects.
- [x] A single-pass shader remains straightforward and does not require a
      ceremonial graph in application code.

Current evidence: `VisualizerPassGraph::three_pass_signal` adds a bounded
`signal -> composite -> surface` graph alongside the original two-pass fixture.
Every pass now declares a provider-neutral pipeline label without carrying a
renderer pipeline handle or shader object. `validate_with_summary` records pass
count, target count, distinct pipeline count, source-texture reads,
render-target reads and writes, surface outputs, and maximum target dimensions.
`hello-audio-visualizer --write-artifacts` emits both graph forms plus the
three-pass summary, while `--structural-fixture` prints the same evidence.
Validation rejects read-before-completion cycle-like ordering, a missing final
surface output, incompatible feedback-pair dimensions or color interpretation,
and target or pass graphs beyond the explicit bounds before backend submission.
This proves structural dependency ordering and bounded resource accounting. The
native corpus separately proves its narrow one-sampled-texture execution path.
`RenderFrameCpuTimings` now exposes accumulated offscreen target encoding and
queue-submission call durations separately from surface phases, and the native
corpus prints them as labeled CPU observations. These timings do not measure
GPU completion or establish framebuffer equivalence.

`visualizer-tools` has no renderer dependency and its graph records only
strings, bounded requirements, and structural resources. It therefore cannot
carry WGPU handles, command encoders, or backend views. A surface-only
`VisualizerPassGraph::single_pass_surface` fixture validates without targets
or feedback pairs, preserving a direct path for a simple fullscreen shader.

### Slice 6: Audio Analysis Incubation

Deliverables:

- [x] Define a corpus-side PCM input and deterministic windowing policy.
- [x] Produce waveform, reference spectrum magnitude, named bands, and a
      deliberately simple energy/onset observation.
- [x] Keep mechanism-specific capture outside the analysis implementation.
- [x] Add fixed PCM fixtures and expected numeric artifacts.
- [x] Define deterministic bounded-backlog behavior with explicit loss policy
      and counters.
- [x] Adapt a bounded deterministic PCM16 WAVE byte fixture into the same PCM
      analysis contract without admitting playback or a general media decoder.
- [x] Add a renderer-free second corpus consumer that independently adapts the
      generated PCM16 WAVE fixtures into analysis, timing, and working-set
      evidence.
- [x] Measure bounded native-host analysis latency under representative fixed
      windows without treating it as a cross-machine performance contract.
- [x] Establish a portable source-structural working-set observation before
      making any allocation claim for audio analysis.
- [x] Compare the resulting requirement-resolution boundary with
      `AR-0006-raster-image-requirement-pipeline.md`.

Acceptance criteria:

- [x] Fixed PCM and configuration produce deterministic observations within
      documented numeric tolerances.
- [x] Silence, clipping, and stereo-to-mono channel conversion are explicit.
      Sample-rate mismatch and backlog policies remain provider-slice work.
- [x] Analysis runs headlessly and independently of rendering.
- [x] No audio device or browser API enters a semantic contract.
- [x] `AR-0008` records the initial comparison with AR-0006 without admitting
      a shared requirement-propagation abstraction.

Current evidence: `visualizer-tools::PcmAudioWindow` accepts only bounded,
finite normalized PCM frames with an explicit mono or stereo channel count.
`PcmAnalyzer` applies a deterministic Hann window, mixes stereo frames to mono,
and produces a bounded reference spectrum, raw and smoothed named bands, plus
an onset observation from explicit caller-owned prior energy. The spectrum currently uses
`direct-dft-magnitude-v1`, intentionally a small correctness reference rather
than a production FFT claim. Fixed silence, impulse, mono-tone, and stereo-tone
fixtures are checked by tests and can be serialized through
`hello-audio-visualizer --pcm-fixtures` or `--write-artifacts`. Capture,
sample-rate adaptation, smoothing, and historical onset detection remain outside
this slice. `PcmAnalysisBacklog` adds bounded corpus-side overload evidence:
callers choose `drop-oldest` or `drop-newest`, retain explicit pending and
dropped-window counts, and drain analysis in arrival order. It is not an audio
device ring buffer or a timing mechanism. The deterministic artifact command
also writes `pcm-backlog-drop-oldest.json`; the explicit
`--pcm-backlog-fixture` mode prints the same compact structural snapshot.
`hello-audio-visualizer --pcm-measure` runs each fixed window through 32
reference analyses and prints a separately labeled native-host timing
observation plus a deterministic source-structural working-set observation.
The latter records retained and transient `f32` slots implied by the reference
algorithm, not allocator calls, `Vec` capacity, platform overhead, or a
portable performance budget. Both observations are written beside fixed analysis artifacts by
`--write-artifacts`, so corpus review does not depend on terminal output.
`visualizer-tools::decode_pcm16_wav` now provides one deliberately narrow
byte-source adapter: it accepts bounded canonical PCM16 little-endian
RIFF/WAVE fixture bytes, skips benign unknown RIFF chunks, and returns only the
existing validated `PcmAudioWindow`. Generated fixture bytes round-trip through
the adapter before analysis, while malformed headers, lengths, encodings, and
frame alignment fail before the analysis stage. `--wav-fixtures` prints the
same source-to-window evidence; `--write-artifacts` records the fixture bytes,
decoded analysis, and a source provenance record. This is not a file playback,
device capture, or general media-decoding guarantee.
`hello-audio-analysis` repeats that exact byte-source-to-analysis handoff from
a renderer-free executable and writes one combined inspection artifact per
fixture. It records decoded-window facts, analysis output, timing, and
source-structural working-set evidence while explicitly declaring that no
renderer, audio device, or playback path participated. This establishes a
second consumer of the analysis seam, not a native or browser audio provider.

### Slice 7: Native And Browser Audio Providers

Deliverables:

- [ ] Add one native PCM source provider and one browser/WASM provider only
      after the analysis contract stabilizes.
- [x] Support a deterministic generated PCM16 WAVE source before microphone or
      system-loopback capture.
- [x] Prove the generated PCM16 WAVE adapter through the renderer-free
      `hello-audio-analysis` corpus consumer before treating a visualizer as
      the analysis contract's only consumer.
- [ ] Keep browser permission and autoplay behavior in the browser adapter.
- [ ] Define bounded ring-buffer, latency, underrun, and overrun diagnostics.
- [ ] Map both providers into the same analysis input contract.

Acceptance criteria:

- [ ] Native and browser sources drive the same visualizer without changing its
      semantic model.
- [ ] Denied permission, unavailable devices, underruns, and disconnection are
      explicit diagnostics.
- [ ] Capture callbacks do not render or mutate simulation state.
- [ ] Synthetic input remains available for deterministic tests.

### Slice 8: Tokimu-Native Visualizer Model

Deliverables:

- [x] Define a compact corpus-side model for stable parameters and pass graphs;
      waves, shapes, textures, transitions, and equations remain deferred.
- [x] Select explicit Rust data for the first definitions. A bounded interpreter
      or ahead-of-time lowering input needs independent corpus pressure.
- [x] Provide a small corpus-side library of three original Tokimu visualizer
      definitions: `signal-field`, `feedback-bloom`, and `signal-composite`.
- [x] Preserve stable parameter identity for UI controls and automation in the
      structural definitions.
- [x] Add deterministic serialization and validation for the incubating
      definitions and their pass graphs.

Acceptance criteria:

- [x] At least three visually distinct original visualizers share the same
      frame-input and pass contracts.
- [x] Structural evidence includes one two-pass non-feedback definition, one
      previous-frame feedback definition, and one three-pass signal definition.
- [x] Invalid parameters, missing required graph inputs, and excessive graph
      work fail within documented bounds. Expressions remain out of scope until
      a bounded expression form is admitted for study.
- [x] The structural model contains no MilkDrop parser objects or backend-native
      handles.

Current evidence: `visualizer-tools::NativeVisualizerDefinition` remains a
corpus-local, provider-neutral description. It intentionally excludes equation
evaluation, waves, shapes, textures, transition execution, MilkDrop parser
objects, renderer handles, and backend resources. `hello-audio-visualizer`
now executes `Signal Field` through its declared two-pass target-to-surface
shape, `Feedback Bloom` through its narrow previous-frame proof, and `Signal
Composite` through a distinct signal -> composite target -> surface route.
Each route remains explicit corpus code; the implementation does not promote a
general graph executor or visualizer language prematurely. Native manual
evidence confirms that `Q`, `W`, and `E` select visibly distinct modes while a
phase-driven scan marker advances under the shared `VisualizerFrameInput`.

### Slice 9: TypeScript Visualizer Authoring

Deliverables:

- [ ] Prototype an unpublished `@tokimu/visualizer` authoring surface after the
      Rust model has independent callers.
- [ ] Lower one-way into the Tokimu visualizer and shader semantic models.
- [ ] Reuse `@tokimu/shader` expression and source-diagnostic work where it has
      stabilized.
- [ ] Reject DOM, network, files, ambient time, and runtime JavaScript callbacks
      in authored visualizer definitions.
- [ ] Preserve authored source locations in validation diagnostics.

Acceptance criteria:

- [ ] A TypeScript-authored visualizer lowers deterministically to the same
      artifact as an equivalent Rust definition.
- [ ] TypeScript owns authoring ergonomics, not execution or renderer state.
- [ ] Runtime parameter changes do not trigger shader recompilation unless the
      authored shader definition actually changes.
- [ ] Native Rust consumers remain first-class and TypeScript-free.

### Slice 10: MilkDrop Preset Parser And Equation Runtime

Deliverables:

- [x] Parse a deliberately selected MilkDrop 1 subset before MilkDrop 2 shader
      compatibility.
- [x] Preserve source sections, variable names, equation order, and locations
      for diagnostics.
- [x] Implement bounded initialization and per-frame scalar equation semantics
      required by the Tokimu-authored fixture.
- [ ] Implement per-pixel equation semantics only after a renderer-facing
      lowering contract has an honest consumer.
- [x] Define selected scalar compatibility defaults explicitly rather than
      inferring them from successful rendering.
- [x] Reject unknown, invalid, and excessive preset constructs deterministically.

Acceptance criteria:

- [x] Parser artifacts are deterministic and retain enough provenance to map a
      diagnostic back to a preset line or section.
- [x] Selected initialization and per-frame equation fixtures produce a
      deterministic scalar-state artifact.
- [ ] Equation fixtures match a documented external compatibility oracle for
      admitted third-party preset inputs.
- [x] Preset evaluation cannot access files, network, process state, or ambient
      randomness.
- [x] Unsupported MilkDrop 2 shader sections are labeled `unsupported`, not
      silently ignored.

Current evidence: `milkdrop-tools` and `hello-milkdrop` establish a bounded,
headless parser and scalar-evaluation boundary using a Tokimu-authored
MilkDrop 1-style fixture.
The parser preserves ordered sections, keys, values, and source lines while
classifying selected scalar parameters plus initialization, per-frame, and
per-pixel equation declarations. Selected literal custom waves and convex
custom shapes are separately classified for bounded lowering; custom code,
warp shaders, composite shaders, and unknown keys remain explicit
`unsupported` evidence.
The selected initialization and per-frame scalar subset now evaluates in source
order into an explicit, deterministic state map. Selected classic scalar keys
resolve against documented Tokimu defaults, while duplicate and non-finite
declarations are rejected rather than silently overwritten. The evaluator accepts numeric literals,
variables, arithmetic, parentheses, and `sin`, `cos`, and `abs`; unsupported
syntax, variables, functions, division by zero, excessive work, and non-finite
results fail with the owning source line. Per-pixel equations, external preset
loading, general shader translation, and arbitrary preset rendering remain
deferred. External preset
provenance remains a prerequisite for admitting any third-party fixture.
Regression coverage rejects unknown ambient-style identifiers and unsupported
function calls before any host evaluation could occur. The evaluator has no
file, network, device, renderer, random, or wall-clock input; that local proof
does not replace the still-unmet external compatibility-oracle requirement.
`hello-milkdrop` now emits separate artifacts for a parser-classification
fixture, an equation matrix, and a construct matrix. The equation matrix
verifies its expected scalar state before writing evidence, covering operator
precedence, parenthesized terms, selected pure functions,
initialization-to-frame state progression, and semicolon-delimited assignment
ordering. The construct matrix proves that every selected classification branch
is retained: scalar parameters and initialization/per-frame equations are
admitted; per-pixel equations are explicitly deferred; literal custom-wave and
custom-shape properties are preserved for bounded lowering; custom code,
warp/composite shaders, and unknown keys are explicitly unsupported. It is
first-party evidence for Tokimu's bounded subset, not an external compatibility
oracle.
Direct `milkdrop-tools` tests resolve all eleven admitted scalar keys and reject
non-finite scalar values plus fractional or out-of-range echo orientations.
Those tests make the selected scalar contract explicit without broadening the
first-party subset or implying third-party preset compatibility.

`MilkDropSelectedRuntime` now supplies the first bounded execution proof above
the parser. It owns no clock, audio device, filesystem, network, renderer, or
random source; callers explicitly provide a monotonically increasing semantic
frame number, time, and normalized bass/mid/treble observations. The runtime
evaluates initialization and per-frame equations, validates selected scalar
state, and lowers that state into provider-neutral phase, audio-energy, decay,
and zoom controls. `hello-audio-visualizer` executes those controls with `M`
through an original Tokimu WGSL previous-frame feedback shader and records the
preset source, parsed document, resolved parameters, frame state, shader, and
binding contract as artifacts. This proves only the Tokimu-authored selected
subset; it is neither projectM integration nor evidence that arbitrary
third-party presets are compatible.

### Slice 11: MilkDrop Waves, Shapes, Textures, And Shaders

Deliverables:

- [x] Lower one selected built-in waveform behavior into provider-neutral
      presentation geometry.
- [x] Lower bounded spectrum-bar rectangles from provider-neutral spectrum
      observations.
- [x] Render the same spectrum-bar lowering through the native quad pipeline
      (`X` in `hello-audio-visualizer`) without a renderer-specific audio
      analysis path.
- [x] Lower one original bounded radial shape from provider-neutral spectrum
      and beat observations into deterministic presentation geometry.
- [x] Resolve selected literal custom-wave section properties into bounded,
      provider-neutral descriptions without executing `wavecode` or creating
      renderer objects.
- [x] Lower selected custom-wave descriptions through a renderer-neutral point
      contract using explicit waveform or spectrum sample sources.
- [x] Render that selected custom-wave point contract in the browser consumer
      without TypeScript parsing preset source or executing `wavecode`.
- [x] Prove the same selected custom-wave point contract through an independent
      native presentation consumer before treating it as reusable capability
      evidence.
- [x] Add one selected literal convex custom-shape subset from corpus evidence:
      bounded scalar properties lower to renderer-neutral normalized polygon
      points without executing `shapecode`, selecting a fill rule, resolving a
      texture, or owning renderer resources. A `textured=1` request is rejected
      at the provider boundary until a resolved asset contract exists, rather
      than being silently rendered as a solid polygon. The untextured contract
      is asserted without audio input, serialized in the selected frame
      artifact, observed through browser WASM, and lowered independently by the
      native consumer.
- [ ] Resolve preset texture names through `tokimu-assets` and normalized image
      contracts.
- [x] Investigate bounded HLSL-compatible translation to WGSL for MilkDrop 2
      shaders by preserving shader-bearing entries as source inspections before
      attempting translation.
- [x] Preserve selected additive-blend intent and previous-frame semantics
      explicitly.
- [ ] Resolve texture wrap and filtering only through a normalized texture
      requirement contract.
- [ ] Where useful and legally permitted, compare output and structural
      artifacts against a separately installed projectM executable for selected
      presets without treating pixels as the only oracle or making projectM a
      corpus prerequisite.

Acceptance criteria:

- [ ] Each supported construct has a focused fixture and at least one real
      preset consumer.
- [ ] Missing textures and unsupported shader operations identify the owning
      provider or shader-lowering stage.
- [ ] Encoded texture formats and source paths do not leak into shaders.
- [ ] Compatibility differences are recorded per feature and preset.

Current evidence: `visualizer-tools::VisualizerWaveform` lowers normalized
waveform samples already carried by `VisualizerFrameInput` into a bounded
provider-neutral line-strip point sequence. `VisualizerSpectrumBars` lowers
the same frame's normalized spectrum into stable, non-overlapping rectangle
geometry. `VisualizerRadialShape` adds one original Tokimu-authored polygon
proof with bounded side count, radius, and audio gain. All three reject invalid
inputs before producing geometry. The
native `hello-audio-visualizer --write-artifacts` command writes deterministic
`.waveform.json`, `.spectrum-bars.json`, and `.radial-shape.json` artifacts for
every synthetic fixture beside its input and CPU preview evidence. These are
original built-in presentation proofs only: they do not admit MilkDrop
custom-wave semantics, MilkDrop custom shapes, texture resolution, shader
translation, renderer mesh ownership, or third-party preset compatibility.
The same corpus now also preserves previous-frame sampling plus selected
classic `decay` and `zoom` semantics for the Tokimu-authored MilkDrop fixture.
That execution path deliberately stops before named textures, per-pixel
equations, embedded shaders, or projectM behavior. It now
also resolves one selected literal `[wave_0]` description into bounded semantic
data (sample count, flags, color, scale, and center). Given explicit normalized
waveform or spectrum observations, `milkdrop-tools` lowers that description
into renderer-neutral point data. Both the browser consumer and the native
`hello-audio-visualizer` consume that same point contract: Canvas selects its
  own line/dot presentation, while the native corpus converts points into
  triangle strips in the consumer and overlays them over the feedback target.
  The provider still owns neither mesh generation nor blend execution. Native
per-frame point-to-mesh uploads are intentionally visible in the renderer
performance counters. The selected fixture additionally resolves one
untextured literal `[shape_0]` description into bounded convex-polygon points.
Canvas chooses a local fill/outline presentation, while the native consumer
creates a convex triangle fan only on first use or after a viewport-aspect
change. A `textured=1` request stops at the provider boundary with a
  texture-resolution diagnostic; it cannot silently become a solid polygon.
  Neither path executes `shapecode`, selects a fill rule, samples a texture, or
  owns blend execution. The native consumer now maps selected `additive=1`
  wave/shape intent through the renderer-neutral `BlendMode::Additive` pipeline
  policy; this is intentionally below the MilkDrop provider boundary. Texture
  wrapping and filtering remain deferred to the raster requirement review. The
  selected custom-shape contract proves
that untextured lowering does not depend on audio input and is serialized beside
the custom-wave evidence. This remains a first-party selected-subset proof: it
does not resolve textures or claim Canvas/native pixel equivalence.

`milkdrop-tools::inspect_shader_entries` now records every selected
warp/composite shader entry as provider-owned source evidence: pass identity,
source line, byte count, detected HLSL-like feature markers, and explicit
translation blockers. Each record labels translation as `deferred`; it does
not parse HLSL, generate WGSL, compile a shader, or create renderer objects.
The construct matrix covers function declarations, scalar/vector type spelling,
texture-sampling tokens, control flow, and a preprocessor directive so a future
lowering effort begins with explicit source facts rather than silent
approximation. Texture sampling is diagnosed as a raster-requirement dependency
under AR-0006; control flow, preprocessor directives, and HLSL translation
remain provider-side work.

### Slice 12: External projectM Compatibility Study

Deliverables:

- [x] Treat projectM as an optional, separately installed differential oracle;
      do not link, embed, copy, wrap, redistribute, or ship projectM as part of
      Tokimu or its corpus libraries.
- [x] Require any future external observation to record the exact executable
      version, invocation, input hashes, preset provenance, output assumptions,
      and comparison limitations.
- [ ] Compare Tokimu-native execution with externally captured projectM evidence
      using equivalent audio and lifecycle inputs where practical.
- [x] Prefer Tokimu-authored structural assertions and focused compatibility
      fixtures whenever an external comparison cannot be reproduced cleanly.
- [x] Require a separate Architectural Review and license/dependency review
      before any future proposal to integrate projectM code or libraries.

Acceptance criteria:

- [x] The study ends with an explicit `external-oracle-only` or `defer` finding.
- [x] A clean Tokimu checkout builds, tests, and runs its corpus without a
      projectM checkout, executable, library, header, or runtime resource.
- [x] No projectM implementation type, OpenGL object, or projectM-specific
      preset representation crosses a public Tokimu semantic boundary.
- [x] External comparison artifacts are labeled as observations rather than
      guarantees, and absence of projectM skips only those optional comparisons.
- [x] No projectM source or implementation logic is copied into Tokimu under the
      description of compatibility work.

Finding (2026-08-02): **defer; external oracle only**. Tokimu does not use,
link, wrap, embed, redistribute, or require projectM. The clean workspace build
and corpus validation have no projectM dependency. No external differential
capture has been admitted yet, so the comparison deliverable remains open and
skips without affecting Tokimu validation. Any future executable comparison is
optional observational evidence; any proposal to integrate projectM code or a
projectM library requires a separate Architectural Review and license review.

### Slice 13: Visualizer Consumer And Website Lab

Deliverables:

- [x] Build a consumer corpus with bounded synthetic-source selection,
      pause/reset, bounded parameters, diagnostics, and performance observations.
- [x] Deploy one original Tokimu visualizer as a progressive-enhancement island
      on the Tokimu website.
- [x] Execute one Tokimu-authored selected MilkDrop fixture in browser
      Rust/WASM without broadening the claim to third-party preset compatibility.
- [x] Expose the selected browser subset's execution boundary in the island:
      Rust/WASM preset evaluation and literal geometry are active; shader
      translation and texture resolution remain visibly deferred.
- [x] Keep the page useful when WASM, audio, or GPU initialization fails.
- [x] Label native, compatible, partial, and unsupported behavior visibly.
- [x] Let the public island distinguish executing selected-subset behavior from
      inspected-only shader source and deferred texture resolution.

Acceptance criteria:

- [x] The browser hosts and controls Tokimu but does not evaluate presets or
      redefine visualizer semantics in TypeScript.
- [x] Synthetic input works without microphone permission.
- [x] Browser-side resource and frame-time diagnostics remain bounded and
      inspectable.
- [x] Website evidence never claims compatibility beyond the selected corpus.

Implementation evidence, 2026-08-02:

- `corpus/consumers/tokimu-website-visualizer` provides a Rust/WASM session
  with five bounded synthetic fixture identities, deterministic frame
  advancement, pause/reset controls, and provider-neutral waveform/band/beat
  observations.
- The TypeScript adapter renders only the returned frame observation on a
  Canvas and owns DOM controls and lifecycle cleanup. It does not create an
  `AudioContext`, request microphone access, or evaluate MilkDrop presets.
- The browser session now selects either original Tokimu signal-field controls
  or `MilkDropSelectedRuntime`, which evaluates the same Tokimu-authored
  fixture as native execution from explicit synthetic frame/time/band inputs.
  Canvas consumes the returned scalar controls as a labeled browser observation
  only; it is not a native WGPU feedback-shader equivalence claim.
- `website/docs/lab/visualizer.md` and the generated website payload expose the
  proof as an explicit-activation iframe island with a useful static
  explanation when activation fails. The generated island is included in the
  website deployment.
- The selected-mode WASM snapshot carries a bounded shader-inspection summary
  from `milkdrop-tools`: source-entry count, blocker count, and the count of
  texture-sampling entries. The TypeScript island presents those Rust-produced
  facts as `not translated` and `not resolved`; it does not infer compatibility
  or parse preset source itself.
- Website contract tests and the local build verify the public boundary,
  generated payload, and bounded first-load artifact size. CI will run the
  same checks after the consumer is committed. Native feedback remains stronger
  renderer evidence; this browser proof establishes consumer composition, not
  backend equivalence.
- The standalone browser consumer retains one Canvas 2D context and reports
  separately labeled WASM startup, frame-interval, and Canvas draw-duration
  observations. These values are local browser-host evidence only and never
  stand in for native WGPU encode, submit, or GPU-completion timings.

### Slice 14: Admission Review

Deliverables:

- [x] Compare audio, visualizer, shader, texture, multipass, and renderer
      ownership against the SDD, TTSDD, and accepted ADRs.
- [x] Decide whether audio analysis, multipass presentation, or visualizer
      semantics have independent consumers sufficient for capability admission.
- [x] Record accepted and deferred MilkDrop compatibility findings.
- [x] Update architecture documents before extracting permanent crates.
- [x] Retain corpus evidence and rejected alternatives.

Acceptance criteria:

- [x] Crate extraction follows demonstrated ownership rather than this plan's
      proposed names.
- [x] Audio capture, analysis, visualizer meaning, MilkDrop compatibility, and
      renderer execution remain distinguishable.
- [x] Native and WASM guarantees are stated separately where they differ.
- [x] Remaining compatibility gaps have explicit diagnostic ownership.

Admission finding, 2026-08-02: **defer capability extraction**. The corpus
establishes bounded headless audio analysis, a second analysis consumer,
selected MilkDrop scalar and literal geometry lowering, native feedback
execution, and a browser consumer. It does not establish capture-provider
lifecycle semantics, audio-resource requirements, texture resolution,
MilkDrop shader compatibility, an external preset oracle, or a reusable
multipass executor. `visualizer-tools` and `milkdrop-tools` therefore remain
corpus libraries. See `AR-0008-audio-observation-and-visualizer-boundary.md`
Cycle 5 for the ownership decision and reopening triggers.

## Proposed Corpus Shape

Names remain provisional until the code exists:

```text
corpus/
    hello-audio-analysis/
    hello-audio-visualizer/
    hello-feedback-texture/
    hello-multipass/
    hello-milkdrop/

    lib/
        audio-analysis-tools/
        visualizer-tools/
        milkdrop-tools/

    consumers/
        tokimu-website-visualizer/
```

Do not create all entries in advance. Add each only when its slice has a
concrete acceptance proof.

## Compatibility Ladder

MilkDrop support should be reported by independently verifiable levels:

| Level | Evidence |
|---|---|
| 0: Inspectable | preset metadata and sections parse |
| 1: Equations | selected initialization and frame equations evaluate |
| 2: Classic | warp, decay, waveform, shapes, and feedback render |
| 3: Textured | external texture resolution and sampling work |
| 4: MilkDrop 2 | selected custom shader presets lower and render |
| 5: Corpus compatible | a versioned preset selection meets recorded tolerances |

No level implies compatibility with every community preset.

## Validation Matrix

| Boundary | Required evidence |
|---|---|
| Frame input | deterministic synthetic fixtures and serialization |
| Audio analysis | fixed PCM, numeric tolerances, and headless tests |
| Fullscreen shader | native render plus structural binding artifacts |
| Render target | two-pass sample-after-write corpus |
| Feedback | deterministic reset and warm steady-state counters |
| Multipass | validated graph, resource bounds, and pass timings |
| Native visualizer | three original definitions with different pass needs |
| TypeScript authoring | deterministic lowering and source diagnostics |
| MilkDrop parser | selected preset parse artifacts and malformed fixtures |
| Compatibility | Tokimu structural assertions plus optional external projectM differential evidence |
| Browser | WASM consumer with synthetic and permitted audio paths |

Preferred checks include:

```text
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check --target wasm32-unknown-unknown
npm run typecheck
```

Native and browser visual runs remain necessary. Screenshots are complementary
evidence; pass graphs, evaluated parameters, texture lifecycles, diagnostics,
and timings retain stage ownership.

## Non-Goals

The first version does not attempt:

- complete compatibility with every MilkDrop preset;
- system-wide audio loopback on every platform;
- DRM-protected or browser-forbidden audio capture;
- unrestricted runtime shader compilation from JavaScript;
- projectM code, libraries, binaries, or OpenGL integration as a Tokimu
  dependency, backend, shipped tool, or required corpus component;
- an audio player, DAW, or general media framework;
- video export or offline encoding;
- simulation changes driven implicitly by visualizer state;
- promotion of audio or visualizer implementation into `tokimu-core`; or
- redistribution of community presets without explicit provenance and license
  review.

## Risks

### Compatibility becomes architecture

MilkDrop carries historical variable, timing, shader, texture, and rendering
behavior. Keep compatibility logic in its provider and require the Tokimu-native
model to have independent consumers.

### Audio and rendering clocks drift

Capture, analysis, simulation, and presentation may advance independently.
Every observation needs timestamps and an explicit backlog/drop policy.

### Feedback hides resource churn

The effect can look correct while reallocating targets or bindings every frame.
Require warm steady-state counters and sustained-budget diagnostics.

### Shader translation scope expands without bounds

MilkDrop 2 shader compatibility may require substantial legacy-HLSL behavior.
Select corpus features deliberately, cap source and generated work, and report
unsupported operations.

### Preset packs have mixed provenance

The projectM library does not ship presets, and external packs may have their
own licensing and asset terms. Review each admitted selection independently.

### Reference implementation becomes a hidden dependency

Calling projectM a compatibility reference must not turn it into a linked
library, wrapped backend, copied implementation, required test executable, or
redistributed corpus tool. Tokimu owns its implementation and semantic
contracts. Any projectM comparison remains an optional external observation;
proposals for deeper integration require a separate Architectural Review and
license/dependency decision.

### Browser audio appears equivalent prematurely

Autoplay, microphone permissions, system-audio capture, latency, and WebGPU
availability differ materially from native execution. State guarantees per
provider and keep synthetic input first-class.

### A universal expression engine is invented too early

MilkDrop equations, TypeScript shader authoring, and future simulation scripts
may look similar while carrying different safety and lifecycle semantics. Reuse
only after repeated evidence proves a shared contract.

## Completion Criteria

The first useful version of this effort is complete when:

- one native and one browser consumer render the same Tokimu-native visualizer;
- deterministic synthetic audio drives waveform, spectrum, and beat inputs;
- offscreen, feedback, and multipass rendering run without steady-state
  resource allocation;
- at least three original Tokimu visualizers exercise distinct pass shapes;
- one legally admitted MilkDrop preset reaches at least Compatibility Level 2;
- unsupported preset features produce structured diagnostics;
- application, audio, visualizer-provider, presentation, and renderer ownership
  remain separate; and
- Architectural Review records which capabilities, if any, have earned
  admission.

## External References

- [projectM](https://github.com/projectM-visualizer/projectm) provides a mature
  open-source MilkDrop-compatible implementation and documents the combined
  preset parsing, PCM/FFT analysis, equation evaluation, and rendering problem.
- [projectM releases](https://github.com/projectM-visualizer/projectm/releases)
  provide useful compatibility history, including feedback-source, shader
  translation, waveform, texture, and coordinate-system corrections.
