# Slice 9 Browser WGPU Bulk Control

This is a corpus-local browser control for Option C Slice 9. It runs the
fixed-seed ordered-point workload, compares every browser-WGPU readback flag
with the Slice 8 CPU reference, and reports bounded browser/provider metadata.
It does not expose a Tokimu compute API or make GPU output authoritative.

From the workspace root:

```powershell
node corpus/campaigns/native-math/hello-bulk-compute-web/web/serve.mjs
```

Open `http://127.0.0.1:4186`, then select **Run 100K browser WGPU**. The control
creates its provider/resources once, then runs three upload/dispatch/readback
samples and reports medians alongside cold adapter/device/setup-allocation timing. A success
record uses `status=completed`; it is a compute completion, not a presented
render frame. **Run 100K CPU fallback** deliberately bypasses WGPU and retains
one caller-owned terminal observation for the same bounded workload.
**Run invalid-input control** demonstrates the shared bounded count validation
without attempting provider acquisition. It is distinct from provider failure.
**Run idle-disposal control** creates then explicitly destroys an idle provider
buffer. This is a bounded resource-release observation, not a device-loss or
in-flight cancellation simulation.
**Run scoped invalid-shader control** intentionally asks the browser provider to
reject malformed WGSL inside a validation error scope; its expected outcome is
the bounded `status=provider-validation-rejected`, not a panic or fallback.

The browser control is evidence only after an actual browser run. A successful
WASM build or a visible page before clicking the WGPU control is not evidence
of browser provider execution.

**Run AR-0026 chart A/B/C control** runs the fixed corpus-local three-chart
transition trace using pinned Alternative A, Full B, and the owned C0 candidate.
It creates no GPU provider and has no renderer meaning; matching fingerprints
only prove the same bounded ordinary-math trace executed in the browser.
