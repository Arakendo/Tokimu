# Doom Visibility Synthetic Browser Evidence

Browser/WASM companion for the Doom-local synthetic visibility controls. The
web page supplies only fixture selection and bounded status presentation; Rust
owns source maps, lowering, pipelines, and diagnostics.

Build the browser package using the repository's normal WASM packaging flow,
then serve `web/`. Run all six controls and retain browser metadata alongside
the native observations. `Shared plane key` is the source-plane identity
control: green sector 0 and orange sector 1 share a floor key but must survive
as distinct provider-lowered floor instances. Their claim is semantic parity,
not pixel identity. `Dynamic door snapshots` compares a production-lowered
closed height band with an explicit open snapshot whose same source boundary
has no lowered band; it does not simulate a Doom door controller.
`Projection epsilon` magnifies the retained classifications for a
behind-viewer fail-open case, a thin valid SEG, and an extremely-close valid
SEG. Only the two valid source walls are rendered.
`Platform snapshots` lowers the same immutable source fixture through declared
floor heights `0` and `48`; the green and yellow walls make the resulting
source-local height difference visible without simulating a platform
controller.
`Cutout non-occluder` presents a caller-declared checkerboard cutout over an
opaque far wall. Its transparent texels must expose that wall; this is a
Doom-local negative authority control, not a generic occlusion contract.
