# AR-0008: Audio Observation And Visualizer Boundary

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-08-01 |
| Last reviewed | 2026-08-02 |
| Scope | Foundational audio-analysis, presentation, renderer, and provider boundary |
| Trigger | The audio-visualizer corpus now transforms bounded PCM windows into deterministic provider-neutral observations without an audio device, window, or GPU. |
| Related ADRs | ADR-0001, ADR-0003, ADR-0007 |
| Related evidence | `visualizer-tools`, `hello-audio-visualizer`, `hello-audio-analysis`, synthetic PCM fixtures, generated PCM16 WAVE byte fixtures, bounded backlog tests, and native-host timing observations |
| Admission exception | None |

## Architectural Question

Where should Tokimu separate audio acquisition, provider-neutral audio
observations, visualizer semantics, and renderer execution without admitting an
audio device, a preset language, or a renderer mechanism into the wrong layer?

This review also compares the audio path with AR-0006's
intent-to-requirement-to-execution observation. It does not assume that audio
proves a reusable universal `Requirement` abstraction.

## Context

The visualizer corpus accepts explicit normalized PCM windows and creates
deterministic waveform, spectrum, named-band, and onset observations. Its
reference analysis is deliberately headless:

```text
native device / browser callback / file decoder
        -> provider-specific PCM delivery
        -> bounded PCM window
        -> provider-neutral analysis observation
        -> visualizer input
        -> presentation-pass intent
        -> renderer execution
```

The first corpus visualizer uses synthetic inputs only. It does not open an
audio device, decode media, sample an audio texture, execute a multipass graph,
or parse MilkDrop presets.

## Trigger And Evidence

- Corpus library: `visualizer-tools::PcmAudioWindow` accepts only finite,
  normalized mono or stereo PCM within explicit frame and channel limits.
- Deterministic analysis: `PcmAnalyzer` uses a documented Hann window and
  `direct-dft-magnitude-v1` reference algorithm to produce waveform, spectrum,
  named bands, and onset observations from fixed input and caller-owned prior
  state.
- Fixtures: silence, impulse, mono tone, and stereo tone fixtures exercise
  deterministic results, clipping boundaries, and stereo-to-mono conversion.
- Overload evidence: `PcmAnalysisBacklog` uses bounded capacity with explicit
  `drop-oldest` or `drop-newest` policy and visible dropped-window counts. It
  is not an audio-device ring buffer.
- Timing evidence: `hello-audio-visualizer --pcm-measure` runs a fixed bounded
  reference workload and labels host timings as observations, not portable
  performance guarantees or allocation claims.
- Byte-source evidence: `decode_pcm16_wav` accepts only bounded canonical
  PCM16 little-endian RIFF/WAVE fixture bytes, validates RIFF metadata and
  frame alignment, and adapts them into the already provider-neutral
  `PcmAudioWindow`. Generated fixtures round-trip through this seam before
  analysis, including a benign aligned unknown RIFF chunk; it does not claim
  general file decoding or playback.
- Second-consumer evidence: `hello-audio-analysis` independently consumes the
  generated PCM16 WAVE bytes, adapts them into `PcmAudioWindow`, and writes
  source, analysis, timing, and working-set artifacts without a renderer,
  window, audio device, or playback path.
- Compatibility reference evidence: projectM and the MilkDrop authoring guide
  confirm that compatibility joins PCM analysis, preset equations, custom
  waves/shapes, pixel shaders, and renderer feedback. This supports the
  existing separation; it does not admit projectM, a preset pack, or any
  MilkDrop execution contract.
- Presentation evidence: the visualizer consumes already-produced audio
  observations and explicit time; it neither obtains PCM nor mutates world
  state.
- Missing evidence:
  - native and browser PCM providers mapped into the same input contract;
  - sample-rate adaptation, timestamp provenance, underrun, overrun, and
    disconnection semantics;
- a stable visualizer model used by a second independent consumer;
  - render-target sampling, feedback, or multipass execution;
  - audio-derived texture or uniform requirements at a shader boundary;
  - media decoding, playback, spatial audio, and permission policy;
  - allocator observations and a justified performance budget.

## Ownership Analysis

### Audio input providers

Native devices, browser callbacks, file decoders, and host integrations own
their platform mechanisms, permissions, buffering, sample-rate facts, and
source-specific failures. They may produce bounded PCM windows and diagnostics.

They must not own visualizer equations, shader bindings, render passes, or
simulation truth.

### Audio analysis

Audio analysis owns deterministic transformation of explicitly supplied PCM
into bounded observations. It owns analysis configuration, windowing,
channel-mixing policy, and observation diagnostics.

It does not own device callbacks, a platform clock, output playback, a GPU,
or rendering resources. The current implementation remains corpus-side until
another independent consumer or provider proves a stable capability boundary.

### Visualizer model

The visualizer model owns which named observations, explicit time, parameters,
and pass descriptions a visualization uses. It may reference a resolved audio
observation but does not own raw provider buffers, devices, or backend texture
objects.

### Renderer

The renderer owns shader execution, render targets, bind groups, sampling, and
backend timing. A renderer may consume resolved visualizer data, but it must
not infer PCM format, acquire audio, or define analysis policy.

### Application

The application owns source selection, permission policy, playback and
transition policy, and whether an audio-driven visualization has any simulation
meaning. Presentation remains an observer unless an application explicitly
feeds a derived command back into its world.

## Dependency Direction

```text
Current corpus direction:

synthetic PCM
    -> corpus audio analysis
    -> VisualizerFrameInput
    -> corpus shader/material bridge
    -> tokimu-render adapter

Candidate provider direction:

platform audio provider
    -> bounded PCM delivery
    -> provider-neutral analysis observation
    -> visualizer semantic input
    -> renderer-facing resolved parameters/resources
    -> backend execution
```

Rules under review:

- semantic analysis may not depend on native device, browser, file, or GPU
  objects;
- provider-native capture objects must stop before analysis and visualizer
  contracts;
- visualizers consume observations rather than platform audio objects;
- renderer resources and shader bindings remain below visualizer semantics;
- `tokimu-core` and `tokimu-runtime` do not acquire audio-device or media
  decoder dependencies from this work.

## Alternatives Considered

### A: Make Audio Capture The Visualizer Contract

- Benefits: a direct demo path from microphone to pixels.
- Costs: device permissions, callback lifetime, browser behavior, and capture
  buffering leak through presentation.
- Failure mode: native and WASM visualizers diverge before analysis semantics
  are established.

### B: Promote The Current PCM Analyzer Immediately

- Benefits: shared code and a named audio subsystem now.
- Costs: one corpus reference algorithm and synthetic fixtures would freeze an
  immature API.
- Failure mode: a future provider needs timestamp, sample-rate, allocation, or
  loss semantics the initial contract did not represent.

### C: Let Each Visualizer Analyze Its Own PCM

- Benefits: no shared analysis vocabulary.
- Costs: repeated channel mixing, windowing, overflow, and diagnostic policy.
- Failure mode: visualizers become accidental audio providers and cannot be
  compared headlessly.

### D: Incubate Provider-Neutral Analysis And Visualizer Inputs In The Corpus

- Benefits: preserves deterministic evidence while capture, playback,
  multipass rendering, and compatibility semantics remain separable.
- Costs: temporary corpus-local types and intentionally limited reuse.
- Failure mode: independent consumers duplicate the same contracts before the
  review can decide whether promotion is justified.

## Findings

The evidence supports these provisional findings:

1. Raw PCM delivery, provider-neutral analysis observations, visualizer
   semantics, and renderer execution are distinct responsibilities.
2. Audio analysis can be headless and deterministic when it consumes explicit
   bounded windows rather than a live device.
3. Loss policy is semantic diagnostic evidence at the PCM-delivery boundary;
   it must not be an invisible side effect of an eventual device callback.
4. A bounded reference DFT is valuable correctness evidence but is not a
   throughput, allocation, or production FFT guarantee.
5. Audio presently resembles AR-0006's staged handoff pattern, but the shared
   vocabulary is not yet proven beyond a broad architectural observation.

The evidence does not establish:

- an admitted audio capability, crate, or public API;
- a capture, playback, or media-decoder abstraction;
- a universal requirement-resolution service;
- audio texture, sampler, or shader-binding semantics;
- multipass/feedback renderer ownership;
- MilkDrop compatibility or preset execution.

## Disposition

Incubating. Keep audio analysis and visualizer-input contracts in
`corpus/lib/visualizer-tools` while a native or browser provider and a second
consumer test whether their ownership and diagnostic vocabulary hold. Do not
promote audio analysis, introduce `tokimu-audio`, or revise AR-0006 into a
universal requirement model from this evidence alone.

## Consequences

The visualizer work may continue with deterministic synthetic input and bounded
analysis artifacts. Future capture providers must adapt to the existing
corpus-side seam or explicitly demonstrate why its information is insufficient.
Any device failure, permission denial, timestamp discontinuity, underrun, or
overrun must be diagnosed by the owning provider boundary.

## Required Follow-Up

- [x] Record deterministic PCM-analysis and backlog evidence.
- [x] Record separately labeled bounded native-host timing observations.
- [x] Add one deterministic generated PCM16 WAVE byte source adapter.
- [ ] Add one browser/WASM PCM provider after its permission and lifecycle
      behavior can be recorded honestly.
- [ ] Add sample-rate, timestamp, underrun, and disconnection diagnostics.
- [x] Exercise analysis observations through a second consumer.
- [ ] Determine whether resolved audio observations need a shader resource
      contract without conflating them with raster-image requirements.
- [ ] Reassess capability admission only after the named evidence exists.

## Reopening Triggers

Reopen or advance this review when:

- a second independent consumer needs the same provider-neutral analysis
  observations;
- a native or WASM provider cannot map into the bounded PCM-window contract;
- sample-rate, timestamp, or overflow semantics require a materially different
  ownership split;
- a renderer or shader requires audio resource semantics that duplicate or
  contradict AR-0006;
- a capture provider leaks into visualizer or simulation contracts;
- a simpler decomposition becomes available.

## Review History

### Cycle 1 -- 2026-08-01

- Status entering review: Proposed
- New evidence: deterministic PCM windows and reference DFT analysis, fixed
  synthetic fixtures, explicit bounded backlog policies, separately labeled
  native-host timings, and a headless visualizer input path.
- Participants or reviewers: Arakendo, Codex working review
- Findings: acquisition, analysis, visualizer semantics, and renderer
  execution remain distinct; current evidence is sufficient for incubation but
  not capability admission.
- Disposition: Incubating
- Resulting ADR or documentation change: AR-0008 opened; no ADR or crate
  admission.

### Cycle 2 -- 2026-08-01

- Status entering review: Incubating
- New evidence: a bounded generated PCM16 RIFF/WAVE byte adapter, explicit
  malformed-container diagnostics, an aligned unknown-chunk regression case,
  source/provenance artifacts, and a documentation-only projectM/MilkDrop
  construct inventory.
- Participants or reviewers: Arakendo, Codex working review
- Findings: a source byte container can stop at the provider-neutral PCM seam;
  WAVE container metadata and MilkDrop/provider specifics remain outside the
  analysis contract. Compatibility reference material reinforces, rather than
  replaces, the proposed ownership split.
- Disposition: Incubating
- Resulting ADR or documentation change: no ADR or crate admission; the
  visualizer plan records the external-reference review and prerequisite preset
  provenance requirements.

### Cycle 3 -- 2026-08-02

- Status entering review: Incubating
- New evidence: three provider-neutral structural pass graphs, three original
  visualizer definitions, a renderer-local ping-pong feedback execution proof,
  explicit resize/reset lifecycle diagnostics, a warm native frequency-sweep
  observation with zero frame-local resource churn after frame 120, and a
  runtime material-uniform regression test.
- Participants or reviewers: Arakendo, Codex working review
- Findings: a pass graph can remain structural while concrete WGPU execution
  owns targets, views, bind groups, and submission order. The live feedback
  proof also showed that a CPU-side material-write observation is insufficient
  unless the backend buffer is allocated for runtime copy writes; the required
  `UNIFORM | COPY_DST` usage remains renderer-local execution policy.
- Disposition: Incubating
- Resulting ADR or documentation change: no ADR or crate admission. The plan
  records steady-state feedback evidence, explicit manual capture guidance,
  and a direct surface-only graph fixture. Real audio providers, a second
  consumer, and any provider-neutral multipass execution contract remain
  required before capability admission.

### Cycle 4 -- 2026-08-02

- Status entering review: Incubating
- New evidence: `hello-audio-analysis` independently exercises generated
  PCM16 RIFF/WAVE bytes through the bounded source adapter, `PcmAudioWindow`,
  reference analysis, timing observation, and source-structural working-set
  observation. It writes one provenance-bearing inspection artifact per
  fixture without a window, renderer, audio device, or playback mechanism.
- Participants or reviewers: Arakendo, Codex working review
- Findings: the source-byte-to-analysis handoff is reusable outside the
  visualizer and remains understandable as a headless contract. This is
  evidence for the existing ownership split, not evidence that capture or a
  permanent audio capability has stabilized.
- Disposition: Incubating
- Resulting ADR or documentation change: no ADR or crate admission. The
  second-consumer follow-up is complete; native/browser provider behavior,
  timestamp and loss semantics, and a second consumer of the visualizer model
  remain required.

## References

- `docs/Plans/audio-reactive-visualizers-and-milkdrop-compatibility.md`
- `docs/Architectural Reviews/AR-0006-raster-image-requirement-pipeline.md`
- `docs/ADR/ADR-0003-capability-ownership-boundary.md`
- `docs/ADR/ADR-0007-kernel-performance-diagnostics.md`
- `corpus/lib/visualizer-tools/src/audio_analysis.rs`
- `corpus/lib/visualizer-tools/src/lib.rs`
- `corpus/hello-audio-analysis/src/main.rs`
- `corpus/hello-audio-visualizer/src/main.rs`
- https://github.com/projectM-visualizer/projectm
- https://milkdrop.org/resources/preset-authoring
