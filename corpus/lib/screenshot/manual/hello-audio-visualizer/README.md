# Hello Audio Visualizer Manual Evidence

This directory holds optional native-window captures for
`hello-audio-visualizer`. They complement deterministic input, CPU signal, and
structural graph artifacts written under `target/audio-visualizer/`; they do
not replace them.

## Capture Procedure

1. Run the visualizer with the changing default fixture.

   ```powershell
   cargo run -p hello-audio-visualizer
   ```

2. Capture one window for each native route after the scan marker has visibly
   advanced.

   ```text
   Q  signal-field
   W  feedback-bloom
   E  signal-composite
   ```

3. If practical, capture without a cursor and record the observed backend and
   window dimensions.
4. Store the image and companion manifest in this directory.

Suggested names:

```text
frequency-sweep-signal-field.png
frequency-sweep-feedback-bloom.png
frequency-sweep-signal-composite.png
frequency-sweep-feedback-bloom.manual.manifest
```

Each manifest must include:

```text
kind=manual-native-window
example=hello-audio-visualizer
case=frequency-sweep-feedback-bloom
fixture=frequency-sweep
visualizer=feedback-bloom
platform=windows
backend=<observed backend>
window_dimensions=<observed dimensions>
cursor_included=false
authoritative=false
gpu_framebuffer_capture=false
```

These captures can establish that a person observed a selected native route and
its moving scan marker. They do not prove GPU framebuffer contents, browser or
WASM behavior, audio-device behavior, or a portable performance result.
