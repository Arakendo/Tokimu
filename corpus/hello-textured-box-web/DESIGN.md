# Browser textured-box evidence

This consumer independently exercises the same pinned `Box.glb` geometry and
corpus-owned PNG set as `hello-textured-box`. It supplies its own deterministic
planar UVs, maps one image to each face by default, and renders one textured 3D
draw through the browser/WASM `WgpuBackend`. An explicit `E` mode scales the
same UVs beyond the unit range for addressing stress.

It deliberately does not import GLB material or UV semantics. A successful
browser frame establishes that the explicitly scoped geometry, texture decode,
texture upload, UV, sampler, and WebGPU paths compose in this browser consumer;
it does not establish browser/native pixels equivalence or GLB material support.

The fixture accepts browser `M`, `R`, `X`, and `E` keys after it presents the first
frame, and exposes matching buttons which dispatch those narrow corpus inputs.
They cycle the same source texture, sampler vocabulary, and corpus UV transform
and extent modes as the native consumer, while retaining the selected state in the status
line. The event listener is owned by this browser corpus consumer; no
browser/input mechanism is admitted to `tokimu-render`.
