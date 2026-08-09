# Browser Alpha-Policy Comparative Evidence

| Field | Value |
| --- | --- |
| Status | Slice 0 boundary record only; executable consumer not started |
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

No Cargo member, generated binding, or renderer contract is introduced by this
Slice 0 record.
