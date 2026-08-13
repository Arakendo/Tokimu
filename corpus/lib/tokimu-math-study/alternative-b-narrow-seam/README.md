# Narrow Option B Camera/Projection Seam

This independently compiled corpus crate implements only AR-0029's three
checked semantic-construction families. It is not a production crate or stable
Tokimu API.

The default feature uses the audited production `glam` 0.29.3 gitlink. The
`provider-033` feature uses the separately reviewed 0.33.3 candidate worktree:

```powershell
cargo test --manifest-path corpus/lib/tokimu-math-study/alternative-b-narrow-seam/Cargo.toml
cargo test --manifest-path corpus/lib/tokimu-math-study/alternative-b-narrow-seam/Cargo.toml --no-default-features --features provider-033
```

Both configurations compile the same public caller and contract tests. The
provider-specific source is limited to three private construction calls. The
dependency on the candidate worktree is deliberate study machinery and makes
this crate non-portable outside the retained repository experiment.
