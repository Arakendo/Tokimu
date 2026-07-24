# Screenshot Artifact Helper

`screenshot` is an incubating corpus-support library for deterministic review
artifacts.

## Owns

- validation of CPU-owned RGBA8 buffers;
- deterministic BMP encoding;
- simple text metadata manifests.

## Does Not Own

- GPU framebuffer readback;
- renderer surfaces or backend handles;
- screenshot comparison policy;
- semantic UI or text layout.

The helper exists so multiple corpus examples can produce comparable evidence
without each example copying an image encoder. GPU capture remains a separate
renderer capability and should not be implied by this crate.

## Manual Native-Window Evidence

Native-window captures are valid diagnostic evidence, but they are not
authoritative CPU artifacts. They may include window chrome, compositor
scaling, backend-specific rasterization, display scaling, or cursor state.

Keep them beside the example-specific evidence using this convention:

```text
examples/lib-example/screenshot/manual/<example-id>/
    README.md
    <case-id>.png
    <case-id>.manual.manifest
```

The manifest should identify at least:

```text
kind=manual-native-window
example=hello-ui-text-vectors
case=U+004D
backend=vulkan
platform=windows
window_dimensions=900x630
display_scale=1.0
cursor_included=false
authoritative=false
```

Manual captures may support visual review or AI-assisted inspection, but they
must not replace structural artifacts, CPU image fingerprints, or reviewed
goldens. The helper does not capture the window; it only provides the common
metadata vocabulary and deterministic CPU artifact path.
