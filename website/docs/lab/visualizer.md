---
title: Tokimu Visualizer
description: A bounded Rust/WASM audio-reactive visualizer consumer corpus.
---

# Tokimu Visualizer

This laboratory island hosts an original Tokimu visualizer through a bounded
Rust/WASM observation interface. It uses deterministic synthetic fixtures, so
the visual proof works without microphone permission or a media device.

```text
synthetic fixture
        ↓
Rust/WASM visualizer session
        ↓
frame observation + waveform geometry
        ↓
TypeScript controls + Canvas presentation
```

The browser does not evaluate MilkDrop presets, perform audio analysis, or own
visualizer time. Canvas renders the returned observation only.

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
      Activate the original Tokimu visualizer. Selectable deterministic sources
      exercise fixed-step signal observation, waveform lowering, bands, and beat
      state without requiring audio capture.
    </p>
    <p>
      MilkDrop compatibility, microphone capture, and external presets remain
      explicitly outside this browser proof.
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

- This is one original visualizer, not a MilkDrop compatibility claim.
- Synthetic sources are intentional and do not establish microphone support.
- Canvas output is presentation evidence, not a renderer-equivalence oracle.
- The corpus retains native visualizer and structural artifacts for deeper
  provider and performance validation.
