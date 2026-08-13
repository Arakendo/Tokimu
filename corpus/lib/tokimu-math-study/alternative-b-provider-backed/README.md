# Alternative B: Provider-Backed Vocabulary

This area tests corpus-local Tokimu-owned public shapes whose mechanics may
delegate to the pinned `glam` provider. Foreign types and traits must not cross
the candidate's public boundary.

The probes are `Vec2`, `Vec3`, `Vec4`, `Quat`, and `Mat4` in
`src/alternative_b.rs`. Each keeps its `glam` value private and permits
provider conversion only within the study crate. `Vec3`, `Vec4`, and `Mat4`
cover actual caller pressure; `Vec2` and `Quat` deliberately remain minimal
because the source scan found no direct callers. This remains an isolated
candidate rather than a migration recommendation.

## Independent Compilation Boundary

`Cargo.toml` and `src/lib.rs` make the same shared B source independently
compilable against either the retained 0.29.3 provider or the isolated 0.33.3
update candidate. Provider visibility remains private in the exported
candidate API, but the selected dependency is intentionally visible in this
target's closure.

```powershell
cargo test --manifest-path corpus/lib/tokimu-math-study/alternative-b-provider-backed/Cargo.toml --offline
cargo test --manifest-path corpus/lib/tokimu-math-study/alternative-b-provider-backed/Cargo.toml --locked --offline --no-default-features --features provider-033
```

This target exists to compare the real B and C compilation boundaries. It does
not make B a stable API or imply that private provider mechanics are free.

The same crate also exposes a corpus-only plain-WASM checksum probe for
isolated Node WebAssembly-engine execution. It is not a browser/API contract.
