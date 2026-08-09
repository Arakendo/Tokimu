# Hello Alpha Policy

This headless corpus freezes the first-party RGBA8 fixtures, scene identities,
candidate fragment semantics, caller draw ordering, and typed failure evidence
for AR-0023. It does not render and does not define a public Tokimu API.

From the repository root:

```powershell
cargo test -p hello-alpha-policy
cargo run -p hello-alpha-policy
cargo run -p hello-alpha-policy --features native-visual --bin native_scene
```

The executable prints the schema-v1 deterministic JSON report. Fixture and
scene hashes are also retained in `fixture-manifest.md`; tests ensure reversed
blend cases preserve transforms and change only submission order.

The optional `native_scene` is the Slice 2 visual comparison. Its top row is
the same mixed-alpha fixture under opaque, discard-below, and
discard-at-or-below candidates; the lower panel places the binary cutout over
an opaque blue quad to expose categorical depth behavior. It is deliberately
not evidence of a public renderer contract.
