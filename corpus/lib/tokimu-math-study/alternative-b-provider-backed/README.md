# Alternative B: Provider-Backed Vocabulary

This area tests corpus-local Tokimu-owned public shapes whose mechanics may
delegate to the pinned `glam` provider. Foreign types and traits must not cross
the candidate's public boundary.

The initial probes are `Vec2`, `Vec3`, `Vec4`, `Quat`, and `Mat4` in
`src/alternative_b.rs`. Each keeps its `glam` value private and permits
provider conversion only within the study crate. `Vec3`, `Vec4`, and `Mat4`
cover actual caller pressure; `Vec2` and `Quat` deliberately remain minimal
because the source scan found no direct callers. This is not yet a complete
Alternative B implementation or a migration recommendation.

## Independent Compilation Boundary

`Cargo.toml` and `src/lib.rs` make the shared B source independently
compilable. The crate has one direct dependency: the pinned local `glam`
provider. Its provider visibility remains private in the exported candidate
API, but the dependency is intentionally visible in this target's closure.

```powershell
cargo test --manifest-path corpus/lib/tokimu-math-study/alternative-b-provider-backed/Cargo.toml --offline
```

This target exists to compare the real B and C compilation boundaries. It does
not make B a stable API or imply that private provider mechanics are free.

The same crate also exposes a corpus-only plain-WASM checksum probe for
isolated Node WebAssembly-engine execution. It is not a browser/API contract.
