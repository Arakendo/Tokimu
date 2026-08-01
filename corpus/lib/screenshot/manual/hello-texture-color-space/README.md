# Hello Texture Color Space Manual Evidence

`hello-texture-color-space` draws identical encoded sRGB RGBA8 ramp bytes twice:

- the left sample allocates them as `Rgba8TextureColorSpace::Linear`;
- the right sample allocates them as `Rgba8TextureColorSpace::Srgb`.

The corpus verifies the descriptor difference structurally. This directory
defines the separately labeled manual observation path for the native renderer.
It does not establish a browser, monitor, ICC, HDR, or final-output color
policy.

## Capture Procedure

Run the corpus:

```powershell
cargo run -p hello-texture-color-space
```

Capture the complete native window after both samples are visible. Store the
image and adjacent manifest under this directory using:

```text
linear-vs-srgb.native.png
linear-vs-srgb.native.manifest
```

The manifest must contain:

```text
kind=manual-native-window
example=hello-texture-color-space
case=encoded-srgb-ramp-linear-vs-srgb
platform=windows
backend=<observed backend>
window_dimensions=<observed dimensions>
display_color_mode=<observed mode or unknown>
cursor_included=false
authoritative=false
gpu_framebuffer_capture=false
```

## Interpretation

The expected observation is that both samples use the same byte payload while
the renderer selects different texture formats. A visible difference can help
validate that the native adapter is applying the requested interpretation.
Absence of a visible difference does not by itself invalidate the semantic
descriptor: output conversion, monitor configuration, and presentation policy
remain outside this bounded corpus.

This evidence is complementary to, not a replacement for, descriptor tests and
future consumer golden-frame validation.
