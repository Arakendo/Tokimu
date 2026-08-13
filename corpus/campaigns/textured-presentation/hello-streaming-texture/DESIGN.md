# Hello Streaming Texture

## Purpose

`hello-streaming-texture` proves that application-owned RGBA8 frames can update
one renderer texture without replacing its identity or rebuilding dependent
material bindings.

The application generates deterministic pixels. `tokimu-render` owns the GPU
texture allocation and whole-resource writes. The renderer never learns what
the moving pattern means.

## Primary Proof

```text
application RGBA8 frame
        |
        v
stable TextureHandle
        |
        v
in-place texture write
        |
        v
existing material binding
        |
        v
visible sampled image
```

At frame 300 the application checks that steady-state updates produced exactly
one lifetime texture allocation, no replacements, one texture write per frame
plus the initial creation write, and no per-frame texture allocation.

At that checkpoint it also writes deterministic CPU source-frame artifacts for
frame zero and the validated frame under `target/hello-streaming-texture/`.
Their manifests explicitly say they are not GPU framebuffer captures. They
provide reviewable evidence that the application produced changing payloads;
the native window remains the manual proof that the already-bound material
samples those writes.

## Profiles

The default profile uses a 320 by 180 texture so the lifecycle proof stays
quick to launch. A separate 1920 by 1080 native stress profile is selected
explicitly:

```powershell
cargo run -p hello-streaming-texture -- --stress-1080p
```

For a bounded native validation run that exits after frame 300, add
`--exit-after-validation`:

```powershell
cargo run -p hello-streaming-texture -- --stress-1080p --exit-after-validation
```

The stress profile is not an automatic performance claim. Its native-window
observation and any saved screenshot remain manual evidence; the deterministic
source-frame artifacts still describe CPU-generated input, not framebuffer
output.

The optional native-window capture procedure and required manual-artifact
metadata live in
`corpus/lib/screenshot/manual/hello-streaming-texture/README.md`.

## Non-Goals

- partial rectangle updates;
- texture resizing;
- encoded image decoding;
- DMX, Spout, or OBS integration;
- GPU completion timing;
- a generalized image requirement model.
