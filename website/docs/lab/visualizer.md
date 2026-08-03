---
title: Tokimu Visualizer
description: A bounded Rust/WASM audio-reactive visualizer consumer corpus.
---

# Tokimu Visualizer

This laboratory island hosts original Tokimu visualizer behavior and one
bounded first-party MilkDrop scalar subset through a Rust/WASM observation
interface. It uses deterministic synthetic fixtures, so the visual proof works
without microphone permission or a media device.

```text
synthetic fixture
        ↓
Rust/WASM visualizer session
        ↓
frame observation + waveform geometry
        ↓
TypeScript controls + Canvas presentation
```

The browser does not evaluate presets, perform audio analysis, or own
visualizer time. The optional selected MilkDrop mode is evaluated inside the
Rust/WASM session against explicit frame, time, bass, mid, and treble inputs.
For the selected fixture, Rust/WASM also lowers literal custom-wave properties,
explicit audio samples, and one untextured convex custom shape into point data.
Canvas renders that returned data without parsing preset source or executing
`wavecode` or `shapecode`. A MilkDrop shape requesting a texture remains an
explicit provider diagnostic until texture resolution has a separate contract.

When the selected MilkDrop mode is active, the island's observation grid names
the executing Rust/WASM subset, its first-party fixture, and the count of
returned literal waves and shapes. It separately labels shader translation as
inspection-only and texture resolution as deferred. Those labels are part of
the evidence: they prevent a visual result from being mistaken for general
MilkDrop shader or texture compatibility.

The island exposes separately labeled browser-host observations for WASM startup,
frame interval, Canvas draw duration, and its single retained Canvas 2D context.
They describe this browser consumer only; native renderer timing remains separate
evidence.

<section
  class="island-stage visualizer-island"
  data-tokimu-island="tokimu-visualizer"
  data-state="idle"
  aria-labelledby="tokimu-visualizer-title"
>
  <div class="island-fallback">
    <p class="eyebrow">Experimental consumer evidence / synthetic input</p>
    <h2 id="tokimu-visualizer-title">Signal field</h2>
    <p>
      Activate the Tokimu visualizer. Selectable deterministic sources exercise
      fixed-step signal observation, waveform lowering, bands, beat state, and
      one bounded first-party MilkDrop scalar execution mode without requiring
      audio capture.
    </p>
    <p>
      General MilkDrop compatibility, microphone capture, external presets, and
      native feedback-shader equivalence remain explicitly outside this browser
      proof.
    </p>
    <button class="button button-primary" type="button" data-island-action="activate">
      Open visualizer
    </button>
    <button class="button button-secondary" type="button" data-island-action="reset" hidden>
      Close visualizer
    </button>
  </div>
  <div class="island-mount" data-island-mount hidden></div>
  <div class="island-status" role="status" aria-live="polite">
    <span data-island-status-state>Idle</span>
    <span data-island-status-detail>Synthetic visualizer session not loaded</span>
  </div>
  <script type="application/json" data-island-config>
    { "schema": 1, "activation": "explicit" }
  </script>
</section>

## Limits

- This includes one first-party, selected MilkDrop scalar subset; it is not a
  general MilkDrop compatibility claim.
- Synthetic sources are intentional and do not establish microphone support.
- Canvas output is presentation evidence, not a native feedback-shader or
  renderer-equivalence oracle.
- The corpus retains native visualizer and structural artifacts for deeper
  provider and performance validation.
