# Browser WebGPU And WASM Initialization

This is the known-working Tokimu startup pattern established by the AR-0021
browser orientation fixture. It describes provider construction, not a general
requirement that every renderer operation become asynchronous.

## Working Sequence

```text
browser host
    -> verify navigator.gpu when useful for diagnostics
    -> load and await wasm-bindgen initialization
    -> yield one requestAnimationFrame for the canvas host
    -> call the exported Rust start function

Rust/WASM start function
    -> obtain the HtmlCanvasElement
    -> spawn the async provider-construction future
    -> await WGPU browser backend detection
    -> create a surface for the canvas
    -> request a surface-compatible adapter
    -> request the device and queue
    -> inspect surface capabilities
    -> choose an sRGB format when available
    -> create dependent layouts and depth resources
    -> configure the surface
    -> submit and present the first frame
    -> report ready only after presentation succeeds
```

Tokimu's current implementation is in
[`backend_init.rs`](../../crates/tokimu-render/src/wgpu_backend/backend_init.rs),
with the executable browser proof in
[`hello-render-orientation-web`](../../corpus/campaigns/coordinate-conformance/hello-render-orientation-web/).

## WGPU 23 Instance Construction

On the current WGPU 23 browser path, use and await:

```rust
let instance =
    wgpu::util::new_instance_with_webgpu_detection(
        wgpu::InstanceDescriptor::default(),
    )
    .await;
```

Do not treat synchronous `wgpu::Instance::default()` construction as proof
that browser WebGPU discovery has completed. This helper is version-specific;
recheck WGPU's API when upgrading rather than preserving the spelling by habit.

Request the adapter with the created surface as `compatible_surface`. Choose
the surface format from reported capabilities, preferring an sRGB format, and
clamp surface dimensions to at least one pixel before configuration.

## Browser Bootstrap Shape

The browser should await the generated module initialization before calling
Rust. Starting the one-shot fixture on the next animation frame proved a clean
canvas-host scheduling boundary:

```javascript
const { default: init, start_fixture: startFixture } =
  await import("./pkg/hello-render-orientation-web.js");

await init();
await new Promise((resolve) => requestAnimationFrame(resolve));
startFixture();
```

Rust can then retain the asynchronous mechanism locally:

```rust
#[wasm_bindgen]
pub fn start_fixture() -> Result<(), JsValue> {
    let canvas = /* obtain HtmlCanvasElement */;
    spawn_local(async move {
        // Await provider construction, render, and report the result.
    });
    Ok(())
}
```

This does not require ordinary render submission or presentation APIs to
become async. Browser adapter/device acquisition belongs in platform/provider
construction; rendering semantics begin after a ready provider exists.

## Readiness And Failure Evidence

Keep these claims separate:

```text
WASM compiled
    != generated module loaded
    != navigator.gpu available
    != adapter acquired
    != device acquired
    != Tokimu provider initialized
    != surface configured
    != commands submitted
    != first frame presented
```

Expose bounded `loading`, `ready`, `failed`, and `unsupported` states. Report
`ready` only after the first `present()` succeeds. A timeout must name the last
completed stage; it is not evidence that geometry, culling, or WebGPU support
failed.

A direct `navigator.gpu` adapter/device preflight can distinguish browser
capability from Tokimu provider failure in a corpus host. It is diagnostic
evidence, not a second provider and not required application bootstrap policy.

Browser adapter information may omit a useful adapter name. Preserve the empty
or unavailable value honestly rather than inventing an identity.

## Portable CPU Timing

Do not call `std::time::Instant::now()` on this WASM browser path. During the
AR-0021 capture it aborted immediately before surface-frame acquisition, which
initially looked like a WebGPU initialization timeout.

Tokimu's provider-local timer uses:

- `std::time::Instant` on native targets;
- `window.performance().now()` on browser WASM;
- no measurement when the provider cannot supply a monotonic clock.

Timing is optional diagnostic evidence. Absence of a clock must not prevent a
frame from rendering. See
[`cpu_timer.rs`](../../crates/tokimu-render/src/wgpu_backend/cpu_timer.rs).

## Fast Debugging Order

When browser rendering stalls, retain the last completed stage and narrow in
this order:

1. generated JavaScript and WASM load;
2. exported Rust start function executes;
3. browser adapter and device preflight completes;
4. WGPU instance and canvas surface exist;
5. surface-compatible adapter and device exist;
6. capabilities are non-empty and the surface is configured;
7. resources and pipelines are created without validation diagnostics;
8. commands submit;
9. surface texture acquisition and presentation complete;
10. readiness changes only after the first frame.

Also install browser `error` and `unhandledrejection` reporting. A Rust panic
otherwise easily degrades into a misleading timeout.

## Evidence

- [`AR-0021`](../Architectural%20Reviews/AR-0021-geometry-orientation-and-facing-conformance.md)
- [Browser fixture design](../../corpus/campaigns/coordinate-conformance/hello-render-orientation-web/DESIGN.md)
- [Browser/WASM capture manifest](../../corpus/lib/render-orientation-conformance/results/browser-wasm.md)

