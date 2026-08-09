# Browser/WASM orientation capture

- Date: 2026-08-08
- Consumer: `hello-render-orientation-web`
- Runtime: Microsoft Edge on Windows, served from `http://127.0.0.1:4174/`
- State: `ready`
- Adapter identity: unavailable from the WGPU 23 browser adapter-info surface;
  the retained status therefore leaves the adapter name blank rather than
  inventing an identity.
- Capture: `browser-wasm.png`

The browser completed a direct `navigator.gpu` adapter/device preflight and
then initialized Tokimu's asynchronous browser WGPU provider. The captured
canvas uses the same Rust-owned fixture geometry, shader, cull pipelines,
transforms, compensation rules, and layout as the native WGPU capture.

The result agrees with `native-wgpu.png` in every cell:

- no culling shows both the green front-facing and magenta back-facing
  triangles;
- back-face culling retains only the green triangle;
- front-face culling retains only the magenta triangle;
- rotation and translation preserve those classifications;
- an uncompensated X reflection reverses them;
- the once-compensated reflection restores the declared result.

The retained browser window includes developer tools showing
`data-orientation-state="ready"`. The adapter/device preflight is corpus-host
diagnostic evidence and is not part of the renderer contract.

## Blocker discovered during capture

The initial browser runs reached surface configuration and first presentation,
then aborted in `std::time::Instant::now()` before calling
`Surface::get_current_texture()`. Tokimu's WGPU CPU timing instrumentation had
assumed the native clock was available on WASM. The renderer now uses
`Performance.now()` when the browser supplies that clock and otherwise omits
the affected optional measurement. Native builds continue to use
`std::time::Instant`.

Temporary browser stage probes used to isolate the abort were removed after
the cause was proven. This finding did not require making renderer operations
asynchronous; asynchronous browser adapter/device acquisition remains in the
provider construction path.
