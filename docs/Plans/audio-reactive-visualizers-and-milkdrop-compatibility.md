# Audio-Reactive Visualizers And MilkDrop Compatibility

## Status

Implementation started on 2026-08-01. Deterministic visualizer observations
and a single-pass native corpus application now incubate under `corpus/`.
No MilkDrop parser, preset evaluator, audio analyzer, feedback renderer, or
visualizer capability is currently admitted.

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
- renderable offscreen textures;
- sampling a render target in a later pass;
- ping-pong previous-frame feedback;
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
- [x] Review projectM as a compatibility reference and possible optional
      adapter, not as Tokimu's architectural model.
- [ ] Select only presets and texture assets with recorded provenance,
      redistribution terms, upstream revision, and hashes.
- [x] Define maturity labels: `native`, `compatible`, `partial`, `unsupported`,
      and `invalid`.
- [x] Open `AR-0008` before admitting audio-analysis or multipass presentation
      as a permanent Tokimu capability.

Initial reference review, 2026-08-01:

- [x] Record projectM as an optional compatibility reference, not a Tokimu
      dependency or architectural template. Its public project description
      combines preset parsing, PCM analysis, beat detection, and OpenGL
      rendering, which reinforces the need to keep those responsibilities
      separate in Tokimu. The core library is LGPL-2.1; preset packs are
      separately distributed and require their own provenance review.
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

This review admits neither projectM code nor external `.milk` presets. Before
any preset becomes a fixture, record its exact upstream repository, revision,
author/license information, redistribution terms, hash, intended construct,
and expected maturity label.

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
- [ ] Keep native-window screenshots as separately labeled manual evidence.
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
- [ ] Support sampling one completed offscreen target in a later pass.
- [ ] Add resize, replacement, release, and resource-bound diagnostics.
- [x] Distinguish source textures, render targets, and surface images.

Acceptance criteria:

- [ ] A two-pass corpus renders pass A offscreen and samples it in pass B.
- [x] No pass samples a target while that target is being written.
- [ ] Resize behavior is deterministic and reports allocation/replacement
      counters.
- [ ] Render-target resources remain renderer-private.

Current evidence: `visualizer-tools::VisualizerPassGraph` serializes a
validated two-pass `signal -> present` graph to
`target/audio-visualizer/two-pass-signal.graph.json`. Its preflight rejects
missing targets, read-before-write ordering, same-pass read/write hazards,
invalid target dimensions, and non-final surface writes before backend
allocation or submission. It additionally rejects sampling a target declared
`not-sampled` and writing one target more than once in a graph. This is
structural corpus evidence, not yet an offscreen renderer implementation or a
stable cross-capability API.

`tokimu-render::WgpuBackend::create_render_target_rgba8` now allocates an
opaque, sampleable RGBA8 target with render-attachment usage from the existing
`TextureHandle` identity. The renderer rejects source-pixel updates to a
renderer-owned target, preserving the source-texture versus render-target
distinction at the adapter boundary. The corpus command
`hello-audio-visualizer --offscreen-probe` successfully allocated a headless
640x360 sRGB target on 2026-08-01, including a repeat verification after the
PCM16 WAVE fixture work. This proves allocation only: no pass has yet rendered
into the target, sampled it, resized it, released it, or claimed GPU framebuffer
equivalence.

Target replacement is intentionally not treated as an ordinary texture upload:
uploaded materials cache backend bind groups built from a texture view. Replacing
a target would therefore require explicit material dependency invalidation and
rebinding. `WgpuBackend::try_upload_texture` now rejects replacement of a
renderer-owned target, and the historical infallible upload helper delegates to
that check. Resize and release remain deferred until this dependency lifecycle
is represented deliberately.

Execution boundary finding: the current `RenderCommand` stream can clear and
draw only to the presentation surface. A real offscreen pass would need an
explicit renderer-owned pass target, attachment/load policy, and material
rebinding lifecycle. That is larger than target allocation and has no second
consumer yet, so the corpus keeps validating provider-neutral graph intent
instead of introducing a speculative multipass command API.

### Slice 4: Previous-Frame Feedback

Deliverables:

- [ ] Add a bounded ping-pong target pair with explicit current and previous
      frame roles.
- [ ] Define first-frame initialization and reset semantics.
- [ ] Add decay, warp, and blend parameters to the visualizer corpus.
- [ ] Preserve feedback across ordinary frames and reset it on explicit preset,
      size, or lifecycle transitions.
- [ ] Capture structural pass graphs and separately labeled image evidence.

Acceptance criteria:

- [ ] No frame reads and writes the same texture subresource.
- [ ] Reset and first-frame output are deterministic.
- [ ] Feedback survives steady-state rendering without CPU texture readback or
      per-frame resource allocation.
- [ ] The corpus diagnoses unsupported feedback formats or sampling features.

### Slice 5: Provider-Neutral Multipass Description

Deliverables:

- [ ] Describe named passes, inputs, outputs, dependencies, and final surface
      output as bounded data.
- [ ] Validate acyclic pass ordering and resource compatibility before backend
      execution.
- [ ] Keep pipeline selection explicit per pass.
- [ ] Add bounded intermediate-target and pass-count limits.
- [ ] Expose pass timings and resource counts through diagnostics.

Acceptance criteria:

- [ ] One corpus visualizer uses at least three passes, including feedback and
      final compositing.
- [ ] Cycles, missing outputs, incompatible formats, and excessive graphs fail
      before backend submission.
- [ ] The graph contains no backend-native resources or command objects.
- [ ] A single-pass shader remains straightforward and does not require a
      ceremonial graph in application code.

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
- [x] Measure bounded native-host analysis latency under representative fixed
      windows without treating it as a cross-machine performance contract.
- [ ] Establish a portable allocation-observation method before making any
      allocation claim for audio analysis.
- [ ] Compare the resulting requirement-resolution boundary with
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
observation. It records workload shape and algorithm identity but does not
declare a portable performance budget or allocation result. The same timing
observations are written beside fixed analysis artifacts by
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

### Slice 7: Native And Browser Audio Providers

Deliverables:

- [ ] Add one native PCM source provider and one browser/WASM provider only
      after the analysis contract stabilizes.
- [x] Support a deterministic generated PCM16 WAVE source before microphone or
      system-loopback capture.
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

- [ ] Define a compact model for parameters, equations or expressions, passes,
      waves, shapes, textures, and transitions.
- [ ] Decide whether the first equation form is a bounded interpreter, an
      ahead-of-time lowering input, or explicit Rust data based on corpus
      pressure.
- [ ] Provide a small built-in library of original Tokimu visualizers.
- [ ] Preserve stable parameter identity for UI controls and automation.
- [ ] Add deterministic serialization and validation.

Acceptance criteria:

- [ ] At least three visually distinct original visualizers share the same
      frame-input and pass contracts.
- [ ] One visualizer uses no feedback, one uses feedback, and one uses multiple
      passes.
- [ ] Invalid expressions, missing inputs, and excessive work fail within
      documented bounds.
- [ ] The model contains no MilkDrop parser objects or backend-native handles.

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

- [ ] Parse a deliberately selected MilkDrop 1 subset before MilkDrop 2 shader
      compatibility.
- [ ] Preserve source sections, variable names, equation order, and locations
      for diagnostics.
- [ ] Implement bounded initialization, per-frame, and per-pixel equation
      semantics required by selected presets.
- [ ] Define compatibility defaults explicitly rather than inferring them from
      successful rendering.
- [ ] Reject unknown, invalid, and excessive preset constructs deterministically.

Acceptance criteria:

- [ ] Parser artifacts are deterministic and retain enough provenance to map a
      diagnostic back to a preset line or section.
- [ ] Equation fixtures match a documented compatibility oracle for selected
      inputs.
- [ ] Preset evaluation cannot access files, network, process state, or ambient
      randomness.
- [ ] Unsupported MilkDrop 2 shader sections are labeled `unsupported`, not
      silently ignored.

### Slice 11: MilkDrop Waves, Shapes, Textures, And Shaders

Deliverables:

- [ ] Lower selected built-in waveform behavior into Tokimu presentation data.
- [ ] Add custom wave and shape semantics incrementally from corpus evidence.
- [ ] Resolve preset texture names through `tokimu-assets` and normalized image
      contracts.
- [ ] Investigate bounded HLSL-compatible translation to WGSL for MilkDrop 2
      shaders.
- [ ] Preserve blend, wrap, filtering, and previous-frame semantics explicitly.
- [ ] Compare output and structural artifacts against projectM for selected
      presets without treating pixels as the only oracle.

Acceptance criteria:

- [ ] Each supported construct has a focused fixture and at least one real
      preset consumer.
- [ ] Missing textures and unsupported shader operations identify the owning
      provider or shader-lowering stage.
- [ ] Encoded texture formats and source paths do not leak into shaders.
- [ ] Compatibility differences are recorded per feature and preset.

### Slice 12: Optional projectM Adapter Study

Deliverables:

- [ ] Evaluate libprojectM as an optional native backend or differential oracle.
- [ ] Record OpenGL-context, render-to-texture, audio-ingestion, build, license,
      native packaging, and WASM implications.
- [ ] Keep projectM behind an adapter boundary if a proof is built.
- [ ] Compare the adapter with Tokimu-native execution using the same application
      audio and lifecycle policy where possible.

Acceptance criteria:

- [ ] The study ends with an explicit `adopt`, `oracle-only`, or `defer` finding.
- [ ] Tokimu remains usable without projectM.
- [ ] No OpenGL object or projectM preset type crosses a public Tokimu semantic
      boundary.
- [ ] An adapter is not presented as native/WASM parity unless both are proven.

### Slice 13: Visualizer Consumer And Website Lab

Deliverables:

- [ ] Build a consumer corpus with preset selection, audio-source selection,
      pause/reset, bounded parameters, diagnostics, and performance observations.
- [ ] Publish one original Tokimu visualizer as a progressive-enhancement island
      on the Tokimu website.
- [ ] Add MilkDrop presets only after redistribution and browser execution are
      proven.
- [ ] Keep the page useful when WASM, audio, or GPU initialization fails.
- [ ] Label native, compatible, partial, and unsupported behavior visibly.

Acceptance criteria:

- [ ] The browser hosts and controls Tokimu but does not evaluate presets or
      redefine visualizer semantics in TypeScript.
- [ ] Synthetic input works without microphone permission.
- [ ] Resource and frame-time diagnostics remain bounded and inspectable.
- [ ] Website evidence never claims compatibility beyond the selected corpus.

### Slice 14: Admission Review

Deliverables:

- [ ] Compare audio, visualizer, shader, texture, multipass, and renderer
      ownership against the SDD, TTSDD, and accepted ADRs.
- [ ] Decide whether audio analysis, multipass presentation, or visualizer
      semantics have independent consumers sufficient for capability admission.
- [ ] Record accepted and deferred MilkDrop compatibility findings.
- [ ] Update architecture documents before extracting permanent crates.
- [ ] Retain corpus evidence and rejected alternatives.

Acceptance criteria:

- [ ] Crate extraction follows demonstrated ownership rather than this plan's
      proposed names.
- [ ] Audio capture, analysis, visualizer meaning, MilkDrop compatibility, and
      renderer execution remain distinguishable.
- [ ] Native and WASM guarantees are stated separately where they differ.
- [ ] Remaining compatibility gaps have explicit diagnostic ownership.

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
| Compatibility | projectM differential evidence plus structural assertions |
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
- projectM or OpenGL as a required Tokimu dependency;
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
