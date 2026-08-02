# Hello Audio Analysis

## Purpose

`hello-audio-analysis` is a headless corpus consumer for the bounded PCM
analysis contract incubating in `visualizer-tools`.

It proves this semantic handoff without a window, GPU, playback engine, audio
device, or visualizer shader:

```text
generated PCM16 WAVE bytes
        -> WAVE source adapter
        -> PcmAudioWindow
        -> PcmAnalyzer
        -> structural analysis evidence
```

## What It Proves

- A second consumer can use the same PCM analysis contract independently of
  `hello-audio-visualizer`.
- The PCM16 WAVE source adapter remains separate from analysis and can be
  inspected through source provenance artifacts.
- Analysis timing and source-structural working-set observations remain
  renderer-free and explicitly bounded.

## Non-Goals

This corpus does not prove:

- live audio capture, playback, or media decoding;
- browser permissions, autoplay, or device lifecycle;
- a native audio provider;
- GPU, shader, or visualizer behavior; or
- admission of `tokimu-audio` or a permanent audio-analysis capability.

## Artifact Contract

`cargo run -p hello-audio-analysis -- --write-artifacts` writes one JSON
artifact per generated fixture under `target/hello-audio-analysis/`.

Each artifact records source identity, bounded decoded-window facts, analysis,
timing, and working-set observations. Timings are host observations, not
portable guarantees. The working set is a source-structural model, not an
allocator profile.
