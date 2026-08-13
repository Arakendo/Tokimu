# Hello Render Resource Identity

This corpus reproduces the AR-0024/AR-0027 mutable-offset mesh-handle failure
without Doom input, a native window, WGPU execution, or source geometry.

It models the current replace-on-upload behavior using Tokimu's existing typed
`MeshHandle`. Uploading an existing handle is valid and required for deliberate
mesh replacement, but the handle alone cannot distinguish that operation from
an unrelated logical resource accidentally reusing the same value.

Run the deterministic baseline and native timing observation:

```powershell
cargo run -p hello-render-resource-identity --bin hello-render-resource-identity
```

This is corpus evidence only. It does not admit an allocator, registry,
generational handle, lifecycle operation, recovery policy, or renderer API.
Elapsed churn timings are environment-local observations, not portable budgets.
