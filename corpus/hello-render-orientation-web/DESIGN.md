# Hello Render Orientation Web

## Purpose

This browser/WASM consumer renders the exact shared AR-0021 fixture used by
`hello-render-orientation`. Rust/WASM owns fixture selection, mesh upload,
pipeline construction, transforms, culling, and submission. JavaScript only
loads the generated module and reports bootstrap failures.

The canvas uses the same four transform rows and three cull columns documented
by the native consumer. A browser capture must retain WebGPU adapter identity
and readiness alongside the canvas.

The host checks for `navigator.gpu`, then makes bounded browser-side adapter
and device requests before loading the Rust/WASM renderer. It requires the Rust
renderer to report readiness within ten seconds after that preflight. This
distinguishes a browser that exposes WebGPU but cannot supply an adapter or
device from a Tokimu/WGPU initialization stall. Either unsupported result is
environment or adapter-path evidence, not a failed orientation comparison and
not permission to substitute Canvas 2D.

After WASM loads, the host starts the one-shot renderer on the next browser
animation frame. This preserves the browser's presentation scheduling boundary:
the renderer owns WebGPU acquisition and rendering, while the browser host
chooses when its canvas is first eligible to acquire a surface texture.

The first retained browser run also exposed a renderer-diagnostic portability
defect: `std::time::Instant::now()` aborts on this WASM target. WGPU CPU timing
now uses the browser performance clock when available and otherwise omits the
optional measurement. The next-animation-frame scheduling remains a deliberate
host boundary, but it was not the correction for that abort.

The browser adapter/device preflight is evidence instrumentation for this
consumer. It intentionally distinguishes browser capability failure from
Tokimu provider failure, but it is not a second renderer provider and does not
define ordinary application bootstrap policy.

`web/camera.html` is the AR-0028 Slice 3 browser consumer. It uses the same
corpus-local camera pose, basis, commands, pointer observation, and
first-person mapping policy as the native fixture. Browser pointer lock remains
an acquisition mechanism and does not become camera meaning.

The retained
`corpus/lib/render-orientation-conformance/results/browser-wasm.png` capture
reaches `ready` and agrees with the native fixture in all 12 cells. WGPU 23 does
not expose a useful adapter name through this browser path, so the capture
leaves the name blank and the manifest records that limitation explicitly.
