# Hello Audio Visualizer

## Purpose

`hello-audio-visualizer` is the first executable corpus proof for deterministic
visualizer input and a single fullscreen WGSL pass. It deliberately uses
synthetic observations rather than an audio device, decoded file, or MilkDrop
preset.

## Primary Proof

```text
explicit frame index and viewport
        |
        v
deterministic synthetic audio observation
        |
        v
provider-neutral shader binding declaration
        |
        v
one fullscreen renderer pass
```

The application owns fixture selection, pause, reset, and time-scale policy.
`visualizer-tools` owns bounded deterministic observations. Tokimu presentation
owns the shader declaration. The WGPU adapter owns compilation and pixels.

## Current Binding Limitation

The semantic material model can describe `Vector4`, but the current renderer
execution material only uploads its legacy four-float color slot. This corpus
therefore packs `[phase, low, mid, high]` into that existing slot and declares
the intended `Vector4` binding in the shader module.

This is transitional evidence, not a claim that visualizer data is color. The
example records per-frame material replacement pressure so a future arbitrary
runtime material-parameter path can be justified and measured honestly.

## Controls

- `Left` / `Right`: select a synthetic input fixture
- `Space`: pause or resume visualizer time
- `Up` / `Down`: change visualizer time scale
- `R`: reset visualizer time

## Diagnostic Fixture

Run:

```text
cargo run -p hello-audio-visualizer -- --structural-fixture
```

This produces serialized observations for all five deterministic fixtures
without creating a window, audio device, or GPU backend.

To write review artifacts instead:

```text
cargo run -p hello-audio-visualizer -- --write-artifacts
```

This writes one input JSON file, CPU signal preview BMP, and manifest per
fixture under `target/audio-visualizer/`. It also writes the WGSL source and a
provider-neutral shader/material binding contract record, plus a separately
labeled fixed-workload native-host timing observation for every PCM fixture.
The timing observation is evidence for the current machine only, not a portable
performance budget. The preview is explicitly not a GPU framebuffer capture
or an assertion of backend pixel equivalence. Native-window screenshots remain
manual evidence.

To exercise the bounded byte-source adapter without opening a file or audio
device:

```text
cargo run -p hello-audio-visualizer -- --wav-fixtures
```

The adapter accepts only generated PCM16 little-endian RIFF/WAVE fixture bytes,
then produces the existing provider-neutral `PcmAudioWindow`. It is evidence
for the source-bytes-to-analysis seam, not a playback, capture, or general
media-decoding API. `--write-artifacts` also writes the generated WAVE files,
decoded analyses, and provenance records under `target/audio-visualizer/`.

Before native pipeline registration, the corpus preflights its declared
`Vector4` visualizer signal against the material definition and fullscreen
quad. This makes declaration and draw-contract failures observable before
backend compilation. Current frame submission still uses the renderer's legacy
four-float execution material slot, so this preflight does not claim general
`Vector4` execution support yet.

## Non-Goals

- real audio capture or playback
- FFT or beat-detection algorithms
- render-to-texture or previous-frame feedback
- multiple passes
- MilkDrop parsing or evaluation
- a permanent `tokimu-visualizer` capability

## Maturity

`native`: synthetic observation generation and one-pass shader response.

`compatible`: not assigned yet; compatibility requires an external visualizer
definition to preserve its documented semantics through Tokimu's contracts.

`partial`: the provider-neutral shader declaration describes a `Vector4`
visualizer signal, but renderer execution currently carries that signal through
the legacy four-float material slot.

`unsupported`: real audio, feedback, multipass execution, MilkDrop semantics,
and preset transitions.

`invalid`: malformed, non-finite, empty, over-frequency, and over-limit frame
inputs are rejected before renderer submission with stable diagnostics.
