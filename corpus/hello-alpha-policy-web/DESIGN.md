# Browser Alpha-Policy Comparative Evidence

| Field | Value |
| --- | --- |
| Status | ADR-0013 Cutout migration compiles for browser/WASM; renewed browser observation remains pending. Slice 3 browser/WASM Blend evidence is retained, but Blend remains incubating. |
| Purpose | Realize the shared alpha-policy scene matrix through browser WebGPU after headless semantics are frozen |
| Governing review | [AR-0023](../../docs/Architectural%20Reviews/AR-0023-textured-surface-alpha-and-depth-policy.md) |
| Reads | Shared first-party RGBA8 fixtures and immutable scene/profile observations from `hello-alpha-policy` |
| Emits | Readiness, adapter/device, first-presentation, policy, depth, order, and visual-observation metadata |
| Durable state | None beyond explicitly downloaded observation artifacts |
| Semantic authority | None; target code realizes shared corpus requests without redefining them |
| Execution authority | Browser canvas, asynchronous WebGPU acquisition, and explicit corpus controls only |

## Boundary Assertion

The browser consumer will depend on the shared headless corpus crate once a
GPU candidate seam is authorized. It may select a frozen case and display its
status. It may not inspect fixture alpha to select policy, reorder blended
draws, invent a threshold, or convert browser support into renderer semantics.

```text
shared immutable case
        |
        v
browser/WASM candidate realization
        |
        v
ready -> submitted -> presented observation
```

Compilation, module execution, adapter acquisition, device acquisition,
surface configuration, submission, and first presentation remain separate
states. The consumer will follow
[`webgpu-wasm-quick-reference.md`](../../docs/lessions/webgpu-wasm-quick-reference.md)
and retain browser/adapter/build/viewport metadata beside manual images.

The executable is now a Cargo member and imports the exact RGBA8 fixtures and
visual layout from `hello-alpha-policy`. Its Cutout profiles invoke the
ADR-0013 renderer declaration; its Blend profiles retain corpus-local WGSL
study machinery. JavaScript performs only bounded adapter/device preflight,
module loading, presentation scheduling, and failure reporting.

## Local Build

From the repository root:

```powershell
cargo build -p hello-alpha-policy-web --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/hello-alpha-policy-web.wasm --target web --out-dir corpus/hello-alpha-policy-web/web/pkg --out-name hello-alpha-policy-web
python -m http.server 4178 --directory corpus/hello-alpha-policy-web/web
```

Open `http://127.0.0.1:4178/`. Readiness is reported only after the first
`present()` succeeds. Generated `web/pkg` output remains local evidence and is
not the source of semantic truth.

The optional `?threshold=0`, `?threshold=interior`, and `?threshold=1` query
values select only the frozen corpus threshold for the cutout comparison.
`?mode=blend` selects the separate frozen Slice 3 comparison with continuous
gradient control, fixed caller orders, and explicit depth-write states.
`?mode=interaction` selects the Slice 4 fixed three-panel scene: binary cutout
over opaque backing, mixed-alpha Blend over opaque backing, and a binary
cutout plane crossing a depth-writing sloped mixed-alpha Blend plane over
opaque backing. Its seven submissions and all source fixture, UV, camera,
transform, and depth values are shared with the native fixture. Neither query
is a browser-facing renderer setting.

The Blend path presents the same frozen command array twice. The first and warm
frame counters retain current provider-observable material resolution, pipeline
selection, binding-allocation, and mesh-upload behavior. This is corpus
instrumentation, not a public render-order, shader-resource, batching, or WGPU
bind-group contract.
