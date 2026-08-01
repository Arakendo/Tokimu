# Hello Streaming Texture Manual Evidence

This directory holds optional native-window captures for the
`hello-streaming-texture` corpus. They complement, but do not replace, the
deterministic CPU source-frame artifacts under
`target/hello-streaming-texture/`.

## Capture Procedure

1. Run the bounded default proof or the explicit native stress profile.

   ```powershell
   cargo run -p hello-streaming-texture
   cargo run -p hello-streaming-texture -- --stress-1080p
   ```

2. Wait for the window title or terminal output to report validation after
   frame 300.
3. Capture the native window without a cursor if practical.
4. Store the image and a companion manifest in this directory.

Suggested names:

```text
default-validated.png
default-validated.manual.manifest
stress-1080p-validated.png
stress-1080p-validated.manual.manifest
```

Each manifest must include:

```text
kind=manual-native-window
example=hello-streaming-texture
case=stress-1080p-validated
profile=stress-1080p
platform=windows
backend=<observed backend>
window_dimensions=<observed dimensions>
cursor_included=false
authoritative=false
gpu_framebuffer_capture=false
```

These images show what a particular native window displayed. They do not prove
GPU framebuffer contents, browser/WASM behavior, color-management equivalence,
or steady-state lifecycle counters. The source artifacts and contract tests
remain authoritative for those narrower claims.
