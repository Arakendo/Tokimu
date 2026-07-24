# Manual Screenshot Evidence

This directory is reserved for native-window screenshots captured outside the
deterministic CPU artifact path.

Manual captures are complementary evidence. They are useful for checking what
a person sees in a real backend window, but they are not equivalent to:

- a normalized CPU RGBA8 source-buffer artifact;
- a GPU framebuffer readback;
- a structural vector or mesh artifact.

## Required Layout

Each example or investigation should use its own directory:

```text
manual/<example-id>/
    README.md
    <case-id>.png
    <case-id>.manual.manifest
```

The manifest should record the example, case, backend, platform, window size,
display scale, cursor policy, and `authoritative=false`.

Use manual captures to explain or inspect a discrepancy. Use structural and
deterministic CPU artifacts to decide whether a geometry or raster regression
exists.
