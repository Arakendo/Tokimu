# Hello Audio Visualizer

## Purpose

`hello-audio-visualizer` is the first executable corpus proof for deterministic
visualizer input and bounded fullscreen WGSL passes. It deliberately uses
synthetic observations rather than an audio device or decoded file. It also
executes one Tokimu-authored MilkDrop 1-style fixture through a deliberately
selected compatibility subset.

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

- `Left` / `Right`: select a synthetic input fixture (the default is the
  visibly changing `frequency-sweep` fixture)
- `Space`: pause or resume visualizer time
- `Up` / `Down`: change visualizer time scale
- `R`: reset visualizer time
- `Q`: execute the two-pass `Signal Field` proof
- `W`: execute the previous-frame `Feedback Bloom` proof
- `E`: execute the three-pass `Signal Composite` proof
- `X`: execute the provider-neutral spectrum-bar proof
- `M`: execute the selected MilkDrop classic feedback subset

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

This writes one input JSON file, deterministic waveform, spectrum-bar, and
radial-shape geometry records, CPU signal preview BMP, and manifest per fixture
under `target/audio-visualizer/`. The radial shape is an original Tokimu
presentation proof; it does not claim MilkDrop custom-shape compatibility. The
command also writes the WGSL source and a provider-neutral shader/material
binding contract record, plus separately labeled fixed-workload native-host
timing and source-structural working-set observations for every PCM fixture.
It additionally writes three native visualizer definition records:
`native-visualizer-signal-field.json`,
`native-visualizer-feedback-bloom.json`, and
`native-visualizer-signal-composite.json`. Those records prove stable
identity, bounded parameters, and structural pass requirements. The native
window additionally executes `Signal Field` as an explicit two-pass proof,
`Feedback Bloom` as an explicit previous-frame proof, and `Signal Composite`
as an explicit signal -> composite target -> surface proof. This is not a
general definition executor.
The timing observation is evidence for the current machine only, not a portable
performance budget. The working-set observation counts only retained and
temporary `f32` slots implied by the reference analysis; it is not an allocator
call count and excludes `Vec` capacity, allocator overhead, caller-owned input,
and platform audio state. The preview is explicitly not a GPU framebuffer capture
or an assertion of backend pixel equivalence. Native-window screenshots remain
manual evidence.

The command also writes the Tokimu-authored `.milk` source, parsed document,
resolved scalar parameters, evaluated frame state, original compatibility
WGSL, and shader-contract record. The admitted execution subset covers classic
scalar decay/zoom controls plus bounded initialization and per-frame scalar
equations. It also renders selected literal custom waves and untextured convex
custom shapes through consumer-owned geometry. Per-pixel equations, texture
lookup, embedded shaders, and third-party preset compatibility remain deferred.

The selected first-party preset also proves one literal custom-wave path. When
the caller supplies explicit waveform or spectrum samples, `milkdrop-tools`
returns normalized point records without producing a mesh. The browser renders
those Rust/WASM-produced points through Canvas, while this native consumer
converts them into bounded line or dot triangle geometry over the feedback
target. That conversion is intentionally consumer-owned. Its changing mesh
uploads remain visible in runtime performance diagnostics. The selected
`bAdditive=1` wave selects the renderer-neutral `BlendMode::Additive` policy;
the provider preserves intent but never owns backend blend execution. Texture
wrapping and filtering remain deferred because they require the unresolved
raster texture-requirement contract.

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

The structural fixture validates bounded two-pass, three-pass, and
previous-frame feedback graph descriptions. The native WGPU route additionally
executes one deliberately narrow feedback path: two same-sized history targets
alternate as distinct `previous` and `current` members. Each frame samples only
the previous target while writing the other, presents the completed current
target to the surface, and then swaps their roles. Explicit reset and resize
clear both targets. This is renderer-local evidence, not a provider-neutral
multipass execution contract. Stable target identities retain material and
instance bindings across ordinary frames; only the active signal uniform is
updated. Target replacement rebuilds target-sampling material bindings
explicitly at the resize boundary.

After 120 visualizer frames, the native app prints a warm feedback observation
covering frame-local binding, pipeline, mesh, and render-target
allocation/replacement counters, plus the current renderer-local offscreen
target footprint. The footprint counts known RGBA8 color and Depth32 images
only; it excludes driver overhead, surface images, views, samplers, caches,
and GPU residency. This is a CPU-side lifecycle diagnostic, not a GPU
completion metric or an automatic performance policy.

The dynamic material uniform must be created with both `UNIFORM` and
`COPY_DST` usage in the WGPU adapter. The corpus found this requirement when
the scan marker remained at phase zero despite successful CPU-side write
observations; a focused backend test now protects the runtime-update contract.

Each native mode also contains a small phase-driven scan marker. This is
intentional visual evidence: it must move even when the selected audio fixture
has static bands. A stationary marker localizes failure to the frame-time or
material-uniform path rather than to audio analysis.

## Non-Goals

- real audio capture or playback
- FFT or beat-detection algorithms
- a general previous-frame feedback execution API
- a provider-neutral multipass backend execution contract
- general MilkDrop preset compatibility
- MilkDrop per-pixel equations, texture lookup, or embedded shader translation
- a permanent `tokimu-visualizer` capability

## Maturity

`native`: synthetic observation generation, one-pass shader response, and a
bounded renderer-local previous-frame feedback proof.

`compatible`: selected Tokimu-authored MilkDrop 1-style scalar parameters and
initialization/per-frame equations reach a renderer-owned feedback pass. This
is a bounded subset, not compatibility with arbitrary third-party presets.

`partial`: the provider-neutral shader declaration describes a `Vector4`
visualizer signal, but renderer execution currently carries that signal through
the legacy four-float material slot.

`unsupported`: real audio, provider-neutral feedback/multipass execution,
MilkDrop per-pixel/texture/shader semantics, and preset transitions. Selected
literal waves, untextured convex shapes, and additive overlay intent are
bounded compatible evidence only.

`invalid`: malformed, non-finite, empty, over-frequency, and over-limit frame
inputs are rejected before renderer submission with stable diagnostics.
