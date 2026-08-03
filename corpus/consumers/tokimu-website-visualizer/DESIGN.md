# Tokimu Website Visualizer

## Purpose

This consumer corpus proves that a browser can host original Tokimu visualizer
behavior and one bounded, first-party MilkDrop scalar subset through a WASM
interface without reimplementing fixture generation, audio observation,
waveform lowering, or preset evaluation in TypeScript.

## Ownership

- Rust/WASM owns deterministic fixture selection, fixed-step progression,
  analysis observations, waveform lowering, and selected MilkDrop scalar
  evaluation from explicit semantic frame/time/band inputs.
- TypeScript owns DOM controls, Canvas presentation, focus, and failure UI.
- The browser owns Canvas pixels and animation scheduling, not visualizer or
  preset semantics.
- This consumer uses no microphone permission, external preset, or audio
  device provider.

## Current Scope

- Five bounded synthetic audio fixtures.
- Pause, reset, and fixture-selection controls.
- Original and `milkdrop-selected` execution-mode controls.
- A provider-neutral waveform plus band/beat observation.
- Optional selected scalar controls (`phase`, `audio energy`, `decay`, and
  `zoom`) plus selected literal custom-wave point samples returned from
  Rust/WASM for browser presentation.
- Explicit diagnostics that distinguish the bounded first-party evaluator from
  missing microphone and external-preset paths.

It is not a claim of general MilkDrop compatibility, projectM integration,
browser audio-capture support, Canvas/native feedback-renderer equivalence, or
browser parsing or execution of MilkDrop `wavecode`.
