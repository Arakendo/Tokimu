# Hello Alpha Policy

This headless corpus freezes the first-party RGBA8 fixtures, scene identities,
candidate fragment semantics, caller draw ordering, and typed failure evidence
for AR-0023. It does not render and does not define a public Tokimu API.

From the repository root:

```powershell
cargo test -p hello-alpha-policy
cargo run -p hello-alpha-policy
cargo run -p hello-alpha-policy --features native-visual --bin native_scene
cargo run -p hello-alpha-policy --features native-visual --bin native_scene -- --threshold=0
cargo run -p hello-alpha-policy --features native-visual --bin native_scene -- --threshold=1
cargo run -p hello-alpha-policy --features native-visual --bin native_blend_scene
cargo run -p hello-alpha-policy --features native-visual --bin native_interaction_scene
```

The executable prints the schema-v1 deterministic JSON report. Fixture and
scene hashes are also retained in `fixture-manifest.md`; tests ensure reversed
blend cases preserve transforms and change only submission order.

The optional `native_scene` is the Slice 2 visual comparison. Its top row is
the same mixed-alpha fixture under opaque, discard-below, and
discard-at-or-below candidates; the lower panel places the binary cutout over
an opaque blue quad to expose categorical depth behavior. It exercises the
admitted ADR-0013 Cutout capability; visual observations remain target evidence
rather than a claim of pixel-identical conformance.

`native_scene` accepts only the frozen corpus thresholds `0`, `interior`, and
`1`; the selector changes no policy beyond that shared test input.

`native_blend_scene` is the Slice 3 comparison. Its panels are, clockwise from
top left: caller order far-then-near, caller order near-then-far, explicit
blend depth writes, and explicit blend depth writes disabled. The source
texture and blend equation remain constant. The red and green quads are tinted
instances of the same shared `mixed-alpha` RGBA8 fixture. A separate upper-left
control overlays the first-party `continuous-gradient` fixture on opaque blue,
exercising each alpha byte `0..=255` under the same straight-alpha path.

`native_interaction_scene` begins Slice 4. Its upper panels are cutout over
opaque and Blend over opaque. Its lower panel places a fixed-depth binary
cutout plane over a sloped blended plane that crosses the cutout depth, all
over the same opaque backing. It is a bounded interaction fixture, not a public
material or renderer scheduling contract.

Retained manual observations:

- [`native-cutout-observation-2026-08-09.md`](results/native-cutout-observation-2026-08-09.md)
- [`native-blend-observation-2026-08-09.md`](results/native-blend-observation-2026-08-09.md)

The Blend executable prints both first- and warm-frame resource observations to
the terminal. Those counters describe current provider work only; they do not
promise batching, render-order scheduling, or a public shader-resource model.
