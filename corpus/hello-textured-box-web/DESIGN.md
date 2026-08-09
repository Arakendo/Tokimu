# Browser textured-box evidence

This consumer independently exercises the same pinned `Box.glb` geometry and
corpus-owned grid PNG as `hello-textured-box`. It supplies its own deterministic
planar UVs, repeats them beyond the unit range, and renders one textured 3D
draw through the browser/WASM `WgpuBackend`.

It deliberately does not import GLB material or UV semantics. A successful
browser frame establishes that the explicitly scoped geometry, texture decode,
texture upload, UV, sampler, and WebGPU paths compose in this browser consumer;
it does not establish browser/native pixels equivalence or GLB material support.

The fixture accepts browser `M`, `R`, and `X` keys after it presents the first
frame, and exposes matching buttons which dispatch those narrow corpus inputs.
They cycle the same source texture, sampler vocabulary, and corpus UV transform
modes as the native consumer, while retaining the selected state in the status
line. The event listener is owned by this browser corpus consumer; no
browser/input mechanism is admitted to `tokimu-render`.
