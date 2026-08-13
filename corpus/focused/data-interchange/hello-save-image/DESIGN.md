# Hello Save Image

`hello-save-image` is an example-level image export proof. It renders a small
deterministic scene and writes the same scene to
`target/hello-save-image/scene.bmp`.
The example also writes `scene.txt` beside the image with the dimensions,
buffer format, and explicit `gpu_readback=false` status.

The current renderer does not expose surface readback, so this first slice uses
a CPU scene buffer and a small BMP encoder. It deliberately does not claim to
capture GPU pixels. A future renderer-owned readback contract can add PNG or
JPEG export without changing the example's scene description.
