# Tokimu Website Visualizer

## Purpose

This consumer corpus proves that a browser can host one original Tokimu
visualizer through a bounded WASM interface without reimplementing fixture
generation, audio observation, waveform lowering, or preset evaluation in
TypeScript.

## Ownership

- Rust/WASM owns deterministic fixture selection, fixed-step progression,
  analysis observations, and waveform lowering.
- TypeScript owns DOM controls, Canvas presentation, focus, and failure UI.
- The browser owns canvas pixels and animation scheduling, not visualizer or
  preset semantics.
- This consumer uses no microphone permission, external preset, or audio
  device provider.

## Current Scope

- Five bounded synthetic audio fixtures.
- Pause, reset, and fixture-selection controls.
- A provider-neutral waveform plus band/beat observation.
- Explicit diagnostics that describe the missing microphone and preset paths.

It is not a claim of MilkDrop compatibility or browser audio-capture support.
